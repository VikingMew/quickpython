# QuickPython 语言规格

## 当前实现状态

### ✅ 已实现 (Phase 1 - MVP)

#### 数据类型
- [x] 整数 (i32)
- [x] 布尔值 (bool)
- [x] 空值 (None)
- [x] 函数 (Function)
- [ ] 字符串 (String) - 待实现 (Task 006)
- [ ] 浮点数 (f64) - 待实现

#### 运算符
- [x] 算术: +, -, *, /
- [x] 比较: ==, !=, <, <=, >, >=
- [ ] 字符串拼接: + - 待实现 (Task 006)

#### 控制流
- [x] if/else
- [x] while 循环
- [ ] for 循环 - 待实现
- [ ] break/continue - 待实现

#### 函数
- [x] 函数定义 (def)
- [x] 函数调用
- [x] 参数传递
- [x] return 语句
- [x] 递归

#### 内置函数
- [ ] print() - 待实现 (Task 006)
- [ ] len() - 待实现
- [ ] range() - 待实现

---

## 支持的 Python 语法子集

### 数据类型

#### 基础类型 (Phase 1)
```python
# 整数 ✅
x = 42
y = -100

# 浮点数 (待实现)
pi = 3.14
e = 2.718

# 字符串 (Task 006)
name = "QuickPython"
message = 'Hello, World!'

# 布尔值 ✅
is_valid = True
is_empty = False

# 空值 ✅
result = None
```

#### 复合类型 (Phase 2)
```python
# 列表
numbers = [1, 2, 3, 4, 5]
mixed = [1, "two", 3.0, True]

# 字典
person = {"name": "Alice", "age": 30}
config = {1: "one", 2: "two"}

# 元组 (可选)
point = (10, 20)
```

### 运算符

#### 算术运算符 ✅
```python
a + b    # 加法
a - b    # 减法
a * b    # 乘法
a / b    # 除法
a // b   # 整除
a % b    # 取模
a ** b   # 幂运算 (Phase 2)
```

#### 比较运算符
```python
a == b   # 等于
a != b   # 不等于
a < b    # 小于
a <= b   # 小于等于
a > b    # 大于
a >= b   # 大于等于
```

#### 逻辑运算符
```python
a and b  # 与
a or b   # 或
not a    # 非
```

#### 成员运算符 (Phase 2)
```python
x in list
x not in list
```

### 控制流

#### 条件语句
```python
if x > 0:
    print("positive")
elif x < 0:
    print("negative")
else:
    print("zero")

# 三元表达式 (Phase 2)
result = "yes" if condition else "no"
```

#### 循环语句
```python
# while 循环
i = 0
while i < 10:
    print(i)
    i = i + 1

# for 循环
for i in range(10):
    print(i)

for item in [1, 2, 3]:
    print(item)

# break 和 continue
for i in range(10):
    if i == 5:
        break
    if i % 2 == 0:
        continue
    print(i)
```

### 函数定义

#### 基础函数 (Phase 1)
```python
def greet(name):
    return "Hello, " + name

def add(a, b):
    return a + b

# 无返回值
def print_message(msg):
    print(msg)
```

#### 高级特性 (Phase 2)
```python
# 默认参数
def greet(name, greeting="Hello"):
    return greeting + ", " + name

# 可变参数 (Phase 3)
def sum_all(*args):
    total = 0
    for x in args:
        total = total + x
    return total

# Lambda 表达式 (Phase 2)
square = lambda x: x * x
```

### 类和对象 (Phase 2)

```python
class Point:
    def __init__(self, x, y):
        self.x = x  # 编译时确定 slot 位置
        self.y = y
    
    def distance(self):
        return (self.x ** 2 + self.y ** 2) ** 0.5

p = Point(3, 4)
print(p.distance())  # 5.0

# 注意：不支持动态添加属性
# p.z = 5  # 错误！属性必须在 __init__ 中定义
```

**限制**：
- 所有属性必须在 `__init__` 中定义
- 不支持动态添加属性（`p.new_attr = value`）
- 不支持 `__dict__` 访问
- 使用固定 slot，性能优先

### 异常处理 (Phase 2)

```python
try:
    result = 10 / 0
except ZeroDivisionError:
    print("Cannot divide by zero")
except Exception as e:
    print("Error:", e)
finally:
    print("Cleanup")
```

### 模块系统 (Phase 2)

```python
# 导入模块
import math
from collections import defaultdict

# 使用模块
print(math.sqrt(16))
```

## 不支持的特性

以下 Python 特性在 QuickPython 中**不支持**：

1. **高级语法**
   - 装饰器 (@decorator)
   - 生成器 (yield)
   - 上下文管理器 (with statement)
   - 列表推导式（可能在 Phase 3 支持）

2. **高级类型**
   - Set 集合
   - Frozenset
   - Bytes/Bytearray
   - Complex numbers

3. **元编程**
   - Metaclass
   - 动态代码执行 (eval/exec)
   - 反射（部分）
   - `__dict__` 访问
   - 动态添加属性

4. **标准库**
   - 只提供核心的内置函数
   - 不支持完整的 Python 标准库

5. **C API**
   - 不兼容 Python C API
   - 不支持 CPython 扩展模块

## 内置函数

### Phase 1
```python
print(...)      # 打印输出
len(obj)        # 获取长度
type(obj)       # 获取类型
str(obj)        # 转换为字符串
int(obj)        # 转换为整数
float(obj)      # 转换为浮点数
bool(obj)       # 转换为布尔值
range(n)        # 生成范围
```

### Phase 2
```python
min(...)        # 最小值
max(...)        # 最大值
sum(iterable)   # 求和
abs(x)          # 绝对值
round(x)        # 四舍五入
enumerate(...)  # 枚举
zip(...)        # 拉链
map(...)        # 映射
filter(...)     # 过滤
```

## 语法限制

1. **缩进**: 只支持空格缩进，不支持 Tab（或统一转换）
2. **编码**: 只支持 UTF-8
3. **注释**: 只支持 # 单行注释
4. **字符串**: 只支持基础转义序列 (\n, \t, \\, \", \')
5. **数字**: 不支持复数和下划线分隔符（1_000_000）

## 与标准 Python 的差异

1. **性能优先**: 某些行为可能与 CPython 不完全一致
2. **简化语义**: 去除一些复杂的边界情况
3. **有限标准库**: 只提供核心功能
4. **严格模式**: 某些 Python 的动态特性被限制
