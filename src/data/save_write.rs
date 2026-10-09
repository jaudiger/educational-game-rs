use bevy::prelude::*;
use bevy_persistent::{PersistenceError, prelude::Persistent};

use super::save::SaveData;

#[derive(Clone, Copy, Debug, Eq, PartialEq, Reflect)]
pub enum SaveWriteAction {
    ClassAnswer,
    IndividualLessonProgress,
    TeacherStatsReset,
}

#[derive(Resource, Clone, Copy, Debug, Default, Eq, PartialEq, Reflect)]
pub struct SaveWriteStatus(Option<SaveWriteAction>);

impl SaveWriteStatus {
    pub fn failed(self, action: SaveWriteAction) -> bool {
        self.0 == Some(action)
    }

    pub const fn clear(&mut self) {
        self.0 = None;
    }

    const fn fail(&mut self, action: SaveWriteAction) {
        self.0 = Some(action);
    }
}

/// Updates save data and exposes write failures through a runtime status resource.
pub fn update_save_data(
    save_data: &mut Persistent<SaveData>,
    status: &mut SaveWriteStatus,
    action: SaveWriteAction,
    update: impl Fn(&mut SaveData),
) {
    match save_data.update(update) {
        Ok(()) => status.clear(),
        Err(error) => {
            status.fail(action);
            log_write_failure(action, &error);
        }
    }
}

fn log_write_failure(action: SaveWriteAction, error: &PersistenceError) {
    bevy::log::error!(
        "Failed to persist save data for {action:?}; in-memory data may have changed: {error}"
    );
}
