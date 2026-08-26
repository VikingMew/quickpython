# Bytecode serialization limitations

## Supported surface

`serialize_bytecode` and `deserialize_bytecode` use a private `.pyq` format with
the four-byte magic `QPY\0`, a little-endian version (`5`), an instruction
count, and an instruction-by-instruction payload. Round trips cover scalar
pushes, `Pop`, arithmetic and ordering/equality comparisons, global/local
loads and stores, `Jump`, `JumpIfFalse`, scalar conversions, `Len`, `Range`,
`GetIter`, and `ForIter`.

## Known CPython divergences

- `.pyq` is neither `.pyc` nor stable CPython bytecode and has no compatibility
  promise across QuickPython releases.
- The file stores the enum instruction stream directly; there is no constant
  pool, string intern table, function table, debug table, or compact
  variable-width bytecode representation.
- The deserializer accepts format versions up to the current version, ignores
  trailing bytes, and provides only structural string errors rather than a
  hardened untrusted-input format.

## Unsupported behavior

Serialization rejects containment/identity, logical short-circuiting,
functions/calls/returns, print, collections and indexing, break/continue,
exceptions, imports/attributes, slices, type objects/`isinstance`, `await`, and
`yield`. Consequently, `quickpython compile` fails for many programs that run
successfully from source.

## Implementation and tests

- [`src/serializer.rs`](../../src/serializer.rs)
- Instruction definitions in [`src/bytecode.rs`](../../src/bytecode.rs)
- Serializer tests in [`src/main.rs`](../../src/main.rs) and
  [`src/serializer.rs`](../../src/serializer.rs)
