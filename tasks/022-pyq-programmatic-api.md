# Task 022: Expose pyq bytecode API for programmatic use

## Status
- **Status**: DONE
- **Priority**: P2
- **Estimated effort**: Medium

## Goal

Expose the compiler and serializer as public API so that Rust applications can compile Python source to `.pyq` bytecode, serialize/deserialize it, and execute it directly — without going through the CLI.

Currently `.pyq` is only usable via the CLI (`quickpython compile` / `quickpython run`). The compiler and serializer modules are internal. This task makes them part of the public library API and adds a use case demonstrating the workflow.

## Background

The `.pyq` format is QuickPython's serialized bytecode format:
- Magic: `QPY\0`
- Version: u32 (currently 5)
- Instructions: serialized bytecode

Current limitations:
- Cannot serialize: functions, lists, dicts, exceptions, imports, try-except
- Supports: arithmetic, variables, control flow (if/while/for), strings, floats, bools, print, range

These limitations are acceptable — `.pyq` is useful for precompiling simple scripts, config evaluation, and caching hot paths.

## Implementation

### 1. Export compiler and serializer from lib

In `src/main.rs`, add public re-exports:

```rust
pub use compiler::Compiler;
pub use serializer::{serialize_bytecode, deserialize_bytecode};
```

Also export the `ByteCode` type alias and `Instruction` enum:

```rust
pub use bytecode::{ByteCode, Instruction};
```

### 2. Add `Context::eval_bytecode` method

Add a method to `Context` that executes pre-compiled bytecode directly:

```rust
impl Context {
    pub fn eval_bytecode(&mut self, bytecode: &ByteCode) -> Result<Value, String> {
        self.vm
            .execute(bytecode, &mut self.globals)
            .map_err(|e| format!("{:?}", e))
    }
}
```

### 3. Create example: `examples/pyq_embed.rs`

A Rust example demonstrating the full workflow:

```rust
use quickpython::{Compiler, Context, serialize_bytecode, deserialize_bytecode};

fn main() {
    // Step 1: Compile Python source to bytecode
    let source = r#"
x = 10 + 20
y = x * 2
print(y)
"#;
    let bytecode = Compiler::compile(source).unwrap();
    println!("Compiled {} instructions", bytecode.len());

    // Step 2: Serialize to bytes (could write to file / embed in binary)
    let bytes = serialize_bytecode(&bytecode).unwrap();
    println!("Serialized to {} bytes", bytes.len());

    // Step 3: Deserialize (could read from file / bundled resource)
    let restored = deserialize_bytecode(&bytes).unwrap();

    // Step 4: Execute
    let mut ctx = Context::new();
    ctx.eval_bytecode(&restored).unwrap();

    // Step 5: Read results from context
    let y = ctx.get("y");
    println!("y = {:?}", y);
}
```

### 4. Add tests

Add Rust unit tests for the programmatic workflow:

```rust
#[test]
fn test_compile_serialize_execute() {
    let source = "x = 1 + 2";
    let bytecode = Compiler::compile(source).unwrap();
    let bytes = serialize_bytecode(&bytecode).unwrap();
    let restored = deserialize_bytecode(&bytes).unwrap();

    let mut ctx = Context::new();
    ctx.eval_bytecode(&restored).unwrap();
    assert_eq!(ctx.get("x"), Some(Value::Int(3)));
}

#[test]
fn test_serialize_error_on_unsupported() {
    let source = "def foo(): return 1";
    let bytecode = Compiler::compile(source).unwrap();
    assert!(serialize_bytecode(&bytecode).is_err());
}
```

## Use Cases

1. **Precompilation** — compile Python scripts at build time, ship `.pyq` bytes in the binary, skip parsing at runtime
2. **Caching** — compile once, serialize to disk, load on subsequent runs
3. **Embedding** — include `.pyq` bytecode as `include_bytes!()` in Rust, execute without shipping `.py` source
4. **Validation** — compile to check syntax without executing

## Acceptance Criteria

- [ ] `Compiler`, `ByteCode`, `Instruction` exported from lib
- [ ] `serialize_bytecode` and `deserialize_bytecode` exported from lib
- [ ] `Context::eval_bytecode()` method added
- [ ] Rust example `examples/pyq_embed.rs` runnable via `cargo run --example pyq_embed`
- [ ] Unit tests for compile → serialize → deserialize → execute round-trip
- [ ] Unit test for serialization error on unsupported instructions
- [ ] Existing CLI behavior unchanged
- [ ] All existing tests still pass
