/* error.rs
 *
 */

use crate::prelude::*;

pub enum Error {
    ScannerError { message: String, line: usize, col: i64 },

    WrongToken { message: String },

    MissingExpression { message: String },

    MissingPrimary { message: String },
}
