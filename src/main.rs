mod grid;
mod grid_cell;
mod stitch;
mod symbolic_sum;

use crate::grid::GridState;
use crate::stitch::StartingStitchCorner;
use grid_cell::GridCell;
use iced::widget::{button, checkbox, column, container, pick_list, row};
use iced::{Element, Fill, Task, Theme};
use std::collections::{HashMap, VecDeque};

fn main() -> iced::Result {
    iced::application(
        "Cross Stitch Solver",
        CrossStitchSolver::update,
        CrossStitchSolver::view,
    )
    .theme(|_| Theme::Dark)
    .antialiasing(true)
    .centered()
    .run()
}

#[derive(Debug, Clone)]
pub enum Message {
    Grid(grid::Message),
    ClearGrid,
    ChangeCalculationSpecificity(bool),
    ChangeBottomStitchCorner(StartingStitchCorner),
    ChangeTopStitchCorner(StartingStitchCorner),
}

#[derive(Debug, Default)]
struct CrossStitchSolver {
    grid_state: GridState,
}

impl CrossStitchSolver {
    fn update(&mut self, message: Message) -> Task<Message> {
        match message {
            Message::Grid(message) => {
                self.grid_state.update(message);
            }
            Message::ClearGrid => self.grid_state.clear(),
            Message::ChangeCalculationSpecificity(check_box) => {
                self.grid_state.precise_cost = check_box;
            }
            Message::ChangeBottomStitchCorner(first_stitch_corner) => {
                self.grid_state.bottom_stitch_corner = first_stitch_corner;
                self.grid_state.top_stitch_corner =
                    first_stitch_corner.get_possible_top_stitch_corners()[0];
                self.grid_state.clear_cache();
            }
            Message::ChangeTopStitchCorner(second_stitch_corner) => {
                self.grid_state.top_stitch_corner = second_stitch_corner;
                self.grid_state.clear_cache();
            }
        }
        Task::none()
    }
    fn view(&self) -> Element<Message> {
        let bottom_stitch_directions = [
            StartingStitchCorner::BottomLeft,
            StartingStitchCorner::BottomRight,
            StartingStitchCorner::TopLeft,
            StartingStitchCorner::TopRight,
        ];
        let content = column![
            self.grid_state.view().map(Message::Grid),
            button("Clear")
                .on_press(Message::ClearGrid)
                .style(button::danger),
            checkbox("Precise Cost", self.grid_state.precise_cost)
                .on_toggle(Message::ChangeCalculationSpecificity),
            row![
                "Bottom Stitch Start Corner: ",
                pick_list(
                    bottom_stitch_directions,
                    Some(&self.grid_state.bottom_stitch_corner),
                    Message::ChangeBottomStitchCorner
                ),
                "Top Stitch Start Corner: ",
                pick_list(
                    self.grid_state
                        .bottom_stitch_corner
                        .get_possible_top_stitch_corners(),
                    Some(&self.grid_state.top_stitch_corner),
                    Message::ChangeTopStitchCorner
                ),
            ]
            .spacing(5)
            .width(Fill),
        ]
        .height(Fill);

        container(content).width(Fill).height(Fill).into()
    }
}

#[derive(Debug, Clone, Default)]
struct ProgramState {
    pub selected_cells: VecDeque<GridCell>,
    cell_counts: HashMap<GridCell, usize>,
}

impl ProgramState {
    fn select_cell(&mut self, cell: GridCell) {
        match self.cell_counts.get(&cell).unwrap_or(&0) {
            0 => {
                self.cell_counts.insert(cell, 1);
                self.selected_cells.push_back(cell);
            }
            1 => {
                self.cell_counts.insert(cell, 2);
                self.selected_cells.push_back(cell);
            }
            _ => {}
        }
    }

    fn unselect_cell(&mut self, cell: GridCell) {
        match self.cell_counts.get(&cell).unwrap_or(&0) {
            1 => {
                self.cell_counts.remove(&cell);
                let first_position = self
                    .selected_cells
                    .iter()
                    .position(|&x| x == cell)
                    .unwrap_or_else(|| {
                        panic!("Cell {:?} in cell count map but not in vector ", cell)
                    });
                self.selected_cells.remove(first_position);
            }
            2 => {
                self.cell_counts.insert(cell, 1);
                self._remove_last_cell_in_vec(cell);
            }
            _ => {}
        }
    }
    fn _remove_last_cell_in_vec(&mut self, cell: GridCell) {
        let reversed_position = self
            .selected_cells
            .iter()
            .rev()
            .position(|&x| x == cell)
            .unwrap();
        let real_position = self.selected_cells.len() - reversed_position - 1;
        self.selected_cells.remove(real_position);
    }
    pub fn clear(&mut self) {
        self.selected_cells.clear();
        self.cell_counts.clear();
    }
}
