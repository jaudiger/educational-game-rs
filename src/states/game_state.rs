use bevy::prelude::*;

use super::AppState;

#[derive(Clone, Copy, Debug, Default, Eq, Hash, PartialEq, SubStates)]
#[source(AppState = AppState::LessonPlay)]
pub enum LessonPhase {
    #[default]
    ShowQuestion,
    WaitingAnswer,
    ShowFeedback,
    Transitioning,
}

#[derive(Clone, Copy, Debug, Default, Eq, Hash, PartialEq, SubStates)]
#[source(AppState = AppState::ThemeExploration)]
pub enum ExplorationView {
    #[default]
    Themes,
    ThemeLessons,
}
