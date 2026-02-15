# Task 032: Multiple Assignment and Tuple Unpacking

## Status
- [ ] Not started

## Priority
Medium-High - common Python idiom

## Description
Implement multiple assignment and tuple unpacking: `a, b = 1, 2`

## Current Issue
```python
a, b = 1, 2  # Error: Unsupported assignment target
x, y = [10, 20]  # Error: Unsupported assignment target
```

## Implementation

### 1. Bytecode Changes (`src/bytecode.rs`)
Add new instruction:
```rust
pub enum Instruction {
    // ... existing ...
    UnpackSequence(usize),  // Unpack TOS into n values
}
```

### 2. Compiler Changes (`src/compiler.rs`)

Handle tuple assignment in `Assign` statement:
```rust
ast::Stmt::Assign(assign) => {
    // Check if target is a tuple
    if let ast::Expr::Tuple(tuple) = &*assign.targets[0] {
        // Multiple assignment: a, b, c = expr
        
        // 1. Compile the value expression
        self.compile_expr(&assign.value, bytecode)?;
        
        // 2. Unpack into n values
        let n = tuple.elts.len();
        bytecode.push(Instruction::UnpackSequence(n));
        
        // 3. Assign to each target (in reverse order due to stack)
        for target in tuple.elts.iter().rev() {
            match target {
                ast::Expr::Name(name) => {
                    bytecode.push(Instruction::SetGlobal(name.id.to_string()));
                }
                _ => return Err("Unsupported unpacking target".to_string()),
            }
        }
    } else {
        // Regular assignment
        // ... existing code ...
    }
}
```

Handle tuple literals in expressions:
```rust
ast::Expr::Tuple(tuple) => {
    // Compile each element
    for elt in &tuple.elts {
        self.compile_expr(elt, bytecode)?;
    }
    
    // Build tuple from n stack values
    bytecode.push(Instruction::BuildTuple(tuple.elts.len()));
}
```

### 3. Value Changes (`src/value.rs`)
Add Tuple variant:
```rust
pub enum Value {
    // ... existing ...
    Tuple(Rc<Vec<Value>>),  // Immutable sequence
}
```

### 4. VM Changes (`src/vm.rs`)

#### UnpackSequence instruction:
```rust
Instruction::UnpackSequence(count) => {
    let value = self.stack.pop()
        .ok_or_else(|| Value::error(ExceptionType::RuntimeError, "Stack underflow"))?;
    
    let items = match value {
        Value::Tuple(tuple) => tuple.as_ref().clone(),
        Value::List(list) => list.borrow().clone(),
        _ => return Err(Value::error(
            ExceptionType::TypeError,
            format!("cannot unpack non-sequence {}", Self::type_name(&value))
        )),
    };
    
    if items.len() != *count {
        return Err(Value::error(
            ExceptionType::ValueError,
            format!("too many values to unpack (expected {}, got {})", count, items.len())
        ));
    }
    
    // Push items onto stack (in order)
    for item in items {
        self.stack.push(item);
    }
    
    *ip += 1;
}
```

#### BuildTuple instruction:
```rust
Instruction::BuildTuple(count) => {
    let mut items = Vec::new();
    for _ in 0..*count {
        items.push(self.stack.pop()
            .ok_or_else(|| Value::error(ExceptionType::RuntimeError, "Stack underflow"))?);
    }
    items.reverse();
    
    self.stack.push(Value::Tuple(Rc::new(items)));
    *ip += 1;
}
```

## Test Cases

```python
# Test 1: Basic tuple unpacking
a, b = 1, 2
assert a == 1
assert b == 2

# Test 2: Unpack from list
x, y = [10, 20]
assert x == 10
assert y == 20

# Test 3: Unpack from tuple
p, q = (100, 200)
assert p == 100
assert q == 200

# Test 4: Swap variables
a, b = 5, 10
a, b = b, a
assert a == 10
assert b == 5

# Test 5: Multiple values
a, b, c = 1, 2, 3
assert a == 1
assert b == 2
assert c == 3

# Test 6: Function return multiple values
def get_coords():
    return 3, 4

x, y = get_coords()
assert x == 3
assert y == 4

# Test 7: Error - too many values
try:
    a, b = [1, 2, 3]  # ValueError
except ValueError:
    print("caught")

# Test 8: Error - too few values
try:
    a, b, c = [1, 2]  # ValueError
except ValueError:
    print("caught")

# Test 9: Nested unpacking (Phase 2)
# (a, b), c = (1, 2), 3

# Test 10: In for loop
pairs = [(1, 2), (3, 4), (5, 6)]
for x, y in pairs:
    print(x, y)
```

## Supported Patterns (Phase 1)
- `a, b = expr` - basic unpacking
- `a, b, c = expr` - multiple values
- Works with tuples and lists
- Function returns multiple values

## Not Supported (Phase 1)
- Nested unpacking: `(a, b), c = ...`
- Star unpacking: `a, *rest, b = ...`
- Unpacking in function parameters: `def f((a, b)):`

## Verification
- [ ] `cargo test` - all tests pass
- [ ] Add unit tests for unpacking
- [ ] Test error cases (wrong count)
- [ ] Test with lists and tuples
- [ ] `cargo clippy -- -D warnings` - no warnings

## Dependencies
- Requires Tuple type in Value enum

## Notes
- Tuples are immutable, lists are mutable
- Python allows unpacking any iterable
- Count mismatch raises `ValueError`
- Common idiom: `a, b = b, a` for swapping
