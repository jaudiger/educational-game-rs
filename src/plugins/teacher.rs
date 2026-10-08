use bevy::camera::RenderTarget;
use bevy::ecs::system::SystemParam;
use bevy::prelude::*;
use bevy::window::{
    EnabledButtons, Monitor, PrimaryMonitor, PrimaryWindow, WindowPosition, WindowRef,
    WindowResolution,
};
use bevy_persistent::prelude::*;

use crate::data::content::QuestionType;
use crate::data::{GameMode, GameSettings, PlayerContext};
use crate::i18n::{I18n, TranslationKey};
use crate::states::{AppState, InLessonFlow, LESSON_FLOW_STATES};
use crate::ui::components::toggle_button;
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
pub struct TeacherTabButton(pub TeacherTab);

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
            OnEnter(AppState::MapExploration),
            spawn_teacher_window_if_class_mode.in_set(TeacherWindowInit),
        )
        .add_systems(
            Update,
            (handle_tab_click, cleanup_teacher_view_overlays)
                .run_if(in_state(InLessonFlow))
                .run_if(teacher_window_exists),
        );

        for &state in &LESSON_FLOW_STATES {
            if state == AppState::MapExploration {
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
    settings: Res<Persistent<GameSettings>>,
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

/// Returns a tab header bundle (horizontal row with two tab buttons).
/// Active tab gets `COLOR_PRIMARY` bg; inactive gets `COLOR_TOGGLE_INACTIVE`.
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
        children![
            (
                tab_button(&students_label, active_tab == TeacherTab::Students, window),
                TeacherTabButton(TeacherTab::Students),
            ),
            (
                tab_button(&lessons_label, active_tab == TeacherTab::Lessons, window),
                TeacherTabButton(TeacherTab::Lessons),
            ),
        ],
    )
}

fn tab_button(label: &str, active: bool, window: Entity) -> impl Bundle + use<> {
    toggle_button(label, active, window)
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

fn handle_tab_click(
    query: Query<(&Interaction, &TeacherTabButton), Changed<Interaction>>,
    mut states: Query<&mut TeacherWindowState, With<TeacherWindow>>,
) {
    let Ok(mut state) = states.single_mut() else {
        return;
    };

    for (interaction, tab_btn) in &query {
        if *interaction != Interaction::Pressed {
            continue;
        }
        if tab_btn.0 == state.view.tab() && !state.view.is_detail() {
            continue;
        }
        state.view = TeacherView::base(tab_btn.0);
    }
}
