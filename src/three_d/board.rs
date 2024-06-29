use super::sim::Cell;

// NOTE: This struct is only used for loading/saving the initial board state,
// see sim.rs for the simulation state and logic.
#[derive(Debug, Clone, Default)]
pub struct ThreeDBoard {
    pub board: Vec<Vec<Option<Cell>>>,
}

impl ThreeDBoard {
    pub fn load(s: &str) -> Self {
        let mut board = Vec::new();
        for line in s.lines() {
            let mut row = vec![];
            for cell in line.split_ascii_whitespace() {
                match cell {
                    "." => row.push(None),
                    "<" => row.push(Some(Cell::MoveLeft)),
                    ">" => row.push(Some(Cell::MoveRight)),
                    "^" => row.push(Some(Cell::MoveUp)),
                    "v" => row.push(Some(Cell::MoveDown)),
                    "+" => row.push(Some(Cell::Add)),
                    "-" => row.push(Some(Cell::Subtract)),
                    "*" => row.push(Some(Cell::Multiply)),
                    "/" => row.push(Some(Cell::Divide)),
                    "%" => row.push(Some(Cell::Modulo)),
                    "=" => row.push(Some(Cell::Equal)),
                    "#" => row.push(Some(Cell::NotEqual)),
                    "@" => row.push(Some(Cell::TimeWarp)),
                    "S" => row.push(Some(Cell::Submit)),
                    "A" => row.push(Some(Cell::InputA)),
                    "B" => row.push(Some(Cell::InputB)),
                    " " => {}
                    v => {
                        let data = v.parse().unwrap();
                        if !(-99..=99).contains(&data) {
                            panic!("invalid data: {}", data);
                        }
                        row.push(Some(Cell::Data(data)));
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
                    Some(Cell::MoveLeft) => "<",
                    Some(Cell::MoveRight) => ">",
                    Some(Cell::MoveUp) => "^",
                    Some(Cell::MoveDown) => "v",
                    Some(Cell::Add) => "+",
                    Some(Cell::Subtract) => "-",
                    Some(Cell::Multiply) => "*",
                    Some(Cell::Divide) => "/",
                    Some(Cell::Modulo) => "%",
                    Some(Cell::Equal) => "=",
                    Some(Cell::NotEqual) => "#",
                    Some(Cell::TimeWarp) => "@",
                    Some(Cell::Submit) => "S",
                    Some(Cell::Data(data)) => &format!("{}", data),
                    Some(Cell::InputA) => "A",
                    Some(Cell::InputB) => "B",
                    None => ".",
                };
                s.push_str(c);
                s.push(' ');
            }
            s.push('\n');
        }
        s
    }
}
