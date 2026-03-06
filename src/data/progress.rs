use std::collections::HashMap;

use bevy::prelude::*;
use serde::{Deserialize, Serialize};

use super::content::{AnswerResult, QuestionType, ResolvedQuestion};
use super::save::TypeScore;

pub use crate::i18n::Language;

/// Individual or group (class) play mode.
#[derive(Clone, Copy, Debug, Default, Eq, PartialEq, Reflect, Deserialize, Serialize)]
pub enum GameMode {
    #[default]
    Individual,
    Group,
}

/// Visual theme for the map exploration screen.
#[derive(Clone, Copy, Debug, Default, Eq, PartialEq, Reflect, Deserialize, Serialize)]
pub enum MapTheme {
    #[default]
    Sky,
    Ocean,
    Space,
}

/// Persistent user preferences (volume, language, mode, theme).
#[derive(Resource, Clone, Debug, Reflect, Deserialize, Serialize)]
pub struct GameSettings {
    pub music_volume: f32,
    pub sfx_volume: f32,
    pub show_explanations: bool,
    pub mode: GameMode,
    pub language: Language,
    pub map_theme: MapTheme,
    pub gamepad_navigation: bool,
}

impl Default for GameSettings {
    fn default() -> Self {
        Self {
            music_volume: 0.75,
            sfx_volume: 0.75,
            show_explanations: true,
            mode: GameMode::Individual,
            language: Language::default(),
            map_theme: MapTheme::default(),
            gamepad_navigation: false,
        }
    }
}

/// Tracks which lesson ID is selected for the current session.
#[derive(Resource, Clone, Debug, Default, Deref, Reflect)]
pub struct SelectedLesson(pub Option<String>);

/// Runtime state for an active lesson: question list, index, and scores.
#[derive(Resource, Clone, Debug, Default, Reflect)]
pub struct LessonSession {
    #[reflect(ignore)]
    pub questions: Vec<ResolvedQuestion>,
    pub current_index: usize,
    pub correct_count: u32,
    pub total_answered: u32,
    /// Per-question-type score breakdown for the current session.
    #[reflect(ignore)]
    pub type_scores: HashMap<QuestionType, TypeScore>,
}

impl LessonSession {
    /// Returns the question at the current index.
    pub fn current(&self) -> Option<&ResolvedQuestion> {
        self.questions.get(self.current_index)
    }
}

/// Tracks which theme is currently selected when in `MapView::ThemeDetail`.
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
