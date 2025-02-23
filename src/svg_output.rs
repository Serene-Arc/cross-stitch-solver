use crate::grid_cell::GridCell;
use crate::stitch::HalfStitch;
use svg::node::element::{Circle, Group};
use svg::{Document, Node};

const DOT_SPACING: f64 = 50.0;
const DOT_RADIUS: f64 = 5.0;

pub fn create_graphic(stitches: &[HalfStitch]) -> Document {
    let centred_stitches = re_centre_stitches(stitches);
    let (bottom_stitches, top_stitches): (Vec<HalfStitch>, Vec<HalfStitch>) = centred_stitches
        .iter()
        .partition(|s| s.stitch_corner == centred_stitches[0].stitch_corner);

    let max_x = centred_stitches
        .iter()
        .map(|s| s.start.x)
        .reduce(isize::max)
        .unwrap();
    let max_y = centred_stitches
        .iter()
        .map(|s| s.start.y)
        .reduce(isize::max)
        .unwrap();

    let mut document = Document::new().set(
        "viewBox",
        (
            0,
            0,
            ((max_x + 1) as f64) * DOT_SPACING + (2.0 * DOT_RADIUS),
            ((max_y + 1) as f64) * DOT_SPACING + (2.0 * DOT_RADIUS),
        ),
    );

    let dot_group = draw_grid(max_x, max_y);
    let bottom_stitches_group = draw_stitches(&bottom_stitches, "green");
    let top_stitches_group = draw_stitches(&top_stitches, "red");
    let inter_stitch_group = draw_inter_stitch_movement(&centred_stitches);

    document = document.add(dot_group);
    document = document.add(bottom_stitches_group);
    document = document.add(top_stitches_group);
    document = document.add(inter_stitch_group);

    // Flip the SVG since the origin is the top left corner.
    document = document.set("transform", "scale(1,-1)");

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

fn draw_stitches(stitches: &[HalfStitch], colour: &str) -> Group {
    let mut bottom_stitch_group = Group::new()
        .set("fill", colour)
        .set("stroke-width", 1)
        .set("stroke", colour);
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
            );
        bottom_stitch_group.append(line);
    }
    bottom_stitch_group
}

/// Draw the lines that show where the thread travels on the back of the fabric.
fn draw_inter_stitch_movement(stitches: &[HalfStitch]) -> Group {
    let mut inter_stitch_movements = Group::new()
        .set("fill", "blue")
        .set("stroke-width", 1)
        .set("stroke", "blue")
        .set("stroke-dasharray", "10,10");
    for stitch in stitches.windows(2) {
        let line = svg::node::element::Line::new()
            .set(
                "x1",
                stitch[0].get_end_location().x as f64 * DOT_SPACING + DOT_RADIUS,
            )
            .set(
                "y1",
                stitch[0].get_end_location().y as f64 * DOT_SPACING + DOT_RADIUS,
            )
            .set("x2", stitch[1].start.x as f64 * DOT_SPACING + DOT_RADIUS)
            .set("y2", stitch[1].start.y as f64 * DOT_SPACING + DOT_RADIUS);
        inter_stitch_movements = inter_stitch_movements.add(line);
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
}
