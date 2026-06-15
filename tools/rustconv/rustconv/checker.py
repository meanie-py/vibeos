"""Cargo check orchestration + structured error reporting."""

import json
import subprocess
import sys
from dataclasses import dataclass, field
from pathlib import Path

from .config import OUTPUT_CRATE, RUST_TARGETS


@dataclass
class CompileError:
    file: str
    line: int
    column: int
    level: str  # "error", "warning"
    code: str | None  # e.g., "E0432"
    message: str
    category: str  # "missing_import", "type_mismatch", "missing_extern", "syntax", "other"


@dataclass
class CheckResult:
    target: str
    errors: list[CompileError] = field(default_factory=list)
    warnings: list[CompileError] = field(default_factory=list)
    success: bool = False


def _categorize_error(code: str | None, message: str) -> str:
    """Categorize a compile error for reporting."""
    if code == "E0432" or "unresolved import" in message:
        return "missing_import"
    if code == "E0412" or "cannot find type" in message:
        return "missing_type"
    if code == "E0425" or "cannot find value" in message or "cannot find function" in message:
        return "missing_extern"
    if code == "E0308" or "mismatched types" in message:
        return "type_mismatch"
    if code == "E0061" or "arguments were supplied" in message:
        return "arg_count"
    if "expected" in message and "found" in message:
        return "type_mismatch"
    if code and code.startswith("E"):
        return "other"
    return "syntax"


def check_target(target: str, crate_dir: Path = OUTPUT_CRATE) -> CheckResult:
    """Run cargo check for a single target."""
    result = CheckResult(target=target)

    try:
        proc = subprocess.run(
            [
                "cargo",
                "check",
                "--target",
                target,
                "--message-format=json",
            ],
            capture_output=True,
            text=True,
            cwd=str(crate_dir),
            timeout=120,
        )
    except subprocess.TimeoutExpired:
        result.errors.append(
            CompileError(
                file="",
                line=0,
                column=0,
                level="error",
                code=None,
                message="cargo check timed out after 120s",
                category="other",
            )
        )
        return result
    except FileNotFoundError:
        result.errors.append(
            CompileError(
                file="",
                line=0,
                column=0,
                level="error",
                code=None,
                message="cargo not found",
                category="other",
            )
        )
        return result

    # Parse JSON messages
    for line in proc.stdout.splitlines():
        try:
            msg = json.loads(line)
        except json.JSONDecodeError:
            continue

        if msg.get("reason") != "compiler-message":
            continue

        compiler_msg = msg.get("message", {})
        level = compiler_msg.get("level", "")
        if level not in ("error", "warning"):
            continue

        code_obj = compiler_msg.get("code")
        code = code_obj.get("code") if code_obj else None
        message = compiler_msg.get("message", "")

        # Get primary span
        spans = compiler_msg.get("spans", [])
        primary = None
        for span in spans:
            if span.get("is_primary"):
                primary = span
                break
        if primary is None and spans:
            primary = spans[0]

        file_name = primary.get("file_name", "") if primary else ""
        line_num = primary.get("line_start", 0) if primary else 0
        col = primary.get("column_start", 0) if primary else 0

        error = CompileError(
            file=file_name,
            line=line_num,
            column=col,
            level=level,
            code=code,
            message=message,
            category=_categorize_error(code, message),
        )

        if level == "error":
            result.errors.append(error)
        else:
            result.warnings.append(error)

    result.success = len(result.errors) == 0
    return result


def check_all_targets(crate_dir: Path = OUTPUT_CRATE) -> dict[str, CheckResult]:
    """Run cargo check for all 3 targets."""
    results = {}
    for arch, target in sorted(RUST_TARGETS.items()):
        print(f"  Checking {arch} ({target})...", file=sys.stderr)
        result = check_target(target, crate_dir)
        results[arch] = result
        if result.success:
            print("    ✓ No errors", file=sys.stderr)
        else:
            print(
                f"    ✗ {len(result.errors)} errors, {len(result.warnings)} warnings",
                file=sys.stderr,
            )
    return results


def print_check_report(results: dict[str, CheckResult]) -> None:
    """Print a structured error report."""
    all_ok = all(r.success for r in results.values())

    if all_ok:
        print("All targets compile successfully!")
        return

    # Categorize errors across targets
    categories: dict[str, list[tuple[str, CompileError]]] = {}
    for arch, result in results.items():
        for error in result.errors:
            cat = error.category
            if cat not in categories:
                categories[cat] = []
            categories[cat].append((arch, error))

    for cat, errors in sorted(categories.items()):
        print(f"\n{cat.upper()} ({len(errors)} errors):")
        seen = set()
        for arch, err in errors:
            key = (err.file, err.line, err.message)
            if key in seen:
                continue
            seen.add(key)
            loc = f"{err.file}:{err.line}:{err.column}" if err.file else "(unknown)"
            code = f"[{err.code}] " if err.code else ""
            print(f"  {loc}: {code}{err.message} ({arch})")
