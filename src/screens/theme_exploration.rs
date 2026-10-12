use std::collections::HashMap;

use bevy::color::Luminance;
use bevy::input_focus::AutoFocus;
use bevy::picking::hover::Hovered;
use bevy::prelude::*;
use bevy::ui::Pressed;
use bevy::window::PrimaryWindow;

use crate::data::{
    ActiveStudent, ActiveTheme, ContentLibrary, ExplorationTheme, GameMode, GameSettings,
    LessonLaunchSelection, LessonProgress, PersistenceAction, PersistenceStatus, PlayerContext,
    PlayerSession, SaveData, SelectedLesson, get_current_progress,
};
use crate::i18n::{I18n, TranslationKey};
use crate::states::{AppState, ExplorationView, InLessonFlow, StateScopedResourceExt};
use crate::ui::animation::{AnimatedButton, FloatingCard};
use crate::ui::components::{
    ButtonActivated, HoverTooltip, button_base, persistence_notice, screen_root, standard_button,
};
use crate::ui::theme;

/// Theme exploration screen showing themes and their lessons.
pub struct ThemeExplorationScreenPlugin;

impl Plugin for ThemeExplorationScreenPlugin {
    fn build(&self, app: &mut App) {
        app.register_state_scoped_resource::<InLessonFlow, ActiveStudent>(InLessonFlow)
            .register_state_scoped_resource::<InLessonFlow, ActiveTheme>(InLessonFlow)
            .add_systems(OnEnter(ExplorationView::Themes), setup_themes)
            .add_systems(
                Update,
                handle_themes.run_if(in_state(ExplorationView::Themes)),
            )
            .add_systems(OnEnter(ExplorationView::ThemeLessons), setup_theme_lessons)
            .add_systems(
                Update,
                update_exploration_progress.run_if(
                    in_state(AppState::ThemeExploration).and_then(
                        resource_changed_or_removed::<ActiveStudent>
                            .or_else(resource_changed::<SaveData>),
                    ),
                ),
            )
            .add_systems(
                Update,
                (
                    handle_theme_lessons.run_if(in_state(ExplorationView::ThemeLessons)),
                    update_exploration_card_hover.run_if(in_state(AppState::ThemeExploration)),
                ),
            );
    }
}

#[derive(Component, Reflect)]
struct ExplorationScreenRoot;

#[derive(Component, Reflect)]
struct ThemeButton(String);

#[derive(Clone, Copy, Reflect)]
enum LessonLaunchAction {
    CompleteSeries,
    ChooseExercise,
}

#[derive(Component, Reflect)]
struct LessonActionButton {
    lesson_id: String,
    action: LessonLaunchAction,
}

#[derive(Component, Reflect)]
enum ExplorationProgressText {
    Theme(String),
    Lesson(String),
}

#[derive(Component, Reflect)]
struct BackToSaveSlotsButton;

#[derive(Component, Reflect)]
struct BackToThemesButton;

/// Colors used by exploration cards that cannot use transform-based hover animation.
#[derive(Component, Reflect)]
struct ExplorationCardHover {
    base: Color,
    hovered: Color,
    pressed: Color,
}

/// Color parameters for a card-styled exploration button.
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
    shadow_base: Color,
    back_button_bg: Color,
}

/// Shadow and floating-animation parameters for a card-styled exploration button.
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

/// Visual parameters for exploration buttons styled by the selected theme.
///
/// The selected theme controls whether buttons use floating cards or the standard
/// flat style.
#[derive(Clone, Copy)]
struct ExplorationCardStyle {
    colors: CardColors,
    dims: CardDimensions,
    shadow_anim: CardShadowAnim,
}

impl ExplorationCardStyle {
    /// Returns theme-specific card styling, or `None` for default flat buttons.
    const fn for_exploration_theme(exploration_theme: ExplorationTheme) -> Option<Self> {
        match exploration_theme {
            ExplorationTheme::Sky => Some(Self {
                colors: CardColors {
                    bg_available: Color::srgba(0.88, 0.93, 1.0, 0.82),
                    bg_unavailable: Color::srgba(0.78, 0.82, 0.90, 0.45),
                    text_available: theme::colors::TEXT_DARK,
                    progress_color: theme::colors::PRIMARY,
                    border_available: Color::srgba(0.72, 0.86, 1.0, 0.55),
                    border_unavailable: Color::srgba(0.58, 0.65, 0.75, 0.35),
                    title_shadow: Color::srgba(0.72, 0.86, 1.0, 0.7),
                    card_text_shadow: Color::srgba(0.78, 0.90, 1.0, 0.5),
                    shadow_base: Color::srgba(0.08, 0.12, 0.28, 1.0),
                    back_button_bg: Color::srgb(0.30, 0.55, 0.88),
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
            ExplorationTheme::Ocean | ExplorationTheme::Space => None,
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

const fn restored_exploration_view(
    has_active_theme: bool,
    launch_selection: Option<&LessonLaunchSelection>,
) -> Option<ExplorationView> {
    if !has_active_theme {
        return None;
    }
    Some(match launch_selection {
        Some(LessonLaunchSelection::SingleExercise { .. }) => ExplorationView::ExerciseSelection,
        Some(LessonLaunchSelection::CompleteLesson) | None => ExplorationView::ThemeLessons,
    })
}

#[allow(clippy::too_many_arguments)]
fn setup_themes(
    mut commands: Commands,
    content: Res<ContentLibrary>,
    i18n: Res<I18n>,
    status: Res<PersistenceStatus>,
    active_theme: Option<Res<ActiveTheme>>,
    launch_selection: Option<Res<LessonLaunchSelection>>,
    mut next_exploration_view: ResMut<NextState<ExplorationView>>,
    ctx: PlayerContext<'_>,
    primary_window: Single<Entity, With<PrimaryWindow>>,
    existing_roots: Query<Entity, With<ExplorationScreenRoot>>,
) {
    // Preserve the lesson grid or exercise list when returning from play.
    if let Some(view) =
        restored_exploration_view(active_theme.is_some(), launch_selection.as_deref())
    {
        next_exploration_view.set(view);
        return;
    }

    for root in &existing_roots {
        commands.entity(root).despawn();
    }

    let window = *primary_window;
    let write_notice = persistence_notice(PersistenceAction::SaveSlot, *status, &i18n, window);

    // Pre-compute all strings before the SpawnWith closure.
    let title = i18n.t(&TranslationKey::Themes).into_owned();
    let back_label = i18n.t(&TranslationKey::Back).into_owned();

    let card_style = ExplorationCardStyle::for_exploration_theme(ctx.settings.exploration_theme);
    let back_btn_color = card_style.map_or(theme::colors::PRIMARY, |s| s.colors.back_button_bg);

    let theme_buttons: Vec<ThemeButtonData> = content
        .themes
        .iter()
        .map(|theme_data| {
            let (completed, total_lessons) = if exploration_progress_visible(ctx.settings.mode) {
                count_completed_for_theme(
                    theme_data,
                    &ctx.save_data,
                    &ctx.settings,
                    ctx.session.as_deref(),
                    ctx.active_student.as_deref(),
                )
            } else {
                (0, theme_data.lessons.len())
            };
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
        ExplorationScreenRoot,
        DespawnOnExit(AppState::ThemeExploration),
        DespawnOnEnter(ExplorationView::ThemeLessons),
        Children::spawn(SpawnWith(move |parent: &mut ChildSpawner| {
            // Title
            let mut title_cmd = parent.spawn((
                Text::new(title),
                theme::typography::text(theme::fonts::TITLE, window),
                TextColor(theme::colors::TEXT_DARK),
            ));
            if let Some(style) = card_style {
                title_cmd.insert(TextShadow {
                    color: style.colors.title_shadow,
                    offset: Vec2::new(0.0, 2.0),
                });
            }

            parent.spawn(write_notice);

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
                    back_btn_color,
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
    card_style: Option<ExplorationCardStyle>,
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
fn card_dimensions(card_style: Option<ExplorationCardStyle>) -> CardDimensions {
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

/// Returns the standard `Node` layout for an exploration button.
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
    style: ExplorationCardStyle,
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

    let shadow_color = style.colors.shadow_base.with_alpha(shadow_alpha);

    button.insert((
        Node {
            border: px(2.0).all(),
            ..card_node_layout(dims)
        },
        BorderColor::all(border_color),
        BoxShadow::new(
            shadow_color,
            Val::Px(0.0),
            Val::Px(4.0),
            Val::Px(16.0),
            Val::Px(2.0),
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
        Text::new(text),
        theme::typography::text(font_size, window),
        TextColor(color),
        TextLayout::justify(Justify::Center),
    )
}

fn spawn_theme_button(
    parent: &mut ChildSpawner,
    data: &ThemeButtonData,
    auto_focus: bool,
    window: Entity,
    card_style: Option<ExplorationCardStyle>,
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
    button.remove::<AnimatedButton>();

    button.insert(ExplorationCardHover {
        base: bg,
        hovered: bg.lighter(0.08),
        pressed: bg.darker(0.08),
    });

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

        if data.available {
            btn.spawn((
                card_text(
                    &data.completed_text,
                    theme::fonts::SMALL,
                    progress_color,
                    window,
                ),
                ExplorationProgressText::Theme(data.id.clone()),
                Node {
                    display: if data.completed > 0 {
                        Display::Flex
                    } else {
                        Display::None
                    },
                    ..default()
                },
            ));
        }
    });
}

fn count_completed_for_theme(
    theme_data: &crate::data::content::Theme,
    save_data: &SaveData,
    settings: &GameSettings,
    session: Option<&PlayerSession>,
    active_student: Option<&ActiveStudent>,
) -> (usize, usize) {
    let total = theme_data.lessons.len();
    let Some(session) = session else {
        return (0, total);
    };
    let progress = get_current_progress(
        save_data,
        settings.mode,
        session.slot_index,
        active_student.map(|s| **s),
    );
    let completed = progress.map_or(0, |p| {
        theme_data
            .lessons
            .iter()
            .filter(|lesson| p.contains_key(&lesson.id))
            .count()
    });
    (completed, total)
}

fn update_exploration_card_hover(
    mut query: Query<(
        &Hovered,
        Has<Pressed>,
        &ExplorationCardHover,
        &mut BackgroundColor,
    )>,
) {
    for (hovered, pressed, card_hover, mut background) in &mut query {
        let color = if pressed {
            card_hover.pressed
        } else if hovered.get() {
            card_hover.hovered
        } else {
            card_hover.base
        };
        if background.0 != color {
            background.0 = color;
        }
    }
}

fn handle_themes(
    mut activations: MessageReader<ButtonActivated>,
    theme_query: Query<&ThemeButton>,
    back_query: Query<(), With<BackToSaveSlotsButton>>,
    content: Res<ContentLibrary>,
    mut commands: Commands,
    mut next_app_state: ResMut<NextState<AppState>>,
    mut next_exploration_view: ResMut<NextState<ExplorationView>>,
) {
    for activation in activations.read() {
        if let Ok(theme_btn) = theme_query.get(activation.0) {
            if content.theme(&theme_btn.0).is_some_and(|t| t.available) {
                commands.insert_resource(ActiveTheme(theme_btn.0.clone()));
                next_exploration_view.set(ExplorationView::ThemeLessons);
            }
        } else if back_query.contains(activation.0) {
            next_app_state.set(AppState::SaveSlots);
        }
    }
}

/// Pre-computed data for a single lesson button (all strings owned for `'static`).
struct LessonButtonData {
    id: String,
    title_text: String,
    best_percent_text: Option<String>,
}

fn build_lesson_button_data(
    theme_data: &crate::data::content::Theme,
    progress: Option<&HashMap<String, LessonProgress>>,
    i18n: &I18n,
) -> Vec<LessonButtonData> {
    theme_data
        .lessons
        .iter()
        .map(|lesson| {
            let best_percent = progress
                .and_then(|progress| progress.get(&lesson.id))
                .map(LessonProgress::percentage);
            LessonButtonData {
                id: lesson.id.clone(),
                title_text: i18n.t(&lesson.title_key).into_owned(),
                best_percent_text: best_percent.map(|percentage| {
                    i18n.t(&TranslationKey::BestPercent(percentage))
                        .into_owned()
                }),
            }
        })
        .collect()
}

struct LessonActionLabels {
    play_series: String,
    choose_exercise: String,
}

#[allow(clippy::too_many_arguments)]
fn setup_theme_lessons(
    mut commands: Commands,
    content: Res<ContentLibrary>,
    active_theme: Res<ActiveTheme>,
    i18n: Res<I18n>,
    status: Res<PersistenceStatus>,
    ctx: PlayerContext<'_>,
    primary_window: Single<Entity, With<PrimaryWindow>>,
    existing_roots: Query<Entity, With<ExplorationScreenRoot>>,
) {
    let Some(theme_data) = content.theme(&active_theme) else {
        return;
    };

    for root in &existing_roots {
        commands.entity(root).despawn();
    }

    let window = *primary_window;
    let write_notice = persistence_notice(PersistenceAction::SaveSlot, *status, &i18n, window);

    // Pre-compute all strings before the SpawnWith closure.
    let theme_title = i18n.t(&theme_data.title_key).into_owned();
    let back_label = i18n.t(&TranslationKey::BackToThemes).into_owned();

    let progress = if exploration_progress_visible(ctx.settings.mode) {
        ctx.session.as_ref().and_then(|slot| {
            get_current_progress(
                &ctx.save_data,
                ctx.settings.mode,
                slot.slot_index,
                ctx.active_student.as_ref().map(|student| student.0),
            )
        })
    } else {
        None
    };

    let card_style = ExplorationCardStyle::for_exploration_theme(ctx.settings.exploration_theme);
    let back_btn_color = card_style.map_or(theme::colors::PRIMARY, |s| s.colors.back_button_bg);

    let action_labels = LessonActionLabels {
        play_series: i18n.t(&TranslationKey::PlaySeries).into_owned(),
        choose_exercise: i18n.t(&TranslationKey::ChooseExercise).into_owned(),
    };

    let lesson_buttons = build_lesson_button_data(theme_data, progress, &i18n);

    let mut root = commands.spawn((
        screen_root(),
        ExplorationScreenRoot,
        DespawnOnExit(AppState::ThemeExploration),
        DespawnOnEnter(ExplorationView::ExerciseSelection),
        Children::spawn(SpawnWith(move |parent: &mut ChildSpawner| {
            // Title
            let mut title_cmd = parent.spawn((
                Text::new(theme_title),
                theme::typography::text(theme::fonts::TITLE, window),
                TextColor(theme::colors::TEXT_DARK),
            ));
            if let Some(style) = card_style {
                title_cmd.insert(TextShadow {
                    color: style.colors.title_shadow,
                    offset: Vec2::new(0.0, 2.0),
                });
            }

            parent.spawn(write_notice);

            // Lesson actions
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
                        spawn_lesson_button(
                            grid,
                            data,
                            i == 0,
                            &action_labels,
                            window,
                            card_style,
                            i,
                        );
                    }
                });

            // Back button
            parent.spawn((
                standard_button(
                    &back_label,
                    back_btn_color,
                    theme::scaled(theme::sizes::BUTTON_WIDTH),
                    window,
                ),
                BackToThemesButton,
            ));
        })),
    ));

    // Transparent background lets themed backgrounds show through.
    if card_style.is_some() {
        root.insert(BackgroundColor(Color::NONE));
    }
}

const fn exploration_progress_visible(mode: GameMode) -> bool {
    matches!(mode, GameMode::Individual)
}

fn update_exploration_progress(
    ctx: PlayerContext<'_>,
    content: Res<ContentLibrary>,
    i18n: Res<I18n>,
    active_theme: Option<Res<ActiveTheme>>,
    exploration_view: Res<State<ExplorationView>>,
    mut progress_query: Query<(&ExplorationProgressText, &mut Text, &mut Node)>,
) {
    let theme_data = active_theme
        .as_deref()
        .and_then(|theme| content.theme(&theme.0));
    let progress = if exploration_progress_visible(ctx.settings.mode) {
        ctx.session.as_deref().and_then(|session| {
            get_current_progress(
                &ctx.save_data,
                ctx.settings.mode,
                session.slot_index,
                ctx.active_student.as_deref().map(|student| student.0),
            )
        })
    } else {
        None
    };

    for (progress_kind, mut text, mut node) in &mut progress_query {
        let label = if exploration_progress_visible(ctx.settings.mode) {
            match (*exploration_view.get(), progress_kind) {
                (ExplorationView::Themes, ExplorationProgressText::Theme(theme_id)) => {
                    content.theme(theme_id).and_then(|theme_data| {
                        let (completed, total) = count_completed_for_theme(
                            theme_data,
                            &ctx.save_data,
                            &ctx.settings,
                            ctx.session.as_deref(),
                            ctx.active_student.as_deref(),
                        );
                        (completed > 0).then(|| {
                            i18n.t(&TranslationKey::LessonsCompleted(completed, total))
                                .into_owned()
                        })
                    })
                }
                (ExplorationView::ThemeLessons, ExplorationProgressText::Lesson(lesson_id)) => {
                    theme_data
                        .and_then(|theme| theme.lesson(lesson_id))
                        .and_then(|lesson| progress.and_then(|progress| progress.get(&lesson.id)))
                        .map(|progress| {
                            i18n.t(&TranslationKey::BestPercent(progress.percentage()))
                                .into_owned()
                        })
                }
                _ => continue,
            }
        } else {
            None
        };
        set_exploration_progress_text(&mut text, &mut node, label);
    }
}

fn set_exploration_progress_text(text: &mut Text, node: &mut Node, label: Option<String>) {
    if let Some(label) = label {
        if **text != label {
            **text = label;
        }
        if node.display != Display::Flex {
            node.display = Display::Flex;
        }
    } else {
        if !text.is_empty() {
            **text = String::new();
        }
        if node.display != Display::None {
            node.display = Display::None;
        }
    }
}

fn spawn_lesson_button(
    parent: &mut ChildSpawner,
    data: &LessonButtonData,
    auto_focus: bool,
    action_labels: &LessonActionLabels,
    window: Entity,
    card_style: Option<ExplorationCardStyle>,
    index: usize,
) {
    let (bg, text_color, progress_color) = card_colors(card_style, true, theme::colors::SUCCESS);
    let dims = card_dimensions(card_style);
    let mut card = parent.spawn((
        Node {
            border: px(1.0).all(),
            ..card_node_layout(&dims)
        },
        BackgroundColor(bg),
        BorderColor::all(theme::colors::INPUT_BORDER),
        Hovered::default(),
    ));
    if let Some(style) = card_style {
        insert_card_overlay(&mut card, style, true, index, &dims);
    }

    card.with_children(|contents| {
        let mut title_cmd = contents.spawn(card_text(
            &data.title_text,
            theme::fonts::BUTTON,
            text_color,
            window,
        ));
        if let Some(style) = card_style {
            title_cmd.insert(TextShadow {
                color: style.colors.card_text_shadow,
                offset: Vec2::new(0.0, 1.0),
            });
        }

        let has_progress = data.best_percent_text.is_some();
        let progress = data.best_percent_text.as_deref().unwrap_or_default();
        contents.spawn((
            card_text(progress, theme::fonts::SMALL, progress_color, window),
            ExplorationProgressText::Lesson(data.id.clone()),
            Node {
                display: if has_progress {
                    Display::Flex
                } else {
                    Display::None
                },
                ..default()
            },
        ));
        contents
            .spawn(Node {
                width: percent(100.0),
                flex_direction: FlexDirection::Column,
                row_gap: theme::scaled(theme::spacing::SMALL),
                ..default()
            })
            .with_children(|actions| {
                spawn_lesson_action(
                    actions,
                    data,
                    LessonLaunchAction::CompleteSeries,
                    &action_labels.play_series,
                    auto_focus,
                    window,
                );
                spawn_lesson_action(
                    actions,
                    data,
                    LessonLaunchAction::ChooseExercise,
                    &action_labels.choose_exercise,
                    false,
                    window,
                );
            });
    });
}

fn spawn_lesson_action(
    parent: &mut ChildSpawner,
    data: &LessonButtonData,
    action: LessonLaunchAction,
    label: &str,
    auto_focus: bool,
    window: Entity,
) {
    let color = match action {
        LessonLaunchAction::CompleteSeries => theme::colors::PRIMARY,
        LessonLaunchAction::ChooseExercise => theme::colors::SECONDARY,
    };
    let mut button = parent.spawn((
        button_base(color),
        Node {
            width: percent(100.0),
            min_height: theme::scaled(40.0),
            padding: UiRect::axes(theme::scaled(theme::spacing::SMALL), theme::scaled(4.0)),
            justify_content: JustifyContent::Center,
            align_items: AlignItems::Center,
            border_radius: BorderRadius::all(theme::scaled(theme::sizes::BUTTON_BORDER_RADIUS)),
            ..default()
        },
        LessonActionButton {
            lesson_id: data.id.clone(),
            action,
        },
        ExplorationCardHover {
            base: color,
            hovered: color.lighter(0.08),
            pressed: color.darker(0.08),
        },
        children![(
            Text::new(label.to_owned()),
            theme::typography::text(theme::fonts::SMALL, window),
            TextColor(theme::colors::TEXT_LIGHT),
            TextLayout::justify(Justify::Center),
        )],
    ));
    if auto_focus {
        button.insert(AutoFocus);
    }
}

#[allow(clippy::too_many_arguments)]
fn handle_theme_lessons(
    mut activations: MessageReader<ButtonActivated>,
    lesson_query: Query<&LessonActionButton>,
    back_query: Query<(), With<BackToThemesButton>>,
    content: Res<ContentLibrary>,
    active_theme: Res<ActiveTheme>,
    mut commands: Commands,
    mut next_app_state: ResMut<NextState<AppState>>,
    mut next_exploration_view: ResMut<NextState<ExplorationView>>,
) {
    for activation in activations.read() {
        if let Ok(lesson_btn) = lesson_query.get(activation.0) {
            if let Some(theme_data) = content.theme(&active_theme)
                && theme_data.lesson(&lesson_btn.lesson_id).is_some()
            {
                commands.insert_resource(SelectedLesson(Some(lesson_btn.lesson_id.clone())));
                match lesson_btn.action {
                    LessonLaunchAction::CompleteSeries => {
                        commands.insert_resource(LessonLaunchSelection::CompleteLesson);
                        next_app_state.set(AppState::LessonPlay);
                    }
                    LessonLaunchAction::ChooseExercise => {
                        commands.insert_resource(LessonLaunchSelection::CompleteLesson);
                        next_exploration_view.set(ExplorationView::ExerciseSelection);
                    }
                }
            }
        } else if back_query.contains(activation.0) {
            commands.remove_resource::<ActiveTheme>();
            next_exploration_view.set(ExplorationView::Themes);
        }
    }
}
