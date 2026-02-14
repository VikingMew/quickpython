# Task 026: Fix print() with Multiple Arguments

## Status
- [x] Completed

## Priority
Critical - blocking CI (examples/break_continue.py fails, nested loops broken)

## Description
Fix `print()` function to properly handle multiple arguments. Currently it only pops one value from the stack, leaving extra values that corrupt the stack state.

## Current Issue
```python
print(1, 2)  # Only prints 2, leaves 1 on stack
for i in range(2):
    for j in range(2):
        print(i, j)  # Crashes with "Expected iterator"
```

## Root Cause
1. Compiler pushes all arguments onto stack: `[arg1, arg2, ...]`
2. `Print` instruction only pops one value
3. Extra values remain on stack, corrupting stack state
4. In nested loops, this causes ForIter to see wrong values

## Implementation

### 1. Bytecode Changes (`src/bytecode.rs`)
Change `Print` instruction to include argument count:
```rust
Print(usize),  // Print n values from stack, separated by spaces
```

### 2. Compiler Changes (`src/compiler.rs`)
Pass argument count to Print instruction:
```rust
"print" => {
    let arg_count = call.args.len();
    for arg in &call.args {
        self.compile_expr(arg, bytecode)?;
    }
    bytecode.push(Instruction::Print(arg_count));
    return Ok(());
}
```

### 3. VM Changes (`src/vm.rs`)
Pop all arguments and print them:
```rust
Instruction::Print(arg_count) => {
    let mut values = Vec::new();
    for _ in 0..*arg_count {
        values.push(self.stack.pop().ok_or(...)?);
    }
    values.reverse();  // Restore original order
    
    for (i, value) in values.iter().enumerate() {
        if i > 0 {
            print!(" ");  // Space separator
        }
        Self::print_value(value);
    }
    println!();  // Newline at end
    
    self.stack.push(Value::None);
    *ip += 1;
}
```

### 4. Serializer Changes (`src/serializer.rs`)
Update serialization to handle argument count (currently not serializable).

## Test Cases

```python
# Test 1: Single argument
print(42)  # 42

# Test 2: Multiple arguments
print(1, 2, 3)  # 1 2 3

# Test 3: No arguments
print()  # (empty line)

# Test 4: Nested loops (regression test)
for i in range(2):
    for j in range(2):
        print(i, j)
# Output:
# 0 0
# 0 1
# 1 0
# 1 1
```

## Verification
- [x] `cargo test` - all tests pass (131 tests)
- [x] `cargo run -- run examples/break_continue.py` - PASS
- [x] Unit tests for print with multiple args (existing tests updated)
- [x] `cargo clippy -- -D warnings` - no warnings

## Notes
- Python's print() separates arguments with spaces by default
- print() with no arguments prints an empty line
- This is a critical bug affecting stack integrity

## Dependencies
None - standalone fix
