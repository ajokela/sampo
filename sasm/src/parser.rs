//! Parser for Sampo assembly language

use crate::lexer::Token;

#[derive(Debug, Clone)]
#[allow(dead_code)]
pub enum Operand {
    Register(u8),
    Immediate(i32),
    Label(String),
    Indirect(u8, i32),  // Register + offset: (Rs + imm)
}

#[derive(Debug, Clone)]
pub enum Statement {
    Label(String),
    Instruction {
        mnemonic: String,
        operands: Vec<Operand>,
    },
    Directive {
        name: String,
        args: Vec<DirectiveArg>,
    },
}

#[derive(Debug, Clone)]
pub enum DirectiveArg {
    Number(i32),
    String(String),
    Ident(String),
}

pub struct Program {
    pub statements: Vec<Statement>,
}

pub struct Parser {
    tokens: Vec<Token>,
    pos: usize,
}

impl Parser {
    pub fn new(tokens: Vec<Token>) -> Self {
        Parser { tokens, pos: 0 }
    }

    pub fn parse(&mut self) -> Result<Program, String> {
        let mut statements = Vec::new();

        while !self.is_at_end() {
            self.skip_newlines();
            if self.is_at_end() {
                break;
            }

            if let Some(stmt) = self.parse_statement()? {
                statements.push(stmt);
            }
        }

        Ok(Program { statements })
    }

    fn parse_statement(&mut self) -> Result<Option<Statement>, String> {
        match self.peek() {
            Token::Eof => Ok(None),
            Token::Newline => {
                self.advance();
                Ok(None)
            }
            Token::Directive(name) => {
                let name = name.clone();
                self.advance();
                let args = self.parse_directive_args()?;
                Ok(Some(Statement::Directive { name, args }))
            }
            Token::Ident(name) => {
                let name = name.clone();
                self.advance();

                // Check if it's a label
                if self.check(&Token::Colon) {
                    self.advance();
                    Ok(Some(Statement::Label(name)))
                } else {
                    // It's an instruction
                    let operands = self.parse_operands()?;
                    Ok(Some(Statement::Instruction {
                        mnemonic: name.to_uppercase(),
                        operands,
                    }))
                }
            }
            _ => Err(format!("Unexpected token: {:?}", self.peek())),
        }
    }

    fn parse_directive_args(&mut self) -> Result<Vec<DirectiveArg>, String> {
        let mut args = Vec::new();

        loop {
            match self.peek() {
                Token::Newline | Token::Eof => break,
                Token::Number(n) => {
                    let n = *n;
                    self.advance();
                    args.push(DirectiveArg::Number(n));
                }
                Token::StringLit(s) => {
                    let s = s.clone();
                    self.advance();
                    args.push(DirectiveArg::String(s));
                }
                Token::Ident(s) => {
                    let s = s.clone();
                    self.advance();
                    args.push(DirectiveArg::Ident(s));
                }
                Token::Comma => {
                    self.advance();
                }
                _ => break,
            }
        }

        Ok(args)
    }

    fn parse_operands(&mut self) -> Result<Vec<Operand>, String> {
        let mut operands = Vec::new();

        loop {
            match self.peek() {
                Token::Newline | Token::Eof => break,
                Token::Comma => {
                    self.advance();
                    continue;
                }
                Token::Register(r) => {
                    let r = *r;
                    self.advance();
                    operands.push(Operand::Register(r));
                }
                Token::Number(n) => {
                    let n = *n;
                    self.advance();

                    // Check for indirect addressing: imm(Rs)
                    if self.check(&Token::LParen) {
                        self.advance();
                        if let Token::Register(r) = self.peek() {
                            let r = *r;
                            self.advance();
                            self.expect(&Token::RParen)?;
                            operands.push(Operand::Indirect(r, n));
                        } else {
                            return Err("Expected register in indirect addressing".to_string());
                        }
                    } else {
                        operands.push(Operand::Immediate(n));
                    }
                }
                Token::Ident(name) => {
                    let name = name.clone();
                    self.advance();
                    operands.push(Operand::Label(name));
                }
                Token::LParen => {
                    // Indirect addressing: (Rs) or (Rs + imm)
                    self.advance();
                    if let Token::Register(r) = self.peek() {
                        let r = *r;
                        self.advance();

                        let offset = if self.check(&Token::Plus) {
                            self.advance();
                            if let Token::Number(n) = self.peek() {
                                let n = *n;
                                self.advance();
                                n
                            } else {
                                0
                            }
                        } else if self.check(&Token::Minus) {
                            self.advance();
                            if let Token::Number(n) = self.peek() {
                                let n = *n;
                                self.advance();
                                -n
                            } else {
                                0
                            }
                        } else {
                            0
                        };

                        self.expect(&Token::RParen)?;
                        operands.push(Operand::Indirect(r, offset));
                    } else {
                        return Err("Expected register in indirect addressing".to_string());
                    }
                }
                Token::Minus => {
                    self.advance();
                    if let Token::Number(n) = self.peek() {
                        let n = *n;
                        self.advance();
                        operands.push(Operand::Immediate(-n));
                    } else {
                        return Err("Expected number after minus".to_string());
                    }
                }
                _ => break,
            }
        }

        Ok(operands)
    }

    fn peek(&self) -> &Token {
        self.tokens.get(self.pos).unwrap_or(&Token::Eof)
    }

    fn advance(&mut self) -> &Token {
        if !self.is_at_end() {
            self.pos += 1;
        }
        self.tokens.get(self.pos - 1).unwrap_or(&Token::Eof)
    }

    fn is_at_end(&self) -> bool {
        matches!(self.peek(), Token::Eof)
    }

    fn check(&self, token: &Token) -> bool {
        std::mem::discriminant(self.peek()) == std::mem::discriminant(token)
    }

    fn expect(&mut self, expected: &Token) -> Result<(), String> {
        if self.check(expected) {
            self.advance();
            Ok(())
        } else {
            Err(format!("Expected {:?}, got {:?}", expected, self.peek()))
        }
    }

    fn skip_newlines(&mut self) {
        while matches!(self.peek(), Token::Newline) {
            self.advance();
        }
    }
}
