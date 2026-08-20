/* src/bin/lox.rs
 *
 */

use std::io;
use std::io::Write;
use std::path::PathBuf;

use clap::Parser;
pub use color_eyre::{Context, Result};

use crafting_interpreters::error::*;
use crafting_interpreters::prelude::*;

use crafting_interpreters::parser;
use crafting_interpreters::scanner;

#[derive(clap::Parser)] // requires `derive` feature
struct LoxAst {
    file: PathBuf,
}

impl LoxAst {
    fn run(self) -> Result<()> {
        let code = std::fs::read_to_string(self.file)?;
        let tokens = scanner::scan_tokens(code)?;
        let ast = parser::parse(tokens)?;

        print!("{}", ast);

        Ok(())
    }
}

fn main() -> Result<()> {
    let lox = LoxAst::parse();

    lox.run()
}
