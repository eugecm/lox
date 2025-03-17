use std::path::Path;

use clap::Parser;
use eyre::Result;

mod builtins;
mod callable;
mod environment;
mod eval;
mod interpreter;
mod scanner;
mod syntax;
mod types;
use interpreter::Interpreter;
use scanner::Scanner;

#[derive(Parser, Debug)]
#[command(version, about, long_about = None)]
struct Args {
    filename: String,
}

fn main() -> Result<()> {
    let args = Args::parse();

    let input_file = Path::new(&args.filename);

    run_file(input_file)?;
    Ok(())
}

fn run_file<P: AsRef<Path>>(input_file: P) -> Result<()> {
    let contents = std::fs::read_to_string(input_file)?;
    let scanner = Scanner::new(&contents);
    let mut parser = syntax::Parser::new(scanner.scan_tokens().map(|t| t.unwrap()));
    let ast = parser.parse();
    let mut interpreter = Interpreter::new();
    interpreter.interpret(ast);

    Ok(())
}
