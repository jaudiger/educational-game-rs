use bevy::camera::RenderTarget;
use bevy::ecs::system::SystemParam;
use bevy::input_focus::InputFocus;
use bevy::picking::events::PointerClick;
use bevy::picking::hover::Hovered;
use bevy::picking::pointer::PointerButton;
use bevy::prelude::*;
use bevy::ui::Selected;
use bevy::ui::auto_directional_navigation::AutoDirectionalNavigation;
use bevy::ui_widgets::{
    ControlOrientation, SelectedTab, Tab, TabActivation, TabList, ValueChange, observe,
    tablist_self_update,
};
use bevy::window::{
    EnabledButtons, Monitor, PrimaryMonitor, PrimaryWindow, WindowPosition, WindowRef,
    WindowResolution,
};

use crate::data::content::QuestionType;
use crate::data::{GameMode, GameSettings, PlayerContext};
use crate::i18n::{I18n, TranslationKey};
use crate::states::{AppState, InLessonFlow, LESSON_FLOW_STATES};
use crate::ui::animation::AnimatedButton;
use crate::ui::theme;

/// Spawns and manages the secondary teacher window in class mode.
pub struct TeacherPlugin;

#[derive(Component, Reflect)]
pub struct TeacherWindow;

#[derive(Component, Reflect)]
pub struct TeacherCamera;

#[derive(Component, Reflect)]
pub struct TeacherViewOverlay;

#[derive(Component, Reflect)]
pub struct TeacherWindowState {
    #[reflect(ignore)]
    pub view: TeacherView,
}

impl Default for TeacherWindowState {
    fn default() -> Self {
        Self {
            view: TeacherView::Students,
        }
    }
}

#[derive(Clone, Debug, Default)]
pub enum TeacherView {
    #[default]
    Students,
    StudentStats {
        student_index: usize,
    },
    Lessons,
    LessonConfig {
        lesson_id: String,
        lesson_title: String,
        questions: Vec<TeacherQuestionDraft>,
    },
}

impl TeacherView {
    pub const fn tab(&self) -> TeacherTab {
        match self {
            Self::Students | Self::StudentStats { .. } => TeacherTab::Students,
            Self::Lessons | Self::LessonConfig { .. } => TeacherTab::Lessons,
        }
    }

    pub const fn is_detail(&self) -> bool {
        matches!(self, Self::StudentStats { .. } | Self::LessonConfig { .. })
    }

    pub const fn base(tab: TeacherTab) -> Self {
        match tab {
            TeacherTab::Students => Self::Students,
            TeacherTab::Lessons => Self::Lessons,
        }
    }
}

#[derive(Clone, Debug)]
pub struct TeacherQuestionDraft {
    pub index: usize,
    pub question_type: QuestionType,
    pub full_prompt: String,
    pub count: usize,
    pub has_visual: bool,
    pub show_visual: bool,
    pub default_show_visual: bool,
}

#[derive(Component, Reflect)]
pub struct TeacherTabItem(pub TeacherTab);

#[derive(Component, Reflect)]
struct TeacherTabLabel;

#[derive(Clone, Copy, Debug, Default, Eq, PartialEq, Reflect)]
pub enum TeacherTab {
    #[default]
    Students,
    Lessons,
}

#[derive(SystemParam)]
pub struct TeacherWindowParam<'w, 's> {
    pub camera: Query<'w, 's, Entity, With<TeacherCamera>>,
    pub window: Query<'w, 's, Entity, With<TeacherWindow>>,
    pub state: Query<'w, 's, Ref<'static, TeacherWindowState>, With<TeacherWindow>>,
}

#[derive(SystemParam)]
pub struct TeacherScreenParam<'w, 's> {
    pub ctx: PlayerContext<'w>,
    pub teacher: TeacherWindowParam<'w, 's>,
    pub i18n: Res<'w, I18n>,
}

impl Plugin for TeacherPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(
            OnEnter(AppState::ThemeExploration),
            spawn_teacher_window_if_class_mode.in_set(TeacherWindowInit),
        )
        .add_systems(
            Update,
            (
                sync_teacher_tab_selection,
                reset_tab_on_keyboard_reselection,
                update_teacher_tab_styles,
                cleanup_teacher_view_overlays,
            )
                .run_if(in_state(InLessonFlow))
                .run_if(teacher_window_exists),
        );

        for &state in &LESSON_FLOW_STATES {
            if state == AppState::ThemeExploration {
                app.add_systems(OnEnter(state), reset_teacher_view.after(TeacherWindowInit));
            } else {
                app.add_systems(OnEnter(state), reset_teacher_view);
            }
        }
    }
}

#[derive(SystemSet, Clone, Debug, Eq, Hash, PartialEq)]
pub struct TeacherWindowInit;

pub fn teacher_window_exists(query: Query<(), With<TeacherWindow>>) -> bool {
    query.single().is_ok()
}

pub fn teacher_roster_view_active(query: Query<&TeacherWindowState, With<TeacherWindow>>) -> bool {
    query
        .single()
        .is_ok_and(|state| matches!(state.view, TeacherView::Students))
}

pub fn teacher_lessons_tree_view_active(
    query: Query<&TeacherWindowState, With<TeacherWindow>>,
) -> bool {
    query
        .single()
        .is_ok_and(|state| matches!(state.view, TeacherView::Lessons))
}

pub fn teacher_lesson_config_view_active(
    query: Query<&TeacherWindowState, With<TeacherWindow>>,
) -> bool {
    query
        .single()
        .is_ok_and(|state| matches!(state.view, TeacherView::LessonConfig { .. }))
}

pub fn teacher_stats_view_active(query: Query<&TeacherWindowState, With<TeacherWindow>>) -> bool {
    query
        .single()
        .is_ok_and(|state| matches!(state.view, TeacherView::StudentStats { .. }))
}

fn spawn_teacher_window_if_class_mode(
    mut commands: Commands,
    settings: Res<GameSettings>,
    i18n: Res<I18n>,
    existing: Query<(), With<TeacherWindow>>,
    primary_window: Query<&Window, With<PrimaryWindow>>,
    primary_monitor: Query<&Monitor, With<PrimaryMonitor>>,
) {
    if settings.mode != GameMode::Group || !existing.is_empty() {
        return;
    }

    let teacher_logical_width: u32 = 500;
    let position = compute_left_of_primary(
        primary_window.single().ok(),
        primary_monitor.single().ok(),
        teacher_logical_width,
    );

    let window = commands
        .spawn((
            Window {
                title: i18n.t(&TranslationKey::TeacherDashboard).into_owned(),
                resolution: WindowResolution::new(500, 700),
                position,
                resize_constraints: WindowResizeConstraints {
                    min_width: 500.0,
                    min_height: 700.0,
                    max_width: 800.0,
                    max_height: 1000.0,
                },
                enabled_buttons: EnabledButtons {
                    close: false,
                    ..default()
                },
                ..default()
            },
            TeacherWindow,
            TeacherWindowState::default(),
            DespawnOnExit(InLessonFlow),
        ))
        .id();

    commands.spawn((
        Camera2d,
        RenderTarget::Window(WindowRef::Entity(window)),
        TeacherCamera,
        DespawnOnExit(InLessonFlow),
    ));
}

fn reset_teacher_view(mut states: Query<&mut TeacherWindowState, With<TeacherWindow>>) {
    if let Ok(mut state) = states.single_mut() {
        state.view = TeacherView::base(state.view.tab());
    }
}

/// Compute a [`WindowPosition`] that places a window of the given logical
/// width directly to the left of the primary window.
///
/// The primary window's position is read from `Window.position` when
/// available (i.e. after the OS has sent a `WindowMoved` event). Otherwise
/// we estimate it by assuming it is centred on the primary monitor.
fn compute_left_of_primary(
    primary: Option<&Window>,
    monitor: Option<&Monitor>,
    teacher_logical_width: u32,
) -> WindowPosition {
    let (Some(primary), Some(monitor)) = (primary, monitor) else {
        return WindowPosition::default();
    };

    let scale = monitor.scale_factor;
    #[allow(clippy::cast_possible_truncation)]
    let teacher_phys_w = (f64::from(teacher_logical_width) * scale) as i32;

    // If the OS already told us the actual position, use it directly.
    if let WindowPosition::At(pos) = primary.position {
        let x = (pos.x - teacher_phys_w).max(monitor.physical_position.x);
        return WindowPosition::At(IVec2::new(x, pos.y));
    }

    // Otherwise estimate the centred position from monitor dimensions.
    let primary_phys_w = primary.resolution.physical_width().cast_signed();
    let primary_phys_h = primary.resolution.physical_height().cast_signed();
    let monitor_w = monitor.physical_width.cast_signed();
    let monitor_h = monitor.physical_height.cast_signed();

    let primary_x = monitor.physical_position.x + (monitor_w - primary_phys_w) / 2;
    let primary_y = monitor.physical_position.y + (monitor_h - primary_phys_h) / 2;

    let x = (primary_x - teacher_phys_w).max(monitor.physical_position.x);
    WindowPosition::At(IVec2::new(x, primary_y))
}

/// Returns a tab list bundle with the current app styling.
pub fn tab_header(i18n: &I18n, active_tab: TeacherTab, window: Entity) -> impl Bundle {
    let students_label = i18n.t(&TranslationKey::TabStudents).into_owned();
    let lessons_label = i18n.t(&TranslationKey::TabLessons).into_owned();
    (
        Node {
            flex_direction: FlexDirection::Row,
            column_gap: theme::scaled(theme::spacing::SMALL),
            margin: theme::scaled(theme::spacing::MEDIUM).bottom(),
            ..default()
        },
        TabList {
            orientation: ControlOrientation::Horizontal,
            activation: TabActivation::Manual,
        },
        observe(tablist_self_update),
        observe(handle_tab_change),
        Children::spawn(SpawnWith(move |parent: &mut ChildSpawner| {
            parent.spawn((
                tab_button(&students_label, active_tab == TeacherTab::Students, window),
                TeacherTabItem(TeacherTab::Students),
                observe(reset_tab_on_pointer_reselection),
            ));
            parent.spawn((
                tab_button(&lessons_label, active_tab == TeacherTab::Lessons, window),
                TeacherTabItem(TeacherTab::Lessons),
                observe(reset_tab_on_pointer_reselection),
            ));
        })),
    )
}

fn tab_button(label: &str, active: bool, window: Entity) -> impl Bundle + use<> {
    let background = if active {
        theme::colors::PRIMARY
    } else {
        theme::colors::TOGGLE_INACTIVE
    };
    let text_color = if active {
        theme::colors::TEXT_LIGHT
    } else {
        theme::colors::TEXT_DARK
    };
    (
        Tab,
        Hovered::default(),
        AnimatedButton,
        AutoDirectionalNavigation::default(),
        Outline::new(
            Val::Px(theme::sizes::FOCUS_RING_WIDTH),
            Val::Px(theme::sizes::FOCUS_RING_OFFSET),
            Color::NONE,
        ),
        BackgroundColor(background),
        Node {
            min_width: theme::scaled(160.0),
            height: theme::scaled(theme::sizes::BUTTON_HEIGHT),
            padding: UiRect::axes(
                theme::scaled(theme::sizes::BUTTON_PADDING),
                theme::scaled(0.0),
            ),
            justify_content: JustifyContent::Center,
            align_items: AlignItems::Center,
            overflow: Overflow::clip(),
            border_radius: BorderRadius::all(theme::scaled(theme::sizes::BUTTON_BORDER_RADIUS)),
            ..default()
        },
        children![(
            Text::new(label),
            theme::typography::text(theme::fonts::BUTTON_SMALL, window),
            TextColor(text_color),
            TextLayout::justify(Justify::Center),
            TeacherTabLabel,
        )],
    )
}

fn handle_tab_change(
    change: On<ValueChange<Option<Entity>>>,
    tabs: Query<&TeacherTabItem>,
    mut states: Query<&mut TeacherWindowState, With<TeacherWindow>>,
) {
    let Some(entity) = change.value else {
        return;
    };
    let Ok(tab) = tabs.get(entity) else {
        return;
    };
    let Ok(mut state) = states.single_mut() else {
        return;
    };
    if state.view.tab() != tab.0 || state.view.is_detail() {
        state.view = TeacherView::base(tab.0);
    }
}

fn reset_tab_on_pointer_reselection(
    click: On<PointerClick>,
    tabs: Query<&TeacherTabItem>,
    mut states: Query<&mut TeacherWindowState, With<TeacherWindow>>,
) {
    if click.button != PointerButton::Primary {
        return;
    }
    let Ok(tab) = tabs.get(click.entity) else {
        return;
    };
    let Ok(mut state) = states.single_mut() else {
        return;
    };
    if state.view.is_detail() && state.view.tab() == tab.0 {
        state.view = TeacherView::base(tab.0);
    }
}

fn reset_tab_on_keyboard_reselection(
    keyboard: Res<ButtonInput<KeyCode>>,
    focus: Res<InputFocus>,
    tabs: Query<&TeacherTabItem, With<Tab>>,
    mut states: Query<&mut TeacherWindowState, With<TeacherWindow>>,
) {
    if !keyboard.just_pressed(KeyCode::Enter) && !keyboard.just_pressed(KeyCode::Space) {
        return;
    }
    let Some(focused) = focus.get() else {
        return;
    };
    let Ok(tab) = tabs.get(focused) else {
        return;
    };
    let Ok(mut state) = states.single_mut() else {
        return;
    };
    if state.view.is_detail() && state.view.tab() == tab.0 {
        state.view = TeacherView::base(tab.0);
    }
}

fn sync_teacher_tab_selection(
    states: Query<&TeacherWindowState, With<TeacherWindow>>,
    mut lists: Query<(&Children, &mut SelectedTab), With<TabList>>,
    tabs: Query<(Entity, &TeacherTabItem), With<Tab>>,
) {
    let Ok(state) = states.single() else {
        return;
    };
    let active_tab = state.view.tab();
    for (children, mut selection) in &mut lists {
        let selected = children.iter().find_map(|child| {
            tabs.get(child)
                .ok()
                .filter(|(_, tab)| tab.0 == active_tab)
                .map(|(entity, _)| entity)
        });
        if let Some(selected) = selected
            && selection.0 != Some(selected)
        {
            selection.0 = Some(selected);
        }
    }
}

fn update_teacher_tab_styles(
    tabs: Query<(Has<Selected>, &mut BackgroundColor, &Children), With<TeacherTabItem>>,
    mut labels: Query<&mut TextColor, With<TeacherTabLabel>>,
) {
    for (selected, mut background, children) in tabs {
        let background_color = if selected {
            theme::colors::PRIMARY
        } else {
            theme::colors::TOGGLE_INACTIVE
        };
        if background.0 != background_color {
            background.0 = background_color;
        }
        let text_color = if selected {
            theme::colors::TEXT_LIGHT
        } else {
            theme::colors::TEXT_DARK
        };
        for child in children.iter() {
            if let Ok(mut color) = labels.get_mut(child)
                && color.0 != text_color
            {
                color.0 = text_color;
            }
        }
    }
}

fn cleanup_teacher_view_overlays(
    mut commands: Commands,
    states: Query<Ref<TeacherWindowState>, With<TeacherWindow>>,
    overlays: Query<Entity, With<TeacherViewOverlay>>,
) {
    if !states.single().is_ok_and(|state| state.is_changed()) {
        return;
    }
    for entity in &overlays {
        commands.entity(entity).despawn();
    }
}
