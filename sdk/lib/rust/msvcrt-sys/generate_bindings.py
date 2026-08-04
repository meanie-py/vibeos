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
    # math.c's two public struct types: `_exception` is what `__setusermatherr`'s callback
    # is handed (five fields, one of them a `char *` the caller may print) and `_complex`
    # is `_cabs`'s by-VALUE parameter, so its layout is the calling convention. Both are
    # the shape a hand-written Rust struct gets right on one arch and wrong on another.
    "_exception",
    "_complex",
    # winnt.h's fixed-width aliases math.c transmutes floats through.
    "UINT64",
    "INT64",
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
    # ── file.c (the stream layer) ────────────────────────────────────────
    # FILE's layout IS part of the contract: file.c reads and writes _ptr/_cnt/
    # _base/_flag/_file/_charbuf/_bufsiz/_tmpfname directly, and msvcrt.h's
    # `_MSVCR_VER >= 140` block orders those fields DIFFERENTLY from the header the
    # build reaches. Deriving it is the only safe option; the difftest's own
    # layout guardian caught a hand-transcription of exactly this.
    # ── dir.c (the directory / path surface) ────────────────────────────
    # The eight `_finddata*` shapes are the whole reason `_findfirst` has eight entry
    # points: they differ ONLY in the width of `time_t` (32/64) and `_fsize_t` (32/64),
    # and dir.c memcpy's Win32's FILETIME results into whichever one the caller passed.
    # A hand-written struct would be right on one arch and one width and silently wrong
    # on the other seven — and the failure is a garbage file size or a 2038 timestamp,
    # not a compile error. `_wfinddata*` are the same eight with a `wchar_t` name field.
    # These are the tags in msvcrt's OWN dll/win32/msvcrt/include/corecrt_io.h and
    # corecrt_wio.h, not the SDK's — the module's CMakeLists drops sdk/include/crt and
    # substitutes its own dir, and the two spell the 64-bit tag differently (`_finddata64_t`
    # here, `__finddata64_t` in the SDK's io.h). Allowlisting the SDK spelling emits nothing.
    "_finddata32_t",
    "_finddata32i64_t",
    "_finddata64i32_t",
    "_finddata64_t",
    "_wfinddata32_t",
    "_wfinddata32i64_t",
    "_wfinddata64i32_t",
    "_wfinddata64_t",
    # direct.h: `_getdiskfree`'s out-parameter.
    "_diskfree_t",
    # winbase.h. dir.c passes these straight to FindFirstFileW/FindNextFileW and reads
    # ftCreationTime/ftLastAccessTime/ftLastWriteTime/nFileSizeLow/cFileName out of them.
    "_WIN32_FIND_DATAA",
    "_WIN32_FIND_DATAW",
    "WIN32_FIND_DATAA",
    "WIN32_FIND_DATAW",
    "_iobuf",
    "FILE",  # the typedef NAME; allowlisting only the `_iobuf` tag emits the struct
    # but not the alias, and every signature in file.c spells it `FILE`.
    # `ioinfo` is deliberately absent: it is TU-local to file.c, so the
    # translation carries it and the island must not.
    # `struct _stat` / `struct _stati64` are DISTINCT structs (sys/stat.h:40,113), not
    # aliases of the 32/64 variants below — file.c's _fstat/_stat convert into them.
    # Rust keeps types and fns in separate namespaces, so these coexist with the
    # `fn _stat` file.rs exports. Derived, because the caller reads this layout.
    "_stat",
    "_stati64",
    "_stat32",
    "_stat32i64",
    "_stat64",
    "_stat64i32",
    "__utimbuf32",
    "__utimbuf64",
    "_RTL_CRITICAL_SECTION",
    "_LARGE_INTEGER",
    "_FILETIME",
    "_SECURITY_ATTRIBUTES",
    "_WIN32_FILE_ATTRIBUTE_DATA",
    "_STARTUPINFOA",
    "_IO_STATUS_BLOCK",
    "_FILE_BASIC_INFORMATION",
    "_FILE_STANDARD_INFORMATION",
    "HANDLE",
    "DWORD",
    "LONG",
    "ULONG",
    "WCHAR",
    # Two ENUMS, listed for their VARIANTS rather than their layout: file.c passes
    # `FileBasicInformation`/`FileStandardInformation` to NtQueryInformationFile and
    # `GetFileExInfoStandard` to GetFileAttributesExW. An enum variant is not a `var`,
    # so allowlisting the constant name does nothing -- the TYPE has to come in for the
    # variants to exist at all. Left out, each becomes an undefined name (loud) or, if
    # someone "fixes" it with a literal, a silently wrong information class.
    "_FILE_INFORMATION_CLASS",
    "_GET_FILEEX_INFO_LEVELS",
    # The TYPEDEF NAMES for the structs above. Allowlisting the struct tag emits the
    # `struct _SECURITY_ATTRIBUTES` but NOT the `typedef ... SECURITY_ATTRIBUTES` that
    # every caller actually spells, so without these the tag is present and the name the
    # code uses is still undefined. Cheap to add and impossible to get subtly wrong --
    # bindgen writes `pub type X = _X;` from the header's own typedef.
    "CRITICAL_SECTION",
    "RTL_CRITICAL_SECTION",
    "RTL_CRITICAL_SECTION_DEBUG",
    "SECURITY_ATTRIBUTES",
    "FILE_BASIC_INFORMATION",
    "FILE_STANDARD_INFORMATION",
    "WIN32_FILE_ATTRIBUTE_DATA",
    "STARTUPINFOA",
    "IO_STATUS_BLOCK",
    "LARGE_INTEGER",
    "FILETIME",
    "PCHAR",
    # Same story as `BYTE` above and worth repeating because it recurred: `ULONGLONG`
    # rode into the amd64/arm64 islands transitively and was simply absent from
    # bindgen_i386.rs, so time.c's `_gmtime64_s` compiled on two arches and not the
    # third. Naming a type here is what makes all three islands carry one vocabulary.
    "ULONGLONG",
    "LONGLONG",
    "fpos_t",
    # msvcrt's own: the printf argument union and file.c's text/binary mode enum.
    "_printf_arg",
    "printf_arg",
    "textmode",
    # ── time.c (the clock and calendar layer) ────────────────────────────
    # `tm` and `FILETIME` already rode in transitively on file.c's closure; the rest
    # did not, and every one of them is a struct the translation READS AND WRITES
    # FIELD BY FIELD rather than passes through. `SYSTEMTIME` is the worked example:
    # time.c converts through it in both directions on every gmtime/localtime/mktime
    # call, and its sixteen bytes are eight `WORD`s in a fixed order — a hand-written
    # `#[repr(C)]` that put wDayOfWeek where wDay goes would link, run, and give a
    # calendar that is wrong by a variable number of days.
    "tm",
    "_SYSTEMTIME",
    "SYSTEMTIME",
    "_TIME_ZONE_INFORMATION",
    "TIME_ZONE_INFORMATION",
    # <sys/timeb.h>: what _ftime32/_ftime64 fill in. Two DISTINCT structs — the
    # 32-bit one is not a narrowing view of the other, it has its own field order.
    "__timeb32",
    "__timeb64",
    # <time.h>: _timespec32_get / _timespec64_get.
    "_timespec32",
    "_timespec64",
    # The scalar time types. Named rather than left transitive because the exported
    # signatures spell them, and __time32_t vs __time64_t is the whole point of half
    # of time.c's export list — a translation that resolved both to the same width
    # would compile and would silently make the 32-bit family alias the 64-bit one.
    "__time32_t",
    "__time64_t",
    "clock_t",
    # ── console.c ────────────────────────────────────────────────────────
    # wincon.h's INPUT_RECORD. Its `Event` member is a UNION of five event structs and
    # the KEY_EVENT arm nests a further union of `UnicodeChar`/`AsciiChar` over the same
    # two bytes; `_getch` reads the narrow member and `_getwch` the wide one from the
    # SAME record. A hand-written struct that got the union nesting or the u16/u8 overlay
    # wrong would compile and decode every keystroke to garbage.
    "_INPUT_RECORD",
    "INPUT_RECORD",
    # ── exit.c ───────────────────────────────────────────────────────────
    # winuser.h's MSGBOXPARAMSW — a 10-field struct passed BY POINTER to
    # MessageBoxIndirectW with its own `cbSize` self-description, so its layout is
    # validated by the callee at runtime rather than by the linker.
    "tagMSGBOXPARAMSW",
    "MSGBOXPARAMSW",
    # winbase.h's CRITICAL_SECTION_DEBUG: exit.c statically initialises a
    # CRITICAL_SECTION and its debug record as a mutually-pointing pair, which only
    # works if both layouts are the real ones.
    "_RTL_CRITICAL_SECTION_DEBUG",
    "RTL_CRITICAL_SECTION_DEBUG",
    "CRITICAL_SECTION_DEBUG",
    # stdlib.h / msvcrt.h: the atexit registry vocabulary. `_onexit_table_t` is three
    # pointers whose ORDER is the growth algorithm (`_first`/`_last`/`_end`), and
    # `_onexit_t` vs `_PVFV` differ by return type — `__dllonexit` stores one as the
    # other, so the two must be the header's own types and not two spellings of the
    # same invented alias.
    "_onexit_t",
    "_onexit_table_t",
    "_PVFV",
    "_purecall_handler",
    # windef.h's plain `INT`, spelled by console.c's and exit.c's own signatures.
    "INT",
    # winnt.h: data.c's `_get_osver`/`_get_winmajor`/`_get_winminor` fill one of these.
    "_OSVERSIONINFOW",
    "OSVERSIONINFOW",
    # ── locale.c ─────────────────────────────────────────────────────────
    # The typedef spellings locale.c uses in its own signatures. The underlying
    # `threadlocaleinfostruct` is already above; these are the names the code says.
    "threadlocinfo",
    "pthreadlocinfo",
    "threadmbcinfo",
    "pthreadmbcinfo",
    "_locale_tstruct",
    "LCTYPE",
    "LCID",
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
/* file.c's own prologue. Derived, not guessed: `profile_derive dll/win32/msvcrt
   --target msvcrt_static --usermode` mirrors each TU's REAL include closure, and
   §9.108 says that if the .c compiles, that exact sequence is bindgen-able. The
   stream layer needs FILE and ioinfo plus the whole open-flag, stream-flag and
   stat-mode vocabulary, none of which the locale-scoped island above reaches.
   (Spelling those families with globs here would end this comment early -- the
   token is an asterisk followed by a slash.) */
#include <fcntl.h>
#include <io.h>
#include <share.h>
#include <stdio.h>
#include <sys/locking.h>
#include <sys/stat.h>
#include <sys/utime.h>
#include <direct.h>  /* _getdrive */
#include <time.h>    /* _time64, struct tm, _timespec32/64 */
#include <sys/timeb.h>  /* time.c's _ftime32/_ftime64 fill a struct __timeb32/64 */
#include <wctype.h>  /* iswalpha */
#include "wincon.h"
#include "winbase.h"
#include "winternl.h"
/* file.c's lock numbering. `_STREAM_LOCKS` (28) and `_IOB_ENTRIES` (20) live here, and
   mtdll.h is the header file.c itself takes them from. Deriving rather than copying
   matters more than usual for `_IOB_ENTRIES`: it is 20 in crt/stdio.h and mtdll.h but
   3 in ucrt/stdio.h, so a transcribed constant is right only until someone reads the
   other header -- the same silent divergence `_IOMYBUF` is the worked example of. */
#include "mtdll.h"
/* math.c: float.h owns the whole _MCW_, _EM_, _RC_, _PC_ and _DN_ control-word
   vocabulary, and math.h owns the _FPCLASS_ and FP_ classification results. These are
   the worst case for an invented literal: _setfp maps hardware bits onto them in BOTH
   directions, so one constant arriving as a number instead of a name silently changes
   the floating-point environment of the whole OS. process.c: process.h owns the _P_
   spawn modes, where a swapped _P_WAIT/_P_OVERLAY pair changes whether a spawned
   process is waited for or replaces the caller. */
#include <float.h>
#include <math.h>
#include <process.h>
/* exit.c: winuser.h owns MSGBOXPARAMSW plus the MB_/ID* vocabulary its message-box path
   uses, and signal.h owns SIGABRT (which `abort` raises before falling through to
   `_exit(3)`). misc.c: ntsecapi.h owns RtlGenRandom, which is `SystemFunction036` under
   an alias returning BOOLEAN — the shape an extern block gets wrong quietly. */
#include "winuser.h"
#include <signal.h>
#include "ntsecapi.h"
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
    "__msvcrt_ulong": (
        "u32",
        "corecrt.h: 32-bit on BOTH branches -- `unsigned int` under "
        "__LP64__, `unsigned long` (LLP64 => 32-bit) on Windows",
    ),
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

# NOT PINNABLE HERE: `size_t`. Worth recording, because it looks like it belongs above
# and an entry for it is a silent no-op.
#
# bindgen maps size_t to `usize` by default, but only the COMPILER's size_t.
# vcruntime.h:110 does `#undef size_t` and declares its own `typedef unsigned int` /
# `unsigned __int64`, which bindgen canonicalises straight to a per-arch primitive and
# then drops -- grep the generated island and `size_t` appears ZERO times, so there is
# no `pub type size_t = ...` line for pin_fixed_width() to rewrite and allowlisting the
# name does not resurrect one. The visible symptom is `calloc(c_uint, c_uint)` on i386
# and `calloc(c_ulonglong, c_ulonglong)` on amd64.
#
# Consequence for callers: a size argument cannot be spelled with one fixed width. Cast
# with `as _` at the call site and let inference take the callee's parameter type per
# arch -- which is what `crate-repair` means by "hand-write the semantic alias, do not
# bake either width" when it reports `truncating_numeric_cast/target_divergent`.


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
        pat = re.compile(rf"^pub type {re.escape(name)} = ::core::ffi::c_[a-z]+;$", re.MULTILINE)
        src, n = pat.subn(
            f"// PINNED by generate_bindings.py: {why}.\n"
            f"// bindgen's own spelling is HOST-dependent here, and msvcrt-sys is also\n"
            f"// compiled for the linux host that builds msvcrt-difftest -- see\n"
            f"// pin_fixed_width().\n"
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
        # Macros whose body is a CAST are invisible to bindgen's normal constant path:
        # `#define INVALID_FILE_ATTRIBUTES ((DWORD)-1)` and the `STD_*_HANDLE` family are
        # parsed as expressions it will not evaluate, so they silently do not appear.
        # The fallback asks clang for the value instead of guessing it here -- same
        # derived-not-transcribed rule as the offsets below, and the reason `_IOMYBUF`
        # is the worked example: a hand-written `-1` is right until the day it is not.
        "--clang-macro-fallback",
        "--allowlist-function",
        # The CRT state accessors, plus the Win32/NT surface file.c calls straight
        # through to. Named explicitly rather than by prefix: the point of this island
        # is a TIGHT surface, and a signature that arrives by accident is one nobody
        # checked. Getting one wrong is not a build error -- it links and corrupts --
        # which is exactly why these are DERIVED from the real headers and never
        # hand-written as `extern` blocks in the consuming crate.
        "(get_locinfo|get_mbcinfo|msvcrt_get_thread_data"
        # winbase.h / fileapi.h: the file API
        "|CreateFileW|ReadFile|WriteFile|CloseHandle|DeleteFileW|MoveFileExW"
        "|SetFilePointer|SetFilePointerEx|SetEndOfFile|FlushFileBuffers"
        "|GetFileType|GetFileSizeEx|GetFileAttributesA|GetFileAttributesW"
        "|GetFileAttributesExW|SetFileAttributesW|SetFileTime|GetTempFileNameW"
        "|LockFile|UnlockFile|CreatePipe|DuplicateHandle"
        # handles, process, console
        # GetLastError is here rather than hand-written in the consuming crate because
        # it was hand-written once and got the convention wrong: `extern "C"` for a
        # WINAPI function links clean on amd64/arm64 and fails only on i386, as
        # `_GetLastError` with no `@0`. Derived from winbase.h it is `extern "stdcall"`
        # on i386 and `extern "C"` elsewhere, with no judgement call left to make.
        "|GetLastError"
        "|GetStdHandle|SetStdHandle|GetCurrentProcess|GetCurrentProcessId"
        "|GetStartupInfoA|WriteConsoleW|VerifyConsoleIoHandle|Sleep"
        # synchronisation
        "|InitializeCriticalSection|InitializeCriticalSectionEx|DeleteCriticalSection"
        "|EnterCriticalSection|LeaveCriticalSection|TryEnterCriticalSection"
        "|InterlockedCompareExchange|InterlockedCompareExchangePointer"
        # winternl.h / ntdll: the NT calls file.c and time.c make directly
        "|NtQueryInformationFile|RtlTimeToSecondsSince1970|NtQuerySystemTime"
        # winbase.h / timezoneapi.h / datetimeapi.h: time.c's clock and calendar
        # substrate. `SystemTimeToTzSpecificLocalTime` takes three pointers and
        # `GetTimeZoneInformation` returns a DWORD sentinel (TIME_ZONE_ID_INVALID is
        # 0xFFFFFFFF, not -1) — both are the shape where a hand-written extern block
        # is right on two arches and wrong on the third.
        "|GetSystemTimeAsFileTime|GetSystemTimePreciseAsFileTime|GetLocalTime"
        "|GetTimeZoneInformation|SystemTimeToFileTime|FileTimeToSystemTime"
        "|SystemTimeToTzSpecificLocalTime|GetDateFormatW|GetTimeFormatW"
        "|WideCharToMultiByte|lstrcpynA"
        # winbase.h: process.c's half of the Win32 surface. CreateProcessW takes ten
        # arguments including two structs and three BOOLs — the single worst shape to
        # hand-write an extern for, and the reason this island exists.
        "|CreateProcessW|WaitForSingleObject|GetExitCodeProcess|TerminateProcess"
        "|GetProcessHeap|HeapAlloc|HeapFree|HeapReAlloc"
        "|LoadLibraryW|FreeLibrary|GetProcAddress|GetModuleFileNameW"
        # winbase.h: dir.c's Win32 surface. FindFirstFileA/W hand back a WIN32_FIND_DATA
        # whose five FILETIME/size fields dir.c copies straight into a `_finddata*_t` — the
        # struct pair whose per-arch layout is the entire reason this island exists rather
        # than a hand-written extern block.
        "|FindFirstFileA|FindFirstFileW|FindNextFileA|FindNextFileW|FindClose"
        "|CreateDirectoryW|RemoveDirectoryW|SetCurrentDirectoryW|GetCurrentDirectoryW"
        "|GetFullPathNameW|GetDiskFreeSpaceW|GetDriveTypeW"
        # winbase.h: environ.c's. GetEnvironmentStringsA is a genuine trap — the ANSI
        # entry point's real exported name is `GetEnvironmentStrings` with no suffix
        # (environ.c.obj imports `_GetEnvironmentStrings@0`), so bindgen must supply the
        # alias rather than a hand-written extern inventing a symbol that does not exist.
        "|GetEnvironmentStrings|GetEnvironmentStringsA|GetEnvironmentStringsW"
        "|FreeEnvironmentStringsA|FreeEnvironmentStringsW|SetEnvironmentVariableW"
        # CRT siblings file.c calls that are still C. The ones already Rust in
        # msvcrt-crt-rust are IMPORTED there instead, so rustc checks them against their
        # definition; these have no Rust definition to check against, which is exactly
        # why their signatures come from the headers rather than from a hand-written
        # `extern` block. `_time64` and `_getdrive` are the reason time.h and direct.h
        # joined the wrapper above.
        "|calloc|iswalpha|_wcsdup|_wgetenv|_time64|_getdrive|_get_fmode"
        "|msvcrt_wstrdupa|__doserrno"
        # wincon.h: console.c's whole surface. It opens CONIN$/CONOUT$ by NAME through
        # CreateFileA, so the ANSI entry point joins the W one already above, and
        # ReadConsoleInputA hands back an INPUT_RECORD whose KEY_EVENT arm is a UNION of
        # an AsciiChar and a UnicodeChar over the same two bytes — read the wrong member
        # and every enhanced key decodes to garbage. Derived, not transcribed.
        "|CreateFileA|WriteConsoleA|ReadConsoleA|GetConsoleMode|SetConsoleMode"
        "|ReadConsoleInputA|ReadConsoleInputW|PeekConsoleInputA"
        "|GetNumberOfConsoleInputEvents"
        # winuser.h / winbase.h / ntsecapi.h: exit.c's message-box path and misc.c's four.
        # `SystemFunction036`, NOT `RtlGenRandom`: ntsecapi.h:691 is
        # `#define RtlGenRandom SystemFunction036` and the prototype below it is spelled
        # with the macro, so by the time bindgen sees the declaration the name is already
        # rewritten and allowlisting `RtlGenRandom` matches nothing at all — silently, with
        # a clean regeneration. Verified by grepping the emitted bindings for it, which is
        # the only way this class shows up. The consuming crate re-exports it under the
        # documented name. It is also `BOOLEAN` rather than `BOOL`, and stdcall on i386.
        "|ExitProcess|GetModuleHandleW|GetActiveWindow|MessageBoxIndirectW|DebugBreak"
        "|Beep|SystemFunction036|VirtualProtect"
        # winbase.h: data.c's process-startup surface. `GetVersionExW` fills an
        # OSVERSIONINFOW whose leading `dwOSVersionInfoSize` the callee validates, so the
        # struct layout is part of the call contract rather than a detail.
        "|GetCommandLineA|GetCommandLineW|GetModuleFileNameA|GetVersionExW"
        # winnls.h: locale.c's NLS surface. `EnumSystemLocalesEx` takes a CALLBACK
        # whose convention is stdcall on i386 — deriving it is what makes the Rust
        # callback's `extern "system"` checkable instead of a guess — and
        # `GetLocaleInfoW` returns a character count that locale.c uses to size
        # allocations.
        "|CompareStringA|CompareStringW|EnumSystemLocalesEx|GetLocaleInfoA"
        "|GetLocaleInfoW|GetLocaleInfoEx|GetUserDefaultLocaleName|IsValidCodePage"
        "|IsValidLocaleName|LCIDToLocaleName|GetACP"
        ")",
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
        "|CP_ACP|CP_UTF8|CSTR_EQUAL|NORM_IGNORECASE|MB_[A-Z_]+"
        "|LCMAP_[A-Z]+|LOCALE_[A-Z_0-9]+|C1_[A-Z]+|C3_[A-Z]+|CT_CTYPE[0-9]"
        # `LOCALE_[A-Z_]+` did NOT match the digit-suffixed half of the family, so
        # `LOCALE_SMONTHNAME1`..`12`, `LOCALE_SDAYNAME1`..`7`, `LOCALE_SABBREV*` and
        # `LOCALE_S1159`/`S2359` were all absent — 42 of the LCTYPEs locale.c passes to
        # GetLocaleInfoW to build `__lc_time_data`. Missing, they would have become
        # invented literals: adjacent LCTYPE numbers name adjacent calendar fields, so a
        # wrong one yields a locale whose month names are shifted rather than one that
        # fails. The `0-9` is the fix and the reason it is spelled out here.
        # ── file.c ──────────────────────────────────────────────────────────
        # fcntl.h / share.h / sys/locking.h / sys/stat.h / stdio.h. `_IOMYBUF` is
        # the worked example of why these are DERIVED and not transcribed: it is
        # 0x0040 in msvcrt.h's _MSVCR_VER>=140 block and 0x0008 in the header the
        # build actually reaches, and getting it wrong is silent.
        "|_O_[A-Z0-9_]+|_SH_[A-Z]+|_LK_[A-Z]+|_S_I[A-Z]+"
        "|_IOREAD|_IOWRT|_IOEOF|_IOERR|_IOMYBUF|_IORW|_IOSTRG|_IOFBF|_IOLBF|_IONBF"
        "|BUFSIZ|_IOB_ENTRIES|FILENAME_MAX|TMP_MAX|TMP_MAX_S|L_tmpnam|L_tmpnam_s"
        "|SEEK_SET|SEEK_CUR|SEEK_END|_NSTREAM_"
        # ── errno.c ─────────────────────────────────────────────────────────
        # errno.h's COMPLETE non-POSIX set, deliberately, not the subset a given wave
        # happens to name. This list grew on demand until file.c's wave, at which point
        # 9 of `msvcrt_set_errno`'s 18 targets were absent -- and a missing one does not
        # fail to compile, it invites an invented literal (§9.113). The trap is specific:
        # EDEADLK is 36 here and 35 in glibc, ENOTEMPTY 41 here and 39 there, so a guess
        # taken from host knowledge is plausible AND wrong. The POSIX block behind
        # `#ifndef _CRT_NO_POSIX_ERROR_CODES` is correctly absent -- msvcrt defines that
        # macro, and the flags come from the build's own compile_commands.json.
        "|EPERM|ENOENT|ESRCH|EINTR|EIO|ENXIO|E2BIG|ENOEXEC|EBADF|ECHILD|EAGAIN"
        "|ENOMEM|EACCES|EFAULT|EBUSY|EEXIST|EXDEV|ENODEV|ENOTDIR|EISDIR|ENFILE"
        "|EMFILE|ENOTTY|EFBIG|ENOSPC|ESPIPE|EROFS|EMLINK|EPIPE|EDOM|EDEADLK"
        "|ENAMETOOLONG|ENOLCK|ENOSYS|ENOTEMPTY|EDEADLOCK"
        # winbase.h / winnt.h: the Win32 file API file.c calls straight through to.
        "|INVALID_HANDLE_VALUE|INVALID_FILE_ATTRIBUTES|INVALID_SET_FILE_POINTER"
        "|GENERIC_READ|GENERIC_WRITE|CREATE_[A-Z]+|OPEN_[A-Z]+|TRUNCATE_EXISTING"
        "|FILE_SHARE_[A-Z]+|FILE_ATTRIBUTE_[A-Z]+|FILE_FLAG_[A-Z_]+|FILE_TYPE_[A-Z]+"
        "|FILE_BEGIN|FILE_CURRENT|FILE_END|FILE_[A-Z_]*INFORMATION"
        "|STD_[A-Z]+_HANDLE|MAX_PATH|MOVEFILE_[A-Z_]+|DUPLICATE_[A-Z_]+"
        # ── dir.c ───────────────────────────────────────────────────────────
        # stdlib.h's path-component maxima, which size every buffer `_splitpath`,
        # `_makepath` and `_wsplitpath` write into. Deriving matters: _MAX_DIR is 256
        # and _MAX_PATH 260, so a transcription that reuses one for the other overflows
        # by exactly four bytes and only on the longest paths.
        "|_MAX_DRIVE|_MAX_DIR|_MAX_FNAME|_MAX_EXT|_MAX_PATH"
        # winbase.h's GetDriveTypeW results — `_getdrives`/`_chdrive` dispatch on them.
        "|DRIVE_[A-Z_]+"
        "|ERROR_[A-Z0-9_]+|NO_ERROR"
        "|RTL_CRITICAL_SECTION_FLAG_[A-Z_]+"
        # mtdll.h's lock numbering (`_IOB_ENTRIES` is above, with the errno block).
        # setypes.h's DELETE is the access right file.c asks for when opening a file it
        # intends to unlink; windef.h's TRUE/FALSE are BOOL, not Rust bools.
        "|_STREAM_LOCKS|_TOTAL_LOCKS|_LAST_STREAM_LOCK|DELETE|TRUE|FALSE"
        # ── math.c ──────────────────────────────────────────────────────────
        # float.h's control/status-word vocabulary, whole families rather than the
        # names one wave happens to touch: `_setfp` and `_setfp_sse` translate between
        # these and the raw x87/MXCSR/FPCR bits in BOTH directions, so any one of them
        # arriving as a literal is a mask landing in the wrong field, and no compiler
        # or boot will say so. math.h's classification results come along for _fpclass
        # and the sqrt validators.
        "|_MCW_[A-Z]+|_EM_[A-Z]+|_SW_[A-Z]+|_RC_[A-Z]+|_PC_[0-9]+|_DN_[A-Z_]+|_IC_[A-Z]+"
        "|_FPCLASS_[A-Z]+|FP_[A-Z]+|_CVTBUFSIZE|_FPE_[A-Z]+"
        # winnt.h: the KUSER_SHARED_DATA feature slot `msvcrt_init_math` reads to decide
        # whether SSE2 exists. An index, so a wrong value reads a different feature bit.
        "|PF_[A-Z0-9_]+"
        # ── process.c ───────────────────────────────────────────────────────
        # process.h's spawn modes (_P_WAIT=0, _P_NOWAIT=1, _P_OVERLAY=2, _P_NOWAITO=3,
        # _P_DETACH=4 — five small integers in one family, i.e. exactly the shape a
        # permuted recall produces), mtdll.h's popen lock slot, and the winbase.h
        # process-creation flags `msvcrt_spawn` passes to CreateProcessW.
        "|_P_[A-Z]+|_POPEN_LOCK|STARTF_[A-Z_]+|DETACHED_PROCESS|INFINITE"
        "|WAIT_[A-Z_]+|STILL_ACTIVE|NORMAL_PRIORITY_CLASS"
        # ── time.c ──────────────────────────────────────────────────────────
        # winbase.h's three time-zone verdicts, and they are the §9.112 shape
        # exactly: `_ftime64` compares GetTimeZoneInformation's return against
        # TIME_ZONE_ID_DAYLIGHT to fill `dstflag`, and TIME_ZONE_ID_INVALID is
        # 0xFFFFFFFF — a `-1` written from host habit is a different value in a
        # DWORD comparison and would make every failure look like TIME_ZONE_ID_UNKNOWN.
        # `_TIME_LOCK` is mtdll.h's slot number for _tzset's critical section, in the
        # same family as the _STREAM_LOCKS above and wrong by silence if guessed.
        "|TIME_ZONE_ID_[A-Z]+|_TIME_LOCK|CLOCKS_PER_SEC|_MAX__TIME64_T"
        # ── console.c ───────────────────────────────────────────────────────
        # wincon.h's INPUT_RECORD vocabulary. `handle_enhanced_keys` dispatches a
        # four-way `idx` on the ALT/CTRL/SHIFT modifier bits and indexes a 10x4x2 table
        # with it, so one wrong bit silently returns a different scan code — the same
        # shape as the _P_* family above, and unobservable without a real keyboard.
        "|KEY_EVENT|MOUSE_EVENT|WINDOW_BUFFER_SIZE_EVENT|MENU_EVENT|FOCUS_EVENT"
        "|LEFT_ALT_PRESSED|RIGHT_ALT_PRESSED|LEFT_CTRL_PRESSED|RIGHT_CTRL_PRESSED"
        "|SHIFT_PRESSED|ENABLE_[A-Z_]+"
        # ── exit.c / misc.c ─────────────────────────────────────────────────
        # stdlib.h's `_set_error_mode`/`_set_abort_behavior` vocabulary and signal.h's
        # SIGABRT. `_OUT_TO_DEFAULT`/`_OUT_TO_STDERR`/`_OUT_TO_MSGBOX` are 0/1/2 and
        # `_REPORT_ERRMODE` is 3 — four consecutive small integers in one family, which
        # is the permuted-recall shape, and `_EXIT_LOCK1` already arrived that way as
        # `_CONIO_LOCK`'s 8. winnt.h's page protections back `_resetstkoflw`, and
        # `_TWO_DIGIT_EXPONENT` is the only legal argument to `_set_output_format`.
        "|_OUT_TO_[A-Z]+|_REPORT_ERRMODE|_WRITE_ABORT_MSG|_CALL_REPORTFAULT"
        "|_TWO_DIGIT_EXPONENT|SIGABRT|RAND_MAX|PAGE_[A-Z_]+|IDCANCEL|LANG_NEUTRAL"
        "|DLL_PROCESS_DETACH|DLL_PROCESS_ATTACH|DLL_THREAD_ATTACH|DLL_THREAD_DETACH"
        "|_CONIO_LOCK|_EXIT_LOCK1|_EXIT_LOCK2"
        # ── locale.c ────────────────────────────────────────────────────────
        # locale.h's category numbering (LC_ALL=0 .. LC_TIME=5, with LC_MIN/LC_MAX
        # bounding them) indexes six parallel arrays including the exported
        # `MSVCRT___lc_handle[LC_MAX-LC_MIN+1]`, so an off-by-one here silently
        # mis-files a whole locale category. Plus winnt.h's language ids that
        # `remap_synonym` special-cases, and mtdll.h's setlocale lock slot.
        "|LC_[A-Z]+|LANG_[A-Z_]+|SUBLANG_[A-Z_]+|SORT_[A-Z_]+|UCHAR_MAX|_SETLOCALE_LOCK"
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
        lines.append(f"char __vibeos_off_{i}[offsetof(struct {tag}, {field}) + 1];")
    src = "\n".join(lines) + "\n"

    with tempfile.TemporaryDirectory() as td:
        c = Path(td) / "probe.c"
        c.write_text(src)
        res = subprocess.run(
            [
                probe_cc(arch),
                "-S",
                "-emit-llvm",
                "-o",
                "-",
                str(c),
                "-ferror-limit=0",
                "-Wno-everything",
                *probe_flags(arch),
            ],
            capture_output=True,
            text=True,
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
        '        "msvcrt-sys layout drift: " #tag "." #field " moved -- rerun " \\',
        '        "sdk/lib/rust/msvcrt-sys/generate_bindings.py")',
        "",
    ]
    guards = {
        "i386": "defined(_M_IX86)",
        "amd64": "defined(_M_AMD64)",
        "arm64": "defined(_M_ARM64)",
    }
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
            print(
                f"  {len(offs)} guarded offsets, e.g. "
                f"threadlocinfo.pcumap @ {offs[('threadlocaleinfostruct', 'pcumap')]}"
            )
            pending.append((SRC_DIR / f"bindgen_{arch}.rs", bindings))
            pending.append((SRC_DIR / f"layout_{arch}.rs", rust_layout(arch, offs)))

        pending.append((REPO / "dll/win32/msvcrt/rust_layout_assert.c", c_guardian(per_arch)))

    stale = [p for p, content in pending if not p.exists() or p.read_text() != content]
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
