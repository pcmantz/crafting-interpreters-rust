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

pub type ClassRef = Rc<LoxClass>;

impl Callable for ClassRef {
    fn arity(&self) -> usize {
        0
    }

    fn call(&self, interpreter: &mut Interpreter, _arguments: &[Value]) -> Result<Value, Error> {
        let instance = LoxInstance::for_class(Rc::clone(self));

        Ok(Value::from(instance))
    }
}

#[derive(Debug)]
pub struct LoxInstance {
    pub klass: Rc<LoxClass>,
    pub fields: RefCell<HashMap<String, Value>>,
}

impl LoxInstance {
    pub fn for_class(klass: ClassRef) -> Self {
        Self {
            klass,
            fields: RefCell::new(HashMap::new()),
        }
    }

    pub fn get(&self, name: &Token) -> Result<Value, Error> {
        match self.fields.borrow().get(&name.lexeme) {
            Some(val) => Ok(val.clone()),
            None => Err(Error::runtime(
                name,
                format!("Undefined property '{}'", name.lexeme),
            )),
        }
    }

    pub fn set(&self, name: &Token, value: Value) {
        self.fields.borrow_mut().insert(name.lexeme.clone(), value);
    }
}
