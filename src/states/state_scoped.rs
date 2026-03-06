use bevy::prelude::*;

/// Despawns every entity carrying marker component `M`.
///
/// Use this as an `OnExit` system wherever a dedicated cleanup function
/// would just iterate a single-marker query and call `despawn`.
pub fn cleanup_root<M: Component>(mut commands: Commands, query: Query<Entity, With<M>>) {
    for entity in &query {
        commands.entity(entity).despawn();
    }
}

/// Extension trait for [`App`] that registers automatic resource cleanup
/// when exiting a state, serving as the resource equivalent of [`DespawnOnExit`]
/// for entities.
pub trait StateScopedResourceExt {
    /// Registers an [`OnExit`] system that removes resource `R` when
    /// leaving `state`. The resource is not inserted automatically;
    /// only its cleanup is registered.
    fn register_state_scoped_resource<S, R>(&mut self, state: S) -> &mut Self
    where
        S: States + Clone,
        R: Resource;
}

impl StateScopedResourceExt for App {
    fn register_state_scoped_resource<S, R>(&mut self, state: S) -> &mut Self
    where
        S: States + Clone,
        R: Resource,
    {
        self.add_systems(OnExit(state), |mut commands: Commands| {
            commands.remove_resource::<R>();
        })
    }
}
