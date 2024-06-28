use std::io::Write;

use lexer::Token;
use logos::Logos;
use parser::parse;
use text_io::read;

mod ast;
mod comms;
mod eval;
mod lexer;
mod parser;

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
    AstPrint(AstPrintCommand),
    Comm(CommCommand),
}

#[derive(FromArgs, PartialEq, Debug)]
/// Evaluate a program
#[argh(subcommand, name = "eval")]
struct EvalCommand {
    #[argh(positional)]
    /// the program
    program: String,
}

#[derive(FromArgs, PartialEq, Debug)]
/// Print the ast of a program
#[argh(subcommand, name = "ast-print")]
struct AstPrintCommand {
    #[argh(positional)]
    /// the program
    program: String,
}

#[derive(FromArgs, PartialEq, Debug)]
/// Communicate
#[argh(subcommand, name = "comm")]
struct CommCommand {}

fn main() {
    let args: CliArgs = argh::from_env();
    match args.subcommand {
        CliSubcommands::Eval(EvalCommand { program }) => {
            todo!()
        }
        CliSubcommands::AstPrint(AstPrintCommand { program }) => {
            let mut lexer = Token::lexer(&program);
            let ast = parse(&mut lexer).unwrap();
            ast.print();
        }
        CliSubcommands::Comm(_) => loop {
            print!("icfp> ");
            std::io::stdout().flush().unwrap();
            let message: String = read!("{}\n");
            match comms::send(message) {
                Some(response) => {
                    std::io::stdout().flush().unwrap();
                    let tokens = lexer::Token::lexer(&response)
                        .collect::<Result<Vec<_>, _>>()
                        .expect("Failed to lex response");
                    if tokens.len() == 1 {
                        if let lexer::Token::String(s) = &tokens[0] {
                            println!("{}", s);
                        } else {
                            println!("Single raw token: {:?}", tokens[0]);
                        }
                    } else {
                        println!("{}", response);
                    }
                }
                None => println!("Failed to send message"),
            }
        },
    }
}
