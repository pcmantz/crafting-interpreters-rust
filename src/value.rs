/* value.rs
 *
 */

use crate::prelude::*;

use crate::error::*;
use crate::expr::*;
use crate::function::*;
use crate::lox_class::*;
use crate::stmt::*;
use crate::token::*;

use crate::interpreter::*;

#[derive(Debug, Clone)]
pub enum CallableKind {
    User(Function),
    Native(NativeFunction),
}

impl From<Function> for CallableKind {
    fn from(source: Function) -> Self {
        Self::User(source)
    }
}

impl From<NativeFunction> for CallableKind {
    fn from(source: NativeFunction) -> Self {
        Self::Native(source)
    }
}

impl Callable for CallableKind {
    fn arity(&self) -> usize {
        match self {
            CallableKind::User(f) => f.arity(),
            CallableKind::Native(f) => f.arity(),
        }
    }

    fn call(&self, interpreter: &mut Interpreter, args: &[Value]) -> Result<Value, Error> {
        match self {
            CallableKind::User(f) => f.call(interpreter, args),
            CallableKind::Native(f) => f.call(interpreter, args),
        }
    }
}

impl CallableKind {
    fn name(&self) -> &str {
        match self {
            CallableKind::User(f) => {
                let name = &f.name;

                match name {
                    Some(val) => val,
                    None => "<anonymous>",
                }
            }

            CallableKind::Native(f) => &f.name,
        }
    }
}

#[derive(Debug, Clone)]
pub enum Value {
    Class(Rc<LoxClass>),
    Fun(Rc<CallableKind>),
    Instance(Rc<LoxInstance>),
    Str(String),
    Num(f64),
    Bool(bool),
    Nil,
}

impl fmt::Display for Value {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Value::Class(klass) => write!(f, "<class {}>", klass.name),
            Value::Fun(fun) => write!(f, "<fun {}>", fun.name()),
            Value::Instance(inst) => write!(f, "<instance {}>", inst.klass.name),
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
            (Value::Class(s), Value::Class(o)) => Rc::ptr_eq(s, o),
            (Value::Fun(s), Value::Fun(o)) => Rc::ptr_eq(s, o),
            (Value::Instance(s), Value::Instance(o)) => Rc::ptr_eq(s, o),
            (Value::Str(s), Value::Str(o)) => s == o,
            (Value::Num(s), Value::Num(o)) => s == o,
            (Value::Bool(s), Value::Bool(o)) => s == o,
            (Value::Nil, Value::Nil) => true,
            _ => false,
        }
    }
}

impl From<ClassStmt> for Value {
    fn from(source: ClassStmt) -> Self {
        Self::Class(Rc::from(LoxClass::from(source)))
    }
}

impl From<LoxClass> for Value {
    fn from(source: LoxClass) -> Self {
        Self::Class(Rc::new(source))
    }
}

impl From<FunctionStmt> for Value {
    fn from(source: FunctionStmt) -> Self {
        Self::Fun(Rc::new(CallableKind::from(Function::from(source))))
    }
}

impl From<FunctionExpr> for Value {
    fn from(source: FunctionExpr) -> Self {
        Self::Fun(Rc::new(CallableKind::from(Function::from(source))))
    }
}

impl From<NativeFunction> for Value {
    fn from(source: NativeFunction) -> Self {
        Self::Fun(Rc::new(CallableKind::from(source)))
    }
}

impl From<LoxInstance> for Value {
    fn from(source: LoxInstance) -> Self {
        Self::Instance(Rc::new(source))
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

    pub fn native_function(
        name: String,
        arity: usize,
        fun: fn(&[Value]) -> Result<Value, Error>,
    ) -> Value {
        Value::from(NativeFunction { name, arity, fun })
    }
}
