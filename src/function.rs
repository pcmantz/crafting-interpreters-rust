/* function.rs
 *
 */

use crate::prelude::*;

use crate::error::*;
use crate::expr::*;
use crate::stmt::*;
use crate::token::*;
use crate::value::*;

use crate::environment::*;
use crate::interpreter::*;

pub trait Callable {
    fn arity(&self) -> usize;
    fn call(&self, interpreter: &mut Interpreter, arguments: &[Value]) -> Result<Value, Error>;
}

#[derive(Debug, Clone, Copy)]
pub enum FunctionKind {
    Function,
    Initializer,
    Method,
    ClassMethod,
}

#[derive(Debug, Clone)]
pub struct FunctionDef {
    pub kind: FunctionKind,
    pub params: Vec<Token>,
    pub statements: Vec<Stmt>,
}

impl FunctionDef {
    fn arity(&self) -> usize {
        self.params.len()
    }

    fn call_in_env(
        &self,
        interpreter: &mut Interpreter,
        arguments: &[Value],
        env: Environment,
    ) -> Result<Value, Error> {
        for (i, param) in self.params.iter().enumerate() {
            let name = &param.lexeme;
            let value = arguments[i].clone();

            env.define(name, value);
        }

        match interpreter.execute_statements_in_env(&self.statements, env) {
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
    pub closure: Environment,
    pub is_initializer: bool,
}

impl Callable for Function {
    fn arity(&self) -> usize {
        self.def.arity()
    }

    fn call(&self, interpreter: &mut Interpreter, arguments: &[Value]) -> Result<Value, Error> {
        let env = self.closure.child();
        let res = self.def.call_in_env(interpreter, arguments, env)?;

        if self.is_initializer {
            Ok(self
                .closure
                .get_at(0, "this")
                .expect("initializer bound without 'this' in scope"))
        } else {
            Ok(res)
        }
    }
}

impl Function {
    pub fn from_stmt_with_closure(stmt: FunctionStmt, closure: Environment) -> Self {
        let name_string = stmt.name.lexeme.clone();
        let is_initializer =
            matches!(stmt.def.kind, FunctionKind::Method) && name_string == "init".to_string();

        Self {
            name: Some(name_string),
            closure,
            def: Rc::clone(&stmt.def),
            is_initializer,
        }
    }

    pub fn from_expr_with_closure(expr: FunctionExpr, closure: Environment) -> Self {
        Self {
            name: None,
            closure,
            def: Rc::clone(&expr.def),
            is_initializer: false,
        }
    }

    pub fn bind(&self, instance: Value) -> Self {
        let env = self.closure.child();
        env.define("this", instance);

        Self {
            name: self.name.clone(),
            closure: env,
            def: self.def.clone(),
            is_initializer: self.is_initializer,
        }
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
