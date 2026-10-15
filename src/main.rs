use bevy::prelude::*;

mod data;
mod editions;
mod i18n;
mod platform;
mod plugins;
mod questions;
#[cfg(not(feature = "demo"))]
mod runtime_paths;
mod screens;
mod states;
mod ui;

fn main() {
    let bootstrap = platform::bootstrap();
    let mut app = App::new();
    app.add_plugins(bootstrap.plugins)
        .add_plugins(bootstrap.content)
        .add_plugins(editions::SharedGamePlugin)
        .add_plugins(editions::SelectedEditionPlugin::new(
            bootstrap.persistence_directory,
        ))
        .add_systems(Startup, setup)
        .run();
}

fn setup(mut commands: Commands) {
    commands.spawn((Camera2d, IsDefaultUiCamera));
}
