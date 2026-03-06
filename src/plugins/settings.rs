use bevy::prelude::*;
use bevy::window::PrimaryWindow;
use bevy_persistent::prelude::*;

use crate::data::GameSettings;
use crate::i18n::{I18n, TranslationKey};

/// Keeps the window title and i18n resource in sync with persisted settings.
pub struct SettingsPlugin;

impl Plugin for SettingsPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(Startup, sync_window_title)
            .add_systems(
                Update,
                sync_i18n.run_if(resource_changed::<Persistent<GameSettings>>),
            )
            .add_systems(Update, sync_window_title.run_if(resource_changed::<I18n>));
    }
}

fn sync_window_title(mut window_query: Query<&mut Window, With<PrimaryWindow>>, i18n: Res<I18n>) {
    for mut window in &mut window_query {
        window.title = i18n.t(&TranslationKey::AppTitle).into_owned();
    }
}

/// Keeps [`I18n`] in sync with the persisted language in [`GameSettings`].
///
/// This is the **single source of sync**; no other code should write to
/// `I18n::language` directly. The guard avoids marking `I18n` as changed
/// when a non-language setting (volume, mode, etc.) is modified.
fn sync_i18n(settings: Res<Persistent<GameSettings>>, mut i18n: ResMut<I18n>) {
    if i18n.language != settings.language {
        i18n.language = settings.language;
    }
}
