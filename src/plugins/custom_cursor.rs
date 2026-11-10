use bevy::camera::visibility::RenderLayers;
use bevy::camera::{ClearColorConfig, NormalizedRenderTarget};
use bevy::curve::Curve;
use bevy::curve::easing::{EaseFunction, EasingCurve};
use bevy::ecs::system::SystemParam;
use bevy::input::touch::{TouchInput, TouchPhase};
use bevy::picking::hover::HoverMap;
use bevy::picking::pointer::{
    PointerAction, PointerButton, PointerId, PointerInput, PointerLocation,
};
use bevy::prelude::*;
use bevy::sprite::Anchor;
use bevy::sprite_render::MeshMaterial2d;
use bevy::window::{CursorMoved, CursorOptions, PrimaryWindow, WindowRef};
use rand::RngExt;

use crate::data::{ExplorationTheme, GameSettings};
use crate::plugins::cloud_material::{CloudShapeId, ProceduralCloudMaterial};
use crate::plugins::cursor_material::{CursorEyeFrame, CursorMaterial};
use crate::states::AppState;
use crate::ui::components::ButtonVisualStyle;
use crate::ui::theme;

/// Provides custom cursors for exploration and lesson screens.
pub struct CustomCursorPlugin;

impl Plugin for CustomCursorPlugin {
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
struct EyeBlinkAnimation {
    timer: Timer,
    phase: EyeBlinkPhase,
}

#[derive(Reflect)]
enum EyeBlinkPhase {
    Waiting,
    Closing,
    Closed,
    Opening,
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

/// Marker for the overlay camera that renders cursor visuals above the UI.
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

/// Render layer used exclusively for custom cursor visuals.
const CURSOR_RENDER_LAYER: usize = 1;
const CURSOR_VISUAL_SIZE: f32 = 96.0;
const MAGNIFIER_SCALE: f32 = 0.6;
const MAGNIFIER_VISUAL_OFFSET: Vec2 = Vec2::new(3.6, -3.6);
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

fn setup_balloon_cursor(
    mut commands: Commands,
    asset_server: Res<AssetServer>,
    mut meshes: ResMut<Assets<Mesh>>,
    settings: Res<GameSettings>,
    mut cursor_opts: Single<&mut CursorOptions, With<PrimaryWindow>>,
) {
    // The balloon cursor is specific to the Sky exploration theme.
    if settings.exploration_theme != ExplorationTheme::Sky {
        return;
    }

    commands.insert_resource(CustomCursorActive);

    // Hide the OS cursor
    cursor_opts.visible = false;

    let envelope_texture: Handle<Image> = asset_server.load("cursor/balloon_envelope.png");
    let basket_texture: Handle<Image> = asset_server.load("cursor/balloon_basket.png");

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

    // Spawn the cursor hierarchy (all entities on the cursor render layer)
    commands.spawn((
        PointerCursor,
        TouchCursorState::default(),
        cursor_layer.clone(),
        CloudPuffSpawner {
            timer: Timer::from_seconds(1.6, TimerMode::Repeating),
            puff_shapes,
            puff_meshes,
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
    cursor_query: Query<Entity, With<PointerCursor>>,
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
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<CursorMaterial>>,
    mut cursor_opts: Single<&mut CursorOptions, With<PrimaryWindow>>,
) {
    commands.insert_resource(CustomCursorActive);
    cursor_opts.visible = false;

    let cursor_layer = RenderLayers::layer(CURSOR_RENDER_LAYER);
    let cursor_mesh = meshes.add(Rectangle::new(CURSOR_VISUAL_SIZE, CURSOR_VISUAL_SIZE));
    let cursor_material = materials.add(CursorMaterial::default());
    let mut rng = rand::rng();
    let blink_interval = rng.random_range(EYE_BLINK_MIN_INTERVAL..EYE_BLINK_MAX_INTERVAL);
    spawn_cursor_camera(&mut commands, &cursor_layer);
    commands.spawn((
        PointerCursor,
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
            children![(
                Mesh2d(cursor_mesh),
                MeshMaterial2d(cursor_material),
                Transform::from_xyz(MAGNIFIER_VISUAL_OFFSET.x, MAGNIFIER_VISUAL_OFFSET.y, 1.0,)
                    .with_scale(Vec3::splat(MAGNIFIER_SCALE)),
                EyeBlinkAnimation {
                    timer: Timer::from_seconds(blink_interval, TimerMode::Once),
                    phase: EyeBlinkPhase::Waiting,
                },
                Pickable::IGNORE,
                cursor_layer,
            )],
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
    mut materials: ResMut<Assets<CursorMaterial>>,
    eye: Single<(&mut EyeBlinkAnimation, &MeshMaterial2d<CursorMaterial>)>,
) {
    let (mut blink, material_handle) = eye.into_inner();
    blink.timer.tick(time.delta());
    if !blink.timer.is_finished() {
        return;
    }

    let (phase, frame, duration) = match blink.phase {
        EyeBlinkPhase::Waiting => (
            EyeBlinkPhase::Closing,
            CursorEyeFrame::Half,
            EYE_BLINK_CLOSING_DURATION,
        ),
        EyeBlinkPhase::Closing => (
            EyeBlinkPhase::Closed,
            CursorEyeFrame::Closed,
            EYE_BLINK_CLOSED_DURATION,
        ),
        EyeBlinkPhase::Closed => (
            EyeBlinkPhase::Opening,
            CursorEyeFrame::Half,
            EYE_BLINK_OPENING_DURATION,
        ),
        EyeBlinkPhase::Opening => (
            EyeBlinkPhase::Waiting,
            CursorEyeFrame::Open,
            rand::rng().random_range(EYE_BLINK_MIN_INTERVAL..EYE_BLINK_MAX_INTERVAL),
        ),
    };
    blink.phase = phase;
    blink.timer = Timer::from_seconds(duration, TimerMode::Once);
    if let Some(mut material) = materials.get_mut(&material_handle.0) {
        material.set_eye_frame(frame);
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
    mut materials: ResMut<Assets<ProceduralCloudMaterial>>,
) {
    let mut rng = rand::rng();

    for (mut spawner, global_transform) in &mut spawner_query {
        spawner.timer.tick(time.delta());

        if spawner.timer.just_finished() {
            let shape_index = rng.random_range(0..spawner.puff_shapes.len());
            let shape_id = spawner.puff_shapes[shape_index];
            let material = materials.add(ProceduralCloudMaterial::new(
                shape_id,
                shape_id.canvas(),
                Color::srgba(1.0, 1.0, 1.0, 0.35),
            ));

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
                Mesh2d(spawner.puff_meshes[shape_index].clone()),
                MeshMaterial2d(material),
                Transform::from_translation(spawn_pos).with_scale(Vec3::splat(0.1)),
                RenderLayers::layer(CURSOR_RENDER_LAYER),
            ));
        }
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

        // Vertical oscillation
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
