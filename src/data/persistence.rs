use std::ffi::OsString;
use std::fs;
use std::io::{self, Write};
use std::path::{Path, PathBuf};

use bevy::prelude::Resource;
use serde::{Serialize, de::DeserializeOwned};
use tempfile::NamedTempFile;

#[derive(Resource, Clone, Debug)]
pub struct PersistencePaths {
    settings_file: PathBuf,
    save_data_file: PathBuf,
}

impl PersistencePaths {
    pub(crate) fn in_directory(directory: impl Into<PathBuf>) -> Self {
        let directory = directory.into();
        Self {
            settings_file: directory.join("settings.json"),
            save_data_file: directory.join("save_data.json"),
        }
    }

    pub(crate) fn settings_file(&self) -> &Path {
        &self.settings_file
    }

    pub(crate) fn save_data_file(&self) -> &Path {
        &self.save_data_file
    }
}

pub fn load_or_default<T>(path: &Path, default: T) -> io::Result<T>
where
    T: DeserializeOwned + Serialize,
{
    let bytes = match fs::read(path) {
        Ok(bytes) => bytes,
        Err(error) if error.kind() == io::ErrorKind::NotFound => {
            write_json_atomic(path, &default)?;
            return Ok(default);
        }
        Err(error) => return Err(error),
    };

    match serde_json::from_slice(&bytes) {
        Ok(value) => Ok(value),
        Err(error) => {
            bevy::log::error!(
                "Failed to deserialize persisted data from {}: {error}",
                path.display()
            );
            let backup_path = preserve_corrupt_file(path)?;
            bevy::log::warn!(
                "Preserved malformed persisted data at {}",
                backup_path.display()
            );
            write_json_atomic(path, &default)?;
            Ok(default)
        }
    }
}

pub fn write_json_atomic<T: Serialize>(path: &Path, value: &T) -> io::Result<()> {
    let bytes = serde_json::to_vec(value).map_err(io::Error::other)?;
    let parent = parent_directory(path);
    fs::create_dir_all(parent)?;

    let mut temporary_file = NamedTempFile::new_in(parent)?;
    temporary_file.write_all(&bytes)?;
    temporary_file.as_file().sync_all()?;
    temporary_file.persist(path).map_err(|error| error.error)?;

    Ok(())
}

fn parent_directory(path: &Path) -> &Path {
    path.parent()
        .filter(|parent| !parent.as_os_str().is_empty())
        .unwrap_or_else(|| Path::new("."))
}

fn preserve_corrupt_file(path: &Path) -> io::Result<PathBuf> {
    let file_name = path
        .file_name()
        .map_or_else(|| OsString::from("data"), OsString::from);
    let mut suffix = 0_usize;

    loop {
        let mut backup_name = file_name.clone();
        backup_name.push(".corrupt");
        if suffix > 0 {
            backup_name.push(format!(".{suffix}"));
        }
        let backup_path = path.with_file_name(backup_name);
        if !backup_path.exists() {
            fs::copy(path, &backup_path)?;
            return Ok(backup_path);
        }
        suffix = suffix.saturating_add(1);
    }
}
