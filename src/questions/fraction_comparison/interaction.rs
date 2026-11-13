use bevy::prelude::*;

use crate::data::QuestionResponse;
use crate::questions::registry::QuestionResponseSubmitted;
use crate::questions::registry::submit_response;
use crate::ui::components::ButtonActivated;

use super::ComparisonChoice;

pub(super) fn handle_comparison_click(
    mut activations: MessageReader<ButtonActivated>,
    query: Query<&ComparisonChoice>,
    mut responses: MessageWriter<QuestionResponseSubmitted>,
) {
    let response = activations.read().find_map(|event| {
        query
            .get(event.0)
            .ok()
            .map(|choice| QuestionResponse::FractionComparison(choice.answer))
    });
    activations.clear();
    if let Some(response) = response {
        submit_response(&mut responses, response);
    }
}
