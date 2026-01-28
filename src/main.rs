mod lexer;
mod parser;
mod generator;

use std::env;
use std::fs;
use std::path::Path;
use lexer::Lexer;
use parser::Parser;
use generator::Generator;

/// This function ensures that any GET "lib" calls pull the library code
/// and place it at the absolute beginning of the source stream.
fn resolve_imports(content: String, base_path: &Path) -> String {
    let mut imports_code = String::new();
    let mut main_body_code = String::new();
    let dir = base_path.parent().unwrap_or(Path::new("."));

    for line in content.lines() {
        let trimmed = line.trim();
        // Check for GET command (case insensitive)
        if trimmed.to_uppercase().starts_with("GET") {
            let parts: Vec<&str> = trimmed.split_whitespace().collect();
            if parts.len() >= 2 {
                let lib_name = parts[1].replace("\"", "");
                let lib_path = dir.join(format!("{}.hmr", lib_name));

                match fs::read_to_string(&lib_path) {
                    Ok(lib_content) => {
                        println!("DEBUG: Successfully injected {} into source head.", lib_name);
                        imports_code.push_str(&lib_content);
                        imports_code.push('\n');
                    }
                    Err(_) => {
                        eprintln!("COMPILER ERROR: Could not find library file at {:?}", lib_path);
                        std::process::exit(1);
                    }
                }
            }
        } else {
            main_body_code.push_str(line);
            main_body_code.push('\n');
        }
    }

    // Single-Pass rule: Definitions (Imports) MUST come before usage (Main Body)
    format!("{}\n{}", imports_code, main_body_code)
}

fn main() {
    let args: Vec<String> = env::args().collect();
    if args.len() < 2 {
        println!("Usage: hamer <file.hmr>");
        return;
    }

    let file_path = Path::new(&args[1]);
    
    // Read the file once
    let raw_content = fs::read_to_string(file_path).expect("Unable to read source file");
    
    // 1. Resolve imports (ONLY ONCE)
    // We pass raw_content here. It is moved into the function.
    let full_content = resolve_imports(raw_content, file_path);

    // 2. Lexical Analysis
    let mut lexer = Lexer::new(full_content);
    let mut tokens = Vec::new();
    loop {
        let t = lexer.next_token();
        tokens.push(t.clone());
        if t == lexer::Token::EOF { break; }
    }

    // 3. Parsing
    let mut parser = Parser::new(tokens);
    let ast = parser.parse_program();

    // 4. Code Generation
    let mut compiler = Generator::new();
    let assembly = compiler.generate(ast);
    
    // 5. Output
    fs::write("out.s", assembly).expect("Failed to write assembly file");
    println!("H@mer: Compilation Successful.");
}

