use bevy::input_focus::AutoFocus;
use bevy::prelude::*;
use bevy::ui::InteractionDisabled;
use bevy::ui_render::ui_material::MaterialNode;

use crate::plugins::balloon_material::{BalloonFlame, BalloonUiMaterial};
use crate::plugins::lesson_background::LessonBackgroundParallaxLayer;
use crate::states::AppState;
use crate::ui::components::ButtonActivated;
use crate::ui::theme;

use super::SummaryBalloon;
use super::layout::{SummaryResultCard, SummaryReturnButton};
use super::theme::{SummaryCloud, SummaryCurtain, SummaryDepartureFade};

const REVEAL_DURATION: f32 = 3.2;
const DEPARTURE_DURATION: f32 = 1.6;
const DEPARTURE_FADE_START: f32 = 0.35;

#[derive(Clone, Copy, Debug, Eq, PartialEq, Reflect)]
pub(super) enum SummaryPhase {
    Revealing,
    Ready,
    Departing,
}

#[derive(Component, Reflect)]
pub(super) struct SummaryScreen {
    pub(super) phase: SummaryPhase,
    pub(super) elapsed: f32,
}

pub(super) fn handle_summary_return(
    mut commands: Commands,
    mut activations: MessageReader<ButtonActivated>,
    buttons: Query<(), With<SummaryReturnButton>>,
    curtains: Query<Entity, With<SummaryCurtain>>,
    mut screen: Single<&mut SummaryScreen>,
) {
    for activation in activations.read() {
        if buttons.contains(activation.0) && screen.phase != SummaryPhase::Departing {
            if screen.phase == SummaryPhase::Revealing {
                for curtain in &curtains {
                    commands.entity(curtain).despawn();
                }
            }
            screen.phase = SummaryPhase::Departing;
            screen.elapsed = 0.0;
            commands.entity(activation.0).insert(InteractionDisabled);
        }
    }
}

pub(super) fn tick_summary_reveal(
    mut commands: Commands,
    time: Res<Time>,
    mut screen: Single<&mut SummaryScreen>,
    mut curtains: Query<(Entity, &SummaryCurtain, &mut BackgroundColor)>,
    mut clouds: Query<(&SummaryCloud, &mut UiTransform, &ComputedUiRenderTargetInfo)>,
    buttons: Query<Entity, With<SummaryReturnButton>>,
) {
    if screen.phase != SummaryPhase::Revealing {
        return;
    }

    screen.elapsed = (screen.elapsed + time.delta_secs()).min(REVEAL_DURATION);
    let linear_progress = screen.elapsed / REVEAL_DURATION;
    let progress = ease_in_out(linear_progress);

    for (entity, curtain, mut background) in &mut curtains {
        background.0 = curtain.color.with_alpha(1.0 - progress);
        if linear_progress >= 1.0 {
            commands.entity(entity).despawn();
        }
    }

    for (cloud, mut transform, target) in &mut clouds {
        let size = target.logical_size();
        transform.translation = Val2::new(
            px(cloud.exit_direction.x * size.x * 1.15 * progress),
            px(cloud.exit_direction.y * size.y * 1.15 * progress),
        );
        transform.scale = Vec2::splat(progress.mul_add(-0.24, 1.0));
        transform.rotation = Rot2::radians(cloud.exit_direction.x * progress * 0.06);
    }

    if linear_progress >= 1.0 {
        screen.phase = SummaryPhase::Ready;
        screen.elapsed = 0.0;
        if let Ok(button) = buttons.single() {
            commands.entity(button).insert(AutoFocus);
        }
    }
}

#[allow(clippy::type_complexity)]
pub(super) fn tick_summary_departure(
    time: Res<Time>,
    mut screen: Single<&mut SummaryScreen>,
    mut balloon_materials: ResMut<Assets<BalloonUiMaterial>>,
    flames: Query<&MaterialNode<BalloonUiMaterial>, With<BalloonFlame>>,
    mut animated_elements: ParamSet<(
        Query<(&mut UiTransform, &ComputedUiRenderTargetInfo), With<SummaryBalloon>>,
        Query<(
            &LessonBackgroundParallaxLayer,
            &mut UiTransform,
            &ComputedUiRenderTargetInfo,
        )>,
        Query<(&mut UiTransform, &ComputedUiRenderTargetInfo), With<SummaryResultCard>>,
    )>,
    mut departure_fades: Query<&mut BackgroundColor, With<SummaryDepartureFade>>,
    mut next_state: ResMut<NextState<AppState>>,
) {
    if screen.phase != SummaryPhase::Departing {
        return;
    }

    screen.elapsed = (screen.elapsed + time.delta_secs()).min(DEPARTURE_DURATION);
    let linear_progress = screen.elapsed / DEPARTURE_DURATION;
    let progress = ease_in_out(linear_progress);
    for flame in &flames {
        if let Some(mut material) = balloon_materials.get_mut(&flame.0) {
            material.set_flame_intensity(
                (1.0 - BalloonFlame::BASE_INTENSITY)
                    .mul_add(progress, BalloonFlame::BASE_INTENSITY),
            );
        }
    }

    {
        for (mut transform, target) in &mut animated_elements.p0() {
            transform.translation.y = px(-target.logical_size().y * 1.1 * progress);
            transform.rotation = Rot2::radians(progress * 0.08);
        }
    }

    {
        for (layer, mut transform, target) in &mut animated_elements.p1() {
            let (vertical_scale, vertical_translation) =
                layer.departure_transform(target.logical_size().y, progress);
            transform.translation = Val2::new(px(0.0), px(vertical_translation));
            transform.scale = Vec2::new(1.0, vertical_scale);
        }
    }

    {
        for (mut transform, target) in &mut animated_elements.p2() {
            let vmin = target.logical_size().min_element();
            let offset = 24.0 * theme::design_scale_factor(vmin) * progress;
            transform.translation = Val2::new(px(0.0), px(offset));
            transform.scale = Vec2::splat(progress.mul_add(-0.035, 1.0));
        }
    }

    let fade_progress =
        ((linear_progress - DEPARTURE_FADE_START) / (1.0 - DEPARTURE_FADE_START)).clamp(0.0, 1.0);
    for mut background in &mut departure_fades {
        background.0 = background.0.with_alpha(ease_in_out(fade_progress));
    }

    if linear_progress >= 1.0 {
        next_state.set(AppState::ThemeExploration);
    }
}

fn ease_in_out(progress: f32) -> f32 {
    0.5 * (1.0 - (std::f32::consts::PI * progress).cos())
}
