use bevy::material::specialize::SpecializedMeshPipelineError;
use bevy::mesh::MeshVertexBufferLayoutRef;
use bevy::prelude::*;
use bevy::render::render_resource::{AsBindGroup, RenderPipelineDescriptor, ShaderType};
use bevy::shader::ShaderRef;
use bevy::sprite_render::{
    AlphaMode2d, Material2d, Material2dKey, Material2dPipeline, Material2dPlugin,
};
use bevy::ui_render::UiMaterialPlugin;
use bevy::ui_render::ui_material::{UiMaterial, UiMaterialKey};

const CLOUD_SHADER_PATH: &str = "shaders/cloud.wgsl";

pub struct CloudMaterialPlugin;

impl Plugin for CloudMaterialPlugin {
    fn build(&self, app: &mut App) {
        app.add_plugins((
            Material2dPlugin::<CloudMaterial>::default(),
            UiMaterialPlugin::<CloudUiMaterial>::default(),
        ));
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq, Reflect)]
#[repr(u32)]
pub enum CloudShapeId {
    Puffy = 0,
    Elongated = 1,
}

impl CloudShapeId {
    pub const ALL: [Self; 2] = [Self::Puffy, Self::Elongated];

    pub const fn id(self) -> u32 {
        self as u32
    }

    pub const fn canvas(self) -> Vec2 {
        match self {
            Self::Puffy => Vec2::new(150.0, 100.0),
            Self::Elongated => Vec2::new(220.0, 70.0),
        }
    }
}

#[derive(Clone, Copy, Debug, Reflect, ShaderType)]
struct CloudUniform {
    color: LinearRgba,
    canvas: Vec2,
    shape_id: u32,
    padding: u32,
}

#[derive(Asset, AsBindGroup, Clone, Reflect)]
pub struct CloudMaterial {
    #[uniform(0)]
    uniform: CloudUniform,
}

#[derive(Asset, AsBindGroup, Clone, Reflect)]
pub struct CloudUiMaterial {
    #[uniform(0)]
    uniform: CloudUniform,
}

impl CloudMaterial {
    pub fn new(shape_id: CloudShapeId, canvas: Vec2, color: Color) -> Self {
        Self {
            uniform: CloudUniform {
                color: color.to_linear(),
                canvas,
                shape_id: shape_id.id(),
                padding: 0,
            },
        }
    }

    pub const fn set_opacity(&mut self, opacity: f32) {
        self.uniform.color.alpha = opacity;
    }
}

impl Material2d for CloudMaterial {
    fn fragment_shader() -> ShaderRef {
        CLOUD_SHADER_PATH.into()
    }

    fn alpha_mode(&self) -> AlphaMode2d {
        AlphaMode2d::Blend
    }

    fn specialize(
        _pipeline: &Material2dPipeline,
        descriptor: &mut RenderPipelineDescriptor,
        _layout: &MeshVertexBufferLayoutRef,
        _key: Material2dKey<Self>,
    ) -> Result<(), SpecializedMeshPipelineError> {
        descriptor.fragment.as_mut().unwrap().entry_point = Some("cloud_mesh_fragment".into());
        Ok(())
    }
}

impl CloudUiMaterial {
    pub fn new(shape_id: CloudShapeId, color: Color) -> Self {
        Self {
            uniform: CloudUniform {
                color: color.to_linear(),
                canvas: Vec2::ZERO,
                shape_id: shape_id.id(),
                padding: 0,
            },
        }
    }
}

impl UiMaterial for CloudUiMaterial {
    fn fragment_shader() -> ShaderRef {
        CLOUD_SHADER_PATH.into()
    }

    fn specialize(descriptor: &mut RenderPipelineDescriptor, _key: UiMaterialKey<Self>) {
        descriptor.fragment.as_mut().unwrap().entry_point = Some("cloud_ui_fragment".into());
    }
}
