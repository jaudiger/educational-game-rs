use bevy::ecs::system::SystemParam;
use bevy::prelude::*;
use bevy_persistent::prelude::Persistent;

use super::{ActiveSlot, ActiveStudent, GameSettings, SaveData};

/// Bundles the four most common read-only player-state resources.
/// Avoids repeating these across every screen system that reads current
/// slot, student, and game settings together.
#[derive(SystemParam)]
pub struct PlayerContext<'w> {
    pub settings: Res<'w, Persistent<GameSettings>>,
    pub save_data: Res<'w, Persistent<SaveData>>,
    pub active_slot: Option<Res<'w, ActiveSlot>>,
    pub active_student: Option<Res<'w, ActiveStudent>>,
}

/// Bundles game settings with mutable save data for systems that write to
/// the save file, without exposing the read-only slot/student context.
#[derive(SystemParam)]
pub struct PersistenceMut<'w> {
    pub settings: Res<'w, Persistent<GameSettings>>,
    pub save_data: ResMut<'w, Persistent<SaveData>>,
}
