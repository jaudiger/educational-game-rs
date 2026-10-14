use std::path::PathBuf;

use bevy::prelude::*;

use crate::data::content::load_content_library;

pub struct ContentPlugin {
    content_directory: PathBuf,
}

impl ContentPlugin {
    pub fn new(content_directory: impl Into<PathBuf>) -> Self {
        Self {
            content_directory: content_directory.into(),
        }
    }
}

impl Plugin for ContentPlugin {
    fn build(&self, app: &mut App) {
        let library = load_content_library(&self.content_directory).unwrap_or_else(|error| {
            bevy::log::error!("Failed to load content catalog: {error}");
            panic!("Content catalog loading failed");
        });
        app.insert_resource(library);
    }
}
