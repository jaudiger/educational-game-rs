use std::path::PathBuf;

use bevy::prelude::*;

use crate::data::{
    ActiveStudent, GameMode, LessonLaunchSelection, LessonScore, PersistenceAction,
    PersistenceStatus, ProductionPlayerContext, ProductionSettings,
};
use crate::i18n::{I18n, ProductionTranslationKey, TranslationKey};
use crate::plugins::{PersistencePlugin, TeacherPlugin};
use crate::screens::home::{HomePresentation, HomeQuitAction};
use crate::screens::lesson_play::explanations::FeedbackPresentation;
use crate::screens::lesson_play::{LessonFeedbackSet, session};
use crate::screens::settings::{
    ModePresentation, ModeSelectionRequested, SettingsPresentation, SettingsScreenSet,
};
use crate::screens::theme_exploration::production as theme_production;
use crate::screens::theme_exploration::{
    ExplorationPresentationSet, setup_theme_lessons, setup_themes,
};
use crate::screens::{
    LessonSummaryScreenPlugin, SaveSlotsScreenPlugin, TeacherLessonsScreenPlugin,
    TeacherRosterScreenPlugin, TeacherStatsScreenPlugin,
    lesson_play::layout::{self, LessonPlanningInput},
};
use crate::states::{
    ActiveLesson, AppState, ExplorationView, InLessonFlow, LessonPhase, StateScopedResourceExt,
};
use crate::ui::TextInputPlugin;
use crate::ui::components::ButtonActivated;
use crate::ui::navigation::AppNavigationPolicy;

pub struct ProductionEditionPlugin {
    persistence_directory: PathBuf,
}

impl ProductionEditionPlugin {
    pub const fn new(persistence_directory: PathBuf) -> Self {
        Self {
            persistence_directory,
        }
    }
}

impl Plugin for ProductionEditionPlugin {
    fn build(&self, app: &mut App) {
        app.insert_resource(AppNavigationPolicy::default())
            .add_plugins((
                PersistencePlugin::new(self.persistence_directory.clone()),
                TeacherPlugin,
                TextInputPlugin,
                SaveSlotsScreenPlugin,
                LessonSummaryScreenPlugin,
                TeacherLessonsScreenPlugin,
                TeacherRosterScreenPlugin,
                TeacherStatsScreenPlugin,
            ))
            .add_systems(
                Update,
                crate::ui::components::dispatch_confirmation_button_actions,
            )
            .register_state_scoped_resource::<ActiveLesson, LessonScore>(ActiveLesson)
            .register_state_scoped_resource::<InLessonFlow, ActiveStudent>(InLessonFlow)
            .add_systems(
                OnEnter(ExplorationView::Themes),
                theme_production::prepare_themes.before(setup_themes),
            )
            .add_systems(
                OnEnter(ExplorationView::ThemeLessons),
                theme_production::prepare_theme_lessons.before(setup_theme_lessons),
            )
            .add_systems(
                Update,
                theme_production::update_exploration_progress.run_if(
                    in_state(AppState::ThemeExploration).and_then(
                        resource_changed_or_removed::<ActiveStudent>
                            .or_else(resource_changed::<crate::data::SaveData>),
                    ),
                ),
            )
            .add_systems(
                Update,
                theme_production::refresh_exploration_notices
                    .in_set(ExplorationPresentationSet::Prepare)
                    .run_if(in_state(AppState::ThemeExploration).and_then(
                        resource_changed::<PersistenceStatus>.or_else(resource_changed::<I18n>),
                    )),
            )
            .add_systems(
                OnEnter(AppState::Home),
                prepare_production_home.before(crate::screens::home::setup_home),
            )
            .add_systems(Update, handle_quit.run_if(in_state(AppState::Home)))
            .add_observer(update_production_mode)
            .add_systems(
                OnEnter(AppState::Settings),
                prepare_settings_presentation.before(crate::screens::settings::setup_settings),
            )
            .add_systems(
                Update,
                prepare_settings_presentation
                    .in_set(SettingsScreenSet::Prepare)
                    .run_if(
                        resource_changed::<I18n>.or_else(resource_changed::<PersistenceStatus>),
                    ),
            )
            .add_systems(
                OnEnter(AppState::LessonPlay),
                (
                    initialize_lesson_score,
                    prepare_lesson_planning.before(layout::setup_lesson_play),
                ),
            )
            .add_systems(
                OnEnter(LessonPhase::ShowFeedback),
                session::record_answer.in_set(LessonFeedbackSet::RecordAnswer),
            )
            .add_systems(
                OnEnter(LessonPhase::ShowFeedback),
                prepare_production_feedback.in_set(LessonFeedbackSet::PreparePresentation),
            )
            .add_systems(
                OnEnter(LessonPhase::ShowQuestion),
                session::clear_active_student,
            );
    }
}

fn initialize_lesson_score(mut commands: Commands) {
    commands.insert_resource(LessonScore::default());
}

fn prepare_lesson_planning(
    mut commands: Commands,
    ctx: ProductionPlayerContext<'_>,
    selected_lesson: Res<crate::data::SelectedLesson>,
    launch_selection: Option<Res<LessonLaunchSelection>>,
    mut status: ResMut<crate::data::PersistenceStatus>,
) {
    status.clear_save_data();
    let default_selection = LessonLaunchSelection::CompleteLesson;
    let selection = launch_selection.as_deref().unwrap_or(&default_selection);
    let lesson_config = if matches!(selection, LessonLaunchSelection::CompleteLesson)
        && ctx.production.mode == crate::data::GameMode::Group
    {
        ctx.session.as_ref().and_then(|slot| {
            ctx.save_data.class_slots[slot.slot_index]
                .as_ref()
                .and_then(|class_save| {
                    selected_lesson
                        .0
                        .as_ref()
                        .and_then(|lesson_id| class_save.lesson_configs.get(lesson_id))
                })
        })
    } else {
        None
    };
    let planning = lesson_config.map(crate::data::LessonSessionConfig::planning_input);
    commands.insert_resource(LessonPlanningInput(planning));
}

fn prepare_production_home(i18n: Res<I18n>, mut presentation: ResMut<HomePresentation>) {
    *presentation = HomePresentation {
        title: i18n.t(&TranslationKey::AppTitle).into_owned(),
        play: i18n.t(&TranslationKey::Play).into_owned(),
        settings: i18n.t(&TranslationKey::Settings).into_owned(),
        quit: Some(i18n.t(&ProductionTranslationKey::Quit).into_owned()),
        play_destination: AppState::SaveSlots,
    };
}

fn handle_quit(
    mut activations: MessageReader<ButtonActivated>,
    query: Query<(), With<HomeQuitAction>>,
    mut app_exit: MessageWriter<AppExit>,
) {
    for activation in activations.read() {
        if query.contains(activation.0) {
            app_exit.write(AppExit::Success);
        }
    }
}

fn prepare_settings_presentation(
    settings: Res<ProductionSettings>,
    status: Res<PersistenceStatus>,
    i18n: Res<I18n>,
    mut presentation: ResMut<SettingsPresentation>,
) {
    let next = SettingsPresentation {
        mode: Some(ModePresentation {
            label: i18n.t(&ProductionTranslationKey::Mode).into_owned(),
            individual_label: i18n
                .t(&ProductionTranslationKey::ModeIndividual)
                .into_owned(),
            group_label: i18n.t(&ProductionTranslationKey::ModeClass).into_owned(),
            group_selected: settings.mode == GameMode::Group,
        }),
        notice: status.failed(PersistenceAction::GameSettings).then(|| {
            i18n.t(&ProductionTranslationKey::SaveWriteFailed)
                .into_owned()
        }),
    };
    if *presentation != next {
        *presentation = next;
    }
}

fn update_production_mode(
    event: On<ModeSelectionRequested>,
    mut settings: crate::data::ProductionSettingsMut<'_>,
) {
    let mode = if event.0 {
        GameMode::Group
    } else {
        GameMode::Individual
    };
    if settings.settings.mode != mode {
        settings.update(|settings| settings.mode = mode);
    }
}

fn prepare_production_feedback(
    session: Res<crate::data::LessonSession>,
    launch_selection: Option<Res<LessonLaunchSelection>>,
    i18n: Res<I18n>,
    status: Res<PersistenceStatus>,
    mut presentation: ResMut<FeedbackPresentation>,
) {
    let default_selection = LessonLaunchSelection::CompleteLesson;
    let selection = launch_selection.as_deref().unwrap_or(&default_selection);
    let (label, return_to_catalogue) = match selection {
        LessonLaunchSelection::SingleActivity { .. } => (
            i18n.t(&TranslationKey::ReturnToActivityList).into_owned(),
            true,
        ),
        LessonLaunchSelection::CompleteLesson
            if session.current_index + 1 >= session.questions.len() =>
        {
            (
                i18n.t(&ProductionTranslationKey::FinishLesson).into_owned(),
                false,
            )
        }
        LessonLaunchSelection::CompleteLesson => (
            i18n.t(&ProductionTranslationKey::NextQuestion).into_owned(),
            false,
        ),
    };
    let next = FeedbackPresentation {
        next_button_label: label,
        return_to_catalogue,
        save_notice: status.failed(PersistenceAction::QuestionAnswer).then(|| {
            i18n.t(&ProductionTranslationKey::SaveWriteFailed)
                .into_owned()
        }),
    };
    if *presentation != next {
        *presentation = next;
    }
}
