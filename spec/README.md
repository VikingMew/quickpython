# QuickPython 项目规格文档

## 项目概述

QuickPython 是一个嵌入式 Python 风格脚本语言，设计目标是在 Rust 项目中提供类似 QuickJS 和 LuaJIT 的高性能胶水语言能力。

## 设计目标

1. **高性能执行** - 基于栈的 VM，Tagged Pointers 优化，Slot 静态查找
2. **轻量级嵌入** - 最小化运行时开销，< 1MB 基础运行时
3. **Python 语法** - 使用 Python 语法子集，降低学习成本
4. **Rust 互操作** - 提供简洁的 Rust API，方便双向调用
5. **原生异步** - 异步功能直接集成到 VM，与 Rust async/await 无缝对接
6. **简单 GC** - 引用计数 + 确定性垃圾回收，无 GC 暂停
7. **精简设计** - 不支持元编程、动态属性、C API，专注性能

## 核心优化技术（借鉴 QuickJS）

### 🚀 NaN Boxing / Tagged Pointers
- 小整数、布尔、None、浮点数**零堆分配**
- Value 只占 8 字节，类型检查仅需位运算
- 避免 Python "万物皆对象" 的性能陷阱

### 🔤 原子化字符串（String Interning）
- 相同字符串内存中只存一份
- 字符串比较 O(n) → O(1)
- 大幅节省内存占用

### ♻️ 引用计数（确定性垃圾回收）
- 简单的引用计数机制，无复杂 GC 算法
- 确定性析构，对象立即释放
- 无 GC 暂停，无 stop-the-world
- 可选的简单循环检测工具

### 📦 紧凑字节码
- 变长指令编码，常用指令 1-2 字节
- `.pyq` 字节码体积比 Python `.pyc` 减少 30-50%
- 更好的缓存局部性和执行性能

### ⚡ 预编译与字节码缓存
- 源码文件：`.py`（标准 Python 语法）
- 字节码文件：`.pyq`（QuickPython 预编译字节码）
- 启动时间微秒级（< 1ms）
- 编译一次，多次执行

### 🔄 原生异步支持
- 异步功能直接集成到 VM
- 与 Rust 的 async/await 无缝对接
- 支持 Python 的 async/await 语法
- 无需复杂的事件循环库
- 高性能协程调度

### ⚡ Slot 和静态查找
- 对象属性使用固定 slot，不使用 `__dict__`
- 编译时确定所有属性位置
- 属性访问 O(1)，直接数组索引
- 内存布局紧凑，缓存友好
- 不支持动态添加属性（性能优先）

## 文档结构

- [架构设计](./architecture.md) - 整体架构和模块划分
- [语言规格](./language-spec.md) - 支持的 Python 语法子集定义
- [API 设计](./api-design.md) - Rust 集成 API 设计
- [文件格式](./file-formats.md) - `.py` 源码和 `.pyq` 字节码文件格式
- [CLI 工具](./cli-design.md) - `quickpython` 命令行工具设计

## 核心特性

### 基础数据类型
- 整数 (int) - 小整数使用 Tagged Pointers，零堆分配
- 浮点数 (float) - NaN Boxing 优化
- 字符串 (str) - 原子化字符串池
- 布尔值 (bool) - Tagged Pointers
- 空值 (None) - Tagged Pointers
- 列表 (list)
- 字典 (dict)

### 控制流
- 条件语句：if/elif/else
- 循环：for, while
- 循环控制：break, continue

### 函数
- 同步函数定义和调用
- 异步函数：async def / await
- 参数传递
- 返回值

### 异步编程
- async/await 语法
- 原生 Future 支持
- 协程
- 与 Rust async 互操作

### Rust 集成
- 简洁的嵌入 API
- 双向值转换（Tagged Pointers）
- 同步函数注册
- 异步函数注册
- 错误处理

### 不支持的特性
- ❌ 元编程（装饰器、元类等）
- ❌ 完整标准库（只提供核心内置函数）
- ❌ 复杂 GC（只用简单引用计数）
- ❌ 动态代码执行（eval/exec）
- ❌ 动态属性（`__dict__`），使用固定 slot
- ❌ Python C API（不兼容 CPython 扩展）
- ❌ C 函数绑定（只支持 Rust 原生绑定）

## 技术架构

### 编译前端
- **Parser**: 使用 `rustpython_parser` 解析 Python 代码
- **AST**: 复用 RustPython 的成熟 AST 定义
- **Compiler**: 将 AST 编译为紧凑字节码

### 运行时
- **VM**: 基于栈的虚拟机执行器
- **Value System**: Tagged Pointers (NaN Boxing) 动态类型系统
- **String Pool**: 原子化字符串池
- **RefCount GC**: 简单的引用计数垃圾回收
- **Async Runtime**: 原生异步运行时（集成 Tokio）
- **Builtins**: 精简内置函数库

### API 层
- **Context**: 执行上下文管理
- **Value Binding**: Rust ↔ Python 值转换
- **Function Registration**: 同步和异步函数注册
- **Error Handling**: 统一错误处理

## 性能特点

- ⚡ **启动速度**: < 1ms（预编译模式）
- 💾 **内存占用**: < 1MB（基础运行时）
- 🚀 **执行性能**: 目标达到 QuickJS 的 70-90%
- 📦 **字节码体积**: 比 Python .pyc 减少 30-50%
- 🔧 **零 GC 暂停**: 引用计数实时回收

## 使用示例

### 基础用法
```rust
use quickpython::{Context, Value};

// 创建上下文
let mut ctx = Context::new();

// 执行 Python 代码
ctx.eval("x = 1 + 2")?;

// 获取变量
let x: i64 = ctx.get("x")?.try_into()?;
println!("x = {}", x);  // x = 3

// 注册 Rust 函数
ctx.register_function("greet", |name: String| {
    format!("Hello, {}!", name)
})?;

// 调用 Rust 函数
ctx.eval(r#"
message = greet("World")
print(message)
"#)?;
```

### 异步用法
```rust
use quickpython::{Context, Value};

#[tokio::main]
async fn main() -> Result<()> {
    let mut ctx = Context::new();
    
    // 注册异步函数
    ctx.register_async_function("fetch", |url: String| async move {
        let response = reqwest::get(&url).await?;
        Ok(response.text().await?)
    })?;
    
    // 执行异步 Python 代码
    ctx.eval_async(r#"
async def main():
    data = await fetch("https://api.example.com")
    print(data)

await main()
    "#).await?;
    
    Ok(())
}
```

## 与 QuickJS 的关系

QuickPython 借鉴了 QuickJS 的核心设计，但针对 Python 和 Rust 生态做了重要调整：

### 借鉴自 QuickJS ✅
- **NaN Boxing** - 完整的 Tagged Pointers 实现
- **引用计数** - 简单高效的内存管理
- **Atom 系统** - 字符串原子化
- **Runtime/Context 分离** - 清晰的架构
- **紧凑字节码** - 变长编码优化

### QuickPython 的创新 🆕
- **原生 Rust async/await** - 不是 Promise，直接集成 Tokio
- **Slot 静态查找** - 不是动态属性表，编译时确定
- **Rust 函数绑定** - 不是 C API，类型安全的 Rust FFI
- **Python 语法** - 不是 JavaScript，使用 Python 子集
- **无 C 依赖** - 纯 Rust 实现，不需要 C 工具链

### 对比表

| 特性 | QuickJS | QuickPython |
|------|---------|-------------|
| 语言 | JavaScript | Python 子集 |
| Value 表示 | NaN Boxing | NaN Boxing（借鉴） |
| 内存管理 | 引用计数 | 引用计数（借鉴） |
| 字符串 | Atom | Atom（借鉴） |
| 异步 | Promise | Rust async/await |
| 属性访问 | 动态属性表 | Slot 静态查找 |
| FFI | C API | Rust 原生 |
| 依赖 | C 工具链 | 纯 Rust |

## 与其他方案对比

### 完整对比表
|------|-------------|---------|---------|--------|
| 语法 | Python 子集 | 完整 Python | JavaScript | Lua |
| 启动速度 | 快 | 慢 | 快 | 非常快 |
| 内存占用 | 小 | 大 | 小 | 小 |
| 嵌入性 | 优秀 | 可行 | 优秀 | 优秀 |
| Rust 集成 | 原生 | FFI | FFI | FFI |
| 标准库 | 精简 | 完整 | 精简 | 精简 |
| 属性查找 | Slot (O(1)) | `__dict__` (O(1) hash) | 属性表 | 表查找 |
| C API | 不兼容 | 兼容 | 不兼容 | 兼容 |

## 适用场景

- 游戏脚本系统
- 配置和规则引擎
- 插件系统
- 数据处理管道
- 自动化测试
- 嵌入式应用脚本化
