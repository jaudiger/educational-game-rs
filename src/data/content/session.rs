use std::collections::{HashMap, HashSet};

use rand::Rng;
use rand::seq::SliceRandom;

use super::generation::resolve_question;
use super::types::{Lesson, QuestionEntry, QuestionInstance, ResolvedQuestion};
use crate::data::{
    ActiveTheme, ContentLibrary, LessonSession, LessonSessionConfig, SelectedLesson,
};

/// Maximum re-roll attempts when deduplicating generated question instances.
const MAX_DEDUP_RETRIES: u32 = 20;

/// Build a lesson session from the selected catalog lesson and teacher settings.
pub fn build_session(
    selected_lesson: &SelectedLesson,
    content: &ContentLibrary,
    active_theme: &ActiveTheme,
    lesson_config: Option<&LessonSessionConfig>,
) -> Result<LessonSession, Vec<String>> {
    let Some(lesson_id) = selected_lesson.as_ref() else {
        return Err(vec!["No lesson is selected".to_owned()]);
    };

    if content
        .themes
        .iter()
        .filter(|theme| theme.id == active_theme.0)
        .count()
        != 1
    {
        return Err(vec![format!(
            "Theme {} is missing or ambiguous",
            active_theme.0
        )]);
    }
    let Some(theme_data) = content.theme(active_theme) else {
        return Err(vec![format!("Theme {} was not indexed", active_theme.0)]);
    };
    if !theme_data.available {
        return Err(vec![format!("Theme {} is unavailable", active_theme.0)]);
    }

    if theme_data
        .lessons
        .iter()
        .filter(|lesson| lesson.id == *lesson_id)
        .count()
        != 1
    {
        return Err(vec![format!("Lesson {lesson_id} is missing or ambiguous")]);
    }
    let Some(lesson) = theme_data.lesson(lesson_id) else {
        return Err(vec![format!("Lesson {lesson_id} was not indexed")]);
    };
    if !lesson.available {
        return Err(vec![format!("Lesson {lesson_id} is unavailable")]);
    }

    let mut rng = rand::rng();
    plan_session(lesson, lesson_config, &mut rng)
}

/// Plan question instances using only the lesson catalog, settings, and RNG.
pub fn plan_session(
    lesson: &Lesson,
    lesson_config: Option<&LessonSessionConfig>,
    rng: &mut impl Rng,
) -> Result<LessonSession, Vec<String>> {
    let mut issues = lesson.validation_issues();
    if let Some(config) = lesson_config {
        issues.extend(validate_lesson_config(lesson, config));
    }
    if !issues.is_empty() {
        return Err(issues);
    }

    let mut seen = HashSet::new();
    let mut pool: Vec<ResolvedQuestion> = Vec::new();
    for entry in &lesson.questions {
        let repetitions = lesson_config
            .and_then(|config| config.counts.get(&entry.id).copied())
            .unwrap_or(1);
        let configured_visual =
            lesson_config.and_then(|config| config.show_visuals.get(&entry.id).copied());
        let hide_optional_visual = entry
            .definition
            .effective_show_visual(configured_visual)
            .is_some_and(|show| !show);
        for _ in 0..repetitions {
            pool.push(ResolvedQuestion {
                exercise_id: entry.id.clone(),
                instance: resolve_unique(entry, rng, &mut seen),
                hide_optional_visual,
            });
        }
    }
    pool.shuffle(rng);

    Ok(LessonSession {
        questions: pool,
        current_index: 0,
        correct_count: 0,
        total_answered: 0,
        type_scores: HashMap::new(),
    })
}

fn validate_lesson_config(lesson: &Lesson, config: &LessonSessionConfig) -> Vec<String> {
    let mut issues = Vec::new();
    for (id, count) in &config.counts {
        if !lesson.questions.iter().any(|entry| entry.id == *id) {
            issues.push(format!(
                "Repetition setting references unknown exercise ID {id}"
            ));
            continue;
        }
        if *count > crate::data::content::MAX_QUESTION_REPETITIONS {
            issues.push(format!("Exercise {id} exceeds the repetition limit"));
        }
    }
    for id in config.show_visuals.keys() {
        let Some(entry) = lesson.questions.iter().find(|entry| entry.id == *id) else {
            issues.push(format!(
                "Visual setting references unknown exercise ID {id}"
            ));
            continue;
        };
        if !entry.definition.has_optional_visual() {
            issues.push(format!("Exercise {id} has no configurable visual"));
        }
    }
    issues
}

fn resolve_unique(
    entry: &QuestionEntry,
    rng: &mut impl Rng,
    seen: &mut HashSet<u64>,
) -> QuestionInstance {
    for _ in 0..MAX_DEDUP_RETRIES {
        let instance = resolve_question(&entry.definition, rng);
        if let Some(fingerprint) = instance.fingerprint() {
            if seen.insert(fingerprint) {
                return instance;
            }
        } else {
            return instance;
        }
    }
    resolve_question(&entry.definition, rng)
}
