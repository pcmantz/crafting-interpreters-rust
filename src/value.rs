/* value.rs
 *
 */

use crate::prelude::*;

use crate::function::*;
use crate::token::*;

#[derive(Debug, Clone)]
pub enum Value {
    Fun(Rc<Function>),
    Str(String),
    Num(f64),
    Bool(bool),
    Nil,
}

impl fmt::Display for Value {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Value::Fun(fun) => write!(f, "<fun {}>", fun.statement.name),
            Value::Str(str) => write!(f, "{str}"),
            Value::Num(n) => write!(f, "{n}"),
            Value::Bool(true) => write!(f, "true"),
            Value::Bool(false) => write!(f, "false"),
            Value::Nil => write!(f, "nil"),
        }
    }
}

impl PartialEq for Value {
    fn eq(&self, other: &Self) -> bool {
        match (self, other) {
            (Value::Bool(s), Value::Bool(o)) => s == o,
            (Value::Num(s), Value::Num(o)) => s == o,
            (Value::Str(s), Value::Str(o)) => s == o,
            (Value::Fun(s), Value::Fun(o)) => Rc::ptr_eq(&s, o),
            (Value::Nil, Value::Nil) => true,
            _ => false,
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
