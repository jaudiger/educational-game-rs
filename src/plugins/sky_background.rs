//! Animated sky background for the `MapTheme::Sky` visual theme.
//!
//! Spawns a gradient sky and multiple layers of drifting clouds during
//! `AppState::MapExploration` when the sky theme is selected. Clouds are
//! procedurally generated with no external assets required.

use bevy::asset::RenderAssetUsages;
use bevy::image::ImageSampler;
use bevy::prelude::*;
use bevy::render::render_resource::{Extent3d, TextureDimension, TextureFormat};
use bevy::window::PrimaryWindow;
use bevy_persistent::prelude::Persistent;
use rand::RngExt;

use crate::data::{GameSettings, MapTheme};
use crate::states::AppState;

/// Renders an animated sky background with parallax clouds.
///
/// Active only during `AppState::MapExploration` when
/// `GameSettings.map_theme == MapTheme::Sky`.
///
/// # Extensibility
///
/// Each `MapTheme` variant gets its own background plugin. Adding a new
/// background theme means creating a new plugin file with the same pattern:
/// `OnEnter`/`OnExit` for setup/cleanup, `Update` systems guarded by
/// `in_state(MapExploration)` and a theme-specific resource run condition.
pub struct SkyBackgroundPlugin;

impl Plugin for SkyBackgroundPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(OnEnter(AppState::MapExploration), setup_sky_background)
            .add_systems(OnExit(AppState::MapExploration), cleanup_sky_background)
            .add_systems(
                Update,
                (resize_gradient_to_viewport, animate_clouds)
                    .run_if(in_state(AppState::MapExploration))
                    .run_if(resource_exists::<SkyBackgroundActive>),
            );
    }
}

/// Marks the full-screen gradient sprite.
#[derive(Component, Reflect)]
struct SkyGradient;

/// Marks a drifting cloud sprite.
#[derive(Component, Reflect)]
struct SkyCloud {
    /// Horizontal speed in pixels per second.
    speed: f32,
    /// Half the visual width (used for viewport wrapping).
    half_width: f32,
}

/// Inserted when the sky background is active. Used as a run condition so
/// update systems don't run when another map theme is selected.
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

/// Generates a procedural cloud image from overlapping soft blobs.
///
/// Each blob contributes a smooth quartic falloff. Overlapping blobs
/// accumulate, producing natural-looking puffy edges. The result is a
/// white texture with varying alpha.
#[allow(
    clippy::cast_possible_truncation,
    clippy::cast_sign_loss,
    clippy::cast_precision_loss
)]
pub fn generate_cloud_image(width: u32, height: u32, blobs: &[(f32, f32, f32)]) -> Image {
    let pixel_count = (width * height * 4) as usize;
    let mut data = vec![0u8; pixel_count];

    for y in 0..height {
        for x in 0..width {
            let px = x as f32;
            let py = y as f32;

            let mut alpha = 0.0_f32;
            for &(cx, cy, radius) in blobs {
                let dx = px - cx;
                let dy = py - cy;
                let dist_sq = dx.mul_add(dx, dy * dy);
                let r_sq = radius * radius;
                if dist_sq < r_sq {
                    let t = 1.0 - dist_sq / r_sq;
                    alpha += t * t; // quartic falloff
                }
            }
            alpha = alpha.min(1.0);

            let idx = ((y * width + x) * 4) as usize;
            data[idx] = 255;
            data[idx + 1] = 255;
            data[idx + 2] = 255;
            data[idx + 3] = (alpha * 230.0) as u8;
        }
    }

    let mut image = Image::new(
        Extent3d {
            width,
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

/// A blob definition: `(center_x, center_y, radius)` in pixel coordinates.
type Blob = (f32, f32, f32);

/// A cloud shape: `(image_width, image_height, blobs)`.
type CloudShape = (u32, u32, Vec<Blob>);

/// Returns cloud shape definitions per variant.
fn cloud_definitions() -> Vec<CloudShape> {
    vec![
        // Cloud 0: wide cumulus
        (
            220,
            90,
            vec![
                (55.0, 55.0, 40.0),
                (110.0, 48.0, 50.0),
                (165.0, 52.0, 42.0),
                (85.0, 35.0, 32.0),
                (140.0, 33.0, 35.0),
                (40.0, 62.0, 28.0),
                (180.0, 58.0, 28.0),
            ],
        ),
        // Cloud 1: tall puffy
        (
            150,
            100,
            vec![
                (75.0, 64.0, 44.0),
                (48.0, 58.0, 36.0),
                (102.0, 60.0, 38.0),
                (75.0, 38.0, 36.0),
                (55.0, 42.0, 28.0),
                (95.0, 40.0, 30.0),
            ],
        ),
        // Cloud 2: small wisp
        (
            100,
            55,
            vec![(35.0, 32.0, 26.0), (62.0, 28.0, 28.0), (50.0, 36.0, 22.0)],
        ),
        // Cloud 3: elongated streak
        (
            200,
            70,
            vec![
                (40.0, 40.0, 28.0),
                (80.0, 38.0, 34.0),
                (120.0, 40.0, 32.0),
                (160.0, 42.0, 28.0),
                (60.0, 30.0, 22.0),
                (100.0, 28.0, 26.0),
                (140.0, 31.0, 24.0),
            ],
        ),
    ]
}

fn setup_sky_background(
    mut commands: Commands,
    mut images: ResMut<Assets<Image>>,
    settings: Res<Persistent<GameSettings>>,
    window: Single<&Window, With<PrimaryWindow>>,
) {
    if settings.map_theme != MapTheme::Sky {
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
        DespawnOnExit(AppState::MapExploration),
    ));

    // --- Cloud textures (one per shape variant) ---
    let definitions = cloud_definitions();
    let cloud_textures: Vec<(Handle<Image>, u32)> = definitions
        .iter()
        .map(|(w, h, blobs)| {
            let handle = images.add(generate_cloud_image(*w, *h, blobs));
            (handle, *w)
        })
        .collect();

    // --- Spawn cloud sprites across parallax layers ---
    let mut rng = rand::rng();
    let half_w = win_w / 2.0;
    let half_h = win_h / 2.0;

    for layer in CLOUD_LAYERS {
        for _ in 0..layer.count {
            let tex_idx = rng.random_range(0..cloud_textures.len());
            let (ref handle, img_w) = cloud_textures[tex_idx];

            let scale = rng.random_range(layer.scale_min..layer.scale_max);
            #[allow(clippy::cast_precision_loss)]
            let cloud_half_w = img_w as f32 * scale / 2.0;

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
                Sprite {
                    image: handle.clone(),
                    color: Color::srgba(1.0, 1.0, 1.0, layer.opacity),
                    ..default()
                },
                Transform::from_xyz(x, y, layer.z).with_scale(Vec3::splat(scale)),
                DespawnOnExit(AppState::MapExploration),
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
        transform.translation.x += cloud.speed * dt;

        // When the cloud fully exits the right edge, teleport to left.
        let wrap_margin = half_w + cloud.half_width + 50.0;
        if transform.translation.x > wrap_margin {
            transform.translation.x = -wrap_margin;
        }
    }
}
