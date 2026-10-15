use bevy::color::Luminance;
use bevy::input_focus::AutoFocus;
use bevy::prelude::*;
use bevy::window::PrimaryWindow;

use crate::data::{
    ActiveTheme, ContentLibrary, ExplorationTheme, GameSettings, LessonLaunchSelection,
    SelectedLesson,
};
use crate::states::{AppState, ExplorationView, InLessonFlow, StateScopedResourceExt};
use crate::ui::animation::{AnimatedButton, FloatingCard};
use crate::ui::components::{
    ButtonActivated, HoverTooltip, button_base, screen_root, standard_button,
};
use crate::ui::theme;

/// Theme exploration screen showing themes and their lessons.
pub struct ThemeExplorationScreenPlugin;

#[derive(SystemSet, Clone, Copy, Debug, Eq, Hash, PartialEq)]
pub enum ExplorationPresentationSet {
    Prepare,
    Render,
}

impl Plugin for ThemeExplorationScreenPlugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<ThemeOverviewPresentation>()
            .init_resource::<ThemeLessonsPresentation>()
            .configure_sets(
                Update,
                (
                    ExplorationPresentationSet::Prepare,
                    ExplorationPresentationSet::Render,
                )
                    .chain(),
            )
            .register_state_scoped_resource::<InLessonFlow, ActiveTheme>(InLessonFlow)
            .add_systems(OnEnter(ExplorationView::Themes), setup_themes)
            .add_systems(
                Update,
                handle_themes.run_if(in_state(ExplorationView::Themes)),
            )
            .add_systems(OnEnter(ExplorationView::ThemeLessons), setup_theme_lessons)
            .add_systems(
                Update,
                handle_theme_lessons.run_if(in_state(ExplorationView::ThemeLessons)),
            )
            .add_systems(
                Update,
                update_exploration_notice
                    .in_set(ExplorationPresentationSet::Render)
                    .run_if(
                        in_state(AppState::ThemeExploration).and_then(
                            resource_changed::<ThemeOverviewPresentation>
                                .or_else(resource_changed::<ThemeLessonsPresentation>),
                        ),
                    ),
            );
    }
}

#[derive(Component, Reflect)]
pub struct ExplorationScreenRoot;

#[derive(Component, Reflect)]
struct ExplorationNoticeText;

#[derive(Component, Reflect)]
pub(super) struct ThemeButton(pub String);

#[derive(Clone, Copy, Debug, Eq, PartialEq, Reflect)]
pub enum LessonLaunchAction {
    CompleteSeries,
    ChooseExercise,
}

#[derive(Component, Reflect)]
struct LessonActionButton {
    lesson_id: String,
    action: LessonLaunchAction,
    selection: Option<LessonLaunchSelection>,
}

#[derive(Component, Reflect)]
pub enum ExplorationProgressText {
    Theme(String),
    Lesson(String),
}

#[derive(Component)]
struct ExploreBackAction(AppState);

#[derive(Component, Reflect)]
struct BackToThemesButton {
    clear_launch_context: bool,
}

#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct ThemeButtonData {
    pub id: String,
    pub available: bool,
    pub title_text: String,
    pub coming_soon_text: String,
    pub progress_text: Option<String>,
    pub track_progress: bool,
}

#[derive(Resource, Clone, Debug, Default, PartialEq, Eq)]
pub struct ThemeOverviewPresentation {
    pub title: String,
    pub back_label: String,
    pub back_destination: AppState,
    pub notice: Option<String>,
    pub themes: Vec<ThemeButtonData>,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct LessonActionPresentation {
    pub action: LessonLaunchAction,
    pub label: String,
    pub selection: Option<LessonLaunchSelection>,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct LessonButtonData {
    pub id: String,
    pub title_text: String,
    pub best_percent_text: Option<String>,
    pub track_progress: bool,
    pub actions: Vec<LessonActionPresentation>,
}

#[derive(Resource, Clone, Debug, Default, PartialEq, Eq)]
pub struct ThemeLessonsPresentation {
    pub available: bool,
    pub title: String,
    pub back_label: String,
    pub notice: Option<String>,
    pub clear_launch_context: bool,
    pub lessons: Vec<LessonButtonData>,
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

/// Color parameters for a card-styled exploration button.
#[derive(Clone, Copy)]
struct CardColors {
    bg_available: Color,
    bg_unavailable: Color,
    text_available: Color,
    border_available: Color,
    border_unavailable: Color,
    title_shadow: Color,
    shadow_base: Color,
}

/// Shadow and floating-animation parameters for exploration buttons.
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
                    bg_available: Color::srgba(0.70, 0.82, 0.95, 0.96),
                    bg_unavailable: Color::srgba(0.58, 0.68, 0.81, 0.88),
                    text_available: theme::colors::TEXT_DARK,
                    border_available: Color::srgba(0.30, 0.55, 0.84, 0.96),
                    border_unavailable: Color::srgba(0.40, 0.51, 0.68, 0.78),
                    title_shadow: Color::srgba(0.54, 0.72, 0.94, 0.66),
                    shadow_base: Color::srgba(0.07, 0.14, 0.29, 1.0),
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

#[allow(clippy::too_many_arguments)]
pub fn setup_themes(
    mut commands: Commands,
    presentation: Res<ThemeOverviewPresentation>,
    settings: Res<GameSettings>,
    active_theme: Option<Res<ActiveTheme>>,
    launch_selection: Option<Res<LessonLaunchSelection>>,
    mut next_exploration_view: ResMut<NextState<ExplorationView>>,
    primary_window: Single<Entity, With<PrimaryWindow>>,
    existing_roots: Query<Entity, With<ExplorationScreenRoot>>,
) {
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
    let title = presentation.title.clone();
    let back_label = presentation.back_label.clone();
    let notice = presentation.notice.clone();
    let themes = presentation.themes.clone();
    let back_destination = presentation.back_destination;
    let card_style = ExplorationCardStyle::for_exploration_theme(settings.exploration_theme);

    let mut root = commands.spawn((
        screen_root(),
        ExplorationScreenRoot,
        DespawnOnExit(AppState::ThemeExploration),
        DespawnOnEnter(ExplorationView::ThemeLessons),
        Children::spawn(SpawnWith(move |parent: &mut ChildSpawner| {
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
            parent.spawn(inline_exploration_notice(notice, window));
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
                    for (index, data) in themes.iter().enumerate() {
                        spawn_theme_button(grid, data, index == 0, window, card_style, index);
                    }
                });
            parent.spawn((
                standard_button(
                    &back_label,
                    theme::button_styles::BACK,
                    theme::scaled(theme::sizes::BUTTON_WIDTH),
                    window,
                ),
                ExploreBackAction(back_destination),
            ));
        })),
    ));

    if card_style.is_some() {
        root.insert(BackgroundColor(Color::NONE));
    }
}

fn inline_exploration_notice(notice: Option<String>, window: Entity) -> impl Bundle + use<> {
    let display = if notice.is_some() {
        Display::Flex
    } else {
        Display::None
    };
    (
        Text::new(notice.unwrap_or_default()),
        theme::typography::text(theme::fonts::SMALL, window),
        TextColor(theme::colors::ERROR),
        Node {
            display,
            ..default()
        },
        ExplorationNoticeText,
    )
}

fn update_exploration_notice(
    exploration_view: Res<State<ExplorationView>>,
    overview: Res<ThemeOverviewPresentation>,
    lessons: Res<ThemeLessonsPresentation>,
    mut notices: Query<(&mut Text, &mut Node), With<ExplorationNoticeText>>,
) {
    let notice = match *exploration_view.get() {
        ExplorationView::Themes => overview.notice.as_deref(),
        ExplorationView::ThemeLessons => lessons.notice.as_deref(),
        ExplorationView::ExerciseSelection => None,
    };
    let display = if notice.is_some() {
        Display::Flex
    } else {
        Display::None
    };
    for (mut text, mut node) in &mut notices {
        let next_text = notice.unwrap_or_default();
        if **text != next_text {
            next_text.clone_into(&mut **text);
        }
        if node.display != display {
            node.display = display;
        }
    }
}

/// Resolves the card background and foreground colors for its availability state.
fn card_colors(card_style: Option<ExplorationCardStyle>, available: bool) -> (Color, Color) {
    card_style.map_or(
        if available {
            (theme::colors::ACTION_PLAY, theme::colors::TEXT_DARK)
        } else {
            (theme::colors::TOGGLE_INACTIVE, theme::colors::TEXT_DARK)
        },
        |style| {
            if available {
                (style.colors.bg_available, style.colors.text_available)
            } else {
                (style.colors.bg_unavailable, theme::colors::TEXT_DARK)
            }
        },
    )
}

fn exploration_button_palette(
    background: Color,
    foreground: Color,
    card_style: Option<ExplorationCardStyle>,
    available: bool,
) -> theme::ButtonPalette {
    let background_states = if available {
        theme::ButtonColorStates::new(background, background.darker(0.08), background.darker(0.14))
    } else {
        theme::ButtonColorStates::flat(background)
    };
    let Some(style) = card_style else {
        return if available {
            theme::ButtonPalette::new(background_states, foreground)
        } else {
            theme::button_styles::UNAVAILABLE
        };
    };
    let border = if available {
        style.colors.border_available
    } else {
        style.colors.border_unavailable
    };
    let border_states = if available {
        theme::ButtonColorStates::new(border, border.darker(0.08), border.darker(0.14))
    } else {
        theme::ButtonColorStates::flat(border)
    };
    theme::ButtonPalette::with_border(background_states, foreground, border_states)
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

/// Inserts a framed card, its shadow, and its floating animation.
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
            border: px(1.0).all(),
            overflow: Overflow::visible(),
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
    let (bg, text_color) = card_colors(card_style, data.available);
    let palette = exploration_button_palette(bg, text_color, card_style, data.available);
    let dims = card_dimensions(card_style);

    let mut button = parent.spawn((
        button_base(palette),
        card_node_layout(&dims),
        ThemeButton(data.id.clone()),
    ));
    button.remove::<AnimatedButton>();

    if let Some(style) = card_style {
        insert_card_overlay(&mut button, style, data.available, index, &dims);
        button.insert(Node {
            border: px(1.0).all(),
            flex_direction: FlexDirection::Column,
            row_gap: theme::scaled(2.0),
            padding: UiRect::axes(theme::scaled(12.0), theme::scaled(3.0)),
            justify_content: JustifyContent::Center,
            ..card_node_layout(&dims)
        });
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

    button.with_children(|contents| {
        let mut title = contents.spawn(card_text(
            &data.title_text,
            theme::fonts::BUTTON,
            text_color,
            window,
        ));
        if let Some(style) = card_style
            && data.available
        {
            title.insert(TextShadow {
                color: style.colors.title_shadow,
                offset: Vec2::new(0.0, 1.0),
            });
        }
        if data.available && data.track_progress {
            let progress_text = data.progress_text.clone().unwrap_or_default();
            let visible = !progress_text.is_empty();
            contents.spawn((
                card_text(
                    &progress_text,
                    theme::fonts::SMALL,
                    theme::colors::TEXT_DARK,
                    window,
                ),
                ExplorationProgressText::Theme(data.id.clone()),
                Node {
                    display: if visible {
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

fn handle_themes(
    mut activations: MessageReader<ButtonActivated>,
    theme_query: Query<&ThemeButton>,
    back_query: Query<&ExploreBackAction>,
    content: Res<ContentLibrary>,
    mut commands: Commands,
    mut next_app_state: ResMut<NextState<AppState>>,
    mut next_exploration_view: ResMut<NextState<ExplorationView>>,
) {
    for activation in activations.read() {
        if let Ok(theme_btn) = theme_query.get(activation.0) {
            if content
                .theme(&theme_btn.0)
                .is_some_and(|theme| theme.available)
            {
                commands.insert_resource(ActiveTheme(theme_btn.0.clone()));
                next_exploration_view.set(ExplorationView::ThemeLessons);
            }
        } else if let Ok(back) = back_query.get(activation.0) {
            next_app_state.set(back.0);
        }
    }
}

#[allow(clippy::too_many_arguments)]
pub fn setup_theme_lessons(
    mut commands: Commands,
    presentation: Res<ThemeLessonsPresentation>,
    settings: Res<GameSettings>,
    primary_window: Single<Entity, With<PrimaryWindow>>,
    existing_roots: Query<Entity, With<ExplorationScreenRoot>>,
) {
    if !presentation.available {
        return;
    }
    for root in &existing_roots {
        commands.entity(root).despawn();
    }

    let window = *primary_window;
    let title = presentation.title.clone();
    let back_label = presentation.back_label.clone();
    let notice = presentation.notice.clone();
    let lessons = presentation.lessons.clone();
    let clear_launch_context = presentation.clear_launch_context;
    let card_style = ExplorationCardStyle::for_exploration_theme(settings.exploration_theme);
    let mut root = commands.spawn((
        screen_root(),
        ExplorationScreenRoot,
        DespawnOnExit(AppState::ThemeExploration),
        DespawnOnEnter(ExplorationView::ExerciseSelection),
        Children::spawn(SpawnWith(move |parent: &mut ChildSpawner| {
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
            parent.spawn(inline_exploration_notice(notice, window));
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
                    for (index, data) in lessons.iter().enumerate() {
                        spawn_lesson_button(grid, data, index == 0, window, card_style, index);
                    }
                });
            parent.spawn((
                standard_button(
                    &back_label,
                    theme::button_styles::BACK,
                    theme::scaled(theme::sizes::BUTTON_WIDTH),
                    window,
                ),
                BackToThemesButton {
                    clear_launch_context,
                },
            ));
        })),
    ));
    if card_style.is_some() {
        root.insert(BackgroundColor(Color::NONE));
    }
}

fn spawn_lesson_button(
    parent: &mut ChildSpawner,
    data: &LessonButtonData,
    auto_focus: bool,
    window: Entity,
    card_style: Option<ExplorationCardStyle>,
    index: usize,
) {
    let (background, text_color) = card_colors(card_style, true);
    let dims = card_dimensions(card_style);
    if data.actions.len() == 1 {
        let action = &data.actions[0];
        let mut card = parent.spawn((
            button_base(exploration_button_palette(
                background, text_color, card_style, true,
            )),
            card_node_layout(&dims),
            LessonActionButton {
                lesson_id: data.id.clone(),
                action: action.action,
                selection: action.selection.clone(),
            },
        ));
        card.remove::<AnimatedButton>();
        if let Some(style) = card_style {
            insert_card_overlay(&mut card, style, true, index, &dims);
        }
        if auto_focus {
            card.insert(AutoFocus);
        }
        card.with_children(|contents| {
            spawn_lesson_heading(contents, data, text_color, window, card_style);
        });
        return;
    }

    let mut card = parent.spawn((
        Node {
            border: px(1.0).all(),
            ..card_node_layout(&dims)
        },
        BackgroundColor(background),
        BorderColor::all(theme::colors::INPUT_BORDER),
    ));
    if let Some(style) = card_style {
        insert_card_overlay(&mut card, style, true, index, &dims);
    }
    card.with_children(|contents| {
        spawn_lesson_heading(contents, data, theme::colors::TEXT_DARK, window, card_style);
        contents
            .spawn((
                Node {
                    width: percent(100.0),
                    flex_direction: FlexDirection::Column,
                    row_gap: theme::scaled(theme::spacing::SMALL),
                    padding: if card_style.is_some() {
                        UiRect::top(theme::scaled(theme::spacing::SMALL))
                    } else {
                        UiRect::default()
                    },
                    border: if card_style.is_some() {
                        UiRect::top(px(1.0))
                    } else {
                        UiRect::default()
                    },
                    ..default()
                },
                BorderColor::all(Color::srgba(0.30, 0.55, 0.84, 0.54)),
            ))
            .with_children(|actions| {
                for (action_index, action) in data.actions.iter().enumerate() {
                    spawn_lesson_action(
                        actions,
                        data,
                        action,
                        auto_focus && action_index == 0,
                        window,
                    );
                }
            });
    });
}

fn spawn_lesson_heading(
    contents: &mut ChildSpawner,
    data: &LessonButtonData,
    text_color: Color,
    window: Entity,
    card_style: Option<ExplorationCardStyle>,
) {
    let mut title = contents.spawn(card_text(
        &data.title_text,
        theme::fonts::BUTTON,
        text_color,
        window,
    ));
    if let Some(style) = card_style {
        title.insert(TextShadow {
            color: style.colors.title_shadow,
            offset: Vec2::new(0.0, 1.0),
        });
    }
    if data.track_progress {
        let progress = data.best_percent_text.clone().unwrap_or_default();
        let visible = !progress.is_empty();
        contents.spawn((
            card_text(
                &progress,
                theme::fonts::SMALL,
                theme::colors::TEXT_DARK,
                window,
            ),
            ExplorationProgressText::Lesson(data.id.clone()),
            Node {
                display: if visible {
                    Display::Flex
                } else {
                    Display::None
                },
                ..default()
            },
        ));
    }
}

fn spawn_lesson_action(
    parent: &mut ChildSpawner,
    data: &LessonButtonData,
    action: &LessonActionPresentation,
    auto_focus: bool,
    window: Entity,
) {
    let palette = match action.action {
        LessonLaunchAction::CompleteSeries => theme::button_styles::ACTION_PRIMARY,
        LessonLaunchAction::ChooseExercise => theme::button_styles::ACTION_SECONDARY,
    };
    let mut button = parent.spawn((
        button_base(palette),
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
            action: action.action,
            selection: action.selection.clone(),
        },
        children![(
            Text::new(action.label.clone()),
            theme::typography::text(theme::fonts::SMALL, window),
            TextColor(palette.foreground),
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
    back_query: Query<&BackToThemesButton>,
    content: Res<ContentLibrary>,
    active_theme: Res<ActiveTheme>,
    mut commands: Commands,
    mut next_app_state: ResMut<NextState<AppState>>,
    mut next_exploration_view: ResMut<NextState<ExplorationView>>,
) {
    for activation in activations.read() {
        if let Ok(lesson_button) = lesson_query.get(activation.0) {
            if let Some(theme_data) = content.theme(&active_theme)
                && theme_data.lesson(&lesson_button.lesson_id).is_some()
            {
                commands.insert_resource(SelectedLesson(Some(lesson_button.lesson_id.clone())));
                match lesson_button.action {
                    LessonLaunchAction::CompleteSeries => {
                        let selection = lesson_button
                            .selection
                            .clone()
                            .unwrap_or(LessonLaunchSelection::CompleteLesson);
                        commands.insert_resource(selection);
                        next_app_state.set(AppState::LessonPlay);
                    }
                    LessonLaunchAction::ChooseExercise => {
                        if let Some(selection) = lesson_button.selection.clone() {
                            commands.insert_resource(selection);
                        } else {
                            commands.remove_resource::<LessonLaunchSelection>();
                        }
                        next_exploration_view.set(ExplorationView::ExerciseSelection);
                    }
                }
            }
        } else if let Ok(back) = back_query.get(activation.0) {
            commands.remove_resource::<ActiveTheme>();
            if back.clear_launch_context {
                commands.remove_resource::<LessonLaunchSelection>();
                commands.remove_resource::<SelectedLesson>();
            }
            next_exploration_view.set(ExplorationView::Themes);
        }
    }
}

#[cfg(test)]
mod tests {
    use super::{ExplorationView, LessonLaunchSelection, restored_exploration_view};

    #[test]
    fn single_exercise_returns_to_the_exercise_catalogue() {
        let selection = LessonLaunchSelection::SingleExercise {
            exercise_id: "exercise-1".to_owned(),
        };

        assert_eq!(
            restored_exploration_view(true, Some(&selection)),
            Some(ExplorationView::ExerciseSelection),
        );
    }
}

#[cfg(not(feature = "demo"))]
pub mod production;
