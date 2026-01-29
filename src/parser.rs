use crate::lexer::Token;
use std::fs;

#[derive(Debug)]
pub enum Stmt {
    LocalAssign { name: String, value: f64 },
    ClassDef { name: String, fields: Vec<String> },
    HeapAlloc { var_name: String, class_name: String },
    FieldAssign { path: Vec<String>, value: f64 },
    FieldMath { path: Vec<String>, op: Token, rhs_val: f64 },
    PrintVar(String),
    PrintString(String),
    IfStmt { path: Vec<String>, op: Token, rhs_val: f64, body: Vec<Stmt> },
    ProbIf { chance: f64, body: Vec<Stmt> },
    WhileStmt { path: Vec<String>, op: Token, rhs_val: f64, body: Vec<Stmt> },
    AsmBlock(String),      
    IntelBlock(String),    
    PythonBlock(String),   
    MergeBlock(String),    
}

pub struct Parser { pub tokens: Vec<Token>, pub pos: usize }

impl Parser {
    pub fn new(tokens: Vec<Token>) -> Self { Self { tokens, pos: 0 } }
    
    fn advance(&mut self) -> Token {
        let t = self.peek();
        if self.pos < self.tokens.len() {
            self.pos += 1;
        }
        t
    }

    fn peek(&self) -> Token {
        if self.pos < self.tokens.len() {
            self.tokens[self.pos].clone()
        } else {
            Token::EOF
        }
    }

    pub fn parse_program(&mut self) -> Vec<Stmt> {
        let mut stmts = Vec::new();
        while self.peek() != Token::EOF {
            stmts.push(self.parse_statement());
        }
        stmts
    }

    fn parse_path(&mut self) -> Vec<String> {
        let mut path = Vec::new();
        if let Token::Identifier(s) = self.peek() {
            self.advance(); 
            path.push(s);
            while self.peek() == Token::Dot {
                self.advance(); // consume dot
                if let Token::Identifier(s) = self.peek() {
                    self.advance();
                    path.push(s);
                } else { break; }
            }
        }
        path
    }

    fn parse_statement(&mut self) -> Stmt {
        match self.peek() {
            Token::Get => {
                self.advance();
                let filename = if let Token::Identifier(s) = self.advance() { s } else { "lib".into() };
                let path = format!("{}.hmr", filename);
                match fs::read_to_string(&path) {
                    Ok(content) => Stmt::MergeBlock(content),
                    Err(_) => Stmt::AsmBlock(format!("// Error: File not found {}.hmr", filename)),
                }
            }
            Token::At => {
                self.advance(); // @
                let type_ident = if let Token::Identifier(s) = self.advance() { s } else { "".into() };
                if self.peek() == Token::Is { self.advance(); }
                
                let mut content = String::new();
                while self.peek() != Token::Done && self.peek() != Token::EOF {
                    let t = self.advance();
                    match t {
                        Token::Identifier(id) => content.push_str(&format!("{} ", id)),
                        Token::Number(n) => content.push_str(&format!("{} ", n)),
                        Token::StringLit(s) => content.push_str(&format!("\"{}\" ", s)),
                        Token::Comma => content.push_str(", "),
                        Token::LeftBracket => content.push_str("[ "),
                        Token::RightBracket => content.push_str("] "),
                        _ => {}
                    }
                }
                if self.peek() == Token::Done { self.advance(); }

                match type_ident.as_str() {
                    "intel" => Stmt::IntelBlock(content),
                    "python" => Stmt::PythonBlock(content),
                    _ => Stmt::AsmBlock(content),
                }
            }
            Token::Local => {
                self.advance();
                let name = if let Token::Identifier(s) = self.advance() { s } else { "tmp".into() };
                if self.peek() == Token::Assign { self.advance(); }
                if self.peek() == Token::New {
                    self.advance();
                    let cn = if let Token::Identifier(s) = self.advance() { s } else { "Object".into() };
                    Stmt::HeapAlloc { var_name: name, class_name: cn }
                } else {
                    let val = if let Token::Number(n) = self.advance() { n } else { 0.0 };
                    Stmt::LocalAssign { name, value: val }
                }
            }
            Token::Class => {
                self.advance();
                let name = if let Token::Identifier(s) = self.advance() { s } else { "Unnamed".into() };
                if self.peek() == Token::Is { self.advance(); }
                let mut fields = Vec::new();
                while self.peek() != Token::Done && self.peek() != Token::EOF {
                    if let Token::Identifier(s) = self.advance() { fields.push(s); }
                    else { self.advance(); }
                }
                if self.peek() == Token::Done { self.advance(); }
                Stmt::ClassDef { name, fields }
            }
            Token::Print => {
                self.advance();
                match self.peek() {
                    Token::StringLit(s) => {
                        self.advance();
                        Stmt::PrintString(s)
                    },
                    _ => {
                        let path = self.parse_path();
                        let name = path.get(0).cloned().unwrap_or("".into());
                        Stmt::PrintVar(name)
                    }
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
                    while self.peek() != Token::Done && self.peek() != Token::EOF {
                        body.push(self.parse_statement());
                    }
                    if self.peek() == Token::Done { self.advance(); }
                    Stmt::ProbIf { chance, body }
                } else {
                    let p = self.parse_path(); 
                    let op = self.advance();
                    let val = if let Token::Number(n) = self.advance() { n } else { 0.0 };
                    while matches!(self.peek(), Token::Then | Token::Is) { self.advance(); }
                    let mut body = Vec::new();
                    while self.peek() != Token::Done && self.peek() != Token::EOF {
                        body.push(self.parse_statement());
                    }
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
                while self.peek() != Token::Done && self.peek() != Token::EOF {
                    body.push(self.parse_statement());
                }
                if self.peek() == Token::Done { self.advance(); }
                Stmt::WhileStmt { path: p, op, rhs_val: val, body }
            }
            _ => {
                let path = self.parse_path();
                if self.peek() == Token::Assign {
                    self.advance();
                    if let Token::Number(v) = self.peek() {
                        self.advance();
                        Stmt::FieldAssign { path, value: v }
                    } else {
                        // Handle math like 'hp = hp + 10' or compressed formats
                        self.advance(); // Skip self-ref identifier if exists
                        let op = self.advance();
                        let val = if let Token::Number(v) = self.advance() { v } else { 0.0 };
                        Stmt::FieldMath { path, op, rhs_val: val }
                    }
                } else {
                    self.advance(); // Safety: always consume at least one token
                    Stmt::AsmBlock("nop".into())
                }
            }
        }
    }
}