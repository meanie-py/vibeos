#!/usr/bin/env python3
"""Regenerate `ks386.inc` (i386 asm constants) from kernel header offsets.

Out-of-band replacement for the build-time `generate_ks386_inc()` helper in
`sdk/lib/rust/ntoskrnl-cargo/build.rs`. Same pipeline (compile genincdata.c
→ parse `.asmdef` COFF section → emit ks386.inc text), but driven from
Python so the build doesn't need the kernel `-I` chain.

Output is checked in to `sdk/lib/rust/ntoskrnl-cargo/data/ks386-i386.inc`.
Re-run only when the templates (`ks386.template.h`, `ksx.template.h`) or
the kernel struct layouts change. Output is byte-identical to cmake's
`build-i386/sdk/include/asm/ks386.inc`.

Usage:
    python3 tools/gen_ks386_inc.py            # Regenerate + write to data/
    python3 tools/gen_ks386_inc.py --check    # Verify checked-in matches
    python3 tools/gen_ks386_inc.py --stdout   # Print to stdout (debug)
"""

from __future__ import annotations

import argparse
import shutil
import struct
import subprocess
import sys
import tempfile
from pathlib import Path

REPO = Path(__file__).resolve().parent.parent
LLVM_MINGW = REPO / ".build-tools/llvm-mingw"
CLANG = LLVM_MINGW / "bin/i686-w64-mingw32-clang"
GENINCDATA = REPO / "sdk/include/asm/genincdata.c"
OUT_FILE = REPO / "sdk/lib/rust/ntoskrnl-cargo/data/ks386-i386.inc"

# Out-of-band regen relies on cmake having already produced the hpp-
# generated kernel headers (`{ddk,psdk}/*.h`) and the windmc-generated
# `bugcodes.h`. Both live under `build-i386/sdk/include/` after `ninja
# ntoskrnl`. The cargo PoC's run-time build doesn't need these — it
# regenerates them in-process via `compile_hpp_host()` /
# `generate_bugcodes_mc()` — but this out-of-band script is happy to
# borrow cmake's copies since it's only run when regenerating
# `ks386.inc` for check-in.
BUILD_I386 = REPO / "build-i386"
# `build-i386/sdk/include/reactos/mc/` is intentionally NOT in this list —
# cmake's copy of the dir has a malformed `ntiologc.h` that shadows the
# good source-tree copy (`sdk/include/psdk/ntiologc.h`). We only need
# `bugcodes.h` from that dir; we stage just that file into a tmp dir
# inside `compile_genincdata()` and add it as a separate `-I`.
GENERATED_INCLUDES = [
    BUILD_I386 / "sdk/include/ddk",
    BUILD_I386 / "sdk/include/psdk",
]
BUGCODES_H = BUILD_I386 / "sdk/include/reactos/mc/bugcodes.h"

# Mirrors `KERNEL_DEFINES` in build.rs (only the subset that affects
# preprocessing of the templates / kernel headers).
KERNEL_DEFINES = [
    ("D3D_UMD_INTERFACE_VERSION", "0x000C"),
    ("DBG", "1"),
    ("DLL_EXPORT_VERSION", "0x502"),
    ("DXGKDDI_INTERFACE_VERSION", "0x1052"),
    ("KDBG", None),
    ("MINGW_HAS_SECURE_API", "1"),
    ("NTDDI_VERSION", "0x05020400"),
    ("NTKRNLVISTA", None),
    ("USE_COMPILER_EXCEPTIONS", None),
    ("WINVER", "0x502"),
    ("_CRT_NON_CONFORMING_SWPRINTFS", None),
    ("_CRT_SUPPRESS_RESTRICT", None),
    ("_GLIBCXX_HAVE_BROKEN_VSWPRINTF", None),
    ("_M_IX86", None),
    ("_NEW_DELETE_OPERATORS_", None),
    ("_NTOSKRNL_", None),
    ("_NTSYSTEM_", None),
    ("_SEH_ENABLE_TRACE", None),
    ("_SETUPAPI_VER", "0x502"),
    ("_USE_32BIT_TIME_T", None),
    ("_USE_DUMMY_PSEH", "1"),
    ("_WIN32_IE", "0x603"),
    ("_WIN32_WINDOWS", "0x502"),
    ("_WIN32_WINNT", "0x502"),
    ("_X86_", None),
    ("__REACTOS__", None),
    ("__ROS_ROSSYM__", None),
    ("__i386__", None),
    ("i386", None),
]

# Kernel `-I` chain genincdata.c needs (mirrors what build.rs's
# `apply_kernel_includes()` adds for the genincdata compile).
KERNEL_INCLUDES = [
    "ntoskrnl",
    "sdk/include",
    "sdk/include/crt",
    "sdk/include/ddk",
    "sdk/include/ndk",
    "sdk/include/psdk",
    "sdk/include/reactos",
    "sdk/include/vcruntime",
    "sdk/lib/pseh/include",
    "ntoskrnl/include",
]


def compile_genincdata(work: Path) -> Path:
    """Compile genincdata.c to a `.obj` with kernel-mode flags + DEFINES.

    Returns the path to the resulting object. Mirrors build.rs's `cc-rs`
    invocation so the resulting `.asmdef` section is identical."""
    obj = work / "genincdata.obj"
    # Stage just bugcodes.h into a clean dir so `<bugcodes.h>` resolves
    # without dragging in the broken `ntiologc.h` sibling cmake produces.
    mc_stage = work / "mc-stage"
    mc_stage.mkdir()
    shutil.copy2(BUGCODES_H, mc_stage / "bugcodes.h")
    cmd = [
        str(CLANG),
        "--target=i686-w64-mingw32",
        "-pipe",
        "-fms-extensions",
        "-fno-strict-aliasing",
        "-fno-common",
        "-mlong-double-64",
        "-Wno-microsoft",
        "-Wno-pragma-pack",
        "-fno-associative-math",
        "-fno-builtin-stpcpy",
        "-gdwarf-2",
        "-gstrict-dwarf",
        "-march=pentium",
        "-mtune=generic",
        "-O1",
        "-fno-optimize-sibling-calls",
        "-fno-omit-frame-pointer",
        "-mstackrealign",
        "-mxsave",
        "-Wno-unknown-warning-option",
        "-std=gnu2x",
        "-nostdinc",
        "-isystem",
        str(REPO / "sdk/include/crt"),
        "-isystem",
        str(LLVM_MINGW / "lib/clang/19/include"),
    ]
    for name, value in KERNEL_DEFINES:
        cmd.append(f"-D{name}={value}" if value is not None else f"-D{name}")
    # cmake-generated header dirs go FIRST (mirrors build.rs's xdk-gen
    # ordering — the hpp/windmc-generated headers shadow any source-tree
    # collisions).
    for inc in GENERATED_INCLUDES:
        cmd.extend(["-I", str(inc)])
    for inc in KERNEL_INCLUDES:
        cmd.extend(["-I", str(REPO / inc)])
    # mc-stage goes LAST (mirrors build.rs ordering for mc_gen_dir) so
    # source-tree headers shadow it for any name overlap.
    cmd.extend(["-I", str(mc_stage)])
    cmd.extend(["-c", "-o", str(obj), str(GENINCDATA)])

    proc = subprocess.run(cmd, capture_output=True, text=True)
    if proc.returncode != 0:
        sys.stderr.write(proc.stderr)
        raise RuntimeError(f"clang failed for {GENINCDATA} (exit {proc.returncode})")
    return obj


def read_coff_section(obj: Path, section_name: str) -> bytes:
    """Minimal PE-COFF object parser: read a named section's raw bytes.

    Mirrors `read_coff_section()` in build.rs. The COFF File Header lives at
    offset 0 (no DOS header for `.obj`)."""
    raw = obj.read_bytes()
    if len(raw) < 20:
        raise RuntimeError(f"{obj} too small to be a COFF object")
    num_sections = int.from_bytes(raw[2:4], "little")
    opt_header_size = int.from_bytes(raw[16:18], "little")
    sec_table = 20 + opt_header_size
    for i in range(num_sections):
        off = sec_table + i * 40
        nb = raw[off : off + 8]
        nul = nb.find(b"\x00")
        if nul < 0:
            nul = 8
        name = nb[:nul].decode("ascii", "replace")
        if name == section_name:
            raw_size = int.from_bytes(raw[off + 16 : off + 20], "little")
            raw_addr = int.from_bytes(raw[off + 20 : off + 24], "little")
            if raw_addr == 0 or raw_size == 0:
                raise RuntimeError(f"section {section_name!r} has no raw data")
            return raw[raw_addr : raw_addr + raw_size]
    raise RuntimeError(f"section {section_name!r} not found in {obj}")


def emit_ks386_inc(asmdef: bytes) -> str:
    """Walk ASMGENDATA records and emit ks386.inc text.

    Mirrors build.rs's loop bit-for-bit:
      type 0 = END  → break
      type 1 = RAW  → name + "\\n"
      type 2 = CONST → "name = 0xVALUE\\n"
      type 3 = HEADER → "\\n/* name */\\n"
    """
    out: list[str] = []
    for off in range(0, len(asmdef), 64):
        chunk = asmdef[off : off + 64]
        if len(chunk) < 64:
            break
        ty = chunk[0]
        nul = chunk[1:56].find(b"\x00")
        if nul < 0:
            nul = 55
        name = chunk[1 : 1 + nul].decode("utf-8")
        (value,) = struct.unpack_from("<Q", chunk, 56)
        if ty == 0:
            break
        elif ty == 1:
            out.append(f"{name}\n")
        elif ty == 2:
            out.append(f"{name} = 0x{value:x}\n")
        elif ty == 3:
            out.append(f"\n/* {name} */\n")
        else:
            raise RuntimeError(f"unknown ASMGENDATA type {ty} at offset {off}")
    return "".join(out)


def regenerate() -> str:
    if not CLANG.exists():
        raise SystemExit(
            f"missing {CLANG} — run from a checkout with .build-tools/llvm-mingw populated"
        )
    if not GENINCDATA.exists():
        raise SystemExit(f"missing {GENINCDATA}")
    for d in [*GENERATED_INCLUDES, BUGCODES_H]:
        if not d.exists():
            raise SystemExit(
                f"missing cmake-generated artifact {d}\n"
                f"  Run `cd build-i386 && ninja ntoskrnl` first."
            )
    with tempfile.TemporaryDirectory(prefix="genks386-") as tmp:
        obj = compile_genincdata(Path(tmp))
        asmdef = read_coff_section(obj, ".asmdef")
    return emit_ks386_inc(asmdef)


def main() -> int:
    ap = argparse.ArgumentParser(description=__doc__.splitlines()[0])
    ap.add_argument(
        "--check", action="store_true", help="verify checked-in file matches a fresh regen"
    )
    ap.add_argument("--stdout", action="store_true", help="print to stdout instead of writing")
    args = ap.parse_args()

    fresh = regenerate()

    if args.stdout:
        sys.stdout.write(fresh)
        return 0
    if args.check:
        if not OUT_FILE.exists():
            print(f"FAIL: {OUT_FILE} does not exist", file=sys.stderr)
            return 2
        existing = OUT_FILE.read_text()
        if existing != fresh:
            print(f"FAIL: {OUT_FILE} differs from fresh regen", file=sys.stderr)
            print(f"  expected {len(fresh)} bytes, got {len(existing)} bytes", file=sys.stderr)
            return 1
        print(f"OK: {OUT_FILE} matches fresh regen ({len(fresh)} bytes)")
        return 0

    OUT_FILE.parent.mkdir(parents=True, exist_ok=True)
    OUT_FILE.write_text(fresh)
    print(f"wrote {OUT_FILE} ({len(fresh)} bytes)")
    return 0


if __name__ == "__main__":
    sys.exit(main())
