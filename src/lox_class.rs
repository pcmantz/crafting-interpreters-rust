/* lox_class.rs
 *
 */

use crate::prelude::*;

use crate::error::*;
use crate::function::*;
use crate::stmt::*;
use crate::token::*;
use crate::value::*;

use crate::interpreter::*;

#[derive(Debug)]
pub struct LoxClass {
    pub name: Token,
    // pub methods: Vec<Function>,
}

impl From<ClassStmt> for LoxClass {
    fn from(source: ClassStmt) -> Self {
        Self {
            name: source.name,
            //     methods: source
            //         .methods
            //         .into_iter()
            //         .map(|s| match &s.kind {
            //             StmtKind::Function(k) => Function::from(k),
            //             _ => unreachable!(),
            //         })
            //         .collect(),
        }
    }
}

impl Callable for LoxClass {
    fn arity(&self) -> usize {
        0
    }

    fn call(&self, interpreter: &mut Interpreter, _arguments: &[Value]) -> Result<Value, Error> {
        todo!()
    }
}

pub struct LoxInstance {}
