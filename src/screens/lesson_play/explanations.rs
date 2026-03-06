use bevy::prelude::*;
use bevy::window::PrimaryWindow;
use bevy_persistent::prelude::Persistent;

use crate::data::content::{ComparisonDifficulty, QuestionDefinition};
use crate::data::{
    AnswerResult, ExplanationVisual, GameSettings, Language, LastAnswer, LessonSession,
    QuestionContainer,
};
use crate::i18n::{I18n, TranslationKey};
use crate::states::LessonPhase;
use crate::ui::animation::AnimateScale;
use crate::ui::components::{stacked_fraction, standard_button};
use crate::ui::navigation::NavigateTo;
use crate::ui::rich_text::spawn_rich_text;
use crate::ui::theme::{self, DesignFontSize};

use super::FeedbackRoot;
use super::visuals::{count_trailing_zeros, spawn_explanation_visual};

/// Data needed to render a colour-coded explanation text that matches the
/// visual elements shown alongside it.
enum ColoredExplanation {
    /// Fraction identification / visualization: numerator in blue, denominator
    /// in orange, same colours carried into the inline stacked fraction.
    FractionIdentification {
        numerator: u32,
        denominator: u32,
    },
    FractionVisualization {
        numerator: u32,
        denominator: u32,
    },
    /// Same-denominator comparison: `na` in blue (character A), `nb` in orange
    /// (character B), matching the fraction bar colours.
    ComparisonSameDenominator {
        na: u32,
        nb: u32,
        denominator: u32,
    },
    /// Multiple-denominator comparison: fraction A in blue, fraction B in
    /// orange, rendered as coloured stacked fractions.
    ComparisonMultipleDenominator {
        na: u32,
        da: u32,
        nb: u32,
        db: u32,
    },
    /// Same-numerator comparison: shared numerator highlighted in blue.
    ComparisonSameNumerator {
        numerator: u32,
    },
    /// Fraction addition: `a` in blue, `c` in orange, sum/result in green,
    /// matching the three fraction bars.
    FractionAddition {
        a: u32,
        b: u32,
        c: u32,
    },
    /// Fraction value (a/b = result): `a` in blue, `b` in orange, `result` in
    /// green, matching the whole-fraction bars.
    FractionValue {
        a: u32,
        b: u32,
        result: u32,
    },
}

struct FeedbackContent<'a> {
    pub is_correct: bool,
    pub show_explanation: bool,
    pub explanation: Option<&'a str>,
    pub explanation_visual: Option<&'a ExplanationVisual>,
    pub colored_explanation: Option<&'a ColoredExplanation>,
    pub is_last: bool,
}

/// Spawns feedback UI after `record_answer` has updated the session and save data.
pub(super) fn setup_feedback_ui(
    mut commands: Commands,
    container: Single<Entity, With<QuestionContainer>>,
    last_answer: Res<LastAnswer>,
    session: Res<LessonSession>,
    settings: Res<Persistent<GameSettings>>,
    i18n: Res<I18n>,
    primary_window: Single<Entity, With<PrimaryWindow>>,
) {
    let window = *primary_window;

    let is_correct = matches!(**last_answer, AnswerResult::Correct);
    let explanation = current_explanation(&session, i18n.language);
    let explanation_visual = current_explanation_visual(&session);
    let colored_explanation = current_colored_explanation(&session);
    let show_explanation = settings.show_explanations;
    let is_last = session.current_index + 1 >= session.questions.len();

    commands.entity(*container).with_children(|parent| {
        spawn_feedback_content(
            parent,
            &i18n,
            &FeedbackContent {
                is_correct,
                show_explanation,
                explanation: explanation.as_deref(),
                explanation_visual: explanation_visual.as_ref(),
                colored_explanation: colored_explanation.as_ref(),
                is_last,
            },
            window,
        );
    });
}

fn current_explanation(session: &LessonSession, language: Language) -> Option<String> {
    session
        .questions
        .get(session.current_index)
        .map(|q| match &q.definition {
            QuestionDefinition::Mcq(d) => d.explanation.get(language).to_owned(),
            QuestionDefinition::FractionVisualization(d) => d.explanation.get(language).to_owned(),
            QuestionDefinition::FractionComparison(d) => d.explanation.get(language).to_owned(),
            QuestionDefinition::FractionIdentification(d) => d.explanation.get(language).to_owned(),
            QuestionDefinition::NumericInput(d) => d.explanation.get(language).to_owned(),
            // Templates are resolved before the session is built.
            QuestionDefinition::McqTemplate(_)
            | QuestionDefinition::FractionVisualizationTemplate(_)
            | QuestionDefinition::FractionComparisonTemplate(_)
            | QuestionDefinition::FractionIdentificationTemplate(_)
            | QuestionDefinition::NumericInputTemplate(_) => {
                unreachable!("templates must be resolved before building the session")
            }
        })
}

fn current_explanation_visual(session: &LessonSession) -> Option<ExplanationVisual> {
    session
        .questions
        .get(session.current_index)
        .and_then(|q| match &q.definition {
            QuestionDefinition::Mcq(d) => d.explanation_visual.clone(),
            QuestionDefinition::FractionComparison(d) => d.explanation_visual.clone(),
            QuestionDefinition::FractionIdentification(d) => d.explanation_visual.clone(),
            QuestionDefinition::NumericInput(d) => d.explanation_visual.clone(),
            _ => None,
        })
}

/// Extracts colour-coded explanation data from the current question so the
/// explanation renderer can highlight numbers that match the visual elements.
fn current_colored_explanation(session: &LessonSession) -> Option<ColoredExplanation> {
    session
        .questions
        .get(session.current_index)
        .and_then(|q| match &q.definition {
            QuestionDefinition::FractionIdentification(d) => {
                Some(ColoredExplanation::FractionIdentification {
                    numerator: d.numerator,
                    denominator: d.denominator,
                })
            }
            QuestionDefinition::FractionVisualization(d) => {
                Some(ColoredExplanation::FractionVisualization {
                    numerator: d.numerator,
                    denominator: d.denominator,
                })
            }
            QuestionDefinition::FractionComparison(d) => match d.difficulty {
                ComparisonDifficulty::SameDenominator => {
                    Some(ColoredExplanation::ComparisonSameDenominator {
                        na: d.fraction_a.0,
                        nb: d.fraction_b.0,
                        denominator: d.fraction_a.1,
                    })
                }
                ComparisonDifficulty::MultipleDenominator => {
                    Some(ColoredExplanation::ComparisonMultipleDenominator {
                        na: d.fraction_a.0,
                        da: d.fraction_a.1,
                        nb: d.fraction_b.0,
                        db: d.fraction_b.1,
                    })
                }
                ComparisonDifficulty::SameNumerator => {
                    Some(ColoredExplanation::ComparisonSameNumerator {
                        numerator: d.fraction_a.0,
                    })
                }
            },
            QuestionDefinition::Mcq(d) => {
                if let Some(ExplanationVisual::FractionAddition { a, b, c }) = &d.explanation_visual
                {
                    Some(ColoredExplanation::FractionAddition {
                        a: *a,
                        b: *b,
                        c: *c,
                    })
                } else if let Some(ExplanationVisual::WholeFractions { count, denominator }) =
                    &d.explanation_visual
                {
                    Some(ColoredExplanation::FractionValue {
                        a: count * denominator,
                        b: *denominator,
                        result: *count,
                    })
                } else {
                    None
                }
            }
            _ => None,
        })
}

fn spawn_feedback_content(
    parent: &mut ChildSpawnerCommands,
    i18n: &I18n,
    content: &FeedbackContent<'_>,
    window: Entity,
) {
    parent
        .spawn((
            FeedbackRoot,
            DespawnOnEnter(LessonPhase::ShowQuestion),
            Node {
                flex_direction: FlexDirection::Column,
                align_items: AlignItems::Center,
                row_gap: theme::scaled(theme::spacing::MEDIUM),
                width: percent(100.0),
                ..default()
            },
        ))
        .with_children(|feedback| {
            spawn_result_text(feedback, i18n, content.is_correct, window);

            if content.show_explanation
                && (content.explanation.is_some() || content.explanation_visual.is_some())
            {
                feedback
                    .spawn(Node {
                        flex_direction: FlexDirection::Column,
                        align_items: AlignItems::Center,
                        row_gap: theme::scaled(theme::spacing::MEDIUM),
                        width: percent(100.0),
                        margin: UiRect::axes(Val::ZERO, theme::scaled(theme::spacing::XLARGE)),
                        ..default()
                    })
                    .with_children(|section| {
                        if let Some(
                            visual @ ExplanationVisual::PlaceValueTable { number, multiplier },
                        ) = content.explanation_visual
                        {
                            spawn_place_value_explanation_text(
                                section,
                                i18n,
                                *number,
                                *multiplier,
                                window,
                            );
                            spawn_explanation_visual(section, visual, window, i18n.language);
                        } else {
                            if let Some(colored) = content.colored_explanation {
                                spawn_colored_explanation_text(section, i18n, colored, window);
                            } else if let Some(explanation) = content.explanation {
                                spawn_explanation_text(section, i18n, explanation, window);
                            }
                            if let Some(visual) = content.explanation_visual {
                                spawn_explanation_visual(section, visual, window, i18n.language);
                            }
                        }
                    });
            }

            spawn_next_button(feedback, i18n, content.is_last, window);
        });
}

fn spawn_result_text(
    parent: &mut ChildSpawnerCommands,
    i18n: &I18n,
    is_correct: bool,
    window: Entity,
) {
    use bevy::math::curve::easing::EaseFunction;

    let (key, color, ease_fn, duration) = if is_correct {
        (
            TranslationKey::CorrectAnswer,
            theme::colors::SUCCESS,
            EaseFunction::BackOut,
            theme::animation::FEEDBACK_CORRECT_DURATION,
        )
    } else {
        (
            TranslationKey::IncorrectAnswer,
            theme::colors::ERROR,
            EaseFunction::CubicOut,
            theme::animation::FEEDBACK_INCORRECT_DURATION,
        )
    };

    parent.spawn((
        Text::new(i18n.t(&key)),
        TextFont {
            font_size: theme::fonts::TITLE,
            ..default()
        },
        TextColor(color),
        UiTransform::from_scale(Vec2::ZERO),
        AnimateScale::new(0.0, 1.0, ease_fn, duration),
        DesignFontSize {
            size: theme::fonts::TITLE,
            window,
        },
    ));
}

fn spawn_explanation_text(
    parent: &mut ChildSpawnerCommands,
    i18n: &I18n,
    explanation: &str,
    window: Entity,
) {
    let text = format!("{} {explanation}", i18n.t(&TranslationKey::Explanation));
    spawn_rich_text(
        parent,
        &text,
        theme::fonts::HEADING,
        theme::colors::TEXT_DARK,
        window,
    );
}

/// Dispatches to the appropriate colour-coded explanation renderer based on
/// the question type.
fn spawn_colored_explanation_text(
    parent: &mut ChildSpawnerCommands,
    i18n: &I18n,
    colored: &ColoredExplanation,
    window: Entity,
) {
    match *colored {
        ColoredExplanation::FractionIdentification {
            numerator,
            denominator,
        } => spawn_colored_fraction_parts_text(parent, i18n, numerator, denominator, false, window),
        ColoredExplanation::FractionVisualization {
            numerator,
            denominator,
        } => spawn_colored_fraction_parts_text(parent, i18n, numerator, denominator, true, window),
        ColoredExplanation::ComparisonSameDenominator {
            na,
            nb,
            denominator,
        } => spawn_colored_comparison_same_den_text(parent, i18n, na, nb, denominator, window),
        ColoredExplanation::ComparisonMultipleDenominator { na, da, nb, db } => {
            spawn_colored_comparison_mult_den_text(parent, i18n, na, da, nb, db, window);
        }
        ColoredExplanation::ComparisonSameNumerator { numerator } => {
            spawn_colored_comparison_same_num_text(parent, i18n, numerator, window);
        }
        ColoredExplanation::FractionAddition { a, b, c } => {
            spawn_colored_addition_text(parent, i18n, a, b, c, window);
        }
        ColoredExplanation::FractionValue { a, b, result } => {
            spawn_colored_fraction_value_text(parent, i18n, a, b, result, window);
        }
    }
}

/// Two-layer centering row used by all colour-coded explanation renderers.
/// The outer row stretches to the parent width and centres the inner row;
/// the inner row wraps at word boundaries and sizes to its content.
fn spawn_colored_row(
    parent: &mut ChildSpawnerCommands,
    font_size: f32,
    build: impl FnOnce(&mut ChildSpawnerCommands),
) {
    parent
        .spawn(Node {
            align_self: AlignSelf::Stretch,
            flex_direction: FlexDirection::Row,
            justify_content: JustifyContent::Center,
            ..default()
        })
        .with_children(|wrapper| {
            wrapper
                .spawn(Node {
                    flex_direction: FlexDirection::Row,
                    align_items: AlignItems::Center,
                    flex_wrap: FlexWrap::Wrap,
                    column_gap: theme::scaled(font_size * 0.28),
                    row_gap: theme::scaled(theme::spacing::SMALL),
                    ..default()
                })
                .with_children(build);
        });
}

/// Spawns each whitespace-separated word as a separate [`Text`] entity so the
/// flex row can wrap at word boundaries.
fn spawn_words(
    row: &mut ChildSpawnerCommands,
    text: &str,
    color: Color,
    font_size: f32,
    window: Entity,
) {
    for word in text.split_whitespace() {
        row.spawn((
            Text::new(word.to_owned()),
            TextFont {
                font_size,
                ..default()
            },
            TextColor(color),
            DesignFontSize {
                size: font_size,
                window,
            },
        ));
    }
}

/// Numerator in blue, denominator in orange, same colours in the inline
/// stacked fraction.
fn spawn_colored_fraction_parts_text(
    parent: &mut ChildSpawnerCommands,
    i18n: &I18n,
    numerator: u32,
    denominator: u32,
    is_visualization: bool,
    window: Entity,
) {
    let font_size = theme::fonts::HEADING;
    let num_color = theme::colors::PRIMARY;
    let den_color = theme::colors::SECONDARY;
    let dark = theme::colors::TEXT_DARK;

    let num_str = numerator.to_string();
    let den_str = denominator.to_string();

    let (before_den, between, after_num) = match (i18n.language, is_visualization) {
        (Language::French, false) => (
            format!("{} Il y a", i18n.t(&TranslationKey::Explanation)),
            "parts et".to_owned(),
            "sont coloriées : c'est".to_owned(),
        ),
        (Language::French, true) => (
            format!("{} Il y a", i18n.t(&TranslationKey::Explanation)),
            "parts au total et".to_owned(),
            "sont coloriées : c'est la fraction".to_owned(),
        ),
        (Language::English, false) => (
            format!("{} There are", i18n.t(&TranslationKey::Explanation)),
            "parts and".to_owned(),
            "are colored: that's".to_owned(),
        ),
        (Language::English, true) => (
            format!("{} There are", i18n.t(&TranslationKey::Explanation)),
            "parts in total and".to_owned(),
            "are colored: that's the fraction".to_owned(),
        ),
    };

    spawn_colored_row(parent, font_size, |row| {
        spawn_words(row, &before_den, dark, font_size, window);
        spawn_words(row, &den_str, den_color, font_size, window);
        spawn_words(row, &between, dark, font_size, window);
        spawn_words(row, &num_str, num_color, font_size, window);
        spawn_words(row, &after_num, dark, font_size, window);
        row.spawn(Node {
            flex_direction: FlexDirection::Row,
            align_items: AlignItems::Center,
            column_gap: theme::scaled(font_size * 0.28),
            ..default()
        })
        .with_children(|group| {
            group.spawn(stacked_fraction(
                numerator,
                denominator,
                font_size,
                num_color,
                den_color,
                window,
            ));
            group.spawn((
                Text::new("."),
                TextFont {
                    font_size,
                    ..default()
                },
                TextColor(dark),
                DesignFontSize {
                    size: font_size,
                    window,
                },
            ));
        });
    });
}

/// `na` in blue (character A), `nb` in orange (character B).
#[allow(clippy::similar_names)]
fn spawn_colored_comparison_same_den_text(
    parent: &mut ChildSpawnerCommands,
    i18n: &I18n,
    na: u32,
    nb: u32,
    denominator: u32,
    window: Entity,
) {
    let font_size = theme::fonts::HEADING;
    let a_color = theme::colors::PRIMARY;
    let b_color = theme::colors::SECONDARY;
    let dark = theme::colors::TEXT_DARK;

    let na_str = na.to_string();
    let nb_str = nb.to_string();
    let den_str = denominator.to_string();

    let (intro, middle, end) = match i18n.language {
        Language::French => (
            format!(
                "{} Les gâteaux ont les mêmes parts ({den_str}), on compare juste le nombre de parts :",
                i18n.t(&TranslationKey::Explanation)
            ),
            "et",
            ".",
        ),
        Language::English => (
            format!(
                "{} The pies have the same slices ({den_str}), so we compare how many slices:",
                i18n.t(&TranslationKey::Explanation)
            ),
            "and",
            ".",
        ),
    };

    spawn_colored_row(parent, font_size, |row| {
        spawn_words(row, &intro, dark, font_size, window);
        spawn_words(row, &na_str, a_color, font_size, window);
        spawn_words(row, middle, dark, font_size, window);
        spawn_words(row, &nb_str, b_color, font_size, window);
        spawn_words(row, end, dark, font_size, window);
    });
}

/// Fraction A in blue, fraction B in orange, rendered as coloured stacked
/// fractions.
fn spawn_colored_comparison_mult_den_text(
    parent: &mut ChildSpawnerCommands,
    i18n: &I18n,
    na: u32,
    da: u32,
    nb: u32,
    db: u32,
    window: Entity,
) {
    let font_size = theme::fonts::HEADING;
    let dark = theme::colors::TEXT_DARK;

    let (intro, middle, conclusion) = match i18n.language {
        Language::French => (
            format!("{} Pour comparer", i18n.t(&TranslationKey::Explanation)),
            "et",
            "il faut couper les gâteaux en parts de la même taille.",
        ),
        Language::English => (
            format!("{} To compare", i18n.t(&TranslationKey::Explanation)),
            "and",
            "we need to cut the pies into same-sized slices.",
        ),
    };

    spawn_colored_row(parent, font_size, |row| {
        spawn_words(row, &intro, dark, font_size, window);
        row.spawn(stacked_fraction(na, da, font_size, dark, dark, window));
        spawn_words(row, middle, dark, font_size, window);
        row.spawn(stacked_fraction(nb, db, font_size, dark, dark, window));
        spawn_words(row, conclusion, dark, font_size, window);
    });
}

/// `a` in blue, `c` in orange, sum/result in green, matching the three
/// fraction bars in the visual.
fn spawn_colored_addition_text(
    parent: &mut ChildSpawnerCommands,
    i18n: &I18n,
    a: u32,
    b: u32,
    c: u32,
    window: Entity,
) {
    let font_size = theme::fonts::HEADING;
    let a_color = theme::colors::PRIMARY;
    let c_color = theme::colors::SECONDARY;
    let sum_color = theme::colors::SUCCESS;
    let dark = theme::colors::TEXT_DARK;

    let sum = a + c;
    let a_str = a.to_string();
    let b_str = b.to_string();
    let c_str = c.to_string();
    let sum_str = sum.to_string();

    let (intro, conclusion) = match i18n.language {
        Language::French => (
            format!(
                "{} Les deux fractions ont le même dénominateur ({b_str}), on additionne les numérateurs :",
                i18n.t(&TranslationKey::Explanation)
            ),
            "Donc",
        ),
        Language::English => (
            format!(
                "{} Both fractions have the same denominator ({b_str}), so we add the numerators:",
                i18n.t(&TranslationKey::Explanation)
            ),
            "So",
        ),
    };

    spawn_colored_row(parent, font_size, |row| {
        spawn_words(row, &intro, dark, font_size, window);
        spawn_words(row, &a_str, a_color, font_size, window);
        spawn_words(row, "+", dark, font_size, window);
        spawn_words(row, &c_str, c_color, font_size, window);
        spawn_words(row, "=", dark, font_size, window);
        spawn_words(row, &sum_str, sum_color, font_size, window);
        spawn_words(row, ".", dark, font_size, window);
        spawn_words(row, conclusion, dark, font_size, window);
        row.spawn(stacked_fraction(a, b, font_size, a_color, dark, window));
        spawn_words(row, "+", dark, font_size, window);
        row.spawn(stacked_fraction(c, b, font_size, c_color, dark, window));
        spawn_words(row, "=", dark, font_size, window);
        row.spawn(Node {
            flex_direction: FlexDirection::Row,
            align_items: AlignItems::Center,
            column_gap: theme::scaled(font_size * 0.28),
            ..default()
        })
        .with_children(|group| {
            group.spawn(stacked_fraction(sum, b, font_size, sum_color, dark, window));
            group.spawn((
                Text::new("."),
                TextFont {
                    font_size,
                    ..default()
                },
                TextColor(dark),
                DesignFontSize {
                    size: font_size,
                    window,
                },
            ));
        });
    });
}

/// Shared numerator highlighted in blue.
fn spawn_colored_comparison_same_num_text(
    parent: &mut ChildSpawnerCommands,
    i18n: &I18n,
    numerator: u32,
    window: Entity,
) {
    let font_size = theme::fonts::HEADING;
    let num_color = theme::colors::PRIMARY;
    let dark = theme::colors::TEXT_DARK;

    let num_str = numerator.to_string();

    let (intro, middle, end) = match i18n.language {
        Language::French => (
            format!(
                "{} Ils mangent le même nombre de parts (",
                i18n.t(&TranslationKey::Explanation)
            ),
            "), mais moins le gâteau a de parts, plus elles sont grosses",
            "!",
        ),
        Language::English => (
            format!(
                "{} They eat the same number of slices (",
                i18n.t(&TranslationKey::Explanation)
            ),
            "), but fewer slices in a pie means bigger slices",
            "!",
        ),
    };

    spawn_colored_row(parent, font_size, |row| {
        spawn_words(row, &intro, dark, font_size, window);
        spawn_words(row, &num_str, num_color, font_size, window);
        spawn_words(row, middle, dark, font_size, window);
        spawn_words(row, end, dark, font_size, window);
    });
}

/// `a` in blue, `b` in orange, `result` in green.
fn spawn_colored_fraction_value_text(
    parent: &mut ChildSpawnerCommands,
    i18n: &I18n,
    a: u32,
    b: u32,
    result: u32,
    window: Entity,
) {
    let font_size = theme::fonts::HEADING;
    let a_color = theme::colors::PRIMARY;
    let b_color = theme::colors::SECONDARY;
    let result_color = theme::colors::SUCCESS;
    let dark = theme::colors::TEXT_DARK;

    let a_str = a.to_string();
    let b_str = b.to_string();
    let result_str = result.to_string();

    let (intro, mid1, mid2, conclusion) = match i18n.language {
        Language::French => (
            format!("{} Si on partage", i18n.t(&TranslationKey::Explanation)),
            "parts en groupes de",
            ", on obtient",
            "groupes. Donc",
        ),
        Language::English => (
            format!("{} If we share", i18n.t(&TranslationKey::Explanation)),
            "parts into groups of",
            ", we get",
            "groups. So",
        ),
    };

    spawn_colored_row(parent, font_size, |row| {
        spawn_words(row, &intro, dark, font_size, window);
        spawn_words(row, &a_str, a_color, font_size, window);
        spawn_words(row, mid1, dark, font_size, window);
        spawn_words(row, &b_str, b_color, font_size, window);
        spawn_words(row, mid2, dark, font_size, window);
        spawn_words(row, &result_str, result_color, font_size, window);
        spawn_words(row, conclusion, dark, font_size, window);
        row.spawn(stacked_fraction(a, b, font_size, a_color, b_color, window));
        spawn_words(row, "=", dark, font_size, window);
        spawn_words(row, &result_str, result_color, font_size, window);
        spawn_words(row, ".", dark, font_size, window);
    });
}

/// Renders the explanation text for a place-value multiplication with the
/// trailing zeros of the multiplier and result displayed in orange.
fn spawn_place_value_explanation_text(
    parent: &mut ChildSpawnerCommands,
    i18n: &I18n,
    number: u32,
    multiplier: u32,
    window: Entity,
) {
    use super::visuals::PV_ZERO_COLOR;

    let result = number * multiplier;
    let zeros_added = count_trailing_zeros(multiplier);

    let mult_str = multiplier.to_string();
    let result_str = result.to_string();

    // Split multiplier: "1" + "00"
    let mult_prefix = &mult_str[..mult_str.len() - zeros_added];
    let mult_zeros = &mult_str[mult_str.len() - zeros_added..];

    // Split result: "23" + "00"
    let result_prefix = &result_str[..result_str.len() - zeros_added];
    let result_zeros = &result_str[result_str.len() - zeros_added..];

    let number_str = number.to_string();

    let (before_mult, between, after_eq) = match i18n.language {
        Language::French => (
            format!("{} Multiplier par ", i18n.t(&TranslationKey::Explanation)),
            format!(" c'est ajouter des zéros : {number_str} × {mult_prefix}"),
            ".".to_owned(),
        ),
        Language::English => (
            format!("{} Multiplying by ", i18n.t(&TranslationKey::Explanation)),
            format!(" means adding zeros: {number_str} × {mult_prefix}"),
            ".".to_owned(),
        ),
    };

    let text_span = |s: &str, color: Color| {
        (
            Text::new(s.to_owned()),
            TextFont {
                font_size: theme::fonts::HEADING,
                ..default()
            },
            TextColor(color),
            DesignFontSize {
                size: theme::fonts::HEADING,
                window,
            },
        )
    };

    // Two-layer structure: outer row (stretch to parent width, centers child)
    // + inner wrapping row (width: auto, wraps at outer's width, sizes to content).
    parent
        .spawn(Node {
            align_self: AlignSelf::Stretch,
            flex_direction: FlexDirection::Row,
            justify_content: JustifyContent::Center,
            ..default()
        })
        .with_children(|wrapper| {
            wrapper
                .spawn(Node {
                    flex_direction: FlexDirection::Row,
                    flex_wrap: FlexWrap::Wrap,
                    align_items: AlignItems::Center,
                    ..default()
                })
                .with_children(|row| {
                    row.spawn(text_span(&before_mult, theme::colors::TEXT_DARK));
                    row.spawn(text_span(mult_prefix, theme::colors::TEXT_DARK));
                    row.spawn(text_span(mult_zeros, PV_ZERO_COLOR));
                    row.spawn(text_span(&between, theme::colors::TEXT_DARK));
                    row.spawn(text_span(mult_zeros, PV_ZERO_COLOR));
                    row.spawn(text_span(" = ", theme::colors::TEXT_DARK));
                    row.spawn(text_span(result_prefix, theme::colors::TEXT_DARK));
                    row.spawn(text_span(result_zeros, PV_ZERO_COLOR));
                    row.spawn(text_span(&after_eq, theme::colors::TEXT_DARK));
                });
        });
}

fn spawn_next_button(
    parent: &mut ChildSpawnerCommands,
    i18n: &I18n,
    is_last: bool,
    window: Entity,
) {
    let key = if is_last {
        TranslationKey::FinishLesson
    } else {
        TranslationKey::NextQuestion
    };

    parent.spawn((
        standard_button(
            &i18n.t(&key),
            theme::colors::PRIMARY,
            theme::scaled(theme::sizes::BUTTON_WIDTH),
            window,
        ),
        NavigateTo(LessonPhase::Transitioning),
    ));
}
