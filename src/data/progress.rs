use bevy::prelude::*;
use serde::{Deserialize, Serialize};

use super::content::{AnswerResult, ResolvedQuestion};

pub use crate::i18n::Language;

/// Visual theme applied to the theme exploration screen.
#[derive(Clone, Copy, Debug, Default, Eq, PartialEq, Reflect, Deserialize, Serialize)]
pub enum ExplorationTheme {
    #[default]
    Sky,
    Ocean,
    Space,
}

/// User preferences; Production persists them between runs.
#[derive(Resource, Clone, Debug, Reflect, Deserialize, Serialize)]
#[serde(deny_unknown_fields)]
pub struct GameSettings {
    pub music_volume: f32,
    pub sfx_volume: f32,
    pub show_explanations: bool,
    pub language: Language,
    pub exploration_theme: ExplorationTheme,
    pub gamepad_navigation: bool,
}

impl Default for GameSettings {
    fn default() -> Self {
        Self {
            music_volume: 0.75,
            sfx_volume: 0.75,
            show_explanations: true,
            language: Language::default(),
            exploration_theme: ExplorationTheme::default(),
            gamepad_navigation: false,
        }
    }
}

impl GameSettings {
    #[allow(dead_code)]
    pub(crate) fn validate_persisted(&self) -> Result<(), String> {
        let mut invalid_fields = Vec::new();
        if !self.music_volume.is_finite() || !(0.0..=1.0).contains(&self.music_volume) {
            invalid_fields.push("music_volume");
        }
        if !self.sfx_volume.is_finite() || !(0.0..=1.0).contains(&self.sfx_volume) {
            invalid_fields.push("sfx_volume");
        }
        if invalid_fields.is_empty() {
            Ok(())
        } else {
            Err(format!(
                "invalid setting fields: {}",
                invalid_fields.join(", ")
            ))
        }
    }
}

/// Tracks which lesson ID is selected for the current session.
#[derive(Resource, Clone, Debug, Default, Deref, Reflect)]
pub struct SelectedLesson(pub Option<String>);

/// Selects whether a lesson launches its configured series or one activity.
#[derive(Resource, Clone, Debug, Default, PartialEq, Eq, Reflect)]
pub enum LessonLaunchSelection {
    #[default]
    CompleteLesson,
    SingleActivity {
        activity_id: String,
    },
}

/// Runtime state for an active lesson: exercise list and index.
#[derive(Resource, Clone, Debug, Default, Reflect)]
pub struct LessonSession {
    #[reflect(ignore)]
    pub questions: Vec<ResolvedQuestion>,
    pub current_index: usize,
}

impl LessonSession {
    /// Returns the question at the current index.
    pub fn current(&self) -> Option<&ResolvedQuestion> {
        self.questions.get(self.current_index)
    }
}

/// Tracks which theme is currently selected when in `ExplorationView::ThemeLessons`.
#[derive(Resource, Clone, Debug, Deref, Reflect)]
pub struct ActiveTheme(pub String);

/// Marker component for the question area in the lesson play screen.
/// Question type plugins spawn their UI as children of this entity.
#[derive(Component, Reflect)]
pub struct QuestionContainer;

/// Stores the result of the last submitted answer.
/// Inserted by question plugins, read by lesson play for feedback display.
#[derive(Resource, Clone, Debug, Deref, Reflect)]
pub struct LastAnswer(#[reflect(ignore)] pub AnswerResult);

#[derive(Message, Clone, Copy, Debug)]
pub struct SettingsChanged;
