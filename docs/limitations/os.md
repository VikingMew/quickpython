# `os` module limitations

## Supported surface

The built-in module exposes `listdir`, `mkdir`, `makedirs`, `remove`, `rmdir`,
`rename`, `getcwd`, `chdir`, and `getenv`, plus a snapshot dictionary in
`os.environ` and `os.name`. `os.path` exposes `exists`, `isfile`, `isdir`,
`join`, `basename`, `dirname`, and `abspath` for string paths.

## Known CPython divergences

- Calls operate directly on the host process and filesystem; QuickPython is not
  a sandbox or capability-based runtime.
- `os.environ` is copied when the module is created. Mutating that dictionary
  does not update the process environment.
- Functions accept a narrower set of positional string arguments. File
  descriptors, path-like objects, bytes paths, keyword options, platform flags,
  and much of CPython's detailed exception mapping are absent.
- Results follow Rust standard-library path conversion and directory iteration,
  including platform-dependent ordering and Unicode-lossy conversion.

## Unsupported behavior

The rest of CPython's `os`/`os.path` surface is unsupported, including process
management, permissions, links, stat APIs, descriptor operations, environment
mutation, and `open`. Python `with`-based file I/O is also unavailable.

## Implementation and tests

- [`src/builtins/os.rs`](../../src/builtins/os.rs)
- OS tests in [`src/main.rs`](../../src/main.rs)
- File-I/O gaps in [`src/tests_pending.rs`](../../src/tests_pending.rs)
