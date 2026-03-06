//! Mouse-wheel scroll support for UI nodes with `Overflow::scroll_y()` / `scroll_x()`.
//!
//! Adapted from the official Bevy scroll example (`examples/ui/scroll.rs`).
//! Register [`ScrollPlugin`] once; any node with `OverflowAxis::Scroll` will
//! respond to the mouse wheel automatically.

use bevy::input::mouse::{MouseScrollUnit, MouseWheel};
use bevy::picking::hover::HoverMap;
use bevy::prelude::*;

const LINE_HEIGHT: f32 = 21.0;

pub struct ScrollPlugin;

impl Plugin for ScrollPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(Update, send_scroll_events)
            .add_observer(on_scroll_handler);
    }
}

/// Custom entity event that propagates up the UI hierarchy.
#[derive(EntityEvent, Debug)]
#[entity_event(propagate, auto_propagate)]
struct Scroll {
    entity: Entity,
    delta: Vec2,
}

/// Reads `MouseWheel` input and triggers [`Scroll`] events on hovered entities.
fn send_scroll_events(
    mut mouse_wheel_reader: MessageReader<MouseWheel>,
    hover_map: Res<HoverMap>,
    keyboard_input: Res<ButtonInput<KeyCode>>,
    mut commands: Commands,
) {
    for mouse_wheel in mouse_wheel_reader.read() {
        let mut delta = -Vec2::new(mouse_wheel.x, mouse_wheel.y);

        if mouse_wheel.unit == MouseScrollUnit::Line {
            delta *= LINE_HEIGHT;
        }

        if keyboard_input.any_pressed([KeyCode::ControlLeft, KeyCode::ControlRight]) {
            std::mem::swap(&mut delta.x, &mut delta.y);
        }

        for pointer_map in hover_map.values() {
            for entity in pointer_map.keys().copied() {
                commands.trigger(Scroll { entity, delta });
            }
        }
    }
}

/// Clamps scrolling on one axis, consuming the delta when the node actually scrolls.
fn apply_scroll_to_axis(overflow: OverflowAxis, pos: &mut f32, delta: &mut f32, max_offset: f32) {
    if overflow == OverflowAxis::Scroll && *delta != 0. {
        let at_limit = if *delta > 0. {
            *pos >= max_offset
        } else {
            *pos <= 0.
        };
        if !at_limit {
            *pos += *delta;
            *delta = 0.;
        }
    }
}

/// Observer that updates `ScrollPosition` on nodes with `OverflowAxis::Scroll`.
fn on_scroll_handler(
    mut scroll: On<Scroll>,
    mut query: Query<(&mut ScrollPosition, &Node, &ComputedNode)>,
) {
    let Ok((mut scroll_position, node, computed)) = query.get_mut(scroll.entity) else {
        return;
    };

    let max_offset = (computed.content_size() - computed.size()) * computed.inverse_scale_factor();
    let delta = &mut scroll.delta;

    apply_scroll_to_axis(
        node.overflow.x,
        &mut scroll_position.x,
        &mut delta.x,
        max_offset.x,
    );
    apply_scroll_to_axis(
        node.overflow.y,
        &mut scroll_position.y,
        &mut delta.y,
        max_offset.y,
    );

    if *delta == Vec2::ZERO {
        scroll.propagate(false);
    }
}
