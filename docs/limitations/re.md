# `re` module limitations

## Supported surface

The module provides `match`, `search`, `findall`, `sub`, `subn`, `split`, and
`compile`. Direct `match`/`search` results expose the VM's supported match
methods (`group`, `start`, and `end`). Patterns are compiled by Rust's `regex`
crate.

## Known CPython divergences

- Pattern syntax and matching semantics are those of Rust `regex`, not
  CPython's `re` engine.
- `findall` always returns whole-match strings rather than changing its result
  shape for capture groups.
- `subn` returns a QuickPython list instead of a tuple.
- Extra `count` arguments to `sub`, `subn`, and `split` are not honored.
- Argument validation and error types/messages are narrower than CPython's.

## Unsupported behavior

Flags, callable replacements, `fullmatch`, `finditer`, `escape`, cache controls,
scanner APIs, and compiled-pattern methods are unsupported. `re.compile`
returns an internal regex value, but that value does not expose `match`,
`search`, or `findall` methods.

## Implementation and tests

- [`src/builtins/re.rs`](../../src/builtins/re.rs)
- Match-object behavior in [`src/vm.rs`](../../src/vm.rs)
- Regex tests in [`src/main.rs`](../../src/main.rs)
- Known gaps in [`src/tests_pending.rs`](../../src/tests_pending.rs)
