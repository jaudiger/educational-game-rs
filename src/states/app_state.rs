use bevy::prelude::*;

/// Top-level application state driving screen transitions.
#[derive(Clone, Copy, Debug, Default, Eq, Hash, PartialEq, States)]
#[states(scoped_entities)]
pub enum AppState {
    #[default]
    Home,
    SaveSlots,
    ThemeExploration,
    LessonPlay,
    LessonSummary,
    Settings,
}

/// The `AppState` variants where the lesson flow is active.
/// Used for screen cleanup and view resets on each intra-flow transition.
pub const LESSON_FLOW_STATES: [AppState; 3] = [
    AppState::ThemeExploration,
    AppState::LessonPlay,
    AppState::LessonSummary,
];

/// Computed state that is active when the application is in the lesson flow
/// (theme exploration, lesson play, or lesson summary).
///
/// Use `in_state(InLessonFlow)` as a run condition instead of chaining
/// `.or(in_state(...))` for each variant.
#[derive(Clone, Copy, Debug, Eq, Hash, PartialEq)]
pub struct InLessonFlow;

impl ComputedStates for InLessonFlow {
    const ALLOW_SAME_STATE_TRANSITIONS: bool = false;

    type SourceStates = AppState;

    fn compute(sources: AppState) -> Option<Self> {
        match sources {
            AppState::ThemeExploration | AppState::LessonPlay | AppState::LessonSummary => {
                Some(Self)
            }
            _ => None,
        }
    }
}

/// Computed state covering the active-lesson cycle (`LessonPlay` and
/// `LessonSummary`). Used as the cleanup scope for per-lesson runtime
/// resources so that both normal completion and early exit (quit button)
/// trigger a single `OnExit` cleanup when returning to `ThemeExploration`.
#[derive(Clone, Copy, Debug, Eq, Hash, PartialEq)]
pub struct ActiveLesson;

impl ComputedStates for ActiveLesson {
    const ALLOW_SAME_STATE_TRANSITIONS: bool = false;

    type SourceStates = AppState;

    fn compute(sources: AppState) -> Option<Self> {
        match sources {
            AppState::LessonPlay | AppState::LessonSummary => Some(Self),
            _ => None,
        }
    }
}
