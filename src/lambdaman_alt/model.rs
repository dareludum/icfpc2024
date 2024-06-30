use crate::geometry::{Move, Vector2D};

#[derive(Debug, Clone, Default)]
pub struct LambdamanModel {
    pub width: usize,
    pub height: usize,
    pub map: Vec<Vec<Cell>>,
    pub player_pos: Vector2D,
    pub fruit_count: usize,
    pub move_count: usize,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Cell {
    Wall,
    Fruit,
    Empty,
}

impl LambdamanModel {
    pub fn is_solved(&self) -> bool {
        self.fruit_count == 0
    }

    pub fn load(s: &str) -> Self {
        let mut width = 0usize;
        let mut height = 0usize;
        let mut fruit_count = 0usize;
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
                    '.' => {
                        fruit_count += 1;
                        row.push(Cell::Fruit);
                    }
                    'L' => {
                        row.push(Cell::Empty);
                        player_pos = Vector2D::new(i as i32, height as i32);
                    }
                    _ => panic!("invalid cell: {}", c),
                }
            }
            map.push(row);
            height += 1;
        }
        Self {
            width,
            height,
            map,
            player_pos,
            fruit_count,
            move_count: 0,
        }
    }

    pub fn check_bounds(&self, pos: &Vector2D) -> bool {
        pos.x >= 0 && pos.x < (self.width as i32) && pos.y >= 0 && pos.y < (self.height as i32)
    }

    pub fn get_mut(&mut self, pos: &Vector2D) -> Option<&mut Cell> {
        if self.check_bounds(pos) {
            Some(&mut self.map[pos.y as usize][pos.x as usize])
        } else {
            None
        }
    }

    pub fn apply(&mut self, mov: Move) {
        self.move_count += 1;
        let new_pos = self.player_pos.apply(mov);

        // if the move is oob, ignore it
        let Some(cell) = self.get_mut(&new_pos) else {
            return;
        };

        // if the move is into a wall, do nothing
        match *cell {
            Cell::Wall => return,
            Cell::Empty => (),
            Cell::Fruit => {
                *cell = Cell::Empty;
                self.fruit_count -= 1;
            }
        }
        self.player_pos = new_pos;
    }

    pub fn print(&self) {
        for (y, line) in self.map.iter().enumerate() {
            for (x, cell) in line.iter().enumerate() {
                if x == self.player_pos.x as usize && y == self.player_pos.y as usize {
                    eprint!("L");
                    continue;
                }
                eprint!(
                    "{}",
                    match cell {
                        Cell::Wall => '#',
                        Cell::Fruit => '.',
                        Cell::Empty => ' ',
                    }
                );
            }
            eprintln!();
        }
    }
}
