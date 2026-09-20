/* interpreter.rs
 *
 */

use crate::prelude::*;

use crate::environment::*;
use crate::error::*;
use crate::expr::*;
use crate::function::*;
use crate::stmt::*;
use crate::token::*;
use crate::value::*;

#[derive(Debug, Clone)]
pub enum Control {
    Error(Error),
    Return { keyword: Token, value: Value },
    Break { keyword: Token },
}

impl fmt::Display for Control {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            Control::Error(e) => write!(f, "{}", e),
            Control::Return {
                keyword: k,
                value: v,
            } => todo!(),
            Control::Break { keyword: k } => todo!(),
        }
    }
}

impl From<Error> for Control {
    fn from(source: Error) -> Self {
        Self::Error(source)
    }
}

impl std::error::Error for Control {}

impl Control {
    fn into_error(self) -> Error {
        match self {
            Control::Return {
                keyword: k,
                value: v,
            } => Error::runtime(&k, "return outside function call."),
            Control::Break { keyword: k } => Error::runtime(&k, "break outside loop."),
            Control::Error(e) => e,
        }
    }
}

type ExecutionResult = Result<Value, Control>;

/// Interpret a lox program.
pub fn run(env: &Environment, program: Program) -> Result<Value, Error> {
    execute_statements(env, &program.0).map_err(Control::into_error)
}

/// Interpret a lox statement.
pub fn interpret(env: &Environment, statement: &Stmt) -> Result<Value, Error> {
    execute(env, statement).map_err(Control::into_error)
}

fn execute(env: &Environment, stmt: &Stmt) -> ExecutionResult {
    match stmt {
        Stmt::Print(stmt) => print_statement(env, stmt),
        Stmt::Expression(stmt) => evaluate(env, &stmt.expression),
        Stmt::Var(stmt) => var_statement(env, stmt),
        Stmt::Block(stmt) => block_statement(env, stmt),
        Stmt::If(stmt) => if_statement(env, stmt),
        Stmt::Return(stmt) => return_statement(env, stmt),
        Stmt::Break(stmt) => break_statement(env, stmt),
        Stmt::While(stmt) => while_statement(env, stmt),
        Stmt::Function(stmt) => function_statement(env, stmt),
    }
}

fn print_statement(env: &Environment, stmt: &PrintStmt) -> ExecutionResult {
    let value = evaluate(env, &stmt.expression)?;
    println!("{}", value);
    Ok(Value::Nil)
}

fn var_statement(env: &Environment, stmt: &VarStmt) -> ExecutionResult {
    let value = match &stmt.initializer {
        Some(init) => evaluate(env, init)?,
        None => Value::Nil,
    };

    env.define(&stmt.name, value);

    Ok(Value::Nil)
}

fn block_statement(env: &Environment, stmt: &BlockStmt) -> ExecutionResult {
    let block_env = env.child();
    execute_statements(&block_env, &stmt.statements)
}

/* NOTE: See how to make sharing this crate-only */
pub fn execute_statements(env: &Environment, statements: &[Stmt]) -> ExecutionResult {
    let mut res = Value::Nil;
    for statement in statements {
        res = execute(env, statement)?;
    }

    Ok(res)
}

fn if_statement(env: &Environment, stmt: &IfStmt) -> ExecutionResult {
    let val = evaluate(env, &stmt.condition)?;

    if is_truthy(&val) {
        execute(env, &stmt.then_branch)
    } else if let Some(else_branch) = &stmt.else_branch {
        execute(env, else_branch)
    } else {
        Ok(Value::Nil) /* nothing runs */
    }
}

fn return_statement(env: &Environment, stmt: &ReturnStmt) -> ExecutionResult {
    let keyword = stmt.keyword.clone();
    let value = match &stmt.value {
        Some(expr) => evaluate(env, &expr)?,
        None => Value::Nil,
    };

    Err(Control::Return { keyword, value })
}

fn break_statement(env: &Environment, stmt: &BreakStmt) -> ExecutionResult {
    let keyword = stmt.keyword.clone();

    Err(Control::Break { keyword })
}

fn while_statement(env: &Environment, stmt: &WhileStmt) -> ExecutionResult {
    while let cond = evaluate(env, &stmt.condition)?
        && is_truthy(&cond)
    {
        match execute(env, &stmt.body) {
            Ok(_) => {}
            Err(Control::Break { keyword }) => break,
            Err(Control::Error(e)) => return Err(Control::Error(e)),
            Err(Control::Return { keyword, value }) => {
                return Err(Control::Return { keyword, value });
            }
        }
    }

    Ok(Value::Nil)
}

fn function_statement(env: &Environment, stmt: &FunctionStmt) -> ExecutionResult {
    let fun = Function::new(stmt.clone());
    let val = Value::Fun(fun.into());
    env.define(&stmt.name, val);

    Ok(Value::Nil)
}

fn evaluate(env: &Environment, expr: &Expr) -> ExecutionResult {
    match expr {
        Expr::Literal(e) => Ok(e.value.clone()),
        Expr::Logical(e) => eval_logical(env, e),
        Expr::Variable(e) => env.get(&e.name).map_err(|err| err.into()),
        Expr::Assign(e) => eval_assign(env, e),
        Expr::Unary(e) => eval_unary(env, e),
        Expr::Binary(e) => eval_binary(env, e),
        Expr::Call(e) => eval_call(env, e),
        Expr::Grouping(e) => evaluate(env, &e.expression),
    }
}

fn eval_logical(env: &Environment, expr: &LogicalExpr) -> ExecutionResult {
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

fn eval_assign(env: &Environment, expr: &AssignExpr) -> ExecutionResult {
    let value = evaluate(env, &expr.expression)?;

    env.assign(&expr.name, value).map_err(|e| e.into())
}

fn eval_unary(env: &Environment, expr: &UnaryExpr) -> ExecutionResult {
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

fn eval_binary(env: &Environment, expr: &BinaryExpr) -> ExecutionResult {
    let left = evaluate(env, expr.left.as_ref())?;
    let right = evaluate(env, expr.right.as_ref())?;

    match expr.operator.ty {
        /* Special Case: Needs to handle strings and numbers */
        TokenType::Plus => match (&left, &right) {
            (Value::Num(a), Value::Num(b)) => Ok(Value::Num(a + b)),
            (Value::Str(a), Value::Str(b)) => Ok(Value::Str(format!("{a}{b}"))),
            _ => Err(Control::Error(Error::runtime(
                &expr.operator,
                "Operands must be numbers.",
            ))),
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

fn eval_call(env: &Environment, expr: &CallExpr) -> ExecutionResult {
    let callee = evaluate(env, expr.callee.as_ref())?;

    let mut arguments = Vec::new();
    for argument in expr.arguments.iter() {
        let arg = evaluate(env, &argument)?;
        arguments.push(arg);
    }

    match callee {
        Value::Fun(fun) => fun.call(env, &arguments).map_err(Control::from),
        _ => Err(Control::Error(Error::value_not_callable(&expr.paren))),
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

    fn run_result(src: &str) -> Result<Value, Error> {
        let tokens =
            scanner::scan(src.to_string()).unwrap_or_else(|e| panic!("scanning failed:\n{e}"));
        let statements = parser::parse(tokens).unwrap_or_else(|e| panic!("parsing failed:\n{e}"));
        let env = Environment::global();

        run(&env, statements)
    }

    fn eval(src: &str) -> Value {
        run_result(src).unwrap_or_else(|e| panic!("interpreting_failed:\n{e}"))
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
    fn interpret_function_definition() {
        assert_eq!(
            eval(
                r#"
fun foo(x, y) {
    x = x + 1;
    print x + y;
}
"#
            ),
            Value::Nil,
        );
    }

    #[test]
    fn interpret_function_call() {
        assert_eq!(
            eval(
                r#"
fun foo(x, y) {
    x = x + 1;
    return x + y;
}
foo(6, 7);
"#
            ),
            Value::Num(14.0)
        );
    }

    #[test]
    fn interpret_break() {
        assert_eq!(
            eval(
                r#"
var x = 0;
while (x < 5 ) {
    if (x == 2) break;
    x = x + 1;
}
x;
"#
            ),
            Value::Num(2.0)
        );
    }
}
