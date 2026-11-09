//! Animated sky background for the `ExplorationTheme::Sky` visual theme.
//!
//! Spawns a gradient sky and multiple layers of drifting clouds during
//! `AppState::ThemeExploration` when the sky theme is selected. Clouds are
//! rendered from WGSL-authored cloud shapes.

use bevy::asset::RenderAssetUsages;
use bevy::image::ImageSampler;
use bevy::prelude::*;
use bevy::render::render_resource::{Extent3d, TextureDimension, TextureFormat};
use bevy::sprite_render::MeshMaterial2d;
use bevy::window::PrimaryWindow;
use rand::RngExt;

use crate::data::{ExplorationTheme, GameSettings};
use crate::plugins::cloud_material::{CloudShapeId, ProceduralCloudMaterial};
use crate::states::AppState;

/// Renders an animated sky background with parallax clouds.
///
/// Active only during `AppState::ThemeExploration` when
/// `GameSettings.exploration_theme == ExplorationTheme::Sky`.
///
/// # Extensibility
///
/// Each `ExplorationTheme` variant gets its own background plugin. Adding a new
/// background theme means creating a new plugin file with the same pattern:
/// `OnEnter`/`OnExit` for setup/cleanup, `Update` systems guarded by
/// `in_state(ThemeExploration)` and a theme-specific resource run condition.
pub struct SkyBackgroundPlugin;

impl Plugin for SkyBackgroundPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(OnEnter(AppState::ThemeExploration), setup_sky_background)
            .add_systems(OnExit(AppState::ThemeExploration), cleanup_sky_background)
            .add_systems(
                Update,
                (resize_gradient_to_viewport, animate_clouds)
                    .run_if(in_state(AppState::ThemeExploration))
                    .run_if(resource_exists::<SkyBackgroundActive>),
            );
    }
}

/// Marks the full-screen gradient sprite.
#[derive(Component, Reflect)]
struct SkyGradient;

/// Marks a drifting cloud mesh.
#[derive(Component, Reflect)]
struct SkyCloud {
    /// Horizontal speed in pixels per second.
    speed: f32,
    /// Half the visual width (used for viewport wrapping).
    half_width: f32,
}

/// Inserted when the sky background is active. Used as a run condition so
/// update systems don't run when another exploration theme is selected.
#[derive(Resource, Reflect)]
struct SkyBackgroundActive;

/// Sky gradient top colour (light sky blue).
const SKY_TOP: (f32, f32, f32) = (0.53, 0.81, 0.92);
/// Sky gradient bottom colour (very light blue, almost white).
const SKY_BOTTOM: (f32, f32, f32) = (0.93, 0.95, 1.0);

/// Height of the procedural gradient image in pixels.
const GRADIENT_HEIGHT: u32 = 512;

/// Configuration for one parallax cloud layer.
struct CloudLayerConfig {
    /// Horizontal drift speed (px/s).
    speed: f32,
    /// Minimum Y position as a fraction of half the viewport height
    /// (-1.0 = bottom edge, 1.0 = top edge).
    y_frac_min: f32,
    /// Maximum Y position (same convention).
    y_frac_max: f32,
    /// Minimum sprite scale.
    scale_min: f32,
    /// Maximum sprite scale.
    scale_max: f32,
    /// Cloud opacity (0.0 to 1.0).
    opacity: f32,
    /// Number of clouds in this layer.
    count: usize,
    /// Z coordinate (controls draw order; lower = further back).
    z: f32,
}

/// Three parallax layers: far (slow, faint), mid, and near (fast, opaque).
const CLOUD_LAYERS: &[CloudLayerConfig] = &[
    CloudLayerConfig {
        speed: 8.0,
        y_frac_min: 0.1,
        y_frac_max: 0.85,
        scale_min: 0.4,
        scale_max: 0.7,
        opacity: 0.25,
        count: 5,
        z: -90.0,
    },
    CloudLayerConfig {
        speed: 15.0,
        y_frac_min: -0.3,
        y_frac_max: 0.6,
        scale_min: 0.7,
        scale_max: 1.2,
        opacity: 0.4,
        count: 4,
        z: -80.0,
    },
    CloudLayerConfig {
        speed: 25.0,
        y_frac_min: -0.7,
        y_frac_max: 0.15,
        scale_min: 1.0,
        scale_max: 1.6,
        opacity: 0.55,
        count: 3,
        z: -70.0,
    },
];

/// Creates a vertical gradient image (2 px wide, [`GRADIENT_HEIGHT`] tall).
///
/// Stretched to fill the viewport at runtime via `Transform` scale.
fn generate_gradient_image() -> Image {
    const WIDTH: u32 = 2;
    let height = GRADIENT_HEIGHT;

    let pixel_count = (WIDTH * height * 4) as usize;
    let mut data = vec![0u8; pixel_count];

    for y in 0..height {
        #[allow(clippy::cast_precision_loss)]
        let t = y as f32 / (height - 1) as f32;

        let r = (SKY_BOTTOM.0 - SKY_TOP.0).mul_add(t, SKY_TOP.0);
        let g = (SKY_BOTTOM.1 - SKY_TOP.1).mul_add(t, SKY_TOP.1);
        let b = (SKY_BOTTOM.2 - SKY_TOP.2).mul_add(t, SKY_TOP.2);

        for x in 0..WIDTH {
            let idx = ((y * WIDTH + x) * 4) as usize;
            #[allow(clippy::cast_possible_truncation, clippy::cast_sign_loss)]
            {
                data[idx] = (r * 255.0) as u8;
                data[idx + 1] = (g * 255.0) as u8;
                data[idx + 2] = (b * 255.0) as u8;
                data[idx + 3] = 255;
            }
        }
    }

    let mut image = Image::new(
        Extent3d {
            width: WIDTH,
            height,
            depth_or_array_layers: 1,
        },
        TextureDimension::D2,
        data,
        TextureFormat::Rgba8UnormSrgb,
        RenderAssetUsages::RENDER_WORLD,
    );
    image.sampler = ImageSampler::linear();
    image
}

fn setup_sky_background(
    mut commands: Commands,
    mut images: ResMut<Assets<Image>>,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<ProceduralCloudMaterial>>,
    settings: Res<GameSettings>,
    window: Single<&Window, With<PrimaryWindow>>,
) {
    if settings.exploration_theme != ExplorationTheme::Sky {
        return;
    }

    commands.insert_resource(SkyBackgroundActive);

    let win_w = window.width();
    let win_h = window.height();

    // --- Gradient background ---
    #[allow(clippy::cast_precision_loss)]
    let grad_h = GRADIENT_HEIGHT as f32;

    let gradient_handle = images.add(generate_gradient_image());
    commands.spawn((
        SkyGradient,
        Sprite::from_image(gradient_handle),
        Transform::from_xyz(0.0, 0.0, -100.0).with_scale(Vec3::new(
            win_w / 2.0,
            win_h / grad_h,
            1.0,
        )),
        DespawnOnExit(AppState::ThemeExploration),
    ));

    let cloud_meshes = CloudShapeId::ALL.map(|shape_id| {
        let canvas = shape_id.canvas();
        meshes.add(Rectangle::new(canvas.x, canvas.y))
    });

    // --- Spawn cloud sprites across parallax layers ---
    let mut rng = rand::rng();
    let half_w = win_w / 2.0;
    let half_h = win_h / 2.0;

    for layer in CLOUD_LAYERS {
        for _ in 0..layer.count {
            let shape_index = rng.random_range(0..CloudShapeId::ALL.len());
            let shape_id = CloudShapeId::ALL[shape_index];

            let scale = rng.random_range(layer.scale_min..layer.scale_max);
            let canvas = shape_id.canvas();
            let cloud_half_w = canvas.x * scale / 2.0;

            let spawn_margin = half_w + cloud_half_w;
            let x = rng.random_range(-spawn_margin..spawn_margin);
            let y_min = half_h * layer.y_frac_min;
            let y_max = half_h * layer.y_frac_max;
            let y = rng.random_range(y_min..y_max);

            commands.spawn((
                SkyCloud {
                    speed: layer.speed,
                    half_width: cloud_half_w,
                },
                Mesh2d(cloud_meshes[shape_index].clone()),
                MeshMaterial2d(materials.add(ProceduralCloudMaterial::new(
                    shape_id,
                    canvas,
                    Color::srgba(1.0, 1.0, 1.0, layer.opacity),
                ))),
                Transform::from_xyz(x, y, layer.z).with_scale(Vec3::splat(scale)),
                DespawnOnExit(AppState::ThemeExploration),
            ));
        }
    }
}

fn cleanup_sky_background(mut commands: Commands, active: Option<Res<SkyBackgroundActive>>) {
    if active.is_some() {
        commands.remove_resource::<SkyBackgroundActive>();
    }
}

/// Keeps the gradient sprite scaled to fill the viewport on window resize.
fn resize_gradient_to_viewport(
    window: Single<&Window, With<PrimaryWindow>>,
    mut gradient: Query<&mut Transform, (With<SkyGradient>, Without<SkyCloud>)>,
) {
    #[allow(clippy::cast_precision_loss)]
    let grad_h = GRADIENT_HEIGHT as f32;

    let win_w = window.width();
    let win_h = window.height();

    for mut transform in &mut gradient {
        transform.scale.x = win_w / 2.0;
        transform.scale.y = win_h / grad_h;
    }
}

/// Drifts clouds horizontally and wraps them when they exit the viewport.
fn animate_clouds(
    time: Res<Time>,
    window: Single<&Window, With<PrimaryWindow>>,
    mut clouds: Query<(&SkyCloud, &mut Transform)>,
) {
    let dt = time.delta_secs();
    let half_w = window.width() / 2.0;

    for (cloud, mut transform) in &mut clouds {
        transform.translation.x = cloud.speed.mul_add(dt, transform.translation.x);

        // When the cloud fully exits the right edge, teleport to left.
        let wrap_margin = half_w + cloud.half_width + 50.0;
        if transform.translation.x > wrap_margin {
            transform.translation.x = -wrap_margin;
        }
    }
}
