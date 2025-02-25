use crate::grid_cell::GridCell;
use crate::stitch::HalfStitch;
use std::collections::HashSet;
use svg::node::element::{Circle, Group, Text};
use svg::Document;

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

    let mut document = Document::new().set(
        "viewBox",
        (
            0,
            0,
            (max_x as f64) * DOT_SPACING + (2.0 * DOT_RADIUS),
            (max_y as f64) * DOT_SPACING + (2.0 * DOT_RADIUS),
        ),
    );

    let dot_group = draw_grid(max_x, max_y);
    let bottom_stitches_group = draw_stitches(&bottom_stitches, "green", 1);
    let inter_stitch_group = draw_inter_stitch_movement(&centred_stitches, 2);
    let top_stitches_group = draw_stitches(&top_stitches, "red", 3);

    // Flip the SVG since the origin is the top left corner.
    document = document.set("transform", "scale(1,-1)");

    document = document.add(dot_group);
    document = document.add(bottom_stitches_group);
    document = document.add(top_stitches_group);
    document = document.add(inter_stitch_group);

    document
}

fn draw_grid(max_x: isize, max_y: isize) -> Group {
    let mut dot_group = Group::new().set("fill", "black");
    for row in 0..=max_y {
        for col in 0..=max_x {
            // Offset by the radius of a dot so that the dot isn't cut off.
            let cx = col as f64 * DOT_SPACING + DOT_RADIUS;
            let cy = row as f64 * DOT_SPACING + DOT_RADIUS;

            let circle = Circle::new()
                .set("cx", cx)
                .set("cy", cy)
                .set("r", DOT_RADIUS);

            dot_group = dot_group.add(circle);
        }
    }
    dot_group
}

fn draw_stitches(stitches: &[HalfStitch], colour: &str, starting_number: usize) -> Group {
    let mut number_sequence = std::iter::successors(Some(starting_number), |n| Some(n + 1));
    let mut bottom_stitch_group = Group::new().set("fill", colour).set("stroke", colour);
    for stitch in stitches {
        let line = svg::node::element::Line::new()
            .set("x1", stitch.start.x as f64 * DOT_SPACING + DOT_RADIUS)
            .set("y1", stitch.start.y as f64 * DOT_SPACING + DOT_RADIUS)
            .set(
                "x2",
                stitch.get_end_location().x as f64 * DOT_SPACING + DOT_RADIUS,
            )
            .set(
                "y2",
                stitch.get_end_location().y as f64 * DOT_SPACING + DOT_RADIUS,
            )
            .set("stroke-width", LINE_WIDTH);
        bottom_stitch_group = bottom_stitch_group.add(line);
        bottom_stitch_group = bottom_stitch_group.add(add_sequence_number(
            number_sequence.next().unwrap(),
            colour,
            stitch.start,
            stitch.get_end_location(),
            (0.0, 0.0),
        ));
    }
    bottom_stitch_group
}

fn add_sequence_number(
    number: usize,
    colour: &str,
    first_point: GridCell,
    second_point: GridCell,
    text_offset: (f64, f64),
) -> Text {
    // First, find the direction that the text is supposed to go.
    // We want the text to be near the beginning of the stroke,
    // but in the direction the line is going.
    let (x_pos, y_pos) = calculate_text_coordinates(first_point, second_point);

    // We need to use the negative of the y coordinate due to the flip.
    Text::new(format!("{}", number))
        .set("x", x_pos + text_offset.0)
        .set("y", -(y_pos + text_offset.1))
        .set("color", "black")
        .set("fill", colour)
        .set("transform", "scale(1,-1)")
        .set("font-size", format!("{}", FONT_SIZE))
        .set("font", "monospace")
        .set("stroke", "0.1")
        .set("paint-order", "stroke fill")
}

fn calculate_text_coordinates(first_point: GridCell, second_point: GridCell) -> (f64, f64) {
    let horizontal_direction = second_point.x - first_point.x;
    let vertical_direction = second_point.y - first_point.y;
    let x_pos = (first_point.x as f64 + (0.1 * horizontal_direction as f64)) * DOT_SPACING
        + DOT_RADIUS
        // Add offset to compensate for the text being drawn from the top left.
        + if horizontal_direction > 0 {
            FONT_SIZE as f64
        } else {
            5.0
        };

    let y_pos = (first_point.y as f64 + (0.1 * (second_point.y - first_point.y) as f64))
        * DOT_SPACING
        + (DOT_RADIUS * vertical_direction as f64);
    (x_pos, y_pos)
}

/// Draw the lines that show where the thread travels on the back of the fabric.
fn draw_inter_stitch_movement(stitches: &[HalfStitch], starting_number: usize) -> Group {
    let mut number_sequence = std::iter::successors(Some(starting_number), |n| Some(n + 1));
    let mut seen_movement_pairs: HashSet<(GridCell, GridCell)> = HashSet::new();
    let mut inter_stitch_movements = Group::new().set("fill", "blue").set("stroke", "blue");
    for stitch in stitches.windows(2) {
        let first_point = stitch[0].get_end_location();
        let second_point = stitch[1].start;
        let line = svg::node::element::Line::new()
            .set("x1", first_point.x as f64 * DOT_SPACING + DOT_RADIUS)
            .set("y1", first_point.y as f64 * DOT_SPACING + DOT_RADIUS)
            .set("x2", second_point.x as f64 * DOT_SPACING + DOT_RADIUS)
            .set("y2", second_point.y as f64 * DOT_SPACING + DOT_RADIUS)
            .set("stroke-width", LINE_WIDTH)
            .set("stroke-dasharray", "10,10");
        inter_stitch_movements = inter_stitch_movements.add(line);
        let offset = if !seen_movement_pairs.contains(&(first_point, second_point)) {
            (0.0, 0.0)
        } else {
            (0.0, -FONT_SIZE as f64)
        };
        inter_stitch_movements = inter_stitch_movements.add(add_sequence_number(
            number_sequence.next().unwrap(),
            "blue",
            first_point,
            second_point,
            offset,
        ));

        seen_movement_pairs.insert((first_point, second_point));
    }
    inter_stitch_movements
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
    fn test_make_svg_and_write_multiple_stitches() {
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
        let result = calculate_text_coordinates(test_stitch.start, test_stitch.get_end_location());
        let expected_x = 0.1 * DOT_SPACING + 50.0 + DOT_RADIUS;
        let expected_y = 2.0 * DOT_RADIUS;
        assert_eq!(result.0, expected_x);
        assert_eq!(result.1, expected_y);
    }

    #[test]
    fn test_calculate_text_position_stitch_vertical_top_to_bottom() {
        let result = calculate_text_coordinates(GridCell::new(0, 1), GridCell::new(0, 0));
        let expected_x = DOT_RADIUS + 5.0;
        let expected_y = DOT_SPACING - (0.1 * DOT_SPACING) - DOT_RADIUS;
        assert_eq!(result.0, expected_x);
        assert_eq!(result.1, expected_y);
    }

    #[test]
    fn test_calculate_text_position_stitch_vertical_bottom_to_top() {
        let result = calculate_text_coordinates(GridCell::new(0, 0), GridCell::new(0, 1));
        let expected_x = DOT_RADIUS + 5.0;
        let expected_y = 2.0 * DOT_RADIUS;
        assert_eq!(result.0, expected_x);
        assert_eq!(result.1, expected_y);
    }
}
