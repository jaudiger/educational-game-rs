use crate::data::content::types::{AnswerResult, ComparisonAnswer, QuestionInstance};

/// A response emitted by a question interaction, before correctness is known.
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum QuestionResponse {
    McqChoice(usize),
    FractionVisualization { colored_parts: u32 },
    FractionComparison(ComparisonAnswer),
    FractionIdentificationChoice(usize),
    NumericInput(u32),
}

/// Why a response cannot be graded against the active question.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum InvalidResponse {
    WrongResponseType,
    ChoiceIndexOutOfRange,
    ColoredPartsOutOfRange,
    NumericValueOutOfRange,
}

/// Grade a typed response against its concrete question instance.
pub fn assess_response(
    question: &QuestionInstance,
    response: &QuestionResponse,
) -> Result<AnswerResult, InvalidResponse> {
    let is_correct = match (question, response) {
        (QuestionInstance::Mcq(definition), QuestionResponse::McqChoice(index)) => {
            if *index >= definition.choices.len() {
                return Err(InvalidResponse::ChoiceIndexOutOfRange);
            }
            *index == definition.correct_index
        }
        (
            QuestionInstance::FractionVisualization(definition),
            QuestionResponse::FractionVisualization { colored_parts },
        ) => {
            if *colored_parts > definition.denominator {
                return Err(InvalidResponse::ColoredPartsOutOfRange);
            }
            *colored_parts == definition.numerator
        }
        (
            QuestionInstance::FractionComparison(definition),
            QuestionResponse::FractionComparison(answer),
        ) => *answer == definition.answer,
        (
            QuestionInstance::FractionIdentification(definition),
            QuestionResponse::FractionIdentificationChoice(index),
        ) => {
            if *index >= definition.choices.len() {
                return Err(InvalidResponse::ChoiceIndexOutOfRange);
            }
            *index == definition.correct_index
        }
        (QuestionInstance::NumericInput(definition), QuestionResponse::NumericInput(value)) => {
            if *value > crate::data::content::types::MAX_NUMERIC_INPUT_ANSWER {
                return Err(InvalidResponse::NumericValueOutOfRange);
            }
            *value == definition.correct_answer
        }
        _ => return Err(InvalidResponse::WrongResponseType),
    };

    Ok(if is_correct {
        AnswerResult::Correct
    } else {
        AnswerResult::Incorrect
    })
}
