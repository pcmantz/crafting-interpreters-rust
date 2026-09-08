/* src/bin/lox_scan.rs
 *
 */

use std::path::PathBuf;

use clap::Parser;
pub use color_eyre::{Context, Result};
use itertools::Itertools;

use crafting_interpreters::scanner;

#[derive(clap::Parser)] // requires `derive` feature
struct LoxScan {
    file: PathBuf,
}

impl LoxScan {
    fn run(self) -> Result<()> {
        let code = std::fs::read_to_string(self.file)?;
        let tokens = scanner::scan(code)?;

        for token in tokens {
            print!("{} ", token);
        }

        Ok(())
    }
}

fn main() -> Result<()> {
    let lox = LoxScan::parse();

    lox.run()
}
