#!/usr/bin/env python3
"""Rewrite `SafeProbeFor*`-then-RAW-access sites onto the fault-tolerant copies.

`scan_probe_then_deref.py` enumerates the class; this applies the fix to the
subset where it is mechanical, and reports the rest for hand work. 165 leads is
too many to hand-edit and far too many to hand-edit CONSISTENTLY, but it is not
a class where a blanket rewrite is safe either -- so the tool is deliberately
conservative and refuses anything it cannot prove.

The fix shape (see `win32k-rust/src/copy.rs`): keep the probe, because it is
what rejects a kernel-range pointer, and route the ACCESS through
`KmProtectedCopyMemory`. On fault, run the error path the probe's OWN failure
block already spells out -- that is the whole trick, and it is why this can be
automated at all. We are not inventing an error path per function; we are
reusing the one the translator already wrote three lines up, so the rewrite
introduces no new semantics and every site's behaviour on a bad pointer is
whatever the C did on a bad pointer.

REFUSALS (reported, never guessed at):
  * no probe within the enclosing function
  * a probe whose failure block does not END in a control-flow transfer
    (`return`/`break`/`continue`). A block like `BadChk = 1;` sets a flag and
    FALLS THROUGH to a check further down; replaying it at the access site
    would set the flag and then carry straight on into the very access that
    just faulted. That is worse than the bug.
  * an access that is not a standalone statement (a copy nested inside a
    larger expression has no place to put the `if`)
  * `memcpy` and friends whose length is not textually recoverable

`SetLastNtError(<probe status>)` in a replayed block is rewritten to
STATUS_ACCESS_VIOLATION: at the access site the probe SUCCEEDED, so the
original variable holds STATUS_SUCCESS and replaying it verbatim would clear
the caller's last-error instead of setting it.
"""

from __future__ import annotations

import argparse
import re
import subprocess
import sys
from pathlib import Path

HERE = Path(__file__).resolve().parent

PROBE = re.compile(r"\bSafeProbeFor(Read|Write)\s*\(")
ACQUIRES = re.compile(
    r"\b(?:Ob[f]?ReferenceObject|ObReferenceObjectByHandle|UserReferenceObject"
    r"|UserRefObjectCo|ExAllocatePool\w*|UserEnter\w*|IntGetWindowObject"
    r"|_LockDc|DC_LockDc|GDIOBJ_\w*Lock\w*|KeAcquire\w*|RtlAllocateHeap)\b"
)
NEG_CHECK = re.compile(r"^\s*\}?\s*(?:else\s+)?if\s*!\s*NT_SUCCESS\s*\(")
FN_START = re.compile(r"^(?:pub\s+)?(?:unsafe\s+)?(?:extern\s+\"[^\"]+\"\s+)?fn\s+\w+")
TRANSFER = re.compile(r"\b(?:return\b[^;]*|break\s+'\w+|break|continue)\s*;\s*$")
# The replayed block may mention the probe's status variable ANYWHERE, not
# just inside SetLastNtError -- window.rs writes `return __SehStatus;`. And at
# the access site that variable is either out of scope (the probe often sits
# in its own `{ }`) or holds STATUS_SUCCESS, so replaying it verbatim reports
# success after a faulted copy. Substitute every occurrence, and cast with
# `as _` because SetLastNtError is declared taking u32 in some files and
# NTSTATUS in others.
AV = "ntkrnl_sys::probe::STATUS_ACCESS_VIOLATION as _"

# Trailing pointer casts we strip before re-casting to the helper's parameter
# type. `pfwi as *mut _` cannot be passed to a generic helper (nothing to infer
# `_` from) and cannot be re-cast blindly either, so it is normalised away.
TRAILING_CAST = re.compile(
    r"\s+as\s+\*(?:mut|const)\s+(?:_|core::ffi::c_void|std::ffi::c_void|"
    r"[A-Za-z_][\w:]*)\s*$"
)
TRAILING_NUM_CAST = re.compile(r"\s+as\s+(?:_|usize|u32|ULONG|SIZE_T|UINT|isize)\s*$")


def as_void(expr: str, mut: bool) -> str:
    """`expr` as `*mut`/`*const c_void`, stripping the casts that get in the way.

    A REFERENCE cannot be cast straight to a differently-typed pointer (E0606:
    `&mut tagCOLORADJUSTMENT as *mut c_void`), so `&mut ca as *mut _` must keep
    its intermediate step -- and that step infers fine, because the reference
    type supplies the pointee. A raw pointer must NOT keep it, because there
    `*mut _` has nothing to infer from. So the two cases get different
    treatment rather than one clever rule.
    """
    expr = expr.strip()
    while True:
        new = TRAILING_CAST.sub("", expr)
        if new == expr:
            break
        expr = new
    kind = "mut" if mut else "const"
    if expr.startswith("&"):
        return f"{expr} as *{kind} _ as *{kind} core::ffi::c_void"
    return f"{expr} as *{kind} core::ffi::c_void"


def strip_num_casts(expr: str) -> str:
    expr = expr.strip()
    while True:
        new = TRAILING_NUM_CAST.sub("", expr)
        if new == expr:
            return expr
        expr = new


def as_usize(expr: str) -> str:
    """`expr` coerced to usize, without a cast clippy will flag as redundant."""
    e = strip_num_casts(expr)
    if e.startswith("core::mem::size_of") or re.fullmatch(r"\d+", e):
        return e
    return f"{e} as usize"


def split_args(text: str) -> list[str]:
    args, depth, cur = [], 0, ""
    for ch in text:
        if ch in "([{":
            depth += 1
        elif ch in ")]}":
            if depth == 0:
                break
            depth -= 1
        if ch == "," and depth == 0:
            args.append(cur.strip())
            cur = ""
            continue
        cur += ch
    if cur.strip():
        args.append(cur.strip())
    return args


def call_extent(lines: list[str], start: int) -> tuple[int, str] | None:
    """From the statement starting at `start`, return (end_index, full text)."""
    depth, text, i, seen = 0, "", start, False
    while i < len(lines) and i < start + 60:
        text += lines[i] + "\n"
        for ch in lines[i]:
            if ch == "(":
                depth += 1
                seen = True
            elif ch == ")":
                depth -= 1
        if seen and depth == 0:
            return i, text
        i += 1
    return None


def enclosing_fn(lines: list[str], idx: int) -> int:
    for i in range(idx, -1, -1):
        if FN_START.match(lines[i]):
            return i
    return 0


def find_probe_block(lines: list[str], site: int) -> tuple[str, str, str | None] | None:
    """The NEGATIVE-form probe-failure block governing `site`, body only.

    Deliberately anchored on `if !NT_SUCCESS(`, never on "a line mentioning
    NT_SUCCESS". The tree writes the check both ways, and the POSITIVE form
    `if NT_SUCCESS(Status) { <copy> }` has no failure block at all -- its body
    IS the copy. An earlier cut of this matched on the bare name and so
    proposed replaying the copy itself as its own error path. Those sites are
    refused instead; they need a `Status = STATUS_ACCESS_VIOLATION` that only a
    human can confirm is actually read downstream.

    Searching BACKWARD from the access, not forward from the probe, is what
    picks up the common `if !NT_SUCCESS(s) { return 0; } else { <copies> }`
    shape, where several probes are chained into one Status first and the
    accesses sit in the else arm.
    """
    top = enclosing_fn(lines, site)
    if not any(PROBE.search(lines[i]) for i in range(top, site)):
        return None
    direction = "Read"
    for i in range(site - 1, top - 1, -1):
        if PROBE.search(lines[i]):
            direction = PROBE.search(lines[i]).group(1)
            break
    j = None
    for i in range(site - 1, top - 1, -1):
        if NEG_CHECK.search(lines[i]):
            j = i
            break
    if j is None:
        return None
    # The failure block must belong to THE probe governing this access, i.e. it
    # must come AFTER it. Without this the search happily picks up the previous
    # loop iteration's block: NtUserSetWindowStationUser got `break 'leave` from
    # an unrelated earlier check, which would have skipped the `if
    # !NT_SUCCESS(Status)` cleanup sitting three lines below the copy. Wrong
    # error path is not a smaller bug than no error path.
    probe_i = max(i for i in range(top, site) if PROBE.search(lines[i]))
    if probe_i > j:
        return None
    # The condition must be single-line, i.e. the line opens the block. The tree
    # also writes `if !NT_SUCCESS(SafeProbeForRead(\n  p,\n  n,\n  1,\n))` --
    # there the brace scan starts below the check and happily scoops the probe's
    # own ARGUMENTS up as the error-path body, which produced syntactic rubbish
    # in NtUserSBGetParms. The build would have caught it; the tool should not
    # have emitted it.
    if not lines[j].rstrip().endswith("{"):
        return None
    # Character-accurate, because `} else {` keeps a line-granular brace count
    # at depth 1 forever and swallows the rest of the function -- and the
    # `if !NT_SUCCESS(s) { return 0; } else { <copies> }` shape, where that
    # matters, is the single commonest one in the tree.
    depth, k, started, closed, inner = 0, j, False, False, []
    while k < len(lines) and not closed:
        for ch in lines[k]:
            if ch == "{":
                depth += 1
                started = True
            elif ch == "}":
                depth -= 1
                # Mid-line, so that `} else {` closes the block instead of
                # re-opening it and running to the end of the function.
                if started and depth == 0:
                    closed = True
                    break
        if closed:
            break
        if k > j:
            inner.append(lines[k])
        k += 1
    if not closed or k >= site:
        return None  # unterminated, or the access is INSIDE this block
    # Replaying an error path is only sound if nothing since it was written has
    # changed what it MEANS. Raw line distance is the wrong proxy -- it refused
    # NtUserGetLayeredWindowAttributes' third out-parameter while fixing the two
    # directly above it, leaving an identical live bugcheck nine lines down. The
    # two things that actually invalidate a replay:
    #
    #  (a) the status the block tests has been REASSIGNED since. That is what
    #      makes NtUserGetObjectInformation unsafe: it probes at the top, then a
    #      whole match arm sets Status, and by the copy a replayed `break 'exit`
    #      would return SUCCESS after a faulted write.
    #  (b) a resource has been acquired that the block's transfer does not
    #      release -- the same function leaks two referenced objects that way.
    #
    # Either one, refuse and let a human decide; the answer at those sites is
    # usually `Status = STATUS_ACCESS_VIOLATION` and fall through to cleanup,
    # which is a judgement about what the cleanup does.
    m = re.search(r"NT_SUCCESS\s*\(\s*([A-Za-z_]\w*)\s*\)", lines[j])
    status_var = m.group(1) if m else None
    between = lines[k + 1 : site]
    if status_var and any(re.search(rf"\b{re.escape(status_var)}\s*=[^=]", ln) for ln in between):
        return None
    if any(ACQUIRES.search(ln) for ln in between):
        return None
    inner = [ln for ln in inner if ln.strip()]
    if not inner or not TRANSFER.search(inner[-1].strip()):
        return None
    base = min(len(ln) - len(ln.lstrip()) for ln in inner)
    return "\n".join(ln[base:] for ln in inner), direction, status_var


def rewrite_site(lines: list[str], site: int, probed: str) -> tuple[int, str] | None:
    """Return (end_line, replacement_text) or None if the site is refused."""
    raw = lines[site]
    indent = raw[: len(raw) - len(raw.lstrip())]
    stmt = raw.strip()
    for name in (
        "RtlCopyMemory",
        "RtlMoveMemory",
        "core::ptr::copy_nonoverlapping",
        "core::ptr::write_volatile",
    ):
        if stmt.startswith(name + "("):
            kind = name
            break
    else:
        return None
    ext = call_extent(lines, site)
    if ext is None:
        return None
    end, text = ext
    tail = text.rstrip()
    if not tail.endswith(");"):
        return None  # not a standalone statement
    args = split_args(text[text.index("(") + 1 :])
    blk = find_probe_block(lines, site)
    if blk is None:
        return None
    body, _direction, status_var = blk
    if status_var:
        body = re.sub(rf"\b{re.escape(status_var)}\b", AV, body)

    if kind in ("RtlCopyMemory", "RtlMoveMemory"):
        if len(args) != 3:
            return None
        dst, src, n = args
        # Which side is the user pointer decides read vs write.
        helper = "capture_from_user" if probed in src else "publish_to_user"
        if probed not in src and probed not in dst:
            return None
        call = (
            f"{indent}if !crate::copy::{helper}(\n"
            f"{indent}    {as_void(dst, True)},\n"
            f"{indent}    {as_void(src, False)},\n"
            f"{indent}    {as_usize(n)},\n"
            f"{indent}) {{\n"
        )
    elif kind == "core::ptr::copy_nonoverlapping":
        if len(args) != 3:
            return None
        src, dst, count = args
        helper = "capture_n" if probed in src else "publish_n"
        if probed not in src and probed not in dst:
            return None
        call = (
            f"{indent}if !crate::copy::{helper}(\n"
            f"{indent}    {dst},\n"
            f"{indent}    {src},\n"
            f"{indent}    {as_usize(count)},\n"
            f"{indent}) {{\n"
        )
    else:  # write_volatile
        if len(args) != 2:
            return None
        dst, val = args
        if probed not in dst:
            return None
        call = f"{indent}if !crate::copy::write_user({dst}, {val}) {{\n"

    body_txt = "\n".join((indent + "    " + ln) if ln else "" for ln in body.splitlines())
    return end, call + body_txt + f"\n{indent}}}\n"


def main() -> int:
    ap = argparse.ArgumentParser(description=__doc__)
    ap.add_argument("--write", action="store_true", help="apply (default: report)")
    ap.add_argument("--only", help="restrict to paths containing this substring")
    args = ap.parse_args()

    out = subprocess.run(
        [sys.executable, str(HERE / "scan_probe_then_deref.py")],
        capture_output=True,
        text=True,
        check=False,
    ).stdout
    sites: dict[str, dict[int, str]] = {}
    for line in out.splitlines():
        m = re.match(r"^(\S+\.rs):(\d+): probed `([^`]+)`", line)
        if not m:
            continue
        f, ln, probed = m.group(1), int(m.group(2)), m.group(3)
        if args.only and args.only not in f:
            continue
        sites.setdefault(f, {})[ln] = probed

    fixed = refused = 0
    refusals: list[str] = []
    for f, per in sorted(sites.items()):
        path = Path(f)
        lines = path.read_text().splitlines()
        edits = []
        for ln in sorted(per):
            r = rewrite_site(lines, ln - 1, per[ln])
            if r is None:
                refused += 1
                refusals.append(f"{f}:{ln}  {lines[ln - 1].strip()[:70]}")
                continue
            edits.append((ln - 1, r[0], r[1]))
        # apply bottom-up so earlier indices stay valid
        for start, end, text in sorted(edits, reverse=True):
            lines[start : end + 1] = text.rstrip("\n").splitlines()
            fixed += 1
        if edits and args.write:
            path.write_text("\n".join(lines) + "\n")

    print(f"rewritten: {fixed}   refused: {refused}")
    if refusals:
        print("\nrefused (hand work -- read each against the C):")
        for r in refusals:
            print("  " + r)
    if not args.write:
        print("\n(report only; pass --write to apply)")
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
