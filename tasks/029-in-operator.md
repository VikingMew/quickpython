# Task 029: `in` Operator (Membership Test)

## Status
- [ ] Not started

## Priority
High - commonly used in loops and conditions

## Description
Implement `in` and `not in` operators for membership testing.

## Current Issue
```python
if 2 in [1, 2, 3]:  # Error: Unsupported operator
    print("found")
```

## Implementation

### 1. Bytecode Changes (`src/bytecode.rs`)
Add new instruction:
```rust
pub enum Instruction {
    // ... existing ...
    Contains,     // Check if TOS1 contains TOS (TOS1 in TOS)
    NotContains,  // Check if TOS1 not in TOS
}
```

### 2. Compiler Changes (`src/compiler.rs`)
Handle `Compare` with `In` and `NotIn` operators:
```rust
ast::Expr::Compare(cmp) => {
    // Compile left operand
    self.compile_expr(&cmp.left, bytecode)?;
    
    // For each comparison
    for (op, right) in cmp.ops.iter().zip(cmp.comparators.iter()) {
        self.compile_expr(right, bytecode)?;
        
        match op {
            ast::CmpOp::In => {
                bytecode.push(Instruction::Contains);
            }
            ast::CmpOp::NotIn => {
                bytecode.push(Instruction::NotContains);
            }
            // ... existing comparison operators ...
        }
    }
}
```

### 3. VM Changes (`src/vm.rs`)
```rust
Instruction::Contains => {
    let container = self.stack.pop()
        .ok_or_else(|| Value::error(ExceptionType::RuntimeError, "Stack underflow"))?;
    let item = self.stack.pop()
        .ok_or_else(|| Value::error(ExceptionType::RuntimeError, "Stack underflow"))?;
    
    let result = match container {
        Value::List(list) => {
            list.borrow().iter().any(|v| Self::values_equal(v, &item))
        }
        Value::Dict(dict) => {
            if let Some(key) = Self::value_to_dict_key(&item) {
                dict.borrow().contains_key(&key)
            } else {
                false
            }
        }
        Value::String(s) => {
            if let Value::String(needle) = item {
                s.contains(&needle)
            } else {
                return Err(Value::error(
                    ExceptionType::TypeError,
                    "'in' requires string for string search"
                ));
            }
        }
        _ => {
            return Err(Value::error(
                ExceptionType::TypeError,
                format!("argument of type '{}' is not iterable", Self::type_name(&container))
            ));
        }
    };
    
    self.stack.push(Value::Bool(result));
    *ip += 1;
}

Instruction::NotContains => {
    // Execute Contains, then negate
    self.execute_instruction(&Instruction::Contains, ip, globals)?;
    
    let value = self.stack.pop().unwrap();
    if let Value::Bool(b) = value {
        self.stack.push(Value::Bool(!b));
    }
}
```

### 4. Serializer Changes (`src/serializer.rs`)
```rust
Instruction::Contains => {
    writer.write_u8(0x1F)?;  // New opcode
}
Instruction::NotContains => {
    writer.write_u8(0x20)?;  // New opcode
}
```

## Test Cases

```python
# Test 1: List membership - found
assert 2 in [1, 2, 3]

# Test 2: List membership - not found
assert 4 not in [1, 2, 3]

# Test 3: String membership
assert "world" in "hello world"
assert "xyz" not in "hello world"

# Test 4: Dict membership (checks keys)
d = {"a": 1, "b": 2}
assert "a" in d
assert "c" not in d

# Test 5: Empty containers
assert 1 not in []
assert "x" not in {}
assert "x" not in ""

# Test 6: In conditions
numbers = [1, 2, 3, 4, 5]
if 3 in numbers:
    print("found")

# Test 7: In loops
for i in range(10):
    if i in [2, 4, 6, 8]:
        print(i, "is even")

# Test 8: Type errors
try:
    result = 1 in 123  # TypeError
except TypeError:
    print("caught")

# Test 9: Different types
assert 1 in [1, "1", 1.0]  # Finds int 1
assert "1" not in [1, 2, 3]  # String not in int list

# Test 10: Nested structures
matrix = [[1, 2], [3, 4]]
assert [1, 2] in matrix
```

## Supported Containers
- **List**: checks if item equals any element
- **Dict**: checks if item is a key
- **String**: checks if substring exists

## Error Cases
- `TypeError` if container is not iterable (int, float, bool, None)
- `TypeError` if searching for non-string in string

## Verification
- [ ] `cargo test` - all tests pass
- [ ] Add unit tests for list/dict/string membership
- [ ] Test error cases
- [ ] `cargo clippy -- -D warnings` - no warnings

## Dependencies
None - uses existing value comparison logic

## Notes
- For lists: uses equality comparison (`==`)
- For dicts: only checks keys, not values
- For strings: substring search
- Python's `in` operator is O(n) for lists, O(1) for dicts
