#!/usr/bin/env python3
"""scan_xchg_ebx_deref.py — i386 code that swaps ebx out and then dereferences ebx.

`cmpxchg8b` takes its new low dword in ebx, and inline asm that cannot name ebx as an
operand swaps it in: `xchg {tmp}, ebx` / `lock cmpxchg8b [{addr}]` / `xchg {tmp}, ebx`.
On x86 ebx is an ordinary allocatable register (only x86_64 reserves rbx), so LLVM is
free to put `{addr}` IN ebx -- and then the first xchg replaces the pointer with the
value, and the instruction dereferences the VALUE.

WHY THIS LINT EXISTS (2026-10-07)
---------------------------------
The slist crate's i386 `try_store` did exactly that in ntdll.dll and ks.sys:
`xchg %ebx,%edi; lock cmpxchg8b (%ebx)` compared against the entry being pushed instead
of the list head. `InterlockedPushEntrySList` "succeeded" into the entry (list
unchanged) when entry+4 happened to match the header's high dword and spun forever
otherwise -- Wine's ucrtbase cpp test hung in `__std_type_info_name`. Nothing in the
source is wrong on its face; only the register allocation is, so only the linked code
can show it.

The finding: after `xchg %ebx, %R` (either operand order), a memory operand based on
%ebx before ebx is swapped back. A deliberate swap-then-deref of ebx would be a strange
thing to write by hand, so there is no baseline: every hit fails --check.

i386 ONLY: the pattern is specific to cmpxchg8b-style ebx juggling on a target where
ebx is allocatable.

Usage:
    python3 tools/scan_xchg_ebx_deref.py build-i386           # report
    python3 tools/scan_xchg_ebx_deref.py --check build-i386   # exit 1 on any finding
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
_SYM = re.compile(r"^[0-9a-f]+ <([^>]+)>:")
_INSN = re.compile(r"^\s*([0-9a-f]+):\s+(.*)$")
# Not eax: cmpxchg8b and cpuid both fix eax, so a real swap never pairs it with ebx, and
# its one-byte encoding (0x93) is what jump-table data in .text decodes as -- 21 false
# hits across user32/ntoskrnl/pci/cdrom before this exclusion.
_XCHG_EBX = re.compile(r"^xchgl\s+(%ebx,\s*%e(?!ax)\w+|%e(?!ax)\w+,\s*%ebx)\s*$")
_EBX_BASE = re.compile(r"\(%ebx[,)]")
# How far past the swap to look. The idiom is swap / op / swap-back.
_WINDOW = 4


def objdump(build_dir: Path) -> str:
    repo = build_dir.resolve().parent
    cand = repo / ".build-tools/llvm-mingw/bin/llvm-objdump"
    return str(cand) if cand.is_file() else "llvm-objdump"


def findings(disasm: str) -> list[tuple[str, str]]:
    """(function, address) of every ebx dereference inside an ebx swap window."""
    hits: list[tuple[str, str]] = []
    func = "?"
    window = 0
    for line in disasm.splitlines():
        m = _SYM.match(line)
        if m:
            func, window = m.group(1), 0
            continue
        m = _INSN.match(line)
        if not m:
            continue
        addr, insn = m.group(1), m.group(2).split("#")[0].strip()
        if _XCHG_EBX.match(insn):
            # The swap-back closes the window; a fresh swap opens one.
            window = 0 if window else _WINDOW
            continue
        if window:
            if _EBX_BASE.search(insn):
                hits.append((func, addr))
            window -= 1
    return hits


def scan_pe(objd: str, pe: Path) -> list[tuple[str, str, str]]:
    try:
        disasm = subprocess.run(
            [objd, "-d", "--no-show-raw-insn", str(pe)],
            capture_output=True,
            text=True,
            timeout=300,
            check=True,
        ).stdout
    except (OSError, subprocess.SubprocessError) as error:
        detail = getattr(error, "stderr", None) or str(error)
        raise RuntimeError(f"cannot read {pe}: {detail}") from error
    return [(pe.name, func, addr) for func, addr in findings(disasm)]


def main() -> int:
    ap = argparse.ArgumentParser(description=__doc__.split("\n")[0])
    ap.add_argument("build_dir", type=Path, help="e.g. build-i386")
    ap.add_argument("--check", action="store_true", help="exit 1 on any finding")
    args = ap.parse_args()

    if not args.build_dir.is_dir():
        print(f"scan_xchg_ebx_deref: no such build dir: {args.build_dir}", file=sys.stderr)
        return 2
    if not args.build_dir.name.endswith("i386"):
        print(
            f"scan_xchg_ebx_deref: {args.build_dir} is not an i386 build; the defect needs "
            "an allocatable ebx (see the docstring)",
            file=sys.stderr,
        )
        return 2

    repo = args.build_dir.resolve().parent
    objd = objdump(args.build_dir)
    hits: list[tuple[str, str, str]] = []
    scanned = 0
    facts = None
    corpus = governed_binaries(args.build_dir, repo, suffixes=_SUFFIXES, staging_dirs=_SKIP_DIRS)
    for pe in corpus.paths:
        if not pe.is_file():
            continue
        try:
            if facts is None:
                facts = BinaryFacts(
                    repo, namespace="xchg-ebx-deref", implementation=[Path(__file__)], tools=[objd]
                )
            hits.extend(tuple(row) for row in facts.get(pe, lambda pe=pe: scan_pe(objd, pe)))
        except RuntimeError as error:
            print(f"scan_xchg_ebx_deref: {error}", file=sys.stderr)
            return 2
        scanned += 1

    if not scanned:
        print(
            f"scan_xchg_ebx_deref: no PE inputs in {args.build_dir} (corpus basis: {corpus.basis})",
            file=sys.stderr,
        )
        return 2
    if not hits:
        print(f"scan_xchg_ebx_deref: {args.build_dir} — clean ({scanned} binaries)")
        return 0
    print(f"scan_xchg_ebx_deref: {len(hits)} ebx dereference(s) inside an ebx swap:\n")
    for mod, func, addr in sorted(set(hits)):
        print(f"  {mod}: {func} @ {addr}")
    print(
        "\nThe instruction dereferences ebx after ebx was swapped out, so it uses the"
        "\nswapped-in VALUE as the address. In Rust inline asm this means LLVM allocated"
        '\nthe pointer operand to ebx: pin it to a fixed register (in("edi") addr) and'
        "\nname that register in the template."
    )
    return 1 if args.check else 0


if __name__ == "__main__":
    sys.exit(main())
