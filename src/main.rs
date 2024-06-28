use std::io::{stdin, Read, Write};

use lexer::Token;
use logos::Logos;
use parser::parse;
use text_io::read;

mod ast;
mod comms;
mod eval;
mod lexer;
mod parser;
mod runner;

use argh::FromArgs;

#[derive(FromArgs, PartialEq, Debug)]
/// Team Dare Ludum @ ICFP Contest 2024
struct CliArgs {
    #[argh(subcommand)]
    subcommand: CliSubcommands,
}

#[derive(FromArgs, PartialEq, Debug)]
#[argh(subcommand)]
enum CliSubcommands {
    Eval(EvalCommand),
    Comm(CommCommand),
    Solve(runner::SolveCommand),
}

#[derive(FromArgs, PartialEq, Debug)]
/// Evaluate a program
#[argh(subcommand, name = "eval")]
struct EvalCommand {
    #[argh(switch, short = 'p')]
    /// print the program's ast
    print: bool,

    #[argh(switch, short = 'f')]
    /// the program is in a file
    file: bool,

    #[argh(positional)]
    /// the program, either directly as the argument of as a file path, if -f is given
    program: Option<String>,

    #[argh(option, short = 'o')]
    /// a file to write the output to
    output: Option<String>,
}

#[derive(FromArgs, PartialEq, Debug)]
/// Communicate
#[argh(subcommand, name = "comm")]
struct CommCommand {
    #[argh(option, short = 'm')]
    /// the message to send
    message: Option<String>,
}

fn main() -> std::io::Result<()> {
    let args: CliArgs = argh::from_env();
    match args.subcommand {
        CliSubcommands::Eval(EvalCommand {
            program,
            print,
            file,
            output,
        }) => {
            // read the program input
            let program = if let Some(program) = program {
                if file {
                    std::fs::read_to_string(program)?
                } else {
                    program
                }
            } else {
                let mut program = String::new();
                stdin().lock().read_to_string(&mut program)?;
                program
            };

            // setup the output file, if any
            let outstream: &mut dyn std::io::Write = if let Some(output) = output {
                &mut std::fs::File::create(output)?
            } else {
                &mut std::io::stdout().lock()
            };

            // parse the AST
            let mut lexer = Token::lexer(&program);
            let ast = parse(&mut lexer).unwrap();

            if print {
                ast.print(outstream)?;
            } else {
                let res = eval::evaluate(ast);
                writeln!(outstream, "{}", res)?;
            }
        }
        CliSubcommands::Comm(cmd) => {
            if let Some(message) = cmd.message {
                send_receive_single_command(message, false);
            } else {
                loop {
                    print!("icfp> ");
                    std::io::stdout().flush().unwrap();
                    let message: String = read!("{}\n");
                    send_receive_single_command(message, true);
                }
            }
        }
        CliSubcommands::Solve(cmd) => cmd.run(),
    };
    Ok(())
}

fn send_receive_single_command(command: String, add_newline: bool) {
    match comms::send(command) {
        Some(response) => {
            std::io::stdout().flush().unwrap();
            let tokens = lexer::Token::lexer(&response)
                .collect::<Result<Vec<_>, _>>()
                .expect("Failed to lex response");
            if tokens.len() == 1 {
                if let lexer::Token::String(s) = &tokens[0] {
                    if add_newline {
                        println!("{}", s);
                    } else {
                        print!("{}", s);
                    }
                } else {
                    eprintln!("Single raw token: {:?}", tokens[0]);
                }
            } else {
                if add_newline {
                    println!("{}", response);
                } else {
                    print!("{}", response);
                }
            }
        }
        None => eprintln!("Failed to send message"),
    }
}
