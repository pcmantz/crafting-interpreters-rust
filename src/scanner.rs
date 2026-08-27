/* scanner.rs
 *
 */

use crate::error::*;
use crate::prelude::*;
use crate::token::*;

pub fn scan(input: String) -> Result<Vec<Token>, Error> {
    let mut scanner: Scanner = Scanner::default();

    scanner.scan(input);

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
    fn default() -> Self {
        Self {
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
    fn scan(&mut self, input: String) {
        self.source = input.into_bytes();

        while !self.done() {
            self.start = self.current;
            self.scan_token();
        }

        match self.err {
            Some(_) => {}
            None => {
                /* Add the EOF character explicitly, reset the scanner so it
                 * doesn't consume the last token as the EOF
                 */
                self.start = self.current;

                self.add_token(TokenType::EOF)
            }
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
                if self.matches('=') {
                    self.add_token(TokenType::BangEqual)
                } else {
                    self.add_token(TokenType::Bang)
                }
            }

            '=' => {
                if self.matches('=') {
                    self.add_token(TokenType::EqualEqual)
                } else {
                    self.add_token(TokenType::Equal)
                }
            }

            '<' => {
                if self.matches('=') {
                    self.add_token(TokenType::LessEqual)
                } else {
                    self.add_token(TokenType::Less)
                }
            }

            '>' => {
                if self.matches('=') {
                    self.add_token(TokenType::GreaterEqual)
                } else {
                    self.add_token(TokenType::Greater)
                }
            }

            '/' => {
                if self.matches('/') {
                    self.comment();
                } else if self.matches('*') {
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

    fn peek(&self) -> char {
        if self.done() {
            '\0'
        } else {
            char::from(self.source[self.current])
        }
    }

    fn matches(&mut self, ch: char) -> bool {
        if self.peek() == ch {
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
        let next = self.peek();

        while next != '\n' && !self.is_at_end() {
            self.advance();
        }
    }

    fn multiline_comment(&mut self) {
        while !self.done() {
            let c = self.advance();

            match c {
                '*' => {
                    if self.matches('/') {
                        return;
                    } else {
                        /* do nothing */
                    }
                }

                '/' => {
                    if self.matches('*') {
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
        while Self::is_digit(self.peek()) {
            self.advance();
        }

        if self.peek() == '.' {
            self.advance();

            while Self::is_digit(self.peek()) {
                self.advance();
            }
        }

        let val: f64 = self.source[self.start..self.current]
            .to_vec()
            .pipe(String::from_utf8)
            .unwrap()
            .parse()
            .unwrap();

        self.add_token(TokenType::Num(val))
    }

    fn string(&mut self) {
        while self.peek() != '"' && !self.is_at_end() {
            if self.peek() == '\n' {
                self.newline();
            }

            self.advance();
        }

        if self.is_at_end() {
            self.err = Some(Error::scanner(
                "unterminated string.".to_string(),
                self.line,
                self.col,
            ));

            return;
        }

        /* consume the closing brace. TODO: error handling here with matches? */
        self.advance();

        let str = self.source[self.start + 1..self.current - 1]
            .to_vec()
            .pipe(String::from_utf8)
            .unwrap();

        self.add_token(TokenType::Str(str));
    }

    fn identifier(&mut self) {
        while Self::is_alphanumeric(self.peek()) {
            self.advance();
        }

        let str = String::from_utf8(self.source[self.start..self.current].to_vec()).unwrap();

        self.add_token(match keyword(&str) {
            Some(ty) => ty,
            None => TokenType::Identifier(str),
        })
    }

    fn add_token(&mut self, token_type: TokenType) {
        let text = self.source[self.start..self.current]
            .to_vec()
            .pipe(String::from_utf8)
            .expect("source was valid UTF-8");

        let token = Token {
            ty: token_type,
            lexeme: text,
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

    /* Helpers */

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

#[cfg(test)]
mod test {
    use super::*;

    use crate::scanner;
    use crate::token::TokenType::*;

    fn types(src: &str) -> Vec<TokenType> {
        src.to_string()
            .pipe(scanner::scan)
            .unwrap()
            .into_iter()
            .map(|t| t.ty)
            .collect()
    }

    #[test]
    fn handles_empty_input() {
        assert_eq!(types(""), vec![EOF])
    }

    #[test]
    fn literal_true() {
        assert_eq!(types("true"), vec![True, EOF])
    }

    #[test]
    fn literal_false() {
        assert_eq!(types("false"), vec![False, EOF])
    }

    #[test]
    fn literal_nil() {
        assert_eq!(types("nil"), vec![Nil, EOF])
    }

    #[test]
    fn literal_identifier() {
        assert_eq!(types("identifier"), vec![Identifier(String::from("identifier")), EOF])
    }

    #[test]
    fn literal_string() {
        assert_eq!(types("\"string\""), vec![Str(String::from("string")), EOF])
    }

    #[test]
    fn literal_number() {
        assert_eq!(types("1234"), vec![Num(1234 as f64), EOF])
    }

    #[test]
    fn scan_addition() {
        assert_eq!(types("1 + 2"), vec![Num(1 as f64), Plus, Num(2 as f64), EOF])
    }

    #[test]
    fn scan_comment() {
        assert_eq!(types("//comment"), vec![EOF])
    }

    #[test]
    fn scan_multiline_comment() {
        assert_eq!(types("/* comment */"), vec![EOF])
    }

    // #[test]
    fn hello_world() {}
}
