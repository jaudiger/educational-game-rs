use bevy::prelude::*;

use crate::data::content::SchoolLevel;
use crate::ui::theme;

pub fn activity_level_badge(school_level: SchoolLevel, window: Entity) -> impl Bundle + use<> {
    (
        Node {
            min_height: theme::scaled(24.0),
            flex_shrink: 0.0,
            padding: UiRect::axes(theme::scaled(theme::spacing::SMALL), theme::scaled(2.0)),
            justify_content: JustifyContent::Center,
            align_items: AlignItems::Center,
            border: px(1.0).all(),
            border_radius: BorderRadius::all(theme::scaled(6.0)),
            ..default()
        },
        BackgroundColor(Color::NONE),
        BorderColor::all(theme::colors::INPUT_BORDER),
        children![(
            Text::new(school_level.label()),
            theme::typography::text(theme::fonts::SMALL, window),
            TextColor(theme::colors::TEXT_DARK),
            TextLayout::no_wrap(),
        )],
    )
}
