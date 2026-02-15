# Task 027: Augmented Assignment Operators

## Status
- [ ] Not started

## Priority
High - basic syntax needed for most real-world code

## Description
Implement augmented assignment operators: `+=`, `-=`, `*=`, `/=`, `%=`

## Current Issue
```python
x = 10
x += 5  # Error: Unsupported statement: AugAssign
```

## Implementation

### 1. Bytecode Changes (`src/bytecode.rs`)
No new instructions needed - desugar to regular operations:
```python
x += 5  →  x = x + 5
```

### 2. Compiler Changes (`src/compiler.rs`)
Add `AugAssign` statement handling:
```rust
ast::Stmt::AugAssign(aug) => {
    // Desugar: x += 5  →  x = x + 5
    // 1. Load current value of target
    // 2. Compile the value expression
    // 3. Apply the operator
    // 4. Store back to target
    
    match &*aug.target {
        ast::Expr::Name(name) => {
            // For simple variables
            self.compile_expr(&aug.target, bytecode)?;
            self.compile_expr(&aug.value, bytecode)?;
            
            match aug.op {
                ast::Operator::Add => bytecode.push(Instruction::Add),
                ast::Operator::Sub => bytecode.push(Instruction::Sub),
                ast::Operator::Mult => bytecode.push(Instruction::Mul),
                ast::Operator::Div => bytecode.push(Instruction::Div),
                ast::Operator::Mod => bytecode.push(Instruction::Mod),
                _ => return Err(format!("Unsupported augmented operator: {:?}", aug.op)),
            }
            
            bytecode.push(Instruction::SetGlobal(name.id.to_string()));
        }
        _ => return Err("Augmented assignment only supports simple variables".to_string()),
    }
    Ok(())
}
```

### 3. VM Changes (`src/vm.rs`)
No changes needed - uses existing instructions.

## Supported Operators
- `+=` - addition
- `-=` - subtraction
- `*=` - multiplication
- `/=` - division
- `%=` - modulo

## Test Cases

```python
# Test 1: Integer addition
x = 10
x += 5
assert x == 15

# Test 2: Integer subtraction
x = 10
x -= 3
assert x == 7

# Test 3: Multiplication
x = 5
x *= 3
assert x == 15

# Test 4: Division
x = 20
x /= 4
assert x == 5

# Test 5: Modulo
x = 17
x %= 5
assert x == 2

# Test 6: Float operations
x = 10.0
x += 2.5
assert x == 12.5

# Test 7: String concatenation
s = "Hello"
s += " World"
assert s == "Hello World"

# Test 8: In loop
total = 0
for i in range(5):
    total += i
assert total == 10
```

## Limitations (Phase 1)
- Only supports simple variable targets (not `obj.attr` or `list[i]`)
- List/dict item assignment (`lst[0] += 1`) - Phase 2

## Verification
- [ ] `cargo test` - all tests pass
- [ ] Add unit tests for each operator
- [ ] Test with int, float, string types
- [ ] `cargo clippy -- -D warnings` - no warnings

## Dependencies
None - uses existing bytecode instructions

## Notes
- Augmented assignment is syntactic sugar, not a new operation
- Python evaluates the target expression only once (matters for `obj.method()[i] += 1`)
- For Phase 1, we only support simple variable names
