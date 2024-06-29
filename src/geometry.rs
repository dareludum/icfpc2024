use std::ops::{Add, Sub};

#[derive(Debug, Clone, Copy, Default, PartialEq, Eq, Hash)]
pub struct Vector2D {
    pub x: i32,
    pub y: i32,
}
impl Vector2D {
    pub fn new(x: i32, y: i32) -> Self {
        Vector2D { x, y }
    }

    pub fn manhattan(&self) -> i32 {
        self.x.abs() + self.y.abs()
    }
}

impl Add for Vector2D {
    type Output = Vector2D;

    fn add(self, other: Vector2D) -> Vector2D {
        Vector2D {
            x: self.x + other.x,
            y: self.y + other.y,
        }
    }
}

impl Sub for Vector2D {
    type Output = Vector2D;

    fn sub(self, rhs: Self) -> Vector2D {
        Vector2D {
            x: self.x - rhs.x,
            y: self.y - rhs.y,
        }
    }
}
