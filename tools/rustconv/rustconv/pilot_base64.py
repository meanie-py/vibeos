#!/usr/bin/env python3
"""Host ABI differential grade for the genuine vendored Mbed TLS base64.c.

Candidate must export mbedtls_base64_encode and mbedtls_base64_decode with the
C signatures. This is behavior coverage on the current host, not a proof of
constant-time execution or compatibility with Windows PE ABIs.
"""

from __future__ import annotations

import argparse
import base64
import ctypes
import hashlib
import json
import random
import subprocess
import tempfile
import time
from pathlib import Path

ROOT = Path.cwd()
SOURCE = ROOT / "dll/3rdparty/mbedtls/base64.c"
SOURCE_SHA256 = "42fb3f63678521d4ba877dc7227337b5534813aa3b8e7beda87335e598bbab6c"
INCLUDES = [
    ROOT / "sdk/include/reactos/libs",
    ROOT / "sdk/include/reactos/libs/mbedtls",
    ROOT / "dll/3rdparty/mbedtls",
]
MANIFEST = Path(__file__).parent.parent / "data/pilots/base64.json"
BYTE = ctypes.c_ubyte
PTR = ctypes.POINTER(BYTE)
GUARD = 16
SENTINEL = 0xA5


def compile_c(source: Path, output: Path) -> None:
    subprocess.run(
        [
            "cc",
            "-std=c11",
            "-O2",
            "-fPIC",
            "-shared",
            "-DMBEDTLS_BASE64_C",
            *(arg for inc in INCLUDES for arg in ("-I", str(inc))),
            str(source),
            "-o",
            str(output),
        ],
        check=True,
        capture_output=True,
        text=True,
    )


def load_api(path: Path):
    dll = ctypes.CDLL(str(path))
    for name in ("mbedtls_base64_encode", "mbedtls_base64_decode"):
        fn = getattr(dll, name)
        fn.argtypes = [PTR, ctypes.c_size_t, ctypes.POINTER(ctypes.c_size_t), PTR, ctypes.c_size_t]
        fn.restype = ctypes.c_int
    return dll


def invoke(dll, operation: str, payload: bytes, capacity: int, null_dst: bool):
    src = (BYTE * max(1, len(payload)))()
    for i, value in enumerate(payload):
        src[i] = value
    dst = (BYTE * (GUARD + max(capacity, 1) + GUARD))(
        *([SENTINEL] * (GUARD + max(capacity, 1) + GUARD))
    )
    dst_ptr = None if null_dst else ctypes.cast(ctypes.byref(dst, GUARD), PTR)
    olen = ctypes.c_size_t(0xBADC0FFEE)
    fn = getattr(dll, "mbedtls_base64_" + operation)
    result = fn(dst_ptr, capacity, ctypes.byref(olen), src, len(payload))
    guard_ok = null_dst or (
        all(value == SENTINEL for value in dst[:GUARD])
        and all(value == SENTINEL for value in dst[GUARD + capacity :])
    )
    return (result, olen.value, None if null_dst else bytes(dst), guard_ok)


def cases(seed=0xBA5E64, random_cases=100, max_length=512):
    # Include all three remainder classes and byte extrema, then seeded varied lengths.
    payloads = [
        b"",
        b"\x00",
        b"\xff",
        b"\x00\xff",
        b"\xff\x00\x80",
        b"a",
        b"ab",
        b"abc",
        b"abcd",
        bytes(range(256)),
    ]
    rng = random.Random(seed)
    payloads += [bytes(rng.randrange(256) for _ in range(n)) for n in range(1, 97)]
    payloads += [
        bytes(rng.randrange(256) for _ in range(rng.randrange(0, max_length + 1)))
        for _ in range(random_cases)
    ]
    for payload in payloads:
        required = 0 if not payload else ((len(payload) + 2) // 3) * 4 + 1
        for capacity in sorted({0, max(0, required - 1), required, required + 1}):
            yield ("encode", payload, capacity, False)
        yield ("encode", payload, required, True)

    texts = [base64.b64encode(p) for p in payloads]
    texts += [
        b"A",
        b"AA",
        b"AAA",
        b"AAAA",
        b"A===",
        b"AA=A",
        b"====",
        b"AA==A",
        b"AA==",
        b"AAA=",
        b"AA=A",
        b"AA== ",
        b"AA==\r\n",
        b"A A=",
        b"AA=\x00",
        b"AA\xff=",
        b"AA\x80=",
        b"\n",
        b"\r",
        b" \n",
        b"AA==\r",
        b"AA==\r\n",
        b" AA==",
        b"AA== ",
        b"AA==\n\t",
        b"AA==\nAA==",
    ]
    texts += [b" \r\n" + t + b" \n" for t in texts[:32]]
    texts += [t[:4] + b"\r\n" + t[4:] for t in texts[:32]]
    texts += [bytes(rng.randrange(256) for _ in range(rng.randrange(0, 33))) for _ in range(150)]
    for text in texts:
        capacity = max(4, len(text))
        for size in (0, max(0, capacity // 2), capacity):
            yield ("decode", text, size, False)
        yield ("decode", text, capacity, True)


def grade(reference, candidate, max_examples=8, *, seed=0xBA5E64, random_cases=100, max_length=512):
    count = 0
    mismatches = []
    mismatch_count = 0
    guard_failures = 0
    for operation, payload, capacity, null_dst in cases(seed, random_cases, max_length):
        expected = invoke(reference, operation, payload, capacity, null_dst)
        actual = invoke(candidate, operation, payload, capacity, null_dst)
        count += 1
        if not expected[3] or not actual[3]:
            guard_failures += 1
        if expected != actual:
            mismatch_count += 1
            if len(mismatches) < max_examples:
                mismatches.append(
                    {
                        "operation": operation,
                        "payload_hex": payload.hex(),
                        "capacity": capacity,
                        "null_dst": null_dst,
                        "expected": [
                            expected[0],
                            expected[1],
                            None if expected[2] is None else expected[2].hex(),
                        ],
                        "actual": [
                            actual[0],
                            actual[1],
                            None if actual[2] is None else actual[2].hex(),
                        ],
                        "reference_guard_ok": expected[3],
                        "candidate_guard_ok": actual[3],
                    }
                )
    return {
        "cases": count,
        "mismatch_examples": mismatches,
        "matched": count - mismatch_count,
        "mismatches": mismatch_count,
        "guard_failures": guard_failures,
        "passed": mismatch_count == 0 and guard_failures == 0,
    }


def mutation_check(temp: Path, genuine) -> list[dict]:
    source = SOURCE.read_text()
    mutations = [
        (
            "encode_bit_shift",
            "mbedtls_ct_base64_enc_char(C3 & 0x3F)",
            "mbedtls_ct_base64_enc_char((C3 >> 1) & 0x3F)",
        ),
        ("padding_acceptance", "if (++equals > 2)", "if (++equals > 4)"),
    ]
    results = []
    for name, before, after in mutations:
        if source.count(before) != 1:
            raise RuntimeError(f"mutation anchor {name} occurs {source.count(before)} times")
        cpath = temp / f"{name}.c"
        so = temp / f"{name}.so"
        cpath.write_text(source.replace(before, after, 1))
        compile_c(cpath, so)
        result = grade(genuine, load_api(so), max_examples=1)
        results.append(
            {
                "mutation": name,
                "detected": not result["passed"],
                "first_mismatch": result["mismatch_examples"][:1],
            }
        )
    return results


def main() -> int:
    global ROOT, SOURCE, INCLUDES
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument(
        "--repo", type=Path, default=Path.cwd(), help="repository root (default: cwd)"
    )
    parser.add_argument(
        "candidate", nargs="?", type=Path, help="Rust or other candidate .so with C ABI exports"
    )
    parser.add_argument(
        "--mutation-check",
        action="store_true",
        help="prove suite rejects two realistic C-source mutations",
    )
    parser.add_argument("--seed", type=lambda s: int(s, 0), default=0xBA5E64)
    parser.add_argument("--random-cases", type=int, default=100)
    parser.add_argument("--max-length", type=int, default=512)
    args = parser.parse_args()
    if args.random_cases < 0 or args.max_length < 0:
        parser.error("case count and maximum length must be nonnegative")
    ROOT = args.repo.resolve()
    SOURCE = ROOT / "dll/3rdparty/mbedtls/base64.c"
    INCLUDES = [
        ROOT / "sdk/include/reactos/libs",
        ROOT / "sdk/include/reactos/libs/mbedtls",
        ROOT / "dll/3rdparty/mbedtls",
    ]
    manifest = json.loads(MANIFEST.read_text())
    for relative, expected_hash in manifest["dependencies_sha256"].items():
        dependency = ROOT / relative
        observed_hash = hashlib.sha256(dependency.read_bytes()).hexdigest()
        if observed_hash != expected_hash:
            parser.error(
                f"dependency changed: {relative}: expected {expected_hash}, "
                f"observed {observed_hash}"
            )
    actual_hash = hashlib.sha256(SOURCE.read_bytes()).hexdigest()
    if actual_hash != SOURCE_SHA256:
        parser.error(f"source hash changed: expected {SOURCE_SHA256}, observed {actual_hash}")
    if args.candidate is None and not args.mutation_check:
        parser.error("supply candidate library or --mutation-check")
    with tempfile.TemporaryDirectory(prefix="base64-grade-") as directory:
        temp = Path(directory)
        reference_path = temp / "genuine-base64.so"
        compile_c(SOURCE, reference_path)
        genuine = load_api(reference_path)
        report: dict = {
            "source": str(SOURCE.relative_to(ROOT)),
            "source_sha256": actual_hash,
            "host": "native current-host C ABI; size_t and pointer width from ctypes",
        }
        if args.candidate is not None:
            report["candidate"] = str(args.candidate.resolve())
            started = time.monotonic()
            report["grade"] = grade(
                genuine,
                load_api(args.candidate.resolve()),
                seed=args.seed,
                random_cases=args.random_cases,
                max_length=args.max_length,
            )
            report["grade_seconds"] = time.monotonic() - started
            report["candidate_sha256"] = hashlib.sha256(args.candidate.read_bytes()).hexdigest()
            report["case_parameters"] = {
                "seed": args.seed,
                "random_cases": args.random_cases,
                "max_length": args.max_length,
            }
        if args.mutation_check:
            report["mutations"] = mutation_check(temp, genuine)
        print(json.dumps(report, indent=2))
        passed = report.get("grade", {}).get("passed", True)
        passed &= all(m["detected"] for m in report.get("mutations", []))
        return 0 if passed else 1


if __name__ == "__main__":
    raise SystemExit(main())
