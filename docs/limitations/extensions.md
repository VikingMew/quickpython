# Rust extension limitations

## Supported surface

Rust code can create a `Module`, add named native functions, and register the
module on a `Context` before evaluation. A native function has the fixed type
`fn(Vec<Value>) -> Result<Value, Value>`, so it receives positional VM values
and returns either a VM value or exception value. Registered modules can be
imported by evaluated code. `quickpython-llm` is the maintained example.

## Known CPython divergences

- This is a QuickPython-specific Rust API, not CPython's C ABI, limited API, or
  Python packaging/import system.
- Native functions are plain function pointers, not capturing closures. They
  perform their own argument validation and conversion.
- Extension calls are synchronous. The LLM example uses a blocking HTTP client
  and process-global mutex-protected configuration.
- `Context::get` returns `Option<Value>` and `Context::set` directly stores a
  value; there is no automatic typed conversion layer promised by older specs.

## Unsupported behavior

Native classes/types, per-interpreter extension state, keyword-argument binding,
async native callbacks, unload/reload hooks, package discovery, shared-library
loading, pip packages, and CPython binary extensions are unsupported.

## Implementation and tests

- [`src/context.rs`](../../src/context.rs)
- Module/native-function definitions in [`src/value.rs`](../../src/value.rs)
- Public exports and CLI in [`src/main.rs`](../../src/main.rs)
- [`quickpython-llm/src/lib.rs`](../../quickpython-llm/src/lib.rs)
- [`test/test_extension_registration.py`](../../test/test_extension_registration.py)
