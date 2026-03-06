use std::borrow::Cow;
use std::collections::HashMap;
use std::hash::{DefaultHasher, Hash, Hasher};

use bevy::prelude::*;
use serde::{Deserialize, Serialize};

use crate::i18n::{Language, TranslationKey};

/// Maximum number of times a single question can appear in a teacher-configured session.
pub const MAX_QUESTION_REPETITIONS: usize = 5;

/// A text string available in both French and English.
/// Used for pedagogical content (prompts, explanations) that cannot be
/// represented by a static `TranslationKey` because templates generate
/// dynamic text at runtime.
#[derive(Clone, Debug, PartialEq, Eq, Hash, Deserialize, Serialize)]
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

/// Top-level resource holding all themes and their lessons.
#[derive(Resource, Debug, Default, Reflect, Deserialize, Serialize)]
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
}

/// A pedagogical theme grouping related lessons.
#[derive(Clone, Debug, Default, Deserialize, Serialize)]
pub struct Theme {
    pub id: String,
    #[serde(skip)]
    pub title_key: TranslationKey,
    pub available: bool,
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

/// A single lesson within a theme, containing question pools.
#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct Lesson {
    pub id: String,
    #[serde(skip)]
    pub title_key: TranslationKey,
    pub available: bool,
    pub questions: Vec<QuestionDefinition>,
}

/// Classifies a question by its pedagogical type, regardless of whether it
/// originates from a static definition or a template.
#[derive(Clone, Copy, Debug, Eq, PartialEq, Hash, Reflect, Deserialize, Serialize)]
pub enum QuestionType {
    Mcq,
    Visualization,
    Comparison,
    Identification,
    NumericInput,
}

/// Tagged union of all question variants, both static definitions and templates.
#[derive(Clone, Debug, Deserialize, Serialize)]
#[serde(tag = "type")]
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

impl QuestionDefinition {
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

    /// Returns a human-readable label for this question, resolved to the given language.
    /// Used in the teacher config UI to display individual questions.
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
            Self::FractionComparisonTemplate(t) => match t.difficulty {
                ComparisonDifficulty::SameDenominator => match language {
                    Language::French => "Même dénominateur",
                    Language::English => "Same denominator",
                },
                ComparisonDifficulty::SameNumerator => match language {
                    Language::French => "Même numérateur",
                    Language::English => "Same numerator",
                },
                ComparisonDifficulty::MultipleDenominator => match language {
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
                || {
                    let diff = difficulty_label(t.difficulty, language);
                    format!("{} ({diff})", t.prompt_template.get(language))
                },
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

    /// Returns the default visibility for the optional visual (when the
    /// teacher hasn't explicitly toggled it).
    pub const fn default_show_visual(&self) -> bool {
        match self {
            Self::FractionComparison(d) => {
                !matches!(d.difficulty, ComparisonDifficulty::SameDenominator)
            }
            Self::FractionComparisonTemplate(t) => {
                !matches!(t.difficulty, ComparisonDifficulty::SameDenominator)
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

    /// Returns a `u64` hash that uniquely identifies a resolved question's
    /// parameters. Used by session building to avoid duplicate questions.
    /// Returns `None` for unresolved templates.
    pub fn fingerprint(&self) -> Option<u64> {
        let mut hasher = DefaultHasher::new();
        match self {
            Self::Mcq(d) => d.hash(&mut hasher),
            Self::FractionVisualization(d) => d.hash(&mut hasher),
            Self::FractionComparison(d) => d.hash(&mut hasher),
            Self::FractionIdentification(d) => d.hash(&mut hasher),
            Self::NumericInput(d) => d.hash(&mut hasher),
            // Templates are not resolved yet, so no fingerprint.
            _ => return None,
        }
        Some(hasher.finish())
    }
}

/// One character's side in a fraction comparison visual.
#[derive(Clone, Debug, PartialEq, Eq, Hash, Deserialize, Serialize)]
pub struct ComparisonSide {
    pub character: Cow<'static, str>,
    pub fraction: (u32, u32),
}

/// One character's side in a fraction comparison visual that also carries
/// the fraction converted to a common denominator.
#[derive(Clone, Debug, PartialEq, Eq, Hash, Deserialize, Serialize)]
pub struct ComparisonSideWithConversion {
    pub character: Cow<'static, str>,
    pub fraction: (u32, u32),
    pub converted: (u32, u32),
}

/// Optional visual element displayed alongside the text explanation in feedback.
#[derive(Clone, Debug, PartialEq, Eq, Hash, Deserialize, Serialize)]
pub enum ExplanationVisual {
    /// A fraction bar with `numerator` coloured slices out of `denominator`.
    FractionBar { numerator: u32, denominator: u32 },
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
pub enum QuestionVisual {
    /// A grid of `rows` by `cols` coloured cells illustrating a multiplication.
    MultiplicationGrid { rows: u32, cols: u32 },
    /// Two fraction bars: `a/b` (blue) + `c/b` (orange).
    /// No result bar; the student must calculate.
    FractionAddition { a: u32, b: u32, c: u32 },
}

/// A fully resolved multiple-choice question with prompt, choices, and explanation.
#[derive(Clone, Debug, PartialEq, Eq, Hash, Deserialize, Serialize)]
pub struct McqDefinition {
    pub prompt: LocalizedText,
    pub choices: Vec<String>,
    pub correct_index: usize,
    pub explanation: LocalizedText,
    /// Optional visual to display alongside the explanation in feedback.
    pub explanation_visual: Option<ExplanationVisual>,
    /// Optional visual displayed alongside the prompt during the question.
    pub question_visual: Option<QuestionVisual>,
}

/// A resolved fraction-bar colouring question.
#[derive(Clone, Debug, PartialEq, Eq, Hash, Deserialize, Serialize)]
pub struct FractionVisualizationDefinition {
    pub prompt: LocalizedText,
    pub numerator: u32,
    pub denominator: u32,
    pub explanation: LocalizedText,
}

/// A resolved fraction comparison question pairing two characters.
#[derive(Clone, Debug, PartialEq, Eq, Hash, Deserialize, Serialize)]
pub struct FractionComparisonDefinition {
    pub prompt: LocalizedText,
    pub character_a: Cow<'static, str>,
    pub fraction_a: (u32, u32),
    pub character_b: Cow<'static, str>,
    pub fraction_b: (u32, u32),
    pub answer: ComparisonAnswer,
    pub difficulty: ComparisonDifficulty,
    pub explanation: LocalizedText,
    pub explanation_visual: Option<ExplanationVisual>,
}

/// Which side won the comparison: A, B, or Equal.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash, Reflect, Deserialize, Serialize)]
pub enum ComparisonAnswer {
    A,
    B,
    Equal,
}

/// Difficulty tier for fraction comparisons based on denominator relationship.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash, Deserialize, Serialize)]
pub enum ComparisonDifficulty {
    SameDenominator,
    SameNumerator,
    MultipleDenominator,
}

/// Explicit difficulty level for question types that have graded variants.
#[derive(Clone, Copy, Debug, Eq, PartialEq, Hash, Reflect, Deserialize, Serialize)]
pub enum Difficulty {
    Beginner,
    Intermediate,
    Advanced,
}

/// Returns a human-readable difficulty label for the given language.
const fn difficulty_label(difficulty: Difficulty, language: Language) -> &'static str {
    match (difficulty, language) {
        (Difficulty::Beginner, Language::French) => "débutant",
        (Difficulty::Beginner, Language::English) => "beginner",
        (Difficulty::Intermediate, Language::French) => "intermédiaire",
        (Difficulty::Intermediate, Language::English) => "intermediate",
        (Difficulty::Advanced, Language::French) => "avancé",
        (Difficulty::Advanced, Language::English) => "advanced",
    }
}

/// A concrete numeric-input question: textual prompt + expected numeric answer.
#[derive(Clone, Debug, PartialEq, Eq, Hash, Deserialize, Serialize)]
pub struct NumericInputDefinition {
    pub prompt: LocalizedText,
    pub correct_answer: u32,
    pub explanation: LocalizedText,
    pub explanation_visual: Option<ExplanationVisual>,
    /// Optional visual displayed alongside the prompt during the question.
    pub question_visual: Option<QuestionVisual>,
}

/// A resolved fraction identification MCQ: given a bar, pick the fraction.
#[derive(Clone, Debug, PartialEq, Eq, Hash, Deserialize, Serialize)]
pub struct FractionIdentificationDefinition {
    pub numerator: u32,
    pub denominator: u32,
    pub choices: Vec<String>,
    pub correct_index: usize,
    pub explanation: LocalizedText,
    pub explanation_visual: Option<ExplanationVisual>,
}

/// Parameterized MCQ template resolved at runtime via an `McqResolver`.
#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct McqTemplate {
    pub prompt_template: LocalizedText,
    pub parameters: Vec<ParameterRange>,
    pub resolver: McqResolver,
    pub explanation_template: LocalizedText,
    /// Optional label shown in the teacher config UI instead of the raw
    /// `prompt_template`. Useful to disambiguate templates that share the
    /// same prompt text.
    pub teacher_label: Option<LocalizedText>,
    /// When `true`, the resolved `McqDefinition` will carry a
    /// `QuestionVisual::MultiplicationGrid` derived from the selected factors.
    #[serde(default)]
    pub with_grid: bool,
}

/// Named parameter with its allowed values, used inside templates.
#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct ParameterRange {
    pub name: Cow<'static, str>,
    pub values: Vec<i32>,
}

/// Resolver strategy that turns template parameters into a concrete MCQ.
#[derive(Clone, Debug, Deserialize, Serialize)]
#[allow(clippy::enum_variant_names)]
pub enum McqResolver {
    FractionValue,
    FractionAddition,
    FractionName,
    Multiplication,
    MultiplyByPowerOf10,
}

/// Template that generates fraction-bar colouring questions from ranges.
#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct FractionVisualizationTemplate {
    pub prompt_template: LocalizedText,
    pub numerator_range: Vec<u32>,
    pub denominator_range: Vec<u32>,
    pub explanation_template: LocalizedText,
}

/// Template that generates fraction-identification questions from ranges.
#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct FractionIdentificationTemplate {
    pub numerator_range: Vec<u32>,
    pub denominator_range: Vec<u32>,
}

/// Template that generates comparison questions for a given difficulty tier.
#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct FractionComparisonTemplate {
    pub prompt: LocalizedText,
    pub character_a: Cow<'static, str>,
    pub character_b: Cow<'static, str>,
    pub difficulty: ComparisonDifficulty,
    pub denominator_range: Vec<u32>,
    pub numerator_range: Vec<u32>,
    pub explanation_template: LocalizedText,
}

/// Template that generates numeric-input multiplication questions.
#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct NumericInputTemplate {
    pub prompt_template: LocalizedText,
    pub factor_a_range: Vec<u32>,
    pub factor_b_range: Vec<u32>,
    pub difficulty: Difficulty,
    pub explanation_template: LocalizedText,
    /// Optional label shown in the teacher config UI instead of the raw
    /// prompt + difficulty.
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
    pub definition: QuestionDefinition,
    /// When `true`, the optional question visual is hidden for this instance.
    pub hide_visual: bool,
}

/// Whether the student answered correctly or not.
#[derive(Clone, Debug, Default)]
pub enum AnswerResult {
    #[default]
    Correct,
    Incorrect,
}
