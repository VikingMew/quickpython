# Task 038: Async/Await Support (Complete Implementation)

## Status
✅ Completed (Full async/await with Tokio integration)

## Description
Add complete support for async functions and await expressions with true asynchronous execution using Tokio runtime.

## Implementation

### Phase 1: Basic Async/Await Syntax (Initial)

#### 1. Added Tokio dependency
- Added `tokio = { version = "1", features = ["full"] }` to Cargo.toml
- Integrated Tokio runtime for true async execution

#### 2. Extended Value enum
- Added `Coroutine(Function, Vec<Value>)` variant to represent async function calls
- Added `AsyncSleep(f64)` variant for async sleep operations
- Coroutines store the async function and its captured arguments
- Updated Debug, is_truthy, and PartialEq implementations

#### 3. Modified Function struct
- Added `is_async: bool` field to distinguish async from sync functions
- Async functions return Coroutine objects when called

#### 4. Added bytecode instructions
- Modified `MakeFunction` to include `is_async` parameter
- Added `Await` instruction to execute coroutines

#### 5. Compiler changes
- Added support for `AsyncFunctionDef` statements
- Added support for `Await` expressions
- Async functions are compiled similarly to sync functions but marked with `is_async: true`

#### 6. VM execution
- Modified `Call` instruction: async functions return Coroutine objects instead of executing immediately
- Added `Await` instruction handler with two modes:
  - For `Coroutine`: creates a new frame to execute the coroutine
  - For `AsyncSleep`: uses Tokio runtime to actually sleep asynchronously
- Updated `print_value_inline` and `type_name` to handle Coroutine and AsyncSleep values

#### 7. Serializer
- Added error handling for `Await` instruction (not yet serializable)

### Phase 2: True Async I/O with asyncio Module

#### 8. Created asyncio builtin module
- New file: `src/builtins/asyncio.rs`
- Implemented `asyncio.sleep(seconds)` function
- Returns `AsyncSleep` value that VM recognizes and executes asynchronously
- Validates arguments (must be non-negative number)

#### 9. Integrated Tokio runtime in VM
- `Await` instruction now uses `tokio::runtime::Runtime` for async operations
- `AsyncSleep` values trigger actual async sleep using `tokio::time::sleep`
- Uses `block_on` to maintain synchronous API while supporting async operations

## Tests
Added 15 comprehensive tests (254 total tests passing):

### Basic async/await tests (8):
1. `test_async_function_basic` - Basic async function with string concatenation
2. `test_async_function_with_computation` - Async function with arithmetic
3. `test_async_function_returns_coroutine` - Verify calling async function returns coroutine
4. `test_await_coroutine` - Await a coroutine to get its result
5. `test_async_function_with_multiple_statements` - Multi-statement async function
6. `test_await_non_coroutine_error` - Error handling for awaiting non-coroutine
7. `test_async_function_with_conditionals` - Async function with if/else
8. `test_async_function_nested_calls` - Nested async function calls with await

### asyncio.sleep tests (7):
9. `test_asyncio_sleep_basic` - Basic sleep with timing verification (100ms)
10. `test_asyncio_sleep_with_int` - Sleep with integer argument (0ms)
11. `test_asyncio_sleep_with_float` - Sleep with float argument (50ms)
12. `test_asyncio_sleep_negative_error` - Error on negative sleep duration
13. `test_asyncio_sleep_wrong_type_error` - Error on non-numeric argument
14. `test_asyncio_sleep_in_async_function` - Sleep inside async function (100ms)
15. `test_asyncio_sleep_multiple_awaits` - Multiple sequential sleeps (100ms total)

## Example Usage

### Basic async/await
```python
# Define an async function
async def greet(name):
    return "Hello, " + name

# Call it (returns a coroutine)
coro = greet("World")

# Await the coroutine to get the result
result = await coro  # "Hello, World"

# Or await directly
result = await greet("World")

# Nested async calls
async def inner(x):
    return x * 2

async def outer(x):
    y = await inner(x)
    return y + 1

result = await outer(5)  # 11
```

### Using asyncio.sleep for true async I/O
```python
import asyncio

# Async function with sleep
async def delayed_greeting(name, delay):
    print(f"Starting to greet {name}...")
    await asyncio.sleep(delay)  # Actually sleeps using Tokio
    print(f"Hello, {name}!")
    return f"Greeted {name}"

result = await delayed_greeting("Alice", 0.5)

# Multiple sequential sleeps
async def count_with_delays():
    for i in range(3):
        print(f"Count: {i}")
        await asyncio.sleep(0.1)
    return "Done"

result = await count_with_delays()

# Nested async calls with I/O
async def fetch_data(id):
    await asyncio.sleep(0.1)  # Simulate network delay
    return f"Data for {id}"

async def process_data(id):
    data = await fetch_data(id)
    return data.upper()

result = await process_data(42)
```

See `examples/async_sleep.py` for a complete working example.

## Notes
- ✅ Async functions return Coroutine objects when called
- ✅ Coroutines must be awaited to execute and get their result
- ✅ `asyncio.sleep()` uses Tokio runtime for true asynchronous sleep
- ✅ Sleep operations actually block for the specified duration
- ✅ Attempting to await a non-coroutine raises TypeError
- ⚠️ Currently sequential execution only (no concurrent coroutine execution yet)
- ⚠️ Each await creates a new Tokio runtime (could be optimized with a shared runtime)

## Test Results
All 254 tests pass, including 15 async/await tests with timing verification.

## Future Enhancements
- Shared Tokio runtime for better performance
- `asyncio.gather()` for concurrent execution of multiple coroutines
- `asyncio.create_task()` for background tasks
- Async context managers (`async with`)
- Async iterators (`async for`)
- More async I/O operations (file I/O, network, etc.)
