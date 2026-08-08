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
    path: Option<PathBuf>,
}

impl Lox {
    fn run(self) -> Result<()> {
        if let Some(path) = self.path {
            Lox::run_file(path)
        } else {
            Lox::run_prompt()
        }
    }

    fn run_file(path: PathBuf) -> Result<()> {
        let code = std::fs::read_to_string(&path)?;

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
            let resp = Lox::run_code(code)?;

            // print!("{}\n", resp);
            std::io::stdout().flush()?;
        }
    }

    fn run_code(code: String) -> Result<()> {
        Ok(())
    }
}

fn main() -> Result<()> {
    let args = Lox::parse();

    Lox::run(args)
}
