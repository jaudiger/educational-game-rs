use bevy::camera::visibility::RenderLayers;
use bevy::prelude::*;
use bevy::sprite_render::MeshMaterial2d;
use bevy::window::{CursorOptions, PrimaryWindow};
use rand::RngExt;

use crate::data::{ExplorationTheme, GameSettings};
use crate::plugins::balloon_material::{
    BALLOON_BASKET_CORD_ATTACHMENTS, BalloonFlame, BalloonMaterial, BalloonPart,
};
use crate::plugins::cloud_material::{CloudShapeId, ProceduralCloudMaterial};
use crate::states::AppState;

use super::{
    CURSOR_RENDER_LAYER, PointerCursor, TouchCursorState, activate_custom_cursor,
    spawn_cursor_camera,
};

pub(super) fn register(app: &mut App) {
    app.add_systems(OnEnter(AppState::ThemeExploration), setup_balloon_cursor)
        .add_systems(
            OnExit(AppState::ThemeExploration),
            (super::cleanup_custom_cursor, cleanup_cloud_puffs),
        )
        .add_systems(
            Update,
            (
                animate_balloon_system,
                animate_cursor_flame_system.after(super::follow_pointer_system),
                spawn_cloud_puffs_system.after(super::follow_pointer_system),
                animate_cloud_puffs_system,
            )
                .run_if(in_state(AppState::ThemeExploration))
                .run_if(resource_exists::<super::CustomCursorActive>),
        );
}

#[derive(Component, Reflect)]
#[require(Transform, Visibility)]
struct BalloonEnvelope {
    base_y: f32,
    #[reflect(ignore)]
    material: Handle<BalloonMaterial>,
}

#[derive(Component, Reflect)]
#[require(Transform, Visibility)]
struct BalloonBasket;

#[derive(Component, Reflect)]
struct CursorFlameMotion {
    last_position: Option<Vec2>,
    intensity: f32,
}

impl Default for CursorFlameMotion {
    fn default() -> Self {
        Self {
            last_position: None,
            intensity: BalloonFlame::BASE_INTENSITY,
        }
    }
}

#[derive(Component, Reflect)]
struct CloudPuffSpawner {
    timer: Timer,
    puff_shapes: Vec<CloudShapeId>,
    #[reflect(ignore)]
    puff_meshes: Vec<Handle<Mesh>>,
}

#[derive(Component, Reflect)]
struct CloudPuff {
    lifetime: Timer,
    velocity: Vec2,
    initial_opacity: f32,
}

const ENVELOPE_SCALE: f32 = 0.19;
const BASKET_SCALE: f32 = 0.25;
const BASKET_OFFSET_Y: f32 = 23.0;
const ENVELOPE_BASE_Y: f32 = 20.0;

fn setup_balloon_cursor(
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut balloon_materials: ResMut<Assets<BalloonMaterial>>,
    settings: Res<GameSettings>,
    mut cursor_options: Single<&mut CursorOptions, With<PrimaryWindow>>,
) {
    if settings.exploration_theme != ExplorationTheme::Sky {
        return;
    }

    activate_custom_cursor(&mut commands, &mut cursor_options);

    let envelope_canvas = BalloonPart::Envelope.canvas();
    let basket_canvas = BalloonPart::Basket.canvas();
    let envelope_mesh = meshes.add(Rectangle::new(envelope_canvas.x, envelope_canvas.y));
    let basket_mesh = meshes.add(Rectangle::new(basket_canvas.x, basket_canvas.y));
    let envelope_material = balloon_materials.add(BalloonMaterial::new(BalloonPart::Envelope));
    let basket_material = balloon_materials.add(BalloonMaterial::new(BalloonPart::Basket));
    let flame_material = balloon_materials.add(BalloonMaterial::new(BalloonPart::Flame));
    let flame_mesh = meshes.add(Rectangle::new(
        BalloonPart::Flame.canvas().x,
        BalloonPart::Flame.canvas().y,
    ));

    let puff_shapes = CloudShapeId::ALL.to_vec();
    let puff_meshes = puff_shapes
        .iter()
        .map(|shape_id| {
            let canvas = shape_id.canvas();
            meshes.add(Rectangle::new(canvas.x, canvas.y))
        })
        .collect();

    let cursor_layer = RenderLayers::layer(CURSOR_RENDER_LAYER);
    spawn_cursor_camera(&mut commands, &cursor_layer);

    commands.spawn((
        PointerCursor,
        TouchCursorState::default(),
        CursorFlameMotion::default(),
        cursor_layer.clone(),
        CloudPuffSpawner {
            timer: Timer::from_seconds(1.6, TimerMode::Repeating),
            puff_shapes,
            puff_meshes,
        },
        children![
            (
                BalloonEnvelope {
                    base_y: ENVELOPE_BASE_Y,
                    material: envelope_material.clone(),
                },
                Transform::from_xyz(0.0, ENVELOPE_BASE_Y, 1.0)
                    .with_scale(Vec3::splat(ENVELOPE_SCALE)),
                children![
                    (
                        BalloonFlame,
                        Mesh2d(flame_mesh),
                        MeshMaterial2d(flame_material),
                        Transform::from_xyz(0.0, (33.0 - ENVELOPE_BASE_Y) / ENVELOPE_SCALE, -0.1,)
                            .with_scale(Vec3::splat(0.2 / ENVELOPE_SCALE)),
                        Pickable::IGNORE,
                        cursor_layer.clone(),
                    ),
                    (
                        Mesh2d(envelope_mesh),
                        MeshMaterial2d(envelope_material),
                        Transform::from_xyz(0.0, envelope_canvas.y * 0.5, 0.0),
                        Pickable::IGNORE,
                        cursor_layer.clone(),
                    ),
                ],
            ),
            (
                BalloonBasket,
                Transform::from_xyz(0.0, BASKET_OFFSET_Y, 2.0)
                    .with_scale(Vec3::splat(BASKET_SCALE)),
                children![(
                    Mesh2d(basket_mesh),
                    MeshMaterial2d(basket_material),
                    Transform::from_xyz(0.0, -basket_canvas.y * 0.5, 0.0),
                    Pickable::IGNORE,
                    cursor_layer.clone(),
                )],
            ),
        ],
    ));
}

fn cleanup_cloud_puffs(mut commands: Commands, puff_query: Query<Entity, With<CloudPuff>>) {
    for entity in &puff_query {
        commands.entity(entity).despawn();
    }
}

fn animate_balloon_system(
    time: Res<Time>,
    envelope: Single<(&BalloonEnvelope, &mut Transform), Without<BalloonBasket>>,
    basket: Single<&mut Transform, (With<BalloonBasket>, Without<BalloonEnvelope>)>,
    mut materials: ResMut<Assets<BalloonMaterial>>,
) {
    let elapsed = time.elapsed_secs();
    let (envelope, mut envelope_transform) = envelope.into_inner();
    let mut basket_transform = basket.into_inner();

    envelope_transform.rotation = Quat::from_rotation_z((elapsed * 1.5).sin() * 0.08);
    envelope_transform.translation.y = (elapsed * 1.2).sin().mul_add(3.0, envelope.base_y);
    basket_transform.rotation = Quat::from_rotation_z(((elapsed - 0.3) * 1.5).sin() * 0.06);
    basket_transform.translation.y = ((elapsed - 0.3) * 1.2).sin().mul_add(3.0, BASKET_OFFSET_Y);

    let endpoints = world_cord_endpoints(&envelope_transform, &basket_transform);
    if let Some(mut material) = materials.get_mut(&envelope.material) {
        material.set_cord_endpoints(endpoints);
    }
}

fn world_cord_endpoints(envelope: &Transform, basket: &Transform) -> [Vec4; 2] {
    let inverse_envelope = envelope.compute_affine().inverse();
    let basket_transform = basket.compute_affine();
    let basket_canvas = BalloonPart::Basket.canvas();
    let mut points = [Vec2::ZERO; 4];
    for (point, attachment) in points.iter_mut().zip(BALLOON_BASKET_CORD_ATTACHMENTS) {
        let basket_endpoint = basket_transform.transform_point3(Vec3::new(
            basket_canvas.x.mul_add(-0.5, attachment.x),
            -attachment.y,
            0.0,
        ));
        let envelope_local = inverse_envelope.transform_point3(basket_endpoint);
        *point = Vec2::new(128.0 + envelope_local.x, 320.0 - envelope_local.y);
    }

    [
        Vec4::new(points[0].x, points[0].y, points[1].x, points[1].y),
        Vec4::new(points[2].x, points[2].y, points[3].x, points[3].y),
    ]
}

fn animate_cursor_flame_system(
    time: Res<Time>,
    cursor: Single<(&Transform, &Visibility, &mut CursorFlameMotion), With<PointerCursor>>,
    flame: Single<&MeshMaterial2d<BalloonMaterial>, With<BalloonFlame>>,
    mut materials: ResMut<Assets<BalloonMaterial>>,
) {
    let (transform, visibility, mut motion) = cursor.into_inner();
    let position = transform.translation.truncate();
    let speed = motion.last_position.map_or(0.0, |previous| {
        position.distance(previous) / time.delta_secs().max(0.001)
    });
    motion.last_position = Some(position);
    let speed = if *visibility == Visibility::Visible {
        speed
    } else {
        0.0
    };
    motion.intensity = smooth_flame_intensity(motion.intensity, speed, time.delta_secs());

    if let Some(mut material) = materials.get_mut(&flame.0) {
        material.set_flame_intensity(motion.intensity);
    }
}

fn smooth_flame_intensity(current: f32, speed: f32, delta: f32) -> f32 {
    let movement = ((speed - 12.0) / 220.0).clamp(0.0, 1.0);
    let target =
        (1.0 - BalloonFlame::BASE_INTENSITY).mul_add(movement, BalloonFlame::BASE_INTENSITY);
    let smoothing = 1.0 - (-12.0 * delta.max(0.0)).exp();
    (target - current).mul_add(smoothing, current)
}

fn spawn_cloud_puffs_system(
    time: Res<Time>,
    mut commands: Commands,
    mut spawner_query: Query<(&mut CloudPuffSpawner, &Transform, &Visibility)>,
    mut materials: ResMut<Assets<ProceduralCloudMaterial>>,
) {
    let mut rng = rand::rng();

    for (mut spawner, cursor_transform, visibility) in &mut spawner_query {
        spawner.timer.tick(time.delta());

        if *visibility != Visibility::Visible || !spawner.timer.just_finished() {
            continue;
        }

        let shape_index = rng.random_range(0..spawner.puff_shapes.len());
        let shape_id = spawner.puff_shapes[shape_index];
        let material = materials.add(ProceduralCloudMaterial::new(
            shape_id,
            shape_id.canvas(),
            Color::srgba(1.0, 1.0, 1.0, 0.35),
        ));

        let root_pos = cursor_transform.translation;
        let offset_y = rng.random_range(-15.0_f32..15.0);
        let spawn_pos = Vec3::new(root_pos.x - 55.0, root_pos.y + offset_y, -1.0);
        let drift_y = rng.random_range(-8.0_f32..8.0);

        commands.spawn((
            CloudPuff {
                lifetime: Timer::from_seconds(5.0, TimerMode::Once),
                velocity: Vec2::new(20.0, drift_y),
                initial_opacity: 0.35,
            },
            Mesh2d(spawner.puff_meshes[shape_index].clone()),
            MeshMaterial2d(material),
            Transform::from_translation(spawn_pos).with_scale(Vec3::splat(0.1)),
            Pickable::IGNORE,
            RenderLayers::layer(CURSOR_RENDER_LAYER),
        ));
    }
}

fn animate_cloud_puffs_system(
    time: Res<Time>,
    mut commands: Commands,
    mut puff_query: Query<(
        Entity,
        &mut CloudPuff,
        &mut Transform,
        &MeshMaterial2d<ProceduralCloudMaterial>,
    )>,
    mut materials: ResMut<Assets<ProceduralCloudMaterial>>,
) {
    let dt = time.delta_secs();
    let elapsed = time.elapsed_secs();

    for (entity, mut puff, mut transform, material_component) in &mut puff_query {
        puff.lifetime.tick(time.delta());
        puff.velocity.y = ((elapsed * 3.0).sin() * 0.5).mul_add(dt, puff.velocity.y);
        transform.translation.x = puff.velocity.x.mul_add(dt, transform.translation.x);
        transform.translation.y = puff.velocity.y.mul_add(dt, transform.translation.y);

        let opacity = puff.initial_opacity * (1.0 - puff.lifetime.fraction());
        if let Some(mut material) = materials.get_mut(&material_component.0) {
            material.set_opacity(opacity);
        }

        if puff.lifetime.is_finished() {
            commands.entity(entity).despawn();
        }
    }
}
