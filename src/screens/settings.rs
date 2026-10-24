use bevy::picking::events::{PointerCancel, PointerRelease};
use bevy::prelude::*;
use bevy::ui::{Checked, Pressed};
use bevy::ui_widgets::{
    ScrollArea, SliderValue, ValueChange, checkbox_self_update, observe, slider_self_update,
};
use bevy::window::PrimaryWindow;

use crate::data::progress::{ExplorationTheme, Language};
use crate::data::{GameSettings, GameSettingsMut};
use crate::i18n::{I18n, TranslationKey};
use crate::states::AppState;
use crate::ui::components::{
    MenuBackgroundStyle, checkbox_scene, menu_background, radio_button_muted_scene,
    radio_button_scene, screen_root, slider_scene, standard_button,
};
use crate::ui::layout::{self, CollectionItemLayout, CollectionLayout};
use crate::ui::navigation::NavigateTo;
use crate::ui::theme;

/// Settings screen for volume, language, mode, and theme preferences.
pub struct SettingsScreenPlugin;

impl Plugin for SettingsScreenPlugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<SettingsPresentation>()
            .configure_sets(
                Update,
                (SettingsScreenSet::Prepare, SettingsScreenSet::Render).chain(),
            )
            .add_systems(OnEnter(AppState::Settings), setup_settings)
            .add_systems(
                Update,
                update_volume_label.run_if(in_state(AppState::Settings)),
            )
            .add_systems(
                Update,
                rebuild_settings_on_presentation_change
                    .in_set(SettingsScreenSet::Render)
                    .run_if(
                        resource_changed::<I18n>.or_else(resource_changed::<SettingsPresentation>),
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

#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct ModePresentation {
    pub label: String,
    pub individual_label: String,
    pub group_label: String,
    pub group_selected: bool,
}

#[derive(Resource, Clone, Debug, Default, PartialEq, Eq)]
pub struct SettingsPresentation {
    pub mode: Option<ModePresentation>,
    pub notice: Option<String>,
}

#[derive(SystemSet, Clone, Copy, Debug, Eq, Hash, PartialEq)]
pub enum SettingsScreenSet {
    Prepare,
    Render,
}

#[derive(Event, Clone, Copy, Debug)]
pub struct ModeSelectionRequested(#[allow(dead_code)] pub bool);

#[derive(Component, Reflect)]
struct ModeRadio(bool);

#[derive(Component, Reflect)]
struct LanguageRadio(Language);

#[derive(Component, Reflect)]
struct ExplorationThemeRadio(ExplorationTheme);

struct LanguageLabels {
    label: String,
    french: String,
    english: String,
}

struct ThemeLabels {
    label: String,
    sky: String,
    ocean: String,
    space: String,
    coming_soon: String,
}

struct ToggleLabels {
    label: String,
    checkbox: String,
}

#[derive(Clone, Copy)]
enum SettingsControlSizing {
    Computed,
    NestedFluidCollection,
}

struct SettingsLabels {
    language: LanguageLabels,
    theme: ThemeLabels,
    explanations: ToggleLabels,
    navigation: ToggleLabels,
    music: String,
    sfx: String,
}

impl SettingsLabels {
    fn new(i18n: &I18n) -> Self {
        Self {
            language: LanguageLabels {
                label: i18n.t(&TranslationKey::LanguageLabel).into_owned(),
                french: i18n.t(&TranslationKey::LanguageFrench).into_owned(),
                english: i18n.t(&TranslationKey::LanguageEnglish).into_owned(),
            },
            theme: ThemeLabels {
                label: i18n.t(&TranslationKey::ExplorationThemeLabel).into_owned(),
                sky: i18n.t(&TranslationKey::ExplorationThemeSky).into_owned(),
                ocean: i18n.t(&TranslationKey::ExplorationThemeOcean).into_owned(),
                space: i18n.t(&TranslationKey::ExplorationThemeSpace).into_owned(),
                coming_soon: i18n.t(&TranslationKey::ComingSoon).into_owned(),
            },
            explanations: ToggleLabels {
                label: i18n.t(&TranslationKey::ExplanationsLabel).into_owned(),
                checkbox: i18n.t(&TranslationKey::ExplanationsOn).into_owned(),
            },
            navigation: ToggleLabels {
                label: i18n.t(&TranslationKey::GamepadNavigationLabel).into_owned(),
                checkbox: i18n.t(&TranslationKey::GamepadNavigationOn).into_owned(),
            },
            music: i18n.t(&TranslationKey::MusicVolumeLabel).into_owned(),
            sfx: i18n.t(&TranslationKey::SfxVolumeLabel).into_owned(),
        }
    }
}

pub fn setup_settings(
    mut commands: Commands,
    settings: Res<GameSettings>,
    presentation: Res<SettingsPresentation>,
    i18n: Res<I18n>,
    primary_window: Single<Entity, With<PrimaryWindow>>,
) {
    spawn_settings_ui(
        &mut commands,
        &settings,
        &presentation,
        &i18n,
        *primary_window,
    );
}

fn spawn_settings_ui(
    commands: &mut Commands,
    settings: &GameSettings,
    presentation: &SettingsPresentation,
    i18n: &I18n,
    window: Entity,
) {
    let mode = presentation.mode.clone();
    let language = settings.language;
    let exploration_theme = settings.exploration_theme;
    let show_explanations = settings.show_explanations;
    let gamepad_navigation = settings.gamepad_navigation;
    let music_volume = settings.music_volume;
    let sfx_volume = settings.sfx_volume;
    let notice = presentation.notice.clone();
    let labels = SettingsLabels::new(i18n);
    let title = (
        Text::new(i18n.t(&TranslationKey::Settings)),
        theme::typography::text(theme::fonts::TITLE, window),
        TextColor(theme::colors::TEXT_DARK),
    );
    let back = (
        standard_button(
            &i18n.t(&TranslationKey::Back),
            theme::button_styles::BACK,
            theme::scaled(theme::sizes::BUTTON_WIDTH),
            window,
        ),
        NavigateTo(AppState::Home),
    );

    commands
        .spawn((
            screen_root(),
            DespawnOnExit(AppState::Settings),
            SettingsRoot,
        ))
        .with_children(move |root| {
            root.spawn(menu_background(MenuBackgroundStyle::new(0.18)));
            root.spawn(settings_card_node()).with_children(move |card| {
                card.spawn(title);
                card.spawn(settings_scroll_node())
                    .with_children(move |scroll| {
                        scroll
                            .spawn(settings_content_node())
                            .with_children(move |content| {
                                if let Some(mode) = mode {
                                    spawn_mode_section(content, mode, window);
                                }
                                spawn_language_section(content, language, labels.language, window);
                                spawn_theme_section(
                                    content,
                                    exploration_theme,
                                    labels.theme,
                                    window,
                                );
                                let explanations = labels.explanations;
                                spawn_bool_setting_section(
                                    content,
                                    BoolSettingField::ShowExplanations,
                                    show_explanations,
                                    explanations.label,
                                    explanations.checkbox,
                                    window,
                                );
                                let navigation = labels.navigation;
                                spawn_bool_setting_section(
                                    content,
                                    BoolSettingField::GamepadNavigation,
                                    gamepad_navigation,
                                    navigation.label,
                                    navigation.checkbox,
                                    window,
                                );
                                spawn_volume_section(
                                    content,
                                    VolumeChannel::Music,
                                    music_volume,
                                    labels.music,
                                    window,
                                );
                                spawn_volume_section(
                                    content,
                                    VolumeChannel::Sfx,
                                    sfx_volume,
                                    labels.sfx,
                                    window,
                                );
                            });
                    });
                if let Some(notice) = notice {
                    card.spawn((
                        Text::new(notice),
                        theme::typography::text(theme::fonts::SMALL, window),
                        TextColor(theme::colors::ERROR),
                    ));
                }
                card.spawn(back);
            });
        });
}

fn settings_card_node() -> impl Bundle {
    (
        Node {
            width: percent(92.0),
            height: percent(94.0),
            min_width: px(0.0),
            min_height: px(0.0),
            border: px(1.0).all(),
            border_radius: BorderRadius::all(theme::scaled(28.0)),
            flex_direction: FlexDirection::Column,
            align_items: AlignItems::Center,
            justify_content: JustifyContent::FlexStart,
            padding: theme::scaled(theme::sizes::CARD_PADDING).all(),
            row_gap: theme::scaled(theme::spacing::LARGE),
            ..default()
        },
        BackgroundColor(Color::srgba(1.0, 1.0, 1.0, 0.88)),
        BorderColor::all(theme::colors::CARD_BORDER),
    )
}

fn settings_scroll_node() -> impl Bundle {
    (
        ScrollArea,
        Node {
            width: theme::layout::FILL_PARENT,
            flex_grow: 1.0,
            flex_basis: px(0.0),
            min_height: px(0.0),
            flex_direction: FlexDirection::Column,
            align_items: AlignItems::Center,
            overflow: Overflow::scroll_y(),
            ..default()
        },
    )
}

fn settings_content_node() -> Node {
    Node {
        width: theme::layout::FILL_PARENT,
        min_width: px(0.0),
        min_height: percent(100.0),
        flex_shrink: 0.0,
        flex_direction: FlexDirection::Column,
        align_items: AlignItems::Center,
        justify_content: JustifyContent::Center,
        row_gap: theme::scaled(theme::spacing::LARGE),
        ..default()
    }
}

fn settings_group_layout() -> CollectionLayout {
    CollectionLayout::adaptive(1.0, theme::scaled(theme::spacing::MEDIUM)).with_row_column_gaps(
        theme::scaled(theme::spacing::SMALL),
        theme::scaled(theme::spacing::MEDIUM),
    )
}

fn spawn_settings_pair(
    parent: &mut ChildSpawnerCommands<'_>,
    label: String,
    window: Entity,
    control_sizing: SettingsControlSizing,
    spawn_control: impl FnOnce(&mut ChildSpawnerCommands<'_>),
) {
    let mut label = Some(label);
    let mut spawn_control = Some(spawn_control);
    layout::spawn_collection(
        parent,
        2,
        settings_group_layout(),
        |index| {
            let cell = CollectionItemLayout::new(Val::Auto, 0.0, 0.0)
                .with_max_width(theme::layout::FILL_PARENT);
            if index == 1 {
                match control_sizing {
                    SettingsControlSizing::Computed => cell.with_layout_width(),
                    SettingsControlSizing::NestedFluidCollection => {
                        cell.with_nested_fluid_collection_width()
                    }
                }
            } else {
                cell.with_layout_width()
            }
        },
        |cell, index| match index {
            0 => {
                if let Some(label) = label.take() {
                    cell.spawn((
                        Text::new(label),
                        theme::typography::text(theme::fonts::HEADING, window),
                        TextColor(theme::colors::TEXT_DARK),
                    ));
                }
            }
            1 => {
                if let Some(spawn_control) = spawn_control.take() {
                    spawn_control(cell);
                }
            }
            _ => {}
        },
    );
}

fn spawn_mode_section(
    parent: &mut ChildSpawnerCommands<'_>,
    mode: ModePresentation,
    window: Entity,
) {
    let individual_checked = !mode.group_selected;
    let group_checked = mode.group_selected;
    let individual_label = mode.individual_label;
    let group_label = mode.group_label;
    spawn_settings_pair(
        parent,
        mode.label,
        window,
        SettingsControlSizing::NestedFluidCollection,
        move |control| {
            let mut group = crate::ui::components::spawn_radio_group(
                control,
                2,
                |_| CollectionItemLayout::new(Val::Auto, 0.0, 0.0),
                move |option, index| {
                    let (value, label, checked) = if index == 0 {
                        (false, &individual_label, individual_checked)
                    } else {
                        (true, &group_label, group_checked)
                    };
                    let mut button = option.spawn_empty();
                    button.insert(ModeRadio(value));
                    if checked {
                        button.insert(Checked);
                    }
                    let _ = button.apply_scene(radio_button_scene(label, window));
                },
            );
            group.observe(handle_mode_radio_change);
        },
    );
}

fn handle_mode_radio_change(
    event: On<ValueChange<Entity>>,
    radio_query: Query<(Entity, &ModeRadio)>,
    mut commands: Commands,
) {
    let Ok((_, selected)) = radio_query.get(event.value) else {
        return;
    };
    let group_selected = selected.0;
    commands.trigger(ModeSelectionRequested(group_selected));
    for (entity, radio) in radio_query.iter().collect::<Vec<_>>() {
        if radio.0 == group_selected {
            commands.entity(entity).insert(Checked);
        } else {
            commands.entity(entity).remove::<Checked>();
        }
    }
}

fn spawn_language_section(
    parent: &mut ChildSpawnerCommands<'_>,
    current_language: Language,
    labels: LanguageLabels,
    window: Entity,
) {
    let LanguageLabels {
        label,
        french: french_label,
        english: english_label,
    } = labels;
    let french_checked = current_language == Language::French;
    let english_checked = current_language == Language::English;
    spawn_settings_pair(
        parent,
        label,
        window,
        SettingsControlSizing::NestedFluidCollection,
        move |control| {
            let mut group = crate::ui::components::spawn_radio_group(
                control,
                2,
                |_| CollectionItemLayout::new(Val::Auto, 0.0, 0.0),
                move |option, index| {
                    let (language, label, checked) = if index == 0 {
                        (Language::French, &french_label, french_checked)
                    } else {
                        (Language::English, &english_label, english_checked)
                    };
                    let mut button = option.spawn_empty();
                    button.insert(LanguageRadio(language));
                    if checked {
                        button.insert(Checked);
                    }
                    let _ = button.apply_scene(radio_button_scene(label, window));
                },
            );
            group.observe(handle_language_radio_change);
        },
    );
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

fn rebuild_settings_on_presentation_change(
    mut commands: Commands,
    settings: Res<GameSettings>,
    presentation: Res<SettingsPresentation>,
    i18n: Res<I18n>,
    root_query: Query<Entity, With<SettingsRoot>>,
    primary_window: Single<Entity, With<PrimaryWindow>>,
) {
    for entity in &root_query {
        commands.entity(entity).despawn();
    }
    spawn_settings_ui(
        &mut commands,
        &settings,
        &presentation,
        &i18n,
        *primary_window,
    );
}

fn spawn_theme_section(
    parent: &mut ChildSpawnerCommands<'_>,
    current_theme: ExplorationTheme,
    labels: ThemeLabels,
    window: Entity,
) {
    let ThemeLabels {
        label,
        sky: sky_label,
        ocean: ocean_label,
        space: space_label,
        coming_soon,
    } = labels;
    let sky_checked = current_theme == ExplorationTheme::Sky;
    let ocean_checked = current_theme == ExplorationTheme::Ocean;
    let space_checked = current_theme == ExplorationTheme::Space;
    spawn_settings_pair(
        parent,
        label,
        window,
        SettingsControlSizing::NestedFluidCollection,
        move |control| {
            let mut group = crate::ui::components::spawn_radio_group(
                control,
                3,
                |_| CollectionItemLayout::new(Val::Auto, 0.0, 0.0),
                move |option, index| {
                    let mut button = option.spawn_empty();
                    match index {
                        0 => {
                            button.insert(ExplorationThemeRadio(ExplorationTheme::Sky));
                            if sky_checked {
                                button.insert(Checked);
                            }
                            let _ = button.apply_scene(radio_button_scene(&sky_label, window));
                        }
                        1 => {
                            button.insert(ExplorationThemeRadio(ExplorationTheme::Ocean));
                            if ocean_checked {
                                button.insert(Checked);
                            }
                            let _ = button.apply_scene(radio_button_muted_scene(
                                &ocean_label,
                                &coming_soon,
                                window,
                            ));
                        }
                        2 => {
                            button.insert(ExplorationThemeRadio(ExplorationTheme::Space));
                            if space_checked {
                                button.insert(Checked);
                            }
                            let _ = button.apply_scene(radio_button_muted_scene(
                                &space_label,
                                &coming_soon,
                                window,
                            ));
                        }
                        _ => {}
                    }
                },
            );
            group.observe(handle_theme_radio_change);
        },
    );
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

fn spawn_bool_setting_section(
    parent: &mut ChildSpawnerCommands<'_>,
    field: BoolSettingField,
    enabled: bool,
    label: String,
    checkbox_label: String,
    window: Entity,
) {
    spawn_settings_pair(
        parent,
        label,
        window,
        SettingsControlSizing::Computed,
        move |control| {
            let mut checkbox = control.spawn_empty();
            checkbox.insert((
                field,
                observe(checkbox_self_update),
                observe(handle_bool_setting_change),
            ));
            if enabled {
                checkbox.insert(Checked);
            }
            let _ = checkbox.apply_scene(checkbox_scene(&checkbox_label, window));
        },
    );
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
fn spawn_volume_section(
    parent: &mut ChildSpawnerCommands<'_>,
    channel: VolumeChannel,
    volume: f32,
    label: String,
    window: Entity,
) {
    spawn_settings_pair(
        parent,
        label,
        window,
        SettingsControlSizing::Computed,
        move |control| {
            let mut controls = control.spawn(layout::atomic_child_group(theme::scaled(
                theme::spacing::MEDIUM,
            )));
            controls.with_children(|controls| {
                let mut slider = controls.spawn_empty();
                slider.insert((
                    channel,
                    observe(slider_self_update),
                    observe(handle_volume_slider_change),
                    observe(persist_volume_on_pointer_release),
                    observe(persist_volume_on_pointer_cancel),
                ));
                let _ = slider.apply_scene(slider_scene(0.0, 1.0, volume, 0.05));

                let percent = ((volume.clamp(0.0, 1.0) * 100.0).round()) as u32;
                controls.spawn((
                    Text::new(format!("{percent} %")),
                    theme::typography::text(theme::fonts::BODY, window),
                    TextColor(theme::colors::TEXT_DARK),
                    channel,
                ));
            });
        },
    );
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

fn persist_volume_on_pointer_cancel(_event: On<PointerCancel>, mut settings: GameSettingsMut<'_>) {
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
