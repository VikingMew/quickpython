# `json` module limitations

## Supported surface

`json.loads(string)` converts JSON nulls, booleans, numbers, strings, arrays,
and objects into QuickPython values. `json.dumps(value)` supports `None`,
booleans, `i32` integers, finite floats, strings, lists, and dictionaries,
including recursive combinations of those values.

## Known CPython divergences

- Only the first positional argument is used; CPython's keyword options and
  formatting/customization controls are not implemented.
- JSON integers are stored as `i32`, so values outside that range do not have
  CPython's arbitrary-precision behavior.
- Integer dictionary keys are converted to decimal strings during dumping.
- Errors use QuickPython's small exception model and do not expose CPython's
  `JSONDecodeError` fields.

## Unsupported behavior

Custom encoders/decoders, hooks, `default`, `object_hook`, `parse_*` callbacks,
streaming `load`/`dump`, formatting options, circular-reference diagnostics,
and serialization of tuples or extension values are unsupported.

## Implementation and tests

- [`src/builtins/json.rs`](../../src/builtins/json.rs)
- JSON unit tests in [`src/main.rs`](../../src/main.rs)
- Escape-sequence coverage in [`src/tests_pending.rs`](../../src/tests_pending.rs)
