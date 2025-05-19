use std::ops::{Add, Sub};

use macroquad::math::{Rect, Vec2};
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]

#[derive(Serialize, Deserialize, Default)]
pub struct Pos {
    pub x: i16,
    pub y: i16,
}

pub mod dirs {
    use super::Pos;

    pub const NONE: Pos = Pos::new(0, 0);

    pub const LEFT: Pos = Pos::new(-1, 0);
    pub const RIGHT: Pos = Pos::new(1, 0);
    pub const UP: Pos = Pos::new(0, -1);
    pub const DOWN: Pos = Pos::new(0, 1);

    pub const ALL: [Pos; 4] = [LEFT, RIGHT, UP, DOWN];
}

impl Pos {
    pub const fn new(x: i16, y: i16) -> Pos {
        Pos { x, y }
    }

    pub(crate) fn diff(&self, pos: Pos) -> i16 {
        (self.x - pos.x).abs() + (self.y - pos.y).abs()
    }

    pub fn min(&self, other: Pos) -> Pos {
        Self {
            x: self.x.min(other.x),
            y: self.y.min(other.y),
        }
    }

    pub fn max(&self, other: Pos) -> Pos {
        Self {
            x: self.x.max(other.x),
            y: self.y.max(other.y),
        }
    }

    // inclusive!
    pub fn iter(&self, other: Self) -> impl Iterator<Item = Self> {
        let min = self.min(other);
        let max = self.max(other);
        (min.y..max.y + 1).flat_map(move |y: i16| (min.x..max.x + 1).map(move |x| (x, y).into()))
    }
}

impl From<(i16, i16)> for Pos {
    fn from(tuple: (i16, i16)) -> Self {
        Pos::new(tuple.0, tuple.1)
    }
}

impl Add<Pos> for Pos {
    type Output = Pos;

    fn add(self, rhs: Pos) -> Self::Output {
        Self {
            x: self.x + rhs.x,
            y: self.y + rhs.y,
        }
    }
}

impl Sub<Pos> for Pos {
    type Output = Pos;

    fn sub(self, rhs: Pos) -> Self::Output {
        Self {
            x: self.x - rhs.x,
            y: self.y - rhs.y,
        }
    }
}

// Default zoom pixel size of Position
pub const GRID_CELL_SIZE: (f32, f32) = (64., 64.);
pub const PIXEL_SIZE: f32 = 64. / 16.;

impl From<Vec2> for Pos {
    fn from(value: Vec2) -> Self {
        Self {
            x: (value.x / GRID_CELL_SIZE.0) as i16,
            y: (value.y / GRID_CELL_SIZE.1) as i16,
        }
    }
}

impl From<Pos> for Vec2 {
    fn from(value: Pos) -> Self {
        Self {
            x: value.x as f32 * GRID_CELL_SIZE.0,
            y: value.y as f32 * GRID_CELL_SIZE.1,
        }
    }
}

impl From<Pos> for Rect {
    fn from(pos: Pos) -> Self {
        Rect::new(
            pos.x as f32 * GRID_CELL_SIZE.0,
            pos.y as f32 * GRID_CELL_SIZE.1, /* - (pos.z as f32 * GRID_Z_OFFSET) */
            GRID_CELL_SIZE.0,
            GRID_CELL_SIZE.1,
        )
    }
}
