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

sys.path.insert(0, str(Path(__file__).resolve().parent))
from source_walk import source_files

PROBE = re.compile(r"\bSafeProbeFor(?:Read|Write)\s*\(")

# Bulk copies that are PLAIN memcpy/memset -- no fault handler. Handing the probed
# pointer to one of these is exactly as exposed as `*p`, and it is invisible to a
# deref-shaped search: `RtlCopyMemory(&mut dsm, pdsm, n)` reads the whole user
# struct without a single `*` in sight. Found because the interprocedural pass
# pointed at a line two above one of these.
UNSAFE_COPIES = (
    "RtlCopyMemory",
    "RtlMoveMemory",
    "RtlZeroMemory",
    "RtlFillMemory",
    "copy_nonoverlapping",
    "write_volatile",
    "read_volatile",
)

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
    # The win32k helpers added when fixing this very class -- without these,
    # every site already fixed reports itself forever.
    "capture_from_user",
    "publish_to_user",
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
    """A comment line, NOT a line that merely starts with a dereference.

    The naive `t.startswith("*")` -- meant to catch block-comment continuation
    lines like ` * foo` -- silently swallowed every statement beginning with a
    store, i.e. `*(p as *mut T) = v;`. That is precisely the shape this scanner
    exists to find, so the guard against false positives created a false
    NEGATIVE on the highest-value case: it hid a live BUGCHECK in
    NtUserScrollDC and reported the file clean.

    A continuation is `*` followed by whitespace or nothing, or `*/`; a
    dereference is `*` followed immediately by an identifier or `(`.
    """
    t = line.strip()
    if t.startswith("//") or t.startswith("/*") or t.startswith("*/"):
        return True
    return t == "*" or (t.startswith("*") and t[1:2].isspace())


def derefs_of(ident: str) -> re.Pattern[str]:
    i = re.escape(ident)
    # `*ident`, `*(ident`, `(*ident)`, and `(*ident).field`
    return re.compile(rf"(?<![\w.])\*\s*\(?\s*{i}(?![\w])|\(\s*\*\s*{i}\s*\)")


FN_DEF = re.compile(
    r'^(?:pub\s+)?(?:unsafe\s+)?(?:extern\s+"[^"]+"\s+)?fn\s+(\w+)\s*\(',
    re.MULTILINE,
)


def deref_params(text: str) -> dict[str, set[int]]:
    """Map fn name -> the parameter POSITIONS whose pointer it raw-dereferences.

    Why this exists: the probe and the dereference are often in different
    functions. `NtUserSetSysColors` probes two user arrays and hands them to
    `IntSetSysColors`, which does `*elem_ptr` in a loop -- a live BUGCHECK 0x1E
    the single-function scan could not see, and did not see. Purely local
    analysis is structurally blind to the whole class of "probe at the syscall,
    deref one level down".

    Deliberately shallow: one call level, positional, same crate. It reports
    LEADS, and a missed second level is better than a scanner nobody trusts.
    """
    out: dict[str, set[int]] = {}
    defs = list(FN_DEF.finditer(text))
    for k, m in enumerate(defs):
        name = m.group(1)
        params = split_args(text[m.end() :])
        end = defs[k + 1].start() if k + 1 < len(defs) else len(text)
        body = text[m.end() : end]
        hits = set()
        for pos, param in enumerate(params):
            pname = param.split(":")[0].strip().lstrip("mut ").strip()
            if not re.fullmatch(r"\w+", pname or ""):
                continue
            pat = derefs_of(pname)
            for ln in body.splitlines():
                if any(sink in ln for sink in SAFE_SINKS) or is_comment(ln):
                    continue
                if "addr_of!" in ln or "addr_of_mut!" in ln:
                    continue
                # A parameter handed to a BULK COPY counts as dereferenced. Missing
                # this is how NtGdiCreateBitmap stayed hidden: the probe is in the
                # syscall, the access is an RtlCopyMemory one frame down inside
                # UnsafeSetBitmapBits, and neither the deref-shaped search nor the
                # interprocedural pass could see it alone. It shipped a live
                # BUGCHECK 0x1E that an apitest found instead.
                if any(c in ln for c in UNSAFE_COPIES) and re.search(
                    rf"\b{re.escape(pname)}\b", ln
                ):
                    hits.add(pos)
                    break
                if pat.search(ln):
                    hits.add(pos)
                    break
        if hits:
            out.setdefault(name, set()).update(hits)
    return out


CALL = re.compile(r"\b(\w+)\s*\(")


def text_of_line_call(lines: list[str], n: int, start: int) -> str:
    """The argument text of a call beginning on line `n`, which may wrap.

    Multi-line call formatting is the norm here (rustfmt splits anything long),
    so reading only the one line would miss most arguments.
    """
    return "\n".join([lines[n][start:], *lines[n + 1 : n + 12]])


def scan_text(
    text: str, path: str, deref_index: dict[str, set[int]] | None = None
) -> list[tuple[int, str, str]]:
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
            # Reassigning the probed identifier to an UNRELATED pointer ends the
            # taint: it now names something else. dibobj's `bmi = pbmiSafe;` swaps a
            # user pointer for a kernel pool copy, so every later `(*bmi).…` is a
            # kernel load -- reported as a lead for as long as this was missing.
            #
            # Assigning NULL does NOT end it. Error paths null the pointer and the
            # success path keeps deref'ing the original; treating that as untainted
            # hid the real `dwHookData` leads two lines further down.
            reassign = re.match(rf"\s*{re.escape(ident)}\s*=\s*([^=].*)$", ln)
            if (
                reassign
                and not pat.search(ln)
                # word boundary, not substring: `bmi = pbmiSafe;` has "bmi" inside
                # "pbmiSafe", which read as self-reference and kept the taint alive.
                and not re.search(rf"\b{re.escape(ident)}\b", reassign.group(1))
                and "null" not in reassign.group(1).lower()
            ):
                break
            # `addr_of!((*p).field)` COMPUTES an address; it does not load, so it
            # cannot fault. It is also the idiom the probes themselves use to name a
            # subfield, so without this every correct probe reports itself.
            #
            # This MUST come after the taint-end check above, not before it. Repointing
            # the probed name at a kernel local -- `prcl = addr_of_mut!(rclLocal);` --
            # is the tree's idiom for the CORRECT fix, and skipping the line for
            # containing `addr_of_mut!` skipped the reassignment that ends the taint,
            # so every such site kept reporting after it was fixed. That is the same
            # false-signal shape as the old break-at-first-hit bug: the tool saying
            # there is work left where there is none.
            if "addr_of!" in ln or "addr_of_mut!" in ln:
                continue
            # The probed pointer handed to a callee that dereferences THAT
            # parameter -- the NtUserSetSysColors -> IntSetSysColors shape.
            if deref_index:
                call = CALL.search(ln)
                # A callee that is ITSELF a bulk copy is the DIRECT case handled
                # below, not an interprocedural one. Reporting both counted every
                # `RtlCopyMemory(probed, ...)` twice -- 281 rows for 165 real sites,
                # and the inflated number got quoted as the size of the workload.
                # The interprocedural rule exists for PROJECT functions that hide the
                # deref behind a call (NtUserSetSysColors -> IntSetSysColors); for a
                # memcpy the offending line is right there and says more.
                if call and call.group(1) in deref_index and call.group(1) not in UNSAFE_COPIES:
                    cargs = split_args(text_of_line_call(lines, n, call.end()))
                    for pos, a in enumerate(cargs):
                        # Strip is_null() guards first. The tree's idiom is
                        # `if !p.is_null() { &SafeLocal } else { null }`, where the
                        # argument actually passed is a KERNEL local and the probed
                        # name appears only in the test. Without this, three of the
                        # first four interprocedural hits were that shape.
                        a = re.sub(rf"\b{re.escape(ident)}\s*\.is_null\s*\(\)", "", a)
                        if pos in deref_index[call.group(1)] and re.search(
                            rf"\b{re.escape(ident)}\b", a
                        ):
                            out.append(
                                (
                                    n + 1,
                                    ident,
                                    f"passed to {call.group(1)}() which dereferences arg {pos} raw",
                                )
                            )
                            break
            # A bulk copy naming the probed pointer -- see UNSAFE_COPIES. Matched
            # against the call's OWN arguments, not a line window: a window flagged
            # stretchblt's three back-to-back copies for each other's pointers.
            copy = next((c for c in UNSAFE_COPIES if c + "(" in ln.replace(" ", "")), None)
            if copy:
                at = ln.find(copy)
                cargs = split_args(text_of_line_call(lines, n, at + len(copy) + 1))
                if any(
                    re.search(rf"\b{re.escape(ident)}\b", a)
                    and not re.search(rf"\b{re.escape(ident)}\s*\.is_null\s*\(\)", a)
                    for a in cargs
                ):
                    out.append((n + 1, ident, ln.strip()))
            if pat.search(ln):
                out.append((n + 1, ident, ln.strip()))
                # NO break: report EVERY unprotected access, not just the first.
                # Stopping at the first one understated the real count and made the
                # tool look like it was finding new bugs each time one was fixed --
                # the same failure as the `is_comment` bug that hid scrollex.rs.
    # one row per (line, ident); a line can match several patterns
    return sorted({(ln, i, src) for ln, i, src in out})


def main(argv: list[str] | None = None) -> int:
    ap = argparse.ArgumentParser(description=__doc__)
    ap.add_argument("roots", nargs="*", default=["sdk/lib/rust"])
    args = ap.parse_args(argv)
    findings = 0
    for root in args.roots or ["sdk/lib/rust"]:
        # source_files() prunes vendored trees and build dirs; a bare rglob
        # would scan both (and `check` lints for exactly that).
        files = sorted(source_files(Path(root), "*.rs"))
        texts = {p: p.read_text(errors="replace") for p in files}
        # Index EVERY file, not just the probing ones: the callee that does the
        # raw deref usually lives in a different module from the syscall.
        deref_index: dict[str, set[int]] = {}
        for t in texts.values():
            for name, positions in deref_params(t).items():
                deref_index.setdefault(name, set()).update(positions)
        for p in files:
            text = texts[p]
            if "SafeProbeFor" not in text:
                continue
            for line, ident, src in scan_text(text, str(p), deref_index):
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
