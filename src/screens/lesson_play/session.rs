use std::time::{SystemTime, UNIX_EPOCH};

use crate::data::{
    ActiveStudent, AnswerResult, GameMode, LessonSession, PersistenceAction, PersistenceMut,
    PlayerSession, SaveData, SelectedLesson,
};

pub(super) const fn update_session_score(session: &mut LessonSession, result: &AnswerResult) {
    let is_correct = matches!(result, AnswerResult::Correct);
    session.total_answered += 1;
    if is_correct {
        session.correct_count += 1;
    }
}

/// Records the answer in the active player's cumulative exercise progress.
pub(super) fn record_answer(
    session: &LessonSession,
    last_answer: &AnswerResult,
    mode: GameMode,
    active_student: Option<&ActiveStudent>,
    player_session: Option<&PlayerSession>,
    selected_lesson: Option<&SelectedLesson>,
    persistence: &mut PersistenceMut<'_>,
) {
    persistence.status.clear_save_data();
    let Some(player_session) = player_session else {
        return;
    };
    let Some(selected) = selected_lesson else {
        return;
    };
    let Some(lesson_id) = selected.0.as_deref() else {
        return;
    };
    let Some(question) = session.questions.get(session.current_index) else {
        return;
    };
    let student_index = active_student.map(|student| student.0);
    if !has_answer_recipient(
        &persistence.save_data,
        mode,
        player_session.slot_index,
        student_index,
    ) {
        return;
    }
    let Ok(answered_at) = SystemTime::now().duration_since(UNIX_EPOCH) else {
        bevy::log::error!(
            "Could not record answer because the system clock predates the UTC epoch"
        );
        persistence.status.fail(PersistenceAction::QuestionAnswer);
        return;
    };

    let lesson_id = lesson_id.to_owned();
    let exercise_id = question.exercise_id.clone();
    let slot_index = player_session.slot_index;
    let is_correct = matches!(last_answer, AnswerResult::Correct);

    let target = AnswerTarget {
        mode,
        slot_index,
        student_index,
    };
    let recorded = persistence.update_save_data(PersistenceAction::QuestionAnswer, |data| {
        apply_answer(
            data,
            target,
            &lesson_id,
            &exercise_id,
            is_correct,
            answered_at.as_secs(),
        )
    });
    if !recorded {
        bevy::log::error!(
            "Could not record answer because the selected player slot is unavailable"
        );
        persistence.status.fail(PersistenceAction::QuestionAnswer);
    }
}

fn has_answer_recipient(
    save_data: &SaveData,
    mode: GameMode,
    slot_index: usize,
    student_index: Option<usize>,
) -> bool {
    match mode {
        GameMode::Individual => save_data
            .individual_slots
            .get(slot_index)
            .is_some_and(Option::is_some),
        GameMode::Group => student_index
            .and_then(|index| {
                save_data
                    .class_slots
                    .get(slot_index)?
                    .as_ref()?
                    .students
                    .get(index)
            })
            .is_some(),
    }
}

#[derive(Clone, Copy)]
struct AnswerTarget {
    mode: GameMode,
    slot_index: usize,
    student_index: Option<usize>,
}

fn apply_answer(
    save_data: &mut SaveData,
    target: AnswerTarget,
    lesson_id: &str,
    exercise_id: &str,
    is_correct: bool,
    timestamp_unix_seconds: u64,
) -> bool {
    let progress = match target.mode {
        GameMode::Individual => {
            let Some(save) = save_data
                .individual_slots
                .get_mut(target.slot_index)
                .and_then(Option::as_mut)
            else {
                return false;
            };
            &mut save.progress
        }
        GameMode::Group => {
            let Some(student) = target.student_index.and_then(|index| {
                save_data
                    .class_slots
                    .get_mut(target.slot_index)?
                    .as_mut()?
                    .students
                    .get_mut(index)
            }) else {
                return false;
            };
            &mut student.progress
        }
    };
    progress
        .entry(lesson_id.to_owned())
        .or_default()
        .record_answer(exercise_id, is_correct, timestamp_unix_seconds);
    true
}
