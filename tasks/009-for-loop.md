# 009: for 循环和迭代器

**状态**: TODO  
**优先级**: P0

## 任务概述

支持 for 循环和迭代器，让程序可以遍历列表、字典和 range。

## 可运行功能

完成后用户可以：
```rust
let mut ctx = Context::new();

// for...in range
ctx.eval(r#"
sum = 0
for i in range(10):
    sum = sum + i
"#).unwrap();
let sum = ctx.get("sum").unwrap();
assert_eq!(sum.as_int(), Some(45));

// for...in list
ctx.eval(r#"
numbers = [1, 2, 3, 4, 5]
total = 0
for n in numbers:
    total = total + n
"#).unwrap();
let total = ctx.get("total").unwrap();
assert_eq!(total.as_int(), Some(15));

// for...in dict (遍历键)
ctx.eval(r#"
person = {"name": "Alice", "age": 30}
keys = []
for key in person:
    keys.append(key)
"#).unwrap();

// for...in dict.keys()
ctx.eval(r#"
for key in person.keys():
    print(key)
"#).unwrap();

// for...in dict.values()
ctx.eval(r#"
for value in person.values():
    print(value)
"#).unwrap();

// for...in dict.items()
ctx.eval(r#"
for key, value in person.items():
    print(key, value)
"#).unwrap();

// range() 的不同形式
ctx.eval("for i in range(5): pass").unwrap();           // 0..5
ctx.eval("for i in range(1, 5): pass").unwrap();        // 1..5
ctx.eval("for i in range(0, 10, 2): pass").unwrap();    // 0,2,4,6,8
```

**现在可以方便地遍历数据了！**

## 依赖任务

- 008: 列表和字典

## 需要实现的模块

### 1. 迭代器系统
- Iterator trait/接口
- ListIterator
- DictIterator (keys)
- RangeIterator

### 2. 扩展编译器
- for...in 语句编译
- 元组解包（for key, value in items）

### 3. 扩展 VM
- GetIter 指令（获取迭代器）
- ForIter 指令（迭代下一个元素）
- 迭代器状态管理

### 4. range() 函数
- range(stop)
- range(start, stop)
- range(start, stop, step)

### 5. 字典迭代方法
- dict.keys() 返回可迭代对象
- dict.values() 返回可迭代对象
- dict.items() 返回可迭代对象

## 验收条件

- [ ] for i in range(n) 正常工作
- [ ] for item in list 正常工作
- [ ] for key in dict 正常工作
- [ ] for key in dict.keys() 正常工作
- [ ] for value in dict.values() 正常工作
- [ ] for key, value in dict.items() 正常工作
- [ ] range(stop), range(start, stop), range(start, stop, step) 都支持
- [ ] 嵌套 for 循环

## 测试要求

### 单元测试
- [ ] range() 生成正确的序列
- [ ] 列表迭代器正确性
- [ ] 字典迭代器正确性
- [ ] 元组解包

### 集成测试
- [ ] 遍历列表求和
- [ ] 遍历字典打印键值
- [ ] 嵌套循环
- [ ] for 循环中修改列表（注意迭代器失效）

## 增量实现步骤

### Step 1: range() 函数
- 实现 range() 内置函数
- 返回 RangeIterator 对象

### Step 2: 迭代器 Value 类型
- 添加 Iterator(Box<dyn Iterator>) 类型
- 实现迭代器接口

### Step 3: for 循环编译
- 编译器支持 For 语句
- 生成 GetIter, ForIter 指令

### Step 4: 列表迭代
- 实现 ListIterator
- 列表支持 __iter__

### Step 5: 字典迭代
- 实现 DictIterator
- 字典支持 __iter__
- 实现 keys(), values(), items()

### Step 6: 元组解包
- 支持 for key, value in items

## 后续任务

完成后可以开始：
- 010: break 和 continue
