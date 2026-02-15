# Task 043: Implement Generators and `yield`

**Status**: Blocked - Requires advanced VM features
**Created**: 2026-02-15
**Priority**: High
**Blocked by**: Need to implement frame state preservation and resumable execution

## Overview

Implement Python generators using the `yield` keyword, allowing functions to produce a sequence of values lazily rather than computing them all at once.

## Background

Generators are a powerful feature for creating iterators. They use `yield` to produce values one at a time, maintaining their state between calls.

```python
def count_up_to(n):
    i = 0
    while i < n:
        yield i
        i += 1

for num in count_up_to(5):
    print(num)  # Prints 0, 1, 2, 3, 4
```

## Requirements

### 1. Add Generator Value Type
```rust
pub enum Value {
    Generator(Rc<RefCell<Generator>>),
}

pub struct Generator {
    pub function: Function,
    pub frame: Frame,
    pub ip: usize,
    pub finished: bool,
}
```

### 2. Bytecode Instructions
- `Yield`: Yield a value from generator
- `YieldFrom`: Yield from another generator (optional, for `yield from`)
- `ResumeGenerator`: Resume generator execution

### 3. Compiler Support
- Detect if function contains `yield` (scan AST)
- Mark function as generator in `MakeFunction` instruction
- Compile `yield` expressions

### 4. VM Execution
- Calling a generator function returns a generator object (not executing it)
- Generator maintains state: locals, IP, stack
- `next()` resumes execution until next `yield` or return
- Raise `StopIteration` when generator exhausted
- Integrate with `GetIter` and `ForIter` instructions

### 5. Update Function Type
Add `is_generator: bool` field to `Function` struct.

## Key Implementation Notes

- **Generator State**: Must preserve local variables, IP, and stack between yields
- **StopIteration**: Raised when generator exhausted; `for` loops catch this automatically
- **Generator vs Function**: Calling generator function returns generator object, not executing it
- **Generator Expressions**: `(x*2 for x in range(10))` - similar to list comprehensions
- **yield from** (optional): Can be implemented later as enhancement

## Test Cases

Key scenarios to test:
- Basic generator with yield in loop
- Manual iteration with `next()`
- StopIteration when exhausted
- Generator with return statement
- Generator with arguments
- Generator expressions (optional)
- State preservation between yields
- Nested generators
- `yield` without value (yields None)

## Success Criteria

- [ ] Functions with `yield` create generator objects
- [ ] Generators can be iterated with `for` loops
- [ ] `next()` function works on generators
- [ ] Generator state is preserved between yields
- [ ] StopIteration raised when exhausted
- [ ] All test cases pass
- [ ] Code passes `cargo fmt` and `cargo clippy`

## References

- PEP 255: Simple Generators
- PEP 342: Coroutines via Enhanced Generators
- Python docs: https://docs.python.org/3/reference/expressions.html#yield-expressions
