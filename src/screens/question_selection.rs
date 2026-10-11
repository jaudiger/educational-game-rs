use bevy::input_focus::AutoFocus;
use bevy::prelude::*;
use bevy::ui_widgets::ScrollArea;
use bevy::window::PrimaryWindow;

use crate::data::content::QuestionType;
use crate::data::{
    ActiveTheme, ContentLibrary, ExplorationTheme, LessonLaunchSelection, SelectedLesson,
};
use crate::i18n::{I18n, TranslationKey};
use crate::screens::teacher_shared::question_type_label;
use crate::states::{AppState, ExplorationView};
use crate::ui::animation::FloatingCard;
use crate::ui::components::{ButtonActivated, screen_root, standard_button};
use crate::ui::theme;

pub struct QuestionSelectionScreenPlugin;

impl Plugin for QuestionSelectionScreenPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(
            OnEnter(ExplorationView::QuestionSelection),
            setup_question_selection,
        )
        .add_systems(
            Update,
            handle_question_selection.run_if(in_state(ExplorationView::QuestionSelection)),
        );
    }
}

#[derive(Component, Reflect)]
struct QuestionSelectionRoot;

#[derive(Component, Reflect)]
struct QuestionExerciseButton(String);

#[derive(Component, Reflect)]
struct BackToLessonGridButton;

struct QuestionGroup<T> {
    question_type: QuestionType,
    questions: Vec<T>,
}

struct ExerciseButtonData {
    id: String,
    label: String,
}

struct QuestionGroupData {
    label: String,
    questions: Vec<ExerciseButtonData>,
}

struct QuestionSelectionScreenData {
    title: String,
    back_label: String,
    groups: Vec<QuestionGroupData>,
    selected_id: Option<String>,
    sky_theme: bool,
}

#[allow(clippy::too_many_arguments)]
fn setup_question_selection(
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

    let groups = localized_question_groups(lesson, &i18n);
    let selected_id = launch_selection
        .as_deref()
        .and_then(|selection| match selection {
            LessonLaunchSelection::SingleQuestion { exercise_id } => Some(exercise_id.as_str()),
            LessonLaunchSelection::CompleteLesson => None,
        })
        .filter(|id| {
            groups
                .iter()
                .any(|group| group.questions.iter().any(|q| q.id == *id))
        })
        .map(str::to_owned);
    let data = QuestionSelectionScreenData {
        title: i18n.t(&lesson.title_key).into_owned(),
        back_label: i18n.t(&TranslationKey::BackToLessons).into_owned(),
        groups,
        selected_id,
        sky_theme: settings.exploration_theme == ExplorationTheme::Sky,
    };
    spawn_question_selection_screen(&mut commands, data, *primary_window);
}

fn localized_question_groups(
    lesson: &crate::data::content::Lesson,
    i18n: &I18n,
) -> Vec<QuestionGroupData> {
    group_questions(
        lesson
            .questions
            .iter()
            .map(|question| (question.definition.question_type(), question)),
    )
    .into_iter()
    .map(|group| QuestionGroupData {
        label: question_type_label(group.question_type, i18n),
        questions: group
            .questions
            .into_iter()
            .map(|question| ExerciseButtonData {
                id: question.id.clone(),
                label: question.definition.prompt_label(i18n.language),
            })
            .collect(),
    })
    .collect()
}

fn spawn_question_selection_screen(
    commands: &mut Commands,
    data: QuestionSelectionScreenData,
    window: Entity,
) {
    let QuestionSelectionScreenData {
        title,
        back_label,
        groups,
        selected_id,
        sky_theme,
    } = data;
    let has_questions = groups.iter().any(|group| !group.questions.is_empty());
    let list_background = if sky_theme {
        Color::srgba(0.88, 0.93, 1.0, 0.76)
    } else {
        theme::colors::CARD_BG
    };
    let button_background = if sky_theme {
        Color::srgba(0.88, 0.93, 1.0, 0.92)
    } else {
        theme::colors::PRIMARY
    };
    let button_text = if sky_theme {
        theme::colors::TEXT_DARK
    } else {
        theme::colors::TEXT_LIGHT
    };
    let mut root = commands.spawn((
        screen_root(),
        QuestionSelectionRoot,
        DespawnOnExit(AppState::ThemeExploration),
        DespawnOnEnter(ExplorationView::ThemeLessons),
        Children::spawn(SpawnWith(move |parent: &mut ChildSpawner| {
            parent.spawn((
                Text::new(title),
                theme::typography::text(theme::fonts::TITLE, window),
                TextColor(theme::colors::TEXT_DARK),
            ));
            let mut question_list = parent.spawn((
                ScrollArea,
                Node {
                    width: percent(88.0),
                    flex_grow: 1.0,
                    flex_basis: px(0.0),
                    min_height: px(0.0),
                    overflow: Overflow::scroll_y(),
                    border: px(1.0).all(),
                    border_radius: BorderRadius::all(theme::scaled(
                        theme::sizes::CARD_BORDER_RADIUS,
                    )),
                    ..default()
                },
                BackgroundColor(list_background),
                BorderColor::all(theme::colors::INPUT_BORDER),
                Children::spawn(SpawnWith(move |scroll_area: &mut ChildSpawner| {
                    scroll_area.spawn(question_groups_node(
                        groups,
                        selected_id,
                        button_background,
                        button_text,
                        window,
                    ));
                })),
            ));
            if sky_theme {
                question_list.insert((
                    BoxShadow::new(
                        Color::srgba(0.08, 0.12, 0.28, 0.15),
                        Val::Px(0.0),
                        Val::Px(4.0),
                        Val::Px(16.0),
                        Val::Px(2.0),
                    ),
                    FloatingCard {
                        phase: 0.0,
                        amplitude: theme::animation::FLOATING_AMPLITUDE,
                    },
                ));
            }
            spawn_question_selection_footer(parent, back_label, has_questions, window);
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

fn spawn_question_selection_footer(
    parent: &mut ChildSpawner,
    back_label: String,
    has_questions: bool,
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
                    theme::colors::PRIMARY,
                    theme::scaled(theme::sizes::BUTTON_WIDTH),
                    window,
                ),
                BackToLessonGridButton,
            ));
            if !has_questions {
                back.insert(AutoFocus);
            }
        })),
    ));
}

fn question_groups_node(
    groups: Vec<QuestionGroupData>,
    selected_id: Option<String>,
    button_background: Color,
    button_text: Color,
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
            let first_question = groups
                .iter()
                .flat_map(|group| &group.questions)
                .next()
                .map(|question| question.id.as_str());
            for group in &groups {
                list.spawn((
                    Text::new(group.label.clone()),
                    theme::typography::text(theme::fonts::BODY, window),
                    TextColor(theme::colors::TEXT_DARK),
                    Node {
                        margin: theme::scaled(theme::spacing::SMALL).top(),
                        ..default()
                    },
                ));
                for question in &group.questions {
                    let mut button = list.spawn((
                        question_button(&question.label, button_background, button_text, window),
                        QuestionExerciseButton(question.id.clone()),
                    ));
                    if selected_id.as_deref() == Some(question.id.as_str())
                        || (selected_id.is_none() && first_question == Some(question.id.as_str()))
                    {
                        button.insert(AutoFocus);
                    }
                }
            }
        })),
    )
}

fn question_button(
    label: &str,
    background: Color,
    text_color: Color,
    window: Entity,
) -> impl Bundle + use<> {
    (
        crate::ui::components::button_base(background),
        Node {
            width: percent(100.0),
            min_height: theme::scaled(52.0),
            padding: theme::scaled(theme::spacing::SMALL).all(),
            justify_content: JustifyContent::Center,
            align_items: AlignItems::Center,
            border_radius: BorderRadius::all(theme::scaled(theme::sizes::BUTTON_BORDER_RADIUS)),
            ..default()
        },
        children![(
            Text::new(label.to_owned()),
            theme::typography::text(theme::fonts::BODY, window),
            TextColor(text_color),
            TextLayout::justify(Justify::Center),
        )],
    )
}

fn group_questions<T>(
    questions: impl IntoIterator<Item = (QuestionType, T)>,
) -> Vec<QuestionGroup<T>> {
    let mut groups: Vec<QuestionGroup<T>> = Vec::new();
    for (question_type, question) in questions {
        if let Some(group) = groups
            .iter_mut()
            .find(|group| group.question_type == question_type)
        {
            group.questions.push(question);
        } else {
            groups.push(QuestionGroup {
                question_type,
                questions: vec![question],
            });
        }
    }
    groups
}

fn handle_question_selection(
    mut activations: MessageReader<ButtonActivated>,
    question_buttons: Query<&QuestionExerciseButton>,
    back_buttons: Query<(), With<BackToLessonGridButton>>,
    mut commands: Commands,
    mut next_app_state: ResMut<NextState<AppState>>,
    mut next_exploration_view: ResMut<NextState<ExplorationView>>,
) {
    for activation in activations.read() {
        if let Ok(button) = question_buttons.get(activation.0) {
            commands.insert_resource(LessonLaunchSelection::SingleQuestion {
                exercise_id: button.0.clone(),
            });
            next_app_state.set(AppState::LessonPlay);
        } else if back_buttons.contains(activation.0) {
            next_exploration_view.set(ExplorationView::ThemeLessons);
        }
    }
}
