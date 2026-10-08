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
                value: _v,
            } => Error::runtime(&k, "return outside function call."),
            Control::Break { keyword: k } => Error::runtime(&k, "break outside loop."),
            Control::Error(e) => e,
        }
    }
}

type ExecutionResult = Result<Value, Control>;

pub struct Interpreter {
    pub env: Environment,
}

impl Interpreter {
    pub fn new() -> Self {
        Self {
            env: Environment::global(),
        }
    }

    /// Interpret a lox program.
    pub fn run(&mut self, program: Program) -> Result<Value, Error> {
        self.execute_statements(&program.0)
            .map_err(Control::into_error)
    }

    fn execute(&mut self, stmt: &Stmt) -> ExecutionResult {
        match &stmt.kind {
            StmtKind::Print(s) => self.print_statement(&s),
            StmtKind::Expression(s) => self.evaluate(&s.expression),
            StmtKind::Var(s) => self.var_statement(&s),
            StmtKind::Block(s) => self.block_statement(&s),
            StmtKind::If(s) => self.if_statement(&s),
            StmtKind::Return(s) => self.return_statement(&s),
            StmtKind::Break(s) => self.break_statement(&s),
            StmtKind::While(s) => self.while_statement(&s),
            StmtKind::Function(s) => self.function_statement(&s),
        }
    }

    fn print_statement(&mut self, stmt: &PrintStmt) -> ExecutionResult {
        let value = self.evaluate(&stmt.expression)?;
        println!("{}", value);
        Ok(Value::Nil)
    }

    fn var_statement(&mut self, stmt: &VarStmt) -> ExecutionResult {
        let value = match &stmt.initializer {
            Some(init) => self.evaluate(init)?,
            None => Value::Nil,
        };

        self.env.define(&stmt.name.lexeme, value);

        Ok(Value::Nil)
    }

    fn block_statement(&mut self, stmt: &BlockStmt) -> ExecutionResult {
        let child = self.env.child();
        let parent = child.enclosing().unwrap();

        self.env = child;
        let res = self.execute_statements(&stmt.statements);
        self.env = parent;

        res
    }

    /* NOTE: See how to make sharing this crate-only */
    pub fn execute_statements(&mut self, statements: &[Stmt]) -> ExecutionResult {
        let mut res = Value::Nil;

        for statement in statements {
            res = self.execute(statement)?;
        }

        Ok(res)
    }

    fn if_statement(&mut self, stmt: &IfStmt) -> ExecutionResult {
        let cond = self.evaluate(&stmt.condition)?;

        if Self::is_truthy(&cond) {
            self.execute(&stmt.then_branch)
        } else if let Some(else_branch) = &stmt.else_branch {
            self.execute(else_branch)
        } else {
            Ok(Value::Nil) /* nothing runs */
        }
    }

    fn return_statement(&mut self, stmt: &ReturnStmt) -> ExecutionResult {
        let keyword = stmt.keyword.clone();
        let value = match &stmt.value {
            Some(expr) => self.evaluate(&expr)?,
            None => Value::Nil,
        };

        Err(Control::Return { keyword, value })
    }

    fn break_statement(&self, stmt: &BreakStmt) -> ExecutionResult {
        let keyword = stmt.keyword.clone();

        Err(Control::Break { keyword })
    }

    fn while_statement(&mut self, stmt: &WhileStmt) -> ExecutionResult {
        while let cond = self.evaluate(&stmt.condition)?
            && Self::is_truthy(&cond)
        {
            match self.execute(&stmt.body) {
                Ok(_) => {}
                Err(Control::Break { .. }) => break,
                Err(Control::Error(e)) => return Err(Control::Error(e)),
                Err(Control::Return { keyword, value }) => {
                    return Err(Control::Return { keyword, value });
                }
            }
        }

        Ok(Value::Nil)
    }

    fn function_statement(&self, stmt: &FunctionStmt) -> ExecutionResult {
        let val = Value::function_from_stmt(stmt.clone());
        self.env.define(&stmt.name.lexeme, val);

        Ok(Value::Nil)
    }

    fn evaluate(&mut self, expr: &Expr) -> ExecutionResult {
        match &expr.kind {
            ExprKind::Literal(e) => Ok(e.value.clone()),
            ExprKind::Logical(e) => self.eval_logical(expr, &e),
            ExprKind::Variable(e) => self.eval_variable(expr, &e),
            ExprKind::Assign(e) => self.eval_assign(expr, &e),
            ExprKind::Unary(e) => self.eval_unary(expr, &e),
            ExprKind::Binary(e) => self.eval_binary(expr, &e),
            ExprKind::Call(e) => self.eval_call(expr, &e),
            ExprKind::Grouping(e) => self.evaluate(&e.expression),
            ExprKind::Function(e) => self.eval_function(expr, &e),
        }
    }

    fn eval_logical(&mut self, expr: &Expr, kind: &LogicalExpr) -> ExecutionResult {
        let left = self.evaluate(&kind.left)?;

        match kind.operator.ty {
            TokenType::And => {
                if !Self::is_truthy(&left) {
                    return Ok(left);
                }
            }
            TokenType::Or => {
                if Self::is_truthy(&left) {
                    return Ok(left);
                }
            }

            /* TODO: Can insert NAND or XOR in here*/
            _ => unreachable!(),
        }

        self.evaluate(&kind.right)
    }

    fn eval_variable(&self, expr: &Expr, kind: &VariableExpr) -> ExecutionResult {
        self.env.get(&kind.name.lexeme).ok_or_else(|| {
            Error::runtime(
                &kind.name,
                format!("Undefined variable '{}'", &kind.name.lexeme),
            )
            .into()
        })
    }

    fn eval_assign(&mut self, expr: &Expr, kind: &AssignExpr) -> ExecutionResult {
        let value = self.evaluate(&kind.expression)?;

        self.env.assign(&kind.name.lexeme, value).ok_or_else(|| {
            Error::runtime(
                &kind.name,
                format!("Undefined variable '{}'", &kind.name.lexeme),
            )
            .into()
        })
    }

    fn eval_unary(&mut self, expr: &Expr, kind: &UnaryExpr) -> ExecutionResult {
        let right = self.evaluate(&kind.right)?;

        match kind.operator.ty {
            TokenType::Minus => {
                let a = Self::as_number(&right, &kind.operator)?;

                Ok(Value::Num(-a))
            }
            TokenType::Bang => Ok(Value::Bool(!Self::is_truthy(&right))),

            _ => unreachable!(),
        }
    }

    fn eval_binary(&mut self, expr: &Expr, kind: &BinaryExpr) -> ExecutionResult {
        let left = self.evaluate(kind.left.as_ref())?;
        let right = self.evaluate(kind.right.as_ref())?;

        match kind.operator.ty {
            /* Special Case: Needs to handle strings and numbers */
            TokenType::Plus => match (&left, &right) {
                (Value::Num(a), Value::Num(b)) => Ok(Value::Num(a + b)),
                (Value::Str(a), Value::Str(b)) => Ok(Value::Str(format!("{a}{b}"))),
                _ => Err(Control::Error(Error::runtime(
                    &kind.operator,
                    "Operands must be numbers.",
                ))),
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
                let (a, b) = Self::as_numbers(&left, &right, &kind.operator)?;

                Ok(match kind.operator.ty {
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

    fn eval_call(&mut self, expr: &Expr, kind: &CallExpr) -> ExecutionResult {
        let callee = self.evaluate(kind.callee.as_ref())?;

        let mut arguments = Vec::new();
        for argument in kind.arguments.iter() {
            let arg = self.evaluate(&argument)?;
            arguments.push(arg);
        }

        match callee {
            Value::Fun(fun) => fun.call(self, &arguments).map_err(Control::from),

            /* TODO: The following should pull the span from the callee, not the paren. needs spans in exprs first. */
            _ => Err(Control::Error(Error::value_not_callable(kind.paren.span))),
        }
    }

    fn eval_function(&self, expr: &Expr, kind: &FunctionExpr) -> ExecutionResult {
        let val = Value::function_from_expr(kind.clone());

        Ok(val)
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

        let mut interpreter = Interpreter::new();

        interpreter.run(statements)
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

    #[test]
    fn interpret_clock_native_function() {
        assert!(matches!(eval("clock();"), Value::Num(n) if n > 0.0));
    }

    #[test]
    fn interpret_first_class_functions() {
        assert_eq!(
            eval(
                r#"
var func = fun (x) { return x + 3; };

fun somefun (x, fn) {
    return fn(x);
}

somefun(3, func);
"#
            ),
            Value::Num(6.0)
        );
    }
}
