//! Fraction-visualisation question type.
//!
//! Displays a horizontal bar divided into N equal parts. The student clicks
//! parts to colour them, then validates when they think the correct fraction
//! is represented.

use bevy::prelude::*;
use bevy::window::PrimaryWindow;

use super::fraction_bar::{BAR_HEIGHT, BAR_WIDTH, fraction_bar};
use super::registry::{QuestionRoot, register_question_systems};
use crate::data::{LessonSession, QuestionContainer, QuestionInstance};
use crate::i18n::{I18n, TranslationKey};
use crate::ui::components::standard_button;
use crate::ui::theme;

/// Handles fraction-bar colouring question UI and answer validation.
pub struct FractionVisualizationPlugin;

impl Plugin for FractionVisualizationPlugin {
    fn build(&self, app: &mut App) {
        register_question_systems(
            app,
            |d| matches!(d, QuestionInstance::FractionVisualization(_)),
            spawn_fraction_vis_ui,
            (
                interaction::handle_slice_toggle,
                interaction::handle_validate,
            )
                .chain(),
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
    let QuestionInstance::FractionVisualization(def) = &question.instance else {
        return;
    };

    commands.entity(*container).with_children(|parent| {
        parent
            .spawn((FractionVisualizationRoot, QuestionRoot))
            .with_children(|root| {
                super::spawn_question_card(root, |card| {
                    super::spawn_question_prompt(card, def.prompt.get(i18n.language), window);
                    super::spawn_question_visuals(
                        card,
                        super::QuestionVisualLayout::Grid { max_columns: 1 },
                        &[super::QuestionVisualSizing::Preferred],
                        |visual, _| {
                            visual.spawn(fraction_bar(
                                def.denominator,
                                0,
                                theme::colors::PRIMARY,
                                true,
                                BAR_WIDTH,
                                BAR_HEIGHT,
                            ));
                        },
                    );
                });
                super::spawn_controls_card(root, |card| {
                    spawn_validate_button(card, &i18n, window);
                });
            });
    });
}

fn spawn_validate_button(parent: &mut ChildSpawnerCommands, i18n: &I18n, window: Entity) {
    parent.spawn((
        standard_button(
            &i18n.t(&TranslationKey::Validate),
            theme::button_styles::ACTION_PRIMARY,
            theme::scaled(theme::sizes::BUTTON_WIDTH),
            window,
        ),
        ValidateButton,
    ));
}

mod interaction;
