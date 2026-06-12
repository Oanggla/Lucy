mod ast;
mod lexer;
mod parser;
mod codegen;
mod vm;

use std::env;
use std::fs;
use std::process;

use lexer::Lexer;
use parser::Parser;
use codegen::Codegen;
use vm::VM;

fn main() {
    let args: Vec<String> = env::args().collect();
    if args.len() < 3 {
        print_usage();
        process::exit(1);
    }

    let command = &args[1];
    match command.as_str() {
        "run" => {
            let filename = &args[2];
            if let Err(e) = run_file(filename) {
                eprintln!("Error: {}", e);
                process::exit(1);
            }
        }
        "compile" => {
            if args.len() < 4 {
                eprintln!("Error: Missing output file path.");
                print_usage();
                process::exit(1);
            }
            let src_file = &args[2];
            let out_file = &args[3];
            if let Err(e) = compile_file(src_file, out_file) {
                eprintln!("Error: {}", e);
                process::exit(1);
            }
        }
        "exec" => {
            let compiled_file = &args[2];
            if let Err(e) = exec_file(compiled_file) {
                eprintln!("Error: {}", e);
                process::exit(1);
            }
        }
        _ => {
            eprintln!("Error: Unknown command '{}'", command);
            print_usage();
            process::exit(1);
        }
    }
}

fn print_usage() {
    println!("Usage:");
    println!("  cargo run -- run <file.lucy>                   Compile and run Lucy file in-memory");
    println!("  cargo run -- compile <file.lucy> <out.lucyc>   Compile Lucy file into JSON bytecode");
    println!("  cargo run -- exec <file.lucyc>                 Execute compiled JSON bytecode");
}

fn run_file(filename: &str) -> Result<(), String> {
    let source = fs::read_to_string(filename)
        .map_err(|e| format!("Failed to read file '{}': {}", filename, e))?;

    let mut lexer = Lexer::new(&source);
    let tokens = lexer.tokenize()?;

    let mut parser = Parser::new(tokens);
    let program = parser.parse_program()?;

    let codegen = Codegen::new();
    let compiled = codegen.compile_program(&program)?;

    let mut vm = VM::new();
    let _result = vm.run(compiled)?;

    Ok(())
}

fn compile_file(src_file: &str, out_file: &str) -> Result<(), String> {
    let source = fs::read_to_string(src_file)
        .map_err(|e| format!("Failed to read file '{}': {}", src_file, e))?;

    let mut lexer = Lexer::new(&source);
    let tokens = lexer.tokenize()?;

    let mut parser = Parser::new(tokens);
    let program = parser.parse_program()?;

    let codegen = Codegen::new();
    let compiled = codegen.compile_program(&program)?;

    let json_bytes = serde_json::to_vec_pretty(&compiled)
        .map_err(|e| format!("Failed to serialize bytecode: {}", e))?;

    fs::write(out_file, json_bytes)
        .map_err(|e| format!("Failed to write output file '{}': {}", out_file, e))?;

    println!("Successfully compiled '{}' to '{}'", src_file, out_file);
    Ok(())
}

fn exec_file(compiled_file: &str) -> Result<(), String> {
    let data = fs::read_to_string(compiled_file)
        .map_err(|e| format!("Failed to read compiled file '{}': {}", compiled_file, e))?;

    let compiled = serde_json::from_str(&data)
        .map_err(|e| format!("Failed to parse JSON bytecode: {}", e))?;

    let mut vm = VM::new();
    let _result = vm.run(compiled)?;

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::lexer::TokenKind;
    use crate::vm::Value;

    #[test]
    fn test_lexer() {
        let mut lexer = Lexer::new("let x = 5 + 10;");
        let tokens = lexer.tokenize().unwrap();
        assert_eq!(tokens[0].kind, TokenKind::Let);
        assert_eq!(tokens[1].kind, TokenKind::Identifier("x".to_string()));
        assert_eq!(tokens[2].kind, TokenKind::Assign);
        assert_eq!(tokens[3].kind, TokenKind::Number(5.0));
        assert_eq!(tokens[4].kind, TokenKind::Plus);
        assert_eq!(tokens[5].kind, TokenKind::Number(10.0));
        assert_eq!(tokens[6].kind, TokenKind::Semicolon);
    }

    #[test]
    fn test_parser_and_vm() {
        let source = "let a = 10; let b = 20; let c = a * b;";
        let mut lexer = Lexer::new(source);
        let tokens = lexer.tokenize().unwrap();
        let mut parser = Parser::new(tokens);
        let program = parser.parse_program().unwrap();
        let codegen = Codegen::new();
        let compiled = codegen.compile_program(&program).unwrap();
        let mut vm = VM::new();
        let _result = vm.run(compiled).unwrap();
        assert_eq!(vm.globals.get("c"), Some(&Value::Number(200.0)));
    }

    #[test]
    fn test_pipeline_operator() {
        let source = "fn add_one(x) { x + 1 } let val = 5 |> add_one();";
        let mut lexer = Lexer::new(source);
        let tokens = lexer.tokenize().unwrap();
        let mut parser = Parser::new(tokens);
        let program = parser.parse_program().unwrap();
        let codegen = Codegen::new();
        let compiled = codegen.compile_program(&program).unwrap();
        let mut vm = VM::new();
        let _result = vm.run(compiled).unwrap();
        assert_eq!(vm.globals.get("val"), Some(&Value::Number(6.0)));
    }
}

