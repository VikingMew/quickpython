# Task 037: List Comprehensions

## Status
- [ ] Not started

## Priority
Low-Medium - convenient but can be worked around with loops

## Description
Implement list comprehensions: `[expr for var in iterable if condition]`

## Current Issue
```python
squares = [x*x for x in range(5)]  # Error: Unsupported expression
```

## Implementation

### 1. Compiler Changes (`src/compiler.rs`)

Handle `ListComp` expression:
```rust
ast::Expr::ListComp(comp) => {
    // [x*x for x in range(5) if x % 2 == 0]
    // Desugar to:
    // _result = []
    // for x in range(5):
    //     if x % 2 == 0:
    //         _result.append(x*x)
    
    // Create empty list
    bytecode.push(Instruction::BuildList(0));
    
    // Store in temporary (use stack position)
    let temp_var = format!("_listcomp_{}", self.temp_counter);
    self.temp_counter += 1;
    bytecode.push(Instruction::SetGlobal(temp_var.clone()));
    
    // Compile the iterator
    self.compile_expr(&comp.generators[0].iter, bytecode)?;
    
    // Start for loop
    let loop_start = bytecode.len();
    bytecode.push(Instruction::ForIter(0));  // Placeholder
    
    // Bind loop variable
    if let ast::Expr::Name(name) = &*comp.generators[0].target {
        bytecode.push(Instruction::SetGlobal(name.id.to_string()));
    }
    
    // Compile filter conditions (if any)
    for filter in &comp.generators[0].ifs {
        self.compile_expr(filter, bytecode)?;
        let jump_offset = bytecode.len();
        bytecode.push(Instruction::JumpIfFalse(0));  // Skip append if false
        // ... (patch later)
    }
    
    // Compile element expression
    self.compile_expr(&comp.elt, bytecode)?;
    
    // Append to result list
    bytecode.push(Instruction::GetGlobal(temp_var.clone()));
    bytecode.push(Instruction::CallMethod("append".to_string(), 1));
    bytecode.push(Instruction::Pop);  // Discard None return
    
    // Jump back to loop start
    bytecode.push(Instruction::Jump(loop_start));
    
    // Patch ForIter jump
    let loop_end = bytecode.len();
    bytecode[loop_start] = Instruction::ForIter(loop_end);
    
    // Load result
    bytecode.push(Instruction::GetGlobal(temp_var));
}
```

### 2. Simplified Implementation (Phase 1)

For Phase 1, only support simple comprehensions:
- Single `for` clause
- Optional single `if` clause
- No nested comprehensions

```python
# Supported:
[x*x for x in range(10)]
[x for x in items if x > 0]

# Not supported (Phase 2):
[x*y for x in range(3) for y in range(3)]  # Nested
[(x, y) for x in a for y in b]  # Multiple for
```

## Test Cases

```python
# Test 1: Basic comprehension
squares = [x*x for x in range(5)]
assert squares == [0, 1, 4, 9, 16]

# Test 2: With filter
evens = [x for x in range(10) if x % 2 == 0]
assert evens == [0, 2, 4, 6, 8]

# Test 3: String transformation
words = ["hello", "world"]
upper = [w.upper() for w in words]
assert upper == ["HELLO", "WORLD"]

# Test 4: Expression
nums = [1, 2, 3]
doubled = [n * 2 for n in nums]
assert doubled == [2, 4, 6]

# Test 5: Empty result
empty = [x for x in range(5) if x > 10]
assert empty == []

# Test 6: Nested data
matrix = [[1, 2], [3, 4]]
flattened = [x for row in matrix for x in row]  # Phase 2
# assert flattened == [1, 2, 3, 4]
```

## Alternative: Desugar to Loop

Instead of special bytecode, desugar to regular loop:
```python
[x*x for x in range(5)]

# Becomes:
_temp = []
for x in range(5):
    _temp.append(x*x)
result = _temp
```

This avoids complexity and reuses existing loop infrastructure.

## Verification
- [ ] `cargo test` - all tests pass
- [ ] Add unit tests for list comprehensions
- [ ] Test with filters
- [ ] Test empty results
- [ ] `cargo clippy -- -D warnings` - no warnings

## Dependencies
- Requires list `.append()` method (already implemented)

## Notes
- List comprehensions are more Pythonic than loops
- Can be desugared to regular loops
- Phase 1: single for + optional if
- Phase 2: nested for, multiple if, dict/set comprehensions
- Generator expressions (`(x for x in ...)`) are Phase 3
