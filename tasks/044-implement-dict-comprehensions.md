# Task 044: Implement Dictionary Comprehensions

**Status**: Completed
**Created**: 2026-02-15
**Completed**: 2026-02-15
**Priority**: Medium

## Overview

Implement Python's dictionary comprehension syntax for creating dictionaries concisely.

## Background

Dictionary comprehensions provide a concise way to create dictionaries, similar to list comprehensions but producing key-value pairs.

```python
# Basic dictionary comprehension
squares = {x: x**2 for x in range(5)}
# Result: {0: 0, 1: 1, 2: 4, 3: 9, 4: 16}

# With condition
even_squares = {x: x**2 for x in range(10) if x % 2 == 0}

# Invert a dictionary
inverted = {v: k for k, v in original.items()}
```

## Requirements

### 1. Compiler Support
Handle `ast::Expr::DictComp` in expression compilation:
- Create empty dict with `BuildDict(0)`
- Compile comprehension loop(s)
- For each iteration:
  - Duplicate dict reference (`Dup`)
  - Compile key expression
  - Compile value expression
  - Add to dict with `SetItem`

### 2. Desugaring Strategy
Dictionary comprehensions are syntactic sugar - desugar to:
1. `BuildDict(0)` - Create empty dict
2. Loop with `GetIter` and `ForIter`
3. `Dup` - Duplicate dict reference
4. Compile key and value expressions
5. `SetItem` - Add key-value pair to dict

**No new bytecode instructions needed!**

### 3. Handle Nested Loops and Conditions
- Multiple `for` clauses create nested loops
- Multiple `if` clauses are ANDed together
- Example: `{x+y: x*y for x in range(3) for y in range(3) if x != y}`

## Key Implementation Notes

- **No New Instructions**: All existing instructions can be reused
- **Key Uniqueness**: If multiple iterations produce same key, last value wins
- **Nested Loops**: `for x ... for y ...` creates nested loops
- **Multiple Conditions**: `if cond1 if cond2` is equivalent to `if cond1 and cond2`
- **Performance**: Pre-allocating dict size could be an optimization

## Test Cases

Key scenarios to test:
- Basic comprehension: `{x: x*2 for x in range(5)}`
- With condition: `{x: x**2 for x in range(10) if x % 2 == 0}`
- String keys: `{word: len(word) for word in words}`
- From tuples: `{k: v for k, v in pairs}`
- Invert dict: `{v: k for k, v in original.items()}`
- Nested loops: `{x*10+y: x+y for x in range(3) for y in range(3)}`
- Multiple conditions: `{x: x**2 for x in range(20) if x % 2 == 0 if x % 3 == 0}`
- Empty result: `{x: x for x in range(10) if x > 100}`
- Complex expressions: `{str(x): x*x+x for x in range(5)}`
- Duplicate keys: `{x % 3: x for x in range(10)}` (last wins)

## Success Criteria

- [x] Basic dictionary comprehensions work
- [x] Comprehensions with conditions work
- [x] Nested loops work
- [x] Multiple conditions work
- [x] String keys and complex expressions work
- [x] Duplicate keys handled correctly (last wins)
- [x] Empty comprehensions work
- [x] All test cases pass
- [x] Code passes `cargo fmt` and `cargo clippy`

## References

- PEP 274: Dict Comprehensions
- Python docs: https://docs.python.org/3/tutorial/datastructures.html#dictionaries
