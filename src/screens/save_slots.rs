use std::collections::HashMap;

use bevy::input_focus::AutoFocus;
use bevy::prelude::*;
use bevy::window::PrimaryWindow;
use bevy_persistent::prelude::*;

use crate::data::{
    ActiveSlot, ClassSave, GameMode, GameSettings, IndividualSave, PersistenceMut, PlayerContext,
    SaveData,
};
use crate::i18n::{I18n, TranslationKey};
use crate::states::{AppState, StateScopedResourceExt};
use crate::ui::components::{
    PopoverCancelButton, PopoverConfirmButton, action_button, button_base, card_node, icon_button,
    screen_root, spawn_confirmation_modal, standard_button,
};
use crate::ui::navigation::NavigateTo;
use crate::ui::text_input::{TextInputState, text_input};
use crate::ui::theme;
use crate::ui::theme::DesignFontSize;

const MAX_NAME_LENGTH: usize = 30;

/// Save slot selection screen for creating, loading, and deleting profiles.
pub struct SaveSlotsScreenPlugin;

impl Plugin for SaveSlotsScreenPlugin {
    fn build(&self, app: &mut App) {
        app.register_state_scoped_resource::<AppState, SaveSlotsState>(AppState::SaveSlots)
            .add_systems(OnEnter(AppState::SaveSlots), setup_save_slots)
            .add_systems(
                Update,
                (
                    handle_slot_click,
                    handle_delete_click,
                    handle_confirm_delete,
                    handle_cancel_delete,
                    handle_create_confirm,
                    handle_cancel_create,
                )
                    .run_if(in_state(AppState::SaveSlots)),
            );
    }
}

#[derive(Resource, Default, Reflect)]
struct SaveSlotsState {
    creating_slot: Option<usize>,
}

#[derive(Component, Reflect)]
struct SlotCard(usize);

#[derive(Component, Reflect)]
struct DeleteSlotButton(usize);

#[derive(Component, Reflect)]
struct DeletePopover;

#[derive(Component, Reflect)]
struct ConfirmDeleteTarget(usize);

#[derive(Component, Reflect)]
struct CreationForm;

#[derive(Component, Reflect)]
struct ConfirmCreateButton;

#[derive(Component, Reflect)]
struct CancelCreateButton;

#[derive(Component, Reflect)]
struct SaveSlotsRoot;

fn setup_save_slots(
    mut commands: Commands,
    settings: Res<Persistent<GameSettings>>,
    save_data: Res<Persistent<SaveData>>,
    i18n: Res<I18n>,
    primary_window: Single<Entity, With<PrimaryWindow>>,
) {
    let window = *primary_window;
    commands.insert_resource(SaveSlotsState::default());
    spawn_save_slots_ui(&mut commands, settings.mode, &save_data, &i18n, window);
}

fn spawn_save_slots_ui(
    commands: &mut Commands,
    mode: GameMode,
    save_data: &SaveData,
    i18n: &I18n,
    window: Entity,
) {
    let title = match mode {
        GameMode::Individual => i18n.t(&TranslationKey::SelectSave),
        GameMode::Group => i18n.t(&TranslationKey::SelectClass),
    };
    let back = i18n.t(&TranslationKey::Back);

    // Pre-compute slot data for the SpawnWith closure
    let slot_data: Vec<_> = (0..3)
        .map(|slot_index| {
            let slot_name = match mode {
                GameMode::Individual => save_data.individual_slots[slot_index]
                    .as_ref()
                    .map(|s| s.name.clone()),
                GameMode::Group => save_data.class_slots[slot_index]
                    .as_ref()
                    .map(|s| s.name.clone()),
            };
            let slot_label = i18n.t(&TranslationKey::SlotN(slot_index + 1)).into_owned();
            let empty_label = i18n.t(&TranslationKey::Empty).into_owned();
            (slot_index, slot_name, slot_label, empty_label)
        })
        .collect();

    commands.spawn((
        screen_root(),
        DespawnOnExit(AppState::SaveSlots),
        SaveSlotsRoot,
        Children::spawn(SpawnWith(move |parent: &mut ChildSpawner| {
            // Title
            parent.spawn((
                Text::new(title),
                TextFont {
                    font_size: theme::fonts::TITLE,
                    ..default()
                },
                TextColor(theme::colors::TEXT_DARK),
                DesignFontSize {
                    size: theme::fonts::TITLE,
                    window,
                },
            ));

            // Slots row
            parent
                .spawn((Node {
                    flex_direction: FlexDirection::Row,
                    column_gap: theme::scaled(theme::spacing::LARGE),
                    justify_content: JustifyContent::Center,
                    align_items: AlignItems::Center,
                    ..default()
                },))
                .with_children(|row| {
                    for (i, (index, name, slot_label, empty_label)) in slot_data.iter().enumerate()
                    {
                        spawn_slot_card(
                            row,
                            *index,
                            name.as_deref(),
                            slot_label,
                            empty_label,
                            i == 0,
                            window,
                        );
                    }
                });

            // Back button
            parent.spawn((
                standard_button(
                    &back,
                    theme::colors::PRIMARY,
                    theme::scaled(theme::sizes::BUTTON_WIDTH),
                    window,
                ),
                NavigateTo(AppState::Home),
            ));
        })),
    ));
}

fn spawn_slot_card(
    parent: &mut ChildSpawner,
    index: usize,
    name: Option<&str>,
    slot_label: &str,
    empty_label: &str,
    auto_focus: bool,
    window: Entity,
) {
    let mut cmd = parent.spawn((
        button_base(theme::colors::CARD_BG),
        Node {
            width: theme::scaled(theme::sizes::CARD_MIN_WIDTH),
            min_height: theme::scaled(theme::sizes::CARD_MIN_HEIGHT),
            padding: theme::scaled(theme::sizes::CARD_PADDING).all(),
            flex_direction: FlexDirection::Column,
            justify_content: JustifyContent::Center,
            align_items: AlignItems::Center,
            row_gap: theme::scaled(theme::spacing::SMALL),
            border_radius: BorderRadius::all(theme::scaled(theme::sizes::CARD_BORDER_RADIUS)),
            ..default()
        },
        SlotCard(index),
    ));
    if auto_focus {
        cmd.insert(AutoFocus);
    }
    cmd.with_children(|card| {
        if let Some(slot_name) = name {
            // Filled slot: show name + delete button
            card.spawn((
                Text::new(slot_name),
                TextFont {
                    font_size: theme::fonts::HEADING,
                    ..default()
                },
                TextColor(theme::colors::TEXT_DARK),
                DesignFontSize {
                    size: theme::fonts::HEADING,
                    window,
                },
            ));

            card.spawn((
                Text::new(slot_label),
                TextFont {
                    font_size: theme::fonts::SMALL,
                    ..default()
                },
                TextColor(theme::colors::TEXT_MUTED),
                DesignFontSize {
                    size: theme::fonts::SMALL,
                    window,
                },
            ));

            // Delete button (icon_button + absolute positioning override)
            card.spawn((
                icon_button(
                    32.0,
                    6.0,
                    "X",
                    theme::fonts::SMALL,
                    theme::colors::ERROR,
                    theme::colors::TEXT_LIGHT,
                    window,
                ),
                DeleteSlotButton(index),
            ))
            .insert(Node {
                width: theme::scaled(32.0),
                height: theme::scaled(32.0),
                justify_content: JustifyContent::Center,
                align_items: AlignItems::Center,
                border_radius: BorderRadius::all(theme::scaled(6.0)),
                position_type: PositionType::Absolute,
                top: theme::scaled(8.0),
                right: theme::scaled(8.0),
                ..default()
            });
        } else {
            // Empty slot
            card.spawn((
                Text::new(empty_label),
                TextFont {
                    font_size: theme::fonts::HEADING,
                    ..default()
                },
                TextColor(theme::colors::TEXT_MUTED),
                DesignFontSize {
                    size: theme::fonts::HEADING,
                    window,
                },
            ));

            card.spawn((
                Text::new(slot_label),
                TextFont {
                    font_size: theme::fonts::SMALL,
                    ..default()
                },
                TextColor(theme::colors::TEXT_MUTED),
                DesignFontSize {
                    size: theme::fonts::SMALL,
                    window,
                },
            ));
        }
    });
}

/// Load the slot and transition to `MapExploration`.
fn navigate_to_existing_slot(
    commands: &mut Commands,
    index: usize,
    next_state: &mut NextState<AppState>,
) {
    commands.insert_resource(ActiveSlot(index));
    next_state.set(AppState::MapExploration);
}

/// Despawn any existing creation form and spawn a new one for `index`.
fn start_slot_creation(
    commands: &mut Commands,
    state: &mut SaveSlotsState,
    index: usize,
    creation_form: &Query<Entity, With<CreationForm>>,
    i18n: &I18n,
    window: Entity,
) {
    state.creating_slot = Some(index);
    for entity in creation_form {
        commands.entity(entity).despawn();
    }
    spawn_creation_form(commands, index, i18n, window);
}

#[allow(clippy::too_many_arguments)]
fn handle_slot_click(
    query: Query<(&Interaction, &SlotCard), Changed<Interaction>>,
    mut commands: Commands,
    mut state: ResMut<SaveSlotsState>,
    ctx: PlayerContext<'_>,
    mut next_state: ResMut<NextState<AppState>>,
    creation_form: Query<Entity, With<CreationForm>>,
    popover_query: Query<(), With<DeletePopover>>,
    i18n: Res<I18n>,
    primary_window: Single<Entity, With<PrimaryWindow>>,
) {
    if !popover_query.is_empty() {
        return;
    }

    for (interaction, slot) in &query {
        if *interaction != Interaction::Pressed {
            continue;
        }

        let index = slot.0;
        let is_filled = match ctx.settings.mode {
            GameMode::Individual => ctx.save_data.individual_slots[index].is_some(),
            GameMode::Group => ctx.save_data.class_slots[index].is_some(),
        };

        if is_filled {
            navigate_to_existing_slot(&mut commands, index, &mut next_state);
        } else if state.creating_slot.is_none() {
            start_slot_creation(
                &mut commands,
                &mut state,
                index,
                &creation_form,
                &i18n,
                *primary_window,
            );
        }
    }
}

fn spawn_creation_form(commands: &mut Commands, slot_index: usize, i18n: &I18n, window: Entity) {
    let title = i18n.t(&TranslationKey::CreateSaveSlotN(slot_index + 1));
    let (card_n, card_bg, card_border) = card_node(Node {
        width: theme::scaled(400.0),
        padding: theme::scaled(theme::sizes::CARD_PADDING).all(),
        flex_direction: FlexDirection::Column,
        align_items: AlignItems::Center,
        row_gap: theme::scaled(theme::spacing::MEDIUM),
        border_radius: BorderRadius::all(theme::scaled(theme::sizes::CARD_BORDER_RADIUS)),
        ..default()
    });

    commands.spawn((
        Node {
            position_type: PositionType::Absolute,
            width: percent(100.0),
            height: percent(100.0),
            justify_content: JustifyContent::Center,
            align_items: AlignItems::Center,
            ..default()
        },
        GlobalZIndex(50),
        CreationForm,
        DespawnOnExit(AppState::SaveSlots),
        children![(
            card_n,
            card_bg,
            card_border,
            children![
                (
                    Text::new(title),
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
                creation_name_input(i18n, window),
                creation_buttons(i18n, window),
            ],
        )],
    ));
}

fn creation_name_input(i18n: &I18n, window: Entity) -> impl Bundle + use<> {
    let name_label = i18n.t(&TranslationKey::NameLabel);
    (
        Node {
            flex_direction: FlexDirection::Row,
            align_items: AlignItems::Center,
            column_gap: theme::scaled(theme::spacing::MEDIUM),
            ..default()
        },
        children![
            (
                Text::new(name_label),
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
            text_input(
                250.0,
                TextInputState::new(MAX_NAME_LENGTH).focused(),
                window
            ),
        ],
    )
}

fn creation_buttons(i18n: &I18n, window: Entity) -> impl Bundle + use<> {
    let create_label = i18n.t(&TranslationKey::Create);
    let cancel_label = i18n.t(&TranslationKey::Cancel);
    (
        Node {
            flex_direction: FlexDirection::Row,
            column_gap: theme::scaled(theme::spacing::MEDIUM),
            ..default()
        },
        children![
            (
                action_button(
                    &create_label,
                    theme::colors::SUCCESS,
                    theme::colors::TEXT_LIGHT,
                    window,
                ),
                ConfirmCreateButton,
            ),
            (
                action_button(
                    &cancel_label,
                    theme::colors::TOGGLE_INACTIVE,
                    theme::colors::TEXT_DARK,
                    window,
                ),
                CancelCreateButton,
            ),
        ],
    )
}

fn handle_delete_click(
    query: Query<(&Interaction, Entity, &DeleteSlotButton), Changed<Interaction>>,
    state: Res<SaveSlotsState>,
    mut commands: Commands,
    existing_popover: Query<Entity, With<DeletePopover>>,
    i18n: Res<I18n>,
    primary_window: Single<Entity, With<PrimaryWindow>>,
) {
    if state.creating_slot.is_some() {
        return;
    }

    for (interaction, _button_entity, delete_btn) in &query {
        if *interaction == Interaction::Pressed {
            // Despawn any existing delete popover first
            for entity in &existing_popover {
                commands.entity(entity).try_despawn();
            }

            let slot_index = delete_btn.0;
            let modal_entity = spawn_confirmation_modal(
                &mut commands,
                &i18n.t(&TranslationKey::DeleteSlotN(slot_index + 1)),
                &i18n.t(&TranslationKey::Delete),
                &i18n.t(&TranslationKey::Cancel),
                theme::colors::ERROR,
                *primary_window,
                None,
            );
            commands
                .entity(modal_entity)
                .insert((DeletePopover, ConfirmDeleteTarget(slot_index)));
        }
    }
}

fn handle_confirm_delete(
    query: Query<&Interaction, (Changed<Interaction>, With<PopoverConfirmButton>)>,
    popover: Single<(Entity, &ConfirmDeleteTarget), With<DeletePopover>>,
    mut persistence: PersistenceMut<'_>,
    mut commands: Commands,
    root_query: Query<Entity, With<SaveSlotsRoot>>,
    i18n: Res<I18n>,
    primary_window: Single<Entity, With<PrimaryWindow>>,
) {
    for interaction in &query {
        if *interaction != Interaction::Pressed {
            continue;
        }

        let (popover_entity, target) = *popover;
        let index = target.0;
        let mode = persistence.settings.mode;

        persistence
            .save_data
            .update(|data| match mode {
                GameMode::Individual => data.individual_slots[index] = None,
                GameMode::Group => data.class_slots[index] = None,
            })
            .expect("failed to update save data");

        // Despawn popover and root UI, then rebuild
        commands.entity(popover_entity).despawn();
        for entity in &root_query {
            commands.entity(entity).despawn();
        }
        spawn_save_slots_ui(
            &mut commands,
            mode,
            &persistence.save_data,
            &i18n,
            *primary_window,
        );
    }
}

fn handle_cancel_delete(
    query: Query<&Interaction, (Changed<Interaction>, With<PopoverCancelButton>)>,
    mut commands: Commands,
    popover_query: Query<Entity, With<DeletePopover>>,
) {
    for interaction in &query {
        if *interaction == Interaction::Pressed {
            for entity in &popover_query {
                commands.entity(entity).try_despawn();
            }
        }
    }
}

fn handle_create_confirm(
    query: Query<&Interaction, (Changed<Interaction>, With<ConfirmCreateButton>)>,
    keyboard: Res<ButtonInput<KeyCode>>,
    mut state: ResMut<SaveSlotsState>,
    mut persistence: PersistenceMut<'_>,
    mut commands: Commands,
    mut next_state: ResMut<NextState<AppState>>,
    input: Single<&TextInputState>,
) {
    let Some(slot_index) = state.creating_slot else {
        return;
    };

    let pressed_button = query.iter().any(|i| *i == Interaction::Pressed);
    let pressed_enter = input.focused && keyboard.just_pressed(KeyCode::Enter);

    if !pressed_button && !pressed_enter {
        return;
    }

    let name = input.text.trim().to_owned();
    if name.is_empty() {
        return;
    }

    let mode = persistence.settings.mode;
    persistence
        .save_data
        .update(|data| match mode {
            GameMode::Individual => {
                data.individual_slots[slot_index] = Some(IndividualSave {
                    name: name.clone(),
                    progress: HashMap::new(),
                });
            }
            GameMode::Group => {
                data.class_slots[slot_index] = Some(ClassSave {
                    name: name.clone(),
                    students: Vec::new(),
                    lesson_configs: HashMap::new(),
                });
            }
        })
        .expect("failed to update save data");

    commands.insert_resource(ActiveSlot(slot_index));

    state.creating_slot = None;
    next_state.set(AppState::MapExploration);
}

fn handle_cancel_create(
    query: Query<&Interaction, (Changed<Interaction>, With<CancelCreateButton>)>,
    mut state: ResMut<SaveSlotsState>,
    mut commands: Commands,
    form_query: Query<Entity, With<CreationForm>>,
) {
    for interaction in &query {
        if *interaction == Interaction::Pressed {
            state.creating_slot = None;
            for entity in &form_query {
                commands.entity(entity).despawn();
            }
        }
    }
}
