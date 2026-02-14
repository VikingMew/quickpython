# Task 021: GitHub Actions CI

## Status
- **Status**: DONE
- **Priority**: P1
- **Estimated effort**: Small

## Goal

Set up GitHub Actions CI pipeline for automated testing on every push and PR to master.

## CI Jobs

### 1. Check (`cargo check --workspace`)
- Validates that all crates in the workspace compile without errors.
- Catches type errors, missing imports, and dependency issues.

### 2. Test (`cargo test --workspace`)
- Runs all 108 Rust unit tests in `src/main.rs`.
- Covers: arithmetic, variables, functions, control flow, strings, floats, lists, dicts, for loops, break/continue, exceptions, try-finally, iterator safety, comparison types, json module, os module, re module.

### 3. Python Integration Tests — Core
- Builds the `quickpython` binary.
- Runs `test/test_extension_registration.py` — verifies that importing an unregistered extension module raises RuntimeError.
- Runs all 15 example scripts in `examples/` — verifies end-to-end execution of real Python programs through the compiler and VM.

### 4. Python Integration Tests — Demo + LLM Extension
- Builds the `quickpython-demo` binary (includes LLM extension).
- Runs `quickpython-demo/test/test_llm_basic.py` — tests:
  - LLM module import succeeds after registration
  - `llm.configure()` with complete dict config
  - `llm.configure()` with missing fields (endpoint, api_key, model) raises KeyError
  - `llm.configure()` with non-dict argument raises TypeError
  - `llm.configure()` with wrong field type raises TypeError
  - `llm.chat()` with no arguments raises TypeError
  - `llm.chat()` with non-list argument raises TypeError
  - `llm.chat()` with empty list raises ValueError
  - `llm.chat()` with non-dict message raises TypeError
  - `llm.chat()` with message missing role/content raises KeyError
- Runs `quickpython-demo/test/test_llm_not_configured.py` — verifies that calling `llm.chat()` before `llm.configure()` raises RuntimeError.

### 5. Clippy (`cargo clippy --workspace -- -D warnings`)
- Runs the Rust linter on all workspace crates.
- Treats all warnings as errors to enforce code quality.

### 6. Format (`cargo fmt --all -- --check`)
- Verifies that all Rust code is formatted according to `rustfmt` standards.
- Fails if any file is not properly formatted.

## Test Coverage Summary

| Layer | What | Count |
|---|---|---|
| Rust unit tests | Core language features via `Context::eval()` | 108 tests |
| Python integration (core) | Extension registration + all examples | 1 + 15 files |
| Python integration (demo) | LLM extension module API validation | 2 files |
| Static analysis | Clippy lints, rustfmt check | workspace-wide |

## Not Covered (Future)

- Performance benchmarks
- Cross-platform testing (macOS, Windows)
- Bytecode serialization round-trip tests
- Network-dependent LLM API tests (require mock server)
