use std::collections::{HashMap, HashSet};

use crate::geometry::Vector2D;

use super::board::{BoardCell, ThreeDBoard};

#[derive(Debug, Clone, Default)]
pub struct ThreeDSimulator {
    current_cells: HashMap<Vector2D, Cell>,
    history: Vec<HashMap<Vector2D, Cell>>,
    current_time: u32,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Cell {
    Data(i64),
    MoveLeft,
    MoveRight,
    MoveUp,
    MoveDown,
    Add,
    Subtract,
    Multiply,
    Divide,
    Modulo,
    Equal,
    NotEqual,
    TimeWarp,
    Submit,
}

impl ThreeDSimulator {
    pub fn new(board: ThreeDBoard, a: i64, b: i64) -> Self {
        let mut current_cells = HashMap::new();
        for (y, row) in board.board.iter().enumerate() {
            for (x, cell) in row.iter().enumerate() {
                let pos = Vector2D::new(x as i32, y as i32);
                match *cell {
                    BoardCell::SimCell(cell) => {
                        current_cells.insert(pos, cell);
                    }
                    BoardCell::InputA => {
                        current_cells.insert(pos, Cell::Data(a));
                    }
                    BoardCell::InputB => {
                        current_cells.insert(pos, Cell::Data(b));
                    }
                    BoardCell::Empty => {}
                }
            }
        }
        Self {
            current_cells,
            history: vec![],
            current_time: 1,
        }
    }

    pub fn step(&mut self) -> Result<bool, Vector2D> {
        enum Action {
            Erase(Vector2D),
            Write(Vector2D, Cell),
        }
        let mut actions = vec![];
        for (pos, cell) in self.current_cells.iter() {
            match *cell {
                Cell::Data(_) => {}
                Cell::MoveLeft => {
                    if let Some(cell) = self.current_cells.get(&pos.right()) {
                        actions.push(Action::Erase(pos.right()));
                        actions.push(Action::Write(pos.left(), *cell));
                    }
                }
                Cell::MoveRight => {
                    if let Some(cell) = self.current_cells.get(&pos.left()) {
                        actions.push(Action::Erase(pos.left()));
                        actions.push(Action::Write(pos.right(), *cell));
                    }
                }
                Cell::MoveUp => {
                    if let Some(cell) = self.current_cells.get(&pos.down()) {
                        actions.push(Action::Erase(pos.down()));
                        actions.push(Action::Write(pos.up(), *cell));
                    }
                }
                Cell::MoveDown => {
                    if let Some(cell) = self.current_cells.get(&pos.up()) {
                        actions.push(Action::Erase(pos.up()));
                        actions.push(Action::Write(pos.down(), *cell));
                    }
                }
                Cell::Add => {
                    if let (Some(cell_x), Some(cell_y)) = (
                        self.current_cells.get(&pos.left()),
                        self.current_cells.get(&pos.up()),
                    ) {
                        match (cell_x, cell_y) {
                            (Cell::Data(x), Cell::Data(y)) => {
                                actions.push(Action::Erase(pos.left()));
                                actions.push(Action::Erase(pos.up()));
                                let res = Cell::Data(x + y);
                                actions.push(Action::Write(pos.right(), res));
                                actions.push(Action::Write(pos.down(), res));
                            }
                            _ => return Err(*pos),
                        }
                    }
                }
                Cell::Subtract => {
                    if let (Some(cell_x), Some(cell_y)) = (
                        self.current_cells.get(&pos.left()),
                        self.current_cells.get(&pos.up()),
                    ) {
                        match (cell_x, cell_y) {
                            (Cell::Data(x), Cell::Data(y)) => {
                                actions.push(Action::Erase(pos.left()));
                                actions.push(Action::Erase(pos.up()));
                                let res = Cell::Data(x - y);
                                actions.push(Action::Write(pos.right(), res));
                                actions.push(Action::Write(pos.down(), res));
                            }
                            _ => return Err(*pos),
                        }
                    }
                }
                Cell::Multiply => {
                    if let (Some(cell_x), Some(cell_y)) = (
                        self.current_cells.get(&pos.left()),
                        self.current_cells.get(&pos.up()),
                    ) {
                        match (cell_x, cell_y) {
                            (Cell::Data(x), Cell::Data(y)) => {
                                actions.push(Action::Erase(pos.left()));
                                actions.push(Action::Erase(pos.up()));
                                let res = Cell::Data(x * y);
                                actions.push(Action::Write(pos.right(), res));
                                actions.push(Action::Write(pos.down(), res));
                            }
                            _ => return Err(*pos),
                        }
                    }
                }
                Cell::Divide => {
                    if let (Some(cell_x), Some(cell_y)) = (
                        self.current_cells.get(&pos.left()),
                        self.current_cells.get(&pos.up()),
                    ) {
                        match (cell_x, cell_y) {
                            (Cell::Data(x), Cell::Data(y)) => {
                                actions.push(Action::Erase(pos.left()));
                                actions.push(Action::Erase(pos.up()));
                                let res = Cell::Data(x / y);
                                actions.push(Action::Write(pos.right(), res));
                                actions.push(Action::Write(pos.down(), res));
                            }
                            _ => return Err(*pos),
                        }
                    }
                }
                Cell::Modulo => {
                    if let (Some(cell_x), Some(cell_y)) = (
                        self.current_cells.get(&pos.left()),
                        self.current_cells.get(&pos.up()),
                    ) {
                        match (cell_x, cell_y) {
                            (Cell::Data(x), Cell::Data(y)) => {
                                actions.push(Action::Erase(pos.left()));
                                actions.push(Action::Erase(pos.up()));
                                let res = Cell::Data(x % y);
                                actions.push(Action::Write(pos.right(), res));
                                actions.push(Action::Write(pos.down(), res));
                            }
                            _ => return Err(*pos),
                        }
                    }
                }
                Cell::Equal => {
                    if let (Some(cell_x), Some(cell_y)) = (
                        self.current_cells.get(&pos.left()),
                        self.current_cells.get(&pos.up()),
                    ) {
                        if cell_x == cell_y {
                            actions.push(Action::Erase(pos.left()));
                            actions.push(Action::Erase(pos.up()));
                            actions.push(Action::Write(pos.right(), *cell_x));
                            actions.push(Action::Write(pos.down(), *cell_x));
                        }
                    }
                }
                Cell::NotEqual => {
                    if let (Some(cell_x), Some(cell_y)) = (
                        self.current_cells.get(&pos.left()),
                        self.current_cells.get(&pos.up()),
                    ) {
                        if cell_x != cell_y {
                            actions.push(Action::Erase(pos.left()));
                            actions.push(Action::Erase(pos.up()));
                            actions.push(Action::Write(pos.right(), *cell_y));
                            actions.push(Action::Write(pos.down(), *cell_x));
                        }
                    }
                }
                Cell::TimeWarp => todo!(),
                Cell::Submit => {}
            }
        }

        let mut new_cells = HashMap::new();

        let mut moved_to_cells = HashSet::new();
        for action in actions {
            match action {
                Action::Erase(pos) => {
                    new_cells.remove(&pos);
                }
                Action::Write(pos, cell) => {
                    if moved_to_cells.contains(&pos) {
                        return Err(pos);
                    }
                    new_cells.insert(pos, cell);
                    moved_to_cells.insert(pos);
                }
            }
        }

        self.history
            .push(std::mem::replace(&mut self.current_cells, new_cells));
        self.current_time += 1;

        Ok(false)
    }

    pub fn as_board(&self) -> ThreeDBoard {
        let mut min_x = i32::MAX;
        let mut max_x = i32::MIN;
        let mut min_y = i32::MAX;
        let mut max_y = i32::MIN;
        for pos in self.current_cells.keys() {
            min_x = min_x.min(pos.x);
            max_x = max_x.max(pos.x);
            min_y = min_y.min(pos.y);
            max_y = max_y.max(pos.y);
        }

        let mut board = vec![
            vec![BoardCell::Empty; (max_x - min_x + 1) as usize];
            (max_y - min_y + 1) as usize
        ];
        for (pos, cell) in &self.current_cells {
            let x = (pos.x - min_x) as usize;
            let y = (pos.y - min_y) as usize;
            board[y][x] = BoardCell::SimCell(*cell);
        }

        ThreeDBoard { board }
    }
}
