/* expr.rs
 *
 */

use crate::prelude::*;

use crate::function::*;
use crate::stmt::*;
use crate::token::*;
use crate::value::*;

#[derive(Debug, Clone)]
pub struct Expr {
    pub id: u32,
    pub span: Span,
    pub kind: ExprKind,
}

impl fmt::Display for Expr {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        self.kind.fmt(f)
    }
}

#[derive(Debug, Clone)]
pub enum ExprKind {
    Literal(LiteralExpr),
    Logical(LogicalExpr),
    Variable(VariableExpr),
    Assign(AssignExpr),
    Unary(UnaryExpr),
    Binary(BinaryExpr),
    Call(CallExpr),
    Grouping(GroupingExpr),
    Function(FunctionExpr),
}

impl fmt::Display for ExprKind {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            ExprKind::Literal(e) => write!(f, "{}", e.value),
            ExprKind::Logical(e) => write!(f, "({} {} {})", e.operator.lexeme, e.left, e.right),
            ExprKind::Unary(e) => write!(f, "({} {})", e.operator.lexeme, e.right),
            ExprKind::Binary(e) => write!(f, "({} {} {})", e.operator.lexeme, e.left, e.right),
            ExprKind::Grouping(e) => write!(f, "(group {})", e.expression),
            ExprKind::Variable(e) => write!(f, "{}", e.name),
            ExprKind::Assign(e) => write!(f, "(= {} {})", e.name, e.expression),
            ExprKind::Call(e) => write!(f, "({} {})", e.callee, e.arguments.iter().join(" ")),
            ExprKind::Function(e) => write!(f, "(fn <anonymous>)"),
        }
    }
}

impl ExprKind {
    pub fn literal(value: Value) -> ExprKind {
        ExprKind::Literal(LiteralExpr { value })
    }

    pub fn logical(left: Expr, operator: Token, right: Expr) -> ExprKind {
        ExprKind::Logical(LogicalExpr {
            left: Box::new(left),
            operator,
            right: Box::new(right),
        })
    }

    pub fn unary(operator: Token, expr: Expr) -> ExprKind {
        ExprKind::Unary(UnaryExpr {
            operator,
            right: Box::new(expr),
        })
    }

    pub fn binary(left: Expr, operator: Token, right: Expr) -> ExprKind {
        ExprKind::Binary(BinaryExpr {
            left: Box::new(left),
            operator,
            right: Box::new(right),
        })
    }

    pub fn call(callee: Expr, paren: Token, arguments: Vec<Expr>) -> ExprKind {
        ExprKind::Call(CallExpr {
            callee: Box::new(callee),
            paren,
            arguments,
        })
    }

    pub fn grouping(expr: Expr) -> ExprKind {
        ExprKind::Grouping(GroupingExpr {
            expression: Box::new(expr),
        })
    }

    pub fn variable(name: Token) -> ExprKind {
        ExprKind::Variable(VariableExpr { name })
    }

    pub fn assign(name: Token, expr: Expr) -> ExprKind {
        ExprKind::Assign(AssignExpr {
            name,
            expression: Box::new(expr),
        })
    }

    pub fn function(params: Vec<Token>, statements: Vec<Stmt>) -> ExprKind {
        ExprKind::Function(FunctionExpr {
            def: Rc::new(FunctionDef { params, statements }),
        })
    }
}

#[derive(Debug, Clone)]
pub struct LiteralExpr {
    pub value: Value,
}

#[derive(Debug, Clone)]
pub struct LogicalExpr {
    pub left: Box<Expr>,
    pub operator: Token,
    pub right: Box<Expr>,
}

#[derive(Debug, Clone)]
pub struct UnaryExpr {
    pub operator: Token,
    pub right: Box<Expr>,
}

#[derive(Debug, Clone)]
pub struct BinaryExpr {
    pub left: Box<Expr>,
    pub operator: Token,
    pub right: Box<Expr>,
}

#[derive(Debug, Clone)]
pub struct CallExpr {
    pub callee: Box<Expr>,
    pub paren: Token,
    pub arguments: Vec<Expr>,
}

#[derive(Debug, Clone)]
pub struct GroupingExpr {
    pub expression: Box<Expr>,
}

#[derive(Debug, Clone)]
pub struct VariableExpr {
    pub name: Token,
}

#[derive(Debug, Clone)]
pub struct AssignExpr {
    pub name: Token,
    pub expression: Box<Expr>,
}

#[derive(Debug, Clone)]
pub struct FunctionExpr {
    pub def: Rc<FunctionDef>,
}
