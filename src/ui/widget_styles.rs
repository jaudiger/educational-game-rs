use bevy::prelude::*;
use bevy::ui::Checked;
use bevy::ui_widgets::{Slider, SliderRange, SliderThumb, SliderValue};

use super::components::{
    HoverTooltip, HoverTooltipPopover, TooltipLifetime, TooltipPopover, spawn_tooltip_popover,
};
use super::theme;

/// Toggles the mark visibility for any widget that carries a `Checked` component.
///
/// `W` is the widget marker and `M` is the mark descendant.
/// Register once per widget type via turbofish.
#[allow(clippy::type_complexity)]
pub fn update_checked_visuals<W: Component, M: Component>(
    changed_query: Query<(Entity, Has<Checked>), (With<W>, Changed<Checked>)>,
    mut removed: RemovedComponents<Checked>,
    all_widgets: Query<Entity, With<W>>,
    children_query: Query<&Children>,
    mut mark_query: Query<&mut Visibility, With<M>>,
) {
    for (entity, is_checked) in &changed_query {
        let target = if is_checked {
            Visibility::Visible
        } else {
            Visibility::Hidden
        };
        set_descendant_visibility(entity, target, &children_query, &mut mark_query);
    }

    for entity in removed.read() {
        if all_widgets.get(entity).is_ok() {
            set_descendant_visibility(entity, Visibility::Hidden, &children_query, &mut mark_query);
        }
    }
}

/// Positions the slider thumb based on the current `SliderValue` and `SliderRange`.
///
/// Bevy's headless slider does not position the thumb visually; this system
/// handles that responsibility.
pub fn update_slider_thumb_position(
    slider_query: Query<(&SliderValue, &SliderRange, &Children), With<Slider>>,
    mut thumb_query: Query<&mut Node, With<SliderThumb>>,
) {
    for (value, range, children) in &slider_query {
        let pct = if range.span().abs() > f32::EPSILON {
            ((value.0 - range.start()) / range.span()).clamp(0.0, 1.0)
        } else {
            0.0
        };
        // Express thumb position as a percentage of the parent so it
        // works at any scale (the ratio is constant regardless of vmin).
        let travel_fraction = 1.0 - theme::sizes::SLIDER_THUMB_SIZE / theme::sizes::SLIDER_WIDTH;
        for child in children.iter() {
            if let Ok(mut node) = thumb_query.get_mut(child) {
                node.left = percent(pct * travel_fraction * 100.0);
            }
        }
    }
}

/// Despawns tooltip popovers after their lifetime timer expires.
pub fn dismiss_tooltip_timer(
    mut commands: Commands,
    time: Res<Time>,
    mut query: Query<(Entity, &mut TooltipLifetime), With<TooltipPopover>>,
) {
    for (entity, mut lifetime) in &mut query {
        lifetime.0.tick(time.delta());
        if lifetime.0.is_finished() {
            commands.entity(entity).try_despawn();
        }
    }
}

/// Spawns/despawns tooltips on hover for entities with `HoverTooltip`.
pub fn handle_hover_tooltips(
    mut commands: Commands,
    query: Query<(Entity, &Interaction, &HoverTooltip), Changed<Interaction>>,
    existing: Query<Entity, With<HoverTooltipPopover>>,
) {
    for (entity, interaction, hover_tooltip) in &query {
        match interaction {
            Interaction::Hovered => {
                for e in &existing {
                    commands.entity(e).try_despawn();
                }
                let tooltip = spawn_tooltip_popover(
                    &mut commands,
                    entity,
                    &hover_tooltip.message,
                    hover_tooltip.window,
                );
                commands
                    .entity(tooltip)
                    .insert(HoverTooltipPopover)
                    .remove::<TooltipLifetime>();
            }
            Interaction::None => {
                for e in &existing {
                    commands.entity(e).try_despawn();
                }
            }
            Interaction::Pressed => {}
        }
    }
}

/// Helper: sets the visibility of a specific marker component among an entity's descendants.
fn set_descendant_visibility<M: Component>(
    entity: Entity,
    target: Visibility,
    children_query: &Query<&Children>,
    mark_query: &mut Query<&mut Visibility, With<M>>,
) {
    for descendant in children_query.iter_descendants(entity) {
        if let Ok(mut visibility) = mark_query.get_mut(descendant) {
            *visibility = target;
        }
    }
}
