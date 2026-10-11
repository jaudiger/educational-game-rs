use bevy::prelude::*;

use crate::data::QuestionResponse;
use crate::questions::numeric_keypad::{
    KeypadDeleteButton, KeypadDigitButton, KeypadDisplay, KeypadValidateButton,
};
use crate::questions::registry::QuestionResponseSubmitted;
use crate::questions::registry::submit_response;
use crate::ui::components::ButtonActivated;

use super::NumericInputState;

/// Maximum number of digits the student can type.
const MAX_DIGITS: usize = 7;

#[allow(clippy::too_many_arguments)]
pub(super) fn handle_numeric_input(
    mut activations: MessageReader<ButtonActivated>,
    digit_query: Query<&KeypadDigitButton>,
    delete_query: Query<(), With<KeypadDeleteButton>>,
    validate_query: Query<(), With<KeypadValidateButton>>,
    mut state: Single<&mut NumericInputState>,
    mut display_query: Query<&mut Text, With<KeypadDisplay>>,
    mut responses: MessageWriter<QuestionResponseSubmitted>,
) {
    let activated_entities: Vec<Entity> = activations.read().map(|event| event.0).collect();
    let mut value_changed = false;
    for entity in activated_entities {
        if let Ok(digit_btn) = digit_query.get(entity) {
            value_changed |= append_digit(&mut state.value, digit_btn.0);
        } else if delete_query.contains(entity) {
            value_changed |= state.value.pop().is_some();
        } else if validate_query.contains(entity) && !state.value.is_empty() {
            let Ok(typed) = state.value.parse::<u32>() else {
                continue;
            };
            submit_response(&mut responses, QuestionResponse::NumericInput(typed));
            return;
        }
    }

    if value_changed {
        for mut text in &mut display_query {
            (**text).clone_from(&state.value);
        }
    }
}

fn append_digit(value: &mut String, digit: u8) -> bool {
    if digit > 9 || value.len() >= MAX_DIGITS {
        return false;
    }
    value.push(char::from(b'0' + digit));
    true
}

#[cfg(test)]
mod tests {
    use super::{MAX_DIGITS, append_digit};

    #[test]
    fn zero_can_be_entered_as_the_first_digit() {
        let mut value = String::new();
        assert!(append_digit(&mut value, 0));
        assert_eq!(value, "0");
    }

    #[test]
    fn keypad_rejects_digits_outside_the_supported_range_and_length() {
        let mut value = String::new();
        assert!(!append_digit(&mut value, 10));
        value = "1".repeat(MAX_DIGITS);
        assert!(!append_digit(&mut value, 2));
        assert_eq!(value, "1".repeat(MAX_DIGITS));
    }
}
