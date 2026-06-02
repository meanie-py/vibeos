#!/usr/bin/env python3
"""Pre-process ntdll's per-arch asm (dispatch.S) via `clang -E` for the
cargo-driven ntdll link (`ntdll-cargo`).

`reactos-link`'s `.asm()` compiles each `.s` with NO kernel `-I` chain
(just `-nostdinc` + clang's own headers), so the macro-asm `#include`s
(`asm.inc` + `ks386.inc`/`ksamd64.inc` on x86; none on arm64) must be
expanded offline. Outputs are checked in at
`sdk/lib/rust/ntdll-cargo/data/asm/dispatch-<arch>.s`; the engine
assembles each via cc-rs (`-x assembler-with-cpp`). GAS `.macro`/`.endm`
blocks survive (the assembler handles those).

Resolves the generated `ks*.inc` from the cmake build dir directly (no
cargo PoC run needed first). Re-run when `dispatch.S`, `asm.inc`, or
`ks*.inc` changes. Builds whichever arches have a `build-<arch>/`.

Usage:
    python3 tools/gen_ntdll_asm.py [--arch i386|amd64|arm64] [--check]
"""
from __future__ import annotations

import argparse
import subprocess
import sys
from pathlib import Path

REPO = Path(__file__).resolve().parent.parent
LLVM_MINGW = REPO / ".build-tools/llvm-mingw"

# Common (arch-agnostic) asm defines + __ASM__.
COMMON_DEFINES = [
    ("__ASM__", None), ("DBG", "1"), ("NTDDI_VERSION", "0x05020400"),
    ("WINVER", "0x502"), ("_NTSYSTEM_", None), ("_USE_DUMMY_PSEH", "1"),
    ("_WIN32_WINNT", "0x502"), ("__REACTOS__", None),
]

# Per-arch: clang triple + the arch `_M_*` defines.
ARCHES = {
    "i386":  ("i686-w64-mingw32",    [("_M_IX86", None), ("_X86_", None), ("__i386__", None), ("i386", None)]),
    "amd64": ("x86_64-w64-mingw32",  [("_M_AMD64", None), ("_AMD64_", None), ("_WIN64", None), ("__x86_64__", None)]),
    "arm64": ("aarch64-w64-mingw32", [("_M_ARM64", None), ("_ARM64_", None), ("_WIN64", None), ("__aarch64__", None)]),
}


def clang() -> Path:
    return LLVM_MINGW / "bin/i686-w64-mingw32-clang"  # any triple driver works for -E


def build_command(arch: str, source: Path) -> list[str]:
    triple, arch_defs = ARCHES[arch]
    build = REPO / f"build-{arch}"
    cmd = [
        str(LLVM_MINGW / f"bin/{triple}-clang"),
        f"--target={triple}", "-E", "-x", "assembler-with-cpp", "-nostdinc",
        "-isystem", str(REPO / "sdk/include/crt"),
        "-isystem", str(LLVM_MINGW / "lib/clang/19/include"),
    ]
    for name, value in COMMON_DEFINES + arch_defs:
        cmd.append(f"-D{name}={value}" if value is not None else f"-D{name}")
    # asm.inc (source tree) + generated ks*.inc (build dir) + asmmacro.S
    # (ntoskrnl/include). arm64 dispatch.S has no includes; the dirs are
    # harmless if unused.
    cmd += [
        "-I", str(REPO / "sdk/include/asm"),
        "-I", str(build / "sdk/include/asm"),
        "-I", str(REPO / "ntoskrnl/include"),
        str(source),
    ]
    return cmd


def regen_one(arch: str, source: Path) -> bytes:
    proc = subprocess.run(build_command(arch, source), capture_output=True)
    if proc.returncode != 0:
        sys.stderr.write(proc.stderr.decode("utf-8", "replace"))
        raise RuntimeError(f"clang -E failed for {source} (exit {proc.returncode})")
    out = [ln for ln in proc.stdout.splitlines()
           if not (ln.startswith(b"# ") and b"\"" in ln)]  # drop line markers
    return (b"\n".join(out).lstrip(b"\n")) + b"\n"


def main() -> int:
    ap = argparse.ArgumentParser(description=__doc__.splitlines()[0])
    ap.add_argument("--arch", choices=list(ARCHES), help="only this arch (default: all built)")
    ap.add_argument("--check", action="store_true", help="verify checked-in files match a fresh regen")
    args = ap.parse_args()

    arches = [args.arch] if args.arch else list(ARCHES)
    failures = 0
    did_any = False
    for arch in arches:
        src = REPO / f"dll/ntdll/dispatch/{arch}/dispatch.S"
        dst = REPO / f"sdk/lib/rust/ntdll-cargo/data/asm/dispatch-{arch}.s"
        if not src.exists():
            print(f"skip {arch}: no {src.relative_to(REPO)}")
            continue
        if not (REPO / f"build-{arch}").exists():
            print(f"skip {arch}: no build-{arch}/ (run cmake/ninja first)")
            continue
        did_any = True
        fresh = regen_one(arch, src)
        if args.check:
            if not dst.exists() or dst.read_bytes() != fresh:
                print(f"FAIL: {dst.relative_to(REPO)} differs from fresh regen", file=sys.stderr)
                failures += 1
            else:
                print(f"OK:   {dst.relative_to(REPO)} ({len(fresh)} bytes)")
        else:
            dst.parent.mkdir(parents=True, exist_ok=True)
            dst.write_bytes(fresh)
            print(f"wrote {dst.relative_to(REPO)} ({len(fresh)} bytes)")
    if not did_any:
        print("nothing to do — no build-<arch>/ present", file=sys.stderr)
    return 1 if failures else 0


if __name__ == "__main__":
    sys.exit(main())
