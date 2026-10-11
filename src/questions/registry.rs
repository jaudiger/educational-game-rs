use bevy::ecs::system::ScheduleSystem;
use bevy::prelude::*;

use crate::data::{
    AnswerResult, LastAnswer, LessonSession, QuestionInstance, QuestionResponse, assess_response,
};
use crate::states::LessonPhase;

/// Buffered typed response emitted by a question interaction.
#[derive(Message, Clone, Debug)]
pub struct QuestionResponseSubmitted(pub QuestionResponse);

/// Fired after central assessment determines correctness.
#[derive(Event, Clone, Debug)]
pub struct AnswerSubmitted {
    pub result: AnswerResult,
}

/// Shared helper for input plugins to submit a typed response.
pub fn submit_response(
    writer: &mut MessageWriter<QuestionResponseSubmitted>,
    response: QuestionResponse,
) {
    writer.write(QuestionResponseSubmitted(response));
}

/// Record an assessed result and transition to feedback.
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
#[derive(Component, Reflect)]
pub struct QuestionRoot;

/// Orders input handlers before central assessment.
#[derive(SystemSet, Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub struct QuestionInteractionSet;

/// Registers the spawn and interaction systems for a question type.
pub fn register_question_systems<SpawnM, HandleM>(
    app: &mut App,
    predicate: fn(&QuestionInstance) -> bool,
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
            .run_if(is_current_question(predicate))
            .in_set(QuestionInteractionSet),
    );
}

/// Factory for run conditions that check the current concrete question variant.
pub fn is_current_question(
    predicate: fn(&QuestionInstance) -> bool,
) -> impl Fn(Option<Res<LessonSession>>) -> bool {
    move |session: Option<Res<LessonSession>>| {
        session.is_some_and(|session| {
            session
                .questions
                .get(session.current_index)
                .is_some_and(|question| predicate(&question.instance))
        })
    }
}

/// Assess a response against the current question and enter feedback on success.
pub fn assess_question_responses(
    mut responses: MessageReader<QuestionResponseSubmitted>,
    session: Option<Res<LessonSession>>,
    mut commands: Commands,
    mut next_phase: ResMut<NextState<LessonPhase>>,
) {
    let Some(session) = session else {
        return;
    };
    let Some(question) = session.current() else {
        return;
    };

    for response in responses.read() {
        match assess_response(&question.instance, &response.0) {
            Ok(result) => {
                submit_answer(&mut commands, &mut next_phase, result);
                return;
            }
            Err(issue) => bevy::log::warn!(
                "Ignored invalid response for exercise {}: {issue:?}",
                question.exercise_id
            ),
        }
    }
}
