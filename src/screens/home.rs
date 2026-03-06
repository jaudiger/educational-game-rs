use bevy::input_focus::AutoFocus;
use bevy::prelude::*;
use bevy::window::PrimaryWindow;

use crate::i18n::{I18n, TranslationKey};
use crate::states::AppState;
use crate::ui::components::{screen_root, standard_button};
use crate::ui::navigation::NavigateTo;
use crate::ui::theme;
use crate::ui::theme::DesignFontSize;

/// Main menu screen with play and settings buttons.
pub struct HomeScreenPlugin;

impl Plugin for HomeScreenPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(OnEnter(AppState::Home), setup_home)
            .add_systems(Update, handle_quit.run_if(in_state(AppState::Home)));
    }
}

#[derive(Component, Reflect)]
struct QuitButton;

fn setup_home(
    mut commands: Commands,
    i18n: Res<I18n>,
    primary_window: Single<Entity, With<PrimaryWindow>>,
) {
    let window = *primary_window;
    let title = i18n.t(&TranslationKey::AppTitle);
    let play = i18n.t(&TranslationKey::Play);
    let settings = i18n.t(&TranslationKey::Settings);
    let quit = i18n.t(&TranslationKey::Quit);

    commands.spawn((
        screen_root(),
        DespawnOnExit(AppState::Home),
        children![
            // Title
            (
                Text::new(title),
                TextFont {
                    font_size: theme::fonts::HERO,
                    ..default()
                },
                TextColor(theme::colors::TEXT_DARK),
                DesignFontSize {
                    size: theme::fonts::HERO,
                    window,
                },
            ),
            // Buttons group
            (
                Node {
                    flex_direction: FlexDirection::Column,
                    align_items: AlignItems::Center,
                    row_gap: theme::scaled(theme::spacing::MEDIUM),
                    ..default()
                },
                children![
                    (
                        standard_button(
                            &play,
                            theme::colors::PRIMARY,
                            theme::scaled(theme::sizes::BUTTON_WIDTH),
                            window,
                        ),
                        NavigateTo(AppState::SaveSlots),
                        AutoFocus,
                    ),
                    (
                        standard_button(
                            &settings,
                            theme::colors::SECONDARY,
                            theme::scaled(theme::sizes::BUTTON_WIDTH),
                            window,
                        ),
                        NavigateTo(AppState::Settings),
                    ),
                    (
                        standard_button(
                            &quit,
                            theme::colors::ERROR,
                            theme::scaled(theme::sizes::BUTTON_WIDTH),
                            window,
                        ),
                        QuitButton,
                    ),
                ],
            ),
        ],
    ));
}

fn handle_quit(
    query: Query<&Interaction, (Changed<Interaction>, With<QuitButton>)>,
    mut app_exit: MessageWriter<AppExit>,
) {
    for interaction in &query {
        if *interaction == Interaction::Pressed {
            app_exit.write(AppExit::Success);
        }
    }
}
