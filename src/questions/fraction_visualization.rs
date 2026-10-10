//! Fraction-visualisation question type.
//!
//! Displays a horizontal bar divided into N equal parts. The student clicks
//! parts to colour them, then validates when they think the correct fraction
//! is represented.

use bevy::prelude::*;
use bevy::window::PrimaryWindow;

use super::fraction_bar::{BAR_HEIGHT, BAR_WIDTH, COLOR_UNCOLORED, FractionSlice, fraction_bar};
use super::registry::{QuestionRoot, register_question_systems, submit_answer};
use crate::data::{AnswerResult, LessonSession, QuestionContainer, QuestionDefinition};
use crate::i18n::{I18n, TranslationKey};
use crate::states::LessonPhase;
use crate::ui::components::ButtonActivated;
use crate::ui::components::standard_button;
use crate::ui::theme;

/// Handles fraction-bar colouring question UI and answer validation.
pub struct FractionVisualizationPlugin;

impl Plugin for FractionVisualizationPlugin {
    fn build(&self, app: &mut App) {
        register_question_systems(
            app,
            |d| matches!(d, QuestionDefinition::FractionVisualization(_)),
            spawn_fraction_vis_ui,
            (handle_slice_toggle, handle_validate).chain(),
        );
    }
}

#[derive(Component, Reflect)]
#[require(Node = super::question_root_node())]
struct FractionVisualizationRoot;

#[derive(Component, Reflect)]
struct ValidateButton;

fn spawn_fraction_vis_ui(
    mut commands: Commands,
    container: Single<Entity, With<QuestionContainer>>,
    session: Res<LessonSession>,
    i18n: Res<I18n>,
    primary_window: Single<Entity, With<PrimaryWindow>>,
) {
    let window = *primary_window;
    let Some(question) = session.current() else {
        return;
    };
    let QuestionDefinition::FractionVisualization(def) = &question.definition else {
        return;
    };

    commands.entity(*container).with_children(|parent| {
        parent
            .spawn((FractionVisualizationRoot, QuestionRoot))
            .with_children(|root| {
                super::spawn_question_prompt(root, def.prompt.get(i18n.language), window);
                root.spawn(fraction_bar(
                    def.denominator,
                    0,
                    theme::colors::PRIMARY,
                    true,
                    BAR_WIDTH,
                    BAR_HEIGHT,
                ));
                spawn_validate_button(root, &i18n, window);
            });
    });
}

fn spawn_validate_button(parent: &mut ChildSpawnerCommands, i18n: &I18n, window: Entity) {
    parent
        .spawn(Node {
            margin: theme::scaled(theme::spacing::LARGE).top(),
            ..default()
        })
        .with_child((
            standard_button(
                &i18n.t(&TranslationKey::Validate),
                theme::colors::PRIMARY,
                theme::scaled(theme::sizes::BUTTON_WIDTH),
                window,
            ),
            ValidateButton,
        ));
}

fn handle_slice_toggle(
    mut activations: MessageReader<ButtonActivated>,
    mut slice_query: Query<(&mut FractionSlice, &mut BackgroundColor)>,
) {
    let activated_entities: Vec<Entity> = activations.read().map(|event| event.0).collect();
    for entity in activated_entities {
        let Ok((mut slice, mut bg)) = slice_query.get_mut(entity) else {
            continue;
        };
        slice.colored = !slice.colored;
        *bg = if slice.colored {
            BackgroundColor(theme::colors::PRIMARY)
        } else {
            BackgroundColor(COLOR_UNCOLORED)
        };
    }
}

#[allow(clippy::cast_possible_truncation)]
fn handle_validate(
    mut activations: MessageReader<ButtonActivated>,
    validate_query: Query<(), With<ValidateButton>>,
    slices: Query<&FractionSlice>,
    session: Res<LessonSession>,
    mut commands: Commands,
    mut next_phase: ResMut<NextState<LessonPhase>>,
) {
    let activated_entities: Vec<Entity> = activations.read().map(|event| event.0).collect();
    for entity in activated_entities {
        if !validate_query.contains(entity) {
            continue;
        }

        let Some(question) = session.current() else {
            return;
        };
        let QuestionDefinition::FractionVisualization(def) = &question.definition else {
            return;
        };

        let colored_count = slices.iter().filter(|s| s.colored).count() as u32;

        let result = if colored_count == def.numerator {
            AnswerResult::Correct
        } else {
            AnswerResult::Incorrect
        };

        submit_answer(&mut commands, &mut next_phase, result);
        return;
    }
}
