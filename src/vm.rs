use crate::bytecode::{ByteCode, Instruction};
use crate::value::{DictKey, Function, Value};
use std::cell::RefCell;
use std::collections::HashMap;
use std::rc::Rc;

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
                    Self::print_value(&value);
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
                Instruction::Len => {
                    let value = self
                        .stack
                        .pop()
                        .ok_or_else(|| "Stack underflow".to_string())?;
                    let result = match value {
                        Value::String(s) => s.len() as i32,
                        Value::List(list) => list.borrow().len() as i32,
                        Value::Dict(dict) => dict.borrow().len() as i32,
                        _ => return Err("object has no len()".to_string()),
                    };
                    self.stack.push(Value::Int(result));
                    ip += 1;
                }
                Instruction::BuildList(count) => {
                    let mut elements = Vec::new();
                    for _ in 0..*count {
                        elements.push(
                            self.stack
                                .pop()
                                .ok_or_else(|| "Stack underflow".to_string())?,
                        );
                    }
                    elements.reverse();
                    self.stack
                        .push(Value::List(Rc::new(RefCell::new(elements))));
                    ip += 1;
                }
                Instruction::BuildDict(count) => {
                    let mut dict = HashMap::new();
                    for _ in 0..*count {
                        let value = self
                            .stack
                            .pop()
                            .ok_or_else(|| "Stack underflow".to_string())?;
                        let key = self
                            .stack
                            .pop()
                            .ok_or_else(|| "Stack underflow".to_string())?;

                        let dict_key = match key {
                            Value::String(s) => DictKey::String(s),
                            Value::Int(i) => DictKey::Int(i),
                            _ => {
                                return Err("unhashable type: only str and int can be dict keys"
                                    .to_string());
                            }
                        };
                        dict.insert(dict_key, value);
                    }
                    self.stack.push(Value::Dict(Rc::new(RefCell::new(dict))));
                    ip += 1;
                }
                Instruction::GetItem => {
                    let index = self
                        .stack
                        .pop()
                        .ok_or_else(|| "Stack underflow".to_string())?;
                    let obj = self
                        .stack
                        .pop()
                        .ok_or_else(|| "Stack underflow".to_string())?;

                    match obj {
                        Value::List(list) => {
                            let idx = index
                                .as_int()
                                .ok_or_else(|| "list indices must be integers".to_string())?;
                            let list_ref = list.borrow();
                            let len = list_ref.len() as i32;
                            let actual_idx = if idx < 0 { len + idx } else { idx };
                            if actual_idx < 0 || actual_idx >= len {
                                return Err("list index out of range".to_string());
                            }
                            self.stack.push(list_ref[actual_idx as usize].clone());
                        }
                        Value::Dict(dict) => {
                            let dict_key = match index {
                                Value::String(s) => DictKey::String(s),
                                Value::Int(i) => DictKey::Int(i),
                                _ => {
                                    return Err(
                                        "unhashable type: only str and int can be dict keys"
                                            .to_string(),
                                    );
                                }
                            };
                            let dict_ref = dict.borrow();
                            let value = dict_ref
                                .get(&dict_key)
                                .ok_or_else(|| "KeyError".to_string())?
                                .clone();
                            self.stack.push(value);
                        }
                        _ => return Err("object is not subscriptable".to_string()),
                    }
                    ip += 1;
                }
                Instruction::SetItem => {
                    let index = self
                        .stack
                        .pop()
                        .ok_or_else(|| "Stack underflow".to_string())?;
                    let obj = self
                        .stack
                        .pop()
                        .ok_or_else(|| "Stack underflow".to_string())?;
                    let value = self
                        .stack
                        .last()
                        .ok_or_else(|| "Stack underflow".to_string())?
                        .clone();

                    match obj {
                        Value::List(list) => {
                            let idx = index
                                .as_int()
                                .ok_or_else(|| "list indices must be integers".to_string())?;
                            let mut list_ref = list.borrow_mut();
                            let len = list_ref.len() as i32;
                            let actual_idx = if idx < 0 { len + idx } else { idx };
                            if actual_idx < 0 || actual_idx >= len {
                                return Err("list assignment index out of range".to_string());
                            }
                            list_ref[actual_idx as usize] = value;
                        }
                        Value::Dict(dict) => {
                            let dict_key = match index {
                                Value::String(s) => DictKey::String(s),
                                Value::Int(i) => DictKey::Int(i),
                                _ => {
                                    return Err(
                                        "unhashable type: only str and int can be dict keys"
                                            .to_string(),
                                    );
                                }
                            };
                            dict.borrow_mut().insert(dict_key, value);
                        }
                        _ => return Err("object does not support item assignment".to_string()),
                    }
                    ip += 1;
                }
                Instruction::CallMethod(method_name, arg_count) => {
                    // 获取参数
                    let mut args = Vec::new();
                    for _ in 0..*arg_count {
                        args.push(
                            self.stack
                                .pop()
                                .ok_or_else(|| "Stack underflow".to_string())?,
                        );
                    }
                    args.reverse();

                    // 获取对象
                    let obj = self
                        .stack
                        .pop()
                        .ok_or_else(|| "Stack underflow".to_string())?;

                    match obj {
                        Value::List(list) => match method_name.as_str() {
                            "append" => {
                                if args.len() != 1 {
                                    return Err("append() takes exactly one argument".to_string());
                                }
                                list.borrow_mut().push(args[0].clone());
                                self.stack.push(Value::None);
                            }
                            "pop" => {
                                if args.len() != 0 {
                                    return Err("pop() takes no arguments".to_string());
                                }
                                let value = list
                                    .borrow_mut()
                                    .pop()
                                    .ok_or_else(|| "pop from empty list".to_string())?;
                                self.stack.push(value);
                            }
                            _ => return Err(format!("list has no method '{}'", method_name)),
                        },
                        Value::Dict(dict) => match method_name.as_str() {
                            "keys" => {
                                if args.len() != 0 {
                                    return Err("keys() takes no arguments".to_string());
                                }
                                let keys: Vec<Value> = dict
                                    .borrow()
                                    .keys()
                                    .map(|k| match k {
                                        DictKey::String(s) => Value::String(s.clone()),
                                        DictKey::Int(i) => Value::Int(*i),
                                    })
                                    .collect();
                                self.stack.push(Value::List(Rc::new(RefCell::new(keys))));
                            }
                            _ => return Err(format!("dict has no method '{}'", method_name)),
                        },
                        _ => return Err(format!("object has no method '{}'", method_name)),
                    }
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

    fn print_value(value: &Value) {
        match value {
            Value::Int(i) => println!("{}", i),
            Value::Float(f) => println!("{}", f),
            Value::Bool(b) => println!("{}", b),
            Value::String(s) => println!("{}", s),
            Value::None => println!("None"),
            Value::List(list) => {
                print!("[");
                let list_ref = list.borrow();
                for (i, item) in list_ref.iter().enumerate() {
                    if i > 0 {
                        print!(", ");
                    }
                    Self::print_value_inline(item);
                }
                println!("]");
            }
            Value::Dict(dict) => {
                print!("{{");
                let dict_ref = dict.borrow();
                for (i, (key, value)) in dict_ref.iter().enumerate() {
                    if i > 0 {
                        print!(", ");
                    }
                    match key {
                        DictKey::String(s) => print!("'{}': ", s),
                        DictKey::Int(i) => print!("{}: ", i),
                    }
                    Self::print_value_inline(value);
                }
                println!("}}");
            }
            Value::Function(f) => println!("<function {}>", f.name),
        }
    }

    fn print_value_inline(value: &Value) {
        match value {
            Value::Int(i) => print!("{}", i),
            Value::Float(f) => print!("{}", f),
            Value::Bool(b) => print!("{}", b),
            Value::String(s) => print!("'{}'", s),
            Value::None => print!("None"),
            Value::List(list) => {
                print!("[");
                let list_ref = list.borrow();
                for (i, item) in list_ref.iter().enumerate() {
                    if i > 0 {
                        print!(", ");
                    }
                    Self::print_value_inline(item);
                }
                print!("]");
            }
            Value::Dict(dict) => {
                print!("{{");
                let dict_ref = dict.borrow();
                for (i, (key, value)) in dict_ref.iter().enumerate() {
                    if i > 0 {
                        print!(", ");
                    }
                    match key {
                        DictKey::String(s) => print!("'{}': ", s),
                        DictKey::Int(i) => print!("{}: ", i),
                    }
                    Self::print_value_inline(value);
                }
                print!("}}");
            }
            Value::Function(f) => print!("<function {}>", f.name),
        }
    }
}
