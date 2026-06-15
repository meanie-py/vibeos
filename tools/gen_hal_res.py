#!/usr/bin/env python3
"""Regenerate `hal-i386.rc.res` (i386 PE-COFF resource section).

HAL sibling of `tools/gen_ntoskrnl_res.py`. Runs `windres` on
`hal/halx86/pic/halpic.rc` (the resource cmake links into i386
`hal.dll` via `add_hal(... SOURCES pic/halpic.rc ...)`) with the kernel
`-I` chain, out-of-band, so `hal-cargo`'s build.rs doesn't need the
version-header chain (`<reactos/version.rc>` → `<winver.h>`,
`<ntverp.h>`, `common.ver`) on every build.

Simpler than the ntoskrnl one: halpic.rc only `#include`s
`<reactos/version.rc>` — no bugcodes.mc machinery.

Output is checked in to `sdk/lib/rust/hal-cargo/data/hal-i386.rc.res`,
byte-identical to cmake's
`build-i386/hal/halx86/CMakeFiles/hal.dir/pic/halpic.rc.res`.

Usage:
    python3 tools/gen_hal_res.py            # Regenerate + write to data/
    python3 tools/gen_hal_res.py --check    # Verify checked-in matches
"""

from __future__ import annotations

import argparse
import subprocess
import sys
import tempfile
from pathlib import Path

REPO = Path(__file__).resolve().parent.parent
LLVM_MINGW = REPO / ".build-tools/llvm-mingw"
CLANG = LLVM_MINGW / "bin/i686-w64-mingw32-clang"
WINDRES = LLVM_MINGW / "bin/i686-w64-mingw32-windres"
RC_SOURCE = REPO / "hal/halx86/pic/halpic.rc"
OUT_FILE = REPO / "sdk/lib/rust/hal-cargo/data/hal-i386.rc.res"

# Mirrors the -D set cmake emits for the halpic.rc.res rule.
RC_DEFINES = [
    ("__REACTOS__", None),
    ("_CRT_NON_CONFORMING_SWPRINTFS", None),
    ("DBG", "1"),
    ("_SEH_ENABLE_TRACE", None),
    ("WINVER", "0x502"),
    ("_WIN32_IE", "0x603"),
    ("_WIN32_WINNT", "0x502"),
    ("_WIN32_WINDOWS", "0x502"),
    ("_SETUPAPI_VER", "0x502"),
    ("MINGW_HAS_SECURE_API", "1"),
    ("D3D_UMD_INTERFACE_VERSION", "0x000C"),
    ("DXGKDDI_INTERFACE_VERSION", "0x1052"),
    ("DLL_EXPORT_VERSION", "0x502"),
    ("_M_IX86", None),
    ("_X86_", None),
    ("__i386__", None),
    ("i386", None),
    ("_NEW_DELETE_OPERATORS_", None),
    ("USE_COMPILER_EXCEPTIONS", None),
    ("_USE_32BIT_TIME_T", None),
    ("_USE_DUMMY_PSEH", "1"),
    ("_GLIBCXX_HAVE_BROKEN_VSWPRINTF", None),
    ("_CRT_SUPPRESS_RESTRICT", None),
    ("_MIDL_USE_GUIDDEF_", None),
]

# halpic.rc → <reactos/version.rc> → winver.h + version.h + buildno.h.
# version.h/buildno.h are cmake-build-generated (carry the build/git
# revision); we borrow them from build-i386/ at regen time exactly like
# gen_ntoskrnl_res.py borrows bugcodes.rc. The resulting .res is checked
# in — re-run this script when the version/buildno changes.
KERNEL_INCLUDES = [
    "hal/halx86",
    "hal/halx86/include",
    "sdk/include",
    "sdk/include/crt",
    "sdk/include/psdk",
    "sdk/include/reactos",
    "sdk/include/reactos/wine",
    "sdk/include/vcruntime",
    "build-i386/sdk/include/reactos",
]


def regenerate(work: Path) -> bytes:
    out_path = work / "hal.rc.res"
    cmd = [str(WINDRES), "-O", "coff"]
    for inc in KERNEL_INCLUDES:
        cmd.extend(["-I", str(REPO / inc)])
    cmd.extend(
        [
            f"--preprocessor={CLANG}",
            "--preprocessor-arg=--target=i686-w64-mingw32",
            "--preprocessor-arg=-E",
            "--preprocessor-arg=-nostdinc",
            "--preprocessor-arg=-xc-header",
            "-DRC_INVOKED",
            "-D__WIN32__=1",
            "-D__FLAT__=1",
        ]
    )
    for name, value in RC_DEFINES:
        cmd.append(f"-D{name}={value}" if value is not None else f"-D{name}")
    cmd.extend([str(RC_SOURCE), str(out_path)])

    proc = subprocess.run(cmd, capture_output=True, text=True)
    if proc.returncode != 0:
        sys.stderr.write(proc.stderr)
        raise RuntimeError(f"windres failed (exit {proc.returncode})")
    return out_path.read_bytes()


def main() -> int:
    ap = argparse.ArgumentParser(description=__doc__.splitlines()[0])
    ap.add_argument(
        "--check", action="store_true", help="verify checked-in file matches a fresh regen"
    )
    args = ap.parse_args()

    for p in (WINDRES, CLANG, RC_SOURCE):
        if not p.exists():
            raise SystemExit(f"missing {p}")

    with tempfile.TemporaryDirectory(prefix="halres-") as tmp:
        fresh = regenerate(Path(tmp))

    if args.check:
        if not OUT_FILE.exists():
            print(f"FAIL: {OUT_FILE} does not exist", file=sys.stderr)
            return 2
        if OUT_FILE.read_bytes() != fresh:
            print(f"FAIL: {OUT_FILE} differs from fresh regen", file=sys.stderr)
            return 1
        print(f"OK: {OUT_FILE} matches fresh regen ({len(fresh)} bytes)")
        return 0

    OUT_FILE.parent.mkdir(parents=True, exist_ok=True)
    OUT_FILE.write_bytes(fresh)
    print(f"wrote {OUT_FILE} ({len(fresh)} bytes)")
    return 0


if __name__ == "__main__":
    sys.exit(main())
