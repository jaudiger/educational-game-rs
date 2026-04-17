use std::collections::HashMap;

use bevy::input_focus::tab_navigation::TabGroup;
use bevy::prelude::*;
use bevy_persistent::prelude::Persistent;

use crate::data::content::QuestionType;
use crate::data::{
    ActiveSlot, ContentLibrary, GameMode, Language, LessonProgress, PlayerContext, SaveData,
};
use crate::i18n::{I18n, TranslationKey};
use crate::plugins::teacher::{
    TeacherContentRoot, TeacherInDetailView, TeacherScreenParam, TeacherTab, TeacherWindowParam,
    tab_header,
};
use crate::screens::teacher_shared::{
    RebuildRoster, RebuildStats, ViewingStudentStats, question_type_label,
};
use crate::states::{AppState, StateScopedResourceExt, cleanup_root};
use crate::ui::components::{
    PopoverCancelButton, PopoverConfirmButton, icon_button, spawn_confirmation_modal,
    standard_button,
};
use crate::ui::theme;
use crate::ui::theme::DesignFontSize;

/// Teacher stats tab showing per-student and per-lesson score breakdowns.
pub struct TeacherStatsScreenPlugin;

impl Plugin for TeacherStatsScreenPlugin {
    fn build(&self, app: &mut App) {
        app.register_state_scoped_resource::<AppState, ViewingStudentStats>(
            AppState::MapExploration,
        )
        .add_observer(on_rebuild_stats)
        .add_systems(
            Update,
            (
                handle_return_to_list,
                handle_reset_click,
                handle_confirm_reset,
                handle_cancel_reset,
            )
                .run_if(in_state(AppState::MapExploration))
                .run_if(resource_exists::<ViewingStudentStats>),
        )
        .add_systems(
            OnExit(AppState::MapExploration),
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

/// Builds the stats UI when triggered via [`RebuildStats`].
/// Called from the roster's double-click handler.
fn on_rebuild_stats(
    _event: On<RebuildStats>,
    mut commands: Commands,
    viewing: Option<Res<ViewingStudentStats>>,
    ts: TeacherScreenParam<'_, '_>,
    content: Res<ContentLibrary>,
    existing_root: Query<Entity, With<TeacherStatsRoot>>,
) {
    for entity in &existing_root {
        commands.entity(entity).despawn();
    }
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
    let Some(ref viewing) = viewing else { return };
    let camera_entity = *ts.teacher.camera;
    let window = *ts.teacher.window;
    let Some(ref slot) = ts.ctx.active_slot else {
        return;
    };
    let Some(ref class_save) = ts.ctx.save_data.class_slots[slot.0] else {
        return;
    };
    let Some(student) = class_save.students.get(viewing.0) else {
        return;
    };

    let has_any_progress = !student.progress.is_empty();
    let active_tab = ts.teacher_tab.map_or(TeacherTab::Students, |t| *t);

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
        TeacherContentRoot,
        Children::spawn(SpawnWith(move |parent: &mut ChildSpawner| {
            parent.spawn(tab);

            spawn_stats_title_row(parent, &data.title_text, window);

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
                Text::new(title_text.to_owned()),
                TextFont {
                    font_size: theme::fonts::HEADING,
                    ..default()
                },
                TextColor(theme::colors::TEXT_DARK),
                DesignFontSize {
                    size: theme::fonts::HEADING,
                    window,
                },
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
                score_text: format!("{}/{}", ts.correct, ts.total),
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

            grand_correct += lp.total_correct();
            grand_total += lp.total_questions();

            lessons.push(LessonStatsData {
                lesson_name: i18n.t(&lesson.title_key).into_owned(),
                lesson_id: lesson.id.clone(),
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
            (
                reset_icon_button(window),
                StatsResetButton(StatsResetTarget::Lesson(lesson_id)),
            ),
        ],
    ));
}

fn spawn_global_total(parent: &mut ChildSpawner, total: &GlobalTotal, window: Entity) {
    parent
        .spawn((
            Text::new(total.label.clone()),
            TextFont {
                font_size: theme::fonts::BODY,
                ..default()
            },
            TextColor(theme::colors::TEXT_DARK),
            DesignFontSize {
                size: theme::fonts::BODY,
                window,
            },
        ))
        .with_child((
            TextSpan::new(total.score.clone()),
            TextFont {
                font_size: theme::fonts::BODY,
                ..default()
            },
            TextColor(total.score_color),
            DesignFontSize {
                size: theme::fonts::BODY,
                window,
            },
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
                TextFont {
                    font_size: theme::fonts::SMALL,
                    ..default()
                },
                TextColor(theme::colors::TEXT_DARK),
                Node {
                    flex_grow: 1.0,
                    ..default()
                },
                DesignFontSize {
                    size: theme::fonts::SMALL,
                    window,
                },
            ));
            // Score (colored by percentage)
            row.spawn((
                Text::new(score_text),
                TextFont {
                    font_size: theme::fonts::SMALL,
                    ..default()
                },
                TextColor(score_color),
                Node {
                    margin: UiRect::right(theme::scaled(theme::spacing::MEDIUM)),
                    ..default()
                },
                DesignFontSize {
                    size: theme::fonts::SMALL,
                    window,
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
    viewing: Res<ViewingStudentStats>,
    ctx: PlayerContext<'_>,
    teacher: TeacherWindowParam<'_, '_>,
) {
    let Some(ref slot) = ctx.active_slot else {
        return;
    };
    let Some(ref class_save) = ctx.save_data.class_slots[slot.0] else {
        return;
    };
    if class_save.students.get(viewing.0).is_none() {
        return;
    }
    let window = *teacher.window;

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
            Some(*teacher.camera),
        );
        commands
            .entity(modal_entity)
            .insert((StatsResetPopover, reset_btn.0.clone()));
    }
}

fn handle_confirm_reset(
    query: Query<&Interaction, (Changed<Interaction>, With<PopoverConfirmButton>)>,
    popover: Query<(Entity, &StatsResetTarget), With<StatsResetPopover>>,
    viewing: Res<ViewingStudentStats>,
    active_slot: Option<Res<ActiveSlot>>,
    mut save_data: ResMut<Persistent<SaveData>>,
    mut commands: Commands,
) {
    let Some(ref slot) = active_slot else { return };
    let Ok((popover_entity, target)) = popover.single() else {
        return;
    };

    for interaction in &query {
        if *interaction != Interaction::Pressed {
            continue;
        }

        let student_index = viewing.0;
        let slot_index = slot.0;
        let target = target.clone();

        let _ = save_data.update(|data| {
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

        commands.entity(popover_entity).try_despawn();
        commands.trigger(RebuildStats);
    }
}

fn handle_cancel_reset(
    query: Query<&Interaction, (Changed<Interaction>, With<PopoverCancelButton>)>,
    mut commands: Commands,
    popover_query: Query<Entity, With<StatsResetPopover>>,
) {
    for interaction in &query {
        if *interaction == Interaction::Pressed {
            for entity in &popover_query {
                commands.entity(entity).try_despawn();
            }
        }
    }
}

fn handle_return_to_list(
    query: Query<&Interaction, (Changed<Interaction>, With<ReturnToListButton>)>,
    mut commands: Commands,
) {
    for interaction in &query {
        if *interaction == Interaction::Pressed {
            commands.remove_resource::<ViewingStudentStats>();
            commands.remove_resource::<TeacherInDetailView>();
            commands.trigger(RebuildStats);
            commands.trigger(RebuildRoster);
        }
    }
}
