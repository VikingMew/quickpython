# QuickPython vs CPython talk benchmark

This is the one-off, process-level benchmark used for QPY-83. It is separate
from the long-term suite proposed in `spec/benchmarks.md`.

The harness builds QuickPython once in release mode, generates each workload
from the committed templates and fixture, and then invokes the resulting
binary directly. CPython and QuickPython receive the exact same generated
source file. Non-cold workloads run three times and report the minimum
wall-clock time divided by the fixed iteration count. Cold start uses twenty
fresh processes and reports minimum and average wall-clock time.

From the repository root, run:

```sh
python3 benchmarks/talk_comparison/run.py
```

The default output files are `raw-results.json` and `results.md` in this
directory. A single workload process is limited to five minutes by default;
use `--timeout SECONDS` to change that. A timeout or unsupported operation is
reported as a failure, never as a timing result. The harness does not invoke a
container engine.

The checked-in result files are a snapshot from the machine described in
`results.md`. Re-running replaces them with a new same-machine comparison.
