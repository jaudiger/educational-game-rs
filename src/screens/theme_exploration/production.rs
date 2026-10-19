use bevy::prelude::*;
use std::collections::HashMap;

use crate::data::content::Theme;
use crate::data::{
    ContentLibrary, GameMode, LessonLaunchSelection, LessonProgress, PersistenceAction,
    PersistenceStatus, ProductionPlayerContext, get_current_progress,
};
use crate::i18n::{I18n, ProductionTranslationKey, TranslationKey};
use crate::states::{AppState, ExplorationView};

use super::{
    ExplorationProgressText, LessonActionPresentation, LessonButtonData, LessonLaunchAction,
    ThemeButtonData, ThemeLessonsPresentation, ThemeOverviewPresentation,
};

pub fn prepare_themes(
    content: Res<ContentLibrary>,
    ctx: ProductionPlayerContext<'_>,
    i18n: Res<I18n>,
    status: Res<PersistenceStatus>,
    mut presentation: ResMut<ThemeOverviewPresentation>,
) {
    let progress = if exploration_progress_visible(ctx.production.mode) {
        ctx.session.as_ref().and_then(|session| {
            get_current_progress(
                &ctx.save_data,
                ctx.production.mode,
                session.slot_index,
                ctx.active_student.as_deref().map(|student| student.0),
            )
        })
    } else {
        None
    };
    let themes = content
        .themes
        .iter()
        .map(|theme| {
            let (completed, total) = progress.map_or((0, theme.lessons.len()), |progress| {
                count_completed_for_theme(theme, progress)
            });
            ThemeButtonData {
                id: theme.id.clone(),
                available: theme.available,
                title_text: theme.title.get(i18n.language).to_owned(),
                coming_soon_text: i18n.t(&TranslationKey::ComingSoon).into_owned(),
                progress_text: (completed > 0).then(|| {
                    i18n.t(&ProductionTranslationKey::LessonsCompleted(
                        completed, total,
                    ))
                    .into_owned()
                }),
                track_progress: true,
            }
        })
        .collect();
    let next = ThemeOverviewPresentation {
        title: i18n.t(&TranslationKey::Themes).into_owned(),
        back_label: i18n.t(&TranslationKey::Back).into_owned(),
        back_destination: AppState::SaveSlots,
        notice: status.failed(PersistenceAction::SaveSlot).then(|| {
            i18n.t(&ProductionTranslationKey::SaveWriteFailed)
                .into_owned()
        }),
        themes,
    };
    if *presentation != next {
        *presentation = next;
    }
}

pub fn refresh_exploration_notices(
    status: Res<PersistenceStatus>,
    i18n: Res<I18n>,
    mut overview: ResMut<ThemeOverviewPresentation>,
    mut lessons: ResMut<ThemeLessonsPresentation>,
) {
    let notice = status.failed(PersistenceAction::SaveSlot).then(|| {
        i18n.t(&ProductionTranslationKey::SaveWriteFailed)
            .into_owned()
    });
    if overview.notice != notice {
        overview.notice.clone_from(&notice);
    }
    if lessons.notice != notice {
        lessons.notice = notice;
    }
}

pub fn prepare_theme_lessons(
    content: Res<ContentLibrary>,
    active_theme: Res<crate::data::ActiveTheme>,
    ctx: ProductionPlayerContext<'_>,
    i18n: Res<I18n>,
    status: Res<PersistenceStatus>,
    mut presentation: ResMut<ThemeLessonsPresentation>,
) {
    let next =
        content
            .theme(&active_theme)
            .map_or_else(ThemeLessonsPresentation::default, |theme| {
                let progress = if exploration_progress_visible(ctx.production.mode) {
                    ctx.session.as_ref().and_then(|session| {
                        get_current_progress(
                            &ctx.save_data,
                            ctx.production.mode,
                            session.slot_index,
                            ctx.active_student.as_deref().map(|student| student.0),
                        )
                    })
                } else {
                    None
                };
                let actions = [
                    LessonActionPresentation {
                        action: LessonLaunchAction::CompleteSeries,
                        label: i18n.t(&ProductionTranslationKey::PlaySeries).into_owned(),
                        selection: Some(LessonLaunchSelection::CompleteLesson),
                    },
                    LessonActionPresentation {
                        action: LessonLaunchAction::ChooseActivity,
                        label: i18n
                            .t(&ProductionTranslationKey::ChooseActivity)
                            .into_owned(),
                        selection: Some(LessonLaunchSelection::CompleteLesson),
                    },
                ];
                let lessons = theme
                    .lessons
                    .iter()
                    .map(|lesson| {
                        let best_percent = progress
                            .and_then(|progress| progress.get(&lesson.id))
                            .map(LessonProgress::percentage)
                            .map(|percentage| {
                                i18n.t(&ProductionTranslationKey::BestPercent(percentage))
                                    .into_owned()
                            });
                        LessonButtonData {
                            id: lesson.id.clone(),
                            title_text: lesson.title.get(i18n.language).to_owned(),
                            best_percent_text: best_percent,
                            track_progress: true,
                            actions: actions.to_vec(),
                        }
                    })
                    .collect();
                ThemeLessonsPresentation {
                    available: true,
                    title: theme.title.get(i18n.language).to_owned(),
                    back_label: i18n.t(&TranslationKey::BackToThemes).into_owned(),
                    notice: status.failed(PersistenceAction::SaveSlot).then(|| {
                        i18n.t(&ProductionTranslationKey::SaveWriteFailed)
                            .into_owned()
                    }),
                    clear_launch_context: false,
                    lessons,
                }
            });
    if *presentation != next {
        *presentation = next;
    }
}

fn exploration_progress_visible(mode: GameMode) -> bool {
    mode == GameMode::Individual
}

fn count_completed_for_theme(
    theme: &Theme,
    progress: &HashMap<String, LessonProgress>,
) -> (usize, usize) {
    let completed = theme
        .lessons
        .iter()
        .filter(|lesson| progress.contains_key(&lesson.id))
        .count();
    (completed, theme.lessons.len())
}

pub fn update_exploration_progress(
    ctx: ProductionPlayerContext<'_>,
    content: Res<ContentLibrary>,
    i18n: Res<I18n>,
    active_theme: Option<Res<crate::data::ActiveTheme>>,
    exploration_view: Res<State<ExplorationView>>,
    mut progress_query: Query<(&ExplorationProgressText, &mut Text, &mut Node)>,
) {
    let theme_data = active_theme
        .as_deref()
        .and_then(|theme| content.theme(&theme.0));
    let progress = if exploration_progress_visible(ctx.production.mode) {
        ctx.session.as_deref().and_then(|session| {
            get_current_progress(
                &ctx.save_data,
                ctx.production.mode,
                session.slot_index,
                ctx.active_student.as_deref().map(|student| student.0),
            )
        })
    } else {
        None
    };

    for (kind, mut text, mut node) in &mut progress_query {
        let label = if exploration_progress_visible(ctx.production.mode) {
            match (*exploration_view.get(), kind) {
                (ExplorationView::Themes, ExplorationProgressText::Theme(theme_id)) => {
                    content.theme(theme_id).and_then(|theme| {
                        let (completed, total) = progress.map_or((0, theme.lessons.len()), |p| {
                            count_completed_for_theme(theme, p)
                        });
                        (completed > 0).then(|| {
                            i18n.t(&ProductionTranslationKey::LessonsCompleted(
                                completed, total,
                            ))
                            .into_owned()
                        })
                    })
                }
                (ExplorationView::ThemeLessons, ExplorationProgressText::Lesson(lesson_id)) => {
                    theme_data
                        .and_then(|theme| theme.lesson(lesson_id))
                        .and_then(|lesson| progress.and_then(|progress| progress.get(&lesson.id)))
                        .map(|progress| {
                            i18n.t(&ProductionTranslationKey::BestPercent(
                                progress.percentage(),
                            ))
                            .into_owned()
                        })
                }
                _ => continue,
            }
        } else {
            None
        };
        set_exploration_progress_text(&mut text, &mut node, label);
    }
}

fn set_exploration_progress_text(text: &mut Text, node: &mut Node, label: Option<String>) {
    if let Some(label) = label {
        if **text != label {
            **text = label;
        }
        if node.display != Display::Flex {
            node.display = Display::Flex;
        }
    } else {
        if !text.is_empty() {
            **text = String::new();
        }
        if node.display != Display::None {
            node.display = Display::None;
        }
    }
}
