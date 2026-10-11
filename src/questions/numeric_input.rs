//! Numeric-input question type.
//!
//! Displays a textual prompt (e.g. "7 x 8 = ?") and a numeric keypad.
//! The student types their answer and validates.

use bevy::prelude::*;
use bevy::window::PrimaryWindow;

use super::multiplication_grid::spawn_prompt_and_grid;
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
                // If the question has a grid visual and it's not hidden, show prompt + grid.
                if !hide_visual
                    && let Some(QuestionVisual::MultiplicationGrid { rows, cols }) =
                        &question_visual
                {
                    spawn_prompt_and_grid(root, &prompt_text, *rows, *cols, window);
                    root.spawn(numeric_keypad("", window));
                    return;
                }
                // Default: text prompt only + keypad.
                super::spawn_question_prompt(root, &prompt_text, window);
                root.spawn(numeric_keypad("", window));
            });
    });
}

mod interaction;
