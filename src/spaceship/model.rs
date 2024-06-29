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

#[derive(Debug, Clone, Default)]
pub struct SpaceshipModel {
    pub points: Vec<Vector2D>,
}

impl SpaceshipModel {
    pub fn load(s: &str) -> SpaceshipModel {
        let mut points = vec![];
        for line in s.lines() {
            let mut coords = line.split_whitespace();
            let x = coords.next().unwrap().parse().unwrap();
            let y = coords.next().unwrap().parse().unwrap();
            points.push(Vector2D { x, y });
        }
        SpaceshipModel { points }
    }
}

pub enum Command {
    UpLeft,
    Up,
    UpRight,
    Left,
    KeepSpeed,
    Right,
    DownLeft,
    Down,
    DownRight,
}

#[derive(Debug, Clone)]
pub struct SpaceshipState {
    pub pos: Vector2D,
    pub speed: Vector2D,
    pub path: String,
}

impl SpaceshipState {
    pub fn new(pos: Vector2D, speed: Vector2D, path: String) -> Self {
        SpaceshipState { pos, speed, path }
    }

    pub fn next(&self, command: Command) -> SpaceshipState {
        let (new_pos, new_speed, path) = match command {
            Command::UpLeft => {
                let new_speed = self.speed + Vector2D::new(-1, 1);
                (self.pos + new_speed, new_speed, "7".to_string())
            }
            Command::Up => {
                let new_speed = self.speed + Vector2D::new(0, 1);
                (self.pos + new_speed, new_speed, "8".to_string())
            }
            Command::UpRight => {
                let new_speed = self.speed + Vector2D::new(1, 1);
                (self.pos + new_speed, new_speed, "9".to_string())
            }
            Command::Left => {
                let new_speed = self.speed + Vector2D::new(-1, 0);
                (self.pos + new_speed, new_speed, "4".to_string())
            }
            Command::KeepSpeed => (self.pos + self.speed, self.speed, "5".to_string()),
            Command::Right => {
                let new_speed = self.speed + Vector2D::new(1, 0);
                (self.pos + new_speed, new_speed, "6".to_string())
            }
            Command::DownLeft => {
                let new_speed = self.speed + Vector2D::new(-1, -1);
                (self.pos + new_speed, new_speed, "1".to_string())
            }
            Command::Down => {
                let new_speed = self.speed + Vector2D::new(0, -1);
                (self.pos + new_speed, new_speed, "2".to_string())
            }
            Command::DownRight => {
                let new_speed = self.speed + Vector2D::new(1, -1);
                (self.pos + new_speed, new_speed, "3".to_string())
            }
        };
        SpaceshipState {
            pos: new_pos,
            speed: new_speed,
            path: self.path.clone() + &path,
        }
    }
}
