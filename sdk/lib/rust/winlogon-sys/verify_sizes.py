#!/usr/bin/env python3
"""Compile-time struct-size assertions for ntdll-sys.

Cross-compiles `sizeof(<typedef>)` for every bindgen struct to LLVM IR
via the project clang, then emits Rust `const _: () = assert!(...)`
that fail to compile if a bindgen struct ever drifts from the C layout
(the FFI-correctness guard required before translation begins).

Mirrors ntkrnl-sys/verify_sizes.py but: (1) lifts DEFINES/INCLUDES/
FLAGS from build-<arch>/build.ninja's csrsrv rule + the extra -I dirs,
exactly like generate_bindings.sh (no rot-prone hardcoded include
list / toolchain path); (2) #include "wrapper.h" so the C TU sees the
identical type set bindgen saw; (3) one module (`nt`), no hand-written
types.

Usage:
    [VIBEOS_BUILD_ROOT=/path] python3 verify_sizes.py          # generate
    [VIBEOS_BUILD_ROOT=/path] python3 verify_sizes.py --check   # verify
"""

import os
import re
import subprocess
import sys
import tempfile
from pathlib import Path

SCRIPT_DIR = Path(__file__).parent
SRC_DIR = SCRIPT_DIR / "src"
SRC = (SCRIPT_DIR / "../../../..").resolve()
BUILD_ROOT = Path(os.environ.get("VIBEOS_BUILD_ROOT", str(SRC)))
# .build-tools/ (like build-*/) is gitignored — only in the main
# checkout, not a fresh worktree. Resolve the toolchain from BUILD_ROOT.
RES = BUILD_ROOT / ".build-tools/llvm-mingw/lib/clang/19/include"
CLANG = BUILD_ROOT / ".build-tools/llvm-mingw/bin/clang"
WRAPPER = SCRIPT_DIR / "wrapper.h"
# A TU still COMPILED as C (the module's own .c went to Rust when it was flipped, and a
# rule that matches nothing used to leave clang with no -I/-D and exit 0 having verified nothing).
NINJA_RULE = "base/system/winlogon/CMakeFiles/winlogon_c.dir/winlogon_rust_stub.c.obj:"

ARCHS = [
    {"name": "i386",  "target": "i686-pc-windows-gnu",    "out": "size_check_i386.rs",  "rs": "bindgen_i386.rs"},
    {"name": "amd64", "target": "x86_64-pc-windows-gnu",   "out": "size_check_amd64.rs", "rs": "bindgen_amd64.rs"},
    {"name": "arm64", "target": "aarch64-pc-windows-gnu",  "out": "size_check_arm64.rs", "rs": "bindgen_arm64.rs"},
]

SKIP_TYPES = {
    "_GENERAL_LOOKASIDE_POOL",  # complex union, bindgen disagrees w/ C
    "_LONGDOUBLE",              # x87 long double, platform padding
}


def lift_ninja_flags(arch_name: str) -> tuple[list[str], bool]:
    """DEFINES+INCLUDES+sanitized FLAGS from build-<arch>/build.ninja."""
    ninja = BUILD_ROOT / f"build-{arch_name}" / "build.ninja"
    if not ninja.exists():
        return [], False
    text = ninja.read_text().splitlines()
    grabbed = {"DEFINES": "", "INCLUDES": "", "FLAGS": ""}
    for i, line in enumerate(text):
        if NINJA_RULE in line:
            for l in text[i:i + 9]:
                for key in grabbed:
                    if l.strip().startswith(key + " =") and not grabbed[key]:
                        grabbed[key] = l.split("=", 1)[1].strip()
            break
    if not grabbed["DEFINES"] or not grabbed["INCLUDES"]:
        sys.exit(f"!! {arch_name}: no DEFINES/INCLUDES for {NINJA_RULE} in {ninja}")
    flags: list[str] = []
    flags += grabbed["DEFINES"].split()
    flags += grabbed["INCLUDES"].split()
    for f in grabbed["FLAGS"].split():
        if re.match(r"^-(g|O|Winvalid-pch|ffile-prefix-map|include-pch|fno-omit|"
                    r"mstackrealign|fno-optimize|march|mtune|Xclang)", f):
            continue
        if f.endswith(".pch") or "cmake_pch" in f or f in ("-include",):
            continue
        flags.append(f)
    flags += [
        f"-I{SRC}/base/system/winlogon",
        "-isystem", str(RES),
    ]
    return flags, True


def extract_types(rs_path: Path) -> list[tuple[str, str]]:
    """(struct_name, shortest non-pointer typedef alias) from bindgen."""
    content = rs_path.read_text()
    structs = set()
    for m in re.finditer(r"^pub struct (\w+)", content, re.M):
        n = m.group(1)
        if n.startswith(("__Bindgen", "__Incomplete")) or "__bindgen_ty_" in n:
            continue
        if n in SKIP_TYPES:
            continue
        structs.add(n)
    alias: dict[str, str] = {}
    for m in re.finditer(r"^pub type (\w+)\s*=\s*(\w+)\s*;", content, re.M):
        a, tgt = m.group(1), m.group(2)
        if tgt not in structs or f"pub type {a} = *" in content:
            continue
        if tgt not in alias or len(a) < len(alias[tgt]):
            alias[tgt] = a
    return [(n, alias.get(n, n)) for n in sorted(structs)]


def gen_c(types: list[tuple[str, str]]) -> str:
    out = [f'#include "{WRAPPER}"', ""]
    for i, (rn, ce) in enumerate(types):
        out.append(f"volatile unsigned int size_{i} = sizeof({ce}); /* {rn} */")
    return "\n".join(out) + "\n"


def compile_ir(c_src: str, arch: dict, tmp: Path) -> str | None:
    cpath = tmp / f"sz_{arch['name']}.c"
    lpath = tmp / f"sz_{arch['name']}.ll"
    cpath.write_text(c_src)
    flags, ok = lift_ninja_flags(arch["name"])
    if not ok:
        print(f"  SKIP — build-{arch['name']}/build.ninja missing "
              f"(set VIBEOS_BUILD_ROOT)")
        return None
    for attempt in range(20):
        cmd = [str(CLANG), "-emit-llvm", "-S", "-O0", "-std=gnu89",
               "-target", arch["target"], "-ferror-limit=0", "-Wno-everything",
               *flags, str(cpath), "-o", str(lpath)]
        r = subprocess.run(cmd, capture_output=True, text=True)
        if r.returncode == 0:
            return lpath.read_text()
        errs = {int(m.group(1)) for line in r.stderr.split("\n")
                if (m := re.search(rf"{re.escape(cpath.name)}:(\d+):\d+: error:", line))}
        if not errs:
            print(f"  ERROR: unrecoverable for {arch['name']}")
            for l in r.stderr.split("\n")[:6]:
                print(f"    {l}")
            return None
        sl = cpath.read_text().split("\n")
        for ln in sorted(errs, reverse=True):
            if 0 <= ln - 1 < len(sl) and sl[ln - 1].startswith("volatile"):
                sl[ln - 1] = ""
        cpath.write_text("\n".join(sl))
    print("  ERROR: failed after max retries")
    return None


def parse_sizes(ir: str) -> dict[int, int]:
    return {int(m.group(1)): int(m.group(2))
            for m in re.finditer(r"@size_(\d+)\s*=\s*.*?\bi32\s+(\d+)", ir)}


def gen_rust(types, sizes, arch) -> tuple[str, int]:
    lines = [
        f'//! Compile-time struct size assertions for {arch["name"]}.',
        "//!",
        "//! Auto-generated by `verify_sizes.py` — do not edit manually.",
        "//! Regenerate: `cd sdk/lib/rust/ntdll-sys && "
        "[VIBEOS_BUILD_ROOT=…] python3 verify_sizes.py`",
        "",
        "#![allow(dead_code)]",
        "",
    ]
    n = 0
    for i, (rn, _ce) in enumerate(types):
        if i in sizes:
            lines.append(
                f"const _: () = assert!(::core::mem::size_of::<super::nt::{rn}>() "
                f"== {sizes[i]});")
            n += 1
    lines.append("")
    return "\n".join(lines), n


def main() -> None:
    check = "--check" in sys.argv
    print("=== ntdll-sys struct size verification ===")
    print(f"SRC={SRC}  BUILD_ROOT={BUILD_ROOT}")
    all_ok = True
    with tempfile.TemporaryDirectory() as td:
        td = Path(td)
        for arch in ARCHS:
            print(f"--- {arch['name']} ---")
            rs = SRC_DIR / arch["rs"]
            if not rs.exists():
                print(f"  SKIP — {rs.name} missing")
                all_ok = False
                continue
            types = extract_types(rs)
            print(f"  {len(types)} bindgen struct types")
            ir = compile_ir(gen_c(types), arch, td)
            if ir is None:
                all_ok = False
                continue
            sizes = parse_sizes(ir)
            code, n = gen_rust(types, sizes, arch)
            outp = SRC_DIR / arch["out"]
            if check:
                if outp.exists() and outp.read_text() == code:
                    print(f"  OK — {n} assertions verified")
                else:
                    print("  MISMATCH — regenerate")
                    all_ok = False
            else:
                outp.write_text(code)
                print(f"  Wrote {arch['out']}: {n} size assertions")
    if check and not all_ok:
        print("FAILED — some assertions need regeneration")
        sys.exit(1)
    if not all_ok:
        sys.exit("FAILED — an arch produced no assertions (see above)")
    if not check:
        print("Done. Ensure size_check_* modules are in lib.rs; cargo check.")


if __name__ == "__main__":
    main()
