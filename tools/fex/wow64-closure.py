#!/usr/bin/env python3
"""Compute the transitive import closure of an i386 PE, for staging into SysWOW64.

Why this exists
---------------
A WoW64 guest's DLLs are loaded by OUR loader (ntdll-rust ldrpe.rs). When a DLL
it needs is absent, `LdrpLoadImportModule` fails and the loader prints

    LDR: Skipping missing import <dll> for <module> (status c0000135)

and then leaves that module's IAT slots UNBOUND. The image still loads, so the
failure is silent until the guest actually CALLS through one of those slots —
at which point it jumps to a file-relative RVA and dies somewhere unrelated.
(This killed gui32 with a guest AV @0x76AA4 on 2026-07-11, which is why the
"stage only user32+gdi32 and let the rest go missing" strategy was adopted: an
import you never load can't leave a half-bound module behind.)

That strategy stops working the moment a guest is linked against a real CRT:
msvcrt drags in advapi32 -> rpcrt4/secur32 -> ... and the guest DOES call into
them. The fix is not "stage more" — a bigger PARTIAL set has exactly the same
hazard, just later. The fix is to stage the COMPLETE closure, so no module is
ever left half-bound. This tool computes that closure, so it can't rot when an
import edge changes.

Output: one "<name><TAB><path>" line per DLL, for mkramdisk.sh to mcopy.
Exits non-zero (loudly) if any import cannot be resolved to a built file —
a silently-omitted DLL is precisely the landmine we're defusing.
"""

import argparse
import os
import re
import subprocess
import sys
from pathlib import Path

# PE extensions ReactOS actually ships as importable modules. Missing one here
# means a phantom "not built" report (winspool.drv is a .drv, not a .dll).
PE_SUFFIXES = (".dll", ".drv", ".ocx", ".acm", ".cpl")

# Present in SysWOW64 already, staged explicitly by mkramdisk.sh.
PRESTAGED = {"ntdll.dll", "kernel32.dll", "user32.dll", "gdi32.dll", "ntdll_vista.dll"}


def find_objdump(project_dir: Path) -> str:
    bundled = project_dir / ".build-tools/llvm-mingw/bin/llvm-objdump"
    if bundled.exists():
        return str(bundled)
    from shutil import which

    found = which("llvm-objdump") or which("objdump")
    if not found:
        sys.exit("wow64-closure: no llvm-objdump found (need .build-tools/llvm-mingw)")
    return found


def index_build(build_dir: Path) -> dict:
    """Map lowercase module name -> path, preferring the stripped variant.

    The build tree carries both a fat debug copy and a stripped copy
    (boot/stripped/<hash>-<name>). The stripped set is what the livecd ships and
    is ~2x smaller in aggregate; the ramdisk has finite room, so prefer it.
    """
    stripped = {}
    stripped_dir = build_dir / "boot/stripped"
    if stripped_dir.is_dir():
        for p in stripped_dir.iterdir():
            if p.suffix.lower() in PE_SUFFIXES and "-" in p.name:
                stripped[p.name.split("-", 1)[1].lower()] = p

    index = {}
    for p in build_dir.rglob("*"):
        if p.suffix.lower() not in PE_SUFFIXES:
            continue
        if "CMakeFiles" in p.parts or "stripped" in p.parts:
            continue
        index.setdefault(p.name.lower(), p)

    # Overlay the stripped copies over the fat ones.
    for name, p in stripped.items():
        if name in index:
            index[name] = p
    return index


def imports_of(objdump: str, pe: Path) -> list:
    out = subprocess.run(
        [objdump, "-p", str(pe)], capture_output=True, text=True
    ).stdout
    return [
        m.group(1).lower()
        for m in (re.match(r"\s*DLL Name:\s*(\S+)", ln) for ln in out.splitlines())
        if m
    ]


def main() -> int:
    ap = argparse.ArgumentParser()
    ap.add_argument("roots", nargs="+", help="root PE(s) whose closure to compute")
    ap.add_argument("--build-dir", default="build-i386")
    ap.add_argument(
        "--include-prestaged",
        action="store_true",
        help="also emit ntdll/kernel32/user32/gdi32 (normally staged by name)",
    )
    args = ap.parse_args()

    project_dir = Path(__file__).resolve().parent.parent.parent
    build_dir = Path(args.build_dir)
    if not build_dir.is_absolute():
        build_dir = project_dir / build_dir
    if not build_dir.is_dir():
        sys.exit(f"wow64-closure: no such build dir: {build_dir}")

    objdump = find_objdump(project_dir)
    index = index_build(build_dir)

    seen, missing = set(), {}
    out = {}
    stack = []
    for r in args.roots:
        p = Path(r)
        if not p.is_absolute():
            p = build_dir / r
        if not p.exists():
            sys.exit(f"wow64-closure: root not found: {p}")
        stack.append(p)

    while stack:
        cur = stack.pop()
        for dep in imports_of(objdump, cur):
            if dep in seen:
                continue
            seen.add(dep)
            if dep in index:
                if dep not in PRESTAGED or args.include_prestaged:
                    out[dep] = index[dep]
                stack.append(index[dep])
            else:
                missing.setdefault(dep, []).append(cur.name)

    if missing:
        # FAIL LOUD: a DLL we can't resolve is a module that will load half-bound
        # and crash at first call, far from here. Never stage a partial closure.
        for dep, importers in sorted(missing.items()):
            print(
                f"wow64-closure: UNRESOLVED import {dep} "
                f"(wanted by {', '.join(sorted(set(importers)))})",
                file=sys.stderr,
            )
        print(
            "wow64-closure: refusing to emit a PARTIAL closure — a half-bound module "
            "crashes at first call, not at load. Build the missing module(s) first.",
            file=sys.stderr,
        )
        return 1

    for name in sorted(out):
        print(f"{name}\t{out[name]}")
    total = sum(p.stat().st_size for p in out.values())
    print(
        f"wow64-closure: {len(out)} module(s), {total / 1e6:.1f} MB",
        file=sys.stderr,
    )
    return 0


if __name__ == "__main__":
    sys.exit(main())
