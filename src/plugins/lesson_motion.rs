use bevy::prelude::*;

use crate::states::{ActiveLesson, StateScopedResourceExt};

const WIND_FADE_SECONDS: f32 = 2.0;
const WIND_PERIOD_SECONDS: f32 = 6.5;

/// Provides shared wind motion for the active lesson and its summary.
pub struct LessonMotionPlugin;

#[derive(SystemSet, Clone, Copy, Debug, Eq, Hash, PartialEq)]
pub enum LessonMotionSet {
    AdvanceWind,
}

/// Tracks the wind phase shared by lesson scenery and mascots.
#[derive(Resource, Reflect, Default)]
pub struct LessonWind {
    phase: f32,
    fade_elapsed: f32,
}

impl LessonWind {
    pub fn value(&self) -> f32 {
        self.value_at_phase(self.phase)
    }

    pub fn delayed_value(&self, delay_seconds: f32) -> f32 {
        self.value_at_delay(delay_seconds, 0.0)
    }

    pub fn quadrature_value(&self) -> f32 {
        self.value_at_phase(self.phase + core::f32::consts::FRAC_PI_2)
    }

    pub fn delayed_quadrature_value(&self, delay_seconds: f32) -> f32 {
        self.value_at_delay(delay_seconds, core::f32::consts::FRAC_PI_2)
    }

    fn value_at_delay(&self, delay_seconds: f32, phase_offset: f32) -> f32 {
        let phase = (-delay_seconds).mul_add(
            core::f32::consts::TAU / WIND_PERIOD_SECONDS,
            self.phase + phase_offset,
        );
        self.value_at_phase(phase.rem_euclid(core::f32::consts::TAU))
    }

    fn value_at_phase(&self, phase: f32) -> f32 {
        let secondary_wave = 2.0_f32.mul_add(phase, 0.45).sin();
        let wind = 0.2_f32.mul_add(secondary_wave, 0.8 * phase.sin());
        wind * smooth_motion_fade(self.fade_elapsed, WIND_FADE_SECONDS)
    }
}

impl Plugin for LessonMotionPlugin {
    fn build(&self, app: &mut App) {
        app.register_state_scoped_resource::<ActiveLesson, LessonWind>(ActiveLesson)
            .add_systems(OnEnter(ActiveLesson), initialize_lesson_wind)
            .add_systems(
                Update,
                advance_lesson_wind
                    .in_set(LessonMotionSet::AdvanceWind)
                    .run_if(in_state(ActiveLesson)),
            );
    }
}

fn initialize_lesson_wind(mut commands: Commands) {
    commands.insert_resource(LessonWind::default());
}

fn advance_lesson_wind(time: Res<Time>, mut wind: ResMut<LessonWind>) {
    let delta = time.delta_secs();
    wind.phase = delta
        .mul_add(core::f32::consts::TAU / WIND_PERIOD_SECONDS, wind.phase)
        .rem_euclid(core::f32::consts::TAU);
    wind.fade_elapsed = (wind.fade_elapsed + delta).min(WIND_FADE_SECONDS);
}

pub fn smooth_motion_fade(elapsed: f32, duration: f32) -> f32 {
    let progress = (elapsed / duration).clamp(0.0, 1.0);
    progress * progress * (-2.0_f32).mul_add(progress, 3.0)
}
