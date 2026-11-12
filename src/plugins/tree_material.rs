use bevy::prelude::*;
use bevy::render::render_resource::{AsBindGroup, ShaderType};
use bevy::shader::ShaderRef;
use bevy::ui_render::ui_material::UiMaterial;

const TREE_SHADER_PATH: &str = "shaders/tree.wgsl";

#[derive(Clone, Copy, Debug, Reflect, ShaderType)]
struct TreeUniform {
    color: LinearRgba,
}

#[derive(Asset, AsBindGroup, Clone, Reflect)]
pub struct TreeMaterial {
    #[uniform(0)]
    uniform: TreeUniform,
}

impl TreeMaterial {
    pub fn new(color: Color) -> Self {
        Self {
            uniform: TreeUniform {
                color: color.to_linear(),
            },
        }
    }
}

impl UiMaterial for TreeMaterial {
    fn fragment_shader() -> ShaderRef {
        TREE_SHADER_PATH.into()
    }
}
