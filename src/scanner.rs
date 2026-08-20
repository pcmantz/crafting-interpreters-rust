/* scanner.rs
 *
 */

use crate::error::*;
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
            '*' => self.add_token(TokenType::Star),

            /* Potential double character tokens */
            '!' => {
                if self.maybe_match('=') {
                    self.add_token(TokenType::BangEqual)
                } else {
                    self.add_token(TokenType::Bang)
                }
            }

            '=' => {
                if self.maybe_match('=') {
                    self.add_token(TokenType::EqualEqual)
                } else {
                    self.add_token(TokenType::Equal)
                }
            }

            '<' => {
                if self.maybe_match('=') {
                    self.add_token(TokenType::LessEqual)
                } else {
                    self.add_token(TokenType::Less)
                }
            }

            '>' => {
                if self.maybe_match('=') {
                    self.add_token(TokenType::GreaterEqual)
                } else {
                    self.add_token(TokenType::Greater)
                }
            }

            '/' => {
                if self.maybe_match('/') {
                    self.comment();
                } else if self.maybe_match('*') {
                    self.multiline_comment();
                } else {
                    self.add_token(TokenType::Slash);
                }
            }

            ' ' => {} /* do nothing */

            '\r' => {} /* do nothing */

            '\t' => {} /* do nothing */

            '\n' => self.newline(),

            '"' => self.string(),

            _ => {
                if Self::is_digit(c) {
                    self.number();
                } else if Self::is_alpha(c) {
                    self.identifier();
                } else {
                    self.err = Some(Error::scanner(
                        format!("scanner can't handle {}", c),
                        self.line,
                        self.col,
                    ))
                }
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

    fn maybe_match(&mut self, ch: char) -> bool {
        if let Some(next) = self.peek()
            && next == ch
        {
            self.advance();

            true
        } else {
            false
        }
    }

    fn newline(&mut self) {
        self.col = -1;
        self.line += 1;
    }

    fn comment(&mut self) {
        while let Some(next) = self.peek()
            && next != '\n'
            && !self.is_at_end()
        {
            self.advance();
        }
    }

    fn multiline_comment(&mut self) {
        while !self.done() {
            let c = self.advance();

            match c {
                '*' => {
                    if self.maybe_match('/') {
                        return;
                    } else {
                        /* do nothing */
                    }
                }

                '/' => {
                    if self.maybe_match('*') {
                        self.multiline_comment();
                    } else {
                        /* do nothing */
                    }
                }

                '\n' => self.newline(),

                _ => { /* do nothing */ }
            }
        }

        self.err = Some(Error::scanner("unterminated comment.", self.line, self.col))
    }

    fn number(&mut self) {
        self.advance_integer();

        while let Some(next) = self.peek()
            && next == '.'
        {
            self.advance();

            self.advance_integer();
        }

        let val: f64 = self.source[self.start..self.current]
            .to_vec()
            .pipe(String::from_utf8)
            .unwrap()
            .parse()
            .unwrap();

        self.add_token_literal(TokenType::Number, Some(Literal::Number(val)))
    }

    fn string(&mut self) {
        while let Some(next) = self.peek()
            && next != '"'
            && !self.is_at_end()
        {
            if next == '\n' {
                self.newline();
            }

            self.advance();
        }

        if self.is_at_end() {
            self.err = Some(Error::scanner(
                "unterminated string.".to_string(),
                self.line,
                self.col,
            ))
        }

        /* consume the closing brace. TODO: error handling here with maybe_match? */
        self.advance();

        let str = self.source[(self.start + 1)..(self.current - 1)]
            .to_vec()
            .pipe(String::from_utf8)
            .unwrap();

        self.add_token_literal(TokenType::String, Some(Literal::Str(str)));
    }

    fn identifier(&mut self) {
        while let Some(next) = self.peek()
            && Self::is_alphanumeric(next)
        {
            self.advance();
        }

        let str = String::from_utf8(self.source[self.start..self.current].to_vec()).unwrap();

        if let Some(token_type) = keyword(&str) {
            self.add_token(token_type);
        } else {
            self.add_token_literal(TokenType::Identifier, Some(Literal::Identifier(str)));
        }
    }

    fn advance_integer(&mut self) {
        while let Some(next) = self.peek()
            && Self::is_digit(next)
        {
            self.advance();
        }
    }

    fn add_token(&mut self, token_type: TokenType) {
        self.add_token_literal(token_type, None)
    }

    fn add_token_literal(&mut self, token_type: TokenType, literal: Option<Literal>) {
        let text = self.source[self.start..self.current]
            .to_vec()
            .pipe(String::from_utf8)
            .expect("source was valid UTF-8");

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

    fn is_digit(c: char) -> bool {
        c.is_ascii_digit()
    }

    fn is_alpha(c: char) -> bool {
        c.is_alphabetic()
    }

    fn is_alphanumeric(c: char) -> bool {
        Self::is_digit(c) || Self::is_alpha(c)
    }
}
