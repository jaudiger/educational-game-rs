use bevy::ecs::system::ScheduleSystem;
use bevy::prelude::*;

use crate::data::{AnswerResult, LastAnswer, LessonSession, QuestionDefinition};
use crate::states::LessonPhase;

/// Fired when the student submits an answer, carrying the result.
#[derive(Event, Clone, Debug)]
pub struct AnswerSubmitted {
    pub result: AnswerResult,
}

/// Shared helper for question plugins to submit an answer.
///
/// Inserts the `LastAnswer` resource, triggers the `AnswerSubmitted` event,
/// and transitions to `LessonPhase::ShowFeedback`.
pub fn submit_answer(
    commands: &mut Commands,
    next_phase: &mut ResMut<NextState<LessonPhase>>,
    result: AnswerResult,
) {
    commands.insert_resource(LastAnswer(result.clone()));
    commands.trigger(AnswerSubmitted { result });
    next_phase.set(LessonPhase::ShowFeedback);
}

/// Shared marker on every question UI root entity.
/// Queried via `cleanup_root::<QuestionRoot>` when feedback starts.
#[derive(Component, Reflect)]
pub struct QuestionRoot;

/// Registers the spawn and interaction systems for a question type.
///
/// Applies `is_current_question(predicate)` to both systems. Cleanup runs
/// centrally in `LessonPlayScreenPlugin` via `cleanup_root::<QuestionRoot>`.
pub fn register_question_systems<SpawnM, HandleM>(
    app: &mut App,
    predicate: fn(&QuestionDefinition) -> bool,
    spawn: impl IntoScheduleConfigs<ScheduleSystem, SpawnM>,
    handle: impl IntoScheduleConfigs<ScheduleSystem, HandleM>,
) {
    app.add_systems(
        OnEnter(LessonPhase::ShowQuestion),
        spawn.run_if(is_current_question(predicate)),
    )
    .add_systems(
        Update,
        handle
            .run_if(in_state(LessonPhase::WaitingAnswer))
            .run_if(is_current_question(predicate)),
    );
}

/// Factory for Bevy run-conditions that check the current question variant.
///
/// Returns a system-compatible closure usable with `.run_if(...)`.
/// Adding a new question type only requires a new
/// `is_current_question(|d| matches!(d, QuestionDefinition::NewVariant(_)))` call.
pub fn is_current_question(
    predicate: fn(&QuestionDefinition) -> bool,
) -> impl Fn(Option<Res<LessonSession>>) -> bool {
    move |session: Option<Res<LessonSession>>| {
        session.is_some_and(|s| {
            s.questions
                .get(s.current_index)
                .is_some_and(|q| predicate(&q.definition))
        })
    }
}
