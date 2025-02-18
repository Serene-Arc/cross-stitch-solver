use crate::stitch::HalfStitch;
use crate::ProgramState;
use iced::event::Status;
use iced::mouse::Cursor;
use iced::widget::canvas::{Cache, Event, Frame, Geometry, Path, Stroke, Style, Text};
use iced::widget::{canvas, Canvas};
use iced::{
    alignment, mouse, Color, Element, Fill, Font, Point, Rectangle, Renderer, Size, Theme, Vector,
};
use std::ops::RangeInclusive;

#[derive(Debug, Clone)]
pub enum Message {
    Select(GridCell),
    Unselect(GridCell),
    Translated(Vector),
    Scaled(f32),
}

#[derive(Debug)]
pub struct GridState {
    /// Cache for the drawn grid.
    grid_cache: Cache,

    /// Cache for the selected cells and stitch markings.
    cell_cache: Cache,

    /// Offset for the view of the screen from the origin.
    translation: Vector,

    /// Scaling factor for the view.
    scaling: f32,
    pub program_state: ProgramState,

    /// Bool for whether to display the cost in precise mathematical terms.
    pub precise_cost: bool,
}

impl Default for GridState {
    fn default() -> Self {
        Self {
            grid_cache: Cache::default(),
            cell_cache: Cache::default(),
            translation: Default::default(),
            scaling: 2.0,
            program_state: Default::default(),
            precise_cost: false,
        }
    }
}

impl GridState {
    const MIN_SCALING: f32 = 0.1;
    const MAX_SCALING: f32 = 4.0;

    /// Determine the region that should be visible.
    fn visible_region(&self, size: Size) -> Region {
        let view_width = size.width / self.scaling;
        let view_height = size.height / self.scaling;

        Region {
            x: -self.translation.x - (view_width / 2.0),
            y: -self.translation.y - (view_height / 2.0),
            width: view_width,
            height: view_height,
        }
    }

    /// Clear everything to return to as-new state.
    pub fn clear(&mut self) {
        self.grid_cache.clear();
        self.cell_cache.clear();
        self.program_state.clear();
    }

    /// Project a given screen coordinate onto the visible region of the grid.
    fn project_screen_to_mathematical_point(
        &self,
        screen_input_position: Point,
        visible_size: Size,
    ) -> Point {
        let region = self.visible_region(visible_size);

        Point::new(
            (screen_input_position.x / self.scaling) + region.x,
            -((screen_input_position.y / self.scaling) + region.y),
        )
    }

    pub fn update(&mut self, message: Message) {
        match message {
            Message::Select(grid_cell) => {
                self.program_state.select_cell(grid_cell);
                self.cell_cache.clear();
            }
            Message::Unselect(grid_cell) => {
                self.program_state.unselect_cell(grid_cell);
                self.cell_cache.clear();
            }
            Message::Translated(translation) => {
                self.translation = translation;

                self.grid_cache.clear();
                self.cell_cache.clear();
            }
            Message::Scaled(scaling) => {
                self.scaling = scaling;

                self.grid_cache.clear();
                self.cell_cache.clear();
            }
        }
    }

    pub fn view(&self) -> Element<Message> {
        Canvas::new(self).width(Fill).height(Fill).into()
    }

    fn transform_frame_for_cells(&self, screen_centre: Vector, frame: &mut Frame<Renderer>) {
        // Order here is necessary for it to work correctly.
        // First translate so the origin is the centre of the screen,
        // then scale so the pan is correct for the scale.
        // Finally, make it so the cells are drawn correctly.
        frame.translate(screen_centre);
        frame.scale(self.scaling);
        frame.translate(self.translation);
        frame.scale(GridCell::SIZE);
    }

    fn make_grid_background(
        &self,
        renderer: &Renderer,
        bounds: Rectangle,
        screen_centre: Vector,
    ) -> Geometry {
        self.grid_cache.draw(renderer, bounds.size(), |frame| {
            self.transform_frame_for_cells(screen_centre, frame);

            let region = self.visible_region(frame.size());
            let rows = region.rows();
            let columns = region.columns();
            let (total_rows, total_columns) = (rows.clone().count(), columns.clone().count());
            let width = 2.0 / GridCell::SIZE as f32;
            let color = Color::from_rgb8(70, 74, 83);

            frame.translate(Vector::new(-width / 2.0, -width / 2.0));

            for row in region.rows() {
                frame.fill_rectangle(
                    Point::new(*columns.start() as f32, row as f32),
                    Size::new(total_columns as f32, width),
                    color,
                );
            }

            for column in region.columns() {
                frame.fill_rectangle(
                    Point::new(column as f32, *rows.start() as f32),
                    Size::new(width, total_rows as f32),
                    color,
                );
            }
        })
    }
}

impl canvas::Program<Message> for GridState {
    type State = GridInteraction;

    fn update(
        &self,
        interaction: &mut Self::State,
        event: Event,
        bounds: Rectangle,
        cursor: Cursor,
    ) -> (Status, Option<Message>) {
        if let Event::Mouse(mouse::Event::ButtonReleased(_)) = event {
            *interaction = GridInteraction::None;
        }
        let screen_cursor_position = match cursor.position_in(bounds) {
            None => {
                return (Status::Ignored, None);
            }
            Some(pos) => pos,
        };

        let cell = GridCell::cell_at_screen_point(
            self.project_screen_to_mathematical_point(screen_cursor_position, bounds.size()),
        );
        match event {
            Event::Mouse(mouse_event) => match mouse_event {
                mouse::Event::ButtonPressed(button) => {
                    let message = match button {
                        mouse::Button::Left => Some(Message::Select(cell)),
                        mouse::Button::Right => Some(Message::Unselect(cell)),
                        mouse::Button::Middle => {
                            *interaction = GridInteraction::Panning {
                                translation: self.translation,
                                origin: screen_cursor_position,
                            };
                            None
                        }
                        _ => None,
                    };
                    (Status::Captured, message)
                }
                mouse::Event::CursorMoved { .. } => {
                    let message = match *interaction {
                        GridInteraction::Panning {
                            translation,
                            origin: pan_origin,
                        } => {
                            let new_vector =
                                (screen_cursor_position - pan_origin) * (1.0 / self.scaling);
                            Some(Message::Translated(translation + new_vector))
                        }
                        GridInteraction::None => None,
                    };

                    (Status::Captured, message)
                }
                mouse::Event::WheelScrolled { delta } => match delta {
                    mouse::ScrollDelta::Lines { y, .. } | mouse::ScrollDelta::Pixels { y, .. } => {
                        if y < 0.0 && self.scaling > Self::MIN_SCALING
                            || y > 0.0 && self.scaling < Self::MAX_SCALING
                        {
                            // Calculate the new scaling.
                            // Note that 30.0 restricts the speed of the zoom.
                            let scaling = (self.scaling * (1.0 + (y / 30.0)))
                                .clamp(Self::MIN_SCALING, Self::MAX_SCALING);

                            let message = Message::Scaled(scaling);
                            (Status::Captured, Some(message))
                        } else {
                            (Status::Ignored, None)
                        }
                    }
                },

                _ => (Status::Ignored, None),
            },
            _ => (Status::Ignored, None),
        }
    }

    fn draw(
        &self,
        _state: &Self::State,
        renderer: &Renderer,
        _theme: &Theme,
        bounds: Rectangle,
        cursor: Cursor,
    ) -> Vec<Geometry<Renderer>> {
        let screen_centre = Vector::new(bounds.width / 2.0, bounds.height / 2.0);

        // Convert the stitches that already exist and check if they're valid,
        // computing the cost as we go.
        let stitches = HalfStitch::convert_grid_cells(self.program_state.selected_cells.iter());
        let valid_sequence = if self.precise_cost {
            HalfStitch::check_valid_sequence_symbolic(&stitches)
        } else {
            HalfStitch::check_valid_sequence_float(&stitches)
        };

        let selected_cells = self.cell_cache.draw(renderer, bounds.size(), |frame| {
            let background = Path::rectangle(Point::ORIGIN, frame.size());
            frame.fill(&background, Color::from_rgb8(0x40, 0x44, 0x4B));

            frame.with_save(|frame| {
                self.transform_frame_for_cells(screen_centre, frame);

                let region = self.visible_region(frame.size());

                frame.scale_nonuniform(Vector { x: 1.0, y: -1.0 });

                for cell in region.cull(self.program_state.selected_cells.iter()) {
                    frame.fill_rectangle(Point::from(cell), Size::UNIT, Color::WHITE);
                }

                // Mark the first pair of invalid stitches, if there are any.
                match &valid_sequence {
                    Ok(_) => {}
                    Err((first, second)) => {
                        for cell in region.cull([*first, *second].iter()) {
                            frame.fill_rectangle(
                                Point::from(cell),
                                Size::UNIT,
                                Color::from_rgb(100.0, 0.0, 0.0),
                            );
                        }
                    }
                }

                let mut alpha = 1.0;
                // Iterate in verse order so we can decrease the opacity for each stitch.
                for stitch in stitches.iter().rev() {
                    let line = stitch.make_path_stroke();
                    let line_stroke = Stroke {
                        width: 5.0,
                        style: Style::Solid(Color {
                            a: alpha,
                            ..Color::BLACK
                        }),
                        ..Default::default()
                    };
                    frame.stroke(&line, line_stroke);
                    if alpha > 0.4 {
                        let reduction = if alpha < 0.95 { 0.05 } else { 0.01 };
                        alpha -= reduction;
                    }
                }
            });
        });

        let cell_highlight = {
            let mut frame = Frame::new(renderer, bounds.size());

            let hovered_grid_cell = cursor.position_in(bounds).map(|position| {
                GridCell::cell_at_screen_point(
                    self.project_screen_to_mathematical_point(position, frame.size()),
                )
            });

            if let Some(cell) = hovered_grid_cell {
                frame.with_save(|frame| {
                    self.transform_frame_for_cells(screen_centre, frame);

                    frame.scale_nonuniform(Vector { x: 1.0, y: -1.0 });
                    frame.fill_rectangle(
                        Point::from(cell),
                        Size::UNIT,
                        Color {
                            a: 0.2,
                            ..Color::BLACK
                        },
                    );
                });
            }

            // Make text for coordinates in the corner
            let text = Text {
                color: Color::WHITE,
                size: 14.0.into(),
                position: Point::new(frame.width(), frame.height()),
                horizontal_alignment: alignment::Horizontal::Right,
                vertical_alignment: alignment::Vertical::Bottom,
                font: Font::MONOSPACE,
                ..Text::default()
            };
            if let Some(cell) = hovered_grid_cell {
                // Since there is a grid cell under the cursor, we know that unwrap will work.
                let cursor = cursor.position_in(bounds).unwrap();
                let math_cursor = self.project_screen_to_mathematical_point(cursor, frame.size());
                frame.fill_text(Text {
                    content: format!(
                        "({}, {}) grid, ({:07.2}, {:07.2}) raw screen, ({:07.2}, {:07.2}) screen",
                        cell.x, cell.y, cursor.x, cursor.y, math_cursor.x, math_cursor.y
                    ),
                    position: text.position - Vector::new(0.0, 32.0),
                    ..text
                });

                let visible_region = self.visible_region(frame.size());
                frame.fill_text(Text {
                    content: format!(
                        "Visible Area: columns {} to {}, rows {} to {}",
                        visible_region.columns().start(),
                        visible_region.columns().end(),
                        visible_region.rows().start(),
                        visible_region.rows().end(),
                    ),
                    position: text.position - Vector::new(0.0, 16.0),
                    ..text
                });
                let cell_count = self.program_state.selected_cells.len();

                frame.fill_text(Text {
                    content: format!(
                        "{cell_count} cell{} @ {}",
                        if cell_count == 1 { "" } else { "s" },
                        if valid_sequence.is_ok() {
                            format!("{} distance", valid_sequence.unwrap())
                        } else {
                            "invalid sequence".to_string()
                        },
                    ),
                    ..text
                });
            }

            frame.into_geometry()
        };

        // Make the grid for the cells
        let grid = self.make_grid_background(renderer, bounds, screen_centre);
        vec![selected_cells, grid, cell_highlight]
    }
}

#[derive(Copy, Clone, Debug, PartialEq, Eq, Hash)]
pub struct GridCell {
    pub(crate) x: isize,
    pub(crate) y: isize,
}

impl GridCell {
    const SIZE: u16 = 20;

    fn cell_at_screen_point(position: Point) -> GridCell {
        let mathematical_x = (position.x / GridCell::SIZE as f32).ceil() as isize;
        let mathematical_y = (position.y / GridCell::SIZE as f32).ceil() as isize;

        GridCell {
            x: mathematical_x.saturating_sub(1),
            y: mathematical_y.saturating_sub(1),
        }
    }

    pub fn new(x: isize, y: isize) -> GridCell {
        Self { x, y }
    }

    pub fn euclidean_distance(&self, other: &Self) -> f64 {
        (((other.x - self.x) as f64).powi(2) + ((other.y - self.y) as f64).powi(2)).sqrt()
    }

    pub fn euclidean_distance_squared(&self, other: &Self) -> usize {
        ((other.x - self.x).checked_pow(2).unwrap() + (other.y - self.y).checked_pow(2).unwrap())
            as usize
    }

    pub fn invert_y(&self) -> Self {
        GridCell {
            x: self.x,
            y: -self.y,
        }
    }
}

impl From<GridCell> for Point {
    fn from(val: GridCell) -> Self {
        Point {
            x: val.x as f32,
            y: val.y as f32,
        }
    }
}

impl From<&GridCell> for Point {
    fn from(val: &GridCell) -> Self {
        Point::from(*val)
    }
}

pub struct Region {
    /// The x-coordinate for the top left corner of the region.
    x: f32,

    /// The y-coordinate for the top left corner of the region.
    y: f32,

    width: f32,
    height: f32,
}

impl Region {
    /// Get indices of all cell rows that should be visible
    fn rows(&self) -> RangeInclusive<isize> {
        let first_row = (self.y / GridCell::SIZE as f32).floor() as isize;

        let visible_rows = (self.height / GridCell::SIZE as f32).ceil() as isize;

        first_row..=first_row + visible_rows
    }

    /// Get indices of all cell columns that should be visible
    fn columns(&self) -> RangeInclusive<isize> {
        let first_column = (self.x / GridCell::SIZE as f32).floor() as isize;

        let visible_columns = (self.width / GridCell::SIZE as f32).ceil() as isize;

        first_column..=first_column + visible_columns
    }

    fn cull<'a>(
        &self,
        cells: impl Iterator<Item = &'a GridCell>,
    ) -> impl Iterator<Item = &'a GridCell> {
        let rows = self.rows();
        let columns = self.columns();

        cells.filter(move |cell| rows.contains(&cell.x) && columns.contains(&cell.y))
    }
}

#[derive(Debug, Clone, Default)]
pub enum GridInteraction {
    #[default]
    None,
    Panning {
        translation: Vector,
        origin: Point,
    },
}
