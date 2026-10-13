use bevy::prelude::*;

use crate::data::content::load_content_library;

/// Loads and inserts the content library during plugin construction.
pub struct ContentPlugin;

impl Plugin for ContentPlugin {
    fn build(&self, app: &mut App) {
        let library = load_content_library("assets/content/themes").unwrap_or_else(|error| {
            bevy::log::error!("Failed to load content catalog: {error}");
            panic!("Content catalog loading failed");
        });
        app.insert_resource(library);
    }
}
