use clap::Parser;
use hir::expr::Expr;
use parser::{expr_parser, EasyParser};
use std::io::{self, Write};
use type_system::test_infer;

/// Butter compiler
#[derive(Parser, Debug, Clone, Copy, PartialEq, Eq)]
enum Command {
    /// Start a repl for testing type inference
    TypeRepl,
    /// Start a repl for testing parser
    ParserRepl,
}
fn main() {
    match Command::parse() {
        Command::TypeRepl => type_repl().unwrap(),
        Command::ParserRepl => parser_repl().unwrap(),
    }
}
fn type_repl() -> io::Result<()> {
    let mut stdout = io::stdout();
    let stdin = io::stdin();
    println!("Butter Type Inference Repl");
    println!();
    println!("Enter expression to infer type. Enter :q to exit.");
    loop {
        let mut input = String::new();
        print!("> ");
        stdout.flush()?;
        stdin.read_line(&mut input)?;
        if input.is_empty() || input.starts_with(":q") {
            break;
        }
        let ast: Expr<()> = match expr_parser().easy_parse(&input[..]) {
            Ok((ast, _)) => ast,
            Err(err) => {
                eprintln!("{}", err);
                continue;
            }
        };
        let ty = match test_infer(ast) {
            Ok(ty) => ty,
            Err(err) => {
                eprintln!("{:?}", err);
                continue;
            }
        };
        println!("type: {}", ty);
    }
    Ok(())
}
fn parser_repl() -> io::Result<()> {
    let mut stdout = io::stdout();
    let stdin = io::stdin();
    println!("Butter Parser Repl");
    println!();
    println!("Enter expression to parse. Use :{{ and :}} to enter multiline block expression.");
    println!("Enter :q to exit.");
    let mut input = String::new();
    let mut multiline_input = String::new();
    loop {
        input.clear();
        print!("> ");
        stdout.flush()?;
        stdin.read_line(&mut input)?;
        if input.is_empty() || input.starts_with(":q") {
            break;
        } else if input.starts_with(":{") {
            input.clear();
            input.push_str("{\n");
            loop {
                multiline_input.clear();
                print!("... ");
                stdout.flush()?;
                stdin.read_line(&mut multiline_input)?;
                if multiline_input.starts_with(":}") {
                    input.push_str("}\n");
                    break;
                } else {
                    input.push_str(&multiline_input);
                }
            }
        }
        match expr_parser::<(), _>().easy_parse(&input[..]) {
            Ok((ast, _)) => {
                println!("{:#?}", ast);
            }
            Err(err) => {
                eprintln!("{}", err);
                continue;
            }
        };
    }
    Ok(())
}
