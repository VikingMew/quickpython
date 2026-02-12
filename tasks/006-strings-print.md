# 006: 字符串和 print 函数

**状态**: TODO  
**优先级**: P1

## 任务概述

支持字符串类型和 print 函数，让程序可以输出信息。

## 可运行功能

完成后用户可以：
```rust
let mut ctx = Context::new();

// 字符串字面量
let result = ctx.eval("\"hello\"").unwrap();
assert_eq!(result.as_string(), Some("hello"));

// 字符串拼接
let result = ctx.eval("\"hello\" + \" \" + \"world\"").unwrap();
assert_eq!(result.as_string(), Some("hello world"));

// print 函数
ctx.eval("print(\"Hello, World!\")").unwrap();
// 输出: Hello, World!

ctx.eval("print(42)").unwrap();
// 输出: 42

// 多参数 print
ctx.eval("print(\"x =\", 10)").unwrap();
// 输出: x = 10

// f-string (基础版)
ctx.eval(r#"
x = 42
print(f"The answer is {x}")
"#).unwrap();
// 输出: The answer is 42
```

**现在可以输出调试信息了！**

## 依赖任务

- 003: 函数定义和调用

## 需要实现的模块

### 1. 扩展 Value 系统
- 添加 String 类型
- 字符串拼接操作

### 2. 扩展编译器
- 字符串字面量解析
- 字符串拼接编译
- f-string 解析和编译

### 3. 内置函数系统
- 注册内置函数的机制
- print 函数实现
- 支持多参数

### 4. 扩展 VM
- 字符串拼接指令
- 调用内置函数

## 验收条件

- [ ] 支持字符串字面量
- [ ] 支持字符串拼接 (+)
- [ ] print() 可以输出字符串
- [ ] print() 可以输出整数
- [ ] print() 支持多个参数
- [ ] 基础 f-string 支持 (f"text {var}")
- [ ] 字符串可以作为函数参数和返回值

## 测试要求

### 单元测试
- [ ] 字符串字面量创建和读取
- [ ] 字符串拼接正确性
- [ ] print 输出到 stdout
- [ ] f-string 变量替换

### 集成测试
- [ ] 运行包含 print 的 .py 文件
- [ ] 编译包含字符串的代码
- [ ] f-string 在函数中使用

## 增量实现步骤

### Step 1: 字符串 Value 类型
- 在 Value enum 添加 String(String)
- 实现 as_string() 方法
- 更新序列化器

### Step 2: 字符串字面量
- 编译器支持 Constant::Str
- VM 支持 PushString 指令

### Step 3: 字符串拼接
- 添加 Concat 指令
- VM 实现字符串拼接

### Step 4: print 函数
- 设计内置函数注册机制
- 实现 print 函数
- 支持多参数

### Step 5: f-string
- 解析 f-string 语法
- 编译为字符串拼接操作

## 后续任务

完成后可以开始：
- 007: 列表和字典
- 008: 字符串方法 (split, join, etc.)
