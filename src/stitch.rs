use crate::grid::GridCell;
use crate::symbolic_sum::SymbolicSum;
use iced::widget::canvas::Path;
use iced::Point;
use std::collections::HashMap;

#[derive(PartialEq, Eq, Copy, Clone, Debug, Hash)]
pub struct HalfStitch {
    // The start is the cell of the stitch, from the bottom left corner.
    pub start: GridCell,
    pub facing_right: bool,
}

impl HalfStitch {
    pub fn get_end_location(&self) -> GridCell {
        match self.facing_right {
            true => GridCell {
                x: self.start.x + 1,
                y: self.start.y + 1,
            },
            false => GridCell {
                x: self.start.x - 1,
                y: self.start.y + 1,
            },
        }
    }

    pub fn make_path_stroke(&self) -> Path {
        if self.facing_right {
            Path::line(
                Point {
                    x: self.start.x as f32,
                    y: self.start.y as f32,
                },
                Point {
                    x: (self.start.x + 1) as f32,
                    y: (self.start.y + 1) as f32,
                },
            )
        } else {
            Path::line(
                Point {
                    x: self.start.x as f32,
                    y: self.start.y as f32,
                },
                Point {
                    x: (self.start.x - 1) as f32,
                    y: (self.start.y + 1) as f32,
                },
            )
        }
    }

    pub fn convert_grid_cells<'a>(cells: impl Iterator<Item = &'a GridCell>) -> Vec<HalfStitch> {
        let mut seen_cells = HashMap::new();
        let mut out = Vec::new();
        for cell in cells {
            match seen_cells.contains_key(cell) {
                false => {
                    out.push(HalfStitch {
                        start: *cell,
                        facing_right: true,
                    });
                    seen_cells.insert(cell, true);
                }
                true => {
                    out.push(HalfStitch {
                        start: GridCell {
                            x: cell.x + 1,
                            y: cell.y,
                        },
                        facing_right: false,
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
    fn test_get_end_facing_right() {
        let result = HalfStitch {
            start: GridCell { x: 0, y: 0 },
            facing_right: true,
        }
        .get_end_location();
        assert_eq!(result, GridCell { x: 1, y: 1 });
    }

    #[test]
    fn test_get_end_facing_left() {
        let result = HalfStitch {
            start: GridCell { x: 0, y: 0 },
            facing_right: false,
        }
        .get_end_location();
        assert_eq!(result, GridCell { x: -1, y: 1 });
    }

    #[test]
    fn test_get_end_facing_left_2() {
        let result = HalfStitch {
            start: GridCell { x: 1, y: 0 },
            facing_right: false,
        }
        .get_end_location();
        assert_eq!(result, GridCell { x: 0, y: 1 });
    }

    #[test]
    fn test_convert_grid_cells_single_cell() {
        let result = HalfStitch::convert_grid_cells([GridCell { x: 0, y: 0 }].iter());
        assert_eq!(
            result[0],
            HalfStitch {
                start: GridCell { x: 0, y: 0 },
                facing_right: true
            }
        )
    }

    #[test]
    fn test_convert_grid_cells_doubled_cells() {
        let result = HalfStitch::convert_grid_cells(
            [GridCell { x: 0, y: 0 }, GridCell { x: 0, y: 0 }].iter(),
        );
        assert_eq!(
            result,
            vec![
                HalfStitch {
                    start: GridCell { x: 0, y: 0 },
                    facing_right: true
                },
                HalfStitch {
                    start: GridCell { x: 1, y: 0 },
                    facing_right: false
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
        );
        assert_eq!(
            result,
            vec![
                HalfStitch {
                    start: GridCell { x: 0, y: 0 },
                    facing_right: true
                },
                HalfStitch {
                    start: GridCell { x: 1, y: 0 },
                    facing_right: false
                },
                HalfStitch {
                    start: GridCell { x: 1, y: 0 },
                    facing_right: true
                },
            ]
        )
    }

    /// The distance of a single full stitch on a single full cell.
    #[test]
    fn test_stitch_distance_one_full_stitch() {
        let stitches = [GridCell { x: 0, y: 0 }, GridCell { x: 0, y: 0 }];
        let result =
            HalfStitch::_calculate_cost_float(&HalfStitch::convert_grid_cells(stitches.iter()));
        assert_eq!(_round_float(result), 1.0);
    }

    /// The distance of two half-stitches in sequence from left to right.
    #[test]
    fn test_stitch_distance_two_consecutive_half_stitches() {
        let stitches = [GridCell { x: 0, y: 0 }, GridCell { x: 1, y: 0 }];
        let result =
            HalfStitch::_calculate_cost_float(&HalfStitch::convert_grid_cells(stitches.iter()));
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
        let result =
            HalfStitch::_calculate_cost_float(&HalfStitch::convert_grid_cells(stitches.iter()));
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
        let result =
            HalfStitch::_calculate_cost_float(&HalfStitch::convert_grid_cells(stitches.iter()));
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
        let result =
            HalfStitch::_calculate_cost_float(&HalfStitch::convert_grid_cells(stitches.iter()));
        assert_eq!(_round_float(result), 2.0);
    }

    /// The distance of two half-stitches in a column.
    #[test]
    fn test_stitch_distance_two_half_stitches_column_up() {
        let stitches = [GridCell { x: 0, y: 0 }, GridCell { x: 0, y: 1 }];
        let result =
            HalfStitch::_calculate_cost_float(&HalfStitch::convert_grid_cells(stitches.iter()));
        assert_eq!(_round_float(result), 1.0);
    }

    /// The distance of two half-stitches in a column.
    #[test]
    fn test_stitch_distance_two_half_stitches_column_down() {
        let stitches = [GridCell { x: 0, y: 0 }, GridCell { x: 0, y: -1 }];
        let result =
            HalfStitch::_calculate_cost_float(&HalfStitch::convert_grid_cells(stitches.iter()));
        assert_eq!(_round_float(result), 2.236);
    }
}
