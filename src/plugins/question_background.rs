use bevy::ecs::system::SystemParam;
use bevy::prelude::*;
use bevy::ui_render::ui_material::{MaterialNode, UiMaterial};
use bevy::window::WindowResized;

use crate::plugins::cloud_material::{CloudShapeId, CloudUiMaterial};
use crate::ui::theme;

use crate::plugins::flower_material::FlowerMaterial;
use crate::plugins::grass_material::GrassMaterial;
use crate::plugins::landscape_material::{LandscapeLayer, LandscapeMaterial, PathGeometry};
use crate::plugins::stone_material::{STONE_HALF_WIDTH_RATIO, StoneMaterial};
use crate::plugins::tree_material::TreeMaterial;

const PATH_GEOMETRY: PathGeometry = PathGeometry::DEFAULT;

pub struct QuestionBackgroundPlugin;

#[derive(SystemParam)]
pub struct QuestionBackgroundAssets<'w> {
    pub landscape: ResMut<'w, Assets<LandscapeMaterial>>,
    pub trees: ResMut<'w, Assets<TreeMaterial>>,
    pub clouds: ResMut<'w, Assets<CloudUiMaterial>>,
    pub flowers: ResMut<'w, Assets<FlowerMaterial>>,
    pub grass: ResMut<'w, Assets<GrassMaterial>>,
    pub stones: ResMut<'w, Assets<StoneMaterial>>,
}

impl Plugin for QuestionBackgroundPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(
            Update,
            (update_path_stone_positions, update_tree_positions)
                .run_if(in_state(crate::states::AppState::LessonPlay)),
        );
    }
}

#[derive(Component, Clone, Copy, Default, Reflect)]
#[require(UiTransform)]
pub struct QuestionBackgroundCloud;

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

const CLOUDS: [CloudPlacement; 8] = [
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
];

#[derive(Clone, Copy)]
enum FlowerPalette {
    Gold,
    Pink,
    Lavender,
}

const FLOWERS: [(VisualPlacement, FlowerPalette); 12] = [
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
struct ResponsiveTreePosition {
    window: Entity,
    edge_center: f32,
    half_width: f32,
    scale: f32,
    side: TreeSide,
}

struct BackgroundMaterials {
    sky: Handle<LandscapeMaterial>,
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

impl BackgroundMaterials {
    fn new(assets: &mut QuestionBackgroundAssets) -> Self {
        Self {
            sky: assets
                .landscape
                .add(LandscapeMaterial::new(LandscapeLayer::Sky)),
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

pub fn spawn_question_background(
    commands: &mut Commands,
    assets: &mut QuestionBackgroundAssets,
    window: Entity,
    viewport_size: Vec2,
) {
    let materials = BackgroundMaterials::new(assets);
    commands.spawn((
        full_screen_material_bundle(materials.sky.clone()),
        GlobalZIndex(-1),
        Pickable::IGNORE,
        DespawnOnExit(crate::states::AppState::LessonPlay),
        Children::spawn(SpawnWith(move |parent: &mut ChildSpawner| {
            spawn_trees(
                parent,
                &materials,
                window,
                viewport_size,
                TreeBand::Background,
            );
            parent.spawn((
                full_screen_material_bundle(materials.meadow.clone()),
                ZIndex(1),
                Pickable::IGNORE,
            ));
            spawn_trees(
                parent,
                &materials,
                window,
                viewport_size,
                TreeBand::Midground,
            );
            spawn_clouds(parent, &materials);
            spawn_flowers(parent, &materials);
            spawn_grass_tufts(parent, &materials);
            spawn_path_stones(parent, &materials, window, viewport_size);
            spawn_trees(
                parent,
                &materials,
                window,
                viewport_size,
                TreeBand::Foreground,
            );
        })),
    ));
}

fn full_screen_material_bundle<M: UiMaterial>(material: Handle<M>) -> impl Bundle + use<M> {
    (
        Node {
            position_type: PositionType::Absolute,
            left: px(0.0),
            top: px(0.0),
            width: percent(100.0),
            height: percent(100.0),
            ..default()
        },
        MaterialNode(material),
    )
}

fn spawn_clouds(parent: &mut ChildSpawner, materials: &BackgroundMaterials) {
    for cloud in CLOUDS {
        let material = match cloud.shape {
            CloudShapeId::Puffy => materials.puffy_cloud.clone(),
            CloudShapeId::Elongated => materials.elongated_cloud.clone(),
        };
        let canvas = cloud.shape.canvas();
        parent.spawn((
            placed_material_bundle(
                VisualPlacement::new(
                    cloud.center_x,
                    cloud.center_y,
                    canvas.x,
                    canvas.y,
                    cloud.scale,
                ),
                material,
                3,
            ),
            QuestionBackgroundCloud,
        ));
    }
}

fn spawn_flowers(parent: &mut ChildSpawner, materials: &BackgroundMaterials) {
    for (placement, palette) in FLOWERS {
        parent.spawn(placed_material_bundle(
            placement,
            materials.flower(palette),
            4,
        ));
    }
}

fn spawn_grass_tufts(parent: &mut ChildSpawner, materials: &BackgroundMaterials) {
    for placement in GRASS_TUFTS {
        parent.spawn(placed_material_bundle(
            placement,
            materials.grass.clone(),
            4,
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
                4,
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
            Self::Background => 0,
            Self::Midground => 2,
            Self::Foreground => 5,
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
        MaterialNode(material),
        ZIndex(z_index),
        Pickable::IGNORE,
    )
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
