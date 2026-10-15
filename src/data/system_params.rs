use bevy::ecs::system::SystemParam;
use bevy::prelude::*;

use super::{GameSettings, SettingsChanged};

#[derive(SystemParam)]
pub struct GameSettingsMut<'w> {
    pub settings: ResMut<'w, GameSettings>,
    changed: MessageWriter<'w, SettingsChanged>,
}

impl GameSettingsMut<'_> {
    pub fn update(&mut self, update: impl FnOnce(&mut GameSettings)) {
        update(&mut self.settings);
        self.changed.write(SettingsChanged);
    }

    pub fn persist(&mut self) {
        self.changed.write(SettingsChanged);
    }
}
