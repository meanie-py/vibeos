"""
migrate_guards.py — Automated migration of manual lock/IRQL patterns to RAII guards.

Targets the "easy 60%" — simple acquire-release pairs in the same scope with
no reacquire between them.

Usage:
    python tools/rustconv/rustconv/migrate_guards.py --scan           # find candidates
    python tools/rustconv/rustconv/migrate_guards.py --scan file.rs   # single file
    python tools/rustconv/rustconv/migrate_guards.py --fix            # apply transforms
    python tools/rustconv/rustconv/migrate_guards.py --fix file.rs    # single file
"""

import argparse
import re
import sys
from collections import Counter
from dataclasses import dataclass
from pathlib import Path

from rustconv.config import REACTOS_ROOT as _RUSTCONV_REACTOS_ROOT  # invoked tree root


@dataclass
class LockPair:
    """A matched acquire-release pair."""

    file: str
    acquire_line: int
    release_line: int
    pattern: str  # 'pfn', 'spinlock', 'irql'
    acquire_text: str
    release_text: str
    irql_var: str  # the OldIrql variable name
    lock_expr: str  # the lock expression (for spinlocks)
    simple: bool  # True if safe to auto-transform
    reason: str  # why not simple, if applicable


def scan_file(path: Path) -> list[LockPair]:
    """Scan a Rust file for acquire-release pairs."""
    pairs = []
    lines = path.read_text().splitlines()

    # Track open acquires
    open_acquires = []  # (line_num, pattern, irql_var, lock_expr, acquire_text)

    for i, line in enumerate(lines, 1):
        stripped = line.strip()

        # Skip comments
        if stripped.startswith("//"):
            continue

        # PFN lock acquire: OldIrql = MiAcquirePfnLock();
        # or: let OldIrql = MiAcquirePfnLock();
        m = re.match(r"(?:let\s+(?:mut\s+)?)?(\w+)\s*=\s*MiAcquirePfnLock\(\)", stripped)
        if m:
            open_acquires.append((i, "pfn", m.group(1), "", stripped))
            continue

        # PFN lock release: MiReleasePfnLock(OldIrql);
        m = re.match(r"MiReleasePfnLock\((\w+)\);?", stripped)
        if m:
            irql_var = m.group(1)
            # Find matching acquire
            for j in range(len(open_acquires) - 1, -1, -1):
                acq = open_acquires[j]
                if acq[1] == "pfn" and acq[2] == irql_var:
                    # Check if simple
                    simple, reason = _check_simple(lines, acq[0], i, irql_var, "MiAcquirePfnLock")
                    pairs.append(
                        LockPair(
                            file=str(path),
                            acquire_line=acq[0],
                            release_line=i,
                            pattern="pfn",
                            acquire_text=acq[4],
                            release_text=stripped,
                            irql_var=irql_var,
                            lock_expr="",
                            simple=simple,
                            reason=reason,
                        )
                    )
                    open_acquires.pop(j)
                    break
            continue

        # Spinlock acquire: KeAcquireSpinLock(lock_expr, &mut old_irql);
        m = re.match(r"KeAcquireSpinLock\((.+?),\s*&mut\s+(\w+)\)", stripped)
        if not m:
            m = re.match(r"KeAcquireSpinLock\((.+?),\s*core::ptr::addr_of_mut!\((\w+)\)", stripped)
        if m:
            lock_expr = m.group(1).strip().rstrip(",")
            irql_var = m.group(2)
            open_acquires.append((i, "spinlock", irql_var, lock_expr, stripped))
            continue

        # Spinlock release: KeReleaseSpinLock(lock_expr, old_irql);
        m = re.match(r"KeReleaseSpinLock\((.+?),\s*(\w+)\);?", stripped)
        if m:
            lock_expr = m.group(1).strip().rstrip(",")
            irql_var = m.group(2)
            for j in range(len(open_acquires) - 1, -1, -1):
                acq = open_acquires[j]
                if acq[1] == "spinlock" and acq[2] == irql_var:
                    simple, reason = _check_simple(lines, acq[0], i, irql_var, "KeAcquireSpinLock")
                    pairs.append(
                        LockPair(
                            file=str(path),
                            acquire_line=acq[0],
                            release_line=i,
                            pattern="spinlock",
                            acquire_text=acq[4],
                            release_text=stripped,
                            irql_var=irql_var,
                            lock_expr=lock_expr,
                            simple=simple,
                            reason=reason,
                        )
                    )
                    open_acquires.pop(j)
                    break
            continue

        # IRQL raise: KeRaiseIrql(LEVEL, &mut OldIrql);
        m = re.match(
            r"KeRaiseIrql\((.+?),\s*(?:&mut\s+(\w+)|core::ptr::addr_of_mut!\((\w+)\))", stripped
        )
        if m:
            level = m.group(1).strip()
            irql_var = m.group(2) or m.group(3)
            open_acquires.append((i, "irql", irql_var, level, stripped))
            continue

        # IRQL lower: KeLowerIrql(OldIrql);
        m = re.match(r"KeLowerIrql\((\w+)\);?", stripped)
        if m:
            irql_var = m.group(1)
            for j in range(len(open_acquires) - 1, -1, -1):
                acq = open_acquires[j]
                if acq[1] == "irql" and acq[2] == irql_var:
                    simple, reason = _check_simple(lines, acq[0], i, irql_var, "KeRaiseIrql")
                    pairs.append(
                        LockPair(
                            file=str(path),
                            acquire_line=acq[0],
                            release_line=i,
                            pattern="irql",
                            acquire_text=acq[4],
                            release_text=stripped,
                            irql_var=irql_var,
                            lock_expr=acq[3],  # the IRQL level
                            simple=simple,
                            reason=reason,
                        )
                    )
                    open_acquires.pop(j)
                    break

    return pairs


def _check_simple(
    lines: list[str], acquire_line: int, release_line: int, irql_var: str, acquire_fn: str
) -> tuple[bool, str]:
    """Check if an acquire-release pair is simple enough to auto-transform."""
    between = lines[acquire_line : release_line - 1]  # lines between acquire and release

    # Check for reacquire
    for line in between:
        if acquire_fn in line:
            return False, f"reacquire of {acquire_fn} between acquire and release"

    # Check if irql_var is used for anything other than the release
    for line in between:
        stripped = line.strip()
        if stripped.startswith("//"):
            continue
        # Skip the line that declares the variable
        if f"let {irql_var}" in stripped or f"let mut {irql_var}" in stripped:
            continue
        # Check if irql_var is referenced (other than in the release itself)
        if irql_var in stripped and "Release" not in stripped and "Lower" not in stripped:
            return False, f"{irql_var} used in expression: {stripped[:60]}"

    # Check for conditional release (release inside an if/match that acquire isn't in)
    indent_acquire = len(lines[acquire_line - 1]) - len(lines[acquire_line - 1].lstrip())
    indent_release = len(lines[release_line - 1]) - len(lines[release_line - 1].lstrip())
    if indent_release > indent_acquire + 4:
        return False, f"release is more deeply nested (indent {indent_release} vs {indent_acquire})"

    # Check span isn't too large (might indicate complex control flow)
    if release_line - acquire_line > 50:
        return False, f"span too large ({release_line - acquire_line} lines)"

    return True, ""


def fix_file(path: Path, pairs: list[LockPair]) -> int:
    """Apply guard transforms to simple pairs in a file. Returns count of transforms."""
    simple_pairs = [p for p in pairs if p.simple]
    if not simple_pairs:
        return 0

    lines = path.read_text().splitlines()
    # Process pairs in reverse order (bottom-up) to preserve line numbers
    simple_pairs.sort(key=lambda p: p.release_line, reverse=True)

    # Track unique guard counter per function to avoid name collisions
    needs_import = {"pfn": False, "spinlock": False, "irql": False}

    for guard_counter, pair in enumerate(simple_pairs):
        acquire_idx = pair.acquire_line - 1
        release_idx = pair.release_line - 1
        indent = " " * (len(lines[acquire_idx]) - len(lines[acquire_idx].lstrip()))

        # Comment out the release line (keep it visible for review)
        release_indent = " " * (len(lines[release_idx]) - len(lines[release_idx].lstrip()))
        lines[release_idx] = (
            f"{release_indent}// GUARD: {lines[release_idx].strip()}  // replaced by guard drop"
        )

        # Add guard AFTER the acquire line (keep original acquire for variable declaration)
        if pair.pattern == "pfn":
            guard_name = f"_pfn_guard_{guard_counter}"
            guard_line = (
                f"{indent}let {guard_name} = unsafe "
                "{ ntkrnl_sys::guards::PfnLockGuard::acquire() };"
            )
            needs_import["pfn"] = True
        elif pair.pattern == "spinlock":
            guard_name = f"_spin_guard_{guard_counter}"
            lock_expr = pair.lock_expr
            guard_line = (
                f"{indent}let {guard_name} = unsafe "
                f"{{ ntkrnl_sys::guards::SpinLockGuard::acquire({lock_expr}) }};"
            )
            needs_import["spinlock"] = True
        elif pair.pattern == "irql":
            guard_name = f"_irql_guard_{guard_counter}"
            level = pair.lock_expr
            guard_line = (
                f"{indent}let {guard_name} = unsafe "
                f"{{ ntkrnl_sys::guards::IrqlGuard::raise({level}) }};"
            )
            needs_import["irql"] = True

        # Comment out the acquire and add guard after it
        lines[acquire_idx] = (
            f"{indent}// GUARD: {lines[acquire_idx].strip()}"
            f"  // replaced by guard below\n{guard_line}"
        )

    # Remove marked lines
    lines = [line for line in lines if line is not None]

    path.write_text("\n".join(lines) + "\n")
    return len(simple_pairs)


def main():
    parser = argparse.ArgumentParser(description="Migrate lock patterns to RAII guards")
    parser.add_argument("files", nargs="*", help="Rust files to scan (default: all)")
    parser.add_argument("--scan", action="store_true", help="Scan and report candidates")
    parser.add_argument("--fix", action="store_true", help="Apply transforms")
    args = parser.parse_args()

    if not args.scan and not args.fix:
        args.scan = True

    reactos_root = _RUSTCONV_REACTOS_ROOT

    if args.files:
        rs_files = [Path(f) for f in args.files]
    else:
        rs_files = sorted((reactos_root / "sdk/lib/rust/ntoskrnl-rust/src").rglob("*.rs"))

    total = {"pfn": 0, "spinlock": 0, "irql": 0}
    simple = {"pfn": 0, "spinlock": 0, "irql": 0}
    complex_reasons = []
    files_fixed = 0
    total_fixed = 0

    for rs_file in rs_files:
        pairs = scan_file(rs_file)
        for pair in pairs:
            total[pair.pattern] += 1
            if pair.simple:
                simple[pair.pattern] += 1
                if args.scan:
                    print(
                        f"  [SIMPLE] {pair.file}:{pair.acquire_line}-{pair.release_line}: "
                        f"{pair.pattern} ({pair.irql_var})"
                    )
            else:
                if args.scan or args.fix:
                    print(
                        f"  [COMPLEX] {pair.file}:{pair.acquire_line}-{pair.release_line}: "
                        f"{pair.pattern} ({pair.irql_var}) — {pair.reason}"
                    )
                complex_reasons.append(pair.reason)

        if args.fix and pairs:
            count = fix_file(rs_file, pairs)
            if count > 0:
                files_fixed += 1
                total_fixed += count
                print(f"  FIXED: {rs_file} ({count} pairs)")

    print("\nSummary:")
    for pat in ["pfn", "spinlock", "irql"]:
        if total[pat] > 0:
            pct = (simple[pat] / total[pat] * 100) if total[pat] > 0 else 0
            print(f"  {pat}: {simple[pat]}/{total[pat]} simple ({pct:.0f}%)")
    total_all = sum(total.values())
    simple_all = sum(simple.values())
    pct = (simple_all / total_all * 100) if total_all > 0 else 0
    print(f"  TOTAL: {simple_all}/{total_all} simple ({pct:.0f}%)")

    if complex_reasons:
        print("\n  Complex reasons:")
        for reason, count in Counter(complex_reasons).most_common(5):
            print(f"    {count}x: {reason}")


if __name__ == "__main__":
    sys.exit(main())
