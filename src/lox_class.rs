/* lox_class.rs
 *
 */

use crate::prelude::*;

use crate::error::*;
use crate::function::*;
use crate::stmt::*;
use crate::token::*;
use crate::value::*;

use crate::environment::*;
use crate::interpreter::*;

pub type MethodMap = HashMap<String, Function>;

#[derive(Debug)]
pub struct LoxClass {
    pub name: Token,
    pub methods: MethodMap,
}

impl LoxClass {
    pub fn from_class_stmt_with_closure(stmt: ClassStmt, closure: Environment) -> LoxClass {
        let methods = stmt
            .methods
            .into_iter()
            .filter_map(|m| match m.kind {
                StmtKind::Function(k) => {
                    let fun = Function::from_stmt_with_closure(k, closure.clone());

                    Some((fun.name.clone().unwrap(), fun))
                }

                _ => unreachable!("non-function in class body"),
            })
            .collect();

        Self {
            name: stmt.name,
            methods,
        }
    }

    fn find_method(&self, name: &String) -> Option<&Function> {
        self.methods.get(name)
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
        if let Some(val) = self.fields.borrow().get(&name.lexeme) {
            Ok(val.clone())
        } else if let Some(method) = self.klass.find_method(&name.lexeme) {
            Ok(Value::from(method.clone()))
        } else {
            Err(Error::runtime(
                name,
                format!("Undefined property '{}'", name.lexeme),
            ))
        }
    }

    pub fn set(&self, name: &Token, value: Value) {
        self.fields.borrow_mut().insert(name.lexeme.clone(), value);
    }
}
