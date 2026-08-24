/* parser.rs
 *
 */

use crate::prelude::*;

use crate::error::*;
use crate::expr::*;
use crate::token::*;

pub fn parse(tokens: Vec<Token>) -> Result<Expr, Error> {
    let mut parser = Parser { tokens, current: 0 };

    /* TODO: This will change to something else */
    parser.expression()
}

pub struct Parser {
    tokens: Vec<Token>,
    current: usize,
}

impl Parser {
    /* Patterns */

    fn expression(&mut self) -> Result<Expr, Error> {
        let expr = self.equality()?;

        Ok(expr)
    }

    fn equality(&mut self) -> Result<Expr, Error> {
        let mut expr: Expr = self.comparison()?;

        while self.matches(vec![TokenType::Slash, TokenType::Star]) {
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

        while self.matches(vec![TokenType::Slash, TokenType::Star]) {
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
            TokenType::False => Ok(Expr::literal(Literal::False)),
            TokenType::True => Ok(Expr::literal(Literal::True)),
            TokenType::Nil => Ok(Expr::literal(Literal::Nil)),
            TokenType::Number | TokenType::String => {
                Ok(Expr::literal(token.literal.clone().unwrap()))
            }

            TokenType::LeftParen => {
                let expr = self.expression()?;
                self.consume(TokenType::RightParen)?;

                Ok(Expr::grouping(expr))
            }

            /* This should throw an error. */
            _ => Err(Error::missing_expression(
                "Expected primary expression.",
                token.line,
                token.col,
            )),
        }
    }

    /* Helper Functions */

    fn consume(&mut self, ty: TokenType) -> Result<Token, Error> {
        if self.check(ty) {
            Ok(self.advance().clone())
        } else {
            Err(Error::wrong_token(ty, self.peek()))
        }
    }

    // fn synchronize(&self) -> Type {}

    fn previous(&mut self) -> &Token {
        &self.tokens[self.current - 1]
    }

    fn peek(&mut self) -> &Token {
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

    fn check(&mut self, ty: TokenType) -> bool {
        if self.is_at_end() {
            return false;
        }

        self.peek().ty == ty
    }

    fn is_at_end(&mut self) -> bool {
        self.peek().ty == TokenType::EOF
    }
}

#[cfg(test)]
mod tests {
    use std::string::String;

    use crate::scanner;
    use crate::token::TokenType::*;

    use super::*;

    fn sexpr(src: &str) -> String {
        let tokens = scanner::scan(src.to_string()).expect("scanning failed.");
        let ast = parse(tokens).expect("parsing failed.");

        ast.to_string()
    }

    #[test]
    fn parse_literal() {
        assert_eq!(sexpr("123"), "123")
    }
}
