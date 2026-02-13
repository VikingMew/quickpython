# QuickPython 任务目录

本目录包含 QuickPython 项目的所有开发任务。

## 任务原则

**每个任务完成后必须能运行一个可见的功能**，不能只是内部实现。

## 任务编号

使用简单的顺序编号：001, 002, 003...

## 任务内容

每个任务只包含**业务逻辑**，不包含具体代码实现：
- 任务概述
- 依赖关系
- 业务需求
- **可运行的功能**（用户能看到什么）
- 验收条件
- 测试要求

## 任务状态

- `TODO`: 未开始
- `IN_PROGRESS`: 进行中
- `DONE`: 已完成
- `VERIFIED`: 已验证

## 当前任务列表

| 编号 | 任务 | 可运行功能 | 状态 |
|------|------|-----------|------|
| 001 | 执行简单表达式 | `eval("1 + 2")` → 3 | DONE ✅ |
| 002 | 变量赋值和读取 | `eval("x = 1")` + `get("x")` | DONE ✅ |
| 003 | 函数定义和调用 | `def add(a,b): return a+b` | DONE ✅ |
| 004 | 控制流 (while) | `if/while` + Fibonacci | DONE ✅ |
| 005 | CLI 工具 | `quickpython run test.py` | DONE ✅ |
| 006 | 字符串和 print | `print("hello")` + 字符串拼接 | DONE ✅ |
| 007 | 浮点数支持 | `3.14 * 2.0` → 6.28 | DONE ✅ |
| 008 | 列表和字典 | `[1,2,3]` + `{"a": 1}` | DONE ✅ |
| 009 | for 循环和迭代器 | `for i in range(10)` | DONE ✅ |
| 010 | break 和 continue | 循环控制语句 | DONE ✅ |
| 011 | 异常类型和基础结构 | ExceptionType, ExceptionValue | DONE ✅ |
| 012 | raise 语句 | `raise ValueError("msg")` | DONE ✅ |
| 013 | try-except 语句 | `try-except` 异常捕获 | DONE ✅ |
| 014 | finally 和迭代器检测 | `try-finally` + 迭代安全 | DONE ✅ |
| 015 | VM 单循环架构 | 单循环 + 帧切换 | DONE ✅ |

## 增量开发路径

```
001: 能算 1+2 ✅
  ↓
002: 能存变量 x=1 ✅
  ↓
003: 能定义函数 ✅
  ↓
004: 能用 if/while ✅
  ↓
005: 能运行文件 ✅
  ↓
006: 能输出 print ✅
  ↓
007: 能算小数 ✅
  ↓
008: 能用列表字典 ✅
  ↓
009: 能用 for 循环 ✅
  ↓
010: 能用 break/continue ✅
  ↓
011-015: 完整异常系统 ✅
```

## Phase 1 (MVP) - 已完成 ✅

- ✅ 可以运行 Python 文件
- ✅ 支持基础运算和变量
- ✅ 支持函数和控制流 (if/while)
- ✅ 有可用的 CLI 工具
- ✅ 支持递归和字节码编译

## Phase 2 (实用功能) - 已完成 ✅

- ✅ 字符串和 print 输出 (Task 006)
- ✅ 浮点数运算 (Task 007)
- ✅ 列表和字典 (Task 008)
- ✅ for 循环遍历 (Task 009)
- ✅ break/continue (Task 010)

## Phase 3 (异常系统) - 已完成 ✅

- ✅ 异常类型和基础结构 (Task 011)
- ✅ raise 语句和异常抛出 (Task 012)
- ✅ try-except 语句 (Task 013)
- ✅ finally 块和迭代器安全 (Task 014)
- ✅ VM 单循环架构重构 (Task 015)

## 当前状态

✨ **75 个测试全部通过！**

已实现的功能：
- 基础类型：int, float, bool, None, string
- 复合类型：list, dict
- 运算符：+, -, *, /, ==, !=, <, >, <=, >=
- 控制流：if/else, while, for, break, continue
- 函数：def, return, 递归调用
- 异常：try-except-finally, raise, 7 种异常类型
- 迭代器：range, list, dict.keys()，带修改检测
- CLI：`cargo run -- run <file.py>`

## 后续规划

Phase 4 (高级特性)：
- 更多运算符 (//, %, and, or, not, in, is)
- elif 多分支条件
- 列表推导式
- 类和对象
- 模块和导入系统
- 装饰器
- 生成器和 yield
- with 语句
