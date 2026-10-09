use bevy::prelude::*;
use bevy::ui_widgets::MenuPlugin;

mod data;
mod i18n;
mod plugins;
mod questions;
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
use screens::{
    HomeScreenPlugin, LessonPlayScreenPlugin, LessonSummaryScreenPlugin, SaveSlotsScreenPlugin,
    SettingsScreenPlugin, TeacherLessonsScreenPlugin, TeacherRosterScreenPlugin,
    TeacherStatsScreenPlugin, ThemeExplorationScreenPlugin,
};
use states::{ActiveLesson, AppState, ExplorationView, InLessonFlow, LessonPhase};
use ui::{
    FocusNavigationPlugin, NavigationPlugin, ScrollPlugin, TextInputPlugin, ThemePlugin,
    UiAnimationPlugin,
};

fn main() {
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
            ContentPlugin,
            FocusNavigationPlugin,
            GameAudioPlugin,
            LessonMascotPlugin,
            NavigationPlugin,
            PersistencePlugin::default(),
            ScrollPlugin,
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
