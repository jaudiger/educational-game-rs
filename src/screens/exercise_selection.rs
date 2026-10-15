use bevy::input_focus::AutoFocus;
use bevy::prelude::*;
use bevy::ui_widgets::{ScrollArea, ScrollIntoView};
use bevy::window::PrimaryWindow;

use crate::data::content::QuestionType;
use crate::data::{
    ActiveTheme, ContentLibrary, ExplorationTheme, LessonLaunchSelection, SelectedLesson,
};
use crate::i18n::{I18n, TranslationKey};
use crate::screens::teacher_shared::exercise_type_label;
use crate::states::{AppState, ExplorationView};
use crate::ui::animation::FloatingCard;
use crate::ui::components::{
    ButtonActivated, button_base, disclosure_chevron, disclosure_chevron_transform,
    list_separator_border, screen_root, standard_button,
};
use crate::ui::theme;

pub struct ExerciseSelectionScreenPlugin;

impl Plugin for ExerciseSelectionScreenPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(
            OnEnter(ExplorationView::ExerciseSelection),
            setup_exercise_selection,
        )
        .add_systems(
            Update,
            (
                handle_exercise_selection,
                handle_exercise_group_disclosure,
                scroll_selected_exercise_into_view,
            )
                .run_if(in_state(ExplorationView::ExerciseSelection)),
        );
    }
}

#[derive(Component, Reflect)]
struct ExerciseSelectionRoot;

#[derive(Component, Reflect)]
struct ExerciseButton(String);

#[derive(Component, Reflect)]
struct ExerciseGroupToggle(QuestionType);

#[derive(Component, Reflect)]
struct ExerciseGroupPanel(QuestionType);

#[derive(Component, Reflect)]
struct ExerciseGroupChevron(QuestionType);

#[derive(Component, Reflect)]
struct ScrollToSelectedExercise;

#[derive(Component, Reflect)]
struct BackToLessonGridButton;

struct ExerciseGroup<T> {
    exercise_type: QuestionType,
    exercises: Vec<T>,
}

#[derive(Clone)]
struct ExerciseButtonData {
    id: String,
    label: String,
}

#[derive(Clone)]
struct ExerciseGroupData {
    exercise_type: QuestionType,
    label: String,
    exercise_count: String,
    exercises: Vec<ExerciseButtonData>,
}

struct ExerciseGroupView {
    group: ExerciseGroupData,
    expanded: bool,
    autofocus_group: bool,
    selected_exercise_id: Option<String>,
    autofocus_exercise_id: Option<String>,
}

struct ExerciseSelectionScreenData {
    lesson_context: String,
    back_label: String,
    groups: Vec<ExerciseGroupData>,
    selected_exercise_id: Option<String>,
    sky_theme: bool,
}

#[allow(clippy::too_many_arguments)]
fn setup_exercise_selection(
    mut commands: Commands,
    content: Res<ContentLibrary>,
    active_theme: Res<ActiveTheme>,
    selected_lesson: Res<SelectedLesson>,
    launch_selection: Option<Res<LessonLaunchSelection>>,
    i18n: Res<I18n>,
    settings: Res<crate::data::GameSettings>,
    primary_window: Single<Entity, With<PrimaryWindow>>,
) {
    let Some(lesson_id) = selected_lesson.0.as_deref() else {
        return;
    };
    let Some(lesson) = content
        .theme(&active_theme)
        .and_then(|theme| theme.lesson(lesson_id))
    else {
        return;
    };

    let groups = localized_exercise_groups(lesson, &i18n);
    let sky_theme = settings.exploration_theme == ExplorationTheme::Sky;
    let selected_exercise_id = match launch_selection.as_deref() {
        Some(LessonLaunchSelection::SingleExercise { exercise_id }) => Some(exercise_id.as_str()),
        Some(LessonLaunchSelection::CompleteLesson) | None => None,
    }
    .filter(|id| {
        groups
            .iter()
            .any(|group| group.exercises.iter().any(|exercise| exercise.id == *id))
    })
    .map(str::to_owned);
    let data = ExerciseSelectionScreenData {
        lesson_context: lesson.title.get(i18n.language).to_owned(),
        back_label: i18n.t(&TranslationKey::BackToLessons).into_owned(),
        groups,
        selected_exercise_id,
        sky_theme,
    };
    spawn_exercise_selection_screen(&mut commands, data, *primary_window);
}

fn localized_exercise_groups(
    lesson: &crate::data::content::Lesson,
    i18n: &I18n,
) -> Vec<ExerciseGroupData> {
    group_exercises(
        lesson
            .questions
            .iter()
            .map(|exercise| (exercise.definition.question_type(), exercise)),
    )
    .into_iter()
    .map(|group| ExerciseGroupData {
        exercise_type: group.exercise_type,
        label: exercise_type_label(group.exercise_type, i18n),
        exercise_count: i18n
            .t(&TranslationKey::ExerciseCount(group.exercises.len()))
            .into_owned(),
        exercises: group
            .exercises
            .into_iter()
            .map(|exercise| ExerciseButtonData {
                id: exercise.id.clone(),
                label: exercise.definition.prompt_label(i18n.language),
            })
            .collect(),
    })
    .collect()
}

fn spawn_exercise_selection_screen(
    commands: &mut Commands,
    data: ExerciseSelectionScreenData,
    window: Entity,
) {
    let sky_theme = data.sky_theme;
    let list_background = if sky_theme {
        theme::colors::LIST_SURFACE
    } else {
        theme::colors::CARD_BG
    };
    let mut root = commands.spawn((
        screen_root(),
        ExerciseSelectionRoot,
        DespawnOnExit(AppState::ThemeExploration),
        DespawnOnEnter(ExplorationView::ThemeLessons),
        Children::spawn(SpawnWith(move |parent: &mut ChildSpawner| {
            spawn_exercise_selection_content(parent, data, list_background, window);
        })),
    ));
    root.insert(Node {
        width: percent(100.0),
        height: percent(100.0),
        padding: theme::scaled(theme::spacing::MEDIUM).all(),
        justify_content: JustifyContent::FlexStart,
        align_items: AlignItems::Center,
        flex_direction: FlexDirection::Column,
        row_gap: theme::scaled(theme::spacing::SMALL),
        ..default()
    });
    if sky_theme {
        root.insert(BackgroundColor(Color::NONE));
    }
}

fn spawn_exercise_selection_content(
    parent: &mut ChildSpawner,
    data: ExerciseSelectionScreenData,
    list_background: Color,
    window: Entity,
) {
    let ExerciseSelectionScreenData {
        lesson_context,
        back_label,
        groups,
        selected_exercise_id,
        sky_theme,
    } = data;
    let has_exercises = groups.iter().any(|group| !group.exercises.is_empty());
    if sky_theme {
        parent.spawn((
            Text::new(lesson_context),
            theme::typography::text(theme::fonts::HEADING, window),
            TextColor(theme::colors::TEXT_DARK),
        ));
        parent.spawn((
            ScrollArea,
            Node {
                width: percent(88.0),
                flex_grow: 1.0,
                flex_basis: px(0.0),
                min_height: px(0.0),
                overflow: Overflow::scroll_y(),
                border: px(1.0).all(),
                border_radius: BorderRadius::all(theme::scaled(24.0)),
                ..default()
            },
            BackgroundColor(list_background),
            BorderColor::all(Color::srgba(0.30, 0.55, 0.84, 0.78)),
            BoxShadow::new(
                Color::srgba(0.08, 0.14, 0.27, 0.15),
                Val::Px(0.0),
                Val::Px(3.0),
                Val::Px(12.0),
                Val::Px(1.0),
            ),
            FloatingCard {
                phase: 0.0,
                amplitude: theme::animation::FLOATING_AMPLITUDE,
            },
            Children::spawn(SpawnWith(move |scroll_area: &mut ChildSpawner| {
                scroll_area.spawn(exercise_groups_node(groups, selected_exercise_id, window));
            })),
        ));
    } else {
        parent.spawn((
            ScrollArea,
            Node {
                width: percent(88.0),
                flex_grow: 1.0,
                flex_basis: px(0.0),
                min_height: px(0.0),
                overflow: Overflow::scroll_y(),
                border: px(1.0).all(),
                border_radius: BorderRadius::all(theme::scaled(theme::sizes::CARD_BORDER_RADIUS)),
                ..default()
            },
            BackgroundColor(list_background),
            BorderColor::all(theme::colors::INPUT_BORDER),
            Children::spawn(SpawnWith(move |scroll_area: &mut ChildSpawner| {
                scroll_area.spawn(exercise_groups_node(groups, selected_exercise_id, window));
            })),
        ));
    }
    spawn_exercise_selection_footer(parent, back_label, has_exercises, window);
}

fn spawn_exercise_selection_footer(
    parent: &mut ChildSpawner,
    back_label: String,
    has_exercises: bool,
    window: Entity,
) {
    parent.spawn((
        Node {
            width: percent(88.0),
            flex_shrink: 0.0,
            justify_content: JustifyContent::Center,
            align_items: AlignItems::Center,
            ..default()
        },
        Children::spawn(SpawnWith(move |footer: &mut ChildSpawner| {
            let mut back = footer.spawn((
                standard_button(
                    &back_label,
                    theme::button_styles::BACK,
                    theme::scaled(theme::sizes::BUTTON_WIDTH),
                    window,
                ),
                BackToLessonGridButton,
            ));
            if !has_exercises {
                back.insert(AutoFocus);
            }
        })),
    ));
}

fn exercise_groups_node(
    groups: Vec<ExerciseGroupData>,
    selected_exercise_id: Option<String>,
    window: Entity,
) -> impl Bundle + use<> {
    (
        Node {
            width: percent(100.0),
            flex_direction: FlexDirection::Column,
            flex_shrink: 0.0,
            row_gap: theme::scaled(theme::spacing::SMALL),
            padding: theme::scaled(theme::spacing::MEDIUM).all(),
            ..default()
        },
        Children::spawn(SpawnWith(move |list: &mut ChildSpawner| {
            for (index, group) in groups.into_iter().enumerate() {
                let selected_exercise_id = selected_exercise_id.clone();
                let selected_in_group = selected_exercise_id.as_ref().is_some_and(|selected_id| {
                    group
                        .exercises
                        .iter()
                        .any(|exercise| exercise.id == *selected_id)
                });
                let expanded = selected_in_group;
                let autofocus_group = selected_exercise_id.is_none() && index == 0;
                let autofocus_exercise_id = selected_exercise_id.clone();
                spawn_exercise_group(
                    list,
                    ExerciseGroupView {
                        group,
                        expanded,
                        autofocus_group,
                        selected_exercise_id,
                        autofocus_exercise_id,
                    },
                    window,
                );
            }
        })),
    )
}

fn spawn_exercise_group(parent: &mut ChildSpawner, view: ExerciseGroupView, window: Entity) {
    let ExerciseGroupView {
        group,
        expanded,
        autofocus_group,
        selected_exercise_id,
        autofocus_exercise_id,
    } = view;
    let exercise_type = group.exercise_type;
    let label = group.label;
    let exercise_count = group.exercise_count;
    let exercises = group.exercises;
    parent.spawn((
        Node {
            width: percent(100.0),
            flex_direction: FlexDirection::Column,
            flex_shrink: 0.0,
            ..default()
        },
        Children::spawn(SpawnWith(move |section: &mut ChildSpawner| {
            let mut toggle = section.spawn((
                button_base(theme::button_styles::DISCLOSURE),
                ExerciseGroupToggle(exercise_type),
                Node {
                    width: percent(100.0),
                    min_height: theme::scaled(40.0),
                    margin: theme::scaled(theme::spacing::MEDIUM).top(),
                    padding: UiRect::bottom(theme::scaled(theme::spacing::SMALL)),
                    flex_direction: FlexDirection::Row,
                    align_items: AlignItems::Center,
                    column_gap: theme::scaled(theme::spacing::SMALL),
                    border: px(1.0).bottom(),
                    ..default()
                },
                list_separator_border(),
                Children::spawn(SpawnWith(move |header: &mut ChildSpawner| {
                    header.spawn((
                        disclosure_chevron(expanded, theme::colors::PRIMARY, window),
                        ExerciseGroupChevron(exercise_type),
                    ));
                    header.spawn((
                        Text::new(label),
                        theme::typography::text(theme::fonts::BODY, window),
                        TextColor(theme::colors::PRIMARY),
                        Node {
                            flex_grow: 1.0,
                            ..default()
                        },
                    ));
                    header.spawn((
                        Text::new(exercise_count),
                        theme::typography::text(theme::fonts::SMALL, window),
                        TextColor(theme::colors::TEXT_MUTED),
                    ));
                })),
            ));
            if autofocus_group {
                toggle.insert(AutoFocus);
            }
            section.spawn((
                ExerciseGroupPanel(exercise_type),
                Node {
                    display: disclosure_display(expanded),
                    flex_direction: FlexDirection::Column,
                    row_gap: theme::scaled(theme::spacing::SMALL),
                    padding: UiRect {
                        left: theme::scaled(theme::spacing::MEDIUM),
                        top: theme::scaled(theme::spacing::SMALL),
                        ..default()
                    },
                    ..default()
                },
                Children::spawn(SpawnWith(move |items: &mut ChildSpawner| {
                    for exercise in exercises {
                        let selected = selected_exercise_id.as_deref() == Some(&exercise.id);
                        let mut button = items.spawn((
                            exercise_button(&exercise.label, selected, window),
                            ExerciseButton(exercise.id.clone()),
                        ));
                        if selected {
                            button.insert((AutoFocus, ScrollToSelectedExercise));
                        } else if autofocus_exercise_id.as_deref() == Some(&exercise.id) {
                            button.insert(AutoFocus);
                        }
                    }
                })),
            ));
        })),
    ));
}

fn exercise_button(label: &str, selected: bool, window: Entity) -> impl Bundle + use<> {
    let palette = if selected {
        theme::button_styles::EXERCISE_ITEM_SELECTED
    } else {
        theme::button_styles::EXERCISE_ITEM
    };
    let border = palette.border.map_or(Color::NONE, |states| states.normal);
    (
        button_base(palette),
        Node {
            width: percent(100.0),
            min_height: theme::scaled(64.0),
            padding: UiRect::axes(
                theme::scaled(theme::spacing::MEDIUM),
                theme::scaled(theme::spacing::SMALL),
            ),
            justify_content: JustifyContent::FlexStart,
            align_items: AlignItems::Center,
            border: px(1.0).all(),
            border_radius: BorderRadius::all(theme::scaled(theme::sizes::BUTTON_BORDER_RADIUS)),
            ..default()
        },
        BorderColor::all(border),
        children![(
            Text::new(label.to_owned()),
            theme::typography::text(theme::fonts::BODY, window),
            TextColor(palette.foreground),
            TextLayout::justify(Justify::Left),
        )],
    )
}

fn group_exercises<T>(
    exercises: impl IntoIterator<Item = (QuestionType, T)>,
) -> Vec<ExerciseGroup<T>> {
    let mut groups: Vec<ExerciseGroup<T>> = Vec::new();
    for (exercise_type, exercise) in exercises {
        if let Some(group) = groups
            .iter_mut()
            .find(|group| group.exercise_type == exercise_type)
        {
            group.exercises.push(exercise);
        } else {
            groups.push(ExerciseGroup {
                exercise_type,
                exercises: vec![exercise],
            });
        }
    }
    groups
}

const fn disclosure_display(expanded: bool) -> Display {
    if expanded {
        Display::Flex
    } else {
        Display::None
    }
}

fn handle_exercise_group_disclosure(
    mut activations: MessageReader<ButtonActivated>,
    toggles: Query<&ExerciseGroupToggle>,
    mut panels: Query<(&ExerciseGroupPanel, &mut Node)>,
    mut chevrons: Query<(&ExerciseGroupChevron, &mut UiTransform)>,
) {
    for event in activations.read() {
        let Ok(toggle) = toggles.get(event.0) else {
            continue;
        };
        let exercise_type = toggle.0;
        for (panel, mut node) in &mut panels {
            if panel.0 == exercise_type {
                let expanded = node.display == Display::None;
                node.display = disclosure_display(expanded);
                for (chevron, mut transform) in &mut chevrons {
                    if chevron.0 == exercise_type {
                        *transform = disclosure_chevron_transform(expanded);
                    }
                }
            }
        }
    }
}

fn scroll_selected_exercise_into_view(
    mut commands: Commands,
    selected: Query<Entity, With<ScrollToSelectedExercise>>,
    laid_out: Query<(), (With<ComputedNode>, With<UiGlobalTransform>)>,
) {
    for entity in &selected {
        if laid_out.contains(entity) {
            commands.trigger(ScrollIntoView { entity });
            commands.entity(entity).remove::<ScrollToSelectedExercise>();
        }
    }
}

fn handle_exercise_selection(
    mut activations: MessageReader<ButtonActivated>,
    exercise_buttons: Query<&ExerciseButton>,
    back_buttons: Query<(), With<BackToLessonGridButton>>,
    mut commands: Commands,
    mut next_app_state: ResMut<NextState<AppState>>,
    mut next_exploration_view: ResMut<NextState<ExplorationView>>,
) {
    for activation in activations.read() {
        if let Ok(button) = exercise_buttons.get(activation.0) {
            commands.insert_resource(LessonLaunchSelection::SingleExercise {
                exercise_id: button.0.clone(),
            });
            next_app_state.set(AppState::LessonPlay);
        } else if back_buttons.contains(activation.0) {
            next_exploration_view.set(ExplorationView::ThemeLessons);
        }
    }
}
