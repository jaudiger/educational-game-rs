use std::collections::{HashMap, HashSet};

use bevy_persistent::prelude::Persistent;
use rand::seq::SliceRandom;

use crate::data::content::QuestionDefinition;
use crate::data::{
    ActiveSlot, ActiveStudent, ActiveTheme, AnswerResult, ContentLibrary, LessonSession,
    LessonSessionConfig, ResolvedQuestion, SaveData, SelectedLesson,
};

/// Maximum re-roll attempts when deduplicating resolved template questions.
const MAX_DEDUP_RETRIES: u32 = 20;

pub(super) fn build_session(
    selected_lesson: &SelectedLesson,
    content: &ContentLibrary,
    active_theme: &ActiveTheme,
    lesson_config: Option<&LessonSessionConfig>,
) -> LessonSession {
    let Some(lesson_id) = selected_lesson.as_ref() else {
        return LessonSession::default();
    };

    let Some(theme_data) = content.theme(active_theme) else {
        return LessonSession::default();
    };

    let Some(lesson) = theme_data.lesson(lesson_id) else {
        return LessonSession::default();
    };

    let mut rng = rand::rng();

    let resolved: Vec<(QuestionDefinition, bool)> = if let Some(config) = lesson_config {
        // Teacher config: repetition counts per question + visual toggles
        let mut pool: Vec<(QuestionDefinition, bool)> = Vec::new();
        let mut seen = HashSet::new();
        for (i, question) in lesson.questions.iter().enumerate() {
            let count = config.counts.get(i).copied().unwrap_or(1);
            let hide = question.has_optional_visual()
                && !config
                    .show_visuals
                    .get(i)
                    .copied()
                    .unwrap_or_else(|| question.default_show_visual());
            for _ in 0..count {
                pool.push((resolve_unique(question, &mut rng, &mut seen), hide));
            }
        }
        pool.shuffle(&mut rng);
        pool
    } else {
        // No config: default behavior (all questions once, visuals shown)
        let mut seen = HashSet::new();
        let mut pool: Vec<(QuestionDefinition, bool)> = lesson
            .questions
            .iter()
            .map(|q| (resolve_unique(q, &mut rng, &mut seen), false))
            .collect();
        pool.shuffle(&mut rng);
        pool
    };

    let questions = resolved
        .into_iter()
        .map(|(definition, hide_visual)| ResolvedQuestion {
            definition,
            hide_visual,
        })
        .collect();

    LessonSession {
        questions,
        current_index: 0,
        correct_count: 0,
        total_answered: 0,
        type_scores: HashMap::new(),
    }
}

/// Resolve a single `QuestionDefinition`. Templates are turned into their
/// concrete counterpart; static definitions are returned unchanged.
fn resolve_question(def: &QuestionDefinition, rng: &mut impl rand::Rng) -> QuestionDefinition {
    match def {
        // Static definitions: pass through.
        QuestionDefinition::Mcq(_)
        | QuestionDefinition::FractionVisualization(_)
        | QuestionDefinition::FractionComparison(_)
        | QuestionDefinition::FractionIdentification(_)
        | QuestionDefinition::NumericInput(_) => def.clone(),

        // Templates: resolve.
        QuestionDefinition::McqTemplate(t) => QuestionDefinition::Mcq(t.resolve(rng)),
        QuestionDefinition::FractionVisualizationTemplate(t) => {
            QuestionDefinition::FractionVisualization(t.resolve(rng))
        }
        QuestionDefinition::FractionComparisonTemplate(t) => {
            QuestionDefinition::FractionComparison(t.resolve(rng))
        }
        QuestionDefinition::FractionIdentificationTemplate(t) => {
            QuestionDefinition::FractionIdentification(t.resolve(rng))
        }
        QuestionDefinition::NumericInputTemplate(t) => {
            QuestionDefinition::NumericInput(t.resolve(rng))
        }
    }
}

/// Resolve a template, re-rolling up to `MAX_DEDUP_RETRIES` times to avoid
/// producing a question already present in `seen`. The fingerprint is
/// inserted into `seen` on success.
fn resolve_unique(
    def: &QuestionDefinition,
    rng: &mut impl rand::Rng,
    seen: &mut HashSet<u64>,
) -> QuestionDefinition {
    for _ in 0..MAX_DEDUP_RETRIES {
        let resolved = resolve_question(def, rng);
        if let Some(fp) = resolved.fingerprint() {
            if seen.insert(fp) {
                return resolved;
            }
        } else {
            // Static definition or no fingerprint; accept as-is.
            return resolved;
        }
    }
    // Exhausted retries; accept the last attempt to avoid an infinite loop.
    resolve_question(def, rng)
}

pub(super) fn update_session_score(session: &mut LessonSession, result: &AnswerResult) {
    let is_correct = matches!(result, AnswerResult::Correct);
    session.total_answered += 1;
    if is_correct {
        session.correct_count += 1;
    }

    // Track per-question-type scores in the session.
    if let Some(question) = session.questions.get(session.current_index) {
        let qt = question.definition.question_type();
        let entry = session.type_scores.entry(qt).or_default();
        entry.total += 1;
        if is_correct {
            entry.correct += 1;
        }
    }
}

/// In class mode, persist the current answer's result to the active student's
/// cumulative progress. If no student is selected, the result is discarded.
pub(super) fn record_class_answer(
    session: &LessonSession,
    last_answer: &AnswerResult,
    active_student: Option<&ActiveStudent>,
    active_slot: Option<&ActiveSlot>,
    selected_lesson: Option<&SelectedLesson>,
    save_data: &mut Persistent<SaveData>,
) {
    let Some(student) = active_student else {
        return;
    };
    let Some(slot) = active_slot else { return };
    let Some(selected) = selected_lesson else {
        return;
    };
    let Some(ref lesson_id) = selected.0 else {
        return;
    };
    let Some(question) = session.questions.get(session.current_index) else {
        return;
    };

    let is_correct = matches!(last_answer, AnswerResult::Correct);
    let qt = question.definition.question_type();
    let lesson_id = lesson_id.clone();
    let slot_index = slot.0;
    let student_index = student.0;

    let _ = save_data.update(|data| {
        let Some(class_save) = data.class_slots[slot_index].as_mut() else {
            return;
        };
        let Some(student_data) = class_save.students.get_mut(student_index) else {
            return;
        };
        let lesson_progress = student_data.progress.entry(lesson_id.clone()).or_default();
        let type_score = lesson_progress.type_scores.entry(qt).or_default();
        type_score.total += 1;
        if is_correct {
            type_score.correct += 1;
        }
    });
}
