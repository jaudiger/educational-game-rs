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

const BALLOON_SHADER_PATH: &str = "shaders/balloon.wgsl";

pub struct BalloonMaterialPlugin;

impl Plugin for BalloonMaterialPlugin {
    fn build(&self, app: &mut App) {
        app.add_plugins((
            Material2dPlugin::<BalloonMaterial>::default(),
            UiMaterialPlugin::<BalloonUiMaterial>::default(),
        ));
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq, Reflect)]
#[repr(u32)]
pub enum BalloonPart {
    Envelope = 0,
    Basket = 1,
    Flame = 2,
}

pub const BALLOON_BASKET_CORD_ATTACHMENTS: [Vec2; 4] = [
    Vec2::new(28.0, 10.0),
    Vec2::new(48.0, 10.0),
    Vec2::new(80.0, 10.0),
    Vec2::new(100.0, 10.0),
];

impl BalloonPart {
    const fn id(self) -> u32 {
        self as u32
    }

    pub const fn canvas(self) -> Vec2 {
        match self {
            Self::Envelope => Vec2::new(256.0, 320.0),
            Self::Basket => Vec2::new(128.0, 80.0),
            Self::Flame => Vec2::new(64.0, 80.0),
        }
    }

    pub const fn height_for_width(self, width: f32) -> f32 {
        let canvas = self.canvas();
        width * canvas.y / canvas.x
    }
}

#[derive(Clone, Copy, Debug, Default, Reflect, ShaderType)]
struct BalloonUniform {
    part_id: u32,
    flame_intensity: f32,
    cord_endpoints: [Vec4; 2],
}

#[derive(Asset, AsBindGroup, Clone, Reflect)]
pub struct BalloonMaterial {
    #[uniform(0)]
    uniform: BalloonUniform,
}

#[derive(Component, Clone, Default, Reflect)]
pub struct BalloonFlame;

impl BalloonFlame {
    pub const BASE_INTENSITY: f32 = 0.4;
}

#[derive(Asset, AsBindGroup, Clone, Reflect)]
pub struct BalloonUiMaterial {
    #[uniform(0)]
    uniform: BalloonUniform,
}

impl BalloonMaterial {
    pub const fn new(part: BalloonPart) -> Self {
        Self {
            uniform: BalloonUniform::new(part),
        }
    }

    pub const fn set_flame_intensity(&mut self, intensity: f32) {
        self.uniform.flame_intensity = intensity.clamp(0.0, 1.0);
    }

    pub const fn set_cord_endpoints(&mut self, endpoints: [Vec4; 2]) {
        self.uniform.cord_endpoints = endpoints;
    }
}

impl BalloonUiMaterial {
    pub const fn new(part: BalloonPart) -> Self {
        Self {
            uniform: BalloonUniform::new(part),
        }
    }

    pub const fn set_flame_intensity(&mut self, intensity: f32) {
        self.uniform.flame_intensity = intensity.clamp(0.0, 1.0);
    }

    pub const fn set_cord_endpoints(&mut self, endpoints: [Vec4; 2]) {
        self.uniform.cord_endpoints = endpoints;
    }
}

impl BalloonUniform {
    const fn new(part: BalloonPart) -> Self {
        Self {
            part_id: part.id(),
            flame_intensity: match part {
                BalloonPart::Flame => BalloonFlame::BASE_INTENSITY,
                BalloonPart::Envelope | BalloonPart::Basket => 0.0,
            },
            cord_endpoints: [
                Vec4::new(80.0, 318.0, 107.0, 318.0),
                Vec4::new(149.0, 318.0, 176.0, 318.0),
            ],
        }
    }
}

impl Material2d for BalloonMaterial {
    fn fragment_shader() -> ShaderRef {
        BALLOON_SHADER_PATH.into()
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
        descriptor.fragment.as_mut().unwrap().entry_point = Some("balloon_mesh_fragment".into());
        Ok(())
    }
}

impl UiMaterial for BalloonUiMaterial {
    fn fragment_shader() -> ShaderRef {
        BALLOON_SHADER_PATH.into()
    }

    fn specialize(descriptor: &mut RenderPipelineDescriptor, _key: UiMaterialKey<Self>) {
        descriptor.fragment.as_mut().unwrap().entry_point = Some("balloon_ui_fragment".into());
    }
}
