# Compiler and bytecode limitations

## Supported surface

QuickPython parses source with `rustpython_parser` and compiles the supported
AST subset into a stack-machine `ByteCode`, which is a `Vec<Instruction>`.
The implemented subset includes scalar and collection literals, arithmetic and
comparison operations, assignments, functions, conditionals, loops, imports of
registered modules, exceptions, comprehensions, f-strings, slicing, identity
operations, async functions/`await`, and generator functions. The regular test
suite is the authority for the precise combinations that work.

## Known CPython divergences

- Integer constants are narrowed to `i32`; CPython integers have arbitrary
  precision.
- The instruction stream is an in-memory Rust enum vector. It is not CPython
  bytecode and is not the compact variable-width encoding described by older
  design documents.
- Name and local-scope handling is intentionally smaller than CPython's symbol
  table and closure model.
- Compile failures are returned as strings and do not reproduce CPython's full
  `SyntaxError` metadata.

## Unsupported behavior

Classes, lambdas, sets, generator expressions, decorators/variadic calls,
starred unpacking, `global`, `nonlocal`, `with`, `assert`, `del`, conditional
expressions, assignment expressions, and nested list comprehensions are among
the known gaps. Some generator-function instruction combinations also remain
incomplete. See the ignored-test registry for executable examples and current
rationales.

## Implementation and tests

- [`src/compiler.rs`](../../src/compiler.rs)
- [`src/bytecode.rs`](../../src/bytecode.rs)
- [`src/tests_pending.rs`](../../src/tests_pending.rs)
- [`docs/ignored-tests.json`](../ignored-tests.json)
