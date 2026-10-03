#!/usr/bin/env python3
"""Audit simple source-owned C++ numeric macros against compiler and Rust output.

This produces evidence and explicit unsupported results. It never inserts a
Rust declaration or infers a type from an integer spelling.
"""

from __future__ import annotations

import argparse
import hashlib
import json
import re
import subprocess
from pathlib import Path
from typing import TypedDict


class MacroAudit(TypedDict):
    source: str
    source_sha256: str
    candidate: str | None
    candidate_sha256: str | None
    compiler: str
    macros: list[dict[str, object]]


NAME = re.compile(r"^\s*#\s*define\s+([A-Za-z_]\w*)(.*)$")
LITERAL = re.compile(r"^([+-]?)(0[xX][0-9a-fA-F']+|0[bB][01']+|0[0-7']*|[1-9][0-9']*)([uUlLzZ]*)$")


def numeric_literal(body: str) -> int | None:
    matched = LITERAL.fullmatch(body.strip())
    if not matched:
        return None
    sign, digits, _suffix = matched.groups()
    digits = digits.replace("'", "")
    if digits.startswith(("0x", "0X")):
        value = int(digits[2:], 16)
    elif digits.startswith(("0b", "0B")):
        value = int(digits[2:], 2)
    elif digits.startswith("0") and len(digits) > 1:
        value = int(digits, 8)
    else:
        value = int(digits, 10)
    return -value if sign == "-" else value


def source_definitions(source: str) -> dict[str, list[dict[str, object]]]:
    definitions: dict[str, list[dict[str, object]]] = {}
    for line_number, line in enumerate(source.splitlines(), 1):
        matched = NAME.match(line)
        if not matched:
            continue
        name, tail = matched.groups()
        definitions.setdefault(name, []).append(
            {
                "line": line_number,
                "body": tail.strip(),
                "function_like": tail.startswith("("),
                "multiline": line.rstrip().endswith("\\"),
            }
        )
    return definitions


def compiler_definitions(source: Path, compiler: str) -> dict[str, str]:
    result = subprocess.run(
        [compiler, "-std=c++17", "-dM", "-E", str(source)],
        check=True,
        text=True,
        capture_output=True,
    )
    definitions: dict[str, str] = {}
    for line in result.stdout.splitlines():
        matched = NAME.match(line)
        if matched:
            name, tail = matched.groups()
            definitions[name] = tail.strip()
    return definitions


def audit(source: Path, candidate: Path | None = None, compiler: str = "g++") -> MacroAudit:
    raw = source.read_text(encoding="utf-8")
    own = source_definitions(raw)
    active = compiler_definitions(source, compiler)
    rust = candidate.read_text(encoding="utf-8") if candidate else None
    rows: list[dict[str, object]] = []
    for name, definitions in sorted(own.items()):
        entry = {"name": name, "source_lines": [d["line"] for d in definitions]}
        if len(definitions) != 1:
            entry.update(status="unsupported", reason="multiple source definitions")
        else:
            definition = definitions[0]
            body = str(definition["body"])
            if name not in active:
                entry.update(status="inactive", reason="absent from compiler -dM output")
            elif definition["function_like"]:
                entry.update(status="unsupported", reason="function-like macro")
            elif definition["multiline"]:
                entry.update(status="unsupported", reason="multiline macro")
            elif active[name] != body:
                entry.update(
                    status="unsupported", reason="compiler expansion differs from source spelling"
                )
            elif (value := numeric_literal(body)) is None:
                entry.update(status="unsupported", reason="nonliteral body")
            else:
                entry.update(status="supported", literal=body, integer_value=value)
                if rust is not None:
                    declaration = re.search(
                        rf"(?m)^\s*(?:pub\s+)?const\s+{re.escape(name)}\s*:\s*[^=;]+\s*=\s*([^;]+);",
                        rust,
                    )
                    if declaration is None:
                        entry["candidate"] = "missing_declaration"
                    else:
                        candidate_value = numeric_literal(declaration.group(1).strip())
                        entry["candidate"] = (
                            "same_value"
                            if candidate_value == value
                            else "unsupported_initializer"
                            if candidate_value is None
                            else "different_value"
                        )
        rows.append(entry)
    return {
        "source": str(source),
        "source_sha256": hashlib.sha256(source.read_bytes()).hexdigest(),
        "candidate": str(candidate) if candidate else None,
        "candidate_sha256": hashlib.sha256(candidate.read_bytes()).hexdigest()
        if candidate
        else None,
        "compiler": compiler,
        "macros": rows,
    }


def main() -> None:
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("source", type=Path)
    parser.add_argument("--candidate", type=Path)
    parser.add_argument("--compiler", default="g++")
    parser.add_argument("--output", type=Path)
    args = parser.parse_args()
    report = audit(args.source, args.candidate, args.compiler)
    serialized = json.dumps(report, indent=2) + "\n"
    if args.output:
        args.output.write_text(serialized, encoding="utf-8")
    print(serialized, end="")


if __name__ == "__main__":
    main()
