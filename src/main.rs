use iced::mouse::Cursor;
use iced::widget::canvas::{Cache, Frame, Geometry, Text};
use iced::widget::{canvas, column};
use iced::{alignment, Color, Element, Fill, Point, Rectangle, Renderer, Size, Theme, Vector};
use std::ops::RangeInclusive;

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
struct Message {}

#[derive(Debug, Default)]
struct CrossStitchSolver {
    grid_state: GridState,
}

impl CrossStitchSolver {
    fn update(&mut self, message: Message) {}
    fn view(&self) -> Element<Message> {
        column![canvas(&self.grid_state).width(Fill).height(Fill)]
            .spacing(10)
            .into()
    }
}

#[derive(Debug)]
struct GridState {
    cache: Cache,
    translation: Vector,
    scaling: f32,
}

impl Default for GridState {
    fn default() -> Self {
        Self {
            cache: Cache::default(),
            translation: Default::default(),
            scaling: 2.0,
        }
    }
}

impl GridState {
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

    /// Project a given point onto the visible region of the grid.
    fn project(&self, input_position: Point, visible_size: Size) -> Point {
        let region = self.visible_region(visible_size);

        Point::new(
            input_position.x / self.scaling + region.x,
            input_position.y / self.scaling + region.y,
        )
    }
}
impl<Message> canvas::Program<Message> for GridState {
    type State = ();

    fn draw(
        &self,
        state: &Self::State,
        renderer: &Renderer,
        theme: &Theme,
        bounds: Rectangle,
        cursor: Cursor,
    ) -> Vec<Geometry<Renderer>> {
        let centre = Vector::new(bounds.width / 2.0, bounds.height / 2.0);

        let cell_highlight = {
            let mut frame = Frame::new(renderer, bounds.size());

            let hovered_grid_cell = cursor
                .position_in(bounds)
                .map(|position| GridCell::at(self.project(position, frame.size())));

            if let Some(cell) = hovered_grid_cell {
                frame.with_save(|frame| {
                    frame.translate(centre);
                    frame.scale(self.scaling);
                    frame.translate(self.translation);
                    frame.scale(GridCell::SIZE);

                    frame.fill_rectangle(
                        Point::new(cell.x as f32, cell.y as f32),
                        Size::UNIT,
                        Color {
                            a: 0.5,
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
            }

            frame.into_geometry()
        };

        // Make the grid for the cells
        let grid = self.cache.draw(renderer, bounds.size(), |frame| {
            frame.translate(centre);
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
        vec![grid, cell_highlight]
    }
}

#[derive(Copy, Clone, Debug)]
struct GridCell {
    x: isize,
    y: isize,
}

impl GridCell {
    const SIZE: u16 = 20;

    fn at(position: Point) -> GridCell {
        let x = (position.x / GridCell::SIZE as f32).ceil() as isize;
        let y = (position.y / GridCell::SIZE as f32).ceil() as isize;

        GridCell {
            x: x.saturating_sub(1),
            y: y.saturating_sub(1),
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
