# 比较运算符类型支持规范

## 概述

定义比较运算符（==, !=, <, >, <=, >=）对不同数据类型的支持规则。

## 当前限制

当前实现仅支持整数比较，这导致以下 Python 代码无法执行：

```python
# 浮点数比较
x = 3.14
if x > 3.0:  # TypeError: Expected integer
    pass

# 字符串比较
if "Alice" == "Bob":  # TypeError: Expected integer
    pass

# 混合类型比较
score = 87.5
if score >= 90:  # TypeError: Expected integer
    pass
```

## 语义定义

### 相等性比较（==, !=）

**整数比较**
```python
5 == 5      # True
5 == 3      # False
5 != 3      # True
```

**浮点数比较**
```python
3.14 == 3.14    # True
3.14 == 2.71    # False
3.14 != 2.71    # True
```

**混合数值比较（整数与浮点数）**
```python
# 整数提升为浮点数后比较
10 == 10.0      # True
5 == 5.1        # False
3 != 3.0        # False
```

**字符串比较**
```python
"hello" == "hello"  # True
"hello" == "world"  # False
"" == ""            # True
```

**布尔值比较**
```python
True == True    # True
True == False   # False
False == False  # True
```

**None 比较**
```python
None == None    # True
None != None    # False
```

**不同类型之间**
```python
# 除 int/float 外，不同类型比较返回 False
"hello" == 5        # False
[1, 2] == 10        # False
True == 5           # False
```

### 顺序比较（<, >, <=, >=）

**整数比较**
```python
5 < 10      # True
10 > 5      # True
5 <= 5      # True
5 >= 5      # True
```

**浮点数比较**
```python
3.14 < 3.15     # True
2.71 > 2.70     # True
3.14 <= 3.14    # True
```

**混合数值比较**
```python
5 < 5.5         # True
10.0 > 9        # True
5 <= 5.0        # True
```

**字符串比较（字典序）**
```python
"apple" < "banana"      # True
"zebra" > "apple"       # True
"hello" <= "hello"      # True
"world" >= "hello"      # True
```

**不支持的类型组合**
```python
# 以下应抛出 TypeError
"hello" < 5     # TypeError
[1, 2] > 10     # TypeError
True < False    # TypeError
```

## 类型提升规则

### 整数与浮点数

当比较整数和浮点数时，整数自动提升为浮点数：

```python
# 等价转换
5 == 5.0    →  5.0 == 5.0
10 < 10.5   →  10.0 < 10.5
```

## 类型支持矩阵

### 相等性比较（==, !=）

| 类型 A ↓ | int | float | string | bool | None |
|---------|-----|-------|--------|------|------|
| **int** | ✅ | ✅ 提升 | ❌ → False | ❌ → False | ❌ → False |
| **float** | ✅ 提升 | ✅ | ❌ → False | ❌ → False | ❌ → False |
| **string** | ❌ → False | ❌ → False | ✅ | ❌ → False | ❌ → False |
| **bool** | ❌ → False | ❌ → False | ❌ → False | ✅ | ❌ → False |
| **None** | ❌ → False | ❌ → False | ❌ → False | ❌ → False | ✅ |

### 顺序比较（<, >, <=, >=）

| 类型 A ↓ | int | float | string | bool | None |
|---------|-----|-------|--------|------|------|
| **int** | ✅ | ✅ 提升 | ❌ TypeError | ❌ TypeError | ❌ TypeError |
| **float** | ✅ 提升 | ✅ | ❌ TypeError | ❌ TypeError | ❌ TypeError |
| **string** | ❌ TypeError | ❌ TypeError | ✅ 字典序 | ❌ TypeError | ❌ TypeError |
| **bool** | ❌ TypeError | ❌ TypeError | ❌ TypeError | ❌ TypeError | ❌ TypeError |
| **None** | ❌ TypeError | ❌ TypeError | ❌ TypeError | ❌ TypeError | ❌ TypeError |

**图例**：
- ✅ 支持
- ❌ → False：不同类型，相等性比较返回 False
- ❌ TypeError：不支持，抛出 TypeError
- **提升**：类型自动提升
- **字典序**：按字典顺序比较

## 边界情况

### 浮点数精度

```python
# 浮点数比较使用标准 IEEE 754
0.1 + 0.2 == 0.3  # 可能是 False（精度问题）
```

### 特殊浮点值

```python
# NaN、Inf 的处理遵循 IEEE 754
float('nan') == float('nan')  # False（标准行为）
float('inf') > 1000           # True
```

### 空字符串

```python
"" == ""        # True
"" < "a"        # True
```

### Unicode 字符串

```python
# 字符串比较按 Unicode 码点
"a" < "b"       # True
"ä" < "b"       # True (U+00E4 < U+0062)
```

## 与 Python 标准的差异

### 列表和字典比较

**Python 标准**：
```python
[1, 2] == [1, 2]    # True（递归比较）
{"a": 1} == {"a": 1}  # True（递归比较）
```

**QuickPython**：
```python
[1, 2] == [1, 2]    # False（对象身份比较）
```

未来可能支持深度相等性比较。

### 布尔值与整数

**Python 标准**：
```python
True == 1    # True
False == 0   # True
True < False # False
```

**QuickPython**：
```python
True == 1    # False（不同类型）
True < False # TypeError（不支持）
```

布尔值与整数被视为不同类型，不支持混合比较。
