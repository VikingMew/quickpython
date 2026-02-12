# 004: 控制流 (if/while)

**状态**: TODO  
**优先级**: P0

## 任务概述

支持 if/else 和 while 循环，让程序可以做决策和重复操作。

## 可运行功能

完成后用户可以：
```rust
let mut ctx = Context::new();

// if/else
ctx.eval(r#"
x = 10
if x > 5:
    result = "big"
else:
    result = "small"
"#).unwrap();

let result = ctx.get("result").unwrap();
assert_eq!(result.as_string(), Some("big"));

// while 循环
ctx.eval(r#"
i = 0
sum = 0
while i < 10:
    sum = sum + i
    i = i + 1
"#).unwrap();

let sum = ctx.get("sum").unwrap();
assert_eq!(sum.as_int(), Some(45));

// 完整的 Fibonacci
ctx.eval(r#"
def fibonacci(n):
    if n <= 1:
        return n
    return fibonacci(n - 1) + fibonacci(n - 2)
"#).unwrap();

let result = ctx.eval("fibonacci(10)").unwrap();
assert_eq!(result.as_int(), Some(55));
```

**现在是图灵完备的语言了！**

## 依赖任务

- 003: 函数定义和调用

## 需要实现的模块

### 1. 扩展编译器
- if/elif/else 语句编译
- while 循环编译
- break/continue（可选）

### 2. 扩展 VM
- Jump, JumpIfTrue, JumpIfFalse 指令
- 条件跳转逻辑

### 3. 比较运算
- ==, !=, <, <=, >, >= 运算符

## 验收条件

- [ ] if/else 正常工作
- [ ] while 循环正常工作
- [ ] 比较运算符正确
- [ ] 可以实现 Fibonacci
- [ ] 可以实现阶乘
- [ ] 嵌套控制流正常

## 测试要求

```rust
#[test]
fn test_if_else() {
    let mut ctx = Context::new();
    ctx.eval("x = 10; if x > 5: y = 1; else: y = 0").unwrap();
    let y = ctx.get("y").unwrap();
    assert_eq!(y.as_int(), Some(1));
}

#[test]
fn test_while() {
    let mut ctx = Context::new();
    ctx.eval("i = 0; sum = 0; while i < 5: sum = sum + i; i = i + 1").unwrap();
    let sum = ctx.get("sum").unwrap();
    assert_eq!(sum.as_int(), Some(10));
}

#[test]
fn test_fibonacci() {
    let mut ctx = Context::new();
    ctx.eval("def fib(n): if n <= 1: return n; return fib(n-1) + fib(n-2)").unwrap();
    let result = ctx.eval("fib(10)").unwrap();
    assert_eq!(result.as_int(), Some(55));
}
```

## 后续任务

完成后可以开始：
- 005: CLI 工具
- 006: 列表和字典
