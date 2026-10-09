use std::path::Path;

use bevy::prelude::*;

use super::{GameSettings, SaveData, write_json_atomic};

#[derive(Clone, Copy, Debug, Eq, PartialEq, Reflect)]
pub enum PersistenceAction {
    ClassAnswer,
    IndividualLessonProgress,
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

    fn fail(&mut self, action: PersistenceAction) {
        if action == PersistenceAction::GameSettings {
            self.settings_failure = Some(action);
        } else {
            self.save_data_failure = Some(action);
        }
    }
}

pub fn update_save_data(
    save_data: &mut SaveData,
    path: &Path,
    status: &mut PersistenceStatus,
    action: PersistenceAction,
    update: impl FnOnce(&mut SaveData),
) {
    update(save_data);
    match write_json_atomic(path, save_data) {
        Ok(()) => status.clear_save_data(),
        Err(error) => {
            status.fail(action);
            log_write_failure(action, path, &error);
        }
    }
}

pub fn update_game_settings(
    settings: &mut GameSettings,
    path: &Path,
    status: &mut PersistenceStatus,
    update: impl FnOnce(&mut GameSettings),
) {
    update(settings);
    persist_game_settings(settings, path, status);
}

pub fn persist_game_settings(settings: &GameSettings, path: &Path, status: &mut PersistenceStatus) {
    match write_json_atomic(path, settings) {
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
