use bevy::input_focus::tab_navigation::TabGroup;
use bevy::prelude::*;

use crate::data::{
    ActiveStudent, ClassStudent, GameMode, PersistenceAction, PersistenceStatus, PlayerSession,
    SaveData, SaveDataMut,
};
use crate::i18n::{I18n, TranslationKey};
use crate::plugins::teacher::{
    TeacherScreenParam, TeacherView, TeacherViewOverlay, TeacherWindow, TeacherWindowParam,
    TeacherWindowState, tab_header, teacher_roster_view_active, teacher_window_exists,
};
use crate::states::{AppState, InLessonFlow, LESSON_FLOW_STATES, LessonPhase, cleanup_root};
use crate::ui::components::{
    ButtonActivated, ConfirmationDialogAction, ConfirmationDialogActionEvent, button_base,
    icon_button, persistence_notice, spawn_confirmation_modal,
};
use crate::ui::text_input::{TextInputState, text_input};
use crate::ui::theme;

/// Teacher roster tab for managing student names in a class slot.
pub struct TeacherRosterScreenPlugin;

impl Plugin for TeacherRosterScreenPlugin {
    fn build(&self, app: &mut App) {
        for &state in &LESSON_FLOW_STATES {
            app.add_systems(OnExit(state), cleanup_root::<TeacherRosterRoot>);
        }

        app.add_systems(
            Update,
            rebuild_roster_ui
                .run_if(in_state(InLessonFlow))
                .run_if(teacher_window_exists),
        )
        .add_systems(
            Update,
            (handle_add_student, handle_remove_student_click)
                .run_if(in_state(AppState::ThemeExploration))
                .run_if(teacher_roster_view_active),
        )
        .add_systems(
            Update,
            (
                sync_roster_selection.run_if(resource_changed_or_removed::<ActiveStudent>),
                handle_student_click,
            )
                .chain()
                .run_if(in_state(InLessonFlow))
                .run_if(teacher_roster_view_active),
        );
    }
}

#[derive(Component, Reflect)]
pub struct TeacherRosterRoot;

#[derive(Component, Reflect)]
struct StudentRow(usize, Option<f64>);

#[derive(Component, Reflect)]
struct RemoveStudentButton(usize);

#[derive(Component, Reflect)]
struct StudentRemovePopover;

#[derive(Component, Reflect)]
struct RemoveStudentTarget(usize, String);

#[derive(Component, Reflect)]
struct AddStudentButton;

/// Rebuilds the roster when its view or backing save data changes.
fn rebuild_roster_ui(
    mut commands: Commands,
    ts: TeacherScreenParam<'_, '_>,
    status: Res<PersistenceStatus>,
    app_state: Res<State<AppState>>,
    existing_root: Query<Entity, With<TeacherRosterRoot>>,
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

    if !matches!(&state.view, TeacherView::Students) {
        return;
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
    let Some(ref session) = ts.ctx.session else {
        return;
    };
    let Some(ref class_save) = ts.ctx.save_data.class_slots[session.slot_index] else {
        return;
    };

    let selected_index = ts.ctx.active_student.as_ref().map(|student| student.0);

    let student_names: Vec<String> = class_save.students.iter().map(|s| s.name.clone()).collect();
    let show_input = *app_state.get() == AppState::ThemeExploration;
    let active_tab = state.view.tab();

    // Pre-compute all i18n strings before the SpawnWith closure
    let tab_header_bundle = tab_header(&ts.i18n, active_tab, window);
    let title_text = ts
        .i18n
        .t(&TranslationKey::StudentsOf(class_save.name.clone()))
        .into_owned();
    let no_students_text = ts.i18n.t(&TranslationKey::NoStudentsYet).into_owned();
    let name_label = ts.i18n.t(&TranslationKey::NameLabel).into_owned();
    let add_label = ts.i18n.t(&TranslationKey::Add).into_owned();
    let write_notice =
        persistence_notice(PersistenceAction::TeacherRoster, *status, &ts.i18n, window);

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
        Children::spawn(SpawnWith(move |parent: &mut ChildSpawner| {
            parent.spawn(tab_header_bundle);
            parent.spawn(write_notice);

            parent.spawn((
                Text::new(title_text),
                theme::typography::text(theme::fonts::HEADING, window),
                TextColor(theme::colors::TEXT_DARK),
            ));

            spawn_student_list(
                parent,
                &student_names,
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
    names: &[String],
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
            for (i, name) in names.iter().enumerate() {
                let is_selected = selected_index == Some(i);
                spawn_student_row(list, i, name, is_selected, show_delete, window);
            }

            if names.is_empty() {
                list.spawn((
                    Text::new(no_students_text),
                    theme::typography::text(theme::fonts::BODY, window),
                    TextColor(theme::colors::TEXT_MUTED),
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
                theme::typography::text(theme::fonts::BODY, window),
                TextColor(theme::colors::TEXT_DARK),
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
                    theme::typography::text(theme::fonts::SMALL, window),
                    TextColor(theme::colors::TEXT_LIGHT),
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
            StudentRow(index, None),
        ))
        .with_children(|row| {
            row.spawn((
                Text::new(name),
                theme::typography::text(theme::fonts::BODY, window),
                TextColor(theme::colors::TEXT_DARK),
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
    mut activations: MessageReader<ButtonActivated>,
    query: Query<(), With<AddStudentButton>>,
    keyboard: Res<ButtonInput<KeyCode>>,
    session: Option<Res<PlayerSession>>,
    mut persistence: SaveDataMut<'_>,
    input: Query<&TextInputState>,
) {
    let activated_entities: Vec<Entity> = activations.read().map(|event| event.0).collect();
    let pressed_button = activated_entities
        .iter()
        .any(|entity| query.contains(*entity));
    let Some(ref session) = session else { return };
    let input = input.single().ok();
    let pressed_enter =
        input.is_some_and(|input| input.focused && keyboard.just_pressed(KeyCode::Enter));

    if !pressed_button && !pressed_enter {
        return;
    }

    let Some(input) = input else {
        return;
    };
    let name = input.text.trim().to_owned();
    if name.is_empty() {
        return;
    }

    persistence.update(PersistenceAction::TeacherRoster, |data| {
        if let Some(ref mut class_save) = data.class_slots[session.slot_index] {
            class_save.students.push(ClassStudent {
                name: name.clone(),
                ..Default::default()
            });
        }
    });
}

#[allow(clippy::too_many_arguments)]
fn handle_remove_student_click(
    mut activations: MessageReader<ButtonActivated>,
    query: Query<&RemoveStudentButton>,
    session: Option<Res<PlayerSession>>,
    save_data: Res<SaveData>,
    mut commands: Commands,
    existing_popover: Query<Entity, With<StudentRemovePopover>>,
    i18n: Res<I18n>,
    teacher: TeacherWindowParam<'_, '_>,
) {
    let activated_entities: Vec<Entity> = activations.read().map(|event| event.0).collect();
    let Some(ref session) = session else { return };
    let Ok(window) = teacher.window.single() else {
        return;
    };
    let Ok(camera) = teacher.camera.single() else {
        return;
    };

    for entity in activated_entities {
        let Ok(remove_btn) = query.get(entity) else {
            continue;
        };
        // Despawn any existing popover first
        for entity in &existing_popover {
            commands.entity(entity).try_despawn();
        }

        let student_index = remove_btn.0;
        let student_name = save_data.class_slots[session.slot_index]
            .as_ref()
            .and_then(|cs| cs.students.get(student_index))
            .map_or_else(String::new, |s| s.name.clone());

        let modal_entity = spawn_confirmation_modal(
            &mut commands,
            &i18n.t(&TranslationKey::RemoveStudentConfirm(student_name.clone())),
            &i18n.t(&TranslationKey::Delete),
            &i18n.t(&TranslationKey::Cancel),
            theme::colors::ERROR,
            window,
            Some(camera),
        );
        commands
            .entity(modal_entity)
            .insert((
                StudentRemovePopover,
                TeacherViewOverlay,
                RemoveStudentTarget(student_index, student_name),
                DespawnOnExit(AppState::ThemeExploration),
            ))
            .observe(handle_confirm_remove_student);
    }
}

fn handle_confirm_remove_student(
    event: On<ConfirmationDialogActionEvent>,
    target_query: Query<&RemoveStudentTarget>,
    session: Option<Res<PlayerSession>>,
    active_student: Option<Res<ActiveStudent>>,
    mut persistence: SaveDataMut<'_>,
    mut commands: Commands,
) {
    if event.action != ConfirmationDialogAction::Confirm {
        return;
    }
    let Some(ref session) = session else { return };
    let Ok(target) = target_query.get(event.entity) else {
        return;
    };
    let student_index = target.0;
    let student_name = target.1.clone();
    let Some(class_save) = persistence.save_data.class_slots[session.slot_index].as_ref() else {
        return;
    };
    if class_save
        .students
        .get(student_index)
        .is_none_or(|student| student.name != student_name)
    {
        return;
    }

    persistence.update(PersistenceAction::TeacherRoster, |data| {
        if let Some(ref mut class_save) = data.class_slots[session.slot_index] {
            class_save.students.remove(student_index);
        }
    });

    match active_student.as_deref().map(|student| student.0) {
        Some(index) if index == student_index => commands.remove_resource::<ActiveStudent>(),
        Some(index) if index > student_index => {
            commands.insert_resource(ActiveStudent(index - 1));
        }
        _ => {}
    }
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

#[allow(clippy::too_many_arguments, clippy::type_complexity)]
fn handle_student_click(
    mut activations: MessageReader<ButtonActivated>,
    mut row_queries: ParamSet<(
        Query<(Entity, &mut StudentRow)>,
        Query<(&StudentRow, &mut BackgroundColor)>,
    )>,
    mut commands: Commands,
    active_student: Option<Res<ActiveStudent>>,
    time: Res<Time>,
    popover_query: Query<Entity, With<StudentRemovePopover>>,
    app_state: Res<State<AppState>>,
    lesson_phase: Option<Res<State<LessonPhase>>>,
    mut teacher_state: Query<&mut TeacherWindowState, With<TeacherWindow>>,
) {
    let activated_entities: Vec<Entity> = activations.read().map(|event| event.0).collect();
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
    let Ok(mut teacher_state) = teacher_state.single_mut() else {
        return;
    };

    let mut selected_student = None;
    for entity in activated_entities {
        let now = time.elapsed_secs_f64();
        let (student_index, is_double_click) = {
            let mut rows = row_queries.p0();
            let Ok((_, mut row)) = rows.get_mut(entity) else {
                continue;
            };
            let is_double_click = active_student.as_deref().is_some_and(|student| {
                student.0 == row.0 && row.1.is_some_and(|last_click| now - last_click < 0.4)
            });
            let student_index = row.0;
            if !is_double_click {
                row.1 = Some(now);
            }
            (student_index, is_double_click)
        };

        if is_double_click && *app_state.get() == AppState::ThemeExploration {
            teacher_state.view = TeacherView::StudentStats { student_index };
            return;
        }

        selected_student = Some(student_index);
    }

    if let Some(student_index) = selected_student {
        if active_student.as_deref().map(|student| student.0) != Some(student_index) {
            commands.insert_resource(ActiveStudent(student_index));
        }
        update_roster_selection(&mut row_queries.p1(), student_index);
    }
}

fn sync_roster_selection(
    active_student: Option<Res<ActiveStudent>>,
    mut bg_query: Query<(&StudentRow, &mut BackgroundColor)>,
) {
    let selected_index = active_student.as_deref().map(|student| student.0);
    for (row, mut bg) in &mut bg_query {
        *bg = if selected_index == Some(row.0) {
            theme::colors::PRIMARY_HOVER.into()
        } else {
            theme::colors::CARD_BG.into()
        };
    }
}
