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

# Preceding-line shapes that make the call a legitimate noreturn tail rather
# than an assertion: a bugcheck/raise/entry-point call, or an explicit
# DEFAULT_UNREACHABLE translation note. NOTE the names are matched as
# SUBSTRINGS, not \b-anchored words — the real call sites are driver-prefixed
# (`FatBugCheck`, `CdRaiseStatus`), and a leading \b rejects exactly those.
NORETURN_CTX_RE = re.compile(
    r"(BugCheck|RaiseStatus|RaiseException|KiSystemStartup|DEFAULT_UNREACHABLE)",
    re.IGNORECASE,
)


def scan_file(path: Path) -> list[tuple[int, str]]:
    """[(line_no, source_line)] for unreachable_unchecked that is not a noreturn tail."""
    try:
        lines = path.read_text(errors="replace").splitlines()
    except OSError:
        return []
    out = []
    for i, line in enumerate(lines):
        stripped = line.strip()
        if stripped.startswith("//") or stripped.startswith("*"):
            continue
        if not HIT_RE.search(line):
            continue
        if NOLINT_RE.search(line):
            continue
        # Look back a few lines for a noreturn call this is tailing.
        ctx = "\n".join(lines[max(0, i - 3) : i])
        if NORETURN_CTX_RE.search(ctx):
            continue
        out.append((i + 1, stripped[:110]))
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
        for ln, src in scan_file(f):
            try:
                rel = f.relative_to(REPO)
            except ValueError:
                rel = f
            findings.append((str(rel), ln, src))

    for rel, ln, src in findings:
        if lint:
            print(f"{rel}:{ln}: assert-to-ub")
        else:
            print(f"{rel}:{ln}: unreachable_unchecked() is UB — C asserts are not __assume(0)")
            print(f"    {src}")

    if not lint:
        print(f"\n{len(findings)} assert-lowered-to-UB site(s)")
        if findings:
            print("fix: emit the C diagnostic (DbgPrint) and continue, or return an error;")
            print("     if it really tails a noreturn call: // nolint: assert-to-ub (reason)")
    return 1 if findings else 0


if __name__ == "__main__":
    sys.exit(main())
