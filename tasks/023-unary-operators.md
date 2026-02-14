# Task 023: Unary Operators

## Status
- [x] Completed

## Priority
High - blocking CI (examples/02_types.py fails)

## Description
Implement unary operators in the compiler to support expressions like `-17`, `-x`, `+x`, `not x`.

## Current Issue
```python
print(-17)  # Error: Unsupported expression: UnaryOp(ExprUnaryOp { op: USub, ... })
```

The compiler doesn't handle `UnaryOp` AST nodes from RustPython parser.

## Implementation

### 1. Compiler Changes (`src/compiler.rs`)

Add `UnaryOp` handling in `compile_expr()`:

```rust
Expr::UnaryOp(unary) => {
    match unary.op {
        UnaryOp::USub => {
            // Compile operand first
            self.compile_expr(&unary.operand)?;
            // Emit negate instruction
            self.emit(Instruction::Negate);
        }
        UnaryOp::UAdd => {
            // Unary plus is a no-op, just compile operand
            self.compile_expr(&unary.operand)?;
        }
        UnaryOp::Not => {
            self.compile_expr(&unary.operand)?;
            self.emit(Instruction::Not);
        }
        _ => return Err(format!("Unsupported unary operator: {:?}", unary.op)),
    }
}
```

### 2. Bytecode Changes (`src/bytecode.rs`)

Add new instructions:

```rust
pub enum Instruction {
    // ... existing instructions ...
    Negate,  // Pop value, push -value
    // Not already exists for logical not
}
```

### 3. VM Changes (`src/vm.rs`)

Implement `Negate` instruction:

```rust
Instruction::Negate => {
    let value = self.pop()?;
    match value {
        Value::Int(n) => self.push(Value::Int(-n)),
        Value::Float(f) => self.push(Value::Float(-f)),
        _ => return Err(format!("Cannot negate {:?}", value)),
    }
}
```

### 4. Serializer Changes (`src/serializer.rs`)

Add serialization for `Negate`:

```rust
Instruction::Negate => {
    writer.write_u8(OPCODE_NEGATE)?;
}
```

And deserialization:

```rust
OPCODE_NEGATE => Instruction::Negate,
```

## Test Cases

### Unit Tests (`src/compiler.rs` or `src/vm.rs`)

```rust
#[test]
fn test_unary_minus_int() {
    let mut ctx = Context::new();
    assert_eq!(ctx.eval("print(-17)").unwrap(), Value::None);
}

#[test]
fn test_unary_minus_float() {
    let mut ctx = Context::new();
    assert_eq!(ctx.eval("print(-3.14)").unwrap(), Value::None);
}

#[test]
fn test_unary_minus_variable() {
    let mut ctx = Context::new();
    ctx.eval("x = 42").unwrap();
    ctx.eval("y = -x").unwrap();
    // Check y == -42
}

#[test]
fn test_unary_plus() {
    let mut ctx = Context::new();
    assert_eq!(ctx.eval("print(+17)").unwrap(), Value::None);
}

#[test]
fn test_double_negative() {
    let mut ctx = Context::new();
    assert_eq!(ctx.eval("print(--17)").unwrap(), Value::None);  // Should print 17
}
```

### Python Examples

Update `examples/02_types.py` to include:

```python
# Unary operators
print(-17)        # -17
print(-3.14)      # -3.14
print(+42)        # 42
x = 10
print(-x)         # -10
print(--x)        # 10 (double negative)
```

## Verification

- [x] `cargo test` - all tests pass (122 tests)
- [x] `cargo run -- run examples/02_types.py` - PASS
- [x] `cargo clippy -- -D warnings` - no warnings
- [x] CI pipeline passes

## Notes

- Start with `USub` (unary minus) as it's blocking CI
- `UAdd` (unary plus) is trivial - just compile operand
- `Not` (logical not) may already be implemented - check existing code
- Consider `Invert` (bitwise not `~`) for future work
- Unary minus on bool/string should raise TypeError (match Python behavior)

## Dependencies

None - standalone feature

## Estimated Complexity

Low - straightforward AST→bytecode→VM pipeline
