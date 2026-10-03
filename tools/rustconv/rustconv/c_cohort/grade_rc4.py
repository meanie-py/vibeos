#!/usr/bin/env python3
"""Differential RC4 grade: original cryptlib C versus shipping Rust source."""

from __future__ import annotations

import argparse
import ctypes
import hashlib
import json
import random
import shutil
import subprocess
import tempfile
from pathlib import Path

HASHES = {
    "sdk/lib/cryptlib/rc4.c": "62fbc8b3dd9951544d3452dd9f0aff6081e10074cd8c892a37a0a25a5c1d9acd",
    "sdk/lib/cryptlib/rc4.h": "be2bfd34aa80a33177e135028daa59ad0bc3f49f7b78e9009292f4dc91631c63",
    "sdk/lib/rust/cryptlib-rust/src/rc4.rs": (
        "87c33f05dfac7f98771ab4a2fa8dfff15fa7e30b547f089b2a34b76aed9e24f2"
    ),
    "sdk/lib/rust/cryptlib-rust/src/types.rs": (
        "c86017f200af320e7d7970fcf5ccd6e24c6cb670cd2b5526d510e9a1b66ee2a0"
    ),
}
GUARD = 16
SENTINEL = 0xA5


class RC4Context(ctypes.Structure):
    _fields_ = [("state", ctypes.c_ubyte * 256), ("x", ctypes.c_ubyte), ("y", ctypes.c_ubyte)]


def build_c(root: Path, temporary: Path, label: str, source: str) -> Path:
    directory = temporary / label
    directory.mkdir()
    (directory / "rc4.c").write_text(source)
    shutil.copyfile(root / "sdk/lib/cryptlib/rc4.h", directory / "rc4.h")
    library = directory / "librc4-c.so"
    subprocess.run(
        ["cc", "-std=c11", "-O2", "-fPIC", "-shared", str(directory / "rc4.c"), "-o", str(library)],
        check=True,
        capture_output=True,
        text=True,
    )
    return library


def build_rust(root: Path, temporary: Path) -> Path:
    types = root / "sdk/lib/rust/cryptlib-rust/src/types.rs"
    rc4 = root / "sdk/lib/rust/cryptlib-rust/src/rc4.rs"
    wrapper = temporary / "host_rc4.rs"
    wrapper.write_text(
        "#![allow(non_snake_case, non_camel_case_types, non_upper_case_globals, "
        "dead_code, unused_mut)]\n"
        f"#[path = {json.dumps(str(types))}] mod types;\n"
        f"#[path = {json.dumps(str(rc4))}] mod rc4;\n"
    )
    library = temporary / "librc4-rust.so"
    subprocess.run(
        [
            "rustc",
            "--edition",
            "2021",
            "--crate-type",
            "cdylib",
            "-O",
            str(wrapper),
            "-o",
            str(library),
        ],
        check=True,
        capture_output=True,
        text=True,
    )
    return library


def load(path: Path):
    dll = ctypes.CDLL(str(path))
    dll.rc4_init.argtypes = [
        ctypes.POINTER(RC4Context),
        ctypes.POINTER(ctypes.c_ubyte),
        ctypes.c_uint,
    ]
    dll.rc4_init.restype = None
    dll.rc4_crypt.argtypes = [
        ctypes.POINTER(RC4Context),
        ctypes.POINTER(ctypes.c_ubyte),
        ctypes.c_uint,
    ]
    dll.rc4_crypt.restype = None
    return dll


def guarded_context():
    size = ctypes.sizeof(RC4Context)
    if size != 258:
        raise RuntimeError(f"host RC4_CONTEXT layout changed: expected 258, observed {size}")
    storage = (ctypes.c_ubyte * (GUARD + size + GUARD))(*([SENTINEL] * (GUARD + size + GUARD)))
    pointer = ctypes.cast(ctypes.byref(storage, GUARD), ctypes.POINTER(RC4Context))
    return storage, pointer


def guards_ok(storage, payload_size: int) -> bool:
    return all(value == SENTINEL for value in storage[:GUARD]) and all(
        value == SENTINEL for value in storage[GUARD + payload_size :]
    )


def run(dll, key: bytes, data: bytes, chunks: tuple[int, ...]):
    context_storage, context = guarded_context()
    key_storage = (ctypes.c_ubyte * len(key))(*key)
    dll.rc4_init(context, key_storage, len(key))
    context_after_init = bytes(context_storage[GUARD : GUARD + ctypes.sizeof(RC4Context)])
    checks = [guards_ok(context_storage, ctypes.sizeof(RC4Context))]
    data_storage = (ctypes.c_ubyte * (GUARD + max(len(data), 1) + GUARD))(
        *([SENTINEL] * (GUARD + max(len(data), 1) + GUARD))
    )
    for index, value in enumerate(data):
        data_storage[GUARD + index] = value
    checkpoints = []
    offset = 0
    for size in chunks:
        if size < 0 or offset + size > len(data):
            raise ValueError("invalid chunk partition")
        pointer = ctypes.cast(
            ctypes.byref(data_storage, GUARD + offset), ctypes.POINTER(ctypes.c_ubyte)
        )
        dll.rc4_crypt(context, pointer, size)
        offset += size
        checkpoints.append(
            (
                bytes(data_storage[GUARD : GUARD + len(data)]),
                bytes(context_storage[GUARD : GUARD + ctypes.sizeof(RC4Context)]),
            )
        )
        checks.append(guards_ok(context_storage, ctypes.sizeof(RC4Context)))
        checks.append(guards_ok(data_storage, len(data)))
    if offset != len(data):
        raise ValueError("chunks do not cover input")
    return context_after_init, checkpoints, all(checks)


def cases(seed: int, random_cases: int):
    rng = random.Random(seed)
    keys = [b"\0", b"K", b"ab", b"Key", bytes(range(16)), bytes(range(256)), b"\xff" * 32]
    lengths = [0, 1, 2, 3, 7, 16, 31, 64, 255, 256, 257, 512]
    for key in keys:
        for length in lengths:
            data = bytes(rng.randrange(256) for _ in range(length))
            half = length // 2
            yield key, data, (length,)
            yield key, data, (0, half, 0, length - half)
    for _ in range(random_cases):
        key = bytes(rng.randrange(256) for _ in range(rng.randrange(1, 257)))
        data = bytes(rng.randrange(256) for _ in range(rng.randrange(0, 1025)))
        split = rng.randrange(len(data) + 1)
        yield key, data, (split, len(data) - split)


def grade(reference, candidate, seed: int, random_cases: int):
    total = mismatch_count = guard_failures = 0
    examples = []
    for key, data, chunks in cases(seed, random_cases):
        expected = run(reference, key, data, chunks)
        actual = run(candidate, key, data, chunks)
        total += 1
        if not expected[2] or not actual[2]:
            guard_failures += 1
        if expected != actual:
            mismatch_count += 1
            if len(examples) < 5:
                examples.append(
                    {
                        "key_hex": key.hex(),
                        "data_len": len(data),
                        "chunks": chunks,
                        "reference_guard": expected[2],
                        "candidate_guard": actual[2],
                        "init_equal": expected[0] == actual[0],
                        "first_differing_chunk": next(
                            (
                                index
                                for index, pair in enumerate(
                                    zip(expected[1], actual[1], strict=False)
                                )
                                if pair[0] != pair[1]
                            ),
                            None,
                        ),
                    }
                )
    return {
        "cases": total,
        "mismatches": mismatch_count,
        "guard_failures": guard_failures,
        "examples": examples,
        "passed": mismatch_count == 0 and guard_failures == 0,
    }


def main() -> int:
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("--repo", type=Path, default=Path.cwd())
    parser.add_argument("--candidate", type=Path, help="optional ABI-compatible RC4 shared library")
    parser.add_argument("--seed", type=int, default=20260923)
    parser.add_argument("--random-cases", type=int, default=100)
    parser.add_argument("--mutation-check", action="store_true")
    args = parser.parse_args()
    root = args.repo.resolve()
    for name, expected in HASHES.items():
        if hashlib.sha256((root / name).read_bytes()).hexdigest() != expected:
            parser.error(f"source changed: {name}")
    source = (root / "sdk/lib/cryptlib/rc4.c").read_text()
    report = {
        "source_sha256": HASHES["sdk/lib/cryptlib/rc4.c"],
        "seed": args.seed,
        "host_abi": "native Linux C ABI; Windows guest ABI untested",
        "source_hashes": HASHES,
    }
    with tempfile.TemporaryDirectory(prefix="rc4-grade-") as directory:
        temporary = Path(directory)
        reference = load(build_c(root, temporary, "genuine", source))
        candidate_path = args.candidate.resolve() if args.candidate else build_rust(root, temporary)
        report["candidate"] = str(candidate_path)
        report["grade"] = grade(reference, load(candidate_path), args.seed, args.random_cases)
        if args.mutation_check:
            mutations = [
                (
                    "key_schedule",
                    "stateIndex += key[keyIndex] + a;",
                    "stateIndex += key[keyIndex] + a + 1;",
                ),
                (
                    "xor_vs_copy",
                    "*inoutString++ ^= s[(a+b) & 0xff];",
                    "*inoutString++ = s[(a+b) & 0xff];",
                ),
            ]
            report["mutations"] = []
            for name, before, after in mutations:
                if source.count(before) != 1:
                    raise RuntimeError(
                        f"mutation anchor {name} occurrence count {source.count(before)}"
                    )
                mutant = load(build_c(root, temporary, name, source.replace(before, after, 1)))
                result = grade(reference, mutant, args.seed, args.random_cases)
                report["mutations"].append(
                    {
                        "name": name,
                        "detected": not result["passed"],
                        "mismatches": result["mismatches"],
                        "first_example": result["examples"][:1],
                    }
                )
    print(json.dumps(report, indent=2))
    return (
        0
        if report["grade"]["passed"]
        and all(mutation["detected"] for mutation in report.get("mutations", []))
        else 1
    )


if __name__ == "__main__":
    raise SystemExit(main())
