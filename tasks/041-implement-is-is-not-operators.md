# Task 041: Implement `is` and `is not` Identity Operators

**Status**: Pending
**Created**: 2026-02-15
**Priority**: Medium

## Overview

Implement Python's identity operators `is` and `is not` for checking object identity (pointer equality) rather than value equality.

## Background

Python has two types of equality:
- **Value equality** (`==`, `!=`): Compares the values of objects
- **Identity equality** (`is`, `is not`): Compares whether two references point to the same object

Example:
```python
a = [1, 2, 3]
b = [1, 2, 3]
c = a

a == b  # True (same values)
a is b  # False (different objects)
a is c  # True (same object)
```

## Requirements

### 1. Add Bytecode Instructions
- `Is`: Check if two objects are the same (identity)
- `IsNot`: Check if two objects are not the same

### 2. Compiler Support
Handle `ast::CmpOp::Is` and `ast::CmpOp::IsNot` in comparison compilation.

### 3. VM Execution
Implement identity check using pointer comparison:
- For `None`: Always the same object
- For reference types (List, Dict, Tuple): Use `Rc::ptr_eq()`
- For small integers: Optional caching optimization (-5 to 256)
- Different types or values: Not identical

### 4. Serialization Support
Add serialization/deserialization for new instructions.

## Key Implementation Notes

- Use `Rc::ptr_eq()` to compare if two `Rc` pointers point to the same allocation
- `None is None` should always return `True`
- Small integer caching is optional (CPython caches -5 to 256)
- String interning is optional for now

## Test Cases

Key scenarios to test:
- `None is None` → True
- Different list instances with same values: `is` → False, `==` → True
- Same list referenced twice: `is` → True
- Different types: `is` → False
- Small integers (optional caching behavior)

## Success Criteria

- [ ] `is` and `is not` operators compile and execute correctly
- [ ] Identity checks work for all value types
- [ ] All test cases pass
- [ ] Code passes `cargo fmt` and `cargo clippy`

## References

- Python docs: https://docs.python.org/3/reference/expressions.html#is
- PEP 8: "Comparisons to singletons like None should always be done with `is` or `is not`"
