# Task 028: Logical Operators (and, or, not)

## Status
- [ ] Not started

## Priority
High - essential for control flow

## Description
Implement logical operators: `and`, `or`, `not`

## Current Issue
```python
if True and False:  # Error: Unsupported expression: BoolOp
    print("test")

if not x:  # Error: Unsupported unary operator: Not
    print("test")
```

## Implementation

### 1. Bytecode Changes (`src/bytecode.rs`)
Add new instructions:
```rust
pub enum Instruction {
    // ... existing ...
    
    // Logical operators with short-circuit evaluation
    JumpIfFalseOrPop(usize),  // If TOS is false, jump; else pop and continue
    JumpIfTrueOrPop(usize),   // If TOS is true, jump; else pop and continue
    Not,                       // Logical not: !TOS
}
```

### 2. Compiler Changes (`src/compiler.rs`)

#### `and` operator (short-circuit):
```rust
// a and b  →  if not a: result = a; else: result = b
ast::Expr::BoolOp(boolop) if boolop.op == ast::BoolOp::And => {
    // Compile first operand
    self.compile_expr(&boolop.values[0], bytecode)?;
    
    // For each additional operand
    for value in &boolop.values[1..] {
        let jump_offset = bytecode.len();
        bytecode.push(Instruction::JumpIfFalseOrPop(0));  // Placeholder
        
        self.compile_expr(value, bytecode)?;
        
        // Patch jump offset
        let end_offset = bytecode.len();
        bytecode[jump_offset] = Instruction::JumpIfFalseOrPop(end_offset);
    }
}
```

#### `or` operator (short-circuit):
```rust
// a or b  →  if a: result = a; else: result = b
ast::Expr::BoolOp(boolop) if boolop.op == ast::BoolOp::Or => {
    self.compile_expr(&boolop.values[0], bytecode)?;
    
    for value in &boolop.values[1..] {
        let jump_offset = bytecode.len();
        bytecode.push(Instruction::JumpIfTrueOrPop(0));  // Placeholder
        
        self.compile_expr(value, bytecode)?;
        
        let end_offset = bytecode.len();
        bytecode[jump_offset] = Instruction::JumpIfTrueOrPop(end_offset);
    }
}
```

#### `not` operator:
```rust
ast::UnaryOp::Not => {
    self.compile_expr(&unary.operand, bytecode)?;
    bytecode.push(Instruction::Not);
}
```

### 3. VM Changes (`src/vm.rs`)
```rust
Instruction::JumpIfFalseOrPop(offset) => {
    let value = self.stack.last()
        .ok_or_else(|| Value::error(ExceptionType::RuntimeError, "Stack underflow"))?;
    
    if Self::is_truthy(value) {
        self.stack.pop();  // Pop and continue
        *ip += 1;
    } else {
        *ip = *offset;  // Keep value, jump
    }
}

Instruction::JumpIfTrueOrPop(offset) => {
    let value = self.stack.last()
        .ok_or_else(|| Value::error(ExceptionType::RuntimeError, "Stack underflow"))?;
    
    if Self::is_truthy(value) {
        *ip = *offset;  // Keep value, jump
    } else {
        self.stack.pop();  // Pop and continue
        *ip += 1;
    }
}

Instruction::Not => {
    let value = self.stack.pop()
        .ok_or_else(|| Value::error(ExceptionType::RuntimeError, "Stack underflow"))?;
    
    let result = !Self::is_truthy(&value);
    self.stack.push(Value::Bool(result));
    *ip += 1;
}

// Helper function
fn is_truthy(value: &Value) -> bool {
    match value {
        Value::Bool(b) => *b,
        Value::None => false,
        Value::Int(i) => *i != 0,
        Value::Float(f) => *f != 0.0,
        Value::String(s) => !s.is_empty(),
        Value::List(l) => !l.borrow().is_empty(),
        Value::Dict(d) => !d.borrow().is_empty(),
        _ => true,
    }
}
```

### 4. Serializer Changes (`src/serializer.rs`)
Add opcodes for new instructions:
```rust
Instruction::JumpIfFalseOrPop(offset) => {
    writer.write_u8(0x1C)?;  // New opcode
    writer.write_u32::<LittleEndian>(*offset as u32)?;
}
Instruction::JumpIfTrueOrPop(offset) => {
    writer.write_u8(0x1D)?;  // New opcode
    writer.write_u32::<LittleEndian>(*offset as u32)?;
}
Instruction::Not => {
    writer.write_u8(0x1E)?;  // New opcode
}
```

## Test Cases

```python
# Test 1: and - both true
assert (True and True) == True

# Test 2: and - first false
assert (False and True) == False

# Test 3: and - second false
assert (True and False) == False

# Test 4: or - both false
assert (False or False) == False

# Test 5: or - first true
assert (True or False) == True

# Test 6: or - second true
assert (False or True) == True

# Test 7: not
assert not False == True
assert not True == False

# Test 8: Short-circuit and
def side_effect():
    print("called")
    return True

result = False and side_effect()  # side_effect not called
assert result == False

# Test 9: Short-circuit or
result = True or side_effect()  # side_effect not called
assert result == True

# Test 10: Chaining
assert (True and True and True) == True
assert (False or False or True) == True

# Test 11: Truthiness
assert (1 and 2) == 2  # Returns last truthy value
assert (0 or 5) == 5   # Returns first truthy value
assert not 0 == True
assert not "" == True
assert not [] == True

# Test 12: In conditions
x = 5
if x > 0 and x < 10:
    print("in range")

if x < 0 or x > 10:
    print("out of range")
else:
    print("in range")
```

## Python Semantics
- `and` returns first falsy value or last value
- `or` returns first truthy value or last value
- Short-circuit evaluation (right side not evaluated if not needed)
- Truthiness: `False`, `None`, `0`, `0.0`, `""`, `[]`, `{}` are falsy

## Verification
- [ ] `cargo test` - all tests pass
- [ ] Add unit tests for and/or/not
- [ ] Test short-circuit behavior
- [ ] Test truthiness for all types
- [ ] `cargo clippy -- -D warnings` - no warnings

## Dependencies
None

## Notes
- Short-circuit evaluation is important for performance and correctness
- Python's `and`/`or` return operand values, not just True/False
- `not` always returns a boolean
