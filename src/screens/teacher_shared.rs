use crate::data::content::QuestionType;
use crate::i18n::{I18n, TranslationKey};

/// Translates a question type to a label in the active language.
pub fn question_type_label(qt: QuestionType, i18n: &I18n) -> String {
    let key = match qt {
        QuestionType::Mcq => TranslationKey::QuestionTypeMcq,
        QuestionType::Visualization => TranslationKey::QuestionTypeVisualization,
        QuestionType::Comparison => TranslationKey::QuestionTypeComparison,
        QuestionType::Identification => TranslationKey::QuestionTypeIdentification,
        QuestionType::NumericInput => TranslationKey::QuestionTypeNumericInput,
    };
    i18n.t(&key).into_owned()
}
