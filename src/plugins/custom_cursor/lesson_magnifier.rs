use bevy::camera::NormalizedRenderTarget;
use bevy::camera::visibility::RenderLayers;
use bevy::curve::Curve;
use bevy::curve::easing::{EaseFunction, EasingCurve};
use bevy::ecs::system::SystemParam;
use bevy::picking::hover::HoverMap;
use bevy::picking::pointer::{
    PointerAction, PointerButton, PointerId, PointerInput, PointerLocation,
};
use bevy::prelude::*;
use bevy::sprite_render::MeshMaterial2d;
use bevy::window::{CursorOptions, PrimaryWindow, WindowRef};
use rand::RngExt;

use crate::data::{AnswerResult, LastAnswer};
use crate::plugins::cursor_material::{CursorEyeFrame, CursorMaterial};
use crate::states::{AppState, LessonPhase};
use crate::ui::components::ButtonVisualStyle;
use crate::ui::theme;

use super::{
    CURSOR_RENDER_LAYER, PointerCursor, TouchCursorState, activate_custom_cursor,
    spawn_cursor_camera,
};

pub(super) fn register(app: &mut App) {
    app.add_systems(OnEnter(AppState::LessonPlay), setup_lesson_cursor)
        .add_systems(OnExit(AppState::LessonPlay), super::cleanup_custom_cursor)
        .add_systems(
            OnEnter(LessonPhase::ShowFeedback),
            show_feedback_eye_expression,
        )
        .add_systems(OnExit(LessonPhase::ShowFeedback), restore_lesson_eye)
        .add_systems(
            Update,
            (
                (
                    handle_lesson_cursor_click_system,
                    animate_lesson_cursor_system,
                )
                    .chain()
                    .after(super::follow_pointer_system),
                animate_lesson_eye_system,
            )
                .run_if(in_state(AppState::LessonPlay))
                .run_if(resource_exists::<super::CustomCursorActive>),
        );
}

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
struct EyeAnimation {
    timer: Timer,
    phase: EyeBlinkPhase,
    feedback_motion: Option<FeedbackEyeMotion>,
}

#[derive(Clone, Copy, Reflect)]
enum FeedbackEyeMotion {
    Smiling {
        bob_elapsed: f32,
        blink_elapsed: f32,
    },
    Thinking {
        elapsed: f32,
    },
}

#[derive(Reflect)]
enum EyeBlinkPhase {
    Waiting,
    Closing,
    Closed,
    Opening,
}

#[derive(SystemParam)]
struct ButtonHoverState<'w, 's> {
    hover_map: Res<'w, HoverMap>,
    pointer_query: Query<'w, 's, (&'static PointerId, &'static PointerLocation)>,
    parent_query: Query<'w, 's, &'static ChildOf>,
    button_query: Query<'w, 's, (), With<ButtonVisualStyle>>,
}

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

fn setup_lesson_cursor(
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<CursorMaterial>>,
    mut cursor_options: Single<&mut CursorOptions, With<PrimaryWindow>>,
) {
    activate_custom_cursor(&mut commands, &mut cursor_options);

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
                EyeAnimation {
                    timer: Timer::from_seconds(blink_interval, TimerMode::Once),
                    phase: EyeBlinkPhase::Waiting,
                    feedback_motion: None,
                },
                Pickable::IGNORE,
                cursor_layer,
            )],
        )],
    ));
}

fn handle_lesson_cursor_click_system(
    primary_window: Single<Entity, With<PrimaryWindow>>,
    displayed_pointer: Single<&TouchCursorState, With<PointerCursor>>,
    mut pointer_inputs: MessageReader<PointerInput>,
    mut click_query: Query<&mut MagnifierScaleAnimation, With<MagnifierVisual>>,
) {
    let primary_target = WindowRef::Primary
        .normalize(Some(*primary_window))
        .map(NormalizedRenderTarget::Window);
    let displayed_pointer = displayed_pointer.displayed_pointer_id();

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
        let scale = Vec3::splat(hover_scale + click_growth);
        if transform.scale != scale {
            transform.scale = scale;
        }
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

fn show_feedback_eye_expression(
    answer: Res<LastAnswer>,
    mut materials: ResMut<Assets<CursorMaterial>>,
    mut eye_query: Query<(&mut EyeAnimation, &MeshMaterial2d<CursorMaterial>)>,
) {
    let (frame, motion) = match &answer.result {
        AnswerResult::Correct => (
            CursorEyeFrame::Smiling,
            FeedbackEyeMotion::Smiling {
                bob_elapsed: 0.0,
                blink_elapsed: 0.0,
            },
        ),
        AnswerResult::Incorrect => (
            CursorEyeFrame::Thinking,
            FeedbackEyeMotion::Thinking { elapsed: 0.0 },
        ),
    };

    for (mut animation, material_handle) in &mut eye_query {
        animation.feedback_motion = Some(motion);
        if let Some(mut material) = materials.get_mut(&material_handle.0) {
            material.set_eye_state(frame, Vec2::ZERO);
        }
    }
}

fn restore_lesson_eye(
    mut materials: ResMut<Assets<CursorMaterial>>,
    mut eye_query: Query<(&mut EyeAnimation, &MeshMaterial2d<CursorMaterial>)>,
) {
    let mut rng = rand::rng();
    let blink_interval = rng.random_range(EYE_BLINK_MIN_INTERVAL..EYE_BLINK_MAX_INTERVAL);

    for (mut animation, material_handle) in &mut eye_query {
        animation.feedback_motion = None;
        animation.phase = EyeBlinkPhase::Waiting;
        animation.timer = Timer::from_seconds(blink_interval, TimerMode::Once);
        if let Some(mut material) = materials.get_mut(&material_handle.0) {
            material.set_eye_state(CursorEyeFrame::Open, Vec2::ZERO);
        }
    }
}

const SMILING_EYE_BOB_AMPLITUDE: f32 = 3.0;
const SMILING_EYE_BOB_SPEED: f32 = 4.2;
const SMILING_EYE_BOB_CYCLE_DURATION: f32 = std::f32::consts::TAU / SMILING_EYE_BOB_SPEED;
const SMILING_EYE_CLOSED_DURATION: f32 = 2.4;
const SMILING_EYE_OPENING_DURATION: f32 = 0.12;
const SMILING_EYE_OPEN_DURATION: f32 = 0.18;
const SMILING_EYE_CLOSING_DURATION: f32 = 0.12;
const SMILING_EYE_BLINK_CYCLE_DURATION: f32 = SMILING_EYE_CLOSED_DURATION
    + SMILING_EYE_OPENING_DURATION
    + SMILING_EYE_OPEN_DURATION
    + SMILING_EYE_CLOSING_DURATION;
const THINKING_EYE_SWAY_AMPLITUDE: f32 = 5.0;
const THINKING_EYE_PAUSE_DURATION: f32 = 1.0;
const THINKING_EYE_MOVE_DURATION: f32 = 0.8;
const THINKING_EYE_CYCLE_DURATION: f32 =
    2.0 * (THINKING_EYE_PAUSE_DURATION + THINKING_EYE_MOVE_DURATION);

fn smiling_eye_bob(elapsed: f32) -> f32 {
    SMILING_EYE_BOB_AMPLITUDE * (elapsed * SMILING_EYE_BOB_SPEED).sin()
}

fn smiling_eye_frame(phase: f32) -> CursorEyeFrame {
    if phase < SMILING_EYE_CLOSED_DURATION {
        return CursorEyeFrame::Smiling;
    }

    let phase = phase - SMILING_EYE_CLOSED_DURATION;
    if phase < SMILING_EYE_OPENING_DURATION {
        return CursorEyeFrame::Half;
    }

    let phase = phase - SMILING_EYE_OPENING_DURATION;
    if phase < SMILING_EYE_OPEN_DURATION {
        return CursorEyeFrame::Open;
    }

    if phase < SMILING_EYE_OPEN_DURATION + SMILING_EYE_CLOSING_DURATION {
        return CursorEyeFrame::Half;
    }

    CursorEyeFrame::Smiling
}

fn thinking_eye_sway(mut phase: f32) -> f32 {
    if phase < THINKING_EYE_PAUSE_DURATION {
        return -THINKING_EYE_SWAY_AMPLITUDE;
    }

    phase -= THINKING_EYE_PAUSE_DURATION;
    if phase < THINKING_EYE_MOVE_DURATION {
        return interpolate_eye_sway(
            -THINKING_EYE_SWAY_AMPLITUDE,
            THINKING_EYE_SWAY_AMPLITUDE,
            phase / THINKING_EYE_MOVE_DURATION,
        );
    }

    phase -= THINKING_EYE_MOVE_DURATION;
    if phase < THINKING_EYE_PAUSE_DURATION {
        return THINKING_EYE_SWAY_AMPLITUDE;
    }

    phase -= THINKING_EYE_PAUSE_DURATION;
    interpolate_eye_sway(
        THINKING_EYE_SWAY_AMPLITUDE,
        -THINKING_EYE_SWAY_AMPLITUDE,
        phase / THINKING_EYE_MOVE_DURATION,
    )
}

fn interpolate_eye_sway(start: f32, end: f32, progress: f32) -> f32 {
    EasingCurve::new(start, end, EaseFunction::SineInOut).sample_clamped(progress)
}

fn animate_lesson_eye_system(
    time: Res<Time>,
    mut materials: ResMut<Assets<CursorMaterial>>,
    eye: Single<(&mut EyeAnimation, &MeshMaterial2d<CursorMaterial>)>,
) {
    let (mut animation, material_handle) = eye.into_inner();
    if let Some(motion) = &mut animation.feedback_motion {
        let delta = time.delta_secs();
        let (frame, eye_motion) = match motion {
            FeedbackEyeMotion::Smiling {
                bob_elapsed,
                blink_elapsed,
            } => {
                *bob_elapsed = (*bob_elapsed + delta).rem_euclid(SMILING_EYE_BOB_CYCLE_DURATION);
                *blink_elapsed =
                    (*blink_elapsed + delta).rem_euclid(SMILING_EYE_BLINK_CYCLE_DURATION);
                let frame = smiling_eye_frame(*blink_elapsed);
                (frame, Vec2::new(0.0, smiling_eye_bob(*bob_elapsed)))
            }
            FeedbackEyeMotion::Thinking { elapsed } => {
                *elapsed = (*elapsed + delta).rem_euclid(THINKING_EYE_CYCLE_DURATION);
                (
                    CursorEyeFrame::Thinking,
                    Vec2::new(thinking_eye_sway(*elapsed), 0.0),
                )
            }
        };
        if let Some(mut material) = materials.get_mut(&material_handle.0) {
            material.set_eye_state(frame, eye_motion);
        }
        return;
    }

    animation.timer.tick(time.delta());
    if !animation.timer.is_finished() {
        return;
    }

    let (phase, frame, duration) = match animation.phase {
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
    animation.phase = phase;
    animation.timer = Timer::from_seconds(duration, TimerMode::Once);
    if let Some(mut material) = materials.get_mut(&material_handle.0) {
        material.set_eye_frame(frame);
    }
}
