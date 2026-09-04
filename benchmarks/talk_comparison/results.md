# QuickPython vs CPython benchmark results

## Environment

- os_arch: `Linux-7.0.11-orbstack-00354-gde2ab3f9e944-aarch64-with-glibc2.41`
- cpu: `unavailable`
- quickpython_commit: `a8168c3432111fd341c581ffacb798351054650f`
- rustc: `rustc 1.98.1 (48a229cea 2026-09-01)`
- cargo: `cargo 1.98.1 (797e8a9bc 2026-08-05)`
- cpython: `Python 3.11.16`

## Commands

- `cargo build --release`
- `target/release/quickpython run <generated-or-committed-workload.py>`
- `python3 <generated-or-committed-workload.py>`

## Comparison

| workload | N | QuickPython | CPython | QPY/CPython | status / notes |
|---|---:|---:|---:|---:|---|
| cold_start | 20 | 0.491/1.779 ms min/avg | 5.422/7.012 ms min/avg | 0.09x | success; checksum verified |
| json_loads | 200000 | 15.900 µs/iter | 5.849 µs/iter | 2.72x | success; checksum verified |
| json_dumps | 200000 | failed | success | — | runtime; semantic difference: expected stdout '1815', got '1685' |
| compute_loop | 20000 | 60.102 µs/iter | 7.614 µs/iter | 7.89x | success; checksum verified |
| string_join | 200000 | 0.515 µs/iter | 0.101 µs/iter | 5.09x | success; checksum verified |
| dict_ops | 50000 | 16.499 µs/iter | 2.200 µs/iter | 7.50x | success; checksum verified |

## Raw elapsed samples

### quickpython

- cold_start: `0.002, 0.001, 0.002, 0.001, 0.002, 0.002, 0.003, 0.000, 0.003, 0.002, 0.002, 0.002, 0.001, 0.002, 0.002, 0.000, 0.002, 0.002, 0.001, 0.002` seconds
- json_loads: `3.180, 3.335, 3.269` seconds
- json_dumps: failed at runtime (semantic difference): expected stdout '1815', got '1685'
- compute_loop: `1.312, 1.301, 1.202` seconds
- string_join: `0.105, 0.103, 0.103` seconds
- dict_ops: `0.841, 0.825, 0.884` seconds

### cpython

- cold_start: `0.009, 0.008, 0.007, 0.007, 0.007, 0.007, 0.007, 0.007, 0.007, 0.006, 0.006, 0.009, 0.005, 0.007, 0.007, 0.007, 0.006, 0.007, 0.007, 0.007` seconds
- json_loads: `1.208, 1.170, 1.253` seconds
- json_dumps: `1.722, 1.672, 1.696` seconds
- compute_loop: `0.152, 0.176, 0.220` seconds
- string_join: `0.022, 0.020, 0.021` seconds
- dict_ops: `0.126, 0.111, 0.110` seconds

## Conclusion

This same-machine snapshot supports comparisons only for the successful workloads (cold_start, json_loads, compute_loop, string_join, dict_ops). For short-lived agent calls, cold-start results include process startup plus parsing, compilation, and execution, while non-cold results amortize those costs over the fixed in-runtime loop. Unavailable workloads (json_dumps) are compatibility findings and provide no evidence about their relative performance.
