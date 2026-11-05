use bevy::prelude::*;
use bevy::text::TextLayoutInfo;
use bevy::ui::UiSystems;

use crate::ui::theme;

#[derive(Component, Reflect, Default)]
pub(super) struct EqualChoiceWidthGroup {
    button_width: Option<f32>,
    available_width: Option<f32>,
    design_scale: Option<f32>,
}

#[derive(Component, Reflect)]
pub(super) struct EqualChoiceWidthButton;

pub struct QuestionChoiceLayoutPlugin;

impl Plugin for QuestionChoiceLayoutPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(
            PostUpdate,
            equalize_choice_button_widths.after(UiSystems::PostLayout),
        );
    }
}

fn equalize_choice_button_widths(
    mut groups: Query<(
        &Children,
        &ComputedUiRenderTargetInfo,
        &ComputedNode,
        &mut EqualChoiceWidthGroup,
    )>,
    children: Query<&Children>,
    text_layouts: Query<(&Text, &TextLayoutInfo)>,
    mut buttons: Query<(&ComputedNode, &mut Node), With<EqualChoiceWidthButton>>,
) {
    for (rows, target, computed_group, mut group) in &mut groups {
        let logical_vmin = target.logical_size().min_element();
        let scale = theme::design_scale_factor(logical_vmin);
        let available_width = computed_group.size.x * computed_group.inverse_scale_factor;
        if !scale.is_finite()
            || scale <= 0.0
            || !available_width.is_finite()
            || available_width <= 0.0
        {
            continue;
        }

        let choices = choice_buttons(rows, &children);
        if choices.is_empty()
            || !choices
                .iter()
                .all(|button| choice_text_layout_ready(*button, &children, &text_layouts))
        {
            continue;
        }

        let available_width_changed = group
            .available_width
            .is_some_and(|previous| (previous - available_width).abs() > 0.5);
        let design_scale_changed = group
            .design_scale
            .is_some_and(|previous| (previous - scale).abs() > f32::EPSILON);
        group.available_width = Some(available_width);
        group.design_scale = Some(scale);
        if group.button_width.is_some() && (available_width_changed || design_scale_changed) {
            group.button_width = None;
            set_choice_button_width(&choices, &mut buttons, Val::Auto);
            continue;
        }

        let button_width = if let Some(width) = group.button_width {
            width
        } else {
            let width = choices
                .iter()
                .filter_map(|button| buttons.get(*button).ok())
                .map(|(computed, _)| computed.size.x * computed.inverse_scale_factor)
                .filter(|width| width.is_finite())
                .fold(0.0_f32, f32::max);
            if width <= 0.0 {
                continue;
            }

            group.button_width = Some(width);
            width
        };

        set_choice_button_width(&choices, &mut buttons, px(button_width));
    }
}

fn choice_buttons(rows: &Children, children: &Query<&Children>) -> Vec<Entity> {
    rows.iter()
        .filter_map(|row| children.get(row).ok())
        .flat_map(Children::iter)
        .filter_map(|cell| children.get(cell).ok())
        .flat_map(Children::iter)
        .collect()
}

fn choice_text_layout_ready(
    button: Entity,
    children: &Query<&Children>,
    text_layouts: &Query<(&Text, &TextLayoutInfo)>,
) -> bool {
    let mut pending = vec![button];
    let mut found_text = false;

    while let Some(entity) = pending.pop() {
        if let Ok((text, layout)) = text_layouts.get(entity)
            && !text.0.is_empty()
        {
            found_text = true;
            if layout.glyphs.is_empty() {
                return false;
            }
        }
        if let Ok(descendants) = children.get(entity) {
            pending.extend(descendants.iter());
        }
    }

    found_text
}

fn set_choice_button_width(
    choices: &[Entity],
    buttons: &mut Query<(&ComputedNode, &mut Node), With<EqualChoiceWidthButton>>,
    width: Val,
) {
    for button in choices {
        if let Ok((_, mut style)) = buttons.get_mut(*button)
            && style.width != width
        {
            style.width = width;
        }
    }
}
