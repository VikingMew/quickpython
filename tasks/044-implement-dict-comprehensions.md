# Task 044: Implement Dictionary Comprehensions

**Status**: Pending
**Created**: 2026-02-15
**Priority**: Medium

## Overview

Implement Python's dictionary comprehension syntax for creating dictionaries concisely.

## Background

Dictionary comprehensions provide a concise way to create dictionaries, similar to list comprehensions but producing key-value pairs.

```python
# Basic dictionary comprehension
squares = {x: x**2 for x in range(5)}
# Result: {0: 0, 1: 1, 2: 4, 3: 9, 4: 16}

# With condition
even_squares = {x: x**2 for x in range(10) if x % 2 == 0}
# Result: {0: 0, 2: 4, 4: 16, 6: 36, 8: 64}

# From existing dict
inverted = {v: k for k, v in original.items()}
```

## Requirements

### 1. Compiler Support

In `src/compiler.rs`, handle `ast::Expr::DictComp`:

```rust
fn compile_expr(&mut self, expr: &ast::Expr) {
    match expr {
        ast::Expr::DictComp(ast::ExprDictComp { key, value, generators, .. }) => {
            self.compile_dict_comprehension(key, value, generators);
        }
        // ... other expressions ...
    }
}

fn compile_dict_comprehension(
    &mut self,
    key_expr: &ast::Expr,
    value_expr: &ast::Expr,
    generators: &[ast::Comprehension],
) {
    // Create empty dict
    self.bytecode.push(Instruction::BuildDict(0));
    
    // Compile the comprehension loop(s)
    self.compile_comprehension_loops(generators, |compiler| {
        // Duplicate dict reference
        compiler.bytecode.push(Instruction::Dup);
        
        // Compile key expression
        compiler.compile_expr(key_expr);
        
        // Compile value expression
        compiler.compile_expr(value_expr);
        
        // Set item: dict[key] = value
        compiler.bytecode.push(Instruction::SetItem);
    });
}

fn compile_comprehension_loops<F>(
    &mut self,
    generators: &[ast::Comprehension],
    body_fn: F,
) where
    F: Fn(&mut Self),
{
    if generators.is_empty() {
        return;
    }
    
    let gen = &generators[0];
    
    // Compile iterable
    self.compile_expr(&gen.iter);
    self.bytecode.push(Instruction::GetIter);
    
    // Loop start
    let loop_start = self.bytecode.len();
    let jump_placeholder = self.bytecode.len();
    self.bytecode.push(Instruction::ForIter(0)); // Placeholder
    
    // Store loop variable
    self.compile_store(&gen.target);
    
    // Compile conditions (if any)
    for condition in &gen.ifs {
        self.compile_expr(condition);
        let skip_placeholder = self.bytecode.len();
        self.bytecode.push(Instruction::JumpIfFalse(0)); // Placeholder
        
        // If condition is false, continue to next iteration
        // (will be patched later)
    }
    
    // Compile nested generators or body
    if generators.len() > 1 {
        self.compile_comprehension_loops(&generators[1..], body_fn);
    } else {
        body_fn(self);
    }
    
    // Jump back to loop start
    self.bytecode.push(Instruction::Jump(loop_start));
    
    // Patch ForIter jump target
    let loop_end = self.bytecode.len();
    if let Instruction::ForIter(ref mut target) = self.bytecode[jump_placeholder] {
        *target = loop_end;
    }
}
```

### 2. Bytecode Instructions

No new instructions needed! Dictionary comprehensions can be desugared to:
1. `BuildDict(0)` - Create empty dict
2. Loop with `GetIter` and `ForIter`
3. `Dup` - Duplicate dict reference
4. Compile key and value expressions
5. `SetItem` - Add key-value pair to dict

### 3. VM Support

No VM changes needed - all instructions already exist.

## Test Cases

```rust
#[test]
fn test_dict_comprehension_basic() {
    let mut ctx = Context::new();
    ctx.eval(r#"
result = {x: x * 2 for x in range(5)}
"#).unwrap();
    
    let result = ctx.get("result").unwrap();
    if let Value::Dict(dict) = result {
        let dict = dict.borrow();
        assert_eq!(dict.get(&DictKey::Int(0)), Some(&Value::Int(0)));
        assert_eq!(dict.get(&DictKey::Int(1)), Some(&Value::Int(2)));
        assert_eq!(dict.get(&DictKey::Int(2)), Some(&Value::Int(4)));
        assert_eq!(dict.get(&DictKey::Int(3)), Some(&Value::Int(6)));
        assert_eq!(dict.get(&DictKey::Int(4)), Some(&Value::Int(8)));
    } else {
        panic!("Expected dict");
    }
}

#[test]
fn test_dict_comprehension_with_condition() {
    let mut ctx = Context::new();
    ctx.eval(r#"
result = {x: x * x for x in range(10) if x % 2 == 0}
"#).unwrap();
    
    let result = ctx.get("result").unwrap();
    if let Value::Dict(dict) = result {
        let dict = dict.borrow();
        assert_eq!(dict.len(), 5);
        assert_eq!(dict.get(&DictKey::Int(0)), Some(&Value::Int(0)));
        assert_eq!(dict.get(&DictKey::Int(2)), Some(&Value::Int(4)));
        assert_eq!(dict.get(&DictKey::Int(4)), Some(&Value::Int(16)));
        assert_eq!(dict.get(&DictKey::Int(6)), Some(&Value::Int(36)));
        assert_eq!(dict.get(&DictKey::Int(8)), Some(&Value::Int(64)));
    } else {
        panic!("Expected dict");
    }
}

#[test]
fn test_dict_comprehension_string_keys() {
    let mut ctx = Context::new();
    ctx.eval(r#"
words = ["hello", "world", "python"]
result = {word: len(word) for word in words}
"#).unwrap();
    
    let result = ctx.get("result").unwrap();
    if let Value::Dict(dict) = result {
        let dict = dict.borrow();
        assert_eq!(dict.get(&DictKey::String("hello".to_string())), Some(&Value::Int(5)));
        assert_eq!(dict.get(&DictKey::String("world".to_string())), Some(&Value::Int(5)));
        assert_eq!(dict.get(&DictKey::String("python".to_string())), Some(&Value::Int(6)));
    } else {
        panic!("Expected dict");
    }
}

#[test]
fn test_dict_comprehension_from_list_of_tuples() {
    let mut ctx = Context::new();
    ctx.eval(r#"
pairs = [(1, "a"), (2, "b"), (3, "c")]
result = {k: v for k, v in pairs}
"#).unwrap();
    
    let result = ctx.get("result").unwrap();
    if let Value::Dict(dict) = result {
        let dict = dict.borrow();
        assert_eq!(dict.get(&DictKey::Int(1)), Some(&Value::String("a".to_string())));
        assert_eq!(dict.get(&DictKey::Int(2)), Some(&Value::String("b".to_string())));
        assert_eq!(dict.get(&DictKey::Int(3)), Some(&Value::String("c".to_string())));
    } else {
        panic!("Expected dict");
    }
}

#[test]
fn test_dict_comprehension_invert_dict() {
    let mut ctx = Context::new();
    ctx.eval(r#"
original = {"a": 1, "b": 2, "c": 3}
result = {v: k for k, v in original.items()}
"#).unwrap();
    
    let result = ctx.get("result").unwrap();
    if let Value::Dict(dict) = result {
        let dict = dict.borrow();
        assert_eq!(dict.get(&DictKey::Int(1)), Some(&Value::String("a".to_string())));
        assert_eq!(dict.get(&DictKey::Int(2)), Some(&Value::String("b".to_string())));
        assert_eq!(dict.get(&DictKey::Int(3)), Some(&Value::String("c".to_string())));
    } else {
        panic!("Expected dict");
    }
}

#[test]
fn test_dict_comprehension_nested_loops() {
    let mut ctx = Context::new();
    ctx.eval(r#"
result = {x * 10 + y: x + y for x in range(3) for y in range(3)}
"#).unwrap();
    
    let result = ctx.get("result").unwrap();
    if let Value::Dict(dict) = result {
        let dict = dict.borrow();
        assert_eq!(dict.len(), 9);
        assert_eq!(dict.get(&DictKey::Int(0)), Some(&Value::Int(0)));   // 0*10+0: 0+0
        assert_eq!(dict.get(&DictKey::Int(1)), Some(&Value::Int(1)));   // 0*10+1: 0+1
        assert_eq!(dict.get(&DictKey::Int(10)), Some(&Value::Int(1)));  // 1*10+0: 1+0
        assert_eq!(dict.get(&DictKey::Int(22)), Some(&Value::Int(4)));  // 2*10+2: 2+2
    } else {
        panic!("Expected dict");
    }
}

#[test]
fn test_dict_comprehension_multiple_conditions() {
    let mut ctx = Context::new();
    ctx.eval(r#"
result = {x: x * x for x in range(20) if x % 2 == 0 if x % 3 == 0}
"#).unwrap();
    
    let result = ctx.get("result").unwrap();
    if let Value::Dict(dict) = result {
        let dict = dict.borrow();
        // Should only include numbers divisible by both 2 and 3 (i.e., 6)
        assert_eq!(dict.len(), 4);  // 0, 6, 12, 18
        assert_eq!(dict.get(&DictKey::Int(0)), Some(&Value::Int(0)));
        assert_eq!(dict.get(&DictKey::Int(6)), Some(&Value::Int(36)));
        assert_eq!(dict.get(&DictKey::Int(12)), Some(&Value::Int(144)));
        assert_eq!(dict.get(&DictKey::Int(18)), Some(&Value::Int(324)));
    } else {
        panic!("Expected dict");
    }
}

#[test]
fn test_dict_comprehension_empty() {
    let mut ctx = Context::new();
    ctx.eval(r#"
result = {x: x for x in range(10) if x > 100}
"#).unwrap();
    
    let result = ctx.get("result").unwrap();
    if let Value::Dict(dict) = result {
        let dict = dict.borrow();
        assert_eq!(dict.len(), 0);
    } else {
        panic!("Expected dict");
    }
}

#[test]
fn test_dict_comprehension_complex_expressions() {
    let mut ctx = Context::new();
    ctx.eval(r#"
result = {str(x): x * x + x for x in range(5)}
"#).unwrap();
    
    let result = ctx.get("result").unwrap();
    if let Value::Dict(dict) = result {
        let dict = dict.borrow();
        assert_eq!(dict.get(&DictKey::String("0".to_string())), Some(&Value::Int(0)));
        assert_eq!(dict.get(&DictKey::String("1".to_string())), Some(&Value::Int(2)));
        assert_eq!(dict.get(&DictKey::String("2".to_string())), Some(&Value::Int(6)));
        assert_eq!(dict.get(&DictKey::String("3".to_string())), Some(&Value::Int(12)));
        assert_eq!(dict.get(&DictKey::String("4".to_string())), Some(&Value::Int(20)));
    } else {
        panic!("Expected dict");
    }
}

#[test]
fn test_dict_comprehension_duplicate_keys() {
    let mut ctx = Context::new();
    ctx.eval(r#"
# Later values should overwrite earlier ones
result = {x % 3: x for x in range(10)}
"#).unwrap();
    
    let result = ctx.get("result").unwrap();
    if let Value::Dict(dict) = result {
        let dict = dict.borrow();
        assert_eq!(dict.len(), 3);
        // Last occurrence of each key wins
        assert_eq!(dict.get(&DictKey::Int(0)), Some(&Value::Int(9)));  // 0, 3, 6, 9
        assert_eq!(dict.get(&DictKey::Int(1)), Some(&Value::Int(7)));  // 1, 4, 7
        assert_eq!(dict.get(&DictKey::Int(2)), Some(&Value::Int(8)));  // 2, 5, 8
    } else {
        panic!("Expected dict");
    }
}
```

## Implementation Notes

1. **Desugaring Strategy**:
   - Dictionary comprehensions are syntactic sugar
   - Desugar to: empty dict + loop + SetItem operations
   - No new VM instructions needed

2. **Key Uniqueness**:
   - If multiple iterations produce the same key, last value wins
   - This matches Python's behavior

3. **Nested Loops**:
   - Multiple `for` clauses create nested loops
   - `{x+y: x*y for x in range(3) for y in range(3)}`
   - Equivalent to nested for loops

4. **Multiple Conditions**:
   - Multiple `if` clauses are ANDed together
   - `{x: x for x in range(10) if x > 5 if x % 2 == 0}`
   - Equivalent to `if x > 5 and x % 2 == 0`

5. **Performance**:
   - Dictionary comprehensions are generally faster than equivalent loops
   - Pre-allocating dict size could be an optimization

## Success Criteria

- [ ] Basic dictionary comprehensions work
- [ ] Comprehensions with conditions work
- [ ] Nested loops in comprehensions work
- [ ] Multiple conditions work
- [ ] String keys and complex expressions work
- [ ] Duplicate keys handled correctly (last wins)
- [ ] Empty comprehensions work
- [ ] All test cases pass
- [ ] Code passes `cargo fmt` and `cargo clippy`

## Related Tasks

- Task 041: Implement `is` and `is not` operators
- Task 042: Implement `with` statement
- Task 043: Implement generators and `yield`
- Task 045: Implement `pass` statement

## References

- PEP 274: Dict Comprehensions
- Python documentation: https://docs.python.org/3/tutorial/datastructures.html#dictionaries
