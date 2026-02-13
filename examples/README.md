# QuickPython Examples

这个目录包含展示 QuickPython 所有功能的示例程序。

## 📚 按编号组织的完整教程

### 01_basics.py - 基础功能
- 简单表达式（+, -, *, /）
- 变量赋值和使用
- 比较运算符（==, !=, <, >, <=, >=）
- 布尔值（True, False）

### 02_types.py - 数据类型
- 整数（int）
- 浮点数（float）
- 混合运算和类型提升
- 类型转换（int(), float()）
- 字符串（string）
- None

### 03_control_flow.py - 控制流
- if 语句
- if-else 语句
- 嵌套 if
- while 循环
- while 循环应用（计算总和）

### 04_functions.py - 函数
- 简单函数定义和调用
- 无返回值函数
- 递归函数（阶乘、斐波那契）
- 多参数函数
- 函数调用其他函数

### 05_lists.py - 列表操作
- 列表字面量
- 列表索引访问（正数和负数索引）
- 列表索引赋值
- 列表方法（append, pop）
- len() 函数
- 嵌套列表

### 06_dicts.py - 字典操作
- 字典字面量（字符串键和整数键）
- 字典访问
- 字典赋值（添加和修改）
- 字典方法（keys()）
- len() 函数
- 嵌套字典

### 07_for_loops.py - for 循环
- range() 的三种形式
- 遍历列表
- 遍历列表求和
- 遍历字典
- 遍历 dict.keys()
- 嵌套 for 循环
- 使用 for 构建列表

### 08_break_continue.py - 循环控制
- break 跳出 for 循环
- break 跳出 while 循环
- continue 跳过 for 循环
- continue 跳过 while 循环
- 使用 break 查找元素
- 嵌套循环中的 break
- break 和 continue 组合使用

### 09_exceptions.py - 异常处理
- try-except 基本用法
- 捕获异常并绑定到变量
- 多个 except 子句
- 捕获所有异常
- raise 抛出异常
- try-finally
- try-except-finally 完整形式
- 嵌套 try-except
- 函数中的异常处理
- 不同的异常类型（ValueError, TypeError, KeyError, RuntimeError, ZeroDivisionError, IndexError）

### 10_iterator_safety.py - 迭代器安全
- 安全的迭代
- 循环后修改列表
- 循环中修改列表检测（append, pop, 索引赋值）
- 嵌套循环中的修改检测
- 版本号机制说明

### 11_comprehensive.py - 综合示例
- 学生成绩管理系统
- 综合使用函数、循环、字典、列表、异常处理
- 实际应用场景演示

### 12_comparison_types.py - 比较运算符类型支持
- 浮点数比较
- 混合类型比较（int 和 float）
- 字符串比较（相等性和字典序）
- 布尔值和 None 比较
- 不同类型比较的行为
- 实际应用：评分系统和字符串排序

## 🚀 快速开始

### 运行示例

```bash
# 按顺序学习
cargo run -- run examples/01_basics.py
cargo run -- run examples/02_types.py
cargo run -- run examples/03_control_flow.py
# ... 依此类推

# 运行综合示例
cargo run -- run examples/11_comprehensive.py
```

### 编译为字节码

```bash
# 编译
cargo run -- compile examples/04_functions.py -o functions.pyq

# 运行字节码
cargo run -- run functions.pyq
```

## 📋 功能覆盖清单

### ✅ 基础功能（Task 001-005）
- [x] 算术运算（+, -, *, /）
- [x] 变量赋值和读取
- [x] 比较运算符（==, !=, <, >, <=, >=）
- [x] 布尔值（True, False）
- [x] 函数定义和调用（def, return）
- [x] 递归函数
- [x] if/else 条件语句
- [x] while 循环

### ✅ 数据类型（Task 006-008）
- [x] 字符串（字面量、拼接）
- [x] 浮点数（字面量、运算）
- [x] 类型转换（int(), float()）
- [x] 列表（字面量、索引、append, pop）
- [x] 字典（字符串键、整数键、keys()）
- [x] 嵌套数据结构
- [x] len() 函数
- [x] print() 函数

### ✅ 高级控制流（Task 009-010）
- [x] for 循环
- [x] range() 函数（三种形式）
- [x] 遍历列表
- [x] 遍历字典
- [x] break 语句
- [x] continue 语句
- [x] 嵌套循环

### ✅ 异常系统（Task 011-015）
- [x] 异常类型（7 种）
- [x] raise 语句
- [x] try-except 语句
- [x] try-finally 语句
- [x] try-except-finally 组合
- [x] 多个 except 子句
- [x] 异常变量绑定（as e）
- [x] 迭代器修改检测
- [x] 版本号机制

## 🎯 推荐学习路径

### 初学者路径
1. `01_basics.py` - 了解基础语法
2. `02_types.py` - 掌握数据类型
3. `03_control_flow.py` - 学习控制流
4. `04_functions.py` - 理解函数
5. `05_lists.py` 和 `06_dicts.py` - 掌握复合类型

### 进阶路径
6. `07_for_loops.py` - 掌握迭代
7. `08_break_continue.py` - 循环控制
8. `09_exceptions.py` - 异常处理
9. `10_iterator_safety.py` - 高级特性
10. `11_comprehensive.py` - 综合应用

## 📝 旧示例文件（保留用于兼容）

以下文件是早期开发时的简单示例，已被上述编号示例覆盖，但保留用于向后兼容：

- `simple_add.py` - 被 01_basics.py 覆盖
- `arithmetic.py` - 被 01_basics.py 覆盖
- `variables.py` - 被 01_basics.py 覆盖
- `floats.py` - 被 02_types.py 覆盖
- `strings.py` - 被 02_types.py 覆盖
- `lists_dicts.py` - 被 05_lists.py 和 06_dicts.py 覆盖
- `factorial.py` - 被 04_functions.py 覆盖
- `fibonacci_expr.py` - 被 04_functions.py 覆盖
- `fibonacci_while.py` - 被 03_control_flow.py 覆盖
- `simple_range.py` - 被 07_for_loops.py 覆盖
- `simple_for.py` - 被 07_for_loops.py 覆盖
- `for_loops.py` - 被 07_for_loops.py 覆盖
- `nested_for.py` - 被 07_for_loops.py 覆盖
- `simple_nested.py` - 被 07_for_loops.py 覆盖
- `break_continue.py` - 被 08_break_continue.py 覆盖
- `exceptions.py` - 被 09_exceptions.py 覆盖
- `iterator_safety.py` - 被 10_iterator_safety.py 覆盖

## 🧪 测试所有示例

```bash
# 测试所有编号示例
for i in {01..11}; do
    echo "Running ${i}_*.py..."
    cargo run -- run examples/${i}_*.py
    echo "---"
done
```

## 💡 贡献新示例

如果你想添加新示例：
1. 确保示例展示了特定功能
2. 添加清晰的注释
3. 包含预期输出
4. 更新本 README.md

## 📊 测试覆盖率

当前所有 75 个测试通过 ✅

所有功能均有对应示例！
