#!/usr/bin/env python3
"""Prove msvcrt-seh-difftest's i386 type transcription against the REAL headers.

`msvcrt-seh-difftest` compiles the genuine `dll/win32/msvcrt/except_i386.c` for a 32-bit
Linux host, where the tree's PE-flavoured PSDK headers cannot be included, so
`refstub/seh_types.h` transcribes the eleven structs that TU touches.  Every one of them is
walked BY OFFSET — 32-bit SEH is a linked list of registration records and a scope table
indexed by trylevel — so a transcription that is one field out does not fail to compile, it
grades correct Rust as wrong (or, worse, wrong Rust as right).  That is the §9.66 class:
the oracle is not the code under test.

So nothing in that header is trusted.  This script compiles the real i386 headers with the
pinned cross-compiler, reads every offset and size back out of the object, and emits
`refstub/seh_layout.h` — a wall of `_Static_assert`s the HOST compiler checks the
transcription against on every build.  The compiler is the authority; this only transports
the answer.

`--check` re-derives and diffs against the committed file, and is wired into
`vibeos-cli difftest`'s `drift:` stage.  A PSDK header that moves a field then fails loudly
before a single row runs, instead of silently regrading SEH against a layout msvcrt.dll no
longer has.

Usage:
    python3 tools/gen_msvcrt_seh_layout.py            # write the header
    python3 tools/gen_msvcrt_seh_layout.py --check    # fail on drift
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
OUT_H = REPO / "sdk/lib/rust/msvcrt-seh-difftest/refstub/seh_layout.h"

# i386, because that is the arch whose layouts this grades.  Fleet isolation law: THIS
# worktree's build dir, never a sibling's — a foreign build dir would green-light a real
# layout drift (the verify_offsets incident).
BUILD_DIR = REPO / "build-i386"

# The headers, in the order except_i386.c reaches them.
PROBE_INCLUDES = [
    "stdarg.h",
    "fpieee.h",
    "setjmp.h",
    '"windef.h"',
    '"winbase.h"',
    '"winternl.h"',
    '"msvcrt.h"',
    '"wine/exception.h"',
    '"excpt.h"',
    '"wine/debug.h"',
    '"cppexcept.h"',
]

#: struct -> the fields whose offsets are asserted.  Bit-fields are absent by necessity
#: (`offsetof` cannot name one); the struct's SIZE plus the offsets of the members that
#: follow them is what pins a bit-field run in place.
LAYOUTS: dict[str, list[str]] = {
    "EXCEPTION_RECORD": [
        "ExceptionCode",
        "ExceptionFlags",
        "ExceptionRecord",
        "ExceptionAddress",
        "NumberParameters",
        "ExceptionInformation",
    ],
    "EXCEPTION_REGISTRATION_RECORD": ["Prev", "Handler"],
    "EXCEPTION_POINTERS": ["ExceptionRecord", "ContextRecord"],
    "FLOATING_SAVE_AREA": [
        "ControlWord",
        "StatusWord",
        "TagWord",
        "ErrorOffset",
        "ErrorSelector",
        "DataOffset",
        "DataSelector",
        "RegisterArea",
        "Cr0NpxState",
    ],
    "CONTEXT": [
        "ContextFlags",
        "Dr0",
        "Dr7",
        "FloatSave",
        "SegGs",
        "Edi",
        "Eax",
        "Ebp",
        "Eip",
        "Esp",
        "SegSs",
        "ExtendedRegisters",
    ],
    "NT_TIB": ["ExceptionList", "StackBase", "StackLimit", "Self"],
    "_JUMP_BUFFER": [
        "Ebp",
        "Ebx",
        "Edi",
        "Esi",
        "Esp",
        "Eip",
        "Registration",
        "TryLevel",
        "Cookie",
        "UnwindFunc",
        "UnwindData",
    ],
    # The FPIEEE record is `handle_fpieee_flt`'s entire output, and every one of its
    # sub-records is a bit-field run — so the offsets below are the only handle on it, and
    # the sizes are what say the runs did not grow.
    "_FPIEEE_VALUE": ["Value"],
    "_FPIEEE_RECORD": ["Cause", "Enable", "Status", "Operand1", "Operand2", "Result"],
    "cxx_function_descr": [
        "unwind_count",
        "unwind_table",
        "tryblock_count",
        "tryblock",
        "ipmap_count",
        "ipmap",
        "expect_list",
        "flags",
    ],
    "tryblock_info": [
        "start_level",
        "end_level",
        "catch_level",
        "catchblock_count",
        "catchblock",
    ],
    "unwind_info": ["prev", "handler"],
    "catchblock_info": ["flags", "type_info", "offset", "handler"],
    "cxx_exception_type": ["flags", "destructor", "custom_handler", "type_info_table"],
    "frame_info": ["object", "next"],
    "cxx_frame_info": ["frame_info", "rec", "context"],
    # Only the five members the reference reads, plus the size.  The subject's view of this
    # struct is msvcrt-sys's bindgen of the same header, and sehimpl.c allocates it — so
    # agreement on these five and the total is exactly what the two sides need.
    "thread_data_t": [
        "se_translator",
        "exc_record",
        "ctx_record",
        "processing_throw",
        "frame_info_head",
    ],
}

#: Scalars whose width the transcription assumes.  `long` is the one that would bite: it is
#: 4 bytes on i386 PE and on i386 ELF, and 8 on the x86-64 host this script runs on — so an
#: accidental 64-bit build of the reference would be caught here rather than by a wrong row.
SCALAR_SIZES = ["void *", "long", "unsigned long", "int", "DWORD", "ULONG_PTR", "WCHAR"]

#: Constants the reference TU names that the prelude transcribes.  Same argument as the
#: layouts: a magic number spelled from memory is an oracle that grades against a value the
#: DLL never sees (§9.113's 61 invented stop codes).
CONSTANTS = [
    "EXCEPTION_UNWINDING",
    "EXCEPTION_EXIT_UNWIND",
    "EXCEPTION_MAXIMUM_PARAMETERS",
    "EXCEPTION_EXECUTE_HANDLER",
    "EXCEPTION_CONTINUE_SEARCH",
    "EXCEPTION_CONTINUE_EXECUTION",
    "ExceptionContinueExecution",
    "ExceptionContinueSearch",
    "ExceptionNestedException",
    "ExceptionCollidedUnwind",
    "CXX_FRAME_MAGIC_VC6",
    "CXX_FRAME_MAGIC_VC7",
    "CXX_FRAME_MAGIC_VC8",
    "CXX_EXCEPTION",
    "FUNC_DESCR_SYNCHRONOUS",
    "FUNC_DESCR_NOEXCEPT",
    "STATUS_FLOAT_DIVIDE_BY_ZERO",
    "STATUS_FLOAT_INVALID_OPERATION",
    "_FpCodeDivide",
    "_FpFormatFp64",
    "_FpFormatFp80",
    "SIZE_OF_80387_REGISTERS",
    "MAXIMUM_SUPPORTED_EXTENSION",
    "LOCALE_NAME_MAX_LENGTH",
]


def include_flags() -> list[str]:
    """msvcrt's OWN include order — see gen_msvcrt_difftest_consts.py's copy for why the
    global `sdk/include/crt` is absent and `dll/win32/msvcrt/include` is present."""
    r, b = REPO, BUILD_DIR
    dirs = [
        r / "sdk/include/wine",
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
        r / "dll/win32/msvcrt/include",
        r / "dll/win32/msvcrt",
    ]
    return [f"-I{d}" for d in dirs]


def _probe_exprs() -> list[tuple[str, str]]:
    items: list[tuple[str, str]] = []
    for name in CONSTANTS:
        items.append((f"const:{name}", name))
    for struct_name, fields in LAYOUTS.items():
        for f in fields:
            items.append((f"off:{struct_name}:{f}", f"__builtin_offsetof({struct_name}, {f})"))
        items.append((f"size:{struct_name}", f"sizeof({struct_name})"))
    for s in SCALAR_SIZES:
        items.append((f"size:{s}", f"sizeof({s})"))
    return items


def _ishex(s: str) -> bool:
    return all(ch in "0123456789abcdefABCDEF" for ch in s)


def harvest() -> dict[str, int]:
    if not CLANG.exists():
        sys.exit(
            f"FAIL: pinned clang not found at {CLANG}\n"
            f"fix: the llvm-mingw toolchain must be provisioned"
        )
    if not (BUILD_DIR / "sdk/include/reactos").is_dir():
        sys.exit(
            f"FAIL: {BUILD_DIR.relative_to(REPO)} has no generated headers.\n"
            f"fix: ./vibeos-cli build --arch i386\n"
            f"(this worktree's build dir only — never a sibling's)"
        )

    items = _probe_exprs()
    names = [k for k, _ in items]
    src = "\n".join(
        f"#include {inc}" if inc.startswith('"') else f"#include <{inc}>" for inc in PROBE_INCLUDES
    )
    src += "\n__attribute__((used)) const long long __harvest[] = {\n"
    src += "".join(f"  (long long)({expr}),\n" for _, expr in items)
    src += "};\n"

    with tempfile.TemporaryDirectory() as td:
        c, o = Path(td) / "probe.c", Path(td) / "probe.o"
        c.write_text(src)
        cmd = [
            str(CLANG),
            "-target",
            "i686-w64-mingw32",
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
                "FAIL: the i386 SEH layout probe did not compile.\n"
                "A name in LAYOUTS/CONSTANTS is misspelled, or its header moved.\n\n"
                + p.stderr[-3000:]
            )

        d = subprocess.run(
            [str(OBJDUMP), "-s", "-j", ".rdata", str(o)], capture_output=True, text=True
        )
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


def render(values: dict[str, int]) -> str:
    out = [
        "/* GENERATED by tools/gen_msvcrt_seh_layout.py — DO NOT EDIT.",
        " *",
        " * Every number here was computed by the PINNED CROSS-COMPILER from the REAL i386",
        " * headers, then asserted against refstub/seh_types.h's transcription by the HOST",
        " * compiler.  A failure names the field that moved; regenerate rather than editing",
        " * the assert, and then fix the transcription it is pointing at.",
        " */",
        "",
        "#ifndef MSVCRT_SEH_LAYOUT_H",
        "#define MSVCRT_SEH_LAYOUT_H",
        "",
        "/* ── scalar widths ─────────────────────────────────────────────────── */",
    ]
    for s in SCALAR_SIZES:
        v = values[f"size:{s}"]
        ident = s.replace(" ", "_").replace("*", "p")
        out.append(f'_Static_assert(sizeof({s}) == {v}, "seh refstub: sizeof({s}) != {v}");')
        del ident
    out.append("")
    out.append("/* ── constants ─────────────────────────────────────────────────────── */")
    for name in CONSTANTS:
        v = values[f"const:{name}"]
        out.append(f'_Static_assert((long long)({name}) == {v}LL, "seh refstub: {name} != {v}");')
    out.append("")
    out.append("/* ── struct layouts ────────────────────────────────────────────────── */")
    for struct_name, fields in LAYOUTS.items():
        out.append(f"/* {struct_name} */")
        for f in fields:
            v = values[f"off:{struct_name}:{f}"]
            out.append(
                f"_Static_assert(__builtin_offsetof({struct_name}, {f}) == {v},"
                f' "seh refstub: {struct_name}.{f} is not at {v}");'
            )
        v = values[f"size:{struct_name}"]
        out.append(
            f"_Static_assert(sizeof({struct_name}) == {v},"
            f' "seh refstub: sizeof({struct_name}) != {v}");'
        )
        out.append("")
    out.append("#endif /* MSVCRT_SEH_LAYOUT_H */")
    return "\n".join(out) + "\n"


def main() -> int:
    ap = argparse.ArgumentParser(description=__doc__)
    ap.add_argument("--check", action="store_true", help="fail if the committed header drifted")
    args = ap.parse_args()

    text = render(harvest())
    if args.check:
        if not OUT_H.is_file():
            print(f"FAIL: {OUT_H.relative_to(REPO)} does not exist")
            return 1
        if OUT_H.read_text() != text:
            print(
                f"FAIL: {OUT_H.relative_to(REPO)} is STALE — an i386 header moved a field.\n"
                f"fix: python3 tools/gen_msvcrt_seh_layout.py"
            )
            return 1
        n = sum(len(f) + 1 for f in LAYOUTS.values()) + len(SCALAR_SIZES) + len(CONSTANTS)
        print(f"gen_msvcrt_seh_layout: in sync ({n} assertions)")
        return 0

    OUT_H.parent.mkdir(parents=True, exist_ok=True)
    OUT_H.write_text(text)
    n = sum(len(f) + 1 for f in LAYOUTS.values()) + len(SCALAR_SIZES) + len(CONSTANTS)
    print(f"wrote {OUT_H.relative_to(REPO)} ({n} assertions)")
    return 0


if __name__ == "__main__":
    sys.exit(main())
