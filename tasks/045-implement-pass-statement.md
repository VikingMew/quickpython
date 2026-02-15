# Task 045: Implement `pass` Statement

**Status**: Pending
**Created**: 2026-02-15
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

### 1. Add Bytecode Instruction (Optional)

In `src/bytecode.rs`:
```rust
pub enum Instruction {
    // ... existing instructions ...
    Pass,  // No-op instruction (optional)
}
```

**Note**: `pass` can also be implemented by simply not emitting any bytecode at all, since it does nothing.

### 2. Compiler Support

In `src/compiler.rs`, handle `ast::Stmt::Pass`:

```rust
fn compile_stmt(&mut self, stmt: &ast::Stmt) {
    match stmt {
        ast::Stmt::Pass(_) => {
            // Option 1: Emit no bytecode (preferred)
            // Do nothing - pass is a no-op
            
            // Option 2: Emit explicit Pass instruction
            // self.bytecode.push(Instruction::Pass);
        }
        // ... other statements ...
    }
}
```

### 3. VM Execution (if using Pass instruction)

In `src/vm.rs`:
```rust
Instruction::Pass => {
    // Do nothing, just advance IP
    *ip += 1;
}
```

## Test Cases

```rust
#[test]
fn test_pass_statement_in_function() {
    let mut ctx = Context::new();
    ctx.eval(r#"
def empty_function():
    pass

result = empty_function()
"#).unwrap();
    assert_eq!(ctx.get("result"), Some(Value::None));
}

#[test]
fn test_pass_statement_in_if() {
    let mut ctx = Context::new();
    ctx.eval(r#"
x = 5
if x > 0:
    pass
else:
    x = 0

result = x
"#).unwrap();
    assert_eq!(ctx.get("result"), Some(Value::Int(5)));
}

#[test]
fn test_pass_statement_in_loop() {
    let mut ctx = Context::new();
    ctx.eval(r#"
count = 0
for i in range(5):
    pass
    count += 1

result = count
"#).unwrap();
    assert_eq!(ctx.get("result"), Some(Value::Int(5)));
}

#[test]
fn test_pass_statement_in_while() {
    let mut ctx = Context::new();
    ctx.eval(r#"
x = 0
while x < 3:
    x += 1
    pass
    
result = x
"#).unwrap();
    assert_eq!(ctx.get("result"), Some(Value::Int(3)));
}

#[test]
fn test_pass_statement_in_try_except() {
    let mut ctx = Context::new();
    ctx.eval(r#"
result = "ok"
try:
    x = 1 / 0
except ZeroDivisionError:
    pass
"#).unwrap();
    assert_eq!(ctx.get("result"), Some(Value::String("ok".to_string())));
}

#[test]
fn test_multiple_pass_statements() {
    let mut ctx = Context::new();
    ctx.eval(r#"
def multi_pass():
    pass
    pass
    pass
    return 42

result = multi_pass()
"#).unwrap();
    assert_eq!(ctx.get("result"), Some(Value::Int(42)));
}

#[test]
fn test_pass_only_function() {
    let mut ctx = Context::new();
    ctx.eval(r#"
def do_nothing():
    pass

do_nothing()
result = "done"
"#).unwrap();
    assert_eq!(ctx.get("result"), Some(Value::String("done".to_string())));
}

#[test]
fn test_pass_in_nested_blocks() {
    let mut ctx = Context::new();
    ctx.eval(r#"
result = 0
for i in range(3):
    if i == 1:
        pass
    else:
        result += i
"#).unwrap();
    assert_eq!(ctx.get("result"), Some(Value::Int(2)));  // 0 + 2
}

#[test]
fn test_pass_with_other_statements() {
    let mut ctx = Context::new();
    ctx.eval(r#"
x = 10
pass
y = 20
pass
result = x + y
"#).unwrap();
    assert_eq!(ctx.get("result"), Some(Value::Int(30)));
}

#[test]
fn test_pass_in_elif() {
    let mut ctx = Context::new();
    ctx.eval(r#"
x = 5
if x < 0:
    result = "negative"
elif x == 0:
    pass
elif x < 10:
    result = "small"
else:
    result = "large"
"#).unwrap();
    assert_eq!(ctx.get("result"), Some(Value::String("small".to_string())));
}
```

## Implementation Notes

1. **No-op Implementation**:
   - The simplest approach is to not emit any bytecode for `pass`
   - This is efficient and matches Python's behavior
   - No VM changes needed

2. **Explicit Instruction**:
   - Alternatively, emit a `Pass` instruction that does nothing
   - Useful for debugging or bytecode analysis
   - Slightly less efficient but more explicit

3. **Syntax Requirement**:
   - `pass` is required where Python syntax demands a statement
   - Empty blocks are not allowed in Python
   - `pass` serves as a placeholder

4. **Common Use Cases**:
   - Stub functions during development
   - Empty exception handlers (though often not recommended)
   - Placeholder for future implementation
   - Minimal loop bodies

5. **Difference from `continue`**:
   - `pass` does nothing and continues to next statement
   - `continue` skips rest of loop body and goes to next iteration

## Success Criteria

- [ ] `pass` statement compiles without errors
- [ ] Functions with only `pass` return `None`
- [ ] `pass` in if/elif/else blocks works
- [ ] `pass` in loops works
- [ ] `pass` in try/except blocks works
- [ ] Multiple `pass` statements work
- [ ] All test cases pass
- [ ] Code passes `cargo fmt` and `cargo clippy`

## Related Tasks

- Task 041: Implement `is` and `is not` operators
- Task 042: Implement `with` statement
- Task 043: Implement generators and `yield`
- Task 044: Implement dictionary comprehensions

## References

- Python documentation: https://docs.python.org/3/reference/simple_stmts.html#the-pass-statement
- PEP 8: Use `pass` for empty code blocks
