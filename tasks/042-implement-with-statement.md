# Task 042: Implement `with` Statement (Context Managers)

**Status**: Pending
**Created**: 2026-02-15
**Priority**: High

## Overview

Implement Python's `with` statement for context management, ensuring proper resource acquisition and release (RAII pattern).

## Background

The `with` statement simplifies exception handling by encapsulating common preparation and cleanup tasks.

```python
with open("file.txt") as f:
    content = f.read()
# File is automatically closed here, even if an exception occurred
```

## Requirements

### 1. Context Manager Protocol
Add support for `__enter__()` and `__exit__()` methods:
- `__enter__()`: Called when entering the with block, returns value to bind to `as` variable
- `__exit__(exc_type, exc_val, exc_tb)`: Called when exiting, receives exception info if any

### 2. Bytecode Instructions
- `SetupWith(usize)`: Setup with block, parameter is exit handler offset
- `WithCleanup`: Call `__exit__` method
- `PopWith`: Pop with block from stack

### 3. Compiler Support
Handle `ast::Stmt::With`:
- Compile context expression
- Call `__enter__()` method
- Store result in optional variable (if `as` clause present)
- Setup with block
- Compile body
- Cleanup with `__exit__()` call

### 4. VM Execution
- Maintain a with block stack
- Call `__enter__()` when entering block
- Call `__exit__()` when exiting (even if exception occurs)
- Handle exception suppression if `__exit__()` returns True

### 5. Built-in File Context Manager
Implement file object with context manager support for testing.

## Key Implementation Notes

- `__exit__` is called even if an exception occurs in the with block
- If `__exit__` returns `True`, the exception is suppressed
- Multiple context managers: `with a, b:` is equivalent to `with a: with b:`
- Each context manager's `__exit__` is called in reverse order
- Integration with try/finally: `with` is essentially syntactic sugar

## Test Cases

Key scenarios to test:
- Basic with statement with `as` clause
- With statement without `as` clause
- Exception handling (ensure `__exit__` is called)
- Nested with statements
- Multiple context managers in one statement
- Exception suppression by `__exit__` returning True

## Success Criteria

- [ ] `with` statement compiles successfully
- [ ] `__enter__` and `__exit__` are called correctly
- [ ] `as` variable binding works
- [ ] Nested `with` statements work
- [ ] Exception handling works correctly
- [ ] All test cases pass
- [ ] Code passes `cargo fmt` and `cargo clippy`

## References

- PEP 343: The "with" Statement
- Python docs: https://docs.python.org/3/reference/compound_stmts.html#with
