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
        self.0.euclidean_distance(&self.1).floor() as usize
    }

    /// Determines if two LineSegments overlap.
    pub fn overlaps(&self, other: &LineSegment) -> bool {
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

impl From<(GridCell, GridCell)> for LineSegment {
    fn from((start, end): (GridCell, GridCell)) -> Self {
        LineSegment(start, end)
    }
}

#[cfg(test)]
mod test {
    use super::*;

    fn assert_commutative<T>(a: T, b: T, function: Box<dyn Fn(&T, &T) -> bool>, expected: bool) {
        assert_eq!(function(&a, &b), expected);
        assert_eq!(function(&b, &a), expected);
    }

    #[test]
    fn test_contains_segment_far_disjoint() {
        let first_segment = LineSegment::new(GridCell::new(0, 0), GridCell::new(0, 1));
        let second_segment = LineSegment::new(GridCell::new(1, 2), GridCell::new(1, 3));
        assert_commutative(
            first_segment,
            second_segment,
            Box::from(LineSegment::overlaps),
            false,
        );
    }

    #[test]
    fn test_contains_segment_corner_touching_orthogonal() {
        let first_segment = LineSegment::new(GridCell::new(0, 0), GridCell::new(0, 1));
        let second_segment = LineSegment::new(GridCell::new(0, 0), GridCell::new(1, 0));
        assert_commutative(
            first_segment,
            second_segment,
            Box::from(LineSegment::overlaps),
            false,
        );
    }

    #[test]
    fn test_contains_segment_direct_overlap() {
        let first_segment = LineSegment::new(GridCell::new(0, 0), GridCell::new(0, 1));
        let second_segment = LineSegment::new(GridCell::new(0, 0), GridCell::new(0, 1));
        assert_commutative(
            first_segment,
            second_segment,
            Box::from(LineSegment::overlaps),
            true,
        );
    }

    #[test]
    fn test_contains_segment_partial_overlap_inside() {
        let first_segment = LineSegment::new(GridCell::new(0, 0), GridCell::new(0, 5));
        let second_segment = LineSegment::new(GridCell::new(0, 0), GridCell::new(0, 1));
        assert_commutative(
            first_segment,
            second_segment,
            Box::from(LineSegment::overlaps),
            true,
        );
    }

    #[test]
    fn test_contains_segment_partial_overlap_outside() {
        let first_segment = LineSegment::new(GridCell::new(0, 0), GridCell::new(0, 5));
        let second_segment = LineSegment::new(GridCell::new(0, 4), GridCell::new(0, 8));
        assert_commutative(
            first_segment,
            second_segment,
            Box::from(LineSegment::overlaps),
            true,
        );
    }

    #[test]
    fn test_contains_segment_end_touching_no_overlap() {
        let first_segment = LineSegment::new(GridCell::new(0, 0), GridCell::new(0, 1));
        let second_segment = LineSegment::new(GridCell::new(0, 1), GridCell::new(0, 2));
        assert_commutative(
            first_segment,
            second_segment,
            Box::from(LineSegment::overlaps),
            false,
        );
    }

    #[test]
    fn test_contains_segment_smaller_overlap_larger() {
        let first_segment = LineSegment::new(GridCell::new(0, 0), GridCell::new(0, 2));
        let second_segment = LineSegment::new(GridCell::new(0, 0), GridCell::new(0, 1));
        assert_commutative(
            first_segment,
            second_segment,
            Box::from(LineSegment::overlaps),
            true,
        );
    }
}
