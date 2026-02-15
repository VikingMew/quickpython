# Async/Await Specification

## Overview

QuickPython 支持 Python 的 async/await 语法，用于异步编程。本规格定义了协程（coroutine）、异步函数和事件循环的实现。

## 核心概念

### 1. 协程 (Coroutine)

协程是可以暂停和恢复执行的函数。

```python
async def fetch_data():
    result = await api_call()
    return result
```

**特性**：
- 使用 `async def` 定义
- 调用协程函数返回协程对象（不立即执行）
- 必须使用 `await` 或事件循环来执行
- 可以在内部使用 `await` 表达式

### 2. Await 表达式

`await` 用于等待协程完成并获取结果。

```python
result = await coroutine()
```

**规则**：
- 只能在 `async def` 函数内使用
- 暂停当前协程执行
- 等待被 await 的协程完成
- 返回协程的结果

### 3. 事件循环 (Event Loop)

QuickPython 使用 Rust 的 Tokio 运行时作为底层事件循环。

```python
# 运行协程直到完成
result = run(main())
```

**实现**: 直接映射到 Tokio 的 async/await，无需自己实现事件循环。

## 语法

### 异步函数定义

```python
async def function_name(parameters):
    # 函数体
    result = await some_coroutine()
    return result
```

### Await 表达式

```python
# 基本用法
value = await coroutine()

# 在表达式中
result = (await func1()) + (await func2())

# 多个 await
a = await fetch_a()
b = await fetch_b()
c = await process(a, b)
```

### 异步上下文管理器 (Phase 2)

```python
async with resource() as r:
    await r.process()
```

### 异步迭代器 (Phase 2)

```python
async for item in async_iterator():
    await process(item)
```

## 执行模型

### 基于 Tokio 的实现

QuickPython 的 async/await 直接映射到 Rust 的 Tokio 运行时：

```python
async def main():
    a = await task1()  # 在 Tokio 运行时中执行
    b = await task2()  # 真正的异步执行
    return a + b
```

**架构**：
```
Python async def  →  Rust async fn  →  Tokio Runtime
Python await      →  Rust .await     →  Tokio scheduler
```

**优势**：
- 无需自己实现事件循环
- 利用 Tokio 的成熟调度器
- 支持真正的异步 I/O
- 可以直接调用 Rust 异步生态

### 并发执行

```python
async def main():
    # 并发执行多个任务（使用 Tokio）
    results = await gather(task1(), task2(), task3())
    return results
```

内部使用 `tokio::join!` 或 `tokio::spawn` 实现并发。

### 异步 I/O

```python
async def fetch(url):
    # 使用 Tokio 的异步 HTTP 客户端
    response = await http.get(url)
    return response.text()
```

可以直接集成 Rust 的异步库（reqwest, tokio::fs 等）。

## 数据类型

### Coroutine 类型

```rust
use tokio::task::JoinHandle;

pub enum Value {
    // Python 协程映射到 Tokio 任务
    Coroutine(JoinHandle<Result<Value, Value>>),
}
```

Python 协程直接映射到 Tokio 的 `JoinHandle`，无需自己管理状态。

### 异步函数实现

```rust
// Python async def 编译为 Rust async fn
impl VM {
    async fn execute_async(&mut self, bytecode: &ByteCode) -> Result<Value, Value> {
        // VM 执行逻辑
        // 可以在内部 .await Rust 的异步操作
    }
}
```

## 字节码指令

```rust
pub enum Instruction {
    // 协程相关
    MakeCoroutine,           // 将函数转换为协程
    Await,                   // 等待协程完成
    
    // Phase 2: 生成器支持
    Yield(Option<usize>),    // 暂停并返回值
    YieldFrom,               // 委托给另一个生成器
    
    // Phase 2: 异步迭代
    GetAIter,                // 获取异步迭代器
    GetANext,                // 获取下一个异步值
}
```

## 错误处理

### 1. Await 在非异步函数中

```python
def regular_function():
    result = await coroutine()  # SyntaxError
```

**错误**: `SyntaxError: 'await' outside async function`

### 2. Await 非协程对象

```python
async def main():
    result = await 123  # TypeError
```

**错误**: `TypeError: object int can't be used in 'await' expression`

### 3. 协程未被 await

```python
async def task():
    return 42

def main():
    task()  # 警告：协程未被 await
```

**警告**: `RuntimeWarning: coroutine 'task' was never awaited`

## 内置函数

### Phase 1

```python
# 运行协程（简化版）
def run(coro):
    """运行协程直到完成"""
    return coro.run()
```

### Phase 2

```python
# 并发执行多个协程
async def gather(*coros):
    """并发执行多个协程，返回结果列表"""
    results = []
    for coro in coros:
        results.append(await coro)
    return results

# 创建任务
def create_task(coro):
    """将协程包装为任务"""
    return Task(coro)

# 休眠
async def sleep(seconds):
    """异步休眠"""
    # 暂停指定时间
    pass
```

## 使用示例

### 基本用法

```python
async def greet(name):
    return f"Hello, {name}"

async def main():
    message = await greet("Alice")
    print(message)

# 运行
run(main())
```

### 错误处理

```python
async def risky_operation():
    if error_condition:
        raise ValueError("Something went wrong")
    return "Success"

async def main():
    try:
        result = await risky_operation()
        print(result)
    except ValueError as e:
        print(f"Error: {e}")
```

### 多个 await

```python
async def fetch_user(id):
    # 模拟异步操作
    return {"id": id, "name": "User"}

async def fetch_posts(user_id):
    # 模拟异步操作
    return [{"title": "Post 1"}, {"title": "Post 2"}]

async def main():
    user = await fetch_user(1)
    posts = await fetch_posts(user["id"])
    return {"user": user, "posts": posts}
```

### 并发执行 (Phase 2)

```python
async def main():
    # 并发执行三个任务
    results = await gather(
        fetch_data("url1"),
        fetch_data("url2"),
        fetch_data("url3")
    )
    return results
```

## 实现方案

### 直接使用 Tokio

QuickPython 的 async/await 实现直接基于 Tokio，不做简化版本：

**优势**：
1. **成熟稳定**: Tokio 经过大量生产验证
2. **真正异步**: 支持真正的非阻塞 I/O
3. **高性能**: 接近原生 Rust 性能
4. **丰富生态**: 直接使用 Rust 异步库
5. **无需造轮子**: 不需要自己实现事件循环
6. **并发支持**: 天然支持并发执行
7. **跨平台**: Tokio 处理平台差异

### 架构设计

```rust
// Context 需要支持异步执行
impl Context {
    // 同步 API（现有）
    pub fn eval(&mut self, source: &str) -> Result<Value, String> {
        // ...
    }
    
    // 异步 API（新增）
    pub async fn eval_async(&mut self, source: &str) -> Result<Value, String> {
        let bytecode = Compiler::compile(source)?;
        self.vm.execute_async(&bytecode).await
    }
}

// VM 支持异步执行
impl VM {
    pub async fn execute_async(&mut self, bytecode: &ByteCode) -> Result<Value, Value> {
        // 可以在这里 .await Rust 的异步操作
    }
}
```

### 集成 Tokio

```toml
# Cargo.toml
[dependencies]
tokio = { version = "1", features = ["full"] }
```

```rust
// 运行异步代码
#[tokio::main]
async fn main() {
    let mut ctx = Context::new();
    let result = ctx.eval_async(r#"
        async def main():
            return await fetch_data()
        
        await main()
    "#).await.unwrap();
}
```

### 不支持的特性（初期）
- `async with` - 需要异步上下文管理器协议
- `async for` - 需要异步迭代器协议
- 异步生成器 - 需要 yield 支持
- 异步推导式 - 需要推导式 + async

## 与 Python 的差异

1. **运行时**
   - Python: asyncio 事件循环（纯 Python）
   - QuickPython: Tokio 运行时（Rust）

2. **性能**
   - Python: 解释器开销
   - QuickPython: 接近原生 Rust 性能

3. **生态集成**
   - Python: Python 异步库（aiohttp, asyncpg 等）
   - QuickPython: Rust 异步库（reqwest, tokio-postgres 等）

4. **API 兼容性**
   - Python: 完整的 asyncio API
   - QuickPython: 核心 async/await 语法，有限的 asyncio API

## 性能考虑

1. **协程开销**
   - 协程创建和切换有开销
   - 适合 I/O 密集型任务
   - 不适合 CPU 密集型任务

2. **内存使用**
   - 每个协程需要保存状态
   - 大量协程会占用内存

3. **调度开销**
   - 事件循环调度有开销
   - 简单任务可能不如同步快

## 参考

- PEP 492 - Coroutines with async and await syntax
- PEP 525 - Asynchronous Generators
- PEP 530 - Asynchronous Comprehensions
- Python asyncio documentation
