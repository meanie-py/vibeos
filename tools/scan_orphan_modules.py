#!/usr/bin/env python3
"""
scan_orphan_modules.py — translated Rust that NOTHING COMPILES.

THE BUG SHAPE:
A `.rs` file that no `mod` chain reaches from any crate root is not part of the
crate. cargo never opens it. So:

  * `cargo check` is green no matter what is in it — including code that does not
    compile. classpnp's `class_gap3.rs` was edited to fix a link error; the edits
    checked clean and the link still failed, because the file was never built.
  * every source-level gate in this repo silently skips it. `verify_match_consts`
    literally suppresses its own findings there, on the correct reasoning that a
    catch-all arm in dead code cannot execute.
  * `grep` still finds it. So the next agent reads a dead copy as the truth, or
    "fixes" a bug in the copy that does not run. Every orphan is a decoy.

It is also paid artifact: these are AI translations that cost real money, sitting
where nothing links them.

THE RULE:
An orphan is not automatically wrong — parking a translation is a legitimate move
(dhcpcsvc keeps `dhcp/tree.rs`, a translation of upstream-dead source; rtlk keeps
`path.rs`; ntoskrnl's `global_barrier.rs` was retired by decision on 2026-07-07).
What is wrong is an orphan nobody can tell is deliberate. So the gate is not
"no orphans", it is **no UNDECLARED orphans**: put the reason in the file, at the
top, where the person reading it will see it:

    //! ORPHAN-BY-DESIGN: <why this is not in the module tree, and what owns the
    //! symbols instead>

Baseline is EMPTY = strict. A new unexplained orphan fails the gate.

WHAT IS NOT AN ORPHAN:
  build.rs / tests/ / benches/ / examples/ (cargo compiles these by their own
  rules), files pulled in by `include!`, and every file of a crate whose root does
  `include!(env!(...))` — that names a build.rs output no source-level walk can
  resolve, so the whole crate fails open.
"""

import argparse
import os
import sys

sys.path.insert(0, os.path.dirname(os.path.abspath(__file__)))
from rust_module_tree import (
    crate_is_opaque,
    crate_live_files,
    crate_phantom_mods,
    crate_root_of,
)

BANNER = "ORPHAN-BY-DESIGN:"
BANNER_SCAN_LINES = 40

# cargo compiles these outside the lib/bin module tree.
SKIP_DIRS = {"tests", "benches", "examples", "target", ".git"}
SKIP_NAMES = {"build.rs"}


def banner_reason(path):
    """The declared reason, or None. Read only the head — a banner buried at line
    900 does not do the job the banner exists for."""
    try:
        with open(path, encoding="utf-8", errors="replace") as fh:
            for _ in range(BANNER_SCAN_LINES):
                line = fh.readline()
                if not line:
                    break
                if BANNER in line:
                    return line.split(BANNER, 1)[1].strip() or "(no reason given)"
    except OSError:
        return None
    return None


def has_crate_root(crate_dir):
    """Does this Cargo.toml directory actually carry a crate root? A workspace-only
    manifest does not, and neither does a translation dump that was never scaffolded."""
    src = os.path.join(crate_dir, "src")
    return any(
        os.path.exists(os.path.join(d, f)) for d in (src, crate_dir) for f in ("lib.rs", "main.rs")
    )


def scan_phantoms(roots):
    """The DUAL finding: `mod X;` declared against a file that does not exist.

    An orphan is a file nothing compiles; a phantom is a declaration with nothing behind it.
    Same module tree, opposite direction, and the phantom is the one no gate could see: a
    crate parked out of `workspace.members` is never compiled, and no source scanner opens a
    path that is absent. desk shipped `pub mod guid;` with no guid.rs AND lib.rs claiming 20
    TUs when 17 had been translated — through a full green `check` and `check-tooling`, twice.

    There is no banner escape here on purpose. An orphan can be deliberately parked; a `mod`
    pointing at nothing is never deliberate — it cannot compile, so the only question is
    whether anyone has tried to compile it yet."""
    out, seen = [], set()
    for root in roots:
        for dirpath, dirnames, filenames in os.walk(root):
            dirnames[:] = [d for d in dirnames if d not in SKIP_DIRS]
            if "Cargo.toml" not in filenames or dirpath in seen:
                continue
            seen.add(dirpath)
            if not has_crate_root(dirpath) or crate_is_opaque(dirpath):
                continue
            out.extend(crate_phantom_mods(dirpath))
    return out


def scan(roots):
    findings, declared, rootless = [], [], []
    for root in roots:
        for dirpath, dirnames, filenames in os.walk(root):
            dirnames[:] = [d for d in dirnames if d not in SKIP_DIRS]
            for fn in sorted(filenames):
                if not fn.endswith(".rs") or fn in SKIP_NAMES:
                    continue
                path = os.path.join(dirpath, fn)
                crate = crate_root_of(path)
                if crate is None or not has_crate_root(crate):
                    # `crate_root_of` returns the nearest ancestor holding a Cargo.toml.
                    # A directory of .rs with no Cargo.toml of its own therefore resolves
                    # to the WORKSPACE manifest, whose crate_live_files() is empty, and
                    # the "unparseable -> fail open" branch below then swallowed it. That
                    # made the most complete kind of orphan the one kind this gate could
                    # not see: mscms-rust is 6 files / 4,782 lines of paid translation
                    # with no lib.rs and no Cargo.toml, and it reported clean for as long
                    # as this gate has existed. Distinguish "crate root exists but we
                    # could not parse it" (still fail open, below) from "there is no crate
                    # root at all" (report). Same banner escape as any other orphan.
                    reason = banner_reason(path)
                    (declared if reason else rootless).append((path, reason))
                    continue
                if crate_is_opaque(crate):
                    continue
                live = crate_live_files(crate)
                if not live:  # unparseable crate -> fail open
                    continue
                if os.path.realpath(path) in {os.path.realpath(x) for x in live}:
                    continue
                reason = banner_reason(path)
                (declared if reason else findings).append((path, reason))
    return findings, declared, rootless


def main():
    ap = argparse.ArgumentParser(
        description=__doc__, formatter_class=argparse.RawDescriptionHelpFormatter
    )
    ap.add_argument("paths", nargs="*", help="dirs to scan (default: sdk/lib/rust)")
    ap.add_argument(
        "--show-declared",
        action="store_true",
        help="also list orphans that DO carry a banner (the accepted set)",
    )
    args = ap.parse_args()

    repo = os.path.dirname(os.path.dirname(os.path.abspath(__file__)))
    roots = args.paths or [os.path.join(repo, "sdk", "lib", "rust")]
    findings, declared, rootless = scan(roots)

    for path, _ in findings:
        rel = os.path.relpath(path, repo)
        with open(path, encoding="utf-8", errors="replace") as fh:
            n = sum(1 for _ in fh)
        print(
            f"{rel}:1: [ORPHAN] no `mod` chain reaches this file — {n} lines that "
            f"nothing compiles; wire it up, or declare it with a "
            f"`//! {BANNER} <reason>` banner"
        )
    # Deliberately NO per-file line count in this message. A rootless crate is usually one
    # somebody is actively integrating, so its line counts move every commit — and a count
    # inside the finding text is part of the baseline key, so each edit would read as a NEW
    # finding and bounce their gate. The total goes in the SUMMARY, which is not baselined.
    rootless_lines = 0
    for path, _ in rootless:
        with open(path, encoding="utf-8", errors="replace") as fh:
            rootless_lines += sum(1 for _ in fh)
        print(
            f"{os.path.relpath(path, repo)}:1: [ROOTLESS] no crate root (lib.rs/main.rs) "
            f"above this file — nothing compiles it; scaffold the crate, or declare it "
            f"with a `//! {BANNER} <reason>` banner"
        )
    phantoms = scan_phantoms(roots)
    for decl, name, cands in phantoms:
        where = " | ".join(os.path.relpath(c, repo) for c in cands)
        print(
            f"{os.path.relpath(decl, repo)}:1: [PHANTOM] `mod {name};` resolves to no file "
            f"(looked for {where}) — write it, or drop the declaration"
        )

    if args.show_declared:
        for path, reason in declared:
            print(f"{os.path.relpath(path, repo)}:1: [DECLARED] {reason}")

    print(
        f"\nSUMMARY: undeclared-orphans={len(findings)}  phantom-modules={len(phantoms)}  "
        f"rootless={len(rootless)} ({rootless_lines} lines)  declared={len(declared)}"
    )
    return 1 if (findings or rootless or phantoms) else 0


if __name__ == "__main__":
    sys.exit(main())
