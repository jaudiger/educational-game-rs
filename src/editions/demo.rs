use bevy::prelude::*;

use crate::data::{ActiveTheme, ContentLibrary, GameSettings};
use crate::i18n::I18n;
use crate::screens::home::HomePresentation;
use crate::screens::lesson_play::LessonFeedbackSet;
use crate::screens::lesson_play::explanations::FeedbackPresentation;
use crate::screens::lesson_play::layout::{
    LessonPlanningInput, RejectedLessonLaunch, setup_lesson_play,
};
use crate::screens::theme_exploration::{
    LessonActionPresentation, LessonButtonData, LessonLaunchAction, ThemeButtonData,
    ThemeLessonsPresentation, ThemeOverviewPresentation, setup_theme_lessons, setup_themes,
};
use crate::states::AppState;
use crate::states::{ExplorationView, LessonPhase};
use crate::ui::navigation::AppNavigationPolicy;

pub struct DemoEditionPlugin;

impl Plugin for DemoEditionPlugin {
    fn build(&self, app: &mut App) {
        let settings = GameSettings::default();
        app.insert_resource(AppNavigationPolicy::new([
            AppState::Home,
            AppState::ThemeExploration,
            AppState::LessonPlay,
            AppState::Settings,
        ]))
        .insert_resource(I18n::new(settings.language))
        .insert_resource(settings)
        .add_systems(
            OnEnter(AppState::Home),
            prepare_demo_home.before(crate::screens::home::setup_home),
        )
        .add_systems(
            OnEnter(ExplorationView::Themes),
            prepare_demo_themes.before(setup_themes),
        )
        .add_systems(
            OnEnter(ExplorationView::ThemeLessons),
            prepare_demo_theme_lessons.before(setup_theme_lessons),
        )
        .add_systems(
            OnEnter(AppState::LessonPlay),
            validate_demo_lesson_launch.before(setup_lesson_play),
        )
        .add_systems(
            OnEnter(LessonPhase::ShowFeedback),
            prepare_demo_feedback.in_set(LessonFeedbackSet::PreparePresentation),
        );
    }
}

fn validate_demo_lesson_launch(
    mut commands: Commands,
    selection: Option<Res<crate::data::LessonLaunchSelection>>,
    mut next_state: ResMut<NextState<AppState>>,
) {
    commands.remove_resource::<RejectedLessonLaunch>();
    commands.insert_resource(LessonPlanningInput::default());
    if !matches!(
        selection.as_deref(),
        Some(crate::data::LessonLaunchSelection::SingleExercise { .. })
    ) {
        bevy::log::error!("Demo requires a single-exercise launch");
        commands.insert_resource(RejectedLessonLaunch);
        next_state.set(AppState::ThemeExploration);
    }
}

fn prepare_demo_home(i18n: Res<I18n>, mut presentation: ResMut<HomePresentation>) {
    *presentation = HomePresentation {
        title: i18n.t(&crate::i18n::TranslationKey::AppTitle).into_owned(),
        play: i18n.t(&crate::i18n::TranslationKey::Play).into_owned(),
        settings: i18n.t(&crate::i18n::TranslationKey::Settings).into_owned(),
        quit: None,
        play_destination: AppState::ThemeExploration,
    };
}

fn prepare_demo_themes(
    content: Res<ContentLibrary>,
    i18n: Res<I18n>,
    mut presentation: ResMut<ThemeOverviewPresentation>,
) {
    *presentation = ThemeOverviewPresentation {
        title: i18n.t(&crate::i18n::TranslationKey::Themes).into_owned(),
        back_label: i18n.t(&crate::i18n::TranslationKey::Back).into_owned(),
        back_destination: AppState::Home,
        notice: None,
        themes: content
            .themes
            .iter()
            .map(|theme| ThemeButtonData {
                id: theme.id.clone(),
                available: theme.available,
                title_text: theme.title.get(i18n.language).to_owned(),
                coming_soon_text: i18n
                    .t(&crate::i18n::TranslationKey::ComingSoon)
                    .into_owned(),
                progress_text: None,
                track_progress: false,
            })
            .collect(),
    };
}

fn prepare_demo_theme_lessons(
    content: Res<ContentLibrary>,
    active_theme: Res<ActiveTheme>,
    i18n: Res<I18n>,
    mut presentation: ResMut<ThemeLessonsPresentation>,
) {
    let Some(theme) = content.theme(&active_theme) else {
        *presentation = ThemeLessonsPresentation::default();
        return;
    };
    *presentation = ThemeLessonsPresentation {
        available: true,
        title: theme.title.get(i18n.language).to_owned(),
        back_label: i18n
            .t(&crate::i18n::TranslationKey::BackToThemes)
            .into_owned(),
        notice: None,
        clear_launch_context: true,
        lessons: theme
            .lessons
            .iter()
            .map(|lesson| LessonButtonData {
                id: lesson.id.clone(),
                title_text: lesson.title.get(i18n.language).to_owned(),
                best_percent_text: None,
                track_progress: false,
                actions: vec![LessonActionPresentation {
                    action: LessonLaunchAction::ChooseExercise,
                    label: String::new(),
                    selection: None,
                }],
            })
            .collect(),
    };
}

fn prepare_demo_feedback(i18n: Res<I18n>, mut presentation: ResMut<FeedbackPresentation>) {
    *presentation = FeedbackPresentation {
        next_button_label: i18n
            .t(&crate::i18n::TranslationKey::ReturnToExerciseList)
            .into_owned(),
        return_to_catalogue: true,
        save_notice: None,
    };
}
