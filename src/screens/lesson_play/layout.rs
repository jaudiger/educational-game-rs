use bevy::input_focus::tab_navigation::TabGroup;
use bevy::prelude::*;
use bevy::window::PrimaryWindow;

use crate::data::{
    ActiveTheme, ContentLibrary, GameMode, PlayerContext, QuestionContainer, SelectedLesson,
};
use crate::i18n::{I18n, TranslationKey};
use crate::plugins::lesson_mascot::spawn_lesson_mascot;
use crate::states::AppState;
use crate::ui::components::icon_button;
use crate::ui::theme::{self, DesignFontSize, GameImages};

use super::{ProgressText, QuitLessonButton, session};

#[allow(clippy::too_many_arguments)]
pub(super) fn setup_lesson_play(
    mut commands: Commands,
    asset_server: Res<AssetServer>,
    images: Res<GameImages>,
    selected_lesson: Res<SelectedLesson>,
    content: Res<ContentLibrary>,
    active_theme: Res<ActiveTheme>,
    i18n: Res<I18n>,
    ctx: PlayerContext<'_>,
    primary_window: Single<Entity, With<PrimaryWindow>>,
) {
    let window = *primary_window;
    // Look up teacher config for class mode
    let lesson_config = if ctx.settings.mode == GameMode::Group {
        ctx.active_slot.as_ref().and_then(|slot| {
            ctx.save_data.class_slots[slot.0].as_ref().and_then(|cs| {
                selected_lesson
                    .0
                    .as_ref()
                    .and_then(|lid| cs.lesson_configs.get(lid))
            })
        })
    } else {
        None
    };
    let session = session::build_session(&selected_lesson, &content, &active_theme, lesson_config);
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
                ..default()
            },
            DespawnOnExit(AppState::LessonPlay),
            TabGroup::new(0),
        ))
        .with_children(|parent| {
            // Mascot first: absolutely positioned, renders behind siblings.
            spawn_lesson_mascot(parent, ctx.settings.map_theme, &asset_server);
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
                    Text::new(progress.to_owned()),
                    TextFont {
                        font_size: theme::fonts::HEADING,
                        ..default()
                    },
                    TextColor(theme::colors::TEXT_DARK),
                    ProgressText,
                    DesignFontSize {
                        size: theme::fonts::HEADING,
                        window,
                    },
                )],
            ));
            // Quit button (right, compact icon button matching save-slot style)
            bar.spawn((
                icon_button(
                    32.0,
                    6.0,
                    "X",
                    theme::fonts::SMALL,
                    theme::colors::ERROR,
                    theme::colors::TEXT_LIGHT,
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
            width: percent(100.0),
            row_gap: theme::scaled(theme::spacing::LARGE),
            padding: theme::scaled(theme::spacing::LARGE).all(),
            border: UiRect::all(px(1.0)),
            border_radius: BorderRadius::all(theme::scaled(theme::sizes::CARD_BORDER_RADIUS)),
            ..default()
        },
        BackgroundColor(theme::colors::CARD_OVERLAY),
        BorderColor::all(theme::colors::CARD_BORDER),
        QuestionContainer,
    ));
}
