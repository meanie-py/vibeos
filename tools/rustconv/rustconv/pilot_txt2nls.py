#!/usr/bin/env python3
"""Differential grade for the genuine ReactOS txt2nls C++ host tool.

The candidate interface is: <program> <input.txt> <output.nls>.
Expected binary output is generated only by the compiled C++ source and is never
embedded in prompts or this harness. The reference is compiled once per run.
"""

from __future__ import annotations

import argparse
import hashlib
import json
import os
import shlex
import subprocess
import tempfile
import time
from dataclasses import dataclass
from pathlib import Path

from .attempts import CHECKER_FINGERPRINT, record_validation

SOURCE = Path("sdk/tools/txt2nls/main.cpp")
SOURCE_SHA256 = "74053f42959ab949e58ab27587672740f954b2ffba5e3a931223fe8f9d6c1355"
REAL_CASES = (
    "media/nls/src/c_1252.txt",  # SBCS
    "media/nls/src/c_437.txt",  # SBCS with GLYPHTABLE
    "media/nls/src/c_932.txt",  # DBCS with two lead-byte ranges
    "media/nls/src/c_936.txt",  # larger DBCS table
)
SYNTHETIC_CASES = {
    "small_sbcs": """CODEPAGE 1252
CPINFO 1 0x3f 0x003f
MBTABLE 2
0x3f 0x003f
0x41 0x0041
WCTABLE 2
0x003f 0x3f
0x0041 0x41
ENDCODEPAGE
""",
    "small_dbcs": """CODEPAGE 932
CPINFO 2 0x8140 0x003f
MBTABLE 1
0x3f 0x003f
WCTABLE 2
0x003f 0x003f
0x4e00 0x8140
DBCSRANGE 1
0x81 0x81
DBCSTABLE 1
0x40 0x4e00
ENDCODEPAGE
""",
}
# Held-out lexical forms exercise C++ library semantics, not just nominal tables.
_sbcs = SYNTHETIC_CASES["small_sbcs"]
SYNTHETIC_CASES.update(
    {
        "crlf_comments": "; leading comment\r\n" + _sbcs.replace("\n", " ; comment\r\n"),
        "octal_defaults": _sbcs.replace("CPINFO 1 0x3f 0x003f", "CPINFO 1 077 077"),
        "numeric_prefix_and_suffix": _sbcs.replace(
            "CPINFO 1 0x3f 0x003f", "CPINFO 1 +0X3Ftail 63suffix"
        ),
        "no_final_newline": _sbcs.rstrip("\n"),
    }
)
# All negatives reach an explicit error() branch before serialization. No malformed
# numeric conversion, out-of-range table access, wraparound lead-byte loop, or
# undefined behavior is used as a reference oracle.
INVALID_CASES = {
    # The genuine tokenizer passes token_end as substr LENGTH, so leading
    # keyword whitespace is rejected. Preserve the source quirk in the oracle.
    "leading_keyword_space": (" " + _sbcs, "expected CODEPAGE"),
    "invalid_max_char_size": (
        "CODEPAGE 1252\nCPINFO 3 0x3f 0x003f\n",
        "Expected 1 or 2",
    ),
    "oversized_mbtable": (
        "CODEPAGE 1252\nCPINFO 1 0x3f 0x003f\nMBTABLE 257\n",
        "MBTABLE size can't be larger",
    ),
    "missing_mbtable": (
        "CODEPAGE 1252\nCPINFO 1 0x3f 0x003f\nWCTABLE 0\nENDCODEPAGE\n",
        "File has no MBTABLE",
    ),
    "missing_wctable": (
        "CODEPAGE 1252\nCPINFO 1 0x3f 0x003f\nMBTABLE 0\nENDCODEPAGE\n",
        "File has no WCTABLE",
    ),
    "duplicate_mbtable": (
        "CODEPAGE 1252\nCPINFO 1 0x3f 0x003f\nMBTABLE 0\nMBTABLE 0\n",
        "MBTABLE can only be declared once",
    ),
}
# Mutations retain valid C++ and exercise distinct binary regions. These are
# validation probes, not candidate implementations or expected answers.
MUTATIONS = {
    "header_default_character": (
        "FileHeader.TransDefaultChar = mb_table[FileHeader.DefaultChar];",
        "FileHeader.TransDefaultChar = mb_table[0x41];",
    ),
    "wide_table_entry": (
        "wc_table[wc] = mb;",
        "wc_table[wc] = FileHeader.DefaultChar;",
    ),
    "dbcs_entry": (
        "dbcs_table[current_offset + mb] = wc;",
        "dbcs_table[current_offset + mb] = FileHeader.UniDefaultChar;",
    ),
    "dbcs_mode_flag": (
        "FileHeader.MaximumCharacterSize == 1 ? 0 : 4;",
        "FileHeader.MaximumCharacterSize == 1 ? 4 : 0;",
    ),
    "missing_wctable_acceptance": (
        'if (!has_wctable)\n        error("File has no WCTABLE statement");',
        'if (false && !has_wctable)\n        error("File has no WCTABLE statement");',
    ),
}


@dataclass(frozen=True)
class Outcome:
    code: int | None
    data: bytes | None
    stderr: str


def sha256(data: bytes) -> str:
    return hashlib.sha256(data).hexdigest()


def compile_cpp(source: Path, binary: Path) -> list[str]:
    argv = ["g++", "-std=c++17", "-O2", "-Wall", "-Wextra", str(source), "-o", str(binary)]
    subprocess.run(argv, check=True, text=True, capture_output=True)
    return argv


def header_manifest(source: Path) -> tuple[list[dict[str, str]], str]:
    # -M records the full dependency closure, including C++ standard library
    # and libc headers. A preprocessed hash additionally captures macros and
    # compiler builtins that path/digest lists alone do not express.
    dep_output = subprocess.run(
        ["g++", "-std=c++17", "-M", str(source)],
        check=True,
        capture_output=True,
        text=True,
    ).stdout.replace("\\\n", " ")
    dependencies = shlex.split(dep_output.split(":", 1)[1])
    headers = [
        {"path": str(Path(path).resolve()), "sha256": sha256(Path(path).read_bytes())}
        for path in dependencies
        if Path(path).resolve() != source.resolve()
    ]
    preprocessed = subprocess.run(
        ["g++", "-std=c++17", "-E", str(source)],
        check=True,
        capture_output=True,
    ).stdout
    return headers, sha256(preprocessed)


def execute(program: Path, input_path: Path, output_path: Path) -> Outcome:
    output_path.unlink(missing_ok=True)
    try:
        result = subprocess.run(
            [str(program), str(input_path), str(output_path)],
            capture_output=True,
            text=True,
            timeout=30,
            check=False,
        )
    except subprocess.TimeoutExpired:
        return Outcome(
            None,
            output_path.read_bytes() if output_path.exists() else None,
            "timeout after 30 seconds",
        )
    return Outcome(
        result.returncode, output_path.read_bytes() if output_path.exists() else None, result.stderr
    )


def materialize_inputs(
    repo: Path, work: Path, all_codepages: bool
) -> list[tuple[str, Path, bool, str | None]]:
    cases: list[tuple[str, Path, bool, str | None]] = []
    real_paths = (
        sorted((repo / "media/nls/src").glob("c_*.txt"))
        if all_codepages
        else [repo / relative for relative in REAL_CASES]
    )
    for path in real_paths:
        if not path.is_file():
            raise FileNotFoundError(path)
        cases.append((path.stem, path, True, None))
    for name, body in SYNTHETIC_CASES.items():
        path = work / f"{name}.txt"
        path.write_text(body, encoding="ascii")
        cases.append((name, path, True, None))
    for name, (body, error_fragment) in INVALID_CASES.items():
        path = work / f"{name}.txt"
        path.write_text(body, encoding="ascii")
        cases.append((name, path, False, error_fragment))
    return cases


def reference_results(
    program: Path,
    cases: list[tuple[str, Path, bool, str | None]],
    work: Path,
    allow_unsupported: bool,
) -> tuple[dict[str, Outcome], list[dict[str, object]]]:
    results: dict[str, Outcome] = {}
    unsupported: list[dict[str, object]] = []
    for name, path, valid, error_fragment in cases:
        result = execute(program, path, work / f"reference-{name}.nls")
        if valid and (result.code != 0 or result.data is None):
            if not allow_unsupported:
                raise RuntimeError(
                    f"reference failed valid case {name}: {result.code} {result.stderr}"
                )
            unsupported.append(
                {
                    "name": name,
                    "input": str(path),
                    "status": result.code,
                    "output_exists": result.data is not None,
                    "stderr": result.stderr[:1000],
                }
            )
            continue
        if not valid and (
            result.code == 0
            or result.data is not None
            or (error_fragment is None or error_fragment not in result.stderr)
        ):
            raise RuntimeError(f"unexpected reference failure shape {name}: {result}")
        results[name] = result
    return results, unsupported


def grade_candidate(
    program: Path,
    cases: list[tuple[str, Path, bool, str | None]],
    expected: dict[str, Outcome],
    work: Path,
) -> dict[str, dict[str, object]]:
    verdicts: dict[str, dict[str, object]] = {}
    for name, path, valid, _ in cases:
        if name not in expected:
            continue
        got = execute(program, path, work / f"candidate-{name}.nls")
        want = expected[name]
        # Invalid inputs are graded on exact exit status and absence of output.
        # Diagnostic wording may differ in a legitimate Rust implementation.
        passed = (
            (got.code == 0 and got.data == want.data)
            if valid
            else (got.code == want.code and got.data is None)
        )
        verdicts[name] = {
            "passed": passed,
            "kind": "valid" if valid else "invalid",
            "reference_status": want.code,
            "candidate_status": got.code,
            "reference_bytes": len(want.data) if want.data is not None else None,
            "candidate_bytes": len(got.data) if got.data is not None else None,
            "reference_sha256": sha256(want.data) if want.data is not None else None,
            "candidate_sha256": sha256(got.data) if got.data is not None else None,
        }
    return verdicts


def mutation_check(
    repo: Path,
    cases: list[tuple[str, Path, bool, str | None]],
    expected: dict[str, Outcome],
    work: Path,
) -> dict[str, list[str]]:
    source = (repo / SOURCE).read_text(encoding="utf-8")
    detected: dict[str, list[str]] = {}
    for name, (before, after) in MUTATIONS.items():
        if before not in source:
            raise RuntimeError(f"mutation anchor missing: {name}")
        mutant_source = work / f"{name}.cpp"
        mutant_source.write_text(source.replace(before, after, 1), encoding="utf-8")
        mutant_binary = work / name
        compile_cpp(mutant_source, mutant_binary)
        verdicts = grade_candidate(mutant_binary, cases, expected, work)
        failures = sorted(case for case, result in verdicts.items() if not result["passed"])
        if not failures:
            raise RuntimeError(f"mutation escaped: {name}")
        detected[name] = failures
    return detected


def main() -> int:
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("--repo", type=Path, required=True)
    parser.add_argument(
        "--candidate", type=Path, help="executable with <input.txt> <output.nls> interface"
    )
    parser.add_argument(
        "--mutations", action="store_true", help="compile and grade five wrong C++ variants"
    )
    parser.add_argument(
        "--all-codepages",
        action="store_true",
        help="include every media/nls/src/c_*.txt; report reference failures as unsupported",
    )
    parser.add_argument(
        "--manifest", type=Path, help="save reproducibility metadata (no expected bytes)"
    )
    parser.add_argument("--attempt-id", help="attach this grade to an existing immutable purchase")
    args = parser.parse_args()
    if args.attempt_id and not args.candidate:
        parser.error("--attempt-id requires --candidate")
    repo = args.repo.resolve()
    source = repo / SOURCE
    if not source.is_file():
        parser.error(f"missing {source}")
    actual_source_sha256 = sha256(source.read_bytes())
    if actual_source_sha256 != SOURCE_SHA256:
        parser.error(
            f"source drift: expected sha256 {SOURCE_SHA256}, got {actual_source_sha256}; "
            "review and update the grader"
        )
    with tempfile.TemporaryDirectory(prefix="grade-txt2nls-") as tmp:
        work = Path(tmp)
        reference = work / "txt2nls-reference"
        build_argv = compile_cpp(source, reference)
        cases = materialize_inputs(repo, work, args.all_codepages)
        expected, unsupported = reference_results(reference, cases, work, args.all_codepages)
        report: dict = {
            "target": str(SOURCE),
            "source_sha256": actual_source_sha256,
            "input_sha256": {name: sha256(path.read_bytes()) for name, path, _, _ in cases},
            "reference_cases": len(cases),
            "valid_cases": sum(valid for name, _, valid, _ in cases if name in expected),
            "invalid_cases": sum(not valid for _, _, valid, _ in cases),
            "reference_unsupported": unsupported,
        }
        if args.manifest:
            headers, preprocessed_sha256 = header_manifest(source)
            manifest = {
                "source": str(SOURCE),
                "source_sha256": actual_source_sha256,
                "direct_includes": [
                    line.strip()
                    for line in source.read_text(encoding="utf-8").splitlines()
                    if line.startswith("#include ")
                ],
                "header_dependencies": headers,
                "preprocessed_sha256": preprocessed_sha256,
                "compiler": subprocess.run(
                    ["g++", "--version"], check=True, capture_output=True, text=True
                ).stdout.splitlines()[0],
                "reference_build_argv": build_argv,
                "candidate_interface": "<program> <input.txt> <output.nls>",
                "standalone_rust_candidate_compile_argv": [
                    "rustc",
                    "--edition=2021",
                    "-O",
                    "<candidate.rs>",
                    "-o",
                    "<candidate>",
                ],
                "inputs": [
                    {
                        "name": name,
                        "path": str(path.relative_to(repo))
                        if path.is_relative_to(repo)
                        else "synthetic:" + name,
                        "sha256": sha256(path.read_bytes()),
                        "valid": valid,
                        "error_fragment": error_fragment,
                    }
                    for name, path, valid, error_fragment in cases
                ],
            }
            args.manifest.parent.mkdir(parents=True, exist_ok=True)
            args.manifest.write_text(json.dumps(manifest, indent=2) + "\n", encoding="utf-8")
        if args.candidate:
            candidate = args.candidate.resolve()
            if not candidate.is_file() or not os.access(candidate, os.X_OK):
                parser.error(f"candidate is not executable: {candidate}")
            started = time.monotonic()
            report["candidate_verdicts"] = grade_candidate(candidate, cases, expected, work)
            report["grade_seconds"] = time.monotonic() - started
            report["candidate_sha256"] = sha256(candidate.read_bytes())
        if args.mutations:
            selected = [
                case
                for case in cases
                if case[0] in {Path(x).stem for x in REAL_CASES}
                or case[0] in SYNTHETIC_CASES
                or case[0] in INVALID_CASES
            ]
            report["mutations_detected_by"] = mutation_check(repo, selected, expected, work)
        if args.attempt_id:
            passed = not unsupported and all(
                v["passed"] for v in report["candidate_verdicts"].values()
            )
            record_validation(
                args.attempt_id,
                checker=CHECKER_FINGERPRINT,
                verdict="host_grade_pass" if passed else "host_grade_fail",
                scope="finite host corpus; not guest ABI or semantic equivalence proof",
                report=report,
            )
        print(json.dumps(report, indent=2))
        if unsupported:
            return 2
        if args.candidate and not all(v["passed"] for v in report["candidate_verdicts"].values()):
            return 1
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
