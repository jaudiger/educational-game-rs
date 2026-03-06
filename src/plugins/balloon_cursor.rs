use bevy::camera::ClearColorConfig;
use bevy::camera::visibility::RenderLayers;
use bevy::prelude::*;
use bevy::sprite::Anchor;
use bevy::window::{CursorOptions, PrimaryWindow};
use bevy_persistent::prelude::Persistent;
use rand::Rng;

use crate::data::{GameSettings, MapTheme};
use crate::states::AppState;

/// Animated hot-air balloon cursor for the `MapTheme::Sky` theme.
///
/// Active only during `AppState::MapExploration` **and** when
/// `GameSettings.map_theme == MapTheme::Sky`.
///
/// # Extensibility
///
/// Each `MapTheme` variant gets its own cursor plugin (one file = one cursor).
/// Adding a new cursor theme means:
/// 1. Create a new plugin file (e.g. `ocean_cursor.rs`) with the same
///    structure: `OnEnter`/`OnExit` setup/cleanup, `Update` systems guarded
///    by `in_state(MapExploration)`, and a theme check in setup.
/// 2. Register the new plugin in `main.rs`.
///
/// Cursor plugins don't interfere: each spawns/queries its own marker
/// components and early-returns when the active `MapTheme` doesn't match.
pub struct BalloonCursorPlugin;

impl Plugin for BalloonCursorPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(OnEnter(AppState::MapExploration), setup_balloon_cursor)
            .add_systems(OnExit(AppState::MapExploration), cleanup_balloon_cursor)
            .add_systems(
                Update,
                (
                    follow_mouse_system,
                    animate_balloon_system,
                    spawn_air_waves_system,
                    animate_air_waves_system,
                )
                    .run_if(in_state(AppState::MapExploration))
                    .run_if(resource_exists::<BalloonCursorActive>),
            );
    }
}

#[derive(Component, Reflect)]
#[require(Transform, Visibility)]
struct BalloonCursor;

#[derive(Component, Reflect)]
struct BalloonEnvelope {
    base_y: f32,
}

#[derive(Component, Reflect)]
struct BalloonBasket;

#[derive(Component, Reflect)]
struct AirWaveSpawner {
    timer: Timer,
    #[reflect(ignore)]
    wave_textures: Vec<Handle<Image>>,
}

#[derive(Component, Reflect)]
struct AirWave {
    lifetime: Timer,
    velocity: Vec2,
    initial_opacity: f32,
}

/// Marker for the overlay camera that renders cursor sprites on top of UI.
#[derive(Component, Reflect)]
struct CursorCamera;

/// Inserted when the balloon cursor is actually active (theme matches).
/// Used as a run condition for Update systems so they don't run at all
/// when another theme is selected.
#[derive(Resource, Reflect)]
struct BalloonCursorActive;

/// Stores the texture handles for the balloon cursor.
#[derive(Resource, Reflect)]
struct CursorAssets {
    envelope: Handle<Image>,
    basket: Handle<Image>,
    waves: Vec<Handle<Image>>,
}

/// Render layer used exclusively for cursor sprites. The overlay camera
/// and all cursor entities share this layer so they don't interfere with
/// the main scene or UI.
const CURSOR_RENDER_LAYER: usize = 1;

/// Scale factor for the envelope sprite (~48x60 px from 256x320 source).
const ENVELOPE_SCALE: f32 = 0.19;
/// Scale factor for the basket sprite (~32x20 px from 128x80 source).
const BASKET_SCALE: f32 = 0.25;
/// Y position of the basket (anchor = `TOP_CENTER`) relative to cursor root.
/// Basket displayed height: 80 * 0.25 = 20.0 px, so bottom ~ y = 3, top ~ y = 23.
/// Slightly raised so the basket overlaps the envelope bottom by a few pixels.
const BASKET_OFFSET_Y: f32 = 23.0;
/// Y position of the envelope (anchor = `BOTTOM_CENTER`) relative to cursor root.
/// Overlaps the basket top by 3 px for a snug visual connection.
///
/// Envelope displayed height: 320 * 0.19 ~ 60.8 px
const ENVELOPE_BASE_Y: f32 = 20.0;

fn setup_balloon_cursor(
    mut commands: Commands,
    asset_server: Res<AssetServer>,
    settings: Res<Persistent<GameSettings>>,
    mut cursor_opts: Single<&mut CursorOptions, With<PrimaryWindow>>,
) {
    // Only activate for the Sky theme; other themes will have their own plugin.
    if settings.map_theme != MapTheme::Sky {
        return;
    }

    commands.insert_resource(BalloonCursorActive);

    // Hide the OS cursor
    cursor_opts.visible = false;

    // Load textures
    let envelope_texture: Handle<Image> = asset_server.load("cursor/balloon_envelope.png");
    let basket_texture: Handle<Image> = asset_server.load("cursor/balloon_basket.png");
    let wave_textures: Vec<Handle<Image>> = vec![
        asset_server.load("cursor/air_wave_1.png"),
        asset_server.load("cursor/air_wave_2.png"),
        asset_server.load("cursor/air_wave_3.png"),
    ];

    commands.insert_resource(CursorAssets {
        envelope: envelope_texture.clone(),
        basket: basket_texture.clone(),
        waves: wave_textures.clone(),
    });

    let cursor_layer = RenderLayers::layer(CURSOR_RENDER_LAYER);

    // Spawn an overlay camera that renders AFTER the main camera + UI.
    // ClearColorConfig::None ensures it draws on top without clearing.
    commands.spawn((
        Camera2d,
        Camera {
            order: 1,
            clear_color: ClearColorConfig::None,
            ..default()
        },
        cursor_layer.clone(),
        CursorCamera,
    ));

    // Spawn the cursor hierarchy (all entities on the cursor render layer)
    commands.spawn((
        BalloonCursor,
        cursor_layer.clone(),
        AirWaveSpawner {
            timer: Timer::from_seconds(0.8, TimerMode::Repeating),
            wave_textures,
        },
        children![
            // Balloon envelope
            (
                BalloonEnvelope {
                    base_y: ENVELOPE_BASE_Y,
                },
                Sprite::from_image(envelope_texture),
                Anchor::BOTTOM_CENTER,
                Transform::from_xyz(0.0, ENVELOPE_BASE_Y, 1.0)
                    .with_scale(Vec3::splat(ENVELOPE_SCALE)),
                cursor_layer.clone(),
            ),
            // Balloon basket
            (
                BalloonBasket,
                Sprite::from_image(basket_texture),
                Anchor::TOP_CENTER,
                Transform::from_xyz(0.0, BASKET_OFFSET_Y, 2.0)
                    .with_scale(Vec3::splat(BASKET_SCALE)),
                cursor_layer,
            ),
        ],
    ));
}

fn cleanup_balloon_cursor(
    mut commands: Commands,
    active: Option<Res<BalloonCursorActive>>,
    cursor_query: Query<Entity, With<BalloonCursor>>,
    wave_query: Query<Entity, With<AirWave>>,
    camera_query: Query<Entity, With<CursorCamera>>,
    mut cursor_opts: Single<&mut CursorOptions, With<PrimaryWindow>>,
) {
    // Nothing to clean up if this theme wasn't active
    if active.is_none() {
        return;
    }

    // Restore the OS cursor
    cursor_opts.visible = true;

    for entity in &cursor_query {
        commands.entity(entity).despawn();
    }
    for entity in &wave_query {
        commands.entity(entity).despawn();
    }
    for entity in &camera_query {
        commands.entity(entity).despawn();
    }

    commands.remove_resource::<CursorAssets>();
    commands.remove_resource::<BalloonCursorActive>();
}

fn follow_mouse_system(
    window: Single<&Window, With<PrimaryWindow>>,
    camera_data: Single<(&Camera, &GlobalTransform), With<CursorCamera>>,
    mut cursor_query: Query<&mut Transform, With<BalloonCursor>>,
) {
    let Some(cursor_position) = window.cursor_position() else {
        return;
    };

    // Use the cursor overlay camera for coordinate conversion
    let (camera, camera_transform) = *camera_data;

    let Ok(world_pos) = camera.viewport_to_world_2d(camera_transform, cursor_position) else {
        return;
    };

    for mut transform in &mut cursor_query {
        transform.translation.x = world_pos.x;
        transform.translation.y = world_pos.y;
    }
}

fn animate_balloon_system(
    time: Res<Time>,
    mut envelope_query: Query<(&BalloonEnvelope, &mut Transform), Without<BalloonBasket>>,
    mut basket_query: Query<&mut Transform, (With<BalloonBasket>, Without<BalloonEnvelope>)>,
) {
    let elapsed = time.elapsed_secs();

    for (envelope, mut transform) in &mut envelope_query {
        transform.rotation = Quat::from_rotation_z((elapsed * 1.5).sin() * 0.08);
        transform.translation.y = (elapsed * 1.2).sin().mul_add(3.0, envelope.base_y);
    }

    for mut transform in &mut basket_query {
        transform.rotation = Quat::from_rotation_z(((elapsed - 0.3) * 1.5).sin() * 0.06);
        transform.translation.y = ((elapsed - 0.3) * 1.2).sin().mul_add(3.0, BASKET_OFFSET_Y);
    }
}

fn spawn_air_waves_system(
    time: Res<Time>,
    mut commands: Commands,
    mut spawner_query: Query<(&mut AirWaveSpawner, &GlobalTransform)>,
) {
    let mut rng = rand::rng();

    for (mut spawner, global_transform) in &mut spawner_query {
        spawner.timer.tick(time.delta());

        if spawner.timer.just_finished() && !spawner.wave_textures.is_empty() {
            let texture_index = rng.random_range(0..spawner.wave_textures.len());
            let texture = spawner.wave_textures[texture_index].clone();

            let root_pos = global_transform.translation();
            let offset_y = rng.random_range(-15.0_f32..15.0);
            let spawn_pos = Vec3::new(root_pos.x - 55.0, root_pos.y + offset_y, -1.0);

            let drift_y = rng.random_range(-8.0_f32..8.0);

            commands.spawn((
                AirWave {
                    lifetime: Timer::from_seconds(5.0, TimerMode::Once),
                    velocity: Vec2::new(20.0, drift_y),
                    initial_opacity: 0.6,
                },
                Sprite {
                    image: texture,
                    color: Color::srgba(1.0, 1.0, 1.0, 0.6),
                    ..default()
                },
                Transform::from_translation(spawn_pos).with_scale(Vec3::splat(0.5)),
                RenderLayers::layer(CURSOR_RENDER_LAYER),
            ));
        }
    }
}

fn animate_air_waves_system(
    time: Res<Time>,
    mut commands: Commands,
    mut wave_query: Query<(Entity, &mut AirWave, &mut Transform, &mut Sprite)>,
) {
    let dt = time.delta_secs();
    let elapsed = time.elapsed_secs();

    for (entity, mut wave, mut transform, mut sprite) in &mut wave_query {
        wave.lifetime.tick(time.delta());

        // Vertical oscillation
        wave.velocity.y += (elapsed * 3.0).sin() * 0.5 * dt;

        // Move
        transform.translation.x += wave.velocity.x * dt;
        transform.translation.y += wave.velocity.y * dt;

        // Fade out
        let opacity = wave.initial_opacity * (1.0 - wave.lifetime.fraction());
        sprite.color = Color::srgba(1.0, 1.0, 1.0, opacity);

        if wave.lifetime.is_finished() {
            commands.entity(entity).despawn();
        }
    }
}
