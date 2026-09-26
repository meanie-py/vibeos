#!/usr/bin/env python3
"""Bounded CEnumIDListBase host grade using verbatim C++ and banked Rust."""

from __future__ import annotations

import argparse
import contextlib
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
SOURCE = Path("dll/win32/shell32/CEnumIDListBase.cpp")
SOURCE_SHA = "910963d2c5685f375e5ebd6af903b8022a3dc1806fff969e10c4684eb5b1c013"
BANK = Path("20260718-205701-CEnumIDListBase.cpp")
BANKED = {
    "Next": (
        "06_CEnumIDListBase_Next.rs",
        "0c7592cd4e982c48c1c51e78c10312c23b86bbbc9e9145b466c4c970d353c9b2",
    ),
    "Skip": (
        "07_CEnumIDListBase_Skip.rs",
        "ce1935328000b824b5dd54a2d20708886844dcbfe1c276dfdef53e5d0f11fb5f",
    ),
    "Reset": (
        "08_CEnumIDListBase_Reset.rs",
        "bff6be42f7617e11925a8fc6f9f9bc5a137c0094cb23299ea234874a1f8a6b4f",
    ),
}


def sha(path: Path) -> str:
    return hashlib.sha256(path.read_bytes()).hexdigest()


def body(source: str, name: str) -> str:
    start = source.index("HRESULT WINAPI CEnumIDListBase::" + name + "(")
    opening = source.index("{", start)
    level = 0
    for at in range(opening, len(source)):
        if source[at] == "{":
            level += 1
        elif source[at] == "}":
            level -= 1
            if level == 0:
                return source[start : at + 1]
    raise ValueError(f"unterminated {name}")


def generate(repo: Path, bank: Path, work: Path) -> dict[str, object]:
    source = repo / SOURCE
    if sha(source) != SOURCE_SHA:
        raise ValueError("CEnum source changed")
    for name, (filename, expected) in BANKED.items():
        if sha(bank / BANK / filename) != expected:
            raise ValueError(f"banked {name} changed")
    text = source.read_text(encoding="utf-8")
    cpp = (
        """#include <cstdint>
#include <cstdlib>
#include <iostream>
#include <string>
#include <vector>
#define WINAPI
#define TRACE(...) ((void)0)
using ULONG = uint32_t;
using DWORD = uint32_t;
using HRESULT = int32_t;
struct ENUMLIST { ENUMLIST *pNext; uint32_t *pidl; };
class CEnumIDListBase {
public:
    ENUMLIST *mpFirst, *mpLast, *mpCurrent;
    HRESULT Skip(ULONG);
    HRESULT Reset();
};
constexpr HRESULT S_OK = 0;
constexpr HRESULT S_FALSE = 1;
"""
        + body(text, "Skip")
        + "\n\n"
        + body(text, "Reset")
        + """
int main(int argc, char **argv) {
    if (argc < 3) return 2;
    unsigned count = std::stoul(argv[1]);
    std::vector<ENUMLIST> nodes(count);
    CEnumIDListBase object{};
    for (unsigned i = 0; i < count; ++i) nodes[i].pNext = i + 1 < count ? &nodes[i + 1] : nullptr;
    object.mpFirst = count ? &nodes[0] : nullptr;
    object.mpLast = count ? &nodes[count - 1] : nullptr;
    object.mpCurrent = object.mpFirst;
    for (int i = 2; i < argc; ++i) {
        std::string op(argv[i]);
        HRESULT hr = op == "R" ? object.Reset() : object.Skip(std::stoul(op.substr(1)));
        unsigned cursor = 0;
        for (ENUMLIST *node = object.mpFirst; node && node != object.mpCurrent;
             node = node->pNext) ++cursor;
        std::cout << hr << ':' << cursor << ';';
    }
    std::cout << '\\n';
}
"""
    )
    rust = (
        """#![allow(non_snake_case, non_camel_case_types, unused_mut)]
type ULONG = u32;
type HRESULT = i32;
const S_OK: HRESULT = 0;
const S_FALSE: HRESULT = 1;
#[repr(C)]
struct ENUMLIST { pNext: *mut ENUMLIST, pidl: *mut u32 }
#[repr(C)]
struct CEnumIDListBase { mpFirst: *mut ENUMLIST, mpLast: *mut ENUMLIST, mpCurrent: *mut ENUMLIST }
"""
        + f'include!("{bank / BANK / BANKED["Skip"][0]}");\n'
        + f'include!("{bank / BANK / BANKED["Reset"][0]}");\n'
        + """
fn main() {
    let args: Vec<String> = std::env::args().collect();
    let count: usize = args[1].parse().unwrap();
    let mut nodes: Vec<ENUMLIST> = (0..count)
        .map(|_| ENUMLIST { pNext: std::ptr::null_mut(), pidl: std::ptr::null_mut() })
        .collect();
    for i in 0..count {
        nodes[i].pNext = if i + 1 < count { &mut nodes[i + 1] }
                         else { std::ptr::null_mut() };
    }
    let first = if count > 0 { &mut nodes[0] as *mut ENUMLIST } else { std::ptr::null_mut() };
    let last = if count > 0 { &mut nodes[count - 1] as *mut ENUMLIST }
               else { std::ptr::null_mut() };
    let mut object = CEnumIDListBase { mpFirst: first, mpLast: last, mpCurrent: first };
    for op in &args[2..] {
        let hr = unsafe {
            if op == "R" { CEnumIDListBase_Reset_body(&mut object) }
            else { CEnumIDListBase_Skip_body(&mut object, op[1..].parse().unwrap()) }
        };
        let mut cursor = 0;
        let mut node = object.mpFirst;
        unsafe {
            while !node.is_null() && node != object.mpCurrent {
                cursor += 1;
                node = (*node).pNext;
            }
        }
        print!("{hr}:{cursor};");
    }
    println!();
}
"""
    )
    (work / "cenum_reference_generated.cpp").write_text(cpp, encoding="utf-8")
    (work / "cenum_candidate_generated.rs").write_text(rust, encoding="utf-8")
    subprocess.run(
        [
            "g++",
            "-std=c++17",
            "-O2",
            str(work / "cenum_reference_generated.cpp"),
            "-o",
            str(work / "cenum_reference"),
        ],
        check=True,
    )
    subprocess.run(
        [
            "rustc",
            "--edition=2021",
            "-O",
            "-A",
            "private_interfaces",
            str(work / "cenum_candidate_generated.rs"),
            "-o",
            str(work / "cenum_candidate"),
        ],
        check=True,
    )

    probe = (
        """#![allow(non_snake_case, non_camel_case_types, unused_mut, dead_code)]
type ULONG = u32;
type HRESULT = i32;
type PITEMID_CHILD = *mut u32;
type LPITEMIDLIST = *mut u32;
const S_OK: HRESULT = 0;
const S_FALSE: HRESULT = 1;
const E_INVALIDARG: HRESULT = -2147024809;
const E_OUTOFMEMORY: HRESULT = -2147024882;
#[repr(C)]
struct ENUMLIST { pNext: *mut ENUMLIST, pidl: LPITEMIDLIST }
#[repr(C)]
struct CEnumIDListBase { mpFirst: *mut ENUMLIST, mpLast: *mut ENUMLIST, mpCurrent: *mut ENUMLIST }
unsafe fn ILClone(pidl: LPITEMIDLIST) -> LPITEMIDLIST { pidl }
"""
        + f'include!("{bank / BANK / BANKED["Next"][0]}");\nfn main() {{}}\n'
    )
    (work / "cenum_next_probe.rs").write_text(probe, encoding="utf-8")
    compile_next = subprocess.run(
        [
            "rustc",
            "--edition=2021",
            "-O",
            str(work / "cenum_next_probe.rs"),
            "-o",
            str(work / "cenum_next_probe"),
        ],
        capture_output=True,
        text=True,
    )
    (work / "cenum_next_rustc.log").write_text(compile_next.stderr, encoding="utf-8")
    return {
        "source_sha256": SOURCE_SHA,
        "bank_sha256": {name: digest for name, (_, digest) in BANKED.items()},
        "next_compile_status": compile_next.returncode,
        "next_compile_log": compile_next.stderr,
    }


def main() -> int:
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("--repo", required=True, type=Path)
    parser.add_argument("--bank", required=True, type=Path)
    parser.add_argument("--output", type=Path)
    parser.add_argument(
        "--artifact-dir",
        type=Path,
        help="retain generated sources/binaries/log; default uses a temporary directory",
    )
    args = parser.parse_args()
    scope = (
        contextlib.nullcontext(args.artifact_dir.resolve())
        if args.artifact_dir
        else tempfile.TemporaryDirectory(prefix="rustconv-cenum-")
    )
    with scope as location:
        work = Path(location)
        work.mkdir(parents=True, exist_ok=True)
        evidence = generate(args.repo.resolve(), args.bank.resolve(), work)
        reference_provenance = cpp_reference_provenance(work / "cenum_reference_generated.cpp")
        suite = [
            (0, ("S0", "R", "S1")),
            (1, ("S1", "S1", "R", "S0")),
            (3, ("S1", "S2", "R", "S4", "R", "S3")),
        ]
        rng = random.Random(0xCE0)
        for _ in range(80):
            suite.append(
                (
                    rng.randrange(0, 6),
                    tuple(
                        ("R" if rng.random() < 0.2 else f"S{rng.randrange(0, 7)}")
                        for _ in range(rng.randrange(1, 10))
                    ),
                )
            )
        mismatches = []
        for index, (count, ops) in enumerate(suite):
            command = (str(count), *ops)
            cpp = subprocess.run(
                [str(work / "cenum_reference"), *command],
                check=True,
                capture_output=True,
                text=True,
            ).stdout.strip()
            rust = subprocess.run(
                [str(work / "cenum_candidate"), *command],
                check=True,
                capture_output=True,
                text=True,
            ).stdout.strip()
            if cpp != rust:
                mismatches.append(
                    {
                        "index": index,
                        "count": count,
                        "ops": ops,
                        "reference": cpp,
                        "banked_rust": rust,
                    }
                )
    report = {
        "unit": "CEnumIDListBase selected methods",
        "whole_unit_status": "rejected_and_incomplete",
        "selected_functions": ["Skip", "Reset"],
        "cases_compared": len(suite),
        "mismatch_count": len(mismatches),
        "matched": len(suite) - len(mismatches),
        "mismatches": mismatches,
        "guards": [
            "pinned C++ source SHA-256",
            "pinned banked-body SHA-256",
            "verbatim method extraction",
            "untouched Rust include",
        ],
        "candidate_failures": [
            {
                "method": "Next",
                "status": "rejected",
                "reason": "rustc E0308 pointer slot assigned integer 0",
                "evidence_field": "next_compile_log",
            }
        ],
        "required_unsupported_methods": [
            {
                "method": "AppendItemsFromEnumerator",
                "missing_substrate": (
                    "IEnumIDList virtual Reset/Next, owned PIDLs and "
                    "allocation failure instrumentation"
                ),
            },
            {"method": "DeleteList", "missing_substrate": "SHFree/PIDL ownership instrumentation"},
        ],
        **evidence,
        "reference_provenance": reference_provenance,
    }
    if args.output:
        args.output.write_text(json.dumps(report, indent=2) + "\n", encoding="utf-8")
    print(
        json.dumps(
            {
                "unit": report["unit"],
                "cases_compared": len(suite),
                "mismatch_count": len(mismatches),
                "next_status": evidence["next_compile_status"],
                "header_count": len(reference_provenance["headers"]),
            }
        )
    )
    return 0 if not mismatches else 1


if __name__ == "__main__":
    raise SystemExit(main())
