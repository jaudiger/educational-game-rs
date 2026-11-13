use bevy::prelude::*;

use crate::data::QuestionResponse;
use crate::questions::registry::QuestionResponseSubmitted;
use crate::questions::registry::submit_response;
use crate::ui::components::ButtonActivated;

use super::IdentificationChoice;

pub(super) fn handle_identification_click(
    mut activations: MessageReader<ButtonActivated>,
    query: Query<&IdentificationChoice>,
    mut responses: MessageWriter<QuestionResponseSubmitted>,
) {
    let response = activations.read().find_map(|event| {
        query
            .get(event.0)
            .ok()
            .map(|choice| QuestionResponse::FractionIdentificationChoice(choice.index))
    });
    activations.clear();
    if let Some(response) = response {
        submit_response(&mut responses, response);
    }
}
