use crate::stitch::HalfStitch;
use crate::ProgramState;
use iced::event::Status;
use iced::mouse::Cursor;
use iced::widget::canvas::{Cache, Event, Frame, Geometry, Path, Stroke, Style, Text};
use iced::widget::{canvas, Canvas};
use iced::{
    alignment, mouse, Color, Element, Fill, Point, Rectangle, Renderer, Size, Theme, Vector,
};
use std::ops::RangeInclusive;

#[derive(Debug, Clone)]
pub enum Message {
    Select(GridCell),
    Unselect(GridCell),
    Translated(Vector),
    Scaled(f32, Option<Vector>),
}

#[derive(Debug)]
pub struct GridState {
    grid_cache: Cache,
    cell_cache: Cache,
    translation: Vector,
    scaling: f32,
    pub program_state: ProgramState,
}

impl Default for GridState {
    fn default() -> Self {
        Self {
            grid_cache: Cache::default(),
            cell_cache: Cache::default(),
            translation: Default::default(),
            scaling: 4.0,
            program_state: Default::default(),
        }
    }
}

impl GridState {
    const MIN_SCALING: f32 = 0.1;
    const MAX_SCALING: f32 = 4.0;

    /// Determine what part of the grid should be visible.
    fn visible_region(&self, size: Size) -> Region {
        let width = size.width / self.scaling;
        let height = size.height / self.scaling;

        Region {
            x: -self.translation.x - width / 2.0,
            y: -self.translation.y - height / 2.0,
            width,
            height,
        }
    }

    pub fn clear(&mut self) {
        self.grid_cache.clear();
        self.cell_cache.clear();
        self.program_state.clear();
    }

    /// Project a given point onto the visible region of the grid.
    fn project(&self, input_position: Point, visible_size: Size) -> Point {
        let region = self.visible_region(visible_size);

        Point::new(
            input_position.x / self.scaling + region.x,
            input_position.y / self.scaling + region.y,
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
            Message::Scaled(scaling, translation) => {
                self.scaling = scaling;

                if let Some(translation) = translation {
                    self.translation = translation;
                }

                self.grid_cache.clear();
                self.cell_cache.clear();
            }
        }
    }

    pub fn view(&self) -> Element<Message> {
        Canvas::new(self).width(Fill).height(Fill).into()
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
        let cursor_position = match cursor.position_in(bounds) {
            None => {
                return (Status::Ignored, None);
            }
            Some(pos) => pos,
        };

        let cell = GridCell::at(self.project(cursor_position, bounds.size()));
        match event {
            Event::Mouse(mouse_event) => match mouse_event {
                mouse::Event::ButtonPressed(button) => {
                    let message = match button {
                        mouse::Button::Left => Some(Message::Select(cell)),
                        mouse::Button::Right => Some(Message::Unselect(cell)),
                        mouse::Button::Middle => {
                            *interaction = GridInteraction::Panning {
                                translation: self.translation,
                                origin: cursor_position,
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
                            origin,
                        } => Some(Message::Translated(
                            translation + (cursor_position - origin) * (1.0 / self.scaling),
                        )),
                        GridInteraction::None => None,
                    };

                    (Status::Captured, message)
                }
                mouse::Event::WheelScrolled { delta } => match delta {
                    mouse::ScrollDelta::Lines { y, .. } | mouse::ScrollDelta::Pixels { y, .. } => {
                        if y < 0.0 && self.scaling > Self::MIN_SCALING
                            || y > 0.0 && self.scaling < Self::MAX_SCALING
                        {
                            let old_scaling = self.scaling;

                            let scaling = (self.scaling * (1.0 + y / 30.0))
                                .clamp(Self::MIN_SCALING, Self::MAX_SCALING);

                            let translation = if let Some(cursor_to_center) =
                                cursor.position_from(bounds.center())
                            {
                                let factor = scaling - old_scaling;

                                Some(
                                    self.translation
                                        - Vector::new(
                                            cursor_to_center.x * factor
                                                / (old_scaling * old_scaling),
                                            cursor_to_center.y * factor
                                                / (old_scaling * old_scaling),
                                        ),
                                )
                            } else {
                                None
                            };

                            let message = Message::Scaled(scaling, translation);
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
        let centre = Vector::new(bounds.width / 2.0, bounds.height / 2.0);
        let stitches = HalfStitch::convert_grid_cells(self.program_state.selected_cells.iter());
        let valid_sequence = HalfStitch::check_valid_sequence(&stitches);

        let selected_cells = self.cell_cache.draw(renderer, bounds.size(), |frame| {
            let background = Path::rectangle(Point::ORIGIN, frame.size());
            frame.fill(&background, Color::from_rgb8(0x40, 0x44, 0x4B));

            frame.with_save(|frame| {
                frame.translate(centre);
                frame.scale_nonuniform(Vector::new(1.0, -1.0));
                frame.scale(self.scaling);
                frame.translate(self.translation);
                frame.scale(GridCell::SIZE);

                let region = self.visible_region(frame.size());

                for cell in region.cull(self.program_state.selected_cells.iter()) {
                    frame.fill_rectangle(
                        Point::new(cell.x as f32, cell.y as f32),
                        Size::UNIT,
                        Color::WHITE,
                    );
                }
                match &valid_sequence {
                    Ok(_) => {}
                    Err((first, second)) => {
                        for cell in region.cull([*first, *second].iter()) {
                            frame.fill_rectangle(
                                Point::new(cell.x as f32, cell.y as f32),
                                Size::UNIT,
                                Color::from_rgb(100.0, 0.0, 0.0),
                            );
                        }
                    }
                }
                let mut alpha = 1.0;
                for stitch in stitches.iter().rev() {
                    let line = if stitch.facing_right {
                        Path::line(
                            Point {
                                x: stitch.start.x as f32,
                                y: stitch.start.y as f32,
                            },
                            Point {
                                x: (stitch.start.x + 1) as f32,
                                y: (stitch.start.y + 1) as f32,
                            },
                        )
                    } else {
                        Path::line(
                            Point {
                                x: (stitch.start.x) as f32,
                                y: stitch.start.y as f32,
                            },
                            Point {
                                x: (stitch.start.x - 1) as f32,
                                y: (stitch.start.y + 1) as f32,
                            },
                        )
                    };
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

            let hovered_grid_cell = cursor
                .position_in(bounds)
                .map(|position| GridCell::at(self.project(position, frame.size())));

            if let Some(cell) = hovered_grid_cell {
                frame.with_save(|frame| {
                    frame.translate(centre);
                    frame.scale_nonuniform(Vector::new(1.0, -1.0));
                    frame.scale(self.scaling);
                    frame.translate(self.translation);
                    frame.scale(GridCell::SIZE);

                    frame.fill_rectangle(
                        Point::new(cell.x as f32, cell.y as f32),
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
                ..Text::default()
            };
            if let Some(cell) = hovered_grid_cell {
                frame.fill_text(Text {
                    content: format!("({}, {})", cell.x, -cell.y),
                    position: text.position - Vector::new(0.0, 16.0),
                    ..text
                });
                let cell_count = self.program_state.selected_cells.len();

                frame.fill_text(Text {
                    content: format!(
                        "{cell_count} cell{} @ {}",
                        if cell_count == 1 { "" } else { "s" },
                        if valid_sequence.is_ok() {
                            format!("{:.3} distance", valid_sequence.unwrap())
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
        let grid = self.grid_cache.draw(renderer, bounds.size(), |frame| {
            frame.translate(centre);
            frame.scale_nonuniform(Vector::new(1.0, -1.0));
            frame.scale(self.scaling);
            frame.translate(self.translation);
            frame.scale(GridCell::SIZE);

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
        });
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

    fn at(position: Point) -> GridCell {
        let x = (position.x / GridCell::SIZE as f32).ceil() as isize;
        let y = (position.y / GridCell::SIZE as f32).ceil() as isize;

        GridCell {
            x: x.saturating_sub(1),
            y: -y,
        }
    }

    pub fn new(x: isize, y: isize) -> GridCell {
        Self { x, y }
    }

    pub fn euclidean_distance(&self, other: &Self) -> f64 {
        (((other.x - self.x) as f64).powi(2) + ((other.y - self.y) as f64).powi(2)).sqrt()
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

pub struct Region {
    x: f32,
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
