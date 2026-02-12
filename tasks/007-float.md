# 007: 浮点数支持

**状态**: TODO  
**优先级**: P0

## 任务概述

支持浮点数类型和浮点运算，让程序可以处理小数。

## 可运行功能

完成后用户可以：
```rust
let mut ctx = Context::new();

// 浮点数字面量
let result = ctx.eval("3.14").unwrap();
assert_eq!(result.as_float(), Some(3.14));

// 浮点运算
let result = ctx.eval("3.14 * 2.0").unwrap();
assert_eq!(result.as_float(), Some(6.28));

// 整数和浮点数混合运算
let result = ctx.eval("10 / 3.0").unwrap();
assert_eq!(result.as_float(), Some(3.333...));

// 类型转换
ctx.eval("x = 3.14").unwrap();
ctx.eval("y = int(x)").unwrap();
let y = ctx.get("y").unwrap();
assert_eq!(y.as_int(), Some(3));

ctx.eval("z = float(42)").unwrap();
let z = ctx.get("z").unwrap();
assert_eq!(z.as_float(), Some(42.0));
```

**现在可以做科学计算了！**

## 依赖任务

- 001: 执行简单表达式

## 需要实现的模块

### 1. 扩展 Value 系统
- 添加 Float(f64) 类型
- 实现 as_float() 方法

### 2. 扩展编译器
- 浮点数字面量解析
- 混合类型运算处理

### 3. 扩展 VM
- PushFloat 指令
- 浮点运算指令
- 类型提升（int -> float）

### 4. 内置函数
- int() 类型转换
- float() 类型转换

### 5. 序列化
- 浮点数序列化支持

## 验收条件

- [ ] 支持浮点数字面量
- [ ] 浮点数四则运算
- [ ] 整数和浮点数混合运算（自动类型提升）
- [ ] int() 转换函数
- [ ] float() 转换函数
- [ ] 浮点数可以序列化

## 测试要求

### 单元测试
- [ ] 浮点数字面量解析
- [ ] 浮点数运算精度
- [ ] 类型提升正确性
- [ ] int()/float() 转换

### 集成测试
- [ ] 运行包含浮点数的 .py 文件
- [ ] 编译包含浮点数的代码
- [ ] 混合运算结果正确

## 增量实现步骤

### Step 1: Float Value 类型
- 在 Value enum 添加 Float(f64)
- 实现 as_float() 方法

### Step 2: 浮点数字面量
- 编译器支持 Constant::Float
- VM 支持 PushFloat 指令

### Step 3: 浮点运算
- 修改算术指令支持 Float
- 实现类型提升逻辑

### Step 4: 类型转换函数
- 实现 int() 内置函数
- 实现 float() 内置函数

### Step 5: 序列化
- 更新序列化器支持 Float

## 后续任务

完成后可以开始：
- 008: 列表和字典
