use crate::grid_cell::GridCell;
use crate::symbolic_sum::SymbolicSum;
use iced::widget::canvas::Path;
use iced::Point;
use num_derive::{FromPrimitive, ToPrimitive};
use num_traits::{FromPrimitive, ToPrimitive};
use std::collections::HashMap;
use std::fmt;
use std::fmt::Formatter;

#[derive(Debug, Clone, PartialEq, Eq, Copy, Hash, Default, FromPrimitive, ToPrimitive)]
pub enum StartingStitchCorner {
    #[default]
    BottomLeft = 0,
    TopLeft = 1,
    TopRight = 2,
    BottomRight = 3,
}

impl StartingStitchCorner {
    /// Get the corners where the alternate stitch can start.
    /// Given a bottom stitch's starting corner, there are two options to make a cross.
    pub fn get_possible_top_stitch_corners(&self) -> [Self; 2] {
        let first_option = (ToPrimitive::to_u8(self).unwrap() + 1) % 4;
        let second_option = (ToPrimitive::to_u8(self).unwrap() + 3) % 4;
        [
            FromPrimitive::from_u8(first_option).unwrap(),
            FromPrimitive::from_u8(second_option).unwrap(),
        ]
    }

    pub fn get_offset_from_bottom_left(&self) -> GridCell {
        match self {
            StartingStitchCorner::BottomLeft => GridCell::new(0, 0),
            StartingStitchCorner::BottomRight => GridCell::new(1, 0),
            StartingStitchCorner::TopLeft => GridCell::new(0, 1),
            StartingStitchCorner::TopRight => GridCell::new(1, 1),
        }
    }

    pub fn get_opposite_corner(&self) -> StartingStitchCorner {
        let opposite: Option<StartingStitchCorner> = FromPrimitive::from_u8((*self as u8 + 2) % 4);
        opposite.unwrap_or_else(|| panic!("Not a valid stitch corner"))
    }
}

impl fmt::Display for StartingStitchCorner {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        f.write_str(match self {
            StartingStitchCorner::BottomLeft => "Bottom Left",
            StartingStitchCorner::BottomRight => "Bottom Right",
            StartingStitchCorner::TopLeft => "Top Left",
            StartingStitchCorner::TopRight => "Top Right",
        })
    }
}

#[derive(PartialEq, Eq, Copy, Clone, Debug, Hash, Default)]
pub struct HalfStitch {
    // The start is the cell of the stitch, from the bottom left corner.
    pub start: GridCell,
    pub stitch_corner: StartingStitchCorner,
}

impl HalfStitch {
    pub fn get_end_location(&self) -> GridCell {
        let origin = self.start - self.stitch_corner.get_offset_from_bottom_left();
        origin
            + self
                .stitch_corner
                .get_opposite_corner()
                .get_offset_from_bottom_left()
    }

    pub fn make_path_stroke(&self) -> Path {
        let first_corner = self.start;
        let second_corner = self.get_end_location();
        Path::line(Point::from(first_corner), Point::from(second_corner))
    }

    pub fn convert_grid_cells<'a>(
        cells: impl Iterator<Item = &'a GridCell>,
        first_stitch_direction: StartingStitchCorner,
        second_stitch_direction: StartingStitchCorner,
    ) -> Vec<HalfStitch> {
        let mut seen_cells = HashMap::new();
        let mut out = Vec::new();
        for cell in cells {
            match seen_cells.contains_key(cell) {
                false => {
                    out.push(HalfStitch {
                        start: *cell + first_stitch_direction.get_offset_from_bottom_left(),
                        stitch_corner: first_stitch_direction,
                    });
                    seen_cells.insert(cell, true);
                }
                true => {
                    out.push(HalfStitch {
                        start: *cell + second_stitch_direction.get_offset_from_bottom_left(),
                        stitch_corner: second_stitch_direction,
                    });
                }
            }
        }
        out
    }

    pub fn check_valid_sequence_float(
        stitches: &[HalfStitch],
    ) -> Result<String, (GridCell, GridCell)> {
        Self::_check_valid_sequence(stitches)?;
        Ok(format!(
            "{:.4}",
            HalfStitch::_calculate_cost_float(stitches)
        ))
    }

    pub fn check_valid_sequence_symbolic(
        stitches: &[HalfStitch],
    ) -> Result<String, (GridCell, GridCell)> {
        Self::_check_valid_sequence(stitches)?;
        Ok(HalfStitch::_calculate_cost_symbolic(stitches).to_string())
    }

    fn _check_valid_sequence(stitches: &[HalfStitch]) -> Result<(), (GridCell, GridCell)> {
        let mut last_stitch: Option<&HalfStitch> = None;
        for stitch in stitches {
            match last_stitch {
                None => {}
                Some(&last) => {
                    if last.get_end_location() == stitch.start {
                        return Err((last.start, stitch.start));
                    }
                }
            }
            last_stitch = Some(stitch);
        }
        Ok(())
    }

    /// Calculate the total cost of the sequence of half-stitches.
    /// This is in units, where one unit is the distance between cells.
    /// It does not include the length of the actual stitch, just distance on the 'back'.
    /// Calculated as a float.
    fn _calculate_cost_float(stitches: &[HalfStitch]) -> f64 {
        let mut total = 0.0;
        for stitch in stitches.windows(2) {
            let first_point = stitch[0].get_end_location();
            let second_point = stitch[1].start;
            total += first_point.euclidean_distance(&second_point);
        }
        total
    }

    fn _calculate_cost_symbolic(stitches: &[HalfStitch]) -> SymbolicSum {
        let mut distance = SymbolicSum::default();
        for stitch in stitches.windows(2) {
            distance.add_distance(stitch[0].get_end_location(), stitch[1].start)
        }
        distance
    }
}

#[cfg(test)]
mod test {
    use super::*;

    fn _round_float(number: f64) -> f64 {
        (number * 1000.0).round() / 1000.0
    }

    #[test]
    fn test_get_end_bottom_left() {
        let result = HalfStitch {
            start: GridCell { x: 0, y: 0 },
            stitch_corner: StartingStitchCorner::BottomLeft,
        }
        .get_end_location();
        assert_eq!(result, GridCell { x: 1, y: 1 });
    }

    #[test]
    fn test_get_end_bottom_right() {
        let result = HalfStitch {
            start: GridCell { x: 0, y: 0 },
            stitch_corner: StartingStitchCorner::BottomRight,
        }
        .get_end_location();
        assert_eq!(result, GridCell { x: -1, y: 1 });
    }

    #[test]
    fn test_get_end_bottom_left_2() {
        let result = HalfStitch {
            start: GridCell { x: 1, y: 0 },
            stitch_corner: StartingStitchCorner::BottomRight,
        }
        .get_end_location();
        assert_eq!(result, GridCell { x: 0, y: 1 });
    }

    #[test]
    fn test_convert_grid_cells_single_cell() {
        let result = HalfStitch::convert_grid_cells(
            [GridCell { x: 0, y: 0 }].iter(),
            StartingStitchCorner::BottomLeft,
            StartingStitchCorner::BottomRight,
        );
        assert_eq!(
            result[0],
            HalfStitch {
                start: GridCell { x: 0, y: 0 },
                stitch_corner: StartingStitchCorner::BottomLeft,
            }
        )
    }

    #[test]
    fn test_convert_grid_cells_doubled_cells() {
        let result = HalfStitch::convert_grid_cells(
            [GridCell { x: 0, y: 0 }, GridCell { x: 0, y: 0 }].iter(),
            StartingStitchCorner::BottomLeft,
            StartingStitchCorner::BottomRight,
        );
        assert_eq!(
            result,
            vec![
                HalfStitch {
                    start: GridCell { x: 0, y: 0 },
                    stitch_corner: StartingStitchCorner::BottomLeft,
                },
                HalfStitch {
                    start: GridCell { x: 1, y: 0 },
                    stitch_corner: StartingStitchCorner::BottomRight,
                },
            ]
        )
    }

    #[test]
    fn test_convert_grid_cells_full_then_half() {
        let result = HalfStitch::convert_grid_cells(
            [
                GridCell { x: 0, y: 0 },
                GridCell { x: 0, y: 0 },
                GridCell { x: 1, y: 0 },
            ]
            .iter(),
            StartingStitchCorner::BottomLeft,
            StartingStitchCorner::BottomRight,
        );
        assert_eq!(
            result,
            vec![
                HalfStitch {
                    start: GridCell { x: 0, y: 0 },
                    stitch_corner: StartingStitchCorner::BottomLeft,
                },
                HalfStitch {
                    start: GridCell { x: 1, y: 0 },
                    stitch_corner: StartingStitchCorner::BottomRight,
                },
                HalfStitch {
                    start: GridCell { x: 1, y: 0 },
                    stitch_corner: StartingStitchCorner::BottomLeft,
                },
            ]
        )
    }

    /// The distance of a single full stitch on a single full cell.
    #[test]
    fn test_stitch_distance_one_full_stitch() {
        let stitches = [GridCell { x: 0, y: 0 }, GridCell { x: 0, y: 0 }];
        let result = HalfStitch::_calculate_cost_float(&HalfStitch::convert_grid_cells(
            stitches.iter(),
            StartingStitchCorner::BottomLeft,
            StartingStitchCorner::BottomRight,
        ));
        assert_eq!(_round_float(result), 1.0);
    }

    /// The distance of two half-stitches in sequence from left to right.
    #[test]
    fn test_stitch_distance_two_consecutive_half_stitches() {
        let stitches = [GridCell { x: 0, y: 0 }, GridCell { x: 1, y: 0 }];
        let result = HalfStitch::_calculate_cost_float(&HalfStitch::convert_grid_cells(
            stitches.iter(),
            StartingStitchCorner::BottomLeft,
            StartingStitchCorner::BottomRight,
        ));
        assert_eq!(_round_float(result), 1.0);
    }

    /// The distance of three half-stitches in sequence from left to right.
    #[test]
    fn test_stitch_distance_three_consecutive_half_stitches() {
        let stitches = [
            GridCell { x: 0, y: 0 },
            GridCell { x: 1, y: 0 },
            GridCell { x: 2, y: 0 },
        ];
        let result = HalfStitch::_calculate_cost_float(&HalfStitch::convert_grid_cells(
            stitches.iter(),
            StartingStitchCorner::BottomLeft,
            StartingStitchCorner::BottomRight,
        ));
        assert_eq!(_round_float(result), 2.0);
    }

    /// The distance of one full stitch then beginning the next half-stitch to the right.
    #[test]
    fn test_stitch_distance_full_then_half() {
        let stitches = [
            GridCell { x: 0, y: 0 },
            GridCell { x: 0, y: 0 },
            GridCell { x: 1, y: 0 },
        ];
        let result = HalfStitch::_calculate_cost_float(&HalfStitch::convert_grid_cells(
            stitches.iter(),
            StartingStitchCorner::BottomLeft,
            StartingStitchCorner::BottomRight,
        ));
        assert_eq!(_round_float(result), 2.414);
    }

    /// The distance of one full stitch then beginning the next half-stitch to the up and right
    #[test]
    fn test_stitch_distance_full_then_half_up() {
        let stitches = [
            GridCell { x: 0, y: 0 },
            GridCell { x: 0, y: 0 },
            GridCell { x: 1, y: 1 },
        ];
        let result = HalfStitch::_calculate_cost_float(&HalfStitch::convert_grid_cells(
            stitches.iter(),
            StartingStitchCorner::BottomLeft,
            StartingStitchCorner::BottomRight,
        ));
        assert_eq!(_round_float(result), 2.0);
    }

    /// The distance of two half-stitches in a column.
    #[test]
    fn test_stitch_distance_two_half_stitches_column_up() {
        let stitches = [GridCell { x: 0, y: 0 }, GridCell { x: 0, y: 1 }];
        let result = HalfStitch::_calculate_cost_float(&HalfStitch::convert_grid_cells(
            stitches.iter(),
            StartingStitchCorner::BottomLeft,
            StartingStitchCorner::BottomRight,
        ));
        assert_eq!(_round_float(result), 1.0);
    }

    /// The distance of two half-stitches in a column.
    #[test]
    fn test_stitch_distance_two_half_stitches_column_down() {
        let stitches = [GridCell { x: 0, y: 0 }, GridCell { x: 0, y: -1 }];
        let result = HalfStitch::_calculate_cost_float(&HalfStitch::convert_grid_cells(
            stitches.iter(),
            StartingStitchCorner::BottomLeft,
            StartingStitchCorner::BottomRight,
        ));
        assert_eq!(_round_float(result), 2.236);
    }

    #[test]
    fn test_get_opposite_corner_from_bottom_left() {
        let result = StartingStitchCorner::BottomLeft.get_opposite_corner();
        assert_eq!(result, StartingStitchCorner::TopRight);
    }

    #[test]
    fn test_get_opposite_corner_from_bottom_right() {
        let result = StartingStitchCorner::BottomRight.get_opposite_corner();
        assert_eq!(result, StartingStitchCorner::TopLeft);
    }

    #[test]
    fn test_get_alternate_corners_from_bottom_left() {
        let result = StartingStitchCorner::BottomLeft.get_possible_top_stitch_corners();
        assert_eq!(
            result,
            [
                StartingStitchCorner::TopLeft,
                StartingStitchCorner::BottomRight
            ]
        );
    }
}
