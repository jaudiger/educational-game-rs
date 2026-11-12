use bevy::prelude::*;
use bevy::render::render_resource::{AsBindGroup, ShaderType};
use bevy::shader::ShaderRef;
use bevy::ui::RelativeCursorPosition;
use bevy::ui_render::UiMaterialPlugin;
use bevy::ui_render::ui_material::{MaterialNode, UiMaterial};

const UI_SHAPE_SHADER_PATH: &str = "shaders/ui_shape.wgsl";
const UI_SHAPE_HOVER_SPEED: f32 = 3.5;
const UI_SHAPE_HOVER_SCALE: f32 = 0.06;

pub struct UiShapePlugin;

impl Plugin for UiShapePlugin {
    fn build(&self, app: &mut App) {
        app.add_plugins(UiMaterialPlugin::<UiShapeMaterial>::default())
            .add_systems(
                Update,
                (initialize_ui_shape_materials, update_ui_shape_hover),
            );
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq, Reflect)]
#[repr(u32)]
pub enum UiShapeKind {
    Ellipse = 0,
    RoundedRectangle = 1,
}

impl UiShapeKind {
    const fn id(self) -> u32 {
        self as u32
    }
}

#[derive(Component, Reflect)]
#[require(RelativeCursorPosition)]
pub struct UiShapeVisual {
    shape: UiShapeKind,
    color: Color,
    opacity: f32,
}

impl UiShapeVisual {
    pub const fn new(shape: UiShapeKind, color: Color, opacity: f32) -> Self {
        Self {
            shape,
            color,
            opacity,
        }
    }
}

#[derive(Component, Reflect)]
struct UiShapeHover {
    strength: f32,
    cursor_position: Vec2,
}

impl Default for UiShapeHover {
    fn default() -> Self {
        Self {
            strength: 0.0,
            cursor_position: Vec2::splat(0.5),
        }
    }
}

#[derive(Clone, Copy, Debug, Reflect, ShaderType)]
struct UiShapeUniform {
    color: LinearRgba,
    cursor_position: Vec2,
    shape_id: u32,
    hover: f32,
}

#[derive(Asset, AsBindGroup, Clone, Reflect)]
pub struct UiShapeMaterial {
    #[uniform(0)]
    uniform: UiShapeUniform,
}

impl UiShapeMaterial {
    fn new(visual: &UiShapeVisual) -> Self {
        let mut color = visual.color.to_linear();
        color.alpha *= visual.opacity;
        Self {
            uniform: UiShapeUniform {
                color,
                cursor_position: Vec2::splat(0.5),
                shape_id: visual.shape.id(),
                hover: 0.0,
            },
        }
    }

    const fn set_hover(&mut self, strength: f32, cursor_position: Vec2) {
        self.uniform.hover = strength;
        self.uniform.cursor_position = cursor_position;
    }
}

impl UiMaterial for UiShapeMaterial {
    fn fragment_shader() -> ShaderRef {
        UI_SHAPE_SHADER_PATH.into()
    }
}

fn initialize_ui_shape_materials(
    mut commands: Commands,
    mut materials: ResMut<Assets<UiShapeMaterial>>,
    decorations: Query<(Entity, &UiShapeVisual), Added<UiShapeVisual>>,
) {
    for (entity, visual) in &decorations {
        commands.entity(entity).insert((
            MaterialNode(materials.add(UiShapeMaterial::new(visual))),
            UiShapeHover::default(),
        ));
    }
}

fn update_ui_shape_hover(
    time: Res<Time>,
    mut materials: ResMut<Assets<UiShapeMaterial>>,
    mut decorations: Query<(
        &RelativeCursorPosition,
        &mut UiShapeHover,
        &MaterialNode<UiShapeMaterial>,
        &mut UiTransform,
    )>,
) {
    let step = time.delta_secs() * UI_SHAPE_HOVER_SPEED;
    for (relative_cursor, mut state, material_node, mut transform) in &mut decorations {
        let target = f32::from(relative_cursor.cursor_over());
        let strength = if state.strength < target {
            (state.strength + step).min(target)
        } else {
            (state.strength - step).max(target)
        };
        let cursor_position = if relative_cursor.cursor_over() {
            relative_cursor
                .normalized
                .map_or(state.cursor_position, |position| {
                    (position + Vec2::splat(0.5)).clamp(Vec2::ZERO, Vec2::ONE)
                })
        } else {
            state.cursor_position
        };
        let strength_changed = (state.strength - strength).abs() > f32::EPSILON;
        let cursor_changed = state.cursor_position.distance_squared(cursor_position) > 0.000_001;
        if !strength_changed && !cursor_changed {
            continue;
        }

        state.strength = strength;
        state.cursor_position = cursor_position;
        transform.scale = Vec2::splat(strength.mul_add(UI_SHAPE_HOVER_SCALE, 1.0));
        if let Some(mut material) = materials.get_mut(&material_node.0) {
            material.set_hover(strength, cursor_position);
        }
    }
}
