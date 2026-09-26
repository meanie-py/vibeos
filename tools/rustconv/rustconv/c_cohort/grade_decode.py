#!/usr/bin/env python3
"""Compare a JScript decode_source candidate with the genuine Wine C body."""

from __future__ import annotations

import argparse
import ast
import ctypes
import hashlib
import json
import random
import re
import subprocess
import tempfile
from pathlib import Path

SOURCE = "dll/win32/jscript/decode.c"
SHA256 = "3443a8d012990725000e8b3e190bce24369927c7c03a7f0161bbb4d2093ff166"
HEADER = "dll/win32/jscript/jscript.h"
RESOURCE = "dll/win32/jscript/resource.h"
WINERROR = "sdk/include/psdk/winerror.h"
WINETEST = "modules/rostests/winetests/jscript/run.c"
PINNED_INPUTS = {
    HEADER: "6e62e0fb600c795506773e0bdd57add65a19027245a89df4adf6cd399855867e",
    RESOURCE: "64dafc3b64909cb9b7fff18e83bc32db73de8eb6da9e3afe69c927c8fa16f693",
    WINERROR: "d13af64457cbf86cedc2788363edc0e4985e82f923e5cfe0af1b9685b621ed86",
    WINETEST: "02b471e3ab3d8f6ec6b144e8f7fb84c5ae6082d430ed1e6d47c9f201b17f9ddd",
}
GUARD = 16
PAD = 256
SENTINEL = 0xA55A
WCHAR = ctypes.c_uint16


def digest(path: Path) -> str:
    return hashlib.sha256(path.read_bytes()).hexdigest()


def actual_macros(root: Path) -> str:
    found = []
    for path, name in (
        (WINERROR, "FACILITY_CONTROL"),
        (WINERROR, "SEVERITY_ERROR"),
        (WINERROR, "MAKE_HRESULT"),
        (WINERROR, "_HRESULT_TYPEDEF_"),
        (WINERROR, "S_OK"),
        (WINERROR, "E_FAIL"),
        (RESOURCE, "IDS_INVALID_CHAR"),
        (HEADER, "MAKE_JSERROR"),
        (HEADER, "JS_E_INVALID_CHAR"),
    ):
        lines = (root / path).read_text().splitlines()
        matches = [
            index
            for index, line in enumerate(lines)
            if re.match(r"#define\s+" + name + r"(?:\s|\()", line)
        ]
        if name == "_HRESULT_TYPEDEF_":
            matches = [index for index in matches if "((HRESULT)" in lines[index]]
        if len(matches) != 1:
            raise RuntimeError(f"expected exactly one {name} in {path}, found {len(matches)}")
        index = matches[0]
        definition = lines[index]
        while definition.endswith("\\"):
            index += 1
            definition += "\n" + lines[index]
        found.append(definition)
    return "\n".join(found) + "\n"


def build_reference(root: Path, temporary: Path, name: str, source_text: str) -> Path:
    work = temporary / name
    work.mkdir()
    (work / "wine").mkdir()
    (work / "decode.c").write_text(source_text)
    (work / "wine/debug.h").write_text(
        "#pragma once\n#define WINE_DEFAULT_DEBUG_CHANNEL(x)\n"
        "#define FIXME(...) ((void)0)\n#define TRACE(...) ((void)0)\n"
        '#define debugstr_w(x) ""\n'
    )
    (work / "jscript.h").write_text(
        "#pragma once\n#include <stdint.h>\n#include <stddef.h>\n"
        "typedef wchar_t WCHAR;\ntypedef uint32_t DWORD;\ntypedef int BOOL;\n"
        "typedef int32_t HRESULT;\n#define TRUE 1\n#define FALSE 0\n"
        + actual_macros(root)
        + "#define ARRAY_SIZE(a) (sizeof(a)/sizeof((a)[0]))\n"
        "static int host_wcsncmp(const WCHAR*a,const WCHAR*b,size_t n) {\n"
        "  while(n--) { if(*a!=*b) return *a<*b?-1:1; if(!*a) return 0; a++; b++; }\n"
        "  return 0;\n}\n#define wcsncmp host_wcsncmp\n"
    )
    library = work / "libdecode.so"
    cmd = [
        "cc",
        "-std=c11",
        "-fshort-wchar",
        "-fPIC",
        "-shared",
        "-O2",
        "-I",
        str(work),
        str(work / "decode.c"),
        "-o",
        str(library),
    ]
    result = subprocess.run(cmd, capture_output=True, text=True)
    if result.returncode:
        raise RuntimeError(f"reference compile failed ({result.returncode}):\n{result.stderr}")
    return library


def load(path: Path):
    library = ctypes.CDLL(str(path))
    function = library.decode_source
    function.argtypes = [ctypes.POINTER(WCHAR)]
    function.restype = ctypes.c_int32
    return function


def invoke(function, value: str):
    units = [ord(char) for char in value]
    if any(unit == 0 or unit > 0xFFFF for unit in units):
        raise ValueError("corpus permits only nonzero UTF-16 BMP units")
    count = GUARD + len(units) + 1 + PAD + GUARD
    buffer = (WCHAR * count)(*([SENTINEL] * count))
    for index, unit in enumerate(units):
        buffer[GUARD + index] = unit
    buffer[GUARD + len(units)] = 0
    pointer = ctypes.cast(ctypes.byref(buffer, GUARD * ctypes.sizeof(WCHAR)), ctypes.POINTER(WCHAR))
    status = function(pointer)
    result = tuple(buffer)
    guards_ok = all(unit == SENTINEL for unit in result[:GUARD]) and all(
        unit == SENTINEL for unit in result[GUARD + len(units) + 1 :]
    )
    return status, result, guards_ok


def winetest_fixtures(root: Path) -> list[str]:
    source = (root / WINETEST).read_text()
    literals = re.findall(r'(?:run_script|SysAllocString)\(L"((?:\\.|[^"\\])*)"', source)
    values = [ast.literal_eval('"' + literal + '"') for literal in literals]
    return [value for value in values if "#@~^" in value and "^#~@" in value]


def encoded_script_generator(root: Path):
    """Derive stimulus tables from the pinned original C, never expected output."""
    source = (root / SOURCE).read_text()

    def values(name: str) -> list[int]:
        match = re.search(r"static const [^\n]*\b" + name + r"\b[^=]*=\s*\{(.*?)\};", source, re.S)
        if match is None:
            raise RuntimeError(f"missing original C table {name}")
        return [int(token, 0) for token in re.findall(r"0x[0-9a-fA-F]+|\b\d+\b", match.group(1))]

    phases = values("pick_encoding")
    raw_dictionary = values("dictionary")
    digit_values = values("digits")
    if len(phases) != 64 or len(raw_dictionary) != 128 * 3 or len(digit_values) != 128:
        raise RuntimeError("original C table dimensions changed")
    dictionary = [raw_dictionary[index : index + 3] for index in range(0, len(raw_dictionary), 3)]
    inverse = {value: chr(index) for index, value in enumerate(digit_values) if value != 0xFF}
    if set(inverse) != set(range(64)):
        raise RuntimeError("original C digit alphabet is not a 64-value bijection")

    def dword(value: int) -> str:
        if not 0 <= value <= 0xFFFFFFFF:
            raise ValueError("encoded length/checksum outside DWORD")
        chunks = [
            (value >> 2) & 63,
            ((value & 3) << 4) | ((value >> 12) & 15),
            (((value >> 8) & 15) << 2) | ((value >> 22) & 3),
            (value >> 16) & 63,
            (value >> 26) & 63,
            ((value >> 24) & 3) << 4,
        ]
        return "".join(inverse[chunk] for chunk in chunks) + "=="

    def generate(body: str) -> str:
        checksum = sum(dictionary[ord(char)][phases[index % 64]] for index, char in enumerate(body))
        return "#@~^" + dword(len(body)) + body + dword(checksum) + "^#~@"

    return generate


def cases(root: Path, seed: int, random_cases: int):
    fixtures = winetest_fixtures(root)
    if len(fixtures) < 3:
        raise RuntimeError("fewer than three real encoded-script fixtures found")
    valid = fixtures[:3]
    rng = random.Random(seed)
    generate = encoded_script_generator(root)
    yield "plain-empty", ""
    yield "plain-ascii", "report('plain');"
    yield "plain-unicode", "\u00e9\u20ac\u4e2d"
    for index, fixture in enumerate(fixtures):
        yield f"winetest-{index}", fixture
    body_alphabet = "abcdefXYZ0123456789!#$%^&*()_-+./:;?[]{}"
    for length in (0, 1, 2, 3, 7, 15, 16, 31, 63, 64, 65, 127, 128, 255):
        body = "".join(rng.choice(body_alphabet) for _ in range(length))
        yield f"generated-{length}", generate(body)
    for index in range(random_cases // 2):
        body = "".join(rng.choice(body_alphabet) for _ in range(rng.randrange(0, 256)))
        yield f"generated-random-{index}", generate(body)
    for index in range(random_cases):
        fixture = valid[index % len(valid)]
        alphabet = "abcXYZ0123456789 ;()"
        prefix = "".join(rng.choice(alphabet) for _ in range(rng.randrange(0, 25)))
        suffix = "".join(rng.choice(alphabet) for _ in range(rng.randrange(0, 25)))
        yield f"wrapped-{index}", prefix + fixture + suffix
        if index % 7 == 0:
            yield f"two-block-{index}", prefix + valid[0] + ";" + valid[1] + suffix
        if index % 5 == 0:
            pos = fixture.find("#@~^") + 4
            replacement = "*" if fixture[pos] != "*" else "?"
            yield f"bad-length-{index}", fixture[:pos] + replacement + fixture[pos + 1 :]
        if index % 5 == 1:
            pos = fixture.rfind("^#~@") - 2
            yield f"bad-checksum-{index}", fixture[:pos] + "!" + fixture[pos + 1 :]
        if index % 5 == 2:
            pos = fixture.rfind("^#~@")
            yield f"bad-end-{index}", fixture[:pos] + "!" + fixture[pos + 1 :]
    for index in range(random_cases):
        alphabet = "abcdefXYZ12345#@~^=.;() "
        yield f"random-{index}", "".join(rng.choice(alphabet) for _ in range(rng.randrange(0, 48)))


def grade(root: Path, reference, candidate, seed: int, random_cases: int):
    count = mismatch_count = guard_failures = 0
    examples = []
    for name, value in cases(root, seed, random_cases):
        expected = invoke(reference, value)
        actual = invoke(candidate, value)
        count += 1
        if not expected[2] or not actual[2]:
            guard_failures += 1
        if expected != actual:
            mismatch_count += 1
            if len(examples) < 5:
                examples.append(
                    {
                        "case": name,
                        "input_units": len(value),
                        "reference_status": expected[0],
                        "candidate_status": actual[0],
                        "reference_guard": expected[2],
                        "candidate_guard": actual[2],
                        "first_difference": next(
                            (
                                i
                                for i, pair in enumerate(zip(expected[1], actual[1], strict=False))
                                if pair[0] != pair[1]
                            ),
                            None,
                        ),
                    }
                )
    return {
        "cases": count,
        "mismatches": mismatch_count,
        "guard_failures": guard_failures,
        "examples": examples,
        "passed": mismatch_count == 0 and guard_failures == 0,
    }


def main() -> int:
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("candidate", nargs="?", type=Path)
    parser.add_argument("--repo", type=Path, default=Path.cwd())
    parser.add_argument("--seed", type=int, default=20260923)
    parser.add_argument("--random-cases", type=int, default=100)
    parser.add_argument("--mutation-check", action="store_true")
    args = parser.parse_args()
    root = args.repo.resolve()
    if digest(root / SOURCE) != SHA256:
        parser.error("decode.c source hash changed")
    for path, expected_hash in PINNED_INPUTS.items():
        if digest(root / path) != expected_hash:
            parser.error(f"pinned header/fixture changed: {path}")
    if args.candidate is None and not args.mutation_check:
        parser.error("supply candidate library or --mutation-check")
    source = (root / SOURCE).read_text()
    report = {
        "source": SOURCE,
        "source_sha256": SHA256,
        "seed": args.seed,
        "host_abi": "Linux native C ABI with -fshort-wchar; Windows PE ABI untested",
        "input_sha256": PINNED_INPUTS,
    }
    with tempfile.TemporaryDirectory(prefix="jscript-decode-grade-") as directory:
        temporary = Path(directory)
        genuine = load(build_reference(root, temporary, "genuine", source))
        for fixture in winetest_fixtures(root)[:3]:
            if invoke(genuine, fixture)[0] != 0:
                raise RuntimeError("a genuine winetest encoded-script fixture no longer decodes")
        for name, value in cases(root, args.seed, args.random_cases):
            if name.startswith("generated-") and invoke(genuine, value)[0] != 0:
                raise RuntimeError(f"generated encoded-script case is not valid: {name}")
        report["reference_safety"] = grade(root, genuine, genuine, args.seed, args.random_cases)
        if not report["reference_safety"]["passed"]:
            raise RuntimeError("genuine C reference violates corpus guard bounds")
        if args.candidate is not None:
            report["candidate"] = str(args.candidate.resolve())
            report["grade"] = grade(
                root, genuine, load(args.candidate.resolve()), args.seed, args.random_cases
            )
        if args.mutation_check:
            mutants = [
                ("table_phase", "pick_encoding[j%64]", "pick_encoding[(j+1)%64]"),
                ("checksum_inverted", "s != csum", "s == csum"),
            ]
            report["mutations"] = []
            for name, before, after in mutants:
                if source.count(before) != 1:
                    raise RuntimeError(
                        f"mutation anchor {name} occurrence count {source.count(before)}"
                    )
                mutant = load(
                    build_reference(root, temporary, name, source.replace(before, after, 1))
                )
                result = grade(root, genuine, mutant, args.seed, args.random_cases)
                report["mutations"].append(
                    {
                        "name": name,
                        "detected": not result["passed"],
                        "mismatches": result["mismatches"],
                        "guard_failures": result["guard_failures"],
                        "first_example": result["examples"][:1],
                    }
                )
    print(json.dumps(report, indent=2))
    return (
        0
        if report.get("grade", {}).get("passed", True)
        and all(mutant["detected"] for mutant in report.get("mutations", []))
        else 1
    )


if __name__ == "__main__":
    raise SystemExit(main())
