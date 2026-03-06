use bevy::prelude::*;
use bevy::ui::FocusPolicy;
use bevy::ui_widgets::popover::{Popover, PopoverAlign, PopoverPlacement, PopoverSide};

use crate::ui::theme;
use crate::ui::theme::DesignFontSize;

use super::buttons::action_button;
use super::{PopoverCancelButton, PopoverConfirmButton, TooltipPopover, card_node};

/// Spawns a centered confirmation modal (full-screen overlay + card).
///
/// The modal contains a message, a confirm button, and a cancel button.
/// The caller should query for `PopoverConfirmButton` / `PopoverCancelButton`
/// to handle the actions, or attach additional markers via the returned entity.
pub fn spawn_confirmation_modal(
    commands: &mut Commands,
    message: &str,
    confirm_label: &str,
    cancel_label: &str,
    confirm_color: Color,
    window: Entity,
    camera: Option<Entity>,
) -> Entity {
    let message_owned = message.to_owned();
    let confirm_owned = confirm_label.to_owned();
    let cancel_owned = cancel_label.to_owned();

    let (card_n, card_bg, card_border) = card_node(Node {
        min_width: theme::scaled(theme::sizes::POPOVER_MIN_WIDTH),
        padding: theme::scaled(theme::spacing::MEDIUM).all(),
        flex_direction: FlexDirection::Column,
        align_items: AlignItems::Center,
        row_gap: theme::scaled(theme::spacing::MEDIUM),
        border_radius: BorderRadius::all(theme::scaled(theme::sizes::CARD_BORDER_RADIUS)),
        ..default()
    });

    let mut entity_commands = commands.spawn((
        Node {
            position_type: PositionType::Absolute,
            width: percent(100.0),
            height: percent(100.0),
            justify_content: JustifyContent::Center,
            align_items: AlignItems::Center,
            ..default()
        },
        BackgroundColor(Color::srgba(0.0, 0.0, 0.0, 0.3)),
        GlobalZIndex(100),
        FocusPolicy::Block,
        Interaction::None,
        children![(
            card_n,
            card_bg,
            card_border,
            children![
                // Message text
                (
                    Text::new(message_owned),
                    TextFont {
                        font_size: theme::fonts::BODY,
                        ..default()
                    },
                    TextColor(theme::colors::TEXT_DARK),
                    DesignFontSize {
                        size: theme::fonts::BODY,
                        window,
                    },
                ),
                // Button row
                (
                    Node {
                        flex_direction: FlexDirection::Row,
                        column_gap: theme::scaled(theme::spacing::MEDIUM),
                        ..default()
                    },
                    children![
                        // Confirm button
                        (
                            action_button(
                                &confirm_owned,
                                confirm_color,
                                theme::colors::TEXT_LIGHT,
                                window,
                            ),
                            PopoverConfirmButton,
                        ),
                        // Cancel button
                        (
                            action_button(
                                &cancel_owned,
                                theme::colors::TOGGLE_INACTIVE,
                                theme::colors::TEXT_DARK,
                                window,
                            ),
                            PopoverCancelButton,
                        ),
                    ],
                ),
            ],
        )],
    ));

    if let Some(cam) = camera {
        entity_commands.insert(UiTargetCamera(cam));
    }

    entity_commands.id()
}

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
                TextFont {
                    font_size: theme::fonts::SMALL,
                    ..default()
                },
                TextColor(theme::colors::TEXT_DARK),
                DesignFontSize {
                    size: theme::fonts::SMALL,
                    window,
                },
            )],
        ))
        .id();

    commands.entity(anchor).add_child(tooltip_entity);

    tooltip_entity
}
