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

#[derive(Debug)]
pub struct Scanner {
    source: Vec<u8>,
    tokens: Vec<Token>,
    err: Option<Error>,
    start: Pos,
    current: Pos,
}

impl Default for Scanner {
    fn default() -> Self {
        Self {
            source: Vec::new(),
            tokens: Vec::new(),
            err: None,
            start: Pos::default(),
            current: Pos::default(),
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
                    self.error(format!("scanner can't handle {}", c));
                }
            }
        }
    }

    fn advance(&mut self) -> char {
        self.current.offset += 1;
        // self.col += 1;

        char::from(self.source[self.current.offset - 1])
    }

    fn peek(&self) -> char {
        if self.done() {
            '\0'
        } else {
            char::from(self.source[self.current.offset])
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
        // self.col = -1;
        // self.line += 1;
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

        self.error("unterminated comment.");
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

        let val: f64 = self
            .substr(self.start.offset, self.current.offset)
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
            self.error("unterminated string.".to_string());

            return;
        }

        /* consume the closing brace. TODO: error handling here with matches? */
        self.advance();

        let str = self.substr(self.start.offset + 1, self.current.offset - 1);

        self.add_token(TokenType::Str(str));
    }

    fn identifier(&mut self) {
        while Self::is_alphanumeric(self.peek()) {
            self.advance();
        }

        let str = self.substr(self.start.offset, self.current.offset);

        self.add_token(match keyword(&str) {
            Some(ty) => ty,
            None => TokenType::Identifier(str),
        })
    }

    fn add_token(&mut self, token_type: TokenType) {
        let str = self.substr(self.start.offset, self.current.offset);

        let token = Token {
            ty: token_type,
            lexeme: str,
            span: Span {
                start: self.start,
                end: self.current,
            },
        };

        self.tokens.push(token);
    }

    fn error(&mut self, message: impl Into<String>) {
        self.err = Some(Error::scanner(
            message.into(),
            Span {
                start: self.start,
                end: self.current,
            },
        ))
    }

    fn substr(&mut self, from: usize, to: usize) -> String {
        self.source[from..to]
            .to_vec()
            .pipe(String::from_utf8)
            .unwrap()
    }

    fn done(&self) -> bool {
        self.err.is_some() || self.is_at_end()
    }

    fn is_at_end(&self) -> bool {
        self.current.offset >= self.source.len()
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
        assert_eq!(
            types("identifier"),
            vec![Identifier(String::from("identifier")), EOF]
        )
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
        assert_eq!(
            types("1 + 2"),
            vec![Num(1 as f64), Plus, Num(2 as f64), EOF]
        )
    }

    #[test]
    fn scan_comment() {
        assert_eq!(types("//comment"), vec![EOF])
    }

    #[test]
    fn scan_multiline_comment() {
        assert_eq!(types("/* comment */"), vec![EOF])
    }

    #[test]
    fn scan_hello_world() {
        assert_eq!(
            types(r#"print "Hello, World!";"#),
            vec![Print, Str(String::from("Hello, World!")), Semicolon, EOF]
        )
    }

    #[test]
    fn scan_block() {
        assert_eq!(
            types("{ var b = 3; b; }"),
            vec![
                LeftBrace,
                Var,
                Identifier(String::from("b")),
                Equal,
                Num(3 as f64),
                Semicolon,
                Identifier(String::from("b")),
                Semicolon,
                RightBrace,
                EOF
            ]
        )
    }

    #[test]
    fn scan_for_statement() {
        assert_eq!(
            types(
                r#"
for (var i = 0; i < 10; i = i + 1) {
    print i;
}
"#
            ),
            vec![
                For,
                LeftParen,
                Var,
                Identifier("i".to_string()),
                Equal,
                Num(0.0),
                Semicolon,
                Identifier("i".to_string()),
                Less,
                Num(10.0),
                Semicolon,
                Identifier("i".to_string()),
                Equal,
                Identifier("i".to_string()),
                Plus,
                Num(1.0),
                RightParen,
                LeftBrace,
                Print,
                Identifier("i".to_string()),
                Semicolon,
                RightBrace,
                EOF
            ]
        )
    }
}
