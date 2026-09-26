#!/usr/bin/env python3
"""
scan_dropped_seh_pairs.py — the C caught the fault in a file the Rust replaced,
and the Rust kept the raise without the catch.

THE POINT, and why `scan_dropped_probes` cannot see this. That scanner reads the
commented-out C that rustconv leaves inline in the same `.rs`, so it needs the C
and the Rust to share a file. A COHAB FLIP does the opposite: the `.c` stays on
disk, gets commented out of the CMake source list, and a hand-written or
separately-translated `.rs` takes over under `#[no_mangle]`. Nothing then links
the two files, and `lpc-rust` is the proof — it carries zero `_SEH2` text and
zero inline C, so the inline scanner reports nothing about it while
`NtReplyPort` sits there dereferencing a user pointer with no handler.

THE BUG CLASS. Upstream wraps a user-pointer probe and its capture in
`_SEH2_TRY { ... } _SEH2_EXCEPT(EXCEPTION_EXECUTE_HANDLER) { return
_SEH2_GetExceptionCode(); }`. We build with DUMMY_PSEH, so a translation that
keeps the `ProbeForRead` and drops the handler has no catch at all: an
unprivileged caller passing a bad pointer to a syscall entry point bugchecks the
box. The sanctioned shape is validate-first — `ntkrnl_sys::probe::*` returning
`Result<(), NTSTATUS>` (`docs/engineering/dummy-pseh-verdict.md`), worked through
in `ntoskrnl-rust/src/ARM3/virtual.rs`.

WHY THE OBVIOUS SWEEP DOES NOT WORK, recorded because it was tried. The research
lane proposed "an unreachable error path is the fingerprint of a deleted
handler" — a flag set, reset, then tested. Implemented, that finds ONLY
`virtual.rs`, because the fingerprint exists solely where the C happened to use a
flag; a dropped handler guarding a plain `return` leaves no trace whatsoever,
which is exactly the LPC shape. A naive version also produced 8 false positives
by reading `X = 1` in an `if` arm and `X = 0` in the `else` arm as set-then-reset.

WHAT DOES GENERALISE, and is what this implements: pair each C file containing
`_SEH2_EXCEPT` with its translated `.rs`, then flag the pair when the Rust can
RAISE and has no CATCH idiom anywhere in it.

TWO FILTERS THAT TURN 19 NOISY HITS INTO REAL LEADS.

  1. **Is the C actually out of the build?** If the `.c` is still compiled, it —
     not the `.rs` — may be what ships, and the Rust may be parked or dead. This
     parses the CMake source lists and reports the flip state, so a finding
     against live C is separated from a finding against replaced C rather than
     mixed in with it. This is the filter the shell one-liner never had.
  2. **Same-subsystem pairing.** Basename matching alone pairs
     `fastfat/create.c` with `lpc-rust/create.rs`, which is nonsense. A pair
     whose C parent directory is echoed in the Rust crate name is a real lead;
     everything else is reported separately and needs a human to confirm.

Exit 1 when a high-confidence finding exists (flipped-out C, same-subsystem
pair, Rust raises with no catch). `--lint` compares against a baseline file so
the gate can demand zero NEW.
"""

from __future__ import annotations

import argparse
import re
import sys
from pathlib import Path

sys.path.insert(0, str(Path(__file__).resolve().parent))
from source_walk import source_files

REPO = Path(__file__).resolve().parent.parent
ABSOLUTE_PATHS = False

# Calls that can raise into nothing under DUMMY_PSEH.
RAISERS = (
    "ProbeForRead",
    "ProbeForWrite",
    "ExRaiseStatus",
    "ExRaiseAccessViolation",
    "ExRaiseDatatypeMisalignment",
)

# Any of these in the Rust means somebody thought about faults here. Kept wide
# on purpose: demanding one spelling of a correct thing is how a scanner starts
# lying about code that is already right.
CATCHERS = (
    "probe::",
    "probe_user_buffer",
    "safe_copy_raw",
    "safe_copy_to_user",
    "safe_copy_from_user",
    "Result<(), NTSTATUS>",
    "catch_unwind",
    "_SEH2",
    "try_probe",
)

# A cohab flip sometimes DELETES the .c instead of commenting it out, and then
# the oracle is gone from the working tree entirely -- `--deleted` recovers it
# from git history instead. That case is invisible to the pairing above and is
# where the biggest find so far came from: win32k's ntuser/*.c were deleted on
# flip, so basename pairing silently matched user32's unrelated same-named
# files and reported nothing about the kernel side.
_CMAKE_COMMENTED_C = re.compile(r"^\s*#.*?([A-Za-z0-9_./-]+\.c)\s*$", re.M)
_CMAKE_LIVE_C = re.compile(r"^\s*(?!#)\S*?([A-Za-z0-9_./-]+\.c)\s*$", re.M)


def cmake_flip_state() -> tuple[set[str], set[str]]:
    """(commented-out .c basenames+parent, still-compiled ones) across all CMake.

    Keyed by "<parent>/<name>.c" so `lpc/reply.c` cannot be confused with some
    other tree's `reply.c`.
    """
    out_, live = set(), set()
    for pat in ("*.cmake", "CMakeLists.txt"):
        for f in source_files(REPO, pat):
            try:
                text = f.read_text(errors="ignore")
            except OSError:
                continue
            for m in _CMAKE_COMMENTED_C.finditer(text):
                out_.add("/".join(Path(m.group(1)).parts[-2:]))
            for m in _CMAKE_LIVE_C.finditer(text):
                live.add("/".join(Path(m.group(1)).parts[-2:]))
    return out_, live


def main() -> int:
    ap = argparse.ArgumentParser()
    ap.add_argument("--lint", action="store_true", help="compare against the baseline")
    ap.add_argument("--baseline", default="tools/lint-baseline/dropped_seh_pairs.txt")
    # A root the caller can override, so the scanner is testable against a
    # fixture tree instead of only against the one repo it lives in. A scanner
    # that can only run on the real tree cannot have a control, and the control
    # is what proves it still MATCHES -- silent blindness is the failure mode
    # these tools exist to prevent, and they are not exempt from it.
    ap.add_argument(
        "--deleted",
        metavar="CRATE:CDIR",
        help="also check Rust whose C was DELETED by a cohab flip, recovering it "
        "from git history. e.g. --deleted win32k-rust:win32ss/user/ntuser . Opt-in "
        "because it shells out to git once per file; the gate stays fast without it.",
    )
    ap.add_argument("root", nargs="?", default=None)
    args = ap.parse_args()

    global REPO, ABSOLUTE_PATHS
    if args.root:
        REPO = Path(args.root).resolve()
        # Paths are printed RELATIVE to the repo normally, because the gate
        # diffs this output against a committed baseline and an absolute path
        # differs per worktree. When the caller names its own root -- a fixture
        # tree under pytest -- relative paths are useless to it, so print
        # absolute ones instead.
        ABSOLUTE_PATHS = True

    flipped, live = cmake_flip_state()

    # Every .rs that can raise, indexed by basename.
    rust_by_stem: dict[str, list[Path]] = {}
    for rs in source_files(REPO / "sdk" / "lib" / "rust", "*.rs"):
        rust_by_stem.setdefault(rs.stem, []).append(rs)

    findings: list[str] = []
    unpaired: list[str] = []

    for c in source_files(REPO, "*.c"):
        try:
            ctext = c.read_text(errors="ignore")
        except OSError:
            continue
        if "_SEH2_EXCEPT" not in ctext:
            continue
        key = "/".join(c.parts[-2:])
        for rs in rust_by_stem.get(c.stem, []):
            rtext = rs.read_text(errors="ignore")
            raises = [r for r in RAISERS if r in rtext]
            if not raises:
                continue
            if any(k in rtext for k in CATCHERS):
                continue
            # Same subsystem? Match the C's parent directory against the Rust
            # CRATE name, or against the .rs's own parent directory -- NOT
            # against the whole path. Matching the path degenerates into
            # basename matching whenever a C file is named after its own
            # directory: `kernel32/client/console/console.c` scored
            # "same subsystem" as `win32k-rust/src/console.rs` because the word
            # "console" appears in both, which is precisely the mis-pairing this
            # filter exists to reject. Caught by reading the output, not by a
            # test -- a heuristic that grades itself on a substring will always
            # find one.
            subsys = c.parent.name.lower()
            try:
                crate = rs.relative_to(REPO / "sdk" / "lib" / "rust").parts[0].lower()
            except ValueError:
                crate = ""
            same = bool(subsys) and (subsys in crate or subsys == rs.parent.name.lower())
            rel_c = c if ABSOLUTE_PATHS else c.relative_to(REPO)
            rel_r = rs if ABSOLUTE_PATHS else rs.relative_to(REPO)
            state = "C-FLIPPED-OUT" if key in flipped and key not in live else "C-STILL-BUILT"
            line = (
                f"{rel_r}: raises {','.join(sorted(raises))} with no catch "
                f"— C at {rel_c} has _SEH2_EXCEPT [{state}]"
            )
            if same and state == "C-FLIPPED-OUT":
                findings.append(line)
            else:
                why = "basename-only pairing" if not same else "C still compiled"
                unpaired.append(f"{line} ({why} — confirm by hand)")

    # STDOUT IS THE FINDING CHANNEL and nothing else. `vibeos-cli check` diffs a
    # scanner's stdout against a baseline line-by-line, so a friendly summary
    # printed there counts as a finding -- this scanner's first gate run reported
    # "1 new" with zero findings, which was the "OK -- 0 known, 0 new." line
    # being graded as a defect. Summaries go to stderr.
    for f in sorted(findings):
        print(f)
    if unpaired:
        print(f"\n-- {len(unpaired)} lower-confidence pair(s), not gated --", file=sys.stderr)
        for u in sorted(unpaired):
            print(f"   {u}", file=sys.stderr)

    if args.deleted:
        import subprocess

        crate, cdir = args.deleted.split(":", 1)
        src = REPO / "sdk" / "lib" / "rust" / crate / "src"
        # QUALIFIED paths only. A bare helper name is not evidence of protection:
        # win32k-rust/class.rs defines its OWN `probe_read_generic` with different
        # semantics (range-check then return a DEFAULT, where the C macro raises),
        # and matching the bare name hid the file with the most dropped handlers
        # in the tree.
        qualified = ("ntkrnl_sys::probe::", "probe::probe_", "safe_copy_", "catch_unwind", "_SEH2")
        found = []
        # source_files(), not rglob(): a crate's target/ holds generated copies
        # of its own modules, so an unpruned walk answers from build artifacts.
        for rs in sorted(source_files(src, "*.rs")) if src.is_dir() else []:
            t = rs.read_text(errors="ignore")
            if not any(r in t for r in RAISERS):
                continue
            if any(q in t for q in qualified):
                continue
            rel_c = f"{cdir}/{rs.stem}.c"
            sha = subprocess.run(
                ["git", "log", "--diff-filter=D", "--format=%H", "-1", "--", rel_c],
                capture_output=True,
                text=True,
                cwd=REPO,
            ).stdout.strip()
            if not sha:
                continue
            old = subprocess.run(
                ["git", "show", f"{sha}^:{rel_c}"], capture_output=True, text=True, cwd=REPO
            ).stdout
            n = old.count("_SEH2_EXCEPT")
            if n:
                found.append((n, rs.relative_to(REPO), rel_c, sha[:11]))
        for n, r, c, sha in sorted(found, reverse=True):
            print(f"{r}: raises with no catch — DELETED C {c} had {n} _SEH2_EXCEPT (in {sha})")
        print(
            f"\n{len(found)} file(s), {sum(f[0] for f in found)} handlers present in the "
            f"deleted C with no counterpart in the Rust",
            file=sys.stderr,
        )
        return 1 if found else 0

    if args.lint:
        base = REPO / args.baseline
        known = set()
        if base.is_file():
            known = {ln.strip() for ln in base.read_text().splitlines() if ln.strip()}
        new = [f for f in findings if f not in known]
        if new:
            print(f"\n{len(new)} NEW dropped-SEH pair(s):", file=sys.stderr)
            for n in new:
                print(f"  {n}", file=sys.stderr)
            return 1
        print(f"OK — {len(findings)} known, 0 new.", file=sys.stderr)
        return 0

    print(
        f"\n{len(findings)} high-confidence finding(s), {len(unpaired)} to confirm by hand.",
        file=sys.stderr,
    )
    return 1 if findings else 0


if __name__ == "__main__":
    raise SystemExit(main())
