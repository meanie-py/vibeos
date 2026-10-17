#!/usr/bin/env python3
"""A function the C declares through a STDCALL function class, defined `extern "C"` in Rust.

    python3 tools/scan_fnclass_cc.py [paths]   # report
    python3 tools/scan_fnclass_cc.py --lint    # terse, for the baseline gate

The C spells a callback's convention through a function-class typedef:

    typedef NTSTATUS (NTAPI POWER_SETTING_CALLBACK)(LPCGUID, PVOID, ULONG, PVOID);
    POWER_SETTING_CALLBACK DevicePowerSettingCallback;          // cdrom.h

The prototype `DevicePowerSettingCallback(...)` that follows carries no NTAPI of its own, so
a translation that reads only the definition writes `extern "C"`, which on i386 is cdecl.
The framework then calls it stdcall: the callee does not pop its arguments, and the caller's
stack is off by their size after every call. On amd64 and arm64 the two are the same ABI,
so nothing there can notice.

The usual oracles are blind to this. bindgen emits the island declaration of a function
declared this way as `extern "C"` (cdrom's i386 island, 2026-10-17), so scan_cc_mismatch,
which compares definitions against island declarations, sees agreement. cdrom had three:
two WDF work items (`EVT_WDF_WORKITEM`) and the power-setting callback. classpnp had an
`EXT_CALLBACK`.

The classes are every `typedef R (NTAPI X)(..)` / `typedef R NTAPI X(..)` in the headers
(NTAPI, STDCALL, WINAPI, CALLBACK, __stdcall), plus every kmdf `EVT_*` whose `PFN_` type is
`extern "system"` in kmdf-sys's i386 island. A declaration `X Name;` in a C file of the same
module (c_twin.same_module) pairs it with the Rust definition of `Name`.

Suppress a reviewed site with `// nolint: fnclass-cc` on the Rust fn's line. Exit 1 if any
finding.
"""

from __future__ import annotations

import re
import subprocess
import sys
from functools import cache
from pathlib import Path

REPO = Path(__file__).resolve().parent.parent
sys.path.insert(0, str(REPO))
from tools.c_twin import same_module  # noqa: E402
from tools.source_walk import source_files  # noqa: E402

RUST_ROOT = REPO / "sdk" / "lib" / "rust"
NOLINT = "nolint: fnclass-cc"
_CC = r"(?:NTAPI|STDCALL|WINAPI|CALLBACK|APIENTRY|__stdcall|_stdcall)"
TYPEDEF = re.compile(
    rf"typedef\s+[\w\s\*]+?\(\s*{_CC}\s+(\w+)\s*\)\s*\(|typedef\s+[\w\s\*]+?\s{_CC}\s+(\w+)\s*\(",
    re.S,
)
KMDF_PFN = re.compile(
    r'pub type (?:PFN_|EVT_)(\w+) = ::core::option::Option<unsafe extern "system"'
)
DEF = re.compile(r'^\s*pub\s+(?:unsafe\s+)?extern\s+"C"\s+fn\s+(\w+)\s*\(', re.M)


def _git_files(*patterns: str) -> list[Path]:
    proc = subprocess.run(
        ["git", "-C", str(REPO), "ls-files", *patterns, ":!sdk/lib/rust", ":!modules/rostests"],
        capture_output=True,
        text=True,
        check=False,
    )
    if proc.returncode:
        raise SystemExit(f"scan_fnclass_cc: git ls-files failed: {proc.stderr.strip()}")
    return [REPO / p for p in proc.stdout.split()]


@cache
def stdcall_classes() -> frozenset[str]:
    out: set[str] = set()
    for h in _git_files("*.h"):
        try:
            text = h.read_text(errors="replace")
        except OSError:
            continue
        if "typedef" not in text:
            continue
        out.update(a or b for a, b in TYPEDEF.findall(text))
    island = RUST_ROOT / "kmdf-sys" / "src" / "types_i386.rs"
    if island.is_file():
        out.update("EVT_" + n for n in KMDF_PFN.findall(island.read_text(errors="replace")))
    return frozenset(out)


class _CPath:
    """What c_twin.same_module reads off a Func: its path."""

    def __init__(self, path: Path):
        self.path = path


@cache
def classed_declarations() -> dict[str, list[tuple[str, Path]]]:
    """{function: [(class, C file)]} for every `CLASS Name;` declaration in C sources."""
    classes = stdcall_classes()
    decl = re.compile(r"^[ \t]*(\w+)[ \t]+(\w+)[ \t]*;", re.M)
    out: dict[str, list[tuple[str, Path]]] = {}
    for f in _git_files("*.c", "*.h", "*.cpp"):
        try:
            text = f.read_text(errors="replace")
        except OSError:
            continue
        for m in decl.finditer(text):
            if m.group(1) in classes:
                out.setdefault(m.group(2), []).append((m.group(1), f))
    return out


def scan(roots: list[Path]) -> list[tuple[str, int, str, str, str]]:
    """(rust path, line, fn, class, C file) for each `extern "C"` def of a classed fn."""
    decls = classed_declarations()
    findings = []
    for f in (f for r in roots for f in ([r] if r.is_file() else source_files(r))):
        if f.suffix != ".rs" or re.search(r"(?:bindgen|types)_\w+\.rs$", f.name):
            continue
        text = f.read_text(errors="replace")
        if 'extern "C"' not in text:
            continue
        lines = text.split("\n")
        for m in DEF.finditer(text):
            name = m.group(1)
            line = text.count("\n", 0, m.start(1)) + 1
            if name not in decls or NOLINT in lines[line - 1]:
                continue
            twins = [(c, p) for c, p in decls[name] if same_module(f, _CPath(p))]
            if not twins:
                continue
            try:
                rel = f.relative_to(REPO).as_posix()
            except ValueError:
                rel = f.as_posix()
            cls, cpath = twins[0]
            findings.append((rel, line, name, cls, cpath.relative_to(REPO).as_posix()))
    return sorted(findings)


def main() -> int:
    lint = "--lint" in sys.argv
    args = [Path(a).resolve() for a in sys.argv[1:] if not a.startswith("--")]
    findings = scan(args or [RUST_ROOT])
    for rel, line, name, cls, cpath in findings:
        if lint:
            print(f"{rel}: {name} [FNCLASS_CC]")
        else:
            print(
                f'{rel}:{line}: {name} is `extern "C"`, but {cpath} declares it `{cls} {name};` '
                f'and {cls} is stdcall -- define it `extern "system"`'
            )
    return 1 if findings else 0


if __name__ == "__main__":
    sys.exit(main())
