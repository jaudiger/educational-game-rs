use std::path::PathBuf;

use bevy::prelude::*;

use crate::data::{
    ContentLibrary, GameSettings, PersistedSettings, PersistencePaths, PersistenceStatus,
    ProductionSettings, SaveData, SettingsChanged, load_or_default,
};
use crate::i18n::I18n;
use crate::ui::components::sync_persistence_notices;

/// Loads and synchronously persists save data and game settings.
pub struct PersistencePlugin {
    paths: PersistencePaths,
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
        let save_data = {
            let content = app.world().resource::<ContentLibrary>();
            load_or_default(self.paths.save_data_file(), SaveData::default(), |data| {
                data.validate_persisted(content)
            })
            .expect("failed to initialize save data")
        };
        let persisted_settings = load_or_default(
            self.paths.settings_file(),
            PersistedSettings::default(),
            PersistedSettings::validate,
        )
        .expect("failed to initialize game settings");
        let (settings, production_settings) = persisted_settings.into_resources();
        let language = settings.language;

        app.insert_resource(self.paths.clone())
            .insert_resource(save_data)
            .insert_resource(settings)
            .insert_resource(production_settings)
            .init_resource::<PersistenceStatus>()
            .insert_resource(I18n::new(language))
            .add_systems(
                Update,
                sync_persistence_notices.run_if(
                    resource_changed::<PersistenceStatus>.or_else(resource_changed::<I18n>),
                ),
            )
            .add_systems(Update, persist_changed_settings);
    }
}

fn persist_changed_settings(
    mut changed: MessageReader<SettingsChanged>,
    settings: Res<GameSettings>,
    production: Res<ProductionSettings>,
    paths: Res<PersistencePaths>,
    mut status: ResMut<PersistenceStatus>,
) {
    let has_changes = changed.read().count() > 0;
    if has_changes {
        crate::data::persist_game_settings(
            &settings,
            *production,
            paths.settings_file(),
            &mut status,
        );
    }
}
