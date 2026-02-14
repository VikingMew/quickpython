# Task 025: Exception Type Hierarchy

## Status
- [x] Completed

## Priority
High - blocking CI (examples/09_exceptions.py fails)

## Description
Implement exception type hierarchy so that `except Exception:` can catch all exception types, not just exact matches.

## Current Issue
```python
try:
    raise ValueError("Some error")
except Exception:  # Should catch ValueError but doesn't
    print("Caught an exception")
```

Currently fails with: `Error: Exception(ExceptionValue { exception_type: ValueError, ... })`

## Root Cause
The exception type matching uses exact equality (`Eq` instruction), so `except Exception:` only catches exceptions of type `Exception`, not its subclasses like `ValueError`, `TypeError`, etc.

## Implementation

### 1. Value Changes (`src/value.rs`)
Add methods to `ExceptionType`:
- `from_i32(i32) -> Option<ExceptionType>` - convert from integer ID
- `matches(&ExceptionType) -> bool` - check if exception matches handler type
  - `Exception` matches all types (base class)
  - Other types require exact match

### 2. Bytecode Changes (`src/bytecode.rs`)
Add new instruction:
```rust
MatchException,  // Check if exception matches handler type (supports inheritance)
```

### 3. Compiler Changes (`src/compiler.rs`)
In `compile_try_except()`, replace:
```rust
bytecode.push(Instruction::GetExceptionType);
bytecode.push(Instruction::PushInt(expected_type.as_i32()));
bytecode.push(Instruction::Eq);
```

With:
```rust
bytecode.push(Instruction::PushInt(expected_type.as_i32()));
bytecode.push(Instruction::MatchException);
```

### 4. VM Changes (`src/vm.rs`)
Implement `MatchException` instruction:
- Pop handler type (int) from stack
- Peek exception object from stack
- Use `ExceptionType::matches()` to check compatibility
- Push boolean result

### 5. Serializer Changes (`src/serializer.rs`)
Add serialization/deserialization for `MatchException` instruction.

## Test Cases

```python
# Test 1: Exception catches ValueError
try:
    raise ValueError("test")
except Exception:
    print("Caught")  # Should print

# Test 2: Exception catches all types
for exc_type in [ValueError, TypeError, IndexError, KeyError]:
    try:
        raise exc_type("test")
    except Exception:
        print("Caught")  # Should print for all

# Test 3: Specific handler still works
try:
    raise ValueError("test")
except ValueError:
    print("Caught ValueError")  # Should print

# Test 4: Wrong handler doesn't catch
try:
    raise ValueError("test")
except TypeError:
    print("Caught TypeError")  # Should NOT print
```

## Verification
- [x] `cargo test` - all tests pass (131 tests)
- [x] `cargo run -- run examples/09_exceptions.py` - PASS
- [x] Unit tests for exception hierarchy (4 tests added)
- [x] `cargo clippy -- -D warnings` - no warnings

## Notes
- Python's exception hierarchy: `Exception` is the base class for all built-in exceptions
- Our simplified hierarchy: `Exception` catches everything, others are exact match
- Future: Could implement multi-level hierarchy (e.g., `OSError` as base for file errors)

## Dependencies
None - standalone feature
