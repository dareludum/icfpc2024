use std::io::{stdin, Read, Write};

use icfp::evaluate;
use icfp::parse;
use icfp::serialize_str;
use icfp::Token;
use icfp::Value;
use logos::Logos;
use text_io::read;

mod comms;
mod geometry;
mod icfp;
// TODO: Remove when fixed
#[allow(dead_code)]
#[allow(clippy::all)]
mod lambdaman;
mod lambdaman_alt;
mod lasm;
mod runner;
mod spaceship;
mod three_d;

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
    Compile(CompileCommand),
    Solve(runner::SolveCommand),
    ThreeD(three_d::ThreeDCommand),
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

    #[argh(switch, short = 'r')]
    /// print raw token values (no newline, no quotes, etc.)
    raw: bool,
}

#[derive(FromArgs, PartialEq, Debug)]
/// Compile a lambdasm program
#[argh(subcommand, name = "compile")]
struct CompileCommand {
    #[argh(positional)]
    /// the program in a file
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
    #[argh(switch, short = 'r')]
    /// whether to print the raw response or try to parse it
    raw_response: bool,
}

fn main() -> std::io::Result<()> {
    let args: CliArgs = argh::from_env();
    match args.subcommand {
        CliSubcommands::Eval(EvalCommand {
            program,
            print,
            file,
            output,
            raw,
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
                ast.pretty_print(outstream)?;
            } else {
                let res = icfp::evaluate(ast);
                if raw {
                    match res {
                        Value::Bool(b) => write!(outstream, "{}", b)?,
                        Value::Int(i) => write!(outstream, "{}", i)?,
                        Value::Str(s) => write!(outstream, "{}", s)?,
                    }
                } else {
                    writeln!(outstream, "{}", res)?;
                }
            }
        }
        CliSubcommands::Comm(cmd) => {
            if let Some(message) = cmd.message {
                send_receive_single_command(message, cmd.raw_response, false);
            } else {
                loop {
                    print!("icfp> ");
                    std::io::stdout().flush().unwrap();
                    let message: String = read!("{}\n");
                    send_receive_single_command(message, cmd.raw_response, true);
                    std::io::stdout().flush().unwrap();
                }
            }
        }
        CliSubcommands::Compile(CompileCommand { program, output }) => {
            // read the program input
            let program = if let Some(program) = program {
                std::fs::read_to_string(program)?
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

            // parse the program
            let program = match lasm::parse(&program) {
                Ok(res) => res,
                Err(err) => {
                    eprintln!(
                        "Parsing failed:\n{}",
                        nom::error::convert_error(program.as_str(), err)
                    );
                    std::process::exit(1);
                }
            };

            // compile and write the result
            let res = lasm::compile(program);
            let bin = serialize_str(res);
            writeln!(outstream, "{bin}")?;
        }
        CliSubcommands::Solve(cmd) => cmd.run(),
        CliSubcommands::ThreeD(cmd) => cmd.run(),
    };
    Ok(())
}

fn send_receive_single_command(command: String, print_raw_response: bool, add_newline: bool) {
    match comms::send_string(command) {
        Some(response) => {
            print_response(response, print_raw_response, add_newline);
        }
        None => eprintln!("Failed to send message"),
    }
}

pub fn print_response(response: String, print_raw_response: bool, add_newline: bool) {
    if print_raw_response {
        println!("{}", response);
        return;
    }
    let node = parse(&mut Token::lexer(&response)).expect("Failed to parse response");
    match evaluate(node) {
        Value::Bool(b) => print!("{}", b),
        Value::Int(i) => print!("{}", i),
        Value::Str(s) => print!("{}", s),
    }
    if add_newline {
        println!();
    }
}
