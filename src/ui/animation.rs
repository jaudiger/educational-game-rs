use super::theme;
use crate::states::AppState;
use bevy::math::Curve;
use bevy::math::curve::easing::{EaseFunction, EasingCurve};
use bevy::prelude::*;

pub struct UiAnimationPlugin;

impl Plugin for UiAnimationPlugin {
    fn build(&self, app: &mut App) {
        app.add_observer(auto_screen_entry_animation).add_systems(
            Update,
            (
                tick_animate_scale,
                tick_screen_entry,
                tick_floating_cards,
                animate_button_hover,
            ),
        );
    }
}

/// Marker: opts a `Button` into hover/press scale animation.
#[derive(Component, Reflect)]
pub struct AnimatedButton;

/// One-shot scale animation via `UiTransform`.
/// Self-removes when complete.
#[derive(Component)]
pub struct AnimateScale {
    curve: EasingCurve<f32>,
    duration: f32,
    elapsed: f32,
}

impl AnimateScale {
    pub fn new(start: f32, end: f32, ease_fn: EaseFunction, duration: f32) -> Self {
        Self {
            curve: EasingCurve::new(start, end, ease_fn),
            duration,
            elapsed: 0.0,
        }
    }
}

/// Continuous floating oscillation for sky-theme cards.
///
/// Applies a sinusoidal vertical offset to `UiTransform.translation`
/// without affecting scale (which `AnimateScale` owns).
#[derive(Component, Reflect)]
pub struct FloatingCard {
    pub phase: f32,
    pub amplitude: f32,
}

/// Slide-up + scale-in entry animation for screen roots.
/// Self-removes when complete.
#[derive(Component)]
struct ScreenEntryAnimation {
    y_curve: EasingCurve<f32>,
    scale_curve: EasingCurve<f32>,
    duration: f32,
    elapsed: f32,
}

fn tick_animate_scale(
    mut commands: Commands,
    time: Res<Time>,
    mut query: Query<(Entity, &mut AnimateScale, &mut UiTransform)>,
) {
    let dt = time.delta_secs();
    for (entity, mut anim, mut transform) in &mut query {
        anim.elapsed += dt;
        let t = (anim.elapsed / anim.duration).min(1.0);
        let scale_val = anim.curve.sample_clamped(t);
        transform.scale = Vec2::splat(scale_val);
        if t >= 1.0 {
            commands.entity(entity).try_remove::<AnimateScale>();
        }
    }
}

fn tick_screen_entry(
    mut commands: Commands,
    time: Res<Time>,
    mut query: Query<(Entity, &mut ScreenEntryAnimation, &mut UiTransform)>,
) {
    let dt = time.delta_secs();
    for (entity, mut anim, mut transform) in &mut query {
        anim.elapsed += dt;
        let t = (anim.elapsed / anim.duration).min(1.0);
        let y_val = anim.y_curve.sample_clamped(t);
        transform.translation = Val2::new(vmin(0.0), theme::scaled(y_val));
        let scale_val = anim.scale_curve.sample_clamped(t);
        transform.scale = Vec2::splat(scale_val);
        if t >= 1.0 {
            commands.entity(entity).try_remove::<ScreenEntryAnimation>();
        }
    }
}

fn tick_floating_cards(time: Res<Time>, mut query: Query<(&FloatingCard, &mut UiTransform)>) {
    let elapsed = time.elapsed_secs();
    for (card, mut transform) in &mut query {
        let y = card.amplitude
            * elapsed
                .mul_add(theme::animation::FLOATING_SPEED, card.phase)
                .sin();
        transform.translation = Val2::new(px(0.0), px(y));
    }
}

type AnimatedButtonQuery<'w, 's> = Query<
    'w,
    's,
    (Entity, &'static Interaction, &'static UiTransform),
    (Changed<Interaction>, With<AnimatedButton>),
>;

fn animate_button_hover(mut commands: Commands, query: AnimatedButtonQuery<'_, '_>) {
    for (entity, interaction, transform) in &query {
        let current = transform.scale.x;
        let (target, ease_fn, duration) = match interaction {
            Interaction::Hovered => (
                theme::animation::BUTTON_HOVER_SCALE,
                EaseFunction::CubicOut,
                theme::animation::BUTTON_HOVER_DURATION,
            ),
            Interaction::Pressed => (
                theme::animation::BUTTON_PRESS_SCALE,
                EaseFunction::CubicIn,
                theme::animation::BUTTON_PRESS_DURATION,
            ),
            Interaction::None => (
                1.0,
                EaseFunction::CubicOut,
                theme::animation::BUTTON_HOVER_DURATION,
            ),
        };
        commands
            .entity(entity)
            .try_insert(AnimateScale::new(current, target, ease_fn, duration));
    }
}

/// Observer that auto-inserts a `ScreenEntryAnimation` on every entity that
/// gets a `DespawnOnExit<AppState>` component added (i.e. screen roots).
fn auto_screen_entry_animation(trigger: On<Add, DespawnOnExit<AppState>>, mut commands: Commands) {
    let entity = trigger.event_target();

    commands.entity(entity).insert((
        ScreenEntryAnimation {
            y_curve: EasingCurve::new(
                theme::animation::SCREEN_ENTRY_START_Y,
                0.0,
                EaseFunction::CubicOut,
            ),
            scale_curve: EasingCurve::new(
                theme::animation::SCREEN_ENTRY_START_SCALE,
                1.0,
                EaseFunction::CubicOut,
            ),
            duration: theme::animation::SCREEN_ENTRY_DURATION,
            elapsed: 0.0,
        },
        UiTransform {
            translation: Val2::new(
                vmin(0.0),
                theme::scaled(theme::animation::SCREEN_ENTRY_START_Y),
            ),
            scale: Vec2::splat(theme::animation::SCREEN_ENTRY_START_SCALE),
            ..default()
        },
    ));
}
