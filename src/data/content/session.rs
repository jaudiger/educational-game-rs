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

#[cfg(test)]
mod tests {
    use std::collections::HashMap;

    use rand::SeedableRng;
    use rand::rngs::StdRng;

    use crate::data::content::types::{
        Difficulty, LocalizedExplanation, LocalizedText, McqDefinition, NumericInputTemplate,
        QuestionDefinition, QuestionEntry, QuestionInstance,
    };

    use super::plan_session;
    use crate::data::LessonSessionConfig;

    fn numeric_entry(id: &str, answer: u32, rows: u32) -> QuestionEntry {
        QuestionEntry::new(
            id,
            QuestionDefinition::NumericInput(crate::data::content::NumericInputDefinition {
                prompt: LocalizedText::new("prompt", "prompt"),
                correct_answer: answer,
                explanation: LocalizedExplanation::plain(LocalizedText::new("", "")),
                explanation_visual: None,
                question_visual: Some(crate::data::content::QuestionVisual::MultiplicationGrid {
                    rows,
                    cols: 1,
                }),
            }),
        )
    }

    fn lesson(entries: Vec<QuestionEntry>) -> crate::data::content::Lesson {
        crate::data::content::Lesson {
            id: "lesson".to_owned(),
            title_key: crate::i18n::TranslationKey::LessonFractions,
            available: true,
            questions: entries,
        }
    }

    #[test]
    fn teacher_settings_follow_stable_exercise_ids_after_reordering() {
        let config = LessonSessionConfig {
            counts: HashMap::from([("exercise-a".to_owned(), 2), ("exercise-b".to_owned(), 0)]),
            show_visuals: HashMap::from([("exercise-a".to_owned(), true)]),
        };
        let mut rng = StdRng::seed_from_u64(12);
        for entries in [
            vec![
                numeric_entry("exercise-a", 11, 1),
                numeric_entry("exercise-b", 22, 2),
            ],
            vec![
                numeric_entry("exercise-b", 22, 2),
                numeric_entry("exercise-a", 11, 1),
            ],
        ] {
            let session = plan_session(&lesson(entries), Some(&config), &mut rng).unwrap();
            assert_eq!(session.questions.len(), 2);
            assert!(session.questions.iter().all(|question| {
                question.exercise_id == "exercise-a"
                    && matches!(question.instance, QuestionInstance::NumericInput(ref definition) if definition.correct_answer == 11)
                    && !question.hide_optional_visual
            }));
        }
    }

    #[test]
    fn generated_template_resolves_to_a_concrete_instance_before_planning() {
        let entry = QuestionEntry::new(
            "generated-product",
            QuestionDefinition::NumericInputTemplate(NumericInputTemplate {
                prompt_template: LocalizedText::new("{a} x {b} = ?", "{a} x {b} = ?"),
                factor_a_range: vec![4],
                factor_b_range: vec![3],
                difficulty: Difficulty::Beginner,
                explanation_template: LocalizedText::new(
                    "{a} x {b} = {result}",
                    "{a} x {b} = {result}",
                ),
                teacher_label: None,
                with_grid: false,
                place_value_explanation: false,
            }),
        );
        let mut rng = StdRng::seed_from_u64(7);
        let session = plan_session(&lesson(vec![entry]), None, &mut rng).unwrap();
        let question = &session.questions[0];
        assert_eq!(question.exercise_id, "generated-product");
        assert!(matches!(
            question.instance,
            QuestionInstance::NumericInput(ref definition)
                if definition.correct_answer == 12 && definition.prompt.fr == "4 x 3 = ?"
        ));
    }

    #[test]
    fn invalid_content_and_repetition_settings_reject_session_planning() {
        let invalid = QuestionEntry::new(
            "bad-choice",
            QuestionDefinition::Mcq(McqDefinition {
                prompt: LocalizedText::new("prompt", "prompt"),
                choices: vec!["only choice".to_owned()],
                correct_index: 1,
                explanation: LocalizedExplanation::plain(LocalizedText::new("", "")),
                explanation_visual: None,
                question_visual: None,
            }),
        );
        let mut rng = StdRng::seed_from_u64(4);
        assert!(plan_session(&lesson(vec![invalid]), None, &mut rng).is_err());

        let config = LessonSessionConfig {
            counts: HashMap::from([("exercise-a".to_owned(), 6)]),
            show_visuals: HashMap::new(),
        };
        assert!(
            plan_session(
                &lesson(vec![numeric_entry("exercise-a", 11, 1)]),
                Some(&config),
                &mut rng,
            )
            .is_err()
        );
    }
}
