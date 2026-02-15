# CLAUDE.md

This file provides guidance to Claude Code (claude.ai/code) when working with code in this repository.

## Project Overview

QuickPython is a lightweight Python bytecode VM written in Rust. It compiles Python source to custom bytecode and executes it on a stack-based virtual machine. The project prioritizes being small, fast to start, and easy to extend over full CPython compatibility.


## Development Commands

### Building
```bash
cargo build                    # Debug build
cargo build --release          # Release build
cargo build --workspace        # Build all workspace members (includes quickpython-llm, quickpython-demo)
```

### Testing
```bash
cargo test                     # Run all tests
cargo test --lib               # Run only library tests 
cargo test async               # Run all async-related tests
cargo test --workspace         # Run tests for all workspace members
```

### Code Quality
```bash
cargo fmt                      # Format code (REQUIRED before commit)
cargo clippy --workspace -- -D warnings  # Lint code (REQUIRED before commit)
```

**IMPORTANT:** All commits must pass both `cargo fmt` and `cargo clippy --workspace -- -D warnings` with no errors. Run these commands before committing any code changes.

### Running
```bash
cargo run -- run script.py           # Execute Python file
cargo run -- compile script.py       # Compile to bytecode (.pyq)
cargo run -- compile script.py -o out.pyq
cargo run --release -- run script.py # Run with optimizations
```

### Examples
```bash
cargo run -- run examples/async_sleep.py
cargo run -- run examples/11_comprehensive.py
```

## Architecture

### Core Pipeline
```
Python Source → rustpython_parser → AST → Compiler → ByteCode → VM → Result
```

The project uses RustPython's parser for the frontend and implements a custom bytecode compiler and stack-based VM.

### Value System Design

Current implementation uses `enum Value` with `Rc<RefCell<...>>` for heap objects (not yet NaN boxing as described in spec/architecture.md). The spec describes future NaN boxing optimization, but current code is simpler:

```rust
pub enum Value {
    Int(i32),
    Float(f64),
    Bool(bool),
    None,
    String(String),
    List(Rc<RefCell<ListValue>>),
    Dict(Rc<RefCell<HashMap<DictKey, Value>>>),
    Tuple(Rc<Vec<Value>>),
    Function(Function),
    Coroutine(Function, Vec<Value>),  // async function + captured args
    AsyncSleep(f64),                  // Special marker for asyncio.sleep
    // ... more types
}
```

### Async/Await Implementation

**Two-phase approach:**
1. Async functions return `Coroutine` objects when called (not immediately executed)
2. `await` expression executes the coroutine

**Key insight:** VM maintains sync API by using Tokio's `Runtime::block_on` in Await instruction:
```rust
Instruction::Await => {
    match coroutine {
        Value::Coroutine(func, args) => {
            // Create new frame and execute synchronously
        }
        Value::AsyncSleep(seconds) => {
            // Create Tokio runtime and block_on async sleep
            let rt = Runtime::new()?;
            rt.block_on(async {
                tokio::time::sleep(Duration::from_secs_f64(seconds)).await;
            });
        }
    }
}
```

This allows:
- ✅ True async I/O (real sleep using Tokio)
- ✅ Maintains synchronous `Context::eval()` API
- ✅ No breaking changes to existing code
- ⚠️ Creates new runtime per await (could be optimized with shared runtime)

### Module System

Three types of modules:

1. **Builtin modules** (json, os, re, asyncio) - compiled into binary
   - Registered in `src/builtins/mod.rs`
   - VM checks `is_builtin_module()` during import

2. **Extension modules** - registered at runtime via `Context::register_extension_module()`
   - See `quickpython-llm/` and `quickpython-demo/` for examples
   - Extension creates Module with functions/attributes

3. **Python modules** - not yet implemented

## Common Patterns

### Adding a new builtin function

1. Add to appropriate module in `src/builtins/*.rs`:
```rust
fn my_function(args: Vec<Value>) -> Result<Value, Value> {
    // Validate args
    if args.len() != 1 {
        return Err(Value::error(ExceptionType::TypeError, "..."));
    }
    // Implementation
    Ok(Value::Int(42))
}
```

2. Register in `create_module()`:
```rust
module.attributes.insert("my_func".to_string(), Value::NativeFunction(my_function));
```

### Adding a new bytecode instruction

1. Add variant to `Instruction` enum in `src/bytecode.rs`
2. Add compiler support in `src/compiler.rs` (in appropriate `compile_*` method)
3. Add VM execution in `src/vm.rs` (in main match statement around line 700-2000)
4. Add serializer support in `src/serializer.rs` (or return error if not serializable)

### Desugaring complex syntax

List comprehensions and f-strings are **desugared** to simpler instructions during compilation rather than having dedicated bytecode. See `compile_expr()` for examples:
- List comprehensions → for loop + append
- F-strings → concatenation of str() calls

This keeps bytecode simpler at cost of larger bytecode size.

## Workspace Structure

- **quickpython/** - Main library and CLI
- **quickpython-llm/** - Extension module example (wraps OpenAI-compatible APIs)
- **quickpython-demo/** - Demo application using quickpython-llm extension

Extension modules are separate crates that create `Module` objects and register them with Context.

## Important Notes

- **Value equality:** Lists and tuples use content equality (not pointer equality)
- **String iteration:** Implemented for for-loops and list comprehensions (Task 039)
- **Type objects:** `list`, `dict`, `int`, etc. are reserved keywords (can't use as variable names)
- **Iterator safety:** VM detects list modification during iteration (raises RuntimeError)
- **Async execution:** Sequential only (no concurrent execution of multiple coroutines yet)

## Task System

Tasks are documented in `tasks/*.md`. Each task has:
- Implementation description
- Test cases
- Status (completed/pending)

When implementing new tasks:
1. Create task file in `tasks/NNN-feature-name.md`
2. Implement feature across relevant files
3. Add comprehensive tests
4. Update CHANGELOG.md
5. Mark task as completed

## Spec Documents

`spec/` contains design documents (some aspirational):
- `spec/architecture.md` - Describes NaN boxing and other future optimizations (not all implemented)
- `spec/import-system.md` - Module import mechanism
- `spec/exception-system.md` - Exception handling design

**Note:** Spec files describe ideal architecture; actual implementation is simpler.

## Programming Principles

This project follows the engineering philosophy of Linus Torvalds and John Carmack:

### Code Quality Over Cleverness
- **Simple, obvious code beats clever code** - If it's hard to understand, it's wrong
- **Readability first** - Code is read far more often than written
- **No premature optimization** - Make it work, make it right, then make it fast
- **Avoid abstraction layers** - Every layer has a cost; only add them when clearly beneficial

### Performance Through Simplicity
- **Measure, don't guess** - Profile before optimizing
- **Data structure choice matters more than algorithm tricks** - Cache-friendly data structures win
- **Keep hot paths simple** - The VM main loop should be straightforward, not clever

### Engineering Discipline
- **Fix the root cause, not symptoms** - Understand the problem before coding the solution
- **Delete code aggressively** - The best code is no code; remove unused features/abstractions
- **Small, focused changes** - Large refactors are where bugs hide
- **When in doubt, keep it simple** - Complexity is the enemy of reliability

### Specific to This Project
- **Desugaring over new bytecode** - F-strings and list comprehensions desugar to simple instructions
- **Sync API with async internals** - Use `block_on` rather than forcing callers to be async
- **Reference counting over complex GC** - Simple, predictable, deterministic
- **Content equality over pointer equality** - Lists/tuples compare by value, not identity

When adding features, ask:
1. Can this be implemented by desugaring to existing instructions?
2. Does this add a new abstraction layer, or simplify an existing one?
3. Will this still make sense when reading the code in 6 months?
4. Is there a simpler way that's "good enough"?
