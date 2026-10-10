use bevy::picking::events::PointerRelease;
use bevy::prelude::*;
use bevy::ui::{Checked, Pressed};
use bevy::ui_widgets::{
    SliderValue, ValueChange, checkbox_self_update, observe, slider_self_update,
};
use bevy::window::PrimaryWindow;

use crate::data::progress::{ExplorationTheme, GameMode, Language};
use crate::data::{GameSettings, GameSettingsMut, PersistenceAction, PersistenceStatus};
use crate::i18n::{I18n, TranslationKey};
use crate::states::AppState;
use crate::ui::components::{
    checkbox_scene, persistence_notice, radio_button_muted_scene, radio_button_scene, radio_group,
    screen_root, slider_scene, standard_button,
};
use crate::ui::navigation::NavigateTo;
use crate::ui::theme;

/// Settings screen for volume, language, mode, and theme preferences.
pub struct SettingsScreenPlugin;

impl Plugin for SettingsScreenPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(OnEnter(AppState::Settings), setup_settings)
            .add_systems(
                Update,
                (
                    update_volume_label,
                    rebuild_settings_on_language_change.run_if(resource_changed::<I18n>),
                )
                    .run_if(in_state(AppState::Settings)),
            );
    }
}

#[derive(Component, Reflect)]
struct SettingsRoot;

/// Identifies which audio channel a volume slider or its percentage label belongs to.
#[derive(Component, Reflect, Clone, Copy, PartialEq, Eq)]
enum VolumeChannel {
    Music,
    Sfx,
}

/// Identifies which boolean setting a checkbox controls.
#[derive(Component, Reflect, Clone, Copy)]
enum BoolSettingField {
    ShowExplanations,
    GamepadNavigation,
}

#[derive(Component, Reflect)]
struct ModeRadio(GameMode);

#[derive(Component, Reflect)]
struct LanguageRadio(Language);

#[derive(Component, Reflect)]
struct ExplorationThemeRadio(ExplorationTheme);

fn setup_settings(
    mut commands: Commands,
    settings: Res<GameSettings>,
    status: Res<PersistenceStatus>,
    i18n: Res<I18n>,
    primary_window: Single<Entity, With<PrimaryWindow>>,
) {
    spawn_settings_ui(&mut commands, &settings, *status, &i18n, *primary_window);
}

fn spawn_settings_ui(
    commands: &mut Commands,
    settings: &GameSettings,
    status: PersistenceStatus,
    i18n: &I18n,
    window: Entity,
) {
    let notice = persistence_notice(PersistenceAction::GameSettings, status, i18n, window);
    let title = i18n.t(&TranslationKey::Settings);
    let back = i18n.t(&TranslationKey::Back);

    commands.spawn((
        screen_root(),
        DespawnOnExit(AppState::Settings),
        SettingsRoot,
        children![
            // Title
            (
                Text::new(title),
                theme::typography::text(theme::fonts::TITLE, window),
                TextColor(theme::colors::TEXT_DARK),
            ),
            // Settings sections group
            (
                Node {
                    flex_direction: FlexDirection::Column,
                    align_items: AlignItems::Center,
                    row_gap: theme::scaled(theme::spacing::LARGE),
                    ..default()
                },
                children![
                    mode_section(settings.mode, i18n, window),
                    language_section(settings.language, i18n, window),
                    exploration_theme_section(settings.exploration_theme, i18n, window),
                    bool_setting_section(
                        BoolSettingField::ShowExplanations,
                        settings.show_explanations,
                        i18n,
                        window,
                    ),
                    bool_setting_section(
                        BoolSettingField::GamepadNavigation,
                        settings.gamepad_navigation,
                        i18n,
                        window,
                    ),
                    volume_section(VolumeChannel::Music, settings.music_volume, i18n, window),
                    volume_section(VolumeChannel::Sfx, settings.sfx_volume, i18n, window),
                ],
            ),
            notice,
            // Back button
            (
                standard_button(
                    &back,
                    theme::colors::PRIMARY,
                    theme::scaled(theme::sizes::BUTTON_WIDTH),
                    window,
                ),
                NavigateTo(AppState::Home),
            ),
        ],
    ));
}

fn mode_section(current_mode: GameMode, i18n: &I18n, window: Entity) -> impl Bundle + use<> {
    let mode_label = i18n.t(&TranslationKey::Mode);
    let individual_checked = current_mode == GameMode::Individual;
    let class_checked = current_mode == GameMode::Group;
    let individual_label = i18n.t(&TranslationKey::ModeIndividual).into_owned();
    let class_label = i18n.t(&TranslationKey::ModeClass).into_owned();

    (
        Node {
            flex_direction: FlexDirection::Row,
            align_items: AlignItems::Center,
            column_gap: theme::scaled(theme::spacing::MEDIUM),
            ..default()
        },
        children![
            (
                Text::new(mode_label),
                theme::typography::text(theme::fonts::HEADING, window),
                TextColor(theme::colors::TEXT_DARK),
            ),
            (
                radio_group(),
                observe(handle_mode_radio_change),
                Children::spawn(SpawnWith(move |parent: &mut ChildSpawner| {
                    let mut cmd = parent.spawn_empty();
                    cmd.insert(ModeRadio(GameMode::Individual));
                    if individual_checked {
                        cmd.insert(Checked);
                    }
                    let _ = cmd.apply_scene(radio_button_scene(&individual_label, window));

                    let mut cmd = parent.spawn_empty();
                    cmd.insert(ModeRadio(GameMode::Group));
                    if class_checked {
                        cmd.insert(Checked);
                    }
                    let _ = cmd.apply_scene(radio_button_scene(&class_label, window));
                })),
            ),
        ],
    )
}

fn handle_mode_radio_change(
    event: On<ValueChange<Entity>>,
    radio_query: Query<(Entity, &ModeRadio)>,
    mut settings: GameSettingsMut<'_>,
    mut commands: Commands,
) {
    let Ok((_, mode_radio)) = radio_query.get(event.value) else {
        return;
    };
    let mode = mode_radio.0;
    settings.update(|s| s.mode = mode);

    // Update Checked states on all radio buttons
    for (entity, radio) in radio_query.iter().collect::<Vec<_>>() {
        if radio.0 == mode {
            commands.entity(entity).insert(Checked);
        } else {
            commands.entity(entity).remove::<Checked>();
        }
    }
}

fn language_section(
    current_language: Language,
    i18n: &I18n,
    window: Entity,
) -> impl Bundle + use<> {
    let label = i18n.t(&TranslationKey::LanguageLabel);
    let french_checked = current_language == Language::French;
    let english_checked = current_language == Language::English;
    let french_label = i18n.t(&TranslationKey::LanguageFrench).into_owned();
    let english_label = i18n.t(&TranslationKey::LanguageEnglish).into_owned();

    (
        Node {
            flex_direction: FlexDirection::Row,
            align_items: AlignItems::Center,
            column_gap: theme::scaled(theme::spacing::MEDIUM),
            ..default()
        },
        children![
            (
                Text::new(label),
                theme::typography::text(theme::fonts::HEADING, window),
                TextColor(theme::colors::TEXT_DARK),
            ),
            (
                radio_group(),
                observe(handle_language_radio_change),
                Children::spawn(SpawnWith(move |parent: &mut ChildSpawner| {
                    let mut cmd = parent.spawn_empty();
                    cmd.insert(LanguageRadio(Language::French));
                    if french_checked {
                        cmd.insert(Checked);
                    }
                    let _ = cmd.apply_scene(radio_button_scene(&french_label, window));

                    let mut cmd = parent.spawn_empty();
                    cmd.insert(LanguageRadio(Language::English));
                    if english_checked {
                        cmd.insert(Checked);
                    }
                    let _ = cmd.apply_scene(radio_button_scene(&english_label, window));
                })),
            ),
        ],
    )
}

fn handle_language_radio_change(
    event: On<ValueChange<Entity>>,
    radio_query: Query<(Entity, &LanguageRadio)>,
    mut settings: GameSettingsMut<'_>,
    mut commands: Commands,
) {
    let Ok((_, lang_radio)) = radio_query.get(event.value) else {
        return;
    };
    let language = lang_radio.0;

    if language == settings.settings.language {
        return;
    }

    let lang = language;
    settings.update(|s| s.language = lang);

    for (entity, radio) in radio_query.iter().collect::<Vec<_>>() {
        if radio.0 == language {
            commands.entity(entity).insert(Checked);
        } else {
            commands.entity(entity).remove::<Checked>();
        }
    }
}

/// Reactively rebuilds the settings UI when [`I18n`] changes (language switch).
///
/// Registered with `resource_changed::<I18n>` so it fires automatically after
/// `sync_i18n` (in [`SettingsPlugin`]) propagates the language change.
fn rebuild_settings_on_language_change(
    mut commands: Commands,
    settings: Res<GameSettings>,
    status: Res<PersistenceStatus>,
    i18n: Res<I18n>,
    root_query: Query<Entity, With<SettingsRoot>>,
    primary_window: Single<Entity, With<PrimaryWindow>>,
) {
    for entity in &root_query {
        commands.entity(entity).despawn();
    }
    spawn_settings_ui(&mut commands, &settings, *status, &i18n, *primary_window);
}

fn exploration_theme_section(
    current_theme: ExplorationTheme,
    i18n: &I18n,
    window: Entity,
) -> impl Bundle + use<> {
    let label = i18n.t(&TranslationKey::ExplorationThemeLabel);
    let sky_checked = current_theme == ExplorationTheme::Sky;
    let ocean_checked = current_theme == ExplorationTheme::Ocean;
    let space_checked = current_theme == ExplorationTheme::Space;
    let sky_label = i18n.t(&TranslationKey::ExplorationThemeSky).into_owned();
    let ocean_label = i18n.t(&TranslationKey::ExplorationThemeOcean).into_owned();
    let space_label = i18n.t(&TranslationKey::ExplorationThemeSpace).into_owned();
    let coming_soon = i18n.t(&TranslationKey::ComingSoon).into_owned();

    (
        Node {
            flex_direction: FlexDirection::Row,
            align_items: AlignItems::Center,
            column_gap: theme::scaled(theme::spacing::MEDIUM),
            ..default()
        },
        children![
            (
                Text::new(label),
                theme::typography::text(theme::fonts::HEADING, window),
                TextColor(theme::colors::TEXT_DARK),
            ),
            (
                radio_group(),
                observe(handle_theme_radio_change),
                Children::spawn(SpawnWith(move |parent: &mut ChildSpawner| {
                    let mut cmd = parent.spawn_empty();
                    cmd.insert(ExplorationThemeRadio(ExplorationTheme::Sky));
                    if sky_checked {
                        cmd.insert(Checked);
                    }
                    let _ = cmd.apply_scene(radio_button_scene(&sky_label, window));

                    let mut cmd = parent.spawn_empty();
                    cmd.insert(ExplorationThemeRadio(ExplorationTheme::Ocean));
                    if ocean_checked {
                        cmd.insert(Checked);
                    }
                    let _ = cmd.apply_scene(radio_button_muted_scene(
                        &ocean_label,
                        &coming_soon,
                        window,
                    ));

                    let mut cmd = parent.spawn_empty();
                    cmd.insert(ExplorationThemeRadio(ExplorationTheme::Space));
                    if space_checked {
                        cmd.insert(Checked);
                    }
                    let _ = cmd.apply_scene(radio_button_muted_scene(
                        &space_label,
                        &coming_soon,
                        window,
                    ));
                })),
            ),
        ],
    )
}

fn handle_theme_radio_change(
    event: On<ValueChange<Entity>>,
    radio_query: Query<(Entity, &ExplorationThemeRadio)>,
    mut settings: GameSettingsMut<'_>,
    mut commands: Commands,
) {
    let Ok((_, theme_radio)) = radio_query.get(event.value) else {
        return;
    };
    let exploration_theme = theme_radio.0;

    // Only allow selecting Sky for now
    if exploration_theme != ExplorationTheme::Sky {
        return;
    }

    settings.update(|s| s.exploration_theme = exploration_theme);

    // Update Checked states on all radio buttons
    for (entity, radio) in radio_query.iter().collect::<Vec<_>>() {
        if radio.0 == exploration_theme {
            commands.entity(entity).insert(Checked);
        } else {
            commands.entity(entity).remove::<Checked>();
        }
    }
}

fn bool_setting_section(
    field: BoolSettingField,
    enabled: bool,
    i18n: &I18n,
    window: Entity,
) -> impl Bundle + use<> {
    let (label_key, checkbox_key) = match field {
        BoolSettingField::ShowExplanations => (
            TranslationKey::ExplanationsLabel,
            TranslationKey::ExplanationsOn,
        ),
        BoolSettingField::GamepadNavigation => (
            TranslationKey::GamepadNavigationLabel,
            TranslationKey::GamepadNavigationOn,
        ),
    };
    let label = i18n.t(&label_key);
    let checkbox_label = i18n.t(&checkbox_key);

    (
        Node {
            flex_direction: FlexDirection::Row,
            align_items: AlignItems::Center,
            column_gap: theme::scaled(theme::spacing::MEDIUM),
            ..default()
        },
        Children::spawn(SpawnWith(move |parent: &mut ChildSpawner| {
            parent.spawn((
                Text::new(label),
                theme::typography::text(theme::fonts::HEADING, window),
                TextColor(theme::colors::TEXT_DARK),
            ));

            let mut cmd = parent.spawn_empty();
            cmd.insert((
                field,
                observe(checkbox_self_update),
                observe(handle_bool_setting_change),
            ));
            if enabled {
                cmd.insert(Checked);
            }
            let _ = cmd.apply_scene(checkbox_scene(&checkbox_label, window));
        })),
    )
}

fn handle_bool_setting_change(
    event: On<ValueChange<bool>>,
    field_query: Query<&BoolSettingField>,
    mut settings: GameSettingsMut<'_>,
) {
    let val = event.value;
    let Ok(field) = field_query.get(event.event_target()) else {
        return;
    };
    let f = *field;
    settings.update(|s| match f {
        BoolSettingField::ShowExplanations => s.show_explanations = val,
        BoolSettingField::GamepadNavigation => s.gamepad_navigation = val,
    });
}

#[allow(clippy::cast_possible_truncation, clippy::cast_sign_loss)]
fn volume_section(
    channel: VolumeChannel,
    volume: f32,
    i18n: &I18n,
    window: Entity,
) -> impl Bundle + use<> {
    let label_key = match channel {
        VolumeChannel::Music => TranslationKey::MusicVolumeLabel,
        VolumeChannel::Sfx => TranslationKey::SfxVolumeLabel,
    };
    let label = i18n.t(&label_key);
    let percent = ((volume.clamp(0.0, 1.0) * 100.0).round()) as u32;

    (
        Node {
            flex_direction: FlexDirection::Row,
            align_items: AlignItems::Center,
            column_gap: theme::scaled(theme::spacing::MEDIUM),
            ..default()
        },
        Children::spawn(SpawnWith(move |parent: &mut ChildSpawner| {
            parent.spawn((
                Text::new(label),
                theme::typography::text(theme::fonts::HEADING, window),
                TextColor(theme::colors::TEXT_DARK),
            ));

            let mut slider = parent.spawn_empty();
            slider.insert((
                channel,
                observe(slider_self_update),
                observe(handle_volume_slider_change),
                observe(persist_volume_on_pointer_release),
            ));
            let _ = slider.apply_scene(slider_scene(0.0, 1.0, volume, 0.05));

            parent.spawn((
                Text::new(format!("{percent} %")),
                theme::typography::text(theme::fonts::BODY, window),
                TextColor(theme::colors::TEXT_DARK),
                channel,
            ));
        })),
    )
}

fn handle_volume_slider_change(
    event: On<ValueChange<f32>>,
    channel_query: Query<&VolumeChannel>,
    pressed_query: Query<(), With<Pressed>>,
    mut settings: GameSettingsMut<'_>,
) {
    let volume = event.value;
    let Ok(channel) = channel_query.get(event.event_target()) else {
        return;
    };
    let ch = *channel;
    {
        match ch {
            VolumeChannel::Music => settings.settings.music_volume = volume,
            VolumeChannel::Sfx => settings.settings.sfx_volume = volume,
        }
    }
    if event.is_final && !pressed_query.contains(event.event_target()) {
        settings.persist();
    }
}

fn persist_volume_on_pointer_release(
    _event: On<PointerRelease>,
    mut settings: GameSettingsMut<'_>,
) {
    settings.persist();
}

/// Updates the volume percentage label to match the current slider value.
#[allow(clippy::cast_possible_truncation, clippy::cast_sign_loss)]
fn update_volume_label(
    slider_query: Query<(&SliderValue, &VolumeChannel), Changed<SliderValue>>,
    mut label_query: Query<(&mut Text, &VolumeChannel), Without<SliderValue>>,
) {
    for (slider_value, channel) in &slider_query {
        let percent = ((slider_value.0.clamp(0.0, 1.0) * 100.0).round()) as u32;
        for (mut text, label_channel) in &mut label_query {
            if label_channel == channel {
                **text = format!("{percent} %");
            }
        }
    }
}
