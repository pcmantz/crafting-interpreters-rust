/* interpreter.rs
 *
 */

use crate::prelude::*;

use crate::error::*;
use crate::expr::*;
use crate::stmt::*;
use crate::token::*;
use crate::value::*;

pub struct Interpreter {}

impl Default for Interpreter {
    fn default() -> Self {
        Self {}
    }
}

impl Interpreter {
    pub fn run(&mut self, program: Program) -> Result<Value, Error> {
        let mut res = Value::Nil;

        for statement in program {
            res = self.execute(&statement)?;
        }

        Ok(res)
    }

    pub fn interpret(&mut self, statement: Stmt) -> Result<Value, Error> {
        self.execute(&statement)
    }

    fn execute(&mut self, stmt: &Stmt) -> Result<Value, Error> {
        match stmt {
            Stmt::Print(stmt) => self.print_statement(stmt),
            Stmt::Expression(stmt) => self.evaluate(&stmt.expression),
            Stmt::Var(stmt) => todo!(),
        }
    }

    fn print_statement(&mut self, stmt: &PrintStmt) -> Result<Value, Error> {
        let value = self.evaluate(&stmt.expression)?;
        println!("{}", value);
        Ok(Value::Nil)
    }

    fn evaluate(&mut self, expr: &Expr) -> Result<Value, Error> {
        match expr {
            Expr::Literal(e) => Ok(e.value.clone()),
            Expr::Variable(e) => todo!(),
            Expr::Assign(e) => todo!(),
            Expr::Unary(e) => self.eval_unary(e),
            Expr::Binary(e) => self.eval_binary(e),
            Expr::Grouping(e) => self.evaluate(&e.expression),
        }
    }

    fn eval_unary(&mut self, expr: &UnaryExpr) -> Result<Value, Error> {
        let right = self.evaluate(&expr.right)?;

        match expr.operator.ty {
            TokenType::Minus => {
                let a = Self::as_number(&right, &expr.operator)?;

                Ok(Value::Num(-a))
            }
            TokenType::Bang => Ok(Value::Bool(!Self::is_truthy(&right))),

            _ => unreachable!(),
        }
    }

    fn eval_binary(&mut self, expr: &BinaryExpr) -> Result<Value, Error> {
        let left = self.evaluate(expr.left.as_ref())?;
        let right = self.evaluate(expr.right.as_ref())?;

        match expr.operator.ty {
            /* Special Case: Needs to handle strings and numbers */
            TokenType::Plus => match (&left, &right) {
                (Value::Num(a), Value::Num(b)) => Ok(Value::Num(a + b)),
                (Value::Str(a), Value::Str(b)) => Ok(Value::Str(format!("{a}{b}"))),
                _ => Err(Error::runtime(&expr.operator, "Operands must be numbers.")),
            },

            TokenType::EqualEqual => Ok(Value::Bool(Self::is_equal(&left, &right))),
            TokenType::BangEqual => Ok(Value::Bool(!Self::is_equal(&left, &right))),

            TokenType::Minus
            | TokenType::Slash
            | TokenType::Star
            | TokenType::Greater
            | TokenType::GreaterEqual
            | TokenType::Less
            | TokenType::LessEqual => {
                let (a, b) = Self::as_numbers(&left, &right, &expr.operator)?;

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
}
