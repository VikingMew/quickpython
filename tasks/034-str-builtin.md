# Task 034: `str()` Builtin Function

## Status
- [ ] Not started

## Priority
High - essential for type conversion and string formatting

## Description
Implement `str()` builtin function to convert values to strings.

## Current Issue
```python
x = 123
s = str(x)  # Error: name 'str' is not defined
```

## Implementation

### 1. Builtins Changes (`src/builtins/mod.rs`)

Add `str()` to builtin functions:
```rust
pub fn get_builtin_function(name: &str) -> Option<Value> {
    match name {
        "print" => Some(Value::BuiltinFunction("print".to_string())),
        "len" => Some(Value::BuiltinFunction("len".to_string())),
        "int" => Some(Value::BuiltinFunction("int".to_string())),
        "float" => Some(Value::BuiltinFunction("float".to_string())),
        "str" => Some(Value::BuiltinFunction("str".to_string())),
        "range" => Some(Value::BuiltinFunction("range".to_string())),
        _ => None,
    }
}
```

### 2. VM Changes (`src/vm.rs`)

Add `str()` handling in builtin function calls:
```rust
"str" => {
    if args.len() != 1 {
        return Err(Value::error(
            ExceptionType::TypeError,
            format!("str() takes exactly 1 argument ({} given)", args.len())
        ));
    }
    
    let value = &args[0];
    let result = match value {
        Value::String(s) => s.clone(),
        Value::Int(i) => i.to_string(),
        Value::Float(f) => {
            // Format float nicely
            if f.fract() == 0.0 && f.is_finite() {
                format!("{:.1}", f)  // "5.0" not "5"
            } else {
                f.to_string()
            }
        }
        Value::Bool(b) => {
            if *b { "True" } else { "False" }.to_string()
        }
        Value::None => "None".to_string(),
        Value::List(list) => {
            // "[1, 2, 3]"
            let items: Vec<String> = list.borrow()
                .iter()
                .map(|v| Self::value_repr(v))
                .collect();
            format!("[{}]", items.join(", "))
        }
        Value::Dict(dict) => {
            // "{'a': 1, 'b': 2}"
            let items: Vec<String> = dict.borrow()
                .iter()
                .map(|(k, v)| format!("{}: {}", Self::dict_key_repr(k), Self::value_repr(v)))
                .collect();
            format!("{{{}}}", items.join(", "))
        }
        Value::Function(f) => {
            format!("<function {} at {:p}>", f.name, f)
        }
        _ => format!("<{} object>", Self::type_name(value)),
    };
    
    self.stack.push(Value::String(result));
}

// Helper for repr-style formatting (with quotes for strings)
fn value_repr(value: &Value) -> String {
    match value {
        Value::String(s) => format!("'{}'", s),
        Value::Int(i) => i.to_string(),
        Value::Float(f) => f.to_string(),
        Value::Bool(b) => if *b { "True" } else { "False" }.to_string(),
        Value::None => "None".to_string(),
        _ => format!("{:?}", value),
    }
}

fn dict_key_repr(key: &DictKey) -> String {
    match key {
        DictKey::String(s) => format!("'{}'", s),
        DictKey::Int(i) => i.to_string(),
    }
}
```

## Test Cases

```python
# Test 1: Integer to string
assert str(123) == "123"
assert str(-456) == "-456"
assert str(0) == "0"

# Test 2: Float to string
assert str(3.14) == "3.14"
assert str(5.0) == "5.0"

# Test 3: Bool to string
assert str(True) == "True"
assert str(False) == "False"

# Test 4: None to string
assert str(None) == "None"

# Test 5: String to string (identity)
assert str("hello") == "hello"

# Test 6: List to string
assert str([1, 2, 3]) == "[1, 2, 3]"
assert str([]) == "[]"

# Test 7: Dict to string
d = {"a": 1, "b": 2}
s = str(d)
assert "a" in s and "1" in s  # Order may vary

# Test 8: In expressions
x = 10
y = 20
result = str(x) + " + " + str(y)
assert result == "10 + 20"

# Test 9: Error - no arguments
try:
    str()  # TypeError
except TypeError:
    print("caught")

# Test 10: Error - too many arguments
try:
    str(1, 2)  # TypeError
except TypeError:
    print("caught")
```

## Python Semantics
- `str(x)` returns a string representation of x
- For strings, returns the string itself (not quoted)
- For numbers, returns decimal representation
- For bool, returns "True" or "False" (capitalized)
- For None, returns "None"
- For containers, returns a readable representation

## Comparison with `repr()`
- `str()` - human-readable string
- `repr()` - unambiguous representation (Phase 2)
- For strings: `str("hi")` → `"hi"`, `repr("hi")` → `"'hi'"`

## Verification
- [ ] `cargo test` - all tests pass
- [ ] Add unit tests for str() with all types
- [ ] Test error cases
- [ ] `cargo clippy -- -D warnings` - no warnings

## Dependencies
None

## Notes
- `str()` is used implicitly by `print()` and f-strings
- Essential for string concatenation with non-strings
- Python's `str()` calls `__str__()` method (we don't support that yet)
