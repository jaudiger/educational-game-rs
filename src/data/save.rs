use std::collections::HashMap;

use bevy::prelude::*;
use serde::{Deserialize, Deserializer, Serialize};

use super::content::QuestionType;
use super::progress::GameMode;

/// Teacher-configured settings for a single lesson in a class slot.
#[derive(Clone, Debug, Serialize)]
pub struct LessonSessionConfig {
    pub counts: HashMap<String, usize>,
    pub show_visuals: HashMap<String, bool>,
}

#[derive(Deserialize)]
#[serde(untagged)]
enum LessonSessionConfigWire {
    StableIds {
        counts: HashMap<String, usize>,
        #[serde(default)]
        show_visuals: HashMap<String, bool>,
    },
    LegacyPositions {
        counts: Vec<usize>,
        #[serde(default)]
        show_visuals: Vec<bool>,
    },
}

impl<'de> Deserialize<'de> for LessonSessionConfig {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        match LessonSessionConfigWire::deserialize(deserializer)? {
            LessonSessionConfigWire::StableIds {
                counts,
                show_visuals,
            } => Ok(Self {
                counts,
                show_visuals,
            }),
            LessonSessionConfigWire::LegacyPositions {
                counts,
                show_visuals,
            } => {
                let _ = (counts, show_visuals);
                Ok(Self {
                    counts: HashMap::new(),
                    show_visuals: HashMap::new(),
                })
            }
        }
    }
}

/// Score for one stable exercise ID within a lesson.
#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct QuestionScore {
    pub question_type: QuestionType,
    pub correct: u32,
    pub total: u32,
}

/// Cumulative lesson progress, broken down by stable exercise ID.
#[derive(Clone, Debug, Default, Deserialize, Serialize)]
pub struct LessonProgress {
    pub question_scores: HashMap<String, QuestionScore>,
}

impl LessonProgress {
    /// Adds one answer to the cumulative score for an exercise.
    pub fn record_answer(
        &mut self,
        exercise_id: &str,
        question_type: QuestionType,
        is_correct: bool,
    ) {
        let score = self
            .question_scores
            .entry(exercise_id.to_owned())
            .or_insert(QuestionScore {
                question_type,
                correct: 0,
                total: 0,
            });
        score.total += 1;
        if is_correct {
            score.correct += 1;
        }
    }

    /// Total correct answers across all persisted exercise scores.
    pub fn total_correct(&self) -> u32 {
        self.question_scores
            .values()
            .map(|score| score.correct)
            .sum()
    }

    /// Total answers across all persisted exercise scores.
    pub fn total_questions(&self) -> u32 {
        self.question_scores.values().map(|score| score.total).sum()
    }

    /// Overall percentage derived from all persisted exercise scores.
    pub fn percentage(&self) -> u32 {
        (self.total_correct() * 100)
            .checked_div(self.total_questions())
            .unwrap_or(0)
    }
}

/// A student record within a class save slot, holding per-lesson progress.
#[derive(Clone, Debug, Default, Deserialize, Serialize)]
pub struct ClassStudent {
    pub name: String,
    pub progress: HashMap<String, LessonProgress>,
}

/// An individual save slot with a player name and per-lesson progress.
#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct IndividualSave {
    pub name: String,
    pub progress: HashMap<String, LessonProgress>,
}

/// A class save slot with a class name, student roster, and teacher-configured sessions.
#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct ClassSave {
    pub name: String,
    pub students: Vec<ClassStudent>,
    /// Per-lesson session configuration set by the teacher.
    /// Key is the lesson ID.
    #[serde(default)]
    pub lesson_configs: HashMap<String, LessonSessionConfig>,
}

/// Top-level persistent resource holding all individual and class save slots.
#[derive(Resource, Clone, Debug, Default, Reflect, Deserialize, Serialize)]
pub struct SaveData {
    #[reflect(ignore)]
    pub individual_slots: [Option<IndividualSave>; 3],
    #[reflect(ignore)]
    pub class_slots: [Option<ClassSave>; 3],
}

/// Runtime context for the selected save slot during the lesson flow.
#[derive(Resource, Clone, Debug, Reflect)]
pub struct PlayerSession {
    pub slot_index: usize,
}

/// Runtime resource tracking which student is answering in class mode.
///
/// Automatically cleared at each new question during `LessonPlay`.
/// If absent when an answer is submitted, the result is not attributed
/// to anyone.
#[derive(Resource, Clone, Debug, Deref, Reflect)]
pub struct ActiveStudent(pub usize);

/// Returns the progress map for the currently active player.
/// - Individual mode: reads from the active slot's `IndividualSave`.
/// - Class mode: reads from the active student in the class slot.
///
/// Returns `None` if the slot/student doesn't exist or if no student is selected in class mode.
pub fn get_current_progress(
    save_data: &SaveData,
    mode: GameMode,
    slot_index: usize,
    active_student: Option<usize>,
) -> Option<&HashMap<String, LessonProgress>> {
    match mode {
        GameMode::Individual => save_data.individual_slots[slot_index]
            .as_ref()
            .map(|s| &s.progress),
        GameMode::Group => {
            let student_idx = active_student?;
            save_data.class_slots[slot_index]
                .as_ref()?
                .students
                .get(student_idx)
                .map(|s| &s.progress)
        }
    }
}
