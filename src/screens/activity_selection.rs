use bevy::input_focus::AutoFocus;
use bevy::prelude::*;
use bevy::ui_widgets::{ScrollArea, ScrollIntoView};
use bevy::window::PrimaryWindow;

use crate::data::content::QuestionType;
use crate::data::{
    ActiveTheme, ContentLibrary, ExplorationTheme, LessonLaunchSelection, SelectedLesson,
};
use crate::i18n::{I18n, TranslationKey};
use crate::screens::activity_categories::activity_category_label;
use crate::states::{AppState, ExplorationView};
use crate::ui::animation::FloatingCard;
use crate::ui::components::{
    ButtonActivated, button_base, disclosure_chevron, disclosure_chevron_transform,
    list_separator_border, screen_root, standard_button,
};
use crate::ui::theme;

pub struct ActivitySelectionScreenPlugin;

impl Plugin for ActivitySelectionScreenPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(
            OnEnter(ExplorationView::ActivitySelection),
            setup_activity_selection,
        )
        .add_systems(
            Update,
            (
                handle_activity_selection,
                handle_activity_group_disclosure,
                scroll_selected_activity_into_view,
            )
                .run_if(in_state(ExplorationView::ActivitySelection)),
        );
    }
}

#[derive(Component, Reflect)]
struct ActivitySelectionRoot;

#[derive(Component, Reflect)]
struct ActivityButton(String);

#[derive(Component, Reflect)]
struct ActivityGroupToggle(QuestionType);

#[derive(Component, Reflect)]
struct ActivityGroupPanel(QuestionType);

#[derive(Component, Reflect)]
struct ActivityGroupChevron(QuestionType);

#[derive(Component, Reflect)]
struct ScrollToSelectedActivity;

#[derive(Component, Reflect)]
struct BackToLessonGridButton;

struct ActivityGroup<T> {
    activity_type: QuestionType,
    activities: Vec<T>,
}

#[derive(Clone)]
struct ActivityButtonData {
    id: String,
    label: String,
}

#[derive(Clone)]
struct ActivityGroupData {
    activity_type: QuestionType,
    label: String,
    activity_count: String,
    activities: Vec<ActivityButtonData>,
}

struct ActivityGroupView {
    group: ActivityGroupData,
    expanded: bool,
    autofocus_group: bool,
    selected_activity_id: Option<String>,
    autofocus_activity_id: Option<String>,
}

struct ActivitySelectionScreenData {
    lesson_context: String,
    back_label: String,
    groups: Vec<ActivityGroupData>,
    selected_activity_id: Option<String>,
    sky_theme: bool,
}

#[allow(clippy::too_many_arguments)]
fn setup_activity_selection(
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

    let groups = localized_activity_groups(lesson, &i18n);
    let sky_theme = settings.exploration_theme == ExplorationTheme::Sky;
    let selected_activity_id = match launch_selection.as_deref() {
        Some(LessonLaunchSelection::SingleActivity { activity_id }) => Some(activity_id.as_str()),
        Some(LessonLaunchSelection::CompleteLesson) | None => None,
    }
    .filter(|id| {
        groups
            .iter()
            .any(|group| group.activities.iter().any(|activity| activity.id == *id))
    })
    .map(str::to_owned);
    let data = ActivitySelectionScreenData {
        lesson_context: lesson.title.get(i18n.language).to_owned(),
        back_label: i18n.t(&TranslationKey::BackToLessons).into_owned(),
        groups,
        selected_activity_id,
        sky_theme,
    };
    spawn_activity_selection_screen(&mut commands, data, *primary_window);
}

fn localized_activity_groups(
    lesson: &crate::data::content::Lesson,
    i18n: &I18n,
) -> Vec<ActivityGroupData> {
    group_activities(
        lesson
            .questions
            .iter()
            .map(|activity| (activity.definition.question_type(), activity)),
    )
    .into_iter()
    .map(|group| ActivityGroupData {
        activity_type: group.activity_type,
        label: activity_category_label(group.activity_type, i18n),
        activity_count: i18n
            .t(&TranslationKey::ActivityCount(group.activities.len()))
            .into_owned(),
        activities: group
            .activities
            .into_iter()
            .map(|activity| ActivityButtonData {
                id: activity.id.clone(),
                label: activity.catalog_label.get(i18n.language).to_owned(),
            })
            .collect(),
    })
    .collect()
}

fn spawn_activity_selection_screen(
    commands: &mut Commands,
    data: ActivitySelectionScreenData,
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
        ActivitySelectionRoot,
        DespawnOnExit(AppState::ThemeExploration),
        DespawnOnEnter(ExplorationView::ThemeLessons),
        Children::spawn(SpawnWith(move |parent: &mut ChildSpawner| {
            spawn_activity_selection_content(parent, data, list_background, window);
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

fn spawn_activity_selection_content(
    parent: &mut ChildSpawner,
    data: ActivitySelectionScreenData,
    list_background: Color,
    window: Entity,
) {
    let ActivitySelectionScreenData {
        lesson_context,
        back_label,
        groups,
        selected_activity_id,
        sky_theme,
    } = data;
    let has_activities = groups.iter().any(|group| !group.activities.is_empty());
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
                scroll_area.spawn(activity_groups_node(groups, selected_activity_id, window));
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
                scroll_area.spawn(activity_groups_node(groups, selected_activity_id, window));
            })),
        ));
    }
    spawn_activity_selection_footer(parent, back_label, has_activities, window);
}

fn spawn_activity_selection_footer(
    parent: &mut ChildSpawner,
    back_label: String,
    has_activities: bool,
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
            if !has_activities {
                back.insert(AutoFocus);
            }
        })),
    ));
}

fn activity_groups_node(
    groups: Vec<ActivityGroupData>,
    selected_activity_id: Option<String>,
    window: Entity,
) -> impl Bundle + use<> {
    (
        Node {
            width: percent(100.0),
            flex_direction: FlexDirection::Column,
            flex_shrink: 0.0,
            row_gap: theme::scaled(theme::spacing::MEDIUM),
            padding: theme::scaled(theme::spacing::MEDIUM).all(),
            ..default()
        },
        Children::spawn(SpawnWith(move |list: &mut ChildSpawner| {
            for (index, group) in groups.into_iter().enumerate() {
                let selected_activity_id = selected_activity_id.clone();
                let autofocus_group = selected_activity_id.is_none() && index == 0;
                let autofocus_activity_id = selected_activity_id.clone();
                spawn_activity_group(
                    list,
                    ActivityGroupView {
                        group,
                        expanded: true,
                        autofocus_group,
                        selected_activity_id,
                        autofocus_activity_id,
                    },
                    window,
                );
            }
        })),
    )
}

fn spawn_activity_group(parent: &mut ChildSpawner, view: ActivityGroupView, window: Entity) {
    let ActivityGroupView {
        group,
        expanded,
        autofocus_group,
        selected_activity_id,
        autofocus_activity_id,
    } = view;
    let activity_type = group.activity_type;
    let label = group.label;
    let activity_count = group.activity_count;
    let activities = group.activities;
    parent.spawn((
        Node {
            width: percent(100.0),
            flex_direction: FlexDirection::Column,
            flex_shrink: 0.0,
            border: px(1.0).all(),
            border_radius: BorderRadius::all(theme::scaled(theme::sizes::CARD_BORDER_RADIUS)),
            ..default()
        },
        BackgroundColor(theme::colors::CARD_BG),
        BorderColor::all(Color::srgba(0.30, 0.55, 0.84, 0.42)),
        Children::spawn(SpawnWith(move |section: &mut ChildSpawner| {
            let mut toggle = section.spawn((
                button_base(theme::button_styles::ACTIVITY_CATEGORY),
                ActivityGroupToggle(activity_type),
                Node {
                    width: percent(100.0),
                    min_height: theme::scaled(56.0),
                    padding: UiRect::axes(
                        theme::scaled(theme::spacing::MEDIUM),
                        theme::scaled(theme::spacing::SMALL),
                    ),
                    flex_direction: FlexDirection::Row,
                    align_items: AlignItems::Center,
                    column_gap: theme::scaled(theme::spacing::SMALL),
                    border_radius: BorderRadius::new(
                        theme::scaled(theme::sizes::CARD_BORDER_RADIUS),
                        theme::scaled(theme::sizes::CARD_BORDER_RADIUS),
                        px(0.0),
                        px(0.0),
                    ),
                    border: px(1.0).bottom(),
                    ..default()
                },
                list_separator_border(),
                Children::spawn(SpawnWith(move |header: &mut ChildSpawner| {
                    header.spawn((
                        disclosure_chevron(expanded, theme::colors::PRIMARY, window),
                        ActivityGroupChevron(activity_type),
                    ));
                    header.spawn((
                        Text::new(label),
                        theme::typography::text(theme::fonts::BODY, window),
                        TextColor(theme::colors::TEXT_DARK),
                        Node {
                            flex_grow: 1.0,
                            min_width: px(0.0),
                            ..default()
                        },
                    ));
                    header.spawn(activity_count_badge(activity_count, window));
                })),
            ));
            if autofocus_group {
                toggle.insert(AutoFocus);
            }
            section.spawn((
                ActivityGroupPanel(activity_type),
                Node {
                    display: disclosure_display(expanded),
                    flex_direction: FlexDirection::Column,
                    row_gap: theme::scaled(theme::spacing::SMALL),
                    padding: theme::scaled(theme::spacing::MEDIUM).all(),
                    ..default()
                },
                Children::spawn(SpawnWith(move |items: &mut ChildSpawner| {
                    for activity in activities {
                        let selected = selected_activity_id.as_deref() == Some(&activity.id);
                        let mut button = items.spawn((
                            activity_button(&activity.label, selected, window),
                            ActivityButton(activity.id.clone()),
                        ));
                        if selected {
                            button.insert((AutoFocus, ScrollToSelectedActivity));
                        } else if autofocus_activity_id.as_deref() == Some(&activity.id) {
                            button.insert(AutoFocus);
                        }
                    }
                })),
            ));
        })),
    ));
}

fn activity_count_badge(label: String, window: Entity) -> impl Bundle + use<> {
    (
        Node {
            min_height: theme::scaled(32.0),
            flex_shrink: 0.0,
            padding: UiRect::axes(theme::scaled(theme::spacing::SMALL), theme::scaled(4.0)),
            justify_content: JustifyContent::Center,
            align_items: AlignItems::Center,
            border_radius: BorderRadius::all(theme::scaled(16.0)),
            ..default()
        },
        BackgroundColor(theme::colors::ACTIVITY_BADGE),
        children![(
            Text::new(label),
            theme::typography::text(theme::fonts::SMALL, window),
            TextColor(theme::colors::TEXT_DARK),
        )],
    )
}

fn activity_button(label: &str, selected: bool, window: Entity) -> impl Bundle + use<> {
    let palette = if selected {
        theme::button_styles::ACTIVITY_ITEM_SELECTED
    } else {
        theme::button_styles::ACTIVITY_ITEM
    };
    let border = palette.border.map_or(Color::NONE, |states| states.normal);
    (
        button_base(palette),
        Node {
            width: percent(100.0),
            min_height: theme::scaled(72.0),
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

fn group_activities<T>(
    activities: impl IntoIterator<Item = (QuestionType, T)>,
) -> Vec<ActivityGroup<T>> {
    let mut groups: Vec<ActivityGroup<T>> = Vec::new();
    for (activity_type, activity) in activities {
        if let Some(group) = groups
            .iter_mut()
            .find(|group| group.activity_type == activity_type)
        {
            group.activities.push(activity);
        } else {
            groups.push(ActivityGroup {
                activity_type,
                activities: vec![activity],
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

fn handle_activity_group_disclosure(
    mut activations: MessageReader<ButtonActivated>,
    toggles: Query<&ActivityGroupToggle>,
    mut panels: Query<(&ActivityGroupPanel, &mut Node)>,
    mut chevrons: Query<(&ActivityGroupChevron, &mut UiTransform)>,
) {
    for event in activations.read() {
        let Ok(toggle) = toggles.get(event.0) else {
            continue;
        };
        let activity_type = toggle.0;
        for (panel, mut node) in &mut panels {
            if panel.0 == activity_type {
                let expanded = node.display == Display::None;
                node.display = disclosure_display(expanded);
                for (chevron, mut transform) in &mut chevrons {
                    if chevron.0 == activity_type {
                        *transform = disclosure_chevron_transform(expanded);
                    }
                }
            }
        }
    }
}

fn scroll_selected_activity_into_view(
    mut commands: Commands,
    selected: Query<Entity, With<ScrollToSelectedActivity>>,
    laid_out: Query<(), (With<ComputedNode>, With<UiGlobalTransform>)>,
) {
    for entity in &selected {
        if laid_out.contains(entity) {
            commands.trigger(ScrollIntoView { entity });
            commands.entity(entity).remove::<ScrollToSelectedActivity>();
        }
    }
}

fn handle_activity_selection(
    mut activations: MessageReader<ButtonActivated>,
    activity_buttons: Query<&ActivityButton>,
    back_buttons: Query<(), With<BackToLessonGridButton>>,
    mut commands: Commands,
    mut next_app_state: ResMut<NextState<AppState>>,
    mut next_exploration_view: ResMut<NextState<ExplorationView>>,
) {
    for activation in activations.read() {
        if let Ok(button) = activity_buttons.get(activation.0) {
            commands.insert_resource(LessonLaunchSelection::SingleActivity {
                activity_id: button.0.clone(),
            });
            next_app_state.set(AppState::LessonPlay);
        } else if back_buttons.contains(activation.0) {
            next_exploration_view.set(ExplorationView::ThemeLessons);
        }
    }
}
