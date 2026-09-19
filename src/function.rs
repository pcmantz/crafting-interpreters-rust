/* function.rs
 *
 */

use crate::prelude::*;

use crate::environment::*;
use crate::error::*;
use crate::stmt::*;
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
pub struct Function {
    pub statement: FunctionStmt,
}

impl Callable for Function {
    fn arity(&self) -> usize {
        self.statement.params.len()
    }

    fn call(&self, env: &Environment, arguments: &[Value]) -> Result<Value, Error> {
        let call_env = env.child();

        for i in 0..arguments.len() {
            let name = self.statement.params[i].clone();
            let value = arguments[i].clone();

            call_env.define(&name, value);
        }

        match execute_statements(&call_env, &self.statement.statements) {
            Ok(_) => Ok(Value::Nil),
            Err(Control::Return { value: v, .. }) => Ok(v),
            Err(Control::Error(e)) => Err(e),
            Err(Control::Break { keyword: k }) => {
                Err(Error::runtime(&k, "return outside function call."))
            }
        }
    }
}

impl Function {
    pub fn new(statement: FunctionStmt) -> Self {
        Self { statement }
    }
}
