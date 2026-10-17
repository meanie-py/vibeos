#!/usr/bin/env python3
"""scan_dropped_asserts.py — a C `ASSERT` that VANISHED when its FORCEINLINE was
hand-translated to Rust.

WHY THIS EXISTS
---------------
`MI_MAKE_HARDWARE_PTE_USER` is a FORCEINLINE in ntoskrnl/mm/ARM3/miarm.h that
opens with three asserts, commented in the C as "check that we are not setting
valid a page that should not be":

    ASSERT(ProtectionMask & MM_PROTECT_ACCESS);   /* and two siblings */

Both Rust translations dropped all three. On x86 a mask granting no access still
yields a page the MMU can translate, so the missing precondition is invisible.
On arm64 `MmProtectToPteMask` is the ONLY source of the level-3 descriptor-type
bit, so a no-access mask produces `Valid = 1` on a descriptor the hardware
rejects — the pager reports success forever, with interrupts masked, and the
machine stops without a bugcheck. An assert that would have fired loudly on
every arch instead became an unobservable freeze on one.

The general shape: a C assert is a documented PRECONDITION. Dropping it does not
change behaviour where the precondition holds, so nothing goes red until an
arch or a workload violates it — and then the symptom is arbitrarily far from
the cause.

WHY NO EXISTING GATE CATCHES IT
-------------------------------
  * `scan_assert_to_ub` catches an assert LOWERED to `unreachable_unchecked()`
    — present and wrong. This one is deleted outright; there is nothing to see.
  * `scan_dropped_prints` is the closest shape and it CANNOT be cloned for this.
    Its oracle is rustconv's trailing C comments in the same `.rs`, and the two
    files that carried this bug (`ntoskrnl-rust/src/ARM3/helpers.rs`,
    `ntkrnl-sys/src/inline/mm.rs`) are HAND-WRITTEN bridge modules with no
    trailing C. A same-file diff has nothing to compare and reports clean on the
    exact function that cost the boot.
  * `verify_signatures` / `verify_c_signatures` compare types, not bodies.

So the oracle has to be CROSS-FILE: the C FORCEINLINE paired by NAME against its
Rust twin anywhere in the workspace. That pairing is only sound where the two
sides deliberately share names — which is exactly the FORCEINLINE surface in the
internal headers, where the bridge modules mirror C names one-for-one.

WHAT IT REPORTS, AND WHY ONLY THAT
----------------------------------
FEWER asserts on the Rust side than the C side. Not "no asserts": a partial drop
is the same class, and #39 dropped 3 of 3 only because that is the easy end of
the distribution.

Counted on both sides: ASSERT, NT_ASSERT, PAGED_CODE, plus Rust's `assert!` /
`debug_assert!`. `ASSERT(` and `ASSERT!` are both live spellings in the tree.

Comments and string literals are stripped before counting on BOTH sides. This is
load-bearing, not hygiene: rustconv parks the original C in trailing comments, so
counting raw text would score a translated file by the C it quotes rather than
the code it runs, and every dropped assert would read as present.

A symbol with several Rust definitions (per-arch `#[cfg]` twins) is scored
against the MAX over them, so an arch-specific variant that legitimately checks
less does not report while its sibling still carries the assert. That deliberately
under-reports; the alternative floods on every cfg-split pair. #39 is still caught
because BOTH of its definitions had zero.
"""

from __future__ import annotations

import re
import sys
from bisect import bisect_left
from pathlib import Path

sys.path.insert(0, str(Path(__file__).resolve().parent.parent))
from tools.lint_facts import source_facts

REPO = Path(__file__).resolve().parent.parent
RUST_WS = REPO / "sdk" / "lib" / "rust"

# The FORCEINLINE surface where C and Rust deliberately share names. Kept
# explicit rather than "all headers": the pairing is only sound where the bridge
# modules mirror C names, and a name collision outside it would be a false pair.
C_HEADER_ROOTS = (
    REPO / "ntoskrnl" / "include" / "internal",
    REPO / "ntoskrnl" / "mm" / "ARM3",
)

ASSERT_RE = re.compile(r"\b(?:NT_ASSERT|ASSERT|PAGED_CODE|debug_assert|assert)\s*[!(]")

# `FORCEINLINE [qualifiers] NAME(` — the name is the last identifier before the
# parameter list.
C_FUNC_RE = re.compile(
    r"\bFORCEINLINE\b[^;{()]*?\b(?P<name>[A-Za-z_]\w*)\s*\(",
    re.S,
)
RUST_FUNC_RE = re.compile(r"\bfn\s+(?P<name>[A-Za-z_]\w*)\s*[(<]")


def strip_noise(text: str) -> str:
    """Remove comments and string/char literals.

    Load-bearing for this scanner: rustconv keeps the original C in trailing
    comments, so an unstripped count scores a file by the C it quotes.

    Every removed span is replaced by its own newlines, so offsets still map to
    the ORIGINAL line numbers. Dropping them silently shifted every reported
    line past the first block comment, which points a reader at unrelated code.
    """
    out = []
    i, n = 0, len(text)
    special = re.compile(r"//|/\*|[\"']")
    quotes = re.compile(r"[\\\"']")
    while i < n:
        token = special.search(text, i)
        if token is None:
            out.append(text[i:])
            break
        out.append(text[i : token.start()])
        i = token.start()
        c = text[i]
        nxt = text[i + 1] if i + 1 < n else ""
        if c == "/" and nxt == "/":
            j = text.find("\n", i)
            i = n if j < 0 else j
        elif c == "/" and nxt == "*":
            j = text.find("*/", i + 2)
            end = n if j < 0 else j + 2
            out.append("\n" * text.count("\n", i, end))
            i = end
        elif c in "\"'":
            quote, start, i = c, i, i + 1
            while i < n:
                marker = quotes.search(text, i)
                if marker is None:
                    i = n
                    break
                i = marker.end()
                if marker.group() == "\\":
                    i += 1
                elif marker.group() == quote:
                    break
            out.append("\n" * text.count("\n", start, i))
        else:
            out.append(c)
            i += 1
    return "".join(out)


def body_after(text: str, start: int) -> str | None:
    """Brace-matched body of the definition whose signature begins at `start`.

    Returns None for a declaration (a `;` before any `{`), which is what tells a
    prototype apart from a definition.
    """
    i = text.find("{", start)
    if i < 0:
        return None
    semi = text.find(";", start)
    if 0 <= semi < i:
        return None
    depth = 0
    for brace in re.compile(r"[{}]").finditer(text, i):
        depth += 1 if brace.group() == "{" else -1
        if depth == 0:
            return text[i : brace.end()]
    return None


def count_asserts(body: str) -> int:
    return len(ASSERT_RE.findall(body))


@source_facts()
def assertion_facts(raw: str, rust: bool):
    text = strip_noise(raw)
    pattern = RUST_FUNC_RE if rust else C_FUNC_RE
    out = []
    newlines = [m.start() for m in re.finditer("\n", text)]
    for m in pattern.finditer(text):
        body = body_after(text, m.end())
        if body is not None:
            out.append((m.group("name"), bisect_left(newlines, m.start()) + 1, count_asserts(body)))
    return out


def c_forceinlines(paths: list[Path]) -> dict[str, tuple[Path, int, int]]:
    """name -> (file, line, assert count). Highest count wins a duplicate name."""
    found: dict[str, tuple[Path, int, int]] = {}
    for p in paths:
        try:
            raw = p.read_text(errors="replace")
        except OSError:
            continue
        for name, line, n in assertion_facts(raw, False):
            if name not in found or n > found[name][2]:
                found[name] = (p, line, n)
    return found


def rust_fns(paths: list[Path]) -> dict[str, tuple[list[tuple[Path, int]], int]]:
    """name -> (definition sites, MAX assert count over them)."""
    found: dict[str, tuple[list[tuple[Path, int]], int]] = {}
    for p in paths:
        try:
            raw = p.read_text(errors="replace")
        except OSError:
            continue
        for name, line, n in assertion_facts(raw, True):
            sites, best = found.get(name, ([], -1))
            sites.append((p, line))
            found[name] = (sites, max(best, n))
    # `X` forwarding to `XSited(…, site)` is the lockdep convention in ntkrnl-sys:
    # the export shims call the Sited body directly with their return address, so
    # the C's asserts belong there, and the plain wrapper inherits its count.
    for name, (sites, best) in list(found.items()):
        sited = found.get(name + "Sited")
        if sited is not None and sited[1] > best:
            found[name] = (sites, sited[1])
    return found


def scan(
    header_roots: tuple[Path, ...] = C_HEADER_ROOTS,
    rust_ws: Path = RUST_WS,
    repo: Path = REPO,
) -> list[tuple[Path, int, str]]:
    headers = sorted(h for root in header_roots if root.is_dir() for h in root.rglob("*.h"))
    # Exclude cargo output: sdk/lib/rust/target is a build tree, not source, and
    # cargo deletes directories underneath a walk in progress. Prune rather than
    # filter after the fact — the audited source-early partition runs alongside
    # live builds.
    rs: list[Path] = []
    stack = [rust_ws]
    while stack:
        d = stack.pop()
        try:
            entries = list(d.iterdir())
        except OSError:
            continue
        for e in entries:
            try:
                if e.is_dir():
                    if e.name not in ("target", ".git"):
                        stack.append(e)
                elif e.suffix == ".rs":
                    rs.append(e)
            except OSError:
                continue

    cfns = c_forceinlines(headers)
    rfns = rust_fns(sorted(rs))

    findings = []
    for name, (cfile, cline, cn) in sorted(cfns.items()):
        if cn == 0 or name not in rfns:
            continue
        sites, rn = rfns[name]
        if rn >= cn:
            continue
        where = ", ".join(f"{p.relative_to(repo)}:{ln}" for p, ln in sorted(sites)[:3])
        findings.append(
            (
                cfile,
                cline,
                f"{name}: C has {cn} assert(s), Rust has {rn} ({where})",
            )
        )
    return sorted(findings, key=lambda f: (str(f[0]), f[1], f[2]))


def main() -> int:
    argv = sys.argv[1:]
    lint = "--lint" in argv
    rest = [a for a in argv if a != "--lint"]
    if rest and rest[0] == "--root":
        # One directory standing in for both sides, so a control fixture can ship
        # a .h and its .rs twin together. Used by tools/test_scanner_controls.py.
        root = Path(rest[1]).resolve()
        findings = scan(header_roots=(root,), rust_ws=root, repo=root)
    else:
        findings = scan()
    for f, line, msg in findings:
        try:
            shown = f.relative_to(REPO)
        except ValueError:
            shown = f
        print(f"{shown}:{line}: {msg}")
    # A tally is presentation, not a finding: under --lint the gate diffs stdout
    # against the baseline line by line, so a moving count reads as "1 new".
    if not lint:
        print(f"\n{len(findings)} dropped-assert finding(s)")
    return 1 if findings else 0


if __name__ == "__main__":
    sys.exit(main())
