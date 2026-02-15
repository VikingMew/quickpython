# Generators Specification

## Overview

Generators are functions that can pause execution and resume later, yielding values one at a time. They provide a memory-efficient way to create iterators.

## Core Concepts

### 1. Generator Functions

A function containing `yield` is a generator function:

```python
def count_up_to(n):
    i = 0
    while i < n:
        yield i
        i += 1

# Calling returns a generator object, doesn't execute yet
gen = count_up_to(5)

# Iterate to get values
for num in gen:
    print(num)  # 0, 1, 2, 3, 4
```

### 2. Generator Objects

Generator objects are iterators that:
- Maintain execution state between yields
- Can be iterated with `for` loops
- Support `next()` function
- Raise `StopIteration` when exhausted

### 3. Yield Expression

`yield` pauses execution and returns a value:

```python
def simple_gen():
    yield 1
    yield 2
    yield 3

# Each next() call resumes until next yield
gen = simple_gen()
print(next(gen))  # 1
print(next(gen))  # 2
print(next(gen))  # 3
print(next(gen))  # StopIteration
```

## Implementation Design

### Value Type

Add `Generator` variant to `Value` enum:

```rust
pub enum Value {
    // ... existing variants
    Generator(Rc<RefCell<GeneratorState>>),
}

pub struct GeneratorState {
    pub function: Function,      // The generator function
    pub frame: Frame,             // Saved execution frame
    pub ip: usize,                // Instruction pointer
    pub stack: Vec<Value>,        // Saved stack state
    pub finished: bool,           // Whether generator is exhausted
}
```

### Bytecode Instructions

```rust
pub enum Instruction {
    // ... existing instructions
    
    Yield,              // Yield a value and pause execution
    ResumeGenerator,    // Resume generator execution (internal)
}
```

### Compiler Changes

1. **Detect Generator Functions**
   - Scan function AST for `yield` expressions
   - Mark function as generator in `MakeFunction`

2. **Compile Yield**
   - `yield expr` compiles to: compile(expr) + Yield
   - `yield` (no value) compiles to: LoadConst(None) + Yield

3. **Function Metadata**
   - Add `is_generator: bool` to `Function` struct
   - Generator functions create generator objects when called

### VM Execution

#### Calling Generator Function

When calling a function marked as generator:
```rust
if function.is_generator {
    // Don't execute, create generator object
    let generator = GeneratorState {
        function: function.clone(),
        frame: Frame::new(),  // Empty frame
        ip: 0,
        stack: Vec::new(),
        finished: false,
    };
    return Value::Generator(Rc::new(RefCell::new(generator)));
}
```

#### Yield Instruction

```rust
Instruction::Yield => {
    let value = self.stack.pop()?;
    
    // Save current state in generator
    // (This happens in next() implementation)
    
    return Ok(value);  // Return yielded value
}
```

#### next() Builtin Function

```rust
fn builtin_next(args: Vec<Value>) -> Result<Value, Value> {
    let generator = args[0].as_generator()?;
    let mut gen_state = generator.borrow_mut();
    
    if gen_state.finished {
        return Err(Value::exception(ExceptionType::StopIteration, ""));
    }
    
    // Restore generator state
    let saved_frame = self.current_frame;
    let saved_ip = self.ip;
    let saved_stack = self.stack.clone();
    
    self.current_frame = gen_state.frame;
    self.ip = gen_state.ip;
    self.stack = gen_state.stack.clone();
    
    // Execute until Yield or Return
    let result = self.run_until_yield_or_return()?;
    
    match result {
        YieldResult::Yielded(value) => {
            // Save state for next iteration
            gen_state.frame = self.current_frame;
            gen_state.ip = self.ip;
            gen_state.stack = self.stack.clone();
            
            // Restore caller state
            self.current_frame = saved_frame;
            self.ip = saved_ip;
            self.stack = saved_stack;
            
            Ok(value)
        }
        YieldResult::Returned(_) => {
            gen_state.finished = true;
            
            // Restore caller state
            self.current_frame = saved_frame;
            self.ip = saved_ip;
            self.stack = saved_stack;
            
            Err(Value::exception(ExceptionType::StopIteration, ""))
        }
    }
}
```

#### Integration with For Loops

Generators work with existing `GetIter` and `ForIter`:

```rust
Instruction::GetIter => {
    let value = self.stack.pop()?;
    match value {
        Value::Generator(_) => {
            // Generator is already an iterator
            self.stack.push(value);
        }
        // ... other types
    }
}

Instruction::ForIter(end_offset) => {
    let iter = self.stack.last()?.clone();
    match iter {
        Value::Generator(gen) => {
            match builtin_next(vec![Value::Generator(gen.clone())]) {
                Ok(value) => {
                    self.stack.push(value);
                }
                Err(e) if e.is_stop_iteration() => {
                    self.stack.pop(); // Remove iterator
                    self.ip += end_offset;
                }
                Err(e) => return Err(e),
            }
        }
        // ... other iterator types
    }
}
```

## Key Features

### 1. State Preservation

Generator must preserve:
- Local variables (in Frame)
- Instruction pointer (where to resume)
- Stack state (intermediate values)
- Loop counters and control flow state

### 2. Memory Efficiency

Generators are lazy - values computed on demand:

```python
# This doesn't create a million-element list
def big_range(n):
    i = 0
    while i < n:
        yield i
        i += 1

# Only one value in memory at a time
for x in big_range(1000000):
    if x > 10:
        break
```

### 3. Generator Expressions

Generator expressions are syntactic sugar:

```python
# Generator expression
gen = (x * 2 for x in range(10))

# Equivalent to:
def _gen():
    for x in range(10):
        yield x * 2
gen = _gen()
```

Compile generator expressions by:
1. Create anonymous generator function
2. Compile loop with yield
3. Call function to get generator

### 4. Return in Generators

`return` in generator raises `StopIteration`:

```python
def gen_with_return():
    yield 1
    yield 2
    return 42  # Raises StopIteration (value ignored in simple case)
    yield 3    # Never reached
```

## Implementation Phases

### Phase 1: Basic Generators (Task 043)

- [x] Add `Generator` value type
- [x] Add `Yield` instruction
- [x] Compiler detects generator functions
- [x] VM creates generator objects
- [x] Implement `next()` builtin
- [x] Integration with `for` loops
- [x] State preservation (frame, IP, stack)

**Test cases:**
- Basic generator with yield in loop
- Manual iteration with next()
- StopIteration when exhausted
- Generator with return statement
- Generator with arguments
- State preservation between yields
- Nested loops in generator
- yield without value (yields None)

### Phase 2: Generator Expressions

- [ ] Compile generator expressions
- [ ] Anonymous generator functions
- [ ] Closure support for generator expressions

### Phase 3: Advanced Features

- [ ] `yield from` for generator delegation
- [ ] `send()` method for two-way communication
- [ ] `throw()` method for exception injection
- [ ] `close()` method for cleanup
- [ ] Generator return values in StopIteration

## Differences from Async/Await

| Feature | Generators | Async/Await |
|---------|-----------|-------------|
| Keyword | `yield` | `await` |
| Purpose | Lazy iteration | Asynchronous I/O |
| Execution | Synchronous | Asynchronous |
| State | Saved in generator | Saved in coroutine |
| Integration | For loops | Event loop |
| Use case | Memory efficiency | Concurrency |

**Key difference:** Generators are synchronous iterators, while async/await is for asynchronous concurrency.

## Examples

### Basic Generator

```python
def fibonacci():
    a, b = 0, 1
    while True:
        yield a
        a, b = b, a + b

gen = fibonacci()
print(next(gen))  # 0
print(next(gen))  # 1
print(next(gen))  # 1
print(next(gen))  # 2
print(next(gen))  # 3
```

### Generator with State

```python
def counter(start=0):
    count = start
    while True:
        yield count
        count += 1

c = counter(10)
print(next(c))  # 10
print(next(c))  # 11
print(next(c))  # 12
```

### Generator in For Loop

```python
def squares(n):
    for i in range(n):
        yield i * i

for sq in squares(5):
    print(sq)  # 0, 1, 4, 9, 16
```

### Multiple Generators

```python
def gen1():
    yield 1
    yield 2

def gen2():
    yield 3
    yield 4

g1 = gen1()
g2 = gen2()

print(next(g1))  # 1
print(next(g2))  # 3
print(next(g1))  # 2
print(next(g2))  # 4
```

## Performance Considerations

1. **Memory**: Generators use constant memory regardless of sequence length
2. **Lazy evaluation**: Values computed only when needed
3. **State overhead**: Each generator maintains frame + stack state
4. **No random access**: Can't index into generator like a list

## Error Handling

### 1. Yield Outside Generator

```python
def not_a_generator():
    x = yield 1  # SyntaxError if not in generator context
```

Compiler must track whether currently compiling a generator.

### 2. StopIteration

```python
gen = simple_gen()
next(gen)  # OK
next(gen)  # OK
next(gen)  # Raises StopIteration
```

### 3. Generator Already Executing

```python
def recursive_gen():
    yield next(recursive_gen())  # Error: generator already executing
```

Track execution state to prevent re-entrance.

## Testing Strategy

Key test scenarios:
1. Basic yield in loop
2. Manual next() calls
3. StopIteration handling
4. Generator with arguments
5. Generator with return
6. State preservation across yields
7. Multiple independent generators
8. Nested loops in generator
9. Generator exhaustion
10. Integration with for loops

## References

- PEP 255: Simple Generators
- PEP 342: Coroutines via Enhanced Generators  
- PEP 380: Syntax for Delegating to a Subgenerator (yield from)
- Python Generator Documentation
