# Task 038: Async/Await Support

## Status
- [ ] Not started

## Priority
Medium-High - 重要的现代 Python 特性，直接基于 Tokio 实现

## Description
Implement `async`/`await` syntax for asynchronous programming support.

## Current Issue
```python
async def fetch_data():
    result = await api_call()
    return result

# Error: Unsupported async syntax
```

## Use Cases
- Asynchronous I/O operations
- Concurrent task execution
- Integration with async libraries (aiohttp, asyncio)
- Non-blocking API calls

## Implementation Overview

### 使用 Tokio 运行时

**核心思路**: 直接将 Python 的 async/await 映射到 Rust/Tokio 的 async/await，无需自己实现事件循环。

```
Python async def  →  Rust async fn  →  Tokio Runtime
Python await      →  Rust .await     →  Tokio scheduler
```

### 1. 依赖添加 (`Cargo.toml`)
```toml
[dependencies]
tokio = { version = "1", features = ["full"] }
```

### 2. Value Changes (`src/value.rs`)
```rust
use tokio::task::JoinHandle;

pub enum Value {
    // ... existing ...
    
    // Python 协程直接映射到 Tokio 任务
    Coroutine(JoinHandle<Result<Value, Value>>),
}
```

### 3. Bytecode Changes (`src/bytecode.rs`)
```rust
pub enum Instruction {
    // ... existing ...
    
    // Async support
    MakeCoroutine,           // Mark function as async
    Await,                   // Await a coroutine
}
```

### 4. Context Changes (`src/context.rs`)
```rust
impl Context {
    // 现有同步 API
    pub fn eval(&mut self, source: &str) -> Result<Value, String> {
        let bytecode = Compiler::compile(source)?;
        self.vm.execute(&bytecode, &mut self.globals)
            .map_err(|e| format!("{:?}", e))
    }
    
    // 新增异步 API
    pub async fn eval_async(&mut self, source: &str) -> Result<Value, String> {
        let bytecode = Compiler::compile(source)?;
        self.vm.execute_async(&bytecode, &mut self.globals).await
            .map_err(|e| format!("{:?}", e))
    }
}
```

### 5. VM Changes (`src/vm.rs`)
```rust
impl VM {
    // 异步执行方法
    pub async fn execute_async(
        &mut self,
        bytecode: &ByteCode,
        globals: &mut HashMap<String, Value>,
    ) -> Result<Value, Value> {
        // 类似 execute()，但可以 .await
        // ...
    }
}

// Await 指令处理
Instruction::Await => {
    let coroutine = self.stack.pop()
        .ok_or_else(|| Value::error(ExceptionType::RuntimeError, "Stack underflow"))?;
    
    match coroutine {
        Value::Coroutine(join_handle) => {
            // 直接 await Tokio 任务
            let result = join_handle.await
                .map_err(|e| Value::error(ExceptionType::RuntimeError, format!("Task failed: {}", e)))?;
            
            self.stack.push(result?);
        }
        _ => return Err(Value::error(
            ExceptionType::TypeError,
            "object cannot be awaited"
        )),
    }
    *ip += 1;
}
```

### 6. Compiler Changes (`src/compiler.rs`)
```rust
// Handle async function definition
ast::Stmt::AsyncFunctionDef(func) => {
    // 编译为标记了 async 的函数
    self.compile_function(func, bytecode, true)?;
    bytecode.push(Instruction::MakeCoroutine);
}

// Handle await expression
ast::Expr::Await(await_expr) => {
    self.compile_expr(&await_expr.value, bytecode)?;
    bytecode.push(Instruction::Await);
}
```

## Test Cases

```python
# Test 1: Basic async function
async def hello():
    return "Hello"

result = await hello()
assert result == "Hello"

# Test 2: Await chain
async def fetch():
    return 42

async def process():
    data = await fetch()
    return data * 2

result = await process()
assert result == 84

# Test 3: Multiple awaits
async def multi():
    a = await async_func1()
    b = await async_func2()
    return a + b

# Test 4: Error handling
async def failing():
    raise ValueError("error")

try:
    await failing()
except ValueError:
    print("caught")

# Test 5: Async with regular code
async def mixed():
    x = 10  # Regular code
    y = await async_call()  # Async
    return x + y
```

## Implementation Phases

### Phase 1: 基础异步支持
- 添加 Tokio 依赖
- 实现 `Context::eval_async()` 和 `VM::execute_async()`
- 支持 `async def` 和 `await` 语法
- 单个协程执行

### Phase 2: 并发执行
- 实现 `gather()` 函数（使用 `tokio::join!`）
- 支持 `tokio::spawn` 创建并发任务
- 任务取消和超时

### Phase 3: 异步 I/O 集成
- 集成 Rust 异步库（reqwest, tokio::fs）
- 提供 Python 异步 I/O API
- 异步 HTTP 客户端模块
- 异步文件操作模块

### Phase 4: 高级语法
- `async for` 循环
- `async with` 上下文管理器
- 异步生成器

## 使用 Tokio 的优势

1. **成熟稳定**: Tokio 是 Rust 生态最成熟的异步运行时，经过大量生产验证
2. **真正异步**: 支持真正的非阻塞 I/O，不是语法糖
3. **高性能**: Tokio 调度器经过高度优化，性能接近原生
4. **丰富生态**: 直接使用 Rust 异步库（reqwest, tokio-postgres, tokio::fs 等）
5. **无需造轮子**: 不需要自己实现事件循环、状态管理、I/O 多路复用
6. **并发支持**: 天然支持并发执行多个协程
7. **跨平台**: Tokio 处理了所有平台差异（epoll, kqueue, IOCP）

## Python Semantics
- `async def` creates a coroutine function
- Calling coroutine function returns coroutine object (doesn't execute)
- `await` executes coroutine and gets result
- Can only `await` inside `async def`
- Coroutines must be awaited or scheduled

## Verification
- [ ] `cargo test` - all tests pass
- [ ] Add unit tests for async/await
- [ ] Test error cases (await outside async)
- [ ] Test nested awaits
- [ ] `cargo clippy -- -D warnings` - no warnings

## Dependencies
- Tokio runtime (`tokio = { version = "1", features = ["full"] }`)
- 可能需要 generator/yield 支持（用于异步生成器，Phase 4）

## 示例：集成 Rust 异步库

```rust
// 在扩展模块中提供异步 HTTP 客户端
pub fn create_http_module() -> Module {
    let mut m = Module::new("http");
    
    m.add_async_function("get", |args| async move {
        let url = args[0].as_string()?;
        let response = reqwest::get(url).await?;
        let text = response.text().await?;
        Ok(Value::String(text))
    });
    
    m
}
```

```python
# Python 代码可以直接使用
import http

async def fetch():
    html = await http.get("https://example.com")
    return html
```

## Notes
- 使用 Tokio 避免了自己实现事件循环的复杂性
- Python 协程直接映射到 Tokio 任务
- 可以无缝集成 Rust 异步生态
- 性能接近原生 Rust async/await
- VM 需要支持异步执行（`execute_async`）

## References
- Tokio documentation: https://tokio.rs
- PEP 492 - Coroutines with async and await syntax
- PEP 525 - Asynchronous Generators
- Rust async book: https://rust-lang.github.io/async-book/
