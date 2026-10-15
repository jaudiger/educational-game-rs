use bevy::prelude::*;
use bevy::ui::FocusPolicy;

use crate::data::{PersistenceAction, PersistenceStatus};
use crate::i18n::{I18n, ProductionTranslationKey};

use super::ButtonActivated;
use super::buttons::production::action_button_scene;
use super::card_node;
use super::theme;

#[derive(Component, Reflect)]
pub struct PersistenceNotice(PersistenceAction);

pub fn persistence_message(
    action: PersistenceAction,
    status: PersistenceStatus,
    i18n: &I18n,
) -> Option<String> {
    status.failed(action).then(|| {
        i18n.t(&ProductionTranslationKey::SaveWriteFailed)
            .into_owned()
    })
}

pub fn persistence_notice(
    action: PersistenceAction,
    status: PersistenceStatus,
    i18n: &I18n,
    window: Entity,
) -> impl Bundle + use<> {
    let message = persistence_message(action, status, i18n).unwrap_or_default();
    (
        Text::new(message),
        PersistenceNotice(action),
        theme::typography::text(theme::fonts::SMALL, window),
        TextColor(theme::colors::ERROR),
    )
}

pub fn sync_persistence_notices(
    status: Res<PersistenceStatus>,
    i18n: Res<I18n>,
    mut notice_query: Query<(&PersistenceNotice, &mut Text)>,
) {
    for (notice, mut text) in &mut notice_query {
        let message = if status.failed(notice.0) {
            i18n.t(&ProductionTranslationKey::SaveWriteFailed)
                .into_owned()
        } else {
            String::new()
        };
        if **text != message {
            message.clone_into(&mut **text);
        }
    }
}

pub fn save_write_failure_notice(text: String, window: Entity) -> impl Bundle + use<> {
    (
        Text::new(text),
        theme::typography::text(theme::fonts::SMALL, window),
        TextColor(theme::colors::ERROR),
    )
}

#[derive(Component, Reflect)]
pub struct PopoverConfirmButton;

#[derive(Component, Reflect)]
pub struct PopoverCancelButton;

/// Spawns a centered confirmation modal (full-screen overlay + card).
///
/// The caller can attach a confirmation observer to the returned dialog entity.
pub fn spawn_confirmation_modal(
    commands: &mut Commands,
    message: &str,
    confirm_label: &str,
    cancel_label: &str,
    confirm_palette: theme::ButtonPalette,
    window: Entity,
    camera: Option<Entity>,
) -> Entity {
    let message_owned = message.to_owned();
    let confirm_owned = confirm_label.to_owned();
    let cancel_owned = cancel_label.to_owned();

    let (card_n, card_bg, card_border) = card_node(Node {
        min_width: theme::scaled(theme::production::sizes::POPOVER_MIN_WIDTH),
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
                            confirm_palette,
                            window,
                        ));

                        let mut cancel = buttons.spawn_empty();
                        cancel.insert(PopoverCancelButton);
                        let _ = cancel.apply_scene(action_button_scene(
                            &cancel_owned,
                            theme::button_styles::NEUTRAL,
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

pub fn dispatch_confirmation_button_actions(
    mut activations: MessageReader<ButtonActivated>,
    confirm_buttons: Query<(), With<PopoverConfirmButton>>,
    cancel_buttons: Query<(), With<PopoverCancelButton>>,
    parents: Query<&ChildOf>,
    dialogs: Query<(), With<ConfirmationDialog>>,
    mut commands: Commands,
) {
    for activation in activations.read() {
        if confirm_buttons.contains(activation.0) {
            dispatch_confirmation_action(
                activation.0,
                ConfirmationDialogAction::Confirm,
                &mut commands,
                &parents,
                &dialogs,
            );
        } else if cancel_buttons.contains(activation.0) {
            dispatch_confirmation_action(
                activation.0,
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
