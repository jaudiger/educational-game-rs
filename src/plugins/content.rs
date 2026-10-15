#[cfg(not(target_arch = "wasm32"))]
use std::path::PathBuf;

use bevy::prelude::*;

#[cfg(not(target_arch = "wasm32"))]
use crate::data::content::load_content_library;
#[cfg(target_arch = "wasm32")]
use crate::data::content::loading::load_embedded_content_library;

enum ContentSource {
    #[cfg(not(target_arch = "wasm32"))]
    Directory(PathBuf),
    #[cfg(target_arch = "wasm32")]
    Embedded,
}

pub struct ContentPlugin {
    source: ContentSource,
}

impl ContentPlugin {
    #[cfg(not(target_arch = "wasm32"))]
    pub fn new(content_directory: impl Into<PathBuf>) -> Self {
        Self {
            source: ContentSource::Directory(content_directory.into()),
        }
    }

    #[cfg(target_arch = "wasm32")]
    pub fn embedded() -> Self {
        Self {
            source: ContentSource::Embedded,
        }
    }
}

impl Plugin for ContentPlugin {
    fn build(&self, app: &mut App) {
        let library = match &self.source {
            #[cfg(not(target_arch = "wasm32"))]
            ContentSource::Directory(directory) => load_content_library(directory),
            #[cfg(target_arch = "wasm32")]
            ContentSource::Embedded => load_embedded_content_library(),
        }
        .unwrap_or_else(|error| {
            bevy::log::error!("Failed to load content catalog: {error}");
            panic!("Content catalog loading failed");
        });
        app.insert_resource(library);
    }
}
