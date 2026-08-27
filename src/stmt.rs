/* stmt.rs
 *
 */

use crate::prelude::*;

use crate::expr::*;

pub enum Stmt {
    Expression(ExpressionStmt),
    Print(PrintStmt),
}

// impl fmt::Display for Stmt {
//     fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
//         todo!("I'll get to this later")
//     }
// }

impl Stmt {
    pub fn expression(expression: Expr) -> Stmt {
        Stmt::Expression(ExpressionStmt {
            expression: Box::new(expression),
        })
    }

    pub fn print(expression: Expr) -> Stmt {
        Stmt::Print(PrintStmt {
            expression: Box::new(expression),
        })
    }
}

pub struct ExpressionStmt {
    pub expression: Box<Expr>,
}

pub struct PrintStmt {
    pub expression: Box<Expr>,
}
