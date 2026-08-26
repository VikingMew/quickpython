# QuickPython architecture

> **Status: current behavior.** This document describes the checked-in code.
> The final section names non-current design directions explicitly.

## Execution pipeline

```text
Python source
    │
    ▼
rustpython_parser AST
    │
    ▼
Compiler ──► Vec<Instruction>
    │
    ▼
stack VM ──► Value or exception
```

[`Context::eval`](../src/context.rs) compiles source with
[`Compiler`](../src/compiler.rs), then executes it with a persistent
[`VM`](../src/vm.rs) and globals map. `Context::eval_bytecode` executes an
existing instruction vector. Both APIs are synchronous.

## Compiler and bytecode

The compiler consumes `rustpython_parser` AST nodes and emits the supported
subset of instructions defined in [`src/bytecode.rs`](../src/bytecode.rs).
Bytecode is represented as:

```rust
pub type ByteCode = Vec<Instruction>;
```

`Instruction` is a normal Rust enum containing data such as `String`, `usize`,
and `f64`. This representation is convenient for the interpreter but is not a
compact byte array, CPython bytecode, or a stable external ABI. Compiler gaps
are documented in [`compiler.md`](../docs/limitations/compiler.md).

## Runtime values and memory management

[`Value`](../src/value.rs) is a Rust enum with direct scalar variants and owned
or reference-counted compound variants. In particular:

- integers are `i32`, floats are `f64`, and strings are owned `String` values;
- lists use `Rc<RefCell<ListValue>>`;
- dictionaries use `Rc<RefCell<HashMap<DictKey, Value>>>` and accept only
  string/integer keys;
- tuples use `Rc<Vec<Value>>`;
- modules, iterators, and generators use reference-counted state;
- functions, coroutines, exceptions, native functions, regex/match objects,
  and a small set of type objects have dedicated enum variants.

Rust ownership drops unshared values, while `Rc` reference counts shared values.
There is no NaN boxing, tagged-pointer representation, interned-string pool,
cycle detector, or tracing garbage collector. An `Rc` cycle can therefore
remain allocated.

## VM and object model

The VM interprets the instruction vector using an operand stack, call frames,
globals, exception handlers, and loaded modules. It implements selected built-in
operations and methods directly with matches on `Value` variants.

This is not a slot-based user object system. User-defined `class` statements,
instances, descriptors, metaclasses, `__dict__`, and fixed attribute slots are
unsupported. `Module`, `Match`, `Regex`, `BoundMethod`, and `Type` are special
runtime variants, not a general Python object model. See the
[runtime limitations](../docs/limitations/runtime.md).

## Imports and extensions

Imports resolve either the fixed built-in modules (`json`, `os`, `re`, and
`asyncio`) or modules registered on the VM. Rust extensions build a `Module`
whose attributes may contain `NativeFunction` values. A native function is a
non-capturing function pointer:

```rust
fn(Vec<Value>) -> Result<Value, Value>
```

There is no filesystem package loader, pip compatibility, CPython C ABI, native
class API, keyword binding layer, or async native-callback API. The example in
`quickpython-llm` uses synchronous/blocking HTTP calls.

## Async execution

The compiler marks async functions and emits `Await`. Calling an async function
creates a `Value::Coroutine`; handling `Await` executes that coroutine through
the VM. Awaited coroutines run one after another. `asyncio.sleep` produces a
special `AsyncSleep` value, and the VM creates a Tokio runtime and calls
`block_on` for the timer.

QuickPython therefore supports async syntax but does not currently provide a
native scheduler, concurrent tasks, an event-loop API, cancellation, or direct
interoperation with a host Rust future. See
[`asyncio.md`](../docs/limitations/asyncio.md).

## Serialization

[`src/serializer.rs`](../src/serializer.rs) writes a private format containing
`QPY\0`, a version number, an instruction count, and encoded instructions. Only
a subset of `Instruction` has serializer/deserializer support. Unsupported
families return an error, so many source programs that execute correctly cannot
be compiled to `.pyq`. The exact coverage is maintained in
[`serialization.md`](../docs/limitations/serialization.md).

## Non-current design directions

The following ideas appear in older specs or research notes but are not part of
the current architecture: NaN boxing/tagged pointers, interned atom strings,
compact variable-length bytecode, a constant/string/function table file format,
slot-based user objects/classes, cycle detection, sub-millisecond or size
guarantees, and a native concurrent async scheduler. Those documents are kept as
proposals or external design references and are labeled accordingly.
