use bevy::input_focus::tab_navigation::TabGroup;
use bevy::prelude::*;
use bevy::window::PrimaryWindow;

use crate::data::content::{LessonPlanningConfig, build_session_with_selection};
use crate::data::{
    ActiveTheme, ContentLibrary, GameSettings, LessonLaunchSelection, QuestionContainer,
    SelectedLesson,
};
use crate::i18n::{I18n, TranslationKey};
use crate::plugins::lesson_mascot::spawn_lesson_mascot;
use crate::states::AppState;
use crate::ui::components::icon_button;
use crate::ui::theme::{self, GameImages};

use super::{ProgressText, QuitLessonButton};

#[derive(Resource)]
pub struct RejectedLessonLaunch;

#[derive(Resource, Default)]
pub struct LessonPlanningInput(pub Option<LessonPlanningConfig>);

pub(super) fn fit_feedback_to_available_height(
    containers: Query<&ComputedNode, With<QuestionContainer>>,
    mut feedbacks: Query<(&ComputedNode, &mut UiTransform), With<super::FeedbackRoot>>,
) {
    let Ok(container) = containers.single() else {
        return;
    };
    let Ok((feedback, mut transform)) = feedbacks.single_mut() else {
        return;
    };

    let content_inset = container.content_inset();
    let available_height =
        (container.size().y - content_inset.min_inset.y - content_inset.max_inset.y).max(0.0);
    let feedback_height = feedback.size().y;
    let scale = if feedback_height > 0.0 {
        (available_height / feedback_height).min(1.0)
    } else {
        1.0
    };
    let scale = Vec2::splat(scale);
    if transform.scale != scale {
        transform.scale = scale;
    }
}

#[allow(clippy::too_many_arguments)]
pub fn setup_lesson_play(
    mut commands: Commands,
    asset_server: Res<AssetServer>,
    images: Res<GameImages>,
    selected_lesson: Res<SelectedLesson>,
    launch_selection: Option<Res<LessonLaunchSelection>>,
    rejected_launch: Option<Res<RejectedLessonLaunch>>,
    content: Res<ContentLibrary>,
    active_theme: Res<ActiveTheme>,
    i18n: Res<I18n>,
    planning_input: Res<LessonPlanningInput>,
    primary_window: Single<Entity, With<PrimaryWindow>>,
    mut next_app_state: ResMut<NextState<AppState>>,
    settings: Res<GameSettings>,
) {
    let window = *primary_window;
    if rejected_launch.is_some() {
        return;
    }
    let default_selection = LessonLaunchSelection::CompleteLesson;
    let selection = launch_selection.as_deref().unwrap_or(&default_selection);
    let planning_config = planning_input.0.as_ref();
    let session = match build_session_with_selection(
        &selected_lesson,
        selection,
        &content,
        &active_theme,
        planning_config,
    ) {
        Ok(session) if !session.questions.is_empty() => session,
        Ok(_) => {
            bevy::log::error!("Cannot start lesson: the session has no exercises");
            next_app_state.set(AppState::ThemeExploration);
            return;
        }
        Err(issues) => {
            bevy::log::error!("Cannot start lesson: {}", issues.join("; "));
            next_app_state.set(AppState::ThemeExploration);
            return;
        }
    };
    let progress = i18n.t(&TranslationKey::QuestionProgress(
        1,
        session.questions.len(),
    ));
    commands.insert_resource(session);

    commands
        .spawn((
            Node {
                width: percent(100.0),
                height: percent(100.0),
                min_width: px(0.0),
                min_height: px(0.0),
                flex_direction: FlexDirection::Column,
                align_items: AlignItems::Center,
                justify_content: JustifyContent::FlexStart,
                row_gap: theme::scaled(theme::spacing::LARGE),
                padding: theme::scaled(theme::spacing::LARGE).all(),
                ..default()
            },
            ImageNode {
                image: images.question_background.clone(),
                image_mode: NodeImageMode::Stretch,
                visual_box: VisualBox::BorderBox,
                ..default()
            },
            DespawnOnExit(AppState::LessonPlay),
            TabGroup::new(0),
        ))
        .with_children(|parent| {
            // Mascot first: absolutely positioned, renders behind siblings.
            spawn_lesson_mascot(parent, settings.exploration_theme, &asset_server);
            spawn_top_bar(parent, &progress, window);
            spawn_question_container(parent);
        });
}

fn spawn_top_bar(parent: &mut ChildSpawnerCommands, progress: &str, window: Entity) {
    parent
        .spawn(Node {
            width: percent(100.0),
            flex_direction: FlexDirection::Row,
            justify_content: JustifyContent::SpaceBetween,
            align_items: AlignItems::Center,
            ..default()
        })
        .with_children(|bar| {
            // Progress text (left)
            bar.spawn((
                Node {
                    padding: UiRect::axes(
                        theme::scaled(theme::spacing::MEDIUM),
                        theme::scaled(theme::spacing::SMALL),
                    ),
                    border: UiRect::all(px(1.0)),
                    border_radius: BorderRadius::all(theme::scaled(
                        theme::sizes::BUTTON_BORDER_RADIUS,
                    )),
                    ..default()
                },
                BackgroundColor(theme::colors::CARD_OVERLAY),
                BorderColor::all(theme::colors::CARD_BORDER),
                children![(
                    Text::new(progress),
                    theme::typography::text(theme::fonts::HEADING, window),
                    TextColor(theme::colors::TEXT_DARK),
                    ProgressText,
                )],
            ));
            // Quit button (right, compact icon button matching save-slot style)
            bar.spawn((
                icon_button(
                    32.0,
                    6.0,
                    "X",
                    theme::fonts::SMALL,
                    theme::button_styles::NEUTRAL,
                    window,
                ),
                QuitLessonButton,
            ));
        });
}

fn spawn_question_container(parent: &mut ChildSpawnerCommands) {
    parent.spawn((
        Node {
            flex_direction: FlexDirection::Column,
            align_items: AlignItems::Center,
            justify_content: JustifyContent::Center,
            flex_grow: 1.0,
            flex_basis: px(0.0),
            min_width: px(0.0),
            min_height: px(0.0),
            width: percent(100.0),
            row_gap: theme::scaled(theme::spacing::LARGE),
            padding: theme::scaled(theme::spacing::LARGE).all(),
            ..default()
        },
        QuestionContainer,
    ));
}
