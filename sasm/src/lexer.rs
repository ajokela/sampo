//! Lexer for Sampo assembly language

use std::iter::Peekable;
use std::str::Chars;

#[derive(Debug, Clone, PartialEq)]
#[allow(dead_code)]
pub enum Token {
    // Registers
    Register(u8),
    // Numbers
    Number(i32),
    // Identifiers (labels, symbols)
    Ident(String),
    // String literals
    StringLit(String),
    // Punctuation
    Comma,
    Colon,
    LParen,
    RParen,
    Plus,
    Minus,
    // Directives
    Directive(String),
    // End of line
    Newline,
    // End of file
    Eof,
}

pub struct Lexer<'a> {
    input: Peekable<Chars<'a>>,
    line: usize,
    col: usize,
}

impl<'a> Lexer<'a> {
    pub fn new(input: &'a str) -> Self {
        Lexer {
            input: input.chars().peekable(),
            line: 1,
            col: 1,
        }
    }

    pub fn tokenize(&mut self) -> Result<Vec<Token>, String> {
        let mut tokens = Vec::new();

        loop {
            match self.next_token()? {
                Token::Eof => {
                    tokens.push(Token::Eof);
                    break;
                }
                token => tokens.push(token),
            }
        }

        Ok(tokens)
    }

    fn next_token(&mut self) -> Result<Token, String> {
        self.skip_whitespace();
        self.skip_comment();
        self.skip_whitespace();

        match self.peek() {
            None => Ok(Token::Eof),
            Some('\n') => {
                self.advance();
                self.line += 1;
                self.col = 1;
                Ok(Token::Newline)
            }
            Some('\r') => {
                self.advance();
                if self.peek() == Some('\n') {
                    self.advance();
                }
                self.line += 1;
                self.col = 1;
                Ok(Token::Newline)
            }
            Some(',') => {
                self.advance();
                Ok(Token::Comma)
            }
            Some(':') => {
                self.advance();
                Ok(Token::Colon)
            }
            Some('(') => {
                self.advance();
                Ok(Token::LParen)
            }
            Some(')') => {
                self.advance();
                Ok(Token::RParen)
            }
            Some('+') => {
                self.advance();
                Ok(Token::Plus)
            }
            Some('-') => {
                self.advance();
                // Check if it's a negative number
                if let Some(c) = self.peek() {
                    if c.is_ascii_digit() {
                        let num = self.read_number()?;
                        return Ok(Token::Number(-num));
                    }
                }
                Ok(Token::Minus)
            }
            Some('.') => {
                self.advance();
                let name = self.read_identifier();
                Ok(Token::Directive(name.to_lowercase()))
            }
            Some('"') => {
                self.advance();
                let s = self.read_string()?;
                Ok(Token::StringLit(s))
            }
            Some('\'') => {
                self.advance();
                let c = self.advance().ok_or("Unexpected end of character literal")?;
                if self.advance() != Some('\'') {
                    return Err(format!("Expected closing quote at line {}", self.line));
                }
                Ok(Token::Number(c as i32))
            }
            Some(c) if c.is_ascii_digit() => {
                let num = self.read_number()?;
                Ok(Token::Number(num))
            }
            Some(c) if c.is_alphabetic() || c == '_' => {
                let ident = self.read_identifier();
                // Check if it's a register
                if let Some(reg) = parse_register(&ident) {
                    Ok(Token::Register(reg))
                } else {
                    Ok(Token::Ident(ident))
                }
            }
            Some(c) => Err(format!("Unexpected character '{}' at line {}", c, self.line)),
        }
    }

    fn peek(&mut self) -> Option<char> {
        self.input.peek().copied()
    }

    fn advance(&mut self) -> Option<char> {
        let c = self.input.next();
        if c.is_some() {
            self.col += 1;
        }
        c
    }

    fn skip_whitespace(&mut self) {
        while let Some(c) = self.peek() {
            if c == ' ' || c == '\t' {
                self.advance();
            } else {
                break;
            }
        }
    }

    fn skip_comment(&mut self) {
        if self.peek() == Some(';') {
            while let Some(c) = self.peek() {
                if c == '\n' || c == '\r' {
                    break;
                }
                self.advance();
            }
        }
    }

    fn read_identifier(&mut self) -> String {
        let mut ident = String::new();
        while let Some(c) = self.peek() {
            if c.is_alphanumeric() || c == '_' {
                ident.push(c);
                self.advance();
            } else {
                break;
            }
        }
        ident
    }

    fn read_number(&mut self) -> Result<i32, String> {
        let mut num_str = String::new();
        let mut base = 10;

        // Check for hex prefix
        if self.peek() == Some('0') {
            num_str.push(self.advance().unwrap());
            if let Some(c) = self.peek() {
                if c == 'x' || c == 'X' {
                    self.advance();
                    base = 16;
                    num_str.clear();
                } else if c == 'b' || c == 'B' {
                    self.advance();
                    base = 2;
                    num_str.clear();
                }
            }
        }

        while let Some(c) = self.peek() {
            if c.is_ascii_hexdigit() || c == '_' {
                if c != '_' {
                    num_str.push(c);
                }
                self.advance();
            } else {
                break;
            }
        }

        i32::from_str_radix(&num_str, base)
            .map_err(|e| format!("Invalid number at line {}: {}", self.line, e))
    }

    fn read_string(&mut self) -> Result<String, String> {
        let mut s = String::new();
        loop {
            match self.advance() {
                None => return Err(format!("Unterminated string at line {}", self.line)),
                Some('"') => break,
                Some('\\') => {
                    match self.advance() {
                        Some('n') => s.push('\n'),
                        Some('r') => s.push('\r'),
                        Some('t') => s.push('\t'),
                        Some('0') => s.push('\0'),
                        Some('\\') => s.push('\\'),
                        Some('"') => s.push('"'),
                        Some(c) => s.push(c),
                        None => return Err(format!("Unterminated escape at line {}", self.line)),
                    }
                }
                Some(c) => s.push(c),
            }
        }
        Ok(s)
    }
}

fn parse_register(name: &str) -> Option<u8> {
    let upper = name.to_uppercase();
    match upper.as_str() {
        "R0" | "ZERO" => Some(0),
        "R1" | "RA" => Some(1),
        "R2" | "SP" => Some(2),
        "R3" | "GP" => Some(3),
        "R4" | "A0" => Some(4),
        "R5" | "A1" => Some(5),
        "R6" | "A2" => Some(6),
        "R7" | "A3" => Some(7),
        "R8" | "T0" => Some(8),
        "R9" | "T1" => Some(9),
        "R10" | "T2" => Some(10),
        "R11" | "T3" => Some(11),
        "R12" | "S0" => Some(12),
        "R13" | "S1" => Some(13),
        "R14" | "S2" => Some(14),
        "R15" | "S3" => Some(15),
        _ => None,
    }
}
