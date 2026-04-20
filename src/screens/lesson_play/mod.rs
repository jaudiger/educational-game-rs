use bevy::prelude::*;
use bevy_persistent::prelude::Persistent;

use crate::data::{
    ActiveSlot, ActiveStudent, GameMode, GameSettings, LastAnswer, LessonSession, PersistenceMut,
    SelectedLesson,
};
use crate::i18n::{I18n, TranslationKey};
use crate::questions::QuestionRoot;
use crate::states::cleanup_root;
use crate::states::{ActiveLesson, AppState, LessonPhase, StateScopedResourceExt};

mod explanations;
mod layout;
mod session;
mod visuals;

/// Lesson play screen orchestrating question display, feedback, and scoring.
pub struct LessonPlayScreenPlugin;

impl Plugin for LessonPlayScreenPlugin {
    fn build(&self, app: &mut App) {
        app.register_state_scoped_resource::<ActiveLesson, LessonSession>(ActiveLesson)
            .register_state_scoped_resource::<ActiveLesson, SelectedLesson>(ActiveLesson)
            .register_state_scoped_resource::<ActiveLesson, LastAnswer>(ActiveLesson)
            .add_systems(OnEnter(AppState::LessonPlay), layout::setup_lesson_play)
            .add_systems(
                OnEnter(LessonPhase::ShowQuestion),
                (
                    clear_active_student_for_question,
                    update_progress_text,
                    transition_to_waiting,
                )
                    .chain(),
            )
            .add_systems(
                OnEnter(LessonPhase::ShowFeedback),
                (
                    cleanup_root::<QuestionRoot>,
                    record_answer,
                    explanations::setup_feedback_ui,
                )
                    .chain(),
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

/// Updates session score and records the answer to the active student in class mode.
/// Runs before `setup_feedback_ui` so the UI reads the updated session.
fn record_answer(
    last_answer: Res<LastAnswer>,
    mut session: ResMut<LessonSession>,
    mut persistence: PersistenceMut<'_>,
    active_student: Option<Res<ActiveStudent>>,
    active_slot: Option<Res<ActiveSlot>>,
    selected_lesson: Option<Res<SelectedLesson>>,
) {
    session::update_session_score(&mut session, &last_answer);

    if persistence.settings.mode == GameMode::Group {
        session::record_class_answer(
            &session,
            &last_answer,
            active_student.as_deref(),
            active_slot.as_deref(),
            selected_lesson.as_deref(),
            &mut persistence.save_data,
        );
    }
}

/// Clear `ActiveStudent` at the start of each question in class mode,
/// so the teacher must re-select a student before the next answer.
fn clear_active_student_for_question(
    mut commands: Commands,
    settings: Res<Persistent<GameSettings>>,
) {
    if settings.mode == GameMode::Group {
        commands.remove_resource::<ActiveStudent>();
    }
}

fn advance_question(
    mut session: ResMut<LessonSession>,
    mut next_phase: ResMut<NextState<LessonPhase>>,
    mut next_app_state: ResMut<NextState<AppState>>,
) {
    session.current_index += 1;
    if session.current_index < session.questions.len() {
        next_phase.set(LessonPhase::ShowQuestion);
    } else {
        next_app_state.set(AppState::LessonSummary);
    }
}

fn handle_quit_lesson(
    query: Query<&Interaction, (Changed<Interaction>, With<QuitLessonButton>)>,
    mut next_state: ResMut<NextState<AppState>>,
) {
    for interaction in &query {
        if *interaction == Interaction::Pressed {
            next_state.set(AppState::MapExploration);
        }
    }
}
