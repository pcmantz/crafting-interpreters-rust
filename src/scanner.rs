/* scanner.rs
 *
 */

use crate::prelude::*;

use crate::token::*;

pub fn scan_tokens(input: String) -> Result<Vec<Token>, Error> {
    let mut scanner: Scanner = Scanner::default();

    scanner.scan_tokens(input);

    match scanner.err {
        Some(err) => Err(err),
        None => Ok(scanner.tokens),
    }
}

pub struct Scanner {
    source: Vec<u8>,
    tokens: Vec<Token>,
    err: Option<Error>,
    start: usize,
    current: usize,
    line: usize,
    col: i64,
}

impl Default for Scanner {
    fn default() -> Scanner {
        Scanner {
            source: Vec::new(),
            tokens: Vec::new(),
            err: None,
            start: 0,
            current: 0,
            line: 1,
            col: -1,
        }
    }
}

impl Scanner {
    fn scan_tokens(&mut self, input: String) {
        self.source = input.into_bytes();

        while !self.done() {
            self.start = self.current;
            self.scan_token();
        }

        match self.err {
            Some(_) => {}
            None => self.add_token(TokenType::EOF),
        }
    }

    fn scan_token(&mut self) {
        let c = self.advance();

        match c {
            /* Single character tokens */
            '(' => self.add_token(TokenType::LeftParen),
            ')' => self.add_token(TokenType::RightParen),
            '{' => self.add_token(TokenType::LeftBrace),
            '}' => self.add_token(TokenType::RightBrace),
            ',' => self.add_token(TokenType::Comma),
            '.' => self.add_token(TokenType::Dot),
            '-' => self.add_token(TokenType::Minus),
            '+' => self.add_token(TokenType::Plus),
            ';' => self.add_token(TokenType::Semicolon),
            '/' => self.add_token(TokenType::Slash),
            '*' => self.add_token(TokenType::Star),

            /* Potential double character tokens */
            '!' => {
                if let Some(next) = self.peek()
                    && next == '='
                {
                    self.advance();
                    self.add_token(TokenType::BangEqual)
                } else {
                    self.add_token(TokenType::Bang)
                }
            }

            '=' => {
                if let Some(next) = self.peek()
                    && next == '='
                {
                    self.advance();
                    self.add_token(TokenType::EqualEqual)
                } else {
                    self.add_token(TokenType::Equal)
                }
            }

            '<' => {
                if let Some(next) = self.peek()
                    && next == '='
                {
                    self.advance();
                    self.add_token(TokenType::LessEqual)
                } else {
                    self.add_token(TokenType::LessEqual)
                }
            }

            '>' => {
                if let Some(next) = self.peek()
                    && next == '='
                {
                    self.advance();
                    self.add_token(TokenType::GreaterEqual)
                } else {
                    self.add_token(TokenType::GreaterEqual)
                }
            }

            _ => {
                self.err = Some(Error {
                    what: format!("scanner can't handle {}", c),
                    line: self.line,
                    col: self.col,
                })
            }
        }
    }

    fn advance(&mut self) -> char {
        self.current += 1;
        self.col += 1;

        char::from(self.source[self.current - 1])
    }

    fn peek(&mut self) -> Option<char> {
        if self.done() {
            None
        } else {
            Some(char::from(self.source[self.current + 1]))
        }
    }

    fn add_token(&mut self, token_type: TokenType) {
        self.add_token_literal(token_type, None)
    }

    fn add_token_literal(&mut self, token_type: TokenType, literal: Option<Literal>) {
        let text = self.source[self.start..self.current].to_vec();

        let token = Token {
            ty: token_type,
            lexeme: text,
            literal,
            line: self.line,
            col: self.col,
        };

        self.tokens.push(token);
    }

    fn done(&self) -> bool {
        self.err.is_some() || self.is_at_end()
    }

    fn is_at_end(&self) -> bool {
        self.current >= self.source.len()
    }
}
