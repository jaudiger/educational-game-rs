use std::path::PathBuf;

use bevy::app::PluginGroupBuilder;
use bevy::prelude::*;
use bevy::ui_widgets::MenuPlugin;

use crate::plugins::ContentPlugin;

#[cfg(all(target_arch = "wasm32", not(feature = "demo")))]
compile_error!("The Production edition is not supported on WASM");

pub struct PlatformBootstrap {
    pub plugins: PluginGroupBuilder,
    pub content: ContentPlugin,
    pub persistence_directory: Option<PathBuf>,
}

pub fn bootstrap() -> PlatformBootstrap {
    #[cfg(not(feature = "demo"))]
    let (asset_directory, content_directory, persistence_directory) = {
        let paths = crate::runtime_paths::RuntimePaths::resolve()
            .unwrap_or_else(|error| panic!("Application path setup failed: {error}"));
        (
            paths.asset_directory().to_string_lossy().into_owned(),
            paths.content_directory().to_path_buf(),
            Some(paths.persistence_directory().to_path_buf()),
        )
    };

    #[cfg(all(feature = "demo", not(target_arch = "wasm32")))]
    let (asset_directory, content_directory, persistence_directory) = (
        format!("{}/assets", env!("CARGO_MANIFEST_DIR")),
        PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("assets/content/themes"),
        None,
    );

    #[cfg(all(feature = "demo", target_arch = "wasm32"))]
    let (asset_directory, persistence_directory) = ("assets".to_owned(), None);

    let primary_window = Window {
        title: "Educational Game".into(),
        #[cfg(target_arch = "wasm32")]
        canvas: Some("#bevy-canvas".into()),
        #[cfg(target_arch = "wasm32")]
        fit_canvas_to_parent: true,
        ..default()
    };
    let plugins = DefaultPlugins
        .set(WindowPlugin {
            primary_window: Some(primary_window),
            ..default()
        })
        .set(AssetPlugin {
            file_path: asset_directory,
            #[cfg(target_arch = "wasm32")]
            meta_check: bevy::asset::AssetMetaCheck::Never,
            ..default()
        })
        .disable::<MenuPlugin>();

    #[cfg(not(target_arch = "wasm32"))]
    let content = ContentPlugin::new(content_directory);
    #[cfg(target_arch = "wasm32")]
    let content = ContentPlugin::embedded();

    PlatformBootstrap {
        plugins,
        content,
        persistence_directory,
    }
}
