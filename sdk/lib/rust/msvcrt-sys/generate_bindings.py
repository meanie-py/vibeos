#!/usr/bin/env python3
"""Generate msvcrt-sys FFI bindings + the LAYOUT GUARDIAN, for all 3 arches.

WHY THIS CRATE EXISTS
---------------------
The Rust bodies we link into msvcrt.dll need to reach fields of msvcrt's *internal*
structs — `thread_data_t.strtok_next` (strtok/wcstok/rand), and above all
`threadlocinfo.pctype/pclmap/pcumap` (the locale's own ctype + case-mapping tables,
which is what makes a *faithful* locale-aware flip possible at all).

Those layouts are NOT stable-by-inspection:

    typedef struct threadlocaleinfostruct {
    #if _MSVCR_VER >= 140
        unsigned short *pctype;      /* <-- FIRST field at >=140 ... */
        int mb_cur_max;
    #endif
        int refcount;
        ...
        unsigned short *pctype;      /* <-- ...and WAY down here at <140 */

so a hand-rolled numeric offset is precisely the bug class that has already cost this
repo two multi-hour hunts (KTHREAD.Teb-0x1e8; the amd64 TEB.GdiBatchCount-at-0x74 read
that killed explorer -- field manual §9.41). `tools/scan_magic_offsets.py` exists to
forbid exactly that. So: bindgen, and `core::mem::offset_of!`, never arithmetic.

WHY THE FLAGS COME FROM compile_commands.json
---------------------------------------------
msvcrt is not built with the default include path. Its CMakeLists *removes*
`sdk/include/crt` from the include dirs and substitutes `dll/win32/msvcrt/include`,
prepends `sdk/include/wine`, and sets `_MSVCR_VER=0`. Hand-copying that flag set into
this script is the drift hazard we are trying to eliminate -- a wrong `-D` here makes
bindgen emit a DIFFERENT struct layout, and a machine-generated wrong offset is *worse*
than a hand-rolled one because it looks trustworthy.

So we read the REAL compile command for `dll/win32/msvcrt/ctype.c` out of the build's
own `compile_commands.json`. If the build's flags change, ours change with them.

WHY THERE IS A SECOND, INDEPENDENT GUARDIAN
-------------------------------------------
Deriving the flags from the build makes bindgen agree with the build *today*. It does
not stop the two from drifting *tomorrow* (someone edits msvcrt.h; someone bumps
_MSVCR_VER; someone regenerates with a stale build dir). And a self-consistent-but-wrong
pair is silent.

So this script emits the offsets it observed into TWO places that are checked by two
DIFFERENT compilers on every single build:

  1. `src/layout_<arch>.rs`  -> `const _: () = assert!(offset_of!(T, f) == N);`
        checked by rustc, every Rust build. Catches: bindgen regenerated wrongly.
  2. `dll/win32/msvcrt/rust_layout_assert.c` -> `_Static_assert(offsetof(T, f) == N);`
        checked by CLANG WITH MSVCRT'S REAL FLAGS, every msvcrt build, all 3 arches.
        Catches: the C headers moved out from under Rust.

Both are pinned to the same literal N. Neither side can move without a loud failure.
That is the whole point: fail loud, never silently degrade.

Usage:
    python3 generate_bindings.py           # regenerate bindings + both guardians
    python3 generate_bindings.py --check   # verify committed output is up to date
"""

from __future__ import annotations

import argparse
import json
import re
import shlex
import subprocess
import sys
import tempfile
from pathlib import Path

SCRIPT_DIR = Path(__file__).resolve().parent
SRC_DIR = SCRIPT_DIR / "src"
# msvcrt-sys is at sdk/lib/rust/msvcrt-sys -> repo root is 4 up
REPO = SCRIPT_DIR.parents[3]
CANON = "/tmp/vibeos"

# The one TU whose compile command we borrow. It includes msvcrt.h (the struct home)
# and is compiled into msvcrt.dll proper, so its flags ARE msvcrt's flags.
PROBE_TU = "dll/win32/msvcrt/ctype.c"

ARCHS = {
    "i386": "i686-pc-windows-gnu",
    "amd64": "x86_64-pc-windows-gnu",
    "arm64": "aarch64-pc-windows-gnu",
}

# Types we need Rust-side. bindgen pulls transitive deps automatically.
ALLOWLIST_TYPES = [
    "__thread_data",
    "threadlocaleinfostruct",
    "threadmbcinfostruct",
    "localeinfo_struct",
    "tagLC_ID",
    "lconv",
    "__lc_time_data",
    # Spellings the WHOLE-FILE translations use in their own signatures. bindgen only
    # emits a type it reaches transitively, and what it reaches differs by ARCH: `BYTE`
    # rode in on i386's CONTEXT and was simply absent from bindgen_amd64.rs, so a file
    # naming it compiled on one arch and not another. Naming them here makes all three
    # islands carry the same vocabulary.
    "_locale_t",  # the CRT's locale sentinel -- see rustconv.typedefs
    "size_t",
    "mbstate_t",
    "wint_t",
    "CPINFO",
    "BYTE",
    "LPCSTR",
]

# The fields we actually touch, and therefore the fields we GUARD.
# (struct tag, field) -- both the C `offsetof` name and the bindgen field name.
GUARDED = [
    ("__thread_data", "thread_errno"),
    ("__thread_data", "random_seed"),
    ("__thread_data", "strtok_next"),
    ("__thread_data", "wcstok_next"),
    ("__thread_data", "mbstok_next"),
    ("__thread_data", "locinfo"),
    ("__thread_data", "mbcinfo"),
    ("threadlocaleinfostruct", "refcount"),
    ("threadlocaleinfostruct", "lc_codepage"),
    ("threadlocaleinfostruct", "lc_handle"),
    ("threadlocaleinfostruct", "lc_clike"),
    ("threadlocaleinfostruct", "mb_cur_max"),
    ("threadlocaleinfostruct", "ctype1"),
    ("threadlocaleinfostruct", "pctype"),
    ("threadlocaleinfostruct", "pclmap"),
    ("threadlocaleinfostruct", "pcumap"),
    # wave 13 (the float parser): fpnum_parse_bnum tests every candidate character
    # against `*locinfo->lconv->decimal_point` — the locale's radix character. That
    # is a two-hop reach (locinfo -> lconv -> decimal_point), so BOTH hops are
    # guarded. `lconv` sits deep in threadlocinfo, well past the refcount block, and
    # its own first field is the one we dereference.
    ("threadlocaleinfostruct", "lconv"),
    ("lconv", "decimal_point"),
    ("localeinfo_struct", "locinfo"),
    ("localeinfo_struct", "mbcinfo"),
]

WRAPPER_SRC = """\
/* msvcrt-sys bindgen wrapper -- see generate_bindings.py */
#include <locale.h>
#include <limits.h>
#include <mbctype.h>
#include <mbstring.h>
#include "msvcrt.h"
#include "winnls.h"
"""


def probe_flags(arch: str) -> list[str]:
    """The REAL -D/-I/-isystem flags msvcrt is compiled with, from the build itself."""
    ccdb = REPO / f"build-{arch}" / "compile_commands.json"
    if not ccdb.exists():
        sys.exit(
            f"FATAL: {ccdb} missing. Configure build-{arch} first "
            f"(./vibeos-cli build --arch {arch}). We derive msvcrt's flags from the "
            f"real build on purpose -- a hand-copied flag list is the drift hazard."
        )
    entries = json.loads(ccdb.read_text())
    hit = [e for e in entries if e["file"].endswith(PROBE_TU)]
    if not hit:
        sys.exit(f"FATAL: no compile command for {PROBE_TU} in {ccdb}")

    # SHELL-LEX, do not .split(). compile_commands.json's `command` is a shell-quoted
    # string: CMake serialises the (empty) value of `-D_CRTIMP=` as `-D_CRTIMP=""`.
    # A naive split hands clang a literal two-quote-character value, so _ACRTIMP --
    # which chains to _CRTIMP -- expands to `""` and every CRT declaration becomes
    # `"" int __cdecl strcmp(...)`, i.e. "expected identifier". shlex strips the
    # quoting exactly as the shell would.
    toks = shlex.split(hit[0]["command"])
    flags: list[str] = []
    i = 0
    while i < len(toks):
        t = toks[i]
        if t.startswith(("-D", "-I")):
            flags.append(t)
        elif t == "-isystem":
            flags.append(t)
            flags.append(toks[i + 1])
            i += 1
        i += 1
    # The build bakes canonical /tmp/vibeos paths (the bind mount). Rewrite to the
    # real tree so bindgen can read the headers without the mount.
    return [f.replace(CANON, str(REPO)) for f in flags]


def probe_cc(arch: str) -> str:
    """The compiler the BUILD actually invokes for msvcrt (e.g. i686-w64-mingw32-clang).

    We use it verbatim for the offset probe, so the offsets we pin are the ones msvcrt
    really gets -- not the ones a differently-configured clang would produce. Its target
    is implied by the wrapper name, so the probe needs no --target.
    """
    ccdb = REPO / f"build-{arch}" / "compile_commands.json"
    entries = json.loads(ccdb.read_text())
    hit = [e for e in entries if e["file"].endswith(PROBE_TU)][0]
    return shlex.split(hit["command"])[0].replace(CANON, str(REPO))


# Windows types that are FIXED-WIDTH by the Windows ABI but which bindgen spells with a
# HOST-dependent Rust alias. See pin_fixed_width() for why this is load-bearing.
FIXED_WIDTH_PINS = {
    "__msvcrt_ulong": ("u32", "corecrt.h: 32-bit on BOTH branches -- `unsigned int` under "
                              "__LP64__, `unsigned long` (LLP64 => 32-bit) on Windows"),
    "__msvcrt_long": ("i32", "corecrt.h: the signed twin, same reasoning"),
    "ULONG": ("u32", "Win32 ULONG is 32-bit (LLP64)"),
    "DWORD": ("u32", "Win32 DWORD is 32-bit (LLP64)"),
    # The signed twin of ULONG, and the one that was missing. It is the type of
    # `threadmbcinfostruct.refcount` -- the FIRST member -- so on the LP64 host it
    # pushed `mbctype[257]` (the lead-byte table all 160 graded mbcs functions read)
    # from offset 28 to 32, and grew the struct by 8 bytes. Both sides of the harness
    # would have read the same wrong bytes and agreed.
    "LONG": ("i32", "Win32 LONG is 32-bit (LLP64)"),
}


def pin_fixed_width(src: str) -> str:
    """Rewrite host-dependent Rust aliases for types that are fixed-width in Win32.

    THIS IS LOAD-BEARING, and the layout guardian caught its absence on day one.

    bindgen reads msvcrt's headers through a *Windows* target, where `unsigned long` is
    32 bits (LLP64), and faithfully records `pub type DWORD = c_ulong`. On the Windows
    targets that is correct. But `msvcrt-sys` is ALSO compiled for the **host** — the
    differential harness (`msvcrt-difftest`) links the very same `msvcrt-crt-rust` code
    it ships, as a linux-x86_64 binary, so it can fuzz it against the genuine C.

    On linux-x86_64, Rust's `c_ulong` is **64 bits**. So `lc_handle: [__msvcrt_ulong; 6]`
    silently becomes 48 bytes instead of 24, every field after it shifts, and the Rust
    reads `locinfo->pcumap` from the wrong place -- *in the harness*, i.e. in exactly the
    component whose entire job is to be the trustworthy oracle. A wrong offset there
    doesn't crash; it makes the oracle lie.

    The C never had this problem: corecrt.h deliberately says

        #ifdef __LP64__
        typedef unsigned int __msvcrt_ulong;   /* "we need 32-bit longs even on 64-bit" */
        #else
        typedef unsigned long __msvcrt_ulong;
        #endif

    -- 32 bits on *both* branches. So pinning these to `u32` is not a workaround; it is
    the type the C actually guarantees, spelled in a way that survives the host.
    """
    for name, (rust_ty, why) in FIXED_WIDTH_PINS.items():
        pat = re.compile(
            rf"^pub type {re.escape(name)} = ::core::ffi::c_[a-z]+;$", re.MULTILINE
        )
        src, n = pat.subn(
            f"// PINNED by generate_bindings.py: {why}.\n"
            f"// bindgen spells this `c_ulong`/`c_long`, which is 64-bit on the linux host\n"
            f"// that builds msvcrt-difftest -- see pin_fixed_width().\n"
            f"pub type {name} = {rust_ty};",
            src,
        )
        # Not every arch/TU pulls every alias; only assert we didn't half-rewrite one.
        if n > 1:
            sys.exit(f"FATAL: {name} declared {n} times in bindgen output")
    return src


def run_bindgen(arch: str, wrapper: Path) -> str:
    args = [
        "bindgen",
        str(wrapper),
        "--use-core",
        "--no-layout-tests",
        "--no-doc-comments",
        # Keep the surface tight: we want the STATE structs, not all of win32.
        *sum((["--allowlist-type", t] for t in ALLOWLIST_TYPES), []),
        "--allowlist-function", "(get_locinfo|get_mbcinfo|msvcrt_get_thread_data)",
        # Constants the translations reference by NAME. A constant missing from the island
        # does not fail loudly -- it becomes a catch-all `match` arm or an invented literal
        # (field manual 9.112/9.113), which is why these are allowlisted rather than left
        # to whatever bindgen happens to reach.
        "--allowlist-var",
        "("
        # ctype1 bits (pre-existing)
        "_UPPER|_LOWER|_DIGIT|_SPACE|_PUNCT|_CONTROL|_BLANK|_HEX|_LEADBYTE|_ALPHA"
        # mbctype.h: the multibyte character classes mbcs.c dispatches on
        "|_MBC_[A-Z]+|_MS|_MP|_M1|_M2|_SBUP|_SBLOW|_MB_CP_[A-Z]+"
        # errno.h / limits.h / stdio.h / wchar.h
        "|EINVAL|ERANGE|EILSEQ|EOF|WEOF|INT_MAX|STRUNCATE|_TRUNCATE|_NLSCMPERROR"
        # winnls.h: the NLS calls the CRT delegates collation and case mapping to
        "|CP_ACP|CP_UTF8|CSTR_EQUAL|NORM_IGNORECASE|MB_ERR_INVALID_CHARS"
        "|LCMAP_[A-Z]+|LOCALE_[A-Z_]+|C1_[A-Z]+|C3_[A-Z]+|CT_CTYPE[0-9]"
        ")",
        "--",
        "-ferror-limit=0",
        "-Wno-everything",
        *probe_flags(arch),
        # bindgen gets the RUST target triple: this is the layout rustc will see.
        # The offset probe below gets the BUILD's own compiler: that is the layout
        # msvcrt really gets. Asserting the two agree is the cross-check.
        f"--target={ARCHS[arch]}",
    ]
    res = subprocess.run(args, capture_output=True, text=True)
    if res.returncode != 0:
        sys.exit(f"FATAL: bindgen failed for {arch}:\n{res.stderr[-4000:]}")
    return pin_fixed_width(res.stdout)


def c_offsets(arch: str) -> dict[tuple[str, str], int]:
    """Ask CLANG -- with msvcrt's REAL flags -- where each guarded field lives.

    This is the independent oracle. It is what makes the two guardians a real
    cross-check rather than a self-consistent echo.
    """
    lines = [WRAPPER_SRC, "#include <stddef.h>"]
    for i, (tag, field) in enumerate(GUARDED):
        # Emit each offset as the size of a distinctively-named array type: the value
        # survives into the LLVM IR / asm without needing to link or run anything.
        lines.append(
            f"char __vibeos_off_{i}[offsetof(struct {tag}, {field}) + 1];"
        )
    src = "\n".join(lines) + "\n"

    with tempfile.TemporaryDirectory() as td:
        c = Path(td) / "probe.c"
        c.write_text(src)
        res = subprocess.run(
            [
                probe_cc(arch), "-S", "-emit-llvm", "-o", "-", str(c),
                "-ferror-limit=0", "-Wno-everything",
                *probe_flags(arch),
            ],
            capture_output=True, text=True,
        )
        if res.returncode != 0:
            sys.exit(f"FATAL: offset probe failed for {arch}:\n{res.stderr[-4000:]}")

    out: dict[tuple[str, str], int] = {}
    for i, key in enumerate(GUARDED):
        m = re.search(rf"@__vibeos_off_{i} = .*?\[(\d+) x i8\]", res.stdout)
        if not m:
            sys.exit(f"FATAL: could not read offset for {key} on {arch}")
        out[key] = int(m.group(1)) - 1  # we asked for offset+1
    return out


BINDGEN_HEADER = """\
// @generated by sdk/lib/rust/msvcrt-sys/generate_bindings.py -- DO NOT EDIT.
//
// Bindings for msvcrt's INTERNAL state structs, generated with msvcrt's OWN compile
// flags (read from build-{arch}/compile_commands.json -- see the generator's docstring
// for why a hand-copied flag list is not acceptable here).
//
// Layout is guarded on two sides, by two different compilers, on every build:
//   * layout_{arch}.rs        -- rustc checks offset_of! against the literals below
//   * dll/win32/msvcrt/rust_layout_assert.c -- clang, with msvcrt's real flags, checks
//     offsetof() against the SAME literals, on all 3 arches, every msvcrt build.
#![allow(non_camel_case_types, non_snake_case, non_upper_case_globals, dead_code)]

"""


def rust_layout(arch: str, offs: dict[tuple[str, str], int]) -> str:
    lines = [
        f"// @generated by generate_bindings.py -- DO NOT EDIT.",
        f"//",
        f"// GUARDIAN 1 of 2 (rustc side). Proves the bindgen struct in bindgen_{arch}.rs",
        f"// still agrees with what msvcrt's C compiler laid out. If bindgen is ever",
        f"// regenerated with wrong flags, THIS FAILS THE RUST BUILD.",
        f"// The twin is dll/win32/msvcrt/rust_layout_assert.c (clang side, same literals).",
        f"use crate::*;",
        f"use core::mem::offset_of;",
        f"",
    ]
    for (tag, field), off in offs.items():
        rs_ty = RUST_TYPE_OF[tag]
        lines.append(
            f"const _: () = assert!(offset_of!({rs_ty}, {field}) == {off}, "
            f'"msvcrt-sys: {tag}.{field} moved on {arch} -- REGENERATE msvcrt-sys");'
        )
    return "\n".join(lines) + "\n"


# bindgen names: `struct threadlocaleinfostruct` -> `threadlocaleinfostruct`
RUST_TYPE_OF = {
    "__thread_data": "__thread_data",
    "threadlocaleinfostruct": "threadlocaleinfostruct",
    "threadmbcinfostruct": "threadmbcinfostruct",
    "localeinfo_struct": "localeinfo_struct",
    "lconv": "lconv",
}


def c_guardian(per_arch: dict[str, dict[tuple[str, str], int]]) -> str:
    out = [
        "/*",
        " * @generated by sdk/lib/rust/msvcrt-sys/generate_bindings.py -- DO NOT EDIT.",
        " *",
        " * GUARDIAN 2 of 2 (clang side).",
        " *",
        " * The Rust bodies linked into msvcrt.dll reach into `struct __thread_data` and",
        " * `struct threadlocaleinfostruct` (strtok's saved pointer; the locale's pctype /",
        " * pclmap / pcumap tables). Rust's view of those layouts comes from bindgen, which",
        " * is a SNAPSHOT. This file makes msvcrt's own build the guardian of that snapshot:",
        " * it is compiled with msvcrt's real flags, on all 3 arches, on every build.",
        " *",
        " * If you edited msvcrt.h / include/corecrt.h / _MSVCR_VER and landed here, the C",
        " * moved and Rust does not know. Regenerate:",
        " *",
        " *     python3 sdk/lib/rust/msvcrt-sys/generate_bindings.py",
        " *",
        " * Do NOT just bump the number -- the Rust side asserts the same literal, and the",
        " * two must be regenerated together.",
        " */",
        "",
        "#include <locale.h>",
        '#include "msvcrt.h"',
        '#include "winnls.h"',
        "#include <stddef.h>",
        "",
        "/* C11 _Static_assert; clang has it in all modes we build. */",
        "#define VIBEOS_ASSERT_OFF(tag, field, off) \\",
        "    _Static_assert(offsetof(struct tag, field) == (off), \\",
        '        \"msvcrt-sys layout drift: \" #tag \".\" #field \" moved -- rerun \" \\',
        '        \"sdk/lib/rust/msvcrt-sys/generate_bindings.py\")',
        "",
    ]
    guards = {"i386": "defined(_M_IX86)", "amd64": "defined(_M_AMD64)", "arm64": "defined(_M_ARM64)"}
    for arch, offs in per_arch.items():
        out.append(f"#if {guards[arch]}")
        for (tag, field), off in offs.items():
            out.append(f"VIBEOS_ASSERT_OFF({tag}, {field}, {off});")
        out.append("#endif")
        out.append("")

    # ── wave 13: the float parser's SCALAR pins ────────────────────────────────
    #
    # msvcrt-crt-rust/src/fpnum.rs is a transcription of string.c's bignum parser, and it
    # hardcodes three kinds of literal that the C gets from a header. A struct offset is
    # not the only thing that can drift out from under a transcription, so these are
    # guarded on exactly the same terms.
    out += [
        "/* ── wave 13 (fpnum.rs): the float parser's scalar pins ──────────────────── */",
        "",
        "#include <float.h>",
        "#include <errno.h>",
        "#include <math.h>",
        "",
        "/* 1. <float.h>'s exponent limits. fpnum_parse_bnum early-outs on these; if the",
        "      header moves them, our Rust silently parses a different range. */",
        '_Static_assert(DBL_MAX_10_EXP == 308, "fpnum.rs pins DBL_MAX_10_EXP=308");',
        '_Static_assert(DBL_MIN_10_EXP == -307, "fpnum.rs pins DBL_MIN_10_EXP=-307");',
        '_Static_assert(DBL_DIG == 15, "fpnum.rs pins DBL_DIG=15");',
        '_Static_assert(DBL80_MAX_10_EXP == 4932, "fpnum.rs pins DBL80_MAX_10_EXP=4932");',
        '_Static_assert(DBL80_MIN_10_EXP == -4951, "fpnum.rs pins DBL80_MIN_10_EXP=-4951");',
        "",
        "/* 2. errno / math return codes the float doors hand back. */",
        '_Static_assert(ERANGE == 34, "fpnum.rs pins ERANGE=34");',
        '_Static_assert(_OVERFLOW == 3, "fpnum.rs pins _OVERFLOW=3");',
        '_Static_assert(_UNDERFLOW == 4, "fpnum.rs pins _UNDERFLOW=4");',
        "",
        "/* 3. `char` MUST be signed.",
        "",
        "      string.c's strtod_str_get() returns `*(*p)++` -- a signed char widened to an",
        "      unsigned short. So an input byte of 0xFF sign-extends to 0xFFFF, which IS WEOF:",
        "      the parser sees end-of-input, but the cursor has already stepped past the byte",
        "      and the `if (nch != WEOF) unget()` therefore does not put it back. fpnum.rs",
        "      reproduces that with `*p as i8 as i16 as u16`. If char were ever UNSIGNED on",
        "      some target, the C would stop doing this and our Rust would be the one that is",
        "      wrong -- so the assumption is asserted rather than assumed. */",
        '_Static_assert((char)-1 < 0, "fpnum.rs assumes signed char (see strtod_str_get)");',
        '_Static_assert((wchar_t)(char)0xFF == 0xFFFF, "byte 0xFF must widen to WEOF");',
        "",
        "/* 4. MSVCRT__LDOUBLE is the 10-byte x87 struct, NOT Wine's 12-byte one.",
        "      __STRINGTOLD_L computes into the 12-byte WINE_BROKEN_LDOUBLE and copies only",
        "      sizeof(MSVCRT__LDOUBLE) bytes out. Getting this wrong overruns the caller. */",
        '_Static_assert(sizeof(MSVCRT__LDOUBLE) == 10, "fpnum.rs pins LDOUBLE=10");',
        '_Static_assert(sizeof(WINE_BROKEN_LDOUBLE) == 12, "fpnum.rs pins WINE_LDOUBLE=12");',
        "",
    ]
    return "\n".join(out)


def main() -> int:
    ap = argparse.ArgumentParser()
    ap.add_argument("--check", action="store_true", help="verify committed output is current")
    args = ap.parse_args()

    SRC_DIR.mkdir(parents=True, exist_ok=True)
    with tempfile.TemporaryDirectory() as td:
        wrapper = Path(td) / "wrapper.h"
        wrapper.write_text(WRAPPER_SRC)

        per_arch: dict[str, dict[tuple[str, str], int]] = {}
        pending: list[tuple[Path, str]] = []

        for arch in ARCHS:
            print(f"=== {arch} ===", flush=True)
            bindings = BINDGEN_HEADER.replace("{arch}", arch) + run_bindgen(arch, wrapper)
            offs = c_offsets(arch)
            per_arch[arch] = offs
            print(f"  {len(offs)} guarded offsets, e.g. "
                  f"threadlocinfo.pcumap @ {offs[('threadlocaleinfostruct','pcumap')]}")
            pending.append((SRC_DIR / f"bindgen_{arch}.rs", bindings))
            pending.append((SRC_DIR / f"layout_{arch}.rs", rust_layout(arch, offs)))

        pending.append((REPO / "dll/win32/msvcrt/rust_layout_assert.c", c_guardian(per_arch)))

    stale = [p for p, content in pending
             if not p.exists() or p.read_text() != content]
    if args.check:
        if stale:
            print("\nSTALE (regenerate with generate_bindings.py):", file=sys.stderr)
            for p in stale:
                print(f"  {p.relative_to(REPO)}", file=sys.stderr)
            return 1
        print("\nup to date.")
        return 0

    for p, content in pending:
        p.write_text(content)
        print(f"wrote {p.relative_to(REPO)}")
    return 0


if __name__ == "__main__":
    sys.exit(main())
