# Task 019: 实现 re 模块

## 状态
- **状态**: DONE
- **优先级**: P1
- **预计工作量**: 中等
- **完成日期**: 2026-02-12

## 目标

实现 Python 的 re 模块，提供正则表达式功能，支持模式匹配、搜索、替换等操作。

## 范围

### 包含
1. 基础匹配函数
   - `re.match(pattern, string)` - 从字符串开头匹配
   - `re.search(pattern, string)` - 在字符串中搜索
   - `re.findall(pattern, string)` - 查找所有匹配
   - `re.finditer(pattern, string)` - 返回匹配迭代器

2. 替换函数
   - `re.sub(pattern, repl, string)` - 替换匹配的子串
   - `re.subn(pattern, repl, string)` - 替换并返回替换次数

3. 分割函数
   - `re.split(pattern, string)` - 按模式分割字符串

4. 编译函数
   - `re.compile(pattern)` - 编译正则表达式

5. Match 对象
   - `match.group(n)` - 获取捕获组
   - `match.groups()` - 获取所有捕获组
   - `match.start()` - 获取匹配开始位置
   - `match.end()` - 获取匹配结束位置
   - `match.span()` - 获取匹配范围

### 不包含
- 正则表达式标志（IGNORECASE, MULTILINE 等）
- 命名捕获组
- 前瞻和后顾断言
- 条件匹配

## 实现要点

1. 添加 `regex = "1.10"` 依赖到 Cargo.toml
2. 扩展 Value 枚举，添加 Regex 和 Match 类型
3. 创建 MatchObject 结构体，存储匹配文本、位置和捕获组
4. 在 VM 的 CallMethod 中添加 Match 对象方法的处理
5. re.match() 只匹配字符串开头，re.search() 在任意位置搜索
6. re.subn() 返回元组 (result, count)
7. Match.group(0) 是整个匹配，group(1) 是第一个捕获组
8. Match.groups() 返回所有捕获组（不包括 group(0)）

## 示例代码

```python
import re

# 基础匹配
m = re.match(r"hello", "hello world")
if m:
    print(m.group(0))  # hello

# 搜索
m = re.search(r"\d+", "abc 123 def")
if m:
    print(m.group(0))  # 123

# 查找所有
matches = re.findall(r"\d+", "abc 123 def 456 ghi")
print(matches)  # ['123', '456']

# 替换
result = re.sub(r"\d+", "X", "abc 123 def 456")
print(result)  # abc X def X

# 分割
parts = re.split(r"\s+", "hello  world   test")
print(parts)  # ['hello', 'world', 'test']

# 捕获组
m = re.search(r"(\d+)-(\d+)", "Phone: 123-456")
if m:
    print(m.group(1))  # 123
    print(m.group(2))  # 456
    print(m.groups())  # ['123', '456']

# 位置信息
m = re.search(r"world", "hello world")
if m:
    print(m.start())  # 6
    print(m.end())    # 11
    print(m.span())   # [6, 11]
```

## 验收标准

- [ ] 可以 `import re`
- [ ] `re.match()` 正常工作
- [ ] `re.search()` 正常工作
- [ ] `re.findall()` 正常工作
- [ ] `re.sub()` 正常工作
- [ ] `re.split()` 正常工作
- [ ] Match 对象的 `group()` 方法正常工作
- [ ] Match 对象的 `groups()` 方法正常工作
- [ ] Match 对象的 `start()`, `end()`, `span()` 方法正常工作
- [ ] 捕获组功能正常
- [ ] 所有测试通过
- [ ] 错误处理正确（无效正则表达式等）
- [ ] 创建 `examples/15_re_module.py` 示例文件并验证运行

## 注意事项

1. **正则表达式语法**：使用 Rust 的 regex crate，语法与 Python 略有不同
2. **Match 对象**：需要存储匹配的文本、位置和捕获组
3. **捕获组索引**：group(0) 是整个匹配，group(1) 是第一个捕获组
4. **None 处理**：未匹配时返回 None，不是抛出异常
5. **错误处理**：无效的正则表达式应该抛出 ValueError

## 后续任务

- Task 020: 实现 sys 模块
- Task 021: 实现文件 I/O（open, read, write）
- Task 022: 添加正则表达式标志支持（IGNORECASE 等）
