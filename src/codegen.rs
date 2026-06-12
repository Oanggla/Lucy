use serde::{Deserialize, Serialize};
use std::collections::HashMap;

use crate::ast::{Expression, Statement};

#[derive(Serialize, Deserialize, Debug, Clone, PartialEq)]
pub enum Instruction {
    LoadConst(usize),
    LoadVar(String),
    StoreVar(String),
    Pop,
    Add,
    Sub,
    Mul,
    Div,
    Mod,
    Eq,
    Ne,
    Lt,
    Gt,
    Le,
    Ge,
    Jump(usize),
    JumpIfFalse(usize),
    Call(String, usize),
    Return,
    Print,
}

#[derive(Serialize, Deserialize, Debug, Clone, PartialEq)]
pub enum Constant {
    Number(f64),
    String(String),
    Boolean(bool),
    Nil,
}

#[derive(Serialize, Deserialize, Debug, Clone, PartialEq)]
pub struct FunctionData {
    pub name: String,
    pub params: Vec<String>,
    pub instructions: Vec<Instruction>,
}

#[derive(Serialize, Deserialize, Debug, Clone, PartialEq)]
pub struct CompiledProgram {
    pub constants: Vec<Constant>,
    pub functions: Vec<FunctionData>,
    pub instructions: Vec<Instruction>,
}

pub struct Codegen {
    constants: Vec<Constant>,
    constant_map: HashMap<String, usize>, // String representation of constant to index
    functions: Vec<FunctionData>,
    instructions: Vec<Instruction>,
}

impl Codegen {
    pub fn new() -> Self {
        Self {
            constants: Vec::new(),
            constant_map: HashMap::new(),
            functions: Vec::new(),
            instructions: Vec::new(),
        }
    }

    fn add_constant(&mut self, constant: Constant) -> usize {
        let key = match &constant {
            Constant::Number(n) => format!("N_{}", n),
            Constant::String(s) => format!("S_{}", s),
            Constant::Boolean(b) => format!("B_{}", b),
            Constant::Nil => "NIL".to_string(),
        };

        if let Some(&idx) = self.constant_map.get(&key) {
            idx
        } else {
            let idx = self.constants.len();
            self.constants.push(constant);
            self.constant_map.insert(key, idx);
            idx
        }
    }

    pub fn compile_program(mut self, statements: &[Statement]) -> Result<CompiledProgram, String> {
        for stmt in statements {
            self.compile_statement(stmt)?;
        }
        Ok(CompiledProgram {
            constants: self.constants,
            functions: self.functions,
            instructions: self.instructions,
        })
    }

    fn compile_statement(&mut self, stmt: &Statement) -> Result<(), String> {
        match stmt {
            Statement::Let(name, expr) => {
                self.compile_expression(expr)?;
                self.instructions.push(Instruction::StoreVar(name.clone()));
            }
            Statement::FnDecl(name, params, body) => {
                // Compile function in a separate codegen context
                let mut fn_codegen = Codegen::new();
                
                // Keep the constants from the main codegen synchronized
                fn_codegen.constants = self.constants.clone();
                fn_codegen.constant_map = self.constant_map.clone();

                fn_codegen.compile_expression(body)?;
                fn_codegen.instructions.push(Instruction::Return);

                // Sync the constants back
                self.constants = fn_codegen.constants;
                self.constant_map = fn_codegen.constant_map;

                self.functions.push(FunctionData {
                    name: name.clone(),
                    params: params.clone(),
                    instructions: fn_codegen.instructions,
                });
            }
            Statement::While(cond, body) => {
                let start_pc = self.instructions.len();
                
                self.compile_expression(cond)?;
                
                let jump_false_idx = self.instructions.len();
                self.instructions.push(Instruction::JumpIfFalse(0)); // Placeholder
                
                self.compile_expression(body)?;
                // Pop the block value since while loop statement doesn't use it
                self.instructions.push(Instruction::Pop);
                
                self.instructions.push(Instruction::Jump(start_pc));
                
                let end_pc = self.instructions.len();
                self.instructions[jump_false_idx] = Instruction::JumpIfFalse(end_pc);
            }
            Statement::Expr(expr) => {
                self.compile_expression(expr)?;
                self.instructions.push(Instruction::Pop); // Pop unused expression statement value
            }
        }
        Ok(())
    }

    fn compile_expression(&mut self, expr: &Expression) -> Result<(), String> {
        match expr {
            Expression::Number(val) => {
                let idx = self.add_constant(Constant::Number(*val));
                self.instructions.push(Instruction::LoadConst(idx));
            }
            Expression::String(val) => {
                let idx = self.add_constant(Constant::String(val.clone()));
                self.instructions.push(Instruction::LoadConst(idx));
            }
            Expression::Boolean(val) => {
                let idx = self.add_constant(Constant::Boolean(*val));
                self.instructions.push(Instruction::LoadConst(idx));
            }
            Expression::Identifier(name) => {
                self.instructions.push(Instruction::LoadVar(name.clone()));
            }
            Expression::BinaryOp(op, left, right) => {
                self.compile_expression(left)?;
                self.compile_expression(right)?;
                let inst = match op.as_str() {
                    "+" => Instruction::Add,
                    "-" => Instruction::Sub,
                    "*" => Instruction::Mul,
                    "/" => Instruction::Div,
                    "%" => Instruction::Mod,
                    "==" => Instruction::Eq,
                    "!=" => Instruction::Ne,
                    "<" => Instruction::Lt,
                    ">" => Instruction::Gt,
                    "<=" => Instruction::Le,
                    ">=" => Instruction::Ge,
                    _ => return Err(format!("Unknown binary operator: {}", op)),
                };
                self.instructions.push(inst);
            }
            Expression::Block(statements, trailing_expr) => {
                for stmt in statements {
                    self.compile_statement(stmt)?;
                }
                if let Some(expr) = trailing_expr {
                    self.compile_expression(expr)?;
                } else {
                    let idx = self.add_constant(Constant::Nil);
                    self.instructions.push(Instruction::LoadConst(idx));
                }
            }
            Expression::Call(name, args) => {
                if name == "print" {
                    if args.len() != 1 {
                        return Err("print function expects exactly 1 argument".to_string());
                    }
                    self.compile_expression(&args[0])?;
                    self.instructions.push(Instruction::Print);
                    // print() expression evaluates to Nil
                    let idx = self.add_constant(Constant::Nil);
                    self.instructions.push(Instruction::LoadConst(idx));
                } else {
                    for arg in args {
                        self.compile_expression(arg)?;
                    }
                    self.instructions.push(Instruction::Call(name.clone(), args.len()));
                }
            }
            Expression::If(cond, then_branch, else_branch) => {
                self.compile_expression(cond)?;
                
                let jump_false_idx = self.instructions.len();
                self.instructions.push(Instruction::JumpIfFalse(0)); // Placeholder
                
                self.compile_expression(then_branch)?;
                
                let jump_end_idx = self.instructions.len();
                self.instructions.push(Instruction::Jump(0)); // Placeholder
                
                let else_pc = self.instructions.len();
                self.instructions[jump_false_idx] = Instruction::JumpIfFalse(else_pc);
                
                self.compile_expression(else_branch)?;
                
                let end_pc = self.instructions.len();
                self.instructions[jump_end_idx] = Instruction::Jump(end_pc);
            }
        }
        Ok(())
    }
}
