use std::path::Path;

use bevy::prelude::*;

use serde::{Deserialize, Serialize};

use super::{
    ExplorationTheme, GameMode, GameSettings, Language, ProductionSettings, SaveData,
    write_json_atomic,
};

#[derive(Clone, Debug, Deserialize, Serialize)]
#[serde(deny_unknown_fields)]
pub struct PersistedSettings {
    pub music_volume: f32,
    pub sfx_volume: f32,
    pub show_explanations: bool,
    pub mode: GameMode,
    pub language: Language,
    pub exploration_theme: ExplorationTheme,
    pub gamepad_navigation: bool,
}

impl Default for PersistedSettings {
    fn default() -> Self {
        let common = GameSettings::default();
        Self {
            music_volume: common.music_volume,
            sfx_volume: common.sfx_volume,
            show_explanations: common.show_explanations,
            mode: GameMode::default(),
            language: common.language,
            exploration_theme: common.exploration_theme,
            gamepad_navigation: common.gamepad_navigation,
        }
    }
}

impl PersistedSettings {
    pub fn validate(&self) -> Result<(), String> {
        self.common_settings().validate_persisted()
    }

    pub const fn into_resources(self) -> (GameSettings, ProductionSettings) {
        (
            GameSettings {
                music_volume: self.music_volume,
                sfx_volume: self.sfx_volume,
                show_explanations: self.show_explanations,
                language: self.language,
                exploration_theme: self.exploration_theme,
                gamepad_navigation: self.gamepad_navigation,
            },
            ProductionSettings { mode: self.mode },
        )
    }

    const fn common_settings(&self) -> GameSettings {
        GameSettings {
            music_volume: self.music_volume,
            sfx_volume: self.sfx_volume,
            show_explanations: self.show_explanations,
            language: self.language,
            exploration_theme: self.exploration_theme,
            gamepad_navigation: self.gamepad_navigation,
        }
    }

    const fn from_resources(settings: &GameSettings, production: ProductionSettings) -> Self {
        Self {
            music_volume: settings.music_volume,
            sfx_volume: settings.sfx_volume,
            show_explanations: settings.show_explanations,
            mode: production.mode,
            language: settings.language,
            exploration_theme: settings.exploration_theme,
            gamepad_navigation: settings.gamepad_navigation,
        }
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq, Reflect)]
pub enum PersistenceAction {
    QuestionAnswer,
    TeacherStatsReset,
    SaveSlot,
    TeacherRoster,
    TeacherLessonConfig,
    GameSettings,
}

#[derive(Resource, Clone, Copy, Debug, Default, Eq, PartialEq, Reflect)]
pub struct PersistenceStatus {
    save_data_failure: Option<PersistenceAction>,
    settings_failure: Option<PersistenceAction>,
}

impl PersistenceStatus {
    pub fn failed(self, action: PersistenceAction) -> bool {
        if action == PersistenceAction::GameSettings {
            self.settings_failure == Some(action)
        } else {
            self.save_data_failure == Some(action)
        }
    }

    pub const fn clear_save_data(&mut self) {
        self.save_data_failure = None;
    }

    const fn clear_settings(&mut self) {
        self.settings_failure = None;
    }

    pub(crate) fn fail(&mut self, action: PersistenceAction) {
        if action == PersistenceAction::GameSettings {
            self.settings_failure = Some(action);
        } else {
            self.save_data_failure = Some(action);
        }
    }
}

pub fn update_save_data<R>(
    save_data: &mut SaveData,
    path: &Path,
    status: &mut PersistenceStatus,
    action: PersistenceAction,
    update: impl FnOnce(&mut SaveData) -> R,
) -> R {
    let result = update(save_data);
    match write_json_atomic(path, save_data) {
        Ok(()) => status.clear_save_data(),
        Err(error) => {
            status.fail(action);
            log_write_failure(action, path, &error);
        }
    }
    result
}

pub fn persist_game_settings(
    settings: &GameSettings,
    production: ProductionSettings,
    path: &Path,
    status: &mut PersistenceStatus,
) {
    let persisted = PersistedSettings::from_resources(settings, production);
    match write_json_atomic(path, &persisted) {
        Ok(()) => status.clear_settings(),
        Err(error) => {
            status.fail(PersistenceAction::GameSettings);
            log_write_failure(PersistenceAction::GameSettings, path, &error);
        }
    }
}

fn log_write_failure(action: PersistenceAction, path: &Path, error: &std::io::Error) {
    bevy::log::error!(
        "Failed to persist data for {action:?} at {}: {error}",
        path.display()
    );
}

#[cfg(test)]
mod tests {
    use super::PersistedSettings;
    use crate::data::{GameMode, GameSettings, ProductionSettings};

    #[test]
    fn settings_adapter_round_trips_common_preferences_and_mode() {
        let settings = GameSettings {
            music_volume: 0.25,
            ..GameSettings::default()
        };
        let production = ProductionSettings {
            mode: GameMode::Group,
        };
        let persisted = PersistedSettings::from_resources(&settings, production);
        let json = serde_json::to_string(&persisted).expect("settings serialize");
        let decoded: PersistedSettings = serde_json::from_str(&json).expect("settings deserialize");
        let (decoded_settings, decoded_production) = decoded.into_resources();

        assert_eq!(decoded_settings.music_volume, 0.25);
        assert_eq!(decoded_production.mode, GameMode::Group);
    }
}
