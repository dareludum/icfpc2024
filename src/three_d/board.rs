use super::sim::Cell;

// NOTE: This struct is only used for loading/saving the initial board state,
// see sim.rs for the simulation state and logic.
#[derive(Debug, Clone, Default)]
pub struct ThreeDBoard {
    pub board: Vec<Vec<BoardCell>>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum BoardCell {
    SimCell(Cell),
    // Only for loading/saving
    InputA,
    InputB,
    Empty,
}

impl ThreeDBoard {
    pub fn load(s: &str) -> Self {
        let mut board = Vec::new();
        for line in s.lines() {
            let mut row = vec![];
            for cell in line.split_ascii_whitespace() {
                match cell {
                    "." => row.push(BoardCell::Empty),
                    "<" => row.push(BoardCell::SimCell(Cell::MoveLeft)),
                    ">" => row.push(BoardCell::SimCell(Cell::MoveRight)),
                    "^" => row.push(BoardCell::SimCell(Cell::MoveUp)),
                    "v" => row.push(BoardCell::SimCell(Cell::MoveDown)),
                    "+" => row.push(BoardCell::SimCell(Cell::Add)),
                    "-" => row.push(BoardCell::SimCell(Cell::Subtract)),
                    "*" => row.push(BoardCell::SimCell(Cell::Multiply)),
                    "/" => row.push(BoardCell::SimCell(Cell::Divide)),
                    "%" => row.push(BoardCell::SimCell(Cell::Modulo)),
                    "=" => row.push(BoardCell::SimCell(Cell::Equal)),
                    "#" => row.push(BoardCell::SimCell(Cell::NotEqual)),
                    "@" => row.push(BoardCell::SimCell(Cell::TimeWarp)),
                    "S" => row.push(BoardCell::SimCell(Cell::Submit)),
                    "A" => row.push(BoardCell::InputA),
                    "B" => row.push(BoardCell::InputB),
                    " " => {}
                    v => {
                        let data = v.parse().unwrap();
                        if !(-99..=99).contains(&data) {
                            panic!("invalid data: {}", data);
                        }
                        row.push(BoardCell::SimCell(Cell::Data(data)));
                    }
                };
            }
            board.push(row);
        }
        Self { board }
    }

    pub fn save(&self) -> String {
        let mut s = String::new();
        for row in &self.board {
            for cell in row {
                let c = match cell {
                    BoardCell::SimCell(Cell::MoveLeft) => "<",
                    BoardCell::SimCell(Cell::MoveRight) => ">",
                    BoardCell::SimCell(Cell::MoveUp) => "^",
                    BoardCell::SimCell(Cell::MoveDown) => "v",
                    BoardCell::SimCell(Cell::Add) => "+",
                    BoardCell::SimCell(Cell::Subtract) => "-",
                    BoardCell::SimCell(Cell::Multiply) => "*",
                    BoardCell::SimCell(Cell::Divide) => "/",
                    BoardCell::SimCell(Cell::Modulo) => "%",
                    BoardCell::SimCell(Cell::Equal) => "=",
                    BoardCell::SimCell(Cell::NotEqual) => "#",
                    BoardCell::SimCell(Cell::TimeWarp) => "@",
                    BoardCell::SimCell(Cell::Submit) => "S",
                    BoardCell::SimCell(Cell::Data(data)) => &format!("{}", data),
                    BoardCell::InputA => "A",
                    BoardCell::InputB => "B",
                    BoardCell::Empty => ".",
                };
                s.push_str(c);
                s.push(' ');
            }
            s.push('\n');
        }
        s
    }
}
