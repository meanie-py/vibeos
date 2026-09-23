#!/usr/bin/env python3
"""Find `SafeProbeFor*` followed by a RAW dereference of the probed pointer.

Why this exists: under `USE_DUMMY_PSEH` there is no exception handler, so a
probe and a dereference are not the same protection the C had.
`SafeProbeForRead` is validate-ONLY by design (`win32k-rust/src/safe_user_io.rs`
says so in its header): it range- and alignment-checks the buffer against
`MmUserProbeAddress` and deliberately does NOT touch the page, because a touch
fault would have nothing to catch it. The C wrapped probe AND access together in
one `_SEH2_TRY`; a translation that keeps the probe and drops the handler looks
protected and is not. A NULL or merely-unmapped user pointer sails through the
probe and faults on the first real access -> BUGCHECK 0x1E.

The apitest harness found four of these one twenty-minute boot at a time
(`NtUserSetSysColors`, `NtUserScrollDC`, `NtGdiCreateDIBSection`, and one more in
ntoskrnl), each needing a crash, a symbolication and a disassembly. Four is a
class, not a coincidence, and the whole class is visible statically -- so this
enumerates the rest for free.

The fix at a flagged site is never "delete the probe": it is to make the ACCESS
fault-tolerant, via `KmProtectedCopyMemory` / `_MmCopyFromCaller`
(`win32k-rust/src/copy.rs`), which all three arches carry with a per-boot
`[KMSAFECOPY] ... (PASS)` oracle. The probe stays, because it is what rejects a
kernel-range pointer.

Heuristic by nature: it reports LEADS, not verdicts. Read each against the C.
"""

from __future__ import annotations

import argparse
import re
import sys
from pathlib import Path

PROBE = re.compile(r"\bSafeProbeFor(?:Read|Write)\s*\(")

# Accesses that are already fault-tolerant: the pointer is handed to a copy
# primitive rather than dereferenced inline.
SAFE_SINKS = (
    "KmProtectedCopyMemory",
    "_MmCopyFromCaller",
    "_MmCopyToCaller",
    "MmCopyFromCaller",
    "MmCopyToCaller",
    "safe_copy_from_user",
    "safe_copy_to_user",
    "safe_copy_raw",
    "probe_read_user",
    "probe_write_user",
)


def base_ident(expr: str) -> str | None:
    """The pointer variable a probe's first argument is built from.

    Handles the shapes the tree actually uses: a bare name, a cast
    (`p as *const c_void`), and `core::ptr::addr_of!((*p).field)`.
    """
    m = re.search(r"addr_of(?:_mut)?!\s*\(\s*\(\s*\*\s*(\w+)", expr)
    if m:
        return m.group(1)
    expr = expr.split(" as ")[0].strip()
    m = re.fullmatch(r"[\w.]*?(\w+)", expr.strip())
    return m.group(1) if m else None


def split_args(text: str) -> list[str]:
    """Split a call's argument list on top-level commas."""
    args, depth, cur = [], 0, ""
    for ch in text:
        if ch in "([{":
            depth += 1
        elif ch in ")]}":
            if depth == 0:
                break
            depth -= 1
        if ch == "," and depth == 0:
            args.append(cur)
            cur = ""
            continue
        cur += ch
    if cur.strip():
        args.append(cur)
    return args


def is_comment(line: str) -> bool:
    t = line.strip()
    return t.startswith("//") or t.startswith("/*") or t.startswith("*")


def derefs_of(ident: str) -> re.Pattern[str]:
    i = re.escape(ident)
    # `*ident`, `*(ident`, `(*ident)`, and `(*ident).field`
    return re.compile(rf"(?<![\w.])\*\s*\(?\s*{i}(?![\w])|\(\s*\*\s*{i}\s*\)")


def scan_text(text: str, path: str) -> list[tuple[int, str, str]]:
    lines = text.splitlines()
    out: list[tuple[int, str, str]] = []
    for m in PROBE.finditer(text):
        args = split_args(text[m.end() :])
        if not args:
            continue
        ident = base_ident(args[0])
        if not ident:
            continue
        probe_line = text.count("\n", 0, m.start())
        pat = derefs_of(ident)
        # Look ahead within this function: stop at a line that starts a new
        # top-level item, which is a good-enough body boundary for this shape.
        for n in range(probe_line + 1, len(lines)):
            ln = lines[n]
            if re.match(r"^(pub\s+)?(unsafe\s+)?(extern\s+\"[^\"]+\"\s+)?fn\s", ln):
                break
            if any(sink in ln for sink in SAFE_SINKS):
                continue
            # These files keep the original C as a trailing comment block, so
            # the commented-out `rcScroll = *prcUnsafeScroll;` is the very text
            # this scanner is looking for. Matching it would bury every real
            # lead under the dead source it was translated from.
            if is_comment(ln):
                continue
            # `addr_of!((*p).field)` COMPUTES an address; it does not load, so
            # it cannot fault. It is also the idiom the probes themselves use to
            # name a subfield, so without this every correct probe reports
            # itself.
            if "addr_of!" in ln or "addr_of_mut!" in ln:
                continue
            if pat.search(ln):
                out.append((n + 1, ident, ln.strip()))
                break
    return [(ln, i, src) for ln, i, src in out]


def main(argv: list[str] | None = None) -> int:
    ap = argparse.ArgumentParser(description=__doc__)
    ap.add_argument("roots", nargs="*", default=["sdk/lib/rust"])
    args = ap.parse_args(argv)
    findings = 0
    for root in args.roots or ["sdk/lib/rust"]:
        for p in sorted(Path(root).rglob("*.rs")):
            text = p.read_text(errors="replace")
            if "SafeProbeFor" not in text:
                continue
            for line, ident, src in scan_text(text, str(p)):
                print(f"{p}:{line}: probed `{ident}` then dereferenced raw — {src}")
                findings += 1
    print(
        f"\n{findings} lead(s). Each needs reading against the C; the fix is a "
        f"fault-tolerant COPY, never deleting the probe.",
        file=sys.stderr,
    )
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
