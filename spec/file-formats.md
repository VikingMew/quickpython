# QuickPython 文件格式

## 文件扩展名

### `.py` - QuickPython 源码文件
- **用途**: QuickPython 脚本源代码
- **格式**: UTF-8 文本文件
- **语法**: Python 语法子集
- **兼容性**: 与标准 Python 源码文件相同扩展名
- **示例**:
  ```python
  # hello.py
  def greet(name):
      return "Hello, " + name
  
  print(greet("World"))
  ```

### `.pyq` - QuickPython 字节码文件
- **用途**: 预编译的字节码
- **格式**: 二进制文件
- **优势**:
  - 启动速度快（< 1ms）
  - 体积小（比 Python .pyc 减少 30-50%）
  - 保护源码
- **生成方式**:
  ```rust
  let ctx = Context::new();
  let bytecode = ctx.compile_file("script.py")?;
  ctx.save_bytecode("script.pyq", &bytecode)?;
  ```
- **加载方式**:
  ```rust
  let ctx = Context::new();
  ctx.load_bytecode("script.pyq")?;
  ```

## 字节码文件结构

```
┌─────────────────────────────────────┐
│ Magic Number (4 bytes)              │  0x50595143 ("PYQC")
├─────────────────────────────────────┤
│ Version (4 bytes)                   │  格式版本号
├─────────────────────────────────────┤
│ Flags (4 bytes)                     │  编译标志
├─────────────────────────────────────┤
│ Constant Pool Size (4 bytes)       │
├─────────────────────────────────────┤
│ Constant Pool                       │  常量数据
│   - Integers                        │
│   - Floats                          │
│   - Strings                         │
├─────────────────────────────────────┤
│ String Table Size (4 bytes)        │
├─────────────────────────────────────┤
│ String Table                        │  变量名、函数名等
├─────────────────────────────────────┤
│ Function Count (4 bytes)            │
├─────────────────────────────────────┤
│ Function Table                      │
│   For each function:                │
│   - Name index                      │
│   - Argument count                  │
│   - Local variable count            │
│   - Stack size                      │
│   - Bytecode length                 │
│   - Bytecode data                   │
│   - Line info (optional)            │
├─────────────────────────────────────┤
│ Debug Info (optional)               │  调试符号
└─────────────────────────────────────┘
```

## 命名约定

### 源码文件
- 使用标准 Python 命名约定
- 示例：`my_script.py`, `game_logic.py`, `config.py`

### 字节码文件
- 与源码文件同名，扩展名改为 `.pyq`
- 示例：`my_script.pyq`, `game_logic.pyq`, `config.pyq`

### 模块导入
```python
# 导入 .py 文件（自动编译）
import my_module

# 或直接加载 .pyq（跳过编译）
import my_module  # 优先查找 my_module.pyq
```

## 工具链

### 编译器
```bash
# 编译单个文件
quickpython compile script.py -o script.pyq

# 自动生成同名字节码文件
quickpython compile script.py

# 编译多个文件
quickpython compile src/*.py -o build/
```

### 运行器
```bash
# 运行源码
quickpython run script.py

# 运行字节码
quickpython run script.pyq

# 简写形式
quickpython script.py
quickpython script.pyq
```

## 与 Python 的对比

| 特性 | Python | QuickPython |
|------|--------|-------------|
| 源码扩展名 | `.py` | `.py` |
| 字节码扩展名 | `.pyc` | `.pyq` |
| 字节码位置 | `__pycache__/` | 同目录或指定位置 |
| 字节码体积 | 较大 | 减少 30-50% |
| 启动速度 | 较慢 | < 1ms |
| 可读性 | 可反编译 | 更难逆向 |
