use crate::geometry::Vector2D;

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

#[derive(Debug, Clone, Default)]
pub struct SpaceshipState {
    pub pos: Vector2D,
    pub speed: Vector2D,
    pub path: String,
}

impl SpaceshipState {
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

    pub fn get_all_next_moves(&self) -> Vec<SpaceshipState> {
        vec![
            Command::UpLeft,
            Command::Up,
            Command::UpRight,
            Command::Left,
            Command::KeepSpeed,
            Command::Right,
            Command::DownLeft,
            Command::Down,
            Command::DownRight,
        ]
        .into_iter()
        .map(|c| self.next(c))
        .collect()
    }
}
