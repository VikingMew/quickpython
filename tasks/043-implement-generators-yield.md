# Task 043: Implement Generators and `yield`

**Status**: Pending
**Created**: 2026-02-15
**Priority**: High

## Overview

Implement Python generators using the `yield` keyword, allowing functions to produce a sequence of values lazily rather than computing them all at once.

## Background

Generators are a powerful feature for creating iterators. They use `yield` to produce values one at a time, maintaining their state between calls.

```python
def count_up_to(n):
    i = 0
    while i < n:
        yield i
        i += 1

for num in count_up_to(5):
    print(num)  # Prints 0, 1, 2, 3, 4
```

## Requirements

### 1. Add Generator Value Type

In `src/value.rs`:
```rust
pub enum Value {
    // ... existing variants ...
    Generator(Rc<RefCell<Generator>>),
}

pub struct Generator {
    pub function: Function,
    pub frame: Frame,
    pub ip: usize,
    pub finished: bool,
}
```

### 2. Add Bytecode Instructions

In `src/bytecode.rs`:
```rust
pub enum Instruction {
    // ... existing instructions ...
    Yield,              // Yield a value from generator
    YieldFrom,          // Yield from another generator (for yield from)
    ResumeGenerator,    // Resume generator execution
}
```

### 3. Compiler Support

In `src/compiler.rs`:

```rust
fn compile_function(&mut self, name: &str, args: &ast::Arguments, body: &[ast::Stmt]) {
    // Check if function contains yield
    let is_generator = self.contains_yield(body);
    
    // ... existing function compilation ...
    
    self.bytecode.push(Instruction::MakeFunction {
        name: name.to_string(),
        arg_count: args.args.len(),
        is_async: false,
        is_generator,  // New field
    });
}

fn contains_yield(&self, stmts: &[ast::Stmt]) -> bool {
    for stmt in stmts {
        match stmt {
            ast::Stmt::Expr(ast::StmtExpr { value, .. }) => {
                if matches!(value.as_ref(), ast::Expr::Yield(_) | ast::Expr::YieldFrom(_)) {
                    return true;
                }
            }
            // Recursively check nested statements
            _ => {}
        }
    }
    false
}

fn compile_expr(&mut self, expr: &ast::Expr) {
    match expr {
        ast::Expr::Yield(ast::ExprYield { value, .. }) => {
            if let Some(val) = value {
                self.compile_expr(val);
            } else {
                self.bytecode.push(Instruction::PushNone);
            }
            self.bytecode.push(Instruction::Yield);
        }
        ast::Expr::YieldFrom(ast::ExprYieldFrom { value, .. }) => {
            self.compile_expr(value);
            self.bytecode.push(Instruction::YieldFrom);
        }
        // ... other expressions ...
    }
}
```

### 4. VM Execution

In `src/vm.rs`:

```rust
Instruction::MakeFunction { name, arg_count, is_async, is_generator } => {
    // ... existing code ...
    
    let func = Function {
        name: name.clone(),
        bytecode: func_bytecode,
        arg_count: *arg_count,
        is_async: *is_async,
        is_generator: *is_generator,  // New field
    };
    
    self.stack.push(Value::Function(func));
    *ip += 1;
}

Instruction::Call(arg_count) => {
    // ... existing code ...
    
    match func {
        Value::Function(f) if f.is_generator => {
            // Create generator instead of executing function
            let generator = Generator {
                function: f.clone(),
                frame: Frame {
                    locals: HashMap::new(),
                    return_address: 0,
                },
                ip: 0,
                finished: false,
            };
            
            // Bind arguments to generator frame
            for i in 0..*arg_count {
                let arg = args[i].clone();
                generator.frame.locals.insert(i, arg);
            }
            
            self.stack.push(Value::Generator(Rc::new(RefCell::new(generator))));
        }
        // ... existing function call code ...
    }
}

Instruction::Yield => {
    // Save current state and return value
    let value = self.stack.pop()
        .ok_or_else(|| Value::error(ExceptionType::RuntimeError, "Stack underflow"))?;
    
    // Mark that we're yielding (need to save IP and frame state)
    // This is handled by the generator's next() method
    self.stack.push(value);
    
    // Return special marker to indicate yield
    return Ok(YieldResult::Yielded);
}

// Add next() method for generators
impl Value {
    pub fn generator_next(&self) -> Result<Value, Value> {
        match self {
            Value::Generator(gen) => {
                let mut gen = gen.borrow_mut();
                
                if gen.finished {
                    return Err(Value::error(ExceptionType::StopIteration, ""));
                }
                
                // Resume execution from saved IP
                let mut vm = VM::new();
                vm.frames.push(gen.frame.clone());
                vm.ip = gen.ip;
                
                match vm.execute(&gen.function.bytecode, &mut HashMap::new())? {
                    YieldResult::Yielded => {
                        // Save state
                        gen.ip = vm.ip;
                        gen.frame = vm.frames.pop().unwrap();
                        
                        // Return yielded value
                        Ok(vm.stack.pop().unwrap())
                    }
                    YieldResult::Returned(val) => {
                        gen.finished = true;
                        Err(Value::error(ExceptionType::StopIteration, ""))
                    }
                }
            }
            _ => Err(Value::error(ExceptionType::TypeError, "not a generator"))
        }
    }
}

// Modify GetIter to handle generators
Instruction::GetIter => {
    let value = self.stack.pop()
        .ok_or_else(|| Value::error(ExceptionType::RuntimeError, "Stack underflow"))?;
    
    match value {
        Value::Generator(_) => {
            // Generators are already iterators
            self.stack.push(value);
        }
        // ... existing iterator creation code ...
    }
    *ip += 1;
}

// Modify ForIter to handle generators
Instruction::ForIter(jump_target) => {
    let iter = self.stack.last()
        .ok_or_else(|| Value::error(ExceptionType::RuntimeError, "Stack underflow"))?;
    
    match iter {
        Value::Generator(gen) => {
            match gen.borrow().generator_next() {
                Ok(value) => {
                    self.stack.push(value);
                    *ip += 1;
                }
                Err(e) if e.is_stop_iteration() => {
                    self.stack.pop(); // Remove generator
                    *ip = *jump_target;
                }
                Err(e) => return Err(e),
            }
        }
        // ... existing iterator handling ...
    }
}
```

### 5. Update Function Type

In `src/value.rs`:
```rust
pub struct Function {
    pub name: String,
    pub bytecode: ByteCode,
    pub arg_count: usize,
    pub is_async: bool,
    pub is_generator: bool,  // New field
}
```

## Test Cases

```rust
#[test]
fn test_generator_basic() {
    let mut ctx = Context::new();
    ctx.eval(r#"
def count_up_to(n):
    i = 0
    while i < n:
        yield i
        i += 1

result = []
for num in count_up_to(3):
    result.append(num)
"#).unwrap();
    
    let result = ctx.get("result").unwrap();
    assert_eq!(result, Value::List(Rc::new(RefCell::new(ListValue {
        items: vec![Value::Int(0), Value::Int(1), Value::Int(2)],
        version: 0,
    }))));
}

#[test]
fn test_generator_manual_iteration() {
    let mut ctx = Context::new();
    ctx.eval(r#"
def simple_gen():
    yield 1
    yield 2
    yield 3

gen = simple_gen()
a = next(gen)
b = next(gen)
c = next(gen)
"#).unwrap();
    
    assert_eq!(ctx.get("a"), Some(Value::Int(1)));
    assert_eq!(ctx.get("b"), Some(Value::Int(2)));
    assert_eq!(ctx.get("c"), Some(Value::Int(3)));
}

#[test]
fn test_generator_stop_iteration() {
    let mut ctx = Context::new();
    let result = ctx.eval(r#"
def simple_gen():
    yield 1

gen = simple_gen()
next(gen)
next(gen)  # Should raise StopIteration
"#);
    
    assert!(result.is_err());
}

#[test]
fn test_generator_with_return() {
    let mut ctx = Context::new();
    ctx.eval(r#"
def gen_with_return():
    yield 1
    yield 2
    return

result = []
for x in gen_with_return():
    result.append(x)
"#).unwrap();
    
    let result = ctx.get("result").unwrap();
    assert_eq!(result, Value::List(Rc::new(RefCell::new(ListValue {
        items: vec![Value::Int(1), Value::Int(2)],
        version: 0,
    }))));
}

#[test]
fn test_generator_with_arguments() {
    let mut ctx = Context::new();
    ctx.eval(r#"
def repeat(value, times):
    for i in range(times):
        yield value

result = []
for x in repeat("hello", 3):
    result.append(x)
"#).unwrap();
    
    let result = ctx.get("result").unwrap();
    // Should contain ["hello", "hello", "hello"]
}

#[test]
fn test_generator_expression() {
    let mut ctx = Context::new();
    ctx.eval(r#"
# Generator expression
gen = (x * 2 for x in range(5))
result = []
for x in gen:
    result.append(x)
"#).unwrap();
    
    let result = ctx.get("result").unwrap();
    // Should contain [0, 2, 4, 6, 8]
}

#[test]
fn test_generator_state_preservation() {
    let mut ctx = Context::new();
    ctx.eval(r#"
def stateful_gen():
    x = 0
    while True:
        x += 1
        yield x

gen = stateful_gen()
a = next(gen)
b = next(gen)
c = next(gen)
"#).unwrap();
    
    assert_eq!(ctx.get("a"), Some(Value::Int(1)));
    assert_eq!(ctx.get("b"), Some(Value::Int(2)));
    assert_eq!(ctx.get("c"), Some(Value::Int(3)));
}

#[test]
fn test_nested_generators() {
    let mut ctx = Context::new();
    ctx.eval(r#"
def outer():
    yield 1
    for x in inner():
        yield x
    yield 4

def inner():
    yield 2
    yield 3

result = []
for x in outer():
    result.append(x)
"#).unwrap();
    
    let result = ctx.get("result").unwrap();
    // Should contain [1, 2, 3, 4]
}

#[test]
fn test_yield_none() {
    let mut ctx = Context::new();
    ctx.eval(r#"
def gen_none():
    yield
    yield None
    yield

result = []
for x in gen_none():
    result.append(x)
"#).unwrap();
    
    let result = ctx.get("result").unwrap();
    // Should contain [None, None, None]
}
```

## Implementation Notes

1. **Generator State**:
   - Generators must preserve local variables between yields
   - Instruction pointer must be saved and restored
   - Stack state must be maintained

2. **StopIteration Exception**:
   - Raised when generator is exhausted
   - `for` loops catch this automatically
   - Manual `next()` calls propagate it

3. **Generator vs Regular Function**:
   - Calling a generator function returns a generator object
   - Calling a regular function executes it immediately
   - Determined by presence of `yield` in function body

4. **Generator Expressions**:
   - Similar to list comprehensions but with parentheses
   - `(x*2 for x in range(10))`
   - Creates a generator, not a list

5. **yield from** (Optional):
   - `yield from iterable` delegates to another generator
   - Can be implemented later as an enhancement

## Success Criteria

- [ ] Functions with `yield` create generator objects
- [ ] Generators can be iterated with `for` loops
- [ ] `next()` function works on generators
- [ ] Generator state is preserved between yields
- [ ] StopIteration is raised when generator exhausted
- [ ] Generator expressions work (optional)
- [ ] All test cases pass
- [ ] Code passes `cargo fmt` and `cargo clippy`

## Related Tasks

- Task 041: Implement `is` and `is not` operators
- Task 042: Implement `with` statement
- Task 044: Implement dictionary comprehensions

## References

- PEP 255: Simple Generators
- PEP 342: Coroutines via Enhanced Generators
- Python documentation: https://docs.python.org/3/reference/expressions.html#yield-expressions
