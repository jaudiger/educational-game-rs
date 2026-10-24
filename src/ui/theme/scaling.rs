use bevy::prelude::*;

use super::{REFERENCE_VMIN, design_scale_factor};

#[derive(Component)]
pub(super) struct DesignNodeMetrics(Node);

pub(super) fn capture_design_node(
    trigger: On<Insert<Node>>,
    nodes: Query<&Node>,
    mut commands: Commands,
) {
    let entity = trigger.event_target();
    let Ok(node) = nodes.get(entity) else {
        return;
    };
    let mut entity_commands = commands.entity(entity);
    if has_design_metrics(node) {
        entity_commands.insert(DesignNodeMetrics(node.clone()));
    } else {
        entity_commands.remove::<DesignNodeMetrics>();
    }
}

#[allow(clippy::type_complexity)]
pub(super) fn resolve_design_nodes(
    mut nodes: Query<
        (&DesignNodeMetrics, &ComputedUiRenderTargetInfo, &mut Node),
        Or<(
            Changed<DesignNodeMetrics>,
            Changed<ComputedUiRenderTargetInfo>,
        )>,
    >,
) {
    for (design, target, mut node) in &mut nodes {
        let vmin = target.logical_size().min_element();
        apply_design_metrics(&mut node, &design.0, design_scale_factor(vmin));
    }
}

fn has_design_metrics(node: &Node) -> bool {
    [
        node.left,
        node.right,
        node.top,
        node.bottom,
        node.width,
        node.height,
        node.min_width,
        node.min_height,
        node.max_width,
        node.max_height,
        node.flex_basis,
        node.row_gap,
        node.column_gap,
    ]
    .into_iter()
    .any(is_viewport_metric)
        || has_rect_metric(node.margin)
        || has_rect_metric(node.padding)
        || has_rect_metric(node.border)
        || has_radius_metric(node.border_radius)
}

const fn is_viewport_metric(value: Val) -> bool {
    matches!(value, Val::VMin(_))
}

fn has_rect_metric(rect: UiRect) -> bool {
    [rect.left, rect.right, rect.top, rect.bottom]
        .into_iter()
        .any(is_viewport_metric)
}

fn has_radius_metric(radius: BorderRadius) -> bool {
    [
        radius.top_left.x,
        radius.top_left.y,
        radius.top_right.x,
        radius.top_right.y,
        radius.bottom_right.x,
        radius.bottom_right.y,
        radius.bottom_left.x,
        radius.bottom_left.y,
    ]
    .into_iter()
    .any(is_viewport_metric)
}

fn apply_design_metrics(node: &mut Node, design: &Node, scale: f32) {
    macro_rules! apply_values {
        ($($field:ident),+ $(,)?) => {
            $(apply_value(&mut node.$field, design.$field, scale);)+
        };
    }

    apply_values!(
        left, right, top, bottom, width, height, min_width, min_height, max_width, max_height,
        flex_basis, row_gap, column_gap,
    );
    apply_rect(&mut node.margin, design.margin, scale);
    apply_rect(&mut node.padding, design.padding, scale);
    apply_rect(&mut node.border, design.border, scale);
    apply_radius(&mut node.border_radius, design.border_radius, scale);
}

fn apply_value(target: &mut Val, design: Val, scale: f32) {
    if let Val::VMin(value) = design {
        *target = px(value * REFERENCE_VMIN / 100.0 * scale);
    }
}

fn apply_rect(target: &mut UiRect, design: UiRect, scale: f32) {
    apply_value(&mut target.left, design.left, scale);
    apply_value(&mut target.right, design.right, scale);
    apply_value(&mut target.top, design.top, scale);
    apply_value(&mut target.bottom, design.bottom, scale);
}

fn apply_radius(target: &mut BorderRadius, design: BorderRadius, scale: f32) {
    apply_corner(&mut target.top_left, design.top_left, scale);
    apply_corner(&mut target.top_right, design.top_right, scale);
    apply_corner(&mut target.bottom_right, design.bottom_right, scale);
    apply_corner(&mut target.bottom_left, design.bottom_left, scale);
}

fn apply_corner(target: &mut CornerRadius, design: CornerRadius, scale: f32) {
    apply_value(&mut target.x, design.x, scale);
    apply_value(&mut target.y, design.y, scale);
}
