#!/usr/bin/env python3
"""PyPy same-source benchmark for talk comparison (2026-09-05).

Reuses the measurement contract from run.py (host-timed whole process,
fixed in-runtime loops, checksum validation) to add a PyPy column that is
directly comparable with the existing QuickPython/CPython snapshot.
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
    with tempfile.TemporaryDirectory(prefix="qpy-pypy-") as temporary:
        sources = prepare_sources(Path(temporary))
        pypy = command_output(["pypy3", "--version"]).splitlines()[0]
        environment = {
            "os_arch": platform.platform(),
            "cpu": cpu_description(),
            "cpython": command_output(["python3", "--version"]),
            "pypy": pypy,
        }
        data = {
            "schema_version": 1,
            "environment": environment,
            "commands": [
                "pypy3 <generated-or-committed-workload.py>",
                "python3 <generated-or-committed-workload.py>",
            ],
            "measurement": {"non_cold_runs": 3, "cold_runs": COLD_RUNS,
                            "clock": "time.perf_counter_ns"},
            "results": {
                "pypy": measure_runtime(["pypy3"], sources, 300.0),
                "cpython": measure_runtime(["python3"], sources, 300.0),
            },
        }
    out = HERE / "raw-results-pypy.json"
    out.write_text(json.dumps(data, indent=2) + "\n", encoding="utf-8")

    print(f"# PyPy vs CPython ({environment['pypy']})\n")
    print("| workload | N | PyPy | CPython | PyPy/CPython | status |")
    print("|---|---:|---:|---:|---:|---|")
    pypy_r, cpy_r = data["results"]["pypy"], data["results"]["cpython"]
    for name in ["cold_start", *SPECS]:
        pr, cr = pypy_r[name], cpy_r[name]
        n = COLD_RUNS if name == "cold_start" else SPECS[name][0]
        if pr["status"] == cr["status"] == "success":
            if name == "cold_start":
                ptext = f"{pr['min_ms']:.2f}/{pr['avg_ms']:.2f} ms"
                ctext = f"{cr['min_ms']:.2f}/{cr['avg_ms']:.2f} ms"
                ratio = pr["min_ms"] / cr["min_ms"]
            else:
                ptext = f"{pr['min_us_per_iter']:.2f} µs"
                ctext = f"{cr['min_us_per_iter']:.2f} µs"
                ratio = pr["min_us_per_iter"] / cr["min_us_per_iter"]
            note = "success"
            rtext = f"{ratio:.2f}x"
        else:
            ptext = "failed" if pr["status"] != "success" else "ok"
            ctext = "failed" if cr["status"] != "success" else "ok"
            rtext = "—"
            failed = pr if pr["status"] != "success" else cr
            note = f"{failed['stage']}; {failed['category']}"
        print(f"| {name} | {n} | {ptext} | {ctext} | {rtext} | {note} |")
    print(f"\nSaved to {out}")


if __name__ == "__main__":
    main()
