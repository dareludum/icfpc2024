use argh::FromArgs;

use crate::three_d::gui::gui_main;

use super::{board::ThreeDBoard, sim::ThreeDSimulator};

#[derive(FromArgs, PartialEq, Debug)]
/// Evaluate a program
#[argh(subcommand, name = "3d")]
pub struct ThreeDCommand {
    #[argh(positional)]
    /// the program file path
    program_path: String,
    #[argh(positional)]
    /// the input value A
    a: i64,
    #[argh(positional)]
    /// the input value B
    b: i64,
    #[argh(switch, short = 'i')]
    /// run the GUI
    interactive: bool,
}

impl ThreeDCommand {
    pub fn run(&self) {
        let board_file =
            std::fs::read_to_string(&self.program_path).expect("Failed to read the board file");
        let board = ThreeDBoard::load(&board_file);
        println!("Initial board:\n{}", board.save());

        if self.interactive {
            return gui_main(board, self.a, self.b);
        }

        let mut sim = ThreeDSimulator::new(board, self.a, self.b);
        loop {
            let result = sim.step();
            println!("Board[t={}]:\n{}", sim.time(), sim.as_board().save());
            match result {
                Ok(Some(v)) => {
                    println!(
                        "Program finished successfully: {} (score={})",
                        v,
                        sim.score()
                    );
                    break;
                }
                Ok(None) => {}
                Err(pos) => {
                    println!("Error at position: {:?}", pos);
                    break;
                }
            }
        }
    }
}
