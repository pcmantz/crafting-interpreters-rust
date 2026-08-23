/* expr.rs
 *
 */

use crate::prelude::*;

use crate::token::*;

#[derive(Debug, Clone)]
pub enum Expr {
    Literal(LiteralExpr),
    Unary(UnaryExpr),
    Binary(BinaryExpr),
    Grouping(GroupingExpr),
}

impl Expr {
    pub fn literal(value: Literal) -> Expr {
        Expr::Literal(LiteralExpr { value })
    }

    pub fn unary(operator: Token, expr: Expr) -> Expr {
        Expr::Unary(UnaryExpr {
            operator,
            right: Box::new(expr),
        })
    }

    pub fn binary(left: Expr, operator: Token, right: Expr) -> Expr {
        Expr::Binary(BinaryExpr {
            left: Box::new(left),
            operator,
            right: Box::new(right),
        })
    }

    pub fn grouping(expr: Expr) -> Expr {
        Expr::Grouping(GroupingExpr {
            expression: Box::new(expr),
        })
    }
}

#[derive(Debug, Clone)]
pub struct LiteralExpr {
    pub value: Literal,
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
pub struct GroupingExpr {
    pub expression: Box<Expr>,
}

impl fmt::Display for Expr {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Expr::Literal(e) => write!(f, "{}", e.value),
            Expr::Unary(e) => write!(f, "({} {})", e.operator, e.right),
            Expr::Binary(e) => write!(f, "({} {} {})", e.operator, e.left, e.right),
            Expr::Grouping(e) => write!(f, "(group {})", e.expression),
        }
    }
}
