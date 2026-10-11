use bevy::prelude::*;

use crate::data::QuestionResponse;
use crate::questions::fraction_bar::{COLOR_UNCOLORED, FractionSlice};
use crate::questions::registry::QuestionResponseSubmitted;
use crate::questions::registry::submit_response;
use crate::ui::components::ButtonActivated;
use crate::ui::theme;

use super::ValidateButton;

pub(super) fn handle_slice_toggle(
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
pub(super) fn handle_validate(
    mut activations: MessageReader<ButtonActivated>,
    validate_query: Query<(), With<ValidateButton>>,
    slices: Query<&FractionSlice>,
    mut responses: MessageWriter<QuestionResponseSubmitted>,
) {
    let activated_entities: Vec<Entity> = activations.read().map(|event| event.0).collect();
    for entity in activated_entities {
        if !validate_query.contains(entity) {
            continue;
        }

        let colored_count = slices.iter().filter(|slice| slice.colored).count() as u32;
        submit_response(
            &mut responses,
            QuestionResponse::FractionVisualization {
                colored_parts: colored_count,
            },
        );
        return;
    }
}
