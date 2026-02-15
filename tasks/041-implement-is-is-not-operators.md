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

Currently, QuickPython only supports value equality. We need to add identity operators.

## Requirements

### 1. Add New Bytecode Instructions

In `src/bytecode.rs`:
```rust
pub enum Instruction {
    // ... existing instructions ...
    Is,       // Check if two objects are the same (identity)
    IsNot,    // Check if two objects are not the same
}
```

### 2. Compiler Support

In `src/compiler.rs`, handle `ast::CmpOp::Is` and `ast::CmpOp::IsNot` in the comparison compilation:

```rust
fn compile_compare(&mut self, left: &ast::Expr, ops: &[ast::CmpOp], comparators: &[ast::Expr]) {
    // ... existing code ...
    match op {
        ast::CmpOp::Is => self.bytecode.push(Instruction::Is),
        ast::CmpOp::IsNot => self.bytecode.push(Instruction::IsNot),
        // ... other operators ...
    }
}
```

### 3. VM Execution

In `src/vm.rs`, implement the identity check:

```rust
Instruction::Is => {
    let b = self.stack.pop().ok_or_else(|| ...)?;
    let a = self.stack.pop().ok_or_else(|| ...)?;
    
    let result = match (&a, &b) {
        // None is always the same object
        (Value::None, Value::None) => true,
        
        // Small integers might be cached (optional optimization)
        (Value::Int(x), Value::Int(y)) if *x == *y && (-5..=256).contains(x) => true,
        
        // For reference types, compare pointers
        (Value::List(a), Value::List(b)) => Rc::ptr_eq(a, b),
        (Value::Dict(a), Value::Dict(b)) => Rc::ptr_eq(a, b),
        (Value::Tuple(a), Value::Tuple(b)) => Rc::ptr_eq(a, b),
        (Value::String(a), Value::String(b)) => a.as_ptr() == b.as_ptr(),
        
        // Different types or values are not identical
        _ => false,
    };
    
    self.stack.push(Value::Bool(result));
    *ip += 1;
}

Instruction::IsNot => {
    self.execute_instruction(&Instruction::Is, ip, globals)?;
    self.execute_instruction(&Instruction::Not, ip, globals)?;
}
```

### 4. Serialization Support

In `src/serializer.rs`, add serialization for the new instructions:

```rust
Instruction::Is => writer.write_u8(0x50)?,
Instruction::IsNot => writer.write_u8(0x51)?,
```

And deserialization:
```rust
0x50 => Instruction::Is,
0x51 => Instruction::IsNot,
```

## Test Cases

Add tests in `src/main.rs`:

```rust
#[test]
fn test_is_operator_none() {
    let mut ctx = Context::new();
    ctx.eval(r#"
a = None
b = None
result = a is b
"#).unwrap();
    assert_eq!(ctx.get("result"), Some(Value::Bool(true)));
}

#[test]
fn test_is_operator_different_lists() {
    let mut ctx = Context::new();
    ctx.eval(r#"
a = [1, 2, 3]
b = [1, 2, 3]
result = a is b
"#).unwrap();
    assert_eq!(ctx.get("result"), Some(Value::Bool(false)));
}

#[test]
fn test_is_operator_same_list() {
    let mut ctx = Context::new();
    ctx.eval(r#"
a = [1, 2, 3]
b = a
result = a is b
"#).unwrap();
    assert_eq!(ctx.get("result"), Some(Value::Bool(true)));
}

#[test]
fn test_is_not_operator() {
    let mut ctx = Context::new();
    ctx.eval(r#"
a = [1, 2]
b = [1, 2]
result = a is not b
"#).unwrap();
    assert_eq!(ctx.get("result"), Some(Value::Bool(true)));
}

#[test]
fn test_is_operator_small_ints() {
    let mut ctx = Context::new();
    ctx.eval(r#"
a = 5
b = 5
result = a is b
"#).unwrap();
    // Small integers might be cached
    assert_eq!(ctx.get("result"), Some(Value::Bool(true)));
}

#[test]
fn test_is_operator_different_types() {
    let mut ctx = Context::new();
    ctx.eval(r#"
result = 5 is "5"
"#).unwrap();
    assert_eq!(ctx.get("result"), Some(Value::Bool(false)));
}

#[test]
fn test_is_vs_eq_lists() {
    let mut ctx = Context::new();
    ctx.eval(r#"
a = [1, 2, 3]
b = [1, 2, 3]
eq_result = a == b
is_result = a is b
"#).unwrap();
    assert_eq!(ctx.get("eq_result"), Some(Value::Bool(true)));
    assert_eq!(ctx.get("is_result"), Some(Value::Bool(false)));
}
```

## Implementation Notes

1. **Identity vs Equality**: 
   - `is` checks if two variables reference the same object in memory
   - `==` checks if two objects have the same value
   - Example: `[1,2] == [1,2]` is `True`, but `[1,2] is [1,2]` is `False`

2. **None Singleton**: 
   - `None` should always be the same object, so `None is None` is always `True`

3. **Small Integer Caching** (Optional):
   - CPython caches small integers (-5 to 256)
   - We can implement this optimization or skip it for simplicity

4. **String Interning** (Optional):
   - CPython interns some strings
   - We can skip this for now

5. **Reference Counting**:
   - Use `Rc::ptr_eq()` to compare if two `Rc` pointers point to the same allocation

## Success Criteria

- [ ] `is` and `is not` operators compile successfully
- [ ] Identity checks work correctly for all value types
- [ ] `None is None` returns `True`
- [ ] Different list instances with same values: `is` returns `False`, `==` returns `True`
- [ ] Same list referenced twice: `is` returns `True`
- [ ] All test cases pass
- [ ] Code passes `cargo fmt` and `cargo clippy`
- [ ] Serialization/deserialization works for new instructions

## Related Tasks

- Task 042: Implement `with` statement (context managers)
- Task 043: Implement generators and `yield`

## References

- Python documentation: https://docs.python.org/3/reference/expressions.html#is
- PEP 8: "Comparisons to singletons like None should always be done with `is` or `is not`"
