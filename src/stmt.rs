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
        write!(f, "{}", self.0.iter().join(""))
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
    If(IfStmt),
    Return(ReturnStmt),
    While(WhileStmt),
    Function(FunctionStmt),
    Break(BreakStmt),
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
            Stmt::Block(b) => write!(f, "(block {})", b.statements.iter().join("")),
            Stmt::If(s) => {
                write!(f, "(if {} {}", &s.condition, &s.then_branch)?;
                if let Some(e) = &s.else_branch {
                    write!(f, " {}", e)?;
                }
                write!(f, ")")
            }
            Stmt::Return(s) => {
                write!(f, "(return")?;
                if let Some(e) = &s.value {
                    write!(f, " {}", e)?;
                }
                write!(f, ")")
            }

            Stmt::While(s) => write!(f, "(while {} {})", &s.condition, &s.body),
            Stmt::Function(s) => write!(
                f,
                "(fun {} ({}) {})",
                s.name.lexeme,
                s.params.iter().map(|p| &p.lexeme).join(" "),
                s.statements.iter().join(""),
            ),
            Stmt::Break(s) => write!(f, "(break)"),
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

    pub fn block(statements: Vec<Stmt>) -> Stmt {
        Stmt::Block(BlockStmt { statements })
    }

    pub fn r#if(condition: Expr, then_branch: Stmt, else_branch: Option<Stmt>) -> Stmt {
        Stmt::If(IfStmt {
            condition,
            then_branch: Box::new(then_branch),
            else_branch: match else_branch {
                Some(stmt) => Some(Box::new(stmt)),
                _ => None,
            },
        })
    }

    pub fn r#return(keyword: Token, value: Option<Expr>) -> Stmt {
        Stmt::Return(ReturnStmt { keyword, value })
    }

    pub fn r#while(condition: Expr, body: Stmt) -> Stmt {
        Stmt::While(WhileStmt {
            condition,
            body: Box::new(body),
        })
    }

    pub fn function(name: Token, params: Vec<Token>, statements: Vec<Stmt>) -> Stmt {
        Stmt::Function(FunctionStmt {
            name,
            params,
            statements,
        })
    }

    pub fn r#break(keyword: Token) -> Stmt {
        Stmt::Break(BreakStmt { keyword })
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

#[derive(Debug, Clone)]
pub struct IfStmt {
    pub condition: Expr,
    pub then_branch: Box<Stmt>,
    pub else_branch: Option<Box<Stmt>>,
}

#[derive(Debug, Clone)]
pub struct ReturnStmt {
    pub keyword: Token,
    pub value: Option<Expr>,
}

#[derive(Debug, Clone)]
pub struct WhileStmt {
    pub condition: Expr,
    pub body: Box<Stmt>,
}

#[derive(Debug, Clone)]
pub struct FunctionStmt {
    pub name: Token,
    pub params: Vec<Token>,
    pub statements: Vec<Stmt>,
}

#[derive(Debug, Clone)]
pub struct BreakStmt {
    pub keyword: Token,
}
