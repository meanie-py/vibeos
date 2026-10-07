#!/usr/bin/env python3
"""scan_ptr_typedef_indirection — a Rust local or parameter one pointer level deeper than its C.

The C declares `PFILEPATHS_W FilePaths` (a pointer TYPEDEF); the Rust declares
`FilePaths: *mut PFILEPATHS_W` -- a pointer to a pointer -- and then reads `(*(*x)).Target`,
the struct's first field taken as the struct's address. It compiles, links and boots.
usetup's FileCopyCallback and four of its pages shipped exactly this (every keypress and
every copy-progress message read garbage), and only the C's own declaration says it is
wrong. `rustconv`'s purchase audit (`c_body_checks.pointer_typedef_indirection_findings`)
now rejects it in a new body; this scan holds every body ALREADY in the tree to the same
check, so the class stays at zero.

Pairing is body_audit's: every module that links a Rust crate, each C function its
flipped-out TUs define, against the same-named Rust definitions in that module's crates.
A C `PFOO *x` is genuinely two levels and is never reported. Suppress a reviewed false
positive on the Rust declaration's line with `nolint: ptr-typedef-indirection` and say why.

Gated at ZERO (no baseline): the sweep that introduced it found two, both fixed.
"""

from __future__ import annotations

import re
import sys
from pathlib import Path

REPO = Path(__file__).resolve().parent.parent
sys.path.insert(0, str(REPO))
sys.path.insert(0, str(REPO / "tools" / "rustconv"))

from rustconv import body_audit as ba  # noqa: E402
from rustconv.c_body_checks import pointer_typedef_indirection_findings  # noqa: E402
from rustconv.c_defs import c_functions  # noqa: E402

NOLINT_RE = re.compile(r"nolint:\s*ptr-typedef-indirection\b")


def c_definitions(text: str) -> dict[str, str]:
    """Each function this C text defines -> its signature and body (the parameters count)."""
    out: dict[str, str] = {}
    for name in c_functions(text):
        for m in re.finditer(rf"\b{re.escape(name)}\s*\(", text):
            close = ba._matching(text, m.end() - 1, "(", ")")
            if close is None:
                continue
            brace = close + 1
            while brace < len(text) and text[brace] in " \t\r\n":
                brace += 1
            if brace >= len(text) or text[brace] != "{":
                continue
            end = ba._matching(text, brace, "{", "}")
            if end is not None:
                out[name] = text[m.start() : end + 1]
                break
    return out


def findings_for(
    c_body: str, rust_text: str, rust_body: str, body_at: int
) -> list[tuple[int, str]]:
    """(1-based line in `rust_text`, message) for each over-deep declaration in `rust_body`,
    which starts at offset `body_at` of `rust_text`; suppressed lines are dropped."""
    out = []
    lines = rust_text.splitlines()
    for msg in pointer_typedef_indirection_findings(c_body, rust_body):
        decl = re.match(r"`(\w+): \*(mut|const) (\w+)`", msg)
        line = rust_text.count("\n", 0, body_at) + 1
        if decl:
            name, kind, ty = decl.groups()
            m = re.search(rf"\b{name}\s*:\s*\*{kind}\s+{ty}\b", rust_body)
            if m:
                line += rust_body.count("\n", 0, m.start())
        if NOLINT_RE.search(lines[line - 1] if line - 1 < len(lines) else ""):
            continue
        out.append((line, msg))
    return out


def scan(repo: Path) -> list[str]:
    out: set[str] = set()
    for module in ba.all_modules(repo):
        c_files, srcs, _glue = ba.module_inputs(module)
        if not c_files:
            continue
        rust: dict[str, list[tuple[Path, str, str, int]]] = {}
        for f in srcs:
            if "bindgen" in f.name or f.name.startswith("types_"):
                continue
            text = f.read_text(encoding="utf-8", errors="replace")
            for name, body in ba.rust_definitions_all(text):
                rust.setdefault(name, []).append((f, text, body, max(text.find(body), 0)))
        for cf in c_files:
            for name, c_body in c_definitions(
                cf.read_text(encoding="utf-8", errors="replace")
            ).items():
                for rf, text, body, at in rust.get(name, []):
                    for line, msg in findings_for(c_body, text, body, at):
                        out.add(
                            f"{rf.relative_to(repo)}:{line}: {name}: {msg} "
                            f"(C: {cf.relative_to(repo)})"
                        )
    return sorted(out)


def main() -> int:
    findings = scan(REPO)
    for f in findings:
        print(f)
    print(f"{len(findings)} declaration(s) one pointer level deeper than the C", file=sys.stderr)
    return 1 if findings else 0


if __name__ == "__main__":
    sys.exit(main())
