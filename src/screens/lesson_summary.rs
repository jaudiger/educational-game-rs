use bevy::input_focus::AutoFocus;
use bevy::prelude::*;
use bevy::window::PrimaryWindow;

use crate::data::LessonScore;
use crate::i18n::{I18n, TranslationKey};
use crate::states::AppState;
use crate::ui::components::{save_write_failure_notice, screen_root, standard_button};
use crate::ui::navigation::NavigateTo;
use crate::ui::theme;

/// End-of-lesson summary screen showing final scores and a return button.
pub struct LessonSummaryScreenPlugin;

#[derive(SystemSet, Clone, Copy, Debug, Eq, Hash, PartialEq)]
pub enum LessonSummarySet {
    Prepare,
    Render,
}

#[derive(Resource, Clone, Debug, Default, PartialEq, Eq)]
pub struct LessonSummaryPresentation {
    pub save_notice: Option<String>,
}

impl Plugin for LessonSummaryScreenPlugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<LessonSummaryPresentation>()
            .configure_sets(
                OnEnter(AppState::LessonSummary),
                (LessonSummarySet::Prepare, LessonSummarySet::Render).chain(),
            )
            .add_systems(
                OnEnter(AppState::LessonSummary),
                setup_lesson_summary.in_set(LessonSummarySet::Render),
            );
    }
}

fn setup_lesson_summary(
    mut commands: Commands,
    score: Res<LessonScore>,
    presentation: Res<LessonSummaryPresentation>,
    i18n: Res<I18n>,
    primary_window: Single<Entity, With<PrimaryWindow>>,
) {
    let window = *primary_window;
    let correct = score.correct_count;
    let total = score.total_answered;
    let percentage = (correct * 100).checked_div(total).unwrap_or(0);

    let message_key = if percentage == 100 {
        TranslationKey::SummaryPerfect
    } else if percentage >= 50 {
        TranslationKey::SummaryGood
    } else {
        TranslationKey::SummaryEncouragement
    };

    let save_notice = presentation.save_notice.clone();
    commands
        .spawn((screen_root(px(0.0)), DespawnOnExit(AppState::LessonSummary)))
        .with_children(|parent| {
            parent.spawn(summary_title(&i18n, window));
            parent.spawn(summary_score(&i18n, correct, total, window));
            parent.spawn(summary_percentage(&i18n, percentage, window));
            parent.spawn(summary_message(&i18n, &message_key, window));
            if let Some(notice) = save_notice {
                parent.spawn(save_write_failure_notice(notice, window));
            }
            parent.spawn(return_button(&i18n, window));
        });
}

fn summary_title(i18n: &I18n, window: Entity) -> impl Bundle + use<> {
    (
        Text::new(i18n.t(&TranslationKey::SummaryTitle)),
        theme::typography::text(theme::fonts::TITLE, window),
        TextColor(theme::colors::TEXT_DARK),
    )
}

fn summary_score(i18n: &I18n, correct: u32, total: u32, window: Entity) -> impl Bundle + use<> {
    (
        Text::new(i18n.t(&TranslationKey::SummaryScore(correct, total))),
        theme::typography::text(theme::fonts::HEADING, window),
        TextColor(theme::colors::TEXT_DARK),
    )
}

fn summary_percentage(i18n: &I18n, percentage: u32, window: Entity) -> impl Bundle + use<> {
    (
        Text::new(i18n.t(&TranslationKey::SummaryPercentage(percentage))),
        theme::typography::text(theme::fonts::HEADING, window),
        TextColor(theme::colors::PRIMARY),
    )
}

fn summary_message(i18n: &I18n, key: &TranslationKey, window: Entity) -> impl Bundle + use<> {
    let color = match key {
        TranslationKey::SummaryPerfect => theme::colors::SUCCESS,
        TranslationKey::SummaryGood => theme::colors::PRIMARY,
        _ => theme::colors::SECONDARY,
    };

    (
        Text::new(i18n.t(key)),
        theme::typography::text(theme::fonts::HEADING, window),
        TextColor(color),
    )
}

fn return_button(i18n: &I18n, window: Entity) -> impl Bundle + use<> {
    (
        standard_button(
            &i18n.t(&TranslationKey::Back),
            theme::button_styles::BACK,
            theme::scaled(theme::sizes::BUTTON_WIDTH),
            window,
        ),
        NavigateTo(AppState::ThemeExploration),
        AutoFocus,
    )
}
