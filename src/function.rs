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
    fn call(&self, interpreter: &mut Interpreter, arguments: &[Value]) -> Result<Value, Error>;
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

    fn call(&self, interpreter: &mut Interpreter, arguments: &[Value]) -> Result<Value, Error> {
        let child = interpreter.env.child();
        let parent = child.enclosing().unwrap();

        interpreter.env = child;

        for i in 0..arguments.len() {
            let name = &self.params[i].lexeme;
            let value = arguments[i].clone();

            interpreter.env.define(name, value);
        }

        let res = match interpreter.execute_statements(&self.statements) {
            Ok(_) => Ok(Value::Nil),
            Err(Control::Return { value: v, .. }) => Ok(v),
            Err(Control::Error(e)) => Err(e),
            Err(Control::Break { keyword: k }) => Err(Error::runtime(&k, "break outside loop.")),
        };

        interpreter.env = parent;

        res
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

    fn call(&self, interpreter: &mut Interpreter, arguments: &[Value]) -> Result<Value, Error> {
        self.def.call(interpreter, arguments)
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

    fn call(&self, _interpreter: &mut Interpreter, args: &[Value]) -> Result<Value, Error> {
        (self.fun)(args)
    }
}
