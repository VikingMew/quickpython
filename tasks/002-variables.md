# 002: 变量赋值和读取

**状态**: DONE  
**优先级**: P0

## 任务概述

支持变量的赋值和读取，这是第一个有状态的功能。

## 可运行功能

完成后用户可以：
```rust
let mut ctx = Context::new();

// 赋值
ctx.eval("x = 42").unwrap();

// 读取
let result = ctx.eval("x").unwrap();
assert_eq!(result.as_int(), Some(42));

// 使用变量计算
ctx.eval("y = x + 10").unwrap();
let y = ctx.eval("y").unwrap();
assert_eq!(y.as_int(), Some(52));

// Rust API
ctx.set("z", Value::new_int(100)).unwrap();
let z = ctx.get("z").unwrap();
```

**现在有状态了！**

## 依赖任务

- 001: 执行简单表达式

## 需要实现的模块

### 1. 扩展 Value 系统
- 添加字符串类型（用于变量名）
- 引用计数机制

### 2. Atom 系统
- 字符串原子化（变量名）

### 3. Context 状态
- 全局变量表
- get/set API

### 4. 扩展编译器
- 支持赋值语句
- 支持变量引用

### 5. 扩展 VM
- GetGlobal, SetGlobal 指令

## 验收条件

- [x] 可以执行 `x = 1`
- [x] 可以读取 `x` 返回 1
- [x] 可以执行 `y = x + 1`
- [x] 多次赋值正确
- [x] 未定义变量报错
- [x] get/set API 正常工作

## 测试要求

```rust
#[test]
fn test_variable() {
    let mut ctx = Context::new();
    ctx.eval("x = 42").unwrap();
    let x = ctx.eval("x").unwrap();
    assert_eq!(x.as_int(), Some(42));
}

#[test]
fn test_variable_expr() {
    let mut ctx = Context::new();
    ctx.eval("x = 10").unwrap();
    ctx.eval("y = x * 2").unwrap();
    let y = ctx.eval("y").unwrap();
    assert_eq!(y.as_int(), Some(20));
}
```

## 后续任务

完成后可以开始：
- 003: 函数定义和调用
