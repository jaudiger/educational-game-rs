use bevy::input_focus::AutoFocus;
use bevy::prelude::*;
use bevy::window::PrimaryWindow;

use crate::data::{
    ActiveSlot, ActiveStudent, ActiveTheme, ContentLibrary, GameSettings, LessonProgress, MapTheme,
    PlayerContext, SaveData, SelectedLesson, get_current_progress,
};
use crate::i18n::{I18n, TranslationKey};
use crate::states::{AppState, MapView};
use crate::ui::animation::FloatingCard;
use crate::ui::components::{HoverTooltip, button_base, screen_root, standard_button};
use crate::ui::theme;
use crate::ui::theme::DesignFontSize;

/// Map exploration screen showing available themes and lessons.
pub struct MapExplorationScreenPlugin;

impl Plugin for MapExplorationScreenPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(OnEnter(MapView::WorldOverview), setup_world_overview)
            .add_systems(
                Update,
                handle_world_overview.run_if(in_state(MapView::WorldOverview)),
            )
            .add_systems(OnEnter(MapView::ThemeDetail), setup_theme_detail)
            .add_systems(
                Update,
                handle_theme_detail.run_if(in_state(MapView::ThemeDetail)),
            );
    }
}

#[derive(Component, Reflect)]
struct ThemeButton(String);

#[derive(Component, Reflect)]
struct LessonButton(String);

#[derive(Component, Reflect)]
struct BackToSaveSlotsButton;

#[derive(Component, Reflect)]
struct BackToWorldOverviewButton;

/// Color parameters for a card-styled map button.
#[derive(Clone, Copy)]
struct CardColors {
    bg_available: Color,
    bg_unavailable: Color,
    text_available: Color,
    progress_color: Color,
    border_available: Color,
    border_unavailable: Color,
    title_shadow: Color,
    card_text_shadow: Color,
}

/// Shadow and floating-animation parameters for a card-styled map button.
#[derive(Clone, Copy)]
struct CardShadowAnim {
    shadow_alpha_available: f32,
    shadow_alpha_unavailable: f32,
    float_amplitude_available: f32,
    float_amplitude_unavailable: f32,
}

/// Pre-computed card dimensions used by `card_node_layout` and `insert_card_overlay`.
#[derive(Clone, Copy)]
struct CardDimensions {
    width: f32,
    min_height: f32,
    pad_h: f32,
    pad_v: f32,
    radius: f32,
}

/// Visual parameters for styled map card buttons.
///
/// When a `MapTheme` returns `Some(MapCardStyle)` from [`MapCardStyle::for_theme`],
/// map buttons are rendered as semi-transparent floating cards instead of flat
/// opaque buttons. Adding a new card-styled theme only requires a new match arm.
#[derive(Clone, Copy)]
struct MapCardStyle {
    colors: CardColors,
    dims: CardDimensions,
    shadow_anim: CardShadowAnim,
}

impl MapCardStyle {
    /// Returns theme-specific card styling, or `None` for default flat buttons.
    const fn for_theme(map_theme: MapTheme) -> Option<Self> {
        match map_theme {
            MapTheme::Sky => Some(Self {
                colors: CardColors {
                    bg_available: Color::srgba(1.0, 1.0, 1.0, 0.8),
                    bg_unavailable: Color::srgba(0.7, 0.7, 0.7, 0.5),
                    text_available: theme::colors::TEXT_DARK,
                    progress_color: theme::colors::PRIMARY,
                    border_available: Color::srgba(1.0, 1.0, 1.0, 0.6),
                    border_unavailable: Color::srgba(0.6, 0.6, 0.6, 0.4),
                    title_shadow: Color::srgba(1.0, 1.0, 1.0, 0.8),
                    card_text_shadow: Color::srgba(1.0, 1.0, 1.0, 0.6),
                },
                dims: CardDimensions {
                    width: theme::sizes::SKY_CARD_WIDTH,
                    min_height: theme::sizes::SKY_CARD_HEIGHT,
                    pad_h: theme::sizes::SKY_CARD_PADDING_H,
                    pad_v: theme::sizes::SKY_CARD_PADDING_V,
                    radius: theme::sizes::SKY_CARD_BORDER_RADIUS,
                },
                shadow_anim: CardShadowAnim {
                    shadow_alpha_available: 0.15,
                    shadow_alpha_unavailable: 0.08,
                    float_amplitude_available: theme::animation::FLOATING_AMPLITUDE,
                    float_amplitude_unavailable: theme::animation::FLOATING_AMPLITUDE_MUTED,
                },
            }),
            MapTheme::Ocean | MapTheme::Space => None,
        }
    }
}

/// Pre-computed data for a single theme button (all strings owned for `'static`).
struct ThemeButtonData {
    id: String,
    available: bool,
    title_text: String,
    coming_soon_text: String,
    completed: usize,
    completed_text: String,
}

fn setup_world_overview(
    mut commands: Commands,
    content: Res<ContentLibrary>,
    i18n: Res<I18n>,
    active_theme: Option<Res<ActiveTheme>>,
    mut next_map_view: ResMut<NextState<MapView>>,
    ctx: PlayerContext<'_>,
    primary_window: Single<Entity, With<PrimaryWindow>>,
) {
    // If ActiveTheme exists, the user is returning from LessonPlay/LessonSummary.
    // Skip WorldOverview and go directly to ThemeDetail.
    if active_theme.is_some() {
        next_map_view.set(MapView::ThemeDetail);
        return;
    }

    let window = *primary_window;

    // Pre-compute all strings before the SpawnWith closure.
    let title = i18n.t(&TranslationKey::WorldMap).into_owned();
    let back_label = i18n.t(&TranslationKey::Back).into_owned();

    let card_style = MapCardStyle::for_theme(ctx.settings.map_theme);

    let theme_buttons: Vec<ThemeButtonData> = content
        .themes
        .iter()
        .map(|theme_data| {
            let (completed, total_lessons) = count_completed_for_theme(
                theme_data,
                &ctx.save_data,
                &ctx.settings,
                ctx.active_slot.as_deref(),
                ctx.active_student.as_deref(),
            );
            ThemeButtonData {
                id: theme_data.id.clone(),
                available: theme_data.available,
                title_text: i18n.t(&theme_data.title_key).into_owned(),
                coming_soon_text: i18n.t(&TranslationKey::ComingSoon).into_owned(),
                completed,
                completed_text: i18n
                    .t(&TranslationKey::LessonsCompleted(completed, total_lessons))
                    .into_owned(),
            }
        })
        .collect();

    let mut root = commands.spawn((
        screen_root(),
        DespawnOnExit(AppState::MapExploration),
        DespawnOnEnter(MapView::ThemeDetail),
        Children::spawn(SpawnWith(move |parent: &mut ChildSpawner| {
            // Title
            let mut title_cmd = parent.spawn((
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
            if let Some(style) = card_style {
                title_cmd.insert(TextShadow {
                    color: style.colors.title_shadow,
                    offset: Vec2::new(0.0, 2.0),
                });
            }

            // Theme buttons grid
            parent
                .spawn(Node {
                    flex_direction: FlexDirection::Row,
                    column_gap: theme::scaled(theme::spacing::LARGE),
                    row_gap: theme::scaled(theme::spacing::LARGE),
                    justify_content: JustifyContent::Center,
                    align_items: AlignItems::Stretch,
                    flex_wrap: FlexWrap::Wrap,
                    ..default()
                })
                .with_children(|grid| {
                    for (i, data) in theme_buttons.iter().enumerate() {
                        spawn_theme_button(grid, data, i == 0, window, card_style, i);
                    }
                });

            // Back button
            parent.spawn((
                standard_button(
                    &back_label,
                    theme::colors::PRIMARY,
                    theme::scaled(theme::sizes::BUTTON_WIDTH),
                    window,
                ),
                BackToSaveSlotsButton,
            ));
        })),
    ));

    // Transparent background lets themed backgrounds show through.
    if card_style.is_some() {
        root.insert(BackgroundColor(Color::NONE));
    }
}

/// Resolves (background, text, progress) colors from an optional card style.
fn card_colors(
    card_style: Option<MapCardStyle>,
    available: bool,
    default_progress: Color,
) -> (Color, Color, Color) {
    card_style.map_or(
        if available {
            (
                theme::colors::PRIMARY,
                theme::colors::TEXT_LIGHT,
                default_progress,
            )
        } else {
            (
                theme::colors::TOGGLE_INACTIVE,
                theme::colors::TEXT_MUTED,
                theme::colors::TEXT_MUTED,
            )
        },
        |style| {
            if available {
                (
                    style.colors.bg_available,
                    style.colors.text_available,
                    style.colors.progress_color,
                )
            } else {
                (
                    style.colors.bg_unavailable,
                    theme::colors::TEXT_MUTED,
                    theme::colors::TEXT_MUTED,
                )
            }
        },
    )
}

/// Resolves card dimensions from an optional card style.
fn card_dimensions(card_style: Option<MapCardStyle>) -> CardDimensions {
    card_style.map_or(
        CardDimensions {
            width: theme::sizes::BUTTON_WIDTH,
            min_height: theme::sizes::BUTTON_HEIGHT,
            pad_h: theme::sizes::BUTTON_PADDING,
            pad_v: theme::spacing::SMALL,
            radius: theme::sizes::BUTTON_BORDER_RADIUS,
        },
        |style| style.dims,
    )
}

/// Returns the standard `Node` layout for a map card button.
fn card_node_layout(dims: &CardDimensions) -> Node {
    Node {
        width: theme::scaled(dims.width),
        min_height: theme::scaled(dims.min_height),
        padding: UiRect::axes(theme::scaled(dims.pad_h), theme::scaled(dims.pad_v)),
        justify_content: JustifyContent::Center,
        align_items: AlignItems::Center,
        overflow: Overflow::clip(),
        flex_direction: FlexDirection::Column,
        row_gap: theme::scaled(theme::spacing::SMALL),
        border_radius: BorderRadius::all(theme::scaled(dims.radius)),
        ..default()
    }
}

/// Inserts card overlay components (border, shadow, float) onto an existing button.
#[allow(clippy::cast_precision_loss)]
fn insert_card_overlay(
    button: &mut EntityWorldMut,
    style: MapCardStyle,
    available: bool,
    index: usize,
    dims: &CardDimensions,
) {
    let (border_color, shadow_alpha, amplitude) = if available {
        (
            style.colors.border_available,
            style.shadow_anim.shadow_alpha_available,
            style.shadow_anim.float_amplitude_available,
        )
    } else {
        (
            style.colors.border_unavailable,
            style.shadow_anim.shadow_alpha_unavailable,
            style.shadow_anim.float_amplitude_unavailable,
        )
    };

    button.insert((
        Node {
            border: px(1.5).all(),
            ..card_node_layout(dims)
        },
        BorderColor::all(border_color),
        BoxShadow::new(
            Color::srgba(0.0, 0.0, 0.0, shadow_alpha),
            Val::Px(0.0),
            Val::Px(4.0),
            Val::Px(12.0),
            Val::Px(0.0),
        ),
        FloatingCard {
            phase: index as f32 * theme::animation::FLOATING_PHASE_OFFSET,
            amplitude,
        },
    ));
}

/// Returns a centered text bundle for use inside card buttons.
fn card_text(text: &str, font_size: f32, color: Color, window: Entity) -> impl Bundle + use<> {
    (
        Text::new(text.to_owned()),
        TextFont {
            font_size,
            ..default()
        },
        TextColor(color),
        TextLayout::new_with_justify(Justify::Center),
        DesignFontSize {
            size: font_size,
            window,
        },
    )
}

fn spawn_theme_button(
    parent: &mut ChildSpawner,
    data: &ThemeButtonData,
    auto_focus: bool,
    window: Entity,
    card_style: Option<MapCardStyle>,
    index: usize,
) {
    let (bg, text_color, progress_color) = card_colors(
        card_style,
        data.available,
        theme::colors::TEXT_MUTED, // default progress color for theme buttons
    );
    let dims = card_dimensions(card_style);

    let mut button = parent.spawn((
        button_base(bg),
        card_node_layout(&dims),
        ThemeButton(data.id.clone()),
    ));

    if let Some(style) = card_style {
        insert_card_overlay(&mut button, style, data.available, index, &dims);
    }
    if auto_focus {
        button.insert(AutoFocus);
    }
    if !data.available {
        button.insert(HoverTooltip {
            message: data.coming_soon_text.clone(),
            window,
        });
    }

    button.with_children(|btn| {
        let mut title_cmd = btn.spawn(card_text(
            &data.title_text,
            theme::fonts::BUTTON,
            text_color,
            window,
        ));
        if let Some(style) = card_style
            && data.available
        {
            title_cmd.insert(TextShadow {
                color: style.colors.card_text_shadow,
                offset: Vec2::new(0.0, 1.0),
            });
        }

        if data.completed > 0 && data.available {
            btn.spawn(card_text(
                &data.completed_text,
                theme::fonts::SMALL,
                progress_color,
                window,
            ));
        }
    });
}

fn count_completed_for_theme(
    theme_data: &crate::data::content::Theme,
    save_data: &SaveData,
    settings: &GameSettings,
    active_slot: Option<&ActiveSlot>,
    active_student: Option<&ActiveStudent>,
) -> (usize, usize) {
    let total = theme_data.lessons.iter().filter(|l| l.available).count();
    let Some(slot) = active_slot else {
        return (0, total);
    };
    let progress = get_current_progress(
        save_data,
        settings.mode,
        **slot,
        active_student.map(|s| **s),
    );
    let completed = progress.map_or(0, |p| {
        theme_data
            .lessons
            .iter()
            .filter(|l| l.available && p.contains_key(&l.id))
            .count()
    });
    (completed, total)
}

fn handle_world_overview(
    theme_query: Query<(&Interaction, &ThemeButton), Changed<Interaction>>,
    back_query: Query<&Interaction, (Changed<Interaction>, With<BackToSaveSlotsButton>)>,
    content: Res<ContentLibrary>,
    mut commands: Commands,
    mut next_app_state: ResMut<NextState<AppState>>,
    mut next_map_view: ResMut<NextState<MapView>>,
) {
    // Handle theme button clicks
    for (interaction, theme_btn) in &theme_query {
        if *interaction == Interaction::Pressed
            && content.theme(&theme_btn.0).is_some_and(|t| t.available)
        {
            commands.insert_resource(ActiveTheme(theme_btn.0.clone()));
            next_map_view.set(MapView::ThemeDetail);
        }
    }

    // Handle back button
    for interaction in &back_query {
        if *interaction == Interaction::Pressed {
            // ActiveTheme is intentionally persistent across lesson states to enable
            // direct return to ThemeDetail. Removed only by explicit back-navigation.
            commands.remove_resource::<ActiveTheme>();
            // ActiveStudent is intentionally persistent across lesson states for class
            // mode progress tracking. Removed only when returning to SaveSlots.
            commands.remove_resource::<ActiveStudent>();
            next_app_state.set(AppState::SaveSlots);
        }
    }
}

/// Pre-computed data for a single lesson button (all strings owned for `'static`).
struct LessonButtonData {
    id: String,
    available: bool,
    title_text: String,
    coming_soon_text: String,
    best_percent_text: Option<String>,
}

fn setup_theme_detail(
    mut commands: Commands,
    content: Res<ContentLibrary>,
    active_theme: Res<ActiveTheme>,
    i18n: Res<I18n>,
    ctx: PlayerContext<'_>,
    primary_window: Single<Entity, With<PrimaryWindow>>,
) {
    let Some(theme_data) = content.theme(&active_theme) else {
        return;
    };

    let window = *primary_window;

    // Pre-compute all strings before the SpawnWith closure.
    let theme_title = i18n.t(&theme_data.title_key).into_owned();
    let back_label = i18n.t(&TranslationKey::BackToWorldMap).into_owned();

    let progress = ctx.active_slot.as_ref().and_then(|slot| {
        get_current_progress(
            &ctx.save_data,
            ctx.settings.mode,
            slot.0,
            ctx.active_student.as_ref().map(|s| s.0),
        )
    });

    let card_style = MapCardStyle::for_theme(ctx.settings.map_theme);

    let lesson_buttons: Vec<LessonButtonData> = theme_data
        .lessons
        .iter()
        .map(|lesson| {
            let best_percent = progress
                .and_then(|p| p.get(&lesson.id))
                .map(LessonProgress::percentage);
            LessonButtonData {
                id: lesson.id.clone(),
                available: lesson.available,
                title_text: i18n.t(&lesson.title_key).into_owned(),
                coming_soon_text: i18n.t(&TranslationKey::ComingSoon).into_owned(),
                best_percent_text: best_percent
                    .map(|pct| i18n.t(&TranslationKey::BestPercent(pct)).into_owned()),
            }
        })
        .collect();

    let mut root = commands.spawn((
        screen_root(),
        DespawnOnExit(AppState::MapExploration),
        DespawnOnEnter(MapView::WorldOverview),
        Children::spawn(SpawnWith(move |parent: &mut ChildSpawner| {
            // Title
            let mut title_cmd = parent.spawn((
                Text::new(theme_title),
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
            if let Some(style) = card_style {
                title_cmd.insert(TextShadow {
                    color: style.colors.title_shadow,
                    offset: Vec2::new(0.0, 2.0),
                });
            }

            // Lesson buttons
            parent
                .spawn(Node {
                    flex_direction: FlexDirection::Row,
                    column_gap: theme::scaled(theme::spacing::LARGE),
                    row_gap: theme::scaled(theme::spacing::LARGE),
                    justify_content: JustifyContent::Center,
                    align_items: AlignItems::Stretch,
                    flex_wrap: FlexWrap::Wrap,
                    ..default()
                })
                .with_children(|grid| {
                    for (i, data) in lesson_buttons.iter().enumerate() {
                        spawn_lesson_button(grid, data, i == 0, window, card_style, i);
                    }
                });

            // Back button
            parent.spawn((
                standard_button(
                    &back_label,
                    theme::colors::PRIMARY,
                    theme::scaled(theme::sizes::BUTTON_WIDTH),
                    window,
                ),
                BackToWorldOverviewButton,
            ));
        })),
    ));

    // Transparent background lets themed backgrounds show through.
    if card_style.is_some() {
        root.insert(BackgroundColor(Color::NONE));
    }
}

fn spawn_lesson_button(
    parent: &mut ChildSpawner,
    data: &LessonButtonData,
    auto_focus: bool,
    window: Entity,
    card_style: Option<MapCardStyle>,
    index: usize,
) {
    let (bg, text_color, progress_color) = card_colors(
        card_style,
        data.available,
        theme::colors::SUCCESS, // default progress color for lesson buttons
    );
    let dims = card_dimensions(card_style);

    let mut button = parent.spawn((
        button_base(bg),
        card_node_layout(&dims),
        LessonButton(data.id.clone()),
    ));

    if let Some(style) = card_style {
        insert_card_overlay(&mut button, style, data.available, index, &dims);
    }
    if auto_focus {
        button.insert(AutoFocus);
    }
    if !data.available {
        button.insert(HoverTooltip {
            message: data.coming_soon_text.clone(),
            window,
        });
    }

    button.with_children(|btn| {
        let mut title_cmd = btn.spawn(card_text(
            &data.title_text,
            theme::fonts::BUTTON,
            text_color,
            window,
        ));
        if let Some(style) = card_style
            && data.available
        {
            title_cmd.insert(TextShadow {
                color: style.colors.card_text_shadow,
                offset: Vec2::new(0.0, 1.0),
            });
        }

        if data.available
            && let Some(ref text) = data.best_percent_text
        {
            btn.spawn(card_text(text, theme::fonts::SMALL, progress_color, window));
        }
    });
}

fn handle_theme_detail(
    lesson_query: Query<(&Interaction, &LessonButton), Changed<Interaction>>,
    back_query: Query<&Interaction, (Changed<Interaction>, With<BackToWorldOverviewButton>)>,
    content: Res<ContentLibrary>,
    active_theme: Res<ActiveTheme>,
    mut commands: Commands,
    mut next_app_state: ResMut<NextState<AppState>>,
    mut next_map_view: ResMut<NextState<MapView>>,
) {
    // Handle lesson button clicks
    for (interaction, lesson_btn) in &lesson_query {
        if *interaction == Interaction::Pressed
            && let Some(theme_data) = content.theme(&active_theme)
            && theme_data
                .lesson(&lesson_btn.0)
                .is_some_and(|l| l.available)
        {
            commands.insert_resource(SelectedLesson(Some(lesson_btn.0.clone())));
            next_app_state.set(AppState::LessonPlay);
        }
    }

    // Handle back button
    for interaction in &back_query {
        if *interaction == Interaction::Pressed {
            // ActiveTheme is intentionally persistent across lesson states to enable
            // direct return to ThemeDetail. Removed only by explicit back-navigation.
            commands.remove_resource::<ActiveTheme>();
            next_map_view.set(MapView::WorldOverview);
        }
    }
}
