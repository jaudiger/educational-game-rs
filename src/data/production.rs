use bevy::prelude::*;
use serde::{Deserialize, Serialize};

/// The Production-only play mode selected in settings.
#[derive(Clone, Copy, Debug, Default, Eq, PartialEq, Reflect, Deserialize, Serialize)]
pub enum GameMode {
    #[default]
    Individual,
    Group,
}

/// Production-owned settings that do not apply to Demo.
#[derive(Resource, Clone, Copy, Debug, Default, Reflect, Deserialize, Serialize)]
#[serde(deny_unknown_fields)]
pub struct ProductionSettings {
    pub mode: GameMode,
}
