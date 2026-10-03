#!/usr/bin/env python3
"""Compare bounded Rust std semantics with genuine C++ stdlib behavior."""

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


def sha256(path: Path) -> str:
    return hashlib.sha256(path.read_bytes()).hexdigest()


def run(program: Path, args: tuple[str, ...]) -> str:
    completed = subprocess.run(
        [str(program), *args], check=True, text=True, capture_output=True, timeout=5
    )
    return completed.stdout.strip()


def cases(split: str) -> list[tuple[str, ...]]:
    suite: list[tuple[str, ...]] = []
    if split == "dev":
        samples = [
            b"",
            b"0",
            b"00",
            b"09",
            b"0x",
            b"0xG",
            b"0x1fEND",
            b"-0X7F!",
            b"+077tail",
            b"  -42rest",
            b"\t+15\n",
            b"0b101",
            b"101101",
            b"zZ",
            b"-9223372036854775808",
            b"9223372036854775807",
            b"9223372036854775808",
            b"-9223372036854775809",
            b"123\x00suffix",
            b"-0",
            b"+0",
            b"  +0x10 tail",
            b"0x10",
            b"010",
        ]
        for sample in samples:
            for base in (0, 2, 8, 10, 16, 36):
                suite.append(("stol", sample.hex(), str(base)))
        for data in (b"", b"abcdef", b"  CODEPAGE 1252", b"a\x00b", bytes(range(256))):
            for pos in (0, 1, len(data), len(data) + 1):
                for count in ("npos", "0", "1", "3", "1000"):
                    suite.append(("substr", data.hex(), str(pos), count))
        seed = 0xC0DE
        n_random = 64
    else:
        seed = 0xC0F7
        n_random = 128
    rng = random.Random(seed)
    alphabet = b"  +-/0123456789abcdefABCDEFxXzZqQ\t\n\x00"
    for _ in range(n_random):
        data = bytes(rng.choice(alphabet) for _ in range(rng.randrange(0, 28)))
        suite.append(("stol", data.hex(), str(rng.choice((0, 2, 8, 10, 16, 36)))))
        data = bytes(rng.randrange(256) for _ in range(rng.randrange(0, 28)))
        suite.append(
            (
                "substr",
                data.hex(),
                str(rng.randrange(0, len(data) + 3)),
                rng.choice(("npos", "0", "1", "3", str(rng.randrange(0, 35)))),
            )
        )
    return suite


def main() -> int:
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("--split", choices=("dev", "heldout"), default="dev")
    parser.add_argument("--output", type=Path)
    parser.add_argument(
        "--artifact-dir",
        type=Path,
        help="retain compiled oracles; default uses a temporary directory",
    )
    args = parser.parse_args()
    scope = (
        contextlib.nullcontext(args.artifact_dir.resolve())
        if args.artifact_dir
        else tempfile.TemporaryDirectory(prefix="rustconv-std-semantics-")
    )
    with scope as location:
        work = Path(location)
        work.mkdir(parents=True, exist_ok=True)
        cpp = work / "semantics_reference"
        rust = work / "semantics_cli"
        subprocess.run(
            [
                "g++",
                "-std=c++17",
                "-O2",
                "-Wall",
                "-Wextra",
                str(HERE / "semantics_reference.cpp"),
                "-o",
                str(cpp),
            ],
            check=True,
        )
        subprocess.run(
            ["rustc", "--edition=2021", "-O", str(HERE / "semantics_cli.rs"), "-o", str(rust)],
            check=True,
        )
        reference_provenance = cpp_reference_provenance(HERE / "semantics_reference.cpp")
        abi = run(cpp, ("abi",)).split(":")
        if len(abi) != 2 or abi[0] not in ("32", "64") or abi[1] not in ("0", "1"):
            raise RuntimeError(f"unsupported C++ long/strtol ABI: {abi}")
        mismatches = []
        suite = cases(args.split)
        for index, case in enumerate(suite):
            expected = run(cpp, case)
            actual = run(rust, case + tuple(abi) if case[0] == "stol" else case)
            if expected != actual:
                mismatches.append(
                    {"index": index, "input": case, "reference": expected, "rust": actual}
                )
    report = {
        "split": args.split,
        "cases_compared": len(suite),
        "mismatch_count": len(mismatches),
        "matches": len(suite) - len(mismatches),
        "mismatches": mismatches,
        "guards": [
            "genuine C++ stdlib oracle",
            "compiler-probed C++ long width and binary-prefix behavior",
            "explicit unsupported input bounds",
        ],
        "required_unsupported_methods": [],
        "cpp_abi": {"long_bits": int(abi[0]), "binary_prefix": abi[1] == "1"},
        "reference_sha256": sha256(HERE / "semantics_reference.cpp"),
        "rust_semantics_sha256": sha256(HERE / "cpp_std_semantics.rs"),
        "reference_provenance": reference_provenance,
    }
    if args.output:
        args.output.write_text(json.dumps(report, indent=2) + "\n", encoding="utf-8")
    print(
        json.dumps(
            {
                "split": args.split,
                "cases_compared": len(suite),
                "mismatch_count": len(mismatches),
                "header_count": len(reference_provenance["headers"]),
            }
        )
    )
    return 0 if not mismatches else 1


if __name__ == "__main__":
    raise SystemExit(main())
