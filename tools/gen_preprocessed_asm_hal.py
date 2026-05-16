#!/usr/bin/env python3
"""Pre-process the 4 i386 HAL asm files via `clang -E`.

HAL sibling of `tools/gen_preprocessed_asm.py`. The HAL `.S` files
(`generic/{v86,trap,systimer}.S`, `pic/pic.S`) `#include <asm.inc>`,
`<ks386.inc>` and `<internal/i386/asmmacro.S>` — the same asmpp/geninc
chain ntoskrnl's asm uses. We expand every `#include`/`#define`
out-of-band so the resulting `.s` is self-contained GAS (GAS `.macro`
/ `.code16` etc. survive — clang's integrated assembler handles them,
exactly as cmake's `clang -x assembler-with-cpp` does today).

Output is checked in to `sdk/lib/rust/hal-cargo/data/asm/hal-*-i386.s`
and consumed by `hal-cargo`'s build.rs via cc-rs (no kernel `-I`
chain). `ks386.inc` is reused from the checked-in
`sdk/lib/rust/ntoskrnl-cargo/data/ks386-i386.inc` (same kernel struct
offsets — keep the two in sync via `tools/gen_ks386_inc.py`).

Out-of-band — re-run when a HAL asm source, `asm.inc`, `asmmacro.S`,
or the kernel struct layout (`ks386.inc`) changes. `--check`
validates the checked-in copies against a fresh regen.

Usage:
    python3 tools/gen_preprocessed_asm_hal.py
    python3 tools/gen_preprocessed_asm_hal.py --check
"""
from __future__ import annotations

import argparse
import shutil
import subprocess
import sys
import tempfile
from pathlib import Path

REPO = Path(__file__).resolve().parent.parent
LLVM_MINGW = REPO / ".build-tools/llvm-mingw"
CLANG = LLVM_MINGW / "bin/i686-w64-mingw32-clang"
KS386_INC = REPO / "sdk/lib/rust/ntoskrnl-cargo/data/ks386-i386.inc"

# Exactly the -D set cmake emits for hal/halx86/generic/v86.S.obj
# (HAL flavor: _NTHALDLL_/_NTHAL_ instead of ntoskrnl's
# _NTOSKRNL_/_NTSYSTEM_; no KDBG/NTDDI_VERSION).
ASM_DEFINES = [
    ("D3D_UMD_INTERFACE_VERSION", "0x000C"),
    ("DBG", "1"),
    ("DLL_EXPORT_VERSION", "0x502"),
    ("DXGKDDI_INTERFACE_VERSION", "0x1052"),
    ("MINGW_HAS_SECURE_API", "1"),
    ("USE_COMPILER_EXCEPTIONS", None),
    ("WINVER", "0x502"),
    ("_CRT_NON_CONFORMING_SWPRINTFS", None),
    ("_CRT_SUPPRESS_RESTRICT", None),
    ("_GLIBCXX_HAVE_BROKEN_VSWPRINTF", None),
    ("_M_IX86", None),
    ("_NEW_DELETE_OPERATORS_", None),
    ("_NTHALDLL_", None),
    ("_NTHAL_", None),
    ("_SEH_ENABLE_TRACE", None),
    ("_SETUPAPI_VER", "0x502"),
    ("_USE_32BIT_TIME_T", None),
    ("_USE_DUMMY_PSEH", "1"),
    ("_WIN32_IE", "0x603"),
    ("_WIN32_WINDOWS", "0x502"),
    ("_WIN32_WINNT", "0x502"),
    ("_X86_", None),
    ("__REACTOS__", None),
    ("__i386__", None),
    ("i386", None),
    ("__ASM__", None),
]

SOURCES = [
    ("hal/halx86/generic/v86.S",      "sdk/lib/rust/hal-cargo/data/asm/hal-v86-i386.s"),
    ("hal/halx86/generic/trap.S",     "sdk/lib/rust/hal-cargo/data/asm/hal-trap-i386.s"),
    ("hal/halx86/generic/systimer.S", "sdk/lib/rust/hal-cargo/data/asm/hal-systimer-i386.s"),
    ("hal/halx86/pic/pic.S",          "sdk/lib/rust/hal-cargo/data/asm/hal-pic-i386.s"),
]


def build_command(source: Path, ks386_dir: Path) -> list[str]:
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
    # Mirrors the -I chain cmake uses for the HAL .S.obj rule, with the
    # generated-ks386 dir (build-i386/sdk/include/asm) replaced by the
    # checked-in copy staged as <ks386.inc>.
    cmd.extend([
        "-I", str(REPO / "sdk/include/asm"),
        "-I", str(ks386_dir),
        "-I", str(REPO / "sdk/include"),
        "-I", str(REPO / "sdk/include/crt"),
        "-I", str(REPO / "sdk/include/ddk"),
        "-I", str(REPO / "sdk/include/ndk"),
        "-I", str(REPO / "sdk/include/psdk"),
        "-I", str(REPO / "sdk/include/reactos"),
        "-I", str(REPO / "sdk/include/reactos/libs"),
        "-I", str(REPO / "sdk/include/vcruntime"),
        "-I", str(REPO / "hal/halx86/include"),
        "-I", str(REPO / "ntoskrnl/include"),
        "-I", str(REPO / "sdk/lib/pseh/include"),
    ])
    cmd.append(str(source))
    return cmd


def regen_one(source: Path, ks386_dir: Path) -> bytes:
    proc = subprocess.run(build_command(source, ks386_dir), capture_output=True)
    if proc.returncode != 0:
        sys.stderr.write(proc.stderr.decode("utf-8", "replace"))
        raise RuntimeError(f"clang -E failed for {source} (exit {proc.returncode})")
    out_lines = []
    for line in proc.stdout.splitlines():
        # Drop cpp line markers (`# 1 "..."`) — they carry checkout-local
        # paths and would make the checked-in file non-deterministic.
        if line.startswith(b"# ") and (b"\"" in line):
            continue
        out_lines.append(line)
    return b"\n".join(out_lines).lstrip(b"\n") + b"\n"


def main() -> int:
    ap = argparse.ArgumentParser(description=__doc__.splitlines()[0])
    ap.add_argument("--check", action="store_true",
                    help="verify checked-in files match a fresh regen")
    args = ap.parse_args()

    for p in (CLANG, KS386_INC):
        if not p.exists():
            raise SystemExit(
                f"missing {p}\n  (ks386 inc: regenerate via "
                f"`python3 tools/gen_ks386_inc.py`)"
            )

    failures = 0
    with tempfile.TemporaryDirectory(prefix="halasm-") as tmp:
        ks386_dir = Path(tmp)
        shutil.copy2(KS386_INC, ks386_dir / "ks386.inc")
        results = []
        for src_rel, dst_rel in SOURCES:
            src = REPO / src_rel
            if not src.exists():
                raise SystemExit(f"missing source {src}")
            results.append((dst_rel, regen_one(src, ks386_dir)))

    for dst_rel, fresh in results:
        dst = REPO / dst_rel
        if args.check:
            if not dst.exists():
                print(f"FAIL: {dst_rel} does not exist", file=sys.stderr)
                failures += 1
                continue
            if dst.read_bytes() != fresh:
                print(f"FAIL: {dst_rel} differs from fresh regen", file=sys.stderr)
                failures += 1
                continue
            print(f"OK:   {dst_rel} ({len(fresh)} bytes)")
        else:
            dst.parent.mkdir(parents=True, exist_ok=True)
            dst.write_bytes(fresh)
            print(f"wrote {dst_rel} ({len(fresh)} bytes)")
    return 1 if failures else 0


if __name__ == "__main__":
    sys.exit(main())
