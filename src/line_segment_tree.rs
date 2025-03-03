use crate::grid_cell::GridCell;
use crate::line_segment::LineSegment;
use std::collections::HashSet;

struct LineSegmentTreeNode {
    line_segment: LineSegment,
    children: Vec<LineSegmentTreeNode>,
}

pub fn group_lines_by_segments(
    lines: Vec<(GridCell, GridCell)>,
) -> HashSet<Vec<(GridCell, GridCell)>> {
    todo!()
}
