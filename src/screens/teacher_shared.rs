use crate::data::content::QuestionType;
use crate::i18n::{I18n, TranslationKey};

/// Translates an exercise type to a label in the active language.
pub fn exercise_type_label(exercise_type: QuestionType, i18n: &I18n) -> String {
    let key = match exercise_type {
        QuestionType::Mcq => TranslationKey::ExerciseTypeMcq,
        QuestionType::Visualization => TranslationKey::ExerciseTypeVisualization,
        QuestionType::Comparison => TranslationKey::ExerciseTypeComparison,
        QuestionType::Identification => TranslationKey::ExerciseTypeIdentification,
        QuestionType::NumericInput => TranslationKey::ExerciseTypeNumericInput,
    };
    i18n.t(&key).into_owned()
}
