use std::collections::{HashMap, HashSet};
use std::ffi::OsStr;
use std::fs;
use std::path::Path;

use serde::Deserialize;

use super::types::{ContentLibrary, Lesson, LocalizedText, QuestionEntry, QuestionType, Theme};

#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct ThemeDocument {
    id: String,
    title: LocalizedText,
    #[serde(default)]
    available: bool,
    #[serde(default)]
    lessons: Vec<LessonDocument>,
}

#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct LessonDocument {
    id: String,
    title: LocalizedText,
    categories: Vec<CategoryDocument>,
}

#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct CategoryDocument {
    #[serde(rename = "type")]
    question_type: QuestionType,
    exercises: Vec<QuestionEntry>,
}

/// Loads and validates the complete declarative content catalog.
#[allow(clippy::too_many_lines)]
pub fn load_content_library(directory: impl AsRef<Path>) -> Result<ContentLibrary, String> {
    let directory = directory.as_ref();
    let entries = fs::read_dir(directory).map_err(|error| {
        format!(
            "Cannot read content directory {}: {error}",
            directory.display()
        )
    })?;
    let mut paths = entries
        .map(|entry| {
            entry.map(|entry| entry.path()).map_err(|error| {
                format!(
                    "Cannot read an entry in content directory {}: {error}",
                    directory.display()
                )
            })
        })
        .collect::<Result<Vec<_>, _>>()?;
    paths.retain(|path| path.extension() == Some(OsStr::new("json")));
    paths.sort_by(|left, right| left.file_name().cmp(&right.file_name()));
    if paths.is_empty() {
        return Err(format!(
            "No theme JSON files found in {}",
            directory.display()
        ));
    }

    let mut documents = Vec::with_capacity(paths.len());
    for path in paths {
        let contents = fs::read_to_string(&path)
            .map_err(|error| format!("Cannot read theme file {}: {error}", path.display()))?;
        let document = serde_json::from_str::<ThemeDocument>(&contents)
            .map_err(|error| format!("Invalid theme file {}: {error}", path.display()))?;
        documents.push((path, document));
    }

    let mut themes = Vec::with_capacity(documents.len());
    let mut theme_sources: HashMap<String, std::path::PathBuf> = HashMap::new();
    let mut lesson_sources: HashMap<String, std::path::PathBuf> = HashMap::new();
    let mut issues = Vec::new();

    for (path, document) in documents {
        if document.available && document.lessons.is_empty() {
            issues.push(format!(
                "{}: available theme {} has no lessons",
                path.display(),
                document.id
            ));
        }
        if document.id.is_empty() {
            issues.push(format!("{}: theme ID must not be empty", path.display()));
        }
        if let Some(previous) = theme_sources.get(&document.id) {
            issues.push(format!(
                "{}: duplicate theme ID {} (first declared in {})",
                path.display(),
                document.id,
                previous.display()
            ));
        } else {
            theme_sources.insert(document.id.clone(), path.clone());
        }

        let mut lessons = Vec::with_capacity(document.lessons.len());
        for lesson_document in document.lessons {
            if lesson_document.id.is_empty() {
                issues.push(format!(
                    "{}: theme {}, lesson ID must not be empty",
                    path.display(),
                    document.id
                ));
            }
            if let Some(previous) = lesson_sources.get(&lesson_document.id) {
                issues.push(format!(
                    "{}: theme {}, duplicate global lesson ID {} (first declared in {})",
                    path.display(),
                    document.id,
                    lesson_document.id,
                    previous.display()
                ));
            } else {
                lesson_sources.insert(lesson_document.id.clone(), path.clone());
            }

            let mut category_types = HashSet::new();
            let mut questions = Vec::new();
            for category in lesson_document.categories {
                if !category_types.insert(category.question_type) {
                    issues.push(format!(
                        "{}: theme {}, lesson {}, duplicate category group {:?}",
                        path.display(),
                        document.id,
                        lesson_document.id,
                        category.question_type
                    ));
                }
                if category.exercises.is_empty() {
                    issues.push(format!(
                        "{}: theme {}, lesson {}, category {:?} has no exercises",
                        path.display(),
                        document.id,
                        lesson_document.id,
                        category.question_type
                    ));
                }
                for exercise in &category.exercises {
                    if exercise.definition.question_type() != category.question_type {
                        issues.push(format!(
                            "{}: theme {}, lesson {}, exercise {}, category {:?} does not match definition type {:?}",
                            path.display(),
                            document.id,
                            lesson_document.id,
                            exercise.id,
                            category.question_type,
                            exercise.definition.question_type()
                        ));
                    }
                }
                questions.extend(
                    category
                        .exercises
                        .into_iter()
                        .map(|exercise| QuestionEntry::new(exercise.id, exercise.definition)),
                );
            }

            let lesson = Lesson {
                id: lesson_document.id,
                title: lesson_document.title,
                questions,
            };
            for issue in lesson.validation_issues() {
                issues.push(format!(
                    "{}: theme {}, lesson {}: {issue}",
                    path.display(),
                    document.id,
                    lesson.id
                ));
            }
            lessons.push(lesson);
        }

        themes.push(Theme {
            id: document.id,
            title: document.title,
            available: document.available,
            lessons,
            lesson_index: HashMap::new(),
        });
    }

    if !issues.is_empty() {
        return Err(issues.join("\n"));
    }

    let mut library = ContentLibrary {
        themes,
        ..Default::default()
    };
    let catalog_issues = library.validation_issues();
    if !catalog_issues.is_empty() {
        return Err(format!(
            "Invalid catalog under {}: {}",
            directory.display(),
            catalog_issues.join("; ")
        ));
    }
    library.build_indexes();
    Ok(library)
}
