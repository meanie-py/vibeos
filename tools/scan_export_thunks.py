#!/usr/bin/env python3
"""scan_export_thunks.py — exports that are really an IMPORT THUNK of the same name.

A module that exports `X` where the code at `X` is `jmp [__imp_X]` is handing callers
someone else's `X`. Sometimes that is the design (msvcrt's `memmove` is ntdll's), but
it is also the exact shape of a LOST LOCAL DEFINITION, and when the other module
forwards `X` back the result is an endless tail-call loop: no stack growth, no fault,
the caller just spins.

WHY THIS LINT EXISTS (2026-10-04)
---------------------------------
oleaut32's delay-import library is one object holding a thunk for every oleaut32
export, including the three it forwards back to ole32 (`-import` in oleaut32.spec).
lld loaded it for `SysAllocString` long before ole32's .def exports were resolved, so
its thunks took `GetErrorInfo`/`SetErrorInfo`/`CreateErrorInfo` and the Rust
errorinfo.rs bodies were never linked. ole32 exported jumps into oleaut32, which
jumped straight back. Wine's errorinfo test hung on every sweep;
`--allow-multiple-definition` kept the link silent. A source lint cannot see this: the
Rust is fine and is even in the archive. Only the linked PE tells the truth.

`--check` fails on findings NOT in tools/lint-baseline/scan_export_thunks.txt, which
lists the deliberate re-exports.

amd64 ONLY. The detection keys on llvm-objdump naming the IAT slot (`<__imp_X>`), which
it does for amd64's `jmp *[rip+...]` and NOT for i386's absolute `jmpl *0x...` or
arm64's adrp/ldr pair, so on those arches every module would look clean. The tool
refuses them instead of saying so. The defect is link-order, not arch: ole32 had it on
all three, so one graded arch catches the class.

Usage:
    python3 tools/scan_export_thunks.py build-amd64           # report all
    python3 tools/scan_export_thunks.py --check build-amd64   # exit 1 on NEW ones
"""

from __future__ import annotations

import argparse
import re
import subprocess
import sys
from pathlib import Path

if __package__:
    from .binary_corpus import governed_binaries
    from .binary_facts import BinaryFacts
else:
    from binary_corpus import governed_binaries
    from binary_facts import BinaryFacts

_SUFFIXES = (".sys", ".dll", ".exe", ".drv", ".cpl", ".ocx")
_SKIP_DIRS = (
    "CMakeFiles",
    "host-tools",
    "/boot/stripped/",
    "/boot/staging/",
    "/boot/ramdisk/",
    "/boot/ramdisk-overlay/",
)
_EXPORT = re.compile(r"^\s+\d+\s+0x[0-9a-f]+\s+(\S+)$")
_SYM = re.compile(r"^[0-9a-f]+ <([^>]+)>:")
# x86 `jmp *[...] <__imp_X>`; arm64 `adrp x16, ...` / `ldr x16, [x16, ...]` / `br x16`
# carries the IAT name only on the ldr, so accept it within the first three lines.
_IMP = re.compile(r"<__imp_([^>+]+)>")


def objdump(build_dir: Path) -> str:
    repo = build_dir.resolve().parent
    cand = repo / ".build-tools/llvm-mingw/bin/llvm-objdump"
    return str(cand) if cand.is_file() else "llvm-objdump"


def _run(objd: str, args: list[str], pe: Path) -> str:
    try:
        return subprocess.run(
            [objd, *args, str(pe)], capture_output=True, text=True, timeout=300, check=True
        ).stdout
    except (OSError, subprocess.SubprocessError) as error:
        detail = getattr(error, "stderr", None) or str(error)
        raise RuntimeError(f"cannot read {pe}: {detail}") from error


def exports_of(headers: str) -> set[str]:
    names, inside = set(), False
    for line in headers.splitlines():
        if line.startswith("Export Table:"):
            inside = True
            continue
        if inside:
            m = _EXPORT.match(line)
            if m:
                names.add(m.group(1))
            elif line.strip() == "" and names:
                break
    return names


def self_thunks(disasm: str, exports: set[str]) -> list[str]:
    lines = disasm.split("\n")
    hits = []
    for i, line in enumerate(lines):
        m = _SYM.match(line)
        if not m:
            continue
        sym = m.group(1)
        bare = sym.lstrip("_").split("@")[0]
        if sym not in exports and bare not in exports:
            continue
        for nxt in lines[i + 1 : i + 4]:
            if _SYM.match(nxt):
                break
            imp = _IMP.search(nxt)
            if imp:
                if imp.group(1).lstrip("_").split("@")[0] == bare:
                    hits.append(bare)
                break
    return sorted(set(hits))


def scan_pe(objd: str, pe: Path) -> list[tuple[str, str]]:
    exports = exports_of(_run(objd, ["-p"], pe))
    if not exports:
        return []
    disasm = _run(objd, ["-d", "--no-show-raw-insn"], pe)
    return [(pe.name, name) for name in self_thunks(disasm, exports)]


def load_baseline(repo: Path) -> set[str]:
    f = repo / "tools" / "lint-baseline" / "scan_export_thunks.txt"
    if not f.is_file():
        return set()
    return {
        ln.strip()
        for ln in f.read_text().splitlines()
        if ln.strip() and not ln.lstrip().startswith("#")
    }


def main() -> int:
    ap = argparse.ArgumentParser(description=__doc__.split("\n")[0])
    ap.add_argument("build_dir", type=Path, help="e.g. build-amd64")
    ap.add_argument("--check", action="store_true", help="exit 1 on NEW (non-baselined) findings")
    args = ap.parse_args()

    if not args.build_dir.is_dir():
        print(f"scan_export_thunks: no such build dir: {args.build_dir}", file=sys.stderr)
        return 2

    if not args.build_dir.name.endswith("amd64"):
        print(
            f"scan_export_thunks: {args.build_dir} is not an amd64 build; the IAT-name "
            "annotation this scan reads exists only there (see the docstring)",
            file=sys.stderr,
        )
        return 2

    repo = args.build_dir.resolve().parent
    objd = objdump(args.build_dir)
    hits: list[tuple[str, str]] = []
    scanned = 0
    facts = None
    corpus = governed_binaries(args.build_dir, repo, suffixes=_SUFFIXES, staging_dirs=_SKIP_DIRS)
    for pe in corpus.paths:
        if not pe.is_file():
            continue
        try:
            if facts is None:
                facts = BinaryFacts(
                    repo, namespace="export-thunks", implementation=[Path(__file__)], tools=[objd]
                )
            hits.extend(tuple(row) for row in facts.get(pe, lambda pe=pe: scan_pe(objd, pe)))
        except RuntimeError as error:
            print(f"scan_export_thunks: {error}", file=sys.stderr)
            return 2
        scanned += 1

    if not scanned:
        print(
            f"scan_export_thunks: no PE inputs in {args.build_dir} (corpus basis: {corpus.basis})",
            file=sys.stderr,
        )
        return 2

    found = {f"{mod}: {sym}" for mod, sym in hits}
    new = sorted(found - load_baseline(repo))
    if not found:
        print(f"scan_export_thunks: {args.build_dir} — clean")
        return 0
    print(
        f"scan_export_thunks: {len(found)} export(s) that are an import thunk of themselves "
        f"({len(new)} NEW):\n"
    )
    for entry in sorted(found):
        print(f"  [{'NEW  ' if entry in new else 'known'}] {entry}")
    if not new:
        return 0
    print(
        "\nA NEW export resolves to another module's same-named import. If the module was"
        "\nmeant to define it (a Rust #[no_mangle] body, say), an import library won the"
        "\nlink: force the local one with LINKER:--require-defined=<sym> (not -u, which lld"
        "\nmaps to -includeoptional). If the re-export is deliberate, add it to the baseline."
    )
    return 1 if args.check else 0


if __name__ == "__main__":
    sys.exit(main())
