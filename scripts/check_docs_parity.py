#!/usr/bin/env python3
"""Require compatibility documentation alongside protected source changes."""

from __future__ import annotations

import argparse
import subprocess
import sys
from pathlib import Path


ROOT = Path(__file__).resolve().parents[1]

# Keep this mapping explicit: reviewers should be able to identify the contract
# owned by every user-visible implementation surface without interpreting code.
EXACT_MAPPINGS: dict[str, tuple[str, ...]] = {
    "src/compiler.rs": ("docs/limitations/compiler.md",),
    "src/bytecode.rs": ("docs/limitations/compiler.md",),
    "src/vm.rs": ("docs/limitations/runtime.md",),
    "src/value.rs": ("docs/limitations/runtime.md",),
    "src/main.rs": ("docs/limitations/extensions.md",),
    "src/context.rs": ("docs/limitations/extensions.md",),
    "src/serializer.rs": ("docs/limitations/serialization.md",),
    "src/builtins/mod.rs": ("docs/limitations/README.md",),
    "src/builtins/json.rs": ("docs/limitations/json.md",),
    "src/builtins/os.rs": ("docs/limitations/os.md",),
    "src/builtins/re.rs": ("docs/limitations/re.md",),
    "src/builtins/asyncio.rs": ("docs/limitations/asyncio.md",),
}

PREFIX_MAPPINGS: tuple[tuple[str, tuple[str, ...]], ...] = (
    ("quickpython-llm/src/", ("docs/limitations/extensions.md",)),
)


def run_git(*args: str) -> list[str]:
    result = subprocess.run(
        ["git", *args],
        cwd=ROOT,
        check=False,
        text=True,
        stdout=subprocess.PIPE,
        stderr=subprocess.PIPE,
    )
    if result.returncode != 0:
        detail = result.stderr.strip() or result.stdout.strip()
        raise RuntimeError(f"git {' '.join(args)} failed: {detail}")
    return [line.strip().replace("\\", "/") for line in result.stdout.splitlines() if line.strip()]


def changed_files(base: str, head: str, include_worktree: bool) -> set[str]:
    files = set(
        run_git("diff", "--name-only", "--diff-filter=ACDMR", "--no-renames", f"{base}...{head}")
    )
    if include_worktree:
        files.update(run_git("diff", "--name-only", "--diff-filter=ACDMR", "--no-renames", "HEAD"))
        files.update(run_git("ls-files", "--others", "--exclude-standard"))
    return files


def mapped_documents(path: str) -> tuple[str, ...] | None:
    if path in EXACT_MAPPINGS:
        return EXACT_MAPPINGS[path]
    for prefix, documents in PREFIX_MAPPINGS:
        if path.startswith(prefix):
            return documents
    return None


def validate_mapping() -> list[str]:
    errors: list[str] = []
    builtins = [
        path.relative_to(ROOT).as_posix()
        for path in (ROOT / "src" / "builtins").rglob("*")
        if path.is_file()
    ]
    for path in builtins:
        if path not in EXACT_MAPPINGS:
            errors.append(f"protected built-in has no explicit mapping: {path}")

    extension_sources = [
        path.relative_to(ROOT).as_posix()
        for path in (ROOT / "quickpython-llm" / "src").rglob("*")
        if path.is_file()
    ]
    for path in extension_sources:
        if mapped_documents(path) is None:
            errors.append(f"protected extension source has no mapping: {path}")

    mapped_docs = {doc for docs in EXACT_MAPPINGS.values() for doc in docs}
    mapped_docs.update(doc for _, docs in PREFIX_MAPPINGS for doc in docs)
    for document in sorted(mapped_docs):
        if not (ROOT / document).is_file():
            errors.append(f"mapped limitations document does not exist: {document}")
    return errors


def parity_failures(files: set[str]) -> list[tuple[str, str]]:
    failures: list[tuple[str, str]] = []
    for path in sorted(files):
        documents = mapped_documents(path)
        if documents is None:
            continue
        for document in documents:
            if document not in files:
                failures.append((path, document))
    return failures


def parse_args() -> argparse.Namespace:
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("--base", default="origin/master", help="base revision (default: origin/master)")
    parser.add_argument("--head", default="HEAD", help="head revision (default: HEAD)")
    parser.add_argument(
        "--no-worktree",
        action="store_true",
        help="do not include staged, unstaged, and untracked local changes",
    )
    parser.add_argument(
        "--changed-file",
        action="append",
        default=[],
        help="check an explicit changed path instead of querying git; repeat as needed",
    )
    return parser.parse_args()


def main() -> int:
    args = parse_args()
    try:
        mapping_errors = validate_mapping()
        if mapping_errors:
            print("Documentation parity mapping is invalid:", file=sys.stderr)
            for error in mapping_errors:
                print(f"  - {error}", file=sys.stderr)
            return 2

        if args.changed_file:
            files = {path.replace("\\", "/") for path in args.changed_file}
        else:
            files = changed_files(args.base, args.head, not args.no_worktree)
    except RuntimeError as error:
        print(f"Documentation parity check could not inspect changes: {error}", file=sys.stderr)
        return 2

    failures = parity_failures(files)
    if failures:
        print("Documentation parity check failed.", file=sys.stderr)
        print("Protected implementation changes are missing mapped limitations updates:", file=sys.stderr)
        for source, document in failures:
            print(f"  - {source} -> {document}", file=sys.stderr)
        print("Update the listed document(s), then rerun scripts/check_docs_parity.py.", file=sys.stderr)
        return 1

    protected = sorted(path for path in files if mapped_documents(path) is not None)
    if protected:
        print(f"Documentation parity check passed for {len(protected)} protected path(s).")
    else:
        print("Documentation parity check passed; no protected implementation paths changed.")
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
