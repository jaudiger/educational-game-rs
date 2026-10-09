use std::collections::HashMap;

use bevy::input_focus::tab_navigation::TabGroup;
use bevy::prelude::*;

use crate::data::content::QuestionType;
use crate::data::{
    ContentLibrary, GameMode, Language, LessonProgress, PersistenceAction, PersistenceStatus,
    PlayerContext, PlayerSession, SaveDataMut,
};
use crate::i18n::{I18n, TranslationKey};
use crate::plugins::teacher::{
    TeacherScreenParam, TeacherView, TeacherViewOverlay, TeacherWindow, TeacherWindowParam,
    TeacherWindowState, tab_header, teacher_stats_view_active, teacher_window_exists,
};
use crate::screens::teacher_shared::question_type_label;
use crate::states::{AppState, cleanup_root};
use crate::ui::components::{
    ConfirmationDialogAction, ConfirmationDialogActionEvent, icon_button,
    save_write_failure_notice, spawn_confirmation_modal, standard_button,
};
use crate::ui::theme;

/// Teacher stats tab showing per-student and per-lesson score breakdowns.
pub struct TeacherStatsScreenPlugin;

impl Plugin for TeacherStatsScreenPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(
            Update,
            rebuild_stats_ui
                .run_if(in_state(AppState::ThemeExploration))
                .run_if(teacher_window_exists),
        )
        .add_systems(
            Update,
            (handle_return_to_list, handle_reset_click)
                .run_if(in_state(AppState::ThemeExploration))
                .run_if(teacher_stats_view_active),
        )
        .add_systems(
            OnExit(AppState::ThemeExploration),
            cleanup_root::<TeacherStatsRoot>,
        );
    }
}

#[derive(Component, Reflect)]
pub struct TeacherStatsRoot;

#[derive(Component, Reflect)]
struct ReturnToListButton;

/// Marker for any reset button (global, lesson, or type level).
#[derive(Component, Clone, Debug, Reflect)]
struct StatsResetButton(StatsResetTarget);

/// Marker for the reset confirmation popover.
#[derive(Component, Reflect)]
struct StatsResetPopover;

#[derive(Component, Reflect)]
struct StatsResetRequest {
    student_index: usize,
    target: StatsResetTarget,
}

/// What to reset when the confirmation is accepted.
#[derive(Component, Clone, Debug, Reflect)]
enum StatsResetTarget {
    /// Reset all stats for the student.
    All,
    /// Reset stats for a specific lesson.
    Lesson(String),
    /// Reset stats for a specific question type within a lesson.
    Type(String, QuestionType),
}

/// Builds the stats UI when its view or backing save data changes.
fn rebuild_stats_ui(
    mut commands: Commands,
    ts: TeacherScreenParam<'_, '_>,
    content: Res<ContentLibrary>,
    existing_root: Query<Entity, With<TeacherStatsRoot>>,
    mut write_status: ResMut<PersistenceStatus>,
) {
    let Ok(state) = ts.teacher.state.single() else {
        return;
    };
    if state.is_changed() {
        write_status.clear_save_data();
    }
    if !state.is_changed() && !ts.ctx.save_data.is_changed() && !write_status.is_changed() {
        return;
    }

    for entity in &existing_root {
        commands.entity(entity).despawn();
    }
    let TeacherView::StudentStats { student_index } = &state.view else {
        return;
    };
    let student_index = *student_index;
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
    let Some(student) = class_save.students.get(student_index) else {
        return;
    };

    let has_any_progress = !student.progress.is_empty();
    let active_tab = state.view.tab();

    let tab = tab_header(&ts.i18n, active_tab, window);
    let title_text = ts
        .i18n
        .t(&TranslationKey::StudentStats(student.name.clone()))
        .into_owned();
    let no_lessons_text = ts.i18n.t(&TranslationKey::NoLessonsCompleted).into_owned();
    let return_text = ts.i18n.t(&TranslationKey::Back).into_owned();
    let (stats_data, global_total) = precompute_stats(&student.progress, &content, &ts.i18n);

    spawn_stats_root(
        &mut commands,
        camera_entity,
        window,
        tab,
        StatsViewData {
            title_text,
            no_lessons_text,
            return_text,
            has_any_progress,
            stats_data,
            global_total,
            save_failure_notice: write_status
                .failed(PersistenceAction::TeacherStatsReset)
                .then(|| ts.i18n.t(&TranslationKey::SaveWriteFailed).into_owned()),
        },
    );
}

struct StatsViewData {
    title_text: String,
    no_lessons_text: String,
    return_text: String,
    has_any_progress: bool,
    stats_data: Vec<ThemeStatsData>,
    global_total: GlobalTotal,
    save_failure_notice: Option<String>,
}

fn spawn_stats_root(
    commands: &mut Commands,
    camera_entity: Entity,
    window: Entity,
    tab: impl Bundle,
    data: StatsViewData,
) {
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
        TeacherStatsRoot,
        Children::spawn(SpawnWith(move |parent: &mut ChildSpawner| {
            parent.spawn(tab);

            spawn_stats_title_row(parent, &data.title_text, window);
            if let Some(notice) = &data.save_failure_notice {
                parent.spawn(save_write_failure_notice(notice.clone(), window));
            }

            parent.spawn((
                Node {
                    flex_direction: FlexDirection::Column,
                    flex_grow: 1.0,
                    row_gap: theme::scaled(theme::spacing::MEDIUM),
                    ..default()
                },
                Children::spawn(SpawnWith(move |content: &mut ChildSpawner| {
                    if data.has_any_progress {
                        spawn_stats_frame(content, data.stats_data, window);
                        spawn_global_total(content, &data.global_total, window);
                    } else {
                        content.spawn((
                            Text::new(data.no_lessons_text),
                            theme::typography::text(theme::fonts::BODY, window),
                            TextColor(theme::colors::TEXT_MUTED),
                        ));
                    }
                })),
            ));

            parent.spawn((
                standard_button(
                    &data.return_text,
                    theme::colors::PRIMARY,
                    theme::scaled(theme::sizes::BUTTON_WIDTH),
                    window,
                ),
                ReturnToListButton,
            ));
        })),
    ));
}

fn spawn_stats_title_row(parent: &mut ChildSpawner, title_text: &str, window: Entity) {
    parent.spawn((
        Node {
            flex_direction: FlexDirection::Row,
            justify_content: JustifyContent::SpaceBetween,
            align_items: AlignItems::Center,
            ..default()
        },
        children![
            (
                Text::new(title_text),
                theme::typography::text(theme::fonts::HEADING, window),
                TextColor(theme::colors::TEXT_DARK),
            ),
            (
                reset_icon_button(window),
                StatsResetButton(StatsResetTarget::All),
            ),
        ],
    ));
}

/// Pre-computed data for a single question type score row.
struct TypeScoreRow {
    type_label: String,
    score_text: String,
    score_color: Color,
    lesson_id: String,
    question_type: QuestionType,
}

/// Pre-computed data for a lesson section.
struct LessonStatsData {
    lesson_name: String,
    lesson_id: String,
    total_label: String,
    total_score: String,
    total_color: Color,
    type_rows: Vec<TypeScoreRow>,
}

/// Pre-computed data for a theme section.
struct ThemeStatsData {
    theme_title: String,
    lessons: Vec<LessonStatsData>,
}

/// Pre-computed global total across all lessons.
struct GlobalTotal {
    label: String,
    score: String,
    score_color: Color,
}

const fn pct_color(pct: u32) -> Color {
    if pct >= 80 {
        theme::colors::SUCCESS
    } else if pct >= 50 {
        theme::colors::PRIMARY
    } else {
        theme::colors::ERROR
    }
}

/// Builds the type score rows for a single lesson, sorted by question type name.
fn collect_lesson_type_rows(
    lesson_id: &str,
    progress: &LessonProgress,
    i18n: &I18n,
) -> Vec<TypeScoreRow> {
    let mut types: Vec<_> = progress.type_scores.iter().collect();
    types.sort_by_key(|(qt, _)| format!("{qt:?}"));
    types
        .into_iter()
        .filter(|(_, ts)| ts.total > 0)
        .map(|(qt, ts)| {
            let pct = ts.percentage();
            TypeScoreRow {
                type_label: question_type_label(*qt, i18n),
                score_text: format!("{}/{} {} %", ts.correct, ts.total, pct),
                score_color: pct_color(pct),
                lesson_id: lesson_id.to_owned(),
                question_type: *qt,
            }
        })
        .collect()
}

fn precompute_stats(
    progress: &HashMap<String, LessonProgress>,
    content: &ContentLibrary,
    i18n: &I18n,
) -> (Vec<ThemeStatsData>, GlobalTotal) {
    let total_text = i18n.t(&TranslationKey::Total).into_owned();
    let mut result = Vec::new();
    let mut grand_correct: u32 = 0;
    let mut grand_total: u32 = 0;

    for theme_data in &content.themes {
        if !theme_data.available {
            continue;
        }
        let has_data = theme_data
            .lessons
            .iter()
            .any(|l| l.available && progress.contains_key(&l.id));
        if !has_data {
            continue;
        }

        let mut lessons = Vec::new();
        for lesson in &theme_data.lessons {
            if !lesson.available {
                continue;
            }
            let Some(lp) = progress.get(&lesson.id) else {
                continue;
            };

            let lesson_correct = lp.total_correct();
            let lesson_total = lp.total_questions();
            let lesson_pct = lp.percentage();
            grand_correct += lesson_correct;
            grand_total += lesson_total;

            lessons.push(LessonStatsData {
                lesson_name: i18n.t(&lesson.title_key).into_owned(),
                lesson_id: lesson.id.clone(),
                total_label: total_text.clone(),
                total_score: format!("{lesson_correct}/{lesson_total} {lesson_pct} %"),
                total_color: pct_color(lesson_pct),
                type_rows: collect_lesson_type_rows(&lesson.id, lp, i18n),
            });
        }

        result.push(ThemeStatsData {
            theme_title: i18n.t(&theme_data.title_key).into_owned(),
            lessons,
        });
    }

    let grand_pct = (grand_correct * 100).checked_div(grand_total).unwrap_or(0);
    let global_total = GlobalTotal {
        label: match i18n.language {
            Language::French => format!("{total_text} score : "),
            Language::English => format!("{total_text} score: "),
        },
        score: format!("{grand_correct}/{grand_total}"),
        score_color: pct_color(grand_pct),
    };

    (result, global_total)
}

/// Small "X" button used for reset actions, same style as the student remove button.
fn reset_icon_button(window: Entity) -> impl Bundle {
    icon_button(
        28.0,
        4.0,
        "X",
        theme::fonts::SMALL,
        theme::colors::TOGGLE_INACTIVE,
        theme::colors::TEXT_DARK,
        window,
    )
}

fn spawn_stats_frame(parent: &mut ChildSpawner, data: Vec<ThemeStatsData>, window: Entity) {
    // Scrollable rounded frame (same style as lesson config view)
    parent.spawn((
        Node {
            flex_direction: FlexDirection::Column,
            row_gap: theme::scaled(theme::spacing::MEDIUM),
            flex_grow: 1.0,
            padding: theme::scaled(theme::spacing::MEDIUM).all(),
            border: UiRect::all(px(1.0)),
            border_radius: BorderRadius::all(theme::scaled(8.0)),
            overflow: Overflow::scroll_y(),
            ..default()
        },
        BackgroundColor(theme::colors::CARD_BG),
        BorderColor::all(theme::colors::TEXT_MUTED),
        Children::spawn(SpawnWith(move |list: &mut ChildSpawner| {
            for theme_section in &data {
                // Theme header (bold)
                list.spawn((
                    Text::new(theme_section.theme_title.clone()),
                    theme::typography::text(theme::fonts::BODY, window),
                    TextColor(theme::colors::TEXT_DARK),
                ));

                for lesson in &theme_section.lessons {
                    spawn_lesson_section(list, lesson, window);
                }
            }
        })),
    ));
}

fn spawn_lesson_section(parent: &mut ChildSpawner, lesson: &LessonStatsData, window: Entity) {
    parent.spawn((
        Node {
            flex_direction: FlexDirection::Column,
            row_gap: theme::scaled(theme::spacing::SMALL),
            padding: theme::scaled(theme::spacing::SMALL).left(),
            ..default()
        },
        Children::spawn(SpawnWith({
            let lesson_name = lesson.lesson_name.clone();
            let lesson_id = lesson.lesson_id.clone();
            let total_label = lesson.total_label.clone();
            let total_score = lesson.total_score.clone();
            let total_color = lesson.total_color;
            let type_rows_data: Vec<_> = lesson
                .type_rows
                .iter()
                .map(|r| {
                    (
                        r.type_label.clone(),
                        r.score_text.clone(),
                        r.score_color,
                        r.lesson_id.clone(),
                        r.question_type,
                    )
                })
                .collect();
            move |section: &mut ChildSpawner| {
                spawn_lesson_header(section, &lesson_name, &lesson_id, window);

                for (type_label, score_text, color, lid, qt) in &type_rows_data {
                    spawn_type_row(section, type_label, score_text, *color, lid, *qt, window);
                }
                spawn_lesson_total(section, &total_label, &total_score, total_color, window);
            }
        })),
    ));
}

fn spawn_lesson_header(
    parent: &mut ChildSpawner,
    lesson_name: &str,
    lesson_id: &str,
    window: Entity,
) {
    let lesson_name = lesson_name.to_owned();
    let lesson_id = lesson_id.to_owned();
    parent.spawn((
        Node {
            flex_direction: FlexDirection::Row,
            justify_content: JustifyContent::SpaceBetween,
            align_items: AlignItems::Center,
            ..default()
        },
        children![
            (
                Text::new(lesson_name),
                theme::typography::text(theme::fonts::BODY, window),
                TextColor(theme::colors::TEXT_DARK),
            ),
            (
                reset_icon_button(window),
                StatsResetButton(StatsResetTarget::Lesson(lesson_id)),
            ),
        ],
    ));
}

fn spawn_lesson_total(
    parent: &mut ChildSpawner,
    label: &str,
    score: &str,
    color: Color,
    window: Entity,
) {
    parent.spawn((
        Node {
            flex_direction: FlexDirection::Row,
            justify_content: JustifyContent::SpaceBetween,
            align_items: AlignItems::Center,
            padding: theme::scaled(theme::spacing::MEDIUM).left(),
            ..default()
        },
        children![
            (
                Text::new(label.to_owned()),
                theme::typography::text(theme::fonts::SMALL, window),
                TextColor(theme::colors::TEXT_DARK),
            ),
            (
                Text::new(score.to_owned()),
                theme::typography::text(theme::fonts::SMALL, window),
                TextColor(color),
            ),
        ],
    ));
}

fn spawn_global_total(parent: &mut ChildSpawner, total: &GlobalTotal, window: Entity) {
    parent
        .spawn((
            Text::new(total.label.clone()),
            theme::typography::text(theme::fonts::BODY, window),
            TextColor(theme::colors::TEXT_DARK),
        ))
        .with_child((
            TextSpan::new(total.score.clone()),
            theme::typography::text(theme::fonts::BODY, window),
            TextColor(total.score_color),
        ));
}

fn spawn_type_row(
    parent: &mut ChildSpawner,
    type_label: &str,
    score_text: &str,
    score_color: Color,
    lesson_id: &str,
    qt: QuestionType,
    window: Entity,
) {
    let type_label = type_label.to_owned();
    let score_text = score_text.to_owned();
    let lesson_id = lesson_id.to_owned();

    parent.spawn((
        Node {
            flex_direction: FlexDirection::Row,
            justify_content: JustifyContent::SpaceBetween,
            align_items: AlignItems::Center,
            padding: theme::scaled(theme::spacing::MEDIUM).left(),
            ..default()
        },
        Children::spawn(SpawnWith(move |row: &mut ChildSpawner| {
            // Type label
            row.spawn((
                Text::new(type_label),
                theme::typography::text(theme::fonts::SMALL, window),
                TextColor(theme::colors::TEXT_DARK),
                Node {
                    flex_grow: 1.0,
                    ..default()
                },
            ));
            // Score (colored by percentage)
            row.spawn((
                Text::new(score_text),
                theme::typography::text(theme::fonts::SMALL, window),
                TextColor(score_color),
                Node {
                    margin: UiRect::right(theme::scaled(theme::spacing::MEDIUM)),
                    ..default()
                },
            ));
            // Reset button for this type
            row.spawn((
                reset_icon_button(window),
                StatsResetButton(StatsResetTarget::Type(lesson_id, qt)),
            ));
        })),
    ));
}

fn handle_reset_click(
    query: Query<(&Interaction, &StatsResetButton), Changed<Interaction>>,
    mut commands: Commands,
    existing_popover: Query<Entity, With<StatsResetPopover>>,
    i18n: Res<I18n>,
    ctx: PlayerContext<'_>,
    teacher: TeacherWindowParam<'_, '_>,
    teacher_state: Query<&TeacherWindowState, With<TeacherWindow>>,
) {
    let Some(ref session) = ctx.session else {
        return;
    };
    let Some(ref class_save) = ctx.save_data.class_slots[session.slot_index] else {
        return;
    };
    let Ok(state) = teacher_state.single() else {
        return;
    };
    let TeacherView::StudentStats { student_index } = &state.view else {
        return;
    };
    let student_index = *student_index;
    if class_save.students.get(student_index).is_none() {
        return;
    }
    let Ok(window) = teacher.window.single() else {
        return;
    };
    let Ok(camera) = teacher.camera.single() else {
        return;
    };

    for (interaction, reset_btn) in &query {
        if *interaction != Interaction::Pressed {
            continue;
        }

        // Despawn any existing popover
        for entity in &existing_popover {
            commands.entity(entity).try_despawn();
        }

        let confirm_message = match &reset_btn.0 {
            StatsResetTarget::All => i18n.t(&TranslationKey::ResetAllStatsConfirm).into_owned(),
            StatsResetTarget::Lesson(lid) => i18n
                .t(&TranslationKey::ResetLessonStatsConfirm(lid.clone()))
                .into_owned(),
            StatsResetTarget::Type(lid, qt) => i18n
                .t(&TranslationKey::ResetTypeStatsConfirm(
                    lid.clone(),
                    question_type_label(*qt, &i18n),
                ))
                .into_owned(),
        };

        let modal_entity = spawn_confirmation_modal(
            &mut commands,
            &confirm_message,
            &i18n.t(&TranslationKey::Delete),
            &i18n.t(&TranslationKey::Cancel),
            theme::colors::ERROR,
            window,
            Some(camera),
        );
        commands
            .entity(modal_entity)
            .insert((
                StatsResetPopover,
                TeacherViewOverlay,
                StatsResetRequest {
                    student_index,
                    target: reset_btn.0.clone(),
                },
                DespawnOnExit(AppState::ThemeExploration),
            ))
            .observe(handle_confirm_reset);
    }
}

fn handle_confirm_reset(
    event: On<ConfirmationDialogActionEvent>,
    target_query: Query<&StatsResetRequest>,
    session: Option<Res<PlayerSession>>,
    mut persistence: SaveDataMut<'_>,
) {
    if event.action != ConfirmationDialogAction::Confirm {
        return;
    }
    let Some(ref session) = session else { return };
    let Ok(request) = target_query.get(event.entity) else {
        return;
    };

    let student_index = request.student_index;
    let slot_index = session.slot_index;
    let target = request.target.clone();

    persistence.update(PersistenceAction::TeacherStatsReset, |data| {
        let Some(class_save) = data.class_slots[slot_index].as_mut() else {
            return;
        };
        let Some(student) = class_save.students.get_mut(student_index) else {
            return;
        };
        match target {
            StatsResetTarget::All => {
                student.progress.clear();
            }
            StatsResetTarget::Lesson(ref lid) => {
                student.progress.remove(lid);
            }
            StatsResetTarget::Type(ref lid, qt) => {
                if let Some(lp) = student.progress.get_mut(lid) {
                    lp.type_scores.remove(&qt);
                    // If no types left, remove the lesson entry entirely
                    if lp.type_scores.is_empty() {
                        student.progress.remove(lid);
                    }
                }
            }
        }
    });
}

fn handle_return_to_list(
    query: Query<&Interaction, (Changed<Interaction>, With<ReturnToListButton>)>,
    mut states: Query<&mut TeacherWindowState, With<TeacherWindow>>,
) {
    if !query
        .iter()
        .any(|interaction| *interaction == Interaction::Pressed)
    {
        return;
    }
    if let Ok(mut state) = states.single_mut() {
        state.view = TeacherView::Students;
    }
}
