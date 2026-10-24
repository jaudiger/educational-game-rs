use std::collections::{HashMap, HashSet};
#[cfg(not(target_arch = "wasm32"))]
use std::ffi::OsStr;
#[cfg(not(target_arch = "wasm32"))]
use std::fs;
#[cfg(not(target_arch = "wasm32"))]
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

/// Loads and validates the complete declarative content catalog from disk.
#[cfg(not(target_arch = "wasm32"))]
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
        documents.push((path.display().to_string(), contents));
    }

    load_content_library_from_documents(documents, directory.display().to_string())
}

/// Loads and validates the compile-time embedded theme catalog.
#[cfg(target_arch = "wasm32")]
pub fn load_embedded_content_library() -> Result<ContentLibrary, String> {
    let documents = embedded_content::THEME_DOCUMENTS
        .iter()
        .map(|(filename, contents)| ((*filename).to_owned(), (*contents).to_owned()))
        .collect::<Vec<_>>();
    if documents.is_empty() {
        return Err("No theme JSON files found in embedded content catalog".to_owned());
    }

    load_content_library_from_documents(documents, "embedded content catalog".to_owned())
}

#[cfg(target_arch = "wasm32")]
mod embedded_content {
    include!(concat!(env!("OUT_DIR"), "/embedded_content.rs"));
}

#[allow(clippy::too_many_lines)]
fn load_content_library_from_documents(
    source_documents: Vec<(String, String)>,
    catalog_source: String,
) -> Result<ContentLibrary, String> {
    let mut documents = Vec::with_capacity(source_documents.len());
    for (source, contents) in source_documents {
        let document = serde_json::from_str::<ThemeDocument>(&contents)
            .map_err(|error| format!("Invalid theme file {source}: {error}"))?;
        documents.push((source, document));
    }

    let mut themes = Vec::with_capacity(documents.len());
    let mut theme_sources: HashMap<String, String> = HashMap::new();
    let mut lesson_sources: HashMap<String, String> = HashMap::new();
    let mut issues = Vec::new();

    for (source, document) in documents {
        if document.available && document.lessons.is_empty() {
            issues.push(format!(
                "{}: available theme {} has no lessons",
                source, document.id
            ));
        }
        if document.id.is_empty() {
            issues.push(format!("{source}: theme ID must not be empty"));
        }
        if let Some(previous) = theme_sources.get(&document.id) {
            issues.push(format!(
                "{}: duplicate theme ID {} (first declared in {})",
                source, document.id, previous
            ));
        } else {
            theme_sources.insert(document.id.clone(), source.clone());
        }

        let mut lessons = Vec::with_capacity(document.lessons.len());
        for lesson_document in document.lessons {
            if lesson_document.id.is_empty() {
                issues.push(format!(
                    "{}: theme {}, lesson ID must not be empty",
                    source, document.id
                ));
            }
            if let Some(previous) = lesson_sources.get(&lesson_document.id) {
                issues.push(format!(
                    "{}: theme {}, duplicate global lesson ID {} (first declared in {})",
                    source, document.id, lesson_document.id, previous
                ));
            } else {
                lesson_sources.insert(lesson_document.id.clone(), source.clone());
            }

            let mut category_types = HashSet::new();
            let mut questions = Vec::new();
            for category in lesson_document.categories {
                if !category_types.insert(category.question_type) {
                    issues.push(format!(
                        "{}: theme {}, lesson {}, duplicate category group {:?}",
                        source, document.id, lesson_document.id, category.question_type
                    ));
                }
                if category.exercises.is_empty() {
                    issues.push(format!(
                        "{}: theme {}, lesson {}, category {:?} has no exercises",
                        source, document.id, lesson_document.id, category.question_type
                    ));
                }
                for exercise in &category.exercises {
                    if exercise.definition.question_type() != category.question_type {
                        issues.push(format!(
                            "{}: theme {}, lesson {}, exercise {}, category {:?} does not match definition type {:?}",
                            source,
                            document.id,
                            lesson_document.id,
                            exercise.id,
                            category.question_type,
                            exercise.definition.question_type()
                        ));
                    }
                }
                questions.extend(category.exercises.into_iter().map(|exercise| {
                    QuestionEntry::new(
                        exercise.id,
                        exercise.catalog_label,
                        exercise.school_level,
                        exercise.definition,
                    )
                }));
            }

            let lesson = Lesson {
                id: lesson_document.id,
                title: lesson_document.title,
                questions,
            };
            for issue in lesson.validation_issues() {
                issues.push(format!(
                    "{}: theme {}, lesson {}: {issue}",
                    source, document.id, lesson.id
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
            catalog_source,
            catalog_issues.join("; ")
        ));
    }
    library.build_indexes();
    Ok(library)
}
