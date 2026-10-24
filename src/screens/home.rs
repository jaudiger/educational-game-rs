use bevy::input_focus::AutoFocus;
use bevy::prelude::*;
use bevy::window::PrimaryWindow;

use crate::states::AppState;
use crate::ui::components::{MenuBackgroundStyle, menu_background, screen_root, standard_button};
use crate::ui::navigation::NavigateTo;
use crate::ui::theme;

#[derive(Resource, Clone, Debug, Default, PartialEq, Eq)]
pub struct HomePresentation {
    pub title: String,
    pub play: String,
    pub settings: String,
    pub quit: Option<String>,
    pub play_destination: AppState,
}

#[derive(Component, Reflect)]
pub struct HomeQuitAction;

/// Main menu screen with play and settings buttons.
pub struct HomeScreenPlugin;

impl Plugin for HomeScreenPlugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<HomePresentation>()
            .add_systems(OnEnter(AppState::Home), setup_home);
    }
}

pub fn setup_home(
    mut commands: Commands,
    presentation: Res<HomePresentation>,
    primary_window: Single<Entity, With<PrimaryWindow>>,
) {
    let window = *primary_window;
    let title = presentation.title.clone();
    let play = presentation.play.clone();
    let settings = presentation.settings.clone();
    let quit = presentation.quit.clone();
    let play_destination = presentation.play_destination;

    commands.spawn((
        screen_root(),
        DespawnOnExit(AppState::Home),
        children![
            menu_background(MenuBackgroundStyle::new(0.48)),
            (
                Text::new(title),
                theme::typography::text(theme::fonts::HERO, window),
                TextColor(theme::colors::TEXT_DARK),
            ),
            (
                Node {
                    width: theme::layout::FILL_PARENT,
                    min_width: px(0.0),
                    flex_direction: FlexDirection::Column,
                    align_items: AlignItems::Center,
                    row_gap: theme::scaled(theme::spacing::MEDIUM),
                    ..default()
                },
                Children::spawn(SpawnWith(move |buttons: &mut ChildSpawner| {
                    buttons.spawn((
                        standard_button(
                            &play,
                            theme::button_styles::ACTION_PRIMARY,
                            theme::scaled(theme::sizes::BUTTON_WIDTH),
                            window,
                        ),
                        NavigateTo(play_destination),
                        AutoFocus,
                    ));
                    buttons.spawn((
                        standard_button(
                            &settings,
                            theme::button_styles::ACTION_SECONDARY,
                            theme::scaled(theme::sizes::BUTTON_WIDTH),
                            window,
                        ),
                        NavigateTo(AppState::Settings),
                    ));
                    if let Some(quit) = quit {
                        buttons.spawn((
                            standard_button(
                                &quit,
                                theme::button_styles::DANGER,
                                theme::scaled(theme::sizes::BUTTON_WIDTH),
                                window,
                            ),
                            HomeQuitAction,
                        ));
                    }
                })),
            ),
        ],
    ));
}
