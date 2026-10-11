use crate::data::{
    ActiveStudent, AnswerResult, LessonSession, PersistenceAction, PersistenceMut, PlayerSession,
    SelectedLesson,
};

pub(super) fn update_session_score(session: &mut LessonSession, result: &AnswerResult) {
    let is_correct = matches!(result, AnswerResult::Correct);
    session.total_answered += 1;
    if is_correct {
        session.correct_count += 1;
    }

    if let Some(question) = session.questions.get(session.current_index) {
        let question_type = question.instance.question_type();
        let entry = session.type_scores.entry(question_type).or_default();
        entry.total += 1;
        if is_correct {
            entry.correct += 1;
        }
    }
}

/// In class mode, persist the current answer's result to the active student's
/// cumulative progress. If no student is selected, the result is discarded.
pub(super) fn record_class_answer(
    session: &LessonSession,
    last_answer: &AnswerResult,
    active_student: Option<&ActiveStudent>,
    player_session: Option<&PlayerSession>,
    selected_lesson: Option<&SelectedLesson>,
    persistence: &mut PersistenceMut<'_>,
) {
    persistence.status.clear_save_data();
    let Some(student) = active_student else {
        return;
    };
    let Some(player_session) = player_session else {
        return;
    };
    let Some(selected) = selected_lesson else {
        return;
    };
    let Some(ref lesson_id) = selected.0 else {
        return;
    };
    let Some(question) = session.questions.get(session.current_index) else {
        return;
    };

    let is_correct = matches!(last_answer, AnswerResult::Correct);
    let question_type = question.instance.question_type();
    let lesson_id = lesson_id.clone();
    let slot_index = player_session.slot_index;
    let student_index = student.0;

    persistence.update_save_data(PersistenceAction::ClassAnswer, |data| {
        let Some(class_save) = data.class_slots[slot_index].as_mut() else {
            return;
        };
        let Some(student_data) = class_save.students.get_mut(student_index) else {
            return;
        };
        let lesson_progress = student_data.progress.entry(lesson_id.clone()).or_default();
        let type_score = lesson_progress
            .type_scores
            .entry(question_type)
            .or_default();
        type_score.total += 1;
        if is_correct {
            type_score.correct += 1;
        }
    });
}
