/* stmt.rs
 *
 */

use crate::prelude::*;

use crate::expr::*;
use crate::token::*;

#[derive(Debug, Clone)]
pub struct Program(pub Vec<Stmt>);

impl fmt::Display for Program {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{}", self.0.iter().join("\n"))
    }
}

impl IntoIterator for Program {
    type Item = Stmt;
    type IntoIter = std::vec::IntoIter<Stmt>;

    fn into_iter(self) -> Self::IntoIter {
        self.0.into_iter()
    }
}

impl<'a> IntoIterator for &'a Program {
    type Item = &'a Stmt;
    type IntoIter = std::slice::Iter<'a, Stmt>;

    fn into_iter(self) -> Self::IntoIter {
        self.0.iter()
    }
}

#[derive(Debug, Clone)]
pub enum Stmt {
    Expression(ExpressionStmt),
    Print(PrintStmt),
    Var(VarStmt),
    Block(BlockStmt),
}

impl fmt::Display for Stmt {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            Stmt::Expression(s) => write!(f, "(expr {})", &s.expression),
            Stmt::Print(s) => write!(f, "(print {})", &s.expression),
            Stmt::Var(s) => match &s.initializer {
                Some(init) => write!(f, "(var {} {})", s.name.lexeme, init),
                None => write!(f, "(var {})", s.name.lexeme),
            },
            Stmt::Block(b) => write!(f, "(block {})", b.statements.iter().join("\n"))
        }
    }
}

impl Stmt {
    pub fn expression(expression: Expr) -> Stmt {
        Stmt::Expression(ExpressionStmt { expression })
    }

    pub fn print(expression: Expr) -> Stmt {
        Stmt::Print(PrintStmt { expression })
    }

    pub fn var(name: Token, initializer: Option<Expr>) -> Stmt {
        Stmt::Var(VarStmt { name, initializer })
    }
}

#[derive(Debug, Clone)]
pub struct ExpressionStmt {
    pub expression: Expr,
}

#[derive(Debug, Clone)]
pub struct PrintStmt {
    pub expression: Expr,
}

#[derive(Debug, Clone)]
pub struct VarStmt {
    pub name: Token,
    pub initializer: Option<Expr>,
}

#[derive(Debug, Clone)]
pub struct BlockStmt {
    pub statements: Vec<Stmt>,
}
