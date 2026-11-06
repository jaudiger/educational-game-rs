use bevy::input_focus::tab_navigation::TabGroup;
use bevy::prelude::*;
use bevy::ui::InteractionDisabled;
use bevy::window::PrimaryWindow;

use crate::data::LessonScore;
use crate::i18n::{I18n, TranslationKey};
use crate::plugins::lesson_mascot::{load_sky_balloon_assets, spawn_summary_balloon};
use crate::states::AppState;
use crate::ui::animation::ScreenEntryAnimationDisabled;
use crate::ui::components::{save_write_failure_notice, standard_button};
use crate::ui::theme;

use super::LessonSummaryPresentation;
use super::flow::{SummaryPhase, SummaryScreen};
use super::theme::{SummaryThemeStyle, summary_backdrop, summary_curtain};

#[derive(Component, Reflect)]
#[require(UiTransform)]
pub(super) struct SummaryResultCard;

#[derive(Component, Reflect)]
pub(super) struct SummaryReturnButton;

struct SummaryCardPresentation {
    title: String,
    percentage: String,
    score: String,
    message: String,
    message_color: Color,
    percentage_value: u32,
    accent: Color,
    back_label: String,
    save_notice: Option<String>,
}

pub(super) fn setup_lesson_summary(
    mut commands: Commands,
    score: Res<LessonScore>,
    presentation: Res<LessonSummaryPresentation>,
    i18n: Res<I18n>,
    asset_server: Res<AssetServer>,
    primary_window: Single<Entity, With<PrimaryWindow>>,
) {
    let window = *primary_window;
    let percentage = score_percentage(*score);
    let message_key = if percentage == 100 {
        TranslationKey::SummaryPerfect
    } else if percentage >= 50 {
        TranslationKey::SummaryGood
    } else {
        TranslationKey::SummaryEncouragement
    };
    let message_color = match &message_key {
        TranslationKey::SummaryPerfect => theme::colors::SUCCESS,
        TranslationKey::SummaryGood => theme::colors::PRIMARY,
        _ => theme::colors::SECONDARY,
    };
    let style = SummaryThemeStyle::sky();
    let balloon_assets = load_sky_balloon_assets(&asset_server);
    let card = SummaryCardPresentation {
        title: i18n.t(&TranslationKey::SummaryTitle).into_owned(),
        percentage: i18n
            .t(&TranslationKey::SummaryPercentage(percentage))
            .into_owned(),
        score: i18n
            .t(&TranslationKey::SummaryScore(
                score.correct_count,
                score.total_answered,
            ))
            .into_owned(),
        message: i18n.t(&message_key).into_owned(),
        message_color,
        percentage_value: percentage,
        accent: style.accent,
        back_label: i18n.t(&TranslationKey::Back).into_owned(),
        save_notice: presentation.save_notice.clone(),
    };

    commands.spawn((
        Node {
            width: theme::layout::FILL_PARENT,
            height: theme::layout::FILL_PARENT,
            min_width: px(0.0),
            min_height: px(0.0),
            justify_content: JustifyContent::Center,
            align_items: AlignItems::Center,
            ..default()
        },
        BackgroundColor(style.background),
        TabGroup::new(0),
        ScreenEntryAnimationDisabled,
        SummaryScreen {
            phase: SummaryPhase::Revealing,
            elapsed: 0.0,
        },
        DespawnOnExit(AppState::LessonSummary),
        children![
            summary_backdrop(),
            (
                Node {
                    position_type: PositionType::Relative,
                    width: percent(88.0),
                    height: percent(82.0),
                    max_width: theme::scaled(1100.0),
                    min_width: px(0.0),
                    justify_content: JustifyContent::Center,
                    align_items: AlignItems::Center,
                    ..default()
                },
                Children::spawn(SpawnWith(move |content: &mut ChildSpawner| {
                    spawn_summary_balloon(content, balloon_assets);
                    content.spawn(summary_card(card, window));
                })),
            ),
            summary_curtain(style),
        ],
    ));
}

fn score_percentage(score: LessonScore) -> u32 {
    if score.total_answered == 0 {
        return 0;
    }
    let percentage = (u64::from(score.correct_count) * 100) / u64::from(score.total_answered);
    u32::try_from(percentage.min(100)).unwrap_or(100)
}

fn summary_card(data: SummaryCardPresentation, window: Entity) -> impl Bundle + use<> {
    (
        Node {
            width: percent(70.0),
            min_width: theme::scaled(300.0),
            max_width: theme::scaled(640.0),
            flex_direction: FlexDirection::Column,
            align_items: AlignItems::Center,
            row_gap: theme::scaled(theme::spacing::MEDIUM),
            padding: theme::scaled(32.0).all(),
            border: px(1.0).all(),
            border_radius: BorderRadius::all(theme::scaled(24.0)),
            ..default()
        },
        BackgroundColor(theme::colors::CARD_BG),
        BorderColor::all(theme::colors::CARD_BORDER),
        BoxShadow::new(
            Color::srgba(0.02, 0.08, 0.18, 0.18),
            px(0.0),
            px(10.0),
            px(28.0),
            px(0.0),
        ),
        SummaryResultCard,
        Children::spawn(SpawnWith(move |parent: &mut ChildSpawner| {
            parent.spawn(summary_title(&data.title, window));
            parent.spawn(summary_percentage(&data.percentage, data.accent, window));
            parent.spawn(score_progress_bar(data.percentage_value, data.accent));
            parent.spawn(summary_score(&data.score, window));
            parent.spawn(summary_message(&data.message, data.message_color, window));
            if let Some(notice) = data.save_notice {
                parent.spawn(save_write_failure_notice(notice, window));
            }
            parent.spawn((
                standard_button(
                    &data.back_label,
                    theme::button_styles::BACK,
                    theme::scaled(theme::sizes::BUTTON_WIDTH),
                    window,
                ),
                SummaryReturnButton,
                InteractionDisabled,
            ));
        })),
    )
}

fn summary_title(text: &str, window: Entity) -> impl Bundle + use<> {
    (
        Text::new(text),
        theme::typography::text(theme::fonts::HEADING, window),
        TextColor(theme::colors::TEXT_DARK),
        TextLayout::justify(Justify::Center),
    )
}

fn summary_percentage(text: &str, color: Color, window: Entity) -> impl Bundle + use<> {
    (
        Text::new(text),
        theme::typography::text(theme::fonts::HERO, window),
        TextColor(color),
        TextLayout::justify(Justify::Center),
    )
}

fn summary_score(text: &str, window: Entity) -> impl Bundle + use<> {
    (
        Text::new(text),
        theme::typography::text(theme::fonts::BODY, window),
        TextColor(theme::colors::TEXT_DARK),
        TextLayout::justify(Justify::Center),
    )
}

fn summary_message(text: &str, color: Color, window: Entity) -> impl Bundle + use<> {
    (
        Text::new(text),
        theme::typography::text(theme::fonts::HEADING, window),
        TextColor(color),
        TextLayout::justify(Justify::Center),
    )
}

fn score_progress_bar(percentage: u32, color: Color) -> impl Bundle + use<> {
    (
        Node {
            width: theme::layout::FILL_PARENT,
            height: theme::scaled(14.0),
            border_radius: BorderRadius::all(percent(50.0)),
            overflow: Overflow::clip(),
            ..default()
        },
        BackgroundColor(theme::colors::ACTIVITY_BADGE),
        children![(
            Node {
                width: percent(f32::from(u8::try_from(percentage.min(100)).unwrap_or(100))),
                height: theme::layout::FILL_PARENT,
                border_radius: BorderRadius::all(percent(50.0)),
                ..default()
            },
            BackgroundColor(color),
        )],
    )
}
