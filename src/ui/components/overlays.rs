use bevy::prelude::*;
use bevy::ui::FocusPolicy;
use bevy::ui_widgets::popover::{Popover, PopoverAlign, PopoverPlacement, PopoverSide};

use crate::ui::theme;

use super::buttons::action_button_scene;
use super::{PopoverCancelButton, PopoverConfirmButton, TooltipPopover, card_node};

/// Spawns a centered confirmation modal (full-screen overlay + card).
///
/// The caller can attach a confirmation observer to the returned dialog entity.
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
                    theme::typography::text(theme::fonts::BODY, window),
                    TextColor(theme::colors::TEXT_DARK),
                ),
                // Button row
                (
                    Node {
                        flex_direction: FlexDirection::Row,
                        column_gap: theme::scaled(theme::spacing::MEDIUM),
                        ..default()
                    },
                    Children::spawn(SpawnWith(move |buttons: &mut ChildSpawner| {
                        let mut confirm = buttons.spawn_empty();
                        confirm.insert(PopoverConfirmButton);
                        let _ = confirm.apply_scene(action_button_scene(
                            &confirm_owned,
                            confirm_color,
                            theme::colors::TEXT_LIGHT,
                            window,
                        ));

                        let mut cancel = buttons.spawn_empty();
                        cancel.insert(PopoverCancelButton);
                        let _ = cancel.apply_scene(action_button_scene(
                            &cancel_owned,
                            theme::colors::TOGGLE_INACTIVE,
                            theme::colors::TEXT_DARK,
                            window,
                        ));
                    })),
                ),
            ],
        )],
    ));

    entity_commands.insert(ConfirmationDialog);
    entity_commands.observe(dismiss_confirmation_dialog);

    if let Some(cam) = camera {
        entity_commands.insert(UiTargetCamera(cam));
    }

    entity_commands.id()
}

#[derive(Component, Reflect)]
pub struct ConfirmationDialog;

#[derive(Clone, Copy, PartialEq, Eq)]
pub enum ConfirmationDialogAction {
    Confirm,
    Cancel,
}

#[derive(EntityEvent)]
pub struct ConfirmationDialogActionEvent {
    pub entity: Entity,
    pub action: ConfirmationDialogAction,
}

#[allow(clippy::type_complexity)]
pub fn dispatch_confirmation_button_actions(
    mut commands: Commands,
    confirm_buttons: Query<
        (Entity, &Interaction),
        (Changed<Interaction>, With<PopoverConfirmButton>),
    >,
    cancel_buttons: Query<
        (Entity, &Interaction),
        (Changed<Interaction>, With<PopoverCancelButton>),
    >,
    parents: Query<&ChildOf>,
    dialogs: Query<(), With<ConfirmationDialog>>,
) {
    for (button, interaction) in &confirm_buttons {
        if *interaction == Interaction::Pressed {
            dispatch_confirmation_action(
                button,
                ConfirmationDialogAction::Confirm,
                &mut commands,
                &parents,
                &dialogs,
            );
        }
    }

    for (button, interaction) in &cancel_buttons {
        if *interaction == Interaction::Pressed {
            dispatch_confirmation_action(
                button,
                ConfirmationDialogAction::Cancel,
                &mut commands,
                &parents,
                &dialogs,
            );
        }
    }
}

fn dispatch_confirmation_action(
    button: Entity,
    action: ConfirmationDialogAction,
    commands: &mut Commands,
    parents: &Query<&ChildOf>,
    dialogs: &Query<(), With<ConfirmationDialog>>,
) {
    let mut target = button;
    loop {
        if dialogs.get(target).is_ok() {
            commands.trigger(ConfirmationDialogActionEvent {
                entity: target,
                action,
            });
            return;
        }

        let Ok(parent) = parents.get(target) else {
            return;
        };
        target = parent.parent();
    }
}

fn dismiss_confirmation_dialog(event: On<ConfirmationDialogActionEvent>, mut commands: Commands) {
    commands.entity(event.entity).despawn();
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
