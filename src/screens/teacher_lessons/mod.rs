mod config;
mod tree;

use bevy::ecs::system::SystemParam;
use bevy::input_focus::tab_navigation::TabGroup;
use bevy::prelude::*;

use crate::data::{ContentLibrary, GameMode, PersistenceAction, PersistenceStatus};
use crate::i18n::I18n;
use crate::plugins::teacher::{
    TeacherScreenParam, TeacherView, tab_header, teacher_lessons_tree_view_active,
    teacher_window_exists,
};
use crate::states::{AppState, InLessonFlow, LESSON_FLOW_STATES, cleanup_root};
use crate::ui::components::persistence_notice;
use crate::ui::theme;

/// Teacher lessons tab for configuring per-lesson exercise selection.
pub struct TeacherLessonsScreenPlugin;

impl Plugin for TeacherLessonsScreenPlugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<tree::ThemeDisclosureState>()
            .init_resource::<tree::LessonDisclosureState>()
            .init_resource::<config::ExerciseGroupDisclosureState>()
            .add_systems(
                Update,
                (
                    tree::handle_theme_disclosure,
                    tree::handle_lesson_disclosure,
                )
                    .run_if(in_state(InLessonFlow))
                    .run_if(teacher_lessons_tree_view_active),
            );

        for &state in &LESSON_FLOW_STATES {
            app.add_systems(OnExit(state), cleanup_root::<TeacherLessonsRoot>);
        }

        app.add_systems(
            Update,
            rebuild_lessons_ui
                .run_if(in_state(InLessonFlow))
                .run_if(teacher_window_exists),
        )
        .add_systems(
            Update,
            (
                config::handle_exercise_group_disclosure,
                config::handle_count_change,
                config::handle_visual_toggle,
                config::handle_reset_click,
                config::update_exercise_labels,
            )
                .run_if(in_state(AppState::ThemeExploration))
                .run_if(teacher_lessons_tree_view_active),
        );
    }
}

#[derive(Component, Reflect)]
pub struct TeacherLessonsRoot;

#[derive(SystemParam)]
struct ExistingTeacherLessonsUi<'w, 's> {
    roots: Query<'w, 's, Entity, With<TeacherLessonsRoot>>,
    scroll: Query<'w, 's, &'static ScrollPosition, With<tree::LessonTreeScroll>>,
}

#[derive(SystemParam)]
struct TeacherLessonsResources<'w> {
    status: Res<'w, PersistenceStatus>,
    themes: ResMut<'w, tree::ThemeDisclosureState>,
    lessons: ResMut<'w, tree::LessonDisclosureState>,
    exercise_groups: ResMut<'w, config::ExerciseGroupDisclosureState>,
    content: Res<'w, ContentLibrary>,
    app_state: Res<'w, State<AppState>>,
}

struct LessonTreeRootData {
    camera_entity: Entity,
    window: Entity,
    language: crate::data::Language,
    status: PersistenceStatus,
    active_tab: crate::plugins::teacher::TeacherTab,
    show_config_controls: bool,
    scroll_position: ScrollPosition,
    tree_specs: Vec<tree::ThemeTreeSpec>,
}

fn rebuild_lessons_ui(
    mut commands: Commands,
    ts: TeacherScreenParam<'_, '_>,
    existing: ExistingTeacherLessonsUi<'_, '_>,
    mut resources: TeacherLessonsResources<'_>,
) {
    let Ok(state) = ts.teacher.state.single() else {
        return;
    };
    if !state.is_changed() && !ts.ctx.save_data.is_changed() && !resources.status.is_changed() {
        return;
    }
    if state.is_changed() {
        resources.themes.reset();
        resources.lessons.reset();
        resources.exercise_groups.reset();
    }

    let scroll_position = existing
        .scroll
        .single()
        .map_or_else(|_| ScrollPosition::default(), Clone::clone);
    for entity in &existing.roots {
        commands.entity(entity).despawn();
    }
    if ts.ctx.settings.mode != GameMode::Group {
        return;
    }
    let Ok(camera_entity) = ts.teacher.camera.single() else {
        return;
    };
    let Ok(window) = ts.teacher.window.single() else {
        return;
    };
    let TeacherView::Lessons = &state.view else {
        return;
    };
    let tree_specs = tree::build_tree_specs(
        &resources.content.themes,
        &ts.ctx.save_data,
        ts.ctx.session.as_deref(),
        ts.i18n.language,
        &resources.themes,
        &resources.lessons,
        &resources.exercise_groups,
    );
    spawn_lesson_tree_root(
        &mut commands,
        LessonTreeRootData {
            camera_entity,
            window,
            language: ts.i18n.language,
            status: *resources.status,
            active_tab: state.view.tab(),
            show_config_controls: *resources.app_state.get() == AppState::ThemeExploration,
            scroll_position,
            tree_specs,
        },
    );
}

fn spawn_lesson_tree_root(commands: &mut Commands, data: LessonTreeRootData) {
    let i18n = I18n::new(data.language);
    let header = tab_header(&i18n, data.active_tab, data.window);
    let write_notice = persistence_notice(
        PersistenceAction::TeacherLessonConfig,
        data.status,
        &i18n,
        data.window,
    );
    commands.spawn((
        Node {
            width: percent(100.0),
            height: percent(100.0),
            flex_direction: FlexDirection::Column,
            padding: theme::scaled(theme::spacing::LARGE).all(),
            row_gap: theme::scaled(theme::spacing::MEDIUM),
            ..default()
        },
        BackgroundColor(theme::colors::BACKGROUND),
        UiTargetCamera(data.camera_entity),
        TabGroup::new(0),
        TeacherLessonsRoot,
        Children::spawn(SpawnWith(move |parent: &mut ChildSpawner| {
            parent.spawn(header);
            parent.spawn(write_notice);
            tree::spawn_tree_view(
                parent,
                data.tree_specs,
                &i18n,
                data.show_config_controls,
                data.scroll_position,
                data.window,
            );
        })),
    ));
}
