/* parser.rs
 *
 */

use crate::prelude::*;

use crate::error::*;
use crate::expr::*;
use crate::token::*;

pub fn parse(tokens: Vec<Token>) {
    let mut parser = Parser {
        tokens,
        current: 0,
    };

    parser.expression();
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

            expr = Expr::Binary(BinaryExpr {
                left: Box::new(expr),
                operator,
                right: Box::new(right),
            });
        }

        Ok(expr)
    }

    fn comparison(&mut self) -> Result<Expr, Error> {
        let mut expr: Expr = self.factor()?;

        while self.matches(vec![
            TokenType::Greater,
            TokenType::GreaterEqual,
            TokenType::Less,
            TokenType::LessEqual,
        ]) {
            let operator = self.previous().clone();
            let right = self.term()?;

            expr = Expr::Binary(BinaryExpr {
                left: Box::new(expr),
                operator,
                right: Box::new(right),
            });
        }

        Ok(expr)
    }

    fn term(&mut self) -> Result<Expr, Error> {
        let mut expr: Expr = self.factor()?;

        while self.matches(vec![TokenType::Slash, TokenType::Star]) {
            let operator = self.previous().clone();
            let right = self.factor()?;

            expr = Expr::Binary(BinaryExpr {
                left: Box::new(expr),
                operator,
                right: Box::new(right),
            });
        }

        Ok(expr)
    }

    fn factor(&mut self) -> Result<Expr, Error> {
        let mut expr: Expr = self.unary()?;

        while self.matches(vec![TokenType::Slash, TokenType::Star]) {
            let operator = self.previous().clone();
            let right = self.unary()?;

            expr = Expr::Binary(BinaryExpr {
                left: Box::new(expr),
                operator,
                right: Box::new(right),
            });
        }

        Ok(expr)
    }

    fn unary(&mut self) -> Result<Expr, Error> {
        if self.matches(vec![TokenType::Bang, TokenType::Minus]) {
            let operator = self.previous().clone();
            let right = self.unary()?;
            return Ok(Expr::Unary(UnaryExpr {
                operator,
                right: Box::new(right),
            }));
        }

        self.primary()
    }

    fn primary(&mut self) -> Result<Expr, Error> {
        let token = self.advance();

        match token.ty {
            TokenType::False => Ok(Expr::Literal(LiteralExpr {
                value: Literal::False,
            })),
            TokenType::True => Ok(Expr::Literal(LiteralExpr {
                value: Literal::True,
            })),
            TokenType::Nil => Ok(Expr::Literal(LiteralExpr {
                value: Literal::Nil,
            })),
            TokenType::Number | TokenType::String => Ok(Expr::Literal(LiteralExpr {
                value: token.literal.clone().unwrap(),
            })),

            TokenType::LeftParen => {
                let expr = self.expression()?;
                self.consume(TokenType::RightParen, "Expect ')' after expression.")?;

                Ok(Expr::Grouping(GroupingExpr {
                    expression: Box::new(expr),
                }))
            }

            /* This should throw an error. */
            _ => Err(Error::MissingExpression {
                message: "Expected primary expression.".into(),
            }),
        }
    }

    /* Helper Functions */

    fn consume(&mut self, ty: TokenType, message: impl Into<String>) -> Result<Token, Error> {
        if self.check(ty) {
            Ok(self.advance().clone())
        } else {
            Err(Error::WrongToken {
                message: message.into(),
            })
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
