/* environment.rs
 *
 */

use crate::prelude::*;

use crate::error::*;
use crate::token::*;
use crate::value::*;

#[derive(Default, Debug, Clone)]
pub struct Environment {
    values: HashMap<String, Value>,
    enclosing: Option<Box<Environment>>,
}

impl Environment {
    pub fn with_enclosing(enclosing: Environment) -> Self {
        Self {
            enclosing: Some(Box::new(enclosing)),
            ..Default::default()
        }
    }

    pub fn define(&mut self, name: &Token, value: Value) {
        self.values.insert(name.lexeme.clone(), value);
    }

    pub fn get(&self, name: &Token) -> Result<Value, Error> {
        if let Some(val) = self.values.get(&name.lexeme) {
            Ok(val.clone())
        } else if let Some(enc) = self.enclosing.as_ref() {
            enc.get(&name)
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
            enc.assign(&name, value)
        } else {
            Err(Error::runtime(
                name,
                format!("Undefined variable '{}'.", name.lexeme),
            ))
        }
    }
}
