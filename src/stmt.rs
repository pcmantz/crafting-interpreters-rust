/* stmt.rs
 *
 */

use crate::prelude::*;

use crate::expr::*;
use crate::function::*;
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
pub struct Stmt {
    pub id: NodeId,
    pub span: Span,
    pub kind: StmtKind,
}

impl fmt::Display for Stmt {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        self.kind.fmt(f)
    }
}

#[derive(Debug, Clone)]
pub enum StmtKind {
    Class(ClassStmt),
    Function(FunctionStmt),
    Var(VarStmt),
    Expression(ExpressionStmt),
    Print(PrintStmt),
    Block(BlockStmt),
    If(IfStmt),
    Return(ReturnStmt),
    While(WhileStmt),
    Break(BreakStmt),
}

impl fmt::Display for StmtKind {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            StmtKind::Class(s) => write!(f, "(class {})", &s.name),
            StmtKind::Function(s) => write!(
                f,
                "(fun {} ({}) {})",
                s.name.lexeme,
                s.def.params.iter().map(|p| &p.lexeme).join(" "),
                s.def.statements.iter().join(""),
            ),
            StmtKind::Var(s) => match &s.initializer {
                Some(init) => write!(f, "(var {} {})", s.name.lexeme, init),
                None => write!(f, "(var {})", s.name.lexeme),
            },
            StmtKind::Expression(s) => write!(f, "(expr {})", &s.expression),
            StmtKind::Print(s) => write!(f, "(print {})", &s.expression),
            StmtKind::Block(b) => write!(f, "(block {})", b.statements.iter().join("")),
            StmtKind::If(s) => {
                write!(f, "(if {} {}", &s.condition, &s.then_branch)?;
                if let Some(e) = &s.else_branch {
                    write!(f, " {}", e)?;
                }
                write!(f, ")")
            }
            StmtKind::Return(s) => {
                write!(f, "(return")?;
                if let Some(e) = &s.value {
                    write!(f, " {}", e)?;
                }
                write!(f, ")")
            }

            StmtKind::While(s) => write!(f, "(while {} {})", &s.condition, &s.body),
            StmtKind::Break(_) => write!(f, "(break)"),
        }
    }
}

impl StmtKind {
    pub fn class(name: Token, methods: Vec<Stmt>) -> StmtKind {
        StmtKind::Class(ClassStmt { name, methods })
    }

    pub fn var(name: Token, initializer: Option<Expr>) -> StmtKind {
        StmtKind::Var(VarStmt { name, initializer })
    }

    pub fn function(name: Token, params: Vec<Token>, statements: Vec<Stmt>) -> StmtKind {
        StmtKind::Function(FunctionStmt {
            name,
            def: Rc::new(FunctionDef {
                kind: FunctionKind::Function,
                params,
                statements,
            }),
        })
    }

    pub fn method(name: Token, params: Vec<Token>, statements: Vec<Stmt>) -> StmtKind {
        StmtKind::Function(FunctionStmt {
            name,
            def: Rc::new(FunctionDef {
                kind: FunctionKind::Method,
                params,
                statements,
            }),
        })
    }

    pub fn expression(expression: Expr) -> StmtKind {
        StmtKind::Expression(ExpressionStmt { expression })
    }

    pub fn print(expression: Expr) -> StmtKind {
        StmtKind::Print(PrintStmt { expression })
    }

    pub fn block(statements: Vec<Stmt>) -> StmtKind {
        StmtKind::Block(BlockStmt { statements })
    }

    pub fn r#if(condition: Expr, then_branch: Stmt, else_branch: Option<Stmt>) -> StmtKind {
        StmtKind::If(IfStmt {
            condition,
            then_branch: Box::new(then_branch),
            else_branch: match else_branch {
                Some(stmt) => Some(Box::new(stmt)),
                _ => None,
            },
        })
    }

    pub fn r#return(keyword: Token, value: Option<Expr>) -> StmtKind {
        StmtKind::Return(ReturnStmt { keyword, value })
    }

    pub fn r#while(condition: Expr, body: Stmt) -> StmtKind {
        StmtKind::While(WhileStmt {
            condition,
            body: Box::new(body),
        })
    }

    pub fn r#break(keyword: Token) -> StmtKind {
        StmtKind::Break(BreakStmt { keyword })
    }
}

#[derive(Debug, Clone)]
pub struct ClassStmt {
    pub name: Token,
    pub methods: Vec<Stmt>,
}

#[derive(Debug, Clone)]
pub struct VarStmt {
    pub name: Token,
    pub initializer: Option<Expr>,
}

#[derive(Debug, Clone)]
pub struct FunctionStmt {
    pub name: Token,
    pub def: Rc<FunctionDef>,
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
pub struct BreakStmt {
    pub keyword: Token,
}
