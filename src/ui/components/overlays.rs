use bevy::prelude::*;
use bevy::ui_widgets::popover::{Popover, PopoverAlign, PopoverPlacement, PopoverSide};

use crate::ui::theme;

use super::{TooltipPopover, card_node};

/// Spawns a simple tooltip popover with text, anchored to the given entity.
///
/// Auto-dismisses after 2 seconds via `TooltipLifetime`.
pub fn spawn_tooltip_popover(
    commands: &mut Commands,
    anchor: Entity,
    message: &str,
    window: Entity,
) -> Entity {
    let message_owned = message.to_owned();

    let (card_n, card_bg, card_border) = card_node(Node {
        position_type: PositionType::Absolute,
        padding: theme::scaled(theme::spacing::SMALL).all(),
        border_radius: BorderRadius::all(theme::scaled(theme::sizes::TOOLTIP_BORDER_RADIUS)),
        ..default()
    });

    let tooltip_entity = commands
        .spawn((
            Popover {
                positions: vec![
                    PopoverPlacement {
                        side: PopoverSide::Bottom,
                        align: PopoverAlign::Center,
                        gap: 4.0,
                    },
                    PopoverPlacement {
                        side: PopoverSide::Top,
                        align: PopoverAlign::Center,
                        gap: 4.0,
                    },
                ],
                window_margin: 8.0,
            },
            card_n,
            card_bg,
            card_border,
            GlobalZIndex(100),
            OverrideClip,
            Visibility::Hidden,
            TooltipPopover,
            children![(
                Text::new(message_owned),
                theme::typography::text(theme::fonts::SMALL, window),
                TextColor(theme::colors::TEXT_DARK),
            )],
        ))
        .id();

    commands.entity(anchor).add_child(tooltip_entity);

    tooltip_entity
}
