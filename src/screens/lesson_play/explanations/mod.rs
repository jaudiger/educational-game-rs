use bevy::prelude::*;
use bevy::window::PrimaryWindow;

use crate::data::content::{LocalizedExplanation, QuestionInstance};
use crate::data::{
    AnswerResult, ExplanationVisual, GameSettings, LastAnswer, LessonSession, QuestionContainer,
};
use crate::i18n::{I18n, TranslationKey};
use crate::questions::{spawn_controls_card, spawn_question_card};
use crate::states::LessonPhase;
use crate::ui::animation::AnimateScale;
use crate::ui::components::wrapping_standard_button;
use crate::ui::navigation::NavigateTo;
use crate::ui::theme;

use super::FeedbackRoot;
use super::visuals::spawn_explanation_visual;

use self::renderer::spawn_explanation_text;

mod renderer;

enum FeedbackExplanation {
    Hidden,
    Visible {
        text: LocalizedExplanation,
        visual: Option<ExplanationVisual>,
    },
}

#[derive(Resource, Clone, Debug, Default, PartialEq, Eq)]
pub struct FeedbackPresentation {
    pub next_button_label: String,
    pub return_to_catalogue: bool,
    pub save_notice: Option<String>,
}

/// Spawns shared answer feedback for the active exercise.
#[allow(clippy::too_many_arguments)]
pub(super) fn setup_feedback_ui(
    mut commands: Commands,
    container: Single<Entity, With<QuestionContainer>>,
    last_answer: Res<LastAnswer>,
    session: Res<LessonSession>,
    settings: Res<GameSettings>,
    i18n: Res<I18n>,
    presentation: Res<FeedbackPresentation>,
    primary_window: Single<Entity, With<PrimaryWindow>>,
) {
    let window = *primary_window;
    let is_correct = matches!(**last_answer, AnswerResult::Correct);
    let explanation = session
        .current()
        .map_or(FeedbackExplanation::Hidden, |question| {
            build_feedback_explanation(settings.show_explanations, &question.instance)
        });

    commands.entity(*container).with_children(|parent| {
        spawn_feedback_content(
            parent,
            &i18n,
            is_correct,
            &presentation,
            &explanation,
            window,
        );
    });
}

fn build_feedback_explanation(
    show_explanation: bool,
    definition: &QuestionInstance,
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
        visual: explanation_visual(definition),
    }
}

fn localized_explanation(definition: &QuestionInstance) -> LocalizedExplanation {
    match definition {
        QuestionInstance::Mcq(d) => d.explanation.clone(),
        QuestionInstance::FractionVisualization(d) => d.explanation.clone(),
        QuestionInstance::FractionComparison(d) => d.explanation.clone(),
        QuestionInstance::FractionIdentification(d) => d.explanation.clone(),
        QuestionInstance::NumericInput(d) => d.explanation.clone(),
    }
}

fn explanation_visual(definition: &QuestionInstance) -> Option<ExplanationVisual> {
    match definition {
        QuestionInstance::Mcq(d) => d.explanation_visual.clone(),
        QuestionInstance::FractionComparison(d) => d.explanation_visual.clone(),
        QuestionInstance::FractionIdentification(d) => d.explanation_visual.clone(),
        QuestionInstance::NumericInput(d) => d.explanation_visual.clone(),
        QuestionInstance::FractionVisualization(_) => None,
    }
}

fn spawn_feedback_content(
    parent: &mut ChildSpawnerCommands,
    i18n: &I18n,
    is_correct: bool,
    presentation: &FeedbackPresentation,
    explanation: &FeedbackExplanation,
    window: Entity,
) {
    parent
        .spawn((
            FeedbackRoot,
            DespawnOnEnter(LessonPhase::ShowQuestion),
            UiTransform::default(),
            Node {
                flex_direction: FlexDirection::Column,
                align_items: AlignItems::Center,
                row_gap: theme::scaled(theme::spacing::LARGE),
                width: theme::layout::FILL_PARENT,
                max_width: theme::scaled(960.0),
                margin: UiRect::vertical(Val::Auto),
                ..default()
            },
        ))
        .with_children(|feedback| {
            spawn_question_card(feedback, |card| {
                spawn_result_text(card, i18n, is_correct, window);
                spawn_explanation_section(card, i18n, explanation, window);
            });
            spawn_controls_card(feedback, |card| {
                if let Some(notice) = &presentation.save_notice {
                    card.spawn((
                        Text::new(notice.clone()),
                        theme::typography::text(theme::fonts::SMALL, window),
                        TextColor(theme::colors::ERROR),
                    ));
                }
                spawn_next_button(card, presentation, window);
            });
        });
}

fn spawn_explanation_section(
    parent: &mut ChildSpawnerCommands,
    i18n: &I18n,
    explanation: &FeedbackExplanation,
    window: Entity,
) {
    let FeedbackExplanation::Visible { text, visual } = explanation else {
        return;
    };

    parent
        .spawn(Node {
            flex_direction: FlexDirection::Column,
            align_items: AlignItems::Center,
            row_gap: theme::scaled(theme::spacing::MEDIUM),
            width: theme::layout::FILL_PARENT,
            ..default()
        })
        .with_children(|section| {
            spawn_explanation_text(section, text, i18n.language, theme::fonts::HEADING, window);
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
    use bevy::curve::easing::EaseFunction;

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
    presentation: &FeedbackPresentation,
    window: Entity,
) {
    parent.spawn((
        wrapping_standard_button(
            &presentation.next_button_label,
            if presentation.return_to_catalogue {
                theme::button_styles::BACK
            } else {
                theme::button_styles::ACTION_PRIMARY
            },
            theme::scaled(theme::sizes::BUTTON_WIDTH),
            window,
        ),
        NavigateTo(LessonPhase::Transitioning),
    ));
}
