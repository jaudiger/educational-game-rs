mod flow;
mod layout;
mod theme;

use bevy::prelude::*;

use crate::states::AppState;

#[derive(Component, Reflect, Default)]
#[require(UiTransform)]
struct SummaryBalloon;

pub struct LessonSummaryScreenPlugin;

#[derive(SystemSet, Clone, Copy, Debug, Eq, Hash, PartialEq)]
pub enum LessonSummarySet {
    Prepare,
    Render,
}

#[derive(Resource, Clone, Debug, Default, PartialEq, Eq)]
pub struct LessonSummaryPresentation {
    pub save_notice: Option<String>,
}

impl Plugin for LessonSummaryScreenPlugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<LessonSummaryPresentation>()
            .configure_sets(
                OnEnter(AppState::LessonSummary),
                (LessonSummarySet::Prepare, LessonSummarySet::Render).chain(),
            )
            .add_systems(
                OnEnter(AppState::LessonSummary),
                layout::setup_lesson_summary.in_set(LessonSummarySet::Render),
            )
            .add_systems(
                Update,
                theme::update_summary_sun_glare.run_if(in_state(AppState::LessonSummary)),
            )
            .add_systems(
                Update,
                (
                    flow::handle_summary_return,
                    flow::tick_summary_reveal,
                    flow::tick_summary_departure,
                )
                    .chain()
                    .run_if(in_state(AppState::LessonSummary)),
            );
    }
}
