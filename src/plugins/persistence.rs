use bevy::prelude::*;
use bevy_persistent::prelude::*;

use crate::data::{GameSettings, SaveData};
use crate::i18n::I18n;

/// Initializes persistent storage for save data and game settings.
pub struct PersistencePlugin;

impl Plugin for PersistencePlugin {
    fn build(&self, app: &mut App) {
        let save_data = Persistent::<SaveData>::builder()
            .name("save data")
            .format(StorageFormat::Json)
            .path("local/save_data.json")
            .default(SaveData::default())
            .revertible(true)
            .revert_to_default_on_deserialization_errors(true)
            .build()
            .expect("failed to initialize save data");
        app.insert_resource(save_data);

        let settings = Persistent::<GameSettings>::builder()
            .name("game settings")
            .format(StorageFormat::Json)
            .path("local/settings.json")
            .default(GameSettings::default())
            .revertible(true)
            .revert_to_default_on_deserialization_errors(true)
            .build()
            .expect("failed to initialize game settings");
        let language = settings.language;
        app.insert_resource(settings);
        app.insert_resource(I18n::new(language));
    }
}
