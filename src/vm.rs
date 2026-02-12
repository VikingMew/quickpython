use crate::bytecode::{ByteCode, Instruction};
use crate::value::Value;
use std::collections::HashMap;

pub struct VM {
    stack: Vec<Value>,
}

impl VM {
    pub fn new() -> Self {
        VM { stack: Vec::new() }
    }

    pub fn execute(
        &mut self,
        bytecode: &ByteCode,
        globals: &mut HashMap<String, Value>,
    ) -> Result<Value, String> {
        for instruction in bytecode {
            match instruction {
                Instruction::PushInt(i) => {
                    self.stack.push(Value::Int(*i));
                }
                Instruction::Add => {
                    let b = self.pop_int()?;
                    let a = self.pop_int()?;
                    self.stack.push(Value::Int(a + b));
                }
                Instruction::Sub => {
                    let b = self.pop_int()?;
                    let a = self.pop_int()?;
                    self.stack.push(Value::Int(a - b));
                }
                Instruction::Mul => {
                    let b = self.pop_int()?;
                    let a = self.pop_int()?;
                    self.stack.push(Value::Int(a * b));
                }
                Instruction::Div => {
                    let b = self.pop_int()?;
                    let a = self.pop_int()?;
                    if b == 0 {
                        return Err("Division by zero".to_string());
                    }
                    self.stack.push(Value::Int(a / b));
                }
                Instruction::GetGlobal(name) => {
                    let value = globals
                        .get(name)
                        .ok_or_else(|| format!("Undefined variable: {}", name))?
                        .clone();
                    self.stack.push(value);
                }
                Instruction::SetGlobal(name) => {
                    let value = self
                        .stack
                        .last()
                        .ok_or_else(|| "Stack underflow".to_string())?
                        .clone();
                    globals.insert(name.clone(), value);
                }
                Instruction::Pop => {
                    self.stack
                        .pop()
                        .ok_or_else(|| "Stack underflow".to_string())?;
                }
            }
        }

        self.stack.pop().ok_or_else(|| "Empty stack".to_string())
    }

    fn pop_int(&mut self) -> Result<i32, String> {
        let value = self
            .stack
            .pop()
            .ok_or_else(|| "Stack underflow".to_string())?;
        value.as_int().ok_or_else(|| "Expected integer".to_string())
    }
}
