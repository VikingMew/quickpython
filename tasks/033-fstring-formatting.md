# Task 033: F-String Formatting

## Status
- [ ] Not started

## Priority
Medium - convenient but can be worked around with string concatenation

## Description
Implement f-string (formatted string literals): `f"Hello {name}"`

## Current Issue
```python
name = "Alice"
msg = f"Hello {name}"  # Error: Unsupported string format
```

## Implementation

### 1. Compiler Changes (`src/compiler.rs`)

Handle `JoinedStr` expression (f-strings):
```rust
ast::Expr::JoinedStr(joined) => {
    // f"Hello {name}" is parsed as JoinedStr with parts:
    // [Constant("Hello "), FormattedValue(name), Constant("")]
    
    let mut parts = Vec::new();
    
    for value in &joined.values {
        match value {
            ast::Expr::Constant(c) => {
                // String literal part
                if let ast::Constant::Str(s) = &c.value {
                    parts.push(s.to_string());
                }
            }
            ast::Expr::FormattedValue(fv) => {
                // Expression to be formatted: {expr}
                // Compile the expression
                self.compile_expr(&fv.value, bytecode)?;
                
                // Convert to string (call str())
                bytecode.push(Instruction::CallBuiltin("str".to_string(), 1));
                
                parts.push(format!("{{}}"));  // Placeholder
            }
            _ => return Err("Unsupported f-string component".to_string()),
        }
    }
    
    // Build the format string
    let format_str = parts.join("");
    
    // Count the number of expressions
    let expr_count = joined.values.iter()
        .filter(|v| matches!(v, ast::Expr::FormattedValue(_)))
        .count();
    
    // Push format string
    bytecode.push(Instruction::PushString(format_str));
    
    // Format with expressions on stack
    bytecode.push(Instruction::FormatString(expr_count));
}
```

### 2. Bytecode Changes (`src/bytecode.rs`)
Add new instruction:
```rust
pub enum Instruction {
    // ... existing ...
    FormatString(usize),  // Format string with n values from stack
}
```

### 3. VM Changes (`src/vm.rs`)
```rust
Instruction::FormatString(count) => {
    // Pop format string
    let format_str = match self.stack.pop() {
        Some(Value::String(s)) => s,
        _ => return Err(Value::error(ExceptionType::RuntimeError, "Expected format string")),
    };
    
    // Pop values (in reverse order)
    let mut values = Vec::new();
    for _ in 0..*count {
        values.push(self.stack.pop()
            .ok_or_else(|| Value::error(ExceptionType::RuntimeError, "Stack underflow"))?);
    }
    values.reverse();
    
    // Convert values to strings
    let str_values: Vec<String> = values.iter()
        .map(|v| Self::value_to_string(v))
        .collect();
    
    // Simple replacement (replace {} with values in order)
    let mut result = format_str.clone();
    for value in str_values {
        if let Some(pos) = result.find("{}") {
            result.replace_range(pos..pos+2, &value);
        }
    }
    
    self.stack.push(Value::String(result));
    *ip += 1;
}

fn value_to_string(value: &Value) -> String {
    match value {
        Value::String(s) => s.clone(),
        Value::Int(i) => i.to_string(),
        Value::Float(f) => f.to_string(),
        Value::Bool(b) => b.to_string(),
        Value::None => "None".to_string(),
        _ => format!("{:?}", value),
    }
}
```

## Alternative: Simpler Implementation

Instead of `FormatString` instruction, desugar to string concatenation:
```python
f"Hello {name}"  →  "Hello " + str(name)
f"{x} + {y} = {x+y}"  →  str(x) + " + " + str(y) + " = " + str(x+y)
```

This avoids adding new bytecode instructions.

## Test Cases

```python
# Test 1: Simple variable
name = "Alice"
assert f"Hello {name}" == "Hello Alice"

# Test 2: Multiple variables
x = 10
y = 20
assert f"{x} + {y} = {x + y}" == "10 + 20 = 30"

# Test 3: Expressions
n = 5
assert f"Square of {n} is {n * n}" == "Square of 5 is 25"

# Test 4: Different types
age = 30
height = 5.9
assert f"Age: {age}, Height: {height}" == "Age: 30, Height: 5.9"

# Test 5: No interpolation
assert f"Hello World" == "Hello World"

# Test 6: Empty string
x = 42
assert f"{x}" == "42"

# Test 7: Function calls
def get_name():
    return "Bob"

assert f"Hello {get_name()}" == "Hello Bob"

# Test 8: Nested expressions
items = [1, 2, 3]
assert f"Length: {len(items)}" == "Length: 3"
```

## Supported Features (Phase 1)
- Basic interpolation: `f"{var}"`
- Expressions: `f"{x + y}"`
- Multiple values: `f"{a} {b} {c}"`
- Function calls: `f"{func()}"`

## Not Supported (Phase 1)
- Format specifiers: `f"{x:.2f}"`, `f"{x:>10}"`
- Conversion flags: `f"{x!r}"`, `f"{x!s}"`
- Nested f-strings: `f"{f'{x}'}"`
- Multiline f-strings

## Verification
- [ ] `cargo test` - all tests pass
- [ ] Add unit tests for f-strings
- [ ] Test with different types
- [ ] Test expressions in interpolation
- [ ] `cargo clippy -- -D warnings` - no warnings

## Dependencies
- Requires `str()` builtin function (Task 034)

## Notes
- F-strings are evaluated at runtime, not compile time
- Expressions are evaluated left-to-right
- F-strings are more readable than `.format()` or `%` formatting
- Can be desugared to string concatenation for simpler implementation
