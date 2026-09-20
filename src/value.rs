/* value.rs
 *
 */

use crate::prelude::*;

use crate::environment::*;
use crate::error::*;
use crate::function::*;
use crate::stmt::*;
use crate::token::*;

#[derive(Debug, Clone)]
pub enum CallableKind {
    User(Function),
    Native(NativeFunction),
}

impl Callable for CallableKind {
    fn arity(&self) -> usize {
        match self {
            CallableKind::User(f) => f.arity(),
            CallableKind::Native(f) => f.arity(),
        }
    }

    fn call(&self, env: &Environment, args: &[Value]) -> Result<Value, Error> {
        match self {
            CallableKind::User(f) => f.call(env, args),
            CallableKind::Native(f) => f.call(env, args),
        }
    }
}

impl CallableKind {
    fn name(&self) -> &str  {
        match self {
            CallableKind::User(f) => &f.statement.name.lexeme,
            CallableKind::Native(f) => &f.name,
        }
    }
}

#[derive(Debug, Clone)]
pub enum Value {
    Fun(Rc<CallableKind>),
    Str(String),
    Num(f64),
    Bool(bool),
    Nil,
}

impl fmt::Display for Value {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Value::Fun(fun) => write!(f, "<fun {}>", fun.name()),
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

    pub fn function(statement: FunctionStmt) -> Value {
        let func = Function { statement: Rc::new(statement) };
        let kind = CallableKind::User(func);

        Value::Fun(Rc::new(kind))
    }

    pub fn native_function(
        name: String,
        arity: usize,
        fun: fn(&[Value]) -> Result<Value, Error>,
    ) -> Value {
        let func = NativeFunction { name, arity, fun };
        let kind = CallableKind::Native(func);

        Value::Fun(Rc::new(kind))
    }
}
