use crate::data::content::QuestionType;
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

    let lesson_id = lesson_id.to_owned();
    let exercise_id = question.exercise_id.clone();
    let question_type = question.instance.question_type();
    let slot_index = player_session.slot_index;
    let is_correct = matches!(last_answer, AnswerResult::Correct);

    let target = AnswerTarget {
        mode,
        slot_index,
        student_index,
    };
    persistence.update_save_data(PersistenceAction::QuestionAnswer, |data| {
        let _ = apply_answer(
            data,
            target,
            &lesson_id,
            &exercise_id,
            question_type,
            is_correct,
        );
    });
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
    question_type: QuestionType,
    is_correct: bool,
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
        .record_answer(exercise_id, question_type, is_correct);
    true
}
