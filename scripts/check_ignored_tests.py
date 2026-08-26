#!/usr/bin/env python3
"""Compare Cargo's ignored tests with the committed rationale registry."""

from __future__ import annotations

import argparse
import json
import re
import subprocess
import sys
from pathlib import Path


ROOT = Path(__file__).resolve().parents[1]
DEFAULT_REGISTRY = ROOT / "docs" / "ignored-tests.json"
REFERENCE_RE = re.compile(r"(?:[A-Z][A-Z0-9]+-\d+|#\d+|https?://\S+)")


def load_registry(path: Path) -> tuple[dict[str, str], list[str]]:
    errors: list[str] = []
    try:
        payload = json.loads(path.read_text(encoding="utf-8"))
    except (OSError, json.JSONDecodeError) as error:
        return {}, [f"cannot read registry {path}: {error}"]

    entries = payload.get("tests") if isinstance(payload, dict) else None
    if not isinstance(entries, list):
        return {}, ["registry root must contain a 'tests' array"]

    registry: dict[str, str] = {}
    for index, entry in enumerate(entries, start=1):
        if not isinstance(entry, dict):
            errors.append(f"entry {index} must be an object")
            continue
        name = entry.get("name")
        rationale = entry.get("rationale")
        if not isinstance(name, str) or not name.strip():
            errors.append(f"entry {index} has no fully qualified test name")
            continue
        name = name.strip()
        if "::" not in name:
            errors.append(f"entry {index} is not fully qualified: {name}")
        if name in registry:
            errors.append(f"duplicate registry entry: {name}")
        if not isinstance(rationale, str) or not rationale.strip():
            errors.append(f"missing rationale: {name}")
            rationale = ""
        elif not REFERENCE_RE.search(rationale):
            errors.append(f"rationale has no issue/task reference: {name}")
        registry[name] = rationale.strip()
    return registry, errors


def collect_ignored_tests() -> set[str]:
    command = ["cargo", "test", "--workspace", "--", "--list", "--ignored", "--format", "terse"]
    result = subprocess.run(
        command,
        cwd=ROOT,
        check=False,
        text=True,
        stdout=subprocess.PIPE,
        stderr=subprocess.STDOUT,
    )
    if result.returncode != 0:
        raise RuntimeError(f"{' '.join(command)} failed:\n{result.stdout.rstrip()}")

    tests: set[str] = set()
    for line in result.stdout.splitlines():
        if line.endswith(": test"):
            tests.add(line[: -len(": test")].strip())
    return tests


def parse_args() -> argparse.Namespace:
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("--registry", type=Path, default=DEFAULT_REGISTRY)
    parser.add_argument(
        "--test-name",
        action="append",
        default=[],
        help="compare explicit collected name(s) instead of running Cargo",
    )
    return parser.parse_args()


def main() -> int:
    args = parse_args()
    registry_path = args.registry if args.registry.is_absolute() else ROOT / args.registry
    registry, errors = load_registry(registry_path)
    if errors:
        print("Ignored-test registry is invalid:", file=sys.stderr)
        for error in errors:
            print(f"  - {error}", file=sys.stderr)
        return 2

    try:
        collected = set(args.test_name) if args.test_name else collect_ignored_tests()
    except RuntimeError as error:
        print(f"Could not collect ignored tests: {error}", file=sys.stderr)
        return 2

    registered = set(registry)
    unregistered = sorted(collected - registered)
    stale = sorted(registered - collected)
    if unregistered or stale:
        print("Ignored-test registry mismatch.", file=sys.stderr)
        if unregistered:
            print("Unregistered ignored tests:", file=sys.stderr)
            for name in unregistered:
                print(f"  - {name}", file=sys.stderr)
        if stale:
            print("Stale registry entries (remove them or restore the ignore):", file=sys.stderr)
            for name in stale:
                print(f"  - {name}", file=sys.stderr)
        return 1

    print(f"Ignored-test registry matches {len(collected)} collected ignored test(s).")
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
