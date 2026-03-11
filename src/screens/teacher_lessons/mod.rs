mod config;
mod tree;

use bevy::input_focus::tab_navigation::TabGroup;
use bevy::prelude::*;

use crate::data::content::QuestionType;
use crate::data::{ContentLibrary, GameMode};
use crate::i18n::I18n;
use crate::plugins::teacher::{
    TeacherContentRoot, TeacherScreenParam, TeacherTab, TeacherTabChanged, TeacherWindowInit,
    tab_header,
};
use crate::states::{AppState, LESSON_FLOW_STATES, StateScopedResourceExt, cleanup_root};
use crate::ui::theme;

/// Teacher lessons tab for configuring per-lesson question selection.
pub struct TeacherLessonsScreenPlugin;

/// Trigger this event to (re)build the lessons tab UI.
#[derive(Event)]
pub struct RebuildLessons;

impl Plugin for TeacherLessonsScreenPlugin {
    fn build(&self, app: &mut App) {
        for &state in &LESSON_FLOW_STATES {
            app.register_state_scoped_resource::<AppState, TeacherLessonsState>(state)
                .add_systems(OnExit(state), cleanup_root::<TeacherLessonsRoot>);

            if state == AppState::MapExploration {
                app.add_systems(
                    OnEnter(state),
                    trigger_rebuild_lessons.after(TeacherWindowInit),
                );
            } else {
                app.add_systems(OnEnter(state), trigger_rebuild_lessons);
            }
        }

        app.add_observer(on_rebuild_lessons)
            .add_observer(on_teacher_tab_changed)
            .add_systems(
                Update,
                (
                    config::handle_config_button_click,
                    config::handle_count_increment,
                    config::handle_count_decrement,
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

fn trigger_rebuild_lessons(mut commands: Commands) {
    commands.trigger(RebuildLessons);
}

#[derive(Resource, Reflect)]
pub struct TeacherLessonsState {
    #[reflect(ignore)]
    view: LessonsView,
}

#[derive(Clone, Default)]
enum LessonsView {
    #[default]
    Tree,
    Config {
        lesson_id: String,
        lesson_title: String,
        editing: LessonConfigDraft,
    },
}

#[derive(Clone, Debug)]
struct LessonConfigDraft {
    questions: Vec<DraftQuestion>,
}

impl LessonConfigDraft {
    /// Returns `true` if at least one question has a count > 0.
    fn has_any_selected(&self) -> bool {
        self.questions.iter().any(|q| q.count > 0)
    }
}

#[derive(Clone, Debug)]
struct DraftQuestion {
    index: usize,
    question_type: QuestionType,
    full_prompt: String,
    count: usize,
    /// Whether this question has an optional visual at all.
    has_visual: bool,
    /// Whether the optional visual is currently enabled.
    show_visual: bool,
    /// The default visibility for the optional visual (used by reset).
    default_show_visual: bool,
}

#[derive(Component, Reflect)]
pub struct TeacherLessonsRoot;

#[derive(Component, Reflect)]
struct ConfigLessonButton {
    theme_id: String,
    lesson_id: String,
}

#[derive(Component, Reflect)]
struct CountIncrementButton {
    index: usize,
    count_text: Entity,
}

#[derive(Component, Reflect)]
struct CountDecrementButton {
    index: usize,
    count_text: Entity,
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
struct QuestionRow(usize);

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

fn on_rebuild_lessons(
    _event: On<RebuildLessons>,
    mut commands: Commands,
    ts: TeacherScreenParam<'_, '_>,
    existing_root: Query<Entity, With<TeacherLessonsRoot>>,
    existing_state: Option<Res<TeacherLessonsState>>,
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
        return;
    }
    if ts.ctx.settings.mode != GameMode::Group {
        commands.remove_resource::<TeacherLessonsState>();
        return;
    }

    let camera_entity = *ts.teacher.camera;
    let window = *ts.teacher.window;
    let active_tab = ts.teacher_tab.map_or(TeacherTab::Lessons, |t| *t);

    // If state already holds a Config view (set by handle_config_button_click),
    // rebuild that config view. Otherwise spawn the lesson tree.
    if let Some(LessonsView::Config {
        lesson_title,
        editing,
        ..
    }) = existing_state.as_ref().map(|s| &s.view)
    {
        let draft = editing.clone();
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
                config::spawn_config_view(parent, &i18n_owned, &title, &draft, active_tab, window);
            })),
        ));
    } else {
        let is_map_exploration = *app_state.get() == AppState::MapExploration;
        commands.insert_resource(TeacherLessonsState {
            view: LessonsView::Tree,
        });
        let header = tab_header(&ts.i18n, active_tab, window);
        let i18n_owned = I18n::new(ts.i18n.language);
        let themes = content.themes.clone();
        let save_data_cloned = (*ts.ctx.save_data).clone();
        let active_slot_cloned = ts.ctx.active_slot.map(|s| (*s).clone());
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
                    &themes,
                    &i18n_owned,
                    is_map_exploration,
                    &save_data_cloned,
                    active_slot_cloned.as_ref(),
                    window,
                );
            })),
        ));
    }
}

/// On any tab switch, clean up lessons state, then rebuild if the Lessons
/// tab is the new target.
fn on_teacher_tab_changed(event: On<TeacherTabChanged>, mut commands: Commands) {
    commands.remove_resource::<TeacherLessonsState>();
    if event.event().0 == TeacherTab::Lessons {
        commands.trigger(RebuildLessons);
    }
}
