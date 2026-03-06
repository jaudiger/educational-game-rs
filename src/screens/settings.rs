use bevy::prelude::*;
use bevy::ui::Checked;
use bevy::ui_widgets::{
    SliderValue, ValueChange, checkbox_self_update, observe, slider_self_update,
};
use bevy::window::PrimaryWindow;
use bevy_persistent::prelude::*;

use crate::data::GameSettings;
use crate::data::progress::{GameMode, Language, MapTheme};
use crate::i18n::{I18n, TranslationKey};
use crate::states::AppState;
use crate::ui::components::{
    checkbox, radio_button, radio_button_muted, radio_group, screen_root, slider, standard_button,
};
use crate::ui::navigation::NavigateTo;
use crate::ui::theme;
use crate::ui::theme::DesignFontSize;

/// Settings screen for volume, language, mode, and theme preferences.
pub struct SettingsScreenPlugin;

impl Plugin for SettingsScreenPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(OnEnter(AppState::Settings), setup_settings)
            .add_systems(
                Update,
                (
                    update_music_volume_label,
                    update_sfx_volume_label,
                    rebuild_settings_on_language_change.run_if(resource_changed::<I18n>),
                )
                    .run_if(in_state(AppState::Settings)),
            );
    }
}

#[derive(Component, Reflect)]
struct SettingsRoot;

#[derive(Component, Reflect)]
struct MusicVolumeSlider;

#[derive(Component, Reflect)]
struct MusicVolumeValueLabel;

#[derive(Component, Reflect)]
struct SfxVolumeSlider;

#[derive(Component, Reflect)]
struct SfxVolumeValueLabel;

#[derive(Component, Reflect)]
struct ExplanationCheckbox;

#[derive(Component, Reflect)]
struct GamepadNavigationCheckbox;

#[derive(Component, Reflect)]
struct ModeRadio(GameMode);

#[derive(Component, Reflect)]
struct LanguageRadio(Language);

#[derive(Component, Reflect)]
struct MapThemeRadio(MapTheme);

fn setup_settings(
    mut commands: Commands,
    settings: Res<Persistent<GameSettings>>,
    i18n: Res<I18n>,
    primary_window: Single<Entity, With<PrimaryWindow>>,
) {
    spawn_settings_ui(&mut commands, &settings, &i18n, *primary_window);
}

fn spawn_settings_ui(
    commands: &mut Commands,
    settings: &GameSettings,
    i18n: &I18n,
    window: Entity,
) {
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
                TextFont {
                    font_size: theme::fonts::TITLE,
                    ..default()
                },
                TextColor(theme::colors::TEXT_DARK),
                DesignFontSize {
                    size: theme::fonts::TITLE,
                    window,
                },
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
                    map_theme_section(settings.map_theme, i18n, window),
                    explanation_section(settings.show_explanations, i18n, window),
                    gamepad_navigation_section(settings.gamepad_navigation, i18n, window),
                    music_volume_section(settings.music_volume, i18n, window),
                    sfx_volume_section(settings.sfx_volume, i18n, window),
                ],
            ),
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
                TextFont {
                    font_size: theme::fonts::HEADING,
                    ..default()
                },
                TextColor(theme::colors::TEXT_DARK),
                DesignFontSize {
                    size: theme::fonts::HEADING,
                    window,
                },
            ),
            (
                radio_group(),
                observe(handle_mode_radio_change),
                Children::spawn(SpawnWith(move |parent: &mut ChildSpawner| {
                    let mut cmd = parent.spawn((
                        radio_button(&individual_label, individual_checked, window),
                        ModeRadio(GameMode::Individual),
                    ));
                    if individual_checked {
                        cmd.insert(Checked);
                    }

                    let mut cmd = parent.spawn((
                        radio_button(&class_label, class_checked, window),
                        ModeRadio(GameMode::Group),
                    ));
                    if class_checked {
                        cmd.insert(Checked);
                    }
                })),
            ),
        ],
    )
}

fn handle_mode_radio_change(
    event: On<ValueChange<Entity>>,
    radio_query: Query<(Entity, &ModeRadio)>,
    mut settings: ResMut<Persistent<GameSettings>>,
    mut commands: Commands,
) {
    let Ok((_, mode_radio)) = radio_query.get(event.value) else {
        return;
    };
    let mode = mode_radio.0;
    settings
        .update(|s| s.mode = mode)
        .expect("failed to update game settings");

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
                TextFont {
                    font_size: theme::fonts::HEADING,
                    ..default()
                },
                TextColor(theme::colors::TEXT_DARK),
                DesignFontSize {
                    size: theme::fonts::HEADING,
                    window,
                },
            ),
            (
                radio_group(),
                observe(handle_language_radio_change),
                Children::spawn(SpawnWith(move |parent: &mut ChildSpawner| {
                    let mut cmd = parent.spawn((
                        radio_button(&french_label, french_checked, window),
                        LanguageRadio(Language::French),
                    ));
                    if french_checked {
                        cmd.insert(Checked);
                    }

                    let mut cmd = parent.spawn((
                        radio_button(&english_label, english_checked, window),
                        LanguageRadio(Language::English),
                    ));
                    if english_checked {
                        cmd.insert(Checked);
                    }
                })),
            ),
        ],
    )
}

fn handle_language_radio_change(
    event: On<ValueChange<Entity>>,
    radio_query: Query<&LanguageRadio>,
    mut settings: ResMut<Persistent<GameSettings>>,
) {
    let Ok(lang_radio) = radio_query.get(event.value) else {
        return;
    };
    let language = lang_radio.0;

    if language == settings.language {
        return;
    }

    let lang = language;
    settings
        .update(|s| s.language = lang)
        .expect("failed to update game settings");
}

/// Reactively rebuilds the settings UI when [`I18n`] changes (language switch).
///
/// Registered with `resource_changed::<I18n>` so it fires automatically after
/// `sync_i18n` (in [`SettingsPlugin`]) propagates the language change.
fn rebuild_settings_on_language_change(
    mut commands: Commands,
    settings: Res<Persistent<GameSettings>>,
    i18n: Res<I18n>,
    root_query: Query<Entity, With<SettingsRoot>>,
    primary_window: Single<Entity, With<PrimaryWindow>>,
) {
    for entity in &root_query {
        commands.entity(entity).despawn();
    }
    spawn_settings_ui(&mut commands, &settings, &i18n, *primary_window);
}

fn map_theme_section(current_theme: MapTheme, i18n: &I18n, window: Entity) -> impl Bundle + use<> {
    let label = i18n.t(&TranslationKey::MapThemeLabel);
    let sky_checked = current_theme == MapTheme::Sky;
    let ocean_checked = current_theme == MapTheme::Ocean;
    let space_checked = current_theme == MapTheme::Space;
    let sky_label = i18n.t(&TranslationKey::MapThemeSky).into_owned();
    let ocean_label = i18n.t(&TranslationKey::MapThemeOcean).into_owned();
    let space_label = i18n.t(&TranslationKey::MapThemeSpace).into_owned();
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
                TextFont {
                    font_size: theme::fonts::HEADING,
                    ..default()
                },
                TextColor(theme::colors::TEXT_DARK),
                DesignFontSize {
                    size: theme::fonts::HEADING,
                    window,
                },
            ),
            (
                radio_group(),
                observe(handle_theme_radio_change),
                Children::spawn(SpawnWith(move |parent: &mut ChildSpawner| {
                    let mut cmd = parent.spawn((
                        radio_button(&sky_label, sky_checked, window),
                        MapThemeRadio(MapTheme::Sky),
                    ));
                    if sky_checked {
                        cmd.insert(Checked);
                    }

                    let mut cmd = parent.spawn((
                        radio_button_muted(&ocean_label, &coming_soon, ocean_checked, window),
                        MapThemeRadio(MapTheme::Ocean),
                    ));
                    if ocean_checked {
                        cmd.insert(Checked);
                    }

                    let mut cmd = parent.spawn((
                        radio_button_muted(&space_label, &coming_soon, space_checked, window),
                        MapThemeRadio(MapTheme::Space),
                    ));
                    if space_checked {
                        cmd.insert(Checked);
                    }
                })),
            ),
        ],
    )
}

fn handle_theme_radio_change(
    event: On<ValueChange<Entity>>,
    radio_query: Query<(Entity, &MapThemeRadio)>,
    mut settings: ResMut<Persistent<GameSettings>>,
    mut commands: Commands,
) {
    let Ok((_, theme_radio)) = radio_query.get(event.value) else {
        return;
    };
    let map_theme = theme_radio.0;

    // Only allow selecting Sky for now
    if map_theme != MapTheme::Sky {
        return;
    }

    settings
        .update(|s| s.map_theme = map_theme)
        .expect("failed to update game settings");

    // Update Checked states on all radio buttons
    for (entity, radio) in radio_query.iter().collect::<Vec<_>>() {
        if radio.0 == map_theme {
            commands.entity(entity).insert(Checked);
        } else {
            commands.entity(entity).remove::<Checked>();
        }
    }
}

fn explanation_section(
    show_explanations: bool,
    i18n: &I18n,
    window: Entity,
) -> impl Bundle + use<> {
    let label = i18n.t(&TranslationKey::ExplanationsLabel);
    let checkbox_label = i18n.t(&TranslationKey::ExplanationsOn);

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

            let mut cmd = parent.spawn((
                checkbox(&checkbox_label, show_explanations, window),
                ExplanationCheckbox,
                observe(checkbox_self_update),
                observe(handle_explanation_checkbox_change),
            ));
            if show_explanations {
                cmd.insert(Checked);
            }
        })),
    )
}

fn handle_explanation_checkbox_change(
    event: On<ValueChange<bool>>,
    mut settings: ResMut<Persistent<GameSettings>>,
) {
    let val = event.value;
    settings
        .update(|s| s.show_explanations = val)
        .expect("failed to update game settings");
}

fn gamepad_navigation_section(enabled: bool, i18n: &I18n, window: Entity) -> impl Bundle + use<> {
    let label = i18n.t(&TranslationKey::GamepadNavigationLabel);
    let checkbox_label = i18n.t(&TranslationKey::GamepadNavigationOn);

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

            let mut cmd = parent.spawn((
                checkbox(&checkbox_label, enabled, window),
                GamepadNavigationCheckbox,
                observe(checkbox_self_update),
                observe(handle_gamepad_navigation_checkbox_change),
            ));
            if enabled {
                cmd.insert(Checked);
            }
        })),
    )
}

fn handle_gamepad_navigation_checkbox_change(
    event: On<ValueChange<bool>>,
    mut settings: ResMut<Persistent<GameSettings>>,
) {
    let val = event.value;
    settings
        .update(|s| s.gamepad_navigation = val)
        .expect("failed to update game settings");
}

#[allow(clippy::cast_possible_truncation, clippy::cast_sign_loss)]
fn music_volume_section(volume: f32, i18n: &I18n, window: Entity) -> impl Bundle + use<> {
    let label = i18n.t(&TranslationKey::MusicVolumeLabel);
    let percent = ((volume.clamp(0.0, 1.0) * 100.0).round()) as u32;

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
                TextFont {
                    font_size: theme::fonts::HEADING,
                    ..default()
                },
                TextColor(theme::colors::TEXT_DARK),
                DesignFontSize {
                    size: theme::fonts::HEADING,
                    window,
                },
            ),
            (
                slider(0.0, 1.0, volume, 0.05),
                MusicVolumeSlider,
                observe(slider_self_update),
                observe(handle_music_volume_slider_change),
            ),
            (
                Text::new(format!("{percent} %")),
                TextFont {
                    font_size: theme::fonts::BODY,
                    ..default()
                },
                TextColor(theme::colors::TEXT_DARK),
                MusicVolumeValueLabel,
                DesignFontSize {
                    size: theme::fonts::BODY,
                    window,
                },
            ),
        ],
    )
}

fn handle_music_volume_slider_change(
    event: On<ValueChange<f32>>,
    mut settings: ResMut<Persistent<GameSettings>>,
) {
    let volume = event.value;
    settings
        .update(|s| s.music_volume = volume)
        .expect("failed to update game settings");
}

/// Updates the music volume percentage label to match the current slider value.
#[allow(clippy::cast_possible_truncation, clippy::cast_sign_loss)]
fn update_music_volume_label(
    slider_query: Query<&SliderValue, (With<MusicVolumeSlider>, Changed<SliderValue>)>,
    mut label_query: Query<&mut Text, With<MusicVolumeValueLabel>>,
) {
    for slider_value in &slider_query {
        let percent = ((slider_value.0.clamp(0.0, 1.0) * 100.0).round()) as u32;
        for mut text in &mut label_query {
            **text = format!("{percent} %");
        }
    }
}

#[allow(clippy::cast_possible_truncation, clippy::cast_sign_loss)]
fn sfx_volume_section(volume: f32, i18n: &I18n, window: Entity) -> impl Bundle + use<> {
    let label = i18n.t(&TranslationKey::SfxVolumeLabel);
    let percent = ((volume.clamp(0.0, 1.0) * 100.0).round()) as u32;

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
                TextFont {
                    font_size: theme::fonts::HEADING,
                    ..default()
                },
                TextColor(theme::colors::TEXT_DARK),
                DesignFontSize {
                    size: theme::fonts::HEADING,
                    window,
                },
            ),
            (
                slider(0.0, 1.0, volume, 0.05),
                SfxVolumeSlider,
                observe(slider_self_update),
                observe(handle_sfx_volume_slider_change),
            ),
            (
                Text::new(format!("{percent} %")),
                TextFont {
                    font_size: theme::fonts::BODY,
                    ..default()
                },
                TextColor(theme::colors::TEXT_DARK),
                SfxVolumeValueLabel,
                DesignFontSize {
                    size: theme::fonts::BODY,
                    window,
                },
            ),
        ],
    )
}

fn handle_sfx_volume_slider_change(
    event: On<ValueChange<f32>>,
    mut settings: ResMut<Persistent<GameSettings>>,
) {
    let volume = event.value;
    settings
        .update(|s| s.sfx_volume = volume)
        .expect("failed to update game settings");
}

/// Updates the SFX volume percentage label to match the current slider value.
#[allow(clippy::cast_possible_truncation, clippy::cast_sign_loss)]
fn update_sfx_volume_label(
    slider_query: Query<&SliderValue, (With<SfxVolumeSlider>, Changed<SliderValue>)>,
    mut label_query: Query<&mut Text, With<SfxVolumeValueLabel>>,
) {
    for slider_value in &slider_query {
        let percent = ((slider_value.0.clamp(0.0, 1.0) * 100.0).round()) as u32;
        for mut text in &mut label_query {
            **text = format!("{percent} %");
        }
    }
}
