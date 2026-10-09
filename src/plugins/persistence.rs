use std::path::PathBuf;

use bevy::prelude::*;

use crate::data::{GameSettings, PersistencePaths, PersistenceStatus, SaveData, load_or_default};
use crate::i18n::I18n;
use crate::ui::components::sync_persistence_notices;

/// Loads and synchronously persists local save data and game settings.
pub struct PersistencePlugin {
    paths: PersistencePaths,
}

impl Default for PersistencePlugin {
    fn default() -> Self {
        Self::new("local")
    }
}

impl PersistencePlugin {
    pub fn new(directory: impl Into<PathBuf>) -> Self {
        Self {
            paths: PersistencePaths::in_directory(directory),
        }
    }
}

impl Plugin for PersistencePlugin {
    fn build(&self, app: &mut App) {
        let save_data = load_or_default(self.paths.save_data_file(), SaveData::default())
            .expect("failed to initialize save data");
        let settings = load_or_default(self.paths.settings_file(), GameSettings::default())
            .expect("failed to initialize game settings");
        let language = settings.language;

        app.insert_resource(self.paths.clone())
            .insert_resource(save_data)
            .insert_resource(settings)
            .init_resource::<PersistenceStatus>()
            .insert_resource(I18n::new(language))
            .add_systems(
                Update,
                sync_persistence_notices.run_if(
                    resource_changed::<PersistenceStatus>.or_else(resource_changed::<I18n>),
                ),
            );
    }
}
