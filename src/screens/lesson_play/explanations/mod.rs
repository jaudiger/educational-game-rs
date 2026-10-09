use bevy::prelude::*;
use bevy::window::PrimaryWindow;
use bevy_persistent::prelude::Persistent;

use crate::data::content::{LocalizedExplanation, QuestionDefinition};
use crate::data::{
    AnswerResult, ExplanationVisual, GameSettings, LastAnswer, LessonSession, QuestionContainer,
    SaveWriteAction, SaveWriteStatus,
};
use crate::i18n::{I18n, TranslationKey};
use crate::states::LessonPhase;
use crate::ui::animation::AnimateScale;
use crate::ui::components::{save_write_failure_notice, standard_button};
use crate::ui::navigation::NavigateTo;
use crate::ui::theme;

use super::FeedbackRoot;
use super::visuals::spawn_explanation_visual;

use self::renderer::{ComparisonMath, spawn_comparison_math, spawn_explanation_text};

mod renderer;

enum FeedbackExplanation {
    Hidden,
    Visible {
        text: LocalizedExplanation,
        comparison: Option<ComparisonMath>,
        visual: Option<ExplanationVisual>,
    },
}

/// Spawns feedback UI after `record_answer` has updated the session and save data.
#[allow(clippy::too_many_arguments)]
pub(super) fn setup_feedback_ui(
    mut commands: Commands,
    container: Single<Entity, With<QuestionContainer>>,
    last_answer: Res<LastAnswer>,
    session: Res<LessonSession>,
    settings: Res<Persistent<GameSettings>>,
    i18n: Res<I18n>,
    primary_window: Single<Entity, With<PrimaryWindow>>,
    write_status: Res<SaveWriteStatus>,
) {
    let window = *primary_window;
    let is_correct = matches!(**last_answer, AnswerResult::Correct);
    let is_last = session.current_index + 1 >= session.questions.len();
    let save_failed = write_status.failed(SaveWriteAction::ClassAnswer);
    let explanation = session.current().map_or(FeedbackExplanation::Hidden, |q| {
        build_feedback_explanation(settings.show_explanations, &q.definition)
    });

    commands.entity(*container).with_children(|parent| {
        spawn_feedback_content(
            parent,
            &i18n,
            is_correct,
            is_last,
            save_failed,
            &explanation,
            window,
        );
    });
}

fn build_feedback_explanation(
    show_explanation: bool,
    definition: &QuestionDefinition,
) -> FeedbackExplanation {
    let text = localized_explanation(definition);
    for issue in text.validation_issues() {
        bevy::log::warn!("Invalid localized explanation: {issue}");
    }
    if !show_explanation {
        return FeedbackExplanation::Hidden;
    }
    FeedbackExplanation::Visible {
        text,
        comparison: comparison_math(definition),
        visual: explanation_visual(definition),
    }
}

fn localized_explanation(definition: &QuestionDefinition) -> LocalizedExplanation {
    match definition {
        QuestionDefinition::Mcq(d) => d.explanation.clone(),
        QuestionDefinition::FractionVisualization(d) => d.explanation.clone(),
        QuestionDefinition::FractionComparison(d) => d.explanation.clone(),
        QuestionDefinition::FractionIdentification(d) => d.explanation.clone(),
        QuestionDefinition::NumericInput(d) => d.explanation.clone(),
        QuestionDefinition::McqTemplate(_)
        | QuestionDefinition::FractionVisualizationTemplate(_)
        | QuestionDefinition::FractionComparisonTemplate(_)
        | QuestionDefinition::FractionIdentificationTemplate(_)
        | QuestionDefinition::NumericInputTemplate(_) => {
            unreachable!("templates must be resolved before building the session")
        }
    }
}

const fn comparison_math(definition: &QuestionDefinition) -> Option<ComparisonMath> {
    match definition {
        QuestionDefinition::FractionComparison(question) => Some(ComparisonMath::new(question)),
        _ => None,
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
    save_failed: bool,
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
            if save_failed {
                feedback.spawn(save_write_failure_notice(
                    i18n.t(&TranslationKey::SaveWriteFailed).into_owned(),
                    window,
                ));
            }
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
    let FeedbackExplanation::Visible {
        text,
        comparison,
        visual,
    } = explanation
    else {
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
            spawn_explanation_text(section, text, i18n.language, theme::fonts::HEADING, window);
            if let Some(comparison) = comparison {
                spawn_comparison_math(section, comparison, theme::fonts::HEADING, window);
            }
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
        theme::typography::text(theme::fonts::TITLE, window),
        TextColor(color),
        UiTransform::from_scale(Vec2::ZERO),
        AnimateScale::new(0.0, 1.0, ease_fn, duration),
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
