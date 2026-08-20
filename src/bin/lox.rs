/* src/bin/lox.rs
 *
 */

use std::io;
use std::io::Write;
use std::path::PathBuf;

use clap::Parser;

use crafting_interpreters::prelude::*;

#[derive(clap::Parser)] // requires `derive` feature
struct Lox {
    file: Option<PathBuf>,
}

impl Lox {
    fn run(self) -> Result<()> {
        if let Some(file) = self.file {
            Lox::run_file(file)
        } else {
            Lox::run_prompt()
        }
    }

    fn run_file(file: PathBuf) -> Result<()> {
        let code = std::fs::read_to_string(&file)?;

        Lox::run_code(code)
    }

    fn run_prompt() -> Result<()> {
        std::io::stdout().flush().expect("Oops");

        loop {
            print!("❯ ");
            std::io::stdout().flush()?;

            let mut input = String::new();
            io::stdin().read_line(&mut input)?;
            let code = input.trim_end().to_string();

            Lox::run_code(code)?;
        }
    }

    fn run_code(code: String) -> Result<()> {
        todo!("Call the parser and do some things, I don't know.");
    }
}

fn main() -> Result<()> {
    let args = Lox::parse();

    Lox::run(args)
}
