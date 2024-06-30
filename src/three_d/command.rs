use std::path::PathBuf;

use argh::FromArgs;

use crate::three_d::{gui::gui_main, sim::SimulationStepResult};

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
    #[argh(switch, short = 'q')]
    /// quiet mode, only print the answer
    quiet: bool,
}

impl ThreeDCommand {
    pub fn run(&self) {
        if self.interactive {
            return gui_main(Some(PathBuf::from(&self.program_path)), self.a, self.b);
        }

        let board_file =
            std::fs::read_to_string(&self.program_path).expect("Failed to read the board file");
        let board = ThreeDBoard::load(&board_file);
        if !self.quiet {
            println!("Initial board:\n{}", board.save());
        }

        let mut sim = ThreeDSimulator::new(board, self.a, self.b);
        loop {
            let result = sim.step();
            if !self.quiet {
                println!("Board[t={}]:\n{}", sim.time(), sim.as_board().save());
            }
            match result {
                SimulationStepResult::Ok => {}
                SimulationStepResult::Finished(v) => {
                    if self.quiet {
                        println!("{}", v);
                    } else {
                        println!(
                            "Program finished successfully: {} (score={})",
                            v,
                            sim.score()
                        );
                    }
                    break;
                }
                SimulationStepResult::AlreadyFinished => unreachable!(),
                SimulationStepResult::Error(pos) => {
                    println!("Error at position: {:?}", pos);
                    break;
                }
            }
        }
    }
}
