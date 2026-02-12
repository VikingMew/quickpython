# 001: 执行简单表达式

**状态**: DONE  
**优先级**: P0

## 任务概述

实现最基础的功能：能够执行简单的算术表达式并返回结果。

## 可运行功能

完成后用户可以：
```rust
let mut ctx = Context::new();
let result = ctx.eval("1 + 2").unwrap();
assert_eq!(result.as_int(), Some(3));

ctx.eval("10 * 5").unwrap();  // 50
ctx.eval("100 / 4").unwrap(); // 25
ctx.eval("7 - 3").unwrap();   // 4
```

**这是第一个可见的功能！**

## 依赖任务

无

## 需要实现的模块

### 1. Value 系统
- 整数类型（i32）
- 类型检查和提取

### 2. 简单编译器
- 解析算术表达式（使用 rustpython_parser）
- 生成简单字节码

### 3. 简单 VM
- 栈式执行
- 支持 4 个指令：PushInt, Add, Sub, Mul, Div

### 4. Context API
- eval() 方法
- 返回 Value

## 验收条件

- [x] 可以执行 `1 + 2` 返回 3
- [x] 可以执行 `10 * 5` 返回 50
- [x] 可以执行 `100 / 4` 返回 25
- [x] 可以执行复合表达式 `(1 + 2) * 3` 返回 9
- [x] 错误的表达式返回错误

## 测试要求

```rust
#[test]
fn test_simple_add() {
    let mut ctx = Context::new();
    let result = ctx.eval("1 + 2").unwrap();
    assert_eq!(result.as_int(), Some(3));
}

#[test]
fn test_complex_expr() {
    let mut ctx = Context::new();
    let result = ctx.eval("(10 + 5) * 2").unwrap();
    assert_eq!(result.as_int(), Some(30));
}
```

## 后续任务

完成后可以开始：
- 002: 变量赋值和读取
