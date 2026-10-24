//! Builds shared fraction visuals for interactive questions and feedback.

use bevy::asset::RenderAssetUsages;
use bevy::color::ColorToPacked;
use bevy::image::ImageSampler;
use bevy::input_focus::AutoFocus;
use bevy::input_focus::tab_navigation::TabIndex;
use bevy::prelude::*;
use bevy::render::render_resource::{Extent3d, TextureDimension, TextureFormat};
use bevy::ui::auto_directional_navigation::AutoDirectionalNavigation;
use bevy::ui::widget::ImageNode;
use bevy::ui_widgets::Button;

use crate::data::FractionShape;
use crate::questions::fraction_bar::{BAR_HEIGHT, BAR_WIDTH, fraction_bar};
use crate::ui::components::ButtonVisualStyle;
use crate::ui::theme;

const SQUARE_SIZE: (u32, u32) = (320, 320);
const CIRCLE_SIZE: (u32, u32) = (320, 320);
const GAP_WIDTH: f32 = 4.0;
const OUTLINE_WIDTH: f32 = 3.0;
const COLOR_UNCOLORED: Color = Color::srgb(0.73, 0.73, 0.78);
const COLOR_OUTLINE: Color = Color::srgb(0.60, 0.60, 0.65);

/// Spawns the selected fraction representation using the same view in question and feedback.
pub fn spawn_fraction_representation(
    parent: &mut ChildSpawnerCommands,
    images: &mut Assets<Image>,
    numerator: u32,
    denominator: u32,
    mode: FractionShape,
) {
    match mode {
        FractionShape::Line => {
            parent.spawn(fraction_bar(
                denominator,
                numerator,
                theme::colors::PRIMARY,
                false,
                BAR_WIDTH,
                BAR_HEIGHT,
            ));
        }
        FractionShape::Square | FractionShape::Circle => {
            let image = images.add(render_fraction_shape_by_numerator(
                mode,
                denominator,
                numerator,
                theme::colors::PRIMARY,
            ));
            let (width, height) = design_size(mode);
            parent.spawn((
                ImageNode::new(image),
                Node {
                    width: theme::scaled(width),
                    height: theme::scaled(height),
                    ..default()
                },
            ));
        }
    }
}

/// Stores the interactive state for a square or circle fraction.
#[derive(Component)]
pub struct InteractiveFractionShape {
    pub(crate) shape: FractionShape,
    pub(crate) denominator: u32,
    pub(crate) colored_parts: Vec<bool>,
    pub(crate) pending_pointer_part: Option<u32>,
    pub(crate) pointer_click_pending: bool,
    pub(crate) keyboard_part: u32,
    pub(crate) image: Handle<Image>,
}

impl InteractiveFractionShape {
    pub fn colored_count(&self) -> u32 {
        self.colored_parts
            .iter()
            .map(|&colored| u32::from(colored))
            .sum()
    }
}

/// Returns an interactive square or circle fraction control.
pub fn interactive_fraction_shape(
    images: &mut Assets<Image>,
    shape: FractionShape,
    denominator: u32,
    colored_color: Color,
) -> impl Bundle {
    let colored_parts = vec![false; denominator as usize];
    let image = images.add(render_fraction_shape_with_parts(
        shape,
        denominator,
        &colored_parts,
        colored_color,
    ));
    let palette = theme::ButtonPalette::new(
        theme::ButtonColorStates::flat(Color::NONE),
        theme::colors::TEXT_DARK,
    );
    let (width, height) = design_size(shape);

    (
        ImageNode::new(image.clone()),
        Node {
            width: theme::scaled(width),
            height: theme::scaled(height),
            ..default()
        },
        Button,
        BackgroundColor(Color::NONE),
        ButtonVisualStyle(palette),
        AutoDirectionalNavigation::default(),
        TabIndex(0),
        AutoFocus,
        Outline::new(
            Val::Px(theme::sizes::FOCUS_RING_WIDTH),
            Val::Px(theme::sizes::FOCUS_RING_OFFSET),
            Color::NONE,
        ),
        InteractiveFractionShape {
            shape,
            denominator,
            colored_parts,
            pending_pointer_part: None,
            pointer_click_pending: false,
            keyboard_part: 0,
            image,
        },
    )
}

/// Returns the part under normalized shape coordinates, excluding outlines and gaps.
#[allow(clippy::cast_precision_loss)]
pub fn fraction_shape_part_at(
    shape: FractionShape,
    denominator: u32,
    x: f32,
    y: f32,
) -> Option<u32> {
    if denominator == 0 || !(0.0..=1.0).contains(&x) || !(0.0..=1.0).contains(&y) {
        return None;
    }

    let (width, height) = pixel_size(shape);
    if is_outline(shape, x, y, width, height) || is_gap(shape, denominator, x, y, width, height) {
        return None;
    }
    part_at(shape, denominator, x, y)
}

/// Builds a pre-colored image from the current interactive state.
pub fn render_fraction_shape_with_parts(
    shape: FractionShape,
    denominator: u32,
    colored_parts: &[bool],
    colored_color: Color,
) -> Image {
    render_fraction_shape(
        shape,
        denominator,
        |part| {
            colored_parts
                .get(part as usize)
                .copied()
                .unwrap_or_default()
        },
        colored_color,
    )
}

/// Returns the design size used to display each generated shape texture.
#[allow(clippy::cast_precision_loss)]
const fn design_size(shape: FractionShape) -> (f32, f32) {
    let (width, height) = pixel_size(shape);
    (width as f32, height as f32)
}

/// Builds a pre-colored fraction image with transparent gaps between its parts.
#[allow(
    clippy::cast_possible_truncation,
    clippy::cast_precision_loss,
    clippy::cast_sign_loss
)]
fn render_fraction_shape_by_numerator(
    shape: FractionShape,
    denominator: u32,
    numerator: u32,
    colored_color: Color,
) -> Image {
    render_fraction_shape(shape, denominator, |part| part < numerator, colored_color)
}

#[allow(
    clippy::cast_possible_truncation,
    clippy::cast_precision_loss,
    clippy::cast_sign_loss
)]
fn render_fraction_shape(
    shape: FractionShape,
    denominator: u32,
    is_colored: impl Fn(u32) -> bool,
    colored_color: Color,
) -> Image {
    let (width, height) = pixel_size(shape);
    let pixel_count = (width * height) as usize;
    let mut data = vec![0_u8; pixel_count * 4];
    let colored = colored_color.to_srgba().to_u8_array();
    let uncolored = COLOR_UNCOLORED.to_srgba().to_u8_array();
    let outline = COLOR_OUTLINE.to_srgba().to_u8_array();

    if denominator > 0 {
        for y in 0..height {
            for x in 0..width {
                let normalized_x = (x as f32 + 0.5) / width as f32;
                let normalized_y = (y as f32 + 0.5) / height as f32;
                let Some(part) = part_at(shape, denominator, normalized_x, normalized_y) else {
                    continue;
                };
                let color = if is_outline(shape, normalized_x, normalized_y, width, height) {
                    outline
                } else if is_gap(
                    shape,
                    denominator,
                    normalized_x,
                    normalized_y,
                    width,
                    height,
                ) {
                    continue;
                } else if is_colored(part) {
                    colored
                } else {
                    uncolored
                };
                let pixel = ((y * width + x) * 4) as usize;
                data[pixel..pixel + 4].copy_from_slice(&color);
            }
        }
    }

    let mut image = Image::new(
        Extent3d {
            width,
            height,
            depth_or_array_layers: 1,
        },
        TextureDimension::D2,
        data,
        TextureFormat::Rgba8UnormSrgb,
        RenderAssetUsages::MAIN_WORLD | RenderAssetUsages::RENDER_WORLD,
    );
    image.sampler = ImageSampler::linear();
    image
}

const fn pixel_size(shape: FractionShape) -> (u32, u32) {
    match shape {
        FractionShape::Line => unreachable!(),
        FractionShape::Square => SQUARE_SIZE,
        FractionShape::Circle => CIRCLE_SIZE,
    }
}

#[allow(
    clippy::cast_precision_loss,
    clippy::cast_possible_truncation,
    clippy::cast_sign_loss
)]
fn part_at(shape: FractionShape, denominator: u32, x: f32, y: f32) -> Option<u32> {
    if denominator == 0 || !(0.0..=1.0).contains(&x) || !(0.0..=1.0).contains(&y) {
        return None;
    }

    match shape {
        FractionShape::Line => unreachable!(),
        FractionShape::Square => {
            let (rows, columns) = square_grid(denominator);
            let row = (y * rows as f32).floor() as u32;
            let column = (x * columns as f32).floor() as u32;
            (row < rows && column < columns).then_some(row * columns + column)
        }
        FractionShape::Circle => {
            let dx = x.mul_add(2.0, -1.0);
            let dy = y.mul_add(2.0, -1.0);
            if dx.mul_add(dx, dy * dy) > 1.0 {
                return None;
            }
            let angle = dx.atan2(-dy).rem_euclid(std::f32::consts::TAU);
            Some((angle * denominator as f32 / std::f32::consts::TAU).floor() as u32)
                .map(|part| part.min(denominator - 1))
        }
    }
}

const fn square_grid(denominator: u32) -> (u32, u32) {
    let mut rows = 1_u32;
    while rows
        .saturating_add(1)
        .saturating_mul(rows.saturating_add(1))
        <= denominator
    {
        rows += 1;
    }
    while !denominator.is_multiple_of(rows) {
        rows -= 1;
    }
    (rows, denominator / rows)
}

#[allow(clippy::cast_precision_loss)]
fn is_outline(shape: FractionShape, x: f32, y: f32, width: u32, height: u32) -> bool {
    let x = x * width as f32;
    let y = y * height as f32;
    match shape {
        FractionShape::Line => unreachable!(),
        FractionShape::Square => {
            x < OUTLINE_WIDTH
                || width as f32 - x < OUTLINE_WIDTH
                || y < OUTLINE_WIDTH
                || height as f32 - y < OUTLINE_WIDTH
        }
        FractionShape::Circle => {
            let radius = (width.min(height) as f32) * 0.5;
            let dx = (width as f32).mul_add(-0.5, x);
            let dy = (height as f32).mul_add(-0.5, y);
            radius - dx.hypot(dy) < OUTLINE_WIDTH
        }
    }
}

#[allow(
    clippy::cast_possible_truncation,
    clippy::cast_precision_loss,
    clippy::cast_sign_loss
)]
fn is_gap(shape: FractionShape, denominator: u32, x: f32, y: f32, width: u32, height: u32) -> bool {
    match shape {
        FractionShape::Line => unreachable!(),
        FractionShape::Square => {
            let (rows, columns) = square_grid(denominator);
            near_grid_line(x, columns, width as f32) || near_grid_line(y, rows, height as f32)
        }
        FractionShape::Circle => {
            let dx = x.mul_add(2.0, -1.0);
            let dy = y.mul_add(2.0, -1.0);
            let angle = dx.atan2(-dy).rem_euclid(std::f32::consts::TAU);
            let section = angle * denominator as f32 / std::f32::consts::TAU;
            if denominator <= 1 {
                return false;
            }
            let angle_delta =
                (section - section.round()).abs() * std::f32::consts::TAU / denominator as f32;
            let radius = dx.hypot(dy) * width.min(height) as f32 * 0.5;
            radius > GAP_WIDTH * 0.5 && radius * angle_delta < GAP_WIDTH * 0.5
        }
    }
}

#[allow(
    clippy::cast_possible_truncation,
    clippy::cast_precision_loss,
    clippy::cast_sign_loss
)]
fn near_grid_line(position: f32, divisions: u32, length: f32) -> bool {
    if divisions <= 1 {
        return false;
    }
    let scaled = position * divisions as f32;
    let boundary = scaled.round() as u32;
    if boundary == 0 || boundary >= divisions {
        return false;
    }
    let boundary_position = boundary as f32 / divisions as f32;
    (position - boundary_position).abs() * length < GAP_WIDTH * 0.5
}
