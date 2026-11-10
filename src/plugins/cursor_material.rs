use bevy::material::specialize::SpecializedMeshPipelineError;
use bevy::mesh::MeshVertexBufferLayoutRef;
use bevy::prelude::*;
use bevy::render::render_resource::{AsBindGroup, RenderPipelineDescriptor, ShaderType};
use bevy::shader::ShaderRef;
use bevy::sprite_render::{
    AlphaMode2d, Material2d, Material2dKey, Material2dPipeline, Material2dPlugin,
};

const CURSOR_SHADER_PATH: &str = "shaders/procedural_cursor.wgsl";

pub struct CursorMaterialPlugin;

impl Plugin for CursorMaterialPlugin {
    fn build(&self, app: &mut App) {
        app.add_plugins(Material2dPlugin::<CursorMaterial>::default());
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq, Reflect)]
#[repr(u32)]
pub enum CursorEyeFrame {
    Open = 0,
    Half = 1,
    Closed = 2,
}

impl CursorEyeFrame {
    const fn id(self) -> u32 {
        self as u32
    }
}

#[derive(Clone, Copy, Debug, Default, Reflect, ShaderType)]
struct CursorUniform {
    eye_frame: u32,
}

#[derive(Asset, AsBindGroup, Clone, Default, Reflect)]
pub struct CursorMaterial {
    #[uniform(0)]
    uniform: CursorUniform,
}

impl CursorMaterial {
    pub const fn set_eye_frame(&mut self, frame: CursorEyeFrame) {
        self.uniform.eye_frame = frame.id();
    }
}

impl Material2d for CursorMaterial {
    fn fragment_shader() -> ShaderRef {
        CURSOR_SHADER_PATH.into()
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
        descriptor.fragment.as_mut().unwrap().entry_point = Some("cursor_mesh_fragment".into());
        Ok(())
    }
}
