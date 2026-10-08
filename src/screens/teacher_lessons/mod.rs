mod config;
mod tree;

use bevy::input_focus::tab_navigation::TabGroup;
use bevy::prelude::*;

use crate::data::{ContentLibrary, GameMode};
use crate::i18n::I18n;
use crate::plugins::teacher::{
    TeacherQuestionDraft, TeacherScreenParam, TeacherView, tab_header,
    teacher_lesson_config_view_active, teacher_lessons_tree_view_active, teacher_window_exists,
};
use crate::states::{AppState, InLessonFlow, LESSON_FLOW_STATES, cleanup_root};
use crate::ui::theme;

/// Teacher lessons tab for configuring per-lesson question selection.
pub struct TeacherLessonsScreenPlugin;

impl Plugin for TeacherLessonsScreenPlugin {
    fn build(&self, app: &mut App) {
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
            config::handle_config_button_click
                .run_if(in_state(AppState::ThemeExploration))
                .run_if(teacher_lessons_tree_view_active),
        )
        .add_systems(
            Update,
            (
                config::handle_count_change,
                config::handle_visual_toggle,
                config::handle_reset_config,
                config::handle_save_config,
                config::handle_return_to_tree,
                config::update_scroll_indicator,
                config::update_question_labels,
                config::update_config_hover_text,
            )
                .run_if(in_state(AppState::ThemeExploration))
                .run_if(teacher_lesson_config_view_active),
        );
    }
}

#[derive(Component, Reflect)]
pub struct TeacherLessonsRoot;

#[derive(Component, Reflect)]
struct ConfigLessonButton {
    theme_id: String,
    lesson_id: String,
}

#[derive(Component, Reflect)]
struct CountButton {
    index: usize,
    count_text: Entity,
    delta: isize,
}

#[derive(Component, Reflect)]
struct SaveConfigButton;

#[derive(Component, Reflect)]
struct ReturnToTreeButton;

#[derive(Component, Reflect)]
struct ResetConfigButton;

#[derive(Component, Reflect)]
struct CountText;

#[derive(Component, Reflect)]
struct VisualToggleButton(usize);

#[derive(Component, Reflect)]
struct QuestionRow(String);

#[derive(Component, Reflect)]
struct ScrollFrame;

#[derive(Component, Reflect)]
struct ScrollContent;

#[derive(Component, Reflect)]
struct ScrollIndicator;

#[derive(Component, Reflect)]
struct QuestionLabel(String);

#[derive(Component, Reflect)]
struct ConfigHoverText;

fn rebuild_lessons_ui(
    mut commands: Commands,
    ts: TeacherScreenParam<'_, '_>,
    existing_root: Query<Entity, With<TeacherLessonsRoot>>,
    content: Res<ContentLibrary>,
    app_state: Res<State<AppState>>,
) {
    let Ok(state) = ts.teacher.state.single() else {
        return;
    };
    if !state.is_changed() && !ts.ctx.save_data.is_changed() {
        return;
    }

    for entity in &existing_root {
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
    let active_tab = state.view.tab();

    match &state.view {
        TeacherView::LessonConfig {
            lesson_title,
            questions,
            ..
        } => {
            let questions = questions.clone();
            let title = lesson_title.clone();
            let i18n_owned = I18n::new(ts.i18n.language);
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
                UiTargetCamera(camera_entity),
                TeacherLessonsRoot,
                Children::spawn(SpawnWith(move |parent: &mut ChildSpawner| {
                    config::spawn_config_view(
                        parent,
                        &i18n_owned,
                        &title,
                        &questions,
                        active_tab,
                        window,
                    );
                })),
            ));
        }
        TeacherView::Lessons => {
            let is_theme_exploration = *app_state.get() == AppState::ThemeExploration;
            let header = tab_header(&ts.i18n, active_tab, window);
            let i18n_owned = I18n::new(ts.i18n.language);
            let tree_specs = tree::build_tree_specs(
                &content.themes,
                &ts.ctx.save_data,
                ts.ctx.session.as_deref(),
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
                UiTargetCamera(camera_entity),
                TabGroup::new(0),
                TeacherLessonsRoot,
                Children::spawn(SpawnWith(move |parent: &mut ChildSpawner| {
                    parent.spawn(header);
                    tree::spawn_tree_view(
                        parent,
                        &tree_specs,
                        &i18n_owned,
                        is_theme_exploration,
                        window,
                    );
                })),
            ));
        }
        TeacherView::Students | TeacherView::StudentStats { .. } => {}
    }
}

pub(super) fn has_any_selected(questions: &[TeacherQuestionDraft]) -> bool {
    questions.iter().any(|question| question.count > 0)
}
