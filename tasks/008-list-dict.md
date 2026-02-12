# 008: 列表和字典

**状态**: DONE  
**优先级**: P0

## 任务概述

支持列表和字典两种复合数据类型，让程序可以处理集合数据。

## 可运行功能

完成后用户可以：
```rust
let mut ctx = Context::new();

// 列表字面量
ctx.eval("numbers = [1, 2, 3, 4, 5]").unwrap();
let numbers = ctx.get("numbers").unwrap();
assert_eq!(numbers.as_list().unwrap().len(), 5);

// 列表索引访问
let result = ctx.eval("numbers[0]").unwrap();
assert_eq!(result.as_int(), Some(1));

// 列表索引赋值
ctx.eval("numbers[0] = 10").unwrap();
let result = ctx.eval("numbers[0]").unwrap();
assert_eq!(result.as_int(), Some(10));

// 列表方法
ctx.eval("numbers.append(6)").unwrap();
ctx.eval("x = numbers.pop()").unwrap();
let x = ctx.get("x").unwrap();
assert_eq!(x.as_int(), Some(6));

// 字典字面量
ctx.eval(r#"person = {"name": "Alice", "age": 30}"#).unwrap();

// 字典访问（字符串键）
let result = ctx.eval(r#"person["name"]"#).unwrap();
assert_eq!(result.as_string(), Some("Alice"));

// 字典访问（整数键）
ctx.eval("scores = {1: 100, 2: 95, 3: 88}").unwrap();
let result = ctx.eval("scores[1]").unwrap();
assert_eq!(result.as_int(), Some(100));

// 字典赋值
ctx.eval(r#"person["age"] = 31"#).unwrap();
ctx.eval("scores[4] = 92").unwrap();

// 字典方法
ctx.eval("keys = person.keys()").unwrap();

// len() 函数
let result = ctx.eval("len(numbers)").unwrap();
assert_eq!(result.as_int(), Some(5));

let result = ctx.eval("len(person)").unwrap();
assert_eq!(result.as_int(), Some(2));
```

**现在可以处理复杂数据结构了！**

## 依赖任务

- 006: 字符串和 print

## 需要实现的模块

### 1. 扩展 Value 系统
- 添加 List(Vec<Value>) 类型
- 添加 Dict 类型，支持 String 和 Int 作为键
  - 内部使用 HashMap<DictKey, Value>
  - DictKey enum { String(String), Int(i32) }
- 实现 as_list(), as_dict() 方法

### 2. 扩展编译器
- 列表字面量 `[1, 2, 3]`
- 字典字面量 `{"key": "value"}`
- 下标访问 `list[0]`, `dict["key"]`
- 下标赋值 `list[0] = x`
- 方法调用 `list.append(x)`

### 3. 扩展 VM
- BuildList 指令
- BuildDict 指令
- GetItem 指令（索引/键访问）
- SetItem 指令（索引/键赋值）
- CallMethod 指令（方法调用）

### 4. 内置函数和方法
- len() 函数
- list.append(x)
- list.pop()
- dict.keys()

### 5. 序列化
- 列表和字典序列化支持

## 验收条件

- [x] 支持列表字面量
- [x] 列表索引读取和赋值
- [x] 列表 append/pop 方法
- [x] 支持字典字面量
- [x] 字典支持字符串键
- [x] 字典支持整数键
- [x] 字典键访问和赋值
- [x] 字典 keys() 方法
- [x] len() 函数支持列表和字典
- [x] 嵌套数据结构（列表的列表，字典的字典）
- [x] 使用不支持的类型作为键时报错（如 bool, list）

## 测试要求

### 单元测试
- [x] 列表创建和访问
- [x] 列表方法正确性
- [x] 字典创建和访问（字符串键）
- [x] 字典创建和访问（整数键）
- [x] 字典方法正确性
- [x] len() 函数
- [x] 嵌套结构
- [x] 不支持的键类型报错

### 集成测试
- [x] 运行包含列表的程序
- [x] 运行包含字典的程序
- [x] 列表和字典混合使用

## 增量实现步骤

### Step 1: List Value 类型
- [x] 在 Value enum 添加 List(Rc<RefCell<Vec<Value>>>)
- [x] 实现 as_list() 方法

### Step 2: 列表字面量
- [x] 编译器支持 List 表达式
- [x] VM 支持 BuildList 指令

### Step 3: 列表索引
- [x] 编译器支持 Subscript 表达式
- [x] VM 支持 GetItem/SetItem 指令

### Step 4: 列表方法
- [x] 实现方法调用机制
- [x] 实现 append, pop 等方法

### Step 5: Dict Value 类型
- [x] 定义 DictKey enum { String(String), Int(i32) }
- [x] 在 Value enum 添加 Dict(Rc<RefCell<HashMap<DictKey, Value>>>)
- [x] 实现字典字面量和访问
- [x] 支持字符串和整数键
- [x] 不支持的键类型报错

### Step 6: 字典方法
- [x] 实现 keys() 方法

### Step 7: len() 函数
- [x] 实现 len() 内置函数

## 后续任务

完成后可以开始：
- 009: for 循环和迭代器
