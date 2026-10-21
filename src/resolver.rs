/* resolver.rb
 *
 */

use crate::prelude::*;

use crate::error::*;
use crate::expr::*;
use crate::function::*;
use crate::stmt::*;
use crate::token::*;

use crate::interpreter::*;

#[derive(Debug, Clone)]
pub enum DeclarationStatus {
    Defined,
    Declared,
}

#[derive(Default, Debug)]
pub struct Scope {
    values: HashMap<String, DeclarationStatus>,
}

impl Scope {
    pub fn declare(&mut self, name: &str) -> Option<DeclarationStatus> {
        self.values
            .insert(name.to_string(), DeclarationStatus::Declared);

        Some(DeclarationStatus::Declared)
    }

    pub fn define(&mut self, name: &str) -> Option<DeclarationStatus> {
        match self.values.get_mut(name) {
            /* NOTE: Cannot use insert b/c variable must be declared or defined first. */
            Some(value_ref) => {
                *value_ref = DeclarationStatus::Defined;

                Some(DeclarationStatus::Declared)
            }
            None => None,
        }
    }
}

#[derive(Default, Debug)]
pub struct ScopeStack {
    scopes: Vec<Scope>,
}

impl ScopeStack {
    pub fn begin_scope(&mut self) {
        self.scopes.push(Scope::default())
    }

    pub fn end_scope(&mut self) {
        self.scopes.pop();
    }

    pub fn is_empty(&self) -> bool {
        self.scopes.is_empty()
    }

    pub fn declare(&mut self, name: &str) -> Option<DeclarationStatus> {
        self.scopes.last_mut()?.declare(name)
    }

    pub fn define(&mut self, name: &str) -> Option<DeclarationStatus> {
        self.scopes.last_mut()?.define(name)
    }

    pub fn is_declared(&self, name: &str) -> bool {
        matches!(
            self.scopes.last().and_then(|s| s.values.get(name)),
            Some(DeclarationStatus::Declared)
        )
    }

    pub fn find_depth(&self, name: &str) -> Option<usize> {
        /* NOTE: iterate last to first */
        for depth in 0..self.scopes.len() {
            let idx = (self.scopes.len() - 1) - depth;

            if self.scopes[idx].values.contains_key(name) {
                return Some(depth);
            }
        }

        None
    }
}

pub fn resolve(interpreter: &mut Interpreter, program: &Program) -> Result<(), ResolveErrors> {
    let mut resolver = Resolver::new(interpreter);

    resolver.resolve_statements(&program.0);

    if resolver.errors.is_empty() {
        Ok(())
    } else {
        Err(ResolveErrors(resolver.errors))
    }
}

#[derive(Debug)]
pub struct Resolver<'a> {
    pub interpreter: &'a mut Interpreter,
    pub scopes: ScopeStack,
    pub errors: Vec<Error>,
}

impl<'a> Resolver<'a> {
    pub fn new(interpreter: &'a mut Interpreter) -> Self {
        Self {
            interpreter,
            scopes: ScopeStack::default(),
            errors: Vec::new(),
        }
    }

    pub fn resolve_statements(&mut self, stmts: &[Stmt]) {
        for stmt in stmts {
            self.resolve_statement(stmt);
        }
    }

    pub fn resolve_statement(&mut self, stmt: &Stmt) {
        match &stmt.kind {
            StmtKind::Expression(s) => self.resolve_expression_statement(s),
            StmtKind::Print(s) => self.resolve_print_statement(s),
            StmtKind::Var(s) => self.resolve_var_statement(s),
            StmtKind::Block(s) => self.resolve_block_statement(s),
            StmtKind::If(s) => self.resolve_if_statement(s),
            StmtKind::Return(s) => self.resolve_return_statement(s),
            StmtKind::While(s) => self.resolve_while_statement(s),
            StmtKind::Function(s) => self.resolve_function_statement(s),
            StmtKind::Break(_) => {} /* NOTE: nothing to do here */
        }
    }

    pub fn resolve_expression_statement(&mut self, kind: &ExpressionStmt) {
        self.resolve_expression(&kind.expression);
    }

    pub fn resolve_print_statement(&mut self, kind: &PrintStmt) {
        self.resolve_expression(&kind.expression);
    }

    pub fn resolve_var_statement(&mut self, kind: &VarStmt) {
        self.declare(&kind.name);

        if let Some(init) = &kind.initializer {
            self.resolve_expression(&init);
        }

        self.define(&kind.name);
    }

    pub fn resolve_block_statement(&mut self, kind: &BlockStmt) {
        self.begin_scope();
        self.resolve_statements(&kind.statements);
        self.end_scope();
    }

    pub fn resolve_if_statement(&mut self, kind: &IfStmt) {
        self.resolve_expression(&kind.condition);
        self.resolve_statement(&kind.then_branch);

        if let Some(else_branch) = &kind.else_branch {
            self.resolve_statement(&else_branch);
        }
    }

    pub fn resolve_return_statement(&mut self, kind: &ReturnStmt) {
        if let Some(value) = &kind.value {
            self.resolve_expression(&value);
        }
    }

    pub fn resolve_while_statement(&mut self, kind: &WhileStmt) {
        self.resolve_expression(&kind.condition);
        self.resolve_statement(&kind.body);
    }

    pub fn resolve_function_statement(&mut self, kind: &FunctionStmt) {
        self.declare(&kind.name);
        self.define(&kind.name);

        self.resolve_function_def(&kind.def);
    }

    pub fn resolve_function_def(&mut self, def: &FunctionDef) {
        self.begin_scope();

        for param in &def.params {
            self.declare(&param);
            self.define(&param);
        }

        self.resolve_statements(&def.statements);

        self.end_scope();
    }

    pub fn resolve_expression(&mut self, expr: &Expr) {
        match &expr.kind {
            ExprKind::Literal(e) => {} /* nothing to do */
            ExprKind::Logical(e) => self.resolve_logical_expression(e),
            ExprKind::Variable(e) => self.resolve_variable_expression(expr, e),
            ExprKind::Assign(e) => self.resolve_assign_expression(expr, e),
            ExprKind::Unary(e) => self.resolve_unary_expression(e),
            ExprKind::Binary(e) => self.resolve_binary_expression(e),
            ExprKind::Call(e) => self.resolve_call_expression(e),
            ExprKind::Grouping(e) => self.resolve_grouping_expression(e),
            ExprKind::Function(e) => self.resolve_function_expression(e),
        }
    }

    pub fn resolve_logical_expression(&mut self, kind: &LogicalExpr) {
        self.resolve_expression(&kind.left);
        self.resolve_expression(&kind.right);
    }

    pub fn resolve_variable_expression(&mut self, expr: &Expr, kind: &VariableExpr) {
        if !self.scopes.is_empty() && self.is_declared(&kind.name) {
            self.errors.push(Error::var_used_in_declaration(expr.span));
        }

        self.resolve_local(expr, &kind.name);
    }

    pub fn resolve_assign_expression(&mut self, expr: &Expr, kind: &AssignExpr) {
        self.resolve_expression(kind.expression.as_ref());
        self.resolve_local(expr, &kind.name);
    }

    fn resolve_local(&mut self, expr: &Expr, name: &Token) {
        if let Some(i) = self.scopes.find_depth(&name.lexeme) {
            self.interpreter.resolve(expr.id, i);
        }
    }

    pub fn resolve_unary_expression(&mut self, kind: &UnaryExpr) {
        self.resolve_expression(&kind.right);
    }

    pub fn resolve_binary_expression(&mut self, kind: &BinaryExpr) {
        self.resolve_expression(&kind.left);
        self.resolve_expression(&kind.right);
    }

    pub fn resolve_call_expression(&mut self, kind: &CallExpr) {
        self.resolve_expression(&kind.callee);

        for argument in &kind.arguments {
            self.resolve_expression(&argument);
        }
    }

    pub fn resolve_grouping_expression(&mut self, kind: &GroupingExpr) {
        self.resolve_expression(&kind.expression);
    }

    pub fn resolve_function_expression(&mut self, kind: &FunctionExpr) {
        self.resolve_function_def(&kind.def);
    }

    /* Helpers */

    fn begin_scope(&mut self) {
        self.scopes.begin_scope();
    }

    fn end_scope(&mut self) {
        self.scopes.end_scope();
    }

    fn declare(&mut self, name: &Token) {
        self.scopes.declare(&name.lexeme);
    }

    fn define(&mut self, name: &Token) {
        self.scopes.define(&name.lexeme);
    }

    fn is_declared(&mut self, name: &Token) -> bool {
        self.scopes.is_declared(&name.lexeme)
    }
}
