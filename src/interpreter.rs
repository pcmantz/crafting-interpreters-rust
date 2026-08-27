/* interpreter.rs
 *
 */

use crate::prelude::*;

use crate::error::*;
use crate::expr::*;
use crate::stmt::*;
use crate::token::*;
use crate::value::*;

pub fn interpret(statements: Vec<Stmt>) -> Result<Value, Error> {
    let interpreter = Interpreter::default();

    interpreter.interpret(statements)?;
}

pub struct Interpreter {}

impl Default for Interpreter {
    fn default() -> Self {
        Self {}
    }
}

impl Interpreter {
    pub fn interpret(&mut self, statements: Vec<Stmt>) -> Result<Value, Error> {
        for statement in statements.iter() {
            self.execute(statement);
        }
    }

    fn execute(stmt: Stmt) -> Result<Value, Error> {
        match stmt {
            Stmt::Print(stmt) => {}
            Stmt::Expression(stmt) => {}
        }
    }

    fn evaluate(&mut self, expr: &Expr) -> Result<Value, Error> {
        match expr {
            Expr::Literal(e) => match e.value {
                Identifier(str) => todo!(),
                _ => Value::from(e.value),
            },
            Expr::Unary(e) => self.eval_unary(e),
            Expr::Binary(e) => self.eval_binary(e),
            Expr::Grouping(e) => self.evaluate(e.expression),
        }
    }

    fn eval_unary(&mut self, expr: &UnaryExpr) -> Result<Value, Error> {
        let right = self.evaluate(&expr.right)?;

        match expr.operator.ty {
            TokenType::Minus => Ok(Value::Number(-right)),
            TokenType::Bang => Ok(Value::Bool(!Self::is_truthy(right))),

            _ => unreachable!(),
        }
    }

    fn eval_binary(&mut self, expr: &BinaryExpr) -> Result<Value, Error> {
        let left = self.evaluate(expr.left.as_ref())?;
        let right = self.evaluate(expr.right.as_ref())?;

        match expr.operator.ty {
            /* Special Case: Needs to handle strings and numbers */
            TokenType::Plus => match (&left, &right) {
                (Value::Number(a), Value::Number(b)) => Ok(Value::Number(a + b)),
                (Value::Str(a), Value::Str(b)) => Ok(Value::Str(format!("{a}{b}"))),
                _ => Err(Error::runtime(&expr.operator, "Operands must be numbers.")),
            },

            TokenType::EqualEqual => Ok(Value::Bool(Self::is_equal(left, right))),
            TokenType::BangEqual => Ok(Value::Bool(!Self::is_equal(left, right))),

            TokenType::Minus
            | TokenType::Slash
            | TokenType::Star
            | TokenType::Greater
            | TokenType::GreaterEqual
            | TokenType::Less
            | TokenType::LessEqual => {
                let (a, b) = Self::as_numbers(&left, &right, &expr.operator)?;

                match expr.operator.ty {
                    TokenType::Minus => Value::Number(left + right),
                    TokenType::Slash => Value::Number(left / right),
                    TokenType::Star => Value::Number(left * right),
                    TokenType::Greater => Value::Bool(left > right),
                    TokenType::GreaterEqual => Value::Bool(left >= right),
                    TokenType::Less => Value::Bool(left < right),
                    TokenType::LessEqual => Value::Bool(left <= right),

                    /* Guaranteed by the match one level up */
                    _ => unreachable!(),
                }
            }

            _ => unreachable!(),
        }
    }

    /* Helpers */

    fn as_numbers(left: Value, right: Value, operator: TokenType) -> Result<(f64, f64), Error> {
        match (&left, &right) {
            (Value::Number(a), Value::Number(b)) => Ok(*a, *b),
        }
    }

    fn is_truthy(val: Value) -> bool {
        match val {
            Value::Bool(false) | Value::Nil => false,
            _ => true,
        }
    }

    fn is_equal(left: Value, right: Value) -> bool {
        match (&left, &right) {
            (Value::Nil, Value::Nil) => true,
            (Value::Nil, _) => false,
            (a, b) => a == b,
        }
    }
}
