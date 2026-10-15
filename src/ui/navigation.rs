use bevy::prelude::*;
use bevy::state::state::FreelyMutableState;

use crate::states::AppState;

use super::components::ButtonActivated;

/// App destinations that shared navigation may enter.
#[derive(Resource, Clone, Debug, Reflect)]
pub struct AppNavigationPolicy {
    allowed_destinations: Vec<AppState>,
}

impl AppNavigationPolicy {
    pub fn new(allowed_destinations: impl IntoIterator<Item = AppState>) -> Self {
        Self {
            allowed_destinations: allowed_destinations.into_iter().collect(),
        }
    }

    fn allows(&self, destination: AppState) -> bool {
        self.allowed_destinations.contains(&destination)
    }
}

impl Default for AppNavigationPolicy {
    fn default() -> Self {
        Self::new([
            AppState::Home,
            AppState::SaveSlots,
            AppState::ThemeExploration,
            AppState::LessonPlay,
            AppState::LessonSummary,
            AppState::Settings,
        ])
    }
}

/// Generic navigation component. Attach to any `Button` entity to make it
/// transition to the given state when pressed.
///
/// Works with any Bevy `FreelyMutableState` type (`AppState`, `LessonPhase`,
/// etc.). A matching `handle_navigate_to::<S>` system must be registered for
/// each state type used.
#[derive(Component, Reflect)]
pub struct NavigateTo<S: FreelyMutableState>(pub S);

fn handle_app_navigate_to(
    mut activated: MessageReader<ButtonActivated>,
    query: Query<&NavigateTo<AppState>>,
    policy: Res<AppNavigationPolicy>,
    mut next_state: ResMut<NextState<AppState>>,
) {
    for activation in activated.read() {
        if let Ok(navigation) = query.get(activation.0)
            && policy.allows(navigation.0)
        {
            next_state.set(navigation.0);
        }
    }
}

/// Bevy system that handles all `NavigateTo<S>` button presses for a given
/// state type `S`. Register once per state type in `NavigationPlugin`.
fn handle_navigate_to<S: FreelyMutableState + Clone>(
    mut activated: MessageReader<ButtonActivated>,
    query: Query<&NavigateTo<S>>,
    mut next_state: ResMut<NextState<S>>,
) {
    for activation in activated.read() {
        if let Ok(navigation) = query.get(activation.0) {
            next_state.set(navigation.0.clone());
        }
    }
}

pub struct NavigationPlugin;

impl Plugin for NavigationPlugin {
    fn build(&self, app: &mut App) {
        use crate::states::LessonPhase;

        app.init_resource::<AppNavigationPolicy>()
            .add_systems(Update, handle_app_navigate_to)
            .add_systems(
                Update,
                handle_navigate_to::<LessonPhase>.run_if(in_state(AppState::LessonPlay)),
            );
    }
}

#[cfg(test)]
mod tests {
    use bevy::prelude::*;

    use super::{AppNavigationPolicy, NavigateTo, handle_app_navigate_to};
    use crate::states::AppState;
    use crate::ui::components::ButtonActivated;

    #[test]
    fn demo_navigation_ignores_production_only_destinations() {
        let mut app = App::new();
        app.insert_resource(NextState::<AppState>::Unchanged)
            .add_message::<ButtonActivated>()
            .insert_resource(AppNavigationPolicy::new([
                AppState::Home,
                AppState::ThemeExploration,
                AppState::LessonPlay,
                AppState::Settings,
            ]))
            .add_systems(Update, handle_app_navigate_to);
        app.update();

        let button = app.world_mut().spawn(NavigateTo(AppState::SaveSlots)).id();
        app.world_mut().write_message(ButtonActivated(button));
        app.update();

        assert!(matches!(
            app.world().resource::<NextState<AppState>>(),
            NextState::Unchanged,
        ));
    }
}
