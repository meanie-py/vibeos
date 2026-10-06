#!/usr/bin/env python3
"""
scan_raw_user_access.py — Nt* syscalls that load or store through a caller's pointer.

THE POINT. An Nt* syscall's pointer arguments are user addresses for a user-mode
caller. The C probes them and then touches them inside `_SEH2_TRY`, so a page that
is unmapped (or freed by another thread after the probe) becomes a returned
STATUS_ACCESS_VIOLATION. This kernel has no SEH, so in Rust every such touch has to
go through a fault-tolerant primitive (`ntkrnl_sys::probe::write_back`,
`write_user`, `probe_read_generic`, `safe_copy_raw`, `OutputBounce`, ...). A plain
`*Ptr = x`, `(*Ptr).Field`, or `copy_nonoverlapping` on one is a bugcheck any
process can trigger: the probe proves where the buffer IS, never that it stays
mapped. `scan_raising_probes` sees the probe; this sees what comes after it.

WHAT IS FLAGGED. In the body of each `pub unsafe extern "system" fn Nt*`, a raw
access through one of its pointer-typed parameters:

    *P = ..   (*P).f   *(P as T)   = *P   P.add(..) deref   copy/move/fill/volatile
    primitives with P as an argument

`addr_of!((*P).f)` is address arithmetic, not an access, and is not flagged. A
parameter re-bound to something other than `opaque_ptr(P)` (a capture, a pool
bounce) stops being tracked from that line on.

WHAT IT CANNOT SEE. Whether the access sits in a kernel-mode-only branch (where the
pointer is trusted), and accesses made by callees. Both are why this gates against
a BASELINE (`tools/lint-baseline/scan_raw_user_access.txt`) and not at zero: rows in
the baseline were read and are either kernel-mode branches or owed. A NEW row fails
`vibeos-cli check`. Suppress a reviewed site with `// nolint: raw-user-access
<reason>` on the line.

Usage:
    python3 tools/scan_raw_user_access.py            # rows: path: NtFoo Param
    python3 tools/scan_raw_user_access.py --lines    # also print each line
    python3 tools/scan_raw_user_access.py --lint     # terse rows (baseline gate)
    python3 tools/scan_raw_user_access.py <path ...>
"""

from __future__ import annotations

import argparse
import re
import sys
from pathlib import Path

sys.path.insert(0, str(Path(__file__).resolve().parent))
from source_walk import source_files

REPO = Path(__file__).resolve().parent.parent
DEFAULT_ROOT = REPO / "sdk" / "lib" / "rust"

SYSCALL_RE = re.compile(r'pub\s+unsafe\s+extern\s+"system"\s+fn\s+(Nt\w+)\s*\(')
NOLINT_RE = re.compile(r"nolint:\s*(raw-user-access|raw_user_access|scan_raw_user_access|all)\b")
# Pointer-typed parameters: raw pointers and the Windows P* aliases.
POINTER_TYPE_RE = re.compile(r"^\s*(\*\s*(mut|const)\b|P[A-Z0-9_]\w*$|LP[A-Z]\w*$)")
RAW_PRIMITIVES = (
    "copy_nonoverlapping",
    "copy",
    "write_bytes",
    "write_volatile",
    "read_volatile",
    "write_unaligned",
    "read_unaligned",
    "RtlCopyMemory",
    "RtlMoveMemory",
    "RtlZeroMemory",
    "RtlFillMemory",
)
SKIP_DIR_PARTS = ("-sys", "/target/", "rtl-strings", "ntdll")


def _strip(text: str) -> list[str]:
    """Code per line with // and /* */ comments and string contents blanked."""
    out: list[str] = []
    in_block = False
    for raw in text.splitlines():
        code: list[str] = []
        i = 0
        in_str = False
        while i < len(raw):
            if in_block:
                end = raw.find("*/", i)
                if end < 0:
                    i = len(raw)
                    break
                in_block = False
                i = end + 2
                continue
            ch = raw[i]
            if in_str:
                if ch == "\\":
                    i += 2
                    continue
                if ch == '"':
                    in_str = False
                    code.append('"')
                i += 1
                continue
            if raw.startswith("//", i):
                break
            if raw.startswith("/*", i):
                in_block = True
                i += 2
                continue
            if ch == '"':
                in_str = True
                code.append('"')
                i += 1
                continue
            code.append(ch)
            i += 1
        out.append("".join(code))
    return out


def _split_params(sig: str) -> list[tuple[str, str]]:
    params: list[tuple[str, str]] = []
    depth = 0
    cur = ""
    for ch in sig:
        if ch in "(<[":
            depth += 1
        elif ch in ")>]":
            depth -= 1
        if ch == "," and depth == 0:
            params.append(cur)
            cur = ""
        else:
            cur += ch
    if cur.strip():
        params.append(cur)
    out = []
    for p in params:
        if ":" not in p:
            continue
        name, ty = p.split(":", 1)
        name = name.replace("mut ", "").strip()
        out.append((name, ty.strip()))
    return out


def _access_patterns(name: str) -> list[re.Pattern[str]]:
    n = re.escape(name)
    prims = "|".join(RAW_PRIMITIVES)
    return [
        re.compile(rf"(?<![\w.])\*\s*{n}\b\s*(\.add\([^)]*\)\s*)?=(?!=)"),  # *P = / *P.add(i) =
        re.compile(rf"\(\*{n}\)\s*\."),  # (*P).f
        re.compile(rf"\*\(\s*{n}\s+as\b"),  # *(P as T)
        re.compile(rf"=\s*\*\s*{n}\b(?!\s*as\b)"),  # = *P
        re.compile(rf"(?<![\w:])(?:core::ptr::|ptr::)?(?:{prims})\s*\((?:[^;]*?[(,\s])?{n}\b"),
    ]


ADDR_OF_RE = re.compile(r"\baddr_of(?:_mut)?!\s*\(")


def _blank_addr_of(lines: list[str], start: int, end: int) -> list[str]:
    """Blank every addr_of!(..)/addr_of_mut!(..) span in lines[start..=end], across
    line breaks: computing an address through a pointer is not an access."""
    text = "\n".join(lines[start : end + 1])
    chars = list(text)
    for m in ADDR_OF_RE.finditer(text):
        depth = 0
        for k in range(m.end() - 1, len(text)):
            if text[k] == "(":
                depth += 1
            elif text[k] == ")":
                depth -= 1
                if depth == 0:
                    for q in range(m.start(), k + 1):
                        if chars[q] != "\n":
                            chars[q] = " "
                    break
    return "".join(chars).split("\n")


def _body_end(lines: list[str], start: int) -> int:
    depth = 0
    seen = False
    for i in range(start, len(lines)):
        for ch in lines[i]:
            if ch == "{":
                depth += 1
                seen = True
            elif ch == "}":
                depth -= 1
                if seen and depth == 0:
                    return i
    return len(lines) - 1


def scan_file(path: Path, rel: str) -> list[tuple[str, str, int, str]]:
    try:
        text = path.read_text(errors="replace")
    except OSError:
        return []
    if 'extern "system" fn Nt' not in text:
        return []
    raw_lines = text.splitlines()
    lines = _strip(text)
    hits: list[tuple[str, str, int, str]] = []
    i = 0
    while i < len(lines):
        # Signature detection reads the raw line: _strip blanks string contents, and
        # the ABI string is what identifies the syscall.
        m = SYSCALL_RE.search(raw_lines[i]) if i < len(raw_lines) else None
        if not m or raw_lines[i].lstrip().startswith("//"):
            i += 1
            continue
        fn = m.group(1)
        # Signature: from the '(' to its matching ')'.
        sig = ""
        depth = 0
        j = i
        col = raw_lines[i].index("(", m.start())
        done = False
        while j < len(lines) and not done:
            seg = raw_lines[j][col:] if j == i else lines[j]
            for ch in seg:
                if ch == "(":
                    depth += 1
                    if depth == 1:
                        continue
                elif ch == ")":
                    depth -= 1
                    if depth == 0:
                        done = True
                        break
                sig += ch
            sig += " "
            j += 1
        params = {
            n: _access_patterns(n) for n, ty in _split_params(sig) if POINTER_TYPE_RE.search(ty)
        }
        end = _body_end(lines, i)
        body = _blank_addr_of(lines, j - 1, end)
        tracked = dict(params)
        for k in range(j - 1, end + 1):
            code = body[k - (j - 1)]
            rebind = re.match(r"\s*let\s+(?:mut\s+)?(\w+)\s*(?::[^=]*)?=\s*(.*)", code)
            if rebind and rebind.group(1) in tracked:
                rhs = rebind.group(2)
                if not re.match(
                    rf"\s*opaque_ptr(_const)?\(\s*{re.escape(rebind.group(1))}\s*\)", rhs
                ):
                    del tracked[rebind.group(1)]
                    continue
            if NOLINT_RE.search(raw_lines[k]):
                continue
            for name, pats in tracked.items():
                if any(p.search(code) for p in pats):
                    hits.append((fn, name, k + 1, raw_lines[k].strip()))
        i = end + 1
    return hits


def main(argv: list[str] | None = None) -> int:
    ap = argparse.ArgumentParser(
        description=__doc__, formatter_class=argparse.RawDescriptionHelpFormatter
    )
    ap.add_argument("--lint", action="store_true", help="terse rows for the baseline gate")
    ap.add_argument("--lines", action="store_true", help="print each flagged line")
    ap.add_argument("paths", nargs="*", type=Path)
    args = ap.parse_args(argv)

    files: list[Path] = []
    for p in args.paths or [DEFAULT_ROOT]:
        if p.is_dir():
            files.extend(sorted(source_files(p)))
        elif p.suffix == ".rs":
            files.append(p)
    rows: dict[str, list[tuple[int, str]]] = {}
    for f in files:
        try:
            rel = f.resolve().relative_to(REPO).as_posix()
        except ValueError:
            rel = f.as_posix()
        if any(part in rel for part in SKIP_DIR_PARTS) or "bindgen" in f.name:
            continue
        for fn, name, line, text in scan_file(f, rel):
            rows.setdefault(f"{rel}: {fn} {name}", []).append((line, text))
    for row in sorted(rows):
        print(row)
        if args.lines and not args.lint:
            for line, text in rows[row]:
                print(f"    {line}: {text[:120]}")
    if not args.lint:
        print(f"\n{len(rows)} (syscall, pointer) pairs with a raw access", file=sys.stderr)
    return 0


if __name__ == "__main__":
    sys.exit(main())
