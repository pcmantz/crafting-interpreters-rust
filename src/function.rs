/* function.rs
 *
 */

use crate::prelude::*;

use crate::environment::*;
use crate::error::*;
use crate::expr::*;
use crate::stmt::*;
use crate::token::*;
use crate::value::*;

use crate::interpreter::*;

pub trait Callable {
    fn arity(&self) -> usize;
    fn call(&self, env: &Environment, arguments: &[Value]) -> Result<Value, Error>;
}

#[derive(Debug, Clone)]
pub enum FunctionKind {
    Function,
    Method,
}

#[derive(Debug, Clone)]
pub struct FunctionDef {
    pub params: Vec<Token>,
    pub statements: Vec<Stmt>,
}

impl Callable for FunctionDef {
    fn arity(&self) -> usize {
        self.params.len()
    }

    fn call(&self, env: &Environment, arguments: &[Value]) -> Result<Value, Error> {
        let call_env = env.child();

        for i in 0..arguments.len() {
            let name = &self.params[i].lexeme;
            let value = arguments[i].clone();

            call_env.define(name, value);
        }

        match execute_statements(&call_env, &self.statements) {
            Ok(_) => Ok(Value::Nil),
            Err(Control::Return { value: v, .. }) => Ok(v),
            Err(Control::Error(e)) => Err(e),
            Err(Control::Break { keyword: k }) => Err(Error::runtime(&k, "break outside loop.")),
        }
    }
}

#[derive(Debug, Clone)]
pub struct Function {
    pub name: Option<String>,
    pub def: Rc<FunctionDef>,
}

impl Callable for Function {
    fn arity(&self) -> usize {
        self.def.arity()
    }

    fn call(&self, env: &Environment, arguments: &[Value]) -> Result<Value, Error> {
        self.def.call(env, arguments)
    }
}

#[derive(Debug, Clone)]
pub struct NativeFunction {
    pub name: String,
    pub arity: usize,
    pub fun: fn(&[Value]) -> Result<Value, Error>,
}

impl Callable for NativeFunction {
    fn arity(&self) -> usize {
        self.arity
    }

    fn call(&self, _env: &Environment, args: &[Value]) -> Result<Value, Error> {
        (self.fun)(args)
    }
}
