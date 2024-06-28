use std::io::Write;

use logos::Logos;
use text_io::read;

mod ast;
mod comms;
mod lexer;
mod parser;

fn main() {
    println!("Team Dare Ludum @ ICFP Contest 2024");
    loop {
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
                        print!("{}", s);
                    } else {
                        println!("Single raw token: {:?}", tokens[0]);
                    }
                } else {
                    println!("{}", response);
                }
            }
            None => println!("Failed to send message"),
        }
    }
}
