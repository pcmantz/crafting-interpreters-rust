/* parser.rs
 *
 */

use crate::prelude::*;

use crate::error::*;
use crate::expr::*;
use crate::stmt::*;
use crate::token::*;
use crate::value::*;

pub fn parse(tokens: Vec<Token>) -> Result<Program, ParseErrors> {
    let parser = Parser::default();

    parser.parse(tokens)
}

pub struct Parser {
    tokens: Vec<Token>,
    current: usize,
    statements: Vec<Stmt>,
    errors: Vec<Error>,
}

impl Default for Parser {
    fn default() -> Self {
        Self {
            tokens: Vec::new(),
            current: 0,
            statements: Vec::new(),
            errors: Vec::new(),
        }
    }
}

impl Parser {
    /* Patterns */

    fn parse(mut self, tokens: Vec<Token>) -> Result<Program, ParseErrors> {
        self.tokens = tokens;

        while !self.is_at_end() {
            match self.declaration() {
                Ok(decl) => self.statements.push(decl),
                Err(err) => self.errors.push(err),
            }
        }

        if self.errors.is_empty() {
            Ok(Program(self.statements))
        } else {
            Err(ParseErrors(self.errors))
        }
    }

    fn declaration(&mut self) -> Result<Stmt, Error> {
        if self.matches(&[TokenType::Var]) {
            match self.var_declaration() {
                Ok(stmt) => Ok(stmt),
                Err(e) => {
                    self.synchronize();
                    Err(e)
                }
            }
        } else {
            match self.statement() {
                Ok(stmt) => Ok(stmt),
                Err(e) => {
                    self.synchronize();
                    Err(e)
                }
            }
        }
    }

    fn var_declaration(&mut self) -> Result<Stmt, Error> {
        let name = self.consume_identifier()?;

        let initializer = if self.matches(&[TokenType::Equal]) {
            Some(self.expression()?)
        } else {
            None
        };

        self.consume(TokenType::Semicolon)?;
        Ok(Stmt::var(name, initializer))
    }

    fn statement(&mut self) -> Result<Stmt, Error> {
        let ty = self.peek().ty.clone();

        match ty {
            TokenType::For => {
                self.consume(TokenType::For)?;
                self.for_statement()
            }
            TokenType::If => {
                self.consume(TokenType::If)?;
                self.if_statement()
            }
            TokenType::Print => {
                self.consume(TokenType::Print)?;
                self.print_statement()
            }
            TokenType::While => {
                self.consume(TokenType::While)?;
                self.while_statement()
            }
            TokenType::LeftBrace => {
                self.consume(TokenType::LeftBrace)?;
                self.block_statement()
            }
            _ => self.expression_statement(),
        }
    }

    fn for_statement(&mut self) -> Result<Stmt, Error> {
        let _ = self.consume(TokenType::LeftParen)?;

        /* Initializer*/
        let initializer = if self.matches(&[TokenType::Semicolon]) {
            None
        } else if self.matches(&[TokenType::Var]) {
            let decl = self.var_declaration()?;
            Some(decl)
        } else {
            let expr = self.expression_statement()?;
            Some(expr)
        };

        /* Condition */
        let condition = if !self.check(&TokenType::Semicolon) {
            let expr = self.expression()?;
            self.consume(TokenType::Semicolon)?;
            Some(expr)
        } else {
            None
        };

        /* Increment */
        let increment = if !self.check(&TokenType::RightParen) {
            Some(Stmt::expression(self.expression()?))
        } else {
            None
        };

        let _ = self.consume(TokenType::RightParen)?;

        /* Body */
        let body = self.statement()?;

        /* Condition is true if it isn't explicitly set */
        let cond = match condition {
            Some(stmt) => stmt,
            None => Expr::literal(Value::Bool(true)),
        };

        /* If present, put the increment statement at the end of the body. */
        let while_loop = Stmt::r#while(
            cond,
            match increment {
                Some(incr) => Stmt::block(vec![body, incr]),
                None => body,
            },
        );

        /* If present, put the initializer before the while loop */
        let for_loop = match initializer {
            Some(init) => Stmt::block(vec![init, while_loop]),
            None => while_loop,
        };

        Ok(for_loop)
    }

    fn if_statement(&mut self) -> Result<Stmt, Error> {
        let _ = self.consume(TokenType::LeftParen)?;
        let condition = self.expression()?;
        let _ = self.consume(TokenType::RightParen)?;

        let then_branch = self.statement()?;
        let else_branch = if self.matches(&[TokenType::Else]) {
            Some(self.statement()?)
        } else {
            None
        };

        Ok(Stmt::r#if(condition, then_branch, else_branch))
    }

    fn print_statement(&mut self) -> Result<Stmt, Error> {
        let value = self.expression()?;
        self.consume(TokenType::Semicolon)?;

        Ok(Stmt::print(value))
    }

    fn while_statement(&mut self) -> Result<Stmt, Error> {
        let _ = self.consume(TokenType::LeftParen)?;
        let condition = self.expression()?;
        let _ = self.consume(TokenType::RightParen)?;

        let body = self.statement()?;

        Ok(Stmt::r#while(condition, body))
    }

    fn block_statement(&mut self) -> Result<Stmt, Error> {
        let mut statements: Vec<Stmt> = Vec::new();

        while !self.check(&TokenType::RightBrace) && !self.is_at_end() {
            let decl = self.declaration()?;
            statements.push(decl);
        }

        self.consume(TokenType::RightBrace)?;

        Ok(Stmt::block(statements))
    }

    fn expression_statement(&mut self) -> Result<Stmt, Error> {
        let expr = self.expression()?;
        self.consume(TokenType::Semicolon)?;

        Ok(Stmt::expression(expr))
    }

    fn expression(&mut self) -> Result<Expr, Error> {
        self.assignment()
    }

    fn assignment(&mut self) -> Result<Expr, Error> {
        let expr = self.logic_or()?;

        if self.matches(&[TokenType::Equal]) {
            let _equals = self.previous(); /* don't really need */
            let value = self.assignment()?;

            match expr {
                Expr::Variable(e) => {
                    let name = e.name;
                    Ok(Expr::assign(name, value))
                }

                _ => Err(Error::invalid_assignment(self.peek())),
            }
        } else {
            Ok(expr)
        }
    }

    fn logic_or(&mut self) -> Result<Expr, Error> {
        let mut expr = self.logic_and()?;

        while self.matches(&[TokenType::Or]) {
            let operator = self.previous().clone();
            let right = self.logic_and()?;

            expr = Expr::logical(expr, operator, right);
        }

        Ok(expr)
    }

    fn logic_and(&mut self) -> Result<Expr, Error> {
        let mut expr = self.equality()?;

        while self.matches(&[TokenType::And]) {
            let operator = self.previous().clone();
            let right = self.logic_and()?;

            expr = Expr::logical(expr, operator, right);
        }

        Ok(expr)
    }

    fn equality(&mut self) -> Result<Expr, Error> {
        let mut expr: Expr = self.comparison()?;

        while self.matches(&[TokenType::EqualEqual, TokenType::BangEqual]) {
            let operator = self.previous().clone();
            let right = self.comparison()?;

            expr = Expr::binary(expr, operator, right);
        }

        Ok(expr)
    }

    fn comparison(&mut self) -> Result<Expr, Error> {
        let mut expr: Expr = self.term()?;

        while self.matches(&[
            TokenType::Greater,
            TokenType::GreaterEqual,
            TokenType::Less,
            TokenType::LessEqual,
        ]) {
            let operator = self.previous().clone();
            let right = self.term()?;

            expr = Expr::binary(expr, operator, right);
        }

        Ok(expr)
    }

    fn term(&mut self) -> Result<Expr, Error> {
        let mut expr: Expr = self.factor()?;

        while self.matches(&[TokenType::Plus, TokenType::Minus]) {
            let operator = self.previous().clone();
            let right = self.factor()?;

            expr = Expr::binary(expr, operator, right);
        }

        Ok(expr)
    }

    fn factor(&mut self) -> Result<Expr, Error> {
        let mut expr: Expr = self.unary()?;

        while self.matches(&[TokenType::Slash, TokenType::Star]) {
            let operator = self.previous().clone();
            let right = self.unary()?;

            expr = Expr::binary(expr, operator, right);
        }

        Ok(expr)
    }

    fn unary(&mut self) -> Result<Expr, Error> {
        if self.matches(&[TokenType::Bang, TokenType::Minus]) {
            let operator = self.previous().clone();
            let right = self.unary()?;

            return Ok(Expr::unary(operator, right));
        }

        self.primary()
    }

    fn primary(&mut self) -> Result<Expr, Error> {
        let token = self.advance();

        match token.ty {
            TokenType::LeftParen => {
                let expr = self.expression()?;
                self.consume(TokenType::RightParen)?;

                Ok(Expr::grouping(expr))
            }

            TokenType::Identifier(_) => Ok(Expr::variable(token.clone())),

            /* NOTE: This tries to pull a value, otherwise it errors. May have to
             * explode into match later.
             */
            _ => Value::from_token(token.clone())
                .map(Expr::literal)
                .ok_or_else(|| Error::missing_expression(&token, "Expected primary expression.")),
        }
    }

    /* Helper Functions */

    fn consume(&mut self, ty: TokenType) -> Result<Token, Error> {
        if self.check(&ty) {
            Ok(self.advance().clone())
        } else {
            Err(Error::wrong_token(ty, self.peek()))
        }
    }

    fn consume_identifier(&mut self) -> Result<Token, Error> {
        if matches!(self.peek().ty, TokenType::Identifier(_)) {
            Ok(self.advance().clone())
        } else {
            Err(Error::wrong_token(
                TokenType::Identifier(String::new()),
                self.peek(),
            ))
        }
    }

    fn synchronize(&mut self) {
        self.advance();

        while !self.is_at_end() {
            let prev = self.previous();

            if prev.ty == TokenType::Semicolon {
                return;
            }

            let curr = self.peek();

            match curr.ty {
                TokenType::Class
                | TokenType::For
                | TokenType::Fun
                | TokenType::If
                | TokenType::Print
                | TokenType::Return
                | TokenType::Var
                | TokenType::While => return,
                _ => {}
            }

            self.advance();
        }
    }

    fn previous(&self) -> &Token {
        &self.tokens[self.current - 1]
    }

    fn peek(&self) -> &Token {
        &self.tokens[self.current]
    }

    fn next(&self) -> &Token {
        if self.is_at_end() {
            self.peek()
        } else {
            &self.tokens[self.current + 1]
        }
    }

    fn advance(&mut self) -> &Token {
        if !self.is_at_end() {
            self.current += 1;
        }

        self.previous()
    }

    fn retreat(&mut self) -> &Token {
        if !self.current == 0 {
            self.current -= 1;
        }

        self.next()
    }

    fn matches(&mut self, tokens: &[TokenType]) -> bool {
        let token = self.peek();

        if tokens.contains(&token.ty) {
            self.advance();

            true
        } else {
            false
        }
    }

    fn check(&self, ty: &TokenType) -> bool {
        if self.is_at_end() {
            return false;
        }

        self.peek().ty == *ty
    }

    fn is_at_end(&self) -> bool {
        self.peek().ty == TokenType::EOF
    }
}

#[cfg(test)]
mod tests {
    use std::string::String;

    use crate::scanner;

    use super::*;

    fn sexpr(src: &str) -> String {
        let tokens = scanner::scan(src.to_string()).expect("scanning failed.");
        let ast = parse(tokens).expect("parsing failed.");

        ast.to_string()
    }

    #[test]
    fn parse_number() {
        assert_eq!(sexpr("123;"), "(expr 123)")
    }

    #[test]
    fn parse_true() {
        assert_eq!(sexpr("true;"), "(expr true)")
    }

    #[test]
    fn parse_false() {
        assert_eq!(sexpr("false;"), "(expr false)")
    }

    #[test]
    fn parse_nil() {
        assert_eq!(sexpr("nil;"), "(expr nil)")
    }

    #[test]
    fn parse_identifier() {
        assert_eq!(sexpr("identifier;"), "(expr Identifier(\"identifier\"))")
    }

    #[test]
    fn parse_addition() {
        assert_eq!(sexpr("1 + 2;"), "(expr (+ 1 2))")
    }

    #[test]
    fn parse_division() {
        assert_eq!(sexpr("1 / 2;"), "(expr (/ 1 2))")
    }

    #[test]
    fn parse_comparison() {
        assert_eq!(sexpr("3 < 5;"), "(expr (< 3 5))")
    }

    #[test]
    fn parse_equality() {
        assert_eq!(sexpr("100 == 100;"), "(expr (== 100 100))")
    }

    #[test]
    fn parse_negation() {
        assert_eq!(sexpr("-32;"), "(expr (- 32))")
    }

    #[test]
    fn parse_grouping() {
        assert_eq!(sexpr("1 / (2 + 3);"), "(expr (/ 1 (group (+ 2 3))))")
    }

    #[test]
    fn parse_mult_addition_ordering() {
        assert_eq!(sexpr("1 / (2 + 3);"), "(expr (/ 1 (group (+ 2 3))))")
    }

    #[test]
    fn parse_var_decl() {
        assert_eq!(sexpr("var a;"), "(var a)")
    }

    #[test]
    fn parse_var_decl_assign() {
        assert_eq!(sexpr("var a = 5;"), "(var a 5)")
    }

    #[test]
    fn parse_assignment_expr() {
        assert_eq!(sexpr("a = 5;"), "(expr (= Identifier(\"a\") 5))")
    }

    #[test]
    fn parse_multiple_statements() {
        assert_eq!(sexpr("1; 2;"), "(expr 1)(expr 2)")
    }

    #[test]
    fn parse_block_statement() {
        assert_eq!(
            sexpr("{ var a; a = 1; }"),
            r#"(block (var a)(expr (= Identifier("a") 1)))"#
        )
    }

    #[test]
    fn parse_if_statement() {
        assert_eq!(sexpr("if (true) 1; else 0;"), "(if true (expr 1) (expr 0))")
    }

    #[test]
    fn parse_logical_and() {
        assert_eq!(sexpr("1 and 2;"), "(expr (and 1 2))")
    }

    #[test]
    fn parse_logical_or() {
        assert_eq!(sexpr("1 or 2;"), "(expr (or 1 2))")
    }

    #[test]
    fn parse_while_statement() {
        assert_eq!(
            sexpr(
                r#"
var a = 1;
while (a < 10) {
    a = a + 1;
}
"#
            ),
            "(var a 1)(while (< Identifier(\"a\") 10) (block (expr (= Identifier(\"a\") (+ Identifier(\"a\") 1)))))"
        )
    }

    #[test]
    fn parse_for_statement() {
        assert_eq!(
            sexpr(
                r#"
for (var i = 0; i < 10; i = i + 1) {
    print i;
}
"#
            ),
            "(block (var i 0)(while (< Identifier(\"i\") 10) (block (block (print Identifier(\"i\")))(expr (= Identifier(\"i\") (+ Identifier(\"i\") 1))))))"
        )
    }




}
