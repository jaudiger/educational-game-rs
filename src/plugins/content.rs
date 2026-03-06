use bevy::prelude::*;

use crate::data::ContentLibrary;
use crate::data::content::{
    ComparisonDifficulty, Difficulty, FractionComparisonTemplate, FractionIdentificationTemplate,
    FractionVisualizationTemplate, Lesson, LocalizedText, McqResolver, McqTemplate,
    NumericInputTemplate, ParameterRange, QuestionDefinition, Theme,
};
use crate::i18n::TranslationKey;

/// Builds and inserts the content library at startup.
pub struct ContentPlugin;

impl Plugin for ContentPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(Startup, start_content_loading);
    }
}

fn start_content_loading(mut commands: Commands) {
    commands.insert_resource(build_content_library());
}

fn build_content_library() -> ContentLibrary {
    let mut library = ContentLibrary {
        themes: vec![
            Theme {
                id: "maths".into(),
                title_key: TranslationKey::ThemeMaths,
                available: true,
                lessons: vec![
                    Lesson {
                        id: "fractions".into(),
                        title_key: TranslationKey::LessonFractions,
                        available: true,
                        questions: build_fractions_questions(),
                    },
                    Lesson {
                        id: "multiplication_tables".into(),
                        title_key: TranslationKey::LessonMultiplicationTables,
                        available: true,
                        questions: build_multiplication_tables_questions(),
                    },
                    Lesson {
                        id: "multiply_10_100_1000".into(),
                        title_key: TranslationKey::LessonMultiplyByPowerOf10,
                        available: true,
                        questions: build_multiply_10_100_1000_questions(),
                    },
                ],
                ..Default::default()
            },
            Theme {
                id: "french".into(),
                title_key: TranslationKey::ThemeFrench,
                available: false,
                lessons: Vec::new(),
                ..Default::default()
            },
            Theme {
                id: "english".into(),
                title_key: TranslationKey::ThemeEnglish,
                available: false,
                lessons: Vec::new(),
                ..Default::default()
            },
            Theme {
                id: "science".into(),
                title_key: TranslationKey::ThemeScience,
                available: false,
                lessons: Vec::new(),
                ..Default::default()
            },
        ],
        ..Default::default()
    };
    library.build_indexes();
    library
}

fn build_fractions_questions() -> Vec<QuestionDefinition> {
    let mut questions = build_static_and_template_questions();

    // FractionIdentification template, resolved per-session in build_session()
    questions.push(QuestionDefinition::FractionIdentificationTemplate(
        FractionIdentificationTemplate {
            numerator_range: vec![1, 2, 3, 4, 5, 6, 7, 8, 9, 10, 11],
            denominator_range: vec![3, 4, 5, 6, 7, 8, 9, 10, 11, 12],
        },
    ));

    questions
}

fn build_static_and_template_questions() -> Vec<QuestionDefinition> {
    let mut questions = build_mcq_questions();
    questions.extend(build_visual_fraction_questions());
    questions
}

#[allow(clippy::literal_string_with_formatting_args)]
fn build_mcq_questions() -> Vec<QuestionDefinition> {
    vec![
        QuestionDefinition::McqTemplate(McqTemplate {
            prompt_template: LocalizedText::new(
                "Quelle fraction représente {name} ?",
                "Which fraction represents {name}?",
            ),
            parameters: vec![ParameterRange {
                name: "d".into(),
                values: vec![2, 3, 4, 5, 6, 8],
            }],
            resolver: McqResolver::FractionName,
            explanation_template: LocalizedText::new(
                "Si on coupe un gâteau en {d} parts égales, {name} c'est la fraction {fraction}.",
                "If you cut a cake into {d} equal pieces, {name} is the fraction {fraction}.",
            ),
            teacher_label: None,
            with_grid: false,
        }),
        QuestionDefinition::McqTemplate(McqTemplate {
            prompt_template: LocalizedText::new(
                "Combien font {a}/{b} + {c}/{b} ?",
                "What is {a}/{b} + {c}/{b}?",
            ),
            parameters: vec![
                ParameterRange {
                    name: "a".into(),
                    values: vec![1, 2, 3],
                },
                ParameterRange {
                    name: "b".into(),
                    values: vec![4, 6, 8],
                },
                ParameterRange {
                    name: "c".into(),
                    values: vec![1, 2, 3],
                },
            ],
            resolver: McqResolver::FractionAddition,
            explanation_template: LocalizedText::new(
                "Les deux fractions ont le même dénominateur ({b}), on additionne les numérateurs : {a} + {c} = {sum}. Donc {a}/{b} + {c}/{b} = {result}.",
                "Both fractions have the same denominator ({b}), so we add the numerators: {a} + {c} = {sum}. So {a}/{b} + {c}/{b} = {result}.",
            ),
            teacher_label: None,
            with_grid: false,
        }),
        QuestionDefinition::McqTemplate(McqTemplate {
            prompt_template: LocalizedText::new(
                "Quel nombre est représenté par {a}/{b} ?",
                "What number is represented by {a}/{b}?",
            ),
            parameters: vec![
                ParameterRange {
                    name: "a".into(),
                    values: vec![2, 3, 4, 5, 6, 8, 9, 10, 12, 15, 16, 20],
                },
                ParameterRange {
                    name: "b".into(),
                    values: vec![2, 3, 4, 5],
                },
            ],
            resolver: McqResolver::FractionValue,
            explanation_template: LocalizedText::new(
                "Si on partage {a} parts en groupes de {b}, on obtient {result} groupes. Donc {a}/{b} = {result}.",
                "If we share {a} parts into groups of {b}, we get {result} groups. So {a}/{b} = {result}.",
            ),
            teacher_label: None,
            with_grid: false,
        }),
    ]
}

fn build_visual_fraction_questions() -> Vec<QuestionDefinition> {
    vec![
        // ----- FractionVisualization template -----
        QuestionDefinition::FractionVisualizationTemplate(FractionVisualizationTemplate {
            prompt_template: LocalizedText::new(
                "Colorie {n}/{d} du gâteau",
                "Colour {n}/{d} of the pie",
            ),
            numerator_range: vec![1, 2, 3, 4, 5, 6, 7],
            denominator_range: vec![3, 4, 5, 6, 8, 10, 12],
            explanation_template: LocalizedText::new(
                "Il y a {d} parts au total et {n} sont coloriées : c'est la fraction {n}/{d}.",
                "There are {d} parts in total and {n} are colored: that's the fraction {n}/{d}.",
            ),
        }),
        // ----- FractionComparison template (same denominator) -----
        QuestionDefinition::FractionComparisonTemplate(FractionComparisonTemplate {
            prompt: LocalizedText::new("Qui a mangé le plus de gâteau ?", "Who ate the most pie?"),
            character_a: "Léa".into(),
            character_b: "Tom".into(),
            difficulty: ComparisonDifficulty::SameDenominator,
            denominator_range: vec![4, 5, 6, 8, 10, 12],
            numerator_range: vec![1, 2, 3, 4, 5, 6, 7, 8, 9, 10, 11],
            explanation_template: LocalizedText::new(
                "Les gâteaux ont les mêmes parts ({da}), on compare juste le nombre de parts : {na} et {nb}.",
                "The pies have the same slices ({da}), so we compare how many slices: {na} and {nb}.",
            ),
        }),
        // ----- FractionComparison template (same numerator) -----
        QuestionDefinition::FractionComparisonTemplate(FractionComparisonTemplate {
            prompt: LocalizedText::new("Qui a mangé le plus de gâteau ?", "Who ate the most pie?"),
            character_a: "Emma".into(),
            character_b: "Hugo".into(),
            difficulty: ComparisonDifficulty::SameNumerator,
            denominator_range: vec![3, 4, 5, 6, 8, 10, 12],
            numerator_range: vec![2, 3, 4, 5],
            explanation_template: LocalizedText::new(
                "Ils mangent le même nombre de parts ({na}), mais moins le gâteau a de parts, plus elles sont grosses !",
                "They eat the same number of slices ({na}), but fewer slices in a pie means bigger slices!",
            ),
        }),
        // ----- FractionComparison template (multiple denominator) -----
        QuestionDefinition::FractionComparisonTemplate(FractionComparisonTemplate {
            prompt: LocalizedText::new("Qui a mangé le plus de gâteau ?", "Who ate the most pie?"),
            character_a: "Jade".into(),
            character_b: "Noah".into(),
            difficulty: ComparisonDifficulty::MultipleDenominator,
            denominator_range: vec![3, 4, 6, 8, 12],
            numerator_range: vec![1, 2, 3, 4, 5, 6, 7],
            explanation_template: LocalizedText::new(
                "Pour comparer {na}/{da} et {nb}/{db} il faut couper les gâteaux en parts de la même taille.",
                "To compare {na}/{da} and {nb}/{db} we need to cut the pies into same-sized slices.",
            ),
        }),
    ]
}

#[allow(clippy::literal_string_with_formatting_args)]
fn build_multiplication_tables_questions() -> Vec<QuestionDefinition> {
    let mut questions = Vec::new();

    // Two difficulty levels:
    // Beginner: even tables [2,4,6,8], factor_b 1..9
    // Intermediate: odd tables [3,5,7,9], factor_b 1..9
    let levels: &[(Difficulty, &[u32])] = &[
        (Difficulty::Beginner, &[2, 4, 6, 8]),
        (Difficulty::Intermediate, &[3, 5, 7, 9]),
    ];

    let factor_b: Vec<u32> = (1..=9).collect();

    for &(difficulty, tables) in levels {
        let factor_a: Vec<u32> = tables.to_vec();

        let table_label_fr = match difficulty {
            Difficulty::Beginner => "Tables paires (2, 4, 6, 8)",
            Difficulty::Intermediate => "Tables impaires (3, 5, 7, 9)",
            Difficulty::Advanced => "Tables avancées",
        };
        let table_label_en = match difficulty {
            Difficulty::Beginner => "Even tables (2, 4, 6, 8)",
            Difficulty::Intermediate => "Odd tables (3, 5, 7, 9)",
            Difficulty::Advanced => "Advanced tables",
        };

        let teacher_label = Some(LocalizedText::new(table_label_fr, table_label_en));

        // 1. McqTemplate with Multiplication resolver
        questions.push(QuestionDefinition::McqTemplate(McqTemplate {
            prompt_template: LocalizedText::new("Combien font {a} × {b} ?", "What is {a} × {b}?"),
            parameters: vec![
                ParameterRange {
                    name: "a".into(),
                    values: factor_a.iter().map(|&v| v.cast_signed()).collect(),
                },
                ParameterRange {
                    name: "b".into(),
                    values: factor_b.iter().map(|&v| v.cast_signed()).collect(),
                },
            ],
            resolver: McqResolver::Multiplication,
            explanation_template: LocalizedText::new(
                "Dans la grille, on compte {a} lignes de {b}. {a} × {b} = {result}.",
                "In the grid, count {a} rows of {b}. {a} × {b} = {result}.",
            ),
            teacher_label: teacher_label.clone(),
            with_grid: true,
        }));

        // 2. NumericInputTemplate with grid visual
        questions.push(QuestionDefinition::NumericInputTemplate(
            NumericInputTemplate {
                prompt_template: LocalizedText::new("{a} × {b} = ?", "{a} × {b} = ?"),
                factor_a_range: factor_a,
                factor_b_range: factor_b.clone(),
                difficulty,
                explanation_template: LocalizedText::new(
                    "Dans la grille, on compte {a} lignes de {b}. {a} × {b} = {result}.",
                    "In the grid, count {a} rows of {b}. {a} × {b} = {result}.",
                ),
                teacher_label,
                with_grid: true,
                place_value_explanation: false,
            },
        ));
    }

    questions
}

#[allow(clippy::literal_string_with_formatting_args)]
fn build_multiply_10_100_1000_questions() -> Vec<QuestionDefinition> {
    let mut questions = Vec::new();

    let factor_b: Vec<u32> = vec![10, 100, 1000];

    // Three difficulty levels with different factor_a ranges.
    let levels: &[(Difficulty, Vec<u32>)] = &[
        (
            Difficulty::Beginner,
            (1..=9).map(|n| n * 10).collect(), // 10,20,...,90
        ),
        (
            Difficulty::Intermediate,
            (0..10).map(|n| n * 10 + 5).collect(), // 5,15,...,95
        ),
        (
            Difficulty::Advanced,
            (1..=99).filter(|n| n % 5 != 0 && n % 10 != 0).collect(),
        ),
    ];

    for (difficulty, factor_a) in levels {
        let label_fr = match difficulty {
            Difficulty::Beginner => "Dizaines entières × 10/100/1000",
            Difficulty::Intermediate => "Nombres en 5 × 10/100/1000",
            Difficulty::Advanced => "Nombres quelconques × 10/100/1000",
        };
        let label_en = match difficulty {
            Difficulty::Beginner => "Round tens × 10/100/1000",
            Difficulty::Intermediate => "Fives × 10/100/1000",
            Difficulty::Advanced => "Any number × 10/100/1000",
        };

        // 1. NumericInputTemplate
        questions.push(QuestionDefinition::NumericInputTemplate(
            NumericInputTemplate {
                prompt_template: LocalizedText::new("{a} × {b} = ?", "{a} × {b} = ?"),
                factor_a_range: factor_a.clone(),
                factor_b_range: factor_b.clone(),
                difficulty: *difficulty,
                explanation_template: LocalizedText::new(
                    "Multiplier par {b}, c'est ajouter des zéros : {a} × {b} = {result}.",
                    "Multiplying by {b} means adding zeros: {a} × {b} = {result}.",
                ),
                teacher_label: Some(LocalizedText::new(label_fr, label_en)),
                with_grid: false,
                place_value_explanation: true,
            },
        ));

        // 2. McqTemplate with MultiplyByPowerOf10 resolver
        questions.push(QuestionDefinition::McqTemplate(McqTemplate {
            prompt_template: LocalizedText::new("Combien font {a} × {b} ?", "What is {a} × {b}?"),
            parameters: vec![
                ParameterRange {
                    name: "a".into(),
                    values: factor_a.iter().map(|&v| v.cast_signed()).collect(),
                },
                ParameterRange {
                    name: "b".into(),
                    values: factor_b.iter().map(|&v| v.cast_signed()).collect(),
                },
            ],
            resolver: McqResolver::MultiplyByPowerOf10,
            explanation_template: LocalizedText::new(
                "Multiplier par {b}, c'est ajouter des zéros : {a} × {b} = {result}.",
                "Multiplying by {b} means adding zeros: {a} × {b} = {result}.",
            ),
            teacher_label: Some(LocalizedText::new(label_fr, label_en)),
            with_grid: false,
        }));
    }

    questions
}
