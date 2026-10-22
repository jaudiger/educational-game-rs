//! Numeric-input question type.
//!
//! Displays a textual prompt (e.g. "7 x 8 = ?") and a numeric keypad.
//! The student types their answer and validates.

use bevy::prelude::*;
use bevy::window::PrimaryWindow;

use super::multiplication_grid::spawn_multiplication_grid;
use super::numeric_keypad::numeric_keypad;
use super::registry::{QuestionRoot, register_question_systems};
use crate::data::content::QuestionVisual;
use crate::data::{LessonSession, QuestionContainer, QuestionInstance};
use crate::i18n::I18n;

/// Handles numeric-input question UI with keypad and answer validation.
pub struct NumericInputPlugin;

impl Plugin for NumericInputPlugin {
    fn build(&self, app: &mut App) {
        register_question_systems(
            app,
            |d| matches!(d, QuestionInstance::NumericInput(_)),
            spawn_numeric_input_ui,
            interaction::handle_numeric_input,
        );
    }
}

#[derive(Component, Reflect)]
#[require(Node = super::question_root_node())]
struct NumericInputRoot;

/// Stores the current typed value for the active numeric-input question.
#[derive(Component, Reflect)]
struct NumericInputState {
    value: String,
}

fn spawn_numeric_input_ui(
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
    let QuestionInstance::NumericInput(def) = &question.instance else {
        return;
    };

    let prompt_text = def.prompt.get(i18n.language).to_owned();
    let hide_visual = question.hide_optional_visual;
    let question_visual = def.question_visual.clone();

    commands.entity(*container).with_children(|parent| {
        parent
            .spawn((
                NumericInputRoot,
                NumericInputState {
                    value: String::new(),
                },
                QuestionRoot,
            ))
            .with_children(|root| {
                super::spawn_question_card(root, |card| {
                    super::spawn_question_prompt(card, &prompt_text, window);
                    if !hide_visual
                        && let Some(QuestionVisual::MultiplicationGrid { rows, cols }) =
                            &question_visual
                    {
                        super::spawn_question_visuals(
                            card,
                            super::QuestionVisualLayout::Grid { max_columns: 1 },
                            &[super::QuestionVisualSizing::Preferred],
                            |visual, _| {
                                spawn_multiplication_grid(visual, *rows, *cols);
                            },
                        );
                    }
                });
                super::spawn_controls_card(root, |card| {
                    card.spawn(numeric_keypad("", window));
                });
            });
    });
}

mod interaction;
