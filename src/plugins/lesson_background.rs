use bevy::ecs::system::SystemParam;
use bevy::prelude::*;
use bevy::ui::UiSystems;
use bevy::ui_render::ui_material::{MaterialNode, UiMaterial};
use bevy::window::WindowResized;

use crate::plugins::bush_material::BushMaterial;
use crate::plugins::cloud_material::{CloudShapeId, CloudUiMaterial};
use crate::ui::components::full_screen_overlay_node;
use crate::ui::{layers, theme};

use crate::plugins::lesson_motion::{LessonMotionSet, LessonWind, smooth_motion_fade};

use crate::plugins::flower_material::FlowerMaterial;
use crate::plugins::grass_material::GrassMaterial;
use crate::plugins::landscape_material::{
    LandscapeLayer, LandscapeMaterial, PathGeometry, RidgeGeometry,
};
use crate::plugins::stone_material::{STONE_HALF_WIDTH_RATIO, StoneMaterial};
use crate::plugins::sun_material::{SunMaterial, sky_sun, sky_sun_material};
use crate::plugins::tree_material::TreeMaterial;
use crate::states::ActiveLesson;

const PATH_GEOMETRY: PathGeometry = PathGeometry::DEFAULT;
const RIDGE_GEOMETRY: RidgeGeometry = RidgeGeometry::DEFAULT;
const CLOUD_DRIFT_SPEED_DESIGN_PX_PER_SECOND: f32 = 15.0;
const CLOUD_MOTION_FADE_SECONDS: f32 = 2.0;
const FLOWER_BEND_AMOUNT: f32 = 0.15;
const GRASS_BEND_AMOUNT: f32 = 0.075;
const PARALLAX_HORIZON_HEIGHT: f32 = 0.32;
const SUN_Z_INDEX: i32 = -3;
const DISTANT_LAYER_Z_INDEX: i32 = -2;
const RIDGE_Z_INDEX: i32 = -1;
const HILL_BUSH_Z_INDEX: i32 = 0;
const BACKGROUND_TREE_Z_INDEX: i32 = 1;
const GROUND_LAYER_Z_INDEX: i32 = 2;
const MEADOW_Z_INDEX: i32 = 2;
const MIDGROUND_TREE_Z_INDEX: i32 = 3;
const DISTANT_FLOWER_LAYER_Z_INDEX: i32 = 3;
pub const LESSON_MASCOT_LAYER_Z_INDEX: i32 = 4;
const PATH_STONE_Z_INDEX: i32 = 5;
const FOREGROUND_LAYER_Z_INDEX: i32 = 5;
const FLOWER_Z_INDEX: i32 = 5;
const GRASS_TUFT_Z_INDEX: i32 = 6;
const FOREGROUND_TREE_Z_INDEX: i32 = 7;

pub struct LessonBackgroundPlugin;

#[derive(Component, Reflect)]
pub struct LessonBackgroundRoot;

#[derive(Component, Reflect)]
pub struct LessonMascotLayer;

pub fn lesson_mascot_layer() -> impl Bundle + use<> {
    (
        full_screen_overlay_node(),
        ZIndex(LESSON_MASCOT_LAYER_Z_INDEX),
        Pickable::IGNORE,
        LessonMascotLayer,
    )
}

#[derive(SystemParam)]
pub struct LessonBackgroundAssets<'w> {
    pub landscape: ResMut<'w, Assets<LandscapeMaterial>>,
    pub bushes: ResMut<'w, Assets<BushMaterial>>,
    pub trees: ResMut<'w, Assets<TreeMaterial>>,
    pub clouds: ResMut<'w, Assets<CloudUiMaterial>>,
    pub sun: ResMut<'w, Assets<SunMaterial>>,
    pub flowers: ResMut<'w, Assets<FlowerMaterial>>,
    pub grass: ResMut<'w, Assets<GrassMaterial>>,
    pub stones: ResMut<'w, Assets<StoneMaterial>>,
}

impl Plugin for LessonBackgroundPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(
            Update,
            (
                update_path_stone_positions,
                update_tree_positions,
                update_hill_bush_positions,
            )
                .run_if(in_state(ActiveLesson)),
        )
        .add_systems(
            Update,
            animate_lesson_vegetation
                .after(LessonMotionSet::AdvanceWind)
                .run_if(in_state(ActiveLesson)),
        )
        .add_systems(
            PostUpdate,
            animate_lesson_clouds
                .after(UiSystems::Propagate)
                .before(UiSystems::Layout)
                .run_if(in_state(ActiveLesson)),
        );
    }
}

#[derive(Component, Clone, Copy, Reflect)]
#[require(UiTransform)]
pub struct LessonBackgroundParallaxLayer {
    vertical_travel: f32,
    vertical_compression: f32,
}

impl LessonBackgroundParallaxLayer {
    pub fn departure_transform(self, viewport_height: f32, progress: f32) -> (f32, f32) {
        let vertical_scale = (-progress).mul_add(self.vertical_compression, 1.0);
        let horizon_anchor = viewport_height * PARALLAX_HORIZON_HEIGHT;
        let anchor_distance = (-viewport_height).mul_add(0.5, horizon_anchor);
        let anchor_offset = (1.0 - vertical_scale) * anchor_distance;
        let travel = viewport_height * self.vertical_travel * progress;
        (vertical_scale, anchor_offset + travel)
    }
}

#[derive(Component, Clone, Copy, Reflect)]
#[require(UiTransform)]
struct LessonBackgroundCloud {
    center_x_percent: f32,
    canvas_width_design_px: f32,
    placement_scale: f32,
    speed_design_px_per_second: f32,
    offset_design_px: f32,
    age_seconds: f32,
}

#[derive(Component, Reflect)]
struct LessonLandscapeMaterials {
    flower_materials: [Handle<FlowerMaterial>; 3],
    grass_material: Handle<GrassMaterial>,
}

#[derive(Clone, Copy)]
struct HillBushPlacement {
    center_offset: f32,
    offset_from_crest: f32,
    width_design_px: f32,
    height_design_px: f32,
}

impl HillBushPlacement {
    const fn new(
        center_offset: f32,
        offset_from_crest: f32,
        width_design_px: f32,
        height_design_px: f32,
    ) -> Self {
        Self {
            center_offset,
            offset_from_crest,
            width_design_px,
            height_design_px,
        }
    }
}

#[derive(Clone, Copy)]
struct VisualPlacement {
    center_x: f32,
    center_y: f32,
    width: f32,
    height: f32,
    scale: f32,
    rotation: f32,
}

impl VisualPlacement {
    const fn new(center_x: f32, center_y: f32, width: f32, height: f32, scale: f32) -> Self {
        Self {
            center_x,
            center_y,
            width,
            height,
            scale,
            rotation: 0.0,
        }
    }

    const fn rotated(mut self, rotation: f32) -> Self {
        self.rotation = rotation;
        self
    }
}

#[derive(Clone, Copy)]
struct CloudPlacement {
    shape: CloudShapeId,
    center_x: f32,
    center_y: f32,
    scale: f32,
}

const CLOUDS: [CloudPlacement; 12] = [
    CloudPlacement {
        shape: CloudShapeId::Puffy,
        center_x: 9.0,
        center_y: 11.0,
        scale: 0.46,
    },
    CloudPlacement {
        shape: CloudShapeId::Elongated,
        center_x: 35.0,
        center_y: 16.0,
        scale: 0.42,
    },
    CloudPlacement {
        shape: CloudShapeId::Puffy,
        center_x: 67.0,
        center_y: 8.0,
        scale: 0.40,
    },
    CloudPlacement {
        shape: CloudShapeId::Elongated,
        center_x: 88.0,
        center_y: 18.0,
        scale: 0.36,
    },
    CloudPlacement {
        shape: CloudShapeId::Puffy,
        center_x: 23.0,
        center_y: 6.0,
        scale: 0.32,
    },
    CloudPlacement {
        shape: CloudShapeId::Elongated,
        center_x: 54.0,
        center_y: 22.0,
        scale: 0.32,
    },
    CloudPlacement {
        shape: CloudShapeId::Puffy,
        center_x: 80.0,
        center_y: 13.0,
        scale: 0.31,
    },
    CloudPlacement {
        shape: CloudShapeId::Elongated,
        center_x: 96.0,
        center_y: 6.0,
        scale: 0.28,
    },
    CloudPlacement {
        shape: CloudShapeId::Puffy,
        center_x: 17.0,
        center_y: 22.0,
        scale: 0.30,
    },
    CloudPlacement {
        shape: CloudShapeId::Elongated,
        center_x: 46.0,
        center_y: 9.0,
        scale: 0.30,
    },
    CloudPlacement {
        shape: CloudShapeId::Puffy,
        center_x: 75.0,
        center_y: 23.0,
        scale: 0.28,
    },
    CloudPlacement {
        shape: CloudShapeId::Elongated,
        center_x: 94.0,
        center_y: 14.0,
        scale: 0.25,
    },
];

#[derive(Clone, Copy)]
enum FlowerPalette {
    Gold,
    Pink,
    Lavender,
}

const HILL_BUSH_PLACEMENTS: [HillBushPlacement; 6] = [
    HillBushPlacement::new(-0.12, 0.055, 20.0, 9.0),
    HillBushPlacement::new(-0.08, 0.030, 16.0, 7.0),
    HillBushPlacement::new(-0.035, 0.065, 22.0, 9.0),
    HillBushPlacement::new(0.045, 0.035, 18.0, 8.0),
    HillBushPlacement::new(0.08, 0.055, 20.0, 9.0),
    HillBushPlacement::new(0.11, 0.070, 14.0, 7.0),
];

const DISTANT_FLOWERS: [(VisualPlacement, FlowerPalette); 6] = [
    (
        VisualPlacement::new(36.0, 35.0, 68.0, 88.0, 0.38),
        FlowerPalette::Gold,
    ),
    (
        VisualPlacement::new(69.0, 39.0, 61.0, 78.0, 0.42),
        FlowerPalette::Pink,
    ),
    (
        VisualPlacement::new(25.0, 51.0, 66.0, 86.0, 0.55),
        FlowerPalette::Lavender,
    ),
    (
        VisualPlacement::new(52.0, 34.0, 60.0, 78.0, 0.36),
        FlowerPalette::Pink,
    ),
    (
        VisualPlacement::new(87.0, 48.0, 62.0, 80.0, 0.48),
        FlowerPalette::Gold,
    ),
    (
        VisualPlacement::new(47.0, 53.0, 64.0, 82.0, 0.56),
        FlowerPalette::Lavender,
    ),
];

const NEAR_FLOWERS: [(VisualPlacement, FlowerPalette); 6] = [
    (
        VisualPlacement::new(64.0, 61.0, 60.0, 78.0, 0.66),
        FlowerPalette::Gold,
    ),
    (
        VisualPlacement::new(36.0, 74.0, 68.0, 88.0, 0.80),
        FlowerPalette::Pink,
    ),
    (
        VisualPlacement::new(78.0, 85.0, 72.0, 94.0, 0.94),
        FlowerPalette::Lavender,
    ),
    (
        VisualPlacement::new(83.0, 64.0, 64.0, 82.0, 0.69),
        FlowerPalette::Pink,
    ),
    (
        VisualPlacement::new(58.0, 73.0, 68.0, 88.0, 0.79),
        FlowerPalette::Gold,
    ),
    (
        VisualPlacement::new(23.0, 88.0, 72.0, 94.0, 0.98),
        FlowerPalette::Lavender,
    ),
];

const GRASS_TUFTS: [VisualPlacement; 16] = [
    VisualPlacement::new(18.0, 42.0, 96.0, 54.0, 0.36),
    VisualPlacement::new(50.0, 46.0, 82.0, 48.0, 0.42),
    VisualPlacement::new(79.0, 49.0, 94.0, 53.0, 0.48),
    VisualPlacement::new(32.0, 57.0, 84.0, 48.0, 0.60).rotated(-0.045),
    VisualPlacement::new(69.0, 67.0, 96.0, 54.0, 0.68),
    VisualPlacement::new(24.0, 77.0, 82.0, 48.0, 0.78),
    VisualPlacement::new(56.0, 83.0, 92.0, 52.0, 0.88),
    VisualPlacement::new(84.0, 91.0, 100.0, 56.0, 0.98),
    VisualPlacement::new(35.0, 36.0, 84.0, 48.0, 0.34),
    VisualPlacement::new(84.0, 40.0, 96.0, 54.0, 0.39),
    VisualPlacement::new(61.0, 53.0, 82.0, 48.0, 0.53),
    VisualPlacement::new(44.0, 65.0, 90.0, 51.0, 0.66),
    VisualPlacement::new(86.0, 74.0, 96.0, 54.0, 0.77),
    VisualPlacement::new(40.0, 82.0, 82.0, 48.0, 0.86),
    VisualPlacement::new(70.0, 88.0, 96.0, 54.0, 0.95),
    VisualPlacement::new(29.0, 92.0, 88.0, 50.0, 0.98),
];

#[derive(Clone, Copy, Debug, Reflect)]
enum PathStoneSide {
    Left,
    Center,
    Right,
}

#[derive(Clone, Copy)]
struct PathStonePlacement {
    y: f32,
    side: PathStoneSide,
    scale: f32,
}

const PATH_STONE_GAP_PERCENT: f32 = 0.25;

const PATH_STONES: [PathStonePlacement; 10] = [
    PathStonePlacement {
        y: 34.0,
        side: PathStoneSide::Left,
        scale: 0.36,
    },
    PathStonePlacement {
        y: 44.0,
        side: PathStoneSide::Right,
        scale: 0.44,
    },
    PathStonePlacement {
        y: 49.0,
        side: PathStoneSide::Left,
        scale: 0.48,
    },
    PathStonePlacement {
        y: 55.0,
        side: PathStoneSide::Center,
        scale: 0.54,
    },
    PathStonePlacement {
        y: 66.0,
        side: PathStoneSide::Left,
        scale: 0.65,
    },
    PathStonePlacement {
        y: 71.0,
        side: PathStoneSide::Right,
        scale: 0.70,
    },
    PathStonePlacement {
        y: 76.0,
        side: PathStoneSide::Left,
        scale: 0.76,
    },
    PathStonePlacement {
        y: 85.0,
        side: PathStoneSide::Right,
        scale: 0.88,
    },
    PathStonePlacement {
        y: 92.0,
        side: PathStoneSide::Left,
        scale: 0.98,
    },
    PathStonePlacement {
        y: 92.0,
        side: PathStoneSide::Right,
        scale: 0.98,
    },
];

#[derive(Clone, Copy, Debug, Reflect)]
enum TreeSide {
    Left,
    Right,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq, Reflect)]
enum TreeBand {
    Background,
    Midground,
    Foreground,
}

#[derive(Clone, Copy)]
struct TreePlacement {
    edge_center: f32,
    top: f32,
    base: f32,
    half_width: f32,
    scale: f32,
    rotation: f32,
    side: TreeSide,
    band: TreeBand,
}

const TREES: [TreePlacement; 13] = [
    TreePlacement {
        edge_center: 0.02,
        top: 0.12,
        base: 0.82,
        half_width: 0.12,
        scale: 1.0,
        rotation: 0.0,
        side: TreeSide::Left,
        band: TreeBand::Background,
    },
    TreePlacement {
        edge_center: 0.12,
        top: 0.22,
        base: 0.82,
        half_width: 0.09,
        scale: 1.0,
        rotation: 0.0,
        side: TreeSide::Left,
        band: TreeBand::Background,
    },
    TreePlacement {
        edge_center: 0.02,
        top: 0.12,
        base: 0.82,
        half_width: 0.12,
        scale: 1.0,
        rotation: 0.0,
        side: TreeSide::Right,
        band: TreeBand::Background,
    },
    TreePlacement {
        edge_center: 0.12,
        top: 0.22,
        base: 0.82,
        half_width: 0.09,
        scale: 1.0,
        rotation: 0.0,
        side: TreeSide::Right,
        band: TreeBand::Background,
    },
    TreePlacement {
        edge_center: 0.195,
        top: 0.24,
        base: 0.37,
        half_width: 0.045,
        scale: 1.0,
        rotation: 0.0,
        side: TreeSide::Left,
        band: TreeBand::Midground,
    },
    TreePlacement {
        edge_center: 0.31,
        top: 0.27,
        base: 0.38,
        half_width: 0.035,
        scale: 1.0,
        rotation: 0.0,
        side: TreeSide::Left,
        band: TreeBand::Midground,
    },
    TreePlacement {
        edge_center: 0.42,
        top: 0.28,
        base: 0.36,
        half_width: 0.025,
        scale: 1.0,
        rotation: 0.0,
        side: TreeSide::Left,
        band: TreeBand::Midground,
    },
    TreePlacement {
        edge_center: 0.02,
        top: 0.40,
        base: 1.04,
        half_width: 0.15,
        scale: 1.0,
        rotation: 0.0,
        side: TreeSide::Left,
        band: TreeBand::Foreground,
    },
    TreePlacement {
        edge_center: 0.13,
        top: 0.50,
        base: 1.04,
        half_width: 0.105,
        scale: 1.0,
        rotation: 0.0,
        side: TreeSide::Left,
        band: TreeBand::Foreground,
    },
    TreePlacement {
        edge_center: 0.21,
        top: 0.63,
        base: 1.04,
        half_width: 0.075,
        scale: 1.0,
        rotation: 0.0,
        side: TreeSide::Left,
        band: TreeBand::Foreground,
    },
    TreePlacement {
        edge_center: 0.02,
        top: 0.40,
        base: 1.04,
        half_width: 0.15,
        scale: 1.0,
        rotation: 0.0,
        side: TreeSide::Right,
        band: TreeBand::Foreground,
    },
    TreePlacement {
        edge_center: 0.13,
        top: 0.50,
        base: 1.04,
        half_width: 0.105,
        scale: 1.0,
        rotation: 0.0,
        side: TreeSide::Right,
        band: TreeBand::Foreground,
    },
    TreePlacement {
        edge_center: 0.21,
        top: 0.63,
        base: 1.04,
        half_width: 0.075,
        scale: 1.0,
        rotation: 0.0,
        side: TreeSide::Right,
        band: TreeBand::Foreground,
    },
];

#[derive(Component, Clone, Copy, Reflect)]
struct PathStonePosition {
    window: Entity,
    y: f32,
    side: PathStoneSide,
    scale: f32,
}

#[derive(Component, Clone, Copy, Reflect)]
struct ResponsiveHillBushPosition {
    window: Entity,
    center_offset: f32,
    offset_from_crest: f32,
}

#[derive(Component, Clone, Copy, Reflect)]
struct ResponsiveTreePosition {
    window: Entity,
    edge_center: f32,
    half_width: f32,
    scale: f32,
    side: TreeSide,
}

#[derive(Clone)]
struct BackgroundMaterials {
    sky: Handle<LandscapeMaterial>,
    ridge: Handle<LandscapeMaterial>,
    bushes: Handle<BushMaterial>,
    meadow: Handle<LandscapeMaterial>,
    background_tree: Handle<TreeMaterial>,
    midground_tree: Handle<TreeMaterial>,
    foreground_tree: Handle<TreeMaterial>,
    puffy_cloud: Handle<CloudUiMaterial>,
    elongated_cloud: Handle<CloudUiMaterial>,
    gold_flower: Handle<FlowerMaterial>,
    pink_flower: Handle<FlowerMaterial>,
    lavender_flower: Handle<FlowerMaterial>,
    grass: Handle<GrassMaterial>,
    stone: Handle<StoneMaterial>,
}

impl LessonLandscapeMaterials {
    fn new(materials: &BackgroundMaterials) -> Self {
        Self {
            flower_materials: [
                materials.gold_flower.clone(),
                materials.pink_flower.clone(),
                materials.lavender_flower.clone(),
            ],
            grass_material: materials.grass.clone(),
        }
    }
}

impl BackgroundMaterials {
    fn new(assets: &mut LessonBackgroundAssets) -> Self {
        Self {
            sky: assets
                .landscape
                .add(LandscapeMaterial::new(LandscapeLayer::Sky)),
            ridge: assets
                .landscape
                .add(LandscapeMaterial::new(LandscapeLayer::Ridge)),
            bushes: assets.bushes.add(BushMaterial::new(
                Color::srgba(0.20, 0.37, 0.28, 0.94),
                Color::srgba(0.32, 0.49, 0.34, 1.0),
            )),
            meadow: assets
                .landscape
                .add(LandscapeMaterial::new(LandscapeLayer::Meadow)),
            background_tree: assets
                .trees
                .add(TreeMaterial::new(Color::srgba(0.24, 0.44, 0.35, 1.0))),
            midground_tree: assets
                .trees
                .add(TreeMaterial::new(Color::srgba(0.36, 0.53, 0.46, 1.0))),
            foreground_tree: assets
                .trees
                .add(TreeMaterial::new(Color::srgba(0.17, 0.35, 0.29, 1.0))),
            puffy_cloud: assets.clouds.add(CloudUiMaterial::new(
                CloudShapeId::Puffy,
                Color::srgba(1.0, 1.0, 1.0, 0.52),
            )),
            elongated_cloud: assets.clouds.add(CloudUiMaterial::new(
                CloudShapeId::Elongated,
                Color::srgba(0.94, 0.98, 1.0, 0.42),
            )),
            gold_flower: assets.flowers.add(FlowerMaterial::new(
                Color::srgba(0.98, 0.75, 0.22, 0.86),
                Color::srgba(0.20, 0.40, 0.20, 0.9),
                Color::srgba(0.91, 0.43, 0.12, 0.93),
            )),
            pink_flower: assets.flowers.add(FlowerMaterial::new(
                Color::srgba(0.89, 0.40, 0.54, 0.84),
                Color::srgba(0.19, 0.39, 0.19, 0.9),
                Color::srgba(0.97, 0.77, 0.28, 0.93),
            )),
            lavender_flower: assets.flowers.add(FlowerMaterial::new(
                Color::srgba(0.65, 0.49, 0.82, 0.84),
                Color::srgba(0.19, 0.39, 0.19, 0.9),
                Color::srgba(0.97, 0.78, 0.28, 0.93),
            )),
            grass: assets.grass.add(GrassMaterial::new(
                Color::srgba(0.25, 0.46, 0.20, 0.8),
                Color::srgba(0.43, 0.62, 0.25, 0.8),
            )),
            stone: assets.stones.add(StoneMaterial::new(
                Color::srgba(0.46, 0.45, 0.38, 0.88),
                Color::srgba(0.75, 0.70, 0.57, 0.75),
                Color::srgba(0.27, 0.27, 0.23, 0.34),
            )),
        }
    }

    fn flower(&self, palette: FlowerPalette) -> Handle<FlowerMaterial> {
        match palette {
            FlowerPalette::Gold => self.gold_flower.clone(),
            FlowerPalette::Pink => self.pink_flower.clone(),
            FlowerPalette::Lavender => self.lavender_flower.clone(),
        }
    }

    fn tree(&self, band: TreeBand) -> Handle<TreeMaterial> {
        match band {
            TreeBand::Background => self.background_tree.clone(),
            TreeBand::Midground => self.midground_tree.clone(),
            TreeBand::Foreground => self.foreground_tree.clone(),
        }
    }
}

pub fn spawn_lesson_background(
    commands: &mut Commands,
    assets: &mut LessonBackgroundAssets,
    window: Entity,
    viewport_size: Vec2,
) -> Entity {
    let materials = BackgroundMaterials::new(assets);
    let motion = LessonLandscapeMaterials::new(&materials);
    let sun_material = assets.sun.add(sky_sun_material());
    commands
        .spawn((
            full_screen_material_bundle(materials.sky.clone()),
            motion,
            LessonBackgroundRoot,
            GlobalZIndex(layers::GLOBAL_BACKGROUND),
            Pickable::IGNORE,
            DespawnOnExit(ActiveLesson),
            Children::spawn(SpawnWith(move |parent: &mut ChildSpawner| {
                parent.spawn((sky_sun(sun_material), ZIndex(SUN_Z_INDEX)));
                spawn_distant_layer(parent, &materials, window, viewport_size);
                spawn_ground_layer(parent, &materials, window, viewport_size);
                spawn_distant_flower_layer(parent, &materials);
                spawn_foreground_layer(parent, &materials, window, viewport_size);
            })),
        ))
        .id()
}

fn spawn_distant_layer(
    parent: &mut ChildSpawner,
    materials: &BackgroundMaterials,
    window: Entity,
    viewport_size: Vec2,
) {
    let materials = materials.clone();
    parent.spawn((
        parallax_layer_bundle(0.025, 0.015, DISTANT_LAYER_Z_INDEX),
        Children::spawn(SpawnWith(move |layer: &mut ChildSpawner| {
            spawn_clouds(layer, &materials);
            layer.spawn((
                full_screen_material_bundle(materials.ridge.clone()),
                ZIndex(RIDGE_Z_INDEX),
                Pickable::IGNORE,
            ));
            spawn_hill_bushes(layer, &materials, window, viewport_size);
            spawn_trees(
                layer,
                &materials,
                window,
                viewport_size,
                TreeBand::Background,
            );
        })),
    ));
}

fn spawn_hill_bushes(
    parent: &mut ChildSpawner,
    materials: &BackgroundMaterials,
    window: Entity,
    viewport_size: Vec2,
) {
    for placement in HILL_BUSH_PLACEMENTS {
        parent.spawn((
            hill_bush_bundle(placement, materials.bushes.clone(), viewport_size),
            ResponsiveHillBushPosition {
                window,
                center_offset: placement.center_offset,
                offset_from_crest: placement.offset_from_crest,
            },
        ));
    }
}

fn hill_bush_bundle(
    placement: HillBushPlacement,
    material: Handle<BushMaterial>,
    viewport_size: Vec2,
) -> impl Bundle + use<> {
    let position = hill_bush_position(
        placement.center_offset,
        placement.offset_from_crest,
        viewport_size,
    );
    (
        Node {
            position_type: PositionType::Absolute,
            left: percent(position.x),
            top: percent(position.y),
            width: theme::scaled(placement.width_design_px),
            height: theme::scaled(placement.height_design_px),
            ..default()
        },
        UiTransform {
            translation: Val2::percent(-50.0, 0.0),
            scale: Vec2::ONE,
            rotation: Rot2::IDENTITY,
        },
        MaterialNode(material),
        ZIndex(HILL_BUSH_Z_INDEX),
        Pickable::IGNORE,
    )
}

fn hill_bush_position(center_offset: f32, offset_from_crest: f32, viewport_size: Vec2) -> Vec2 {
    let aspect = viewport_size.x.max(0.001) / viewport_size.y.max(0.001);
    Vec2::new(
        100.0 * RIDGE_GEOMETRY.center_percent_for_offset(center_offset, aspect),
        100.0 * (RIDGE_GEOMETRY.surface_y_at_offset(0.0) + offset_from_crest),
    )
}

fn spawn_ground_layer(
    parent: &mut ChildSpawner,
    materials: &BackgroundMaterials,
    window: Entity,
    viewport_size: Vec2,
) {
    let materials = materials.clone();
    parent.spawn((
        parallax_layer_bundle(0.07, 0.04, GROUND_LAYER_Z_INDEX),
        Children::spawn(SpawnWith(move |layer: &mut ChildSpawner| {
            layer.spawn((
                full_screen_material_bundle(materials.meadow.clone()),
                ZIndex(MEADOW_Z_INDEX),
                Pickable::IGNORE,
            ));
            spawn_trees(
                layer,
                &materials,
                window,
                viewport_size,
                TreeBand::Midground,
            );
            spawn_path_stones(layer, &materials, window, viewport_size);
        })),
    ));
}

fn spawn_distant_flower_layer(parent: &mut ChildSpawner, materials: &BackgroundMaterials) {
    let materials = materials.clone();
    parent.spawn((
        parallax_layer_bundle(0.09, 0.06, DISTANT_FLOWER_LAYER_Z_INDEX),
        Children::spawn(SpawnWith(move |layer: &mut ChildSpawner| {
            spawn_flowers(layer, &materials, &DISTANT_FLOWERS);
        })),
    ));
}

fn spawn_foreground_layer(
    parent: &mut ChildSpawner,
    materials: &BackgroundMaterials,
    window: Entity,
    viewport_size: Vec2,
) {
    let materials = materials.clone();
    parent.spawn((
        parallax_layer_bundle(0.11, 0.08, FOREGROUND_LAYER_Z_INDEX),
        Children::spawn(SpawnWith(move |layer: &mut ChildSpawner| {
            spawn_flowers(layer, &materials, &NEAR_FLOWERS);
            spawn_grass_tufts(layer, &materials);
            spawn_trees(
                layer,
                &materials,
                window,
                viewport_size,
                TreeBand::Foreground,
            );
        })),
    ));
}

fn parallax_layer_bundle(
    vertical_travel: f32,
    vertical_compression: f32,
    z_index: i32,
) -> impl Bundle + use<> {
    (
        full_screen_overlay_node(),
        LessonBackgroundParallaxLayer {
            vertical_travel,
            vertical_compression,
        },
        ZIndex(z_index),
        Pickable::IGNORE,
    )
}

fn full_screen_material_bundle<M: UiMaterial>(material: Handle<M>) -> impl Bundle + use<M> {
    (full_screen_overlay_node(), MaterialNode(material))
}

fn spawn_clouds(parent: &mut ChildSpawner, materials: &BackgroundMaterials) {
    for cloud in CLOUDS {
        let material = match cloud.shape {
            CloudShapeId::Puffy => materials.puffy_cloud.clone(),
            CloudShapeId::Elongated => materials.elongated_cloud.clone(),
        };
        let canvas = cloud.shape.canvas();
        let placement = VisualPlacement::new(
            cloud.center_x,
            cloud.center_y,
            canvas.x,
            canvas.y,
            cloud.scale,
        );
        let motion = LessonBackgroundCloud {
            center_x_percent: cloud.center_x,
            canvas_width_design_px: canvas.x,
            placement_scale: cloud.scale,
            speed_design_px_per_second: CLOUD_DRIFT_SPEED_DESIGN_PX_PER_SECOND,
            offset_design_px: 0.0,
            age_seconds: 0.0,
        };

        parent.spawn((
            placed_material_container_bundle(placement, -2),
            children![(
                Node {
                    position_type: PositionType::Absolute,
                    left: px(0.0),
                    top: px(0.0),
                    width: percent(100.0),
                    height: percent(100.0),
                    ..default()
                },
                MaterialNode(material),
                Pickable::IGNORE,
                motion,
            )],
        ));
    }
}

fn spawn_flowers(
    parent: &mut ChildSpawner,
    materials: &BackgroundMaterials,
    flowers: &[(VisualPlacement, FlowerPalette)],
) {
    for &(placement, palette) in flowers {
        parent.spawn(placed_material_bundle(
            placement,
            materials.flower(palette),
            FLOWER_Z_INDEX,
        ));
    }
}

fn spawn_grass_tufts(parent: &mut ChildSpawner, materials: &BackgroundMaterials) {
    for placement in GRASS_TUFTS {
        parent.spawn(placed_material_bundle(
            placement,
            materials.grass.clone(),
            GRASS_TUFT_Z_INDEX,
        ));
    }
}

fn spawn_path_stones(
    parent: &mut ChildSpawner,
    materials: &BackgroundMaterials,
    window: Entity,
    viewport_size: Vec2,
) {
    for placement in PATH_STONES {
        parent.spawn((
            placed_material_bundle(
                VisualPlacement::new(
                    path_stone_x_percent(placement, viewport_size),
                    placement.y,
                    48.0,
                    30.0,
                    placement.scale,
                ),
                materials.stone.clone(),
                PATH_STONE_Z_INDEX,
            ),
            PathStonePosition {
                window,
                y: placement.y / 100.0,
                side: placement.side,
                scale: placement.scale,
            },
        ));
    }
}

fn spawn_trees(
    parent: &mut ChildSpawner,
    materials: &BackgroundMaterials,
    window: Entity,
    viewport_size: Vec2,
    band: TreeBand,
) {
    for placement in TREES.into_iter().filter(|tree| tree.band == band) {
        parent.spawn((
            tree_bundle(
                placement,
                materials.tree(placement.band),
                viewport_size,
                band.z_index(),
            ),
            ResponsiveTreePosition {
                window,
                edge_center: placement.edge_center,
                half_width: placement.half_width,
                scale: placement.scale,
                side: placement.side,
            },
        ));
    }
}

impl TreeBand {
    const fn z_index(self) -> i32 {
        match self {
            Self::Background => BACKGROUND_TREE_Z_INDEX,
            Self::Midground => MIDGROUND_TREE_Z_INDEX,
            Self::Foreground => FOREGROUND_TREE_Z_INDEX,
        }
    }
}

fn tree_bundle(
    placement: TreePlacement,
    material: Handle<TreeMaterial>,
    viewport_size: Vec2,
    z_index: i32,
) -> impl Bundle + use<> {
    let aspect = viewport_size.x / viewport_size.y;
    let center = tree_center_percent(placement.edge_center, placement.side, aspect);
    let width = tree_width_percent(placement.half_width, placement.scale, aspect);
    let top = placement.top * 100.0;
    let height = (placement.base - placement.top) * placement.scale * 100.0;
    (
        Node {
            position_type: PositionType::Absolute,
            left: percent(center),
            top: percent(top),
            width: percent(width),
            height: percent(height),
            ..default()
        },
        UiTransform {
            translation: Val2::percent(-50.0, 0.0),
            scale: Vec2::ONE,
            rotation: Rot2::radians(placement.rotation),
        },
        MaterialNode(material),
        ZIndex(z_index),
        Pickable::IGNORE,
    )
}

fn tree_center_percent(edge_center: f32, side: TreeSide, aspect: f32) -> f32 {
    let center = match side {
        TreeSide::Left => edge_center,
        TreeSide::Right => aspect - edge_center,
    };
    center / aspect * 100.0
}

fn tree_width_percent(half_width: f32, scale: f32, aspect: f32) -> f32 {
    2.0 * half_width * scale / aspect * 100.0
}

fn placed_material_bundle<M: UiMaterial>(
    placement: VisualPlacement,
    material: Handle<M>,
    z_index: i32,
) -> impl Bundle + use<M> {
    (
        placed_material_container_bundle(placement, z_index),
        MaterialNode(material),
    )
}

fn placed_material_container_bundle(
    placement: VisualPlacement,
    z_index: i32,
) -> impl Bundle + use<> {
    (
        Node {
            position_type: PositionType::Absolute,
            left: percent(placement.center_x),
            top: percent(placement.center_y),
            width: theme::scaled(placement.width),
            height: theme::scaled(placement.height),
            ..default()
        },
        UiTransform {
            translation: Val2::percent(-50.0, -50.0),
            scale: Vec2::splat(placement.scale),
            rotation: Rot2::radians(placement.rotation),
        },
        ZIndex(z_index),
        Pickable::IGNORE,
    )
}

fn animate_lesson_clouds(
    time: Res<Time>,
    mut clouds: Query<(
        &mut LessonBackgroundCloud,
        &mut UiTransform,
        &ComputedUiRenderTargetInfo,
    )>,
) {
    let delta = time.delta_secs();
    for (mut cloud, mut transform, target) in &mut clouds {
        let logical_size = target.logical_size();
        let vmin = logical_size.min_element();
        if vmin <= 0.0 || logical_size.x <= 0.0 {
            continue;
        }

        cloud.age_seconds = (cloud.age_seconds + delta).min(CLOUD_MOTION_FADE_SECONDS);
        let fade = smooth_motion_fade(cloud.age_seconds, CLOUD_MOTION_FADE_SECONDS);
        let design_scale = theme::design_scale_factor(vmin);
        let visual_scale = design_scale * cloud.placement_scale;
        let half_width = cloud.canvas_width_design_px * visual_scale * 0.5;
        let base_center = logical_size.x * cloud.center_x_percent * 0.01;
        let wrap_span = 2.0_f32.mul_add(half_width, logical_size.x);
        let distance = cloud.speed_design_px_per_second * delta * fade * visual_scale;
        let center =
            (cloud.offset_design_px.mul_add(visual_scale, base_center) + distance + half_width)
                .rem_euclid(wrap_span)
                - half_width;
        cloud.offset_design_px = (center - base_center) / visual_scale;
        transform.translation = Val2::new(px(cloud.offset_design_px * design_scale), px(0.0));
    }
}

fn animate_lesson_vegetation(
    wind: Res<LessonWind>,
    landscapes: Query<&LessonLandscapeMaterials>,
    mut flower_materials: ResMut<Assets<FlowerMaterial>>,
    mut grass_materials: ResMut<Assets<GrassMaterial>>,
) {
    let strength = wind.value();

    for landscape in &landscapes {
        for handle in &landscape.flower_materials {
            if let Some(mut material) = flower_materials.get_mut(handle) {
                material.set_bend(strength * FLOWER_BEND_AMOUNT);
            }
        }
        if let Some(mut material) = grass_materials.get_mut(&landscape.grass_material) {
            material.set_bend(strength * GRASS_BEND_AMOUNT);
        }
    }
}

fn update_path_stone_positions(
    mut resized_windows: MessageReader<WindowResized>,
    mut stones: Query<(&PathStonePosition, &mut Node)>,
) {
    for resized_window in resized_windows.read() {
        let viewport_size = Vec2::new(resized_window.width, resized_window.height);
        if viewport_size.min_element() <= 0.0 {
            continue;
        }
        for (stone, mut node) in &mut stones {
            if stone.window == resized_window.window {
                let placement = PathStonePlacement {
                    y: stone.y * 100.0,
                    side: stone.side,
                    scale: stone.scale,
                };
                node.left = percent(path_stone_x_percent(placement, viewport_size));
            }
        }
    }
}

fn update_hill_bush_positions(
    mut resized_windows: MessageReader<WindowResized>,
    mut bushes: Query<(&ResponsiveHillBushPosition, &mut Node)>,
) {
    for resized_window in resized_windows.read() {
        let viewport_size = Vec2::new(resized_window.width, resized_window.height);
        if viewport_size.min_element() <= 0.0 {
            continue;
        }
        for (bush, mut node) in &mut bushes {
            if bush.window == resized_window.window {
                let position =
                    hill_bush_position(bush.center_offset, bush.offset_from_crest, viewport_size);
                node.left = percent(position.x);
                node.top = percent(position.y);
            }
        }
    }
}

fn update_tree_positions(
    mut resized_windows: MessageReader<WindowResized>,
    mut trees: Query<(&ResponsiveTreePosition, &mut Node)>,
) {
    for resized_window in resized_windows.read() {
        let viewport_size = Vec2::new(resized_window.width, resized_window.height);
        if viewport_size.min_element() <= 0.0 {
            continue;
        }
        let aspect = viewport_size.x / viewport_size.y;
        for (tree, mut node) in &mut trees {
            if tree.window == resized_window.window {
                node.left = percent(tree_center_percent(tree.edge_center, tree.side, aspect));
                node.width = percent(tree_width_percent(tree.half_width, tree.scale, aspect));
            }
        }
    }
}

fn path_stone_x_percent(placement: PathStonePlacement, viewport_size: Vec2) -> f32 {
    let aspect = viewport_size.x / viewport_size.y;
    let y = placement.y / 100.0;
    let center = PATH_GEOMETRY.center_percent(y);
    let half_width = PATH_GEOMETRY.half_width_percent(y, aspect);
    let design_scale = theme::design_scale_factor(viewport_size.min_element());
    let stone_half_width =
        48.0 * design_scale * placement.scale * STONE_HALF_WIDTH_RATIO * 100.0 / viewport_size.x;
    match placement.side {
        PathStoneSide::Left => center - half_width - stone_half_width - PATH_STONE_GAP_PERCENT,
        PathStoneSide::Center => center,
        PathStoneSide::Right => center + half_width + stone_half_width + PATH_STONE_GAP_PERCENT,
    }
    .clamp(stone_half_width, 100.0 - stone_half_width)
}

const _: () = {
    assert!(RIDGE_Z_INDEX < HILL_BUSH_Z_INDEX);
    assert!(HILL_BUSH_Z_INDEX < BACKGROUND_TREE_Z_INDEX);
    assert!(BACKGROUND_TREE_Z_INDEX < GROUND_LAYER_Z_INDEX);
    assert!(GROUND_LAYER_Z_INDEX < DISTANT_FLOWER_LAYER_Z_INDEX);
    assert!(DISTANT_FLOWER_LAYER_Z_INDEX < LESSON_MASCOT_LAYER_Z_INDEX);
    assert!(LESSON_MASCOT_LAYER_Z_INDEX < FOREGROUND_LAYER_Z_INDEX);
    assert!(FLOWER_Z_INDEX < GRASS_TUFT_Z_INDEX);
    assert!(GRASS_TUFT_Z_INDEX < FOREGROUND_TREE_Z_INDEX);
};

#[cfg(test)]
mod hill_bush_tests {
    use bevy::prelude::Vec2;

    use super::{HILL_BUSH_PLACEMENTS, RIDGE_GEOMETRY, hill_bush_position};

    #[test]
    fn hill_bushes_are_smaller_than_the_smallest_distant_cloud() {
        for placement in HILL_BUSH_PLACEMENTS {
            assert!(placement.width_design_px < 42.0);
            assert!(placement.height_design_px < 17.5);
        }
    }

    #[test]
    fn whole_hill_bushes_stay_inside_the_ridge_at_ultrawide_aspect() {
        let viewport_size = Vec2::new(3200.0, 900.0);
        let scale = crate::ui::theme::design_scale_factor(viewport_size.min_element());

        for placement in HILL_BUSH_PLACEMENTS {
            let position = hill_bush_position(
                placement.center_offset,
                placement.offset_from_crest,
                viewport_size,
            );
            let half_width = placement.width_design_px * scale * 0.5 / viewport_size.y;
            let left_surface_y =
                RIDGE_GEOMETRY.surface_y_at_offset(placement.center_offset - half_width);
            let center_surface_y = RIDGE_GEOMETRY.surface_y_at_offset(placement.center_offset);
            let right_surface_y =
                RIDGE_GEOMETRY.surface_y_at_offset(placement.center_offset + half_width);
            let highest_surface_y = left_surface_y.max(center_surface_y).max(right_surface_y);
            let top_y = position.y * 0.01;
            let bottom_y = top_y + placement.height_design_px * scale / viewport_size.y;

            assert!(position.x > 0.0 && position.x < 100.0);
            assert!(top_y > highest_surface_y);
            assert!(bottom_y < 0.242);
        }
    }
}

#[cfg(test)]
mod lifecycle_tests {
    use bevy::prelude::*;
    use bevy::state::app::StatesPlugin;

    use super::{LessonBackgroundRoot, LessonMascotLayer, lesson_mascot_layer};
    use crate::states::{ActiveLesson, AppState};

    #[derive(Component)]
    struct PlayBalloon;

    #[derive(Component)]
    struct SummaryBalloon;

    fn app_with_lesson_lifecycle() -> App {
        let mut app = App::new();
        app.add_plugins(StatesPlugin)
            .insert_state(AppState::LessonPlay)
            .add_computed_state::<ActiveLesson>()
            .add_systems(OnEnter(AppState::LessonPlay), spawn_play_background)
            .add_systems(OnEnter(AppState::LessonSummary), spawn_summary_balloon);
        app
    }

    fn spawn_play_background(mut commands: Commands) {
        let background = commands
            .spawn((
                Node::default(),
                LessonBackgroundRoot,
                DespawnOnExit(ActiveLesson),
            ))
            .id();
        commands.entity(background).with_children(|parent| {
            parent.spawn((
                lesson_mascot_layer(),
                DespawnOnExit(AppState::LessonPlay),
                Children::spawn(SpawnWith(|layer: &mut ChildSpawner| {
                    layer.spawn(PlayBalloon);
                })),
            ));
        });
    }

    fn spawn_summary_balloon(
        mut commands: Commands,
        background: Single<Entity, With<LessonBackgroundRoot>>,
    ) {
        commands.entity(*background).with_children(|parent| {
            parent.spawn((
                lesson_mascot_layer(),
                DespawnOnExit(AppState::LessonSummary),
                Children::spawn(SpawnWith(|layer: &mut ChildSpawner| {
                    layer.spawn(SummaryBalloon);
                })),
            ));
        });
    }

    fn count<T: Component>(app: &mut App) -> usize {
        let world = app.world_mut();
        let mut query = world.query_filtered::<Entity, With<T>>();
        query.iter(world).count()
    }

    #[test]
    fn summary_replaces_play_balloon_and_next_lesson_starts_cleanly() {
        let mut app = app_with_lesson_lifecycle();
        app.update();
        assert_eq!(count::<LessonBackgroundRoot>(&mut app), 1);
        assert_eq!(count::<LessonMascotLayer>(&mut app), 1);
        assert_eq!(count::<PlayBalloon>(&mut app), 1);
        assert_eq!(count::<SummaryBalloon>(&mut app), 0);

        app.world_mut()
            .resource_mut::<NextState<AppState>>()
            .set(AppState::LessonSummary);
        app.update();
        assert_eq!(count::<LessonBackgroundRoot>(&mut app), 1);
        assert_eq!(count::<LessonMascotLayer>(&mut app), 1);
        assert_eq!(count::<PlayBalloon>(&mut app), 0);
        assert_eq!(count::<SummaryBalloon>(&mut app), 1);

        app.world_mut()
            .resource_mut::<NextState<AppState>>()
            .set(AppState::ThemeExploration);
        app.update();
        assert_eq!(count::<LessonBackgroundRoot>(&mut app), 0);
        assert_eq!(count::<LessonMascotLayer>(&mut app), 0);
        assert_eq!(count::<SummaryBalloon>(&mut app), 0);

        app.world_mut()
            .resource_mut::<NextState<AppState>>()
            .set(AppState::LessonPlay);
        app.update();
        assert_eq!(count::<LessonBackgroundRoot>(&mut app), 1);
        assert_eq!(count::<LessonMascotLayer>(&mut app), 1);
        assert_eq!(count::<PlayBalloon>(&mut app), 1);
        assert_eq!(count::<SummaryBalloon>(&mut app), 0);
    }

    #[test]
    fn quitting_lesson_play_cleans_background_and_balloon() {
        let mut app = app_with_lesson_lifecycle();
        app.update();
        assert_eq!(count::<LessonBackgroundRoot>(&mut app), 1);
        assert_eq!(count::<LessonMascotLayer>(&mut app), 1);

        app.world_mut()
            .resource_mut::<NextState<AppState>>()
            .set(AppState::ThemeExploration);
        app.update();
        assert_eq!(count::<LessonBackgroundRoot>(&mut app), 0);
        assert_eq!(count::<LessonMascotLayer>(&mut app), 0);
        assert_eq!(count::<PlayBalloon>(&mut app), 0);
    }
}
