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

pub trait GettersAndSetters {
    fn get(&self, name: &Token) -> Result<Value, Error>;
    fn set(&self, name: &Token, value: Value) -> Result<Value, Error>;
}

pub type MethodMap = HashMap<String, Function>;

#[derive(Debug, Clone)]
pub struct LoxClassInner {
    pub name: Token,
    pub methods: MethodMap,
    pub klass: Option<LoxClass>,
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

impl GettersAndSetters for LoxClass {
    fn get(&self, name: &Token) -> Result<Value, Error> {
        let Some(klass) = self.0.klass.as_ref() else {
            return Err(Error::runtime(
                name,
                format!("This class has no metaclass."),
            ));
        };

        let Some(method) = klass.find_method(&name.lexeme) else {
            return Err(Error::runtime(
                name,
                format!("Undefined property '{}'", name.lexeme),
            ));
        };

        let bound_method = method.bind(Value::from(self.clone()));

        Ok(Value::from(bound_method))
    }

    fn set(&self, name: &Token, _value: Value) -> Result<Value, Error> {
        Err(Error::runtime(
            name,
            format!("Can't define class attribute ('{}').", name.lexeme),
        ))
    }
}

impl LoxClass {
    pub fn new(name: Token, methods: MethodMap, klass: Option<LoxClass>) -> LoxClass {
        Self(Rc::new(LoxClassInner {
            name,
            methods,
            klass,
        }))
    }

    pub fn from_class_stmt_with_closure(stmt: ClassStmt, closure: Environment) -> LoxClass {
        let mut instance_methods: MethodMap = HashMap::new();
        let mut class_methods: MethodMap = HashMap::new();

        for method in stmt.methods {
            let Stmt {
                kind: StmtKind::Function(fs),
                ..
            } = method
            else {
                // TODO: make this an error, need to change function return type.
                unreachable!();
            };

            let fun = Function::from_stmt_with_closure(fs, closure.clone());
            let name = fun.name.clone().unwrap();

            match fun.def.kind {
                FunctionKind::Initializer | FunctionKind::Method => {
                    instance_methods.insert(name, fun.clone());
                }
                FunctionKind::ClassMethod => {
                    class_methods.insert(name, fun.clone());
                }
                _ => {
                    // TODO: make this an error. needs to change function return type.
                    unreachable!()
                }
            }
        }

        // NOTE: This is the metaclass
        let klass = Self::new(stmt.name.clone(), class_methods, None);

        Self::new(stmt.name, instance_methods, Some(klass))
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

impl GettersAndSetters for LoxInstance {
    fn get(&self, name: &Token) -> Result<Value, Error> {
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

    fn set(&self, name: &Token, value: Value) -> Result<Value, Error> {
        self.0
            .fields
            .borrow_mut()
            .insert(name.lexeme.clone(), value.clone());

        Ok(value)
    }
}

impl LoxInstance {
    pub fn for_class(klass: LoxClass) -> Self {
        Self(Rc::new(LoxInstanceInner {
            klass,
            fields: RefCell::new(HashMap::new()),
        }))
    }
}
