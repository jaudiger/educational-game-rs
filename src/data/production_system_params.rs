use bevy::ecs::system::SystemParam;
use bevy::prelude::*;

use super::{
    ActiveStudent, GameSettings, PersistenceAction, PersistencePaths, PersistenceStatus,
    PlayerSession, ProductionSettings, SaveData, persist_game_settings, update_save_data,
};

#[derive(SystemParam)]
pub struct ProductionPlayerContext<'w> {
    pub production: Res<'w, ProductionSettings>,
    pub save_data: Res<'w, SaveData>,
    pub session: Option<Res<'w, PlayerSession>>,
    pub active_student: Option<Res<'w, ActiveStudent>>,
}

#[derive(SystemParam)]
pub struct PersistenceMut<'w> {
    pub save_data: ResMut<'w, SaveData>,
    paths: Res<'w, PersistencePaths>,
    pub status: ResMut<'w, PersistenceStatus>,
}

impl PersistenceMut<'_> {
    pub fn update_save_data<R>(
        &mut self,
        action: PersistenceAction,
        update: impl FnOnce(&mut SaveData) -> R,
    ) -> R {
        update_save_data(
            &mut self.save_data,
            self.paths.save_data_file(),
            &mut self.status,
            action,
            update,
        )
    }
}

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

#[derive(SystemParam)]
pub struct ProductionSettingsMut<'w> {
    pub settings: ResMut<'w, ProductionSettings>,
    common: Res<'w, GameSettings>,
    paths: Res<'w, PersistencePaths>,
    pub status: ResMut<'w, PersistenceStatus>,
}

impl ProductionSettingsMut<'_> {
    pub fn update(&mut self, update: impl FnOnce(&mut ProductionSettings)) {
        update(&mut self.settings);
        persist_game_settings(
            &self.common,
            *self.settings,
            self.paths.settings_file(),
            &mut self.status,
        );
    }
}
