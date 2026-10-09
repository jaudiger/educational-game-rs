use bevy::input_focus::AutoFocus;
use bevy::prelude::*;
use bevy::window::PrimaryWindow;
use bevy_persistent::prelude::Persistent;

use crate::data::{
    GameMode, GameSettings, LessonProgress, LessonSession, PlayerSession, SaveData,
    SaveWriteAction, SaveWriteStatus, SelectedLesson, update_save_data,
};
use crate::i18n::{I18n, TranslationKey};
use crate::states::AppState;
use crate::ui::components::{save_write_failure_notice, screen_root, standard_button};
use crate::ui::navigation::NavigateTo;
use crate::ui::theme;

/// End-of-lesson summary screen showing final scores and a return button.
pub struct LessonSummaryScreenPlugin;

impl Plugin for LessonSummaryScreenPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(
            OnEnter(AppState::LessonSummary),
            (save_lesson_progress, setup_lesson_summary).chain(),
        );
    }
}

fn save_lesson_progress(
    session: Res<LessonSession>,
    selected_lesson: Option<Res<SelectedLesson>>,
    player_session: Option<Res<PlayerSession>>,
    settings: Res<Persistent<GameSettings>>,
    mut save_data: ResMut<Persistent<SaveData>>,
    mut write_status: ResMut<SaveWriteStatus>,
) {
    // Class mode: scores are already recorded per-answer during LessonPlay.
    if settings.mode == GameMode::Group {
        return;
    }
    write_status.clear();

    // Guard: need lesson ID and active slot
    let Some(ref selected) = selected_lesson else {
        return;
    };
    let Some(ref lesson_id) = selected.0 else {
        return;
    };
    let Some(ref slot) = player_session else {
        return;
    };

    if session.total_answered == 0 {
        return;
    }

    // Build LessonProgress from the session's per-type breakdown.
    let new_progress = LessonProgress {
        type_scores: session.type_scores.clone(),
    };
    let lesson_id = lesson_id.clone();
    let slot_index = slot.slot_index;

    update_save_data(
        &mut save_data,
        &mut write_status,
        SaveWriteAction::IndividualLessonProgress,
        |data| {
            // Individual mode: replace the existing entry (last score policy).
            if let Some(ref mut save) = data.individual_slots[slot_index] {
                save.progress
                    .insert(lesson_id.clone(), new_progress.clone());
            }
        },
    );
}

fn setup_lesson_summary(
    mut commands: Commands,
    session: Res<LessonSession>,
    i18n: Res<I18n>,
    primary_window: Single<Entity, With<PrimaryWindow>>,
    write_status: Res<SaveWriteStatus>,
) {
    let window = *primary_window;
    let correct = session.correct_count;
    let total = session.total_answered;
    let percentage = (correct * 100).checked_div(total).unwrap_or(0);

    let message_key = if percentage == 100 {
        TranslationKey::SummaryPerfect
    } else if percentage >= 50 {
        TranslationKey::SummaryGood
    } else {
        TranslationKey::SummaryEncouragement
    };

    let show_save_failure = write_status.failed(SaveWriteAction::ClassAnswer)
        || write_status.failed(SaveWriteAction::IndividualLessonProgress);

    commands
        .spawn((screen_root(), DespawnOnExit(AppState::LessonSummary)))
        .with_children(|parent| {
            parent.spawn(summary_title(&i18n, window));
            parent.spawn(summary_score(&i18n, correct, total, window));
            parent.spawn(summary_percentage(&i18n, percentage, window));
            parent.spawn(summary_message(&i18n, &message_key, window));
            if show_save_failure {
                parent.spawn(save_write_failure_notice(
                    i18n.t(&TranslationKey::SaveWriteFailed).into_owned(),
                    window,
                ));
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
            theme::colors::PRIMARY,
            theme::scaled(theme::sizes::BUTTON_WIDTH),
            window,
        ),
        NavigateTo(AppState::ThemeExploration),
        AutoFocus,
    )
}
