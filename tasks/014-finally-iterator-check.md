# 014: finally 块和迭代器修改检测

**状态**: DONE  
**优先级**: P0  
**依赖**: 013-try-except

## 完成总结

✅ **所有功能实现完成！**

- ✅ finally 块完全实现
- ✅ try-except-finally 组合支持
- ✅ 列表版本号机制实现
- ✅ 迭代器修改检测（append/pop/索引赋值）
- ✅ 添加了 5 个测试用例，全部通过
- ✅ 总测试数：75 passed; 0 failed

### 实现细节

**Finally 块**：
- 添加了 SetupFinally, PopFinally, EndFinally 指令
- 异常处理流程优先执行 finally 块
- finally 块在正常和异常流程中都执行

**迭代器修改检测**：
- List 结构改为 ListValue { items, version }
- 所有修改操作（append/pop/索引赋值）增加版本号
- ForIter 每次迭代检查版本号，不匹配抛出 IteratorError

## 任务概述

实现 finally 块支持，并添加迭代器修改检测功能，解决 for 循环中修改列表的问题。

## 目标

完成后用户可以：
```python
# try-finally
try:
    f = open("file.txt")
    process(f)
finally:
    f.close()

# try-except-finally
try:
    risky_operation()
except ValueError:
    handle_error()
finally:
    cleanup()

# 迭代器修改检测
numbers = [1, 2, 3]
try:
    for n in numbers:
        numbers.append(10)  # 抛出 IteratorError
except IteratorError as e:
    print("caught:", e)
```

## 需要实现的内容

### 1. 添加字节码指令

```rust
// src/bytecode.rs
pub enum Instruction {
    // ... 现有指令
    SetupFinally(usize),  // 设置 finally 块，参数是 finally 块的位置
    PopFinally,           // 移除 finally 块
    EndFinally,           // 结束 finally 块，处理异常重新抛出
}
```

### 2. 扩展 BlockType

```rust
// src/vm.rs
enum BlockType {
    Try { handler_offset: usize },
    Finally { handler_offset: usize },  // 新增
}
```

### 3. 实现 finally 编译

```rust
// src/compiler.rs
ast::Stmt::Try(try_stmt) => {
    let has_finally = !try_stmt.finalbody.is_empty();
    
    if has_finally {
        // 有 finally 块，需要外层包装
        let finally_offset_placeholder = bytecode.len();
        bytecode.push(Instruction::SetupFinally(0));
        
        // 内层的 try-except（如果有）
        if !try_stmt.handlers.is_empty() {
            self.compile_try_except(&try_stmt, bytecode)?;
        } else {
            // 只有 try-finally
            for stmt in &try_stmt.body {
                self.compile_stmt(stmt, bytecode)?;
            }
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
    } else {
        // 没有 finally 块，只有 try-except
        self.compile_try_except(&try_stmt, bytecode)?;
    }
    
    Ok(())
}
```

### 4. 实现 VM 指令

```rust
// src/vm.rs
Instruction::SetupFinally(handler_offset) => {
    self.blocks.push(Block {
        block_type: BlockType::Finally {
            handler_offset: *handler_offset,
        },
        stack_size: self.stack.len(),
    });
    ip += 1;
}

Instruction::PopFinally => {
    self.blocks.pop();
    ip += 1;
}

Instruction::EndFinally => {
    // 检查栈顶是否是异常
    let value = self.stack.pop()
        .ok_or_else(|| Value::error(ExceptionType::RuntimeError, "Stack underflow"))?;
    
    if value.is_exception() {
        // 重新抛出异常
        return Err(value);
    }
    // 否则是 None，正常继续
    ip += 1;
}
```

### 5. 修改异常处理流程

```rust
// src/vm.rs
match self.execute_instruction(instruction, &mut ip, code, globals) {
    Ok(()) => { /* 继续 */ }
    Err(exception) => {
        // 从后往前查找块
        while let Some(block) = self.blocks.pop() {
            match block.block_type {
                BlockType::Finally { handler_offset } => {
                    // 找到 finally 块，必须先执行
                    self.stack.truncate(block.stack_size);
                    self.stack.push(exception.clone());
                    ip = handler_offset;
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

### 6. 添加列表版本号

```rust
// src/value.rs
pub struct ListValue {
    pub items: Vec<Value>,
    pub version: usize,  // 新增：版本号
}

impl Value {
    pub fn new_list(items: Vec<Value>) -> Value {
        Value::List(Rc::new(RefCell::new(ListValue {
            items,
            version: 0,
        })))
    }
}

// 更新所有使用 Vec<Value> 的地方为 ListValue
```

### 7. 在迭代器中存储版本号

```rust
// src/value.rs
pub enum IteratorState {
    Range { current: i32, stop: i32, step: i32 },
    List {
        list: Rc<RefCell<ListValue>>,
        index: usize,
        version: usize,  // 新增：创建时的版本号
    },
    DictKeys { keys: Vec<DictKey>, index: usize },
}
```

### 8. 修改 GetIter 指令

```rust
// src/vm.rs
Instruction::GetIter => {
    let obj = self.stack.pop()
        .ok_or_else(|| Value::error(ExceptionType::RuntimeError, "Stack underflow"))?;
    
    let iter = match obj {
        Value::List(list) => {
            let version = list.borrow().version;
            let iter_state = IteratorState::List {
                list: list.clone(),
                index: 0,
                version,  // 记录版本号
            };
            Value::Iterator(Rc::new(RefCell::new(iter_state)))
        }
        // ...
    };
    
    self.stack.push(iter);
    ip += 1;
}
```

### 9. 在 ForIter 中检查版本号

```rust
// src/vm.rs
Instruction::ForIter(jump_target) => {
    let iter = self.stack.last()
        .ok_or_else(|| Value::error(ExceptionType::RuntimeError, "Stack underflow"))?
        .clone();
    
    match iter {
        Value::Iterator(iter_state) => {
            let mut state = iter_state.borrow_mut();
            match &mut *state {
                IteratorState::List { list, index, version } => {
                    let list_ref = list.borrow();
                    
                    // 检查版本号
                    if list_ref.version != *version {
                        return Err(Value::error(
                            ExceptionType::IteratorError,
                            "list modified during iteration"
                        ));
                    }
                    
                    if *index < list_ref.items.len() {
                        let value = list_ref.items[*index].clone();
                        *index += 1;
                        drop(list_ref);
                        self.stack.push(value);
                        ip += 1;
                    } else {
                        ip = *jump_target;
                    }
                }
                // ...
            }
        }
        _ => return Err(Value::error(ExceptionType::TypeError, "Expected iterator")),
    }
}
```

### 10. 在列表修改时增加版本号

```rust
// src/vm.rs
Instruction::CallMethod(method_name, arg_count) => {
    // ...
    match method_name.as_str() {
        "append" => {
            match obj {
                Value::List(list) => {
                    // ...
                    list.borrow_mut().items.push(value);
                    list.borrow_mut().version += 1;  // 增加版本号
                    // ...
                }
            }
        }
        "pop" => {
            match obj {
                Value::List(list) => {
                    // ...
                    let value = list.borrow_mut().items.pop()
                        .ok_or_else(|| Value::error(ExceptionType::IndexError, "pop from empty list"))?;
                    list.borrow_mut().version += 1;  // 增加版本号
                    // ...
                }
            }
        }
        // ...
    }
}

Instruction::SetItem => {
    // ...
    match obj {
        Value::List(list) => {
            // ...
            list.borrow_mut().items[actual_idx as usize] = value;
            list.borrow_mut().version += 1;  // 增加版本号
        }
        // ...
    }
}
```

## 验收条件

- [ ] SetupFinally, PopFinally, EndFinally 指令实现
- [ ] finally 块编译实现
- [ ] finally 块执行流程实现
- [ ] try-except-finally 组合正确工作
- [ ] 列表版本号机制实现
- [ ] 迭代器版本号检查实现
- [ ] for 循环中修改列表抛出 IteratorError
- [ ] 所有测试通过

## 测试要求

### 单元测试

```rust
#[test]
fn test_try_finally() {
    let mut ctx = Context::new();
    ctx.eval(r#"
executed = []
try:
    executed.append("try")
finally:
    executed.append("finally")
    "#).unwrap();
    
    let executed = ctx.get("executed").unwrap();
    let list = executed.as_list().unwrap();
    assert_eq!(list.borrow().items.len(), 2);
}

#[test]
fn test_try_except_finally() {
    let mut ctx = Context::new();
    ctx.eval(r#"
executed = []
try:
    executed.append("try")
    x = 1 / 0
except ZeroDivisionError:
    executed.append("except")
finally:
    executed.append("finally")
    "#).unwrap();
    
    let executed = ctx.get("executed").unwrap();
    let list = executed.as_list().unwrap();
    assert_eq!(list.borrow().items.len(), 3);
}

#[test]
fn test_finally_with_exception() {
    let mut ctx = Context::new();
    let result = ctx.eval(r#"
executed = []
try:
    executed.append("try")
    raise ValueError("test")
finally:
    executed.append("finally")
    "#);
    
    assert!(result.is_err());
    let executed = ctx.get("executed").unwrap();
    let list = executed.as_list().unwrap();
    assert_eq!(list.borrow().items.len(), 2);
}

#[test]
fn test_iterator_modification_detection() {
    let mut ctx = Context::new();
    let result = ctx.eval(r#"
numbers = [1, 2, 3]
for n in numbers:
    numbers.append(10)
    "#);
    
    assert!(result.is_err());
    let exc = result.unwrap_err().as_exception().unwrap();
    assert_eq!(exc.exception_type, ExceptionType::IteratorError);
}

#[test]
fn test_iterator_modification_caught() {
    let mut ctx = Context::new();
    ctx.eval(r#"
caught = False
numbers = [1, 2, 3]
try:
    for n in numbers:
        numbers.append(10)
except IteratorError:
    caught = True
    "#).unwrap();
    
    let caught = ctx.get("caught").unwrap();
    assert_eq!(caught.as_bool(), Some(true));
}
```

## 注意事项

1. finally 块必须在任何情况下都执行
2. 列表版本号机制会影响所有列表操作
3. 需要更新所有访问列表的代码
4. 确保嵌套的 try-finally 正确工作

## 后续任务

完成后异常系统基本完成，可以考虑：
- 优化异常性能
- 添加 traceback 支持
- 实现更多内置异常类型
