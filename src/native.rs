/* native.rs
 *
 */

use crate::prelude::*;

use std::time::{SystemTime, UNIX_EPOCH};

use crate::environment::*;
use crate::error::*;
use crate::value::*;

pub fn populate_environment(env: &Environment) {
    env.define(
        "clock",
        Value::native_function("clock".to_string(), 0, clock),
    );
}

pub fn clock(args: &[Value]) -> Result<Value, Error> {
    let secs = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map(|d| d.as_secs_f64())
        .unwrap_or(0.0);

    Ok(Value::Num(secs))
}
