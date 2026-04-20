mod config;
mod tree;

use bevy::input_focus::tab_navigation::TabGroup;
use bevy::prelude::*;
use bevy_persistent::prelude::*;

use crate::data::content::QuestionType;
use crate::data::{ContentLibrary, GameMode, GameSettings};
use crate::i18n::I18n;
use crate::plugins::teacher::{
    TeacherContentRoot, TeacherScreenParam, TeacherTab, TeacherTabChanged, TeacherWindowInit,
    tab_header,
};
use crate::states::{
    AppState, InLessonFlow, LESSON_FLOW_STATES, StateScopedResourceExt, cleanup_root,
};
use crate::ui::theme;

/// Teacher lessons tab for configuring per-lesson question selection.
pub struct TeacherLessonsScreenPlugin;

impl Plugin for TeacherLessonsScreenPlugin {
    fn build(&self, app: &mut App) {
        for &state in &LESSON_FLOW_STATES {
            app.register_state_scoped_resource::<AppState, TeacherLessonsState>(state)
                .register_state_scoped_resource::<AppState, LessonConfigDraftRes>(state)
                .add_systems(OnExit(state), cleanup_root::<TeacherLessonsRoot>);

            if state == AppState::MapExploration {
                app.add_systems(
                    OnEnter(state),
                    initialize_lessons_state.after(TeacherWindowInit),
                );
            } else {
                app.add_systems(OnEnter(state), initialize_lessons_state);
            }
        }

        app.add_observer(on_teacher_tab_changed)
            .add_systems(
                Update,
                rebuild_lessons_ui
                    .run_if(in_state(InLessonFlow))
                    .run_if(resource_exists_and_changed::<TeacherLessonsState>),
            )
            .add_systems(
                Update,
                (
                    config::handle_config_button_click,
                    config::handle_count_change,
                    config::handle_visual_toggle,
                    config::handle_reset_config,
                    config::handle_save_config,
                    config::handle_return_to_tree,
                    config::update_scroll_indicator,
                    config::update_question_labels,
                    config::update_config_hover_text,
                )
                    .run_if(in_state(AppState::MapExploration))
                    .run_if(resource_exists::<TeacherLessonsState>),
            );
    }
}

/// Inserts [`TeacherLessonsState`] on state entry when the teacher window is
/// showing the Lessons tab. Guards mirror the ones in [`rebuild_lessons_ui`]
/// so we do not insert state that would immediately be skipped.
fn initialize_lessons_state(
    mut commands: Commands,
    settings: Res<Persistent<GameSettings>>,
    teacher_tab: Option<Res<TeacherTab>>,
) {
    if settings.mode != GameMode::Group {
        return;
    }
    if teacher_tab
        .as_ref()
        .is_none_or(|t| **t != TeacherTab::Lessons)
    {
        return;
    }
    commands.insert_resource(TeacherLessonsState {
        view: LessonsView::Tree,
    });
}

#[derive(Resource, Reflect)]
pub struct TeacherLessonsState {
    #[reflect(ignore)]
    pub(super) view: LessonsView,
}

#[derive(Clone, Default)]
pub(super) enum LessonsView {
    #[default]
    Tree,
    Config {
        lesson_id: String,
        lesson_title: String,
    },
}

/// Holds the per-question draft while the Config view is open.
/// Inserted alongside [`LessonsView::Config`] and removed on return/save.
#[derive(Resource, Clone, Debug)]
pub(super) struct LessonConfigDraftRes {
    pub questions: Vec<DraftQuestion>,
}

impl LessonConfigDraftRes {
    /// Returns `true` if at least one question has a count > 0.
    pub(super) fn has_any_selected(&self) -> bool {
        self.questions.iter().any(|q| q.count > 0)
    }
}

#[derive(Clone, Debug)]
pub(super) struct DraftQuestion {
    pub index: usize,
    pub question_type: QuestionType,
    pub full_prompt: String,
    pub count: usize,
    /// Whether this question has an optional visual at all.
    pub has_visual: bool,
    /// Whether the optional visual is currently enabled.
    pub show_visual: bool,
    /// The default visibility for the optional visual (used by reset).
    pub default_show_visual: bool,
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

/// Marker + storage for the full prompt text on question label entities.
/// Used by [`config::update_question_labels`] to dynamically truncate with "...".
#[derive(Component, Reflect)]
struct QuestionLabel(String);

/// Marker for the hover detail text shown outside the scroll frame.
/// Displays the full prompt of the currently hovered question row.
#[derive(Component, Reflect)]
struct ConfigHoverText;

fn rebuild_lessons_ui(
    mut commands: Commands,
    ts: TeacherScreenParam<'_, '_>,
    existing_root: Query<Entity, With<TeacherLessonsRoot>>,
    state: Res<TeacherLessonsState>,
    draft_res: Option<Res<LessonConfigDraftRes>>,
    content: Res<ContentLibrary>,
    app_state: Res<State<AppState>>,
) {
    // Always tear down before rebuilding.
    for entity in &existing_root {
        commands.entity(entity).despawn();
    }

    // Tab guard: Lessons tab must be active.
    if ts
        .teacher_tab
        .as_ref()
        .is_none_or(|t| **t != TeacherTab::Lessons)
    {
        commands.remove_resource::<TeacherLessonsState>();
        commands.remove_resource::<LessonConfigDraftRes>();
        return;
    }
    if ts.ctx.settings.mode != GameMode::Group {
        commands.remove_resource::<TeacherLessonsState>();
        commands.remove_resource::<LessonConfigDraftRes>();
        return;
    }

    let camera_entity = *ts.teacher.camera;
    let window = *ts.teacher.window;
    let active_tab = ts.teacher_tab.map_or(TeacherTab::Lessons, |t| *t);

    match &state.view {
        LessonsView::Config { lesson_title, .. } => {
            let Some(draft_res) = draft_res else { return };
            let draft = draft_res.clone();
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
                TeacherContentRoot,
                Children::spawn(SpawnWith(move |parent: &mut ChildSpawner| {
                    config::spawn_config_view(
                        parent,
                        &i18n_owned,
                        &title,
                        &draft,
                        active_tab,
                        window,
                    );
                })),
            ));
        }
        LessonsView::Tree => {
            let is_map_exploration = *app_state.get() == AppState::MapExploration;
            let header = tab_header(&ts.i18n, active_tab, window);
            let i18n_owned = I18n::new(ts.i18n.language);
            let tree_specs = tree::build_tree_specs(
                &content.themes,
                &ts.ctx.save_data,
                ts.ctx.active_slot.as_deref(),
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
                TeacherContentRoot,
                Children::spawn(SpawnWith(move |parent: &mut ChildSpawner| {
                    parent.spawn(header);
                    tree::spawn_tree_view(
                        parent,
                        &tree_specs,
                        &i18n_owned,
                        is_map_exploration,
                        window,
                    );
                })),
            ));
        }
    }
}

/// On any tab switch, drop Lessons state (and its draft). When the new tab is
/// Lessons, insert a fresh state so [`rebuild_lessons_ui`] runs on the next
/// `Update` frame via the `resource_changed` run condition.
fn on_teacher_tab_changed(event: On<TeacherTabChanged>, mut commands: Commands) {
    commands.remove_resource::<TeacherLessonsState>();
    commands.remove_resource::<LessonConfigDraftRes>();
    if event.event().0 == TeacherTab::Lessons {
        commands.insert_resource(TeacherLessonsState {
            view: LessonsView::Tree,
        });
    }
}
