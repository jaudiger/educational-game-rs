use std::collections::{HashMap, HashSet};

use bevy::input_focus::tab_navigation::TabGroup;
use bevy::prelude::*;

use crate::data::content::{Lesson, QuestionType};
use crate::data::save::score_percentage;
use crate::data::{
    ContentLibrary, GameMode, Language, LessonProgress, PersistenceAction, PersistenceStatus,
    PlayerContext, PlayerSession, SaveDataMut,
};
use crate::i18n::{I18n, TranslationKey};
use crate::plugins::teacher::{
    TeacherScreenParam, TeacherView, TeacherViewOverlay, TeacherWindow, TeacherWindowParam,
    TeacherWindowState, tab_header, teacher_stats_view_active, teacher_window_exists,
};
use crate::screens::teacher_shared::exercise_type_label;
use crate::states::{AppState, cleanup_root};
use crate::ui::components::{
    ButtonActivated, ConfirmationDialogAction, ConfirmationDialogActionEvent, TeacherListLevel,
    button_base, disclosure_chevron, disclosure_chevron_transform, icon_button,
    persistence_message, save_write_failure_notice, spawn_confirmation_modal, standard_button,
    teacher_list_content_node, teacher_list_exercise_row, teacher_list_frame,
    teacher_list_header_border, teacher_list_header_button_node, teacher_list_header_node,
    teacher_list_indent, teacher_list_label, teacher_list_separator_gap,
};
use crate::ui::theme;

/// Teacher stats tab showing per-student and per-lesson score breakdowns.
pub struct TeacherStatsScreenPlugin;

impl Plugin for TeacherStatsScreenPlugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<StatsDisclosureState>()
            .add_systems(
                Update,
                rebuild_stats_ui
                    .run_if(in_state(AppState::ThemeExploration))
                    .run_if(teacher_window_exists),
            )
            .add_systems(
                Update,
                (
                    handle_return_to_list,
                    handle_reset_click,
                    handle_stats_disclosure,
                )
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

#[derive(Component, Reflect)]
struct TeacherStatsFrame;

#[derive(Component, Clone, Reflect)]
struct StatsDisclosureButton(StatsDisclosureTarget);

#[derive(Component, Clone, Reflect)]
struct StatsDisclosurePanel(StatsDisclosureTarget);

#[derive(Component, Clone, Reflect)]
struct StatsDisclosureChevron(StatsDisclosureTarget);

#[derive(Clone, Debug, PartialEq, Eq, Hash, Reflect)]
enum StatsDisclosureTarget {
    Theme(String),
    Lesson(String),
    Type(String, QuestionType),
}

#[derive(Resource, Default)]
struct StatsDisclosureState {
    expanded: HashSet<StatsDisclosureTarget>,
    collapsed_themes: HashSet<String>,
}

struct StatsDisclosureRowData {
    target: StatsDisclosureTarget,
    label: String,
    score_text: String,
    score_color: Color,
    label_level: TeacherListLevel,
    expanded: bool,
}

impl StatsDisclosureState {
    fn is_expanded(&self, target: &StatsDisclosureTarget) -> bool {
        match target {
            StatsDisclosureTarget::Theme(theme_id) => !self.collapsed_themes.contains(theme_id),
            StatsDisclosureTarget::Lesson(_) | StatsDisclosureTarget::Type(_, _) => {
                self.expanded.contains(target)
            }
        }
    }

    fn toggle(&mut self, target: StatsDisclosureTarget) -> bool {
        if let StatsDisclosureTarget::Theme(theme_id) = target {
            if self.collapsed_themes.remove(&theme_id) {
                return true;
            }
            self.collapsed_themes.insert(theme_id);
            return false;
        }
        if self.expanded.remove(&target) {
            false
        } else {
            self.expanded.insert(target);
            true
        }
    }
}

/// Marker for any stats reset button.
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
    /// Reset stats for a specific theme.
    Theme(String),
    /// Reset stats for a specific lesson.
    Lesson(String),
    /// Reset stats for a specific exercise type within a lesson.
    Type(String, QuestionType),
    /// Reset stats for one exercise within a lesson.
    Exercise(String, String),
}

/// Builds the stats UI when its view or backing save data changes.
fn rebuild_stats_ui(
    mut commands: Commands,
    ts: TeacherScreenParam<'_, '_>,
    content: Res<ContentLibrary>,
    existing_root: Query<Entity, With<TeacherStatsRoot>>,
    existing_frame: Query<&ScrollPosition, With<TeacherStatsFrame>>,
    mut disclosures: ResMut<StatsDisclosureState>,
    mut write_status: ResMut<PersistenceStatus>,
) {
    let Ok(state) = ts.teacher.state.single() else {
        return;
    };
    if state.is_changed() {
        write_status.clear_save_data();
        disclosures.expanded.clear();
        disclosures.collapsed_themes.clear();
    }
    if !state.is_changed() && !ts.ctx.save_data.is_changed() && !write_status.is_changed() {
        return;
    }

    let scroll_position = existing_frame
        .single()
        .map_or_else(|_| ScrollPosition::default(), Clone::clone);
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

    let active_tab = state.view.tab();

    let tab = tab_header(&ts.i18n, active_tab, window);
    let title_text = ts
        .i18n
        .t(&TranslationKey::StudentStats(student.name.clone()))
        .into_owned();
    let no_lessons_text = ts.i18n.t(&TranslationKey::NoLessonsCompleted).into_owned();
    let return_text = ts.i18n.t(&TranslationKey::Back).into_owned();
    let stats_data = precompute_stats(&student.progress, &content, &ts.i18n, &disclosures);

    spawn_stats_root(
        &mut commands,
        camera_entity,
        window,
        tab,
        StatsViewData {
            title_text,
            no_lessons_text,
            return_text,
            stats_data,
            scroll_position,
            save_failure_notice: persistence_message(
                PersistenceAction::TeacherStatsReset,
                *write_status,
                &ts.i18n,
            ),
        },
    );
}

struct StatsViewData {
    title_text: String,
    no_lessons_text: String,
    return_text: String,
    stats_data: Vec<ThemeStatsData>,
    scroll_position: ScrollPosition,
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
                    flex_basis: px(0.0),
                    min_height: px(0.0),
                    row_gap: theme::scaled(theme::spacing::MEDIUM),
                    ..default()
                },
                Children::spawn(SpawnWith(move |content: &mut ChildSpawner| {
                    if data.stats_data.is_empty() {
                        content.spawn((
                            Text::new(data.no_lessons_text),
                            theme::typography::text(theme::fonts::BODY, window),
                            TextColor(theme::colors::TEXT_MUTED),
                        ));
                    } else {
                        spawn_stats_frame(content, data.stats_data, data.scroll_position, window);
                    }
                })),
            ));

            parent.spawn((
                standard_button(
                    &data.return_text,
                    theme::button_styles::BACK,
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

/// Pre-computed data for a single exercise score row.
#[derive(Clone)]
struct ExerciseScoreRow {
    exercise_label: String,
    score_text: String,
    score_color: Color,
    lesson_id: String,
    exercise_id: String,
}

#[derive(Clone, Copy, Default, Debug, PartialEq, Eq)]
struct ScoreTotals {
    correct: u128,
    total: u128,
}

impl ScoreTotals {
    const fn add(&mut self, other: Self) {
        self.correct = self.correct.saturating_add(other.correct);
        self.total = self.total.saturating_add(other.total);
    }

    fn from_progress(progress: Option<&LessonProgress>) -> Self {
        progress.map_or_else(Self::default, |progress| {
            progress
                .question_scores
                .values()
                .fold(Self::default(), |mut totals, score| {
                    totals.correct = totals
                        .correct
                        .saturating_add(u128::from(score.correct_count()));
                    totals.total = totals.total.saturating_add(u128::from(score.total_count()));
                    totals
                })
        })
    }

    fn display(self) -> (String, Color) {
        if self.total == 0 {
            return (
                format!("{}/{}", self.correct, self.total),
                theme::colors::TEXT_MUTED,
            );
        }
        let percentage = score_percentage(self.correct, self.total);
        (
            format!("{}/{} {} %", self.correct, self.total, percentage),
            pct_color(percentage),
        )
    }
}

#[derive(Clone, Copy, Default)]
struct DerivedTypeScore {
    totals: ScoreTotals,
}

/// Pre-computed data for a single exercise type score row.
#[derive(Clone)]
struct TypeScoreRow {
    type_label: String,
    score_text: String,
    score_color: Color,
    lesson_id: String,
    exercise_type: QuestionType,
    details_expanded: bool,
    exercise_rows: Vec<ExerciseScoreRow>,
}

/// Pre-computed data for a lesson section.
#[derive(Clone)]
struct LessonStatsData {
    lesson_name: String,
    lesson_id: String,
    total_label: String,
    total_score: String,
    total_color: Color,
    details_expanded: bool,
    type_rows: Vec<TypeScoreRow>,
    unclassified_rows: Vec<ExerciseScoreRow>,
}

/// Pre-computed data for a theme section.
struct ThemeStatsData {
    theme_title: String,
    theme_id: String,
    expanded: bool,
    content: ThemeStatsContent,
}

enum ThemeStatsContent {
    Available {
        total_label: String,
        total_score: String,
        total_color: Color,
        lessons: Vec<LessonStatsData>,
    },
    Unavailable {
        status_label: String,
    },
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

fn score_row(
    lesson_id: &str,
    exercise_id: &str,
    score: Option<&crate::data::save::QuestionScore>,
    content: &ContentLibrary,
    language: Language,
) -> ExerciseScoreRow {
    let totals = score.map_or_else(ScoreTotals::default, |score| ScoreTotals {
        correct: u128::from(score.correct_count()),
        total: u128::from(score.total_count()),
    });
    let (score_text, score_color) = totals.display();
    ExerciseScoreRow {
        exercise_label: exercise_label(lesson_id, exercise_id, content, language),
        score_text,
        score_color,
        lesson_id: lesson_id.to_owned(),
        exercise_id: exercise_id.to_owned(),
    }
}

/// Derives the exercise category from each catalog entry in the lesson.
fn aggregate_type_scores(
    lesson: &Lesson,
    progress: Option<&LessonProgress>,
) -> HashMap<QuestionType, DerivedTypeScore> {
    let mut totals: HashMap<QuestionType, DerivedTypeScore> = HashMap::new();
    for exercise in &lesson.questions {
        let exercise_type = exercise.definition.question_type();
        let score = progress.and_then(|progress| progress.question_scores.get(&exercise.id));
        let row_totals = score.map_or_else(ScoreTotals::default, |score| ScoreTotals {
            correct: u128::from(score.correct_count()),
            total: u128::from(score.total_count()),
        });
        totals
            .entry(exercise_type)
            .or_default()
            .totals
            .add(row_totals);
    }
    totals
}

fn exercise_type_for_id(lesson: &Lesson, exercise_id: &str) -> Option<QuestionType> {
    lesson
        .questions
        .iter()
        .find(|exercise| exercise.id == exercise_id)
        .map(|exercise| exercise.definition.question_type())
}

fn exercise_label(
    lesson_id: &str,
    exercise_id: &str,
    content: &ContentLibrary,
    language: Language,
) -> String {
    content
        .themes
        .iter()
        .flat_map(|theme| &theme.lessons)
        .find(|lesson| lesson.id == lesson_id)
        .and_then(|lesson| {
            lesson
                .questions
                .iter()
                .find(|exercise| exercise.id == exercise_id)
        })
        .map_or_else(
            || exercise_id.to_owned(),
            |exercise| exercise.definition.prompt_label(language),
        )
}

fn lesson_label(lesson_id: &str, content: &ContentLibrary, i18n: &I18n) -> String {
    content
        .themes
        .iter()
        .flat_map(|theme| &theme.lessons)
        .find(|lesson| lesson.id == lesson_id)
        .map_or_else(
            || lesson_id.to_owned(),
            |lesson| i18n.t(&lesson.title_key).into_owned(),
        )
}

/// Builds exercise rows grouped by type, including unanswered catalog entries.
fn collect_lesson_type_rows(
    lesson: &Lesson,
    progress: Option<&LessonProgress>,
    content: &ContentLibrary,
    i18n: &I18n,
) -> (Vec<TypeScoreRow>, Vec<ExerciseScoreRow>) {
    let totals = aggregate_type_scores(lesson, progress);
    let mut exercise_rows: HashMap<QuestionType, Vec<ExerciseScoreRow>> = HashMap::new();
    for exercise in &lesson.questions {
        let exercise_type = exercise.definition.question_type();
        let score = progress.and_then(|progress| progress.question_scores.get(&exercise.id));
        exercise_rows
            .entry(exercise_type)
            .or_default()
            .push(score_row(
                &lesson.id,
                &exercise.id,
                score,
                content,
                i18n.language,
            ));
    }

    let mut unclassified_rows = Vec::new();
    if let Some(progress) = progress {
        for (exercise_id, score) in &progress.question_scores {
            if exercise_type_for_id(lesson, exercise_id).is_none() {
                unclassified_rows.push(score_row(
                    &lesson.id,
                    exercise_id,
                    Some(score),
                    content,
                    i18n.language,
                ));
            }
        }
    }
    unclassified_rows.sort_by(|left, right| left.exercise_id.cmp(&right.exercise_id));

    let mut types: Vec<_> = totals.into_iter().collect();
    types.sort_by_key(|(exercise_type, _)| format!("{exercise_type:?}"));
    let rows = types
        .into_iter()
        .map(|(exercise_type, total)| {
            let (score_text, score_color) = total.totals.display();
            let mut exercises = exercise_rows.remove(&exercise_type).unwrap_or_default();
            exercises.sort_by(|left, right| left.exercise_id.cmp(&right.exercise_id));
            TypeScoreRow {
                type_label: exercise_type_label(exercise_type, i18n),
                score_text,
                score_color,
                lesson_id: lesson.id.clone(),
                exercise_type,
                details_expanded: false,
                exercise_rows: exercises,
            }
        })
        .collect();
    (rows, unclassified_rows)
}

fn precompute_stats(
    progress: &HashMap<String, LessonProgress>,
    content: &ContentLibrary,
    i18n: &I18n,
    disclosures: &StatsDisclosureState,
) -> Vec<ThemeStatsData> {
    let total_text = i18n.t(&TranslationKey::Total).into_owned();
    let total_label = format!("{total_text}: ");
    content
        .themes
        .iter()
        .map(|theme_data| {
            let theme_content = if theme_data.available {
                let mut theme_totals = ScoreTotals::default();
                let lessons = theme_data
                    .lessons
                    .iter()
                    .map(|lesson| {
                        let lesson_progress = progress.get(&lesson.id);
                        let lesson_totals = ScoreTotals::from_progress(lesson_progress);
                        theme_totals.add(lesson_totals);
                        let (total_score, total_color) = lesson_totals.display();
                        let (mut type_rows, unclassified_rows) =
                            collect_lesson_type_rows(lesson, lesson_progress, content, i18n);
                        for type_row in &mut type_rows {
                            type_row.details_expanded =
                                disclosures.is_expanded(&StatsDisclosureTarget::Type(
                                    lesson.id.clone(),
                                    type_row.exercise_type,
                                ));
                        }
                        LessonStatsData {
                            lesson_name: i18n.t(&lesson.title_key).into_owned(),
                            lesson_id: lesson.id.clone(),
                            total_label: total_label.clone(),
                            total_score,
                            total_color,
                            details_expanded: disclosures
                                .is_expanded(&StatsDisclosureTarget::Lesson(lesson.id.clone())),
                            type_rows,
                            unclassified_rows,
                        }
                    })
                    .collect();
                let (total_score, total_color) = theme_totals.display();
                ThemeStatsContent::Available {
                    total_label: total_label.clone(),
                    total_score,
                    total_color,
                    lessons,
                }
            } else {
                ThemeStatsContent::Unavailable {
                    status_label: i18n.t(&TranslationKey::ComingSoon).into_owned(),
                }
            };
            ThemeStatsData {
                theme_title: i18n.t(&theme_data.title_key).into_owned(),
                theme_id: theme_data.id.clone(),
                expanded: disclosures
                    .is_expanded(&StatsDisclosureTarget::Theme(theme_data.id.clone())),
                content: theme_content,
            }
        })
        .collect()
}

/// Small "X" button used for reset actions, same style as the student remove button.
fn reset_icon_button(window: Entity) -> impl Bundle {
    icon_button(
        28.0,
        4.0,
        "X",
        theme::fonts::SMALL,
        theme::button_styles::NEUTRAL,
        window,
    )
}

fn spawn_stats_frame(
    parent: &mut ChildSpawner,
    data: Vec<ThemeStatsData>,
    scroll_position: ScrollPosition,
    window: Entity,
) {
    parent.spawn((
        teacher_list_frame(),
        TeacherStatsFrame,
        scroll_position,
        Children::spawn(SpawnWith(move |list: &mut ChildSpawner| {
            list.spawn((
                teacher_list_content_node(),
                Children::spawn(SpawnWith(move |content: &mut ChildSpawner| {
                    for theme_section in data {
                        spawn_theme_section(content, theme_section, window);
                    }
                })),
            ));
        })),
    ));
}

fn spawn_theme_section(parent: &mut ChildSpawner, data: ThemeStatsData, window: Entity) {
    let ThemeStatsData {
        theme_title,
        theme_id,
        expanded,
        content,
    } = data;
    let (total_label, total_score, total_color, lessons) = match content {
        ThemeStatsContent::Available {
            total_label,
            total_score,
            total_color,
            lessons,
        } => (total_label, total_score, total_color, lessons),
        ThemeStatsContent::Unavailable { status_label } => {
            spawn_unavailable_theme_header(parent, theme_title, status_label, window);
            return;
        }
    };

    let target = StatsDisclosureTarget::Theme(theme_id.clone());
    let panel_target = target.clone();
    let reset_target = StatsResetTarget::Theme(theme_id);
    let row_data = StatsDisclosureRowData {
        target,
        label: theme_title,
        score_text: format!("{total_label}{total_score}"),
        score_color: total_color,
        label_level: TeacherListLevel::Theme,
        expanded,
    };
    parent.spawn((
        Node {
            width: percent(100.0),
            flex_direction: FlexDirection::Column,
            row_gap: theme::scaled(theme::spacing::SMALL),
            ..default()
        },
        Children::spawn(SpawnWith(move |section: &mut ChildSpawner| {
            section.spawn((
                teacher_list_header_node(),
                teacher_list_header_border(),
                Children::spawn(SpawnWith(move |row: &mut ChildSpawner| {
                    row.spawn(stats_disclosure_button(row_data, window));
                    row.spawn((reset_icon_button(window), StatsResetButton(reset_target)));
                })),
            ));
            section.spawn((
                StatsDisclosurePanel(panel_target),
                Node {
                    display: disclosure_display(expanded),
                    flex_direction: FlexDirection::Column,
                    row_gap: theme::scaled(theme::spacing::SMALL),
                    ..default()
                },
                Children::spawn(SpawnWith(move |details: &mut ChildSpawner| {
                    for lesson in &lessons {
                        spawn_lesson_section(details, lesson, window);
                    }
                })),
            ));
        })),
    ));
}

fn spawn_unavailable_theme_header(
    parent: &mut ChildSpawner,
    title: String,
    status_label: String,
    window: Entity,
) {
    parent.spawn((
        Node {
            justify_content: JustifyContent::SpaceBetween,
            ..teacher_list_header_node()
        },
        teacher_list_header_border(),
        children![
            (
                teacher_list_label(
                    title,
                    TeacherListLevel::Theme,
                    theme::colors::TEXT_MUTED,
                    window,
                ),
                Node {
                    flex_grow: 1.0,
                    ..default()
                },
            ),
            (
                Text::new(status_label),
                theme::typography::text(theme::fonts::SMALL, window),
                TextColor(theme::colors::TEXT_MUTED),
            ),
        ],
    ));
}

fn spawn_lesson_section(parent: &mut ChildSpawner, lesson: &LessonStatsData, window: Entity) {
    let lesson = lesson.clone();
    parent.spawn((
        Node {
            flex_direction: FlexDirection::Column,
            row_gap: theme::scaled(theme::spacing::SMALL),
            padding: teacher_list_indent(TeacherListLevel::Theme, TeacherListLevel::Lesson).left(),
            ..default()
        },
        Children::spawn(SpawnWith(move |section: &mut ChildSpawner| {
            spawn_lesson_header(section, &lesson, window);
            let details_target = StatsDisclosureTarget::Lesson(lesson.lesson_id.clone());
            let details_expanded = lesson.details_expanded;
            let type_rows = lesson.type_rows.clone();
            let unclassified_rows = lesson.unclassified_rows.clone();
            section.spawn((
                StatsDisclosurePanel(details_target),
                Node {
                    display: disclosure_display(details_expanded),
                    flex_direction: FlexDirection::Column,
                    row_gap: theme::scaled(theme::spacing::SMALL),
                    padding: UiRect {
                        left: teacher_list_indent(
                            TeacherListLevel::Lesson,
                            TeacherListLevel::ExerciseType,
                        ),
                        top: teacher_list_separator_gap(),
                        bottom: teacher_list_separator_gap(),
                        ..default()
                    },
                    ..default()
                },
                Children::spawn(SpawnWith(move |details: &mut ChildSpawner| {
                    for type_row in &type_rows {
                        spawn_type_row(details, type_row, window);
                    }
                    for exercise_row in &unclassified_rows {
                        spawn_exercise_row(details, exercise_row, window);
                    }
                })),
            ));
        })),
    ));
}

fn spawn_lesson_header(parent: &mut ChildSpawner, lesson: &LessonStatsData, window: Entity) {
    let lesson_id = lesson.lesson_id.clone();
    let row_data = StatsDisclosureRowData {
        target: StatsDisclosureTarget::Lesson(lesson_id.clone()),
        label: lesson.lesson_name.clone(),
        score_text: format!("{}{}", lesson.total_label, lesson.total_score),
        score_color: lesson.total_color,
        label_level: TeacherListLevel::Lesson,
        expanded: lesson.details_expanded,
    };
    parent.spawn((
        teacher_list_header_node(),
        teacher_list_header_border(),
        Children::spawn(SpawnWith(move |row: &mut ChildSpawner| {
            row.spawn(stats_disclosure_button(row_data, window));
            row.spawn((
                reset_icon_button(window),
                StatsResetButton(StatsResetTarget::Lesson(lesson_id)),
            ));
        })),
    ));
}

fn spawn_type_row(parent: &mut ChildSpawner, type_row: &TypeScoreRow, window: Entity) {
    let lesson_id = type_row.lesson_id.clone();
    let exercise_type = type_row.exercise_type;
    let target = StatsDisclosureTarget::Type(lesson_id.clone(), exercise_type);
    let panel_target = target.clone();
    let row_data = StatsDisclosureRowData {
        target,
        label: type_row.type_label.clone(),
        score_text: type_row.score_text.clone(),
        score_color: type_row.score_color,
        label_level: TeacherListLevel::ExerciseType,
        expanded: type_row.details_expanded,
    };
    let details_expanded = type_row.details_expanded;
    let exercise_rows = type_row.exercise_rows.clone();

    parent.spawn((
        Node {
            flex_direction: FlexDirection::Column,
            ..default()
        },
        Children::spawn(SpawnWith(move |group: &mut ChildSpawner| {
            group.spawn((
                teacher_list_header_node(),
                teacher_list_header_border(),
                Children::spawn(SpawnWith(move |row: &mut ChildSpawner| {
                    row.spawn(stats_disclosure_button(row_data, window));
                    row.spawn((
                        reset_icon_button(window),
                        StatsResetButton(StatsResetTarget::Type(lesson_id, exercise_type)),
                    ));
                })),
            ));
            group.spawn((
                StatsDisclosurePanel(panel_target),
                Node {
                    display: disclosure_display(details_expanded),
                    flex_direction: FlexDirection::Column,
                    row_gap: theme::scaled(theme::spacing::SMALL),
                    padding: UiRect::top(teacher_list_separator_gap()),
                    ..default()
                },
                Children::spawn(SpawnWith(move |details: &mut ChildSpawner| {
                    for exercise_row in &exercise_rows {
                        spawn_exercise_row(details, exercise_row, window);
                    }
                })),
            ));
        })),
    ));
}

fn stats_disclosure_button(data: StatsDisclosureRowData, window: Entity) -> impl Bundle + use<> {
    let chevron_target = data.target.clone();
    (
        button_base(theme::button_styles::DISCLOSURE),
        StatsDisclosureButton(data.target),
        teacher_list_header_button_node(),
        Children::spawn(SpawnWith(move |row: &mut ChildSpawner| {
            row.spawn((
                disclosure_chevron(data.expanded, theme::colors::TEXT_DARK, window),
                StatsDisclosureChevron(chevron_target),
            ));
            row.spawn((
                teacher_list_label(
                    data.label,
                    data.label_level,
                    theme::colors::TEXT_DARK,
                    window,
                ),
                Node {
                    flex_grow: 1.0,
                    min_width: px(0.0),
                    ..default()
                },
            ));
            row.spawn((
                Text::new(data.score_text),
                theme::typography::text(theme::fonts::SMALL, window),
                TextColor(data.score_color),
            ));
        })),
    )
}

const fn disclosure_display(expanded: bool) -> Display {
    if expanded {
        Display::Flex
    } else {
        Display::None
    }
}

fn handle_stats_disclosure(
    mut activations: MessageReader<ButtonActivated>,
    buttons: Query<&StatsDisclosureButton>,
    mut panels: Query<(&StatsDisclosurePanel, &mut Node)>,
    mut chevrons: Query<(&StatsDisclosureChevron, &mut UiTransform)>,
    mut disclosures: ResMut<StatsDisclosureState>,
) {
    for event in activations.read() {
        let Ok(button) = buttons.get(event.0) else {
            continue;
        };
        let target = button.0.clone();
        let expanded = disclosures.toggle(target.clone());
        for (panel, mut node) in &mut panels {
            if panel.0 == target {
                node.display = disclosure_display(expanded);
            }
        }
        for (chevron, mut transform) in &mut chevrons {
            if chevron.0 == target {
                *transform = disclosure_chevron_transform(expanded);
            }
        }
    }
}

fn spawn_exercise_row(parent: &mut ChildSpawner, exercise_row: &ExerciseScoreRow, window: Entity) {
    parent.spawn((
        teacher_list_exercise_row(),
        Children::spawn(SpawnWith({
            let label = exercise_row.exercise_label.clone();
            let score = exercise_row.score_text.clone();
            let color = exercise_row.score_color;
            let lesson_id = exercise_row.lesson_id.clone();
            let exercise_id = exercise_row.exercise_id.clone();
            move |row: &mut ChildSpawner| {
                row.spawn((
                    teacher_list_label(
                        format!("\u{2022} {label}"),
                        TeacherListLevel::Exercise,
                        theme::colors::TEXT_DARK,
                        window,
                    ),
                    Node {
                        flex_grow: 1.0,
                        ..default()
                    },
                ));
                row.spawn((
                    Text::new(score),
                    theme::typography::text(theme::fonts::SMALL, window),
                    TextColor(color),
                    Node {
                        margin: UiRect::right(theme::scaled(theme::spacing::MEDIUM)),
                        ..default()
                    },
                ));
                row.spawn((
                    reset_icon_button(window),
                    StatsResetButton(StatsResetTarget::Exercise(lesson_id, exercise_id)),
                ));
            }
        })),
    ));
}

#[allow(clippy::too_many_arguments)]
fn handle_reset_click(
    mut activations: MessageReader<ButtonActivated>,
    query: Query<&StatsResetButton>,
    mut commands: Commands,
    existing_popover: Query<Entity, With<StatsResetPopover>>,
    i18n: Res<I18n>,
    content: Res<ContentLibrary>,
    ctx: PlayerContext<'_>,
    teacher: TeacherWindowParam<'_, '_>,
    teacher_state: Query<&TeacherWindowState, With<TeacherWindow>>,
) {
    let activated_entities: Vec<Entity> = activations.read().map(|event| event.0).collect();
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

    for entity in activated_entities {
        let Ok(reset_btn) = query.get(entity) else {
            continue;
        };

        // Despawn any existing popover
        for entity in &existing_popover {
            commands.entity(entity).try_despawn();
        }

        let confirm_message = match &reset_btn.0 {
            StatsResetTarget::All => i18n.t(&TranslationKey::ResetAllStatsConfirm).into_owned(),
            StatsResetTarget::Theme(theme_id) => {
                let title = content
                    .themes
                    .iter()
                    .find(|theme| theme.id == *theme_id)
                    .map_or_else(
                        || theme_id.clone(),
                        |theme| i18n.t(&theme.title_key).into_owned(),
                    );
                i18n.t(&TranslationKey::ResetLessonStatsConfirm(title))
                    .into_owned()
            }
            StatsResetTarget::Lesson(lesson_id) => i18n
                .t(&TranslationKey::ResetLessonStatsConfirm(lesson_label(
                    lesson_id, &content, &i18n,
                )))
                .into_owned(),
            StatsResetTarget::Type(lesson_id, exercise_type) => i18n
                .t(&TranslationKey::ResetTypeStatsConfirm(
                    lesson_label(lesson_id, &content, &i18n),
                    exercise_type_label(*exercise_type, &i18n),
                ))
                .into_owned(),
            StatsResetTarget::Exercise(lesson_id, exercise_id) => i18n
                .t(&TranslationKey::ResetExerciseStatsConfirm(
                    lesson_label(lesson_id, &content, &i18n),
                    exercise_label(lesson_id, exercise_id, &content, i18n.language),
                ))
                .into_owned(),
        };

        let modal_entity = spawn_confirmation_modal(
            &mut commands,
            &confirm_message,
            &i18n.t(&TranslationKey::Delete),
            &i18n.t(&TranslationKey::Cancel),
            theme::button_styles::DANGER,
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
    content: Res<ContentLibrary>,
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
        apply_stats_reset(&mut student.progress, &target, &content);
    });
}

fn apply_stats_reset(
    progress: &mut HashMap<String, LessonProgress>,
    target: &StatsResetTarget,
    content: &ContentLibrary,
) {
    match target {
        StatsResetTarget::All => progress.clear(),
        StatsResetTarget::Theme(theme_id) => {
            if let Some(theme) = content.themes.iter().find(|theme| theme.id == *theme_id) {
                for lesson in &theme.lessons {
                    progress.remove(&lesson.id);
                }
            }
        }
        StatsResetTarget::Lesson(lesson_id) => {
            progress.remove(lesson_id);
        }
        StatsResetTarget::Type(lesson_id, exercise_type) => {
            let content_lesson = content
                .themes
                .iter()
                .flat_map(|theme| &theme.lessons)
                .find(|lesson| lesson.id == *lesson_id);
            if let (Some(progress_lesson), Some(content_lesson)) =
                (progress.get_mut(lesson_id), content_lesson)
            {
                progress_lesson.question_scores.retain(|exercise_id, _| {
                    exercise_type_for_id(content_lesson, exercise_id) != Some(*exercise_type)
                });
                if progress_lesson.question_scores.is_empty() {
                    progress.remove(lesson_id);
                }
            }
        }
        StatsResetTarget::Exercise(lesson_id, exercise_id) => {
            if let Some(lesson) = progress.get_mut(lesson_id) {
                lesson.question_scores.remove(exercise_id);
                if lesson.question_scores.is_empty() {
                    progress.remove(lesson_id);
                }
            }
        }
    }
}

fn handle_return_to_list(
    mut activations: MessageReader<ButtonActivated>,
    query: Query<(), With<ReturnToListButton>>,
    mut states: Query<&mut TeacherWindowState, With<TeacherWindow>>,
) {
    let activated_entities: Vec<Entity> = activations.read().map(|event| event.0).collect();
    if !activated_entities
        .iter()
        .any(|entity| query.contains(*entity))
    {
        return;
    }
    if let Ok(mut state) = states.single_mut() {
        state.view = TeacherView::Students;
    }
}
