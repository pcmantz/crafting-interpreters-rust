/* src/bin/lox.rs
 *
 */

use std::io;
use std::io::Write;
use std::path::PathBuf;

use clap::Parser;
use color_eyre::eyre::Result;

use crafting_interpreters::*;

#[derive(clap::Parser)] // requires `derive` feature
struct Cli {
    file: Option<PathBuf>,
}

#[derive(Default)]
struct Lox {
    interpreter: Interpreter,
}

impl Lox {
    fn run(mut self, args: Cli) -> Result<()> {
        if let Some(file) = args.file {
            self.run_file(file)
        } else {
            self.run_prompt()
        }
    }

    fn run_file(&mut self, file: PathBuf) -> Result<()> {
        let code = std::fs::read_to_string(&file)?;
        self.run_code(code)?;

        Ok(())
    }

    fn run_prompt(&mut self) -> Result<()> {
        std::io::stdout().flush().expect("Oops");

        loop {
            print!("❯ ");
            std::io::stdout().flush()?;

            let mut input = String::new();
            let n = io::stdin().read_line(&mut input)?;
            if n == 0 {
                break;
            }

            match self.run_code(input.trim_end().to_string()) {
                Ok(value) => println!("{value}"),
                Err(report) => eprintln!("{report:?}"),
            }
        }

        Ok(())
    }

    fn run_code(&mut self, code: String) -> color_eyre::Result<Value> {
        let tokens = scanner::scan(code)?;
        let statements = parser::parse(tokens)?;

        Ok(self.interpreter.run(statements)?)
    }
}

fn main() -> Result<()> {
    let _ = color_eyre::install();

    let args = Cli::parse();
    let lox = Lox::default();

    lox.run(args)
}
