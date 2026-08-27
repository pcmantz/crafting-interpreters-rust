/* value.rs
 *
 */

use crate::prelude::*;

use crate::token::*;

#[derive(PartialEq, Debug, Clone)]
pub enum Value {
    Str(String),
    Num(f64),
    Bool(bool),
    Nil,
}

impl fmt::Display for Value {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Value::Str(str) => write!(f, "\"{str}\""),
            Value::Num(n) => write!(f, "{n}"),
            Value::Bool(true) => write!(f, "true"),
            Value::Bool(false) => write!(f, "false"),
            Value::Nil => write!(f, "nil"),
        }
    }
}

impl Value {
    pub fn from_token(token: Token) -> Option<Value> {
        match token.ty {
            TokenType::Str(str) => Some(Value::Str(str)),
            TokenType::Num(num) => Some(Value::Num(num)),
            TokenType::True => Some(Value::Bool(true)),
            TokenType::False => Some(Value::Bool(false)),
            TokenType::Nil => Some(Value::Nil),
            _ => None,
        }
    }
}
