use rand::Rng;

use super::types::{QuestionDefinition, QuestionInstance};

/// Resolve one catalog definition into a concrete question instance.
pub(super) fn resolve_question(
    definition: &QuestionDefinition,
    rng: &mut impl Rng,
) -> QuestionInstance {
    match definition {
        QuestionDefinition::Mcq(value) => QuestionInstance::Mcq(value.clone()),
        QuestionDefinition::McqTemplate(template) => QuestionInstance::Mcq(template.resolve(rng)),
        QuestionDefinition::FractionVisualization(value) => {
            QuestionInstance::FractionVisualization(value.clone())
        }
        QuestionDefinition::FractionVisualizationTemplate(template) => {
            QuestionInstance::FractionVisualization(template.resolve(rng))
        }
        QuestionDefinition::FractionComparison(value) => {
            QuestionInstance::FractionComparison(value.clone())
        }
        QuestionDefinition::FractionComparisonTemplate(template) => {
            QuestionInstance::FractionComparison(template.resolve(rng))
        }
        QuestionDefinition::FractionIdentification(value) => {
            QuestionInstance::FractionIdentification(value.clone())
        }
        QuestionDefinition::FractionIdentificationTemplate(template) => {
            QuestionInstance::FractionIdentification(template.resolve(rng))
        }
        QuestionDefinition::NumericInput(value) => QuestionInstance::NumericInput(value.clone()),
        QuestionDefinition::NumericInputTemplate(template) => {
            QuestionInstance::NumericInput(template.resolve(rng))
        }
    }
}
