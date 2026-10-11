use std::collections::{BTreeSet, HashMap, HashSet};

use bevy::prelude::*;
use serde::{Deserialize, Deserializer, Serialize};

use super::content::{ContentLibrary, QuestionType};
use super::progress::GameMode;

/// Teacher-configured settings for a single lesson in a class slot.
#[derive(Clone, Debug, Serialize)]
pub struct LessonSessionConfig {
    pub counts: HashMap<String, usize>,
    pub show_visuals: HashMap<String, bool>,
}

#[derive(Deserialize)]
#[serde(untagged, deny_unknown_fields)]
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
#[serde(deny_unknown_fields)]
pub struct QuestionScore {
    pub question_type: QuestionType,
    pub correct: u64,
    pub total: u64,
}

/// Cumulative lesson progress, broken down by stable exercise ID.
#[derive(Clone, Debug, Default, Deserialize, Serialize)]
#[serde(deny_unknown_fields)]
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
    ) -> bool {
        let score = self
            .question_scores
            .entry(exercise_id.to_owned())
            .or_insert(QuestionScore {
                question_type,
                correct: 0,
                total: 0,
            });
        let Some(total) = score.total.checked_add(1) else {
            return false;
        };
        let correct = if is_correct {
            let Some(correct) = score.correct.checked_add(1) else {
                return false;
            };
            correct
        } else {
            score.correct
        };
        score.total = total;
        score.correct = correct;
        true
    }

    /// Total correct answers across all persisted exercise scores.
    pub fn total_correct(&self) -> u128 {
        self.question_scores
            .values()
            .map(|score| u128::from(score.correct))
            .fold(0, u128::saturating_add)
    }

    /// Total answers across all persisted exercise scores.
    pub fn total_questions(&self) -> u128 {
        self.question_scores
            .values()
            .map(|score| u128::from(score.total))
            .fold(0, u128::saturating_add)
    }

    /// Overall percentage derived from all persisted exercise scores.
    pub fn percentage(&self) -> u32 {
        score_percentage(self.total_correct(), self.total_questions())
    }

    fn invalid_score_count(&self) -> usize {
        self.question_scores
            .values()
            .filter(|score| score.total == 0 || score.correct > score.total)
            .count()
    }
}

pub fn score_percentage(correct: u128, total: u128) -> u32 {
    if total == 0 || correct > total {
        return 0;
    }
    if correct == total {
        return 100;
    }

    let distance = total - correct;
    let mut remainder = 0;
    let mut percentage = 0;
    for _ in 0..100 {
        if remainder >= distance {
            remainder -= distance;
            percentage += 1;
        } else {
            remainder += correct;
        }
    }
    percentage
}

/// A student record within a class save slot, holding per-lesson progress.
#[derive(Clone, Debug, Default, Deserialize, Serialize)]
#[serde(deny_unknown_fields)]
pub struct ClassStudent {
    pub name: String,
    pub progress: HashMap<String, LessonProgress>,
}

/// An individual save slot with a player name and per-lesson progress.
#[derive(Clone, Debug, Deserialize, Serialize)]
#[serde(deny_unknown_fields)]
pub struct IndividualSave {
    pub name: String,
    pub progress: HashMap<String, LessonProgress>,
}

/// A class save slot with a class name, student roster, and teacher-configured sessions.
#[derive(Clone, Debug, Deserialize, Serialize)]
#[serde(deny_unknown_fields)]
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
#[serde(deny_unknown_fields)]
pub struct SaveData {
    #[reflect(ignore)]
    pub individual_slots: [Option<IndividualSave>; 3],
    #[reflect(ignore)]
    pub class_slots: [Option<ClassSave>; 3],
}

impl SaveData {
    pub(crate) fn validate_persisted(&self, content: &ContentLibrary) -> Result<(), String> {
        let known_lesson_ids = content
            .themes
            .iter()
            .flat_map(|theme| &theme.lessons)
            .map(|lesson| lesson.id.as_str())
            .collect::<HashSet<_>>();
        let mut invalid_count = 0_usize;
        let mut unknown_lesson_ids = BTreeSet::new();
        for save in self.individual_slots.iter().flatten() {
            invalid_count = invalid_count.saturating_add(invalid_progress_count(
                &save.progress,
                &known_lesson_ids,
                &mut unknown_lesson_ids,
            ));
        }
        for save in self.class_slots.iter().flatten() {
            for student in &save.students {
                invalid_count = invalid_count.saturating_add(invalid_progress_count(
                    &student.progress,
                    &known_lesson_ids,
                    &mut unknown_lesson_ids,
                ));
            }
        }

        let mut issues = Vec::new();
        if !unknown_lesson_ids.is_empty() {
            issues.push(format!(
                "progress references unknown lesson IDs: {}",
                unknown_lesson_ids
                    .into_iter()
                    .collect::<Vec<_>>()
                    .join(", ")
            ));
        }
        if invalid_count > 0 {
            issues.push(format!("{invalid_count} invalid score records"));
        }
        if issues.is_empty() {
            Ok(())
        } else {
            Err(issues.join("; "))
        }
    }
}

fn invalid_progress_count(
    progress: &HashMap<String, LessonProgress>,
    known_lesson_ids: &HashSet<&str>,
    unknown_lesson_ids: &mut BTreeSet<String>,
) -> usize {
    progress.iter().fold(0_usize, |count, (lesson_id, lesson)| {
        if !known_lesson_ids.contains(lesson_id.as_str()) {
            unknown_lesson_ids.insert(lesson_id.clone());
        }
        let invalid_scores = lesson.invalid_score_count();
        let empty_progress = usize::from(lesson.question_scores.is_empty());
        count
            .saturating_add(invalid_scores)
            .saturating_add(empty_progress)
    })
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
