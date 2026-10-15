pub mod colors {
    use bevy::prelude::Color;

    pub const INPUT_BG: Color = Color::srgb(1.0, 1.0, 1.0);
}

pub mod button_styles {
    use bevy::prelude::Color;

    use super::super::{ButtonColorStates, ButtonPalette};
    use super::super::{button_styles as shared_button_styles, colors as shared_colors};

    pub const LIST_ITEM: ButtonPalette = ButtonPalette::new(
        ButtonColorStates::new(
            Color::srgb(0.8980, 0.9333, 0.9686),
            Color::srgb(0.8196, 0.8745, 0.9294),
            Color::srgb(0.7412, 0.8196, 0.8980),
        ),
        shared_colors::TEXT_DARK,
    );
    pub const LIST_ITEM_SELECTED: ButtonPalette = shared_button_styles::ACTION_PRIMARY;
    pub const SLOT_CARD: ButtonPalette = ButtonPalette::new(
        ButtonColorStates::new(
            shared_colors::CARD_BG,
            Color::srgb(0.8980, 0.9333, 0.9686),
            Color::srgb(0.8275, 0.8824, 0.9412),
        ),
        shared_colors::TEXT_DARK,
    );
}

pub mod fonts {
    pub const BUTTON_SMALL: f32 = 22.0;
}

pub mod sizes {
    pub const CARD_MIN_WIDTH: f32 = 250.0;
    pub const CARD_MIN_HEIGHT: f32 = 150.0;
    pub const INPUT_FIELD_HEIGHT: f32 = 50.0;
    pub const POPOVER_MIN_WIDTH: f32 = 280.0;
}
