use bevy::prelude::*;
use bevy::window::PrimaryWindow;

use crate::data::content::{LocalizedExplanation, QuestionInstance};
use crate::data::{
    AnswerResult, ExplanationVisual, GameSettings, LastAnswer, LessonLaunchSelection,
    LessonSession, PersistenceAction, PersistenceStatus, QuestionContainer,
};
use crate::i18n::{I18n, TranslationKey};
use crate::states::LessonPhase;
use crate::ui::animation::AnimateScale;
use crate::ui::components::{persistence_message, save_write_failure_notice, standard_button};
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

/// Spawns feedback UI after `record_answer` has updated the session and save data.
#[allow(clippy::too_many_arguments)]
pub(super) fn setup_feedback_ui(
    mut commands: Commands,
    container: Single<Entity, With<QuestionContainer>>,
    last_answer: Res<LastAnswer>,
    session: Res<LessonSession>,
    settings: Res<GameSettings>,
    i18n: Res<I18n>,
    primary_window: Single<Entity, With<PrimaryWindow>>,
    write_status: Res<PersistenceStatus>,
    launch_selection: Option<Res<LessonLaunchSelection>>,
) {
    let window = *primary_window;
    let is_correct = matches!(**last_answer, AnswerResult::Correct);
    let is_last = session.current_index + 1 >= session.questions.len();
    let default_selection = LessonLaunchSelection::CompleteLesson;
    let selection = launch_selection.as_deref().unwrap_or(&default_selection);
    let next_button_label = match selection {
        LessonLaunchSelection::SingleExercise { .. } => TranslationKey::ReturnToExerciseList,
        LessonLaunchSelection::CompleteLesson if is_last => TranslationKey::FinishLesson,
        LessonLaunchSelection::CompleteLesson => TranslationKey::NextQuestion,
    };
    let save_notice = persistence_message(PersistenceAction::QuestionAnswer, *write_status, &i18n);
    let explanation = session.current().map_or(FeedbackExplanation::Hidden, |q| {
        build_feedback_explanation(settings.show_explanations, &q.instance)
    });

    commands.entity(*container).with_children(|parent| {
        spawn_feedback_content(
            parent,
            &i18n,
            is_correct,
            &next_button_label,
            save_notice.as_deref(),
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
    next_button_label: &TranslationKey,
    save_notice: Option<&str>,
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
            if let Some(notice) = save_notice {
                feedback.spawn(save_write_failure_notice(notice.to_owned(), window));
            }
            spawn_explanation_section(feedback, i18n, explanation, window);
            spawn_next_button(feedback, i18n, next_button_label, window);
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
            width: percent(100.0),
            margin: UiRect::axes(Val::ZERO, theme::scaled(theme::spacing::XLARGE)),
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
    i18n: &I18n,
    label: &TranslationKey,
    window: Entity,
) {
    parent.spawn((
        standard_button(
            &i18n.t(label),
            if matches!(label, TranslationKey::ReturnToExerciseList) {
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
