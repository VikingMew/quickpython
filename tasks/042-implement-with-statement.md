# Task 042: Implement `with` Statement (Context Managers)

**Status**: Pending
**Created**: 2026-02-15
**Priority**: High

## Overview

Implement Python's `with` statement for context management, which ensures proper resource acquisition and release (RAII pattern).

## Background

The `with` statement simplifies exception handling by encapsulating common preparation and cleanup tasks. It's commonly used for file operations, locks, and database connections.

```python
with open("file.txt") as f:
    content = f.read()
# File is automatically closed here, even if an exception occurred
```

## Requirements

### 1. Add Context Manager Protocol to Value

In `src/value.rs`, add support for context manager methods:

```rust
pub enum Value {
    // ... existing variants ...
    ContextManager {
        enter: Box<dyn Fn() -> Result<Value, Value>>,
        exit: Box<dyn Fn(Option<Value>) -> Result<Value, Value>>,
    },
}
```

Or use a simpler approach with native functions:
```rust
pub struct ContextManager {
    pub enter_fn: NativeFunction,
    pub exit_fn: NativeFunction,
}
```

### 2. Add Bytecode Instructions

In `src/bytecode.rs`:
```rust
pub enum Instruction {
    // ... existing instructions ...
    SetupWith(usize),    // Setup with block, param is exit handler offset
    WithCleanup,         // Call __exit__ method
    PopWith,             // Pop with block from stack
}
```

### 3. Compiler Support

In `src/compiler.rs`, handle `ast::Stmt::With`:

```rust
fn compile_with(&mut self, items: &[ast::WithItem], body: &[ast::Stmt]) {
    for item in items {
        // Compile context expression
        self.compile_expr(&item.context_expr);
        
        // Call __enter__() method
        self.bytecode.push(Instruction::CallMethod("__enter__".to_string(), 0));
        
        // Store result in optional variable
        if let Some(var) = &item.optional_vars {
            self.compile_store(var);
        } else {
            self.bytecode.push(Instruction::Pop);
        }
        
        // Setup with block
        let exit_offset = self.bytecode.len();
        self.bytecode.push(Instruction::SetupWith(0)); // Placeholder
        
        // Compile body
        for stmt in body {
            self.compile_stmt(stmt);
        }
        
        // Cleanup
        self.bytecode.push(Instruction::PopWith);
        self.bytecode.push(Instruction::WithCleanup);
        
        // Patch offset
        if let Instruction::SetupWith(ref mut offset) = self.bytecode[exit_offset] {
            *offset = self.bytecode.len();
        }
    }
}
```

### 4. VM Execution

In `src/vm.rs`:

```rust
// Add with block stack
pub struct VM {
    // ... existing fields ...
    with_stack: Vec<WithBlock>,
}

struct WithBlock {
    context_manager: Value,
    handler_offset: usize,
}

// In execute_instruction:
Instruction::SetupWith(handler_offset) => {
    let context_manager = self.stack.last().cloned()
        .ok_or_else(|| Value::error(ExceptionType::RuntimeError, "Stack underflow"))?;
    
    self.with_stack.push(WithBlock {
        context_manager,
        handler_offset: *handler_offset,
    });
    
    *ip += 1;
}

Instruction::PopWith => {
    self.with_stack.pop()
        .ok_or_else(|| Value::error(ExceptionType::RuntimeError, "No with block to pop"))?;
    *ip += 1;
}

Instruction::WithCleanup => {
    if let Some(with_block) = self.with_stack.last() {
        // Call __exit__(exc_type, exc_value, traceback)
        // For now, pass None, None, None (no exception)
        self.stack.push(with_block.context_manager.clone());
        self.stack.push(Value::None);
        self.stack.push(Value::None);
        self.stack.push(Value::None);
        
        self.execute_instruction(
            &Instruction::CallMethod("__exit__".to_string(), 3),
            ip,
            globals
        )?;
        
        // Pop the result (we don't use it for now)
        self.stack.pop();
    }
    *ip += 1;
}
```

### 5. Built-in File Context Manager

In `src/builtins/mod.rs` or new `src/builtins/io.rs`:

```rust
pub struct File {
    path: String,
    content: String,
    closed: bool,
}

impl File {
    fn enter(&mut self) -> Result<Value, Value> {
        // Return self
        Ok(Value::File(Rc::new(RefCell::new(self.clone()))))
    }
    
    fn exit(&mut self, exc_type: Value, exc_val: Value, exc_tb: Value) -> Result<Value, Value> {
        // Close the file
        self.closed = true;
        Ok(Value::None)
    }
    
    fn read(&self) -> Result<Value, Value> {
        if self.closed {
            return Err(Value::error(ExceptionType::ValueError, "I/O operation on closed file"));
        }
        Ok(Value::String(self.content.clone()))
    }
}

fn builtin_open(args: Vec<Value>) -> Result<Value, Value> {
    if args.is_empty() {
        return Err(Value::error(ExceptionType::TypeError, "open() missing required argument: 'file'"));
    }
    
    let path = args[0].as_string()?;
    let mode = if args.len() > 1 {
        args[1].as_string()?
    } else {
        "r".to_string()
    };
    
    // Read file content
    let content = std::fs::read_to_string(&path)
        .map_err(|e| Value::error(ExceptionType::IOError, &format!("Cannot open file: {}", e)))?;
    
    Ok(Value::File(Rc::new(RefCell::new(File {
        path,
        content,
        closed: false,
    }))))
}
```

## Test Cases

```rust
#[test]
fn test_with_statement_basic() {
    let mut ctx = Context::new();
    ctx.eval(r#"
# Mock file object for testing
class MockFile:
    def __init__(self):
        self.closed = False
        self.content = "test content"
    
    def __enter__(self):
        return self
    
    def __exit__(self, exc_type, exc_val, exc_tb):
        self.closed = True
        return False
    
    def read(self):
        return self.content

f = MockFile()
with f as file:
    content = file.read()

result = f.closed
"#).unwrap();
    assert_eq!(ctx.get("result"), Some(Value::Bool(true)));
    assert_eq!(ctx.get("content"), Some(Value::String("test content".to_string())));
}

#[test]
fn test_with_statement_exception() {
    let mut ctx = Context::new();
    let result = ctx.eval(r#"
class MockFile:
    def __init__(self):
        self.closed = False
    
    def __enter__(self):
        return self
    
    def __exit__(self, exc_type, exc_val, exc_tb):
        self.closed = True
        return False

f = MockFile()
try:
    with f as file:
        raise ValueError("test error")
except ValueError:
    pass

result = f.closed
"#);
    // File should be closed even though exception was raised
    assert!(result.is_ok());
}

#[test]
fn test_with_statement_no_as() {
    let mut ctx = Context::new();
    ctx.eval(r#"
class Counter:
    def __init__(self):
        self.count = 0
    
    def __enter__(self):
        self.count += 1
        return self
    
    def __exit__(self, exc_type, exc_val, exc_tb):
        self.count += 1
        return False

counter = Counter()
with counter:
    pass

result = counter.count
"#).unwrap();
    assert_eq!(ctx.get("result"), Some(Value::Int(2)));
}

#[test]
fn test_nested_with_statements() {
    let mut ctx = Context::new();
    ctx.eval(r#"
class Resource:
    def __init__(self, name):
        self.name = name
        self.opened = False
        self.closed = False
    
    def __enter__(self):
        self.opened = True
        return self
    
    def __exit__(self, exc_type, exc_val, exc_tb):
        self.closed = True
        return False

r1 = Resource("outer")
r2 = Resource("inner")

with r1:
    with r2:
        pass

result1 = r1.closed
result2 = r2.closed
"#).unwrap();
    assert_eq!(ctx.get("result1"), Some(Value::Bool(true)));
    assert_eq!(ctx.get("result2"), Some(Value::Bool(true)));
}
```

## Implementation Notes

1. **Context Manager Protocol**:
   - `__enter__()`: Called when entering the with block, returns value to bind to `as` variable
   - `__exit__(exc_type, exc_val, exc_tb)`: Called when exiting, receives exception info if any

2. **Exception Handling**:
   - `__exit__` is called even if an exception occurs in the with block
   - If `__exit__` returns `True`, the exception is suppressed
   - If `__exit__` returns `False` or `None`, the exception propagates

3. **Multiple Context Managers**:
   - `with a, b:` is equivalent to `with a: with b:`
   - Each context manager's `__exit__` is called in reverse order

4. **Integration with Try/Finally**:
   - `with` is essentially syntactic sugar for try/finally with `__enter__`/`__exit__` calls

## Success Criteria

- [ ] `with` statement compiles successfully
- [ ] `__enter__` is called when entering the block
- [ ] `__exit__` is called when exiting the block (normal or exception)
- [ ] `as` variable binding works correctly
- [ ] Nested `with` statements work
- [ ] Exception handling works correctly
- [ ] All test cases pass
- [ ] Code passes `cargo fmt` and `cargo clippy`

## Related Tasks

- Task 041: Implement `is` and `is not` operators
- Task 043: Implement generators and `yield`
- Task 044: Implement dictionary comprehensions

## References

- PEP 343: The "with" Statement
- Python documentation: https://docs.python.org/3/reference/compound_stmts.html#with
