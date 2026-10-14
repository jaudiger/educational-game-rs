use bevy::prelude::*;
use bevy::ui_widgets::MenuPlugin;

mod data;
mod i18n;
mod plugins;
mod questions;
mod runtime_paths;
mod screens;
mod states;
mod ui;

use plugins::{
    BalloonCursorPlugin, ContentPlugin, GameAudioPlugin, LessonMascotPlugin, PersistencePlugin,
    SettingsPlugin, SkyBackgroundPlugin, TeacherPlugin,
};
use questions::{
    FractionComparisonPlugin, FractionIdentificationPlugin, FractionVisualizationPlugin, McqPlugin,
    NumericInputPlugin,
};
use runtime_paths::RuntimePaths;
use screens::{
    ExerciseSelectionScreenPlugin, HomeScreenPlugin, LessonPlayScreenPlugin,
    LessonSummaryScreenPlugin, SaveSlotsScreenPlugin, SettingsScreenPlugin,
    TeacherLessonsScreenPlugin, TeacherRosterScreenPlugin, TeacherStatsScreenPlugin,
    ThemeExplorationScreenPlugin,
};
use states::{ActiveLesson, AppState, ExplorationView, InLessonFlow, LessonPhase};
use ui::{
    ButtonActivationPlugin, FocusNavigationPlugin, NavigationPlugin, TextInputPlugin, ThemePlugin,
    UiAnimationPlugin,
};

fn main() {
    let paths = RuntimePaths::resolve()
        .unwrap_or_else(|error| panic!("Application path setup failed: {error}"));
    let asset_directory = paths.asset_directory().to_string_lossy().into_owned();

    App::new()
        .add_plugins(
            DefaultPlugins
                .set(WindowPlugin {
                    primary_window: Some(Window {
                        title: "Educational Game".into(),
                        ..default()
                    }),
                    ..default()
                })
                .set(AssetPlugin {
                    file_path: asset_directory,
                    ..default()
                })
                .disable::<MenuPlugin>(),
        )
        // States
        .init_state::<AppState>()
        .add_computed_state::<InLessonFlow>()
        .add_computed_state::<ActiveLesson>()
        .add_sub_state::<LessonPhase>()
        .add_sub_state::<ExplorationView>()
        // Plugins
        .add_plugins((
            BalloonCursorPlugin,
            ButtonActivationPlugin,
            ContentPlugin::new(paths.content_directory()),
            FocusNavigationPlugin,
            GameAudioPlugin,
            LessonMascotPlugin,
            NavigationPlugin,
            PersistencePlugin::new(paths.persistence_directory()),
            SettingsPlugin,
            SkyBackgroundPlugin,
            TeacherPlugin,
            TextInputPlugin,
            ThemePlugin,
            UiAnimationPlugin,
        ))
        // Question plugins
        .add_plugins((
            McqPlugin,
            FractionVisualizationPlugin,
            FractionComparisonPlugin,
            FractionIdentificationPlugin,
            NumericInputPlugin,
        ))
        // Screen plugins
        .add_plugins((
            HomeScreenPlugin,
            SaveSlotsScreenPlugin,
            ThemeExplorationScreenPlugin,
            ExerciseSelectionScreenPlugin,
            LessonPlayScreenPlugin,
            LessonSummaryScreenPlugin,
            SettingsScreenPlugin,
            TeacherLessonsScreenPlugin,
            TeacherRosterScreenPlugin,
            TeacherStatsScreenPlugin,
        ))
        .add_systems(Startup, setup)
        .run();
}

fn setup(mut commands: Commands) {
    commands.spawn((Camera2d, IsDefaultUiCamera));
}
