/* src/bin/lox.rs
 *
 */

use std::path::PathBuf;

use clap::Parser;
pub use color_eyre::{Context, Result};
use itertools::Itertools;

use crafting_interpreters::parser;
use crafting_interpreters::scanner;

#[derive(clap::Parser)] // requires `derive` feature
struct LoxAst {
    file: PathBuf,
}

impl LoxAst {
    fn run(self) -> Result<()> {
        let code = std::fs::read_to_string(self.file)?;
        let tokens = scanner::scan(code)?;

        print!("TOKENS: {}\n", tokens.iter().join(", "));

        let ast = parser::parse(tokens.clone())?;

        print!("AST: {}\n", ast);

        Ok(())
    }
}

fn main() -> Result<()> {
    let lox = LoxAst::parse();

    lox.run()
}
