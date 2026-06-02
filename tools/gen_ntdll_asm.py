#!/usr/bin/env python3
"""Pre-process ntdll's i386 asm (dispatch.S) via `clang -E` for the
cargo-driven ntdll link (`ntdll-cargo`).

`reactos-link`'s `.asm()` compiles each `.s` with NO kernel `-I` chain
(just `-nostdinc` + clang's own headers), so the macro-asm `#include`s
(`asm.inc`, the generated `ks386.inc`) must be expanded offline. Output
is checked in at `sdk/lib/rust/ntdll-cargo/data/asm/dispatch-i386.s`;
the engine assembles it via cc-rs (`-x assembler-with-cpp`). GAS
`.macro`/`.endm` blocks survive (the assembler handles those).

Unlike `gen_preprocessed_asm.py` (kernel, coupled to the ntoskrnl-cargo
OUT_DIR), this resolves `ks386.inc` from the cmake build dir directly —
no cargo PoC run needed first. Re-run when `dispatch.S`, `asm.inc`, or
`ks386.inc` changes.

Usage:
    python3 tools/gen_ntdll_asm.py
    python3 tools/gen_ntdll_asm.py --check
"""
from __future__ import annotations

import argparse
import subprocess
import sys
from pathlib import Path

REPO = Path(__file__).resolve().parent.parent
LLVM_MINGW = REPO / ".build-tools/llvm-mingw"
CLANG = LLVM_MINGW / "bin/i686-w64-mingw32-clang"
BUILD = REPO / "build-i386"

# ntdll asm define set (mirrors the cmake DEFINES for dll/ntdll asm +
# __ASM__). Values only matter for #if branches in the macro headers.
ASM_DEFINES = [
    ("__ASM__", None),
    ("DBG", "1"),
    ("NTDDI_VERSION", "0x05020400"),
    ("WINVER", "0x502"),
    ("_M_IX86", None),
    ("_NTSYSTEM_", None),
    ("_USE_DUMMY_PSEH", "1"),
    ("_WIN32_WINNT", "0x502"),
    ("_X86_", None),
    ("__REACTOS__", None),
    ("__i386__", None),
    ("i386", None),
]

SOURCES = [
    ("dll/ntdll/dispatch/i386/dispatch.S",
     "sdk/lib/rust/ntdll-cargo/data/asm/dispatch-i386.s"),
]


def build_command(source: Path) -> list[str]:
    cmd = [
        str(CLANG),
        "--target=i686-w64-mingw32",
        "-E",
        "-x", "assembler-with-cpp",
        "-nostdinc",
        "-isystem", str(REPO / "sdk/include/crt"),
        "-isystem", str(LLVM_MINGW / "lib/clang/19/include"),
    ]
    for name, value in ASM_DEFINES:
        cmd.append(f"-D{name}={value}" if value is not None else f"-D{name}")
    # asm.inc lives in the source tree; ks386.inc is generated into the
    # cmake build dir; asmmacro.S / sysfuncs.h live under ntoskrnl/include.
    cmd.extend([
        "-I", str(REPO / "sdk/include/asm"),
        "-I", str(BUILD / "sdk/include/asm"),
        "-I", str(REPO / "ntoskrnl/include"),
    ])
    cmd.append(str(source))
    return cmd


def regen_one(source: Path) -> bytes:
    proc = subprocess.run(build_command(source), capture_output=True)
    if proc.returncode != 0:
        sys.stderr.write(proc.stderr.decode("utf-8", "replace"))
        raise RuntimeError(f"clang -E failed for {source} (exit {proc.returncode})")
    out_lines = []
    for line in proc.stdout.splitlines():
        if line.startswith(b"# ") and (b"\"" in line):
            continue  # drop per-checkout line markers → deterministic file
        out_lines.append(line)
    return (b"\n".join(out_lines).lstrip(b"\n")) + b"\n"


def main() -> int:
    ap = argparse.ArgumentParser(description=__doc__.splitlines()[0])
    ap.add_argument("--check", action="store_true",
                    help="verify checked-in files match a fresh regen")
    args = ap.parse_args()
    if not CLANG.exists():
        raise SystemExit(f"missing {CLANG}")
    if not (BUILD / "sdk/include/asm/ks386.inc").exists():
        raise SystemExit(f"missing {BUILD}/sdk/include/asm/ks386.inc — run `ninja` in build-i386 first")

    failures = 0
    for src_rel, dst_rel in SOURCES:
        src = REPO / src_rel
        if not src.exists():
            raise SystemExit(f"missing source {src}")
        fresh = regen_one(src)
        dst = REPO / dst_rel
        if args.check:
            if not dst.exists() or dst.read_bytes() != fresh:
                print(f"FAIL: {dst_rel} differs from fresh regen", file=sys.stderr)
                failures += 1
            else:
                print(f"OK:   {dst_rel} ({len(fresh)} bytes)")
        else:
            dst.parent.mkdir(parents=True, exist_ok=True)
            dst.write_bytes(fresh)
            print(f"wrote {dst_rel} ({len(fresh)} bytes)")
    return 1 if failures else 0


if __name__ == "__main__":
    sys.exit(main())
