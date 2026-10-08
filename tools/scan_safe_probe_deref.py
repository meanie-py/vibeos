#!/usr/bin/env python3
"""
scan_safe_probe_deref.py — a user pointer validated by a NON-raising range probe and
then dereferenced RAW.

THE POINT. win32k and a few drivers validate a user pointer with a `SafeProbeForRead`
/ `SafeProbeForWrite` (ks.sys spells its mirror `KsSafeProbeForRead`). Unlike the
raising `ProbeForRead`/`ProbeForWrite` (see `scan_raising_probes`), these only
RANGE-CHECK: they prove the buffer lies below `MmUserProbeAddress` and is aligned, and
return a status instead of raising. They deliberately DO NOT touch the page — under
DUMMY_PSEH a touch-test would itself be an unhandled fault. So the range probe proves
WHERE the buffer is, never that it STAYS MAPPED: another thread in the caller can
unmap or free it between the probe and the access (and on SMP, concurrently). The C
wrapped the probe AND the touch in one `_SEH2_TRY`; our Rust has no SEH, so a raw
dereference after the probe is an any-process BUGCHECK 0x1E where the C returned
STATUS_ACCESS_VIOLATION.

This is the class behind `NtUserScrollDC`, `NtUserSetSysColors` and `NtGdiGetDIBits`
(roadmap): each probed a user pointer then did `rcScroll = *(prc as *const RECTL)` (or
a raw `copy_nonoverlapping` from it) outside any guard. The fix is always to capture
through a fault-tolerant primitive — win32k's `KmProtectedCopyMemory` / `safe_copy_raw`
/ `copy::{capture_from_user, publish_to_user, write_user}`, the kernel's
`ntkrnl_sys::probe::{safe_copy_raw, safe_copy_from_user, probe_read_*}` — keeping the
range probe for its validation.

WHAT IS FLAGGED. Within one function, a pointer that appears as the first argument of
a `*SafeProbeFor{Read,Write}` call and is then dereferenced raw: `*P`, `(*P).field`,
`*(P as T)`, a store `*P = ..`, or passed to a raw copy/slice primitive
(`copy_nonoverlapping`, `from_raw_parts`, `read_volatile`, `RtlCopyMemory`, ...). The
access detection, the arm-awareness (a probe in one `match`/`if` arm does not taint a
sibling arm), the `if P.is_null()` scope, and the capture-shadow rule (`let bmi =
pbmiSafe` after a safe copy ends tracking of `bmi`) are all shared with
`scan_raw_user_access` via `_walk`. A pointer handed only to a fault-tolerant
primitive yields no finding: those are not in the raw-primitive set, so `_walk`
reports them as forwarding calls, which this scanner ignores.

The kernel-mode arm of an explicit `if mode != KernelMode { .. } else { .. }` (or `if
Mode == 0 { .. }`) is skipped: a kernel caller's pointer is trusted and the C does not
probe it (ks.sys's `KsFastPropertyHandler` copies it raw on that arm by design).

WHAT IT CANNOT SEE. A probe and a deref split across two functions (probe in the
syscall, touch in a worker it calls); a validator this scanner does not know by name
(the house convention is to spell it `*SafeProbeForRead`/`*SafeProbeForWrite` precisely
so the name-based scanners recognise it); and a deref form `_walk` does not model
(a `.read()` method call, bare `f(*P)` by value). The class was driven to zero
(2026-10), so this gates at ZERO rather than against a baseline — a baseline would be
a place for the next one to hide. Suppress a reviewed site with
`// nolint: safe-probe-deref <reason>` on the line.

Usage:
    python3 tools/scan_safe_probe_deref.py            # rows: path: Fn ptr
    python3 tools/scan_safe_probe_deref.py --lines     # also print each flagged line
    python3 tools/scan_safe_probe_deref.py --lint      # terse rows; exit 1 on any
    python3 tools/scan_safe_probe_deref.py <path ...>
"""

from __future__ import annotations

import argparse
import re
import sys
from pathlib import Path

sys.path.insert(0, str(Path(__file__).resolve().parent))
from scan_double_fetch import _arg_at, _norm
from scan_raw_user_access import _walk, parse_file
from source_walk import source_files

REPO = Path(__file__).resolve().parent.parent
DEFAULT_ROOT = REPO / "sdk" / "lib" / "rust"
SKIP_DIR_PARTS = ("-sys", "/target/", "rtl-strings", "ntdll")

# A call to a non-raising range probe. The suffix match catches a crate's own
# house-spelled mirror (ks.sys's `KsSafeProbeForRead`); `(?<!fn )` skips the
# definition and the extern declaration, which are not calls.
PROBE_RE = re.compile(r"(?<!fn )\b\w*SafeProbeFor(?:Read|Write)\s*\(")
NOLINT_RE = re.compile(r"nolint:\s*(safe-probe-deref|safe_probe_deref|scan_safe_probe_deref|all)\b")


def _probe_targets(body: str) -> set[str]:
    """Bare-identifier pointers validated by a `*SafeProbeFor*` call in `body`. A
    probe whose first argument is not a plain name -- a field of an already-captured
    struct (`cap.Buffer`), an `addr_of!(..)` of a field, a complex expression -- is
    not tracked here: a field pointer belongs to the capture class (scan_raw_user_
    access's `read_back` taint), and an address expression is not itself a user
    buffer the caller dereferences by this name."""
    out: set[str] = set()
    for m in PROBE_RE.finditer(body):
        arg = _arg_at(body, m.end() - 1, 0)
        if arg is None:
            continue
        name = _norm(arg)
        if re.fullmatch(r"\w+", name):
            out.add(name)
    return out


def scan_file(path: Path, rel: str) -> list[tuple[str, int, str]]:
    """(function, line, text) for each raw access of a probed pointer."""
    rows: list[tuple[str, int, str]] = []
    for fn in parse_file(path, rel):
        body = "\n".join(fn.lines[fn.body0 : fn.end + 1])
        names = _probe_targets(body)
        if not names:
            continue
        for kind, _name, line, text in _walk(fn, names):
            # Only a raw access is a finding; a forwarding call to a fault-tolerant
            # primitive is the safe case. (A "hit" always carries the line text.)
            if kind != "hit" or not isinstance(text, str):
                continue
            if NOLINT_RE.search(fn.raw_lines[line - 1]):
                continue
            rows.append((fn.name, line, text))
    return rows


def scan_tree(files: list[tuple[Path, str]]) -> dict[str, list[tuple[int, str]]]:
    rows: dict[str, list[tuple[int, str]]] = {}
    for path, rel in files:
        try:
            if "SafeProbeFor" not in path.read_text(errors="replace"):
                continue
        except OSError:
            continue
        for fn_name, line, text in scan_file(path, rel):
            rows.setdefault(f"{rel}: {fn_name}", []).append((line, text))
    return rows


def _pick(paths: list[Path]) -> list[tuple[Path, str]]:
    files: list[Path] = []
    for p in paths or [DEFAULT_ROOT]:
        if p.is_dir():
            files.extend(sorted(source_files(p)))
        elif p.suffix == ".rs":
            files.append(p)
    picked: list[tuple[Path, str]] = []
    for f in files:
        try:
            rel = f.resolve().relative_to(REPO).as_posix()
        except ValueError:
            rel = f.as_posix()
        if any(part in rel for part in SKIP_DIR_PARTS) or "bindgen" in f.name:
            continue
        picked.append((f, rel))
    return picked


def main(argv: list[str] | None = None) -> int:
    ap = argparse.ArgumentParser(
        description=__doc__, formatter_class=argparse.RawDescriptionHelpFormatter
    )
    ap.add_argument("--lint", action="store_true", help="terse rows; exit 1 on any finding")
    ap.add_argument("--lines", action="store_true", help="print each flagged line")
    ap.add_argument("paths", nargs="*", type=Path)
    args = ap.parse_args(argv)

    rows = scan_tree(_pick(args.paths))
    for row in sorted(rows):
        print(row)
        if args.lines and not args.lint:
            for line, text in rows[row]:
                print(f"    {line}: {text[:120]}")
    if not args.lint:
        print(f"\n{len(rows)} (function, probed pointer) pairs with a raw access", file=sys.stderr)
    return 1 if (args.lint and rows) else 0


if __name__ == "__main__":
    sys.exit(main())
