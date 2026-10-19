#[cfg(feature = "demo")]
mod demo;
#[cfg(not(feature = "demo"))]
mod production;

use bevy::prelude::*;
use std::path::PathBuf;

pub struct SelectedEditionPlugin {
    #[cfg(not(feature = "demo"))]
    persistence_directory: PathBuf,
}

impl SelectedEditionPlugin {
    #[allow(clippy::missing_const_for_fn)]
    pub fn new(persistence_directory: Option<PathBuf>) -> Self {
        #[cfg(feature = "demo")]
        {
            let _ = persistence_directory;
            Self {}
        }
        #[cfg(not(feature = "demo"))]
        {
            Self {
                persistence_directory: persistence_directory.unwrap_or_else(|| {
                    panic!("Production bootstrap must provide persistence paths")
                }),
            }
        }
    }
}

impl Plugin for SelectedEditionPlugin {
    fn build(&self, app: &mut App) {
        #[cfg(feature = "demo")]
        app.add_plugins(demo::DemoEditionPlugin);
        #[cfg(not(feature = "demo"))]
        app.add_plugins(production::ProductionEditionPlugin::new(
            self.persistence_directory.clone(),
        ));
    }
}

pub struct SharedGamePlugin;

impl Plugin for SharedGamePlugin {
    fn build(&self, app: &mut App) {
        use crate::plugins::{
            BalloonCursorPlugin, GameAudioPlugin, LessonMascotPlugin, SettingsPlugin,
            SkyBackgroundPlugin,
        };
        use crate::questions::{
            FractionComparisonPlugin, FractionIdentificationPlugin, FractionVisualizationPlugin,
            McqPlugin, NumericInputPlugin,
        };
        use crate::screens::{
            ActivitySelectionScreenPlugin, HomeScreenPlugin, LessonPlayScreenPlugin,
            SettingsScreenPlugin, ThemeExplorationScreenPlugin,
        };
        use crate::states::{ActiveLesson, AppState, ExplorationView, InLessonFlow, LessonPhase};
        use crate::ui::{
            ButtonActivationPlugin, FocusNavigationPlugin, NavigationPlugin,
            PointerInteractionPlugin, ThemePlugin, UiAnimationPlugin,
        };

        app.init_state::<AppState>()
            .add_computed_state::<InLessonFlow>()
            .add_computed_state::<ActiveLesson>()
            .add_sub_state::<LessonPhase>()
            .add_sub_state::<ExplorationView>()
            .add_plugins((
                BalloonCursorPlugin,
                ButtonActivationPlugin,
                FocusNavigationPlugin,
                GameAudioPlugin,
                LessonMascotPlugin,
                NavigationPlugin,
                PointerInteractionPlugin,
                SettingsPlugin,
                SkyBackgroundPlugin,
                ThemePlugin,
                UiAnimationPlugin,
            ))
            .add_plugins((
                McqPlugin,
                FractionVisualizationPlugin,
                FractionComparisonPlugin,
                FractionIdentificationPlugin,
                NumericInputPlugin,
            ))
            .add_plugins((
                HomeScreenPlugin,
                ThemeExplorationScreenPlugin,
                ActivitySelectionScreenPlugin,
                LessonPlayScreenPlugin,
                SettingsScreenPlugin,
            ));
    }
}
