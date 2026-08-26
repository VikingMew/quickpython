# QuickPython benchmark suite design

## Status and intent

This document is the implementation contract for a QuickPython interpreter benchmark suite. It is a design only: this change does not add a benchmark target, benchmark dependencies, a workflow, performance optimizations, or a generator fix.

“对标 Monty” means structural and measurement parity where the runtimes overlap:

- a comparable runner shape and stable workload names;
- separate feature, compile, precompiled-execution, embedded end-to-end, and context-startup measurements;
- the same deterministic Python source, input, and expected result for each QuickPython/CPython pair;
- local QuickPython/CPython ratios and CI history for QuickPython regressions; and
- an explicit adapted, QuickPython-only, deferred, or excluded label where language or runtime architecture differs.

It does not mean that QuickPython must be faster than Monty or CPython. It also does not imply an absolute speed threshold or a QuickPython version of Monty's worker pool, wire protocol, allocator ceiling, or resource-limit paths.

## Pinned reference and repository baseline

The structural reference is [`pydantic/monty@2a6fd3044769a9fafbc0424af23e9d0097ba84b6/crates/monty-bench`](https://github.com/pydantic/monty/tree/2a6fd3044769a9fafbc0424af23e9d0097ba84b6/crates/monty-bench), not an unversioned view of Monty's default branch. At that revision:

- `monty-bench` has `main`, `pool`, and `decode` targets;
- `main.rs` has 47 `bench_function` registrations, mostly `__monty`/`__cpython` pairs plus Monty-only end-to-end, parse, limits, and allocator variants;
- local measurements use Criterion 0.5, optional Unix pprof flamegraphs, and PyO3;
- CodSpeed switches to `codspeed-criterion-compat` and builds a release-derived `codspeed` profile with symbols; and
- JSON load/dump and regex extraction are Monty parity workloads, not QuickPython inventions.

The QuickPython repository is materially different. It has a flat root package and two sibling workspace packages, no `crates/` convention, no benchmark target or profile, and no Criterion, CodSpeed, pprof, or PyO3 dependency. The public `Compiler::compile`, `Context::new`, `Context::eval`, and `Context::eval_bytecode` APIs are sufficient; the benchmark must not expose the private VM. Existing CI runs workspace check, test, clippy, and formatting, but its path filters ignore Markdown/spec-only changes. The later implementation must therefore add explicit benchmark validation; this design-only change is not expected to trigger current CI.

## Decisions at a glance

| Area | Decision |
| --- | --- |
| Layout | Add a non-published `quickpython-bench/` sibling workspace package. Do not introduce `crates/quickpython-bench`. |
| Runner | One `main` benchmark source, using cfg-gated Criterion/CodSpeed imports. |
| Runtime dimensions | `startup`, `compile`, `execute`, and `end_to_end` are distinct groups with fixed boundaries. |
| Correctness | A shared workload registry and shared source files feed non-benchmark correctness tests and the runner. A workload cannot be registered until the correctness tests pass strictly. |
| CPython | Use optional, local-only PyO3 with a pinned standard GIL-enabled CPython 3.14.7. CPython is not built or run by CodSpeed. |
| Regression history | CodSpeed on pull requests and `master` is the longitudinal source of truth for QuickPython cases. |
| Local comparison | A report command reads Criterion medians and emits `QuickPython median / CPython median` only for equivalent pairs. |
| Exclusions | No pool, decode, worker allocator, allocator ceiling, resource limits, GC comparison, or performance optimization. |

## Package and file layout

### Options considered

| Concern | Root `benches/` on `quickpython` | Non-published workspace package |
| --- | --- | --- |
| Public API access | Direct package access. | A normal `quickpython = { path = ".." }` dependency; still only public APIs. |
| Dependency isolation | Criterion/CodSpeed become root dev dependencies. PyO3 as a dev dependency is compiled by normal test builds; making it optional instead pollutes the published package's feature/dependency surface. | Runner, pprof, and PyO3 dependencies and features remain in `publish = false` package metadata. Normal QuickPython consumers do not see them. |
| Workspace CI | Root dev dependencies increase ordinary `cargo test --workspace` work and can accidentally require a Python toolchain. | A small default correctness target participates in workspace CI, while runner and PyO3 dependencies are optional and disabled by default. Bench targets still need explicit check/clippy/smoke commands. |
| Commands | Shorter: `cargo bench --bench main`. | Explicit: `cargo bench -p quickpython-bench --bench main --features runner`. |
| Future targets | Possible, but mixes benchmark-only structure into the published package. | Additional `[[bench]]` targets and helpers stay isolated. |
| Repository convention | Uses Cargo's root convention. | A `crates/` directory would be a new convention. A sibling `quickpython-bench/` matches `quickpython-demo/` and `quickpython-llm/`. |

### Recommendation

Create `quickpython-bench/` as a sibling workspace member with `publish = false`; do not create a new `crates/` hierarchy. The slightly longer commands are worth the dependency and publication isolation, and the sibling layout follows the repository's current convention.

The implementation layout is fixed as follows:

```text
quickpython-bench/
  Cargo.toml
  build.rs                         # no-op unless the cpython feature is enabled
  src/
    lib.rs                         # workload registry, Expected value, wrapper helpers
    bin/benchmark-report.rs        # Criterion median/ratio and baseline metadata reporter
  fixtures/
    *.py                           # exact shared Python sources
    json_medium.json               # deterministic external input
  benches/
    main.rs                        # the only initial benchmark target
  tests/
    quickpython_workloads.rs       # always-on strict correctness tests
    cpython_workloads.rs           # required-features = ["cpython"]
spec/
  benchmark-baselines.md           # created with the initial measured baseline
```

`quickpython-bench/src/lib.rs` must contain no Criterion or PyO3 code under default features. Its workload registry owns the stable ID, category, dimensions, source loaded with `include_str!`, setup kind, initial size, expected value, CPython-comparable flag, and CI-enabled flag. Both correctness tests and `benches/main.rs` iterate this same registry. Generated input such as the 1,000-assignment source must come from one shared registry helper, not a second copy in the benchmark.

The package features are:

- `runner`: enables pinned Criterion 0.5 and `codspeed-criterion-compat` runner dependencies;
- `cpython`: implies `runner` and enables matching exact versions of PyO3 and `pyo3-build-config`;
- `pprof`: implies `runner` and enables the optional Unix pprof dependency.

`[[bench]] main` has `harness = false` and `required-features = ["runner"]`. `[[test]] cpython_workloads` has `required-features = ["cpython"]`. Consequently, default `cargo check/test/clippy --workspace` does not build PyO3 or the benchmark runners, but it does compile the small benchmark package library and run `quickpython_workloads`. Add `quickpython-bench/` to the root package's `exclude` list as an explicit publication safeguard.

Use exact dependency pins in the implementation lockfile: Criterion `0.5.1`, `codspeed-criterion-compat` `4.2.1`, pprof `0.15.0`, and matching PyO3/`pyo3-build-config` `0.29.2`. Updating a runner dependency is a benchmark-environment change and requires a new baseline entry.

## Measurement contract

### Stable IDs

Criterion groups encode the dimension, and benchmark IDs encode the workload and runtime:

```text
<dimension>/<workload>__<runtime>
```

Examples are `startup/context__quickpython`, `compile/assignments_1k__cpython`, `execute/fib_recursive__quickpython`, and `end_to_end/kitchen_sink__cpython`. Dimensions are exactly `startup`, `compile`, `execute`, and `end_to_end`; runtime suffixes are exactly `quickpython` and `cpython`.

Names must remain stable after their first default-branch baseline. A source, setup, input-size, expected-result, or timed-boundary change is a benchmark-definition change: add a versioned workload ID such as `_v2` and retire the old ID, or deliberately reset its CodSpeed history and document why. Never silently reuse a name for different work.

### Timed boundaries

| Dimension | QuickPython timed operation | CPython timed operation | Pairing policy |
| --- | --- | --- | --- |
| Context startup | Construct `Context::new()`. Use Criterion's large-drop form so destruction is outside the sample. | None. Creating a Python dictionary is too small a boundary, while starting a process or subinterpreter is a different lifecycle. | QuickPython-only; no ratio. |
| Compile only | `Compiler::compile(black_box(source))`; retain/black-box the `ByteCode` and drop it after timing. | In an already initialized, GIL-held interpreter, call built-in `compile(source, "fixture.py", "exec")`; retain/black-box the code object and drop it after timing. | Pair `add_two`, `assignments_1k`, and `kitchen_sink`. Interpreter/process startup and fixture loading are excluded for both. |
| Precompiled execution | Compile once. For every measured execution, Criterion `iter_batched_ref` with `BatchSize::PerIteration` supplies a fresh prepared `Context`; only `Context::eval_bytecode` and common integer-result extraction are timed. | Build the wrapped module and function once, outside timing, while holding the GIL outside the iteration closure. Each timed `call0`/`call1` creates fresh function locals; common integer-result extraction is timed. | Pair every equivalent/adapted workload. Do not pair QuickPython-specific cases. |
| Embedded end to end | Within timing, construct `Context`, call `Context::eval(source)`, and extract the integer result. Return context/result to Criterion so teardown is outside timing. | CPython itself is initialized before timing. Within timing, compile and execute the already-built wrapper string as a fresh module, obtain `main`, call it, and extract the integer result. Return Python objects so teardown is outside timing. | Pair `add_two` and `kitchen_sink`. This measures an embedded invocation, never operating-system process startup. |

Criterion's `iter_with_large_drop` or equivalent must keep destruction of contexts, bytecode, Python modules, and result objects out of startup, compile, and end-to-end samples. This prevents destructor cost from drifting into only one runtime's boundary.

### Setup, reset, and result rules

The following rules are mandatory:

1. Load source files, JSON/text fixtures, expected values, wrapper strings, and generated sources before entering the timed loop.
2. Compile outside the timed loop except in `compile` and `end_to_end` groups.
3. Run each fixture once through its non-benchmark correctness test before it can appear in `main.rs`. The test must fail on any error; accepted-error tests are not sufficient.
4. In the runner, repeat expected-result validation once immediately before timing for each runtime and compare normalized values. Do not assert on every timed iteration.
5. Every timed value or compiled artifact is passed to `black_box`. Both execution runners convert the returned integer to `i64` before `black_box`, so the embedding-boundary conversion is represented on both sides.
6. QuickPython execution iterations never reuse mutable globals. The excluded setup creates a new `Context` per iteration, injects immutable external input with `Context::set`, and preloads a required built-in module with a setup `Context::eval("import ...")`. `iter_batched_ref(..., BatchSize::PerIteration)` ensures setup and teardown are excluded and state cannot cross iterations.
7. CPython execution source is wrapped in a function. Function locals are fresh per call; the process, interpreter, GIL attachment, compiled function, imported modules, and immutable argument objects are reused outside timing. Fixtures may not mutate Python globals or process state.
8. JSON, regex, and asyncio imports happen in each runtime's excluded setup. Their timed bodies exercise the built-in operation, not asymmetric import-cache behavior. External payload construction and file I/O are also excluded.
9. End-to-end fixtures have no external input or preload. Otherwise host setup would either enter only one timed boundary or make `Context::eval` unable to run from a new context.
10. No benchmark prints, reads the clock, uses randomness, accesses the network/filesystem, or depends on hash iteration order for its result.

The CPython process and GIL are intentionally already active. Ratios therefore describe embedded, warm-process execution boundaries, not CLI startup.

### Sampling, warm-up, and calibration

Local Criterion defaults are fixed in code to a 3-second warm-up, 10-second measurement, 100 samples, 5% significance level, and 3% noise threshold. Use the same configuration on all local baseline runs. CodSpeed's compatibility runner may control sampling internally, but it must register the same names and bodies.

Before the first baseline, calibrate scalable loop/input counts on an otherwise idle release-equivalent machine:

- target a median of 100 microseconds through 10 milliseconds per body;
- double the input if the median is below 100 microseconds and halve it if above 10 milliseconds, then rerun correctness and three timing trials;
- do not change algorithm, source features, setup, or expected-result derivation during calibration;
- atomic operations such as context startup and `add_two` remain semantically atomic even if below the target; Criterion's iteration batching supplies statistical weight; and
- freeze the calibrated size with the source and expected value. Later size changes follow the version/reset rule above.

For comparable local results, use an idle machine, a stable power mode/CPU governor, the same toolchain and CPython build, and at least two runs. A pprof-enabled run is for diagnosis only and must not supply timing or baseline numbers.

## Workloads and correctness fixtures

### Registry statuses

- **Equivalent**: the QuickPython and CPython pair runs the exact same timed Python body and expected result, and it represents the same Monty workload shape. Scaling alone is recorded but does not make a workload QuickPython-specific.
- **Adapted**: the QuickPython and CPython pair still uses the exact same body/result, but Monty's source was rewritten because QuickPython does not support one of its semantics. An adapted pair gets a QuickPython/CPython ratio but must not be compared numerically with Monty's original result.
- **QuickPython-only**: there is no honest CPython lifecycle or semantic pair. No ratio is emitted.
- **Deferred**: no benchmark registration exists until the named strict correctness prerequisite passes.

Current implementation constraints are part of the fixture definitions. Keyword arguments, lambdas, classes, applied decorators, `sorted`, `sum`, generator expressions, floor division, the rich format mini-language, and rich container `repr` are not assumed. Conditional expressions and tuple subscripting are also avoided by the initial fixtures; statement `if`/`else` and tuple unpacking have strict current coverage.

### Initial matrix

`fixtures/<name>.py` below means `quickpython-bench/fixtures/<name>.py`; `QP test` means `quickpython-bench/tests/quickpython_workloads.rs`, and `CP test` means the feature-gated `quickpython-bench/tests/cpython_workloads.rs`. Both tests load the registry's exact source rather than embedding another source string.

| Workload ID | Category and dimension | Monty relation/status | Exact capability or rewrite rationale | Initial size and deterministic expected result | CPython comparison | Correctness fixture/reuse |
| --- | --- | --- | --- | --- | --- | --- |
| `context` | startup / `startup` | QuickPython-specific | Measures only public `Context::new`; no honest CPython context lifecycle exists. | One construction; expected empty globals (`get("__sentinel__") == None`) and no error. Fixed atomic size. | No; ratio is `—`. | Registry setup case plus QP test `context_starts_empty`; no `.py` source. |
| `add_two` | micro / `compile`, `execute`, `end_to_end` | Equivalent to Monty `ADD_TWO` | Exact body is `1 + 2`. It anchors small-source compilation and embedded overhead as well as execution. | One addition; expected `3`. Fixed atomic size. | Yes in all three dimensions. | `fixtures/add_two.py`; QP test and CP test. |
| `modulo_loop` | micro / `execute` | Equivalent to Monty `LOOP_MOD_13`, with integer count instead of string growth | Count `i` values divisible by 13 in `range(1000)`; modulo, loop, branch, and augmented assignment are supported. | `N=1,000`; expected `77`. | Yes. | `fixtures/modulo_loop.py`; QP test and CP test. |
| `fib_recursive` | micro / `execute` | Equivalent to Monty `FIB_25`, initially scaled | Positional recursion and integer comparisons/arithmetic overlap exactly. | `fib(20)`; expected `6,765`. Calibrate `n` before first baseline. | Yes. | `fixtures/fib_recursive.py`; QP test and CP test. |
| `list_append_int` | micro / `execute` | Adapted from Monty `LIST_APPEND_INT` | Build the same integer list but return `len(a)`, because Monty's final `sum(a)` is unsupported and would add unrelated semantics. | `N=10,000`; expected `10,000`. | Yes for the adapted exact body. | `fixtures/list_append_int.py`; QP test and CP test. |
| `list_append_string` | micro / `execute` | Equivalent to Monty `LIST_APPEND_STR`, scaled | Exercises `str(i)`, list growth, and append with supported semantics. | `N=10,000`; expected `10,000`. | Yes. | `fixtures/list_append_string.py`; QP test and CP test. |
| `positional_function_calls` | micro / `execute` | Adapted replacement for Monty `FUNC_CALL_KWARGS` | `add(a, b)` is called positionally in a loop. It measures supported call/frame overhead and must not be described as kwargs binding. | `N=10,000`; incrementing accumulator returns `10,000`. | Yes for positional calls; no comparison to Monty's kwargs number. | `fixtures/positional_function_calls.py`; QP test and CP test. |
| `list_comprehension` | syntax/collections / `execute` | Equivalent to Monty `LIST_COMP` | Same supported list-comprehension shape, `[x * 2 for x in range(N)]`. | `N=1,000`; expected length `1,000`. | Yes. | `fixtures/list_comprehension.py`; QP test and CP test. |
| `dict_comprehension` | syntax/collections / `execute` | Adapted from Monty `DICT_COMP` | Use `{i: i * 2 for i in range(N)}`. Monty's `i // 2` is unsupported; preserving it would make the fixture uncompilable. | `N=1,000`; expected length `1,000`. | Yes for the adapted exact body. | `fixtures/dict_comprehension.py`; QP test and CP test. |
| `tuple_construction` | syntax/collections / `execute` | Equivalent to Monty pair-tuple creation, scaled | Construct `(i, i + 1)` tuples in a list. The assertion uses the outer list length, not unsupported tuple `len`/subscript assumptions. | `N=10,000`; expected `10,000`. | Yes. | `fixtures/tuple_construction.py`; QP test and CP test. |
| `basic_representation` | syntax/collections / `execute` | Adapted counterpart to Monty `CONTAINER_REPR` | Repeatedly call `str` on a list of integers, a string-keyed integer dict, and a string. It deliberately excludes `Counter`, deque, set, nested rich repr, and ordering-sensitive output. Only lengths contribute to the result. | 200 list/dict items, 20 repetitions; expected `61,620`. | Yes; correctness proves equal lengths for the chosen values. | `fixtures/basic_representation.py`; QP test and CP test. |
| `kitchen_sink` | end-to-end/features / `compile`, `execute`, `end_to_end` | Adapted from Monty's shared kitchen-sink test | Covers list append/index, dict set/get/keys, tuple construction/unpacking, positional function calls, loops/branches, list/dict comprehensions, and a basic f-string. It replaces Monty's `insert`, dict `pop`, `sum`, tuple indexing, and other unsupported assumptions. | Fixed small feature mix; expected checksum `346`. | Yes for all three dimensions. | `fixtures/kitchen_sink.py`; QP test and CP test. |
| `assignments_1k` | parser/compiler / `compile` | Adapted equivalent of Monty `parse_1k_assigns` | Shared helper generates exactly `"x = 1\n"` 1,000 times followed by `x`. Measures the full public parser/compiler path, not a private parse phase. | 1,000 assignments; correctness execution/namespace value is `1`. | Yes for raw `compile(..., "exec")`; no execution ratio is registered. | Registry helper `assignments_1k()`; QP test and CP test consume the same generated string. |
| `aggregation` | agent-shaped / `execute` | Adapted from Monty `AGG_ROWS` | Build 300 rows and aggregate five times with explicit loops, `dict.get`, `dict.keys`, and comparisons. An explicit max scan and count replace lambda, `sorted`, `sum`, generator expressions, and `//`. | 300 rows × 5 passes; expected checksum `147,710`. | Yes for the adapted exact body. | `fixtures/aggregation.py`; QP test and CP test. |
| `basic_fstring_report` | agent-shaped / `execute` | Adapted from Monty `FSTRING_REPORT` | Build 500 `name:price:index` lines with basic interpolation and `join`. Alignment, precision, percentage, zero-padding, and thousands separators are intentionally absent. | 500 lines; expected `len(report) + total == 82,803`. | Yes for supported basic f-string semantics. | `fixtures/basic_fstring_report.py`; QP test and CP test. |
| `string_parsing` | agent-shaped / `execute` | Equivalent workload shape to Monty `STR_PARSE`, scaled/adapted syntax | Uses `split`, `strip`, `lower`, `startswith`, containment, and `int`. Fixture construction uses statement `if`/`else`, not a currently unsupported conditional expression. | 200 lines × 5 passes; expected `127,020`. | Yes. | `fixtures/string_parsing.py`; QP test and CP test. |
| `json_loads` | shared built-in / `execute` | Shared Monty parity case | Preload `json` and inject the same 133-byte JSON string outside timing; timed body loads it 1,000 times and counts three `items`. | 133-byte payload × 1,000; expected `3,000`. | Yes. Import and input setup are excluded for both. | `fixtures/json_loads.py` + `fixtures/json_medium.json`; QP test and CP test. |
| `json_dumps` | shared built-in / `execute` | Shared Monty parity case | Preload `json`, parse the payload in excluded setup, then dump the same supported object 1,000 times. The result counts non-empty outputs, avoiding serializer whitespace/key-order differences; correctness also round-trips the output outside timing. | Three-item object × 1,000; expected `1,000`. | Yes for JSON serialization semantics; encoded bytes need not be identical. | `fixtures/json_dumps.py` + `fixtures/json_medium.json`; QP test and CP test. |
| `regex_extraction` | shared built-in / `execute` | Shared parity, adapted from Monty `RE_EXTRACT` | Preload `re`; repeat `re.search` with two numeric capture groups and call `group(1/2)`. QuickPython lacks Monty's faithful `Pattern.finditer` shape, so this is an explicit supported extraction rewrite, not QuickPython-only. | One 22-byte text × 1,000 searches; expected `130,000`. | Yes for the exact search/group body. | `fixtures/regex_extraction.py`; QP test and CP test. |
| `await_tokio_bridge_zero_sleep` | QuickPython-specific async / `execute` | QuickPython-specific | Define/await one coroutine containing `await asyncio.sleep(0)`. Today each await creates a Tokio runtime and calls `block_on`; this measures await/Tokio-bridge overhead, not scheduler switching or concurrency. | One zero-duration await; expected `1`. Fixed atomic size. | No; CPython's event-loop lifecycle would measure different architecture. | `fixtures/await_tokio_bridge_zero_sleep.py`; QP test only. |
| `generator_traversal` | generator / deferred `execute` | Deferred | Intended body yields `0..99` and counts traversal. Generator creation alone is insufficient: current strict traversal can reach `RuntimeError: Stack underflow`. | Proposed `N=100`; expected `100`, but no timed registration exists. | Eligible only after the prerequisite; not currently paired. | A new strict test must unconditionally succeed and assert all values. Only then add `fixtures/generator_traversal.py` to both correctness suites and registry. |

The numeric results above were selected to fit QuickPython's current integer representation and must be asserted as integers, not compared through printed output. The initial implementation may calibrate scalable sizes before the first baseline; if it does, source, registry size, expected value, and both correctness tests change together.

### Deferred and intentionally excluded Monty cases

Unsupported semantics must never be silently replaced under Monty's original name.

| Monty shape | Policy and reason |
| --- | --- |
| Keyword/default argument binding | Defer. Positional calls are a separately named adapted workload; they do not measure kwargs binding. |
| Rich built-in argument binding | Defer. Monty's `replace`/`encode`/`sorted(..., reverse=False)` mix depends on richer argument binding and unsupported `sorted`. |
| Full format-mini-language report | Defer. The basic f-string workload is separately named and does not claim alignment, percentage, precision, or thousands-separator coverage. |
| Rich container repr | Defer. `Counter`, deque, set, rich container `repr`, and mutation-safe repr semantics are absent. `basic_representation` is explicitly adapted. |
| Datetime analysis | Defer until a compatible datetime module and strict semantic fixtures exist. |
| GC collection/cycle benchmark | Exclude. QuickPython's current architecture has no comparable tracing-GC contract, so a ratio would be misleading. |
| Classes, dataclasses, and decorator application | Defer. Classes/dataclasses are unsupported and parsed decorators are not applied. |
| Generator-expression/sorted/sum variants | Defer as those semantics. `aggregation` uses explicit supported loops and carries an adapted name. |
| Generator traversal | Defer behind the strict correctness prerequisite above; an accepted `Stack underflow` is a failure for benchmark admission. |
| Pool, wire decode, worker allocator/ceiling, and resource-limit variants | Exclude from this suite. They are Monty architecture benchmarks with no QuickPython counterpart. |

## One runner for Criterion, CodSpeed, and profiling

`benches/main.rs` uses one registration function and cfg-gated imports:

```rust
#[cfg(codspeed)]
use codspeed_criterion_compat::{black_box, criterion_group, criterion_main, Bencher, Criterion};
#[cfg(not(codspeed))]
use criterion::{black_box, criterion_group, criterion_main, Bencher, Criterion};

#[cfg(all(not(codspeed), feature = "cpython"))]
use pyo3::prelude::*;
#[cfg(all(not(codspeed), unix, feature = "pprof"))]
use pprof::criterion::{Output, PProfProfiler};
```

The package lint configuration must declare `cfg(codspeed)` through Cargo's `unexpected_cfgs.check-cfg`, so `-D warnings` remains useful. All QuickPython registrations compile under both runner imports. CPython registration and imports require both `not(codspeed)` and the `cpython` feature, so CodSpeed neither builds nor runs CPython cases in the supported command. Deferred workloads are absent from the registry's enabled set and cannot accidentally register.

At the workspace root, add profiles there (Cargo ignores package-local workspace profiles):

```toml
[profile.codspeed]
inherits = "release"
debug = true
strip = false

[profile.profiling]
inherits = "release"
debug = true
strip = false
```

`codspeed` preserves the repository's real release optimization choices by inheritance and changes only symbol retention. `profiling` does the same for separate diagnostic builds. If the release profile changes later, both follow it automatically. Do not add different LTO/codegen settings only to benchmarks unless they are also the production release settings.

### Exact local commands

QuickPython-only correctness and timing:

```bash
cargo test -p quickpython-bench --test quickpython_workloads
cargo bench -p quickpython-bench --bench main --features runner
```

Pinned local CPython correctness, paired timing, and ratio report:

```bash
uv python install 3.14.7
PYO3_PYTHON="$(uv python find 3.14.7)" cargo test -p quickpython-bench --test cpython_workloads --features cpython
PYO3_PYTHON="$(uv python find 3.14.7)" cargo bench -p quickpython-bench --bench main --features cpython
cargo run -p quickpython-bench --bin benchmark-report -- --criterion-dir target/criterion --format markdown
```

Criterion filters may follow `--`, for example `cargo bench -p quickpython-bench --bench main --features runner -- 'execute/'`. The report reads each `new/estimates.json` median, joins only registry entries marked CPython-comparable, and prints `QuickPython median / CPython median`; values greater than one mean QuickPython was slower for that local lifecycle. It prints `—` for QuickPython-only cases and fails if one side of a declared pair is missing.

Optional Unix flamegraph run:

```bash
cargo bench -p quickpython-bench --bench main --profile profiling --features runner,pprof -- 'execute/aggregation__quickpython'
```

On Unix, the pprof feature installs `PProfProfiler::new(100, Output::Flamegraph(None))`; flamegraph SVGs are emitted below `target/criterion`. On non-Unix, the cfg excludes pprof integration and the same benchmark falls back to ordinary Criterion without a flamegraph. The runner must not fail merely because pprof is unavailable. Do not use pprof-enabled medians for ratios or baselines.

### Exact CodSpeed commands

Pin the CLI because its profile handling affects code generation:

```bash
cargo install cargo-codspeed --version 5.0.1 --locked
cargo codspeed build -p quickpython-bench --bench main --features runner --profile codspeed
cargo codspeed run -p quickpython-bench
```

The package initially has only `main`, so the run command cannot select pool/decode targets. The workflow must use these same build/run commands.

## CPython/PyO3 harness

### Recommendation and version contract

Use PyO3 as an optional benchmark-package dependency. It gives compile-once/call-many control within the same Rust host process and follows Monty's proven harness structure. Shelling out to `python` would mostly measure process startup, while linking PyO3 into the published QuickPython package would couple normal users and CI to CPython.

Pin the standard, GIL-enabled CPython build to [3.14.7](https://www.python.org/downloads/release/python-3147/) in `quickpython-bench/.python-version`, the CPython correctness job, and every committed baseline. Free-threaded builds are a different runtime and are not accepted under this version label. Pin matching PyO3 and `pyo3-build-config` versions to `0.29.2` and enable PyO3's `auto-initialize` feature only through `cpython`.

### `wrap_for_cpython`

Execution and end-to-end comparisons pass fixture bodies through one tested `wrap_for_cpython(source, params)` helper. The fixture contract requires its final top-level statement to be a single-line expression that produces the expected integer.

The helper must:

1. normalize line endings but otherwise preserve source text;
2. retain blank lines and full-line comments in their original order;
3. identify the last non-empty line whose trimmed form does not start with `#`;
4. reject it unless it is at top level and parses as the fixture's final expression;
5. replace only that line with `return <trimmed expression>`, retaining trailing blank/comment lines;
6. indent every retained source line by four spaces beneath `def main(<params>):`; and
7. return an error if there is no final expression, the wrapper contains a NUL byte, or the wrapped module fails to compile.

Expected values and skip metadata live in the Rust registry, not magic Python comments. Therefore comments are never deleted. JSON/regex data is passed as a `DATA` parameter where needed, while pre-imported modules are inserted into the wrapper module globals before the function is called.

Unit tests for the helper must cover leading/interior/trailing blank lines, comment-only lines before and after the result, indentation inside a loop/function, a trailing result expression, missing result, and CRLF input. A line-based transformation is acceptable only with this fixture contract and these validation tests; it is not a general Python source rewriter.

### Lifecycle and validation

For execution pairs, attach to CPython and hold the GIL outside Criterion's iteration closure. Compile the wrapper module once, fetch `main` once, construct immutable arguments once, call it once, extract the result as `i64`, and assert the registry's expected value before timing. The timed loop calls the existing function and extracts the integer but performs no assertion. Function locals make calls independent.

For compile pairs, validate the fixture through the correctness harness first. The timed body calls Python's built-in `compile` on the raw unwrapped module source in `exec` mode. For embedded end-to-end pairs, build the wrapper string outside timing but compile the module, fetch/call `main`, and extract the result inside each sample. Interpreter initialization and GIL attachment remain outside every dimension, matching an embedded warm process.

The QuickPython and CPython correctness tests must assert the same registry `Expected` value for every comparable enabled fixture. `json_dumps` also performs an untimed round-trip invariant. A runtime-specific expected value is forbidden; if results cannot match, the case is relabeled QuickPython-only or deferred rather than assigned a misleading ratio.

### Platform prerequisites and isolation

On Linux, install a C toolchain, `pkg-config`, and a shared-library CPython 3.14.7. On macOS, install Xcode command-line tools and a framework/shared CPython 3.14.7. `uv python install 3.14.7` is the documented cross-platform installation route. Set `PYO3_PYTHON` to `uv python find 3.14.7` before the first PyO3 build.

When `cpython` is enabled, `build.rs` calls `pyo3_build_config::add_libpython_rpath_link_args()` so an installed libpython can be found at runtime. If a Linux distribution build still omits a discoverable shared libpython, set its containing directory in `LD_LIBRARY_PATH`; on macOS use the framework build selected by `PYO3_PYTHON` rather than mutating system Python.

Normal workspace CI does not enable `cpython`, does not compile PyO3/`pyo3-build-config`, and needs no Python headers or libpython. The explicit CPython correctness/smoke job installs the pinned Python and enables the feature. CodSpeed passes only `runner`; the `#[cfg(all(not(codspeed), feature = "cpython"))]` blocks remain absent.

## CI, regression history, and baselines

### CodSpeed workflow

Add `.github/workflows/codspeed.yml` for pull requests, pushes to the repository default branch `master`, and manual dispatch. It needs only:

```yaml
permissions:
  contents: read
  id-token: write
```

`contents: read` checks out source; `id-token: write` permits CodSpeed's OIDC upload. Pin third-party actions to reviewed commit SHAs, pin `cargo-codspeed` to `5.0.1`, use stable Rust, build with the exact `codspeed` command above, and run with `CodSpeedHQ/action` in simulation mode. Set a 20-minute job timeout.

External prerequisites are a CodSpeed project linked to this GitHub repository, authorization for pull-request/default-branch uploads, and a first accepted `master` run to establish history. OIDC is preferred; if the CodSpeed project configuration requires a token instead, the repository must provide the documented `CODSPEED_TOKEN` secret before enabling required checks. That project setup is a dependency of the CI implementation ticket, not something benchmark code can create.

CodSpeed registers every enabled QuickPython row in the initial matrix, but no CPython/deferred/excluded row. PR time remains bounded by one `main` target, roughly twenty QuickPython cases, calibrated bodies no longer than 10 milliseconds, no local Criterion warm-up protocol, no pprof, no PyO3, and the workflow timeout. Adding a future case requires an owner to confirm the total CodSpeed job remains under the timeout; otherwise move a redundant case to a separately scheduled extended target rather than silently dropping initial regression coverage.

CodSpeed is the longitudinal performance source of truth. A plain smoke build proves compilation only:

```bash
cargo bench -p quickpython-bench --bench main --features runner --no-run
PYO3_PYTHON="$(uv python find 3.14.7)" cargo bench -p quickpython-bench --bench main --features cpython --no-run
```

Neither command tracks performance and neither replaces CodSpeed.

### Regression policy

A **regression signal** is a CodSpeed comparison that reports a statistically meaningful slowdown for an unchanged benchmark definition, especially when the direction repeats on rerun or across adjacent default-branch commits. A local Criterion change is supporting diagnostic evidence only; machine-to-machine raw medians are not signals.

During suite bootstrap there is no automatic numeric merge-blocking threshold. A signal is investigated and may be held by a maintainer when a rerun reproduces it and the change is attributable, but CodSpeed status alone is informational. After at least 30 comparable default-branch samples, owners may establish an explicit per-workload relative threshold in a separate reviewed change. Such a threshold must use CodSpeed history/confidence, apply to regressions from the tracked baseline rather than QuickPython/CPython speed, and document false-positive handling. Absolute local times and a target such as “must beat CPython/Monty” can never be a merge gate under this design.

### Committed baseline

The implementation's first stable local run creates `spec/benchmark-baselines.md`. It is a reproducibility snapshot and local ratio record, not longitudinal truth or a universal threshold. Use one row per workload/dimension with this required schema:

| Date (UTC) | Git SHA | Workload ID | OS / architecture | CPU | Rust version | CPython version | Command / profile | QuickPython median | CPython median | QP / CPython ratio |
| --- | --- | --- | --- | --- | --- | --- | --- | ---: | ---: | ---: |
| `YYYY-MM-DD` | full SHA | `execute/example` | exact OS / arch | exact model | full `rustc -Vv` release | `3.14.7` or `—` | exact command; `bench` | duration | duration or `—` | quotient or `—` |

The ratio is `QuickPython median / CPython median`; it appears only where the registry declares equivalence. The report tool must capture or require every metadata field and reject placeholder metadata when appending an official entry.

Maintainers owning `quickpython-bench/` and its fixture tests own this file. Append a new dated block, rather than overwriting history, only for the initial suite, an intentional benchmark-definition/version change, a pinned toolchain/runtime change, a materially different reference machine, or an accepted performance change that needs a new snapshot. Each update states the reason. Ordinary pull requests do not refresh local numbers, and reviewers do not treat them as thresholds. CodSpeed remains authoritative for trends.

### Verification for the implementation tickets

The later code/workflow changes must pass all existing repository checks plus benchmark-specific checks:

```bash
cargo fmt --all -- --check
cargo check --workspace
cargo test --workspace
cargo clippy --workspace -- -D warnings
cargo check -p quickpython-bench --all-targets --features runner
cargo clippy -p quickpython-bench --benches --features runner -- -D warnings
cargo bench -p quickpython-bench --bench main --features runner --no-run
```

On Linux and macOS with the pinned CPython:

```bash
PYO3_PYTHON="$(uv python find 3.14.7)" cargo test -p quickpython-bench --test cpython_workloads --features cpython
PYO3_PYTHON="$(uv python find 3.14.7)" cargo check -p quickpython-bench --all-targets --features cpython
PYO3_PYTHON="$(uv python find 3.14.7)" cargo clippy -p quickpython-bench --all-targets --features cpython -- -D warnings
PYO3_PYTHON="$(uv python find 3.14.7)" cargo bench -p quickpython-bench --bench main --features cpython --no-run
```

Finally, run the exact CodSpeed build command in workflow validation. Current CI ignores Markdown-only changes, so these commands begin with the benchmark implementation; this design ticket is validated by document review and repository checks only.

## Dependency-ordered implementation breakdown

### 1. Benchmark scaffold, lifecycle helpers, profiles, and commands

Create the non-published sibling package and add it to the workspace. Add the optional `runner`/`pprof`/`cpython` feature graph, exact dependency pins, the `main` target, root `codspeed` and `profiling` profiles, `cfg(codspeed)` lint declaration, stable dimension/runtime naming, `Expected` normalization, context setup kinds, and four timing helpers with the boundaries/reset/drop behavior defined above. Add `benchmark-report` with median and ratio output. Register only `context` and `add_two` initially to prove all four dimensions and commands.

Done when default workspace commands avoid PyO3, runner check/clippy/smoke succeeds, local Criterion and Unix/non-Unix cfg branches compile as applicable, names match the contract, and helper unit tests prove setup is excluded and mutable state is not reused.

### 2. Correctness fixtures and supported/adapted workloads

Add the shared registry, exact `.py`/JSON sources, always-on QuickPython correctness tests, initial sizes/results from the matrix, and all supported/adapted/QuickPython-only registrations. Calibrate only before establishing the first baseline. Ensure JSON/regex/async modules and external data are preloaded in excluded per-iteration setup. Add explicit documentation beside each adapted fixture naming the unsupported Monty semantic.

Done when every enabled timed fixture has an unconditional non-benchmark QuickPython pass, all expected integer results match the matrix (or a documented pre-baseline calibrated update), CodSpeed's enabled list contains every QuickPython case and no deferred/excluded case, and generator traversal is absent.

Generator traversal is a separate prerequisite ticket, not part of this step: replace the accepted-error behavior with a strict test that executes traversal and asserts all values, fix the runtime, and land that correctness change first. Only a subsequent benchmark change may add `generator_traversal` to fixtures/registry/runners.

### 3. CPython/PyO3 comparison harness

Add pinned CPython/PyO3 metadata, feature-gated rpath build behavior, `wrap_for_cpython` plus its edge-case tests, raw compile, compile-once/call-many execution, embedded end-to-end helpers, feature-gated CPython correctness tests, local-only registrations, and report joining. Validate Linux and macOS prerequisites and ensure `cfg(codspeed)` plus feature gating excludes the whole CPython path from CodSpeed.

Done when the pinned CPython correctness command asserts the same expected value for every comparable fixture, paired smoke and clippy commands pass, the local report emits ratios only for equivalent/adapted exact bodies, and QuickPython-only rows emit `—`.

### 4. CodSpeed workflow and initial baseline documentation

Complete the external CodSpeed project/repository authorization first. Add the PR/`master`/manual workflow with minimum permissions, pinned actions/CLI, release-derived symbolic profile, exact build/run commands, one bounded `main` target, and a timeout. Run a stable local paired benchmark on the documented machine and commit `spec/benchmark-baselines.md` using the full schema and update policy.

Done when a pull request uploads all enabled QuickPython cases, a `master` run establishes history, smoke jobs are described only as build checks, the baseline report is reproducible from its metadata, and the workflow remains informational under the bootstrap regression policy.

These four tickets are ordered: runner contracts precede fixtures, strictly correct fixtures precede CPython comparison, and stable names/measurements precede CI history and the first baseline. External CodSpeed setup blocks only ticket 4. The generator correctness fix blocks only the later addition of that one workload and must not delay the supported initial suite.

## Out of scope for this design ticket

- implementing or running the suite beyond checks needed to validate this design;
- optimizing QuickPython based on any result;
- fixing generator traversal;
- adding Monty pool, decode, worker allocator, allocator ceiling, resource-limit, or GC comparisons;
- benchmarking operating-system process startup; and
- establishing an absolute QuickPython-versus-Monty/CPython performance gate.
