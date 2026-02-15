# Task 038: Async/Await Support

## Status
✅ Completed

## Description
Add support for async functions and await expressions, enabling asynchronous programming patterns in QuickPython.

## Implementation

### 1. Added Tokio dependency
- Added `tokio = { version = "1", features = ["full"] }` to Cargo.toml
- Prepared for future async runtime integration

### 2. Extended Value enum
- Added `Coroutine(Function, Vec<Value>)` variant to represent async function calls
- Coroutines store the async function and its captured arguments
- Updated Debug, is_truthy, and PartialEq implementations

### 3. Modified Function struct
- Added `is_async: bool` field to distinguish async from sync functions
- Async functions return Coroutine objects when called

### 4. Added bytecode instructions
- Modified `MakeFunction` to include `is_async` parameter
- Added `Await` instruction to execute coroutines

### 5. Compiler changes
- Added support for `AsyncFunctionDef` statements
- Added support for `Await` expressions
- Async functions are compiled similarly to sync functions but marked with `is_async: true`

### 6. VM execution
- Modified `Call` instruction: async functions return Coroutine objects instead of executing immediately
- Added `Await` instruction handler: creates a new frame to execute the coroutine synchronously
- Updated `print_value_inline` and `type_name` to handle Coroutine values

### 7. Serializer
- Added error handling for `Await` instruction (not yet serializable)

## Tests
Added 8 comprehensive tests:
1. `test_async_function_basic` - Basic async function with string concatenation
2. `test_async_function_with_computation` - Async function with arithmetic
3. `test_async_function_returns_coroutine` - Verify calling async function returns coroutine
4. `test_await_coroutine` - Await a coroutine to get its result
5. `test_async_function_with_multiple_statements` - Multi-statement async function
6. `test_await_non_coroutine_error` - Error handling for awaiting non-coroutine
7. `test_async_function_with_conditionals` - Async function with if/else
8. `test_async_function_nested_calls` - Nested async function calls with await

## Example Usage

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

## Notes
- Current implementation executes coroutines synchronously
- The Tokio runtime is included but not yet utilized
- Future enhancements could add true async execution with Tokio
- Coroutines must be awaited to get their result
- Attempting to await a non-coroutine raises TypeError

## Test Results
All 247 tests pass, including 8 new async/await tests.
