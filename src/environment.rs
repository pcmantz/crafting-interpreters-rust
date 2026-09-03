/* environment.rs
 *
 */

use crate::prelude::*;

use crate::error::*;
use crate::token::*;
use crate::value::*;

#[derive(Default, Debug)]
pub struct Environment {
    values: HashMap<String, Value>,
}

impl Environment {
    pub fn define(&mut self, name: &Token, value: Value) {
        self.values.insert(name.lexeme.clone(), value);
    }

    pub fn get(&self, name: &Token) -> Result<Value, Error> {
        self.values
            .get(&name.lexeme)
            .cloned()
            .ok_or(Error::runtime(name, format!("Undefined variable '{}'.", name.lexeme)))
    }

    pub fn assign(&mut self, name: &Token, value: Value) -> Result<Value, Error> {
        if let Some(value_ref) = self.values.get_mut(&name.lexeme) {
            *value_ref = value.clone();

            Ok(value)
        } else {
            Err(Error::runtime(name, format!("Undefined variable '{}'.", name.lexeme)))
        }
    }
}
