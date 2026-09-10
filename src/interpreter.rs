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

#[derive(Debug)]
enum Flow {
    Normal(Value),
    Break,
}

pub type ValueResult = Result<Value, Error>;
pub type FlowResult = Result<Flow, Error>;

pub fn run(env: &Env, program: Program) -> ValueResult {

    for statement in program {
        let val = execute(env, statement)?;

        match val {
            Flow::Normal(v) => res = v,
            Flow::Break => Err(Error::break_outside_loop()),
        }
    }
    Ok(Flow::Normal(res))
}

pub fn interpret(env: &Env, statement: &Stmt) -> FlowResult {
    execute(env, statement)
}

fn execute(env: &Env, stmt: &Stmt) -> FlowResult {
    match stmt {
        Stmt::Print(stmt) => print_statement(env, stmt),
        Stmt::Expression(stmt) => evaluate(env, &stmt.expression).and_then(|v| Ok(Flow::Normal(v))),
        Stmt::Var(stmt) => var_statement(env, stmt),
        Stmt::Block(stmt) => block_statement(env, stmt),
        Stmt::If(stmt) => if_statement(env, stmt),
        Stmt::While(stmt) => while_statement(env, stmt),
        Stmt::Break => Ok(Flow::Break),
    }
}

fn print_statement(env: &Env, stmt: &PrintStmt) -> FlowResult {
    let value = evaluate(env, &stmt.expression)?;
    println!("{}", value);
    Ok(Flow::Normal(Value::Nil))
}

fn var_statement(env: &Env, stmt: &VarStmt) -> FlowResult {
    let value = match &stmt.initializer {
        Some(init) => evaluate(env, init)?,
        None => Value::Nil,
    };

    env.borrow_mut().define(&stmt.name, value);

    Ok(Flow::Normal(Value::Nil))
}

fn block_statement(env: &Env, stmt: &BlockStmt) -> FlowResult {
    let block_env = Environment::with_enclosing(Rc::clone(env)).into_rc();
    execute_statements(&block_env, &stmt.statements)
}

fn execute_statements(env: &Env, statements: &[Stmt]) -> FlowResult {
    let mut res = Value::Nil;

    'statements: for statement in statements {
        let val = execute(env, statement)?;

        match val {
            Flow::Normal(v) => res = v,
            Flow::Break => break 'statements,
        }
    }

    Ok(Flow::Normal(res))
}

fn if_statement(env: &Env, stmt: &IfStmt) -> FlowResult {
    let val = evaluate(env, &stmt.condition)?;

    if is_truthy(&val) {
        execute(env, &stmt.then_branch)
    } else if let Some(else_branch) = &stmt.else_branch {
        execute(env, else_branch)
    } else {
        Ok(Flow::Normal(Value::Nil)) /* nothing runs */
    }
}

fn while_statement(env: &Env, stmt: &WhileStmt) -> FlowResult {
    while let cond = evaluate(env, &stmt.condition)?
        && is_truthy(&cond)
    {
        execute(env, &stmt.body)?;
    }

    Ok(Flow::Normal(Value::Nil))
}

fn evaluate(env: &Env, expr: &Expr) -> ValueResult {
    match expr {
        Expr::Literal(e) => Ok(e.value.clone()),
        Expr::Logical(e) => eval_logical(env, e),
        Expr::Variable(e) => env.borrow().get(&e.name),
        Expr::Assign(e) => eval_assign(env, e),
        Expr::Unary(e) => eval_unary(env, e),
        Expr::Binary(e) => eval_binary(env, e),
        Expr::Grouping(e) => evaluate(env, &e.expression),
    }
}

fn eval_logical(env: &Env, expr: &LogicalExpr) -> ValueResult {
    let left = evaluate(env, &expr.left)?;

    match expr.operator.ty {
        TokenType::And => {
            if !is_truthy(&left) {
                return Ok(left);
            }
        }
        TokenType::Or => {
            if is_truthy(&left) {
                return Ok(left);
            }
        }

        /* TODO: Can insert NAND or XOR in here*/
        _ => unreachable!(),
    }

    evaluate(env, &expr.right)
}

fn eval_assign(env: &Env, expr: &AssignExpr) -> ValueResult {
    let value = evaluate(env, &expr.expression)?;

    env.borrow_mut().assign(&expr.name, value)
}

fn eval_unary(env: &Env, expr: &UnaryExpr) -> ValueResult {
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

fn eval_binary(env: &Env, expr: &BinaryExpr) -> ValueResult {
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
    !matches!(val, Value::Bool(false) | Value::Nil)
}

fn is_equal(left: &Value, right: &Value) -> bool {
    match (left, right) {
        (Value::Nil, Value::Nil) => true,
        (Value::Nil, _) => false,
        (a, b) => a == b,
    }
}

#[cfg(test)]
mod tests {
    use crate::parser;
    use crate::scanner;

    use crate::environment::Environment;

    use super::*;

    fn run_result(src: &str) -> FlowResult {
        let tokens =
            scanner::scan(src.to_string()).unwrap_or_else(|e| panic!("scanning failed:\n{e}"));
        let statements = parser::parse(tokens).unwrap_or_else(|e| panic!("parsing failed:\n{e}"));
        let env = Environment::default().into_rc();

        run(&env, statements)
    }

    fn eval(src: &str) -> Value {
        let res = run_result(src).unwrap_or_else(|e| panic!("interpreting_failed:\n{e}"));

        match res {
            Flow::Normal(value) => value,
            other => panic!("Expected a value, got {other:?}"),
        }
    }

    fn eval_err(src: &str) -> String {
        run_result(src)
            .expect_err("expected a runtime error")
            .to_string()
    }

    #[test]
    fn interpret_arithmetic() {
        assert_eq!(eval("2 + 3 + 5;"), Value::Num(10.0));
    }

    #[test]
    fn interpret_grouping() {
        assert_eq!(eval("(2 + 3) * 5;"), Value::Num(25.0));
    }

    #[test]
    fn interpret_string_concatenation() {
        assert_eq!(eval(r#""foo" + "bar";"#), Value::Str("foobar".to_string()));
    }

    #[test]
    fn interpret_mixed_operand_addition_error() {
        assert!(eval_err(r#"5 + "foo";"#).contains("Operands must be numbers."));
    }

    #[test]
    fn interpret_nil_comparison() {
        assert_eq!(eval(r#"nil == false;"#), Value::Bool(false));
        assert_eq!(eval(r#"nil == nil;"#), Value::Bool(true));
    }

    #[test]
    fn interpret_numeric_comparison() {
        assert_eq!(eval(r#"5 <= 3;"#), Value::Bool(false));
        assert_eq!(eval(r#"8 == 8;"#), Value::Bool(true));
    }

    #[test]
    fn interpret_eq() {
        assert_eq!(eval(r#"5 == 5;"#), Value::Bool(true));
        assert_eq!(eval(r#"5 == 3;"#), Value::Bool(false));

        assert_eq!(eval(r#""foo" == "foo";"#), Value::Bool(true));
        assert_eq!(eval(r#""foo" == "bar";"#), Value::Bool(false));
    }

    #[test]
    fn interpret_neq() {
        assert_eq!(eval(r#"3 != 5;"#), Value::Bool(true));
        assert_eq!(eval(r#"5 != 5;"#), Value::Bool(false));
        assert_eq!(eval(r#"5 != 3;"#), Value::Bool(true));

        assert_eq!(eval(r#""foo" != "foo";"#), Value::Bool(false));
        assert_eq!(eval(r#""foo" != "bar";"#), Value::Bool(true));
    }

    #[test]
    fn interpret_leq() {
        assert_eq!(eval(r#"3 <= 5;"#), Value::Bool(true));
        assert_eq!(eval(r#"5 <= 5;"#), Value::Bool(true));
        assert_eq!(eval(r#"5 <= 3;"#), Value::Bool(false));
    }

    #[test]
    fn interpret_geq() {
        assert_eq!(eval(r#"3 >= 5;"#), Value::Bool(false));
        assert_eq!(eval(r#"5 >= 5;"#), Value::Bool(true));
        assert_eq!(eval(r#"5 >= 3;"#), Value::Bool(true));
    }

    #[test]
    fn interpret_assignment() {
        assert_eq!(eval(r#"var a = 3;"#), Value::Nil);
    }

    #[test]
    fn interpret_var_expr() {
        assert_eq!(
            eval(
                r#"
var a = 3;
a;
"#
            ),
            Value::Num(3.0)
        );
    }

    #[test]
    fn interpret_block() {
        assert_eq!(
            eval(
                r#"
{
    var b = 2;
    b;
}
"#
            ),
            Value::Num(2.0)
        );
    }

    #[test]
    fn interpret_if() {
        assert_eq!(
            eval(r#"if (true) "foo"; else "bar";"#),
            Value::Str(String::from("foo"))
        );
    }

    #[test]
    fn interpret_logical_or() {
        assert_eq!(eval(r#""hi" or 2;"#), Value::Str(String::from("hi")));
    }

    #[test]
    fn interpret_logical_and() {
        assert_eq!(eval(r#""hi" and 2;"#), Value::Num(2.0));
    }

    #[test]
    fn interpret_while_loop() {
        assert_eq!(
            eval(
                r#"
var a = 1;
while (a < 10) {
    a = a + 1;
    print a;
}
a;
"#
            ),
            Value::Num(10.0)
        );
    }

    #[test]
    fn interpret_variable_shadowing() {
        assert_eq!(
            eval(
                r#"
var a = 1;
{
    var a = 5;
}
a;
"#
            ),
            Value::Num(1.0)
        );
    }

    #[test]
    fn interpret_variable_block_assignment() {
        assert_eq!(
            eval(
                r#"
var a = 1;
{
    a = 5;
}
a;
"#
            ),
            Value::Num(5.0)
        );
    }

    #[test]
    fn interpret_loop_with_break_statement() {
        assert_eq!(
            eval(
                r#"
var a = 0;
while (a < 10) {
    if (a == 5) { break; }
    a = a + 1;
}
a;
"#
            ),
            Value::Num(5.0)
        );
    }

}
