use crate::geometry::Vector2D;

#[derive(Debug, Clone, Default)]
pub struct LambdamanModel {
    pub width: u32,
    pub height: u32,
    pub map: Vec<Vec<Cell>>,
    pub player_pos: Vector2D,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Cell {
    Wall,
    Fruit,
    Empty,
}

impl LambdamanModel {
    pub fn load(s: &str) -> Self {
        let mut width = 0;
        let mut height = 0;
        let mut player_pos = Vector2D::default();
        let mut map = Vec::new();
        for line in s.lines() {
            if line.is_empty() {
                break;
            }
            if (width > 0) && (line.len() != width) {
                panic!("inconsistent line length");
            }
            width = line.len();
            let mut row = vec![];
            for (i, c) in line.chars().enumerate() {
                match c {
                    '#' => row.push(Cell::Wall),
                    '.' => row.push(Cell::Fruit),
                    'L' => {
                        row.push(Cell::Empty);
                        player_pos = Vector2D::new(i as i32, height);
                    }
                    _ => panic!("invalid cell: {}", c),
                }
            }
            map.push(row);
            height += 1;
        }
        Self {
            width: width as u32,
            height: height as u32,
            map,
            player_pos,
        }
    }
}
