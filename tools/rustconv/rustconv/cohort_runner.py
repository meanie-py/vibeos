#!/usr/bin/env python3
"""Pinned, offline 12-TU rustconv pilot; no translation or model API calls."""

from __future__ import annotations

import argparse
import hashlib
import importlib
import json
import re
import shutil
import subprocess
import sys
import time
from datetime import datetime, timezone
from pathlib import Path


def digest(path: Path) -> str:
    return hashlib.sha256(path.read_bytes()).hexdigest()


def canonical(value: object) -> bytes:
    return json.dumps(value, sort_keys=True, separators=(",", ":")).encode()


def write_new(path: Path, value: object) -> None:
    with path.open("x", encoding="utf-8") as stream:
        json.dump(value, stream, indent=2, sort_keys=True)
        stream.write("\n")


def roots(args):
    return {
        "repo": args.repo.resolve(),
        "staging": args.staging.resolve(),
        "bank": args.shell32_bank.resolve(),
        "fixtures": args.fixtures_root.resolve(),
    }


def resolve(rootmap, token: str) -> Path:
    root, slash, tail = token.partition(":")
    if (
        not slash
        or root not in rootmap
        or not tail
        or Path(tail).is_absolute()
        or ".." in Path(tail).parts
    ):
        raise ValueError(f"bad artifact token: {token}")
    return rootmap[root] / tail


def sha_bundle(rootmap, tokens: list[str]) -> tuple[str, dict[str, str]]:
    parts = {token: digest(resolve(rootmap, token)) for token in tokens}
    return hashlib.sha256(canonical(parts)).hexdigest(), parts


def specs(staging: Path):
    c = json.loads((staging / "c_cohort/cohort.json").read_text())["source_files"]
    cpp = json.loads((staging / "cpp_cohort/cohort-manifest.json").read_text())
    cdefs = {
        "sdk/lib/crt/mem/memmove.c": ("cnt-difftest", ["diff_memmove"]),
        "sdk/lib/crt/string/strtoi64.c": ("cnt-difftest", ["diff_strtoi64_strtoul"]),
        "sdk/lib/crt/math/arm64/fmod.c": ("cnt-difftest", ["diff_math_fmod"]),
        "sdk/lib/crt/stdlib/qsort.c": ("cnt-difftest", ["diff_qsort"]),
        "sdk/lib/rtl/utf8.c": (
            "rtl-difftest",
            ["diff_rtl_unicode_to_utf8", "diff_rtl_utf8_to_unicode"],
        ),
        "sdk/lib/rtl/dos8dot3.c": (
            "rtl-difftest",
            ["diff_rtl_generate_8dot3_name", "diff_rtl_is_name_legal_dos8dot3"],
        ),
    }
    rows = []
    for item in c:
        source = item["source"]
        row = {
            "case_id": source,
            "split": item["split"],
            "source": "repo:" + source,
            "declared_source_sha256": item["sha256"],
            "language": "C",
            "coverage_scope": "selected exported functions on a finite host corpus",
        }
        if source in cdefs:
            crate, symbols = cdefs[source]
            assert crate == item["oracle"] and symbols == item["test_symbols"]
            provider = item["rust_provider"]
            row.update(
                kind="cli",
                crate=crate,
                symbols=symbols,
                candidate_files=["repo:" + provider],
                grader_files=[
                    f"repo:sdk/lib/rust/{crate}/build.rs",
                    f"repo:sdk/lib/rust/{crate}/tests/diff.rs",
                    "repo:tests/qemu/cli/difftest.py",
                ],
            )
        elif source == "dll/win32/jscript/decode.c":
            row.update(
                kind="decode",
                coverage_scope="decode_source finite host corpus; guest ABI assessed separately",
                candidate_files=["repo:sdk/lib/rust/jscript-rust/src/decode.rs"],
                grader_files=[
                    "fixtures:c_cohort/grade_decode.py",
                    "fixtures:c_cohort/decode_host_shims.c",
                    "repo:dll/win32/jscript/jscript.h",
                    "repo:dll/win32/jscript/resource.h",
                    "repo:sdk/include/psdk/winerror.h",
                    "repo:modules/rostests/winetests/jscript/run.c",
                ],
            )
        elif source == "sdk/lib/cryptlib/rc4.c":
            row.update(
                kind="rc4",
                coverage_scope="rc4_init and rc4_crypt finite host corpus",
                candidate_files=[
                    "repo:sdk/lib/rust/cryptlib-rust/src/rc4.rs",
                    "repo:sdk/lib/rust/cryptlib-rust/src/types.rs",
                ],
                grader_files=[
                    "fixtures:c_cohort/grade_rc4.py",
                    "fixtures:c_cohort/rc4_host.rs",
                    "repo:sdk/lib/cryptlib/rc4.h",
                ],
            )
        else:
            raise ValueError(f"unexpected C source: {source}")
        rows.append(row)
    cpp_sources = {
        item["source"]: (split, item)
        for split in ("development", "held_out")
        for item in cpp[split]
    }
    cp = [
        (
            "sdk/tools/txt2nls/main.cpp",
            "txt2nls",
            [
                "staging:cpp-pilot/txt2nls-faithful.rs",
                "repo:tools/rustconv/.responses/host-pilots/txt2nls/nonthinking/candidate-d4c8ac3a0b93b84e.rs",
            ],
            [
                "repo:tools/rustconv/rustconv/pilot_txt2nls.py",
                "repo:tools/rustconv/rustconv/attempts.py",
                "fixtures:cpp_cohort/grade_semantics.py",
                "fixtures:cpp_cohort/semantics_reference.cpp",
                "fixtures:cpp_cohort/semantics_cli.rs",
                "fixtures:cpp_cohort/cpp_std_semantics.rs",
                "staging:cpp-pilot/repair-manifest.json",
            ],
        ),
        (
            "dll/win32/shell32/utils.cpp",
            "quotes",
            [
                "bank:20260913-012131-utils.cpp/30_SheRemoveQuotesA.rs",
                "bank:20260913-034905-utils.cpp/31_SheRemoveQuotesW.rs",
            ],
            ["fixtures:cpp_cohort/grade_quotes.py", "fixtures:cpp_cohort/host_provenance.py"],
        ),
        (
            "dll/win32/shell32/CEnumIDListBase.cpp",
            "cenum",
            [
                "bank:20260718-205701-CEnumIDListBase.cpp/04_CEnumIDListBase_DeleteList.rs",
                "bank:20260718-205701-CEnumIDListBase.cpp/05_CEnumIDListBase_AppendItemsFromEnumerator.rs",
                "bank:20260718-205701-CEnumIDListBase.cpp/06_CEnumIDListBase_Next.rs",
                "bank:20260718-205701-CEnumIDListBase.cpp/07_CEnumIDListBase_Skip.rs",
                "bank:20260718-205701-CEnumIDListBase.cpp/08_CEnumIDListBase_Reset.rs",
            ],
            ["fixtures:cpp_cohort/grade_cenum.py", "fixtures:cpp_cohort/host_provenance.py"],
        ),
        (
            "dll/win32/shell32/CCopyAsPathMenu.cpp",
            "ccopy",
            ["bank:20260716-183948-CCopyAsPathMenu.cpp/01_CCopyAsPathMenu_DoCopyAsPath.rs"],
            ["fixtures:cpp_cohort/heldout-status.json"],
        ),
    ]
    for source, kind, candidates, graders in cp:
        split, frozen = cpp_sources[source]
        declared = {}
        if kind == "txt2nls":
            declared[candidates[-1]] = frozen["banked_rust_sha256"]
        elif kind == "quotes":
            declared = dict(zip(candidates, frozen["banked_rust_sha256"], strict=True))
        elif kind == "cenum":
            declared = {
                token: frozen["banked_functions_sha256"][token.rsplit("/", 1)[-1]]
                for token in candidates
            }
        else:
            declared[candidates[0]] = frozen["banked_rust_sha256"]
        rows.append(
            {
                "case_id": source,
                "split": split,
                "source": "repo:" + source,
                "declared_source_sha256": frozen["source_sha256"],
                "language": "C++",
                "coverage_scope": {
                    "txt2nls": "whole host tool on finite codepage corpus",
                    "quotes": "SheRemoveQuotesA/W only, not whole utils.cpp",
                    "cenum": "Skip/Reset parity only; Next rejected; other methods unsupported",
                    "ccopy": "DoCopyAsPath unsupported host grade",
                }[kind],
                "kind": kind,
                "candidate_files": candidates,
                "grader_files": graders,
                "declared_candidate_file_sha256": declared,
            }
        )
    if len(rows) != 12 or {
        s: sum(r["split"] == s for r in rows) for s in ("development", "held_out")
    } != {"development": 6, "held_out": 6}:
        raise ValueError("frozen cohort must have six source TUs per split")
    if len({r["case_id"] for r in rows}) != 12:
        raise ValueError("source-level development/held-out overlap")
    return rows


def pipeline_files():
    return [
        "fixtures:cohort_runner.py",
        "fixtures:c_cohort/cohort.json",
        "fixtures:cpp_cohort/cohort-manifest.json",
        "repo:tools/rustconv/rustconv/cohort_scorecard.py",
        "repo:tools/rustconv/rustconv/anchor_candidate.py",
        "repo:tools/rustconv/rustconv/anchor_lower.py",
        "repo:tests/qemu/cli/difftest.py",
        "fixtures:c_cohort/grade_decode.py",
        "fixtures:c_cohort/grade_rc4.py",
        "repo:tools/rustconv/rustconv/pilot_txt2nls.py",
        "repo:tools/rustconv/rustconv/attempts.py",
        "fixtures:cpp_cohort/grade_semantics.py",
        "fixtures:cpp_cohort/grade_quotes.py",
        "fixtures:cpp_cohort/grade_cenum.py",
        "fixtures:cpp_cohort/host_provenance.py",
    ]


def import_scorecard(repo):
    sys.path.insert(0, str(repo / "tools/rustconv"))
    return importlib.import_module("rustconv.cohort_scorecard")


def materialize(args):
    rootmap = roots(args)
    score = import_scorecard(args.repo)
    entries = []
    split_pins = {"development": {}, "held_out": {}}
    for row in specs(args.fixtures_root):
        source_sha = digest(resolve(rootmap, row["source"]))
        if source_sha != row["declared_source_sha256"]:
            raise ValueError(f"original source changed: {row['case_id']}")
        candidate_sha, candidate_parts = sha_bundle(rootmap, row["candidate_files"])
        for token, expected in row.get("declared_candidate_file_sha256", {}).items():
            if candidate_parts[token] != expected:
                raise ValueError(f"banked candidate changed: {token}")
        grader_sha, grader_parts = sha_bundle(rootmap, row["grader_files"])
        pins = score.ArtifactPins(source_sha, candidate_sha, grader_sha)
        split_pins[row["split"]][row["case_id"]] = pins
        entries.append(
            {
                **row,
                "pins": pins.as_dict(),
                "candidate_file_sha256": candidate_parts,
                "grader_file_sha256": grader_parts,
            }
        )
    cohort = score.CohortManifest(**split_pins)
    pipeline_tokens = pipeline_files()
    pipeline_parts = {token: digest(resolve(rootmap, token)) for token in pipeline_tokens}
    pipeline_hash = score.pipeline_fingerprint({k: resolve(rootmap, k) for k in pipeline_tokens})
    return {
        "schema": 1,
        "source_split": "translation_unit",
        "offline": True,
        "cohort_manifest_sha256": cohort.sha256,
        "pipeline_sha256": pipeline_hash,
        "pipeline_files": pipeline_parts,
        "cases": entries,
        "host_compilers": {name: compiler_provenance(name) for name in ("cc", "g++", "rustc")},
        "root_convention": {
            "repo": "--repo",
            "staging": "--staging",
            "bank": "--shell32-bank",
            "fixtures": "--fixtures-root",
        },
        "cost_usd": None,
        "effort_seconds": None,
    }


def verify_manifest(args, frozen):
    current = materialize(args)
    if current != frozen:
        for field in ("cohort_manifest_sha256", "pipeline_sha256", "cases"):
            if current[field] != frozen[field]:
                raise ValueError(
                    f"pinned cohort drift in {field}; do not run held-out on changed inputs"
                )
        raise ValueError("manifest drift")
    return current


def execute(argv: list[str], cwd: Path, prefix: Path):
    start = datetime.now(timezone.utc).isoformat()
    tick = time.monotonic()
    try:
        cp = subprocess.run(argv, cwd=cwd, text=True, capture_output=True, timeout=900)
        rc, out, err = cp.returncode, cp.stdout, cp.stderr
    except subprocess.TimeoutExpired as exc:
        rc, out, err = 124, exc.stdout or b"", exc.stderr or b""
        if isinstance(out, bytes):
            out = out.decode(errors="replace")
        if isinstance(err, bytes):
            err = err.decode(errors="replace")
        err += "\nrunner: timeout after 900 seconds\n"
    elapsed = time.monotonic() - tick
    stdout_path = prefix.with_suffix(".stdout.log")
    stderr_path = prefix.with_suffix(".stderr.log")
    stdout_path.write_text(out)
    stderr_path.write_text(err)
    return {
        "argv": argv,
        "cwd": str(cwd),
        "started_utc": start,
        "finished_utc": datetime.now(timezone.utc).isoformat(),
        "elapsed_seconds": elapsed,
        "exit_code": rc,
        "stdout": str(stdout_path),
        "stdout_sha256": digest(stdout_path),
        "stderr": str(stderr_path),
        "stderr_sha256": digest(stderr_path),
    }


def compiler_provenance(name: str) -> dict[str, str]:
    path = shutil.which(name)
    if path is None:
        raise ValueError(f"required host compiler unavailable: {name}")
    # rustc is commonly a rustup proxy: its argv[0] selects the tool. Resolving
    # the symlink before execution runs `rustup --edition=...` instead of rustc.
    binary = Path(path).absolute()
    version = subprocess.run(
        [str(binary), "--version"], text=True, capture_output=True, check=True
    ).stdout.splitlines()[0]
    return {
        "argv0": name,
        "executable": str(binary),
        "resolved_executable": str(binary.resolve()),
        "executable_sha256": digest(binary),
        "version": version,
    }


def build_decode(row, args, output: Path, prefix: Path):
    source = args.repo / "sdk/lib/rust/jscript-rust/src/decode.rs"
    if (
        digest(source)
        != row["candidate_file_sha256"]["repo:sdk/lib/rust/jscript-rust/src/decode.rs"]
    ):
        raise ValueError("shipping JScript decoder changed before host compile")
    wrapper = prefix.with_suffix(".host.rs")
    wrapper.write_text(f"#[path = {json.dumps(str(source))}]\nmod subject;\n")
    shim = prefix.with_suffix(".shim.o")
    library = prefix.with_suffix(".so")
    cc = compiler_provenance("cc")
    rustc = compiler_provenance("rustc")
    cmds = [
        execute(
            [
                cc["executable"],
                "-std=c11",
                "-Wall",
                "-Wextra",
                "-fPIC",
                "-c",
                str(args.fixtures_root / "c_cohort/decode_host_shims.c"),
                "-o",
                str(shim),
            ],
            args.repo,
            output / (prefix.name + "-shim"),
        )
    ]
    if cmds[-1]["exit_code"] != 0:
        return None, cmds, {"cc": cc, "rustc": rustc, "host_wrapper_sha256": digest(wrapper)}
    cmds.append(
        execute(
            [
                rustc["executable"],
                "--edition=2021",
                "--crate-name",
                "decode_cohort",
                "--crate-type",
                "cdylib",
                "-O",
                str(wrapper),
                "-C",
                f"link-arg={shim}",
                "-C",
                "link-arg=-Wl,-Bsymbolic",
                "-o",
                str(library),
            ],
            args.repo,
            output / (prefix.name + "-rustc"),
        )
    )
    build = {
        "cc": cc,
        "rustc": rustc,
        "host_wrapper_sha256": digest(wrapper),
        "shim_object_sha256": digest(shim),
        "library_sha256": digest(library) if library.is_file() else None,
    }
    return library if cmds[-1]["exit_code"] == 0 and library.is_file() else None, cmds, build


def build_txt2nls(args, output: Path, prefix: Path):
    source = args.staging / "cpp-pilot/txt2nls-faithful.rs"
    binary = prefix.with_suffix(".exe")
    rustc = compiler_provenance("rustc")
    command = execute(
        [
            rustc["executable"],
            "--edition=2021",
            "--crate-name",
            "txt2nls_cohort",
            "-O",
            str(source),
            "-o",
            str(binary),
        ],
        args.repo,
        output / (prefix.name + "-rustc"),
    )
    build = {
        "rustc": rustc,
        "candidate_source_sha256": digest(source),
        "candidate_binary_sha256": digest(binary) if binary.is_file() else None,
    }
    return binary if command["exit_code"] == 0 and binary.is_file() else None, command, build


def parse_json_stdout(command):
    return json.loads(Path(command["stdout"]).read_text())


def grader_report(command, kind: str, prefix: Path):
    """C++ stdout is a console summary; verdicts and provenance live in --output."""
    if kind in {"txt2nls", "quotes", "cenum"}:
        suffix = ".semantics.json" if kind == "txt2nls" else ".grade.json"
        path = prefix.with_suffix(suffix)
        report = json.loads(path.read_text())
        command["grader_report"] = str(path)
        command["grader_report_sha256"] = digest(path)
        return report
    return parse_json_stdout(command)


def run_row(row, args, output: Path, score):
    rootmap = roots(args)
    kind = row["kind"]
    tag = re.sub(r"[^a-zA-Z0-9_-]", "_", row["case_id"])
    prefix = output / tag
    commands = []
    report = None
    reasons = []
    mismatches = 0
    behavior_cases = None
    test_functions_run = None
    unsupported = None
    repairs = []
    build_provenance = None
    if kind == "cli":
        argv = [str(args.repo / "vibeos-cli"), "difftest", "--only", row["crate"]]
        for symbol in row["symbols"]:
            argv += ["--case", symbol]
        cmd = execute(argv, args.repo, prefix)
        commands.append(cmd)
        text = Path(cmd["stdout"]).read_text()
        match = re.search(r"(?m)^full log:\s+(\S+)$", text)
        summary = None
        if match:
            summary_path = Path(match.group(1)).parent / "summary.json"
            if summary_path.is_file():
                summary = json.loads(summary_path.read_text())
                shutil.copyfile(summary_path, prefix.with_suffix(".cli-summary.json"))
                cmd["cli_summary_sha256"] = digest(summary_path)
                cmd["cli_summary_snapshot"] = str(prefix.with_suffix(".cli-summary.json"))
                full_log = Path(match.group(1))
                if full_log.is_file():
                    shutil.copyfile(full_log, prefix.with_suffix(".cli-full.log"))
                    cmd["cli_full_log_sha256"] = digest(full_log)
                    cmd["cli_full_log_snapshot"] = str(prefix.with_suffix(".cli-full.log"))
        stages = summary.get("stages", []) if summary else []
        drift_stages = [s for s in stages if s["name"].startswith("drift:")]
        preflight = bool(drift_stages) and all(s["status"] == "ok" for s in drift_stages)
        if summary is None or summary.get("worktree") != str(args.repo):
            raise ValueError(f"{row['case_id']}: missing or foreign CLI summary; logs: {prefix}")
        if not preflight:
            raise ValueError(f"{row['case_id']}: generated-oracle preflight failed; logs: {prefix}")
        crate_stages = [s for s in stages if s["name"] == row["crate"]]
        expected = len(row["symbols"])
        test_functions_run = (
            int(crate_stages[0]["detail"].split()[0])
            if len(crate_stages) == 1 and re.fullmatch(r"\d+ cases", crate_stages[0]["detail"])
            else 0
        )
        passed = (
            cmd["exit_code"] == 0
            and summary is not None
            and summary.get("worktree") == str(args.repo)
            and summary.get("command") == "difftest"
            and summary.get("verdict") == "OK"
            and preflight
            and len(crate_stages) == 1
            and crate_stages[0]["status"] == "ok"
            and test_functions_run == expected
        )
        checks = [score.Check("vibeos-cli preflights and exact differential tests", True, passed)]
        if not passed:
            reasons.append(
                "CLI or its generated-oracle preflight failed; see captured summary and logs"
            )
        # A libtest function can contain multiple comparisons. Do not call it one behavioral case.
        behavior_cases = None
    elif kind == "ccopy":
        report = json.loads(
            resolve(rootmap, "fixtures:cpp_cohort/heldout-status.json").read_text()
        )["copy_as_path"]
        unsupported = report["reason"]
        behavior_cases = 0
        checks = [score.Check("complete banked candidate and faithful host substrate", True, False)]
    else:
        pyth = sys.executable
        if kind == "decode":
            library, build_commands, build_provenance = build_decode(row, args, output, prefix)
            commands.extend(build_commands)
            if build_commands[0]["exit_code"] != 0:
                raise ValueError(f"{row['case_id']}: host shim build failed; logs: {prefix}")
            argv = (
                [
                    pyth,
                    str(resolve(rootmap, "fixtures:c_cohort/grade_decode.py")),
                    "--repo",
                    str(args.repo),
                    "--mutation-check",
                    str(library),
                ]
                if library
                else None
            )
        elif kind == "rc4":
            argv = [
                pyth,
                str(resolve(rootmap, "fixtures:c_cohort/grade_rc4.py")),
                "--repo",
                str(args.repo),
                "--mutation-check",
            ]
        elif kind == "txt2nls":
            argv = [
                pyth,
                str(resolve(rootmap, "fixtures:cpp_cohort/grade_semantics.py")),
                "--split",
                "dev",
                "--output",
                str(prefix.with_suffix(".semantics.json")),
            ]
        elif kind == "quotes":
            argv = [
                pyth,
                str(resolve(rootmap, "fixtures:cpp_cohort/grade_quotes.py")),
                "--repo",
                str(args.repo),
                "--bank",
                str(args.shell32_bank),
                "--output",
                str(prefix.with_suffix(".grade.json")),
            ]
        elif kind == "cenum":
            argv = [
                pyth,
                str(resolve(rootmap, "fixtures:cpp_cohort/grade_cenum.py")),
                "--repo",
                str(args.repo),
                "--bank",
                str(args.shell32_bank),
                "--output",
                str(prefix.with_suffix(".grade.json")),
            ]
        else:
            raise ValueError(kind)
        cmd = (
            execute(argv, args.repo, prefix if kind != "txt2nls" else output / (tag + "-semantics"))
            if argv
            else None
        )
        if cmd:
            commands.append(cmd)
        if cmd:
            try:
                report = grader_report(cmd, kind, prefix)
            except (ValueError, OSError) as exc:
                raise ValueError(
                    f"{row['case_id']}: grader emitted no JSON; logs: {prefix}"
                ) from exc
        else:
            report = {}
        if kind in ("decode", "rc4"):
            if kind == "decode" and report.get("reference_safety", {}).get("passed") is not True:
                raise ValueError(
                    f"{row['case_id']}: original C reference safety failed; logs: {prefix}"
                )
            part = report.get("grade", {})
            behavior_cases = part.get("cases")
            mismatches = int(part.get("mismatches", 0)) + int(part.get("guard_failures", 0))
            mutations = report.get("mutations", [])
            mutation_ok = len(mutations) >= 2 and all(
                item.get("detected") is True for item in mutations
            )
            if not mutation_ok:
                raise ValueError(
                    f"{row['case_id']}: grader failed mutation controls; logs: {prefix}"
                )
            passed = (
                cmd is not None
                and cmd["exit_code"] == 0
                and part.get("passed") is True
                and isinstance(behavior_cases, int)
                and behavior_cases > 0
                and mutation_ok
            )
            checks = [
                score.Check(
                    "original C versus Rust, full buffer and guards, mutation sensitivity",
                    True,
                    passed,
                )
            ]
            if kind == "decode":
                repairs.append(
                    score.Repair(
                        score.RepairKind.AUTOMATIC,
                        "compiler anchor plus deterministic width and type adaptation; "
                        "decode-generation/decode.json",
                    )
                )
        elif kind == "txt2nls":
            semantic_cases = report.get("cases_compared")
            semantic_ok = (
                cmd is not None
                and cmd["exit_code"] == 0
                and report.get("mismatch_count") == 0
                and isinstance(semantic_cases, int)
                and semantic_cases > 0
            )
            candidate, compile_command, build_provenance = build_txt2nls(args, output, prefix)
            commands.append(compile_command)
            argv = (
                [
                    pyth,
                    "-m",
                    "rustconv.pilot_txt2nls",
                    "--repo",
                    str(args.repo),
                    "--candidate",
                    str(candidate),
                    "--all-codepages",
                    "--mutations",
                    "--manifest",
                    str(prefix.with_suffix(".reference-manifest.json")),
                ]
                if candidate
                else None
            )
            cmd2 = execute(argv, args.repo / "tools/rustconv", prefix) if argv else None
            if cmd2:
                commands.append(cmd2)
            if cmd2:
                try:
                    report2 = parse_json_stdout(cmd2)
                except (ValueError, OSError) as exc:
                    raise ValueError(
                        f"{row['case_id']}: txt2nls oracle emitted no JSON; logs: {prefix}"
                    ) from exc
            else:
                report2 = {}
            if report2.get("reference_unsupported"):
                raise ValueError(
                    f"{row['case_id']}: original C++ reference unsupported; logs: {prefix}"
                )
            verdicts = report2.get("candidate_verdicts", {})
            candidate_cases = len(verdicts)
            candidate_ok = (
                cmd2 is not None
                and cmd2["exit_code"] == 0
                and candidate_cases > 0
                and all(v.get("passed") is True for v in verdicts.values())
            )
            mutations = report2.get("mutations_detected_by", {})
            if len(mutations) != 5 or not all(bool(hits) for hits in mutations.values()):
                raise ValueError(
                    f"{row['case_id']}: grader failed mutation controls; logs: {prefix}"
                )
            behavior_cases = {"stdlib_semantics": semantic_cases, "whole_tool": candidate_cases}
            mismatches = int(report.get("mismatch_count", 0)) + sum(
                v.get("passed") is not True for v in verdicts.values()
            )
            checks = [
                score.Check("C++ stdlib semantics", True, semantic_ok),
                score.Check("genuine C++ tool versus repaired Rust executable", True, candidate_ok),
            ]
            repairs.append(
                score.Repair(
                    score.RepairKind.MANUAL,
                    "banked candidate repaired to faithful executable; "
                    "cpp-pilot/repair-manifest.json",
                )
            )
            report = {"semantics": report, "whole_tool": report2}
        else:
            behavior_cases = report.get("cases_compared")
            mismatches = int(report.get("mismatch_count", 0))
            partial_ok = (
                cmd is not None
                and cmd["exit_code"] == 0
                and mismatches == 0
                and isinstance(behavior_cases, int)
                and behavior_cases > 0
            )
            checks = [score.Check("genuine C++ versus banked Rust bounded cases", True, partial_ok)]
            if kind == "cenum":
                # Skip/Reset parity cannot accept a TU with an uncompilable Next
                # and two ungraded methods.
                whole_ok = (
                    report.get("whole_unit_status") == "accepted"
                    and not report.get("candidate_failures")
                    and not report.get("required_unsupported_methods")
                )
                checks.append(
                    score.Check(
                        "all required methods compile and are behaviorally graded", True, whole_ok
                    )
                )
                if not whole_ok:
                    reasons.append(
                        "Next rustc E0308; AppendItemsFromEnumerator and DeleteList ungraded"
                    )
    result = score.CaseResult(
        row["case_id"],
        tuple(checks),
        mismatches=mismatches,
        unsupported_reason=unsupported,
        repairs=tuple(repairs),
        cost_usd=None,
        effort_seconds=None,
    )
    evidence = {
        "case_id": row["case_id"],
        "split": row["split"],
        "coverage_scope": row["coverage_scope"],
        "classification": result.classification.value,
        "behavior_cases": behavior_cases,
        "test_functions_run": test_functions_run,
        "commands": commands,
        "grader_report": report,
        "reasons": reasons,
        "build_provenance": build_provenance,
        "pins": row["pins"],
        "cost_usd": None,
        "effort_seconds": None,
    }
    write_new(prefix.with_suffix(".evidence.json"), evidence)
    return result, evidence


def main():
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("--repo", type=Path, default=Path.cwd())
    parser.add_argument("--staging", type=Path, required=True)
    parser.add_argument("--shell32-bank", type=Path, required=True)
    parser.add_argument(
        "--fixtures-root",
        type=Path,
        help="directory containing runner and grader packages; default is script directory",
    )
    sub = parser.add_subparsers(dest="action", required=True)
    p = sub.add_parser("prepare", help="freeze 12 TUs, artifacts, grader and pipeline hashes")
    p.add_argument("--manifest", type=Path, required=True)
    p = sub.add_parser("verify", help="verify every pinned byte without running a grader")
    p.add_argument("--manifest", type=Path, required=True)
    p = sub.add_parser("run", help="run development first, then held-out, and score both")
    p.add_argument("--manifest", type=Path, required=True)
    p.add_argument(
        "--output", type=Path, required=True, help="new artifact directory; never overwritten"
    )
    args = parser.parse_args()
    args.repo = args.repo.resolve()
    args.staging = args.staging.resolve()
    args.shell32_bank = args.shell32_bank.resolve()
    args.fixtures_root = (args.fixtures_root or Path(__file__).resolve().parent).resolve()
    score = import_scorecard(args.repo)
    if args.action == "prepare":
        manifest = materialize(args)
        write_new(args.manifest, manifest)
        print(
            json.dumps(
                {
                    "manifest": str(args.manifest),
                    "cohort_sha256": manifest["cohort_manifest_sha256"],
                    "pipeline_sha256": manifest["pipeline_sha256"],
                    "sources": len(manifest["cases"]),
                },
                indent=2,
            )
        )
        return 0
    frozen = json.loads(args.manifest.read_text())
    verify_manifest(args, frozen)
    if args.action == "verify":
        print(
            json.dumps(
                {
                    "verified": True,
                    "sources": len(frozen["cases"]),
                    "pipeline_sha256": frozen["pipeline_sha256"],
                },
                indent=2,
            )
        )
        return 0
    args.output.mkdir(parents=True, exist_ok=False)
    split_pins = {s: {} for s in ("development", "held_out")}
    for row in frozen["cases"]:
        split_pins[row["split"]][row["case_id"]] = score.ArtifactPins(**row["pins"])
    cohort = score.CohortManifest(**split_pins)
    if cohort.sha256 != frozen["cohort_manifest_sha256"]:
        raise ValueError("cohort scorecard digest mismatch")
    reports = {}
    for split in ("development", "held_out"):
        # Check every pinned input and pipeline mechanism before held-out work.
        verify_manifest(args, frozen)
        results, evidence = [], []
        for row in frozen["cases"]:
            if row["split"] != split:
                continue
            result, item = run_row(row, args, args.output, score)
            results.append(result)
            evidence.append(item)
        actual = {
            row["case_id"]: score.ArtifactPins(**row["pins"])
            for row in frozen["cases"]
            if row["split"] == split
        }
        report = score.score_run(
            cohort,
            score.Split(split),
            frozen["pipeline_sha256"],
            results,
            actual,
            development_report=reports.get("development"),
        )
        verify_manifest(args, frozen)
        reports[split] = report
        write_new(args.output / f"{split}-scorecard.json", report.as_dict())
        write_new(
            args.output / f"{split}-case-evidence-index.json",
            [
                {
                    k: e[k]
                    for k in (
                        "case_id",
                        "classification",
                        "behavior_cases",
                        "test_functions_run",
                        "pins",
                    )
                }
                for e in evidence
            ],
        )
        scorecard_path = args.output / f"{split}-scorecard.json"
        print(f"{split}: valid={report.valid} passed={report.passed}; scorecard={scorecard_path}")
    write_new(
        args.output / "run-summary.json",
        {
            "manifest": str(args.manifest),
            "manifest_sha256": digest(args.manifest),
            "pipeline_sha256": frozen["pipeline_sha256"],
            "development": reports["development"].as_dict(),
            "held_out": reports["held_out"].as_dict(),
            "host_compilers": {name: compiler_provenance(name) for name in ("cc", "g++", "rustc")},
            "cost_usd": None,
            "effort_seconds": None,
            "no_api_calls": True,
        },
    )
    return 0 if all(r.passed for r in reports.values()) else 1


if __name__ == "__main__":
    try:
        raise SystemExit(main())
    except (ValueError, OSError, KeyError) as exc:
        print(f"cohort runner: {exc}", file=sys.stderr)
        raise SystemExit(2) from exc
