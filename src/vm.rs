use crate::builtins;
use crate::bytecode::{ByteCode, Instruction};
use crate::value::{DictKey, ExceptionType, Function, IteratorState, Module, Value};
use std::cell::RefCell;
use std::collections::HashMap;
use std::rc::Rc;

struct Frame {
    locals: Vec<Value>,
    ip: usize,
    code: ByteCode,
    stack_base: usize, // 此帧在栈上的起始位置
}

enum BlockType {
    Try { handler_offset: usize },
    Finally { handler_offset: usize },
}

struct Block {
    block_type: BlockType,
    stack_size: usize,
}

pub struct VM {
    stack: Vec<Value>,
    frames: Vec<Frame>,
    blocks: Vec<Block>,
    loaded_modules: HashMap<String, Rc<RefCell<Module>>>,
    extension_modules: HashMap<String, Module>,
}

impl VM {
    pub fn new() -> Self {
        VM {
            stack: Vec::new(),
            frames: Vec::new(),
            blocks: Vec::new(),
            loaded_modules: HashMap::new(),
            extension_modules: HashMap::new(),
        }
    }

    pub fn register_extension_module(&mut self, name: &str, module: Module) {
        self.extension_modules.insert(name.to_string(), module);
    }

    pub fn execute(
        &mut self,
        bytecode: &ByteCode,
        globals: &mut HashMap<String, Value>,
    ) -> Result<Value, Value> {
        // Create initial frame
        let main_frame = Frame {
            locals: Vec::new(),
            ip: 0,
            code: bytecode.clone(),
            stack_base: 0,
        };
        self.frames.push(main_frame);

        // Main loop: continue as long as there are frames
        'main_loop: loop {
            // Check if we have frames left
            if self.frames.is_empty() {
                break;
            }

            // Get current frame's ip and code length
            let (current_ip, code_len) = {
                let current_frame = self.frames.last().unwrap();
                (current_frame.ip, current_frame.code.len())
            };

            // Check if current frame is done
            if current_ip >= code_len {
                // Current frame finished execution
                if self.frames.len() == 1 {
                    self.frames.pop();
                    break;
                } else {
                    self.frames.pop();
                    continue;
                }
            }

            // Get the instruction to execute
            let instruction = {
                let current_frame = self.frames.last().unwrap();
                current_frame.code[current_frame.ip].clone()
            };

            let mut ip = current_ip;
            let result = self.execute_instruction(&instruction, &mut ip, globals);

            // Update current frame's ip (unless it's a Return instruction which sets ip to MAX)
            if ip != usize::MAX {
                if let Some(frame) = self.frames.last_mut() {
                    frame.ip = ip;
                }
            }

            // Exception handling
            if let Err(exception) = result {
                loop {
                    if let Some(block) = self.blocks.pop() {
                        match block.block_type {
                            BlockType::Try { handler_offset } => {
                                self.stack.truncate(block.stack_size);
                                self.stack.push(exception);
                                if let Some(frame) = self.frames.last_mut() {
                                    frame.ip = handler_offset;
                                }
                                continue 'main_loop;
                            }
                            BlockType::Finally { handler_offset } => {
                                // For finally blocks, we need to execute the finally code
                                // and then re-raise the exception
                                self.stack.truncate(block.stack_size);
                                self.stack.push(exception.clone()); // Push exception for EndFinally
                                if let Some(frame) = self.frames.last_mut() {
                                    frame.ip = handler_offset;
                                }
                                continue 'main_loop;
                            }
                        }
                    } else {
                        return Err(exception);
                    }
                }
            }
        }

        Ok(self.stack.pop().unwrap_or(Value::None))
    }

    fn load_module(&mut self, name: &str) -> Result<Rc<RefCell<Module>>, Value> {
        // 1. 检查缓存
        if let Some(module) = self.loaded_modules.get(name) {
            return Ok(module.clone());
        }

        // 2. 加载内置模块
        if builtins::is_builtin_module(name) {
            let module = builtins::get_builtin_module(name);
            let module_rc = Rc::new(RefCell::new(module));
            self.loaded_modules
                .insert(name.to_string(), module_rc.clone());
            return Ok(module_rc);
        }

        // 3. 检查扩展模块
        if let Some(module) = self.extension_modules.get(name) {
            let module_rc = Rc::new(RefCell::new(module.clone()));
            self.loaded_modules
                .insert(name.to_string(), module_rc.clone());
            return Ok(module_rc);
        }

        // 4. 未找到
        Err(Value::error(
            ExceptionType::RuntimeError,
            &format!("No module named '{}'", name),
        ))
    }

    fn execute_instruction(
        &mut self,
        instruction: &Instruction,
        ip: &mut usize,
        globals: &mut HashMap<String, Value>,
    ) -> Result<(), Value> {
        match instruction {
            Instruction::PushInt(i) => {
                self.stack.push(Value::Int(*i));
                *ip += 1;
            }
            Instruction::PushFloat(f) => {
                self.stack.push(Value::Float(*f));
                *ip += 1;
            }
            Instruction::PushBool(b) => {
                self.stack.push(Value::Bool(*b));
                *ip += 1;
            }
            Instruction::PushNone => {
                self.stack.push(Value::None);
                *ip += 1;
            }
            Instruction::PushString(s) => {
                self.stack.push(Value::String(s.clone()));
                *ip += 1;
            }
            Instruction::Pop => {
                self.stack
                    .pop()
                    .ok_or_else(|| Value::error(ExceptionType::RuntimeError, "Stack underflow"))?;
                *ip += 1;
            }
            Instruction::Add => {
                let b = self
                    .stack
                    .pop()
                    .ok_or_else(|| Value::error(ExceptionType::RuntimeError, "Stack underflow"))?;
                let a = self
                    .stack
                    .pop()
                    .ok_or_else(|| Value::error(ExceptionType::RuntimeError, "Stack underflow"))?;
                match (a, b) {
                    (Value::Int(a), Value::Int(b)) => self.stack.push(Value::Int(a + b)),
                    (Value::Float(a), Value::Float(b)) => self.stack.push(Value::Float(a + b)),
                    (Value::Int(a), Value::Float(b)) => self.stack.push(Value::Float(a as f64 + b)),
                    (Value::Float(a), Value::Int(b)) => self.stack.push(Value::Float(a + b as f64)),
                    (Value::String(a), Value::String(b)) => {
                        self.stack.push(Value::String(format!("{}{}", a, b)))
                    }
                    _ => {
                        return Err(Value::error(
                            ExceptionType::TypeError,
                            "unsupported operand types for +",
                        ));
                    }
                }
                *ip += 1;
            }
            Instruction::Sub => {
                let b = self
                    .stack
                    .pop()
                    .ok_or_else(|| Value::error(ExceptionType::RuntimeError, "Stack underflow"))?;
                let a = self
                    .stack
                    .pop()
                    .ok_or_else(|| Value::error(ExceptionType::RuntimeError, "Stack underflow"))?;
                match (a, b) {
                    (Value::Int(a), Value::Int(b)) => self.stack.push(Value::Int(a - b)),
                    (Value::Float(a), Value::Float(b)) => self.stack.push(Value::Float(a - b)),
                    (Value::Int(a), Value::Float(b)) => self.stack.push(Value::Float(a as f64 - b)),
                    (Value::Float(a), Value::Int(b)) => self.stack.push(Value::Float(a - b as f64)),
                    _ => {
                        return Err(Value::error(
                            ExceptionType::TypeError,
                            "unsupported operand types for -",
                        ));
                    }
                }
                *ip += 1;
            }
            Instruction::Mul => {
                let b = self
                    .stack
                    .pop()
                    .ok_or_else(|| Value::error(ExceptionType::RuntimeError, "Stack underflow"))?;
                let a = self
                    .stack
                    .pop()
                    .ok_or_else(|| Value::error(ExceptionType::RuntimeError, "Stack underflow"))?;
                match (a, b) {
                    (Value::Int(a), Value::Int(b)) => self.stack.push(Value::Int(a * b)),
                    (Value::Float(a), Value::Float(b)) => self.stack.push(Value::Float(a * b)),
                    (Value::Int(a), Value::Float(b)) => self.stack.push(Value::Float(a as f64 * b)),
                    (Value::Float(a), Value::Int(b)) => self.stack.push(Value::Float(a * b as f64)),
                    _ => {
                        return Err(Value::error(
                            ExceptionType::TypeError,
                            "unsupported operand types for *",
                        ));
                    }
                }
                *ip += 1;
            }
            Instruction::Div => {
                let b = self
                    .stack
                    .pop()
                    .ok_or_else(|| Value::error(ExceptionType::RuntimeError, "Stack underflow"))?;
                let a = self
                    .stack
                    .pop()
                    .ok_or_else(|| Value::error(ExceptionType::RuntimeError, "Stack underflow"))?;
                match (a, b) {
                    (Value::Int(a), Value::Int(b)) => {
                        if b == 0 {
                            return Err(Value::error(
                                ExceptionType::ZeroDivisionError,
                                "division by zero",
                            ));
                        }
                        self.stack.push(Value::Int(a / b))
                    }
                    (Value::Float(a), Value::Float(b)) => {
                        if b == 0.0 {
                            return Err(Value::error(
                                ExceptionType::ZeroDivisionError,
                                "division by zero",
                            ));
                        }
                        self.stack.push(Value::Float(a / b))
                    }
                    (Value::Int(a), Value::Float(b)) => {
                        if b == 0.0 {
                            return Err(Value::error(
                                ExceptionType::ZeroDivisionError,
                                "division by zero",
                            ));
                        }
                        self.stack.push(Value::Float(a as f64 / b))
                    }
                    (Value::Float(a), Value::Int(b)) => {
                        if b == 0 {
                            return Err(Value::error(
                                ExceptionType::ZeroDivisionError,
                                "division by zero",
                            ));
                        }
                        self.stack.push(Value::Float(a / b as f64))
                    }
                    _ => {
                        return Err(Value::error(
                            ExceptionType::TypeError,
                            "unsupported operand types for /",
                        ));
                    }
                }
                *ip += 1;
            }
            Instruction::Eq => {
                let b = self
                    .stack
                    .pop()
                    .ok_or_else(|| Value::error(ExceptionType::RuntimeError, "Stack underflow"))?;
                let a = self
                    .stack
                    .pop()
                    .ok_or_else(|| Value::error(ExceptionType::RuntimeError, "Stack underflow"))?;

                let result = match (a, b) {
                    // 数值类型
                    (Value::Int(a), Value::Int(b)) => a == b,
                    (Value::Float(a), Value::Float(b)) => a == b,
                    (Value::Int(a), Value::Float(b)) => (a as f64) == b,
                    (Value::Float(a), Value::Int(b)) => a == (b as f64),

                    // 字符串
                    (Value::String(a), Value::String(b)) => a == b,

                    // 布尔值
                    (Value::Bool(a), Value::Bool(b)) => a == b,

                    // None
                    (Value::None, Value::None) => true,

                    // 其他组合返回 false
                    _ => false,
                };

                self.stack.push(Value::Bool(result));
                *ip += 1;
            }
            Instruction::Ne => {
                let b = self
                    .stack
                    .pop()
                    .ok_or_else(|| Value::error(ExceptionType::RuntimeError, "Stack underflow"))?;
                let a = self
                    .stack
                    .pop()
                    .ok_or_else(|| Value::error(ExceptionType::RuntimeError, "Stack underflow"))?;

                let result = match (a, b) {
                    // 数值类型
                    (Value::Int(a), Value::Int(b)) => a != b,
                    (Value::Float(a), Value::Float(b)) => a != b,
                    (Value::Int(a), Value::Float(b)) => (a as f64) != b,
                    (Value::Float(a), Value::Int(b)) => a != (b as f64),

                    // 字符串
                    (Value::String(a), Value::String(b)) => a != b,

                    // 布尔值
                    (Value::Bool(a), Value::Bool(b)) => a != b,

                    // None
                    (Value::None, Value::None) => false,

                    // 其他组合返回 true
                    _ => true,
                };

                self.stack.push(Value::Bool(result));
                *ip += 1;
            }
            Instruction::Lt => {
                let b = self
                    .stack
                    .pop()
                    .ok_or_else(|| Value::error(ExceptionType::RuntimeError, "Stack underflow"))?;
                let a = self
                    .stack
                    .pop()
                    .ok_or_else(|| Value::error(ExceptionType::RuntimeError, "Stack underflow"))?;

                let result = match (a, b) {
                    // 数值类型
                    (Value::Int(a), Value::Int(b)) => a < b,
                    (Value::Float(a), Value::Float(b)) => a < b,
                    (Value::Int(a), Value::Float(b)) => (a as f64) < b,
                    (Value::Float(a), Value::Int(b)) => a < (b as f64),

                    // 字符串（字典序）
                    (Value::String(a), Value::String(b)) => a < b,

                    // 不支持的组合
                    _ => {
                        return Err(Value::error(
                            ExceptionType::TypeError,
                            "unsupported operand types for <",
                        ));
                    }
                };

                self.stack.push(Value::Bool(result));
                *ip += 1;
            }
            Instruction::Le => {
                let b = self
                    .stack
                    .pop()
                    .ok_or_else(|| Value::error(ExceptionType::RuntimeError, "Stack underflow"))?;
                let a = self
                    .stack
                    .pop()
                    .ok_or_else(|| Value::error(ExceptionType::RuntimeError, "Stack underflow"))?;

                let result = match (a, b) {
                    // 数值类型
                    (Value::Int(a), Value::Int(b)) => a <= b,
                    (Value::Float(a), Value::Float(b)) => a <= b,
                    (Value::Int(a), Value::Float(b)) => (a as f64) <= b,
                    (Value::Float(a), Value::Int(b)) => a <= (b as f64),

                    // 字符串（字典序）
                    (Value::String(a), Value::String(b)) => a <= b,

                    // 不支持的组合
                    _ => {
                        return Err(Value::error(
                            ExceptionType::TypeError,
                            "unsupported operand types for <=",
                        ));
                    }
                };

                self.stack.push(Value::Bool(result));
                *ip += 1;
            }
            Instruction::Gt => {
                let b = self
                    .stack
                    .pop()
                    .ok_or_else(|| Value::error(ExceptionType::RuntimeError, "Stack underflow"))?;
                let a = self
                    .stack
                    .pop()
                    .ok_or_else(|| Value::error(ExceptionType::RuntimeError, "Stack underflow"))?;

                let result = match (a, b) {
                    // 数值类型
                    (Value::Int(a), Value::Int(b)) => a > b,
                    (Value::Float(a), Value::Float(b)) => a > b,
                    (Value::Int(a), Value::Float(b)) => (a as f64) > b,
                    (Value::Float(a), Value::Int(b)) => a > (b as f64),

                    // 字符串（字典序）
                    (Value::String(a), Value::String(b)) => a > b,

                    // 不支持的组合
                    _ => {
                        return Err(Value::error(
                            ExceptionType::TypeError,
                            "unsupported operand types for >",
                        ));
                    }
                };

                self.stack.push(Value::Bool(result));
                *ip += 1;
            }
            Instruction::Ge => {
                let b = self
                    .stack
                    .pop()
                    .ok_or_else(|| Value::error(ExceptionType::RuntimeError, "Stack underflow"))?;
                let a = self
                    .stack
                    .pop()
                    .ok_or_else(|| Value::error(ExceptionType::RuntimeError, "Stack underflow"))?;

                let result = match (a, b) {
                    // 数值类型
                    (Value::Int(a), Value::Int(b)) => a >= b,
                    (Value::Float(a), Value::Float(b)) => a >= b,
                    (Value::Int(a), Value::Float(b)) => (a as f64) >= b,
                    (Value::Float(a), Value::Int(b)) => a >= (b as f64),

                    // 字符串（字典序）
                    (Value::String(a), Value::String(b)) => a >= b,

                    // 不支持的组合
                    _ => {
                        return Err(Value::error(
                            ExceptionType::TypeError,
                            "unsupported operand types for >=",
                        ));
                    }
                };

                self.stack.push(Value::Bool(result));
                *ip += 1;
            }
            Instruction::GetGlobal(name) => {
                let value = globals
                    .get(name)
                    .ok_or_else(|| {
                        Value::error(
                            ExceptionType::RuntimeError,
                            &format!("name '{}' is not defined", name),
                        )
                    })?
                    .clone();
                self.stack.push(value);
                *ip += 1;
            }
            Instruction::SetGlobal(name) => {
                let value = self
                    .stack
                    .last()
                    .ok_or_else(|| Value::error(ExceptionType::RuntimeError, "Stack underflow"))?
                    .clone();
                globals.insert(name.clone(), value);
                *ip += 1;
            }
            Instruction::GetLocal(index) => {
                if let Some(frame) = self.frames.last() {
                    let value = frame
                        .locals
                        .get(*index)
                        .ok_or_else(|| {
                            Value::error(
                                ExceptionType::RuntimeError,
                                &format!("local variable {} not found", index),
                            )
                        })?
                        .clone();
                    self.stack.push(value);
                } else {
                    return Err(Value::error(ExceptionType::RuntimeError, "no active frame"));
                }
                *ip += 1;
            }
            Instruction::SetLocal(index) => {
                let value = self
                    .stack
                    .last()
                    .ok_or_else(|| Value::error(ExceptionType::RuntimeError, "Stack underflow"))?
                    .clone();
                if let Some(frame) = self.frames.last_mut() {
                    if *index >= frame.locals.len() {
                        frame.locals.resize(*index + 1, Value::None);
                    }
                    frame.locals[*index] = value;
                } else {
                    return Err(Value::error(ExceptionType::RuntimeError, "no active frame"));
                }
                *ip += 1;
            }
            Instruction::Jump(offset) => {
                *ip = *offset;
            }
            Instruction::JumpIfFalse(offset) => {
                let value = self
                    .stack
                    .pop()
                    .ok_or_else(|| Value::error(ExceptionType::RuntimeError, "Stack underflow"))?;
                if !value.is_truthy() {
                    *ip = *offset;
                } else {
                    *ip += 1;
                }
            }
            Instruction::MakeFunction {
                name,
                params,
                code_len,
            } => {
                let current_frame = self
                    .frames
                    .last()
                    .ok_or_else(|| Value::error(ExceptionType::RuntimeError, "no active frame"))?;

                // Extract function body bytecode from current frame
                let func_code = current_frame.code[*ip + 1..*ip + 1 + code_len].to_vec();
                let func = Function {
                    name: name.clone(),
                    params: params.clone(),
                    code: func_code,
                };
                globals.insert(name.clone(), Value::Function(func));
                *ip += 1 + code_len;
            }
            Instruction::Call(arg_count) => {
                // Get arguments from stack
                let mut args = Vec::new();
                for _ in 0..*arg_count {
                    args.push(self.stack.pop().ok_or_else(|| {
                        Value::error(ExceptionType::RuntimeError, "Stack underflow")
                    })?);
                }
                args.reverse();

                // Get function
                let func_value = self
                    .stack
                    .pop()
                    .ok_or_else(|| Value::error(ExceptionType::RuntimeError, "Stack underflow"))?;

                match func_value {
                    Value::NativeFunction(native_fn) => {
                        // Call native function directly
                        let result = native_fn(args)?;
                        self.stack.push(result);
                        *ip += 1;
                    }
                    Value::Function(func) => {
                        // Call Python function
                        if args.len() != func.params.len() {
                            return Err(Value::error(
                                ExceptionType::TypeError,
                                &format!(
                                    "{}() takes {} positional argument{} but {} {} given",
                                    func.name,
                                    func.params.len(),
                                    if func.params.len() == 1 { "" } else { "s" },
                                    args.len(),
                                    if args.len() == 1 { "was" } else { "were" }
                                ),
                            ));
                        }

                        // Update calling frame's IP before creating new frame
                        if let Some(calling_frame) = self.frames.last_mut() {
                            calling_frame.ip = *ip + 1;
                        }

                        // Create new frame
                        let new_frame = Frame {
                            locals: args,
                            ip: 0,
                            code: func.code.clone(),
                            stack_base: self.stack.len(),
                        };
                        self.frames.push(new_frame);

                        // Signal that we shouldn't update IP again in main loop
                        *ip = usize::MAX;
                    }
                    _ => {
                        return Err(Value::error(
                            ExceptionType::TypeError,
                            "object is not callable",
                        ));
                    }
                }
            }
            Instruction::Return => {
                // Return value is already on stack top
                self.frames.pop();
                // Don't update IP - the calling frame's IP is already correct
                // We signal this by setting ip to a special value that we check later
                *ip = usize::MAX; // Signal that we shouldn't update the frame IP
            }
            Instruction::Print => {
                let value = self
                    .stack
                    .pop()
                    .ok_or_else(|| Value::error(ExceptionType::RuntimeError, "Stack underflow"))?;
                Self::print_value(&value);
                self.stack.push(Value::None);
                *ip += 1;
            }
            Instruction::Int => {
                let value = self
                    .stack
                    .pop()
                    .ok_or_else(|| Value::error(ExceptionType::RuntimeError, "Stack underflow"))?;
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
                    Value::String(s) => s.parse::<i32>().map_err(|_| {
                        Value::error(
                            ExceptionType::ValueError,
                            &format!("invalid literal for int() with base 10: '{}'", s),
                        )
                    })?,
                    _ => {
                        return Err(Value::error(
                            ExceptionType::TypeError,
                            "int() argument must be a string or a number",
                        ));
                    }
                };
                self.stack.push(Value::Int(result));
                *ip += 1;
            }
            Instruction::Float => {
                let value = self
                    .stack
                    .pop()
                    .ok_or_else(|| Value::error(ExceptionType::RuntimeError, "Stack underflow"))?;
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
                    Value::String(s) => s.parse::<f64>().map_err(|_| {
                        Value::error(
                            ExceptionType::ValueError,
                            &format!("could not convert string to float: '{}'", s),
                        )
                    })?,
                    _ => {
                        return Err(Value::error(
                            ExceptionType::TypeError,
                            "float() argument must be a string or a number",
                        ));
                    }
                };
                self.stack.push(Value::Float(result));
                *ip += 1;
            }
            Instruction::Len => {
                let value = self
                    .stack
                    .pop()
                    .ok_or_else(|| Value::error(ExceptionType::RuntimeError, "Stack underflow"))?;
                let result = match value {
                    Value::String(s) => s.len() as i32,
                    Value::List(list) => list.borrow().items.len() as i32,
                    Value::Dict(dict) => dict.borrow().len() as i32,
                    _ => {
                        return Err(Value::error(
                            ExceptionType::TypeError,
                            "object of this type has no len()",
                        ));
                    }
                };
                self.stack.push(Value::Int(result));
                *ip += 1;
            }
            Instruction::BuildList(count) => {
                let mut elements = Vec::new();
                for _ in 0..*count {
                    elements.push(self.stack.pop().ok_or_else(|| {
                        Value::error(ExceptionType::RuntimeError, "Stack underflow")
                    })?);
                }
                elements.reverse();
                self.stack.push(Value::List(Rc::new(RefCell::new(
                    crate::value::ListValue::with_items(elements),
                ))));
                *ip += 1;
            }
            Instruction::BuildDict(count) => {
                let mut dict = HashMap::new();
                for _ in 0..*count {
                    let value = self.stack.pop().ok_or_else(|| {
                        Value::error(ExceptionType::RuntimeError, "Stack underflow")
                    })?;
                    let key = self.stack.pop().ok_or_else(|| {
                        Value::error(ExceptionType::RuntimeError, "Stack underflow")
                    })?;

                    let dict_key = match key {
                        Value::String(s) => DictKey::String(s),
                        Value::Int(i) => DictKey::Int(i),
                        _ => {
                            return Err(Value::error(ExceptionType::TypeError, "unhashable type"));
                        }
                    };
                    dict.insert(dict_key, value);
                }
                self.stack.push(Value::Dict(Rc::new(RefCell::new(dict))));
                *ip += 1;
            }
            Instruction::GetItem => {
                let index = self
                    .stack
                    .pop()
                    .ok_or_else(|| Value::error(ExceptionType::RuntimeError, "Stack underflow"))?;
                let obj = self
                    .stack
                    .pop()
                    .ok_or_else(|| Value::error(ExceptionType::RuntimeError, "Stack underflow"))?;

                match obj {
                    Value::List(list) => {
                        let idx = index.as_int().ok_or_else(|| {
                            Value::error(ExceptionType::TypeError, "list indices must be integers")
                        })?;
                        let list_ref = list.borrow();
                        let len = list_ref.items.len() as i32;
                        let actual_idx = if idx < 0 { len + idx } else { idx };
                        if actual_idx < 0 || actual_idx >= len {
                            return Err(Value::error(
                                ExceptionType::IndexError,
                                "list index out of range",
                            ));
                        }
                        self.stack.push(list_ref.items[actual_idx as usize].clone());
                    }
                    Value::Dict(dict) => {
                        let dict_key = match index {
                            Value::String(s) => DictKey::String(s),
                            Value::Int(i) => DictKey::Int(i),
                            _ => {
                                return Err(Value::error(
                                    ExceptionType::TypeError,
                                    "unhashable type",
                                ));
                            }
                        };
                        let dict_ref = dict.borrow();
                        let value = dict_ref
                            .get(&dict_key)
                            .ok_or_else(|| Value::error(ExceptionType::KeyError, "key not found"))?
                            .clone();
                        self.stack.push(value);
                    }
                    _ => {
                        return Err(Value::error(
                            ExceptionType::TypeError,
                            "object is not subscriptable",
                        ));
                    }
                }
                *ip += 1;
            }
            Instruction::SetItem => {
                let index = self
                    .stack
                    .pop()
                    .ok_or_else(|| Value::error(ExceptionType::RuntimeError, "Stack underflow"))?;
                let obj = self
                    .stack
                    .pop()
                    .ok_or_else(|| Value::error(ExceptionType::RuntimeError, "Stack underflow"))?;
                let value = self
                    .stack
                    .last()
                    .ok_or_else(|| Value::error(ExceptionType::RuntimeError, "Stack underflow"))?
                    .clone();

                match obj {
                    Value::List(list) => {
                        let idx = index.as_int().ok_or_else(|| {
                            Value::error(ExceptionType::TypeError, "list indices must be integers")
                        })?;
                        let mut list_ref = list.borrow_mut();
                        let len = list_ref.items.len() as i32;
                        let actual_idx = if idx < 0 { len + idx } else { idx };
                        if actual_idx < 0 || actual_idx >= len {
                            return Err(Value::error(
                                ExceptionType::IndexError,
                                "list assignment index out of range",
                            ));
                        }
                        list_ref.items[actual_idx as usize] = value;
                        list_ref.increment_version(); // Increment version on modification
                    }
                    Value::Dict(dict) => {
                        let dict_key = match index {
                            Value::String(s) => DictKey::String(s),
                            Value::Int(i) => DictKey::Int(i),
                            _ => {
                                return Err(Value::error(
                                    ExceptionType::TypeError,
                                    "unhashable type",
                                ));
                            }
                        };
                        dict.borrow_mut().insert(dict_key, value);
                    }
                    _ => {
                        return Err(Value::error(
                            ExceptionType::TypeError,
                            "object does not support item assignment",
                        ));
                    }
                }
                *ip += 1;
            }
            Instruction::CallMethod(method_name, arg_count) => {
                // 获取参数
                let mut args = Vec::new();
                for _ in 0..*arg_count {
                    args.push(self.stack.pop().ok_or_else(|| {
                        Value::error(ExceptionType::RuntimeError, "Stack underflow")
                    })?);
                }
                args.reverse();

                // 获取对象
                let obj = self
                    .stack
                    .pop()
                    .ok_or_else(|| Value::error(ExceptionType::RuntimeError, "Stack underflow"))?;

                match obj {
                    Value::List(list) => match method_name.as_str() {
                        "append" => {
                            if args.len() != 1 {
                                return Err(Value::error(
                                    ExceptionType::TypeError,
                                    "append() takes exactly one argument",
                                ));
                            }
                            let mut list_ref = list.borrow_mut();
                            list_ref.items.push(args[0].clone());
                            list_ref.increment_version(); // Increment version on modification
                            drop(list_ref);
                            self.stack.push(Value::None);
                        }
                        "pop" => {
                            if args.len() != 0 {
                                return Err(Value::error(
                                    ExceptionType::TypeError,
                                    "pop() takes no arguments",
                                ));
                            }
                            let mut list_ref = list.borrow_mut();
                            let value = list_ref.items.pop().ok_or_else(|| {
                                Value::error(ExceptionType::IndexError, "pop from empty list")
                            })?;
                            list_ref.increment_version(); // Increment version on modification
                            drop(list_ref);
                            self.stack.push(value);
                        }
                        _ => {
                            return Err(Value::error(
                                ExceptionType::RuntimeError,
                                &format!("'list' object has no attribute '{}'", method_name),
                            ));
                        }
                    },
                    Value::Dict(dict) => match method_name.as_str() {
                        "keys" => {
                            if args.len() != 0 {
                                return Err(Value::error(
                                    ExceptionType::TypeError,
                                    "keys() takes no arguments",
                                ));
                            }
                            let keys: Vec<Value> = dict
                                .borrow()
                                .keys()
                                .map(|k| match k {
                                    DictKey::String(s) => Value::String(s.clone()),
                                    DictKey::Int(i) => Value::Int(*i),
                                })
                                .collect();
                            self.stack.push(Value::List(Rc::new(RefCell::new(
                                crate::value::ListValue::with_items(keys),
                            ))));
                        }
                        _ => {
                            return Err(Value::error(
                                ExceptionType::RuntimeError,
                                &format!("'dict' object has no attribute '{}'", method_name),
                            ));
                        }
                    },
                    Value::Module(module) => {
                        let attr = module.borrow().get_attribute(method_name).ok_or_else(|| {
                            Value::error(
                                ExceptionType::RuntimeError,
                                &format!("module has no attribute '{}'", method_name),
                            )
                        })?;

                        match attr {
                            Value::NativeFunction(native_fn) => {
                                let result = native_fn(args)?;
                                self.stack.push(result);
                            }
                            _ => {
                                return Err(Value::error(
                                    ExceptionType::TypeError,
                                    "attribute is not callable",
                                ));
                            }
                        }
                    }
                    Value::Match(match_obj) => {
                        match method_name.as_str() {
                            "group" => {
                                let index = if args.is_empty() {
                                    0
                                } else {
                                    args[0].as_int().ok_or_else(|| {
                                        Value::error(
                                            ExceptionType::TypeError,
                                            "group index must be an integer",
                                        )
                                    })? as usize
                                };

                                if index >= match_obj.groups.len() {
                                    return Err(Value::error(
                                        ExceptionType::IndexError,
                                        "no such group",
                                    ));
                                }

                                let result = match &match_obj.groups[index] {
                                    Some(s) => Value::String(s.clone()),
                                    None => Value::None,
                                };
                                self.stack.push(result);
                            }
                            "groups" => {
                                let groups: Vec<Value> = match_obj
                                    .groups
                                    .iter()
                                    .skip(1) // 跳过第 0 组（整个匹配）
                                    .map(|g| match g {
                                        Some(s) => Value::String(s.clone()),
                                        None => Value::None,
                                    })
                                    .collect();
                                self.stack.push(Value::List(Rc::new(RefCell::new(
                                    crate::value::ListValue::with_items(groups),
                                ))));
                            }
                            "start" => {
                                self.stack.push(Value::Int(match_obj.start as i32));
                            }
                            "end" => {
                                self.stack.push(Value::Int(match_obj.end as i32));
                            }
                            "span" => {
                                self.stack.push(Value::List(Rc::new(RefCell::new(
                                    crate::value::ListValue::with_items(vec![
                                        Value::Int(match_obj.start as i32),
                                        Value::Int(match_obj.end as i32),
                                    ]),
                                ))));
                            }
                            _ => {
                                return Err(Value::error(
                                    ExceptionType::AttributeError,
                                    &format!("'Match' object has no attribute '{}'", method_name),
                                ));
                            }
                        }
                    }
                    _ => {
                        return Err(Value::error(
                            ExceptionType::RuntimeError,
                            &format!("object has no attribute '{}'", method_name),
                        ));
                    }
                }
                *ip += 1;
            }
            Instruction::Range => {
                // 栈布局: [..., arg_count, arg1, arg2, ...]
                // 对于 range(stop): 栈是 [..., 1, stop]
                // 对于 range(start, stop): 栈是 [..., 2, start, stop]
                // 对于 range(start, stop, step): 栈是 [..., 3, start, stop, step]

                let stack_len = self.stack.len();
                if stack_len < 2 {
                    return Err(Value::error(
                        ExceptionType::RuntimeError,
                        "invalid range() call: stack too small",
                    ));
                }

                // 从栈顶往下找参数数量，优先检查更多参数的情况
                // 这样可以避免误判（比如 range(1, 3) 中的 1 不应该被当作参数数量）
                let mut args_count = 0;

                // 先尝试3个参数
                if stack_len >= 4 {
                    if let Some(Value::Int(3)) = self.stack.get(stack_len - 4) {
                        args_count = 3;
                    }
                }
                // 再尝试2个参数
                if args_count == 0 && stack_len >= 3 {
                    if let Some(Value::Int(2)) = self.stack.get(stack_len - 3) {
                        args_count = 2;
                    }
                }
                // 最后尝试1个参数
                if args_count == 0 && stack_len >= 2 {
                    if let Some(Value::Int(1)) = self.stack.get(stack_len - 2) {
                        args_count = 1;
                    }
                }

                if args_count == 0 {
                    return Err(Value::error(
                        ExceptionType::RuntimeError,
                        "invalid range() call: could not find argument count",
                    ));
                }

                let (start, stop, step) = if args_count == 3 {
                    let step = self.pop_int()?;
                    let stop = self.pop_int()?;
                    let start = self.pop_int()?;
                    // 弹出参数数量
                    self.stack.pop();
                    (start, stop, step)
                } else if args_count == 2 {
                    let stop = self.pop_int()?;
                    let start = self.pop_int()?;
                    // 弹出参数数量
                    self.stack.pop();
                    (start, stop, 1)
                } else {
                    let stop = self.pop_int()?;
                    // 弹出参数数量
                    self.stack.pop();
                    (0, stop, 1)
                };

                if step == 0 {
                    return Err(Value::error(
                        ExceptionType::ValueError,
                        "range() arg 3 must not be zero",
                    ));
                }

                let iter_state = IteratorState::Range {
                    current: start,
                    stop,
                    step,
                };
                self.stack
                    .push(Value::Iterator(Rc::new(RefCell::new(iter_state))));
                *ip += 1;
            }
            Instruction::GetIter => {
                let obj = self
                    .stack
                    .pop()
                    .ok_or_else(|| Value::error(ExceptionType::RuntimeError, "Stack underflow"))?;

                let iter = match obj {
                    Value::List(list) => {
                        let version = list.borrow().version; // Record version at iterator creation
                        let iter_state = IteratorState::List {
                            list: list.clone(),
                            index: 0,
                            version,
                        };
                        Value::Iterator(Rc::new(RefCell::new(iter_state)))
                    }
                    Value::Dict(dict) => {
                        let keys: Vec<DictKey> = dict.borrow().keys().cloned().collect();
                        let iter_state = IteratorState::DictKeys { keys, index: 0 };
                        Value::Iterator(Rc::new(RefCell::new(iter_state)))
                    }
                    Value::Iterator(_) => obj, // 已经是迭代器
                    _ => {
                        return Err(Value::error(
                            ExceptionType::TypeError,
                            "object is not iterable",
                        ));
                    }
                };

                self.stack.push(iter);
                *ip += 1;
            }
            Instruction::ForIter(jump_target) => {
                // 栈顶是迭代器，我们需要保留它并压入下一个值
                let iter = self
                    .stack
                    .last()
                    .ok_or_else(|| Value::error(ExceptionType::RuntimeError, "Stack underflow"))?
                    .clone();

                match iter {
                    Value::Iterator(iter_state) => {
                        let mut state = iter_state.borrow_mut();
                        let next_value = match &mut *state {
                            IteratorState::Range {
                                current,
                                stop,
                                step,
                            } => {
                                if (*step > 0 && *current < *stop)
                                    || (*step < 0 && *current > *stop)
                                {
                                    let value = *current;
                                    *current += *step;
                                    Some(Value::Int(value))
                                } else {
                                    None
                                }
                            }
                            IteratorState::List {
                                list,
                                index,
                                version,
                            } => {
                                // Check if list was modified
                                let current_version = list.borrow().version;
                                if *version != current_version {
                                    return Err(Value::error(
                                        ExceptionType::IteratorError,
                                        "list modified during iteration",
                                    ));
                                }

                                let list_ref = list.borrow();
                                if *index < list_ref.items.len() {
                                    let value = list_ref.items[*index].clone();
                                    *index += 1;
                                    Some(value)
                                } else {
                                    None
                                }
                            }
                            IteratorState::DictKeys { keys, index } => {
                                if *index < keys.len() {
                                    let key = &keys[*index];
                                    *index += 1;
                                    let value = match key {
                                        DictKey::String(s) => Value::String(s.clone()),
                                        DictKey::Int(i) => Value::Int(*i),
                                    };
                                    Some(value)
                                } else {
                                    None
                                }
                            }
                        };

                        if let Some(value) = next_value {
                            // 有下一个值：保留迭代器在栈上，压入值
                            // 栈状态变化: [iterator] -> [iterator, value]
                            self.stack.push(value);
                            *ip += 1;
                        } else {
                            // 迭代结束：跳转到循环结束
                            // 迭代器仍在栈上，会被后续的 Pop 清理
                            *ip = *jump_target;
                        }
                    }
                    _ => {
                        return Err(Value::error(ExceptionType::TypeError, "Expected iterator"));
                    }
                }
            }
            Instruction::MakeException(exc_type) => {
                let message = self
                    .stack
                    .pop()
                    .ok_or_else(|| Value::error(ExceptionType::RuntimeError, "Stack underflow"))?;

                let msg_str = match message {
                    Value::String(s) => s,
                    _ => {
                        return Err(Value::error(
                            ExceptionType::TypeError,
                            "Exception message must be a string",
                        ));
                    }
                };

                let exception = Value::error(exc_type.clone(), msg_str);
                self.stack.push(exception);
                *ip += 1;
            }
            Instruction::Raise => {
                let exception = self
                    .stack
                    .pop()
                    .ok_or_else(|| Value::error(ExceptionType::RuntimeError, "Stack underflow"))?;

                if !exception.is_exception() {
                    return Err(Value::error(
                        ExceptionType::TypeError,
                        "raise requires an exception object",
                    ));
                }

                // 返回异常（作为错误）
                return Err(exception);
            }
            Instruction::SetupTry(handler_offset) => {
                self.blocks.push(Block {
                    block_type: BlockType::Try {
                        handler_offset: *handler_offset,
                    },
                    stack_size: self.stack.len(),
                });
                *ip += 1;
            }
            Instruction::PopTry => {
                self.blocks.pop();
                *ip += 1;
            }
            Instruction::GetExceptionType => {
                let exception = self
                    .stack
                    .last()
                    .ok_or_else(|| Value::error(ExceptionType::RuntimeError, "Stack underflow"))?;

                if let Some(exc) = exception.as_exception() {
                    let type_value = Value::Int(exc.exception_type.as_i32());
                    self.stack.push(type_value);
                } else {
                    return Err(Value::error(
                        ExceptionType::TypeError,
                        "Expected exception object",
                    ));
                }
                *ip += 1;
            }
            Instruction::Dup => {
                let value = self
                    .stack
                    .last()
                    .ok_or_else(|| Value::error(ExceptionType::RuntimeError, "Stack underflow"))?
                    .clone();
                self.stack.push(value);
                *ip += 1;
            }
            Instruction::Break => {
                // Break statements are compiled to Jump instructions by the compiler
                // This should never be executed
                return Err(Value::error(
                    ExceptionType::RuntimeError,
                    "'break' outside loop",
                ));
            }
            Instruction::Continue => {
                // Continue statements are compiled to Jump instructions by the compiler
                // This should never be executed
                return Err(Value::error(
                    ExceptionType::RuntimeError,
                    "'continue' outside loop",
                ));
            }
            Instruction::SetupFinally(handler_offset) => {
                self.blocks.push(Block {
                    block_type: BlockType::Finally {
                        handler_offset: *handler_offset,
                    },
                    stack_size: self.stack.len(),
                });
                *ip += 1;
            }
            Instruction::PopFinally => {
                self.blocks.pop();
                *ip += 1;
            }
            Instruction::EndFinally => {
                // Check if there's an exception on the stack
                let value = self
                    .stack
                    .pop()
                    .ok_or_else(|| Value::error(ExceptionType::RuntimeError, "Stack underflow"))?;

                // If the value is an exception, re-raise it
                if value.is_exception() {
                    return Err(value);
                }
                // Otherwise, continue normally (value should be None)
                *ip += 1;
            }
            Instruction::Import(module_name) => {
                let module = self.load_module(module_name)?;
                self.stack.push(Value::Module(module));
                *ip += 1;
            }
            Instruction::GetAttr(attr_name) => {
                let value = self
                    .stack
                    .pop()
                    .ok_or_else(|| Value::error(ExceptionType::RuntimeError, "Stack underflow"))?;

                match value {
                    Value::Module(module) => {
                        let attr = module.borrow().get_attribute(attr_name).ok_or_else(|| {
                            Value::error(
                                ExceptionType::RuntimeError,
                                &format!("module has no attribute '{}'", attr_name),
                            )
                        })?;
                        self.stack.push(attr);
                    }
                    _ => {
                        return Err(Value::error(
                            ExceptionType::TypeError,
                            "getattr on non-module",
                        ));
                    }
                }
                *ip += 1;
            }
        }

        Ok(())
    }

    fn pop_int(&mut self) -> Result<i32, Value> {
        let value = self
            .stack
            .pop()
            .ok_or_else(|| Value::error(ExceptionType::RuntimeError, "Stack underflow"))?;
        value
            .as_int()
            .ok_or_else(|| Value::error(ExceptionType::TypeError, "Expected integer"))
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
                for (i, item) in list_ref.items.iter().enumerate() {
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
            Value::Iterator(_) => println!("<iterator>"),
            Value::Function(f) => println!("<function {}>", f.name),
            Value::Exception(exc) => {
                println!("{:?}: {}", exc.exception_type, exc.message);
            }
            Value::Module(m) => println!("<module '{}'>", m.borrow().name),
            Value::NativeFunction(_) => println!("<built-in function>"),
            Value::Regex(_) => println!("<regex pattern>"),
            Value::Match(m) => println!("<re.Match object; span=({}, {})>", m.start, m.end),
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
                for (i, item) in list_ref.items.iter().enumerate() {
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
            Value::Iterator(_) => print!("<iterator>"),
            Value::Function(f) => print!("<function {}>", f.name),
            Value::Exception(exc) => {
                print!("{:?}: {}", exc.exception_type, exc.message);
            }
            Value::Module(m) => print!("<module '{}'>", m.borrow().name),
            Value::NativeFunction(_) => print!("<built-in function>"),
            Value::Regex(_) => print!("<regex pattern>"),
            Value::Match(m) => print!("<re.Match object; span=({}, {})>", m.start, m.end),
        }
    }
}
