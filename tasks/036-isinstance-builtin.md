# Task 036: `isinstance()` Builtin Function

## Status
- [ ] Not started

## Priority
Medium - useful for type checking

## Description
Implement `isinstance()` builtin function to check object types.

## Current Issue
```python
x = 123
if isinstance(x, int):  # Error: name 'isinstance' is not defined
    print("is int")
```

## Implementation

### 1. Builtins Changes (`src/builtins/mod.rs`)

Add `isinstance()` to builtin functions:
```rust
pub fn get_builtin_function(name: &str) -> Option<Value> {
    match name {
        // ... existing ...
        "isinstance" => Some(Value::BuiltinFunction("isinstance".to_string())),
        _ => None,
    }
}
```

### 2. Value Changes (`src/value.rs`)

Add type objects as values:
```rust
pub enum Value {
    // ... existing ...
    Type(TypeObject),
}

pub enum TypeObject {
    Int,
    Float,
    Bool,
    Str,
    List,
    Dict,
    NoneType,
}
```

### 3. Compiler Changes (`src/compiler.rs`)

Handle type names as special identifiers:
```rust
// When compiling Name expressions
ast::Expr::Name(name) => {
    match name.id.as_str() {
        "int" => bytecode.push(Instruction::PushType(TypeObject::Int)),
        "float" => bytecode.push(Instruction::PushType(TypeObject::Float)),
        "bool" => bytecode.push(Instruction::PushType(TypeObject::Bool)),
        "str" => bytecode.push(Instruction::PushType(TypeObject::Str)),
        "list" => bytecode.push(Instruction::PushType(TypeObject::List)),
        "dict" => bytecode.push(Instruction::PushType(TypeObject::Dict)),
        _ => {
            // Regular variable
            bytecode.push(Instruction::GetGlobal(name.id.to_string()));
        }
    }
}
```

### 4. VM Changes (`src/vm.rs`)

Add `isinstance()` handling:
```rust
"isinstance" => {
    if args.len() != 2 {
        return Err(Value::error(
            ExceptionType::TypeError,
            format!("isinstance() takes exactly 2 arguments ({} given)", args.len())
        ));
    }
    
    let obj = &args[0];
    let type_obj = &args[1];
    
    let Value::Type(expected_type) = type_obj else {
        return Err(Value::error(
            ExceptionType::TypeError,
            "isinstance() arg 2 must be a type"
        ));
    };
    
    let result = match (obj, expected_type) {
        (Value::Int(_), TypeObject::Int) => true,
        (Value::Float(_), TypeObject::Float) => true,
        (Value::Bool(_), TypeObject::Bool) => true,
        (Value::String(_), TypeObject::Str) => true,
        (Value::List(_), TypeObject::List) => true,
        (Value::Dict(_), TypeObject::Dict) => true,
        (Value::None, TypeObject::NoneType) => true,
        _ => false,
    };
    
    self.stack.push(Value::Bool(result));
}
```

## Test Cases

```python
# Test 1: Integer
x = 123
assert isinstance(x, int) == True
assert isinstance(x, str) == False

# Test 2: Float
y = 3.14
assert isinstance(y, float) == True
assert isinstance(y, int) == False

# Test 3: String
s = "hello"
assert isinstance(s, str) == True
assert isinstance(s, list) == False

# Test 4: Bool
b = True
assert isinstance(b, bool) == True
# Note: In Python, bool is subclass of int
# For Phase 1, we don't support inheritance

# Test 5: List
lst = [1, 2, 3]
assert isinstance(lst, list) == True
assert isinstance(lst, dict) == False

# Test 6: Dict
d = {"a": 1}
assert isinstance(d, dict) == True
assert isinstance(d, list) == False

# Test 7: None
n = None
assert isinstance(n, type(None)) == True

# Test 8: In conditions
def process(value):
    if isinstance(value, int):
        return value * 2
    elif isinstance(value, str):
        return value.upper()
    else:
        return None

assert process(5) == 10
assert process("hi") == "HI"

# Test 9: Error - wrong number of args
try:
    isinstance(1)  # TypeError
except TypeError:
    print("caught")

# Test 10: Error - second arg not a type
try:
    isinstance(1, "int")  # TypeError
except TypeError:
    print("caught")
```

## Supported Types (Phase 1)
- `int` - integer type
- `float` - float type
- `bool` - boolean type
- `str` - string type
- `list` - list type
- `dict` - dictionary type
- `type(None)` - NoneType

## Not Supported (Phase 1)
- Tuple of types: `isinstance(x, (int, str))`
- Custom classes
- Type inheritance (bool is not considered subclass of int)

## Verification
- [ ] `cargo test` - all tests pass
- [ ] Add unit tests for isinstance() with all types
- [ ] Test error cases
- [ ] `cargo clippy -- -D warnings` - no warnings

## Dependencies
None

## Notes
- In Python, `bool` is a subclass of `int` (we don't support this yet)
- Type objects are first-class values
- Can be used for runtime type checking
- More Pythonic than checking `type(x) == int`
