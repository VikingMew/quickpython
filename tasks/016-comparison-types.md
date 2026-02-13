# 016: 比较运算符类型支持

**状态**: DONE  
**优先级**: P0  
**依赖**: 无

## 完成总结

✅ **所有功能实现完成！**

- ✅ 所有 6 个比较指令支持多种类型
- ✅ 浮点数比较（包括 int/float 混合）
- ✅ 字符串比较（相等性和字典序）
- ✅ 布尔值比较
- ✅ None 比较
- ✅ 不同类型的错误处理
- ✅ 添加了 8 个新测试，全部通过
- ✅ 总测试数：83 passed; 0 failed

## 任务概述

扩展比较运算符（==, !=, <, >, <=, >=）支持浮点数、字符串等多种类型的比较，移除当前只支持整数的限制。

## 可运行功能

完成后用户可以：

```python
# 浮点数比较
x = 3.14
y = 2.71
if x > y:
    print("x is greater")  # 输出: x is greater

# 字符串比较
name = "Alice"
if name == "Alice":
    print("Welcome Alice")  # 输出: Welcome Alice

if "apple" < "banana":
    print("apple comes first")  # 输出: apple comes first

# 混合类型比较（int 和 float）
score = 87.5
if score >= 90:
    print("Grade A")
elif score >= 80:
    print("Grade B")  # 输出: Grade B

# 布尔值比较
if True == True:
    print("Boolean equality works")

# None 比较
x = None
if x == None:
    print("x is None")
```

**现在可以进行各种类型的比较了！**

## 依赖任务

无

## 当前问题

当前所有比较指令都使用 `pop_int()` 函数，只接受整数：

```rust
// src/vm.rs - 当前实现
Instruction::Eq => {
    let b = self.pop_int()?;  // ❌ 只接受整数
    let a = self.pop_int()?;  // ❌ 只接受整数
    self.stack.push(Value::Bool(a == b));
    *ip += 1;
}
```

这导致以下代码报错：
- `3.14 > 3.0` → TypeError: Expected integer
- `"hello" == "world"` → TypeError: Expected integer
- `87.5 >= 90` → TypeError: Expected integer

## 需要实现的模块

### 1. 修改比较指令

需要修改 6 个比较指令的实现：
- `Instruction::Eq` - 相等
- `Instruction::Ne` - 不等
- `Instruction::Lt` - 小于
- `Instruction::Le` - 小于等于
- `Instruction::Gt` - 大于
- `Instruction::Ge` - 大于等于

### 2. 类型匹配逻辑

每个指令都需要实现模式匹配，支持：
- int 与 int
- float 与 float
- int 与 float（自动类型提升）
- string 与 string
- bool 与 bool
- None 与 None

### 3. 错误处理

不支持的类型组合应：
- 相等性比较（==, !=）：返回 False
- 顺序比较（<, >, <=, >=）：抛出 TypeError

## 验收条件

### 相等性比较（==, !=）

- [x] int == int
- [x] float == float
- [x] int == float（自动提升）
- [x] string == string
- [x] bool == bool
- [x] None == None
- [x] 不同类型返回 False

### 顺序比较（<, >, <=, >=）

- [x] int < int
- [x] float < float
- [x] int < float（自动提升）
- [x] string < string（字典序）
- [x] 不支持的类型组合抛出 TypeError

## 测试要求

### 单元测试

```rust
// 浮点数比较
#[test]
fn test_float_comparison() {
    let mut ctx = Context::new();
    ctx.eval("x = 3.14").unwrap();
    ctx.eval("y = 2.71").unwrap();
    
    let result = ctx.eval("x > y").unwrap();
    assert_eq!(result.as_bool(), Some(true));
    
    let result = ctx.eval("x == y").unwrap();
    assert_eq!(result.as_bool(), Some(false));
}

// 混合类型比较（int 和 float）
#[test]
fn test_mixed_int_float_comparison() {
    let mut ctx = Context::new();
    ctx.eval("score = 87.5").unwrap();
    
    let result = ctx.eval("score >= 90").unwrap();
    assert_eq!(result.as_bool(), Some(false));
    
    let result = ctx.eval("score >= 80").unwrap();
    assert_eq!(result.as_bool(), Some(true));
    
    let result = ctx.eval("10 == 10.0").unwrap();
    assert_eq!(result.as_bool(), Some(true));
}

// 字符串比较
#[test]
fn test_string_comparison() {
    let mut ctx = Context::new();
    
    // 相等性
    let result = ctx.eval(r#""hello" == "hello""#).unwrap();
    assert_eq!(result.as_bool(), Some(true));
    
    let result = ctx.eval(r#""hello" == "world""#).unwrap();
    assert_eq!(result.as_bool(), Some(false));
    
    // 字典序
    let result = ctx.eval(r#""apple" < "banana""#).unwrap();
    assert_eq!(result.as_bool(), Some(true));
}

// 布尔值比较
#[test]
fn test_bool_comparison() {
    let mut ctx = Context::new();
    
    let result = ctx.eval("True == True").unwrap();
    assert_eq!(result.as_bool(), Some(true));
    
    let result = ctx.eval("True == False").unwrap();
    assert_eq!(result.as_bool(), Some(false));
}

// None 比较
#[test]
fn test_none_comparison() {
    let mut ctx = Context::new();
    ctx.eval("x = None").unwrap();
    
    let result = ctx.eval("x == None").unwrap();
    assert_eq!(result.as_bool(), Some(true));
}

// 不同类型比较
#[test]
fn test_different_types() {
    let mut ctx = Context::new();
    
    // 相等性比较返回 False
    let result = ctx.eval(r#""hello" == 5"#).unwrap();
    assert_eq!(result.as_bool(), Some(false));
    
    // 顺序比较抛出 TypeError
    let result = ctx.eval(r#""hello" < 5"#);
    assert!(result.is_err());
}
```

### 集成测试

```rust
#[test]
fn test_grade_function_with_float() {
    let mut ctx = Context::new();
    ctx.eval(r#"
def get_grade(score):
    if score >= 90.0:
        return "A"
    elif score >= 80.0:
        return "B"
    elif score >= 70.0:
        return "C"
    else:
        return "F"

grade = get_grade(85.5)
    "#).unwrap();
    
    let grade = ctx.get("grade").unwrap();
    assert_eq!(grade.as_string(), Some("B"));
}
```

## 增量实现步骤

### Step 1: 修改 Eq 指令

- 移除 `pop_int()` 调用
- 使用模式匹配支持多种类型
- 不同类型返回 False

### Step 2: 修改 Ne 指令

- 与 Eq 类似，结果取反

### Step 3: 修改 Lt 指令

- 支持 int, float, string
- 不支持的类型抛出 TypeError

### Step 4: 修改 Le, Gt, Ge 指令

- 与 Lt 类似的逻辑

### Step 5: 添加测试

- 为每种类型组合添加测试
- 验证错误处理

### Step 6: 更新示例

- 修复综合示例中的浮点数比较问题
- 添加类型比较示例

## 后续任务

完成后可以：
- 综合示例可以使用浮点数评分
- 支持更多实际应用场景
- 为后续的高级比较功能（如列表比较）打基础

## 参考规范

详见 `spec/comparison-operators-fix.md`
