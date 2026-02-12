# 005: CLI 工具

**状态**: DONE  
**优先级**: P0

## 任务概述

提供命令行工具，让用户可以通过命令行运行 Python 代码和文件。

## 可运行功能

完成后用户可以：
```bash
# 运行 Python 文件
echo "10 * 5 + 2" > test.py
quickpython run test.py
# 输出: 52

# 编译为字节码
quickpython compile test.py -o test.pyq

# 运行字节码文件
quickpython run test.pyq
# 输出: 52

# 查看帮助
quickpython --help
quickpython run --help
```

**现在是真正的命令行工具了！**

## 依赖任务

- 001: 执行简单表达式

## 需要实现的模块

### 1. CLI 框架
- 使用 clap 解析命令行参数
- 定义子命令：run, compile
- 错误处理和退出码

### 2. run 子命令
- 读取 .py 文件
- 读取 .pyq 字节码文件（自动检测）
- 执行并打印结果
- 文件不存在时报错

### 3. compile 子命令
- 读取 .py 文件
- 编译为字节码
- 保存为 .pyq 文件（默认或指定路径）
- 显示编译信息

### 4. 字节码序列化
- ByteCode 序列化为二进制格式
- 从二进制反序列化
- 魔数和版本校验

### 5. 示例文件
- examples/*.py 示例代码
- 使用当前支持的功能（简单表达式）

## 验收条件

- [x] `quickpython run test.py` 能运行 .py 文件
- [x] `quickpython run notfound.py` 报错文件不存在
- [x] `quickpython compile test.py` 生成 test.pyq
- [x] `quickpython compile test.py -o out.pyq` 生成 out.pyq
- [x] `quickpython run test.pyq` 能运行字节码文件
- [x] .pyq 文件包含魔数和版本信息
- [x] examples/ 目录有至少 3 个示例文件
- [x] 语法错误报告清晰

## 测试要求

### 单元测试
- [x] 字节码序列化和反序列化正确性
- [x] 序列化后的数据包含正确的魔数 `QPY\0`
- [x] 序列化后的数据包含版本号
- [x] 反序列化无效数据时报错
- [x] 反序列化错误魔数时报错

### 集成测试
- [x] 运行 .py 文件输出正确结果
- [x] 运行不存在的文件报错
- [x] 编译 .py 文件生成 .pyq 文件
- [x] 编译时指定 -o 参数生成指定路径的 .pyq 文件
- [x] 运行 .pyq 文件输出正确结果
- [x] 完整流程：编译后运行字节码，结果与直接运行 .py 一致
- [x] 语法错误的 .py 文件报错清晰
- [x] --help 显示帮助信息

## 增量实现步骤

### Step 1: 完善 run 命令 ✅
- 已实现基础 .py 文件读取
- 添加文件类型检测（.py vs .pyq）

### Step 2: 实现字节码序列化
- 定义 .pyq 文件格式
- 实现序列化函数
- 实现反序列化函数
- 添加单元测试

### Step 3: 实现 compile 命令
- 添加 compile 子命令
- 支持 -o 参数指定输出
- 显示编译成功信息

### Step 4: 集成测试
- 编写集成测试
- 测试所有子命令
- 测试错误情况

### Step 5: 示例文件
- 创建 examples/*.py
- 添加注释说明
- 测试所有示例可运行

## 后续任务

完成后可以开始：
- 002: 变量赋值和读取
- 003: 函数定义和调用
