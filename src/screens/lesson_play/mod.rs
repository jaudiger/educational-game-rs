use bevy::prelude::*;
use bevy::ui::UiSystems;

use crate::data::{LastAnswer, LessonLaunchSelection, LessonSession, SelectedLesson};
use crate::i18n::{I18n, TranslationKey};
use crate::questions::QuestionRoot;
use crate::questions::registry::{
    QuestionInteractionSet, QuestionResponseSubmitted, assess_question_responses,
};
use crate::states::cleanup_root;
use crate::states::{ActiveLesson, AppState, InLessonFlow, LessonPhase, StateScopedResourceExt};
use crate::ui::components::ButtonActivated;

pub mod explanations;
pub mod layout;
#[cfg(not(feature = "demo"))]
pub mod session;
mod visuals;

/// Lesson play screen orchestrating question display, feedback, and scoring.
pub struct LessonPlayScreenPlugin;

#[derive(SystemSet, Clone, Copy, Debug, Eq, Hash, PartialEq)]
pub enum LessonFeedbackSet {
    Cleanup,
    RecordAnswer,
    PreparePresentation,
    Presentation,
}

impl Plugin for LessonPlayScreenPlugin {
    fn build(&self, app: &mut App) {
        app.add_message::<QuestionResponseSubmitted>()
            .init_resource::<explanations::FeedbackPresentation>()
            .register_state_scoped_resource::<ActiveLesson, LessonSession>(ActiveLesson)
            .register_state_scoped_resource::<InLessonFlow, SelectedLesson>(InLessonFlow)
            .register_state_scoped_resource::<InLessonFlow, LessonLaunchSelection>(InLessonFlow)
            .register_state_scoped_resource::<ActiveLesson, LastAnswer>(ActiveLesson)
            .init_resource::<layout::LessonPlanningInput>()
            .configure_sets(
                OnEnter(LessonPhase::ShowFeedback),
                (
                    LessonFeedbackSet::Cleanup,
                    LessonFeedbackSet::RecordAnswer,
                    LessonFeedbackSet::PreparePresentation,
                    LessonFeedbackSet::Presentation,
                )
                    .chain(),
            )
            .add_systems(OnEnter(AppState::LessonPlay), layout::setup_lesson_play)
            .add_systems(
                PostUpdate,
                layout::fit_question_to_available_height
                    .run_if(in_state(AppState::LessonPlay))
                    .run_if(in_state(LessonPhase::ShowQuestion))
                    .after(UiSystems::Layout)
                    .after(crate::ui::layout::ResponsiveLayoutSet),
            )
            .add_systems(
                PostUpdate,
                layout::fit_feedback_to_available_height
                    .run_if(in_state(AppState::LessonPlay))
                    .run_if(in_state(LessonPhase::ShowFeedback))
                    .after(UiSystems::Layout),
            )
            .add_systems(
                Update,
                assess_question_responses
                    .run_if(in_state(LessonPhase::WaitingAnswer))
                    .after(QuestionInteractionSet),
            )
            .add_systems(
                OnEnter(LessonPhase::ShowQuestion),
                (update_progress_text, transition_to_waiting).chain(),
            )
            .add_systems(
                OnEnter(LessonPhase::ShowFeedback),
                cleanup_root::<QuestionRoot>.in_set(LessonFeedbackSet::Cleanup),
            )
            .add_systems(
                OnEnter(LessonPhase::ShowFeedback),
                explanations::setup_feedback_ui.in_set(LessonFeedbackSet::Presentation),
            )
            .add_systems(OnEnter(LessonPhase::Transitioning), advance_question)
            .add_systems(
                Update,
                handle_quit_lesson.run_if(in_state(AppState::LessonPlay)),
            );
    }
}

#[derive(Component, Reflect)]
struct ProgressText;

#[derive(Component, Reflect)]
struct FeedbackRoot;

#[derive(Component, Reflect)]
struct QuitLessonButton;

fn update_progress_text(
    mut progress_query: Query<&mut Text, With<ProgressText>>,
    session: Option<Res<LessonSession>>,
    i18n: Res<I18n>,
) {
    let Some(session) = session else { return };
    for mut text in &mut progress_query {
        *text = Text::new(i18n.t(&TranslationKey::QuestionProgress(
            session.current_index + 1,
            session.questions.len(),
        )));
    }
}

fn transition_to_waiting(mut next_phase: ResMut<NextState<LessonPhase>>) {
    next_phase.set(LessonPhase::WaitingAnswer);
}

fn advance_question(
    mut session: ResMut<LessonSession>,
    mut next_phase: ResMut<NextState<LessonPhase>>,
    launch_selection: Option<Res<LessonLaunchSelection>>,
    mut next_app_state: ResMut<NextState<AppState>>,
) {
    session.current_index += 1;
    if session.current_index < session.questions.len() {
        next_phase.set(LessonPhase::ShowQuestion);
    } else {
        let default_selection = LessonLaunchSelection::CompleteLesson;
        let selection = launch_selection.as_deref().unwrap_or(&default_selection);
        next_app_state.set(completed_session_destination(selection));
    }
}

const fn completed_session_destination(selection: &LessonLaunchSelection) -> AppState {
    match selection {
        LessonLaunchSelection::CompleteLesson => AppState::LessonSummary,
        LessonLaunchSelection::SingleActivity { .. } => AppState::ThemeExploration,
    }
}

fn handle_quit_lesson(
    mut activations: MessageReader<ButtonActivated>,
    query: Query<(), With<QuitLessonButton>>,
    mut next_state: ResMut<NextState<AppState>>,
) {
    for activation in activations.read() {
        if query.contains(activation.0) {
            next_state.set(AppState::ThemeExploration);
        }
    }
}
