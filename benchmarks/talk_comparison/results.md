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

## PyPy cross-check (2026-09-05, same machine, same measurement contract)

Runner: `run_pypy.py` (host-timed whole process, checksum-verified, 3 non-cold runs / 20 cold runs). CPython re-measured in the same invocation, so the PyPy/CPython ratio shares a single baseline; note cold-start CPython varies run to run (~15–18 ms) from machine noise.

| workload | N | PyPy | CPython | PyPy/CPython | status / notes |
|---|---:|---:|---:|---:|---|
| cold_start | 20 | 29.840/46.587 ms min/avg | 17.962/19.915 ms min/avg | 1.66x | success; checksum verified (JIT startup + warmup) |
| json_loads | 200000 | 5.666 µs/iter | 8.248 µs/iter | 0.69x | success; checksum verified |
| json_dumps | 200000 | 14.527 µs/iter | 10.670 µs/iter | 1.36x | success; checksum verified (stdlib json is pure Python on PyPy) |
| compute_loop | 20000 | 1.554 µs/iter | 8.797 µs/iter | 0.18x | success; checksum verified (JIT hot loop) |
| string_join | 200000 | 0.146 µs/iter | 0.168 µs/iter | 0.87x | success; checksum verified |
| dict_ops | 50000 | 1.139 µs/iter | 2.619 µs/iter | 0.44x | success; checksum verified |

Reading: PyPy's JIT dominates pure-Python compute (compute_loop/dict_ops/json_loads faster than CPython), but pays ~1.7x on cold start and loses json.dumps (~1.4x slower) because its stdlib JSON is pure Python without the rjson backend. For short-lived agent calls the JIT never warms up, so PyPy is a long-running compute accelerator, not an agent short-call answer.

### pypy raw elapsed samples

- cold_start: `0.479, 0.047, 0.032, 0.031, 0.030, 0.030, 0.030, 0.030, 0.030, 0.030, 0.030, 0.030, 0.030, 0.030, 0.030, 0.030, 0.030, 0.030, 0.030, 0.030` seconds
- json_loads: `1.133, 1.135, 1.134` seconds
- json_dumps: `2.908, 2.905, 2.906` seconds
- compute_loop: `0.033, 0.031, 0.031` seconds
- string_join: `0.030, 0.029, 0.029` seconds
- dict_ops: `0.061, 0.057, 0.057` seconds

### cpython (re-measured same run) raw elapsed samples

- cold_start: `0.150, 0.018, 0.018, 0.018, 0.018, 0.018, 0.018, 0.018, 0.018, 0.018, 0.018, 0.018, 0.018, 0.018, 0.018, 0.018, 0.018, 0.018, 0.018, 0.018` seconds
- json_loads: `1.650, 1.650, 1.649` seconds
- json_dumps: `2.144, 2.136, 2.131` seconds
- compute_loop: `0.183, 0.176, 0.178` seconds
- string_join: `0.034, 0.034, 0.033` seconds
- dict_ops: `0.140, 0.132, 0.131` seconds
