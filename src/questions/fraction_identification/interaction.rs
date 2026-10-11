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
    let activated_entities: Vec<Entity> = activations.read().map(|event| event.0).collect();
    for entity in activated_entities {
        let Ok(choice) = query.get(entity) else {
            continue;
        };
        submit_response(
            &mut responses,
            QuestionResponse::FractionIdentificationChoice(choice.index),
        );
        return;
    }
}
