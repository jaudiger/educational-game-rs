use bevy::camera::RenderTarget;
use bevy::ecs::system::SystemParam;
use bevy::prelude::*;
use bevy::window::{
    EnabledButtons, Monitor, PrimaryMonitor, PrimaryWindow, WindowPosition, WindowRef,
    WindowResolution,
};
use bevy_persistent::prelude::*;

use crate::data::{GameMode, GameSettings, PlayerContext};
use crate::i18n::{I18n, TranslationKey};
use crate::states::{AppState, InLessonFlow};
use crate::ui::components::toggle_button;
use crate::ui::theme;

/// Spawns and manages the secondary teacher window in class mode.
pub struct TeacherPlugin;

#[derive(Component, Reflect)]
pub struct TeacherWindow;

#[derive(Component, Reflect)]
pub struct TeacherCamera;

/// Marker component added to every teacher tab content root entity.
/// `handle_tab_click` queries this to despawn all tab content on a tab switch
/// without knowing which screen owns the root.
#[derive(Component, Reflect)]
pub struct TeacherContentRoot;

/// Resource inserted by screens when they enter a drill-down (detail) view,
/// and removed when they return to the top-level list. Lets `handle_tab_click`
/// detect whether re-clicking the active tab should navigate back.
#[derive(Resource, Reflect)]
pub struct TeacherInDetailView;

/// Event triggered on every tab switch, after all content roots are despawned.
/// Screens observe this event to clean up their own resources and trigger
/// their rebuild events.
#[derive(Event, Clone, Copy)]
pub struct TeacherTabChanged(pub TeacherTab);

/// System set for the initial teacher window spawn.
/// Other plugins that need the teacher camera to exist should order
/// their `OnEnter(MapExploration)` systems after this set.
#[derive(SystemSet, Clone, Debug, Eq, Hash, PartialEq)]
pub struct TeacherWindowInit;

/// Which tab is currently active in the teacher window.
#[derive(Resource, Clone, Copy, Debug, Default, Eq, PartialEq, Reflect)]
pub enum TeacherTab {
    #[default]
    Students,
    Lessons,
}

/// Marker component on each tab header button, storing which tab it represents.
#[derive(Component, Reflect)]
pub struct TeacherTabButton(pub TeacherTab);

/// Groups the teacher camera and window entities into a single system parameter.
#[derive(SystemParam)]
pub struct TeacherWindowParam<'w, 's> {
    pub camera: Single<'w, 's, Entity, With<TeacherCamera>>,
    pub window: Single<'w, 's, Entity, With<TeacherWindow>>,
}

/// Groups the four params shared by every teacher-tab rebuild observer:
/// player context, teacher window handles, i18n, and the active tab.
#[derive(SystemParam)]
pub struct TeacherScreenParam<'w, 's> {
    pub ctx: PlayerContext<'w>,
    pub teacher: TeacherWindowParam<'w, 's>,
    pub i18n: Res<'w, I18n>,
    pub teacher_tab: Option<Res<'w, TeacherTab>>,
}

impl Plugin for TeacherPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(
            OnEnter(AppState::MapExploration),
            spawn_teacher_window_if_class_mode.in_set(TeacherWindowInit),
        )
        .add_systems(OnEnter(AppState::Home), despawn_teacher_window)
        .add_systems(OnEnter(AppState::SaveSlots), despawn_teacher_window)
        .add_systems(Update, handle_tab_click.run_if(in_state(InLessonFlow)));
    }
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

    commands.insert_resource(TeacherTab::Students);

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
        ))
        .id();

    commands.spawn((
        Camera2d,
        RenderTarget::Window(WindowRef::Entity(window)),
        TeacherCamera,
    ));
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

fn despawn_teacher_window(
    mut commands: Commands,
    window_query: Query<Entity, With<TeacherWindow>>,
    camera_query: Query<Entity, With<TeacherCamera>>,
) {
    for entity in &window_query {
        commands.entity(entity).despawn();
    }
    for entity in &camera_query {
        commands.entity(entity).despawn();
    }
    commands.remove_resource::<TeacherTab>();
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

/// Returns a single tab button bundle.
fn tab_button(label: &str, active: bool, window: Entity) -> impl Bundle + use<> {
    toggle_button(label, active, window)
}

/// Switches tabs when a tab header button is pressed, or navigates back to
/// the list view when re-clicking the active tab from a detail view.
///
/// Despawns all content roots (via `TeacherContentRoot`) and clears
/// `TeacherInDetailView`, then triggers `TeacherTabChanged` so each screen
/// can clean up its own state resources and request a rebuild.
fn handle_tab_click(
    query: Query<(&Interaction, &TeacherTabButton), Changed<Interaction>>,
    current_tab: Option<Res<TeacherTab>>,
    in_detail: Option<Res<TeacherInDetailView>>,
    mut commands: Commands,
    roots: Query<Entity, With<TeacherContentRoot>>,
) {
    let Some(current) = current_tab else { return };

    for (interaction, tab_btn) in &query {
        if *interaction != Interaction::Pressed {
            continue;
        }

        // Re-clicking the active tab only acts when a detail view is open,
        // so the button brings the user back to the top-level list.
        if tab_btn.0 == *current && in_detail.is_none() {
            continue;
        }

        commands.insert_resource(tab_btn.0);
        commands.remove_resource::<TeacherInDetailView>();

        for entity in &roots {
            commands.entity(entity).despawn();
        }

        commands.trigger(TeacherTabChanged(tab_btn.0));
    }
}
