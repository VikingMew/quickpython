# 011: 异常类型和基础结构

**状态**: DONE  
**优先级**: P0  
**依赖**: 009-for-loop

## 任务概述

实现异常系统的基础类型和数据结构，为后续的异常处理功能打下基础。

## 目标

完成后用户可以：
```rust
// 创建异常对象
let exc = Value::Exception(ExceptionValue {
    exception_type: ExceptionType::ValueError,
    message: "invalid value".to_string(),
    traceback: None,
});

// 检查异常类型
if let Value::Exception(exc) = value {
    match exc.exception_type {
        ExceptionType::ValueError => { /* ... */ }
        _ => { /* ... */ }
    }
}
```

## 需要实现的内容

### 1. 定义异常类型枚举

```rust
// src/value.rs
#[derive(Debug, Clone, PartialEq)]
pub enum ExceptionType {
    Exception,          // 基础异常
    RuntimeError,       // 运行时错误
    IndexError,         // 索引越界
    KeyError,           // 键不存在
    ValueError,         // 值错误
    TypeError,          // 类型错误
    ZeroDivisionError,  // 除零错误
    IteratorError,      // 迭代器错误（自定义）
}
```

### 2. 定义异常值结构

```rust
// src/value.rs
#[derive(Debug, Clone)]
pub struct ExceptionValue {
    pub exception_type: ExceptionType,
    pub message: String,
    pub traceback: Option<Vec<TracebackFrame>>, // 暂时为 None
}

#[derive(Debug, Clone)]
pub struct TracebackFrame {
    pub function_name: String,
    pub line_number: usize,
}
```

### 3. 扩展 Value 枚举

```rust
// src/value.rs
#[derive(Debug, Clone)]
pub enum Value {
    Int(i32),
    Float(f64),
    Bool(bool),
    None,
    String(String),
    List(Rc<RefCell<Vec<Value>>>),
    Dict(Rc<RefCell<HashMap<DictKey, Value>>>),
    Iterator(Rc<RefCell<IteratorState>>),
    Function(Function),
    Exception(ExceptionValue),  // 新增
}
```

### 4. 更新 PartialEq 实现

```rust
impl PartialEq for Value {
    fn eq(&self, other: &Self) -> bool {
        match (self, other) {
            // ... 现有匹配
            (Value::Exception(a), Value::Exception(b)) => {
                a.exception_type == b.exception_type && a.message == b.message
            }
            _ => false,
        }
    }
}
```

### 5. 添加便捷方法

```rust
impl Value {
    // 创建异常的便捷方法
    pub fn error(exception_type: ExceptionType, message: impl Into<String>) -> Value {
        Value::Exception(ExceptionValue {
            exception_type,
            message: message.into(),
            traceback: None,
        })
    }
    
    // 检查是否是异常
    pub fn is_exception(&self) -> bool {
        matches!(self, Value::Exception(_))
    }
    
    // 获取异常对象
    pub fn as_exception(&self) -> Option<&ExceptionValue> {
        match self {
            Value::Exception(exc) => Some(exc),
            _ => None,
        }
    }
}
```

### 6. 更新 print 函数

```rust
// src/vm.rs
fn print_value(value: &Value) {
    match value {
        // ... 现有匹配
        Value::Exception(exc) => {
            println!("{:?}: {}", exc.exception_type, exc.message);
        }
    }
}

fn print_value_inline(value: &Value) {
    match value {
        // ... 现有匹配
        Value::Exception(exc) => {
            print!("{:?}: {}", exc.exception_type, exc.message);
        }
    }
}
```

## 验收条件

- [x] ExceptionType 枚举定义完成
- [x] ExceptionValue 结构定义完成
- [x] Value 枚举添加 Exception 变体
- [x] PartialEq 实现更新
- [x] 便捷方法实现
- [x] print 函数支持异常显示
- [x] 所有现有测试通过

## 测试要求

### 单元测试

```rust
#[test]
fn test_exception_creation() {
    let exc = Value::error(ExceptionType::ValueError, "test error");
    assert!(exc.is_exception());
    
    let exc_value = exc.as_exception().unwrap();
    assert_eq!(exc_value.exception_type, ExceptionType::ValueError);
    assert_eq!(exc_value.message, "test error");
}

#[test]
fn test_exception_equality() {
    let exc1 = Value::error(ExceptionType::ValueError, "test");
    let exc2 = Value::error(ExceptionType::ValueError, "test");
    let exc3 = Value::error(ExceptionType::TypeError, "test");
    
    assert_eq!(exc1, exc2);
    assert_ne!(exc1, exc3);
}
```

## 注意事项

1. 这个任务只是定义数据结构，不涉及异常抛出和捕获
2. 确保所有现有测试仍然通过
3. 不要修改现有的错误处理逻辑（仍然使用 `Result<T, String>`）
4. traceback 暂时保持为 None，后续任务会实现

## 后续任务

完成后可以开始：
- 012: raise 语句和异常抛出
