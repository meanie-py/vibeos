#!/usr/bin/env python3
"""
scan_init_then_move.py — a kernel object initialized in a MaybeUninit, then moved out of it.

    let mut Event = MaybeUninit::uninit();
    KeInitializeEvent(Event.as_mut_ptr(), NotificationEvent, FALSE);
    let Event = Event.assume_init();          // a COPY, at a new address

An initialized KEVENT's empty wait list points at ITSELF (Flink = Blink = &WaitListHead), as
does every LIST_ENTRY head, KTIMER, KMUTANT, KSEMAPHORE and ERESOURCE. Moving the object
copies those pointers, which still name the old slot: the list now looks non-empty, and the
first KeSetEvent "unwaits" whatever bytes sit there. iomgr's IoShutdownSystem did exactly
this -- 0x109 when mountmgr's shutdown IRP completed, at the end of every install. Whether
it fires depends on whether LLVM happened to elide the copy, so it can sleep for months.

Flagged: `X.as_mut_ptr()` passed to an `*Initialize*` routine, then `X.assume_init()` taken
by value. Initialize the object in place (`let mut E: KEVENT = zeroed(); KeInitializeEvent(
&mut E, ..)`) and keep using `&mut E`. Gated at zero.

Also flagged, the constructor shape: `&mut X` or `&mut X.field` passed to an `*Initialize*`
routine inside a function that then RETURNS `X` by value (its tail expression, or
`return X;`). The caller receives a copy at a new address. iomgr's NtCancelIoFileEx shipped
this way in its first guest run (a KEVENT inside a request struct built by `new_request()`),
and bugchecked 0x109 on i386 and 0xA on amd64 the first time it waited.

Usage:
    python3 tools/scan_init_then_move.py [paths...]   # default: sdk/lib/rust
    python3 tools/scan_init_then_move.py --lint
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
MARKER = "ORIGINAL C SOURCE"
INIT_RE = re.compile(r"\b(\w*Initiali[sz]e\w*)\s*\(\s*(?:&\s*mut\s+)?(\w+)\.as_mut_ptr\(\)")
WINDOW = 6000  # chars after the init call to look for the move
INIT_REF_RE = re.compile(r"\b(\w*Initiali[sz]e\w*)\s*\(\s*&\s*mut\s+(\w+)\b")
FN_RE = re.compile(r"\bfn\s+\w+[^{;]*\{")


def _fn_bodies(code: str) -> list[tuple[int, int]]:
    """(start, end) of every fn body, by brace matching (comments already blanked)."""
    out = []
    for m in FN_RE.finditer(code):
        depth, i = 1, m.end()
        while i < len(code) and depth:
            c = code[i]
            depth += c == "{"
            depth -= c == "}"
            i += 1
        out.append((m.end(), i - 1))
    return out


def _returned_by_value(body: str, var: str) -> bool:
    """`var` is the body's tail expression or a `return var;`."""
    tail = body.rstrip()
    v = re.escape(var)
    return bool(
        re.search(rf"(?:^|[;{{}}\n])\s*{v}\s*$", tail) or re.search(rf"\breturn\s+{v}\s*;", body)
    )


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
            cut = text.find(MARKER)
            if cut >= 0:
                text = text[:cut]
            code = re.sub(
                r"//[^\n]*|/\*.*?\*/", lambda m: re.sub(r"[^\n]", " ", m.group(0)), text, flags=re.S
            )
            for m in INIT_RE.finditer(code):
                var = m.group(2)
                tail = code[m.end() : m.end() + WINDOW]
                mv = re.search(rf"(?<![\w.]){re.escape(var)}\.assume_init\(\)", tail)
                if mv:
                    line = code.count("\n", 0, m.end() + mv.start()) + 1
                    findings.append(
                        (f, line, f"{m.group(1)}({var}.as_mut_ptr()) then {var}.assume_init()")
                    )
            for start, end in _fn_bodies(code):
                body = code[start:end]
                for m in INIT_REF_RE.finditer(body):
                    var = m.group(2)
                    if _returned_by_value(body, var):
                        line = code.count("\n", 0, start + m.start()) + 1
                        findings.append(
                            (f, line, f"{m.group(1)}(&mut {var}..) then {var} returned by value")
                        )
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
        tag = "init-then-move" if lint else "initialized object moved:"
        print(f"{rel}:{lineno}: {tag} {what}")
    if findings and not lint:
        print(f"\n{len(findings)} finding(s): initialize in place and never move it.")
    return 1 if findings else 0


if __name__ == "__main__":
    sys.exit(main())
