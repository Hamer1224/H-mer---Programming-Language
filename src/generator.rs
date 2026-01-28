use std::collections::HashMap;
use crate::lexer::Token;
use crate::parser::Stmt;

pub struct Generator {
    pub output: String,
    symbols: HashMap<String, String>,
    class_map: HashMap<String, Vec<String>>,
    obj_types: HashMap<String, String>,
    reg_count: usize,
    label_count: usize,
}

impl Generator {
    pub fn new() -> Self {
        Self {
            output: ".global _start\n.section .text\n\n_start:\n    mov x11, #10\n    mov x0, #0\n    mov x1, #4096\n    mov x2, #3\n    mov x3, #34\n    mov x4, #-1\n    mov x5, #0\n    mov x8, #222\n    svc #0\n    mov x20, x0\n".to_string(),
            symbols: HashMap::new(),
            class_map: HashMap::new(),
            obj_types: HashMap::new(),
            reg_count: 12,
            label_count: 0,
        }
    }

    fn get_path_info(&self, path: &Vec<String>) -> (String, usize) {
        if path.is_empty() { return ("x0".to_string(), 0); }
        let base_var = &path[0];
        let reg = self.symbols.get(base_var).cloned().unwrap_or_else(|| "x0".to_string());
        let mut offset = 0;
        if path.len() > 1 {
            if let Some(c_type) = self.obj_types.get(base_var) {
                if let Some(fields) = self.class_map.get(c_type) {
                    offset = fields.iter().position(|f| f == &path[1]).unwrap_or(0) * 8;
                }
            }
        }
        (reg, offset)
    }

    pub fn generate(&mut self, ast: Vec<Stmt>) -> String {
        for stmt in ast { self.gen_stmt(stmt); }
        self.output.push_str("\n    mov x0, #0\n    mov x8, #93\n    svc #0\n");
        self.output.clone()
    }

    fn gen_stmt(&mut self, stmt: Stmt) {
        match stmt {
            Stmt::ProbIf { chance, body } => {
                let id = self.label_count; self.label_count += 1;
                let math_reg = self.symbols.get("math").cloned().unwrap_or("x12".into());
                self.output.push_str(&format!("\n    // Prob Roll {}%\n", chance));
                self.output.push_str(&format!("    ldr x1, [{}, #8]\n", math_reg));
                
                // --- Emergency Auto-Seeding ---
                // If seed is 0, read Hardware Cycle Counter immediately
                self.output.push_str(&format!("    cmp x1, #0\n    b.ne .Lseedok{}\n", id));
                self.output.push_str("    mrs x1, cntvct_el0\n");
                self.output.push_str(&format!(".Lseedok{}:\n", id));

                // --- Improved Mixer ---
                self.output.push_str("    ldr x2, =0x9E3779B97F4A7C15\n");
                self.output.push_str("    mul x1, x1, x2\n");
                self.output.push_str("    eor x1, x1, x1, lsr #33\n");
                self.output.push_str(&format!("    str x1, [{}, #8]\n", math_reg));

                // Modulo 100
                self.output.push_str("    and x1, x1, #0x7FFFFFFF\n"); // Absolute value
                self.output.push_str("    mov x2, #100\n");
                self.output.push_str("    udiv x3, x1, x2\n");
                self.output.push_str("    msub x1, x3, x2, x1\n");
                
                self.output.push_str(&format!("    cmp x1, #{}\n", chance as i64));
                self.output.push_str(&format!("    b.hs .Lif{}\n", id));
                for s in body { self.gen_stmt(s); }
                self.output.push_str(&format!(".Lif{}:\n", id));
            }
            Stmt::LocalAssign { name, value } => {
                let reg = self.symbols.entry(name.clone()).or_insert_with(|| { let r = format!("x{}", self.reg_count); self.reg_count += 1; r }).clone();
                self.output.push_str(&format!("    mov {}, #{}\n", reg, value as i64));
            }
            Stmt::WhileStmt { path, op, rhs_val, body } => {
                let id = self.label_count; self.label_count += 1;
                self.output.push_str(&format!(".Lloop{}:\n", id));
                let (reg, off) = self.get_path_info(&path);
                if path.len() > 1 { self.output.push_str(&format!("    ldr x1, [{}, #{}]\n", reg, off)); } 
                else { self.output.push_str(&format!("    mov x1, {}\n", reg)); }
                self.output.push_str(&format!("    cmp x1, #{}\n", rhs_val as i64));
                let br = match op { Token::Greater => "b.le", Token::Less => "b.ge", _ => "b.ne" };
                self.output.push_str(&format!("    {} .Lexit{}\n", br, id));
                for s in body { self.gen_stmt(s); }
                self.output.push_str(&format!("    b .Lloop{}\n.Lexit{}:\n", id, id));
            }
            Stmt::FieldMath { path, op, rhs_val } => {
                let (reg, off) = self.get_path_info(&path);
                if path.len() > 1 { self.output.push_str(&format!("    ldr x1, [{}, #{}]\n", reg, off)); } 
                else { self.output.push_str(&format!("    mov x1, {}\n", reg)); }
                if matches!(op, Token::Minus) { self.output.push_str(&format!("    sub x1, x1, #{}\n", rhs_val as i64)); } 
                else { self.output.push_str(&format!("    add x1, x1, #{}\n", rhs_val as i64)); }
                if path.len() > 1 { self.output.push_str(&format!("    str x1, [{}, #{}]\n", reg, off)); } 
                else { self.output.push_str(&format!("    mov {}, x1\n", reg)); }
            }
            Stmt::PrintVar(name) => {
                let reg_opt = self.symbols.get(&name).cloned();
                if let Some(reg) = reg_opt { self.write_print_loop(&reg); }
            }
            Stmt::PrintField { path } => {
                let (reg, off) = self.get_path_info(&path);
                self.output.push_str(&format!("    ldr x2, [{}, #{}]\n", reg, off));
                self.write_print_loop("x2");
            }
            Stmt::HeapAlloc { var_name, class_name } => {
                let reg = format!("x{}", self.reg_count); self.reg_count += 1;
                self.symbols.insert(var_name.clone(), reg.clone());
                self.obj_types.insert(var_name, class_name.clone());
                if let Some(f) = self.class_map.get(&class_name) { self.output.push_str(&format!("    mov {}, x20\n    add x20, x20, #{}\n", reg, f.len() * 8)); }
            }
            Stmt::ClassDef { name, fields } => { self.class_map.insert(name, fields); }
            Stmt::PrintString(s) => {
                let l = format!("str_{}", self.label_count); self.label_count += 1;
                self.output.insert_str(0, &format!(".section .data\n{}: .ascii \"{}\\n\"\n.section .text\n", l, s));
                self.output.push_str(&format!("    mov x0, #1\n    ldr x1, ={}\n    mov x2, #{}\n    mov x8, #64\n    svc #0\n", l, s.len()+1));
            }
            Stmt::Rest(s) => {
                self.output.push_str(&format!("    mov x0, #{}\n    mov x1, #0\n    stp x0, x1, [sp, #-16]!\n    mov x0, sp\n    mov x1, #0\n    mov x8, #115\n    svc #0\n    add sp, sp, #16\n", s as i64));
            }
            Stmt::FieldAssign { path, value } => {
                let (reg, off) = self.get_path_info(&path);
                self.output.push_str(&format!("    mov x1, #{}\n", value as i64));
                if path.len() > 1 { self.output.push_str(&format!("    str x1, [{}, #{}]\n", reg, off)); } 
                else { self.output.push_str(&format!("    mov {}, x1\n", reg)); }
            }
            Stmt::AsmBlock(code) => { self.output.push_str(&format!("    {}\n", code)); }
            Stmt::IfStmt { path, op, rhs_val, body } => {
                let id = self.label_count; self.label_count += 1;
                let (reg, off) = self.get_path_info(&path);
                if path.len() > 1 { self.output.push_str(&format!("    ldr x1, [{}, #{}]\n", reg, off)); } 
                else { self.output.push_str(&format!("    mov x1, {}\n", reg)); }
                self.output.push_str(&format!("    mov x2, #{}\n    cmp x1, x2\n", rhs_val as i64));
                let br = match op { Token::Greater => "b.le", Token::Less => "b.ge", _ => "b.ne" };
                self.output.push_str(&format!("    {} .Lif{}\n", br, id));
                for s in body { self.gen_stmt(s); }
                self.output.push_str(&format!(".Lif{}:\n", id));
            }
        }
    }

    fn write_print_loop(&mut self, reg: &str) {
        let id = self.output.len();
        self.output.push_str(&format!("
    stp x0, x1, [sp, #-16]!
    mov x0, {}
    sub sp, sp, #32
    mov x1, sp
    add x1, x1, #31
    mov w2, #10
    strb w2, [x1]
.Lp{}:
    sub x1, x1, #1
    udiv x2, x0, x11
    msub x3, x2, x11, x0
    add x3, x3, #48
    strb w3, [x1]
    mov x0, x2
    cbnz x0, .Lp{}
    mov x0, #1
    mov x2, sp
    add x2, x2, #32
    sub x2, x2, x1
    mov x8, #64
    svc #0
    add sp, sp, #32
    ldp x0, x1, [sp], #16\n", reg, id, id));
    }
} 
