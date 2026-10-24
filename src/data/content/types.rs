use std::borrow::Cow;
use std::collections::{HashMap, HashSet};
use std::hash::Hash;
use std::hash::{DefaultHasher, Hasher};

use bevy::prelude::*;
use serde::{Deserialize, Serialize};

use crate::i18n::Language;

/// Maximum number of times a single question can appear in a teacher-configured session.
pub const MAX_QUESTION_REPETITIONS: usize = 5;
pub const MAX_FRACTION_PARTS: u32 = 1_000;
pub const MAX_MULTIPLICATION_GRID_CELLS: u32 = 10_000;
pub const MAX_NUMERIC_INPUT_ANSWER: u32 = 9_999_999;
pub const MAX_PLACE_VALUE_COLUMNS: usize = 5;
pub const MAX_CONTENT_RANGE_VALUES: usize = 100;
pub const MAX_TEMPLATE_CANDIDATES: usize = 10_000;
pub const MAX_LESSON_EXERCISES: usize = 1_000;

/// Returns whether the place-value renderer can display this multiplication.
pub fn place_value_table_fits(number: u32, multiplier: u32) -> bool {
    if multiplier == 0 {
        return false;
    }
    let Some(result) = number.checked_mul(multiplier) else {
        return false;
    };
    decimal_digit_count(number).max(decimal_digit_count(result)) <= MAX_PLACE_VALUE_COLUMNS
}

const fn decimal_digit_count(mut value: u32) -> usize {
    let mut digits = 1;
    while value >= 10 {
        value /= 10;
        digits += 1;
    }
    digits
}

pub(super) fn fraction_value_pair_is_safe(a: i32, b: i32) -> bool {
    if a <= 0 || b <= 0 || a % b != 0 || b > MAX_FRACTION_PARTS.cast_signed() {
        return false;
    }
    let result = a / b;
    result <= i32::MAX / 2
        && result.checked_mul(b).is_some_and(|parts| {
            parts > 0 && parts.cast_unsigned() <= MAX_MULTIPLICATION_GRID_CELLS
        })
}

/// A plain text string available in both French and English.
/// Used for prompts and as localized prose within structured explanations.
#[derive(Clone, Debug, PartialEq, Eq, Hash, Deserialize, Serialize)]
#[serde(deny_unknown_fields)]
pub struct LocalizedText {
    pub fr: Cow<'static, str>,
    pub en: Cow<'static, str>,
}

impl LocalizedText {
    /// Creates a `LocalizedText` from French and English strings.
    pub fn new(fr: impl Into<Cow<'static, str>>, en: impl Into<Cow<'static, str>>) -> Self {
        Self {
            fr: fr.into(),
            en: en.into(),
        }
    }

    /// Resolve to the active language.
    pub fn get(&self, language: Language) -> &str {
        match language {
            Language::French => &self.fr,
            Language::English => &self.en,
        }
    }
}

/// Semantic color used by values embedded in localized explanations.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash, Deserialize, Serialize)]
#[serde(deny_unknown_fields)]
pub enum ExplanationColorRole {
    Default,
    Primary,
    Secondary,
    Success,
    PlaceValue { highlighted_zeros: usize },
}

/// A named value inserted at an authored explanation placeholder.
#[derive(Clone, Debug, PartialEq, Eq, Hash, Deserialize, Serialize)]
#[serde(deny_unknown_fields)]
pub struct ExplanationPlaceholder {
    pub name: Cow<'static, str>,
    pub value: ExplanationValue,
}

impl ExplanationPlaceholder {
    pub fn new(name: impl Into<Cow<'static, str>>, value: ExplanationValue) -> Self {
        Self {
            name: name.into(),
            value,
        }
    }
}

/// Content bound to a named explanation placeholder.
#[derive(Clone, Debug, PartialEq, Eq, Hash, Deserialize, Serialize)]
#[serde(deny_unknown_fields)]
pub enum ExplanationValue {
    Text(LocalizedText),
    Number {
        value: u32,
        role: ExplanationColorRole,
    },
    Fraction {
        numerator: u32,
        denominator: u32,
        numerator_role: ExplanationColorRole,
        denominator_role: ExplanationColorRole,
    },
}

/// Localized explanation prose with explicit named semantic values.
#[derive(Clone, Debug, PartialEq, Eq, Hash, Deserialize, Serialize)]
#[serde(deny_unknown_fields)]
pub struct LocalizedExplanation {
    pub text: LocalizedText,
    pub placeholders: Vec<ExplanationPlaceholder>,
}

impl LocalizedExplanation {
    pub const fn new(text: LocalizedText, placeholders: Vec<ExplanationPlaceholder>) -> Self {
        Self { text, placeholders }
    }

    pub const fn plain(text: LocalizedText) -> Self {
        Self::new(text, Vec::new())
    }

    pub fn validation_issues(&self) -> Vec<String> {
        let mut issues = Vec::new();
        let mut binding_names = Vec::new();
        for placeholder in &self.placeholders {
            let name = placeholder.name.as_ref();
            if let ExplanationValue::Fraction { denominator: 0, .. } = &placeholder.value {
                issues.push(format!(
                    "Explanation fraction binding {name} must have a non-zero denominator"
                ));
            }
            if !is_valid_placeholder_name(name) {
                issues.push(format!("Invalid explanation binding name: {name}"));
            }
            if binding_names.contains(&name) {
                issues.push(format!("Duplicate explanation binding name: {name}"));
            } else {
                binding_names.push(name);
            }
        }

        for (language, text) in [("French", &self.text.fr), ("English", &self.text.en)] {
            for (name, issue) in scan_placeholder_names(text) {
                if let Some(issue) = issue {
                    issues.push(format!("{language} explanation {issue}"));
                    continue;
                }
                let binding_count = self
                    .placeholders
                    .iter()
                    .filter(|placeholder| placeholder.name == name)
                    .count();
                if binding_count == 0 {
                    issues.push(format!(
                        "{language} explanation has unknown placeholder: {name}"
                    ));
                } else if binding_count > 1 {
                    issues.push(format!(
                        "{language} explanation has ambiguous placeholder: {name}"
                    ));
                }
            }
        }
        issues
    }
}

fn is_valid_placeholder_name(name: &str) -> bool {
    let mut bytes = name.bytes();
    bytes
        .next()
        .is_some_and(|first| first.is_ascii_alphabetic() || first == b'_')
        && bytes.all(|byte| byte.is_ascii_alphanumeric() || byte == b'_')
}

fn scan_placeholder_names(text: &str) -> Vec<(String, Option<String>)> {
    let mut found = Vec::new();
    let bytes = text.as_bytes();
    let mut position = 0;
    while position < bytes.len() {
        match bytes[position] {
            b'{' => {
                let Some(offset) = bytes[position + 1..].iter().position(|byte| *byte == b'}')
                else {
                    found.push((
                        String::new(),
                        Some("has an unclosed placeholder".to_owned()),
                    ));
                    break;
                };
                let end = position + 1 + offset;
                let name = &text[position + 1..end];
                if is_valid_placeholder_name(name) {
                    found.push((name.to_owned(), None));
                } else {
                    found.push((
                        String::new(),
                        Some("has a malformed placeholder".to_owned()),
                    ));
                }
                position = end + 1;
            }
            b'}' => {
                found.push((
                    String::new(),
                    Some("has an unmatched closing brace".to_owned()),
                ));
                position += 1;
            }
            _ => position += 1,
        }
    }
    found
}

impl From<LocalizedText> for LocalizedExplanation {
    fn from(text: LocalizedText) -> Self {
        Self::plain(text)
    }
}

/// Top-level resource holding all themes and their lessons.
#[derive(Resource, Debug, Default, Reflect, Deserialize, Serialize)]
#[serde(deny_unknown_fields)]
pub struct ContentLibrary {
    #[reflect(ignore)]
    pub themes: Vec<Theme>,
    #[serde(skip)]
    #[reflect(ignore)]
    pub(crate) theme_index: HashMap<String, usize>,
}

impl ContentLibrary {
    /// Build index maps from current `Vec` contents. Call after construction or deserialization.
    pub fn build_indexes(&mut self) {
        self.theme_index = self
            .themes
            .iter()
            .enumerate()
            .map(|(i, t)| (t.id.clone(), i))
            .collect();
        for theme in &mut self.themes {
            theme.build_lesson_index();
        }
    }

    /// O(1) theme lookup by ID.
    pub fn theme(&self, id: &str) -> Option<&Theme> {
        self.theme_index.get(id).map(|&i| &self.themes[i])
    }

    pub fn validation_issues(&self) -> Vec<String> {
        let mut issues = Vec::new();
        let mut theme_ids = HashSet::new();
        let mut lesson_ids = HashSet::new();
        for theme in &self.themes {
            if theme.id.is_empty() || !theme_ids.insert(theme.id.as_str()) {
                issues.push(format!("Invalid or duplicate theme ID: {}", theme.id));
            }
            if theme.available && theme.lessons.is_empty() {
                issues.push(format!("Available theme {} has no lessons", theme.id));
            }
            for lesson in &theme.lessons {
                if lesson.id.is_empty() || !lesson_ids.insert(lesson.id.as_str()) {
                    issues.push(format!(
                        "Invalid or duplicate global lesson ID in theme {}: {}",
                        theme.id, lesson.id
                    ));
                }
                issues.extend(
                    lesson
                        .validation_issues()
                        .into_iter()
                        .map(|issue| format!("Theme {}, lesson {}: {issue}", theme.id, lesson.id)),
                );
            }
        }
        issues
    }
}

/// A pedagogical theme grouping related lessons.
#[derive(Clone, Debug, Deserialize, Serialize)]
#[serde(deny_unknown_fields)]
pub struct Theme {
    pub id: String,
    pub title: LocalizedText,
    #[serde(default)]
    pub available: bool,
    #[serde(default)]
    pub lessons: Vec<Lesson>,
    #[serde(skip)]
    pub(crate) lesson_index: HashMap<String, usize>,
}

impl Theme {
    fn build_lesson_index(&mut self) {
        self.lesson_index = self
            .lessons
            .iter()
            .enumerate()
            .map(|(i, l)| (l.id.clone(), i))
            .collect();
    }

    /// O(1) lesson lookup by ID.
    pub fn lesson(&self, id: &str) -> Option<&Lesson> {
        self.lesson_index.get(id).map(|&i| &self.lessons[i])
    }
}

/// The French school level targeted by an activity.
#[derive(Clone, Copy, Debug, Eq, PartialEq, Hash, Reflect, Deserialize, Serialize)]
#[serde(rename_all = "UPPERCASE")]
pub enum SchoolLevel {
    Cp,
    Ce1,
    Ce2,
    Cm1,
    Cm2,
}

impl SchoolLevel {
    pub const fn label(self) -> &'static str {
        match self {
            Self::Cp => "CP",
            Self::Ce1 => "CE1",
            Self::Ce2 => "CE2",
            Self::Cm1 => "CM1",
            Self::Cm2 => "CM2",
        }
    }
}

/// A single lesson within a theme, containing question pools.
#[derive(Clone, Debug, Deserialize, Serialize)]
#[serde(deny_unknown_fields)]
pub struct Lesson {
    pub id: String,
    pub title: LocalizedText,
    pub questions: Vec<QuestionEntry>,
}

#[derive(Clone, Debug, Deserialize, Serialize)]
#[serde(deny_unknown_fields)]
pub struct QuestionEntry {
    pub id: String,
    pub catalog_label: LocalizedText,
    pub school_level: SchoolLevel,
    pub definition: QuestionDefinition,
}

impl QuestionEntry {
    pub fn new(
        id: impl Into<String>,
        catalog_label: LocalizedText,
        school_level: SchoolLevel,
        definition: QuestionDefinition,
    ) -> Self {
        Self {
            id: id.into(),
            catalog_label,
            school_level,
            definition,
        }
    }
}

impl Lesson {
    pub fn validation_issues(&self) -> Vec<String> {
        let mut issues = Vec::new();
        if self.questions.is_empty() {
            issues.push("Lesson must contain at least one exercise".to_owned());
        }
        if self.questions.len() > MAX_LESSON_EXERCISES {
            issues.push(format!(
                "Lesson exceeds the {MAX_LESSON_EXERCISES} exercise limit"
            ));
        }
        let mut ids = HashSet::new();
        for entry in &self.questions {
            if entry.id.is_empty() || !ids.insert(entry.id.as_str()) {
                issues.push(format!("Invalid or duplicate exercise ID: {}", entry.id));
            }
            for (language, label) in [
                ("French", entry.catalog_label.fr.as_ref()),
                ("English", entry.catalog_label.en.as_ref()),
            ] {
                if label.trim().is_empty() {
                    issues.push(format!(
                        "Exercise {} has an empty {language} catalog label",
                        entry.id
                    ));
                }
                if !scan_placeholder_names(label).is_empty() {
                    issues.push(format!(
                        "Exercise {} has placeholders in its {language} catalog label",
                        entry.id
                    ));
                }
            }
            issues.extend(
                entry
                    .definition
                    .validation_issues()
                    .into_iter()
                    .map(|issue| format!("Exercise {}: {issue}", entry.id)),
            );
        }
        issues
    }
}

/// Classifies a question by its pedagogical type, regardless of whether it
/// originates from a static definition or a template.
#[derive(Clone, Copy, Debug, Eq, PartialEq, Hash, Reflect, Deserialize, Serialize)]
#[serde(deny_unknown_fields)]
pub enum QuestionType {
    Mcq,
    Visualization,
    Comparison,
    Identification,
    NumericInput,
}

/// Tagged union of all question variants, both static definitions and templates.
#[derive(Clone, Debug, Deserialize, Serialize)]
#[serde(tag = "type", deny_unknown_fields)]
pub enum QuestionDefinition {
    Mcq(McqDefinition),
    McqTemplate(McqTemplate),
    FractionVisualization(FractionVisualizationDefinition),
    FractionVisualizationTemplate(FractionVisualizationTemplate),
    FractionComparison(FractionComparisonDefinition),
    FractionComparisonTemplate(FractionComparisonTemplate),
    FractionIdentification(FractionIdentificationDefinition),
    FractionIdentificationTemplate(FractionIdentificationTemplate),
    NumericInput(NumericInputDefinition),
    NumericInputTemplate(NumericInputTemplate),
}

/// A fully generated question instance ready for interaction and presentation.
#[derive(Clone, Debug, PartialEq, Eq, Hash)]
pub enum QuestionInstance {
    Mcq(McqDefinition),
    FractionVisualization(FractionVisualizationDefinition),
    FractionComparison(FractionComparisonDefinition),
    FractionIdentification(FractionIdentificationDefinition),
    NumericInput(NumericInputDefinition),
}

impl QuestionInstance {
    pub const fn question_type(&self) -> QuestionType {
        match self {
            Self::Mcq(_) => QuestionType::Mcq,
            Self::FractionVisualization(_) => QuestionType::Visualization,
            Self::FractionComparison(_) => QuestionType::Comparison,
            Self::FractionIdentification(_) => QuestionType::Identification,
            Self::NumericInput(_) => QuestionType::NumericInput,
        }
    }

    pub fn fingerprint(&self) -> Option<u64> {
        let mut hasher = DefaultHasher::new();
        self.question_type().hash(&mut hasher);
        match self {
            Self::Mcq(definition) => {
                definition.prompt.hash(&mut hasher);
                definition
                    .choices
                    .get(definition.correct_index)?
                    .hash(&mut hasher);
            }
            Self::FractionVisualization(definition) => {
                definition.prompt.hash(&mut hasher);
                definition.numerator.hash(&mut hasher);
                definition.denominator.hash(&mut hasher);
                definition.shape.hash(&mut hasher);
            }
            Self::FractionComparison(definition) => {
                definition.prompt.hash(&mut hasher);
                definition.fraction_a.hash(&mut hasher);
                definition.fraction_b.hash(&mut hasher);
                definition.answer.hash(&mut hasher);
            }
            Self::FractionIdentification(definition) => {
                definition.numerator.hash(&mut hasher);
                definition.denominator.hash(&mut hasher);
                definition.mode.hash(&mut hasher);
                definition
                    .choices
                    .get(definition.correct_index)?
                    .hash(&mut hasher);
            }
            Self::NumericInput(definition) => {
                definition.prompt.hash(&mut hasher);
                definition.correct_answer.hash(&mut hasher);
            }
        }
        Some(hasher.finish())
    }
}

impl QuestionDefinition {
    #[allow(clippy::too_many_lines)]
    pub fn validation_issues(&self) -> Vec<String> {
        let mut issues = Vec::new();
        match self {
            Self::Mcq(definition) => {
                validate_choices(&definition.choices, definition.correct_index, &mut issues);
                validate_question_visual(definition.question_visual.as_ref(), &mut issues);
                validate_explanation_visual(definition.explanation_visual.as_ref(), &mut issues);
                issues.extend(definition.explanation.validation_issues());
            }
            Self::McqTemplate(template) => {
                validate_mcq_template(template, &mut issues);
                let (prompt_names, explanation_names) = match template.resolver {
                    McqResolver::FractionValue => {
                        (&["a", "b"][..], &["a", "b", "result", "fraction"][..])
                    }
                    McqResolver::FractionAddition => (
                        &["a", "b", "c"][..],
                        &["a", "b", "c", "sum", "fraction_a", "fraction_c", "result"][..],
                    ),
                    McqResolver::FractionName => (&["name"][..], &["name", "d", "fraction"][..]),
                    McqResolver::Multiplication | McqResolver::MultiplyByPowerOf10 => {
                        (&["a", "b"][..], &["a", "b", "result"][..])
                    }
                };
                validate_text_placeholders(
                    &template.prompt_template,
                    prompt_names,
                    "prompt",
                    &mut issues,
                );
                validate_text_placeholders(
                    &template.explanation_template,
                    explanation_names,
                    "explanation",
                    &mut issues,
                );
            }
            Self::FractionVisualization(definition) => {
                validate_fraction(
                    definition.numerator,
                    definition.denominator,
                    true,
                    &mut issues,
                );
                issues.extend(definition.explanation.validation_issues());
            }
            Self::FractionVisualizationTemplate(template) => {
                validate_text_placeholders(
                    &template.prompt_template,
                    &["n", "d"],
                    "prompt",
                    &mut issues,
                );
                validate_text_placeholders(
                    &template.explanation_template,
                    &["d", "n", "fraction"],
                    "explanation",
                    &mut issues,
                );
                validate_fraction_ranges(
                    &template.numerator_range,
                    &template.denominator_range,
                    true,
                    &mut issues,
                );
            }
            Self::FractionComparison(definition) => {
                validate_fraction(
                    definition.fraction_a.0,
                    definition.fraction_a.1,
                    true,
                    &mut issues,
                );
                validate_fraction(
                    definition.fraction_b.0,
                    definition.fraction_b.1,
                    true,
                    &mut issues,
                );
                validate_comparison_answer(
                    definition.fraction_a,
                    definition.fraction_b,
                    definition.answer,
                    &mut issues,
                );
                validate_explanation_visual(definition.explanation_visual.as_ref(), &mut issues);
                issues.extend(definition.explanation.validation_issues());
            }
            Self::FractionComparisonTemplate(template) => {
                validate_comparison_template(template, &mut issues);
                validate_text_placeholders(&template.prompt, &[], "prompt", &mut issues);
                validate_text_placeholders(
                    &template.explanation_template,
                    &[
                        "na",
                        "da",
                        "nb",
                        "db",
                        "fraction_a",
                        "fraction_b",
                        "greater_fraction",
                        "lesser_fraction",
                        "char_a",
                        "char_b",
                    ],
                    "explanation",
                    &mut issues,
                );
            }
            Self::FractionIdentification(definition) => {
                validate_fraction(
                    definition.numerator,
                    definition.denominator,
                    false,
                    &mut issues,
                );
                validate_choices(&definition.choices, definition.correct_index, &mut issues);
                issues.extend(definition.explanation.validation_issues());
            }
            Self::FractionIdentificationTemplate(template) => {
                validate_fraction_ranges(
                    &template.numerator_range,
                    &template.denominator_range,
                    false,
                    &mut issues,
                );
            }
            Self::NumericInput(definition) => {
                if definition.correct_answer > MAX_NUMERIC_INPUT_ANSWER {
                    issues.push(format!(
                        "Numeric answer exceeds keypad limit {MAX_NUMERIC_INPUT_ANSWER}"
                    ));
                }
                validate_question_visual(definition.question_visual.as_ref(), &mut issues);
                validate_explanation_visual(definition.explanation_visual.as_ref(), &mut issues);
                issues.extend(definition.explanation.validation_issues());
            }
            Self::NumericInputTemplate(template) => {
                validate_numeric_template(template, &mut issues);
                validate_text_placeholders(
                    &template.prompt_template,
                    &["a", "b"],
                    "prompt",
                    &mut issues,
                );
                validate_text_placeholders(
                    &template.explanation_template,
                    &["a", "b", "result"],
                    "explanation",
                    &mut issues,
                );
            }
        }
        issues
    }

    /// Returns the pedagogical category regardless of static vs template origin.
    pub const fn question_type(&self) -> QuestionType {
        match self {
            Self::Mcq(_) | Self::McqTemplate(_) => QuestionType::Mcq,
            Self::FractionVisualization(_) | Self::FractionVisualizationTemplate(_) => {
                QuestionType::Visualization
            }
            Self::FractionComparison(_) | Self::FractionComparisonTemplate(_) => {
                QuestionType::Comparison
            }
            Self::FractionIdentification(_) | Self::FractionIdentificationTemplate(_) => {
                QuestionType::Identification
            }
            Self::NumericInput(_) | Self::NumericInputTemplate(_) => QuestionType::NumericInput,
        }
    }

    /// Returns the detailed label used by teacher tools.
    #[allow(dead_code)]
    pub fn prompt_label(&self, language: Language) -> String {
        match self {
            Self::Mcq(d) => d.prompt.get(language).to_owned(),
            Self::McqTemplate(t) => t
                .teacher_label
                .as_ref()
                .unwrap_or(&t.prompt_template)
                .get(language)
                .to_owned(),
            Self::FractionVisualization(d) => d.prompt.get(language).to_owned(),
            Self::FractionVisualizationTemplate(t) => t.prompt_template.get(language).to_owned(),
            Self::FractionComparison(d) => d.prompt.get(language).to_owned(),
            Self::FractionComparisonTemplate(t) => match t.mode {
                ComparisonMode::SameDenominator => match language {
                    Language::French => "Même dénominateur",
                    Language::English => "Same denominator",
                },
                ComparisonMode::SameNumerator => match language {
                    Language::French => "Même numérateur",
                    Language::English => "Same numerator",
                },
                ComparisonMode::MultipleDenominator => match language {
                    Language::French => "Dénominateurs multiples",
                    Language::English => "Multiple denominators",
                },
            }
            .to_owned(),
            Self::FractionIdentification(d) => match language {
                Language::French => format!("Identifier {}/{}", d.numerator, d.denominator),
                Language::English => format!("Identify {}/{}", d.numerator, d.denominator),
            },
            Self::FractionIdentificationTemplate(_) => match language {
                Language::French => "Identifier une fraction".to_owned(),
                Language::English => "Identify a fraction".to_owned(),
            },
            Self::NumericInput(d) => d.prompt.get(language).to_owned(),
            Self::NumericInputTemplate(t) => t.teacher_label.as_ref().map_or_else(
                || t.prompt_template.get(language).to_owned(),
                |label| label.get(language).to_owned(),
            ),
        }
    }

    /// Returns `true` if this question type carries an optional visual
    /// that can be toggled on/off by the teacher.
    pub const fn has_optional_visual(&self) -> bool {
        match self {
            Self::FractionComparison(_) | Self::FractionComparisonTemplate(_) => true,
            Self::Mcq(d) => d.question_visual.is_some(),
            Self::NumericInput(d) => d.question_visual.is_some(),
            Self::McqTemplate(t) => {
                t.with_grid || matches!(t.resolver, McqResolver::FractionAddition)
            }
            Self::NumericInputTemplate(t) => t.with_grid,
            _ => false,
        }
    }

    /// Returns the effective visibility for an optional visual.
    pub fn effective_show_visual(&self, configured: Option<bool>) -> Option<bool> {
        if !self.has_optional_visual() {
            return None;
        }
        Some(configured.unwrap_or_else(|| self.default_show_visual()))
    }

    /// Returns the default visibility for the optional visual (when the
    /// teacher hasn't explicitly toggled it).
    pub const fn default_show_visual(&self) -> bool {
        match self {
            Self::FractionComparison(d) => !matches!(d.mode, ComparisonMode::SameDenominator),
            Self::FractionComparisonTemplate(t) => {
                !matches!(t.mode, ComparisonMode::SameDenominator)
            }
            // Fraction addition visuals default to shown (help the student).
            Self::Mcq(d) => {
                matches!(
                    d.question_visual,
                    Some(QuestionVisual::FractionAddition { .. })
                )
            }
            Self::McqTemplate(t) => matches!(t.resolver, McqResolver::FractionAddition),
            // Multiplication grid visuals default to hidden.
            _ => false,
        }
    }
}

fn validate_text_placeholders(
    text: &LocalizedText,
    allowed_names: &[&str],
    description: &str,
    issues: &mut Vec<String>,
) {
    for (language, value) in [("French", &text.fr), ("English", &text.en)] {
        for (name, issue) in scan_placeholder_names(value) {
            if let Some(issue) = issue {
                issues.push(format!("{language} {description} {issue}"));
            } else if !allowed_names.contains(&name.as_str()) {
                issues.push(format!(
                    "{language} {description} has unknown placeholder: {name}"
                ));
            }
        }
    }
}

fn validate_choices(choices: &[String], correct_index: usize, issues: &mut Vec<String>) {
    if choices.len() < 2 {
        issues.push("Multiple-choice definitions need at least two choices".to_owned());
    }
    if correct_index >= choices.len() {
        issues.push(format!(
            "Correct choice index {correct_index} is out of range"
        ));
    }
    if choices.iter().any(String::is_empty) {
        issues.push("Multiple-choice values must not be empty".to_owned());
    }
    let mut unique = HashSet::new();
    if choices.iter().any(|choice| !unique.insert(choice)) {
        issues.push("Multiple-choice values must be unique".to_owned());
    }
}

fn validate_fraction(
    numerator: u32,
    denominator: u32,
    allow_equal: bool,
    issues: &mut Vec<String>,
) {
    if denominator == 0 || denominator > MAX_FRACTION_PARTS {
        issues.push(format!(
            "Fraction denominator must be in 1..={MAX_FRACTION_PARTS}"
        ));
    }
    if numerator == 0 || numerator > denominator || (!allow_equal && numerator == denominator) {
        issues.push("Fraction numerator is outside its valid range".to_owned());
    }
}

fn validate_comparison_answer(
    fraction_a: (u32, u32),
    fraction_b: (u32, u32),
    answer: ComparisonAnswer,
    issues: &mut Vec<String>,
) {
    if fraction_a.1 == 0 || fraction_b.1 == 0 {
        return;
    }

    let scaled_a = u64::from(fraction_a.0) * u64::from(fraction_b.1);
    let scaled_b = u64::from(fraction_b.0) * u64::from(fraction_a.1);
    let expected_answer = match scaled_a.cmp(&scaled_b) {
        std::cmp::Ordering::Greater => ComparisonAnswer::A,
        std::cmp::Ordering::Less => ComparisonAnswer::B,
        std::cmp::Ordering::Equal => ComparisonAnswer::Equal,
    };
    if answer != expected_answer {
        issues.push("Comparison answer does not match the fraction values".to_owned());
    }
}

fn validate_fraction_ranges(
    numerators: &[u32],
    denominators: &[u32],
    allow_equal: bool,
    issues: &mut Vec<String>,
) {
    if numerators.len() > MAX_CONTENT_RANGE_VALUES || denominators.len() > MAX_CONTENT_RANGE_VALUES
    {
        issues.push(format!(
            "Fraction ranges exceed {MAX_CONTENT_RANGE_VALUES} values"
        ));
    }
    if numerators.is_empty() || denominators.is_empty() {
        issues.push("Fraction generation ranges must not be empty".to_owned());
        return;
    }
    if numerators.contains(&0) || denominators.contains(&0) {
        issues.push("Fraction generation values must be positive".to_owned());
    }
    if has_duplicates(numerators) || has_duplicates(denominators) {
        issues.push("Fraction generation ranges must not contain duplicates".to_owned());
    }
    if denominators.iter().any(|&d| d > MAX_FRACTION_PARTS) {
        issues.push(format!("Fraction denominator exceeds {MAX_FRACTION_PARTS}"));
    }
    if numerators.iter().any(|&n| n > MAX_FRACTION_PARTS)
        || numerators.iter().any(|&n| {
            !denominators
                .iter()
                .any(|&d| n <= d && (allow_equal || n < d))
        })
        || denominators
            .iter()
            .any(|&d| !numerators.iter().any(|&n| n <= d && (allow_equal || n < d)))
    {
        issues.push("Every fraction range value needs a valid pair".to_owned());
    }
}

fn validate_question_visual(visual: Option<&QuestionVisual>, issues: &mut Vec<String>) {
    match visual {
        Some(QuestionVisual::MultiplicationGrid { rows, cols }) => {
            validate_grid(*rows, *cols, issues);
        }
        Some(QuestionVisual::FractionAddition { a, b, c })
            if *b == 0
                || *b > MAX_FRACTION_PARTS
                || *a == 0
                || *a > *b
                || *c == 0
                || *c > *b
                || a.checked_add(*c).is_none() =>
        {
            issues.push("Fraction addition visual has invalid values".to_owned());
        }
        Some(QuestionVisual::FractionAddition { .. }) | None => {}
    }
}

fn validate_grid(rows: u32, cols: u32, issues: &mut Vec<String>) {
    if rows == 0
        || cols == 0
        || rows
            .checked_mul(cols)
            .is_none_or(|cells| cells > MAX_MULTIPLICATION_GRID_CELLS)
    {
        issues.push(format!(
            "Multiplication grid must contain 1..={MAX_MULTIPLICATION_GRID_CELLS} cells"
        ));
    }
}

fn validate_explanation_visual(visual: Option<&ExplanationVisual>, issues: &mut Vec<String>) {
    match visual {
        Some(ExplanationVisual::FractionRepresentation {
            numerator,
            denominator,
            ..
        }) => {
            validate_fraction(*numerator, *denominator, true, issues);
        }
        Some(ExplanationVisual::FractionAddition { a, b, c }) => {
            validate_question_visual(
                Some(&QuestionVisual::FractionAddition {
                    a: *a,
                    b: *b,
                    c: *c,
                }),
                issues,
            );
        }
        Some(ExplanationVisual::WholeFractions { count, denominator }) => {
            if *count == 0
                || *denominator == 0
                || count
                    .checked_mul(*denominator)
                    .is_none_or(|parts| parts > MAX_MULTIPLICATION_GRID_CELLS)
            {
                issues.push("Whole-fraction visual exceeds its safe bounds".to_owned());
            }
        }
        Some(ExplanationVisual::FractionComparison { a, b }) => {
            validate_fraction(a.fraction.0, a.fraction.1, true, issues);
            validate_fraction(b.fraction.0, b.fraction.1, true, issues);
        }
        Some(ExplanationVisual::FractionComparisonWithConversion { a, b }) => {
            validate_fraction(a.fraction.0, a.fraction.1, true, issues);
            validate_fraction(b.fraction.0, b.fraction.1, true, issues);
            validate_fraction(a.converted.0, a.converted.1, true, issues);
            validate_fraction(b.converted.0, b.converted.1, true, issues);
        }
        Some(ExplanationVisual::MultiplicationGrid { rows, cols }) => {
            validate_grid(*rows, *cols, issues);
        }
        Some(ExplanationVisual::PlaceValueTable { number, multiplier })
            if !place_value_table_fits(*number, *multiplier) =>
        {
            issues.push("Place-value visual exceeds the five-column limit".to_owned());
        }
        Some(ExplanationVisual::PlaceValueTable { .. }) | None => {}
    }
}

fn validate_parameter_ranges(template: &McqTemplate, required: &[&str], issues: &mut Vec<String>) {
    let mut names = HashSet::new();
    for parameter in &template.parameters {
        if parameter.name.is_empty() || !names.insert(parameter.name.as_ref()) {
            issues.push(format!(
                "Invalid or duplicate parameter name: {}",
                parameter.name
            ));
        }
        if !required
            .iter()
            .any(|required_name| *required_name == parameter.name.as_ref())
        {
            issues.push(format!(
                "Parameter {} is not supported by this resolver",
                parameter.name
            ));
        }
        if parameter.values.len() > MAX_CONTENT_RANGE_VALUES {
            issues.push(format!(
                "Parameter {} exceeds the value limit",
                parameter.name
            ));
        }
        if parameter.values.is_empty() || parameter.values.iter().any(|value| *value <= 0) {
            issues.push(format!(
                "Parameter {} must contain positive values",
                parameter.name
            ));
        }
        let mut values = HashSet::new();
        if parameter.values.iter().any(|value| !values.insert(value)) {
            issues.push(format!(
                "Parameter {} contains duplicate values",
                parameter.name
            ));
        }
    }
    for name in required {
        if !template
            .parameters
            .iter()
            .any(|parameter| parameter.name == *name)
        {
            issues.push(format!("Missing required parameter: {name}"));
        }
    }
}

fn mcq_values<'a>(template: &'a McqTemplate, name: &str) -> &'a [i32] {
    template
        .parameters
        .iter()
        .find(|parameter| parameter.name == name)
        .map_or(&[], |parameter| parameter.values.as_slice())
}

fn validate_fraction_value_parameters(
    values_a: &[i32],
    values_b: &[i32],
    issues: &mut Vec<String>,
) {
    let mut has_safe_pair = false;
    let mut has_unsafe_pair = false;
    for &a in values_a {
        for &b in values_b {
            if a > 0 && b > 0 && a % b == 0 {
                if fraction_value_pair_is_safe(a, b) {
                    has_safe_pair = true;
                } else {
                    has_unsafe_pair = true;
                }
            }
        }
    }
    if has_unsafe_pair {
        issues.push("Fraction-value parameters contain unsafe divisible pairs".to_owned());
    }
    if !has_safe_pair {
        issues.push("Fraction-value parameters have no safe divisible pair".to_owned());
    }
}

fn validate_mcq_template(template: &McqTemplate, issues: &mut Vec<String>) {
    let required: &[&str] = match template.resolver {
        McqResolver::FractionValue
        | McqResolver::Multiplication
        | McqResolver::MultiplyByPowerOf10 => &["a", "b"],
        McqResolver::FractionAddition => &["a", "b", "c"],
        McqResolver::FractionName => &["d"],
    };
    validate_parameter_ranges(template, required, issues);
    let values_a = mcq_values(template, "a");
    let values_b = mcq_values(template, "b");
    let candidate_count = required.iter().try_fold(1_usize, |total, name| {
        total.checked_mul(mcq_values(template, name).len())
    });
    if candidate_count.is_none_or(|count| count > MAX_TEMPLATE_CANDIDATES) {
        issues.push(format!(
            "Template combinations exceed {MAX_TEMPLATE_CANDIDATES} candidates"
        ));
    }
    match template.resolver {
        McqResolver::FractionValue => {
            validate_fraction_value_parameters(values_a, values_b, issues);
        }
        McqResolver::FractionAddition => {
            let values_c = mcq_values(template, "c");
            if values_b
                .iter()
                .any(|b| *b > MAX_FRACTION_PARTS.cast_signed())
            {
                issues.push(format!(
                    "Fraction-addition denominator exceeds {MAX_FRACTION_PARTS}"
                ));
            }
            if values_b.iter().any(|b| *b > i32::MAX / 2)
                || !values_a.iter().any(|a| {
                    values_b.iter().any(|b| {
                        values_c.iter().any(|c| {
                            a < b && c < b && a.checked_add(*c).is_some_and(|sum| sum <= *b)
                        })
                    })
                })
            {
                issues.push("Fraction-addition parameters have no valid triplet".to_owned());
            }
        }
        McqResolver::FractionName => {
            if mcq_values(template, "d")
                .iter()
                .any(|d| !matches!(*d, 2 | 3 | 4 | 5 | 6 | 8))
            {
                issues.push("Fraction-name denominator is not supported".to_owned());
            }
        }
        McqResolver::Multiplication | McqResolver::MultiplyByPowerOf10 => {
            let safe_products = values_a.iter().all(|a| {
                values_b.iter().all(|b| {
                    a.checked_mul(*b)
                        .is_some_and(|product| product <= i32::MAX - 10)
                })
            });
            if !safe_products {
                issues.push("Multiplication parameters exceed safe arithmetic bounds".to_owned());
            }
            if matches!(template.resolver, McqResolver::MultiplyByPowerOf10)
                && values_b.iter().any(|value| !is_power_of_ten(*value))
            {
                issues.push("Place-value multiplier must be a power of ten".to_owned());
            }
            if matches!(template.resolver, McqResolver::MultiplyByPowerOf10)
                && values_a.iter().any(|a| {
                    values_b
                        .iter()
                        .any(|b| !place_value_table_fits(a.cast_unsigned(), b.cast_unsigned()))
                })
            {
                issues.push("Place-value result exceeds the five-column limit".to_owned());
            }
            if (template.with_grid || matches!(template.resolver, McqResolver::Multiplication))
                && values_a.iter().any(|a| {
                    values_b.iter().any(|b| {
                        a.checked_mul(*b).is_none_or(|cells| {
                            cells <= 0 || cells.cast_unsigned() > MAX_MULTIPLICATION_GRID_CELLS
                        })
                    })
                })
            {
                issues.push("Multiplication grid parameters exceed the cell limit".to_owned());
            }
        }
    }
}

fn validate_comparison_template(template: &FractionComparisonTemplate, issues: &mut Vec<String>) {
    if template.numerator_range.len() > MAX_CONTENT_RANGE_VALUES
        || template.denominator_range.len() > MAX_CONTENT_RANGE_VALUES
    {
        issues.push(format!(
            "Comparison ranges exceed {MAX_CONTENT_RANGE_VALUES} values"
        ));
    }
    if template.numerator_range.is_empty() || template.denominator_range.is_empty() {
        issues.push("Comparison generation ranges must not be empty".to_owned());
        return;
    }
    if has_duplicates(&template.numerator_range) || has_duplicates(&template.denominator_range) {
        issues.push("Comparison ranges must not contain duplicates".to_owned());
    }
    if template.numerator_range.contains(&0)
        || template.denominator_range.contains(&0)
        || template
            .numerator_range
            .iter()
            .any(|&n| n > MAX_FRACTION_PARTS)
        || template
            .denominator_range
            .iter()
            .any(|&d| d > MAX_FRACTION_PARTS)
    {
        issues.push("Comparison ranges contain values outside safe fraction bounds".to_owned());
    }
    let eligible_denominators: Vec<u32> = template
        .denominator_range
        .iter()
        .copied()
        .filter(|&denominator| {
            template
                .numerator_range
                .iter()
                .any(|&numerator| numerator > 0 && numerator < denominator)
        })
        .collect();
    let can_make_question = match template.mode {
        ComparisonMode::SameDenominator => template.denominator_range.iter().all(|&d| {
            template
                .numerator_range
                .iter()
                .filter(|&&n| n > 0 && n < d)
                .count()
                >= 2
        }),
        ComparisonMode::SameNumerator => template.numerator_range.iter().all(|&n| {
            template
                .denominator_range
                .iter()
                .filter(|&&d| d > n)
                .count()
                >= 2
        }),
        ComparisonMode::MultipleDenominator => eligible_denominators.iter().any(|&a| {
            eligible_denominators
                .iter()
                .any(|&b| a != b && (a % b == 0 || b % a == 0))
        }),
    };
    if !can_make_question {
        issues.push("Comparison ranges cannot generate the selected mode".to_owned());
    }
}

fn validate_numeric_template(template: &NumericInputTemplate, issues: &mut Vec<String>) {
    if template.factor_a_range.len() > MAX_CONTENT_RANGE_VALUES
        || template.factor_b_range.len() > MAX_CONTENT_RANGE_VALUES
    {
        issues.push(format!(
            "Numeric ranges exceed {MAX_CONTENT_RANGE_VALUES} values"
        ));
    }
    if template.factor_a_range.is_empty() || template.factor_b_range.is_empty() {
        issues.push("Numeric multiplication ranges must not be empty".to_owned());
        return;
    }
    if has_duplicates(&template.factor_a_range) || has_duplicates(&template.factor_b_range) {
        issues.push("Numeric multiplication ranges must not contain duplicates".to_owned());
    }
    if template.factor_a_range.contains(&0) || template.factor_b_range.contains(&0) {
        issues.push("Numeric multiplication factors must be positive".to_owned());
    }
    if template.factor_a_range.iter().any(|a| {
        template.factor_b_range.iter().any(|b| {
            a.checked_mul(*b)
                .is_none_or(|product| product > MAX_NUMERIC_INPUT_ANSWER)
        })
    }) {
        issues.push(format!(
            "Numeric answers must not exceed {MAX_NUMERIC_INPUT_ANSWER}"
        ));
    }
    if (template.with_grid || !template.place_value_explanation)
        && template.factor_a_range.iter().any(|a| {
            template.factor_b_range.iter().any(|b| {
                a.checked_mul(*b)
                    .is_none_or(|cells| cells > MAX_MULTIPLICATION_GRID_CELLS)
            })
        })
    {
        issues.push("Multiplication grid parameters exceed the cell limit".to_owned());
    }
    if template.place_value_explanation
        && template
            .factor_b_range
            .iter()
            .any(|&b| !is_power_of_ten(b.cast_signed()))
    {
        issues.push("Place-value multiplier must be a power of ten".to_owned());
    }
    if template.place_value_explanation
        && template.factor_a_range.iter().any(|&a| {
            template
                .factor_b_range
                .iter()
                .any(|&b| !place_value_table_fits(a, b))
        })
    {
        issues.push("Place-value result exceeds the five-column limit".to_owned());
    }
}

fn has_duplicates<T: Eq + Hash>(values: &[T]) -> bool {
    let mut seen = HashSet::new();
    values.iter().any(|value| !seen.insert(value))
}

const fn is_power_of_ten(value: i32) -> bool {
    if value < 10 {
        return false;
    }
    let mut remainder = value;
    while remainder % 10 == 0 {
        remainder /= 10;
    }
    remainder == 1
}

/// One character's side in a fraction comparison visual.
#[derive(Clone, Debug, PartialEq, Eq, Hash, Deserialize, Serialize)]
#[serde(deny_unknown_fields)]
pub struct ComparisonSide {
    pub character: Cow<'static, str>,
    pub fraction: (u32, u32),
}

/// One character's side in a fraction comparison visual that also carries
/// the fraction converted to a common denominator.
#[derive(Clone, Debug, PartialEq, Eq, Hash, Deserialize, Serialize)]
#[serde(deny_unknown_fields)]
pub struct ComparisonSideWithConversion {
    pub character: Cow<'static, str>,
    pub fraction: (u32, u32),
    pub converted: (u32, u32),
}

/// Optional visual element displayed alongside the text explanation in feedback.
#[derive(Clone, Debug, PartialEq, Eq, Hash, Deserialize, Serialize)]
#[serde(deny_unknown_fields)]
pub enum ExplanationVisual {
    /// A fraction representation with a concrete display mode.
    FractionRepresentation {
        numerator: u32,
        denominator: u32,
        mode: FractionShape,
    },
    /// Two fraction bars (`a/b` + `c/b`) and the result bar (`(a+c)/b`).
    FractionAddition { a: u32, b: u32, c: u32 },
    /// `count` fully-coloured fraction bars of `denominator` slices each,
    /// illustrating that a fraction equals a whole number.
    WholeFractions { count: u32, denominator: u32 },
    /// Two labelled fraction bars side by side (same or different denominator).
    /// Used for `SameDenominator` and `SameNumerator` comparison explanations.
    FractionComparison {
        a: ComparisonSide,
        b: ComparisonSide,
    },
    /// Two pairs of bars: original fractions then converted to a common
    /// denominator, showing why one is larger. Used for `MultipleDenominator`.
    FractionComparisonWithConversion {
        a: ComparisonSideWithConversion,
        b: ComparisonSideWithConversion,
    },
    /// A small grid of `rows` by `cols` coloured cells illustrating a product.
    MultiplicationGrid { rows: u32, cols: u32 },
    /// A place-value table showing how multiplying by a power of ten shifts
    /// digits left and appends zeros. The added zeros are highlighted.
    PlaceValueTable {
        /// The number being multiplied.
        number: u32,
        /// The power-of-ten multiplier.
        multiplier: u32,
    },
}

/// Optional visual element displayed alongside the question prompt.
/// Unlike `ExplanationVisual` (shown in feedback), this is shown *during*
/// the question itself and can be toggled off by the teacher.
#[derive(Clone, Debug, PartialEq, Eq, Hash, Deserialize, Serialize)]
#[serde(deny_unknown_fields)]
pub enum QuestionVisual {
    /// A grid of `rows` by `cols` coloured cells illustrating a multiplication.
    MultiplicationGrid { rows: u32, cols: u32 },
    /// Two fraction bars: `a/b` (blue) + `c/b` (orange).
    /// No result bar; the student must calculate.
    FractionAddition { a: u32, b: u32, c: u32 },
}

/// A fully resolved multiple-choice question with prompt, choices, and explanation.
#[derive(Clone, Debug, PartialEq, Eq, Hash, Deserialize, Serialize)]
#[serde(deny_unknown_fields)]
pub struct McqDefinition {
    pub prompt: LocalizedText,
    pub choices: Vec<String>,
    pub correct_index: usize,
    pub explanation: LocalizedExplanation,
    /// Optional visual to display alongside the explanation in feedback.
    pub explanation_visual: Option<ExplanationVisual>,
    /// Optional visual displayed alongside the prompt during the question.
    pub question_visual: Option<QuestionVisual>,
}

/// A resolved fraction-colouring question with a concrete shape.
#[derive(Clone, Debug, PartialEq, Eq, Hash, Deserialize, Serialize)]
#[serde(deny_unknown_fields)]
pub struct FractionVisualizationDefinition {
    pub prompt: LocalizedText,
    pub numerator: u32,
    pub denominator: u32,
    #[serde(default)]
    pub shape: FractionShape,
    pub explanation: LocalizedExplanation,
}

impl FractionVisualizationDefinition {
    pub const fn explanation_visual(&self) -> ExplanationVisual {
        ExplanationVisual::FractionRepresentation {
            numerator: self.numerator,
            denominator: self.denominator,
            mode: self.shape,
        }
    }
}

/// A resolved fraction comparison question pairing two characters.
#[derive(Clone, Debug, PartialEq, Eq, Hash, Deserialize, Serialize)]
#[serde(deny_unknown_fields)]
pub struct FractionComparisonDefinition {
    pub prompt: LocalizedText,
    pub character_a: Cow<'static, str>,
    pub fraction_a: (u32, u32),
    pub character_b: Cow<'static, str>,
    pub fraction_b: (u32, u32),
    pub answer: ComparisonAnswer,
    pub mode: ComparisonMode,
    pub explanation: LocalizedExplanation,
    pub explanation_visual: Option<ExplanationVisual>,
}

/// Which side won the comparison: A, B, or Equal.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash, Reflect, Deserialize, Serialize)]
#[serde(deny_unknown_fields)]
pub enum ComparisonAnswer {
    A,
    B,
    Equal,
}

/// Selects the fraction relationship generated by a comparison template.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash, Deserialize, Serialize)]
#[serde(deny_unknown_fields)]
pub enum ComparisonMode {
    SameDenominator,
    SameNumerator,
    MultipleDenominator,
}

/// A concrete numeric-input question: textual prompt + expected numeric answer.
#[derive(Clone, Debug, PartialEq, Eq, Hash, Deserialize, Serialize)]
#[serde(deny_unknown_fields)]
pub struct NumericInputDefinition {
    pub prompt: LocalizedText,
    pub correct_answer: u32,
    pub explanation: LocalizedExplanation,
    pub explanation_visual: Option<ExplanationVisual>,
    /// Optional visual displayed alongside the prompt during the question.
    pub question_visual: Option<QuestionVisual>,
}

/// A concrete geometric representation of a fraction.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Hash, Reflect, Deserialize, Serialize)]
#[serde(deny_unknown_fields)]
pub enum FractionShape {
    #[default]
    Line,
    Square,
    Circle,
}

/// A resolved fraction identification MCQ: given a shape, pick the fraction.
#[derive(Clone, Debug, PartialEq, Eq, Hash, Deserialize, Serialize)]
#[serde(deny_unknown_fields)]
pub struct FractionIdentificationDefinition {
    pub numerator: u32,
    pub denominator: u32,
    #[serde(default)]
    pub mode: FractionShape,
    pub choices: Vec<String>,
    pub correct_index: usize,
    pub explanation: LocalizedExplanation,
}

impl FractionIdentificationDefinition {
    pub const fn explanation_visual(&self) -> ExplanationVisual {
        ExplanationVisual::FractionRepresentation {
            numerator: self.numerator,
            denominator: self.denominator,
            mode: self.mode,
        }
    }
}

/// Parameterized MCQ template resolved at runtime via an `McqResolver`.
#[derive(Clone, Debug, Deserialize, Serialize)]
#[serde(deny_unknown_fields)]
pub struct McqTemplate {
    pub prompt_template: LocalizedText,
    pub parameters: Vec<ParameterRange>,
    pub resolver: McqResolver,
    pub explanation_template: LocalizedText,
    /// Optional label shown in the teacher config UI instead of the raw
    /// `prompt_template`. Useful to disambiguate templates that share the
    /// same prompt text.
    #[serde(default)]
    pub teacher_label: Option<LocalizedText>,
    /// When `true`, the resolved `McqDefinition` will carry a
    /// `QuestionVisual::MultiplicationGrid` derived from the selected factors.
    #[serde(default)]
    pub with_grid: bool,
}

/// Named parameter with its allowed values, used inside templates.
#[derive(Clone, Debug, Deserialize, Serialize)]
#[serde(deny_unknown_fields)]
pub struct ParameterRange {
    pub name: Cow<'static, str>,
    pub values: Vec<i32>,
}

/// Resolver strategy that turns template parameters into a concrete MCQ.
#[derive(Clone, Debug, Deserialize, Serialize)]
#[serde(deny_unknown_fields)]
#[allow(clippy::enum_variant_names)]
pub enum McqResolver {
    FractionValue,
    FractionAddition,
    FractionName,
    Multiplication,
    MultiplyByPowerOf10,
}

/// Template that generates fraction-colouring questions from ranges.
#[derive(Clone, Debug, Deserialize, Serialize)]
#[serde(deny_unknown_fields)]
pub struct FractionVisualizationTemplate {
    pub prompt_template: LocalizedText,
    pub numerator_range: Vec<u32>,
    pub denominator_range: Vec<u32>,
    pub explanation_template: LocalizedText,
}

/// Template that generates fraction-identification questions from ranges.
#[derive(Clone, Debug, Deserialize, Serialize)]
#[serde(deny_unknown_fields)]
pub struct FractionIdentificationTemplate {
    pub numerator_range: Vec<u32>,
    pub denominator_range: Vec<u32>,
}

/// Template that generates comparison questions for a selected fraction relationship.
#[derive(Clone, Debug, Deserialize, Serialize)]
#[serde(deny_unknown_fields)]
pub struct FractionComparisonTemplate {
    pub prompt: LocalizedText,
    pub character_a: Cow<'static, str>,
    pub character_b: Cow<'static, str>,
    pub mode: ComparisonMode,
    pub denominator_range: Vec<u32>,
    pub numerator_range: Vec<u32>,
    pub explanation_template: LocalizedText,
}

/// Template that generates numeric-input multiplication questions.
#[derive(Clone, Debug, Deserialize, Serialize)]
#[serde(deny_unknown_fields)]
pub struct NumericInputTemplate {
    pub prompt_template: LocalizedText,
    pub factor_a_range: Vec<u32>,
    pub factor_b_range: Vec<u32>,
    pub explanation_template: LocalizedText,
    /// Optional label shown in the teacher config UI instead of the prompt template.
    #[serde(default)]
    pub teacher_label: Option<LocalizedText>,
    /// When `true`, the resolved `NumericInputDefinition` will carry a
    /// `QuestionVisual::MultiplicationGrid` derived from the selected factors.
    #[serde(default)]
    pub with_grid: bool,
    /// When `true`, the explanation visual uses a place-value table instead of
    /// a multiplication grid.
    #[serde(default)]
    pub place_value_explanation: bool,
}

/// A concrete question ready for the session, with visual toggle state.
#[derive(Clone, Debug)]
pub struct ResolvedQuestion {
    pub exercise_id: String,
    pub instance: QuestionInstance,
    /// When `true`, the optional question aid is hidden for this instance.
    pub hide_optional_visual: bool,
}

/// Whether the student answered correctly or not.
#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub enum AnswerResult {
    #[default]
    Correct,
    Incorrect,
}
