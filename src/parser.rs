use crate::lexer::Token;

#[derive(Debug)]
pub enum Stmt {
    LocalAssign { name: String, value: f64 },
    ClassDef { name: String, fields: Vec<String> },
    HeapAlloc { var_name: String, class_name: String },
    FieldAssign { path: Vec<String>, value: f64 },
    FieldMath { path: Vec<String>, op: Token, rhs_val: f64 },
    PrintField { path: Vec<String> },
    PrintVar(String),
    PrintString(String),
    IfStmt { path: Vec<String>, op: Token, rhs_val: f64, body: Vec<Stmt> },
    ProbIf { chance: f64, body: Vec<Stmt> },
    WhileStmt { path: Vec<String>, op: Token, rhs_val: f64, body: Vec<Stmt> },
    Rest(f64),
    AsmBlock(String),
}

pub struct Parser { pub tokens: Vec<Token>, pub pos: usize }

impl Parser {
    pub fn new(tokens: Vec<Token>) -> Self { Self { tokens, pos: 0 } }
    fn advance(&mut self) -> Token {
        let t = self.tokens[self.pos].clone();
        if self.pos < self.tokens.len() - 1 { self.pos += 1; }
        t
    }
    fn peek(&self) -> Token { self.tokens[self.pos].clone() }

    pub fn parse_program(&mut self) -> Vec<Stmt> {
        let mut stmts = Vec::new();
        while self.peek() != Token::EOF { stmts.push(self.parse_statement()); }
        stmts
    }

    fn parse_path(&mut self) -> Vec<String> {
        let mut path = Vec::new();
        if let Token::Identifier(s) = self.peek() {
            self.advance(); path.push(s);
            while self.peek() == Token::Dot {
                self.advance();
                if let Token::Identifier(s) = self.advance() { path.push(s); }
            }
        }
        path
    }

    fn parse_statement(&mut self) -> Stmt {
        match self.peek() {
            Token::Get => { self.advance(); self.advance(); Stmt::AsmBlock("nop".into()) }
            Token::Class => {
                self.advance();
                let name = if let Token::Identifier(s) = self.advance() { s } else { "".into() };
                if self.peek() == Token::Is { self.advance(); }
                let mut fields = Vec::new();
                while !matches!(self.peek(), Token::Done | Token::EOF) {
                    if let Token::Identifier(s) = self.advance() { fields.push(s); }
                }
                if self.peek() == Token::Done { self.advance(); }
                Stmt::ClassDef { name, fields }
            }
            Token::Local => {
                self.advance();
                let name = if let Token::Identifier(s) = self.advance() { s } else { "".into() };
                if self.peek() == Token::Assign { self.advance(); }
                if self.peek() == Token::New {
                    self.advance();
                    let cn = if let Token::Identifier(s) = self.advance() { s } else { "".into() };
                    Stmt::HeapAlloc { var_name: name, class_name: cn }
                } else {
                    let val = if let Token::Number(n) = self.advance() { n } else { 0.0 };
                    Stmt::LocalAssign { name, value: val }
                }
            }
            Token::If => {
                self.advance();
                if self.peek() == Token::Quest {
                    self.advance(); // ?
                    while matches!(self.peek(), Token::Less | Token::Percent) { self.advance(); }
                    let chance = if let Token::Number(n) = self.advance() { n } else { 0.0 };
                    while matches!(self.peek(), Token::Greater | Token::Is | Token::Then) { self.advance(); }
                    let mut body = Vec::new();
                    while !matches!(self.peek(), Token::Done | Token::EOF) { body.push(self.parse_statement()); }
                    if self.peek() == Token::Done { self.advance(); }
                    Stmt::ProbIf { chance, body }
                } else {
                    let p = self.parse_path();
                    let op = self.advance();
                    let val = if let Token::Number(n) = self.advance() { n } else { 0.0 };
                    while matches!(self.peek(), Token::Then | Token::Is) { self.advance(); }
                    let mut body = Vec::new();
                    while !matches!(self.peek(), Token::Done | Token::EOF) { body.push(self.parse_statement()); }
                    if self.peek() == Token::Done { self.advance(); }
                    Stmt::IfStmt { path: p, op, rhs_val: val, body }
                }
            }
            Token::While => {
                self.advance();
                let p = self.parse_path();
                let op = self.advance();
                let val = if let Token::Number(n) = self.advance() { n } else { 0.0 };
                while matches!(self.peek(), Token::Do | Token::Is) { self.advance(); }
                let mut body = Vec::new();
                while !matches!(self.peek(), Token::Done | Token::EOF) { body.push(self.parse_statement()); }
                if self.peek() == Token::Done { self.advance(); }
                Stmt::WhileStmt { path: p, op, rhs_val: val, body }
            }
            Token::Print => {
                self.advance();
                if let Token::StringLit(s) = self.peek() { self.advance(); Stmt::PrintString(s) }
                else {
                    let p = self.parse_path();
                    if p.len() > 1 { Stmt::PrintField { path: p } } else { Stmt::PrintVar(p[0].clone()) }
                }
            }
            Token::Rest => { self.advance(); let v = if let Token::Number(n) = self.advance() { n } else { 1.0 }; Stmt::Rest(v) }
            Token::At => {
                self.advance(); self.advance(); self.advance();
                let mut code = String::new();
                let m = ["MOV", "ADD", "SUB", "LDR", "STR", "SVC", "CMP", "AND", "STP", "LDP", "MUL", "UDIV", "MSUB", "EOR", "LSL", "LSR", "NEG", "MRS"];
                while !matches!(self.peek(), Token::Done | Token::EOF) {
                    match self.advance() {
                        Token::Identifier(s) => { if m.contains(&s.to_uppercase().as_str()) { code.push_str("\n    "); } code.push_str(&format!("{} ", s)); }
                        Token::Number(n) => code.push_str(&format!("#{} ", n)),
                        Token::Comma => code.push_str(", "),
                        Token::LeftBracket => code.push_str("[ "),
                        Token::RightBracket => code.push_str("] "),
                        _ => {}
                    }
                }
                if self.peek() == Token::Done { self.advance(); }
                Stmt::AsmBlock(code)
            }
            _ => {
                let p = self.parse_path();
                if self.peek() == Token::Assign {
                    self.advance();
                    if let Token::Number(v) = self.peek() { self.advance(); Stmt::FieldAssign { path: p, value: v } }
                    else {
                        let _rhs_name = self.advance(); // consume 'wins' in wins = wins + 1
                        let op = self.advance();
                        let val = if let Token::Number(v) = self.advance() { v } else { 0.0 };
                        Stmt::FieldMath { path: p, op, rhs_val: val }
                    }
                } else { self.advance(); Stmt::AsmBlock("nop".into()) }
            }
        }
    }
}
