use bevy::prelude::*;

use crate::data::{ExplanationVisual, Language};
use crate::questions::fraction_bar::fraction_bar;
use crate::questions::fraction_shape::spawn_fraction_representation;
use crate::questions::{QuestionVisualLayout, QuestionVisualSizing, spawn_question_visuals};
use crate::ui::components::{operator_visual_group, stacked_fraction};
use crate::ui::layout::{self, CollectionItemLayout, CollectionLayout};
use crate::ui::theme;

/// Spawns the visual element for a given explanation type as a child of the parent node.
pub fn spawn_explanation_visual(
    parent: &mut ChildSpawnerCommands,
    visual: &ExplanationVisual,
    window: Entity,
    language: Language,
    images: &mut Assets<Image>,
) {
    match visual {
        ExplanationVisual::FractionRepresentation {
            numerator,
            denominator,
            mode,
        } => {
            spawn_fraction_representation(parent, images, *numerator, *denominator, *mode);
        }
        ExplanationVisual::FractionAddition { a, b, c } => {
            spawn_fraction_addition_visual(parent, *a, *b, *c, window);
        }
        ExplanationVisual::WholeFractions { count, denominator } => {
            spawn_whole_fractions_visual(parent, *count, *denominator);
        }
        ExplanationVisual::FractionComparison { a, b } => {
            spawn_fraction_comparison_visual(
                parent,
                &a.character,
                a.fraction,
                &b.character,
                b.fraction,
                window,
            );
        }
        ExplanationVisual::FractionComparisonWithConversion { a, b } => {
            spawn_fraction_comparison_conversion_visual(
                parent,
                &FractionComparisonEntry {
                    name: &a.character,
                    fraction: a.fraction,
                    converted: a.converted,
                },
                &FractionComparisonEntry {
                    name: &b.character,
                    fraction: b.fraction,
                    converted: b.converted,
                },
                window,
            );
        }
        ExplanationVisual::MultiplicationGrid { rows, cols } => {
            spawn_multiplication_grid_visual(parent, *rows, *cols, window);
        }
        ExplanationVisual::PlaceValueTable { number, multiplier } => {
            spawn_place_value_table_visual(parent, *number, *multiplier, language, window);
        }
    }
}

/// Renders `[a/b] + [c/b] = [(a+c)/b]` as three small fraction bars with
/// operator labels between them.
fn spawn_fraction_addition_visual(
    parent: &mut ChildSpawnerCommands,
    a: u32,
    b: u32,
    c: u32,
    window: Entity,
) {
    /// Width of each mini-bar in the addition visual.
    const MINI_BAR_WIDTH: f32 = 200.0;
    /// Height of each mini-bar in the addition visual.
    const MINI_BAR_HEIGHT: f32 = 40.0;

    spawn_question_visuals(
        parent,
        QuestionVisualLayout::SingleRowOrColumn {
            max_row_occupancy: 1.0,
        },
        &[
            QuestionVisualSizing::Intrinsic,
            QuestionVisualSizing::Intrinsic,
            QuestionVisualSizing::Intrinsic,
            QuestionVisualSizing::Intrinsic,
            QuestionVisualSizing::Intrinsic,
        ],
        |cell, index| match index {
            0 => {
                cell.spawn(fraction_bar(
                    b,
                    a,
                    theme::colors::PRIMARY,
                    false,
                    MINI_BAR_WIDTH,
                    MINI_BAR_HEIGHT,
                ));
            }
            1 => {
                cell.spawn((
                    Text::new("+"),
                    theme::typography::text(theme::fonts::HEADING, window),
                    TextColor(theme::colors::TEXT_DARK),
                ));
            }
            2 => {
                cell.spawn(fraction_bar(
                    b,
                    c,
                    theme::colors::SECONDARY,
                    false,
                    MINI_BAR_WIDTH,
                    MINI_BAR_HEIGHT,
                ));
            }
            3 => {
                cell.spawn((
                    Text::new("="),
                    theme::typography::text(theme::fonts::HEADING, window),
                    TextColor(theme::colors::TEXT_DARK),
                ));
            }
            _ => {
                cell.spawn(fraction_bar(
                    b,
                    a + c,
                    theme::colors::SUCCESS,
                    false,
                    MINI_BAR_WIDTH,
                    MINI_BAR_HEIGHT,
                ));
            }
        },
    );
}

/// Renders `count` fully-coloured fraction bars side by side, each with
/// `denominator` slices, illustrating that a fraction equals a whole number.
fn spawn_whole_fractions_visual(parent: &mut ChildSpawnerCommands, count: u32, denominator: u32) {
    /// Width of each mini-bar in the whole-fractions visual.
    const MINI_BAR_WIDTH: f32 = 200.0;
    /// Height of each mini-bar in the whole-fractions visual.
    const MINI_BAR_HEIGHT: f32 = 40.0;

    layout::spawn_collection(
        parent,
        count as usize,
        CollectionLayout::fluid(theme::scaled(theme::spacing::MEDIUM)).with_row_column_gaps(
            theme::scaled(theme::spacing::SMALL),
            theme::scaled(theme::spacing::MEDIUM),
        ),
        |_| CollectionItemLayout::new(Val::Auto, 0.0, 0.0),
        |cell, _| {
            cell.spawn(fraction_bar(
                denominator,
                denominator,
                theme::colors::PRIMARY,
                false,
                MINI_BAR_WIDTH,
                MINI_BAR_HEIGHT,
            ));
        },
    );
}

/// Renders two labelled fraction bars (one per character) for comparison
/// explanations (`SameDenominator` / `SameNumerator`).
fn spawn_fraction_comparison_visual(
    parent: &mut ChildSpawnerCommands,
    character_a: &str,
    fraction_a: (u32, u32),
    character_b: &str,
    fraction_b: (u32, u32),
    window: Entity,
) {
    const MINI_BAR_WIDTH: f32 = 300.0;
    const MINI_BAR_HEIGHT: f32 = 40.0;

    parent
        .spawn(Node {
            flex_direction: FlexDirection::Column,
            align_items: AlignItems::Center,
            width: theme::layout::FILL_PARENT,
            row_gap: theme::scaled(theme::spacing::MEDIUM),
            ..default()
        })
        .with_children(|col| {
            spawn_labelled_bar(
                col,
                character_a,
                fraction_a,
                theme::colors::PRIMARY,
                MINI_BAR_WIDTH,
                MINI_BAR_HEIGHT,
                window,
            );
            spawn_labelled_bar(
                col,
                character_b,
                fraction_b,
                theme::colors::SECONDARY,
                MINI_BAR_WIDTH,
                MINI_BAR_HEIGHT,
                window,
            );
        });
}

fn spawn_labelled_bar(
    parent: &mut ChildSpawnerCommands,
    name: &str,
    fraction: (u32, u32),
    color: Color,
    width: f32,
    height: f32,
    window: Entity,
) {
    parent
        .spawn(Node {
            flex_direction: FlexDirection::Row,
            align_items: AlignItems::Center,
            column_gap: theme::scaled(theme::spacing::MEDIUM),
            ..default()
        })
        .with_children(|row| {
            row.spawn(Node {
                width: theme::scaled(180.0),
                flex_direction: FlexDirection::Row,
                align_items: AlignItems::Center,
                column_gap: theme::scaled(theme::fonts::BODY * 0.28),
                ..default()
            })
            .with_children(|label| {
                label.spawn((
                    Text::new(format!("{name} :")),
                    theme::typography::text(theme::fonts::BODY, window),
                    TextColor(theme::colors::TEXT_DARK),
                ));
                label.spawn(stacked_fraction(
                    fraction.0,
                    fraction.1,
                    theme::fonts::BODY,
                    color,
                    theme::colors::TEXT_DARK,
                    window,
                ));
            });
            row.spawn(fraction_bar(
                fraction.1, fraction.0, color, false, width, height,
            ));
        });
}

/// Groups the data for one character's row in a fraction comparison with conversion.
struct FractionComparisonEntry<'a> {
    name: &'a str,
    fraction: (u32, u32),
    converted: (u32, u32),
}

/// Renders original fraction bars then converted bars with common denominator.
/// Shows `original => converted` for each character (`MultipleDenominator`).
fn spawn_fraction_comparison_conversion_visual(
    parent: &mut ChildSpawnerCommands,
    a: &FractionComparisonEntry<'_>,
    b: &FractionComparisonEntry<'_>,
    window: Entity,
) {
    const MINI_BAR_WIDTH: f32 = 200.0;
    const MINI_BAR_HEIGHT: f32 = 40.0;

    parent
        .spawn(Node {
            flex_direction: FlexDirection::Column,
            align_items: AlignItems::Center,
            width: theme::layout::FILL_PARENT,
            row_gap: theme::scaled(theme::spacing::MEDIUM),
            ..default()
        })
        .with_children(|col| {
            spawn_conversion_row(
                col,
                a,
                theme::colors::PRIMARY,
                MINI_BAR_WIDTH,
                MINI_BAR_HEIGHT,
                window,
            );
            spawn_conversion_row(
                col,
                b,
                theme::colors::SECONDARY,
                MINI_BAR_WIDTH,
                MINI_BAR_HEIGHT,
                window,
            );
        });
}

fn spawn_conversion_row(
    parent: &mut ChildSpawnerCommands,
    entry: &FractionComparisonEntry<'_>,
    color: Color,
    width: f32,
    height: f32,
    window: Entity,
) {
    layout::spawn_collection(
        parent,
        5,
        CollectionLayout::fluid(theme::scaled(theme::spacing::SMALL)),
        |_| CollectionItemLayout::new(Val::Auto, 0.0, 0.0),
        |cell, index| match index {
            0 => {
                cell.spawn((
                    Text::new(format!("{} :", entry.name)),
                    theme::typography::text(theme::fonts::BODY, window),
                    TextColor(theme::colors::TEXT_DARK),
                ));
            }
            1 => {
                cell.spawn(fraction_bar(
                    entry.fraction.1,
                    entry.fraction.0,
                    color,
                    false,
                    width,
                    height,
                ));
            }
            2 => {
                cell.spawn(stacked_fraction(
                    entry.fraction.0,
                    entry.fraction.1,
                    theme::fonts::BODY,
                    color,
                    theme::colors::TEXT_DARK,
                    window,
                ));
            }
            3 => {
                cell.spawn(operator_visual_group(
                    "=>",
                    theme::fonts::BODY,
                    theme::colors::TEXT_MUTED,
                    window,
                    stacked_fraction(
                        entry.converted.0,
                        entry.converted.1,
                        theme::fonts::BODY,
                        color,
                        theme::colors::TEXT_DARK,
                        window,
                    ),
                ));
            }
            _ => {
                cell.spawn(fraction_bar(
                    entry.converted.1,
                    entry.converted.0,
                    color,
                    false,
                    width,
                    height,
                ));
            }
        },
    );
}

/// Renders `groups` groups of `items_per_group` coloured cells separated by
/// "+" signs, illustrating a multiplication as repeated addition.
fn spawn_multiplication_grid_visual(
    parent: &mut ChildSpawnerCommands,
    groups: u32,
    items: u32,
    window: Entity,
) {
    /// Size of each coloured cell.
    const CELL_SIZE: f32 = 20.0;
    /// Gap between cells inside a group.
    const CELL_GAP: f32 = 3.0;
    /// Cell colour.
    const CELL_COLOR: Color = Color::srgb(0.4, 0.7, 0.95);

    let group_gap = theme::scaled(theme::spacing::SMALL);
    let grid_width = theme::scaled(CELL_SIZE.mul_add(6.0, CELL_GAP * 5.0));
    let spawn_cells = |grid_parent: &mut ChildSpawnerCommands| {
        layout::spawn_collection(
            grid_parent,
            items as usize,
            CollectionLayout::grid(grid_width, theme::scaled(CELL_GAP)),
            |_| CollectionItemLayout::new(theme::scaled(CELL_SIZE), 0.0, 0.0),
            |cell, _| {
                cell.spawn((
                    Node {
                        width: theme::scaled(CELL_SIZE),
                        height: theme::scaled(CELL_SIZE),
                        border_radius: BorderRadius::all(theme::scaled(3.0)),
                        ..default()
                    },
                    BackgroundColor(CELL_COLOR),
                ));
            },
        );
    };
    layout::spawn_collection(
        parent,
        groups as usize,
        CollectionLayout::fluid(group_gap),
        |_| CollectionItemLayout::new(Val::Auto, 0.0, 0.0),
        |cell, group_index| {
            if group_index == 0 {
                spawn_cells(cell);
            } else {
                cell.spawn(layout::atomic_child_group(group_gap))
                    .with_children(|group| {
                        group.spawn((
                            Text::new("+"),
                            theme::typography::text(theme::fonts::HEADING, window),
                            TextColor(theme::colors::TEXT_DARK),
                        ));
                        spawn_cells(group);
                    });
            }
        },
    );
}

/// Column metadata for the place-value table: (abbreviation FR, abbreviation EN, color).
const PV_COLUMNS: &[(&str, &str, Color)] = &[
    ("U", "O", Color::srgb(0.2, 0.6, 1.0)), // Unites / Ones (blue)
    ("D", "T", Color::srgb(0.9, 0.2, 0.2)), // Dizaines / Tens (red)
    ("C", "H", Color::srgb(0.2, 0.7, 0.2)), // Centaines / Hundreds (green)
    ("M", "Th", Color::srgb(0.4, 0.2, 0.6)), // Milliers / Thousands (purple)
    ("DM", "TTh", Color::srgb(0.95, 0.6, 0.1)), // Diz. de milliers / Ten Th. (orange)
];

/// Design size of each place-value table cell.
const PV_CELL_SIZE: f32 = 50.0;
/// Border width for place-value table cells.
const PV_BORDER_WIDTH: f32 = 1.5;
/// Text colour for the added zeros.
pub const PV_ZERO_COLOR: Color = Color::srgb(0.85, 0.2, 0.55);

fn pv_cell_node() -> Node {
    Node {
        width: theme::scaled(PV_CELL_SIZE),
        height: theme::scaled(PV_CELL_SIZE),
        justify_content: JustifyContent::Center,
        align_items: AlignItems::Center,
        border: UiRect::all(px(PV_BORDER_WIDTH)),
        border_radius: BorderRadius::all(theme::scaled(4.0)),
        ..default()
    }
}

/// Renders a place-value table showing how multiplying by a power of ten
/// shifts digits left and appends zeros.
fn spawn_place_value_table_visual(
    parent: &mut ChildSpawnerCommands,
    number: u32,
    multiplier: u32,
    language: Language,
    window: Entity,
) {
    if multiplier == 0 {
        warn!("Skipping invalid place-value visual");
        return;
    }
    let Some(result) = number.checked_mul(multiplier) else {
        warn!("Skipping invalid place-value visual");
        return;
    };
    let zeros_added = count_trailing_zeros(multiplier);
    let number_digits = digits_lsb(number);
    let result_digits = digits_lsb(result);
    let num_columns = number_digits.len().max(result_digits.len());
    if num_columns > PV_COLUMNS.len() {
        warn!("Skipping place-value visual beyond renderer capacity");
        return;
    }

    parent
        .spawn(Node {
            flex_direction: FlexDirection::Column,
            align_items: AlignItems::Center,
            ..default()
        })
        .with_children(|col| {
            spawn_pv_header_row(col, num_columns, language, window);
            spawn_pv_digit_row(col, &number_digits, num_columns, None, window);
            spawn_pv_digit_row(col, &result_digits, num_columns, Some(zeros_added), window);
            spawn_pv_legend_row(col, num_columns, language, window);
        });
}

/// Spawns the header row with column abbreviations on coloured backgrounds.
fn spawn_pv_header_row(
    parent: &mut ChildSpawnerCommands,
    num_columns: usize,
    language: Language,
    window: Entity,
) {
    parent
        .spawn(Node {
            flex_direction: FlexDirection::Row,
            ..default()
        })
        .with_children(|row| {
            for i in (0..num_columns).rev() {
                let (abbr_fr, abbr_en, color) = PV_COLUMNS[i];
                let abbr = match language {
                    Language::French => abbr_fr,
                    Language::English => abbr_en,
                };
                row.spawn((
                    pv_cell_node(),
                    BackgroundColor(color),
                    BorderColor::all(color),
                ))
                .with_children(|cell| {
                    cell.spawn((
                        Text::new(abbr),
                        theme::typography::text(theme::fonts::BODY, window),
                        TextColor(theme::colors::TEXT_LIGHT),
                    ));
                });
            }
        });
}

/// Spawns a row of digit cells. When `highlight_zeros` is `Some(n)`, the
/// `n` least-significant cells are highlighted as added zeros.
fn spawn_pv_digit_row(
    parent: &mut ChildSpawnerCommands,
    digits: &[u8],
    num_columns: usize,
    highlight_zeros: Option<usize>,
    window: Entity,
) {
    parent
        .spawn(Node {
            flex_direction: FlexDirection::Row,
            ..default()
        })
        .with_children(|row| {
            for i in (0..num_columns).rev() {
                let digit = digits.get(i).copied();
                let is_highlighted = highlight_zeros.is_some_and(|zeros_added| i < zeros_added);
                let (_, _, col_color) = PV_COLUMNS[i];

                let text_color = if is_highlighted {
                    PV_ZERO_COLOR
                } else {
                    theme::colors::TEXT_DARK
                };

                let border_color = col_color.with_alpha(0.3);
                row.spawn((
                    pv_cell_node(),
                    BackgroundColor(Color::WHITE),
                    BorderColor::all(border_color),
                ))
                .with_children(|cell| {
                    let text = digit.map_or_default(|d| d.to_string());
                    cell.spawn((
                        Text::new(text),
                        theme::typography::text(theme::fonts::HEADING, window),
                        TextColor(text_color),
                    ));
                });
            }
        });
}

/// Spawns the legend below the table.
fn spawn_pv_legend_row(
    parent: &mut ChildSpawnerCommands,
    num_columns: usize,
    language: Language,
    window: Entity,
) {
    parent
        .spawn((
            Node {
                flex_direction: FlexDirection::Column,
                align_items: AlignItems::Center,
                margin: UiRect::top(theme::scaled(theme::spacing::SMALL)),
                padding: UiRect::axes(
                    theme::scaled(theme::spacing::MEDIUM),
                    theme::scaled(theme::spacing::SMALL),
                ),
                border_radius: BorderRadius::all(theme::scaled(6.0)),
                ..default()
            },
            BackgroundColor(Color::srgba(1.0, 1.0, 1.0, 0.7)),
        ))
        .with_children(|container| {
            spawn_pv_legend_entries(container, num_columns, language, window);
        });
}

/// Spawns the legend entries as a horizontal row with each abbreviation
/// coloured to match its header column.
fn spawn_pv_legend_entries(
    parent: &mut ChildSpawnerCommands,
    num_columns: usize,
    language: Language,
    window: Entity,
) {
    const NAMES_FR: &[&str] = &[
        "Unités",
        "Dizaines",
        "Centaines",
        "Milliers",
        "Diz. de milliers",
    ];
    const NAMES_EN: &[&str] = &["Ones", "Tens", "Hundreds", "Thousands", "Ten Th."];

    let names = match language {
        Language::French => NAMES_FR,
        Language::English => NAMES_EN,
    };
    layout::spawn_collection(
        parent,
        num_columns,
        CollectionLayout::fluid(theme::scaled(theme::spacing::SMALL))
            .with_row_column_gaps(theme::scaled(2.0), theme::scaled(theme::spacing::SMALL)),
        |_| CollectionItemLayout::new(Val::Auto, 0.0, 0.0),
        |cell, index| {
            let i = num_columns - index - 1;
            let (abbr_fr, abbr_en, color) = PV_COLUMNS[i];
            let abbr = match language {
                Language::French => abbr_fr,
                Language::English => abbr_en,
            };
            let label = format!("{abbr} = {}", names[i]);
            cell.spawn(layout::atomic_child_group(theme::scaled(2.0)))
                .with_children(|entry| {
                    entry.spawn((
                        Text::new(label),
                        theme::typography::text(theme::fonts::SMALL, window),
                        TextColor(color),
                    ));
                    if i > 0 {
                        entry.spawn((
                            Text::new("\u{00B7}".to_owned()),
                            theme::typography::text(theme::fonts::SMALL, window),
                            TextColor(theme::colors::TEXT_MUTED),
                        ));
                    }
                });
        },
    );
}

/// Returns the digits of `n` in least-significant-first order.
/// Returns `[0]` for input `0`.
fn digits_lsb(mut n: u32) -> Vec<u8> {
    if n == 0 {
        return vec![0];
    }
    let mut digits = Vec::new();
    while n > 0 {
        #[allow(clippy::cast_possible_truncation)]
        digits.push((n % 10) as u8);
        n /= 10;
    }
    digits
}

/// Counts the trailing zeros of `n` (i.e. how many times 10 divides it).
pub const fn count_trailing_zeros(mut n: u32) -> usize {
    if n == 0 {
        return 0;
    }
    let mut count = 0;
    while n.is_multiple_of(10) {
        count += 1;
        n /= 10;
    }
    count
}
