use combine::EasyParser;
use parser::expr_parser;
use std::io::{self, Write};
use structopt::StructOpt;
use type_system::test_infer;

/// Butter compiler
#[derive(StructOpt, Debug, Clone, Copy, PartialEq, Eq)]
#[structopt(name = "butter")]
enum Command {
    /// Start a repl for testing type inference
    TypeRepl,
    /// Start a repl for testing parser
    ParserRepl,
}
fn main() {
    match Command::from_args() {
        Command::TypeRepl => type_repl().unwrap(),
        Command::ParserRepl => parser_repl().unwrap(),
    }
}
fn type_repl() -> io::Result<()> {
    let mut stdout = io::stdout();
    let stdin = io::stdin();
    println!("Butter Type Inference Repl");
    println!("");
    println!("Enter expression to infer type. Enter :q to exit.");
    loop {
        let mut input = String::new();
        print!("> ");
        stdout.flush()?;
        stdin.read_line(&mut input)?;
        if input.is_empty() || input.starts_with(":q") {
            break;
        }
        let ast = match expr_parser::<_, ()>().easy_parse(&input[..]) {
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
    println!("");
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
            input.extend("{\n".chars());
            loop {
                multiline_input.clear();
                print!("... ");
                stdout.flush()?;
                stdin.read_line(&mut multiline_input)?;
                if multiline_input.starts_with(":}") {
                    input.extend("}\n".chars());
                    break;
                } else {
                    input.extend(multiline_input.chars());
                }
            }
        }
        match expr_parser::<_, ()>().easy_parse(&input[..]) {
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
