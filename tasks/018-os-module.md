# Task 018: 实现 os 模块

## 状态
- **状态**: DONE
- **优先级**: P1
- **预计工作量**: 中等
- **完成日期**: 2026-02-12

## 目标

实现 Python 的 os 模块，提供操作系统相关的功能，包括文件系统操作、环境变量、进程管理等。

## 范围

### 包含
1. 文件和目录操作
   - `os.listdir(path)` - 列出目录内容
   - `os.mkdir(path)` - 创建目录
   - `os.makedirs(path)` - 递归创建目录
   - `os.remove(path)` - 删除文件
   - `os.rmdir(path)` - 删除空目录
   - `os.rename(old, new)` - 重命名文件或目录
   - `os.getcwd()` - 获取当前工作目录
   - `os.chdir(path)` - 改变当前工作目录

2. 路径操作（os.path 子模块）
   - `os.path.exists(path)` - 检查路径是否存在
   - `os.path.isfile(path)` - 检查是否为文件
   - `os.path.isdir(path)` - 检查是否为目录
   - `os.path.join(path1, path2, ...)` - 拼接路径
   - `os.path.basename(path)` - 获取文件名
   - `os.path.dirname(path)` - 获取目录名
   - `os.path.abspath(path)` - 获取绝对路径

3. 环境变量
   - `os.environ` - 环境变量字典
   - `os.getenv(key, default=None)` - 获取环境变量

4. 系统信息
   - `os.name` - 操作系统名称（'posix' 或 'nt'）

### 不包含
- 进程管理（os.fork, os.exec 等）
- 文件描述符操作（os.open, os.read 等）
- 权限管理（os.chmod, os.chown 等）
- 符号链接操作

## 实现要点

1. 使用 Rust 标准库的 `std::fs` 和 `std::env` 模块
2. 使用 `std::path::Path` 和 `PathBuf` 处理路径
3. os.path 需要作为子模块（Module 类型）添加到 os 模块
4. os.environ 应该是一个字典，包含所有环境变量
5. 添加 OSError 异常类型用于文件系统错误
6. 路径分隔符使用 Rust 的 Path API 自动处理跨平台差异

## 示例代码

```python
import os

# 获取当前目录
cwd = os.getcwd()
print(cwd)

# 列出目录内容
files = os.listdir(".")
print(files)

# 路径操作
path = os.path.join("dir", "subdir", "file.txt")
exists = os.path.exists("Cargo.toml")
basename = os.path.basename("/path/to/file.txt")

# 环境变量
home = os.getenv("HOME", "/default")
print(os.environ["PATH"])

# 系统信息
print(os.name)

# 创建和删除目录
os.mkdir("test_dir")
os.rmdir("test_dir")
```

## 验收标准

- [x] 可以 `import os`
- [x] 文件和目录操作函数正常工作
- [x] os.path 子模块可以访问
- [x] os.path 的所有函数正常工作
- [x] 环境变量操作正常
- [x] os.environ 字典可以访问
- [x] os.name 返回正确的系统名称
- [x] 所有测试通过
- [x] 错误处理正确（文件不存在、权限错误等）
- [x] 创建 `examples/14_os_module.py` 示例文件并验证运行

## 注意事项

1. **路径分隔符**：使用 Rust 的 Path API 自动处理不同平台的路径分隔符
2. **错误处理**：文件系统操作可能失败，需要转换为 OSError 异常
3. **环境变量**：os.environ 应该是一个字典，可以读取但不支持修改
4. **子模块**：os.path 是一个子模块，需要作为 Module 类型添加到 os 模块
5. **平台差异**：os.name 需要根据编译目标平台返回不同的值

## 后续任务

- Task 019: 实现 re 模块（正则表达式）
- Task 020: 实现 sys 模块
- Task 021: 实现文件 I/O（open, read, write）
