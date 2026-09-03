/* src/lib.rs
 *
 */

pub mod prelude;

/* utility */
pub mod pipe;

/* data types */
pub mod error;
pub mod expr;
pub mod stmt;
pub mod token;
pub mod value;

/* logic */
pub mod environment;
pub mod parser;
pub mod scanner;
pub mod interpreter;

pub use error::Error;
pub use interpreter::Interpreter;
pub use value::Value;
pub use stmt::Program;
