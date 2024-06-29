use std::collections::{HashMap, HashSet};

use crate::geometry::Vector2D;

use super::board::ThreeDBoard;

#[derive(Debug, Clone, Default)]
pub struct ThreeDSimulator {
    current_cells: HashMap<Vector2D, Cell>,
    history: Vec<HashMap<Vector2D, Cell>>,
    current_time: u32,
    all_time_min_x: i32,
    all_time_max_x: i32,
    all_time_min_y: i32,
    all_time_max_y: i32,
    all_time_max_t: u32,
    a: i64,
    b: i64,
    steps_taken: u32,
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
    InputA,
    InputB,
}

impl ThreeDSimulator {
    pub fn new(board: ThreeDBoard, a: i64, b: i64) -> Self {
        let mut cells = HashMap::new();
        for (y, row) in board.board.iter().enumerate() {
            for (x, cell) in row.iter().enumerate() {
                let pos = Vector2D::new(x as i32, y as i32);
                if let Some(cell) = *cell {
                    cells.insert(pos, cell);
                }
            }
        }

        let mut min_x = i32::MAX;
        let mut max_x = i32::MIN;
        let mut min_y = i32::MAX;
        let mut max_y = i32::MIN;
        for pos in cells.keys() {
            min_x = min_x.min(pos.x);
            max_x = max_x.max(pos.x);
            min_y = min_y.min(pos.y);
            max_y = max_y.max(pos.y);
        }

        Self {
            current_cells: cells,
            history: vec![],
            current_time: 0,
            all_time_min_x: min_x,
            all_time_max_x: max_x,
            all_time_min_y: min_y,
            all_time_max_y: max_y,
            all_time_max_t: 0,
            a,
            b,
            steps_taken: 0,
        }
    }

    pub fn time(&self) -> u32 {
        self.current_time
    }

    pub fn score(&self) -> u32 {
        (self.all_time_max_x - self.all_time_min_x + 1) as u32
            * (self.all_time_max_y - self.all_time_min_y + 1) as u32
            * self.all_time_max_t
    }

    pub fn a(&self) -> i64 {
        self.a
    }

    pub fn set_a(&mut self, a: i64) {
        self.a = a;
    }

    pub fn b(&self) -> i64 {
        self.b
    }

    pub fn set_b(&mut self, b: i64) {
        self.b = b;
    }

    pub fn steps_taken(&self) -> u32 {
        self.steps_taken
    }

    pub fn cells(&self) -> &HashMap<Vector2D, Cell> {
        &self.current_cells
    }

    pub fn step(&mut self) -> Result<Option<i64>, Vector2D> {
        self.steps_taken += 1;
        if self.steps_taken > 1_000_000 {
            // TODO: a better error
            return Err(Vector2D::new(0, 0));
        }

        if self.history.is_empty() {
            self.history.push(self.current_cells.clone());

            let mut input_a_positions = vec![];
            let mut input_b_positions = vec![];
            for pos in self.current_cells.keys() {
                if self.current_cells.get(pos).unwrap() == &Cell::InputA {
                    input_a_positions.push(*pos);
                } else if self.current_cells.get(pos).unwrap() == &Cell::InputB {
                    input_b_positions.push(*pos);
                }
            }

            for pos in input_a_positions {
                self.current_cells.insert(pos, Cell::Data(self.a));
            }
            for pos in input_b_positions {
                self.current_cells.insert(pos, Cell::Data(self.b));
            }

            self.current_time += 1;
            return Ok(None);
        }

        enum Action {
            Erase(Vector2D),
            Write(Vector2D, Cell),
            TimeTravel(u32, Vector2D, i64),
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
                Cell::TimeWarp => {
                    if let (Some(cell_dx), Some(cell_dy), Some(cell_dt), Some(cell_v)) = (
                        self.current_cells.get(&pos.left()),
                        self.current_cells.get(&pos.right()),
                        self.current_cells.get(&pos.down()),
                        self.current_cells.get(&pos.up()),
                    ) {
                        match (cell_dx, cell_dy, cell_dt, cell_v) {
                            (Cell::Data(dx), Cell::Data(dy), Cell::Data(dt), Cell::Data(v)) => {
                                if *dt <= 0 {
                                    return Err(*pos);
                                }
                                actions.push(Action::TimeTravel(
                                    self.current_time - (*dt as u32),
                                    *pos - Vector2D::new(*dx as i32, *dy as i32),
                                    *v,
                                ));
                            }
                            _ => return Err(*pos),
                        }
                    }
                }
                Cell::Submit => {}
                Cell::InputA => {}
                Cell::InputB => {}
            }
        }

        if actions.is_empty() {
            // TODO: a better error
            return Err(Vector2D::new(0, 0));
        }

        let time_travels = actions
            .iter()
            .filter_map(|action| match action {
                Action::TimeTravel(time, pos, v) => Some((*time, *pos, *v)),
                _ => None,
            })
            .collect::<Vec<_>>();

        // First, process the new state without time travels, because submits take priority
        let mut new_cells = self.current_cells.clone();

        let mut moved_to_cells = HashSet::new();
        let mut submitted_value = None;
        for action in actions {
            match action {
                Action::Erase(pos) => {
                    new_cells.remove(&pos);
                }
                Action::Write(pos, cell) => {
                    if moved_to_cells.contains(&pos) {
                        return Err(pos);
                    }
                    if let Some(Cell::Submit) = new_cells.get(&pos) {
                        if submitted_value.is_some() {
                            return Err(pos);
                        }
                        if let Cell::Data(cell) = cell {
                            submitted_value = Some(cell);
                        } else {
                            return Err(pos);
                        }
                    }
                    new_cells.insert(pos, cell);
                    moved_to_cells.insert(pos);
                }
                Action::TimeTravel(_, _, _) => {
                    // Processed below
                }
            }
        }

        for pos in new_cells.keys() {
            self.all_time_min_x = self.all_time_min_x.min(pos.x);
            self.all_time_max_x = self.all_time_max_x.max(pos.x);
            self.all_time_min_y = self.all_time_min_y.min(pos.y);
            self.all_time_max_y = self.all_time_max_y.max(pos.y);
        }

        if submitted_value.is_some() {
            return Ok(submitted_value);
        }

        if !time_travels.is_empty() {
            let mut target_times = HashSet::new();
            for (time, _, _) in &time_travels {
                target_times.insert(*time);
            }
            if target_times.len() != 1 {
                // TODO: a better error
                return Err(Vector2D::new(0, 0));
            }
            let target_time = target_times.into_iter().next().unwrap();

            let mut target_writes = HashMap::new();
            for (_, pos, value) in &time_travels {
                if let Some(v) = target_writes.get(pos) {
                    if *v != value {
                        return Err(*pos);
                    }
                }
                target_writes.insert(*pos, value);
            }

            self.history.truncate(target_time as usize + 1);
            // Discard the current new state and fetch it from the history
            new_cells = self.history.pop().unwrap();

            for (pos, value) in target_writes {
                new_cells.insert(pos, Cell::Data(*value));
            }

            self.current_cells = new_cells;
            self.current_time = target_time;
        } else {
            self.history
                .push(std::mem::replace(&mut self.current_cells, new_cells));
            self.current_time += 1;
            self.all_time_max_t = self.all_time_max_t.max(self.current_time);
        }

        Ok(None)
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

        let mut board =
            vec![vec![None; (max_x - min_x + 1) as usize]; (max_y - min_y + 1) as usize];
        for (pos, cell) in &self.current_cells {
            let x = (pos.x - min_x) as usize;
            let y = (pos.y - min_y) as usize;
            board[y][x] = Some(*cell);
        }

        ThreeDBoard { board }
    }

    pub fn step_back(&mut self) {
        // This is somewhat wrong, but it's clearer for the GUI
        self.steps_taken += 1;
        if self.current_time > 0 {
            self.current_time -= 1;
            self.current_cells = self.history.pop().unwrap();
        }
    }

    pub fn remove_cell(&mut self, pos: Vector2D) {
        self.current_cells.remove(&pos);
    }

    pub fn set_cell(&mut self, pos: Vector2D, cell: Cell) {
        self.current_cells.insert(pos, cell);
    }
}
