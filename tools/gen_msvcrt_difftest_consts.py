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
    "time.h",
    "process.h",
    '"windef.h"',
    '"winbase.h"',
    '"winnls.h"',
    '"winternl.h"',
    # …and what the C++ EH pair (cpp.c, except.c) adds on top.  `signal.h` and
    # `float.h` are msvcrt's OWN, and that is the point: the host's <signal.h>
    # has SIGABRT at 6 where msvcrt has it at 22, and glibc's <float.h> has no
    # _FPE_* at all.  A reference TU that took glibc's numbering would index
    # except.c's handler table one slot off the subject's.
    "signal.h",
    "float.h",
    '"wincon.h"',
    '"ntstatus.h"',
    '"winver.h"',
    '"imagehlp.h"',
    # msvcrt's own, for `frame_info` / `cxx_frame_info` — the linked list except.c
    # walks.  Last, so nothing above it is compiled under msvcrt.h's own macros.
    '"msvcrt.h"',
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
        # msvcrt's BUFSIZ is 512; glibc's is 8192.  `setbuf` is literally
        # `setvbuf(file, buf, buf ? _IOFBF : _IONBF, BUFSIZ)`, so a reference TU
        # that took glibc's sized every setbuf'd stream 16x too large and every
        # graded row over it compared 512 against 8192.  Same shape as _IONBF
        # above, and found the same way: by a row that finally reached it.
        "BUFSIZ",
        "TMP_MAX_S",
        "STDIN_FILENO",
        "STDOUT_FILENO",
        "STDERR_FILENO",
    ],
    # `clock()` is literally `(now - init_time) / (TICKSPERSEC / CLOCKS_PER_SEC)`, so this
    # constant IS the unit of its return value.  msvcrt's is 1000 (milliseconds); glibc's
    # is 1000000.  A reference TU that took glibc's divides by 10 instead of 10000 and
    # answers in MICROseconds — every clock() row off by exactly 1000x, in a function whose
    # body is four lines and looks impossible to get wrong.  Same shape as BUFSIZ and
    # _IONBF above, and displaced for the same reason.
    "time (msvcrt's own time.h)": [
        "CLOCKS_PER_SEC",
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
    # process.c's `msvcrt_spawn` opens with `if ((unsigned)flags > _P_DETACH) return -1`
    # and then SWITCHES on the same value, so these five numbers decide which arm every
    # `_spawn*` call takes — and they are not in the obvious order.  msvcrt's process.h
    # numbers them WAIT 0, NOWAIT 1, OVERLAY 2, NOWAITO 3, DETACH 4, so `_P_OVERLAY`
    # (what every `_exec*` passes) sits in the MIDDLE of the valid range rather than at
    # an end.  A transcription that guessed the POSIX-ish order would put the range check
    # in a different place and every rejected-flags row would still read as MATCH.
    "spawn modes (msvcrt's own process.h)": [
        "_P_WAIT",
        "_P_NOWAIT",
        "_P_OVERLAY",
        "_P_NOWAITO",
        "_P_DETACH",
    ],
    # The CreateProcessW / WaitForSingleObject vocabulary processimpl.c answers in.
    # `CREATE_UNICODE_ENVIRONMENT` is the one that has to be right rather than merely
    # present: msvcrt_spawn ORs it into every single spawn, and `DETACHED_PROCESS` only
    # on the `_P_DETACH` arm, so a wrong value for either makes the recorded creation
    # flags agree between two sides that are both reading our number.
    "Win32 process API (psdk)": [
        "CREATE_UNICODE_ENVIRONMENT",
        "DETACHED_PROCESS",
        "STARTF_USESTDHANDLES",
        "INFINITE",
        "WAIT_OBJECT_0",
        "WAIT_TIMEOUT",
        "WAIT_FAILED",
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
        # timeimpl.c's GetDateFormatW/GetTimeFormatW report a too-small
        # output buffer with this, which is what the real ones do.
        "ERROR_INSUFFICIENT_BUFFER",
        # processimpl.c: what an unset COMSPEC and a missing export really
        # report.  `_unloaddll` RETURNS GetLastError() to its caller, so this
        # value is not diagnostic decoration — it is the function's result.
        "ERROR_ENVVAR_NOT_FOUND",
        "ERROR_PROC_NOT_FOUND",
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
        # except.c returns these out of __CxxFrameHandler and the SEH filters.
        "ExceptionContinueSearch",
        "ExceptionContinueExecution",
        "ExceptionNestedException",
        "ExceptionCollidedUnwind",
    ],
    "signal numbers (msvcrt's OWN signal.h — these DISPLACE the host's)": [
        "SIGABRT",
        "SIGBREAK",
        "SIGFPE",
        "SIGILL",
        "SIGINT",
        "SIGSEGV",
        "SIGTERM",
        "NSIG",
    ],
    "_FPE_* subcodes (msvcrt's float.h) — the second argument to a SIGFPE handler": [
        "_FPE_DENORMAL",
        "_FPE_EXPLICITGEN",
        "_FPE_INEXACT",
        "_FPE_INVALID",
        "_FPE_OVERFLOW",
        "_FPE_STACKOVERFLOW",
        "_FPE_UNDERFLOW",
        "_FPE_ZERODIVIDE",
    ],
    "SEH filter dispositions and exception-record flags (winnt.h / excpt.h)": [
        "EXCEPTION_CONTINUE_EXECUTION",
        "EXCEPTION_CONTINUE_SEARCH",
        "EXCEPTION_EXECUTE_HANDLER",
        "EXCEPTION_NONCONTINUABLE",
        "EXCEPTION_UNWINDING",
        "EXCEPTION_EXIT_UNWIND",
        "EXCEPTION_TARGET_UNWIND",
        "EXCEPTION_MAXIMUM_PARAMETERS",
    ],
    "exception CODES — except.c's SIGFPE/SIGSEGV/SIGILL mapping switches on these": [
        "EXCEPTION_ACCESS_VIOLATION",
        "EXCEPTION_ILLEGAL_INSTRUCTION",
        "EXCEPTION_PRIV_INSTRUCTION",
        "EXCEPTION_FLT_DENORMAL_OPERAND",
        "EXCEPTION_FLT_DIVIDE_BY_ZERO",
        "EXCEPTION_FLT_INEXACT_RESULT",
        "EXCEPTION_FLT_INVALID_OPERATION",
        "EXCEPTION_FLT_OVERFLOW",
        "EXCEPTION_FLT_STACK_CHECK",
        "EXCEPTION_FLT_UNDERFLOW",
    ],
    "NTSTATUS values (ntstatus.h) — the same codes on the other side of the map": [
        "STATUS_ACCESS_VIOLATION",
        "STATUS_FLOAT_DIVIDE_BY_ZERO",
        "STATUS_FLOAT_INEXACT_RESULT",
        "STATUS_FLOAT_INVALID_OPERATION",
        "STATUS_FLOAT_OVERFLOW",
        "STATUS_FLOAT_UNDERFLOW",
        "STATUS_LONGJUMP",
        "STATUS_UNWIND_CONSOLIDATE",
    ],
    "console control events (wincon.h) — except.c's Ctrl-C handler": [
        "CTRL_C_EVENT",
        "CTRL_BREAK_EVENT",
    ],
    "undecoration flags (imagehlp.h) — cpp.c's type_info::name() calls __unDName": [
        "UNDNAME_NO_ARGUMENTS",
        "UNDNAME_32_BIT_DECODE",
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
    # glibc 8192 vs msvcrt 512 — see the stdio/io.h group above.
    "BUFSIZ",
    # glibc 1000000 vs msvcrt 1000 — see the time group above.
    "CLOCKS_PER_SEC",
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
    # The signal family, and it is the sharpest displacement in the set after
    # errno's: msvcrt numbers SIGABRT 22 where glibc numbers it 6, and NSIG 23
    # where glibc says 65.  except.c's `signal()` bounds-checks against NSIG and
    # then INDEXES a per-signal handler table, so glibc's numbering would file
    # the reference's abort handler in a different slot from the subject's and
    # every other signal row would keep passing.
    "SIGABRT",
    "SIGFPE",
    "SIGILL",
    "SIGINT",
    "SIGSEGV",
    "SIGTERM",
    "NSIG",
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
    # The wide twin of STARTUPINFOA, and NOT a copy of it: three `char *` become
    # `wchar_t *`, which on a 32-bit target changes nothing and on a 64-bit one changes
    # nothing either — but `cbReserved2`/`lpReserved2` are what msvcrt_spawn fills from
    # `msvcrt_create_io_inherit_block`, so the pair's offsets decide whether the recorder
    # reads the inherit block or two bytes of padding.
    "STARTUPINFOW": [
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
    # msvcrt_spawn's `_P_WAIT` arm calls `GetExitCodeProcess(pi.hProcess, &pi.dwProcessId)`
    # — it writes the EXIT CODE into the PID field and returns that — so this struct's
    # field order is load-bearing logic, not bookkeeping.
    "PROCESS_INFORMATION": ["hProcess", "hThread", "dwProcessId", "dwThreadId"],
}

# Scalar widths the layouts depend on.  Harvested for the same reason.
#
# CONTEXT is here rather than in LAYOUTS on purpose.  Neither reference TU reads
# a single field of it: except.c zeroes one on the stack and hands the address to
# RtlUnwindEx, and stores CONTEXT pointers it never dereferences.  So the refstub
# models it as a correctly sized, correctly aligned OPAQUE BLOCK and this size is
# what proves the block.  The day a row needs `Rip`, that block has to become
# fields and this entry moves up into LAYOUTS — recording the reach rather than
# transcribing 40 registers nothing reads.
SCALAR_SIZES = ["_dev_t", "_ino_t", "_off_t", "__time32_t", "__time64_t"]

# ── the SECOND layout wall: the C++ EH substrate ─────────────────────────
#
# Its own dict, and its own output header, because a wall is only checkable by a
# TU that has every type in it.  msvcrt_crt_layout.h asserts over FILE and the
# _stat family, which only the file.c reference TU defines; the cpp.c/except.c
# pair defines neither and would fail on every line of it.  Splitting the wall by
# WHICH TU CAN CHECK IT is what keeps each one fully checked instead of partially
# included.
CPP_LAYOUTS: dict[str, list[str]] = {
    # ── the C++ EH pair (cpp.c reference TU #14, except.c reference TU #15) ──
    #
    # EXCEPTION_RECORD is the sharpest of these.  `_CxxThrowException` FILLS ONE
    # IN BY HAND and hands it to RaiseException, and `is_cxx_exception` reads
    # ExceptionInformation[0] back out — so a wrong `ExceptionInformation` offset
    # is not a crash, it is a throw that carries the magic in the wrong slot and
    # a catch that silently declines to match.  ExceptionInformation is also a
    # FLEXIBLE-ARRAY-shaped tail (EXCEPTION_MAXIMUM_PARAMETERS entries), i.e. the
    # exact class task #51 found hiding.
    "EXCEPTION_RECORD": [
        "ExceptionCode",
        "ExceptionFlags",
        "ExceptionRecord",
        "ExceptionAddress",
        "NumberParameters",
        "ExceptionInformation",
    ],
    "EXCEPTION_POINTERS": ["ExceptionRecord", "ContextRecord"],
    # except.c passes a DISPATCHER_CONTEXT through unchanged in the parts that are
    # graded and reads ImageBase/ControlPc/FunctionEntry in the parts that are not.
    "DISPATCHER_CONTEXT": [
        "ControlPc",
        "ImageBase",
        "FunctionEntry",
        "EstablisherFrame",
        "TargetIp",
        "ContextRecord",
        "LanguageHandler",
        "HandlerData",
        "HistoryTable",
        "ScopeIndex",
    ],
    "RUNTIME_FUNCTION": ["BeginAddress", "EndAddress", "UnwindData"],
    # `frame_info` and `cxx_frame_info` are msvcrt's OWN, from
    # dll/win32/msvcrt/msvcrt.h.  They are the linked list `_CreateFrameInfo` /
    # `_FindAndUnlinkFrame` walk — the bookkeeping half of except.c, which is the
    # half that IS graded, so their layout is under test rather than incidental.
    "frame_info": ["object", "next"],
    "cxx_frame_info": ["frame_info", "rec", "context"],
}

CPP_SCALAR_SIZES = ["CONTEXT"]

OUT_LAYOUT_H = REPO / "sdk/lib/rust/msvcrt-difftest/refstub/msvcrt_crt_layout.h"
OUT_CPP_LAYOUT_H = REPO / "sdk/lib/rust/msvcrt-difftest/refstub/msvcrt_cpp_layout.h"


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
    for group in (LAYOUTS, CPP_LAYOUTS):
        for struct_name, fields in group.items():
            for f in fields:
                items.append((f"off:{struct_name}:{f}", f"__builtin_offsetof({struct_name}, {f})"))
            items.append((f"size:{struct_name}", f"sizeof({struct_name})"))
    for s in SCALAR_SIZES + CPP_SCALAR_SIZES:
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


def render_layout(
    values: dict[str, int],
    *,
    layouts: dict[str, list[str]],
    scalars: list[str],
    guard: str,
    after: str,
) -> str:
    """A wall of _Static_assert the HOST compiler checks against the transcription."""
    out = [
        "/* GENERATED by tools/gen_msvcrt_difftest_consts.py — DO NOT EDIT.",
        " *",
        " * The layout guardian.  Every number below was measured by the PINNED",
        " * CROSS-COMPILER reading msvcrt's real headers; the host compiler checks",
        f" * them against the transcribed definitions in {after}.  The",
        " * two must agree or the reference TU does not build — which is the whole",
        " * point: a struct layout cannot be derived into a header the way a",
        " * constant can, so it is transcribed and then PROVED.",
        " *",
        f" * Include AFTER {after}.",
        " */",
        f"#ifndef {guard}",
        f"#define {guard}",
        "",
        "/* Scalar widths every layout below depends on. */",
    ]
    for s in scalars:
        v = values[f"size:{s}"]
        out.append(f'_Static_assert(sizeof({s}) == {v}, "{s} width drifted");')
    out.append("")
    for struct_name, fields in layouts.items():
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
    out += [f"#endif /* {guard} */", ""]
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
            if v < 0:
                # A SIGNED 32-bit constant: the NTSTATUS family, whose headers
                # spell it `((NTSTATUS)0xC0000005L)` — a `long` that is 32-bit on
                # Windows and so comes back sign-extended from the probe.  Emit
                # the same bit pattern under the same signedness (`int` is the
                # host's 32-bit signed type), NOT the decimal negative and not
                # `f"0x{v:08X}"`, which for a negative produces the nonsense
                # literal `0x-3FFFFFFB`.  Signedness is not cosmetic here:
                # except.c compares `_SEH2_GetExceptionCode()` — an `int` — against
                # these, and an unsigned spelling would convert the int operand
                # and change which arm of the mapping switch runs.
                lit, suffix = f"((int)0x{v & 0xFFFFFFFF:08X})", ""
            else:
                lit = f"0x{v:08X}" if v > 9 else str(v)
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
    outputs = [
        (OUT_H, render(values)),
        (
            OUT_LAYOUT_H,
            render_layout(
                values,
                layouts=LAYOUTS,
                scalars=SCALAR_SIZES,
                guard="MSVCRT_DIFFTEST_CRT_LAYOUT_H",
                after="msvcrt_file_types.h",
            ),
        ),
        (
            OUT_CPP_LAYOUT_H,
            render_layout(
                values,
                layouts=CPP_LAYOUTS,
                scalars=CPP_SCALAR_SIZES,
                guard="MSVCRT_DIFFTEST_CPP_LAYOUT_H",
                after="msvcrt_cpp_types.h",
            ),
        ),
    ]

    n_const = sum(len(v) for v in CONSTANTS.values()) + len(VERBATIM)
    n_assert = (
        len(SCALAR_SIZES)
        + len(CPP_SCALAR_SIZES)
        + sum(len(f) + 1 for f in LAYOUTS.values())
        + sum(len(f) + 1 for f in CPP_LAYOUTS.values())
    )

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
    print(
        f"wrote {OUT_LAYOUT_H.relative_to(REPO)} + {OUT_CPP_LAYOUT_H.name} ({n_assert} assertions)"
    )
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
