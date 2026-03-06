use bevy::prelude::*;
use bevy::state::state::FreelyMutableState;

/// Generic navigation component. Attach to any `Button` entity to make it
/// transition to the given state when pressed.
///
/// Works with any Bevy `FreelyMutableState` type (`AppState`, `LessonPhase`,
/// etc.). A matching `handle_navigate_to::<S>` system must be registered for
/// each state type used.
#[derive(Component, Reflect)]
pub struct NavigateTo<S: FreelyMutableState>(pub S);

/// Bevy system that handles all `NavigateTo<S>` button presses for a given
/// state type `S`. Register once per state type in `NavigationPlugin`.
fn handle_navigate_to<S: FreelyMutableState + Clone>(
    query: Query<(&Interaction, &NavigateTo<S>), Changed<Interaction>>,
    mut next_state: ResMut<NextState<S>>,
) {
    for (interaction, nav) in &query {
        if *interaction == Interaction::Pressed {
            next_state.set(nav.0.clone());
        }
    }
}

pub struct NavigationPlugin;

impl Plugin for NavigationPlugin {
    fn build(&self, app: &mut App) {
        use crate::states::{AppState, LessonPhase};

        app.add_systems(Update, handle_navigate_to::<AppState>)
            .add_systems(
                Update,
                handle_navigate_to::<LessonPhase>.run_if(in_state(AppState::LessonPlay)),
            );
    }
}
