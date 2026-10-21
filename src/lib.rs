/* src/lib.rs
 *
 */

pub use eyre;

pub mod prelude;

/* utility */
pub mod pipe;

/* data types */
pub mod error;
pub mod expr;
pub mod function;
pub mod native;
pub mod stmt;
pub mod token;
pub mod value;

/* logic */
pub mod environment;
pub mod interpreter;
pub mod parser;
pub mod resolver;
pub mod scanner;

pub use error::Error;
pub use interpreter::Interpreter;
pub use stmt::Program;
pub use value::Value;

pub fn run(interpreter: &mut Interpreter, source: &str) -> eyre::Result<Value> {
    let tokens = scanner::scan(source.to_string())?;
    let statements = parser::parse(tokens)?;

    resolver::resolve(interpreter, &statements)?;

    Ok(interpreter.run(statements)?)
}
