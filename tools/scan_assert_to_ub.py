#!/usr/bin/env python3
"""
scan_assert_to_ub.py — keep C assertions from being lowered to Rust UB.

C's diagnostic macros are NOT control flow:

    PAGED_CODE()        xdk/kefuncs.h  — KdPrint + ASSERT under DBG, NOP_FUNCTION in retail
    ASSERT(exp)         ndk/rtlfuncs.h — RtlAssert(...) under DBG, ((void)0) in retail
    NT_ASSERT(exp)      likewise

None of them is `__assume(0)`. But `core::hint::unreachable_unchecked()` IS —
it is undefined behaviour, and telling LLVM a branch is unreachable licenses it
to delete the guarding check and poison everything downstream. So translating an
assert into `unreachable_unchecked()` is strictly worse than the C in EVERY
build configuration: retail C does nothing, retail Rust invokes UB.

This is not hypothetical. Found 2026-07-25 in a fresh sweep:

  * cdrom-rust/src/common.rs   PAGED_CODE()  -> unreachable_unchecked()
  * fastfat-rust/src/strucsup.rs  NT_ASSERT(Buffer != NULL) -> unreachable_unchecked()
  * classpnp-rust/src/class.rs PAGED_CODE()  -> KeBugCheckEx(0x35u32, ...)   (see
    scan_bugcheck_codes.py — an INVENTED stop code that additionally evaded that
    gate because of a Rust type suffix)

and the class had already recurred once before: fastfat-rust/src/fatinit.rs
carries a comment describing the same mistake being fixed in `__int2c`, which is
precisely why a one-off fix was not enough and this gate exists.

`unreachable_unchecked()` IS legitimate as the tail of a genuinely-noreturn call
(after KeBugCheckEx, a raise helper, or a jump into another image's entry point),
and as a faithful translation of C's `DEFAULT_UNREACHABLE`. Mark those:

    core::hint::unreachable_unchecked()  // nolint: assert-to-ub (after FatBugCheck — noreturn)

Fix for a real hit: emit the C diagnostic instead (a DbgPrint mirroring the DBG
expansion) and let control flow continue, or return an error. Never assert by
assumption.

Usage:
    python3 tools/scan_assert_to_ub.py            # report
    python3 tools/scan_assert_to_ub.py --lint     # terse (CLI baseline gate)

Exit 1 if any finding.
"""

import re
import sys
from pathlib import Path

REPO = Path(__file__).resolve().parent.parent
RUST_ROOT = REPO / "sdk" / "lib" / "rust"

SKIP_FILE_RE = re.compile(r"(bindgen_[a-z0-9_]+\.rs$|/target/|/\.responses/|/tests?/)")
NOLINT_RE = re.compile(r"//\s*nolint(?::\s*assert-to-ub\b|\s*$|:?\s*\()")

HIT_RE = re.compile(r"\bunreachable_unchecked\s*\(")

# Second lowering of the same class, found 2026-07-26: the assert became a `panic!`.
# In kernel mode that is the WORST of the three. The #[panic_handler] prints to debugcon
# and then loops forever, so it is a silent hang -- no bugcheck, no dump, no marker --
# where retail C does nothing at all and DBG C calls RtlAssert and RETURNS. 19 exported
# functions had it across cdrom/classpnp/fastfat/pcix, and this gate reported a clean
# zero for all of them because it only ever looked for UB.
#
# Keyed on ASSERT ORIGIN, not on the macro alone: a bare `unreachable!()` is often
# legitimate (freeldr guards x86 port I/O that genuinely cannot run on arm64), so
# flagging every hang macro would be noise. What makes it a defect is that a C
# diagnostic -- which continues -- was turned into one that does not.
PANIC_RE = re.compile(r"\b(?:core::)?(?:panic|unreachable|todo|unimplemented)!\s*\(")
# CASE-SENSITIVE on purpose. The C macros are uppercase, and matching case-insensitively
# made the word "assert" in ordinary English prose an origin signal -- a comment reading
# "a hang macro with no assert origin is legitimate" three lines above an unrelated
# `unreachable!()` was enough to flag it. The lowercase forms that DO matter are spelled
# out (`assert:` with its colon, `Assertion`), so nothing real is lost.
ASSERT_ORIGIN_RE = re.compile(
    r"(NT_ASSERT|PAGED_CODE|Pageable code|IRQL too high for paged|ASSERT|assert:|Assertion)"
)

# Preceding-line shapes that make the call a legitimate noreturn tail rather
# than an assertion: a bugcheck/raise/entry-point call, or an explicit
# DEFAULT_UNREACHABLE translation note. NOTE the names are matched as
# SUBSTRINGS, not \b-anchored words — the real call sites are driver-prefixed
# (`FatBugCheck`, `CdRaiseStatus`), and a leading \b rejects exactly those.
NORETURN_CTX_RE = re.compile(
    r"(BugCheck|RaiseStatus|RaiseException|KiSystemStartup|DEFAULT_UNREACHABLE)",
    re.IGNORECASE,
)


# Blank string literals (keeping their newlines, so line numbers stay aligned) before
# looking for the MACRO. Required, not cosmetic: ReactOS's UNIMPLEMENTED macro expands to
# DbgPrint("WARNING: %s at %s:%d is unimplemented!()\n", ...) -- the macro name appears as
# TEXT inside a format string, and pcix/fdo.rs was flagged twice for a DbgPrint that
# neither panics nor hangs. The ORIGIN test still reads the raw line, because the evidence
# there ("ASSERT failed", "NT_ASSERT: ...") lives inside the panic message itself.
_STR_RE = re.compile(r'b?"(?:\\.|[^"\\])*"', re.S)


def _blank_strings(text: str) -> str:
    return _STR_RE.sub(lambda m: '""' + "\n" * m.group(0).count("\n"), text)


def scan_file(path: Path) -> list[tuple[int, str, str]]:
    """[(line_no, kind, source_line)] for assert lowerings that are worse than the C."""
    try:
        raw = path.read_text(errors="replace")
    except OSError:
        return []
    lines = raw.split("\n")
    code = _blank_strings(raw).split("\n")
    out = []
    for i, line in enumerate(lines):
        stripped = line.strip()
        if stripped.startswith("//") or stripped.startswith("*"):
            continue
        if NOLINT_RE.search(line):
            continue
        ctx = "\n".join(lines[max(0, i - 3) : i])
        if HIT_RE.search(code[i]):
            # Look back a few lines for a noreturn call this is tailing.
            if NORETURN_CTX_RE.search(ctx):
                continue
            out.append((i + 1, "ub", stripped[:110]))
        elif PANIC_RE.search(code[i]) and ASSERT_ORIGIN_RE.search(line + "\n" + ctx):
            out.append((i + 1, "hang", stripped[:110]))
    return out


def main() -> int:
    args = [a for a in sys.argv[1:] if not a.startswith("-")]
    lint = "--lint" in sys.argv
    roots = [Path(a) for a in args] if args else [RUST_ROOT]

    files: list[Path] = []
    for root in roots:
        if root.is_file():
            files.append(root)
        elif root.is_dir():
            files.extend(root.rglob("*.rs"))

    findings = []
    for f in sorted(set(files)):
        if SKIP_FILE_RE.search(str(f)):
            continue
        for ln, kind, src in scan_file(f):
            try:
                rel = f.relative_to(REPO)
            except ValueError:
                rel = f
            findings.append((str(rel), ln, kind, src))

    WHY = {
        "ub": "unreachable_unchecked() is UB — C asserts are not __assume(0)",
        "hang": "panic! from an assert is a SILENT HANG in kernel mode — C's ASSERT returns",
    }
    for rel, ln, kind, src in findings:
        if lint:
            print(f"{rel}:{ln}: assert-to-{kind}")
        else:
            print(f"{rel}:{ln}: {WHY[kind]}")
            print(f"    {src}")

    if not lint:
        n_ub = sum(1 for f in findings if f[2] == "ub")
        n_hang = len(findings) - n_ub
        print(f"\n{len(findings)} site(s): {n_ub} lowered to UB, {n_hang} lowered to a hang")
        if n_ub:
            print("fix (ub):   emit the C diagnostic (DbgPrint) and continue, or return an error;")
            print("            if it really tails a noreturn call: // nolint: assert-to-ub (reason)")
        if n_hang:
            print("fix (hang): call RtlAssert(cond, file, line!(), msg) — what C's ASSERT calls")
            print("            under DBG. It prints loudly and RETURNS, so control flow matches C.")
    return 1 if findings else 0


if __name__ == "__main__":
    sys.exit(main())
