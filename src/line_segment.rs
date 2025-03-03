use crate::grid_cell::GridCell;

#[derive(Copy, Clone, Debug, Hash, Eq, PartialEq)]
pub struct LineSegment(GridCell, GridCell);

impl LineSegment {
    pub fn break_line(start: GridCell, end: GridCell) -> Vec<LineSegment> {
        let mut segments = Vec::new();
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
                segments.push(LineSegment(current_place, current_end));
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

    pub fn get_length(&self) -> usize {
        ((self.1.x - self.0.x).abs() + (self.1.y - self.0.y).abs()) as usize
    }
}

#[cfg(test)]
mod test {
    use super::*;
    use crate::line_segment_tree::group_lines_by_segments;

    #[test]
    fn test_break_line_single_y() {
        let result = LineSegment::break_line(GridCell::new(0, 0), GridCell::new(0, 1));
        assert_eq!(
            result,
            vec![LineSegment(GridCell::new(0, 0), GridCell::new(0, 1))]
        )
    }

    #[test]
    fn test_break_line_single_x() {
        let result = LineSegment::break_line(GridCell::new(0, 0), GridCell::new(1, 0));
        assert_eq!(
            result,
            vec![LineSegment(GridCell::new(0, 0), GridCell::new(1, 0))]
        )
    }

    #[test]
    fn test_break_line_two_x() {
        let result = LineSegment::break_line(GridCell::new(0, 0), GridCell::new(2, 0));
        assert_eq!(
            result,
            vec![
                LineSegment(GridCell::new(0, 0), GridCell::new(1, 0)),
                LineSegment(GridCell::new(0, 0), GridCell::new(2, 0)),
                LineSegment(GridCell::new(1, 0), GridCell::new(2, 0)),
            ]
        )
    }

    #[test]
    fn test_break_line_four_x() {
        let result = LineSegment::break_line(GridCell::new(0, 0), GridCell::new(4, 0));
        assert_eq!(result.len(), 10)
    }

    #[test]
    fn test_break_line_four_y() {
        let result = LineSegment::break_line(GridCell::new(0, 0), GridCell::new(0, 4));
        assert_eq!(result.len(), 10)
    }

    #[test]
    fn test_group_lines_by_segment_no_overlapping() {
        let segments = vec![
            (GridCell::new(0, 0), GridCell::new(0, 1)),
            (GridCell::new(0, 0), GridCell::new(1, 0)),
        ];
        let result = group_lines_by_segments(segments);
        assert_eq!(result.len(), 2)
    }

    #[test]
    fn test_group_lines_by_segment_one_overlapping_one_not() {
        let segments = vec![
            (GridCell::new(0, 0), GridCell::new(0, 1)),
            (GridCell::new(0, 0), GridCell::new(0, 2)),
            (GridCell::new(0, 0), GridCell::new(1, 0)),
        ];
        let result = group_lines_by_segments(segments);
        assert_eq!(result.len(), 2)
    }

    #[test]
    fn test_group_lines_by_segment_two_overlapping() {
        let segments = vec![
            (GridCell::new(0, 0), GridCell::new(0, 1)),
            (GridCell::new(0, 0), GridCell::new(0, 2)),
            (GridCell::new(0, 0), GridCell::new(0, 3)),
            (GridCell::new(2, 0), GridCell::new(0, 3)),
            (GridCell::new(0, 0), GridCell::new(1, 0)),
            (GridCell::new(1, 0), GridCell::new(2, 0)),
        ];
        let result = group_lines_by_segments(segments);
        assert_eq!(result.len(), 2)
    }

    #[test]
    fn test_group_lines_by_segment_two_overlapping_two_not() {
        let segments = vec![
            (GridCell::new(0, 0), GridCell::new(0, 1)),
            (GridCell::new(0, 0), GridCell::new(0, 2)),
            (GridCell::new(0, 0), GridCell::new(0, 3)),
            (GridCell::new(2, 0), GridCell::new(0, 3)),
            (GridCell::new(0, 0), GridCell::new(1, 0)),
        ];
        let result = group_lines_by_segments(segments);
        assert_eq!(result.len(), 2)
    }
}
