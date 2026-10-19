use std::collections::HashSet;

use rand::Rng;
use rand::seq::SliceRandom;

use super::generation::resolve_question;
use super::types::{
    Lesson, MAX_QUESTION_REPETITIONS, QuestionEntry, QuestionInstance, ResolvedQuestion,
};
use crate::data::{
    ActiveTheme, ContentLibrary, LessonLaunchSelection, LessonSession, SelectedLesson,
};

const MAX_DEDUP_RETRIES: u32 = 20;

#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct LessonPlanningConfig {
    pub counts: std::collections::HashMap<String, usize>,
    pub show_visuals: std::collections::HashMap<String, bool>,
}

pub fn build_session_with_selection(
    selected_lesson: &SelectedLesson,
    selection: &LessonLaunchSelection,
    content: &ContentLibrary,
    active_theme: &ActiveTheme,
    lesson_config: Option<&LessonPlanningConfig>,
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
    let mut rng = rand::rng();
    match selection {
        crate::data::LessonLaunchSelection::CompleteLesson => {
            plan_session(lesson, lesson_config, &mut rng)
        }
        selection @ crate::data::LessonLaunchSelection::SingleActivity { .. } => {
            plan_session_with_selection(lesson, selection, lesson_config, &mut rng)
        }
    }
}

#[allow(dead_code)]
pub fn plan_session(
    lesson: &Lesson,
    lesson_config: Option<&LessonPlanningConfig>,
    rng: &mut impl Rng,
) -> Result<LessonSession, Vec<String>> {
    plan_session_with_selection(
        lesson,
        &LessonLaunchSelection::CompleteLesson,
        lesson_config,
        rng,
    )
}

pub fn plan_session_with_selection(
    lesson: &Lesson,
    selection: &LessonLaunchSelection,
    lesson_config: Option<&LessonPlanningConfig>,
    rng: &mut impl Rng,
) -> Result<LessonSession, Vec<String>> {
    match selection {
        LessonLaunchSelection::CompleteLesson => plan_complete_session(lesson, lesson_config, rng),
        LessonLaunchSelection::SingleActivity { activity_id } => {
            plan_single_activity(lesson, activity_id, rng)
        }
    }
}

fn plan_complete_session(
    lesson: &Lesson,
    lesson_config: Option<&LessonPlanningConfig>,
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
    })
}

fn plan_single_activity(
    lesson: &Lesson,
    activity_id: &str,
    rng: &mut impl Rng,
) -> Result<LessonSession, Vec<String>> {
    let matches: Vec<&QuestionEntry> = lesson
        .questions
        .iter()
        .filter(|entry| entry.id == activity_id)
        .collect();
    let entry = match matches.as_slice() {
        [] => return Err(vec![format!("Unknown activity ID {activity_id}")]),
        [entry] => *entry,
        _ => return Err(vec![format!("Activity {activity_id} is ambiguous")]),
    };
    if entry.id.is_empty() {
        return Err(vec!["Selected activity ID must not be empty".to_owned()]);
    }
    let issues = entry
        .definition
        .validation_issues()
        .into_iter()
        .map(|issue| format!("Exercise {}: {issue}", entry.id))
        .collect::<Vec<_>>();
    if !issues.is_empty() {
        return Err(issues);
    }

    let hide_optional_visual = entry
        .definition
        .effective_show_visual(None)
        .is_some_and(|show| !show);
    let question = ResolvedQuestion {
        exercise_id: entry.id.clone(),
        instance: resolve_question(&entry.definition, rng),
        hide_optional_visual,
    };
    Ok(LessonSession {
        questions: vec![question],
        current_index: 0,
    })
}

fn validate_lesson_config(lesson: &Lesson, config: &LessonPlanningConfig) -> Vec<String> {
    let mut issues = Vec::new();
    for (id, count) in &config.counts {
        if !lesson.questions.iter().any(|entry| entry.id == *id) {
            issues.push(format!(
                "Repetition setting references unknown exercise ID {id}"
            ));
            continue;
        }
        if *count > MAX_QUESTION_REPETITIONS {
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
