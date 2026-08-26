# QuickPython compatibility limitations

This registry is the maintained compatibility contract between the checked-in
QuickPython implementation and CPython. QuickPython intentionally implements a
Python-like subset; behavior not documented as supported here must not be
assumed to match CPython.

The registry describes the current `master`-bound implementation, not the
aspirational designs under `spec/`. Changes to user-visible implementation
areas are paired with the mapped page by `scripts/check_docs_parity.py`.

| Area | Contract |
| --- | --- |
| Compiler and bytecode | [compiler.md](compiler.md) |
| VM, values, and objects | [runtime.md](runtime.md) |
| `json` module | [json.md](json.md) |
| `os` module | [os.md](os.md) |
| `re` module | [re.md](re.md) |
| `asyncio` and `await` | [asyncio.md](asyncio.md) |
| Rust extension API | [extensions.md](extensions.md) |
| `.pyq` serialization | [serialization.md](serialization.md) |

## Maintaining the contract

Run the documentation parity check from the repository root:

```bash
python scripts/check_docs_parity.py --base origin/master
```

If a protected implementation path changes, update every limitation page named
by the failure. The mapping is deliberately explicit in the script. A docs-only
change is valid and does not require an implementation change.

Ignored compatibility tests are tracked separately in
[`docs/ignored-tests.json`](../ignored-tests.json). Run
`python scripts/check_ignored_tests.py` to compare that registry with the tests
collected by Cargo.
