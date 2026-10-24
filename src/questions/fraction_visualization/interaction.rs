use bevy::picking::events::PointerClick;
use bevy::prelude::*;
use bevy::ui::{InteractionDisabled, Pressed};

use crate::data::QuestionResponse;
use crate::questions::fraction_bar::FractionSlice;
use crate::questions::fraction_shape::{
    InteractiveFractionShape, fraction_shape_part_at, render_fraction_shape_with_parts,
};
use crate::questions::registry::QuestionResponseSubmitted;
use crate::questions::registry::submit_response;
use crate::ui::components::{ButtonActivated, ButtonVisualStyle};
use crate::ui::theme;

use super::ValidateButton;

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
    mut slice_query: Query<(&mut FractionSlice, &mut ButtonVisualStyle)>,
    mut shape_query: Query<&mut InteractiveFractionShape>,
    mut images: ResMut<Assets<Image>>,
) {
    let activated_entities: Vec<Entity> = activations.read().map(|event| event.0).collect();
    for entity in activated_entities {
        if let Ok((mut slice, mut style)) = slice_query.get_mut(entity) {
            slice.colored = !slice.colored;
            style.0 = if slice.colored {
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
        let Some(colored) = shape.colored_parts.get_mut(part as usize) else {
            continue;
        };
        *colored = !*colored;

        if let Some(mut image) = images.get_mut(&shape.image) {
            *image = render_fraction_shape_with_parts(
                shape.shape,
                shape.denominator,
                &shape.colored_parts,
                theme::colors::PRIMARY,
            );
        }
    }
}

#[allow(clippy::cast_possible_truncation)]
pub(super) fn handle_validate(
    mut activations: MessageReader<ButtonActivated>,
    validate_query: Query<(), With<ValidateButton>>,
    slices: Query<&FractionSlice>,
    shape_slices: Query<&InteractiveFractionShape>,
    mut responses: MessageWriter<QuestionResponseSubmitted>,
) {
    let activated_entities: Vec<Entity> = activations.read().map(|event| event.0).collect();
    for entity in activated_entities {
        if !validate_query.contains(entity) {
            continue;
        }

        let colored_count = slices.iter().filter(|slice| slice.colored).count() as u32
            + shape_slices
                .iter()
                .map(InteractiveFractionShape::colored_count)
                .sum::<u32>();
        submit_response(
            &mut responses,
            QuestionResponse::FractionVisualization {
                colored_parts: colored_count,
            },
        );
        return;
    }
}
