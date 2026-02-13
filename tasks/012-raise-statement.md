# 012: raise 语句和异常抛出

**状态**: DONE  
**优先级**: P0  
**依赖**: 011-exception-types

## 任务概述

实现 raise 语句和异常抛出机制，将现有的字符串错误转换为异常对象。

## 目标

完成后用户可以：
```python
# 抛出异常
raise ValueError("invalid value")
raise IndexError("list index out of range")

# 现有代码自动抛出异常
x = [1, 2, 3]
y = x[10]  # 抛出 IndexError

z = 1 / 0  # 抛出 ZeroDivisionError
```

## 需要实现的内容

### 1. 添加字节码指令

```rust
// src/bytecode.rs
pub enum Instruction {
    // ... 现有指令
    Raise,  // 抛出异常，栈顶是异常对象
    MakeException(ExceptionType),  // 创建异常，栈顶是消息字符串
}
```

### 2. 实现 raise 语句编译

```rust
// src/compiler.rs
ast::Stmt::Raise(raise) => {
    if let Some(exc) = &raise.exc {
        // 检查是否是简单的异常调用
        if let ast::Expr::Call(call) = &**exc {
            if let ast::Expr::Name(name) = &*call.func {
                let exc_name = name.id.to_string();
                let exc_type = match exc_name.as_str() {
                    "ValueError" => ExceptionType::ValueError,
                    "TypeError" => ExceptionType::TypeError,
                    "IndexError" => ExceptionType::IndexError,
                    "KeyError" => ExceptionType::KeyError,
                    "ZeroDivisionError" => ExceptionType::ZeroDivisionError,
                    "RuntimeError" => ExceptionType::RuntimeError,
                    "IteratorError" => ExceptionType::IteratorError,
                    "Exception" => ExceptionType::Exception,
                    _ => return Err(format!("Unknown exception type: {}", exc_name)),
                };
                
                // 编译消息参数
                if call.args.len() != 1 {
                    return Err("Exception requires exactly one argument".to_string());
                }
                self.compile_expr(&call.args[0], bytecode)?;
                
                // 创建异常对象
                bytecode.push(Instruction::MakeException(exc_type));
                bytecode.push(Instruction::Raise);
                return Ok(());
            }
        }
        
        // 其他情况：编译表达式，应该得到一个异常对象
        self.compile_expr(exc, bytecode)?;
        bytecode.push(Instruction::Raise);
    } else {
        // bare raise（重新抛出当前异常）
        return Err("bare raise not supported yet".to_string());
    }
    Ok(())
}
```

### 3. 实现 VM 指令

```rust
// src/vm.rs
Instruction::MakeException(exc_type) => {
    let message = self.stack.pop()
        .ok_or_else(|| "Stack underflow".to_string())?;
    
    let msg_str = match message {
        Value::String(s) => s,
        _ => return Err("Exception message must be a string".to_string()),
    };
    
    let exception = Value::Exception(ExceptionValue {
        exception_type: exc_type.clone(),
        message: msg_str,
        traceback: None,
    });
    
    self.stack.push(exception);
    ip += 1;
}

Instruction::Raise => {
    let exception = self.stack.pop()
        .ok_or_else(|| "Stack underflow".to_string())?;
    
    if !exception.is_exception() {
        return Err("raise requires an exception object".to_string());
    }
    
    // 返回异常（作为错误）
    return Err(exception);
}
```

### 4. 修改现有错误为异常

将现有的字符串错误转换为异常对象：

```rust
// src/vm.rs

// 除零错误
Instruction::Div => {
    // ...
    match (a, b) {
        (Value::Int(a), Value::Int(b)) => {
            if b == 0 {
                return Err(Value::error(
                    ExceptionType::ZeroDivisionError,
                    "division by zero"
                ));
            }
            // ...
        }
        // ...
    }
}

// 索引错误
Instruction::GetItem => {
    // ...
    match obj {
        Value::List(list) => {
            // ...
            if actual_idx < 0 || actual_idx >= len {
                return Err(Value::error(
                    ExceptionType::IndexError,
                    "list index out of range"
                ));
            }
            // ...
        }
        // ...
    }
}

// 键错误
Instruction::GetItem => {
    // ...
    match obj {
        Value::Dict(dict) => {
            // ...
            let value = dict_ref.get(&dict_key)
                .ok_or_else(|| Value::error(
                    ExceptionType::KeyError,
                    format!("key not found: {:?}", dict_key)
                ))?
                .clone();
            // ...
        }
        // ...
    }
}

// 类型错误
Instruction::Add => {
    // ...
    _ => return Err(Value::error(
        ExceptionType::TypeError,
        "unsupported operand types for +"
    )),
}
```

### 5. 更新 Result 类型

```rust
// src/vm.rs
pub type VMResult<T> = Result<T, Value>;

impl VM {
    pub fn execute(
        &mut self,
        bytecode: &ByteCode,
        globals: &mut HashMap<String, Value>,
    ) -> VMResult<Value> {
        // ...
    }
}
```

### 6. 更新错误显示

```rust
// src/main.rs
match ctx.eval(&source) {
    Ok(result) => {
        if let Some(i) = result.as_int() {
            println!("{}", i);
        }
    }
    Err(e) => {
        // e 现在是 Value 类型
        if let Some(exc) = e.as_exception() {
            eprintln!("{:?}: {}", exc.exception_type, exc.message);
        } else {
            eprintln!("Error: {:?}", e);
        }
        process::exit(1);
    }
}
```

## 验收条件

- [ ] Raise 和 MakeException 指令实现
- [ ] raise 语句编译实现
- [ ] VM 指令执行实现
- [ ] 现有错误转换为异常对象
- [ ] Result 类型更新为 Result<T, Value>
- [ ] 错误显示更新
- [ ] 所有现有测试通过（可能需要更新测试）

## 测试要求

### 单元测试

```rust
#[test]
fn test_raise_value_error() {
    let mut ctx = Context::new();
    let result = ctx.eval(r#"raise ValueError("test error")"#);
    assert!(result.is_err());
    
    let err = result.unwrap_err();
    let exc = err.as_exception().unwrap();
    assert_eq!(exc.exception_type, ExceptionType::ValueError);
    assert_eq!(exc.message, "test error");
}

#[test]
fn test_division_by_zero() {
    let mut ctx = Context::new();
    let result = ctx.eval("x = 1 / 0");
    assert!(result.is_err());
    
    let err = result.unwrap_err();
    let exc = err.as_exception().unwrap();
    assert_eq!(exc.exception_type, ExceptionType::ZeroDivisionError);
}

#[test]
fn test_index_error() {
    let mut ctx = Context::new();
    let result = ctx.eval(r#"
list = [1, 2, 3]
x = list[10]
    "#);
    assert!(result.is_err());
    
    let err = result.unwrap_err();
    let exc = err.as_exception().unwrap();
    assert_eq!(exc.exception_type, ExceptionType::IndexError);
}

#[test]
fn test_key_error() {
    let mut ctx = Context::new();
    let result = ctx.eval(r#"
dict = {"a": 1}
x = dict["b"]
    "#);
    assert!(result.is_err());
    
    let err = result.unwrap_err();
    let exc = err.as_exception().unwrap();
    assert_eq!(exc.exception_type, ExceptionType::KeyError);
}
```

## 注意事项

1. 这个任务会修改很多现有代码的错误处理
2. 需要仔细更新所有返回 `Err(String)` 的地方
3. 确保所有测试都更新以适应新的错误类型
4. 暂时不实现 try-except，异常会直接传播到顶层

## 后续任务

完成后可以开始：
- 013: try-except 语句
