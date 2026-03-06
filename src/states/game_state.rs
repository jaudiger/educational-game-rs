use bevy::prelude::*;

use super::AppState;

#[derive(Clone, Copy, Debug, Default, Eq, Hash, PartialEq, SubStates)]
#[source(AppState = AppState::LessonPlay)]
#[states(scoped_entities)]
pub enum LessonPhase {
    #[default]
    ShowQuestion,
    WaitingAnswer,
    ShowFeedback,
    Transitioning,
}

#[derive(Clone, Copy, Debug, Default, Eq, Hash, PartialEq, SubStates)]
#[source(AppState = AppState::MapExploration)]
#[states(scoped_entities)]
pub enum MapView {
    #[default]
    WorldOverview,
    ThemeDetail,
}
