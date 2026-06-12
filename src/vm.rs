use std::collections::HashMap;
use crate::codegen::{Constant, CompiledProgram, FunctionData, Instruction};

#[derive(Debug, Clone, PartialEq)]
pub enum Value {
    Number(f64),
    String(String),
    Boolean(bool),
    Nil,
}

impl Value {
    pub fn print_str(&self) -> String {
        match self {
            Value::Number(n) => n.to_string(),
            Value::String(s) => s.clone(),
            Value::Boolean(b) => b.to_string(),
            Value::Nil => "nil".to_string(),
        }
    }
}

struct CallFrame {
    _name: String,
    instructions: Vec<Instruction>,
    pc: usize,
    locals: HashMap<String, Value>,
}

pub struct VM {
    pub globals: HashMap<String, Value>,
    pub functions: HashMap<String, FunctionData>,
    pub stack: Vec<Value>,
}

impl VM {
    pub fn new() -> Self {
        Self {
            globals: HashMap::new(),
            functions: HashMap::new(),
            stack: Vec::new(),
        }
    }

    fn pop_stack(&mut self) -> Result<Value, String> {
        self.stack.pop().ok_or_else(|| "VM Error: Stack underflow".to_string())
    }

    pub fn run(&mut self, program: CompiledProgram) -> Result<Value, String> {
        // Load functions
        for func in program.functions {
            self.functions.insert(func.name.clone(), func);
        }

        // Convert constants
        let constants: Vec<Value> = program.constants.iter().map(|c| match c {
            Constant::Number(n) => Value::Number(*n),
            Constant::String(s) => Value::String(s.clone()),
            Constant::Boolean(b) => Value::Boolean(*b),
            Constant::Nil => Value::Nil,
        }).collect();

        // Create main frame
        let mut frames = vec![CallFrame {
            _name: "main".to_string(),
            instructions: program.instructions,
            pc: 0,
            locals: HashMap::new(),
        }];

        while !frames.is_empty() {
            let frame_idx = frames.len() - 1;

            if frames[frame_idx].pc >= frames[frame_idx].instructions.len() {
                // Implicit return from main or end of block
                if frames.len() > 1 {
                    frames.pop();
                    continue;
                } else {
                    break;
                }
            }

            let inst = frames[frame_idx].instructions[frames[frame_idx].pc].clone();
            frames[frame_idx].pc += 1;

            match inst {
                Instruction::LoadConst(idx) => {
                    if idx >= constants.len() {
                        return Err(format!("VM Error: Constant index {} out of bounds", idx));
                    }
                    self.stack.push(constants[idx].clone());
                }
                Instruction::LoadVar(name) => {
                    // Search locals first, then globals
                    if let Some(val) = frames[frame_idx].locals.get(&name) {
                        self.stack.push(val.clone());
                    } else if let Some(val) = self.globals.get(&name) {
                        self.stack.push(val.clone());
                    } else {
                        return Err(format!("VM Error: Undefined variable '{}'", name));
                    }
                }
                Instruction::StoreVar(name) => {
                    let val = self.pop_stack()?;
                    // Assign local first if it exists, or if we are in function scope
                    if frame_idx > 0 {
                        // In a function frame, we default to local store unless it's explicitly global
                        frames[frame_idx].locals.insert(name.clone(), val);
                    } else {
                        // In main scope, store in globals
                        self.globals.insert(name.clone(), val);
                    }
                }
                Instruction::Pop => {
                    self.pop_stack()?;
                }
                Instruction::Add => {
                    let r = self.pop_stack()?;
                    let l = self.pop_stack()?;
                    match (l, r) {
                        (Value::Number(a), Value::Number(b)) => self.stack.push(Value::Number(a + b)),
                        (Value::String(a), Value::String(b)) => self.stack.push(Value::String(format!("{}{}", a, b))),
                        (Value::String(a), Value::Number(b)) => self.stack.push(Value::String(format!("{}{}", a, b))),
                        (Value::Number(a), Value::String(b)) => self.stack.push(Value::String(format!("{}{}", a, b))),
                        (a, b) => return Err(format!("VM Error: Cannot add values of type {:?} and {:?}", a, b)),
                    }
                }
                Instruction::Sub => {
                    let r = self.pop_stack()?;
                    let l = self.pop_stack()?;
                    match (l, r) {
                        (Value::Number(a), Value::Number(b)) => self.stack.push(Value::Number(a - b)),
                        (a, b) => return Err(format!("VM Error: Cannot subtract values of type {:?} and {:?}", a, b)),
                    }
                }
                Instruction::Mul => {
                    let r = self.pop_stack()?;
                    let l = self.pop_stack()?;
                    match (l, r) {
                        (Value::Number(a), Value::Number(b)) => self.stack.push(Value::Number(a * b)),
                        (a, b) => return Err(format!("VM Error: Cannot multiply values of type {:?} and {:?}", a, b)),
                    }
                }
                Instruction::Div => {
                    let r = self.pop_stack()?;
                    let l = self.pop_stack()?;
                    match (l, r) {
                        (Value::Number(a), Value::Number(b)) => {
                            if b == 0.0 {
                                return Err("VM Error: Division by zero".to_string());
                            }
                            self.stack.push(Value::Number(a / b))
                        }
                        (a, b) => return Err(format!("VM Error: Cannot divide values of type {:?} and {:?}", a, b)),
                    }
                }
                Instruction::Mod => {
                    let r = self.pop_stack()?;
                    let l = self.pop_stack()?;
                    match (l, r) {
                        (Value::Number(a), Value::Number(b)) => self.stack.push(Value::Number(a % b)),
                        (a, b) => return Err(format!("VM Error: Cannot perform modulo on values of type {:?} and {:?}", a, b)),
                    }
                }
                Instruction::Eq => {
                    let r = self.pop_stack()?;
                    let l = self.pop_stack()?;
                    self.stack.push(Value::Boolean(l == r));
                }
                Instruction::Ne => {
                    let r = self.pop_stack()?;
                    let l = self.pop_stack()?;
                    self.stack.push(Value::Boolean(l != r));
                }
                Instruction::Lt => {
                    let r = self.pop_stack()?;
                    let l = self.pop_stack()?;
                    match (l, r) {
                        (Value::Number(a), Value::Number(b)) => self.stack.push(Value::Boolean(a < b)),
                        (a, b) => return Err(format!("VM Error: Cannot compare values of type {:?} and {:?}", a, b)),
                    }
                }
                Instruction::Gt => {
                    let r = self.pop_stack()?;
                    let l = self.pop_stack()?;
                    match (l, r) {
                        (Value::Number(a), Value::Number(b)) => self.stack.push(Value::Boolean(a > b)),
                        (a, b) => return Err(format!("VM Error: Cannot compare values of type {:?} and {:?}", a, b)),
                    }
                }
                Instruction::Le => {
                    let r = self.pop_stack()?;
                    let l = self.pop_stack()?;
                    match (l, r) {
                        (Value::Number(a), Value::Number(b)) => self.stack.push(Value::Boolean(a <= b)),
                        (a, b) => return Err(format!("VM Error: Cannot compare values of type {:?} and {:?}", a, b)),
                    }
                }
                Instruction::Ge => {
                    let r = self.pop_stack()?;
                    let l = self.pop_stack()?;
                    match (l, r) {
                        (Value::Number(a), Value::Number(b)) => self.stack.push(Value::Boolean(a >= b)),
                        (a, b) => return Err(format!("VM Error: Cannot compare values of type {:?} and {:?}", a, b)),
                    }
                }
                Instruction::Jump(pc) => {
                    frames[frame_idx].pc = pc;
                }
                Instruction::JumpIfFalse(pc) => {
                    let val = self.pop_stack()?;
                    let cond = match val {
                        Value::Boolean(b) => b,
                        Value::Nil => false,
                        _ => true,
                    };
                    if !cond {
                        frames[frame_idx].pc = pc;
                    }
                }
                Instruction::Call(func_name, arg_count) => {
                    let func_data = self.functions.get(&func_name).cloned().ok_or_else(|| {
                        format!("VM Error: Call to undefined function '{}'", func_name)
                    })?;

                    if func_data.params.len() != arg_count {
                        return Err(format!(
                            "VM Error: Function '{}' expected {} arguments, got {}",
                            func_name, func_data.params.len(), arg_count
                        ));
                    }

                    let mut fn_locals = HashMap::new();
                    for i in (0..arg_count).rev() {
                        let val = self.pop_stack()?;
                        let param_name = &func_data.params[i];
                        fn_locals.insert(param_name.clone(), val);
                    }

                    frames.push(CallFrame {
                        _name: func_name,
                        instructions: func_data.instructions,
                        pc: 0,
                        locals: fn_locals,
                    });
                }
                Instruction::Return => {
                    frames.pop();
                }
                Instruction::Print => {
                    let val = self.pop_stack()?;
                    println!("{}", val.print_str());
                }
            }
        }

        // Return top of stack if present, otherwise Nil
        if self.stack.is_empty() {
            Ok(Value::Nil)
        } else {
            self.pop_stack()
        }
    }
}
