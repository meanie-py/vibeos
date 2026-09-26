#!/usr/bin/env python3
"""Grade preserved SheRemoveQuotes Rust against verbatim C++ bodies."""

from __future__ import annotations

import argparse
import contextlib
import ctypes
import hashlib
import json
import random
import subprocess
import tempfile
from pathlib import Path

if __package__:
    from .host_provenance import cpp_reference_provenance
else:
    from host_provenance import cpp_reference_provenance


HERE = Path(__file__).resolve().parent
SOURCE_REL = Path("dll/win32/shell32/utils.cpp")
SOURCE_SHA = "c276b919e03ae8f7945c96faf329a0f64c843829adf8d7c56994bc02f4253feb"
BANK_A = "20260913-012131-utils.cpp/30_SheRemoveQuotesA.rs"
BANK_W = "20260913-034905-utils.cpp/31_SheRemoveQuotesW.rs"
BANK_SHA = (
    "39459f87315f029ca801cc043bb9a8b66c3a24935aedbd6b4d0291872fadd840",
    "1df553d17a231044949615b7686759b63d98e7031d74740bb40687571b9d32b7",
)


def sha256(path: Path) -> str:
    return hashlib.sha256(path.read_bytes()).hexdigest()


def extract_body(source: str, name: str) -> str:
    position = source.index(name + "(")
    start = source.rfind("EXTERN_C", 0, position)
    if start < 0 or position - start > 40:
        raise ValueError(f"cannot identify original declaration for {name}")
    opening = source.index("{", position)
    level = 0
    for at in range(opening, len(source)):
        if source[at] == "{":
            level += 1
        elif source[at] == "}":
            level -= 1
            if level == 0:
                return source[start : at + 1]
    raise ValueError(f"unterminated source body: {name}")


def generate(repo: Path, bank: Path, work: Path) -> dict[str, str]:
    source = repo / SOURCE_REL
    original_a = bank / BANK_A
    original_w = bank / BANK_W
    for path, expected in (
        (source, SOURCE_SHA),
        (original_a, BANK_SHA[0]),
        (original_w, BANK_SHA[1]),
    ):
        if sha256(path) != expected:
            raise ValueError(f"frozen source/bank changed: {path}")
    text = source.read_text(encoding="utf-8")
    body_a = extract_body(text, "SheRemoveQuotesA")
    body_w = extract_body(text, "SheRemoveQuotesW")
    cpp = (
        """// Verbatim extracted function bodies follow. Only type/macros are shims.
#define EXTERN_C extern "C"
#define WINAPI
using LPSTR = char *;
using LPWSTR = unsigned short *;
using PCHAR = char *;
using PWCHAR = unsigned short *;
constexpr char ANSI_NULL = 0;
constexpr unsigned short UNICODE_NULL = 0;
"""
        + body_a
        + "\n\n"
        + body_w
        + "\n"
    )
    rust = (
        """#![allow(non_camel_case_types)]
use std::ffi::c_char;
type PCHAR = *mut c_char;
type PWSTR = *mut u16;
const ANSI_NULL: c_char = 0;
const UNICODE_NULL: u16 = 0;
"""
        + f'include!("{original_a}");\ninclude!("{original_w}");\n'
    )
    (work / "quotes_reference_generated.cpp").write_text(cpp, encoding="utf-8")
    (work / "quotes_candidate_generated.rs").write_text(rust, encoding="utf-8")
    subprocess.run(
        [
            "g++",
            "-std=c++17",
            "-O2",
            "-shared",
            "-fPIC",
            str(work / "quotes_reference_generated.cpp"),
            "-o",
            str(work / "libquotes_reference.so"),
        ],
        check=True,
    )
    subprocess.run(
        [
            "rustc",
            "--edition=2021",
            "-O",
            "--crate-type=cdylib",
            str(work / "quotes_candidate_generated.rs"),
            "-o",
            str(work / "libquotes_candidate.so"),
        ],
        check=True,
    )
    return {
        "source_sha256": SOURCE_SHA,
        "bank_a_sha256": BANK_SHA[0],
        "bank_w_sha256": BANK_SHA[1],
        "extracted_a_sha256": hashlib.sha256(body_a.encode()).hexdigest(),
        "extracted_w_sha256": hashlib.sha256(body_w.encode()).hexdigest(),
    }


def fixtures() -> list[tuple[str, bytes | tuple[int, ...]]]:
    result: list[tuple[str, bytes | tuple[int, ...]]] = [
        ("A", b"\x00TAIL"),
        ("A", b"abc\x00TAIL"),
        ("A", b'""\x00TAIL'),
        ("A", b'"abc"\x00TAIL'),
        ("A", b'"abc\x00TAIL'),
        ("A", b'"a"b"\x00TAIL'),
        ("A", b'"\x80\xff"\x00TAIL'),
        ("W", (0, 0x1111, 0x2222)),
        ("W", (ord("a"), 0, 0x1111)),
        ("W", (34, 34, 0, 0x1111)),
        ("W", (34, ord("a"), ord("b"), 34, 0, 0x1111)),
        ("W", (34, ord("a"), ord("b"), 0, 0x1111)),
        ("W", (34, 0xD800, 0xDC00, 34, 0, 0x1111)),
    ]
    rng = random.Random(0x51554F54)
    for _ in range(80):
        body = bytes(rng.choice(b'" abcXYZ09\x80\xff') for _ in range(rng.randrange(0, 30)))
        result.append(("A", body + b"\x00TAIL"))
        wide = tuple(
            rng.choice((34, 32, 65, 0x4E00, 0xD800, 0xDC00)) for _ in range(rng.randrange(0, 30))
        )
        result.append(("W", (*wide, 0, 0x1111, 0x2222)))
    return result


def evaluate(
    library: ctypes.CDLL, kind: str, data: bytes | tuple[int, ...]
) -> tuple[bool, bytes | tuple[int, ...]]:
    function = getattr(library, "SheRemoveQuotes" + kind)
    function.argtypes = [ctypes.c_void_p]
    function.restype = ctypes.c_void_p
    if kind == "A":
        assert isinstance(data, bytes)
        array = (ctypes.c_ubyte * len(data))(*data)
        returned = function(ctypes.addressof(array))
        return returned == ctypes.addressof(array), bytes(array)
    assert isinstance(data, tuple)
    array = (ctypes.c_uint16 * len(data))(*data)
    returned = function(ctypes.addressof(array))
    return returned == ctypes.addressof(array), tuple(array)


def main() -> int:
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("--repo", required=True, type=Path)
    parser.add_argument(
        "--bank", required=True, type=Path, help="shell32 worktree tools/rustconv/.responses root"
    )
    parser.add_argument("--output", type=Path)
    parser.add_argument(
        "--artifact-dir",
        type=Path,
        help="retain generated sources and libraries; default uses a temporary directory",
    )
    args = parser.parse_args()
    scope = (
        contextlib.nullcontext(args.artifact_dir.resolve())
        if args.artifact_dir
        else tempfile.TemporaryDirectory(prefix="rustconv-quotes-")
    )
    with scope as location:
        work = Path(location)
        work.mkdir(parents=True, exist_ok=True)
        evidence = generate(args.repo.resolve(), args.bank.resolve(), work)
        reference_provenance = cpp_reference_provenance(
            work / "quotes_reference_generated.cpp",
            compile_flags=("-std=c++17", "-O2", "-shared", "-fPIC"),
        )
        reference = ctypes.CDLL(str(work / "libquotes_reference.so"))
        candidate = ctypes.CDLL(str(work / "libquotes_candidate.so"))
        mismatches = []
        suite = fixtures()
        for index, (kind, data) in enumerate(suite):
            expected = evaluate(reference, kind, data)
            actual = evaluate(candidate, kind, data)
            if expected != actual:
                mismatches.append(
                    {
                        "index": index,
                        "kind": kind,
                        "input": data.hex() if isinstance(data, bytes) else data,
                        "reference": expected,
                        "banked_rust": actual,
                    }
                )
    report = {
        "unit": "SheRemoveQuotesA/W selected functions",
        "cases_compared": len(suite),
        "mismatch_count": len(mismatches),
        "matched": len(suite) - len(mismatches),
        "mismatches": mismatches,
        "guards": [
            "pinned C++ source SHA-256",
            "pinned banked-body SHA-256",
            "verbatim body extraction",
            "full buffer and return-pointer comparison",
        ],
        "required_unsupported_methods": [],
        **evidence,
    }
    report["reference_provenance"] = reference_provenance
    if args.output:
        args.output.write_text(json.dumps(report, indent=2) + "\n", encoding="utf-8")
    print(
        json.dumps(
            {
                "unit": report["unit"],
                "cases_compared": len(suite),
                "mismatch_count": len(mismatches),
                "header_count": len(reference_provenance["headers"]),
            }
        )
    )
    return 0 if not mismatches else 1


if __name__ == "__main__":
    raise SystemExit(main())
