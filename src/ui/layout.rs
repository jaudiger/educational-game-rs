use bevy::prelude::*;
use bevy::ui::UiSystems;

use super::theme;

const ROW_COLUMN_HYSTERESIS: f32 = 0.02;
const GRID_ROUNDING_ALLOWANCE: f32 = 1.0;

/// Runs shared responsive layout decisions after Bevy has computed UI sizes.
pub struct ResponsiveLayoutPlugin;

/// Marks completion of shared responsive layout updates in `PostUpdate`.
#[derive(SystemSet, Clone, Copy, Debug, Eq, Hash, PartialEq)]
pub struct ResponsiveLayoutSet;

impl Plugin for ResponsiveLayoutPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(
            PostUpdate,
            update_adaptive_layouts
                .after(UiSystems::Layout)
                .in_set(ResponsiveLayoutSet),
        );
    }
}

/// Controls a collection that switches between one row and one column.
#[derive(Component, Reflect)]
pub struct AdaptiveLayout {
    /// Fraction of the parent content width available to the row.
    pub max_row_occupancy: f32,
}

impl AdaptiveLayout {
    pub const fn new(max_row_occupancy: f32) -> Self {
        Self { max_row_occupancy }
    }
}

/// Declares the width an item needs before its contents wrap internally.
///
/// The width resolves against the item's target viewport and the group's available content width.
#[derive(Component, Reflect, Clone, Copy)]
pub struct PreferredWidth(pub Val);

#[derive(Component, Reflect)]
struct UseComputedWidth;

#[derive(Component, Reflect)]
struct FluidCollection;

#[derive(Component, Reflect)]
struct MeasureNestedFluidCollectionWidth;

impl PreferredWidth {
    pub const fn new(width: Val) -> Self {
        Self(width)
    }
}

#[derive(Clone, Copy)]
enum CollectionKind {
    Fluid,
    Adaptive { max_row_occupancy: f32 },
}

/// Layout settings for a fluid collection, responsive grid, or adaptive group.
#[derive(Clone, Copy)]
pub struct CollectionLayout {
    kind: CollectionKind,
    max_width: Val,
    row_gap: Val,
    column_gap: Val,
}

impl CollectionLayout {
    /// Creates a collection that wraps its items across rows without a width cap.
    pub const fn fluid(gap: Val) -> Self {
        Self {
            kind: CollectionKind::Fluid,
            max_width: theme::layout::FILL_PARENT,
            row_gap: gap,
            column_gap: gap,
        }
    }

    /// Creates a wrapping grid with a caller-selected maximum width.
    pub const fn grid(max_width: Val, gap: Val) -> Self {
        let mut layout = Self::fluid(gap);
        layout.max_width = max_width;
        layout
    }

    /// Creates an atomic group that switches between one row and one column.
    pub const fn adaptive(max_row_occupancy: f32, gap: Val) -> Self {
        Self {
            kind: CollectionKind::Adaptive { max_row_occupancy },
            max_width: theme::layout::FILL_PARENT,
            row_gap: gap,
            column_gap: gap,
        }
    }

    /// Sets the row and column gaps independently.
    #[allow(dead_code)]
    pub const fn with_row_column_gaps(mut self, row_gap: Val, column_gap: Val) -> Self {
        self.row_gap = row_gap;
        self.column_gap = column_gap;
        self
    }
}

/// Flex sizing and optional pre-wrap width for one collection item.
///
/// Adaptive groups require a preferred width or an explicit layout-width opt-in for each item.
#[derive(Clone, Copy)]
pub struct CollectionItemLayout {
    basis: Val,
    grow: f32,
    shrink: f32,
    preferred_width: Option<Val>,
    use_computed_width: bool,
    measure_nested_fluid_collection: bool,
    max_width: Option<Val>,
}

impl CollectionItemLayout {
    pub const fn new(basis: Val, grow: f32, shrink: f32) -> Self {
        Self {
            basis,
            grow,
            shrink,
            preferred_width: None,
            use_computed_width: false,
            measure_nested_fluid_collection: false,
            max_width: None,
        }
    }

    /// Sets a maximum width for the rendered item cell.
    #[allow(dead_code)]
    pub const fn with_max_width(mut self, max_width: Val) -> Self {
        self.max_width = Some(max_width);
        self
    }

    /// Sets the width to resolve before internal wrapping.
    #[allow(dead_code)]
    pub const fn with_preferred_width(mut self, width: Val) -> Self {
        self.preferred_width = Some(width);
        self.use_computed_width = false;
        self.measure_nested_fluid_collection = false;
        self
    }

    /// Measures a direct nested fluid collection before it wraps.
    ///
    /// Nested items must declare preferred widths or disable flex shrinking.
    #[allow(dead_code)]
    pub const fn with_nested_fluid_collection_width(mut self) -> Self {
        self.preferred_width = None;
        self.use_computed_width = false;
        self.measure_nested_fluid_collection = true;
        self
    }

    /// Uses the computed width when internal wrapping cannot narrow the item.
    pub const fn with_layout_width(mut self) -> Self {
        self.preferred_width = None;
        self.use_computed_width = true;
        self.measure_nested_fluid_collection = false;
        self
    }
}

/// Spawns items using a shared collection layout and per-item flex sizing.
///
/// Returns the collection commands so callers can add components and observers to its container.
pub fn spawn_collection<'a>(
    parent: &'a mut ChildSpawnerCommands<'_>,
    item_count: usize,
    layout: CollectionLayout,
    item_layout: impl Fn(usize) -> CollectionItemLayout,
    mut spawn_item: impl FnMut(&mut ChildSpawnerCommands, usize),
) -> EntityCommands<'a> {
    let (flex_direction, flex_wrap) = match layout.kind {
        CollectionKind::Fluid => (FlexDirection::Row, FlexWrap::Wrap),
        CollectionKind::Adaptive { .. } => (FlexDirection::Column, FlexWrap::NoWrap),
    };
    let mut collection = parent.spawn(Node {
        flex_direction,
        flex_wrap,
        justify_content: JustifyContent::Center,
        align_items: AlignItems::Center,
        column_gap: layout.column_gap,
        row_gap: layout.row_gap,
        width: theme::layout::FILL_PARENT,
        max_width: layout.max_width,
        ..default()
    });
    match layout.kind {
        CollectionKind::Adaptive { max_row_occupancy } => {
            collection.insert(AdaptiveLayout::new(max_row_occupancy));
        }
        CollectionKind::Fluid => {
            collection.insert(FluidCollection);
        }
    }
    collection.with_children(|collection| {
        for index in 0..item_count {
            let sizing = item_layout(index);
            let mut cell = collection.spawn(Node {
                flex_basis: sizing.basis,
                flex_grow: sizing.grow,
                flex_shrink: sizing.shrink,
                min_width: px(0.0),
                max_width: sizing.max_width.unwrap_or(Val::Auto),
                justify_content: JustifyContent::Center,
                align_items: AlignItems::Center,
                ..default()
            });
            if let Some(width) = sizing.preferred_width {
                cell.insert(PreferredWidth::new(width));
            } else if sizing.use_computed_width {
                cell.insert(UseComputedWidth);
            } else if sizing.measure_nested_fluid_collection {
                cell.insert(MeasureNestedFluidCollectionWidth);
            }
            cell.with_children(|cell| spawn_item(cell, index));
        }
    });
    collection
}

/// Returns a no-wrap row node for children that must stay together.
pub fn atomic_child_group(gap: Val) -> Node {
    Node {
        flex_direction: FlexDirection::Row,
        flex_shrink: 0.0,
        align_items: AlignItems::Center,
        column_gap: gap,
        ..default()
    }
}

/// Computes a centered grid width that fits the requested number of cells.
pub const fn responsive_grid_max_width(cell_basis: f32, columns: usize, gap: f32) -> f32 {
    let mut width = cell_basis;
    let mut remaining = columns.saturating_sub(1);
    while remaining > 0 {
        width += cell_basis + gap;
        remaining -= 1;
    }
    if columns > 1 {
        width + GRID_ROUNDING_ALLOWANCE
    } else {
        width
    }
}

fn measure_fluid_collection_width(
    children: &Children,
    computed: &ComputedNode,
    style: &Node,
    target: &ComputedUiRenderTargetInfo,
    viewport_size: Vec2,
    items: &Query<(&ComputedNode, &Node, Option<&PreferredWidth>), Without<AdaptiveLayout>>,
) -> Option<f32> {
    let inset = computed.content_inset();
    let collection_width = (computed.size().x - inset.min_inset.x - inset.max_inset.x).max(0.0);
    if !collection_width.is_finite() || collection_width <= 0.0 {
        return None;
    }
    let column_gap = style
        .column_gap
        .resolve(
            computed.inverse_scale_factor.recip(),
            computed.size().x,
            viewport_size,
            computed.em_size,
            computed.rem_size,
        )
        .ok()?;
    if !column_gap.is_finite() || column_gap < 0.0 {
        return None;
    }
    children
        .iter()
        .try_fold((0_usize, 0.0_f32), |(count, total), child| {
            let (item, item_style, preferred_width) = items.get(child).ok()?;
            let width = if let Some(preferred_width) = preferred_width {
                resolve_preferred_width(
                    preferred_width.0,
                    item,
                    collection_width,
                    target,
                    viewport_size,
                )?
            } else if item_style.flex_shrink == 0.0 {
                item.size().x
            } else {
                return None;
            };
            if !width.is_finite() || width < 0.0 {
                return None;
            }
            let gap = if count == 0 { 0.0 } else { column_gap };
            let total = total + gap + width;
            total.is_finite().then_some((count + 1, total))
        })
        .and_then(|(count, width)| (count > 0).then_some(width))
}

fn resolve_preferred_width(
    width: Val,
    cell: &ComputedNode,
    available_width: f32,
    target: &ComputedUiRenderTargetInfo,
    viewport_size: Vec2,
) -> Option<f32> {
    let width = match width {
        Val::VMin(value) => {
            let logical_vmin = target.logical_size().min_element();
            px(value * theme::REFERENCE_VMIN / 100.0 * theme::design_scale_factor(logical_vmin))
        }
        width => width,
    };
    width
        .resolve(
            cell.inverse_scale_factor.recip(),
            available_width,
            viewport_size,
            cell.em_size,
            cell.rem_size,
        )
        .ok()
}

#[allow(clippy::type_complexity)]
fn update_adaptive_layouts(
    layout_nodes: Query<&ComputedNode>,
    item_nodes: Query<(
        &ComputedNode,
        Option<&PreferredWidth>,
        Has<UseComputedWidth>,
        Has<MeasureNestedFluidCollectionWidth>,
        Option<&Children>,
    )>,
    fluid_collections: Query<
        (&Children, &ComputedNode, &Node),
        (With<FluidCollection>, Without<AdaptiveLayout>),
    >,
    fluid_collection_items: Query<
        (&ComputedNode, &Node, Option<&PreferredWidth>),
        Without<AdaptiveLayout>,
    >,
    mut groups: Query<(
        &AdaptiveLayout,
        &ChildOf,
        &Children,
        &ComputedNode,
        &ComputedUiRenderTargetInfo,
        &mut Node,
    )>,
) {
    for (group, child_of, children, computed, target, mut style) in &mut groups {
        if !group.max_row_occupancy.is_finite() || group.max_row_occupancy <= 0.0 {
            continue;
        }
        let Ok(parent_layout) = layout_nodes.get(child_of.parent()) else {
            continue;
        };
        let parent_inset = parent_layout.content_inset();
        let available_width =
            (parent_layout.size().x - parent_inset.min_inset.x - parent_inset.max_inset.x).max(0.0);
        if available_width <= 0.0 {
            continue;
        }

        let viewport_size = target.physical_size().as_vec2();
        let scale_factor = computed.inverse_scale_factor.recip();
        let column_gap = style
            .column_gap
            .resolve(
                scale_factor,
                computed.size().x,
                viewport_size,
                computed.em_size,
                computed.rem_size,
            )
            .unwrap_or_default();
        let Some(required_width) = children
            .iter()
            .try_fold((0_usize, 0.0_f32), |(count, total), child| {
                let (
                    cell,
                    preferred_width,
                    use_computed_width,
                    measure_nested_fluid_collection,
                    cell_children,
                ) = item_nodes.get(child).ok()?;
                let width = if let Some(preferred_width) = preferred_width {
                    resolve_preferred_width(
                        preferred_width.0,
                        cell,
                        available_width,
                        target,
                        viewport_size,
                    )?
                } else if use_computed_width {
                    cell.size().x
                } else if measure_nested_fluid_collection {
                    let cell_children = cell_children?;
                    let mut nested_collections = cell_children
                        .iter()
                        .filter_map(|entity| fluid_collections.get(entity).ok());
                    let (nested_items, nested_computed, nested_style) =
                        nested_collections.next()?;
                    if nested_collections.next().is_some() {
                        return None;
                    }
                    measure_fluid_collection_width(
                        nested_items,
                        nested_computed,
                        nested_style,
                        target,
                        viewport_size,
                        &fluid_collection_items,
                    )?
                } else {
                    return None;
                };
                if !width.is_finite() || width < 0.0 {
                    return None;
                }
                let column_gap = if count == 0 { 0.0 } else { column_gap };
                let total = total + column_gap + width;
                total.is_finite().then_some((count + 1, total))
            })
            .map(|(_, width)| width)
        else {
            continue;
        };

        let max_row_width = available_width * group.max_row_occupancy;
        let row_threshold = if style.flex_direction == FlexDirection::Column {
            max_row_width * (1.0 - ROW_COLUMN_HYSTERESIS)
        } else {
            max_row_width
        };
        let direction = if required_width <= row_threshold {
            FlexDirection::Row
        } else {
            FlexDirection::Column
        };
        if style.flex_direction != direction {
            style.flex_direction = direction;
        }
    }
}
