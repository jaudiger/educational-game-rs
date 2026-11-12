use bevy::prelude::Plugin;
use bevy::ui_render::UiMaterialPlugin;

use crate::plugins::flower_material::FlowerMaterial;
use crate::plugins::grass_material::GrassMaterial;
use crate::plugins::landscape_material::LandscapeMaterial;
use crate::plugins::stone_material::StoneMaterial;
use crate::plugins::tree_material::TreeMaterial;

pub struct SceneryMaterialsPlugin;

impl Plugin for SceneryMaterialsPlugin {
    fn build(&self, app: &mut bevy::prelude::App) {
        app.add_plugins((
            UiMaterialPlugin::<LandscapeMaterial>::default(),
            UiMaterialPlugin::<TreeMaterial>::default(),
            UiMaterialPlugin::<FlowerMaterial>::default(),
            UiMaterialPlugin::<GrassMaterial>::default(),
            UiMaterialPlugin::<StoneMaterial>::default(),
        ));
    }
}
