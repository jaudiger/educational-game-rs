//! Numeric-input question type.
//!
//! Displays a textual prompt (e.g. "7 x 8 = ?") and a numeric keypad.
//! The student types their answer and validates.

use bevy::prelude::*;
use bevy::window::PrimaryWindow;

use super::multiplication_grid::spawn_prompt_and_grid;
use super::numeric_keypad::{
    KeypadDeleteButton, KeypadDigitButton, KeypadDisplay, KeypadValidateButton, numeric_keypad,
};
use super::registry::{QuestionRoot, register_question_systems, submit_answer};
use crate::data::content::QuestionVisual;
use crate::data::{AnswerResult, LessonSession, QuestionContainer, QuestionDefinition};
use crate::i18n::I18n;
use crate::states::LessonPhase;

/// Handles numeric-input question UI with keypad and answer validation.
pub struct NumericInputPlugin;

impl Plugin for NumericInputPlugin {
    fn build(&self, app: &mut App) {
        register_question_systems(
            app,
            |d| matches!(d, QuestionDefinition::NumericInput(_)),
            spawn_numeric_input_ui,
            handle_numeric_input,
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
    correct_answer: u32,
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
    let QuestionDefinition::NumericInput(def) = &question.definition else {
        return;
    };

    let prompt_text = def.prompt.get(i18n.language).to_owned();
    let correct_answer = def.correct_answer;
    let hide_visual = question.hide_visual;
    let question_visual = def.question_visual.clone();

    commands.entity(*container).with_children(|parent| {
        parent
            .spawn((
                NumericInputRoot,
                NumericInputState {
                    value: String::new(),
                    correct_answer,
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

/// Maximum number of digits the student can type.
const MAX_DIGITS: usize = 7;

type DigitButtonQuery<'w, 's> = Query<
    'w,
    's,
    (&'static Interaction, &'static KeypadDigitButton),
    (Changed<Interaction>, With<Button>),
>;

/// Append any newly pressed digit to `value`. Returns `true` if anything changed.
fn process_digit_presses(query: &DigitButtonQuery<'_, '_>, value: &mut String) -> bool {
    let mut changed = false;
    for (interaction, digit_btn) in query {
        // Avoid leading zeros (unless it's the only digit)
        if *interaction == Interaction::Pressed
            && value.len() < MAX_DIGITS
            && !(digit_btn.0 == 0 && value.is_empty())
        {
            value.push(char::from(b'0' + digit_btn.0));
            changed = true;
        }
    }
    changed
}

/// Remove the last character from `value` if the delete button was pressed. Returns `true` if anything changed.
fn process_delete_press(
    query: &Query<&Interaction, (Changed<Interaction>, With<KeypadDeleteButton>)>,
    value: &mut String,
) -> bool {
    let mut changed = false;
    for interaction in query {
        if *interaction == Interaction::Pressed {
            value.pop();
            changed = true;
        }
    }
    changed
}

fn handle_numeric_input(
    digit_query: DigitButtonQuery<'_, '_>,
    delete_query: Query<&Interaction, (Changed<Interaction>, With<KeypadDeleteButton>)>,
    validate_query: Query<&Interaction, (Changed<Interaction>, With<KeypadValidateButton>)>,
    mut state: Single<&mut NumericInputState>,
    mut display_query: Query<&mut Text, With<KeypadDisplay>>,
    mut commands: Commands,
    mut next_phase: ResMut<NextState<LessonPhase>>,
) {
    let digit_changed = process_digit_presses(&digit_query, &mut state.value);
    let delete_changed = process_delete_press(&delete_query, &mut state.value);
    if digit_changed || delete_changed {
        for mut text in &mut display_query {
            (**text).clone_from(&state.value);
        }
    }

    for interaction in &validate_query {
        if *interaction == Interaction::Pressed && !state.value.is_empty() {
            let typed: u32 = state.value.parse().unwrap_or(0);
            let result = if typed == state.correct_answer {
                AnswerResult::Correct
            } else {
                AnswerResult::Incorrect
            };
            submit_answer(&mut commands, &mut next_phase, result);
            return;
        }
    }
}
