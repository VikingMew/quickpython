# QuickPython vs CPython benchmark results

## Environment

- os_arch: `macOS-26.5.2-arm64-arm-64bit`
- cpu: `arm`
- quickpython_commit: `f85a4a7b2da2bbde2347b7303484f48f7463ca82`
- rustc: `rustc 1.95.0 (59807616e 2026-04-14)`
- cargo: `cargo 1.95.0 (f2d3ce0bd 2026-03-21)`
- cpython: `Python 3.11.5`

## Commands

- `cargo build --release`
- `target/release/quickpython run <generated-or-committed-workload.py>`
- `python3 <generated-or-committed-workload.py>`

## Comparison

| workload | N | QuickPython | CPython | QPY/CPython | status / notes |
|---|---:|---:|---:|---:|---|
| cold_start | 20 | 2.473/3.056 ms min/avg | 14.668/15.284 ms min/avg | 0.17x | success; checksum verified |
| json_loads | 200000 | 12.862 µs/iter | 6.472 µs/iter | 1.99x | success; checksum verified |
| json_dumps | 200000 | 11.030 µs/iter | 9.982 µs/iter | 1.10x | success; checksum verified |
| compute_loop | 20000 | 60.795 µs/iter | 8.442 µs/iter | 7.20x | success; checksum verified |
| string_join | 200000 | 0.686 µs/iter | 0.155 µs/iter | 4.43x | success; checksum verified |
| dict_ops | 50000 | 17.548 µs/iter | 2.556 µs/iter | 6.87x | success; checksum verified |

## Raw elapsed samples

### quickpython

- cold_start: `0.012, 0.003, 0.003, 0.003, 0.003, 0.003, 0.002, 0.003, 0.003, 0.003, 0.003, 0.003, 0.002, 0.003, 0.002, 0.003, 0.003, 0.003, 0.003, 0.003` seconds
- json_loads: `2.572, 2.639, 2.622` seconds
- json_dumps: `2.239, 2.206, 2.263` seconds
- compute_loop: `1.269, 1.224, 1.216` seconds
- string_join: `0.139, 0.137, 0.138` seconds
- dict_ops: `0.878, 0.880, 0.877` seconds

### cpython

- cold_start: `0.024, 0.015, 0.015, 0.015, 0.015, 0.015, 0.015, 0.015, 0.015, 0.015, 0.015, 0.015, 0.015, 0.015, 0.015, 0.015, 0.015, 0.015, 0.015, 0.015` seconds
- json_loads: `1.294, 1.342, 1.302` seconds
- json_dumps: `2.043, 2.008, 1.996` seconds
- compute_loop: `0.169, 0.169, 0.174` seconds
- string_join: `0.031, 0.031, 0.031` seconds
- dict_ops: `0.167, 0.128, 0.165` seconds

## Conclusion

This same-machine snapshot supports comparisons only for the successful workloads (cold_start, json_loads, json_dumps, compute_loop, string_join, dict_ops). For short-lived agent calls, cold-start results include process startup plus parsing, compilation, and execution, while non-cold results amortize those costs over the fixed in-runtime loop. Unavailable workloads (none) are compatibility findings and provide no evidence about their relative performance.
