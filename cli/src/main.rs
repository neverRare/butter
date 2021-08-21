use combine::EasyParser;
use parser::expr_parser;
use std::io;
use std::io::Write;
use structopt::StructOpt;
use type_system::test_infer;

/// Butter compiler
#[derive(StructOpt, Debug, Clone, Copy, PartialEq, Eq)]
#[structopt(name = "butter")]
enum Command {
    /// Start a repl for type inference
    TypeRepl,
}
fn main() {
    match Command::from_args() {
        Command::TypeRepl => repl().unwrap(),
    }
}
fn repl() -> io::Result<()> {
    let mut stdout = io::stdout();
    let stdin = io::stdin();
    println!("Press Ctrl+D to exit");
    loop {
        let mut input = String::new();
        print!("> ");
        stdout.flush()?;
        stdin.read_line(&mut input)?;
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
}
