use bevy::input_focus::tab_navigation::TabGroup;
use bevy::prelude::*;
use bevy_persistent::prelude::*;

use crate::data::{ActiveSlot, ActiveStudent, ClassStudent, GameMode, SaveData};
use crate::i18n::{I18n, TranslationKey};
use crate::plugins::teacher::{
    TeacherContentRoot, TeacherInDetailView, TeacherScreenParam, TeacherTab, TeacherTabChanged,
    TeacherWindowInit, TeacherWindowParam, tab_header,
};
use crate::screens::teacher_shared::{RebuildRoster, RebuildStats, ViewingStudentStats};
use crate::states::{
    AppState, InLessonFlow, LESSON_FLOW_STATES, LessonPhase, StateScopedResourceExt, cleanup_root,
};
use crate::ui::components::{
    PopoverCancelButton, PopoverConfirmButton, button_base, icon_button, spawn_confirmation_modal,
};
use crate::ui::text_input::{TextInputState, text_input};
use crate::ui::theme;
use crate::ui::theme::DesignFontSize;

/// Teacher roster tab for managing student names in a class slot.
pub struct TeacherRosterScreenPlugin;

impl Plugin for TeacherRosterScreenPlugin {
    fn build(&self, app: &mut App) {
        // Per-state registrations: resource scoping, roster rebuild, and cleanup
        // must fire on every intra-flow transition (e.g. MapExploration to LessonPlay).
        for &state in &LESSON_FLOW_STATES {
            app.register_state_scoped_resource::<AppState, TeacherRosterState>(state)
                .add_systems(OnExit(state), cleanup_root::<TeacherRosterRoot>);

            if state == AppState::MapExploration {
                // On first entry the teacher camera may not exist yet;
                // ensure the rebuild runs after TeacherWindowInit.
                app.add_systems(
                    OnEnter(state),
                    trigger_rebuild_roster.after(TeacherWindowInit),
                );
            } else {
                app.add_systems(OnEnter(state), trigger_rebuild_roster);
            }
        }

        app.add_observer(on_rebuild_roster)
            .add_observer(on_teacher_tab_changed)
            // Full roster editing (add, remove, input) only during MapExploration
            .add_systems(
                Update,
                (
                    handle_add_student,
                    handle_remove_student_click,
                    handle_confirm_remove_student,
                    handle_cancel_remove_student,
                )
                    .run_if(in_state(AppState::MapExploration))
                    .run_if(resource_exists::<TeacherRosterState>),
            )
            // Student selection available across all lesson-flow states.
            // sync_roster_selection only runs on the frame ActiveStudent is removed,
            // and is chained before handle_student_click so a click in that same
            // frame is not undone.
            .add_systems(
                Update,
                (
                    sync_roster_selection.run_if(resource_removed::<ActiveStudent>),
                    handle_student_click,
                )
                    .chain()
                    .run_if(in_state(InLessonFlow))
                    .run_if(resource_exists::<TeacherRosterState>),
            );
    }
}

fn trigger_rebuild_roster(mut commands: Commands) {
    commands.trigger(RebuildRoster);
}

#[derive(Resource, Reflect)]
pub struct TeacherRosterState {
    selected_student: Option<usize>,
    last_click: Option<(usize, f64)>,
}

#[derive(Component, Reflect)]
pub struct TeacherRosterRoot;

#[derive(Component, Reflect)]
struct StudentRow(usize);

#[derive(Component, Reflect)]
struct RemoveStudentButton(usize);

#[derive(Component, Reflect)]
struct StudentRemovePopover;

#[derive(Component, Reflect)]
struct RemoveStudentTarget(usize);

#[derive(Component, Reflect)]
struct AddStudentButton;

/// Builds the roster UI when triggered via [`RebuildRoster`].
/// Always tears down any existing root before rebuilding, so callers
/// only need to trigger the event without manually despawning entities.
fn on_rebuild_roster(
    _event: On<RebuildRoster>,
    mut commands: Commands,
    ts: TeacherScreenParam<'_, '_>,
    viewing_stats: Option<Res<ViewingStudentStats>>,
    app_state: Res<State<AppState>>,
    existing_root: Query<Entity, With<TeacherRosterRoot>>,
) {
    for entity in &existing_root {
        commands.entity(entity).despawn();
    }
    commands.remove_resource::<TeacherRosterState>();

    // Don't build while viewing stats
    if viewing_stats.is_some() {
        return;
    }
    // Tab guard: only build if Students tab is active (or no tab resource = legacy)
    if ts
        .teacher_tab
        .as_ref()
        .is_some_and(|t| **t != TeacherTab::Students)
    {
        return;
    }
    if ts.ctx.settings.mode != GameMode::Group {
        return;
    }
    let camera_entity = *ts.teacher.camera;
    let window = *ts.teacher.window;
    let Some(ref slot) = ts.ctx.active_slot else {
        return;
    };
    let Some(ref class_save) = ts.ctx.save_data.class_slots[slot.0] else {
        return;
    };

    // Preserve ActiveStudent during lessons; only clear on MapExploration entry
    let selected_index = ts.ctx.active_student.as_ref().map(|student| student.0);
    if *app_state.get() == AppState::MapExploration {
        commands.remove_resource::<ActiveStudent>();
    }

    commands.insert_resource(TeacherRosterState {
        selected_student: selected_index,
        last_click: None,
    });

    let students = class_save.students.clone();
    let show_input = *app_state.get() == AppState::MapExploration;
    let active_tab = ts.teacher_tab.map_or(TeacherTab::Students, |t| *t);

    // Pre-compute all i18n strings before the SpawnWith closure
    let tab_header_bundle = tab_header(&ts.i18n, active_tab, window);
    let title_text = ts
        .i18n
        .t(&TranslationKey::StudentsOf(class_save.name.clone()))
        .into_owned();
    let no_students_text = ts.i18n.t(&TranslationKey::NoStudentsYet).into_owned();
    let name_label = ts.i18n.t(&TranslationKey::NameLabel).into_owned();
    let add_label = ts.i18n.t(&TranslationKey::Add).into_owned();

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
        TeacherRosterRoot,
        TeacherContentRoot,
        Children::spawn(SpawnWith(move |parent: &mut ChildSpawner| {
            parent.spawn(tab_header_bundle);

            parent.spawn((
                Text::new(title_text),
                TextFont {
                    font_size: theme::fonts::HEADING,
                    ..default()
                },
                TextColor(theme::colors::TEXT_DARK),
                DesignFontSize {
                    size: theme::fonts::HEADING,
                    window,
                },
            ));

            spawn_student_list(
                parent,
                &students,
                &no_students_text,
                selected_index,
                show_input,
                window,
            );
            if show_input {
                spawn_add_student_row(parent, &name_label, &add_label, window);
            }
        })),
    ));
}

fn spawn_student_list(
    parent: &mut ChildSpawner,
    students: &[crate::data::ClassStudent],
    no_students_text: &str,
    selected_index: Option<usize>,
    show_delete: bool,
    window: Entity,
) {
    parent
        .spawn((Node {
            flex_direction: FlexDirection::Column,
            row_gap: theme::scaled(theme::spacing::SMALL),
            flex_grow: 1.0,
            overflow: Overflow::scroll_y(),
            ..default()
        },))
        .with_children(|list| {
            for (i, student) in students.iter().enumerate() {
                let is_selected = selected_index == Some(i);
                spawn_student_row(list, i, &student.name, is_selected, show_delete, window);
            }

            if students.is_empty() {
                list.spawn((
                    Text::new(no_students_text.to_owned()),
                    TextFont {
                        font_size: theme::fonts::BODY,
                        ..default()
                    },
                    TextColor(theme::colors::TEXT_MUTED),
                    DesignFontSize {
                        size: theme::fonts::BODY,
                        window,
                    },
                ));
            }
        });
}

fn spawn_add_student_row(
    parent: &mut ChildSpawner,
    name_label: &str,
    add_label: &str,
    window: Entity,
) {
    parent.spawn((
        Node {
            flex_direction: FlexDirection::Row,
            column_gap: theme::scaled(theme::spacing::SMALL),
            align_items: AlignItems::Center,
            ..default()
        },
        children![
            (
                Text::new(name_label),
                TextFont {
                    font_size: theme::fonts::BODY,
                    ..default()
                },
                TextColor(theme::colors::TEXT_DARK),
                DesignFontSize {
                    size: theme::fonts::BODY,
                    window,
                },
            ),
            text_input(200.0, TextInputState::new(30), window),
            (
                button_base(theme::colors::SUCCESS),
                Node {
                    min_width: theme::scaled(60.0),
                    height: theme::scaled(theme::sizes::INPUT_FIELD_HEIGHT),
                    justify_content: JustifyContent::Center,
                    align_items: AlignItems::Center,
                    border_radius: BorderRadius::all(theme::scaled(
                        theme::sizes::BUTTON_BORDER_RADIUS
                    )),
                    ..default()
                },
                AddStudentButton,
                children![(
                    Text::new(add_label),
                    TextFont {
                        font_size: theme::fonts::SMALL,
                        ..default()
                    },
                    TextColor(theme::colors::TEXT_LIGHT),
                    DesignFontSize {
                        size: theme::fonts::SMALL,
                        window,
                    },
                )],
            ),
        ],
    ));
}

fn spawn_student_row(
    parent: &mut ChildSpawner,
    index: usize,
    name: &str,
    is_selected: bool,
    show_delete: bool,
    window: Entity,
) {
    let bg = if is_selected {
        theme::colors::PRIMARY_HOVER
    } else {
        theme::colors::CARD_BG
    };

    parent
        .spawn((
            button_base(bg),
            Node {
                flex_direction: FlexDirection::Row,
                justify_content: JustifyContent::SpaceBetween,
                align_items: AlignItems::Center,
                padding: theme::scaled(theme::spacing::SMALL).all(),
                border_radius: BorderRadius::all(theme::scaled(6.0)),
                ..default()
            },
            StudentRow(index),
        ))
        .with_children(|row| {
            row.spawn((
                Text::new(name.to_owned()),
                TextFont {
                    font_size: theme::fonts::BODY,
                    ..default()
                },
                TextColor(theme::colors::TEXT_DARK),
                DesignFontSize {
                    size: theme::fonts::BODY,
                    window,
                },
            ));

            if show_delete {
                row.spawn((
                    icon_button(
                        28.0,
                        4.0,
                        "X",
                        theme::fonts::SMALL,
                        theme::colors::ERROR,
                        theme::colors::TEXT_LIGHT,
                        window,
                    ),
                    RemoveStudentButton(index),
                ));
            }
        });
}

fn handle_add_student(
    query: Query<&Interaction, (Changed<Interaction>, With<AddStudentButton>)>,
    keyboard: Res<ButtonInput<KeyCode>>,
    active_slot: Option<Res<ActiveSlot>>,
    mut save_data: ResMut<Persistent<SaveData>>,
    mut commands: Commands,
    input: Single<&TextInputState>,
) {
    let Some(ref slot) = active_slot else { return };

    let pressed_button = query.iter().any(|i| *i == Interaction::Pressed);
    let pressed_enter = input.focused && keyboard.just_pressed(KeyCode::Enter);

    if !pressed_button && !pressed_enter {
        return;
    }

    let name = input.text.trim().to_owned();
    if name.is_empty() {
        return;
    }

    save_data
        .update(|data| {
            if let Some(ref mut class_save) = data.class_slots[slot.0] {
                class_save.students.push(ClassStudent {
                    name: name.clone(),
                    ..Default::default()
                });
            }
        })
        .expect("failed to update save data");

    commands.trigger(RebuildRoster);
}

fn handle_remove_student_click(
    query: Query<(&Interaction, &RemoveStudentButton), Changed<Interaction>>,
    active_slot: Option<Res<ActiveSlot>>,
    save_data: Res<Persistent<SaveData>>,
    mut commands: Commands,
    existing_popover: Query<Entity, With<StudentRemovePopover>>,
    i18n: Res<I18n>,
    teacher: TeacherWindowParam<'_, '_>,
) {
    let Some(ref slot) = active_slot else { return };
    let window = *teacher.window;

    for (interaction, remove_btn) in &query {
        if *interaction == Interaction::Pressed {
            // Despawn any existing popover first
            for entity in &existing_popover {
                commands.entity(entity).try_despawn();
            }

            let student_index = remove_btn.0;
            let student_name = save_data.class_slots[slot.0]
                .as_ref()
                .and_then(|cs| cs.students.get(student_index))
                .map_or_else(String::new, |s| s.name.clone());

            let modal_entity = spawn_confirmation_modal(
                &mut commands,
                &i18n.t(&TranslationKey::RemoveStudentConfirm(student_name)),
                &i18n.t(&TranslationKey::Delete),
                &i18n.t(&TranslationKey::Cancel),
                theme::colors::ERROR,
                window,
                Some(*teacher.camera),
            );
            commands
                .entity(modal_entity)
                .insert((StudentRemovePopover, RemoveStudentTarget(student_index)));
        }
    }
}

fn handle_confirm_remove_student(
    query: Query<&Interaction, (Changed<Interaction>, With<PopoverConfirmButton>)>,
    popover: Single<(Entity, &RemoveStudentTarget), With<StudentRemovePopover>>,
    active_slot: Option<Res<ActiveSlot>>,
    mut save_data: ResMut<Persistent<SaveData>>,
    mut commands: Commands,
) {
    let Some(ref slot) = active_slot else { return };

    for interaction in &query {
        if *interaction != Interaction::Pressed {
            continue;
        }

        let student_index = popover.1.0;

        save_data
            .update(|data| {
                if let Some(ref mut class_save) = data.class_slots[slot.0]
                    && student_index < class_save.students.len()
                {
                    class_save.students.remove(student_index);
                }
            })
            .expect("failed to update save data");

        commands.trigger(RebuildRoster);
    }
}

fn handle_cancel_remove_student(
    query: Query<&Interaction, (Changed<Interaction>, With<PopoverCancelButton>)>,
    mut commands: Commands,
    popover_query: Query<Entity, With<StudentRemovePopover>>,
) {
    for interaction in &query {
        if *interaction == Interaction::Pressed {
            for entity in &popover_query {
                commands.entity(entity).try_despawn();
            }
        }
    }
}

/// Tears down the roster and opens the stats view for `student_index`.
/// Triggering `RebuildRoster` handles roster cleanup; `RebuildStats` builds
/// the stats UI.
fn enter_student_stats(commands: &mut Commands, student_index: usize) {
    commands.insert_resource(ViewingStudentStats(student_index));
    commands.insert_resource(TeacherInDetailView);
    commands.trigger(RebuildRoster);
    commands.trigger(RebuildStats);
}

/// Highlight the selected row and clear all others.
fn update_roster_selection(
    bg_query: &mut Query<(&StudentRow, &mut BackgroundColor)>,
    selected_index: usize,
) {
    for (student_row, mut bg) in bg_query {
        *bg = if student_row.0 == selected_index {
            theme::colors::PRIMARY_HOVER.into()
        } else {
            theme::colors::CARD_BG.into()
        };
    }
}

#[allow(clippy::too_many_arguments)]
fn handle_student_click(
    query: Query<(&Interaction, &StudentRow), Changed<Interaction>>,
    mut state: ResMut<TeacherRosterState>,
    mut bg_query: Query<(&StudentRow, &mut BackgroundColor)>,
    mut commands: Commands,
    time: Res<Time>,
    popover_query: Query<Entity, With<StudentRemovePopover>>,
    app_state: Res<State<AppState>>,
    lesson_phase: Option<Res<State<LessonPhase>>>,
) {
    // Freeze selection during feedback / transition (answer already attributed)
    if let Some(ref phase) = lesson_phase
        && matches!(
            phase.get(),
            LessonPhase::ShowFeedback | LessonPhase::Transitioning
        )
    {
        return;
    }

    if !popover_query.is_empty() {
        return;
    }

    for (interaction, row) in &query {
        if *interaction == Interaction::Pressed {
            let now = time.elapsed_secs_f64();
            let is_double_click = state
                .last_click
                .is_some_and(|(idx, t)| idx == row.0 && now - t < 0.4);

            if is_double_click && *app_state.get() == AppState::MapExploration {
                enter_student_stats(&mut commands, row.0);
                return;
            }

            // Single click (or double-click during lesson): select student
            state.last_click = Some((row.0, now));
            state.selected_student = Some(row.0);
            commands.insert_resource(ActiveStudent(row.0));
            update_roster_selection(&mut bg_query, row.0);
        }
    }
}

/// Clears the roster's visual selection when `ActiveStudent` is removed externally.
/// Gated by `resource_removed` so it runs only on the transition frame.
fn sync_roster_selection(
    mut state: ResMut<TeacherRosterState>,
    mut bg_query: Query<(&StudentRow, &mut BackgroundColor)>,
) {
    state.selected_student = None;
    for (_row, mut bg) in &mut bg_query {
        *bg = theme::colors::CARD_BG.into();
    }
}

/// On any tab switch, clean up roster and stats state, then rebuild if the
/// Students tab is the new target.
fn on_teacher_tab_changed(event: On<TeacherTabChanged>, mut commands: Commands) {
    commands.remove_resource::<TeacherRosterState>();
    commands.remove_resource::<ViewingStudentStats>();
    if event.event().0 == TeacherTab::Students {
        commands.trigger(RebuildRoster);
    }
}
