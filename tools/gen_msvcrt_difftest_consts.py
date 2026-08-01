#!/usr/bin/env python3
"""Derive the CRT/Win32 constants the msvcrt-difftest reference TUs need.

`file.c` (and the rest of the OS-coupled msvcrt tail) is compiled for the HOST
as a reference TU, against `refstub/` rather than the tree's own headers — the
MSVC CRT headers and glibc's cannot share a translation unit.  That leaves ~90
constants undefined: the `_O_*` / `_S_*` / `_SH_*` / `_LK_*` / `_IO*` families,
the Win32 file-API constants, and three NT enum values.

**Do not hand-transcribe them.**  Two traps make that unsafe, and both are live
in this tree:

  * `_IOMYBUF` is `0x0040` in msvcrt.h's `_MSVCR_VER >= 140` block and `0x0008`
    in the header the build actually reaches.  msvcrt compiles at
    `_MSVCR_VER=0`, so `0x0008` is correct — and a transcription that consulted
    the wrong block would be silently wrong, not a build error.
  * msvcrt REPLACES the global `sdk/include/crt` with its own private
    `dll/win32/msvcrt/include` (see dll/win32/msvcrt/CMakeLists.txt).  Harvesting
    from `sdk/include/crt` reads headers the build never sees — the §9.66 class,
    "the oracle is not the code under test".

So every value here is computed by the PINNED CROSS-COMPILER against msvcrt's
REAL include set and compile definitions, then read back out of the object file.
The compiler is the authority; this script only transports the answer.  That
also covers the three NT enum constants, which a `-dM` macro dump cannot see.

Usage:
    python3 tools/gen_msvcrt_difftest_consts.py            # write the header
    python3 tools/gen_msvcrt_difftest_consts.py --check    # fail on drift

`--check` is the gate: it re-derives and diffs against the committed file, so a
header change in the tree that moves a constant fails loudly instead of silently
regrading the reference against a value the DLL no longer uses.
"""

from __future__ import annotations

import argparse
import struct
import subprocess
import sys
import tempfile
from pathlib import Path

REPO = Path(__file__).resolve().parent.parent
CLANG = REPO / ".build-tools" / "llvm-mingw" / "bin" / "clang"
OBJDUMP = REPO / ".build-tools" / "llvm-mingw" / "bin" / "llvm-objdump"
OUT_H = REPO / "sdk/lib/rust/msvcrt-difftest/refstub/msvcrt_crt_consts.h"

# A build dir supplies the generated headers (sdk/include/reactos/mc &c.).
# Fleet isolation law: THIS worktree's build dir only, never a sibling's.
BUILD_DIR = REPO / "build-amd64"

# The headers whose constants we need, in the order file.c includes them.
PROBE_INCLUDES = [
    "errno.h",
    "fcntl.h",
    "share.h",
    "stdio.h",
    "sys/stat.h",
    "sys/locking.h",
    "sys/utime.h",
    "io.h",
    '"windef.h"',
    '"winbase.h"',
    '"winnls.h"',
    '"winternl.h"',
]

# ── the allowlist ────────────────────────────────────────────────────────
#
# Explicit, not "every macro in those headers", for two reasons: emitting all
# ~600 would collide with glibc's own <stdio.h>/<fcntl.h> in the host TU, and an
# explicit list means a constant file.c starts using tomorrow is an UNDEFINED
# IDENTIFIER at compile time — loud — rather than a silent pickup.
#
# Grouped as the reference TU sees them.  Add a name here when the compile says
# it is undeclared; never add one speculatively.
CONSTANTS: dict[str, list[str]] = {
    "open / share / lock modes (msvcrt's own fcntl.h, share.h, sys/locking.h)": [
        "_O_APPEND",
        "_O_BINARY",
        "_O_CREAT",
        "_O_EXCL",
        "_O_NOINHERIT",
        "_O_RANDOM",
        "_O_RDONLY",
        "_O_RDWR",
        "_O_SEQUENTIAL",
        "_O_SHORT_LIVED",
        "_O_TEMPORARY",
        "_O_TEXT",
        "_O_TRUNC",
        "_O_U16TEXT",
        "_O_U8TEXT",
        "_O_WRONLY",
        "_O_WTEXT",
        "_SH_DENYNO",
        "_SH_DENYRD",
        "_SH_DENYRW",
        "_SH_DENYWR",
        "_LK_LOCK",
        "_LK_RLCK",
        "_LK_UNLCK",
    ],
    "stat mode bits (sys/stat.h)": [
        "_S_IEXEC",
        "_S_IFCHR",
        "_S_IFDIR",
        "_S_IFIFO",
        "_S_IFREG",
        "_S_IREAD",
        "_S_IWRITE",
    ],
    "FILE._flag bits — NOTE the _MSVCR_VER trap in this file's docstring": [
        "_IOEOF",
        "_IOERR",
        "_IOMYBUF",
        "_IOREAD",
        "_IORW",
        "_IOSTRG",
        "_IOWRT",
    ],
    # setvbuf modes, and the one that is not merely a mode: at _MSVCR_VER < 140 —
    # the arm this DLL builds — msvcrt.h spells `MSVCRT__NOBUF` as `_IONBF`, so
    # this value IS the "stream is unbuffered" flag bit that file.c tests in
    # _fwrite_nolock, _flsbuf, _filbuf and fflush.  msvcrt's _IONBF is 0x0004;
    # glibc's is 2 — which is _IOWRT.  A reference TU that took glibc's therefore
    # read EVERY write-mode stream as unbuffered, wrote a byte at a time straight
    # to the fd, and never allocated a buffer at all: the subject buffered, the
    # reference did not, and the two file positions came out 9 apart with
    # identical bytes on disk.  Derived and displaced for exactly the reason the
    # errno block below is.
    "setvbuf modes — _IONBF doubles as MSVCRT__NOBUF at _MSVCR_VER < 140": [
        "_IOFBF",
        "_IOLBF",
        "_IONBF",
    ],
    "stdio / io.h": [
        "TMP_MAX_S",
        "STDIN_FILENO",
        "STDOUT_FILENO",
        "STDERR_FILENO",
    ],
    "Win32 file API (psdk)": [
        "CREATE_ALWAYS",
        "CREATE_NEW",
        "OPEN_ALWAYS",
        "OPEN_EXISTING",
        "TRUNCATE_EXISTING",
        "GENERIC_READ",
        "GENERIC_WRITE",
        "DELETE",
        "FILE_SHARE_DELETE",
        "FILE_SHARE_READ",
        "FILE_SHARE_WRITE",
        "FILE_ATTRIBUTE_DIRECTORY",
        "FILE_ATTRIBUTE_NORMAL",
        "FILE_ATTRIBUTE_READONLY",
        "FILE_ATTRIBUTE_TEMPORARY",
        "FILE_FLAG_DELETE_ON_CLOSE",
        "FILE_FLAG_RANDOM_ACCESS",
        "FILE_FLAG_SEQUENTIAL_SCAN",
        "FILE_BEGIN",
        "FILE_CURRENT",
        "FILE_END",
        "FILE_TYPE_CHAR",
        "FILE_TYPE_DISK",
        "FILE_TYPE_PIPE",
        "FILE_TYPE_UNKNOWN",
        "INVALID_FILE_ATTRIBUTES",
        "INVALID_SET_FILE_POINTER",
        "MOVEFILE_COPY_ALLOWED",
        "MAX_PATH",
        "DUPLICATE_SAME_ACCESS",
        "STD_ERROR_HANDLE",
        "STD_INPUT_HANDLE",
        "STD_OUTPUT_HANDLE",
        "RTL_CRITICAL_SECTION_FLAG_FORCE_DEBUG_INFO",
    ],
    "codepage conversion — msvcrt.h's get_aw_cp / convert_*_utf8 inlines": [
        "CP_ACP",
        "MB_PRECOMPOSED",
    ],
    "Win32 error codes": [
        "ERROR_SUCCESS",
        "NO_ERROR",
        "ERROR_ACCESS_DENIED",
        "ERROR_BROKEN_PIPE",
        "ERROR_FILE_NOT_FOUND",
        "ERROR_INVALID_HANDLE",
        "ERROR_INVALID_PARAMETER",
    ],
    # Every case label in errno.c's msvcrt_set_errno, which refimpl.c
    # transcribes.  A missing one would not be a compile error in a `switch` —
    # it would silently fall through to the `default: EINVAL` arm, which is
    # §9.112's catch-all-match class wearing C clothes.
    "Win32 error codes — errno.c's msvcrt_set_errno case labels": [
        "ERROR_ALREADY_EXISTS",
        "ERROR_ARENA_TRASHED",
        "ERROR_BAD_DEVICE",
        "ERROR_BAD_ENVIRONMENT",
        "ERROR_BAD_FORMAT",
        "ERROR_BAD_NETPATH",
        "ERROR_BAD_NET_NAME",
        "ERROR_BAD_PATHNAME",
        "ERROR_BUSY",
        "ERROR_CANNOT_MAKE",
        "ERROR_CHILD_NOT_COMPLETE",
        "ERROR_CURRENT_DIRECTORY",
        "ERROR_DIR_NOT_EMPTY",
        "ERROR_DISK_FULL",
        "ERROR_DRIVE_LOCKED",
        "ERROR_FAIL_I24",
        "ERROR_FILENAME_EXCED_RANGE",
        "ERROR_FILE_EXISTS",
        "ERROR_INVALID_ACCESS",
        "ERROR_INVALID_BLOCK",
        "ERROR_INVALID_DRIVE",
        "ERROR_IO_DEVICE",
        "ERROR_LOCK_FAILED",
        "ERROR_LOCK_VIOLATION",
        "ERROR_MAX_THRDS_REACHED",
        "ERROR_NESTING_NOT_ALLOWED",
        "ERROR_NETWORK_ACCESS_DENIED",
        "ERROR_NOT_ENOUGH_QUOTA",
        "ERROR_NOT_LOCKED",
        "ERROR_NO_MORE_FILES",
        "ERROR_NO_PROC_SLOTS",
        "ERROR_OUTOFMEMORY",
        "ERROR_PATH_NOT_FOUND",
        "ERROR_POSSIBLE_DEADLOCK",
        "ERROR_SEEK_ON_DEVICE",
        "ERROR_SHARING_VIOLATION",
        "ERROR_TOO_MANY_OPEN_FILES",
        "ERROR_WAIT_NO_CHILDREN",
    ],
    "Win32 error codes — raised by the POSIX-backed shims in refimpl.c": [
        "ERROR_HANDLE_EOF",
        "ERROR_NEGATIVE_SEEK",
        "ERROR_NOT_ENOUGH_MEMORY",
        "ERROR_NOT_SUPPORTED",
        "ERROR_SEEK",
        "ERROR_WRITE_FAULT",
    ],
    # msvcrt's own <errno.h>, NOT glibc's — see DISPLACE for why that matters.
    "CRT errno values (msvcrt's errno.h; these DISPLACE the host's)": [
        "E2BIG",
        "EACCES",
        "EAGAIN",
        "EBADF",
        "EBUSY",
        "ECHILD",
        "EDEADLK",
        "EEXIST",
        "EINVAL",
        "EIO",
        "EMFILE",
        "ENODEV",
        "ENOENT",
        "ENOEXEC",
        "ENOMEM",
        "ENOSPC",
        "ENOTEMPTY",
        "EPIPE",
    ],
    "NT enum constants — invisible to a -dM macro dump, hence the object probe": [
        "FileBasicInformation",
        "FileStandardInformation",
        "GetFileExInfoStandard",
    ],
}

# Names the HOST libc also defines, which must be #undef'd before ours lands.
#
# Not a formatting detail — it is the whole point for the errno family.  msvcrt
# and glibc agree on most of <errno.h> and disagree on two of the values file.c
# can actually produce: EDEADLK is 36 on msvcrt and 35 on glibc, ENOTEMPTY is
# 41 and 39.  A reference TU that took glibc's would return the wrong errno from
# every path that maps ERROR_POSSIBLE_DEADLOCK or ERROR_DIR_NOT_EMPTY, while
# every other errno row kept passing — the §9.66 shape exactly.  Deriving them
# from msvcrt's own headers and displacing glibc's is what makes that impossible.
DISPLACE = {
    # glibc numbers these 0/1/2 and msvcrt 0x0000/0x0040/0x0004. The clash that
    # matters is _IONBF: msvcrt.h aliases MSVCRT__NOBUF to it below _MSVCR_VER 140,
    # and glibc's 2 collides bit-for-bit with _IOWRT.
    "_IOFBF",
    "_IOLBF",
    "_IONBF",
    "E2BIG",
    "EACCES",
    "EAGAIN",
    "EBADF",
    "EBUSY",
    "ECHILD",
    "EDEADLK",
    "EEXIST",
    "EINVAL",
    "EIO",
    "EMFILE",
    "ENODEV",
    "ENOENT",
    "ENOEXEC",
    "ENOMEM",
    "ENOSPC",
    "ENOTEMPTY",
    "EPIPE",
}

# Constants whose value is a POINTER cast in the headers (`((HANDLE)(LONG_PTR)-1`)
# and so cannot be an integer initializer.  Emitted verbatim instead of harvested;
# each is a Windows ABI constant that has never varied.
VERBATIM = {
    "INVALID_HANDLE_VALUE": "((HANDLE)(LONG_PTR)-1)",
}

# ── the layout guardian ──────────────────────────────────────────────────
#
# A struct definition is not an integer, so the layouts in
# refstub/msvcrt_file_types.h are TRANSCRIBED rather than derived.  What makes
# that safe is this: every offset and size below is harvested from the pinned
# cross-compiler reading msvcrt's REAL headers, and emitted as a
# `_Static_assert` the HOST compiler checks against the transcription.  The two
# compilers have to agree or the reference TU does not build.
#
# `_dev_t` is the live example — `unsigned short` under _CRTDLL and
# `unsigned int` without, which shifts every field after it.
LAYOUTS: dict[str, list[str]] = {
    # FILE is the sharpest case in the set.  msvcrt.h's `_MSVCR_VER >= 140`
    # block and corecrt_wstdio.h declare the same eight fields in DIFFERENT
    # ORDER, and only the latter is compiled at _MSVCR_VER=0.  file.c reads
    # every one of these directly, so the wrong order is eight silently wrong
    # fields — it is how the first draft of msvcrt_file_prelude.h was written.
    "FILE": ["_ptr", "_cnt", "_base", "_flag", "_file", "_charbuf", "_bufsiz", "_tmpfname"],
    "struct _stat32": [
        "st_dev",
        "st_ino",
        "st_mode",
        "st_nlink",
        "st_uid",
        "st_gid",
        "st_rdev",
        "st_size",
        "st_atime",
        "st_mtime",
        "st_ctime",
    ],
    "struct _stat32i64": [
        "st_dev",
        "st_ino",
        "st_mode",
        "st_nlink",
        "st_uid",
        "st_gid",
        "st_rdev",
        "st_size",
        "st_atime",
        "st_mtime",
        "st_ctime",
    ],
    "struct _stat64i32": [
        "st_dev",
        "st_ino",
        "st_mode",
        "st_nlink",
        "st_uid",
        "st_gid",
        "st_rdev",
        "st_size",
        "st_atime",
        "st_mtime",
        "st_ctime",
    ],
    "struct _stat64": [
        "st_dev",
        "st_ino",
        "st_mode",
        "st_nlink",
        "st_uid",
        "st_gid",
        "st_rdev",
        "st_size",
        "st_atime",
        "st_mtime",
        "st_ctime",
    ],
    "struct _stat": [
        "st_dev",
        "st_ino",
        "st_mode",
        "st_nlink",
        "st_uid",
        "st_gid",
        "st_rdev",
        "st_size",
        "st_atime",
        "st_mtime",
        "st_ctime",
    ],
    "struct _stati64": [
        "st_dev",
        "st_ino",
        "st_mode",
        "st_nlink",
        "st_uid",
        "st_gid",
        "st_rdev",
        "st_size",
        "st_atime",
        "st_mtime",
        "st_ctime",
    ],
    "struct __utimbuf32": ["actime", "modtime"],
    "struct __utimbuf64": ["actime", "modtime"],
    "FILETIME": ["dwLowDateTime", "dwHighDateTime"],
    "WIN32_FILE_ATTRIBUTE_DATA": [
        "dwFileAttributes",
        "ftCreationTime",
        "ftLastAccessTime",
        "ftLastWriteTime",
        "nFileSizeHigh",
        "nFileSizeLow",
    ],
    "FILE_BASIC_INFORMATION": [
        "CreationTime",
        "LastAccessTime",
        "LastWriteTime",
        "ChangeTime",
        "FileAttributes",
    ],
    "FILE_STANDARD_INFORMATION": [
        "AllocationSize",
        "EndOfFile",
        "NumberOfLinks",
        "DeletePending",
        "Directory",
    ],
    "IO_STATUS_BLOCK": ["Information"],
    "LARGE_INTEGER": ["QuadPart"],
    # file.c statically initializes Spare[0] with a string ADDRESS, so this
    # one's element width is load-bearing: DWORD[2] (the first transcription)
    # makes that a truncating conversion the compiler refuses.
    "CRITICAL_SECTION_DEBUG": [
        "Type",
        "CreatorBackTraceIndex",
        "CriticalSection",
        "ProcessLocksList",
        "EntryCount",
        "ContentionCount",
        "Spare",
    ],
    "CRITICAL_SECTION": [
        "DebugInfo",
        "LockCount",
        "RecursionCount",
        "OwningThread",
        "LockSemaphore",
        "SpinCount",
    ],
    "STARTUPINFOA": [
        "cb",
        "lpReserved",
        "dwFlags",
        "wShowWindow",
        "cbReserved2",
        "lpReserved2",
        "hStdInput",
        "hStdOutput",
        "hStdError",
    ],
    "SECURITY_ATTRIBUTES": ["nLength", "lpSecurityDescriptor", "bInheritHandle"],
}

# Scalar widths the layouts depend on.  Harvested for the same reason.
SCALAR_SIZES = ["_dev_t", "_ino_t", "_off_t", "__time32_t", "__time64_t"]

OUT_LAYOUT_H = REPO / "sdk/lib/rust/msvcrt-difftest/refstub/msvcrt_crt_layout.h"


def include_flags() -> list[str]:
    """msvcrt's effective include path, mirroring dll/win32/msvcrt/CMakeLists.txt.

    Order matters and is not the global order: `include_directories(BEFORE
    sdk/include/wine)` prepends, the global list has `sdk/include/crt` REMOVED,
    and `include_directories(include)` appends msvcrt's private CRT headers.
    """
    r, b = REPO, BUILD_DIR
    dirs = [
        r / "sdk/include/wine",  # BEFORE
        # the global set, minus sdk/include/crt (removed by msvcrt's CMakeLists)
        r / "sdk/include",
        r / "sdk/include/ddk",
        r / "sdk/include/ndk",
        r / "sdk/include/psdk",
        r / "sdk/include/reactos",
        r / "sdk/include/reactos/libs",
        r / "sdk/include/vcruntime",
        r / "sdk/include/winrt",
        b / "sdk/include",
        b / "sdk/include/psdk",
        b / "sdk/include/ddk",
        b / "sdk/include/dxsdk",
        b / "sdk/include/reactos",
        b / "sdk/include/reactos/mc",
        r / "sdk/include/dxsdk",
        r / "sdk/lib/pseh/include",
        r / "dll/win32/msvcrt/include",  # msvcrt's OWN CRT headers
        r / "dll/win32/msvcrt",
    ]
    return [f"-I{d}" for d in dirs]


def _probe_exprs() -> list[tuple[str, str]]:
    """(key, C-expression) for everything the object probe measures.

    Constants, then every field offset, then every struct and scalar size —
    all in one TU, so one compile answers the whole substrate.
    """
    items: list[tuple[str, str]] = []
    for group in CONSTANTS.values():
        for n in group:
            items.append((f"const:{n}", n))
    for struct_name, fields in LAYOUTS.items():
        for f in fields:
            items.append((f"off:{struct_name}:{f}", f"__builtin_offsetof({struct_name}, {f})"))
        items.append((f"size:{struct_name}", f"sizeof({struct_name})"))
    for s in SCALAR_SIZES:
        items.append((f"size:{s}", f"sizeof({s})"))
    return items


def harvest() -> dict[str, int]:
    """Compile a probe TU and read the values the real compiler computed."""
    if not CLANG.exists():
        sys.exit(
            f"FAIL: pinned clang not found at {CLANG}\n"
            f"fix: the llvm-mingw toolchain must be provisioned"
        )
    if not (BUILD_DIR / "sdk/include/reactos").is_dir():
        sys.exit(
            f"FAIL: {BUILD_DIR} has no generated headers.\n"
            f"fix: ./vibeos-cli build --arch amd64\n"
            f"(this worktree's build dir only — never a sibling's)"
        )

    items = _probe_exprs()
    names = [k for k, _ in items]
    src = "\n".join(
        f"#include {inc}" if inc.startswith('"') else f"#include <{inc}>" for inc in PROBE_INCLUDES
    )
    # `long long` so a 32-bit constant with the top bit set (GENERIC_READ) is
    # zero-extended rather than sign-extended into the slot.
    src += "\n__attribute__((used)) const long long __harvest[] = {\n"
    src += "".join(f"  (long long)({expr}),\n" for _, expr in items)
    src += "};\n"

    with tempfile.TemporaryDirectory() as td:
        c, o = Path(td) / "probe.c", Path(td) / "probe.o"
        c.write_text(src)
        cmd = [
            str(CLANG),
            "-target",
            "x86_64-w64-mingw32",
            "-c",
            "-o",
            str(o),
            str(c),
            "-nostdinc",
            "-w",
            *include_flags(),
            "-D_MSVCR_VER=0",
            "-D_WIN32_WINNT=0x601",
            "-DWINVER=0x601",
            "-D_CRTIMP=",
            "-D__REACTOS__",
        ]
        p = subprocess.run(cmd, capture_output=True, text=True)
        if p.returncode != 0:
            sys.exit(
                "FAIL: the substrate probe did not compile.\n"
                "A name in CONSTANTS/LAYOUTS is misspelled, or its header "
                "moved.\n\n" + p.stderr[-3000:]
            )

        d = subprocess.run(
            [str(OBJDUMP), "-s", "-j", ".rdata", str(o)], capture_output=True, text=True
        )
        # `0000 00010000 00000000 00800000 00000000  ................`
        # Offset-addressed rather than append-ordered: the trailing ASCII column
        # can itself look like a hex word, and the last line is short.  Placing
        # each word AT its stated offset makes both harmless.
        blob = bytearray()
        for line in d.stdout.splitlines():
            parts = line.split()
            if len(parts) < 2 or not _ishex(parts[0]):
                continue
            off = int(parts[0], 16)
            for i, w in enumerate(parts[1:5]):
                if len(w) != 8 or not _ishex(w):
                    break
                at = off + i * 4
                if len(blob) < at + 4:
                    blob.extend(b"\0" * (at + 4 - len(blob)))
                blob[at : at + 4] = bytes.fromhex(w)

    want = len(names) * 8
    if len(blob) < want:
        sys.exit(
            f"FAIL: probe .rdata held {len(blob)} bytes, need {want}. objdump output shape changed."
        )
    vals = struct.unpack(f"<{len(names)}q", bytes(blob[:want]))
    return dict(zip(names, vals, strict=True))


def _ishex(s: str) -> bool:
    return all(ch in "0123456789abcdefABCDEF" for ch in s)


def render_layout(values: dict[str, int]) -> str:
    """A wall of _Static_assert the HOST compiler checks against the transcription."""
    out = [
        "/* GENERATED by tools/gen_msvcrt_difftest_consts.py — DO NOT EDIT.",
        " *",
        " * The layout guardian.  Every number below was measured by the PINNED",
        " * CROSS-COMPILER reading msvcrt's real headers; the host compiler checks",
        " * them against the transcribed definitions in msvcrt_file_types.h.  The",
        " * two must agree or the reference TU does not build — which is the whole",
        " * point: a struct layout cannot be derived into a header the way a",
        " * constant can, so it is transcribed and then PROVED.",
        " *",
        " * Include AFTER msvcrt_file_types.h.",
        " */",
        "#ifndef MSVCRT_DIFFTEST_CRT_LAYOUT_H",
        "#define MSVCRT_DIFFTEST_CRT_LAYOUT_H",
        "",
        "/* Scalar widths every layout below depends on. */",
    ]
    for s in SCALAR_SIZES:
        v = values[f"size:{s}"]
        out.append(f'_Static_assert(sizeof({s}) == {v}, "{s} width drifted");')
    out.append("")
    for struct_name, fields in LAYOUTS.items():
        ident = struct_name.replace("struct ", "")
        out.append(f"/* {struct_name} */")
        for f in fields:
            v = values[f"off:{struct_name}:{f}"]
            out.append(
                f"_Static_assert(__builtin_offsetof({struct_name}, {f}) == {v},"
                f' "{ident}.{f} moved");'
            )
        v = values[f"size:{struct_name}"]
        out.append(f'_Static_assert(sizeof({struct_name}) == {v}, "{ident} size drifted");')
        out.append("")
    out += ["#endif /* MSVCRT_DIFFTEST_CRT_LAYOUT_H */", ""]
    return "\n".join(out)


def render(values: dict[str, int]) -> str:
    out = [
        "/* GENERATED by tools/gen_msvcrt_difftest_consts.py — DO NOT EDIT.",
        " *",
        " * The CRT/Win32 constants the msvcrt-difftest reference TUs need, with",
        " * every value computed by the PINNED CROSS-COMPILER against msvcrt's real",
        " * include set at _MSVCR_VER=0 — the same model the shipping DLL builds",
        " * under.  Hand-transcription is unsafe here: _IOMYBUF differs between",
        " * msvcrt.h's _MSVCR_VER>=140 block and the header the build reaches, and",
        " * msvcrt substitutes its own CRT headers for sdk/include/crt.  See the",
        " * generator's docstring.",
        " *",
        " * Regenerate:  python3 tools/gen_msvcrt_difftest_consts.py",
        " * Drift gate:  python3 tools/gen_msvcrt_difftest_consts.py --check",
        " */",
        "#ifndef MSVCRT_DIFFTEST_CRT_CONSTS_H",
        "#define MSVCRT_DIFFTEST_CRT_CONSTS_H",
        "",
    ]
    for comment, names in CONSTANTS.items():
        out.append(f"/* {comment} */")
        width = max(len(n) for n in names)
        for n in names:
            v = values[f"const:{n}"]
            lit = f"0x{v:08X}" if v > 9 or v < 0 else str(v)
            suffix = "u" if v > 0x7FFFFFFF else ""
            if n in DISPLACE:
                out.append(f"#undef  {n}")
            out.append(f"#define {n:<{width}} {lit}{suffix}")
        out.append("")
    out.append("/* Pointer-valued; not an integer initializer, so emitted verbatim. */")
    for n, expr in VERBATIM.items():
        out.append(f"#define {n} {expr}")
    out += ["", "#endif /* MSVCRT_DIFFTEST_CRT_CONSTS_H */", ""]
    return "\n".join(out)


def main() -> int:
    ap = argparse.ArgumentParser(description=__doc__)
    ap.add_argument(
        "--check",
        action="store_true",
        help="fail if the committed header differs from a fresh derivation",
    )
    args = ap.parse_args()

    values = harvest()
    outputs = [(OUT_H, render(values)), (OUT_LAYOUT_H, render_layout(values))]

    n_const = sum(len(v) for v in CONSTANTS.values()) + len(VERBATIM)
    n_assert = len(SCALAR_SIZES) + sum(len(f) + 1 for f in LAYOUTS.values())

    if args.check:
        stale = [p for p, text in outputs if not p.exists() or p.read_text() != text]
        if stale:
            for p in stale:
                print(f"DRIFT: {p.relative_to(REPO)} is stale or missing")
            print("fix: python3 tools/gen_msvcrt_difftest_consts.py")
            return 1
        print(f"ok: {n_const} constants + {n_assert} layout assertions match a fresh derivation")
        return 0

    for p, text in outputs:
        p.parent.mkdir(parents=True, exist_ok=True)
        p.write_text(text)
    print(f"wrote {OUT_H.relative_to(REPO)} ({n_const} constants)")
    print(f"wrote {OUT_LAYOUT_H.relative_to(REPO)} ({n_assert} assertions)")
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
