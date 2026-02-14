# Task 020: 实现扩展模块注册机制和 llm 扩展示例

## 状态
- **状态**: TODO
- **优先级**: P1
- **预计工作量**: 大

## 目标

实现扩展模块注册机制，允许第三方 Rust crate 注册为 Python 模块。并以 llm 模块作为第一个扩展模块示例。

## 范围

### 包含
1. 设计扩展模块注册 API
2. 实现模块加载时的扩展模块查找机制
3. 创建独立的 `quickpython-llm` crate 作为扩展示例
4. 在 `quickpython-llm` 中实现 OpenAI compatible API 调用
5. 创建 `quickpython-demo` workspace 演示如何集成扩展模块

### 不包含
- 动态加载（.so/.dll）
- Python wheel 文件支持
- 包管理系统
- 流式响应（streaming）
- 函数调用（function calling）

## 项目结构

```
quickpython/                    # 核心库
├── Cargo.toml                  # 添加 workspace 配置
├── src/
│   ├── extension.rs           # 新增：扩展模块注册机制
│   ├── vm.rs                   # 修改：支持扩展模块加载
│   └── ...

quickpython-llm/               # 扩展模块（独立 crate）
├── Cargo.toml
├── src/
│   └── lib.rs
└── README.md

quickpython-demo/              # 演示项目（sub workspace）
├── Cargo.toml
├── src/
│   └── main.rs
└── examples/
    └── llm_chat.py
```

## 实现要点

### 1. 扩展模块注册机制
- 创建 `src/extension.rs` 文件
- 定义扩展模块工厂函数类型：`fn() -> Module`
- 在 `Context` 上提供 `register_extension` 成员方法
- 扩展模块注册到 Context 实例，而非全局注册表

### 2. 修改模块加载逻辑
- 在 `vm.rs` 的 `load_module` 中添加扩展模块查找
- 模块查找顺序：缓存 → 内置模块 → 扩展模块 → 报错

### 3. 创建 quickpython-llm 扩展
- 使用 `cargo init --lib` 创建
- 实现 `llm.configure()` - 配置 API endpoint 和 key
- 实现 `llm.chat()` - 发送 HTTP 请求到 OpenAI API
- 提供 `pub fn create_module() -> Module` 工厂函数

### 4. 创建 quickpython-demo 演示项目
- 使用 `cargo init` 创建
- 在 main.rs 中通过 `ctx.register_extension("llm", quickpython_llm::create_module)` 注册
- 提供可执行程序运行 Python 文件
- 创建 examples/llm_chat.py 示例

### 5. Workspace 配置
- 在根目录 Cargo.toml 添加 workspace 配置
- members 包含三个 crate

## 依赖管理

**重要：所有依赖通过 `cargo add` 添加**

quickpython-llm 需要的依赖：
- reqwest（features: json, blocking）
- serde（features: derive）
- serde_json
- quickpython（path 依赖）

quickpython-demo 需要的依赖：
- quickpython（path 依赖）
- quickpython-llm（path 依赖）

## Python API 设计

```python
import llm
import json
import os

# 配置
config = {
    "endpoint": os.getenv("LLM_API_ENDPOINT"),
    "api_key": os.getenv("LLM_API_KEY"),
    "model": "gpt-3.5-turbo"
}
llm.configure(config)

# 发送请求
messages = [
    {"role": "system", "content": "You are a helpful assistant."},
    {"role": "user", "content": "Hello!"}
]
response_json = llm.chat(messages)
response = json.loads(response_json)
print(response["content"])
```

## 验收标准

- [ ] 实现扩展模块注册机制（extension.rs）
- [ ] 修改 VM 模块加载逻辑支持扩展模块
- [ ] 创建 workspace 配置
- [ ] 创建独立的 `quickpython-llm` crate
- [ ] 创建 `quickpython-demo` 演示项目
- [ ] `llm.configure()` 可以设置配置
- [ ] `llm.chat()` 可以发送请求并返回 JSON 字符串
- [ ] 核心库（quickpython）不包含 llm 时，import llm 报 ImportError
- [ ] demo 项目可以正常 import llm 并使用
- [ ] 创建 `quickpython-demo/examples/llm_chat.py` 示例
- [ ] 文档说明如何创建扩展模块
- [ ] 文档说明 workspace 结构
- [ ] 所有测试用例通过

## 测试场景

### 场景 1: 核心库不包含扩展模块
**目的**：验证核心库的独立性

**测试步骤**：
1. 使用核心库的 quickpython 运行 Python 代码
2. 尝试 `import llm`

**预期结果**：
- 抛出 ImportError 异常
- 错误信息：`No module named 'llm'`

### 场景 2: 扩展模块注册
**目的**：验证扩展模块注册机制

**测试步骤**：
1. 创建 Context，调用 `ctx.register_extension("llm", quickpython_llm::create_module)`
2. 尝试 `import llm`
3. 检查 llm 模块是否有 configure 和 chat 函数

**预期结果**：
- import llm 成功
- llm.configure 存在且可调用
- llm.chat 存在且可调用

### 场景 3: llm.configure() 配置
**目的**：验证配置功能

**测试步骤**：
1. 创建配置字典（包含 endpoint, api_key, model）
2. 调用 `llm.configure(config)`

**预期结果**：
- 不抛出异常
- 返回 None

### 场景 4: 缺少配置字段
**目的**：验证配置验证

**测试步骤**：
1. 创建不完整的配置字典（缺少 api_key）
2. 调用 `llm.configure(config)`

**预期结果**：
- 抛出 KeyError 异常
- 错误信息包含缺少的字段名

### 场景 5: llm.chat() 发送请求
**目的**：验证 chat 功能

**测试步骤**：
1. 配置 llm
2. 创建 messages 列表
3. 调用 `llm.chat(messages)`
4. 使用 `json.loads()` 解析响应

**预期结果**：
- 返回 JSON 字符串
- 解析后的字典包含 role 和 content 字段
- role 为 "assistant"

### 场景 6: 多轮对话
**目的**：验证对话历史支持

**测试步骤**：
1. 创建包含 system 消息的 messages 列表
2. 添加第一个 user 消息
3. 调用 llm.chat()，将响应添加到 messages
4. 添加第二个 user 消息
5. 再次调用 llm.chat()

**预期结果**：
- 两次调用都成功返回
- 第二次调用能访问完整的对话历史

### 场景 7: 错误处理 - 网络错误
**目的**：验证错误处理

**测试步骤**：
1. 配置错误的 endpoint（如 "http://invalid-endpoint"）
2. 调用 llm.chat()

**预期结果**：
- 抛出适当的 Python 异常（RuntimeError 或 OSError）
- 错误信息描述网络问题

### 场景 8: 错误处理 - 参数错误
**目的**：验证参数验证

**测试步骤**：
1. 调用 llm.chat() 但不传递参数
2. 或传递非列表类型的参数

**预期结果**：
- 抛出 TypeError 异常
- 错误信息说明参数要求

## 测试用例

```python
# test_extension_module.py - 在核心库测试中

def test_import_llm_without_registration():
    """测试未注册时导入 llm 模块"""
    # 预期抛出 ImportError

def test_extension_module_registration():
    """测试扩展模块注册"""
    # 注册后可以导入

# test_llm_module.py - 在 quickpython-llm 测试中

def test_llm_configure():
    """测试 llm.configure() 配置"""
    # 传入完整配置，不应抛出异常

def test_llm_configure_missing_fields():
    """测试缺少配置字段"""
    # 预期抛出 KeyError

def test_llm_chat_basic():
    """测试基础 chat 调用"""
    # 返回 JSON 字符串，包含 assistant 消息

def test_llm_chat_multiround():
    """测试多轮对话"""
    # 传入完整 messages 历史，返回正确响应

def test_llm_chat_invalid_endpoint():
    """测试无效的 endpoint"""
    # 预期抛出网络错误异常

def test_llm_chat_no_args():
    """测试不传递参数"""
    # 预期抛出 TypeError

# test_demo.py - 在 quickpython-demo 测试中

def test_demo_can_import_llm():
    """测试 demo 项目可以导入 llm"""
    # import llm 成功

def test_demo_run_example():
    """测试运行示例文件"""
    # 运行 examples/llm_chat.py 不报错
```

## 注意事项

1. **模块隔离**：llm 模块是独立的 crate，不污染核心库
2. **API 设计**：扩展模块 API 要简单易用
3. **类型导出**：需要导出 `Value`, `Module`, `ExceptionType` 等类型
4. **线程安全**：注册表需要线程安全
5. **错误处理**：HTTP 错误、JSON 错误要转换为 Python 异常
6. **配置存储**：llm 配置需要全局存储

## 架构优势

1. **核心库保持纯净**：内置模块（json, os, re）在核心库
2. **扩展模块独立**：llm 等扩展模块是独立 crate
3. **清晰的边界**：
   - quickpython：核心库 + 扩展机制
   - quickpython-llm：扩展模块
   - quickpython-demo：演示集成
4. **独立维护**：扩展模块可以独立版本和发布
5. **用户选择**：用户可以只用核心库，或创建自己的集成项目

## 后续任务

- Task 021: 实现动态加载扩展模块（.so/.dll）
- Task 022: 实现 Python wheel 文件支持
- Task 023: 创建更多扩展模块示例（http, sqlite 等）
