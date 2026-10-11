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

#[cfg(test)]
mod tests {
    use super::{InvalidResponse, QuestionResponse, assess_response};
    use crate::data::content::types::{
        AnswerResult, ComparisonAnswer, FractionComparisonDefinition,
        FractionIdentificationDefinition, FractionVisualizationDefinition, LocalizedExplanation,
        LocalizedText, McqDefinition, NumericInputDefinition, QuestionInstance,
    };

    fn explanation() -> LocalizedExplanation {
        LocalizedExplanation::plain(LocalizedText::new("", ""))
    }

    #[test]
    fn typed_responses_are_graded_by_their_question_instance() {
        let mcq = QuestionInstance::Mcq(McqDefinition {
            prompt: LocalizedText::new("prompt", "prompt"),
            choices: vec!["wrong".to_owned(), "right".to_owned()],
            correct_index: 1,
            explanation: explanation(),
            explanation_visual: None,
            question_visual: None,
        });
        assert_eq!(
            assess_response(&mcq, &QuestionResponse::McqChoice(1)),
            Ok(AnswerResult::Correct)
        );
        assert_eq!(
            assess_response(&mcq, &QuestionResponse::McqChoice(0)),
            Ok(AnswerResult::Incorrect)
        );

        let comparison = QuestionInstance::FractionComparison(FractionComparisonDefinition {
            prompt: LocalizedText::new("prompt", "prompt"),
            character_a: "A".into(),
            fraction_a: (1, 2),
            character_b: "B".into(),
            fraction_b: (1, 3),
            answer: ComparisonAnswer::A,
            difficulty: crate::data::content::types::ComparisonDifficulty::SameNumerator,
            explanation: explanation(),
            explanation_visual: None,
        });
        assert_eq!(
            assess_response(
                &comparison,
                &QuestionResponse::FractionComparison(ComparisonAnswer::A),
            ),
            Ok(AnswerResult::Correct)
        );

        let numeric = QuestionInstance::NumericInput(NumericInputDefinition {
            prompt: LocalizedText::new("prompt", "prompt"),
            correct_answer: 0,
            explanation: explanation(),
            explanation_visual: None,
            question_visual: None,
        });
        assert_eq!(
            assess_response(&numeric, &QuestionResponse::NumericInput(0)),
            Ok(AnswerResult::Correct)
        );
    }

    #[test]
    fn fraction_inputs_keep_their_existing_correctness_rules() {
        let explanation = explanation();
        let visualization =
            QuestionInstance::FractionVisualization(FractionVisualizationDefinition {
                prompt: LocalizedText::new("prompt", "prompt"),
                numerator: 2,
                denominator: 4,
                explanation: explanation.clone(),
            });
        assert_eq!(
            assess_response(
                &visualization,
                &QuestionResponse::FractionVisualization { colored_parts: 2 },
            ),
            Ok(AnswerResult::Correct)
        );
        assert_eq!(
            assess_response(
                &visualization,
                &QuestionResponse::FractionVisualization { colored_parts: 1 },
            ),
            Ok(AnswerResult::Incorrect)
        );

        let identification =
            QuestionInstance::FractionIdentification(FractionIdentificationDefinition {
                numerator: 2,
                denominator: 4,
                choices: vec!["1/2".to_owned(), "2/4".to_owned()],
                correct_index: 1,
                explanation,
                explanation_visual: None,
            });
        assert_eq!(
            assess_response(
                &identification,
                &QuestionResponse::FractionIdentificationChoice(1),
            ),
            Ok(AnswerResult::Correct)
        );
        assert_eq!(
            assess_response(
                &identification,
                &QuestionResponse::FractionIdentificationChoice(0),
            ),
            Ok(AnswerResult::Incorrect)
        );
    }

    #[test]
    fn invalid_responses_do_not_become_incorrect_answers() {
        let question = QuestionInstance::FractionVisualization(FractionVisualizationDefinition {
            prompt: LocalizedText::new("prompt", "prompt"),
            numerator: 1,
            denominator: 2,
            explanation: explanation(),
        });
        assert_eq!(
            assess_response(&question, &QuestionResponse::McqChoice(0)),
            Err(InvalidResponse::WrongResponseType)
        );
        assert_eq!(
            assess_response(
                &question,
                &QuestionResponse::FractionVisualization { colored_parts: 3 },
            ),
            Err(InvalidResponse::ColoredPartsOutOfRange)
        );

        let numeric = QuestionInstance::NumericInput(NumericInputDefinition {
            prompt: LocalizedText::new("prompt", "prompt"),
            correct_answer: 1,
            explanation: explanation(),
            explanation_visual: None,
            question_visual: None,
        });
        assert_eq!(
            assess_response(
                &numeric,
                &QuestionResponse::NumericInput(
                    crate::data::content::types::MAX_NUMERIC_INPUT_ANSWER + 1
                ),
            ),
            Err(InvalidResponse::NumericValueOutOfRange)
        );

        let mcq = QuestionInstance::Mcq(McqDefinition {
            prompt: LocalizedText::new("prompt", "prompt"),
            choices: vec!["one".to_owned(), "two".to_owned()],
            correct_index: 0,
            explanation: explanation(),
            explanation_visual: None,
            question_visual: None,
        });
        assert_eq!(
            assess_response(&mcq, &QuestionResponse::McqChoice(2)),
            Err(InvalidResponse::ChoiceIndexOutOfRange)
        );
    }
}
