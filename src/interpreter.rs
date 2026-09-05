/* interpreter.rs
 *
 */

use crate::prelude::*;

use crate::environment::*;
use crate::error::*;
use crate::expr::*;
use crate::stmt::*;
use crate::token::*;
use crate::value::*;

pub fn run(env: &mut Environment, program: Program) -> Result<Value, Error> {
    let mut res = Value::Nil;

    for statement in program {
        res = execute(env, &statement)?;
    }

    Ok(res)
}

pub fn interpret(env: &mut Environment, statement: Stmt) -> Result<Value, Error> {
    execute(env, &statement)
}

fn execute(mut env: &mut Environment, stmt: &Stmt) -> Result<Value, Error> {
    match stmt {
        Stmt::Print(stmt) => print_statement(&mut env, stmt),
        Stmt::Expression(stmt) => evaluate(&mut env, &stmt.expression),
        Stmt::Var(stmt) => var_statement(&mut env, stmt),
        Stmt::Block(stmt) => todo!(),
    }
}

fn print_statement(env: &mut Environment, stmt: &PrintStmt) -> Result<Value, Error> {
    let value = evaluate(env, &stmt.expression)?;
    println!("{}", value);
    Ok(Value::Nil)
}

fn var_statement(env: &mut Environment, stmt: &VarStmt) -> Result<Value, Error> {
    let value = match &stmt.initializer {
        Some(init) => evaluate(env, init)?,
        None => Value::Nil,
    };

    env.define(&stmt.name, value);

    Ok(Value::Nil)
}

fn block_statement(env: &mut Environment, stmt: &BlockStmt) -> Result<Value, Error> {
    todo!()
}

fn evaluate(env: &mut Environment, expr: &Expr) -> Result<Value, Error> {
    match expr {
        Expr::Literal(e) => Ok(e.value.clone()),
        Expr::Variable(e) => env.get(&e.name),
        Expr::Assign(e) => eval_assign(env, e),
        Expr::Unary(e) => eval_unary(env, e),
        Expr::Binary(e) => eval_binary(env, e),
        Expr::Grouping(e) => evaluate(env, &e.expression),
    }
}

fn eval_assign(env: &mut Environment, expr: &AssignExpr) -> Result<Value, Error> {
    let value = evaluate(env, &expr.expression)?;

    env.assign(&expr.name, value)
}

fn eval_unary(env: &mut Environment, expr: &UnaryExpr) -> Result<Value, Error> {
    let right = evaluate(env, &expr.right)?;

    match expr.operator.ty {
        TokenType::Minus => {
            let a = as_number(&right, &expr.operator)?;

            Ok(Value::Num(-a))
        }
        TokenType::Bang => Ok(Value::Bool(!is_truthy(&right))),

        _ => unreachable!(),
    }
}

fn eval_binary(env: &mut Environment, expr: &BinaryExpr) -> Result<Value, Error> {
    let left = evaluate(env, expr.left.as_ref())?;
    let right = evaluate(env, expr.right.as_ref())?;

    match expr.operator.ty {
        /* Special Case: Needs to handle strings and numbers */
        TokenType::Plus => match (&left, &right) {
            (Value::Num(a), Value::Num(b)) => Ok(Value::Num(a + b)),
            (Value::Str(a), Value::Str(b)) => Ok(Value::Str(format!("{a}{b}"))),
            _ => Err(Error::runtime(&expr.operator, "Operands must be numbers.")),
        },

        TokenType::EqualEqual => Ok(Value::Bool(is_equal(&left, &right))),
        TokenType::BangEqual => Ok(Value::Bool(!is_equal(&left, &right))),

        TokenType::Minus
        | TokenType::Slash
        | TokenType::Star
        | TokenType::Greater
        | TokenType::GreaterEqual
        | TokenType::Less
        | TokenType::LessEqual => {
            let (a, b) = as_numbers(&left, &right, &expr.operator)?;

            Ok(match expr.operator.ty {
                TokenType::Minus => Value::Num(a - b),
                TokenType::Slash => Value::Num(a / b),
                TokenType::Star => Value::Num(a * b),
                TokenType::Greater => Value::Bool(a > b),
                TokenType::GreaterEqual => Value::Bool(a >= b),
                TokenType::Less => Value::Bool(a < b),
                TokenType::LessEqual => Value::Bool(a <= b),

                /* Guaranteed by the match one level up */
                _ => unreachable!(),
            })
        }

        _ => unreachable!(),
    }
}

/* Helpers */

fn as_numbers(left: &Value, right: &Value, operator: &Token) -> Result<(f64, f64), Error> {
    match (left, right) {
        (Value::Num(a), Value::Num(b)) => Ok((*a, *b)),
        _ => Err(Error::runtime(operator, "Operands must be numbers.")),
    }
}

fn as_number(right: &Value, operator: &Token) -> Result<f64, Error> {
    match right {
        Value::Num(a) => Ok(*a),
        _ => Err(Error::runtime(operator, "Operand must be a number.")),
    }
}

fn is_truthy(val: &Value) -> bool {
    match val {
        Value::Bool(false) | Value::Nil => false,
        _ => true,
    }
}

fn is_equal(left: &Value, right: &Value) -> bool {
    match (left, right) {
        (Value::Nil, Value::Nil) => true,
        (Value::Nil, _) => false,
        (a, b) => a == b,
    }
}
