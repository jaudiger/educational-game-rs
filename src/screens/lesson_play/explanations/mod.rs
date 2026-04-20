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
use crate::ui::components::standard_button;
use crate::ui::navigation::NavigateTo;
use crate::ui::theme::{self, DesignFontSize};

use super::FeedbackRoot;
use super::visuals::spawn_explanation_visual;

use self::comparison_mult_den::ComparisonMultDenRenderer;
use self::comparison_same_den::ComparisonSameDenRenderer;
use self::comparison_same_num::ComparisonSameNumRenderer;
use self::fraction_addition::FractionAdditionRenderer;
use self::fraction_identification::FractionIdentificationRenderer;
use self::fraction_value::FractionValueRenderer;
use self::fraction_visualization::FractionVisualizationRenderer;
use self::place_value::PlaceValueRenderer;
use self::renderer::{ExplanationRenderer, PlainRenderer};

mod comparison_mult_den;
mod comparison_same_den;
mod comparison_same_num;
mod fraction_addition;
mod fraction_identification;
mod fraction_value;
mod fraction_visualization;
mod place_value;
mod renderer;

/// A resolved feedback explanation: a text renderer plus an optional visual.
/// `Hidden` is returned when the setting is off or the question has neither
/// text nor visual to show.
enum FeedbackExplanation {
    Hidden,
    Visible {
        renderer: Box<dyn ExplanationRenderer>,
        visual: Option<ExplanationVisual>,
    },
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
    let is_last = session.current_index + 1 >= session.questions.len();
    let explanation = session.current().map_or(FeedbackExplanation::Hidden, |q| {
        build_feedback_explanation(settings.show_explanations, &q.definition, i18n.language)
    });

    commands.entity(*container).with_children(|parent| {
        spawn_feedback_content(parent, &i18n, is_correct, is_last, &explanation, window);
    });
}

/// Picks the right renderer for the question and pairs it with the optional
/// decorative visual pulled from the definition.
fn build_feedback_explanation(
    show_explanation: bool,
    definition: &QuestionDefinition,
    language: Language,
) -> FeedbackExplanation {
    if !show_explanation {
        return FeedbackExplanation::Hidden;
    }
    FeedbackExplanation::Visible {
        renderer: build_renderer(definition, language),
        visual: explanation_visual(definition),
    }
}

/// Selects the colour-coded renderer when the question type supports one,
/// falling back to a plain-text renderer otherwise. This is the single
/// dispatch point: adding a new renderer means adding one arm here.
fn build_renderer(
    definition: &QuestionDefinition,
    language: Language,
) -> Box<dyn ExplanationRenderer> {
    if let Some(renderer) = build_colored_renderer(definition) {
        return renderer;
    }
    Box::new(PlainRenderer::new(plain_explanation(definition, language)))
}

fn build_colored_renderer(definition: &QuestionDefinition) -> Option<Box<dyn ExplanationRenderer>> {
    match definition {
        QuestionDefinition::FractionIdentification(d) => Some(Box::new(
            FractionIdentificationRenderer::new(d.numerator, d.denominator),
        )),
        QuestionDefinition::FractionVisualization(d) => Some(Box::new(
            FractionVisualizationRenderer::new(d.numerator, d.denominator),
        )),
        QuestionDefinition::FractionComparison(d) => Some(match d.difficulty {
            ComparisonDifficulty::SameDenominator => Box::new(ComparisonSameDenRenderer::new(
                d.fraction_a.0,
                d.fraction_b.0,
                d.fraction_a.1,
            )) as Box<dyn ExplanationRenderer>,
            ComparisonDifficulty::MultipleDenominator => Box::new(ComparisonMultDenRenderer::new(
                d.fraction_a.0,
                d.fraction_a.1,
                d.fraction_b.0,
                d.fraction_b.1,
            )),
            ComparisonDifficulty::SameNumerator => {
                Box::new(ComparisonSameNumRenderer::new(d.fraction_a.0))
            }
        }),
        QuestionDefinition::Mcq(d) => match d.explanation_visual.as_ref()? {
            ExplanationVisual::FractionAddition { a, b, c } => {
                Some(Box::new(FractionAdditionRenderer::new(*a, *b, *c)))
            }
            ExplanationVisual::WholeFractions { count, denominator } => Some(Box::new(
                FractionValueRenderer::new(count * denominator, *denominator, *count),
            )),
            ExplanationVisual::PlaceValueTable { number, multiplier } => {
                Some(Box::new(PlaceValueRenderer::new(*number, *multiplier)))
            }
            _ => None,
        },
        QuestionDefinition::NumericInput(d) => match d.explanation_visual.as_ref()? {
            ExplanationVisual::PlaceValueTable { number, multiplier } => {
                Some(Box::new(PlaceValueRenderer::new(*number, *multiplier)))
            }
            _ => None,
        },
        _ => None,
    }
}

/// Plain-text fallback pulled from the question's localized explanation
/// string. Templates are resolved before the session is built, so reaching
/// a template here is a bug.
fn plain_explanation(definition: &QuestionDefinition, language: Language) -> String {
    match definition {
        QuestionDefinition::Mcq(d) => d.explanation.get(language).to_owned(),
        QuestionDefinition::FractionVisualization(d) => d.explanation.get(language).to_owned(),
        QuestionDefinition::FractionComparison(d) => d.explanation.get(language).to_owned(),
        QuestionDefinition::FractionIdentification(d) => d.explanation.get(language).to_owned(),
        QuestionDefinition::NumericInput(d) => d.explanation.get(language).to_owned(),
        QuestionDefinition::McqTemplate(_)
        | QuestionDefinition::FractionVisualizationTemplate(_)
        | QuestionDefinition::FractionComparisonTemplate(_)
        | QuestionDefinition::FractionIdentificationTemplate(_)
        | QuestionDefinition::NumericInputTemplate(_) => {
            unreachable!("templates must be resolved before building the session")
        }
    }
}

fn explanation_visual(definition: &QuestionDefinition) -> Option<ExplanationVisual> {
    match definition {
        QuestionDefinition::Mcq(d) => d.explanation_visual.clone(),
        QuestionDefinition::FractionComparison(d) => d.explanation_visual.clone(),
        QuestionDefinition::FractionIdentification(d) => d.explanation_visual.clone(),
        QuestionDefinition::NumericInput(d) => d.explanation_visual.clone(),
        _ => None,
    }
}

fn spawn_feedback_content(
    parent: &mut ChildSpawnerCommands,
    i18n: &I18n,
    is_correct: bool,
    is_last: bool,
    explanation: &FeedbackExplanation,
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
            spawn_result_text(feedback, i18n, is_correct, window);
            spawn_explanation_section(feedback, i18n, explanation, window);
            spawn_next_button(feedback, i18n, is_last, window);
        });
}

fn spawn_explanation_section(
    parent: &mut ChildSpawnerCommands,
    i18n: &I18n,
    explanation: &FeedbackExplanation,
    window: Entity,
) {
    let FeedbackExplanation::Visible { renderer, visual } = explanation else {
        return;
    };

    parent
        .spawn(Node {
            flex_direction: FlexDirection::Column,
            align_items: AlignItems::Center,
            row_gap: theme::scaled(theme::spacing::MEDIUM),
            width: percent(100.0),
            margin: UiRect::axes(Val::ZERO, theme::scaled(theme::spacing::XLARGE)),
            ..default()
        })
        .with_children(|section| {
            renderer.spawn(section, i18n, window);
            if let Some(visual) = visual {
                spawn_explanation_visual(section, visual, window, i18n.language);
            }
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
