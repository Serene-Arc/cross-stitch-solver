use crate::grid_cell::GridCell;
use crate::stitch::HalfStitch;
use itertools::Itertools;
use std::collections::HashSet;
use svg::node::element::{Circle, Definitions, Group, Line, Marker, Mask, Path, Text};
use svg::{Document, Node};

const DOT_SPACING: f64 = 500.0;
const DOT_RADIUS: f64 = DOT_SPACING / 10.0;
const LINE_WIDTH: f64 = DOT_RADIUS / 5.0;
const FONT_SIZE: isize = DOT_RADIUS as isize;

pub fn create_graphic(stitches: &[HalfStitch]) -> Document {
    let centred_stitches = re_centre_stitches(stitches);
    let (bottom_stitches, top_stitches): (Vec<HalfStitch>, Vec<HalfStitch>) = centred_stitches
        .iter()
        .partition(|s| s.stitch_corner == centred_stitches[0].stitch_corner);

    let max_x = centred_stitches
        .iter()
        .flat_map(|s| [s.start.x, s.get_end_location().x])
        .reduce(isize::max)
        .unwrap();
    let max_y = centred_stitches
        .iter()
        .flat_map(|s| [s.start.y, s.get_end_location().y])
        .reduce(isize::max)
        .unwrap();

    let view_width = (max_x as f64) * DOT_SPACING + (2.0 * DOT_RADIUS);
    let view_height = (max_y as f64) * DOT_SPACING + (2.0 * DOT_RADIUS);

    let mut document = Document::new().set("viewBox", (0, 0, view_width, view_height));

    let mut defs = Definitions::new();
    defs = defs.add(create_arrow_marker("arrow-green", "green"));
    defs = defs.add(create_arrow_marker("arrow-red", "red"));
    defs = defs.add(create_arrow_marker("arrow-blue", "blue"));
    defs = defs.add(create_intersection_mask(max_x, max_y));
    document = document.add(defs);

    let dot_group = draw_grid(max_x, max_y, view_height);
    let (bottom_stitches_group, bottom_stitch_text) =
        draw_stitches(&bottom_stitches, "blue", 1, view_height);
    let (inter_stitch_group, inter_stitch_text) =
        draw_inter_stitch_movement(&centred_stitches, 2, view_height);
    let (top_stitches_group, top_stitch_text) = draw_stitches(
        &top_stitches,
        "red",
        1 + bottom_stitches.len() * 2,
        view_height,
    );

    let all_lines = bottom_stitches_group
        .iter()
        .chain(inter_stitch_group.iter())
        .chain(top_stitches_group.iter())
        .sorted_by_key(|l| l.0)
        .map(|l| l.1.clone())
        .collect_vec();

    document = document.add(dot_group);

    for l in all_lines {
        document.append(l);
    }

    document = document.add(bottom_stitch_text);
    document = document.add(inter_stitch_text);
    document = document.add(top_stitch_text);

    document
}

fn create_intersection_mask(max_x: isize, max_y: isize) -> Mask {
    let mut mask = Mask::new()
        .set("id", "intersection-mask")
        .set("x", "0")
        .set("y", "0")
        .set("width", "100%")
        .set("height", "100%");

    let mask_colouring = svg::node::element::Rectangle::new()
        .set("x", "0")
        .set("y", "0")
        .set("width", "100%")
        .set("height", "100%")
        .set("fill", "white");
    mask.append(mask_colouring);

    for col in 0..max_x {
        for row in 0..max_y {
            let mid_x: f64 = ((DOT_SPACING / 2.0) + DOT_SPACING * col as f64) + DOT_RADIUS;
            let mid_y: f64 = ((DOT_SPACING / 2.0) + DOT_SPACING * row as f64) + DOT_RADIUS;
            let cutout = Circle::new()
                .set("cx", mid_x)
                .set("cy", mid_y)
                .set("r", DOT_RADIUS / 4.0)
                .set("fill", "black");
            mask.append(cutout);
        }
    }

    mask
}

fn draw_grid(max_x: isize, max_y: isize, view_height: f64) -> Group {
    let mut dot_group = Group::new().set("fill", "black");
    for row in 0..=max_y {
        for col in 0..=max_x {
            // Offset by the radius of a dot so that the dot isn't cut off.
            let cx = col as f64 * DOT_SPACING + DOT_RADIUS;
            let cy = view_height - (row as f64 * DOT_SPACING + DOT_RADIUS);

            let circle = Circle::new()
                .set("cx", cx)
                .set("cy", cy)
                .set("r", DOT_RADIUS);

            dot_group = dot_group.add(circle);
        }
    }
    dot_group
}

fn draw_stitches(
    stitches: &[HalfStitch],
    colour: &str,
    starting_number: usize,
    view_height: f64,
) -> (Vec<(usize, Line)>, Group) {
    let mut number_sequence = std::iter::successors(Some(starting_number), |n| Some(n + 2));
    let mut stitch_lines = Vec::with_capacity(stitches.len());
    let mut text_group = Group::new().set("fill", colour).set("stroke", colour);
    for stitch in stitches {
        let mut line = _draw_line(view_height, stitch.start, stitch.get_end_location())
            .set("marker-end", format!("url(#arrow-{})", colour))
            .set("fill", colour)
            .set("stroke", colour);

        // If the starting number is 1, then this is the bottom stitch,
        // and we should apply the mask.
        if starting_number == 1 {
            line = line.set("mask", "url(#intersection-mask)");
        }
        let i = number_sequence.next().unwrap();
        stitch_lines.push((i, line));
        text_group = text_group.add(add_sequence_number(
            i,
            colour,
            stitch.start,
            stitch.get_end_location(),
            (0.0, 0.0),
            view_height,
        ));
    }
    (stitch_lines, text_group)
}

fn _draw_line(view_height: f64, first_point: GridCell, second_point: GridCell) -> Line {
    let y_1 = view_height - (first_point.y as f64 * DOT_SPACING + DOT_RADIUS);
    let y_2 = view_height - (second_point.y as f64 * DOT_SPACING + DOT_RADIUS);
    Line::new()
        .set("x1", first_point.x as f64 * DOT_SPACING + DOT_RADIUS)
        .set("y1", y_1)
        .set("x2", second_point.x as f64 * DOT_SPACING + DOT_RADIUS)
        .set("y2", y_2)
        .set("stroke-width", LINE_WIDTH)
}

fn create_arrow_marker(id: &str, colour: &str) -> Marker {
    Marker::new()
        .set("id", id)
        .set("viewBox", "0 0 10 10")
        .set("refX", 3) // Position the arrowhead at the end of the line
        .set("refY", 3)
        .set("markerWidth", 6)
        .set("markerHeight", 6)
        .set("orient", "auto-start-reverse") // Automatically orient the arrowhead
        .add(
            Path::new()
                .set("d", "M 0 0 L 6 3 L 0 6 z")
                .set("fill", colour),
        )
}

fn add_sequence_number(
    number: usize,
    colour: &str,
    first_point: GridCell,
    second_point: GridCell,
    text_offset: (f64, f64),
    view_height: f64,
) -> Text {
    // First, find the direction that the text is supposed to go.
    // We want the text to be near the beginning of the stroke,
    // but in the direction the line is going.
    let (x_pos, y_pos) = calculate_text_coordinates(first_point, second_point, view_height);

    Text::new(format!("{}", number))
        .set("x", x_pos + text_offset.0)
        .set("y", y_pos + text_offset.1)
        .set("color", "black")
        .set("fill", colour)
        .set("font-size", format!("{}", FONT_SIZE))
        .set("font", "monospace")
        .set("stroke", "black")
        .set("stroke-width", LINE_WIDTH / 3.0)
        .set("paint-order", "stroke")
}

fn calculate_text_coordinates(
    first_point: GridCell,
    second_point: GridCell,
    view_height: f64,
) -> (f64, f64) {
    let horizontal_direction = second_point.x - first_point.x;
    let vertical_direction = second_point.y - first_point.y;
    let x_pos = (first_point.x as f64 + (0.1 * horizontal_direction as f64)) * DOT_SPACING
        + DOT_RADIUS
        // Add offset to compensate for the text being drawn from the top left.
        // This helps avoid intersections between the text and the lines.
        + if horizontal_direction > 0 {
            FONT_SIZE as f64
        } else {
            5.0
        };

    let unadjusted_y_pos = (first_point.y as f64 + (0.1 * (second_point.y - first_point.y) as f64))
        * DOT_SPACING
        + (DOT_RADIUS * vertical_direction as f64);
    let y_pos = view_height - unadjusted_y_pos;
    (x_pos, y_pos)
}

/// Draw the lines that show where the thread travels on the back of the fabric.
fn draw_inter_stitch_movement(
    stitches: &[HalfStitch],
    starting_number: usize,
    view_height: f64,
) -> (Vec<(usize, Line)>, Group) {
    let mut number_sequence = std::iter::successors(Some(starting_number), |n| Some(n + 2));
    let mut seen_movement_pairs: HashSet<(GridCell, GridCell)> = HashSet::new();
    let mut inter_stitch_movements = Vec::with_capacity(stitches.len());
    let mut text_group = Group::new().set("fill", "green").set("stroke", "green");
    for stitch in stitches.windows(2) {
        let first_point = stitch[0].get_end_location();
        let second_point = stitch[1].start;
        let line = _draw_line(view_height, first_point, second_point)
            .set("stroke-dasharray", "10,10")
            .set("marker-end", format!("url(#arrow-{})", "green"))
            .set("fill", "green")
            .set("stroke", "green");
        let i = number_sequence.next().unwrap();
        inter_stitch_movements.push((i, line));
        let offset = if !seen_movement_pairs.contains(&(first_point, second_point)) {
            (0.0, 0.0)
        } else {
            (0.0, -FONT_SIZE as f64)
        };
        text_group = text_group.add(add_sequence_number(
            i,
            "green",
            first_point,
            second_point,
            offset,
            view_height,
        ));

        seen_movement_pairs.insert((first_point, second_point));
    }
    (inter_stitch_movements, text_group)
}

/// Move the stitches so that the bottommost and leftmost ones are at the origin.
fn re_centre_stitches(stitches: &[HalfStitch]) -> Vec<HalfStitch> {
    let leftmost_x = stitches
        .iter()
        .map(|s| s.start.x)
        .reduce(isize::min)
        .unwrap();
    let bottom_y = stitches
        .iter()
        .map(|s| s.start.y)
        .reduce(isize::min)
        .unwrap();
    stitches
        .iter()
        .map(|stitch| HalfStitch {
            start: GridCell {
                x: stitch.start.x - leftmost_x,
                y: stitch.start.y - bottom_y,
            },
            ..*stitch
        })
        .collect()
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::stitch::StartingStitchCorner;

    const SINGLE_ROW_VIEW_HEIGHT: f64 = DOT_SPACING + 2.0 * DOT_RADIUS;

    #[test]
    fn test_centre_stitches_no_work() {
        let stitches = vec![HalfStitch {
            start: GridCell::new(0, 0),
            stitch_corner: StartingStitchCorner::BottomLeft,
        }];
        let result = re_centre_stitches(&stitches);
        assert_eq!(result, stitches)
    }

    #[test]
    fn test_centre_stitches_one_x_re_centre() {
        let stitches = vec![HalfStitch {
            start: GridCell::new(5, 0),
            stitch_corner: StartingStitchCorner::BottomLeft,
        }];
        let result = re_centre_stitches(&stitches);
        assert_eq!(result, vec![HalfStitch::default()])
    }

    #[test]
    fn test_centre_stitches_one_negative_x_re_centre() {
        let stitches = vec![HalfStitch {
            start: GridCell::new(-5, 0),
            stitch_corner: StartingStitchCorner::BottomLeft,
        }];
        let result = re_centre_stitches(&stitches);
        assert_eq!(result, vec![HalfStitch::default()])
    }

    #[test]
    fn test_centre_stitches_one_y_re_centre() {
        let stitches = vec![HalfStitch {
            start: GridCell::new(0, 5),
            stitch_corner: StartingStitchCorner::BottomLeft,
        }];
        let result = re_centre_stitches(&stitches);
        assert_eq!(result, vec![HalfStitch::default()])
    }

    #[test]
    fn test_centre_stitches_one_negative_y_re_centre() {
        let stitches = vec![HalfStitch {
            start: GridCell::new(0, -5),
            stitch_corner: StartingStitchCorner::BottomLeft,
        }];
        let result = re_centre_stitches(&stitches);
        assert_eq!(result, vec![HalfStitch::default()])
    }

    #[test]
    fn test_centre_stitches_one_xy_re_centre() {
        let stitches = vec![HalfStitch {
            start: GridCell::new(5, 5),
            stitch_corner: StartingStitchCorner::BottomLeft,
        }];
        let result = re_centre_stitches(&stitches);
        assert_eq!(result, vec![HalfStitch::default()])
    }

    #[test]
    fn test_centre_stitches_one_negative_xy_re_centre() {
        let stitches = vec![HalfStitch {
            start: GridCell::new(-5, -5),
            stitch_corner: StartingStitchCorner::BottomLeft,
        }];
        let result = re_centre_stitches(&stitches);
        assert_eq!(result, vec![HalfStitch::default()])
    }

    #[test]
    fn test_centre_stitches_two_re_centre_no_work() {
        let stitches = vec![
            HalfStitch {
                start: GridCell::new(0, 0),
                stitch_corner: StartingStitchCorner::BottomLeft,
            },
            HalfStitch {
                start: GridCell::new(1, 1),
                stitch_corner: StartingStitchCorner::BottomLeft,
            },
        ];
        let result = re_centre_stitches(&stitches);
        assert_eq!(result, stitches)
    }

    #[test]
    fn test_centre_stitches_two_negative_re_centre() {
        let stitches = vec![
            HalfStitch {
                start: GridCell::new(-1, -1),
                stitch_corner: StartingStitchCorner::BottomLeft,
            },
            HalfStitch {
                start: GridCell::new(1, 1),
                stitch_corner: StartingStitchCorner::BottomLeft,
            },
        ];
        let result = re_centre_stitches(&stitches);
        assert_eq!(
            result,
            vec![
                HalfStitch::default(),
                HalfStitch {
                    start: GridCell::new(2, 2),
                    stitch_corner: StartingStitchCorner::BottomLeft,
                }
            ]
        )
    }

    #[test]
    fn test_make_svg_and_write_single_stitch() {
        let test_stitches = vec![
            HalfStitch {
                start: GridCell::new(0, 0),
                stitch_corner: StartingStitchCorner::BottomLeft,
            },
            HalfStitch {
                start: GridCell::new(1, 0),
                stitch_corner: StartingStitchCorner::BottomRight,
            },
        ];
        let document = create_graphic(&test_stitches);
        svg::save("stitches.svg", &document).unwrap()
    }

    #[test]
    fn test_make_svg_and_write_single_row() {
        let test_stitches = vec![
            HalfStitch {
                start: GridCell::new(0, 0),
                stitch_corner: StartingStitchCorner::BottomLeft,
            },
            HalfStitch {
                start: GridCell::new(1, 0),
                stitch_corner: StartingStitchCorner::BottomLeft,
            },
            HalfStitch {
                start: GridCell::new(2, 0),
                stitch_corner: StartingStitchCorner::BottomLeft,
            },
            HalfStitch {
                start: GridCell::new(3, 0),
                stitch_corner: StartingStitchCorner::BottomRight,
            },
            HalfStitch {
                start: GridCell::new(2, 0),
                stitch_corner: StartingStitchCorner::BottomRight,
            },
            HalfStitch {
                start: GridCell::new(1, 0),
                stitch_corner: StartingStitchCorner::BottomRight,
            },
        ];
        let document = create_graphic(&test_stitches);
        svg::save("stitches.svg", &document).unwrap()
    }

    #[test]
    fn test_calculate_text_position_stitch_bottom_left_to_top_right() {
        let test_stitch = HalfStitch {
            start: GridCell::new(0, 0),
            stitch_corner: StartingStitchCorner::BottomLeft,
        };
        let result = calculate_text_coordinates(
            test_stitch.start,
            test_stitch.get_end_location(),
            SINGLE_ROW_VIEW_HEIGHT,
        );
        let expected_x = 0.1 * DOT_SPACING + 50.0 + DOT_RADIUS;
        let expected_y = SINGLE_ROW_VIEW_HEIGHT - (2.0 * DOT_RADIUS);
        assert_eq!(result.0, expected_x);
        assert_eq!(result.1, expected_y);
    }

    #[test]
    fn test_calculate_text_position_stitch_vertical_top_to_bottom() {
        let result = calculate_text_coordinates(
            GridCell::new(0, 1),
            GridCell::new(0, 0),
            SINGLE_ROW_VIEW_HEIGHT,
        );
        let expected_x = DOT_RADIUS + 5.0;
        let expected_y = SINGLE_ROW_VIEW_HEIGHT - (DOT_SPACING - (0.1 * DOT_SPACING) - DOT_RADIUS);
        assert_eq!(result.0, expected_x);
        assert_eq!(result.1, expected_y);
    }

    #[test]
    fn test_calculate_text_position_stitch_vertical_bottom_to_top() {
        let result = calculate_text_coordinates(
            GridCell::new(0, 0),
            GridCell::new(0, 1),
            SINGLE_ROW_VIEW_HEIGHT,
        );
        let expected_x = DOT_RADIUS + 5.0;
        let expected_y = SINGLE_ROW_VIEW_HEIGHT - (2.0 * DOT_RADIUS);
        assert_eq!(result.0, expected_x);
        assert_eq!(result.1, expected_y);
    }
}
