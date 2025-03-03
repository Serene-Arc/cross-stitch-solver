use crate::grid_cell::GridCell;
use std::cmp::{max, min};

/// A struct for working with lines that are orthogonal to a grid i.e. straight between grid points.
#[derive(Copy, Clone, Debug, Hash, Eq, PartialEq)]
pub struct LineSegment(GridCell, GridCell);

#[derive(Debug, PartialEq, Eq)]
enum Axis {
    Horizontal,
    Vertical,
}

impl LineSegment {
    pub fn new(start: GridCell, end: GridCell) -> Self {
        Self(start, end)
    }

    pub fn get_length(&self) -> usize {
        ((self.1.x - self.0.x).abs() + (self.1.y - self.0.y).abs()) as usize
    }

    /// Determines if two LineSegments overlap.
    pub fn contains_segment(&self, other: &LineSegment) -> bool {
        // Determine if both segments are horizontal or vertical
        let self_orientation = self.orientation();
        let other_orientation = other.orientation();

        match (self_orientation, other_orientation) {
            (Some(self_dir), Some(other_dir)) => {
                // We don't consider lines of different orientations to be overlapping.
                if self_dir != other_dir {
                    return false;
                }

                if self_dir == Axis::Horizontal {
                    // Check if they are on the same y-coordinate
                    if self.0.y != other.0.y {
                        return false;
                    }
                    // Check if their x ranges overlap
                    let (self_min_x, self_max_x) =
                        (min(self.0.x, self.1.x), max(self.0.x, self.1.x));
                    let (other_min_x, other_max_x) =
                        (min(other.0.x, other.1.x), max(other.0.x, other.1.x));
                    max(self_min_x, other_min_x) < min(self_max_x, other_max_x)
                } else {
                    // Check if they are on the same x-coordinate
                    if self.0.x != other.0.x {
                        return false;
                    }
                    // Check if their y ranges overlap
                    let (self_min_y, self_max_y) =
                        (min(self.0.y, self.1.y), max(self.0.y, self.1.y));
                    let (other_min_y, other_max_y) =
                        (min(other.0.y, other.1.y), max(other.0.y, other.1.y));
                    max(self_min_y, other_min_y) < min(self_max_y, other_max_y)
                }
            }
            _ => false, // One or both segments are not strictly horizontal or vertical
        }
    }

    /// Determines the orientation of a line segment.
    fn orientation(&self) -> Option<Axis> {
        if self.0.y == self.1.y {
            Some(Axis::Horizontal)
        } else if self.0.x == self.1.x {
            Some(Axis::Vertical)
        } else {
            None
        }
    }
}

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn test_contains_segment_far_disjoint() {
        let first_segment = LineSegment::new(GridCell::new(0, 0), GridCell::new(0, 1));
        let second_segment = LineSegment::new(GridCell::new(1, 2), GridCell::new(1, 3));
        let first_result = first_segment.contains_segment(&second_segment);
        let second_result = second_segment.contains_segment(&first_segment);
        assert_eq!(first_result, false);
        assert_eq!(second_result, false);
    }

    #[test]
    fn test_contains_segment_corner_touching_orthogonal() {
        let first_segment = LineSegment::new(GridCell::new(0, 0), GridCell::new(0, 1));
        let second_segment = LineSegment::new(GridCell::new(0, 0), GridCell::new(1, 0));
        let first_result = first_segment.contains_segment(&second_segment);
        let second_result = second_segment.contains_segment(&first_segment);
        assert_eq!(first_result, false);
        assert_eq!(second_result, false);
    }

    #[test]
    fn test_contains_segment_direct_overlap() {
        let result = LineSegment::new(GridCell::new(0, 0), GridCell::new(0, 1))
            .contains_segment(&LineSegment::new(GridCell::new(0, 0), GridCell::new(0, 1)));
        assert_eq!(result, true);
    }

    #[test]
    fn test_contains_segment_partial_overlap_inside() {
        let first_segment = LineSegment::new(GridCell::new(0, 0), GridCell::new(0, 5));
        let second_segment = LineSegment::new(GridCell::new(0, 0), GridCell::new(0, 1));
        let first_result = first_segment.contains_segment(&second_segment);
        let second_result = second_segment.contains_segment(&first_segment);
        assert_eq!(first_result, true);
        assert_eq!(second_result, true);
    }

    #[test]
    fn test_contains_segment_partial_overlap_outside() {
        let first_segment = LineSegment::new(GridCell::new(0, 0), GridCell::new(0, 5));
        let second_segment = LineSegment::new(GridCell::new(0, 4), GridCell::new(0, 8));
        let first_result = first_segment.contains_segment(&second_segment);
        let second_result = second_segment.contains_segment(&first_segment);
        assert_eq!(first_result, true);
        assert_eq!(second_result, true);
    }

    #[test]
    fn test_contains_segment_end_touching_no_overlap() {
        let first_segment = LineSegment::new(GridCell::new(0, 0), GridCell::new(0, 1));
        let second_segment = LineSegment::new(GridCell::new(0, 1), GridCell::new(0, 2));
        let first_result = first_segment.contains_segment(&second_segment);
        let second_result = second_segment.contains_segment(&first_segment);
        assert_eq!(first_result, false);
        assert_eq!(second_result, false);
    }
}
