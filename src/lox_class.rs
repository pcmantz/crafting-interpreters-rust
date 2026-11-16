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

#[derive(Debug, Clone)]
pub struct LoxClassInner {
    pub name: Token,
    pub methods: MethodMap,
}

#[derive(Debug, Clone)]
pub struct LoxClass(pub Rc<LoxClassInner>);

impl PartialEq for LoxClass {
    fn eq(&self, other: &Self) -> bool {
        Rc::ptr_eq(&self.0, &other.0)
    }
}

impl Callable for LoxClass {
    fn arity(&self) -> usize {
        if let Some(initializer) = self.find_method(&String::from("init")) {
            initializer.arity()
        } else {
            0
        }
    }

    fn call(&self, interpreter: &mut Interpreter, arguments: &[Value]) -> Result<Value, Error> {
        let instance_value = Value::from(LoxInstance::for_class(self.clone()));

        if let Some(initializer) = self.find_method(&String::from("init")) {
            initializer
                .bind(instance_value.clone())
                .call(interpreter, arguments)?;
        }

        Ok(instance_value)
    }
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

        Self(Rc::new(LoxClassInner {
            name: stmt.name,
            methods,
        }))
    }

    fn find_method(&self, name: &String) -> Option<&Function> {
        self.0.methods.get(name)
    }
}

#[derive(Debug, Clone)]
pub struct LoxInstanceInner {
    pub klass: LoxClass,
    pub fields: RefCell<HashMap<String, Value>>,
}

#[derive(Debug, Clone)]
pub struct LoxInstance(pub Rc<LoxInstanceInner>);

impl PartialEq for LoxInstance {
    fn eq(&self, other: &Self) -> bool {
        Rc::ptr_eq(&self.0, &other.0)
    }
}

impl LoxInstance {
    pub fn for_class(klass: LoxClass) -> Self {
        Self(Rc::new(LoxInstanceInner {
            klass,
            fields: RefCell::new(HashMap::new()),
        }))
    }

    pub fn get(&self, name: &Token) -> Result<Value, Error> {
        if let Some(val) = self.0.fields.borrow().get(&name.lexeme) {
            Ok(val.clone())
        } else if let Some(method) = self.0.klass.find_method(&name.lexeme) {
            let bound_method = method.bind(Value::from(self.clone()));

            Ok(Value::from(bound_method))
        } else {
            Err(Error::runtime(
                name,
                format!("Undefined property '{}'", name.lexeme),
            ))
        }
    }

    pub fn set(&self, name: &Token, value: Value) {
        self.0
            .fields
            .borrow_mut()
            .insert(name.lexeme.clone(), value);
    }
}
