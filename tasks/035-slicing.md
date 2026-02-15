# Task 035: Slicing (Lists and Strings)

## Status
- [ ] Not started

## Priority
Medium - useful but can be worked around

## Description
Implement slice syntax for lists and strings: `lst[start:end:step]`

## Current Issue
```python
lst = [1, 2, 3, 4, 5]
sub = lst[1:3]  # Error: Unsupported subscript type
```

## Implementation

### 1. Bytecode Changes (`src/bytecode.rs`)
Add new instruction:
```rust
pub enum Instruction {
    // ... existing ...
    Slice,  // Create slice from TOS3:TOS2:TOS1 (start:stop:step)
    GetItemSlice,  // Get item using slice: TOS1[TOS]
}
```

### 2. Compiler Changes (`src/compiler.rs`)

Handle `Subscript` with `Slice`:
```rust
ast::Expr::Subscript(sub) => {
    // Compile the object
    self.compile_expr(&sub.value, bytecode)?;
    
    match &*sub.slice {
        ast::Expr::Slice(slice) => {
            // Handle slice: obj[start:stop:step]
            
            // Push start (or None)
            if let Some(start) = &slice.lower {
                self.compile_expr(start, bytecode)?;
            } else {
                bytecode.push(Instruction::PushNone);
            }
            
            // Push stop (or None)
            if let Some(stop) = &slice.upper {
                self.compile_expr(stop, bytecode)?;
            } else {
                bytecode.push(Instruction::PushNone);
            }
            
            // Push step (or None)
            if let Some(step) = &slice.step {
                self.compile_expr(step, bytecode)?;
            } else {
                bytecode.push(Instruction::PushNone);
            }
            
            // Create slice and get item
            bytecode.push(Instruction::Slice);
            bytecode.push(Instruction::GetItemSlice);
        }
        _ => {
            // Regular index
            self.compile_expr(&sub.slice, bytecode)?;
            bytecode.push(Instruction::GetItem);
        }
    }
}
```

### 3. Value Changes (`src/value.rs`)
Add Slice type:
```rust
pub enum Value {
    // ... existing ...
    Slice {
        start: Option<i32>,
        stop: Option<i32>,
        step: Option<i32>,
    },
}
```

### 4. VM Changes (`src/vm.rs`)

#### Slice instruction:
```rust
Instruction::Slice => {
    let step = self.stack.pop().ok_or_else(|| Value::error(ExceptionType::RuntimeError, "Stack underflow"))?;
    let stop = self.stack.pop().ok_or_else(|| Value::error(ExceptionType::RuntimeError, "Stack underflow"))?;
    let start = self.stack.pop().ok_or_else(|| Value::error(ExceptionType::RuntimeError, "Stack underflow"))?;
    
    let start_val = match start {
        Value::None => None,
        Value::Int(i) => Some(i),
        _ => return Err(Value::error(ExceptionType::TypeError, "slice indices must be integers or None")),
    };
    
    let stop_val = match stop {
        Value::None => None,
        Value::Int(i) => Some(i),
        _ => return Err(Value::error(ExceptionType::TypeError, "slice indices must be integers or None")),
    };
    
    let step_val = match step {
        Value::None => Some(1),  // Default step is 1
        Value::Int(i) => {
            if i == 0 {
                return Err(Value::error(ExceptionType::ValueError, "slice step cannot be zero"));
            }
            Some(i)
        }
        _ => return Err(Value::error(ExceptionType::TypeError, "slice indices must be integers or None")),
    };
    
    self.stack.push(Value::Slice {
        start: start_val,
        stop: stop_val,
        step: step_val,
    });
    *ip += 1;
}
```

#### GetItemSlice instruction:
```rust
Instruction::GetItemSlice => {
    let slice = self.stack.pop().ok_or_else(|| Value::error(ExceptionType::RuntimeError, "Stack underflow"))?;
    let obj = self.stack.pop().ok_or_else(|| Value::error(ExceptionType::RuntimeError, "Stack underflow"))?;
    
    let Value::Slice { start, stop, step } = slice else {
        return Err(Value::error(ExceptionType::TypeError, "expected slice"));
    };
    
    match obj {
        Value::List(list) => {
            let items = list.borrow();
            let len = items.len() as i32;
            let result = Self::slice_sequence(&items, len, start, stop, step.unwrap())?;
            self.stack.push(Value::List(Rc::new(RefCell::new(result))));
        }
        Value::String(s) => {
            let chars: Vec<char> = s.chars().collect();
            let len = chars.len() as i32;
            let indices = Self::compute_slice_indices(len, start, stop, step.unwrap());
            
            let result: String = indices.iter()
                .filter_map(|&i| chars.get(i as usize))
                .collect();
            
            self.stack.push(Value::String(result));
        }
        _ => return Err(Value::error(
            ExceptionType::TypeError,
            format!("'{}' object is not subscriptable", Self::type_name(&obj))
        )),
    }
    *ip += 1;
}

// Helper: compute slice indices
fn compute_slice_indices(len: i32, start: Option<i32>, stop: Option<i32>, step: i32) -> Vec<i32> {
    let start = start.unwrap_or(if step > 0 { 0 } else { len - 1 });
    let stop = stop.unwrap_or(if step > 0 { len } else { -len - 1 });
    
    // Normalize negative indices
    let start = if start < 0 { (len + start).max(0) } else { start.min(len) };
    let stop = if stop < 0 { (len + stop).max(-1) } else { stop.min(len) };
    
    let mut indices = Vec::new();
    if step > 0 {
        let mut i = start;
        while i < stop {
            if i >= 0 && i < len {
                indices.push(i);
            }
            i += step;
        }
    } else {
        let mut i = start;
        while i > stop {
            if i >= 0 && i < len {
                indices.push(i);
            }
            i += step;
        }
    }
    
    indices
}
```

## Test Cases

```python
# Test 1: Basic list slice
lst = [1, 2, 3, 4, 5]
assert lst[1:3] == [2, 3]

# Test 2: String slice
s = "hello"
assert s[1:4] == "ell"

# Test 3: Omit start
assert lst[:3] == [1, 2, 3]

# Test 4: Omit stop
assert lst[2:] == [3, 4, 5]

# Test 5: Omit both (copy)
assert lst[:] == [1, 2, 3, 4, 5]

# Test 6: Negative indices
assert lst[-2:] == [4, 5]
assert lst[:-2] == [1, 2, 3]

# Test 7: Step
assert lst[::2] == [1, 3, 5]
assert lst[1::2] == [2, 4]

# Test 8: Reverse
assert lst[::-1] == [5, 4, 3, 2, 1]

# Test 9: Out of bounds (no error)
assert lst[10:20] == []
assert lst[-100:2] == [1, 2]

# Test 10: Empty slice
assert lst[3:1] == []
```

## Python Semantics
- `lst[start:stop]` - items from start to stop-1
- `lst[start:stop:step]` - every step-th item
- Negative indices count from end
- Out-of-bounds indices are clamped (no error)
- Slicing creates a new list/string (copy)

## Verification
- [ ] `cargo test` - all tests pass
- [ ] Add unit tests for list and string slicing
- [ ] Test negative indices
- [ ] Test step parameter
- [ ] `cargo clippy -- -D warnings` - no warnings

## Dependencies
None

## Notes
- Slicing always returns a new object (copy)
- Step can be negative (reverse)
- Python's slice object can be created: `slice(1, 3)`
- Assignment to slices (`lst[1:3] = [...]`) is Phase 2
