# Runtime limitations

## Supported surface

The stack VM executes `Vec<Instruction>` bytecode with global variables and
function-local frames. Runtime values are variants of the Rust `Value` enum:
`i32`, `f64`, booleans, `None`, owned strings, lists, dictionaries, tuples,
slices, iterators, functions, coroutines, generators, exceptions, modules,
native functions, bound methods, regex/match objects, and type objects. Lists,
dictionaries, iterators, generators, and modules use `Rc<RefCell<...>>` where
shared mutable state is required.

## Known CPython divergences

- Values are not NaN-boxed or tagged pointers, and strings are not interned.
- Shared containers use non-thread-safe reference counting and interior
  mutability. Cycles are not detected or collected by a Python-style GC.
- Integers are `i32`, dictionary keys are limited to strings and integers, and
  the type/exception hierarchies are small fixed enums.
- Identity follows the VM's value/container representation and is not a promise
  to reproduce CPython interning or singleton implementation details.
- Error messages, traceback detail, method coverage, and argument validation are
  a subset of CPython behavior.

## Unsupported behavior

User-defined classes and instances, dynamic attributes, descriptors,
metaclasses, a general object protocol, weak references, cycle collection,
threads, and the Python/C extension ABI are unsupported. Many built-ins and
collection/string methods are absent; remaining examples are tracked in the
ignored-test registry.

## Implementation and tests

- [`src/vm.rs`](../../src/vm.rs)
- [`src/value.rs`](../../src/value.rs)
- Unit tests in [`src/main.rs`](../../src/main.rs)
- [`src/tests_pending.rs`](../../src/tests_pending.rs)
