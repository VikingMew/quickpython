# Async and `asyncio` limitations

## Supported surface

The compiler recognizes `async def` and `await`. Calling an async function
creates a coroutine value, and awaiting it executes that function. The built-in
`asyncio` module contains only `sleep(seconds)`, accepting non-negative integer
or floating-point seconds.

## Known CPython divergences

- Coroutine execution is sequential. The VM recursively executes a coroutine
  when it reaches `Await`; it does not schedule independent tasks.
- Awaiting `asyncio.sleep` creates a Tokio runtime and blocks on the timer. The
  public `Context::eval` API itself is synchronous.
- Coroutine objects implement only VM-internal behavior and do not provide
  CPython's coroutine protocol, cancellation, or warning behavior.

## Unsupported behavior

There is no event-loop API, task/future API, `gather`, `create_task`, queues,
locks, subprocess support, cancellation, timeouts, async iterators/context
managers, or Rust async extension-function registration.

## Implementation and tests

- Await handling in [`src/vm.rs`](../../src/vm.rs)
- Compilation in [`src/compiler.rs`](../../src/compiler.rs)
- [`src/builtins/asyncio.rs`](../../src/builtins/asyncio.rs)
- Async tests and examples in [`src/main.rs`](../../src/main.rs) and
  [`examples/async_sleep.py`](../../examples/async_sleep.py)
