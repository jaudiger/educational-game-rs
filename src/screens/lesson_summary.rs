use bevy::input_focus::AutoFocus;
use bevy::prelude::*;
use bevy::window::PrimaryWindow;
use bevy_persistent::prelude::Persistent;

use crate::data::{
    ActiveSlot, GameMode, GameSettings, LessonProgress, LessonSession, SaveData, SelectedLesson,
};
use crate::i18n::{I18n, TranslationKey};
use crate::states::AppState;
use crate::ui::components::{screen_root, standard_button};
use crate::ui::navigation::NavigateTo;
use crate::ui::theme;
use crate::ui::theme::DesignFontSize;

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
    active_slot: Option<Res<ActiveSlot>>,
    settings: Res<Persistent<GameSettings>>,
    mut save_data: ResMut<Persistent<SaveData>>,
) {
    // Class mode: scores are already recorded per-answer during LessonPlay.
    if settings.mode == GameMode::Group {
        return;
    }

    // Guard: need lesson ID and active slot
    let Some(ref selected) = selected_lesson else {
        return;
    };
    let Some(ref lesson_id) = selected.0 else {
        return;
    };
    let Some(ref slot) = active_slot else {
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
    let slot_index = slot.0;

    save_data
        .update(|data| {
            // Individual mode: replace the existing entry (last score policy).
            if let Some(ref mut save) = data.individual_slots[slot_index] {
                save.progress
                    .insert(lesson_id.clone(), new_progress.clone());
            }
        })
        .expect("failed to save lesson progress");
}

fn setup_lesson_summary(
    mut commands: Commands,
    session: Res<LessonSession>,
    i18n: Res<I18n>,
    primary_window: Single<Entity, With<PrimaryWindow>>,
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

    commands.spawn((
        screen_root(),
        DespawnOnExit(AppState::LessonSummary),
        children![
            summary_title(&i18n, window),
            summary_score(&i18n, correct, total, window),
            summary_percentage(&i18n, percentage, window),
            summary_message(&i18n, &message_key, window),
            return_button(&i18n, window),
        ],
    ));
}

fn summary_title(i18n: &I18n, window: Entity) -> impl Bundle + use<> {
    (
        Text::new(i18n.t(&TranslationKey::SummaryTitle)),
        TextFont {
            font_size: theme::fonts::TITLE,
            ..default()
        },
        TextColor(theme::colors::TEXT_DARK),
        DesignFontSize {
            size: theme::fonts::TITLE,
            window,
        },
    )
}

fn summary_score(i18n: &I18n, correct: u32, total: u32, window: Entity) -> impl Bundle + use<> {
    (
        Text::new(i18n.t(&TranslationKey::SummaryScore(correct, total))),
        TextFont {
            font_size: theme::fonts::HEADING,
            ..default()
        },
        TextColor(theme::colors::TEXT_DARK),
        DesignFontSize {
            size: theme::fonts::HEADING,
            window,
        },
    )
}

fn summary_percentage(i18n: &I18n, percentage: u32, window: Entity) -> impl Bundle + use<> {
    (
        Text::new(i18n.t(&TranslationKey::SummaryPercentage(percentage))),
        TextFont {
            font_size: theme::fonts::HEADING,
            ..default()
        },
        TextColor(theme::colors::PRIMARY),
        DesignFontSize {
            size: theme::fonts::HEADING,
            window,
        },
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
        TextFont {
            font_size: theme::fonts::HEADING,
            ..default()
        },
        TextColor(color),
        DesignFontSize {
            size: theme::fonts::HEADING,
            window,
        },
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
        NavigateTo(AppState::MapExploration),
        AutoFocus,
    )
}
