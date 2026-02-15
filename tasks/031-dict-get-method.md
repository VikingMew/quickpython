# Task 031: Dictionary `.get()` Method

## Status
- [ ] Not started

## Priority
High - essential for safe dict access

## Description
Implement `.get(key, default=None)` method for dictionaries.

## Current Issue
```python
d = {"a": 1}
value = d.get("b", 0)  # Error: 'dict' object has no attribute 'get'
```

## Implementation

### 1. Value Changes (`src/value.rs`)
No changes needed - method handled in VM.

### 2. VM Changes (`src/vm.rs`)

Add dict method handling in `GetAttr`:
```rust
Instruction::GetAttr(attr_name) => {
    let obj = self.stack.pop()
        .ok_or_else(|| Value::error(ExceptionType::RuntimeError, "Stack underflow"))?;
    
    match obj {
        Value::Dict(dict) => {
            match attr_name.as_str() {
                "get" => {
                    // Create bound method that captures the dict
                    let dict_clone = dict.clone();
                    let method = Value::NativeFunction(Rc::new(move |args| {
                        dict_get(&dict_clone, args)
                    }));
                    self.stack.push(method);
                }
                "keys" => {
                    // Existing implementation
                }
                _ => return Err(Value::error(
                    ExceptionType::AttributeError,
                    format!("'dict' object has no attribute '{}'", attr_name)
                )),
            }
        }
        // ... existing code ...
    }
    *ip += 1;
}
```

### 3. Dict Method Implementation
```rust
fn dict_get(dict: &Rc<RefCell<HashMap<DictKey, Value>>>, args: &[Value]) -> Result<Value, Value> {
    if args.is_empty() {
        return Err(Value::error(
            ExceptionType::TypeError,
            "get() takes at least 1 argument (0 given)"
        ));
    }
    
    // Convert key to DictKey
    let key = match Self::value_to_dict_key(&args[0]) {
        Some(k) => k,
        None => return Err(Value::error(
            ExceptionType::TypeError,
            format!("unhashable type: '{}'", Self::type_name(&args[0]))
        )),
    };
    
    // Get default value (None if not provided)
    let default = if args.len() > 1 {
        args[1].clone()
    } else {
        Value::None
    };
    
    // Look up key
    let result = dict.borrow()
        .get(&key)
        .cloned()
        .unwrap_or(default);
    
    Ok(result)
}
```

## Test Cases

```python
# Test 1: get() with existing key
d = {"a": 1, "b": 2}
assert d.get("a") == 1

# Test 2: get() with missing key, no default
d = {"a": 1}
assert d.get("b") == None

# Test 3: get() with missing key, custom default
d = {"a": 1}
assert d.get("b", 0) == 0
assert d.get("c", "default") == "default"

# Test 4: get() doesn't modify dict
d = {"a": 1}
result = d.get("b", 0)
assert "b" not in d  # Key not added

# Test 5: Different key types
d = {1: "one", "two": 2}
assert d.get(1) == "one"
assert d.get("two") == 2
assert d.get(3, "missing") == "missing"

# Test 6: In real usage
config = {"host": "localhost", "port": 8080}
host = config.get("host", "0.0.0.0")
timeout = config.get("timeout", 30)
assert host == "localhost"
assert timeout == 30

# Test 7: Chaining
d = {"a": {"b": 1}}
inner = d.get("a", {})
value = inner.get("b", 0)
assert value == 1

# Test 8: Error cases
try:
    d = {"a": 1}
    d.get()  # TypeError: missing argument
except TypeError:
    print("caught")
```

## Comparison with Direct Access
```python
# Direct access - raises KeyError if missing
value = d["key"]  # KeyError if "key" not in d

# .get() - returns default if missing
value = d.get("key", default)  # Safe, no exception
```

## Verification
- [ ] `cargo test` - all tests pass
- [ ] Add unit tests for .get() method
- [ ] Test with different key types
- [ ] Test error cases
- [ ] `cargo clippy -- -D warnings` - no warnings

## Dependencies
None - uses existing dict infrastructure

## Notes
- `.get()` never raises `KeyError`
- Default value is `None` if not specified
- Does not modify the dictionary
- More Pythonic than checking `if key in dict`
