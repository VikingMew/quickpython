# 003: 函数定义和调用

**状态**: TODO  
**优先级**: P0

## 任务概述

支持函数定义和调用，这是编程语言的核心功能。

## 可运行功能

完成后用户可以：
```rust
let mut ctx = Context::new();

// 定义函数
ctx.eval(r#"
def add(a, b):
    return a + b
"#).unwrap();

// 调用函数
let result = ctx.eval("add(1, 2)").unwrap();
assert_eq!(result.as_int(), Some(3));

// 递归函数
ctx.eval(r#"
def factorial(n):
    if n <= 1:
        return 1
    return n * factorial(n - 1)
"#).unwrap();

let result = ctx.eval("factorial(5)").unwrap();
assert_eq!(result.as_int(), Some(120));
```

**现在是真正的编程语言了！**

## 依赖任务

- 002: 变量赋值和读取

## 需要实现的模块

### 1. 函数对象
- 函数 Value 类型
- 参数和局部变量

### 2. 扩展编译器
- 函数定义编译
- 函数调用编译
- return 语句

### 3. 扩展 VM
- 调用栈（Frame）
- Call, Return 指令
- 局部变量管理

### 4. 控制流（基础）
- if/else 语句（用于递归）
- 比较运算符

## 验收条件

- [ ] 可以定义函数
- [ ] 可以调用函数
- [ ] 参数传递正确
- [ ] return 正常工作
- [ ] 支持递归
- [ ] 支持 if/else
- [ ] 局部变量不污染全局

## 测试要求

```rust
#[test]
fn test_function() {
    let mut ctx = Context::new();
    ctx.eval("def add(a, b): return a + b").unwrap();
    let result = ctx.eval("add(1, 2)").unwrap();
    assert_eq!(result.as_int(), Some(3));
}

#[test]
fn test_factorial() {
    let mut ctx = Context::new();
    ctx.eval(r#"
def factorial(n):
    if n <= 1:
        return 1
    return n * factorial(n - 1)
    "#).unwrap();
    
    let result = ctx.eval("factorial(5)").unwrap();
    assert_eq!(result.as_int(), Some(120));
}
```

## 后续任务

完成后可以开始：
- 004: 完整控制流（while 循环）
- 005: CLI 工具
