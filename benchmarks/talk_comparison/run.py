#!/usr/bin/env python3
"""Collect the QPY-83 same-source, process-level benchmark snapshot."""

from __future__ import annotations

import argparse
import json
import os
import platform
import statistics
import subprocess
import tempfile
import time
from pathlib import Path
from typing import Any


ROOT = Path(__file__).resolve().parents[2]
HERE = Path(__file__).resolve().parent
WORKLOADS = HERE / "workloads"
FIXTURE = HERE / "fixtures" / "medium_response.json"
QPY = ROOT / "target" / "release" / "quickpython"
RUNS = 3
COLD_RUNS = 20
SPECS = {
    "json_loads": (200_000, "d50887ca-a6ce-4e59-b89f-14f0b5d03b03"),
    "json_dumps": (200_000, "1815"),
    "compute_loop": (20_000, "2646700"),
    "string_join": (200_000, "alpha,beta,gamma,delta"),
    "dict_ops": (50_000, "2401"),
}


def command_output(command: list[str]) -> str:
    return subprocess.run(command, cwd=ROOT, check=True, text=True,
                          capture_output=True).stdout.strip()


def cpu_description() -> str:
    description = platform.processor()
    cpuinfo = Path("/proc/cpuinfo")
    if not description and cpuinfo.is_file():
        for line in cpuinfo.read_text(encoding="utf-8").splitlines():
            if line.lower().startswith(("model name", "hardware", "processor")):
                value = line.partition(":")[2].strip()
                if value and not value.isdigit():
                    return value
    return description or "unavailable"


def prepare_sources(directory: Path) -> dict[str, Path]:
    sources = {"cold_start": WORKLOADS / "cold_start.py"}
    data_literal = repr(FIXTURE.read_text(encoding="utf-8"))
    for name in SPECS:
        plain = WORKLOADS / f"{name}.py"
        if plain.exists():
            sources[name] = plain
        else:
            rendered = (WORKLOADS / f"{name}.py.tmpl").read_text(
                encoding="utf-8").replace("__DATA__", data_literal)
            destination = directory / f"{name}.py"
            destination.write_text(rendered, encoding="utf-8")
            sources[name] = destination
    return sources


def invoke(command: list[str], timeout: float) -> dict[str, Any]:
    start = time.perf_counter_ns()
    try:
        process = subprocess.run(command, cwd=ROOT, text=True,
                                 capture_output=True, timeout=timeout)
    except subprocess.TimeoutExpired as error:
        return {"elapsed_seconds": (time.perf_counter_ns() - start) / 1e9,
                "returncode": None, "stdout": error.stdout or "",
                "stderr": error.stderr or "", "timed_out": True}
    return {"elapsed_seconds": (time.perf_counter_ns() - start) / 1e9,
            "returncode": process.returncode, "stdout": process.stdout,
            "stderr": process.stderr, "timed_out": False}


def classify(sample: dict[str, Any]) -> dict[str, str]:
    message = str(sample.get("stderr") or sample.get("stdout") or "").strip()
    lowered = message.lower()
    if sample["timed_out"]:
        return {"stage": "timeout", "category": "environment issue",
                "error": "process exceeded configured timeout"}
    if "parse" in lowered or "syntax" in lowered:
        stage, category = "parse", "unsupported syntax"
    elif "compile" in lowered or "not implemented" in lowered:
        stage, category = "compile", "unsupported syntax"
    elif "not found" in lowered or "has no attribute" in lowered or "unknown method" in lowered:
        stage, category = "runtime", "missing builtin/method"
    else:
        stage, category = "runtime", "other"
    return {"stage": stage, "category": category,
            "error": " ".join(message.split())[:300] or "non-zero exit"}


def validate(sample: dict[str, Any], expected: str) -> str | None:
    if sample["timed_out"] or sample["returncode"] != 0:
        return None
    lines = str(sample["stdout"]).strip().splitlines()
    return expected if lines == [expected] else None


def measure_runtime(command_prefix: list[str], sources: dict[str, Path],
                    timeout: float) -> dict[str, Any]:
    result: dict[str, Any] = {}
    cold_samples = [invoke(command_prefix + [str(sources["cold_start"])], timeout)
                    for _ in range(COLD_RUNS)]
    if all(validate(sample, "2") for sample in cold_samples):
        elapsed = [sample["elapsed_seconds"] for sample in cold_samples]
        result["cold_start"] = {
            "status": "success", "samples_seconds": elapsed,
            "min_ms": min(elapsed) * 1000,
            "avg_ms": statistics.fmean(elapsed) * 1000,
            "checksum": "2",
        }
    else:
        failed = next(sample for sample in cold_samples if not validate(sample, "2"))
        result["cold_start"] = {"status": "failed", **classify(failed),
                                "samples": cold_samples}

    for name, (iterations, expected) in SPECS.items():
        samples = [invoke(command_prefix + [str(sources[name])], timeout)
                   for _ in range(RUNS)]
        if all(validate(sample, expected) for sample in samples):
            elapsed = [sample["elapsed_seconds"] for sample in samples]
            result[name] = {
                "status": "success", "iterations": iterations,
                "samples_seconds": elapsed,
                "min_us_per_iter": min(elapsed) * 1_000_000 / iterations,
                "checksum": expected,
            }
        else:
            failed = next(sample for sample in samples if not validate(sample, expected))
            detail = classify(failed) if failed["returncode"] != 0 or failed["timed_out"] else {
                "stage": "runtime", "category": "semantic difference",
                "error": f"expected stdout {expected!r}, got {failed['stdout'].strip()!r}",
            }
            result[name] = {"status": "failed", "iterations": iterations,
                            **detail, "samples": samples}
    return result


def fmt(value: float) -> str:
    return f"{value:.3f}"


def render_markdown(data: dict[str, Any]) -> str:
    qpy, cpython = data["results"]["quickpython"], data["results"]["cpython"]
    lines = [
        "# QuickPython vs CPython benchmark results", "",
        "## Environment", "",
    ]
    for key, value in data["environment"].items():
        lines.append(f"- {key}: `{value}`")
    lines += ["", "## Commands", ""]
    lines += [f"- `{command}`" for command in data["commands"]]
    lines += ["", "## Comparison", "",
              "| workload | N | QuickPython | CPython | QPY/CPython | status / notes |",
              "|---|---:|---:|---:|---:|---|"]
    for name in ["cold_start", *SPECS]:
        qr, cr = qpy[name], cpython[name]
        n = COLD_RUNS if name == "cold_start" else SPECS[name][0]
        if qr["status"] == cr["status"] == "success":
            if name == "cold_start":
                qtext = f"{fmt(qr['min_ms'])}/{fmt(qr['avg_ms'])} ms min/avg"
                ctext = f"{fmt(cr['min_ms'])}/{fmt(cr['avg_ms'])} ms min/avg"
                ratio = qr["min_ms"] / cr["min_ms"]
            else:
                qtext = f"{fmt(qr['min_us_per_iter'])} µs/iter"
                ctext = f"{fmt(cr['min_us_per_iter'])} µs/iter"
                ratio = qr["min_us_per_iter"] / cr["min_us_per_iter"]
            note = "success; checksum verified"
            ratio_text = f"{ratio:.2f}x"
        else:
            qtext = "failed" if qr["status"] != "success" else "success"
            ctext = "failed" if cr["status"] != "success" else "success"
            ratio_text = "—"
            failed = qr if qr["status"] != "success" else cr
            note = f"{failed['stage']}; {failed['category']}: {failed['error']}"
        lines.append(f"| {name} | {n} | {qtext} | {ctext} | {ratio_text} | {note} |")
    lines += ["", "## Raw elapsed samples", ""]
    for runtime, runtime_results in data["results"].items():
        lines += [f"### {runtime}", ""]
        for name, item in runtime_results.items():
            if item["status"] == "success":
                samples = ", ".join(fmt(x) for x in item["samples_seconds"])
                lines.append(f"- {name}: `{samples}` seconds")
            else:
                lines.append(f"- {name}: failed at {item['stage']} ({item['category']}): "
                             f"{item['error']}")
        lines.append("")
    successful = [name for name in ["cold_start", *SPECS]
                  if qpy[name]["status"] == cpython[name]["status"] == "success"]
    failed = [name for name in ["cold_start", *SPECS] if name not in successful]
    lines += ["## Conclusion", "",
              "This same-machine snapshot supports comparisons only for the successful "
              f"workloads ({', '.join(successful) or 'none'}). For short-lived agent calls, "
              "cold-start results include process startup plus parsing, compilation, and execution, "
              "while non-cold results amortize those costs over the fixed in-runtime loop. "
              f"Unavailable workloads ({', '.join(failed) or 'none'}) are compatibility findings "
              "and provide no evidence about their relative performance.", ""]
    return "\n".join(lines)


def main() -> None:
    parser = argparse.ArgumentParser()
    parser.add_argument("--timeout", type=float, default=300.0)
    parser.add_argument("--json-output", type=Path, default=HERE / "raw-results.json")
    parser.add_argument("--markdown-output", type=Path, default=HERE / "results.md")
    args = parser.parse_args()

    # This is the only build. Every timed invocation below calls QPY directly.
    subprocess.run(["cargo", "build", "--release"], cwd=ROOT, check=True)
    if not QPY.is_file():
        raise SystemExit(f"release binary not found: {QPY}")

    with tempfile.TemporaryDirectory(prefix="qpy-83-") as temporary:
        sources = prepare_sources(Path(temporary))
        environment = {
            "os_arch": platform.platform(),
            "cpu": cpu_description(),
            "quickpython_commit": command_output(["git", "rev-parse", "HEAD"]),
            "rustc": command_output(["rustc", "--version"]),
            "cargo": command_output(["cargo", "--version"]),
            "cpython": command_output(["python3", "--version"]),
        }
        data = {
            "schema_version": 1,
            "environment": environment,
            "commands": [
                "cargo build --release",
                "target/release/quickpython run <generated-or-committed-workload.py>",
                "python3 <generated-or-committed-workload.py>",
            ],
            "measurement": {"non_cold_runs": RUNS, "cold_runs": COLD_RUNS,
                            "timeout_seconds": args.timeout,
                            "clock": "time.perf_counter_ns"},
            "results": {
                "quickpython": measure_runtime([str(QPY), "run"], sources, args.timeout),
                "cpython": measure_runtime(["python3"], sources, args.timeout),
            },
        }
    args.json_output.write_text(json.dumps(data, indent=2) + "\n", encoding="utf-8")
    args.markdown_output.write_text(render_markdown(data), encoding="utf-8")


if __name__ == "__main__":
    main()
