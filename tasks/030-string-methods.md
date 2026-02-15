# Task 030: String Methods

## Status
- [ ] Not started

## Priority
High - essential for string manipulation

## Description
Implement common string methods: `.split()`, `.strip()`, `.startswith()`, `.endswith()`, `.lower()`, `.upper()`, `.replace()`, `.join()`

## Current Issue
```python
s = "hello world"
words = s.split()  # Error: 'str' object has no attribute 'split'
```

## Implementation

### 1. Value Changes (`src/value.rs`)
No changes needed - methods are handled in VM.

### 2. Compiler Changes (`src/compiler.rs`)
Method calls are already compiled as `GetAttr` + `Call`. No changes needed.

### 3. VM Changes (`src/vm.rs`)

Add string method handling in `GetAttr`:
```rust
Instruction::GetAttr(attr_name) => {
    let obj = self.stack.pop()
        .ok_or_else(|| Value::error(ExceptionType::RuntimeError, "Stack underflow"))?;
    
    match obj {
        Value::String(s) => {
            // Create a bound method
            let method = match attr_name.as_str() {
                "split" => Value::NativeFunction(/* ... */),
                "strip" => Value::NativeFunction(/* ... */),
                "startswith" => Value::NativeFunction(/* ... */),
                "endswith" => Value::NativeFunction(/* ... */),
                "lower" => Value::NativeFunction(/* ... */),
                "upper" => Value::NativeFunction(/* ... */),
                "replace" => Value::NativeFunction(/* ... */),
                "join" => Value::NativeFunction(/* ... */),
                _ => return Err(Value::error(
                    ExceptionType::AttributeError,
                    format!("'str' object has no attribute '{}'", attr_name)
                )),
            };
            self.stack.push(method);
        }
        // ... existing code for other types ...
    }
    *ip += 1;
}
```

### 4. String Method Implementations

#### `.split(sep=None, maxsplit=-1)`
```rust
fn string_split(s: &str, args: &[Value]) -> Result<Value, Value> {
    let sep = if args.is_empty() {
        None  // Split on whitespace
    } else {
        match &args[0] {
            Value::String(sep) => Some(sep.as_str()),
            Value::None => None,
            _ => return Err(Value::error(ExceptionType::TypeError, "sep must be string or None")),
        }
    };
    
    let parts: Vec<Value> = if let Some(sep) = sep {
        s.split(sep).map(|p| Value::String(p.to_string())).collect()
    } else {
        s.split_whitespace().map(|p| Value::String(p.to_string())).collect()
    };
    
    Ok(Value::List(Rc::new(RefCell::new(parts))))
}
```

#### `.strip(chars=None)`
```rust
fn string_strip(s: &str, args: &[Value]) -> Result<Value, Value> {
    let result = if args.is_empty() {
        s.trim()
    } else {
        // Custom character set - Phase 2
        s.trim()
    };
    Ok(Value::String(result.to_string()))
}
```

#### `.startswith(prefix)` / `.endswith(suffix)`
```rust
fn string_startswith(s: &str, args: &[Value]) -> Result<Value, Value> {
    if args.is_empty() {
        return Err(Value::error(ExceptionType::TypeError, "startswith() takes at least 1 argument"));
    }
    
    match &args[0] {
        Value::String(prefix) => Ok(Value::Bool(s.starts_with(prefix))),
        _ => Err(Value::error(ExceptionType::TypeError, "startswith() argument must be str")),
    }
}

fn string_endswith(s: &str, args: &[Value]) -> Result<Value, Value> {
    if args.is_empty() {
        return Err(Value::error(ExceptionType::TypeError, "endswith() takes at least 1 argument"));
    }
    
    match &args[0] {
        Value::String(suffix) => Ok(Value::Bool(s.ends_with(suffix))),
        _ => Err(Value::error(ExceptionType::TypeError, "endswith() argument must be str")),
    }
}
```

#### `.lower()` / `.upper()`
```rust
fn string_lower(s: &str, _args: &[Value]) -> Result<Value, Value> {
    Ok(Value::String(s.to_lowercase()))
}

fn string_upper(s: &str, _args: &[Value]) -> Result<Value, Value> {
    Ok(Value::String(s.to_uppercase()))
}
```

#### `.replace(old, new, count=-1)`
```rust
fn string_replace(s: &str, args: &[Value]) -> Result<Value, Value> {
    if args.len() < 2 {
        return Err(Value::error(ExceptionType::TypeError, "replace() takes at least 2 arguments"));
    }
    
    let old = match &args[0] {
        Value::String(s) => s,
        _ => return Err(Value::error(ExceptionType::TypeError, "replace() argument 1 must be str")),
    };
    
    let new = match &args[1] {
        Value::String(s) => s,
        _ => return Err(Value::error(ExceptionType::TypeError, "replace() argument 2 must be str")),
    };
    
    let result = s.replace(old, new);
    Ok(Value::String(result))
}
```

#### `.join(iterable)`
```rust
fn string_join(sep: &str, args: &[Value]) -> Result<Value, Value> {
    if args.is_empty() {
        return Err(Value::error(ExceptionType::TypeError, "join() takes exactly 1 argument"));
    }
    
    match &args[0] {
        Value::List(list) => {
            let strings: Result<Vec<String>, Value> = list.borrow()
                .iter()
                .map(|v| match v {
                    Value::String(s) => Ok(s.clone()),
                    _ => Err(Value::error(ExceptionType::TypeError, "join() requires all items to be strings")),
                })
                .collect();
            
            let strings = strings?;
            Ok(Value::String(strings.join(sep)))
        }
        _ => Err(Value::error(ExceptionType::TypeError, "join() argument must be a list")),
    }
}
```

## Test Cases

```python
# Test 1: split() - whitespace
s = "hello world python"
assert s.split() == ["hello", "world", "python"]

# Test 2: split() - custom separator
s = "a,b,c"
assert s.split(",") == ["a", "b", "c"]

# Test 3: strip()
s = "  hello  "
assert s.strip() == "hello"

# Test 4: startswith()
s = "hello world"
assert s.startswith("hello") == True
assert s.startswith("world") == False

# Test 5: endswith()
assert s.endswith("world") == True
assert s.endswith("hello") == False

# Test 6: lower() / upper()
s = "Hello World"
assert s.lower() == "hello world"
assert s.upper() == "HELLO WORLD"

# Test 7: replace()
s = "hello world"
assert s.replace("world", "python") == "hello python"

# Test 8: join()
words = ["hello", "world"]
assert " ".join(words) == "hello world"
assert ",".join(words) == "hello,world"

# Test 9: Chaining
s = "  HELLO WORLD  "
result = s.strip().lower()
assert result == "hello world"

# Test 10: Empty strings
assert "".split() == []
assert "".strip() == ""
assert "".startswith("") == True
```

## Supported Methods (Phase 1)
- `.split(sep=None)` - split string into list
- `.strip()` - remove leading/trailing whitespace
- `.startswith(prefix)` - check prefix
- `.endswith(suffix)` - check suffix
- `.lower()` - convert to lowercase
- `.upper()` - convert to uppercase
- `.replace(old, new)` - replace substring
- `.join(iterable)` - join strings with separator

## Phase 2 Methods
- `.find()`, `.index()` - search
- `.count()` - count occurrences
- `.lstrip()`, `.rstrip()` - one-sided strip
- `.splitlines()` - split by newlines
- `.format()` - string formatting

## Verification
- [ ] `cargo test` - all tests pass
- [ ] Add unit tests for each method
- [ ] Test error cases (wrong argument types)
- [ ] `cargo clippy -- -D warnings` - no warnings

## Dependencies
None

## Notes
- String methods return new strings (strings are immutable)
- `.split()` with no args splits on any whitespace
- `.join()` is called on the separator, not the list
