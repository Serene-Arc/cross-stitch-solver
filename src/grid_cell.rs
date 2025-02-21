use iced::Point;

#[derive(Copy, Clone, Debug, PartialEq, Eq, Hash)]
pub struct GridCell {
    pub(crate) x: isize,
    pub(crate) y: isize,
}

impl GridCell {
    pub const SIZE: u16 = 20;

    pub fn cell_at_screen_point(position: Point) -> GridCell {
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
