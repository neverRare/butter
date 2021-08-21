use structopt::StructOpt;

/// Butter compiler
#[derive(StructOpt, Debug, Clone, Copy, PartialEq, Eq)]
#[structopt(name = "butter")]
enum Command {
    /// Start a repl for type inference
    TypeRepl
}
fn main() {
    let opt = Command::from_args();
    // println!("Hello, world!");
}
