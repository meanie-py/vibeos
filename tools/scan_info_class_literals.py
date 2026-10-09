#!/usr/bin/env python3
"""
scan_info_class_literals.py — a bare integer where a FILE_INFORMATION_CLASS belongs.

    C:    IoSetInformation(FileObject, FileEndOfFileInformation, sizeof(...), &MaximumSize);
    Rust: IoSetInformation(FileObject, 2u32, ...)          // 2 is FileFullDirectoryInformation

Mm's file-backed section creation extends a file through that call. With the wrong class
every extend failed, so a section larger than its file came back SECTION_NOT_EXTENDED and
setup's cabinet extraction lost 964 of its files -- found by the install smoke. ps-rust's
apphelp passed 0 (no class at all) for FileBasicInformation, the same way. The literal
compiles, the call returns an ordinary error, and nothing but the C says it was meant to
be something else.

So the class argument of the file-information calls is a NAMED constant, always: a name
cannot silently be the wrong number. Gated at zero.

Usage:
    python3 tools/scan_info_class_literals.py [paths...]   # default: sdk/lib/rust
    python3 tools/scan_info_class_literals.py --lint
"""

from __future__ import annotations

import re
import sys
from pathlib import Path

sys.path.insert(0, str(Path(__file__).resolve().parent.parent))
from tools.scan_wide_byte_copy import SKIP_FILE_RE
from tools.source_walk import source_files

REPO = Path(__file__).resolve().parent.parent
RUST_ROOT = REPO / "sdk" / "lib" / "rust"
COMMENT_RE = re.compile(r"//[^\n]*|/\*.*?\*/", re.S)
# callee -> 0-based index of its information-class argument
CLASS_ARG = {
    "NtQueryInformationFile": 4,
    "ZwQueryInformationFile": 4,
    "NtSetInformationFile": 4,
    "ZwSetInformationFile": 4,
    "IoQueryFileInformation": 1,
    "IoSetInformation": 1,
}
CALL_RE = re.compile(r"(?<![\w:])(?:[A-Za-z_]\w*::)*(" + "|".join(CLASS_ARG) + r")\s*\(")
LITERAL_RE = re.compile(
    r"^\(?\s*-?(?:0x[0-9a-fA-F_]+|\d[\d_]*)(?:[iu](?:8|16|32|64|size))?\s*\)?"
    r"(?:\s+as\s+[A-Za-z_][\w:]*)?$"
)


def _code(text: str) -> str:
    return COMMENT_RE.sub(lambda m: re.sub(r"[^\n]", " ", m.group(0)), text)


def _args(code: str, start: int) -> list[str] | None:
    """Top-level arguments of the call whose `(` is at `start`."""
    depth, cur, out = 0, [], []
    for ch in code[start:]:
        if ch in "([{":
            depth += 1
            if depth == 1:
                continue
        elif ch in ")]}":
            depth -= 1
            if depth == 0:
                out.append("".join(cur).strip())
                return out
        elif ch == "," and depth == 1:
            out.append("".join(cur).strip())
            cur = []
            continue
        cur.append(ch)
    return None


def scan(paths: list[Path]) -> list[tuple[Path, int, str]]:
    findings = []
    for root in paths:
        files = [root] if root.is_file() else list(source_files(root))
        for f in files:
            if f.suffix != ".rs" or SKIP_FILE_RE.search(str(f)):
                continue
            try:
                text = f.read_text(errors="replace")
            except OSError:
                continue
            code = _code(text)
            for m in CALL_RE.finditer(code):
                # A declaration (`fn NtSetInformationFile(`) names parameters, not a class.
                if re.search(r"\bfn\s+$", code[max(0, m.start() - 8) : m.start()]):
                    continue
                args = _args(code, m.end() - 1)
                idx = CLASS_ARG[m.group(1)]
                if args is None or len(args) <= idx:
                    continue
                if LITERAL_RE.match(args[idx]):
                    line = text.count("\n", 0, m.start()) + 1
                    findings.append((f, line, f"{m.group(1)} class `{args[idx]}`"))
    return sorted(findings)


def main(argv: list[str] | None = None) -> int:
    args = list(sys.argv[1:] if argv is None else argv)
    lint = "--lint" in args
    paths = [Path(a).resolve() for a in args if not a.startswith("--")] or [RUST_ROOT]
    findings = scan(paths)
    for f, lineno, what in findings:
        try:
            rel = f.relative_to(REPO)
        except ValueError:
            rel = f
        tag = "info-class-literal" if lint else "bare information class"
        print(f"{rel}:{lineno}: {tag} {what}")
    if findings and not lint:
        print(f"\n{len(findings)} finding(s): name the class (_FILE_INFORMATION_CLASS_File...).")
    return 1 if findings else 0


if __name__ == "__main__":
    sys.exit(main())
