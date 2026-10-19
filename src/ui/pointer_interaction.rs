use std::collections::HashMap;

use bevy::ecs::entity::EntityHashSet;
use bevy::input::touch::{TouchInput, TouchPhase};
use bevy::picking::PickingSystems;
use bevy::picking::events::PointerDrag;
use bevy::picking::hover::{HoverMap, PointerCaptureMap};
use bevy::picking::pointer::{PointerButton, PointerId};
use bevy::prelude::*;
use bevy::ui::{InteractionDisabled, OverflowAxis, Pressed, ScrollPosition};
use bevy::ui_widgets::ScrollArea;

use super::components::PointerHovered;

pub struct PointerInteractionPlugin;

impl Plugin for PointerInteractionPlugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<TouchScrollState>()
            .add_systems(PreUpdate, sync_pointer_hover.after(PickingSystems::Hover))
            .add_systems(Update, cleanup_touch_scroll_gestures)
            .add_observer(scroll_touch_drag);
    }
}

#[derive(Resource, Default, Reflect)]
#[reflect(Resource)]
struct TouchScrollState {
    gestures: HashMap<PointerId, TouchScrollGesture>,
}

#[derive(Reflect)]
struct TouchScrollGesture {
    area: Entity,
    last_position: Vec2,
    blocked_controls: Vec<Entity>,
}

#[derive(Component, Reflect)]
struct TouchScrollBlocked;

fn sync_pointer_hover(
    hover_map: Res<HoverMap>,
    parent_query: Query<&ChildOf>,
    mut tracked: Query<(Entity, &mut PointerHovered)>,
) {
    let mut hovered_entities = EntityHashSet::default();
    for entity in hover_map.values().flat_map(|entities| entities.keys()) {
        hovered_entities.insert(*entity);
        hovered_entities.extend(parent_query.iter_ancestors(*entity));
    }

    for (entity, mut hovered) in &mut tracked {
        let is_hovered = hovered_entities.contains(&entity);
        if hovered.0 != is_hovered {
            hovered.0 = is_hovered;
        }
    }
}

fn scroll_touch_drag(
    mut drag: On<PointerDrag>,
    mut commands: Commands,
    mut scroll_state: ResMut<TouchScrollState>,
    captured_pointers: Res<PointerCaptureMap>,
    parent_query: Query<&ChildOf>,
    mut scroll_areas: Query<(&Node, &ComputedNode, &mut ScrollPosition), With<ScrollArea>>,
    controls: Query<(
        Has<Pressed>,
        Has<InteractionDisabled>,
        Has<TouchScrollBlocked>,
    )>,
) {
    let pointer_id = drag.pointer.id;
    if !pointer_id.is_touch()
        || drag.button != PointerButton::Primary
        || captured_pointers.is_captured(&pointer_id)
    {
        return;
    }

    let scroll_area = if let Some(gesture) = scroll_state.gestures.get_mut(&pointer_id) {
        if gesture.last_position == drag.pointer.position {
            return;
        }
        gesture.last_position = drag.pointer.position;
        gesture.area
    } else {
        let area = std::iter::once(drag.entity)
            .chain(parent_query.iter_ancestors(drag.entity))
            .find(|entity| {
                scroll_areas.get(*entity).is_ok_and(|(node, _, _)| {
                    (node.overflow.x == OverflowAxis::Scroll && drag.delta.x.abs() > f32::EPSILON)
                        || (node.overflow.y == OverflowAxis::Scroll
                            && drag.delta.y.abs() > f32::EPSILON)
                })
            });
        let Some(area) = area else {
            return;
        };

        let mut blocked_controls = Vec::new();
        for entity in std::iter::once(drag.entity)
            .chain(parent_query.iter_ancestors(drag.entity))
            .take_while(|entity| *entity != area)
        {
            let is_pressed = controls
                .get(entity)
                .is_ok_and(|(pressed, disabled, blocked)| (pressed && !disabled) || blocked);
            if is_pressed {
                commands
                    .entity(entity)
                    .insert((InteractionDisabled, TouchScrollBlocked))
                    .remove::<Pressed>();
                blocked_controls.push(entity);
            }
        }

        scroll_state.gestures.insert(
            pointer_id,
            TouchScrollGesture {
                area,
                last_position: drag.pointer.position,
                blocked_controls,
            },
        );
        area
    };

    let Ok((node, computed, mut scroll_position)) = scroll_areas.get_mut(scroll_area) else {
        return;
    };
    let visible_size = computed.size() * computed.inverse_scale_factor;
    let content_size = computed.content_size() * computed.inverse_scale_factor;
    let max_range = (content_size - visible_size).max(Vec2::ZERO);
    let next_position = touch_drag_scroll_position(
        scroll_position.0,
        max_range,
        node.overflow.x == OverflowAxis::Scroll,
        node.overflow.y == OverflowAxis::Scroll,
        drag.delta,
    );
    if scroll_position.0 != next_position {
        scroll_position.0 = next_position;
    }
    drag.propagate(false);
}

fn touch_drag_scroll_position(
    position: Vec2,
    max_range: Vec2,
    scroll_x: bool,
    scroll_y: bool,
    delta: Vec2,
) -> Vec2 {
    let mut next_position = position;
    if scroll_x {
        next_position.x = (position.x - delta.x).clamp(0.0, max_range.x);
    }
    if scroll_y {
        next_position.y = (position.y - delta.y).clamp(0.0, max_range.y);
    }
    next_position
}

fn cleanup_touch_scroll_gestures(
    mut commands: Commands,
    mut touch_events: MessageReader<TouchInput>,
    mut scroll_state: ResMut<TouchScrollState>,
    blocked_controls: Query<(), With<TouchScrollBlocked>>,
) {
    for touch in touch_events.read() {
        if !matches!(touch.phase, TouchPhase::Ended | TouchPhase::Canceled) {
            continue;
        }

        let pointer_id = PointerId::Touch(touch.id);
        let Some(gesture) = scroll_state.gestures.remove(&pointer_id) else {
            continue;
        };

        for entity in gesture.blocked_controls {
            let still_blocked = scroll_state
                .gestures
                .values()
                .any(|gesture| gesture.blocked_controls.contains(&entity));
            if !still_blocked && blocked_controls.contains(entity) {
                commands
                    .entity(entity)
                    .try_remove::<TouchScrollBlocked>()
                    .try_remove::<InteractionDisabled>();
            }
        }
    }
}
