# 异常系统设计

## 概述

QuickPython 的异常系统设计，支持基本的异常抛出、捕获和传播机制。

## 设计目标

1. **简单实用**：实现 Python 异常的核心功能
2. **类型安全**：在 Rust 中安全地表示和处理异常
3. **性能优先**：异常处理不应影响正常执行路径的性能
4. **渐进式**：先实现基础异常，后续可扩展

## 异常类型层次

```
BaseException
├── Exception
│   ├── RuntimeError
│   │   ├── IndexError
│   │   ├── KeyError
│   │   ├── ValueError
│   │   ├── TypeError
│   │   └── ZeroDivisionError
│   └── IteratorError (自定义，用于迭代器相关错误)
└── SystemExit (可选，用于程序退出)
```

### 第一阶段实现的异常类型

1. **Exception** - 基础异常类
2. **RuntimeError** - 运行时错误
3. **IndexError** - 索引越界
4. **KeyError** - 键不存在
5. **ValueError** - 值错误
6. **TypeError** - 类型错误
7. **ZeroDivisionError** - 除零错误
8. **IteratorError** - 迭代器错误（自定义）

## Value 类型扩展

```rust
pub enum Value {
    // ... 现有类型
    Exception(ExceptionValue),
}

pub struct ExceptionValue {
    pub exception_type: ExceptionType,
    pub message: String,
    pub traceback: Option<Vec<TracebackFrame>>, // 可选，后续实现
}

pub enum ExceptionType {
    Exception,
    RuntimeError,
    IndexError,
    KeyError,
    ValueError,
    TypeError,
    ZeroDivisionError,
    IteratorError,
}

pub struct TracebackFrame {
    pub function_name: String,
    pub line_number: usize,
}
```

## 语法支持

### raise 语句

```python
# 抛出异常
raise ValueError("invalid value")
raise IndexError("list index out of range")

# 重新抛出当前异常（后续实现）
raise
```

### try-except 语句

```python
# 基本 try-except
try:
    x = 1 / 0
except ZeroDivisionError:
    print("division by zero")

# 捕获异常并绑定到变量
try:
    x = list[10]
except IndexError as e:
    print(e)

# 多个 except 子句
try:
    risky_operation()
except ValueError:
    print("value error")
except TypeError:
    print("type error")

# 捕获所有异常
try:
    something()
except Exception:
    print("caught exception")
```

### try-finally 语句（后续实现）

```python
try:
    f = open("file.txt")
    process(f)
finally:
    f.close()
```

## 字节码指令

### 新增指令（简化版，借鉴 QuickJS）

```rust
pub enum Instruction {
    // ... 现有指令
    
    // 异常相关（简化设计）
    Raise,                    // 抛出异常，栈顶是异常对象
    SetupTry(usize),          // 设置 try 块，参数是 except 块的位置
    PopTry,                   // 移除 try 块（正常结束时）
    SetupFinally(usize),      // 设置 finally 块，参数是 finally 块的位置
    PopFinally,               // 移除 finally 块
    EndFinally,               // 结束 finally 块，处理异常重新抛出
    
    // 创建异常对象
    MakeException(ExceptionType), // 创建异常，栈顶是消息字符串
}
```

### 异常处理流程（借鉴 QuickJS 的简洁性）

**try-except 编译：**
```
try:
    code1
except ExceptionType:
    handler

编译为：
    SetupTry(handler_offset)    # 记录异常处理器位置
    code1                        # try 块代码
    PopTry                       # 正常结束，移除处理器
    Jump(after_handler)          # 跳过 except 块
handler_offset:
    # 异常发生时，VM 会自动跳转到这里
    # 栈顶是异常对象
    # 检查异常类型是否匹配
    # 如果匹配，执行 handler
    # 如果不匹配，重新抛出
after_handler:
    ...
```

**try-finally 编译：**
```
try:
    code1
finally:
    cleanup

编译为：
    SetupFinally(finally_offset)  # 记录 finally 块位置
    code1                          # try 块代码
    PopFinally                     # 正常结束，移除 finally 块
    PushNone                       # 压入 None 表示无异常
finally_offset:
    # 无论是否异常都会执行到这里
    # 栈顶是异常对象（如果有）或 None（如果无）
    cleanup                        # finally 块代码
    EndFinally                     # 检查是否需要重新抛出异常
```

**try-except-finally 编译：**
```
try:
    code1
except ExceptionType:
    handler
finally:
    cleanup

编译为：
    SetupFinally(finally_offset)   # 外层 finally
    SetupTry(except_offset)        # 内层 try-except
    code1                          # try 块代码
    PopTry                         # 正常结束
    Jump(after_except)
except_offset:
    # except 处理
    ...
after_except:
    PopFinally                     # 正常结束 finally
    PushNone                       # 无异常
finally_offset:
    cleanup                        # finally 块
    EndFinally                     # 处理异常重新抛出
```

**运行时：**
- 当任何指令执行失败时，返回 `Err(exception_value)`
- VM 检查是否有 finally 块
- 如果有 finally 块，先跳转到 finally 块（异常保存在栈上）
- finally 块执行完后，通过 EndFinally 决定是否重新抛出
- 如果没有 finally 块，检查是否有 try 块
- 如果有 try 块，跳转到对应的 except 块
- 如果都没有，向上传播异常

## VM 实现

### 异常处理栈（简化版）

```rust
enum BlockType {
    Try { handler_offset: usize },      // try-except 块
    Finally { handler_offset: usize },  // finally 块
}

struct Block {
    block_type: BlockType,
    stack_size: usize,  // 设置时的栈大小
}

pub struct VM {
    stack: Vec<Value>,
    frames: Vec<Frame>,
    blocks: Vec<Block>,  // 块栈（统一管理 try 和 finally）
}
```

### 异常抛出流程（借鉴 QuickJS）

1. 任何指令执行失败时，返回 `Err(exception_value)`
2. VM 的 execute 函数捕获错误：
   ```rust
   match self.execute_instruction(instruction) {
       Ok(()) => { /* 继续 */ }
       Err(exception) => {
           // 从后往前查找块
           while let Some(block) = self.blocks.pop() {
               match block.block_type {
                   BlockType::Finally { handler_offset } => {
                       // 找到 finally 块，必须先执行
                       self.stack.truncate(block.stack_size);
                       self.stack.push(exception.clone()); // 保存异常
                       ip = handler_offset;
                       // 继续执行，finally 块会通过 EndFinally 重新抛出
                       break;
                   }
                   BlockType::Try { handler_offset } => {
                       // 找到 try 块，跳转到 except
                       self.stack.truncate(block.stack_size);
                       self.stack.push(exception);
                       ip = handler_offset;
                       break;
                   }
               }
           }
           
           // 如果没有找到任何块，向上传播
           if self.blocks.is_empty() {
               return Err(exception);
           }
       }
   }
   ```

### 异常捕获流程

1. `SetupTry` 指令将 try 块信息压入栈
2. 执行 try 块代码
3. 如果没有异常，执行 `PopTry` 移除 try 块
4. 如果有异常，VM 自动跳转到 except 块
5. 在 except 块中检查异常类型是否匹配
6. 如果匹配，执行 except 块
7. 如果不匹配，重新抛出异常（`Raise` 指令）

### Finally 块处理流程

1. `SetupFinally` 指令将 finally 块信息压入栈
2. 执行 try 块代码
3. 如果没有异常：
   - 执行 `PopFinally` 移除 finally 块
   - 执行 `PushNone` 压入 None（表示无异常）
   - 跳转到 finally 块
4. 如果有异常：
   - VM 自动跳转到 finally 块（异常在栈上）
5. 执行 finally 块代码
6. 执行 `EndFinally` 指令：
   - 检查栈顶是否是异常对象
   - 如果是 None，正常继续
   - 如果是异常，重新抛出（`Raise`）

## 编译器实现

### raise 语句编译

```rust
ast::Stmt::Raise(raise) => {
    if let Some(exc) = &raise.exc {
        // 编译异常表达式
        self.compile_expr(exc, bytecode)?;
        bytecode.push(Instruction::Raise);
    } else {
        // 重新抛出当前异常（后续实现）
        return Err("bare raise not supported yet".to_string());
    }
}
```

### try-except 语句编译

```rust
ast::Stmt::Try(try_stmt) => {
    // 检查是否有 finally 块
    let has_finally = !try_stmt.finalbody.is_empty();
    
    if has_finally {
        // 有 finally 块，需要外层包装
        let finally_offset_placeholder = bytecode.len();
        bytecode.push(Instruction::SetupFinally(0)); // 占位符
        
        // 内层的 try-except
        self.compile_try_except(&try_stmt, bytecode)?;
        
        // 正常结束，移除 finally 块
        bytecode.push(Instruction::PopFinally);
        bytecode.push(Instruction::PushNone); // 无异常
        
        // finally 块开始
        let finally_start = bytecode.len();
        bytecode[finally_offset_placeholder] = Instruction::SetupFinally(finally_start);
        
        // 编译 finally 块
        for stmt in &try_stmt.finalbody {
            self.compile_stmt(stmt, bytecode)?;
        }
        
        // 结束 finally，处理异常重新抛出
        bytecode.push(Instruction::EndFinally);
    } else {
        // 没有 finally 块，只有 try-except
        self.compile_try_except(&try_stmt, bytecode)?;
    }
}

fn compile_try_except(&mut self, try_stmt: &ast::Try, bytecode: &mut ByteCode) -> Result<(), String> {
    // 设置 try 块
    let handler_offset_placeholder = bytecode.len();
    bytecode.push(Instruction::SetupTry(0)); // 占位符
    
    // 编译 try 块
    for stmt in &try_stmt.body {
        self.compile_stmt(stmt, bytecode)?;
    }
    
    // 正常结束，移除 try 块
    bytecode.push(Instruction::PopTry);
    let end_offset_placeholder = bytecode.len();
    bytecode.push(Instruction::Jump(0)); // 跳过 except 块，占位符
    
    // except 块开始位置
    let except_start = bytecode.len();
    
    // 回填 SetupTry 的跳转地址
    bytecode[handler_offset_placeholder] = Instruction::SetupTry(except_start);
    
    // 用于回填所有 except 块结束后的跳转
    let mut handler_end_placeholders = Vec::new();
    
    // 编译 except 块
    for (i, handler) in try_stmt.handlers.iter().enumerate() {
        let is_last = i == try_stmt.handlers.len() - 1;
        
        // 如果指定了异常类型，检查类型
        if let Some(exc_type) = &handler.type_ {
            // 复制异常对象（用于类型检查）
            bytecode.push(Instruction::Dup);
            
            // 获取异常类型
            bytecode.push(Instruction::GetExceptionType);
            
            // 压入期望的异常类型
            let expected_type = self.parse_exception_type(exc_type)?;
            bytecode.push(Instruction::PushInt(expected_type as i32));
            
            // 比较类型
            bytecode.push(Instruction::Eq);
            
            // 如果不匹配，跳到下一个 handler
            let next_handler_placeholder = bytecode.len();
            bytecode.push(Instruction::JumpIfFalse(0)); // 占位符
            
            // 类型匹配，弹出比较结果
            bytecode.push(Instruction::Pop);
            
            // 绑定到变量（如果有）
            if let Some(name) = &handler.name {
                let var_name = name.to_string();
                if let Some(&index) = self.local_vars.get(&var_name) {
                    bytecode.push(Instruction::SetLocal(index));
                } else {
                    bytecode.push(Instruction::SetGlobal(var_name));
                }
            }
            bytecode.push(Instruction::Pop); // 清理异常对象
            
            // 编译 except 块体
            for stmt in &handler.body {
                self.compile_stmt(stmt, bytecode)?;
            }
            
            // 跳到 try-except 结束
            let handler_end_placeholder = bytecode.len();
            bytecode.push(Instruction::Jump(0)); // 占位符
            handler_end_placeholders.push(handler_end_placeholder);
            
            // 回填"跳到下一个 handler"的地址
            let next_handler_pos = bytecode.len();
            bytecode[next_handler_placeholder] = Instruction::JumpIfFalse(next_handler_pos);
            
            // 弹出比较结果（类型不匹配的情况）
            bytecode.push(Instruction::Pop);
        } else {
            // 捕获所有异常（except:）
            if let Some(name) = &handler.name {
                let var_name = name.to_string();
                if let Some(&index) = self.local_vars.get(&var_name) {
                    bytecode.push(Instruction::SetLocal(index));
                } else {
                    bytecode.push(Instruction::SetGlobal(var_name));
                }
            }
            bytecode.push(Instruction::Pop); // 清理异常对象
            
            // 编译 except 块体
            for stmt in &handler.body {
                self.compile_stmt(stmt, bytecode)?;
            }
            
            // 跳到 try-except 结束
            let handler_end_placeholder = bytecode.len();
            bytecode.push(Instruction::Jump(0)); // 占位符
            handler_end_placeholders.push(handler_end_placeholder);
        }
    }
    
    // 如果所有 except 都不匹配，重新抛出
    bytecode.push(Instruction::Raise);
    
    // 回填"跳过 except 块"的地址和所有 except 块结束的跳转
    let after_except = bytecode.len();
    bytecode[end_offset_placeholder] = Instruction::Jump(after_except);
    for placeholder in handler_end_placeholders {
        bytecode[placeholder] = Instruction::Jump(after_except);
    }
    
    Ok(())
}
```

### try-finally 语句编译

```rust
// 如果只有 finally，没有 except
if try_stmt.handlers.is_empty() && !try_stmt.finalbody.is_empty() {
    // 设置 finally 块
    let finally_offset_placeholder = bytecode.len();
    bytecode.push(Instruction::SetupFinally(0)); // 占位符
    
    // 编译 try 块
    for stmt in &try_stmt.body {
        self.compile_stmt(stmt, bytecode)?;
    }
    
    // 正常结束，移除 finally 块
    bytecode.push(Instruction::PopFinally);
    bytecode.push(Instruction::PushNone); // 无异常
    
    // finally 块开始
    let finally_start = bytecode.len();
    bytecode[finally_offset_placeholder] = Instruction::SetupFinally(finally_start);
    
    // 编译 finally 块
    for stmt in &try_stmt.finalbody {
        self.compile_stmt(stmt, bytecode)?;
    }
    
    // 结束 finally，处理异常重新抛出
    bytecode.push(Instruction::EndFinally);
}
```

## 内置异常的使用

### 修改现有错误处理

将现有的字符串错误转换为异常对象：

```rust
// 之前：
return Err("list index out of range".to_string());

// 之后：
return Err(Value::Exception(ExceptionValue {
    exception_type: ExceptionType::IndexError,
    message: "list index out of range".to_string(),
    traceback: None,
}));
```

### 在 for 循环中检测列表修改

```rust
// 在 IteratorState::List 中添加版本号
pub enum IteratorState {
    List {
        list: Rc<RefCell<Vec<Value>>>,
        index: usize,
        version: usize, // 列表的版本号
    },
    // ...
}

// 在列表修改时增加版本号
impl Value {
    pub fn list_append(&mut self, value: Value) -> Result<(), Value> {
        match self {
            Value::List(list) => {
                list.borrow_mut().push(value);
                // 增加版本号（需要在 List 中存储版本号）
                Ok(())
            }
            _ => Err(Value::Exception(ExceptionValue {
                exception_type: ExceptionType::TypeError,
                message: "append() requires a list".to_string(),
                traceback: None,
            })),
        }
    }
}

// 在迭代时检查版本号
Instruction::ForIter(jump_target) => {
    let iter = self.stack.last()...;
    match iter {
        Value::Iterator(iter_state) => {
            let mut state = iter_state.borrow_mut();
            match &mut *state {
                IteratorState::List { list, index, version } => {
                    let list_ref = list.borrow();
                    // 检查版本号是否匹配
                    if list_ref.version != *version {
                        return Err(Value::Exception(ExceptionValue {
                            exception_type: ExceptionType::IteratorError,
                            message: "list modified during iteration".to_string(),
                            traceback: None,
                        }));
                    }
                    // ... 正常迭代逻辑
                }
                // ...
            }
        }
        // ...
    }
}
```

## 错误类型统一

### Result 类型调整

```rust
// 将所有 Result<T, String> 改为 Result<T, Value>
pub type VMResult<T> = Result<T, Value>;

impl VM {
    pub fn execute(&mut self, bytecode: &ByteCode, globals: &mut HashMap<String, Value>) 
        -> VMResult<Value> {
        // ...
    }
}
```

### 错误转换

```rust
// 提供便捷的错误创建函数
impl Value {
    pub fn error(exception_type: ExceptionType, message: impl Into<String>) -> Value {
        Value::Exception(ExceptionValue {
            exception_type,
            message: message.into(),
            traceback: None,
        })
    }
}

// 使用示例
return Err(Value::error(ExceptionType::IndexError, "list index out of range"));
```

## 实现阶段

### Phase 1: 基础异常类型和 raise
- [ ] 定义 ExceptionValue 和 ExceptionType
- [ ] 扩展 Value 枚举
- [ ] 实现 Raise 指令
- [ ] 实现 raise 语句编译
- [ ] 修改现有错误为异常对象

### Phase 2: try-except 基础支持
- [ ] 实现 SetupExcept 和 PopExcept 指令
- [ ] 实现异常处理器栈
- [ ] 实现 try-except 语句编译
- [ ] 支持单个 except 子句

### Phase 3: 多 except 和异常绑定
- [ ] 支持多个 except 子句
- [ ] 支持 `except Type as var` 语法
- [ ] 实现异常类型匹配

### Phase 4: 迭代器修改检测
- [ ] 在 List 中添加版本号
- [ ] 在 IteratorState::List 中存储版本号
- [ ] 在 ForIter 中检查版本号
- [ ] 抛出 IteratorError

### Phase 5: 增强功能（可选）
- [ ] 实现 finally 块
- [ ] 实现 traceback
- [ ] 支持 bare raise
- [ ] 实现 else 子句

## 测试用例

### 基础异常测试

```python
# 测试 raise
try:
    raise ValueError("test error")
except ValueError:
    print("caught")

# 测试除零
try:
    x = 1 / 0
except ZeroDivisionError:
    print("division by zero")

# 测试索引错误
try:
    list = [1, 2, 3]
    x = list[10]
except IndexError:
    print("index error")
```

### 迭代器修改检测测试

```python
# 应该抛出 IteratorError
numbers = [1, 2, 3]
try:
    for n in numbers:
        numbers.append(10)
except IteratorError as e:
    print("caught iterator error:", e)
```

## 兼容性考虑

1. **向后兼容**：现有代码应该继续工作
2. **错误消息**：保持清晰的错误消息
3. **性能**：正常执行路径不应受影响

## 未来扩展

1. **自定义异常类**：允许用户定义异常类
2. **异常链**：支持 `raise ... from ...`
3. **上下文管理器**：支持 `with` 语句
4. **异常组**：Python 3.11+ 的 ExceptionGroup
