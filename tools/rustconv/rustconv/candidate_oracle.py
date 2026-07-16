"""Batch-compile candidate Rust workers against the real shell32 substrate."""

from __future__ import annotations

import json
import os
import re
import subprocess
from dataclasses import dataclass
from datetime import datetime
from pathlib import Path

from .config import REACTOS_ROOT, RUST_TARGETS, RUSTCONV_RESPONSES_DIR


class CandidateOracleError(RuntimeError):
    pass


@dataclass(frozen=True)
class CandidateSpec:
    name: str
    function: str
    candidate: Path
    base: Path
    base_function: str | None = None


def masked_rust(text: str) -> str:
    out = list(text)
    i = 0
    while i < len(text):
        if text.startswith("//", i):
            end = text.find("\n", i + 2)
            end = len(text) if end < 0 else end
            for pos in range(i, end):
                out[pos] = " "
            i = end
        elif text.startswith("/*", i):
            end = text.find("*/", i + 2)
            end = len(text) if end < 0 else end + 2
            for pos in range(i, end):
                if out[pos] != "\n":
                    out[pos] = " "
            i = end
        elif text[i] in ('"', "'"):
            quote = text[i]
            i += 1
            while i < len(text):
                if text[i] == "\\":
                    out[i] = " "
                    if i + 1 < len(text):
                        out[i + 1] = " "
                    i += 2
                    continue
                if text[i] == quote:
                    i += 1
                    break
                if out[i] != "\n":
                    out[i] = " "
                i += 1
        else:
            i += 1
    return "".join(out)


def function_span(rust: str, name: str) -> tuple[int, int]:
    masked = masked_rust(rust)
    match = re.search(
        rf"(?m)(?:^[ \t]*|(?<=[;}}])[ \t]*)"
        rf"(?:(?:pub(?:\([^)]*\))?|unsafe|const|extern\s+\"[^\"]+\")\s+)*"
        rf"fn\s+{re.escape(name)}\s*\(",
        masked,
    )
    if match is None:
        raise CandidateOracleError(f"function not found: {name}")
    opening = masked.find("{", match.end())
    if opening < 0:
        raise CandidateOracleError(f"function has no body: {name}")
    depth = 0
    for pos in range(opening, len(masked)):
        if masked[pos] == "{":
            depth += 1
        elif masked[pos] == "}":
            depth -= 1
            if depth == 0:
                return match.start(), pos + 1
    raise CandidateOracleError(f"unbalanced function body: {name}")


def replace_function(base: str, candidate: str, name: str) -> str:
    base_start, base_end = function_span(base, name)
    candidate_start, candidate_end = function_span(candidate, name)
    return base[:base_start] + candidate[candidate_start:candidate_end] + base[base_end:]


_WIDE_STATIC_RE = re.compile(
    r"(?m)^pub static (CPP_COM_WSTR_[0-9A-F]{12}): \[WCHAR; \d+\] = \[[^\n]+;\n?"
)


def carry_deterministic_wide_statics(base: str, candidate: str) -> str:
    """Carry only rustconv-generated UTF-16 data needed by a worker."""
    additions = []
    for match in _WIDE_STATIC_RE.finditer(candidate):
        if not re.search(rf"\b{re.escape(match.group(1))}\s*:", base):
            additions.append(match.group(0).rstrip())
    return ("\n".join(additions) + "\n\n" + base) if additions else base


def load_manifest(path: Path) -> tuple[list[CandidateSpec], Path, Path]:
    data = json.loads(path.read_text())
    if data.get("target", "shell32") != "shell32":
        raise CandidateOracleError("candidate oracle currently supports target=shell32 only")
    root = path.parent

    def resolve(raw: str) -> Path:
        value = Path(raw)
        if value.is_absolute():
            return value
        local = (root / value).resolve()
        return local if local.exists() else (REACTOS_ROOT / value).resolve()

    items = data.get("candidates")
    if items is None:
        items = [
            {
                "name": item["id"],
                "function": item.get("candidate_function", f"{item['function']}_body"),
                "candidate": item.get("candidate", f"candidates/{item['id']}.rs"),
                "base": item["base"],
                "base_function": item.get("base_function"),
            }
            for item in data.get("cases", [])
        ]
    candidates = [
        CandidateSpec(
            item["name"],
            item["function"],
            resolve(item["candidate"]),
            resolve(item["base"]),
            item.get("base_function"),
        )
        for item in items
    ]
    if not candidates:
        raise CandidateOracleError("manifest has no candidates/cases")
    substrate = REACTOS_ROOT / "sdk/lib/rust/shell32-rust/src"
    types_path = resolve(data.get("types", str(substrate / "types.rs")))
    shellcom_path = resolve(data.get("shellcom", str(substrate / "shellcom.rs")))
    for candidate in candidates:
        if not candidate.candidate.is_file() or not candidate.base.is_file():
            raise CandidateOracleError(f"candidate/base file missing for {candidate.name}")
    if not types_path.is_file() or not shellcom_path.is_file():
        raise CandidateOracleError("types/shellcom substrate file missing")
    return candidates, types_path, shellcom_path


def _slug(text: str) -> str:
    return re.sub(r"[^A-Za-z0-9_]+", "_", text).strip("_") or "candidate"


def build_oracle_crate(
    candidates: list[CandidateSpec],
    types_path: Path,
    shellcom_path: Path,
    output: Path,
    repair: bool = False,
) -> tuple[dict[Path, str], dict[str, list[str]]]:
    src = output / "src"
    src.mkdir(parents=True, exist_ok=True)
    (output / "Cargo.toml").write_text(
        '[package]\nname = "rustconv-candidate-oracle"\nversion = "0.0.0"\n'
        'edition = "2021"\n\n[lib]\ncrate-type = ["rlib"]\n\n[workspace]\n'
    )
    module_paths: dict[Path, str] = {}
    repairs: dict[str, list[str]] = {}
    module_lines = []
    for index, candidate in enumerate(candidates):
        module_name = f"candidate_{index:04d}_{_slug(candidate.name)}"
        module_path = (src / f"{module_name}.rs").resolve()
        base_text = candidate.base.read_text()
        candidate_text = candidate.candidate.read_text()
        candidate_function = candidate.function
        if candidate.base_function and candidate.base_function != candidate.function:
            start, end = function_span(candidate_text, candidate.function)
            function_text = candidate_text[start:end]
            function_text, replacements = re.subn(
                rf"(\bfn\s+){re.escape(candidate.function)}\b",
                rf"\g<1>{candidate.base_function}",
                function_text,
                count=1,
            )
            if replacements != 1:
                raise CandidateOracleError(
                    f"cannot map {candidate.function} to base function {candidate.base_function}"
                )
            candidate_text = candidate_text[:start] + function_text + candidate_text[end:]
            candidate_function = candidate.base_function
        if repair:
            from .repair_classifier import repair_worker

            repaired = repair_worker(base_text, candidate_text, candidate_function)
            candidate_text = repaired.code
            repairs[candidate.name] = list(repaired.applied)
        merged = replace_function(base_text, candidate_text, candidate_function)
        merged = carry_deterministic_wide_statics(merged, candidate_text)
        module_path.write_text(merged)
        module_paths[module_path] = candidate.name
        module_lines.append(f"#[path = {json.dumps(str(module_path))}] mod {module_name};")

    substrate_src = types_path.parent
    wine_path = substrate_src / "wine/mod.rs"
    appbar_path = substrate_src / "appbar.rs"
    vista_path = substrate_src / "vista.rs"
    extra_modules = ""
    if wine_path.is_file() and appbar_path.is_file() and vista_path.is_file():
        extra_modules = f"""
#[path = {json.dumps(str(wine_path.resolve()))}]
pub mod wine;
#[path = {json.dumps(str(appbar_path.resolve()))}]
pub mod appbar;
#[path = {json.dumps(str(vista_path.resolve()))}]
pub mod vista;
"""

    root = f"""#![no_std]
#![allow(non_snake_case, non_camel_case_types, non_upper_case_globals)]
#![allow(dead_code, unused_imports, unused_variables, unused_parens, unused_mut)]
#![allow(unused_assignments, unused_macros, static_mut_refs, private_interfaces)]

macro_rules! TRACE {{ ($($arg:tt)*) => {{{{}}}}; }}
macro_rules! WARN {{ ($($arg:tt)*) => {{{{}}}}; }}
macro_rules! ERR {{ ($($arg:tt)*) => {{{{}}}}; }}
macro_rules! FIXME {{ ($($arg:tt)*) => {{{{}}}}; }}
macro_rules! MESSAGE {{ ($($arg:tt)*) => {{{{}}}}; }}

#[path = {json.dumps(str(types_path.resolve()))}]
pub mod types;
#[path = {json.dumps(str(shellcom_path.resolve()))}]
pub mod shellcom;
{extra_modules}

{os.linesep.join(module_lines)}
"""
    (src / "lib.rs").write_text(root)
    return module_paths, repairs


def _diagnostic(message: dict) -> dict:
    diagnostic = {
        "level": message.get("level"),
        "code": (message.get("code") or {}).get("code"),
        "message": message.get("message"),
        "rendered": message.get("rendered"),
        "spans": [
            {
                "file": span.get("file_name"),
                "line": span.get("line_start"),
                "column": span.get("column_start"),
                "primary": span.get("is_primary", False),
            }
            for span in message.get("spans", [])
        ],
    }
    from .repair_classifier import classify_diagnostic

    diagnostic["category"] = classify_diagnostic(diagnostic)
    return diagnostic


def run_oracle(manifest: Path, arches: list[str], repair: bool = False) -> dict:
    candidates, types_path, shellcom_path = load_manifest(manifest)
    timestamp = datetime.now().strftime("%Y%m%d-%H%M%S-%f")
    output = RUSTCONV_RESPONSES_DIR / "candidate-oracles" / timestamp
    module_paths, repairs = build_oracle_crate(
        candidates, types_path, shellcom_path, output, repair=repair
    )
    by_candidate: dict[str, dict[str, list[dict]]] = {
        candidate.name: {arch: [] for arch in arches} for candidate in candidates
    }
    global_errors: dict[str, list[dict]] = {arch: [] for arch in arches}
    commands = []

    for arch in arches:
        target = RUST_TARGETS.get(arch, arch)
        command = [
            "cargo",
            "check",
            "--manifest-path",
            str(output / "Cargo.toml"),
            "--target",
            target,
            "--message-format=json",
        ]
        process = subprocess.run(command, capture_output=True, text=True)
        commands.append({"arch": arch, "command": command, "exit": process.returncode})
        for line in process.stdout.splitlines():
            try:
                event = json.loads(line)
            except json.JSONDecodeError:
                continue
            if event.get("reason") != "compiler-message":
                continue
            message = event.get("message", {})
            if message.get("level") != "error":
                continue
            diagnostic = _diagnostic(message)
            owners = set()
            for span in message.get("spans", []):
                raw_path = Path(span.get("file_name", ""))
                if not raw_path.is_absolute():
                    raw_path = (output / raw_path).resolve()
                owner = module_paths.get(raw_path.resolve())
                if owner is not None:
                    owners.add(owner)
            if owners:
                for owner in owners:
                    by_candidate[owner][arch].append(diagnostic)
            else:
                global_errors[arch].append(diagnostic)
        (output / f"cargo-{arch}.stdout.jsonl").write_text(process.stdout)
        (output / f"cargo-{arch}.stderr.txt").write_text(process.stderr)

    results = []
    for candidate in candidates:
        per_arch = {}
        for arch in arches:
            errors = by_candidate[candidate.name][arch]
            status = "blocked" if global_errors[arch] else "failed" if errors else "ok"
            per_arch[arch] = {"status": status, "diagnostics": errors}
        results.append(
            {
                "name": candidate.name,
                "function": candidate.function,
                "base_function": candidate.base_function or candidate.function,
                "candidate": str(candidate.candidate),
                "base": str(candidate.base),
                "status": "ok"
                if all(item["status"] == "ok" for item in per_arch.values())
                else "failed",
                "arches": per_arch,
            }
        )

    report = {
        "schema": 1,
        "manifest": str(manifest),
        "output": str(output),
        "commands": commands,
        "repair_enabled": repair,
        "repairs": repairs,
        "global_errors": global_errors,
        "results": results,
    }
    (output / "report.json").write_text(json.dumps(report, indent=2, sort_keys=True))
    return report


def print_report(report: dict, as_json: bool = False) -> None:
    if as_json:
        print(json.dumps(report, indent=2, sort_keys=True))
        return
    ok = sum(item["status"] == "ok" for item in report["results"])
    print(f"candidate-check: {ok}/{len(report['results'])} candidates compile on every arch")
    for item in report["results"]:
        states = ", ".join(f"{arch}={result['status']}" for arch, result in item["arches"].items())
        print(f"  {item['name']}: {states}")
        for arch, result in item["arches"].items():
            for diagnostic in result["diagnostics"][:3]:
                print(
                    f"    {arch}: {diagnostic.get('code') or 'rustc'}: {diagnostic.get('message')}"
                )
        applied = report.get("repairs", {}).get(item["name"], [])
        if applied:
            print(f"    repairs: {', '.join(applied)}")
    print(f"  durable oracle: {report['output']}")
