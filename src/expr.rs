/* expr.rs
 *
 */

use crate::prelude::*;

use crate::token::*;

pub enum Expr {
    Literal(LiteralExpr),
    Unary(UnaryExpr),
    Binary(BinaryExpr),
    Grouping(GroupingExpr),
}

pub struct LiteralExpr {
    pub value: Literal,
}

pub struct UnaryExpr {
    pub operator: Token,
    pub right: Box<Expr>,
}

pub struct BinaryExpr {
    pub left: Box<Expr>,
    pub operator: Token,
    pub right: Box<Expr>,
}

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
