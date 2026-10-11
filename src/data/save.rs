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

/// Score for a single question type within a lesson.
#[derive(Clone, Debug, Default, Deserialize, Serialize)]
pub struct TypeScore {
    pub correct: u32,
    pub total: u32,
}

impl TypeScore {
    /// Computes the percentage of correct answers for this question type.
    pub fn percentage(&self) -> u32 {
        (self.correct * 100).checked_div(self.total).unwrap_or(0)
    }
}

/// Progress for a single lesson, broken down by question type.
///
/// - **Class mode**: cumulative, scores add up across sessions.
/// - **Individual mode**: last score, replaced entirely each session.
#[derive(Clone, Debug, Default, Deserialize, Serialize)]
pub struct LessonProgress {
    pub type_scores: HashMap<QuestionType, TypeScore>,
}

impl LessonProgress {
    /// Total correct answers across all question types.
    pub fn total_correct(&self) -> u32 {
        self.type_scores.values().map(|ts| ts.correct).sum()
    }

    /// Total questions answered across all question types.
    pub fn total_questions(&self) -> u32 {
        self.type_scores.values().map(|ts| ts.total).sum()
    }

    /// Overall percentage across all question types.
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

#[cfg(test)]
mod tests {
    use super::LessonSessionConfig;
    use std::collections::HashMap;

    #[test]
    fn old_position_based_settings_are_discarded_without_remapping() {
        let old = r#"{"counts":[3,0],"show_visuals":[true,false]}"#;
        let config: LessonSessionConfig = serde_json::from_str(old).unwrap();
        assert!(config.counts.is_empty());
        assert!(config.show_visuals.is_empty());
    }

    #[test]
    fn stable_id_settings_round_trip_as_keyed_values() {
        let config = LessonSessionConfig {
            counts: HashMap::from([("exercise-a".to_owned(), 3)]),
            show_visuals: HashMap::from([("exercise-a".to_owned(), true)]),
        };
        let json = serde_json::to_string(&config).unwrap();
        let decoded: LessonSessionConfig = serde_json::from_str(&json).unwrap();
        assert_eq!(decoded.counts.get("exercise-a"), Some(&3));
        assert_eq!(decoded.show_visuals.get("exercise-a"), Some(&true));
    }
}
