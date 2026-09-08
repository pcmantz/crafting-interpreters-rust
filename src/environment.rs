/* environment.rs
 *
 */

use crate::prelude::*;

use crate::error::*;
use crate::token::*;
use crate::value::*;

pub type Env = Rc<RefCell<Environment>>;

#[derive(Default, Debug)]
pub struct Environment {
    values: HashMap<String, Value>,
    enclosing: Option<Env>,
}

impl Environment {
    pub fn with_enclosing(enclosing: Env) -> Self {
        Self {
            enclosing: Some(enclosing),
            ..Default::default()
        }
    }

    pub fn into_rc(self) -> Env {
        Rc::new(RefCell::new(self))
    }

    pub fn define(&mut self, name: &Token, value: Value) {
        self.values.insert(name.lexeme.clone(), value);
    }

    pub fn get(&self, name: &Token) -> Result<Value, Error> {
        if let Some(val) = self.values.get(&name.lexeme) {
            Ok(val.clone())
        } else if let Some(enc) = self.enclosing.as_ref() {
            enc.borrow().get(&name)
        } else {
            Err(Error::runtime(
                name,
                format!("Undefined variable '{}'.", name.lexeme),
            ))
        }
    }

    pub fn assign(&mut self, name: &Token, value: Value) -> Result<Value, Error> {
        if let Some(value_ref) = self.values.get_mut(&name.lexeme) {
            *value_ref = value.clone();

            Ok(value)
        } else if let Some(enc) = self.enclosing.as_mut() {
            enc.borrow_mut().assign(&name, value)
        } else {
            Err(Error::runtime(
                name,
                format!("Undefined variable '{}'.", name.lexeme),
            ))
        }
    }
}
