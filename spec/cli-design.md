# QuickPython CLI 工具设计

## 命令行工具：`quickpython`

简洁的命令行工具，用于编译和运行 QuickPython 脚本。

## 基本用法

```bash
quickpython [COMMAND] [OPTIONS] [FILE]
```

## 命令列表

### 1. `compile` - 编译源码为字节码

```bash
# 编译单个文件
quickpython compile script.py -o script.pyq

# 不指定输出文件（自动生成同名 .pyq 文件）
quickpython compile script.py

# 编译多个文件
quickpython compile src/main.py src/utils.py -o build/
```

**选项**：
- `-o, --output <PATH>` - 输出路径（文件或目录）

**示例**：
```bash
# 编译 game.py 生成 game.pyq
quickpython compile game.py

# 编译并指定输出位置
quickpython compile game.py -o dist/game.pyq
```

### 2. `run` - 运行脚本

```bash
# 运行 Python 源码（自动编译）
quickpython run script.py

# 运行字节码
quickpython run script.pyq

# 传递参数给脚本
quickpython run script.py arg1 arg2

# 简写形式（省略 run 命令）
quickpython script.py
quickpython script.pyq
```

**示例**：
```bash
# 运行源码
quickpython run game.py

# 运行字节码（更快启动）
quickpython run game.pyq

# 传递参数
quickpython run server.py --port 8080
```

## 全局选项

```bash
-h, --help              显示帮助信息
-V, --version           显示版本信息
```

## 使用示例

### 开发阶段

```bash
# 直接运行源码（开发时）
quickpython run game.py
```

### 发布阶段

```bash
# 编译为字节码
quickpython compile game.py -o game.pyq

# 运行字节码（生产环境）
quickpython run game.pyq
```

### 批量编译

```bash
# 编译多个文件
quickpython compile src/*.py -o build/
```

## 退出码

- `0` - 成功
- `1` - 一般错误
- `2` - 文件未找到
- `3` - 编译错误
- `4` - 运行时错误

## 集成示例

### Makefile

```makefile
.PHONY: build run clean

build:
	quickpython compile src/main.py -o build/main.pyq

run:
	quickpython run build/main.pyq

clean:
	rm -rf build/
```

### Shell 脚本

```bash
#!/bin/bash

# build.sh - 构建脚本

echo "Compiling..."
quickpython compile src/main.py -o dist/main.pyq

if [ $? -eq 0 ]; then
    echo "Build successful!"
    quickpython run dist/main.pyq
else
    echo "Build failed!"
    exit 1
fi
```

## 工作流程

```
开发 → 测试 → 编译 → 部署

┌─────────────┐
│  编写代码    │  vim game.py
│  game.py    │
└──────┬──────┘
       │
       ▼
┌─────────────┐
│  测试运行    │  quickpython run game.py
└──────┬──────┘
       │
       ▼
┌─────────────┐
│  编译字节码  │  quickpython compile game.py -o game.pyq
│  game.pyq   │
└──────┬──────┘
       │
       ▼
┌─────────────┐
│  部署运行    │  quickpython run game.pyq
└─────────────┘
```
