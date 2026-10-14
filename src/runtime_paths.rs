use std::env;
use std::fs;
use std::io;
use std::path::{Path, PathBuf};

use directories::ProjectDirs;

pub struct RuntimePaths {
    assets: PathBuf,
    content: PathBuf,
    persistence: PathBuf,
}

impl RuntimePaths {
    pub fn resolve() -> Result<Self, String> {
        let executable = env::current_exe()
            .map_err(|error| format!("Cannot locate application executable: {error}"))?;
        let asset_directory =
            resolve_asset_directory(&executable, Path::new(env!("CARGO_MANIFEST_DIR")))
                .ok_or_else(|| {
                    format!(
                        "Cannot find the application assets beside {} or in the source tree",
                        executable.display()
                    )
                })?;
        let persistence_directory =
            ProjectDirs::from("org", "Educational Game", "educational-game-rs")
                .map(|directories| directories.data_local_dir().to_path_buf())
                .ok_or_else(|| "Cannot locate the application data directory".to_string())?;
        let legacy_directory = env::current_dir()
            .map_err(|error| format!("Cannot locate the current directory: {error}"))?
            .join("local");
        migrate_legacy_persistence(&legacy_directory, &persistence_directory)
            .map_err(|error| format!("Cannot migrate local save data: {error}"))?;

        Ok(Self {
            content: asset_directory.join("content/themes"),
            assets: asset_directory,
            persistence: persistence_directory,
        })
    }

    pub fn asset_directory(&self) -> &Path {
        &self.assets
    }

    pub fn content_directory(&self) -> &Path {
        &self.content
    }

    pub fn persistence_directory(&self) -> &Path {
        &self.persistence
    }
}

fn resolve_asset_directory(executable: &Path, source_root: &Path) -> Option<PathBuf> {
    let executable_directory = executable.parent()?;
    let bundle_assets = executable_directory
        .parent()
        .filter(|contents| contents.file_name().is_some_and(|name| name == "Contents"))
        .map(|contents| contents.join("Resources/assets"));

    if let Some(asset_directory) = bundle_assets.filter(|path| path.is_dir()) {
        return Some(asset_directory);
    }

    let adjacent_assets = executable_directory.join("assets");
    if adjacent_assets.is_dir() {
        return Some(adjacent_assets);
    }

    let source_assets = source_root.join("assets");
    source_assets.is_dir().then_some(source_assets)
}

fn migrate_legacy_persistence(source: &Path, destination: &Path) -> io::Result<()> {
    for file_name in ["settings.json", "save_data.json"] {
        let source_file = source.join(file_name);
        let destination_file = destination.join(file_name);
        if !destination_file.exists() && source_file.is_file() {
            fs::create_dir_all(destination)?;
            fs::copy(source_file, destination_file)?;
        }
    }

    Ok(())
}
