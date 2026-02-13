# 013: try-except 语句

**状态**: DONE  
**优先级**: P0  
**依赖**: 012-raise-statement

## 完成总结

✅ **所有功能和测试完成！**

- ✅ 所有 4 个 try-except 专有测试通过
- ✅ 所有 5 个受影响的函数测试通过（由 Task 015 修复）
- ✅ 总计 58/58 测试通过

**Note**: Task 015 的 VM 架构重构修复了函数调用问题，使本任务的所有测试通过。

## 任务概述

实现 try-except 语句，支持异常捕获和处理。

## 目标

完成后用户可以：
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

## 需要实现的内容

### 1. 添加字节码指令

```rust
// src/bytecode.rs
pub enum Instruction {
    // ... 现有指令
    SetupTry(usize),    // 设置 try 块，参数是 except 块的位置
    PopTry,             // 移除 try 块（正常结束时）
    GetExceptionType,   // 获取异常类型（用于类型检查）
    Dup,                // 复制栈顶元素
}
```

### 2. 扩展 VM 结构

```rust
// src/vm.rs
enum BlockType {
    Try { handler_offset: usize },
}

struct Block {
    block_type: BlockType,
    stack_size: usize,
}

pub struct VM {
    stack: Vec<Value>,
    frames: Vec<Frame>,
    blocks: Vec<Block>,  // 新增：块栈
}
```

### 3. 实现 try-except 编译

```rust
// src/compiler.rs
ast::Stmt::Try(try_stmt) => {
    // 暂时只处理没有 finally 的情况
    if !try_stmt.finalbody.is_empty() {
        return Err("finally not supported yet".to_string());
    }
    
    self.compile_try_except(&try_stmt, bytecode)?;
    Ok(())
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
    bytecode.push(Instruction::Jump(0)); // 跳过 except 块
    
    // except 块开始位置
    let except_start = bytecode.len();
    bytecode[handler_offset_placeholder] = Instruction::SetupTry(except_start);
    
    let mut handler_end_placeholders = Vec::new();
    
    // 编译每个 except 子句
    for handler in &try_stmt.handlers {
        if let Some(exc_type) = &handler.type_ {
            // 复制异常对象
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
            bytecode.push(Instruction::JumpIfFalse(0));
            
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
            bytecode.push(Instruction::Jump(0));
            handler_end_placeholders.push(handler_end_placeholder);
            
            // 回填"跳到下一个 handler"的地址
            let next_handler_pos = bytecode.len();
            bytecode[next_handler_placeholder] = Instruction::JumpIfFalse(next_handler_pos);
            
            // 弹出比较结果（类型不匹配）
            bytecode.push(Instruction::Pop);
        } else {
            // 捕获所有异常
            if let Some(name) = &handler.name {
                let var_name = name.to_string();
                if let Some(&index) = self.local_vars.get(&var_name) {
                    bytecode.push(Instruction::SetLocal(index));
                } else {
                    bytecode.push(Instruction::SetGlobal(var_name));
                }
            }
            bytecode.push(Instruction::Pop);
            
            // 编译 except 块体
            for stmt in &handler.body {
                self.compile_stmt(stmt, bytecode)?;
            }
            
            let handler_end_placeholder = bytecode.len();
            bytecode.push(Instruction::Jump(0));
            handler_end_placeholders.push(handler_end_placeholder);
        }
    }
    
    // 如果所有 except 都不匹配，重新抛出
    bytecode.push(Instruction::Raise);
    
    // 回填跳转地址
    let after_except = bytecode.len();
    bytecode[end_offset_placeholder] = Instruction::Jump(after_except);
    for placeholder in handler_end_placeholders {
        bytecode[placeholder] = Instruction::Jump(after_except);
    }
    
    Ok(())
}

fn parse_exception_type(&self, expr: &ast::Expr) -> Result<ExceptionType, String> {
    if let ast::Expr::Name(name) = expr {
        match name.id.to_string().as_str() {
            "ValueError" => Ok(ExceptionType::ValueError),
            "TypeError" => Ok(ExceptionType::TypeError),
            "IndexError" => Ok(ExceptionType::IndexError),
            "KeyError" => Ok(ExceptionType::KeyError),
            "ZeroDivisionError" => Ok(ExceptionType::ZeroDivisionError),
            "RuntimeError" => Ok(ExceptionType::RuntimeError),
            "IteratorError" => Ok(ExceptionType::IteratorError),
            "Exception" => Ok(ExceptionType::Exception),
            _ => Err(format!("Unknown exception type: {}", name.id)),
        }
    } else {
        Err("Exception type must be a name".to_string())
    }
}
```

### 4. 实现 VM 指令

```rust
// src/vm.rs
Instruction::SetupTry(handler_offset) => {
    self.blocks.push(Block {
        block_type: BlockType::Try {
            handler_offset: *handler_offset,
        },
        stack_size: self.stack.len(),
    });
    ip += 1;
}

Instruction::PopTry => {
    self.blocks.pop();
    ip += 1;
}

Instruction::GetExceptionType => {
    let exception = self.stack.last()
        .ok_or_else(|| Value::error(ExceptionType::RuntimeError, "Stack underflow"))?;
    
    if let Some(exc) = exception.as_exception() {
        let type_value = Value::Int(exc.exception_type.clone() as i32);
        self.stack.push(type_value);
    } else {
        return Err(Value::error(
            ExceptionType::TypeError,
            "Expected exception object"
        ));
    }
    ip += 1;
}

Instruction::Dup => {
    let value = self.stack.last()
        .ok_or_else(|| Value::error(ExceptionType::RuntimeError, "Stack underflow"))?
        .clone();
    self.stack.push(value);
    ip += 1;
}
```

### 5. 修改异常处理流程

```rust
// src/vm.rs
pub fn execute(&mut self, bytecode: &ByteCode, globals: &mut HashMap<String, Value>) 
    -> VMResult<Value> {
    let mut ip = 0;
    let code = bytecode;

    while ip < code.len() {
        let instruction = &code[ip];

        match self.execute_instruction(instruction, &mut ip, code, globals) {
            Ok(()) => { /* 继续 */ }
            Err(exception) => {
                // 查找 try 块
                while let Some(block) = self.blocks.pop() {
                    match block.block_type {
                        BlockType::Try { handler_offset } => {
                            // 找到 try 块，跳转到 except
                            self.stack.truncate(block.stack_size);
                            self.stack.push(exception);
                            ip = handler_offset;
                            break;
                        }
                    }
                }
                
                // 如果没有找到 try 块，向上传播
                if self.blocks.is_empty() {
                    return Err(exception);
                }
            }
        }
    }

    Ok(self.stack.pop().unwrap_or(Value::None))
}
```

### 6. ExceptionType 实现 as i32

```rust
// src/value.rs
impl ExceptionType {
    pub fn as_i32(&self) -> i32 {
        match self {
            ExceptionType::Exception => 0,
            ExceptionType::RuntimeError => 1,
            ExceptionType::IndexError => 2,
            ExceptionType::KeyError => 3,
            ExceptionType::ValueError => 4,
            ExceptionType::TypeError => 5,
            ExceptionType::ZeroDivisionError => 6,
            ExceptionType::IteratorError => 7,
        }
    }
}
```

## 验收条件

- [ ] SetupTry, PopTry, GetExceptionType, Dup 指令实现
- [ ] try-except 语句编译实现
- [ ] VM 块栈管理实现
- [ ] 异常捕获流程实现
- [ ] 支持多个 except 子句
- [ ] 支持 except Type as var 语法
- [ ] 所有测试通过

## 测试要求

### 单元测试

```rust
#[test]
fn test_try_except_basic() {
    let mut ctx = Context::new();
    ctx.eval(r#"
result = "ok"
try:
    x = 1 / 0
except ZeroDivisionError:
    result = "caught"
    "#).unwrap();
    
    let result = ctx.get("result").unwrap();
    assert_eq!(result.as_string(), Some("caught"));
}

#[test]
fn test_try_except_with_binding() {
    let mut ctx = Context::new();
    ctx.eval(r#"
msg = ""
try:
    raise ValueError("test error")
except ValueError as e:
    msg = "caught"
    "#).unwrap();
    
    let msg = ctx.get("msg").unwrap();
    assert_eq!(msg.as_string(), Some("caught"));
}

#[test]
fn test_try_except_multiple() {
    let mut ctx = Context::new();
    ctx.eval(r#"
result = ""
try:
    x = 1 / 0
except ValueError:
    result = "value"
except ZeroDivisionError:
    result = "zero"
    "#).unwrap();
    
    let result = ctx.get("result").unwrap();
    assert_eq!(result.as_string(), Some("zero"));
}

#[test]
fn test_try_except_no_match() {
    let mut ctx = Context::new();
    let result = ctx.eval(r#"
try:
    x = 1 / 0
except ValueError:
    pass
    "#);
    
    assert!(result.is_err());
    let exc = result.unwrap_err().as_exception().unwrap();
    assert_eq!(exc.exception_type, ExceptionType::ZeroDivisionError);
}
```

## 注意事项

1. 这个任务暂时不实现 finally 块
2. 需要重构 VM 的 execute 函数以支持异常处理
3. 确保嵌套的 try-except 正确工作
4. 异常类型匹配需要精确

## 实现状态

### 已完成
- ✅ 所有字节码指令实现（SetupTry, PopTry, GetExceptionType, Dup）
- ✅ try-except 语句编译
- ✅ VM 块栈管理
- ✅ 异常捕获流程
- ✅ 多个 except 子句支持
- ✅ except Type as var 语法支持

### 测试结果

**try-except 专有测试（本任务）**：
- ✅ test_try_except_basic - PASS
- ✅ test_try_except_with_binding - PASS  
- ✅ test_try_except_multiple - PASS
- ✅ test_try_except_no_match - PASS (**Task 015 修复**)

**受影响的既有函数测试**：
- ✅ test_factorial - PASS (**Task 015 修复**)
- ✅ test_fibonacci_iterative - PASS (**Task 015 修复**)
- ✅ test_for_with_function - PASS (**Task 015 修复**)
- ✅ test_if_else - PASS (**Task 015 修复**)
- ✅ test_function_def_and_call - PASS (**Task 015 修复**)

**总结**：
- ✅ 异常处理功能完全正常：4/4 专有测试通过
- ✅ 函数调用问题已解决：5/5 既有测试通过
- ✅ 所有测试通过：58/58
- ✅ 修复方案：Task 015 - VM 单循环架构重构

### ~~已知问题~~ 已解决问题

1. **~~函数调用架构问题~~** ✅ **已由 Task 015 修复**
   
   ~~当前 VM 使用递归调用 `execute_frame()` 来处理函数调用，这与新的异常处理机制（基于主循环的异常传播）存在架构不匹配。~~
   
   **已解决**：Task 015 重构了 VM 为单一主循环架构，所有函数调用通过帧切换实现。
   
   **之前影响的测试（现已全部通过）**：
   - `test_factorial` - ~~错误：Stack underflow~~ ✅ PASS
   - `test_fibonacci_iterative` - ~~错误：Stack underflow~~ ✅ PASS
   - `test_for_with_function` - ~~错误：no active frame~~ ✅ PASS
   - `test_if_else` - ~~错误：no active frame~~ ✅ PASS
   - `test_function_def_and_call` - ~~错误：返回值不正确~~ ✅ PASS

2. **~~test_try_except_no_match 格式问题~~** ✅ **已由 Task 015 修复**
   
   ~~异常正确重新抛出，但错误信息格式可能不匹配测试预期。~~
   
   **已解决**：Task 015 添加了 `pass` 语句支持，测试现在通过。

## 后续任务

完成后可以开始：
- 014: finally 块和迭代器修改检测

## 建议的改进

要完全修复函数调用问题，需要：
1. 移除 execute_frame 的递归调用
2. 将 Code 也放入 Frame 中，使 VM 能在单一循环中处理多个栈帧
3. Call 指令应该只创建新帧并修改 ip，不调用新的 execute
4. Return 指令恢复前一个帧的 code 和 ip
