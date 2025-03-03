use crate::grid_cell::GridCell;
use cached::proc_macro::cached;
use std::collections::HashSet;

/// A struct for working with lines that are orthogonal to a grid i.e. straight between grid points.
#[derive(Copy, Clone, Debug, Hash, Eq, PartialEq)]
pub struct LineSegment(GridCell, GridCell);

#[cached]
pub fn break_line(start: GridCell, end: GridCell) -> HashSet<LineSegment> {
    let mut segments = HashSet::new();
    let dx = end.x - start.x;
    let dy = end.y - start.y;
    let offset = if dx > 0 {
        GridCell::new(1, 0)
    } else if dx < 0 {
        GridCell::new(-1, 0)
    } else if dy > 0 {
        GridCell::new(0, 1)
    } else if dy < 0 {
        GridCell::new(0, -1)
    } else {
        panic!("Offset could not be calculated")
    };
    let mut current_place = start;
    let mut current_end = start + offset;
    loop {
        if current_place == end {
            break;
        }
        loop {
            segments.insert(LineSegment(current_place, current_end));
            if current_end == end {
                break;
            }
            current_end = current_end + offset;
        }
        current_place = current_place + offset;
        current_end = current_place + offset;
    }
    segments
}

impl LineSegment {
    pub fn new(start: GridCell, end: GridCell) -> Self {
        Self(start, end)
    }

    pub fn get_length(&self) -> usize {
        ((self.1.x - self.0.x).abs() + (self.1.y - self.0.y).abs()) as usize
    }

    pub fn contains_segment(&self, segment: &LineSegment) -> bool {
        let sub_segments = break_line(self.0, self.1);
        !sub_segments.is_disjoint(&break_line(segment.0, segment.1))
    }
}

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn test_break_line_single_y() {
        let result = break_line(GridCell::new(0, 0), GridCell::new(0, 1));
        assert_eq!(
            result,
            HashSet::from([LineSegment(GridCell::new(0, 0), GridCell::new(0, 1))])
        )
    }

    #[test]
    fn test_break_line_single_x() {
        let result = break_line(GridCell::new(0, 0), GridCell::new(1, 0));
        assert_eq!(
            result,
            HashSet::from([LineSegment(GridCell::new(0, 0), GridCell::new(1, 0))])
        )
    }

    #[test]
    fn test_break_line_two_x() {
        let result = break_line(GridCell::new(0, 0), GridCell::new(2, 0));
        assert_eq!(
            result,
            HashSet::from([
                LineSegment(GridCell::new(0, 0), GridCell::new(1, 0)),
                LineSegment(GridCell::new(0, 0), GridCell::new(2, 0)),
                LineSegment(GridCell::new(1, 0), GridCell::new(2, 0)),
            ])
        )
    }

    #[test]
    fn test_break_line_four_x() {
        let result = break_line(GridCell::new(0, 0), GridCell::new(4, 0));
        assert_eq!(result.len(), 10)
    }

    #[test]
    fn test_break_line_four_y() {
        let result = break_line(GridCell::new(0, 0), GridCell::new(0, 4));
        assert_eq!(result.len(), 10)
    }

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
