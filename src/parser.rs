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
        if self.matches(vec![TokenType::Var]) {
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

        let initializer = if self.matches(vec![TokenType::Equal]) {
            Some(self.expression()?)
        } else {
            None
        };

        self.consume(TokenType::Semicolon)?;
        Ok(Stmt::var(name, initializer))
    }

    fn statement(&mut self) -> Result<Stmt, Error> {
        if self.matches(vec![TokenType::Print]) {
            self.print_statement()
        } else {
            self.expression_statement()
        }
    }

    fn print_statement(&mut self) -> Result<Stmt, Error> {
        let value = self.expression()?;
        self.consume(TokenType::Semicolon)?;

        Ok(Stmt::print(value))
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
        let expr = self.equality()?;

        if self.matches(vec![TokenType::Equal]) {
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

    fn equality(&mut self) -> Result<Expr, Error> {
        let mut expr: Expr = self.comparison()?;

        while self.matches(vec![TokenType::EqualEqual, TokenType::BangEqual]) {
            let operator = self.previous().clone();
            let right = self.comparison()?;

            expr = Expr::binary(expr, operator, right);
        }

        Ok(expr)
    }

    fn comparison(&mut self) -> Result<Expr, Error> {
        let mut expr: Expr = self.term()?;

        while self.matches(vec![
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

        while self.matches(vec![TokenType::Plus, TokenType::Minus]) {
            let operator = self.previous().clone();
            let right = self.factor()?;

            expr = Expr::binary(expr, operator, right);
        }

        Ok(expr)
    }

    fn factor(&mut self) -> Result<Expr, Error> {
        let mut expr: Expr = self.unary()?;

        while self.matches(vec![TokenType::Slash, TokenType::Star]) {
            let operator = self.previous().clone();
            let right = self.unary()?;

            expr = Expr::binary(expr, operator, right);
        }

        Ok(expr)
    }

    fn unary(&mut self) -> Result<Expr, Error> {
        if self.matches(vec![TokenType::Bang, TokenType::Minus]) {
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

    fn advance(&mut self) -> &Token {
        if !self.is_at_end() {
            self.current += 1;
        }

        self.previous()
    }

    fn matches(&mut self, tokens: Vec<TokenType>) -> bool {
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
}
