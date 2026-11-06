use bevy::asset::RenderAssetUsages;
use bevy::camera::visibility::RenderLayers;
use bevy::camera::{ClearColorConfig, NormalizedRenderTarget};
use bevy::curve::Curve;
use bevy::curve::easing::{EaseFunction, EasingCurve};
use bevy::ecs::system::SystemParam;
use bevy::image::ImageSampler;
use bevy::input::touch::{TouchInput, TouchPhase};
use bevy::picking::hover::HoverMap;
use bevy::picking::pointer::{
    PointerAction, PointerButton, PointerId, PointerInput, PointerLocation,
};
use bevy::prelude::*;
use bevy::render::render_resource::{Extent3d, TextureDimension, TextureFormat};
use bevy::sprite::Anchor;
use bevy::window::{CursorMoved, CursorOptions, PrimaryWindow, WindowRef};
use rand::RngExt;

use crate::data::{ExplorationTheme, GameSettings};
use crate::plugins::sky_background::generate_cloud_image;
use crate::states::AppState;
use crate::ui::components::ButtonVisualStyle;
use crate::ui::theme;

/// Provides the Sky theme cursor for exploration and lesson screens.
pub struct SkyCursorPlugin;

impl Plugin for SkyCursorPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(OnEnter(AppState::ThemeExploration), setup_balloon_cursor)
            .add_systems(OnExit(AppState::ThemeExploration), cleanup_custom_cursor)
            .add_systems(OnEnter(AppState::LessonPlay), setup_lesson_cursor)
            .add_systems(OnExit(AppState::LessonPlay), cleanup_custom_cursor)
            .add_systems(
                Update,
                follow_pointer_system.run_if(resource_exists::<CustomCursorActive>),
            )
            .add_systems(
                Update,
                (
                    animate_balloon_system,
                    spawn_cloud_puffs_system,
                    animate_cloud_puffs_system,
                )
                    .run_if(in_state(AppState::ThemeExploration))
                    .run_if(resource_exists::<CustomCursorActive>),
            )
            .add_systems(
                Update,
                (animate_lesson_cursor_system, animate_eye_blink_system)
                    .run_if(in_state(AppState::LessonPlay))
                    .run_if(resource_exists::<CustomCursorActive>),
            );
    }
}

#[derive(Component, Reflect)]
#[require(Transform, Visibility)]
struct PointerCursor;

#[derive(Component, Reflect)]
struct BalloonCursor;

#[derive(Component, Reflect)]
struct LessonMagnifierCursor;

type ActiveCursorEntities<'w, 's> =
    Query<'w, 's, Entity, Or<(With<BalloonCursor>, With<LessonMagnifierCursor>)>>;

#[derive(Component, Reflect)]
#[require(Transform, Visibility)]
struct MagnifierVisual;

#[derive(Component, Reflect)]
struct MagnifierScaleAnimation {
    start_scale: f32,
    target_scale: f32,
    elapsed: f32,
    click_timer: Option<Timer>,
}

#[derive(Component, Reflect)]
struct MagnifierEye;

#[derive(Component, Reflect)]
struct EyeBlinkAnimation {
    timer: Timer,
    phase: EyeBlinkPhase,
    #[reflect(ignore)]
    textures: [Handle<Image>; 3],
}

#[derive(Reflect)]
enum EyeBlinkPhase {
    Waiting,
    Closing,
    Closed,
    Opening,
}

#[derive(Clone, Copy)]
enum EyeFrame {
    Open,
    Half,
    Closed,
}

#[derive(Component, Default, Reflect)]
struct TouchCursorState {
    touches: Vec<(u64, Vec2)>,
    active_touch: Option<u64>,
    last_touch: Option<u64>,
    last_position: Option<Vec2>,
}

impl TouchCursorState {
    fn update_touch(&mut self, phase: TouchPhase, id: u64, position: Vec2) {
        match phase {
            TouchPhase::Started => {
                if let Some((_, touch_position)) = self
                    .touches
                    .iter_mut()
                    .find(|(touch_id, _)| *touch_id == id)
                {
                    *touch_position = position;
                } else {
                    self.touches.push((id, position));
                }

                if self.active_touch.is_none() {
                    self.active_touch = Some(id);
                }
            }
            TouchPhase::Moved => {
                if let Some((_, touch_position)) = self
                    .touches
                    .iter_mut()
                    .find(|(touch_id, _)| *touch_id == id)
                {
                    *touch_position = position;
                }
            }
            TouchPhase::Ended | TouchPhase::Canceled => {
                let was_active = self.active_touch == Some(id);
                if was_active {
                    self.last_position = Some(position);
                }
                self.touches.retain(|(touch_id, _)| *touch_id != id);

                if was_active {
                    self.active_touch = self.touches.first().map(|(touch_id, _)| *touch_id);
                    self.last_touch = Some(id);
                }
            }
        }

        if let Some(active_touch) = self.active_touch {
            self.last_position = self
                .touches
                .iter()
                .find(|(touch_id, _)| *touch_id == active_touch)
                .map(|(_, position)| *position)
                .or(self.last_position);
            self.last_touch = Some(active_touch);
        }
    }

    fn displayed_pointer_id(&self) -> PointerId {
        self.active_touch
            .or(self.last_touch)
            .map_or(PointerId::Mouse, PointerId::Touch)
    }

    fn cursor_position(
        &mut self,
        mouse_moved: bool,
        window_position: Option<Vec2>,
    ) -> Option<Vec2> {
        if self.active_touch.is_some() {
            self.last_position
        } else if mouse_moved {
            self.last_position = None;
            self.last_touch = None;
            window_position
        } else {
            self.last_position.or(window_position)
        }
    }
}

#[derive(Component, Reflect)]
struct BalloonEnvelope {
    base_y: f32,
}

#[derive(Component, Reflect)]
struct BalloonBasket;

#[derive(Component, Reflect)]
struct CloudPuffSpawner {
    timer: Timer,
    #[reflect(ignore)]
    puff_textures: Vec<Handle<Image>>,
}

#[derive(Component, Reflect)]
struct CloudPuff {
    lifetime: Timer,
    velocity: Vec2,
    initial_opacity: f32,
}

/// Marker for the overlay camera that renders cursor sprites on top of UI.
#[derive(Component, Reflect)]
struct CursorCamera;

/// Marks an active custom cursor.
#[derive(Resource, Reflect)]
struct CustomCursorActive;

#[derive(SystemParam)]
struct ButtonHoverState<'w, 's> {
    hover_map: Res<'w, HoverMap>,
    pointer_query: Query<'w, 's, (&'static PointerId, &'static PointerLocation)>,
    parent_query: Query<'w, 's, &'static ChildOf>,
    button_query: Query<'w, 's, (), With<ButtonVisualStyle>>,
}

/// Render layer used exclusively for cursor sprites. The overlay camera
/// and all cursor entities share this layer so they don't interfere with
/// the main scene or UI.
const CURSOR_RENDER_LAYER: usize = 1;
const CURSOR_IMAGE_WIDTH: u32 = 96;
const CURSOR_IMAGE_HEIGHT: u32 = 96;
const CURSOR_IMAGE_SAMPLES: u32 = 4;
const MAGNIFIER_SCALE: f32 = 0.6;
const MAGNIFIER_SPRITE_OFFSET: Vec2 = Vec2::new(3.6, -3.6);
const EYE_BLINK_MIN_INTERVAL: f32 = 2.7;
const EYE_BLINK_MAX_INTERVAL: f32 = 4.8;
const EYE_BLINK_CLOSING_DURATION: f32 = 0.06;
const EYE_BLINK_CLOSED_DURATION: f32 = 0.055;
const EYE_BLINK_OPENING_DURATION: f32 = 0.07;
const MAGNIFIER_CLICK_SCALE: f32 = 1.12;
const MAGNIFIER_CLICK_DURATION: f32 = 0.24;

/// Scale factor for the envelope sprite.
const ENVELOPE_SCALE: f32 = 0.19;
/// Scale factor for the basket sprite.
const BASKET_SCALE: f32 = 0.25;
/// Positions the basket to overlap the envelope.
const BASKET_OFFSET_Y: f32 = 23.0;
/// Positions the envelope above the cursor origin.
const ENVELOPE_BASE_Y: f32 = 20.0;

/// A cloud shape: `(image_width, image_height, blobs)` where each blob
/// is `(center_x, center_y, radius)`.
type CloudShape = (u32, u32, Vec<(f32, f32, f32)>);

/// Small cloud puff shapes for the balloon trail particles.
fn cloud_puff_definitions() -> Vec<CloudShape> {
    vec![
        (
            30,
            30,
            vec![(15.0, 15.0, 14.0), (12.0, 18.0, 10.0), (20.0, 13.0, 11.0)],
        ),
        (
            45,
            22,
            vec![(12.0, 12.0, 10.0), (25.0, 11.0, 12.0), (38.0, 13.0, 9.0)],
        ),
        (
            35,
            28,
            vec![(17.0, 14.0, 14.0), (12.0, 16.0, 10.0), (24.0, 12.0, 11.0)],
        ),
    ]
}

#[allow(
    clippy::cast_possible_truncation,
    clippy::cast_precision_loss,
    clippy::cast_sign_loss
)]
fn generate_cursor_image(
    width: u32,
    height: u32,
    mut pixel_at: impl FnMut(f32, f32) -> [u8; 4],
) -> Image {
    let mut data = vec![0; (width * height * 4) as usize];
    let sample_count = CURSOR_IMAGE_SAMPLES * CURSOR_IMAGE_SAMPLES;

    for y in 0..height {
        for x in 0..width {
            let mut channels = [0_u32; 4];
            for sample_y in 0..CURSOR_IMAGE_SAMPLES {
                for sample_x in 0..CURSOR_IMAGE_SAMPLES {
                    let position_x =
                        x as f32 + (sample_x as f32 + 0.5) / CURSOR_IMAGE_SAMPLES as f32;
                    let position_y =
                        y as f32 + (sample_y as f32 + 0.5) / CURSOR_IMAGE_SAMPLES as f32;
                    let color = pixel_at(position_x, position_y);
                    for (channel, value) in channels.iter_mut().zip(color) {
                        *channel += u32::from(value);
                    }
                }
            }

            let pixel_index = ((y * width + x) * 4) as usize;
            for (channel_index, value) in channels.into_iter().enumerate() {
                data[pixel_index + channel_index] = (value / sample_count) as u8;
            }
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

fn generate_magnifier_image() -> Image {
    generate_cursor_image(CURSOR_IMAGE_WIDTH, CURSOR_IMAGE_HEIGHT, magnifier_pixel)
}

fn generate_eye_image(frame: EyeFrame) -> Image {
    generate_cursor_image(CURSOR_IMAGE_WIDTH, CURSOR_IMAGE_HEIGHT, |x, y| {
        eye_pixel(x, y, frame)
    })
}

fn magnifier_pixel(x: f32, y: f32) -> [u8; 4] {
    const OUTLINE: [u8; 4] = [107, 79, 18, 255];
    const SHADOW: [u8; 4] = [78, 55, 19, 190];
    const GOLD: [u8; 4] = [212, 160, 74, 255];
    const GOLD_LIGHT: [u8; 4] = [253, 216, 104, 255];
    const GOLD_DARK: [u8; 4] = [139, 102, 28, 255];
    const RUST: [u8; 4] = [196, 66, 31, 255];
    const GLASS: [u8; 4] = [166, 215, 238, 255];
    const GLASS_LIGHT: [u8; 4] = [225, 244, 250, 255];
    const GLASS_SHADE: [u8; 4] = [120, 184, 214, 255];

    let point = Vec2::new(x, y);
    let lens_center = Vec2::splat(42.0);
    let lens_distance = point.distance(lens_center);
    let handle_start = Vec2::new(58.0, 58.0);
    let handle_end = Vec2::new(83.0, 83.0);
    let handle_distance = distance_to_segment(point, handle_start, handle_end);

    if lens_distance > 30.5 {
        if handle_distance <= 8.0 {
            if handle_distance > 6.0 {
                return OUTLINE;
            }
            if distance_to_segment(
                point,
                handle_start + Vec2::new(-1.5, 1.5),
                handle_end + Vec2::new(-1.5, 1.5),
            ) <= 1.0
            {
                return GOLD_LIGHT;
            }
            if distance_to_segment(
                point,
                handle_start + Vec2::new(2.8, -2.8),
                handle_end + Vec2::new(2.8, -2.8),
            ) <= 1.1
            {
                return RUST;
            }
            return GOLD;
        }
        if point.distance(Vec2::new(44.0, 45.0)) <= 31.8 {
            return SHADOW;
        }
        return [0, 0, 0, 0];
    }

    if lens_distance > 28.5 {
        return OUTLINE;
    }
    if lens_distance > 26.5 {
        if inside_ellipse(x, y, 34.0, 28.0, 12.0, 3.5) {
            return GOLD_LIGHT;
        }
        if inside_ellipse(x, y, 50.0, 48.0, 9.0, 5.0) {
            return RUST;
        }
        return GOLD;
    }
    if lens_distance > 24.0 {
        return GOLD_DARK;
    }
    if lens_distance > 22.1 {
        if inside_ellipse(x, y, 34.0, 28.0, 10.0, 2.8) {
            return GOLD_LIGHT;
        }
        return GOLD;
    }
    if lens_distance > 21.3 {
        return OUTLINE;
    }
    if inside_ellipse(x, y, 34.0, 32.0, 10.0, 3.0) {
        return GLASS_LIGHT;
    }
    if inside_ellipse(x, y, 49.0, 51.0, 13.0, 8.0) {
        return GLASS_SHADE;
    }
    GLASS
}

fn eye_pixel(x: f32, y: f32, frame: EyeFrame) -> [u8; 4] {
    const OUTLINE: [u8; 4] = [107, 79, 18, 255];
    const SCLERA: [u8; 4] = [255, 248, 226, 255];
    const IRIS: [u8; 4] = [190, 115, 44, 255];
    const IRIS_LIGHT: [u8; 4] = [226, 157, 65, 255];
    const PUPIL: [u8; 4] = [61, 40, 20, 255];
    const GLINT: [u8; 4] = [255, 255, 242, 255];

    if matches!(frame, EyeFrame::Closed) {
        let dx = x - 42.0;
        let eyelid_y = (dx * dx).mul_add(0.014, 42.0);
        if dx.abs() <= 17.5 && (y - eyelid_y).abs() <= 1.25 {
            return OUTLINE;
        }
        if near_segment(x, y, Vec2::new(26.0, 44.0), Vec2::new(22.5, 47.0), 0.85)
            || near_segment(x, y, Vec2::new(58.0, 44.0), Vec2::new(61.5, 47.0), 0.85)
        {
            return OUTLINE;
        }
        return [0, 0, 0, 0];
    }

    let (eye_height, iris_height, pupil_height) = match frame {
        EyeFrame::Open => (12.5, 8.1, 5.4),
        EyeFrame::Half => (7.0, 4.6, 3.5),
        EyeFrame::Closed => unreachable!(),
    };
    let in_eye = inside_ellipse(x, y, 42.0, 42.0, 19.0, eye_height + 1.5);
    let in_sclera = inside_ellipse(x, y, 42.0, 42.0, 17.0, eye_height - 1.0);
    if in_eye && !in_sclera {
        return OUTLINE;
    }
    if in_sclera {
        if inside_ellipse(x, y, 42.0, 43.0, 7.4, iris_height + 1.2) {
            if inside_ellipse(x, y, 42.0, 43.0, 6.0, iris_height) {
                if inside_ellipse(x, y, 43.0, 43.2, 3.4, pupil_height) {
                    return PUPIL;
                }
                if inside_ellipse(x, y, 39.4, 39.7, 2.0, 2.4)
                    || inside_ellipse(x, y, 45.0, 45.0, 0.9, 1.0)
                {
                    return GLINT;
                }
                if inside_ellipse(x, y, 40.5, 40.5, 4.3, 4.2) {
                    return IRIS_LIGHT;
                }
                return IRIS;
            }
            return OUTLINE;
        }
        return SCLERA;
    }
    [0, 0, 0, 0]
}

fn inside_ellipse(x: f32, y: f32, center_x: f32, center_y: f32, rx: f32, ry: f32) -> bool {
    let normalized_x = (x - center_x) / rx;
    let normalized_y = (y - center_y) / ry;
    normalized_x.mul_add(normalized_x, normalized_y * normalized_y) <= 1.0
}

fn distance_to_segment(point: Vec2, start: Vec2, end: Vec2) -> f32 {
    let line = end - start;
    let fraction = ((point - start).dot(line) / line.length_squared()).clamp(0.0, 1.0);
    point.distance(start + line * fraction)
}

fn near_segment(x: f32, y: f32, start: Vec2, end: Vec2, width: f32) -> bool {
    distance_to_segment(Vec2::new(x, y), start, end) <= width
}

fn setup_balloon_cursor(
    mut commands: Commands,
    asset_server: Res<AssetServer>,
    mut images: ResMut<Assets<Image>>,
    settings: Res<GameSettings>,
    mut cursor_opts: Single<&mut CursorOptions, With<PrimaryWindow>>,
) {
    // Only activate for the Sky theme; other themes will have their own plugin.
    if settings.exploration_theme != ExplorationTheme::Sky {
        return;
    }

    commands.insert_resource(CustomCursorActive);

    // Hide the OS cursor
    cursor_opts.visible = false;

    let envelope_texture: Handle<Image> = asset_server.load("cursor/balloon_envelope.png");
    let basket_texture: Handle<Image> = asset_server.load("cursor/balloon_basket.png");

    // Generate cloud puff textures procedurally (no PNG assets needed)
    let puff_textures: Vec<Handle<Image>> = cloud_puff_definitions()
        .iter()
        .map(|(w, h, blobs)| images.add(generate_cloud_image(*w, *h, blobs)))
        .collect();

    let cursor_layer = RenderLayers::layer(CURSOR_RENDER_LAYER);
    spawn_cursor_camera(&mut commands, &cursor_layer);

    // Spawn the cursor hierarchy (all entities on the cursor render layer)
    commands.spawn((
        PointerCursor,
        BalloonCursor,
        TouchCursorState::default(),
        cursor_layer.clone(),
        CloudPuffSpawner {
            timer: Timer::from_seconds(1.6, TimerMode::Repeating),
            puff_textures,
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

fn cleanup_custom_cursor(
    mut commands: Commands,
    active: Option<Res<CustomCursorActive>>,
    cursor_query: ActiveCursorEntities<'_, '_>,
    puff_query: Query<Entity, With<CloudPuff>>,
    camera_query: Query<Entity, With<CursorCamera>>,
    mut cursor_opts: Single<&mut CursorOptions, With<PrimaryWindow>>,
) {
    if active.is_none() {
        return;
    }

    cursor_opts.visible = true;

    for entity in &cursor_query {
        commands.entity(entity).despawn();
    }
    for entity in &puff_query {
        commands.entity(entity).despawn();
    }
    for entity in &camera_query {
        commands.entity(entity).despawn();
    }

    commands.remove_resource::<CustomCursorActive>();
}

fn spawn_cursor_camera(commands: &mut Commands, cursor_layer: &RenderLayers) {
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
}

fn setup_lesson_cursor(
    mut commands: Commands,
    mut images: ResMut<Assets<Image>>,
    settings: Res<GameSettings>,
    mut cursor_opts: Single<&mut CursorOptions, With<PrimaryWindow>>,
) {
    if settings.exploration_theme != ExplorationTheme::Sky {
        return;
    }

    commands.insert_resource(CustomCursorActive);
    cursor_opts.visible = false;

    let cursor_layer = RenderLayers::layer(CURSOR_RENDER_LAYER);
    let magnifier_texture = images.add(generate_magnifier_image());
    let eye_textures = [
        images.add(generate_eye_image(EyeFrame::Open)),
        images.add(generate_eye_image(EyeFrame::Half)),
        images.add(generate_eye_image(EyeFrame::Closed)),
    ];
    let mut rng = rand::rng();
    let blink_interval = rng.random_range(EYE_BLINK_MIN_INTERVAL..EYE_BLINK_MAX_INTERVAL);
    spawn_cursor_camera(&mut commands, &cursor_layer);
    commands.spawn((
        PointerCursor,
        LessonMagnifierCursor,
        TouchCursorState::default(),
        cursor_layer.clone(),
        children![(
            MagnifierVisual,
            MagnifierScaleAnimation {
                start_scale: 1.0,
                target_scale: 1.0,
                elapsed: theme::animation::BUTTON_HOVER_DURATION,
                click_timer: None,
            },
            cursor_layer.clone(),
            children![
                (
                    Sprite::from_image(magnifier_texture),
                    Anchor::CENTER,
                    Transform::from_xyz(MAGNIFIER_SPRITE_OFFSET.x, MAGNIFIER_SPRITE_OFFSET.y, 1.0,)
                        .with_scale(Vec3::splat(MAGNIFIER_SCALE)),
                    Pickable::IGNORE,
                    cursor_layer.clone(),
                ),
                (
                    MagnifierEye,
                    Sprite::from_image(eye_textures[0].clone()),
                    Anchor::CENTER,
                    Transform::from_xyz(MAGNIFIER_SPRITE_OFFSET.x, MAGNIFIER_SPRITE_OFFSET.y, 2.0,)
                        .with_scale(Vec3::splat(MAGNIFIER_SCALE)),
                    EyeBlinkAnimation {
                        timer: Timer::from_seconds(blink_interval, TimerMode::Once),
                        phase: EyeBlinkPhase::Waiting,
                        textures: eye_textures,
                    },
                    Pickable::IGNORE,
                    cursor_layer,
                ),
            ],
        )],
    ));
}

fn follow_pointer_system(
    primary_window: Single<(Entity, &Window), With<PrimaryWindow>>,
    camera_data: Single<(&Camera, &GlobalTransform), With<CursorCamera>>,
    cursor_query: Single<
        (&mut Transform, &mut TouchCursorState, &mut Visibility),
        With<PointerCursor>,
    >,
    mut click_query: Query<&mut MagnifierScaleAnimation, With<MagnifierVisual>>,
    mut cursor_moved_events: MessageReader<CursorMoved>,
    mut touch_events: MessageReader<TouchInput>,
    mut pointer_inputs: MessageReader<PointerInput>,
) {
    let (window_entity, window) = *primary_window;
    let (mut transform, mut touch_state, mut visibility) = cursor_query.into_inner();

    let mut mouse_moved = false;
    for event in cursor_moved_events.read() {
        if event.window == window_entity {
            mouse_moved = true;
        }
    }

    for touch in touch_events.read() {
        if touch.window == window_entity {
            touch_state.update_touch(touch.phase, touch.id, touch.position);
        }
    }

    let cursor_position = touch_state.cursor_position(mouse_moved, window.cursor_position());
    let displayed_pointer = touch_state.displayed_pointer_id();
    let primary_target = WindowRef::Primary
        .normalize(Some(window_entity))
        .map(NormalizedRenderTarget::Window);
    for input in pointer_inputs.read() {
        let is_primary_click = matches!(input.action, PointerAction::Press(PointerButton::Primary));
        if input.pointer_id == displayed_pointer
            && is_primary_click
            && Some(input.location.target.clone()) == primary_target
        {
            for mut animation in &mut click_query {
                animation.click_timer = Some(Timer::from_seconds(
                    MAGNIFIER_CLICK_DURATION,
                    TimerMode::Once,
                ));
            }
        }
    }

    if !window.focused {
        *visibility = Visibility::Hidden;
        return;
    }

    let Some(cursor_position) = cursor_position else {
        *visibility = Visibility::Hidden;
        return;
    };
    *visibility = Visibility::Visible;

    let (camera, camera_transform) = *camera_data;
    let Ok(world_pos) = camera.viewport_to_world_2d(camera_transform, cursor_position) else {
        return;
    };

    transform.translation.x = world_pos.x;
    transform.translation.y = world_pos.y;
}

fn animate_lesson_cursor_system(
    time: Res<Time>,
    primary_window: Single<Entity, With<PrimaryWindow>>,
    displayed_pointer: Single<&TouchCursorState, With<PointerCursor>>,
    button_hover: ButtonHoverState,
    mut cursor_query: Query<(&mut Transform, &mut MagnifierScaleAnimation), With<MagnifierVisual>>,
) {
    let primary_target = WindowRef::Primary
        .normalize(Some(*primary_window))
        .map(NormalizedRenderTarget::Window);
    let hovered = cursor_is_over_button(
        displayed_pointer.displayed_pointer_id(),
        primary_target,
        &button_hover,
    );
    let target_scale = if hovered {
        theme::animation::BUTTON_HOVER_SCALE
    } else {
        1.0
    };

    for (mut transform, mut animation) in &mut cursor_query {
        let target_changed = (animation.target_scale - target_scale).abs() > f32::EPSILON;
        if target_changed {
            animation.start_scale = transform.scale.x;
            animation.target_scale = target_scale;
            animation.elapsed = 0.0;
        } else {
            animation.elapsed = (animation.elapsed + time.delta_secs())
                .min(theme::animation::BUTTON_HOVER_DURATION);
        }

        let hover_progress = animation.elapsed / theme::animation::BUTTON_HOVER_DURATION;
        let hover_curve = EasingCurve::new(
            animation.start_scale,
            animation.target_scale,
            EaseFunction::CubicOut,
        );
        let hover_scale = hover_curve.sample_clamped(hover_progress);
        let click_growth = animation.click_timer.as_mut().map_or(0.0, |timer| {
            timer.tick(time.delta());
            if timer.is_finished() {
                0.0
            } else {
                let progress = timer.fraction();
                (MAGNIFIER_CLICK_SCALE - hover_scale) * (progress * std::f32::consts::PI).sin()
            }
        });
        if animation
            .click_timer
            .as_ref()
            .is_some_and(Timer::is_finished)
        {
            animation.click_timer = None;
        }
        transform.scale = Vec3::splat(hover_scale + click_growth);
    }
}

fn cursor_is_over_button(
    displayed_pointer: PointerId,
    primary_target: Option<NormalizedRenderTarget>,
    hover_state: &ButtonHoverState,
) -> bool {
    let Some(primary_target) = primary_target else {
        return false;
    };

    let Some((_, pointer_location)) = hover_state
        .pointer_query
        .iter()
        .find(|(pointer_id, _)| **pointer_id == displayed_pointer)
    else {
        return false;
    };
    let is_over_primary = pointer_location
        .location()
        .is_some_and(|location| location.target == primary_target);
    if !is_over_primary {
        return false;
    }

    let Some(entities) = hover_state.hover_map.get(&displayed_pointer) else {
        return false;
    };
    for entity in entities.keys() {
        for candidate in
            std::iter::once(*entity).chain(hover_state.parent_query.iter_ancestors(*entity))
        {
            if hover_state.button_query.contains(candidate) {
                return true;
            }
        }
    }

    false
}

fn animate_eye_blink_system(
    time: Res<Time>,
    mut eye_query: Query<(&mut EyeBlinkAnimation, &mut Sprite), With<MagnifierEye>>,
) {
    let mut rng = rand::rng();
    for (mut blink, mut sprite) in &mut eye_query {
        blink.timer.tick(time.delta());
        if !blink.timer.is_finished() {
            continue;
        }

        let (phase, frame, duration) = match blink.phase {
            EyeBlinkPhase::Waiting => (EyeBlinkPhase::Closing, 1, EYE_BLINK_CLOSING_DURATION),
            EyeBlinkPhase::Closing => (EyeBlinkPhase::Closed, 2, EYE_BLINK_CLOSED_DURATION),
            EyeBlinkPhase::Closed => (EyeBlinkPhase::Opening, 1, EYE_BLINK_OPENING_DURATION),
            EyeBlinkPhase::Opening => (
                EyeBlinkPhase::Waiting,
                0,
                rng.random_range(EYE_BLINK_MIN_INTERVAL..EYE_BLINK_MAX_INTERVAL),
            ),
        };
        blink.phase = phase;
        blink.timer = Timer::from_seconds(duration, TimerMode::Once);
        sprite.image = blink.textures[frame].clone();
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

fn spawn_cloud_puffs_system(
    time: Res<Time>,
    mut commands: Commands,
    mut spawner_query: Query<(&mut CloudPuffSpawner, &GlobalTransform)>,
) {
    let mut rng = rand::rng();

    for (mut spawner, global_transform) in &mut spawner_query {
        spawner.timer.tick(time.delta());

        if spawner.timer.just_finished() && !spawner.puff_textures.is_empty() {
            let texture_index = rng.random_range(0..spawner.puff_textures.len());
            let texture = spawner.puff_textures[texture_index].clone();

            let root_pos = global_transform.translation();
            let offset_y = rng.random_range(-15.0_f32..15.0);
            let spawn_pos = Vec3::new(root_pos.x - 55.0, root_pos.y + offset_y, -1.0);

            let drift_y = rng.random_range(-8.0_f32..8.0);

            commands.spawn((
                CloudPuff {
                    lifetime: Timer::from_seconds(5.0, TimerMode::Once),
                    velocity: Vec2::new(20.0, drift_y),
                    initial_opacity: 0.35,
                },
                Sprite {
                    image: texture,
                    color: Color::srgba(1.0, 1.0, 1.0, 0.35),
                    ..default()
                },
                Transform::from_translation(spawn_pos).with_scale(Vec3::splat(0.5)),
                RenderLayers::layer(CURSOR_RENDER_LAYER),
            ));
        }
    }
}

fn animate_cloud_puffs_system(
    time: Res<Time>,
    mut commands: Commands,
    mut puff_query: Query<(Entity, &mut CloudPuff, &mut Transform, &mut Sprite)>,
) {
    let dt = time.delta_secs();
    let elapsed = time.elapsed_secs();

    for (entity, mut puff, mut transform, mut sprite) in &mut puff_query {
        puff.lifetime.tick(time.delta());

        // Vertical oscillation
        puff.velocity.y = ((elapsed * 3.0).sin() * 0.5).mul_add(dt, puff.velocity.y);

        transform.translation.x = puff.velocity.x.mul_add(dt, transform.translation.x);
        transform.translation.y = puff.velocity.y.mul_add(dt, transform.translation.y);

        let opacity = puff.initial_opacity * (1.0 - puff.lifetime.fraction());
        sprite.color = Color::srgba(1.0, 1.0, 1.0, opacity);

        if puff.lifetime.is_finished() {
            commands.entity(entity).despawn();
        }
    }
}
