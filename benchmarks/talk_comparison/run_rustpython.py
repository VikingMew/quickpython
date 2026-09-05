#!/usr/bin/env python3
"""RustPython same-source benchmark for talk comparison (2026-09-05).

Reuses the measurement contract from run.py (host-timed whole process,
fixed in-runtime loops, checksum validation) to add a RustPython column
comparable with the existing QuickPython/CPython/PyPy snapshots.
"""

from __future__ import annotations

import json
import platform
import subprocess
import sys
import tempfile
from pathlib import Path

sys.path.insert(0, str(Path(__file__).resolve().parent))

from run import (  # noqa: E402
    COLD_RUNS,
    HERE,
    ROOT,
    SPECS,
    command_output,
    cpu_description,
    measure_runtime,
    prepare_sources,
)


def main() -> None:
    with tempfile.TemporaryDirectory(prefix="qpy-rustpython-") as temporary:
        sources = prepare_sources(Path(temporary))
        rp_version = command_output(["rustpython", "--version"]).splitlines()[0]
        environment = {
            "os_arch": platform.platform(),
            "cpu": cpu_description(),
            "cpython": command_output(["python3", "--version"]),
            "rustpython": rp_version,
        }
        data = {
            "schema_version": 1,
            "environment": environment,
            "commands": [
                "rustpython <generated-or-committed-workload.py>",
                "python3 <generated-or-committed-workload.py>",
            ],
            "measurement": {"non_cold_runs": 3, "cold_runs": COLD_RUNS,
                            "clock": "time.perf_counter_ns"},
            "results": {
                "rustpython": measure_runtime(["rustpython"], sources, 300.0),
                "cpython": measure_runtime(["python3"], sources, 300.0),
            },
        }
    out = HERE / "raw-results-rustpython.json"
    out.write_text(json.dumps(data, indent=2) + "\n", encoding="utf-8")

    print(f"# RustPython vs CPython ({environment['rustpython']})\n")
    print("| workload | N | RustPython | CPython | RP/CPython | status |")
    print("|---|---:|---:|---:|---:|---|")
    rp_r, cpy_r = data["results"]["rustpython"], data["results"]["cpython"]
    for name in ["cold_start", *SPECS]:
        rr, cr = rp_r[name], cpy_r[name]
        n = COLD_RUNS if name == "cold_start" else SPECS[name][0]
        if rr["status"] == cr["status"] == "success":
            if name == "cold_start":
                rtext = f"{rr['min_ms']:.2f}/{rr['avg_ms']:.2f} ms"
                ctext = f"{cr['min_ms']:.2f}/{cr['avg_ms']:.2f} ms"
                ratio = rr["min_ms"] / cr["min_ms"]
            else:
                rtext = f"{rr['min_us_per_iter']:.2f} µs"
                ctext = f"{cr['min_us_per_iter']:.2f} µs"
                ratio = rr["min_us_per_iter"] / cr["min_us_per_iter"]
            note = "success"
            rtext_r = f"{ratio:.2f}x"
        else:
            rtext = "failed" if rr["status"] != "success" else "ok"
            ctext = "failed" if cr["status"] != "success" else "ok"
            rtext_r = "—"
            failed = rr if rr["status"] != "success" else cr
            note = f"{failed['stage']}; {failed['category']}"
        print(f"| {name} | {n} | {rtext} | {ctext} | {rtext_r} | {note} |")
    print(f"\nSaved to {out}")


if __name__ == "__main__":
    main()
