use bevy::ecs::system::SystemParam;
use bevy::prelude::*;

use super::{
    ActiveStudent, GameSettings, PersistenceAction, PersistencePaths, PersistenceStatus,
    PlayerSession, SaveData, persist_game_settings, update_game_settings, update_save_data,
};

/// Bundles the four most common read-only player-state resources.
/// Avoids repeating these across every screen system that reads current
/// slot, student, and game settings together.
#[derive(SystemParam)]
pub struct PlayerContext<'w> {
    pub settings: Res<'w, GameSettings>,
    pub save_data: Res<'w, SaveData>,
    pub session: Option<Res<'w, PlayerSession>>,
    pub active_student: Option<Res<'w, ActiveStudent>>,
}

/// Bundles settings and save data access for systems that need both.
#[derive(SystemParam)]
pub struct PersistenceMut<'w> {
    pub settings: Res<'w, GameSettings>,
    pub save_data: ResMut<'w, SaveData>,
    paths: Res<'w, PersistencePaths>,
    pub status: ResMut<'w, PersistenceStatus>,
}

impl PersistenceMut<'_> {
    pub fn update_save_data(
        &mut self,
        action: PersistenceAction,
        update: impl FnOnce(&mut SaveData),
    ) {
        update_save_data(
            &mut self.save_data,
            self.paths.save_data_file(),
            &mut self.status,
            action,
            update,
        );
    }
}

/// Bundles save data with its storage path and write status.
#[derive(SystemParam)]
pub struct SaveDataMut<'w> {
    pub save_data: ResMut<'w, SaveData>,
    paths: Res<'w, PersistencePaths>,
    pub status: ResMut<'w, PersistenceStatus>,
}

impl SaveDataMut<'_> {
    pub fn update(&mut self, action: PersistenceAction, update: impl FnOnce(&mut SaveData)) {
        update_save_data(
            &mut self.save_data,
            self.paths.save_data_file(),
            &mut self.status,
            action,
            update,
        );
    }
}

/// Bundles game settings with its storage path and write status.
#[derive(SystemParam)]
pub struct GameSettingsMut<'w> {
    pub settings: ResMut<'w, GameSettings>,
    paths: Res<'w, PersistencePaths>,
    pub status: ResMut<'w, PersistenceStatus>,
}

impl GameSettingsMut<'_> {
    pub fn update(&mut self, update: impl FnOnce(&mut GameSettings)) {
        update_game_settings(
            &mut self.settings,
            self.paths.settings_file(),
            &mut self.status,
            update,
        );
    }

    pub fn persist(&mut self) {
        persist_game_settings(&self.settings, self.paths.settings_file(), &mut self.status);
    }
}
