# Task 045: Implement `pass` Statement

**Status**: Completed
**Created**: 2026-02-15
**Completed**: 2026-02-15
**Priority**: Low

## Overview

Implement Python's `pass` statement, which is a null operation that does nothing. It's used as a placeholder where syntactically a statement is required but no action is needed.

## Background

The `pass` statement is useful in several scenarios:
- Empty function/class definitions during development
- Empty exception handlers
- Empty loop bodies
- Placeholder for future code

```python
def not_implemented_yet():
    pass

try:
    risky_operation()
except Exception:
    pass  # Silently ignore errors

for i in range(10):
    pass  # Do nothing 10 times
```

## Requirements

### 1. Compiler Support
Handle `ast::Stmt::Pass` in statement compilation:
- **Option 1 (Preferred)**: Emit no bytecode - `pass` is a no-op
- **Option 2**: Emit explicit `Pass` instruction that does nothing

### 2. Bytecode Instruction (Optional)
If using explicit instruction approach:
```rust
pub enum Instruction {
    Pass,  // No-op instruction
}
```

### 3. VM Execution (if using Pass instruction)
Simply advance IP, do nothing else.

## Key Implementation Notes

- **No-op Implementation**: Simplest approach is to not emit any bytecode for `pass`
- **Syntax Requirement**: `pass` is required where Python syntax demands a statement
- **Empty Blocks**: Not allowed in Python, `pass` serves as placeholder
- **Difference from `continue`**: `pass` does nothing and continues to next statement; `continue` skips rest of loop body

## Test Cases

Key scenarios to test:
- `pass` in function body
- `pass` in if/elif/else blocks
- `pass` in for loop
- `pass` in while loop
- `pass` in try/except blocks
- Multiple `pass` statements
- `pass` only function (returns None)
- `pass` in nested blocks
- `pass` mixed with other statements

## Success Criteria

- [x] `pass` statement compiles without errors
- [x] Functions with only `pass` return `None`
- [x] `pass` works in all control flow blocks
- [x] All test cases pass
- [x] Code passes `cargo fmt` and `cargo clippy`

## References

- Python docs: https://docs.python.org/3/reference/simple_stmts.html#the-pass-statement
- PEP 8: Use `pass` for empty code blocks
