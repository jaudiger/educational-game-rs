use bevy::picking::events::PointerClick;
use bevy::prelude::*;
use bevy::ui::{InteractionDisabled, Pressed};

use crate::data::QuestionResponse;
use crate::questions::fraction_bar::FractionSlice;
use crate::questions::fraction_shape::{
    InteractiveFractionShape, fraction_shape_part_at, render_fraction_shape,
};
use crate::questions::registry::QuestionResponseSubmitted;
use crate::questions::registry::submit_response;
use crate::ui::components::{ButtonActivated, ButtonVisualStyle};
use crate::ui::theme;

use super::{FractionVisualizationSelection, ValidateButton};

pub(super) fn record_fraction_shape_click(
    mut click: On<PointerClick>,
    mut shapes: Query<(
        &mut InteractiveFractionShape,
        Has<Pressed>,
        Has<InteractionDisabled>,
    )>,
) {
    let Ok((mut shape, pressed, disabled)) = shapes.get_mut(click.entity) else {
        return;
    };
    click.propagate(false);
    if !pressed || disabled {
        return;
    }
    shape.pointer_click_pending = true;
    shape.pending_pointer_part = click.hit.position.and_then(|position| {
        let normalized_position = position.truncate() + Vec2::splat(0.5);
        fraction_shape_part_at(
            shape.shape,
            shape.denominator,
            normalized_position.x,
            normalized_position.y,
        )
    });
}

pub(super) fn handle_slice_toggle(
    mut activations: MessageReader<ButtonActivated>,
    mut slice_query: Query<(&FractionSlice, &mut ButtonVisualStyle)>,
    mut shape_query: Query<&mut InteractiveFractionShape>,
    mut images: ResMut<Assets<Image>>,
    mut selection: ResMut<FractionVisualizationSelection>,
) {
    let activated_entities: Vec<Entity> = activations.read().map(|event| event.0).collect();
    for entity in activated_entities {
        if let Ok((slice, mut style)) = slice_query.get_mut(entity) {
            let Some(colored) = selection.0.toggle(slice.index) else {
                continue;
            };
            style.0 = if colored {
                theme::button_styles::FRACTION_SLICE_COLORED
            } else {
                theme::button_styles::FRACTION_SLICE_UNCOLORED
            };
            continue;
        }

        let Ok(mut shape) = shape_query.get_mut(entity) else {
            continue;
        };
        let selected_part = if shape.pointer_click_pending {
            shape.pointer_click_pending = false;
            shape.pending_pointer_part.take()
        } else {
            let part = shape.keyboard_part;
            shape.keyboard_part = shape.keyboard_part.saturating_add(1) % shape.denominator;
            Some(part)
        };
        let Some(part) = selected_part else {
            continue;
        };
        if selection.0.toggle(part).is_none() {
            continue;
        }

        if let Some(mut image) = images.get_mut(&shape.image) {
            *image = render_fraction_shape(shape.shape, &selection.0, theme::colors::PRIMARY);
        }
    }
}

pub(super) fn handle_validate(
    mut activations: MessageReader<ButtonActivated>,
    validate_query: Query<(), With<ValidateButton>>,
    mut responses: MessageWriter<QuestionResponseSubmitted>,
    selection: Res<FractionVisualizationSelection>,
) {
    for entity in activations.read().map(|event| event.0) {
        if !validate_query.contains(entity) {
            continue;
        }
        submit_response(
            &mut responses,
            QuestionResponse::FractionVisualization {
                colored_parts: selection.0.clone(),
            },
        );
        return;
    }
}
