# 015: VM 单循环架构重构

**状态**: DONE  
**优先级**: P1  
**依赖**: 无（但会修复 013 的函数调用问题）

## 完成总结

✅ **所有目标达成！**

- ✅ VM 使用单一主循环，无递归调用
- ✅ 函数调用通过帧切换实现
- ✅ 所有 58 个测试通过（包括之前失败的 6 个）
- ✅ 符合 spec/architecture.md 设计
- ✅ 额外修复：添加了 pass 语句支持

**测试结果**：58 passed; 0 failed ✨

## 任务概述

将 VM 从递归调用 `execute_frame()` 架构重构为单一主循环架构，使其符合 spec/architecture.md 的设计，并修复当前异常处理与函数调用的架构不匹配问题。

## 背景

当前实现的问题：
1. `Call` 指令递归调用 `execute_frame()`，创建新的执行上下文
2. 递归调用与基于主循环的异常处理机制不兼容
3. 导致 4 个函数相关测试失败（factorial, fibonacci, for_with_function, if_else）
4. 错误信息："no active frame" 或 "Stack underflow"

Spec 设计（spec/architecture.md）：
```rust
pub struct Frame {
    locals: HashMap<String, Value>,
    stack: Vec<Value>,
    ip: usize,
    code: Rc<ByteCode>,  // ✅ code 在 Frame 中
}
```

当前实现：
```rust
struct Frame {
    locals: Vec<Value>,
    ip: usize,
    code: ByteCode,  // ❌ 有这个字段但未使用
}

// ❌ Call 指令递归调用
Instruction::Call => {
    // ...
    let result = self.execute_frame(&func.code, globals)?;  // 递归！
    self.stack.push(result);
}
```

## 目标

完成后 VM 架构应该：
- ✅ 使用单一主循环处理所有指令执行
- ✅ 函数调用通过切换帧来实现，不使用递归
- ✅ 异常处理在主循环中统一管理
- ✅ 所有函数相关测试通过
- ✅ 符合 spec/architecture.md 的设计

## 需要实现的内容

### 1. 更新 Frame 结构

```rust
// src/vm.rs
struct Frame {
    locals: Vec<Value>,
    ip: usize,
    code: ByteCode,        // 每个帧有自己的代码
    stack_base: usize,     // 新增：此帧在栈上的起始位置（可选）
}
```

**说明**：
- `stack_base` 用于隔离不同帧的栈空间（可选，如果使用全局栈）
- 或者可以将 `stack` 也移入 Frame（每帧独立栈）

### 2. 重构 execute 主循环

```rust
// src/vm.rs
pub fn execute(
    &mut self,
    bytecode: &ByteCode,
    globals: &mut HashMap<String, Value>,
) -> Result<Value, Value> {
    // 创建初始帧（main 函数）
    let main_frame = Frame {
        locals: Vec::new(),
        ip: 0,
        code: bytecode.clone(),
        stack_base: 0,
    };
    self.frames.push(main_frame);

    // 主循环：只要有帧就继续执行
    'main_loop: while let Some(current_frame) = self.frames.last_mut() {
        // 执行当前帧的指令，直到帧结束或返回
        while current_frame.ip < current_frame.code.len() {
            let instruction = current_frame.code[current_frame.ip].clone();
            let mut ip = current_frame.ip;

            // 执行指令
            let result = self.execute_instruction(&instruction, &mut ip, globals);

            // 更新当前帧的 ip（如果帧还在）
            if let Some(frame) = self.frames.last_mut() {
                frame.ip = ip;
            }

            // 处理异常
            if let Err(exception) = result {
                // 查找 try 块
                loop {
                    if let Some(block) = self.blocks.pop() {
                        match block.block_type {
                            BlockType::Try { handler_offset } => {
                                // 找到 try 块，跳转到 except
                                self.stack.truncate(block.stack_size);
                                self.stack.push(exception);
                                if let Some(frame) = self.frames.last_mut() {
                                    frame.ip = handler_offset;
                                }
                                continue 'main_loop;
                            }
                        }
                    } else {
                        // 没有 try 块，向上传播
                        return Err(exception);
                    }
                }
            }
        }

        // 当前帧执行完毕
        if self.frames.len() == 1 {
            // 最后一个帧（main），退出
            self.frames.pop();
            break;
        } else {
            // 函数返回，弹出帧（返回值已在栈上）
            self.frames.pop();
        }
    }

    Ok(self.stack.pop().unwrap_or(Value::None))
}
```

### 3. 重构 Call 指令

```rust
// src/vm.rs - execute_instruction
Instruction::Call(arg_count) => {
    // 从栈中获取参数
    let mut args = Vec::new();
    for _ in 0..*arg_count {
        args.push(self.stack.pop().ok_or_else(|| {
            Value::error(ExceptionType::RuntimeError, "Stack underflow")
        })?);
    }
    args.reverse();

    // 获取函数
    let func_value = self.stack.pop().ok_or_else(|| {
        Value::error(ExceptionType::RuntimeError, "Stack underflow")
    })?;
    let func = match func_value {
        Value::Function(f) => f,
        _ => {
            return Err(Value::error(
                ExceptionType::TypeError,
                "object is not callable",
            ));
        }
    };

    // 检查参数数量
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

    // 创建新帧并压入帧栈
    let new_frame = Frame {
        locals: args,
        ip: 0,
        code: func.code.clone(),
        stack_base: self.stack.len(),
    };
    self.frames.push(new_frame);

    // 注意：不修改 ip，execute_instruction 返回后
    // 主循环会切换到新帧（frames.last_mut()）
    // 当前帧的 ip 会在 execute_instruction 返回后被更新为 ip+1
    *ip += 1;
}
```

**关键变化**：
- ❌ 移除 `self.execute_frame(&func.code, globals)` 递归调用
- ✅ 直接创建新帧并 push 到 `self.frames`
- ✅ 主循环会自动切换到新帧执行

### 4. 重构 Return 指令

```rust
// src/vm.rs - execute_instruction
Instruction::Return => {
    // 返回值已经在栈顶
    // 弹出当前帧
    self.frames.pop();
    
    // 不需要修改 ip，因为：
    // 1. 如果还有帧，主循环会切换到上一个帧
    // 2. 上一个帧的 ip 已经指向 Call 指令的下一条
    // 3. 如果没有帧了，主循环会退出
}
```

**关键变化**：
- ❌ 移除 `*ip = frame.ip`（不需要手动恢复 ip）
- ✅ 只需弹出帧，主循环会自动处理

### 5. 更新 execute_instruction 签名

```rust
// src/vm.rs
fn execute_instruction(
    &mut self,
    instruction: &Instruction,
    ip: &mut usize,
    globals: &mut HashMap<String, Value>,
) -> Result<(), Value> {
    // 注意：移除了 code 参数
    // 现在 code 从 self.frames.last().unwrap().code 获取
}
```

**关键变化**：
- ❌ 移除 `code: &ByteCode` 参数
- ✅ 需要 code 的指令（如 MakeFunction）从当前帧获取

### 6. 修复 MakeFunction 指令

```rust
// src/vm.rs - execute_instruction
Instruction::MakeFunction { name, params, code_len } => {
    // 从当前帧的代码中提取函数字节码
    let current_frame = self.frames.last().ok_or_else(|| {
        Value::error(ExceptionType::RuntimeError, "no active frame")
    })?;
    
    let func_code = current_frame.code[*ip + 1..*ip + 1 + code_len].to_vec();
    
    let func = Function {
        name: name.clone(),
        params: params.clone(),
        code: func_code,
    };
    
    self.stack.push(Value::Function(func));
    *ip += 1 + code_len;
}
```

### 7. 移除 execute_frame 函数

```rust
// src/vm.rs
// ❌ 删除这个函数
// fn execute_frame(&mut self, code: &ByteCode, globals: &mut HashMap<String, Value>) 
//     -> Result<Value, Value>
```

### 8. 修复 GetLocal/SetLocal（可选）

如果在 main 级别使用了局部变量（编译器生成了 GetLocal/SetLocal），需要处理无帧的情况：

```rust
Instruction::GetLocal(index) => {
    if let Some(frame) = self.frames.last() {
        // 正常函数调用
        let value = frame.locals.get(*index)
            .ok_or_else(|| Value::error(ExceptionType::RuntimeError, "undefined local variable"))?
            .clone();
        self.stack.push(value);
    } else {
        // Main 级别，当作全局变量（或报错）
        return Err(Value::error(ExceptionType::RuntimeError, "no active frame"));
    }
    *ip += 1;
}
```

## 验收条件

- [ ] Frame 结构包含 code 字段并正确使用
- [ ] execute 使用单一主循环，不递归调用
- [ ] Call 指令创建新帧而不调用 execute_frame
- [ ] Return 指令正确弹出帧
- [ ] execute_frame 函数已移除
- [ ] 所有现有测试通过（包括之前失败的 4 个函数测试）
- [ ] 异常处理在主循环中正确工作
- [ ] 代码符合 spec/architecture.md 设计

## 测试要求

### 来自 Task 013 的失败测试（必须修复）

这些测试在 Task 013 实现后失败，原因是递归调用 `execute_frame()` 与异常处理架构不兼容：

```rust
#[test]
fn test_factorial() { 
    // 错误: "Stack underflow"
    // 原因: 递归调用导致栈状态不一致
}

#[test]
fn test_fibonacci_iterative() { 
    // 错误: "Stack underflow"
    // 原因: 递归调用导致栈状态不一致
}

#[test]
fn test_for_with_function() { 
    // 错误: "no active frame"
    // 原因: 递归调用时帧管理混乱
}

#[test]
fn test_if_else() { 
    // 错误: "no active frame"
    // 原因: 条件语句中调用函数时帧丢失
}

#[test]
fn test_function_def_and_call() { 
    // 失败: 返回值不正确
    // 原因: 递归调用时返回值处理有问题
}
```

**重构后这些测试应该全部通过** ✅

### Task 013 的异常测试（应保持通过）

```rust
#[test]
fn test_try_except_basic() { ... }  // ✅ 已通过，应保持

#[test]
fn test_try_except_with_binding() { ... }  // ✅ 已通过，应保持

#[test]
fn test_try_except_multiple() { ... }  // ✅ 已通过，应保持

#[test]
fn test_try_except_no_match() { 
    // ⚠️ 格式问题，重构后应该修复
    // 当前: 异常正确抛出但错误信息格式不匹配
}
```

### 新增测试（可选）

```rust
#[test]
fn test_nested_function_calls() {
    let mut ctx = Context::new();
    ctx.eval(r#"
def a(x):
    return x + 1

def b(x):
    return a(x) + 2

def c(x):
    return b(x) + 3

result = c(10)
    "#).unwrap();
    
    let result = ctx.get("result").unwrap();
    assert_eq!(result.as_int(), Some(16)); // 10 + 1 + 2 + 3
}

#[test]
fn test_exception_in_nested_calls() {
    let mut ctx = Context::new();
    let result = ctx.eval(r#"
def inner():
    x = 1 / 0

def outer():
    try:
        inner()
    except ZeroDivisionError:
        return "caught"
    return "not caught"

result = outer()
    "#);
    
    assert!(result.is_ok());
    let result = ctx.get("result").unwrap();
    assert_eq!(result.as_string(), Some("caught"));
}
```

## 注意事项

1. **向后兼容**: 这是内部重构，外部 API 不应改变
2. **性能**: 单循环架构应该比递归调用更快
3. **调试**: 重构后应该更容易调试（单一调用栈）
4. **渐进式**: 可以先让基础测试通过，再优化性能

## 实现建议

### 实现顺序

1. **阶段 1**: 更新 Frame 结构，添加 stack_base
2. **阶段 2**: 重构 execute 主循环，创建初始帧
3. **阶段 3**: 更新 execute_instruction 签名，移除 code 参数
4. **阶段 4**: 重构 Call 指令，移除递归调用
5. **阶段 5**: 重构 Return 指令
6. **阶段 6**: 修复 MakeFunction 指令
7. **阶段 7**: 移除 execute_frame 函数
8. **阶段 8**: 运行测试，修复问题

### 调试技巧

如果遇到问题，可以添加调试日志：

```rust
// 在主循环开始
eprintln!("Executing frame {}, ip={}, instruction={:?}", 
    self.frames.len(), current_frame.ip, instruction);

// 在帧切换时
eprintln!("Pushing new frame for function: {}", func.name);
eprintln!("Popping frame, returning to ip={}", 
    self.frames.last().map(|f| f.ip).unwrap_or(0));
```

## 相关问题

- 修复 Task 013 中的 4 个失败测试
- 为后续异步支持打下基础（async/await 也需要栈帧管理）
- 使代码库符合 spec 设计

## 后续任务

完成后可以：
- 继续 Task 014: finally 块和迭代器修改检测
- 实现更复杂的函数特性（闭包、装饰器等）
- 添加调试器支持（栈回溯）
