use crate::data::content::QuestionType;
use crate::i18n::{I18n, TranslationKey};

pub fn activity_category_label(question_type: QuestionType, i18n: &I18n) -> String {
    let key = match question_type {
        QuestionType::Mcq => TranslationKey::ActivityCategoryMcq,
        QuestionType::Visualization => TranslationKey::ActivityCategoryVisualization,
        QuestionType::Comparison => TranslationKey::ActivityCategoryComparison,
        QuestionType::Identification => TranslationKey::ActivityCategoryIdentification,
        QuestionType::NumericInput => TranslationKey::ActivityCategoryNumericInput,
    };
    i18n.t(&key).into_owned()
}
