#!/usr/bin/env python3
"""Flag a list-walk cursor that is read after a walk that can run off the end.

wine/list.h's `LIST_FOR_EACH_ENTRY(cursor, head, ...) if (...) break;` leaves `cursor`
at the container of the HEAD when nothing broke, so a following
`list_add_before(&cursor->entry, ...)` appends at the tail. A translation that walks the
raw entries and re-derives `cursor = container_of(entry)` inside the loop leaves it at
the LAST ELEMENT instead: the insert lands one place early, out of order. rtl-threadpool's
timer thread did this when re-arming a periodic timer (2026-10-17), so the timer it
displaced fired late.

Shape matched, per function:
    while <e> != <head> {           # or `loop` with the same break
        <cursor> = ...offset_of!/CONTAINING_RECORD!(... <e> ...)...;
        if ... { break; }
        <e> = ...next...;
    }
    ... <cursor> ...                # read before any reassignment
Use the entry (`<e>`, which IS the head when the walk ran off the end) after the loop,
or mark a deliberate site `// nolint: list-cursor-after-loop (reason)`.
"""

from __future__ import annotations

import argparse
import re
import sys
from pathlib import Path

from source_walk import source_files

REPO = Path(__file__).resolve().parents[1]
LEX = re.compile(r'/\*.*?\*/|//[^\n]*|b?r#*"(?:.|\n)*?"#*|b?"(?:\\.|[^"\\])*"', re.S)
WHILE = re.compile(r"\bwhile\s+(?P<e>[A-Za-z_]\w*)\s*!=\s*(?P<head>[^{]+?)\s*\{")
CONTAINER = re.compile(r"offset_of!|CONTAINING_RECORD!|LIST_ENTRY!|\blist_entry\b")
FUNCTION = re.compile(r"\bfn\s+([A-Za-z_]\w*)")


def mask(text: str) -> str:
    return LEX.sub(lambda m: re.sub(r"[^\n]", " ", m.group(0)), text)


def block_end(text: str, open_brace: int) -> int:
    depth = 0
    for i in range(open_brace, len(text)):
        c = text[i]
        if c == "{":
            depth += 1
        elif c == "}":
            depth -= 1
            if depth == 0:
                return i
    return -1


def scan_text(text: str) -> list[tuple[int, str, str]]:
    if "while" not in text or not CONTAINER.search(text):
        return []
    masked = mask(text)
    out = []
    for m in WHILE.finditer(masked):
        entry = m.group("e")
        open_brace = m.end() - 1
        end = block_end(masked, open_brace)
        if end < 0:
            continue
        body = masked[open_brace + 1 : end]
        if not re.search(r"\bbreak\b", body):
            continue
        assign = re.search(
            rf"(?<![\w.])(?P<c>[A-Za-z_]\w*)\s*=(?!=)(?P<rhs>[^;]*\b{re.escape(entry)}\b[^;]*);",
            body,
        )
        if not assign or not CONTAINER.search(assign.group("rhs")):
            continue
        cursor = assign.group("c")
        if cursor == entry:
            continue
        # A `let` inside the body is out of scope after the loop; a body that resets the
        # cursor after its `break` (`key = null_mut()`) already makes "not found" NULL.
        if re.search(r"\blet\s+(?:mut\s+)?$", body[: assign.start()]):
            continue
        brk = re.search(r"\bbreak\b", body)
        if brk and re.search(rf"(?<![\w.]){re.escape(cursor)}\s*=(?!=)", body[brk.end() :]):
            continue
        # Only wine/list.h walks (`struct list`, stepped by `.next`/list_next): there the C
        # cursor ends at the head. A ReactOS LIST_ENTRY walk (Flink) is an explicit C loop
        # whose cursor ends at the last element in the C too, so the Rust is faithful.
        if re.search(r"\bFlink\b|\bBlink\b", body) or not re.search(
            rf"\b{re.escape(entry)}\s*=\s*[^;]*(?:\.next\b|\blist_next\s*\()", body
        ):
            continue
        # The rest of the enclosing block after the loop: read before reassigned?
        rest = masked[end + 1 :]
        depth = 0
        stop = len(rest)
        for i, c in enumerate(rest):
            if c == "{":
                depth += 1
            elif c == "}":
                depth -= 1
                if depth < 0:
                    stop = i
                    break
        tail = rest[:stop]
        use = re.search(rf"(?<![\w.]){re.escape(cursor)}\b", tail)
        if not use:
            continue
        before = tail[: use.start()]
        after = tail[use.end() :]
        if re.match(r"\s*=(?!=)", after) and not re.search(r"\blet\b\s*(mut\s+)?$", before):
            continue  # reassigned before any read
        line = text[: m.start()].count("\n") + 1
        src_line = text.splitlines()[line - 1] if line - 1 < len(text.splitlines()) else ""
        if "nolint: list-cursor-after-loop" in src_line:
            continue
        fns = FUNCTION.findall(masked[: m.start()])
        out.append((line, fns[-1] if fns else "?", cursor))
    return out


def main(argv: list[str] | None = None) -> int:
    ap = argparse.ArgumentParser(description=__doc__.splitlines()[0])
    ap.add_argument("--lint", action="store_true", help="one finding per line, for the gate")
    ap.add_argument("--root", type=Path, default=REPO / "sdk" / "lib" / "rust")
    args = ap.parse_args(argv)
    findings = []
    for path in sorted(source_files(args.root)):
        try:
            text = path.read_text(errors="replace")
        except OSError:
            continue
        for line, fn, cursor in scan_text(text):
            rel = path.relative_to(REPO) if path.is_relative_to(REPO) else path
            findings.append(
                f"{rel}: {fn}: cursor `{cursor}` read after a list walk that can run off the end"
                f" (line {line})"
            )
    for f in findings:
        print(f)
    if not args.lint:
        print(f"{len(findings)} finding(s)", file=sys.stderr)
    return 0


if __name__ == "__main__":
    sys.exit(main())
