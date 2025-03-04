use crate::grid_cell::GridCell;
use crate::line_segment::LineSegment;
use itertools::Itertools;

#[derive(Debug, Clone)]
pub struct LineSegmentTreeNode {
    line_segment: LineSegment,
    children: Vec<LineSegmentTreeNode>,
}

impl LineSegmentTreeNode {
    pub fn new(line_segment: LineSegment) -> Self {
        Self {
            line_segment,
            children: vec![],
        }
    }
    pub fn add_child(&mut self, child: LineSegment) {
        Self::insert_segment(&mut self.children, child);
    }

    fn insert_segment(children: &mut Vec<LineSegmentTreeNode>, child: LineSegment) {
        // If there is a containing node, add to as a child.
        if let Some(parent_node) = children
            .iter_mut()
            .find(|node| node.line_segment.overlaps(&child))
        {
            parent_node.add_child(child);
        } else {
            // There is no containing node, so we must find the best place to put the child.
            // It should be in order of size,
            // and we add to the series of same-sized elements at the end
            // to avoid moving a lot of elements down.
            let new_node = LineSegmentTreeNode::new(child);

            match children.binary_search_by(|node| {
                // For descending order, reverse the comparison.
                node.line_segment
                    .get_length()
                    .cmp(&new_node.line_segment.get_length())
            }) {
                Ok(pos) | Err(pos) => children.insert(pos, new_node),
            }
        }
    }
}

#[derive(Debug, Clone)]
pub struct LineSegmentTree {
    pub root_nodes: Vec<LineSegmentTreeNode>,
}

impl LineSegmentTree {
    pub fn new() -> Self {
        Self { root_nodes: vec![] }
    }

    pub fn add_child(&mut self, line_segment: LineSegment) {
        if let Some(parent_node) = self
            .root_nodes
            .iter_mut()
            .find(|node| node.line_segment.overlaps(&line_segment))
        {
            parent_node.add_child(line_segment);
        } else {
            let new_node = LineSegmentTreeNode::new(line_segment);
            self.root_nodes.push(new_node);
        }
    }
}

pub fn group_lines(lines: Vec<(GridCell, GridCell)>) -> LineSegmentTree {
    let mut tree = LineSegmentTree::new();
    for segment in lines
        .into_iter()
        .map(|(start, end)| LineSegment::new(start, end))
        .sorted_by_key(|s| s.get_length())
        .rev()
    {
        tree.add_child(segment);
    }
    tree
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_group_lines_by_segment_no_overlapping() {
        let segments = vec![
            (GridCell::new(0, 0), GridCell::new(0, 1)),
            (GridCell::new(0, 0), GridCell::new(1, 0)),
        ];
        let result = group_lines(segments);
        assert_eq!(result.root_nodes.len(), 2)
    }

    #[test]
    fn test_group_lines_by_segment_one_overlapping_one_not() {
        let segments = vec![
            (GridCell::new(0, 0), GridCell::new(0, 1)),
            (GridCell::new(0, 0), GridCell::new(0, 2)),
            (GridCell::new(0, 0), GridCell::new(1, 0)),
        ];
        let result = group_lines(segments);
        assert_eq!(result.root_nodes.len(), 2)
    }

    #[test]
    fn test_group_lines_by_segment_two_overlapping_groups() {
        let segments = vec![
            (GridCell::new(0, 0), GridCell::new(0, 1)),
            (GridCell::new(0, 0), GridCell::new(0, 2)),
            (GridCell::new(0, 0), GridCell::new(0, 3)),
            (GridCell::new(2, 0), GridCell::new(0, 3)),
            (GridCell::new(0, 0), GridCell::new(1, 0)),
            (GridCell::new(1, 0), GridCell::new(2, 0)),
        ];
        let result = group_lines(segments);
        assert_eq!(result.root_nodes.len(), 2)
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
        let result = group_lines(segments);
        assert_eq!(result.root_nodes.len(), 2)
    }
}
