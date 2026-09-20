/* environment.rs
 *
 */

use crate::prelude::*;

use crate::error::*;
use crate::token::*;
use crate::value::*;

/// Inner body implementation of the Environment.
#[derive(Default, Debug)]
struct Inner {
    values: RefCell<HashMap<String, Value>>,
    enclosing: Option<Environment>,
}

/// Handle over a Scope type.
#[derive(Debug, Clone)]
pub struct Environment(Rc<Inner>);

impl Default for Environment {
    fn default() -> Self {
        Self::global()
    }
}

impl Environment {
    /// Creates a default global environment.
    pub fn global() -> Self {
        let global = Self {
            0: Rc::new(Inner::default()),
        };

        // TODO: Add definition of native functions here.

        global
    }

    pub fn empty() -> Self {
        Self(Rc::new(Inner {
            enclosing: None,
            ..Default::default()
        }))
    }

    /// Creates an environment with self as the enclosing environment.
    pub fn child(&self) -> Self {
        Self(Rc::new(Inner {
            enclosing: Some(self.clone()),
            ..Default::default()
        }))
    }

    pub fn define(&self, name: &str, value: Value) {
        self.0.values.borrow_mut().insert(name.into(), value);
    }

    pub fn get(&self, name: &str) -> Option<Value> {
        if let Some(val) = self.0.values.borrow().get(name) {
            Some(val.clone())
        } else if let Some(enc) = self.0.enclosing.as_ref() {
            enc.get(name)
        } else {
            None
        }
    }

    pub fn assign(&self, name: &str, value: Value) -> Option<Value> {
        if let Some(value_ref) = self.0.values.borrow_mut().get_mut(name) {
            *value_ref = value.clone();

            Some(value)
        } else if let Some(enc) = self.0.enclosing.as_ref() {
            enc.assign(name, value)
        } else {
            None
        }
    }
}
