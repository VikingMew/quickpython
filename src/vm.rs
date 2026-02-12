use crate::bytecode::{ByteCode, Instruction};
use crate::value::{Function, Value};
use std::collections::HashMap;

struct Frame {
    locals: Vec<Value>,
    ip: usize,
    code: ByteCode,
}

pub struct VM {
    stack: Vec<Value>,
    frames: Vec<Frame>,
}

impl VM {
    pub fn new() -> Self {
        VM {
            stack: Vec::new(),
            frames: Vec::new(),
        }
    }

    pub fn execute(
        &mut self,
        bytecode: &ByteCode,
        globals: &mut HashMap<String, Value>,
    ) -> Result<Value, String> {
        let mut ip = 0;
        let code = bytecode;

        while ip < code.len() {
            let instruction = &code[ip];

            match instruction {
                Instruction::PushInt(i) => {
                    self.stack.push(Value::Int(*i));
                    ip += 1;
                }
                Instruction::PushFloat(f) => {
                    self.stack.push(Value::Float(*f));
                    ip += 1;
                }
                Instruction::PushBool(b) => {
                    self.stack.push(Value::Bool(*b));
                    ip += 1;
                }
                Instruction::PushNone => {
                    self.stack.push(Value::None);
                    ip += 1;
                }
                Instruction::PushString(s) => {
                    self.stack.push(Value::String(s.clone()));
                    ip += 1;
                }
                Instruction::Pop => {
                    self.stack
                        .pop()
                        .ok_or_else(|| "Stack underflow".to_string())?;
                    ip += 1;
                }
                Instruction::Add => {
                    let b = self
                        .stack
                        .pop()
                        .ok_or_else(|| "Stack underflow".to_string())?;
                    let a = self
                        .stack
                        .pop()
                        .ok_or_else(|| "Stack underflow".to_string())?;
                    match (a, b) {
                        (Value::Int(a), Value::Int(b)) => self.stack.push(Value::Int(a + b)),
                        (Value::Float(a), Value::Float(b)) => self.stack.push(Value::Float(a + b)),
                        (Value::Int(a), Value::Float(b)) => {
                            self.stack.push(Value::Float(a as f64 + b))
                        }
                        (Value::Float(a), Value::Int(b)) => {
                            self.stack.push(Value::Float(a + b as f64))
                        }
                        (Value::String(a), Value::String(b)) => {
                            self.stack.push(Value::String(format!("{}{}", a, b)))
                        }
                        _ => return Err("Type error: unsupported operand types for +".to_string()),
                    }
                    ip += 1;
                }
                Instruction::Sub => {
                    let b = self
                        .stack
                        .pop()
                        .ok_or_else(|| "Stack underflow".to_string())?;
                    let a = self
                        .stack
                        .pop()
                        .ok_or_else(|| "Stack underflow".to_string())?;
                    match (a, b) {
                        (Value::Int(a), Value::Int(b)) => self.stack.push(Value::Int(a - b)),
                        (Value::Float(a), Value::Float(b)) => self.stack.push(Value::Float(a - b)),
                        (Value::Int(a), Value::Float(b)) => {
                            self.stack.push(Value::Float(a as f64 - b))
                        }
                        (Value::Float(a), Value::Int(b)) => {
                            self.stack.push(Value::Float(a - b as f64))
                        }
                        _ => return Err("Type error: unsupported operand types for -".to_string()),
                    }
                    ip += 1;
                }
                Instruction::Mul => {
                    let b = self
                        .stack
                        .pop()
                        .ok_or_else(|| "Stack underflow".to_string())?;
                    let a = self
                        .stack
                        .pop()
                        .ok_or_else(|| "Stack underflow".to_string())?;
                    match (a, b) {
                        (Value::Int(a), Value::Int(b)) => self.stack.push(Value::Int(a * b)),
                        (Value::Float(a), Value::Float(b)) => self.stack.push(Value::Float(a * b)),
                        (Value::Int(a), Value::Float(b)) => {
                            self.stack.push(Value::Float(a as f64 * b))
                        }
                        (Value::Float(a), Value::Int(b)) => {
                            self.stack.push(Value::Float(a * b as f64))
                        }
                        _ => return Err("Type error: unsupported operand types for *".to_string()),
                    }
                    ip += 1;
                }
                Instruction::Div => {
                    let b = self
                        .stack
                        .pop()
                        .ok_or_else(|| "Stack underflow".to_string())?;
                    let a = self
                        .stack
                        .pop()
                        .ok_or_else(|| "Stack underflow".to_string())?;
                    match (a, b) {
                        (Value::Int(a), Value::Int(b)) => {
                            if b == 0 {
                                return Err("Division by zero".to_string());
                            }
                            self.stack.push(Value::Int(a / b))
                        }
                        (Value::Float(a), Value::Float(b)) => {
                            if b == 0.0 {
                                return Err("Division by zero".to_string());
                            }
                            self.stack.push(Value::Float(a / b))
                        }
                        (Value::Int(a), Value::Float(b)) => {
                            if b == 0.0 {
                                return Err("Division by zero".to_string());
                            }
                            self.stack.push(Value::Float(a as f64 / b))
                        }
                        (Value::Float(a), Value::Int(b)) => {
                            if b == 0 {
                                return Err("Division by zero".to_string());
                            }
                            self.stack.push(Value::Float(a / b as f64))
                        }
                        _ => return Err("Type error: unsupported operand types for /".to_string()),
                    }
                    ip += 1;
                }
                Instruction::Eq => {
                    let b = self.pop_int()?;
                    let a = self.pop_int()?;
                    self.stack.push(Value::Bool(a == b));
                    ip += 1;
                }
                Instruction::Ne => {
                    let b = self.pop_int()?;
                    let a = self.pop_int()?;
                    self.stack.push(Value::Bool(a != b));
                    ip += 1;
                }
                Instruction::Lt => {
                    let b = self.pop_int()?;
                    let a = self.pop_int()?;
                    self.stack.push(Value::Bool(a < b));
                    ip += 1;
                }
                Instruction::Le => {
                    let b = self.pop_int()?;
                    let a = self.pop_int()?;
                    self.stack.push(Value::Bool(a <= b));
                    ip += 1;
                }
                Instruction::Gt => {
                    let b = self.pop_int()?;
                    let a = self.pop_int()?;
                    self.stack.push(Value::Bool(a > b));
                    ip += 1;
                }
                Instruction::Ge => {
                    let b = self.pop_int()?;
                    let a = self.pop_int()?;
                    self.stack.push(Value::Bool(a >= b));
                    ip += 1;
                }
                Instruction::GetGlobal(name) => {
                    let value = globals
                        .get(name)
                        .ok_or_else(|| format!("Undefined variable: {}", name))?
                        .clone();
                    self.stack.push(value);
                    ip += 1;
                }
                Instruction::SetGlobal(name) => {
                    let value = self
                        .stack
                        .last()
                        .ok_or_else(|| "Stack underflow".to_string())?
                        .clone();
                    globals.insert(name.clone(), value);
                    ip += 1;
                }
                Instruction::GetLocal(index) => {
                    if let Some(frame) = self.frames.last() {
                        let value = frame
                            .locals
                            .get(*index)
                            .ok_or_else(|| format!("Local variable {} not found", index))?
                            .clone();
                        self.stack.push(value);
                    } else {
                        return Err("No active frame".to_string());
                    }
                    ip += 1;
                }
                Instruction::SetLocal(index) => {
                    let value = self
                        .stack
                        .last()
                        .ok_or_else(|| "Stack underflow".to_string())?
                        .clone();
                    if let Some(frame) = self.frames.last_mut() {
                        if *index >= frame.locals.len() {
                            frame.locals.resize(*index + 1, Value::None);
                        }
                        frame.locals[*index] = value;
                    } else {
                        return Err("No active frame".to_string());
                    }
                    ip += 1;
                }
                Instruction::Jump(offset) => {
                    ip = *offset;
                }
                Instruction::JumpIfFalse(offset) => {
                    let value = self
                        .stack
                        .pop()
                        .ok_or_else(|| "Stack underflow".to_string())?;
                    if !value.is_truthy() {
                        ip = *offset;
                    } else {
                        ip += 1;
                    }
                }
                Instruction::MakeFunction {
                    name,
                    params,
                    code_len,
                } => {
                    // 提取函数体字节码
                    let func_code = code[ip + 1..ip + 1 + code_len].to_vec();
                    let func = Function {
                        name: name.clone(),
                        params: params.clone(),
                        code: func_code,
                    };
                    globals.insert(name.clone(), Value::Function(func));
                    ip += 1 + code_len;
                }
                Instruction::Call(arg_count) => {
                    // 从栈中获取参数
                    let mut args = Vec::new();
                    for _ in 0..*arg_count {
                        args.push(
                            self.stack
                                .pop()
                                .ok_or_else(|| "Stack underflow".to_string())?,
                        );
                    }
                    args.reverse();

                    // 获取函数
                    let func_value = self
                        .stack
                        .pop()
                        .ok_or_else(|| "Stack underflow".to_string())?;
                    let func = match func_value {
                        Value::Function(f) => f,
                        _ => return Err("Not a function".to_string()),
                    };

                    if args.len() != func.params.len() {
                        return Err(format!(
                            "Function {} expects {} arguments, got {}",
                            func.name,
                            func.params.len(),
                            args.len()
                        ));
                    }

                    // 创建新的栈帧
                    let frame = Frame {
                        locals: args,
                        ip: ip + 1,
                        code: code.clone(),
                    };
                    self.frames.push(frame);

                    // 执行函数体
                    let result = self.execute_frame(&func.code, globals)?;
                    self.stack.push(result);

                    ip += 1;
                }
                Instruction::Return => {
                    let return_value = self.stack.pop().unwrap_or(Value::None);
                    if let Some(frame) = self.frames.pop() {
                        ip = frame.ip;
                        return Ok(return_value);
                    } else {
                        return Ok(return_value);
                    }
                }
                Instruction::Print => {
                    let value = self
                        .stack
                        .pop()
                        .ok_or_else(|| "Stack underflow".to_string())?;
                    match value {
                        Value::Int(i) => println!("{}", i),
                        Value::Float(f) => println!("{}", f),
                        Value::Bool(b) => println!("{}", b),
                        Value::String(s) => println!("{}", s),
                        Value::None => println!("None"),
                        Value::Function(f) => println!("<function {}>", f.name),
                    }
                    self.stack.push(Value::None);
                    ip += 1;
                }
                Instruction::Int => {
                    let value = self
                        .stack
                        .pop()
                        .ok_or_else(|| "Stack underflow".to_string())?;
                    let result = match value {
                        Value::Int(i) => i,
                        Value::Float(f) => f as i32,
                        Value::Bool(b) => {
                            if b {
                                1
                            } else {
                                0
                            }
                        }
                        Value::String(s) => s
                            .parse::<i32>()
                            .map_err(|_| format!("invalid literal for int(): '{}'", s))?,
                        _ => return Err("int() argument must be a number or string".to_string()),
                    };
                    self.stack.push(Value::Int(result));
                    ip += 1;
                }
                Instruction::Float => {
                    let value = self
                        .stack
                        .pop()
                        .ok_or_else(|| "Stack underflow".to_string())?;
                    let result = match value {
                        Value::Int(i) => i as f64,
                        Value::Float(f) => f,
                        Value::Bool(b) => {
                            if b {
                                1.0
                            } else {
                                0.0
                            }
                        }
                        Value::String(s) => s
                            .parse::<f64>()
                            .map_err(|_| format!("could not convert string to float: '{}'", s))?,
                        _ => return Err("float() argument must be a number or string".to_string()),
                    };
                    self.stack.push(Value::Float(result));
                    ip += 1;
                }
            }
        }

        self.stack.pop().ok_or_else(|| "Empty stack".to_string())
    }

    fn execute_frame(
        &mut self,
        code: &ByteCode,
        globals: &mut HashMap<String, Value>,
    ) -> Result<Value, String> {
        self.execute(code, globals)
    }

    fn pop_int(&mut self) -> Result<i32, String> {
        let value = self
            .stack
            .pop()
            .ok_or_else(|| "Stack underflow".to_string())?;
        value.as_int().ok_or_else(|| "Expected integer".to_string())
    }
}
