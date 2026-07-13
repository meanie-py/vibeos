#!/usr/bin/env python3
"""scan_nightly_surface.py — the stable-toolchain RATCHET.

The tree builds on **pinned stable rustc**. Exactly ONE cell is allowed to need
nightly: the i386 kernel, which uses `-Z build-std` + `i686-kernel.json` because it
needs `rustc-abi: x86-softfloat` (no builtin triple offers it). That cell is selected
in `gcc.cmake:add_cargo_module` via `RUSTUP_TOOLCHAIN` from `.rust-nightly`, and it
retires with the wow64 lane.

Everything else must compile on stable. This lint fails on the two ways nightly creeps
back in:

  1. `#![feature(...)]` anywhere in the Rust workspace.
  2. `RUSTC_BOOTSTRAP=1` in the cmake build graph — the stable-rustc escape hatch that
     silently unlocks nightly features.

WHY A RATCHET AND NOT JUST A CLEANUP: the stable-toolchain lane drove both to zero on
2026-07-12, and a concurrent lane re-introduced BOTH the same day (oleaut32-rust:
`#![feature(c_variadic)]` for no-op `TRACE`/`WARN` debug shims + `RUSTC_BOOTSTRAP=1` in
its CMakeLists). Nothing caught it. Now something does.

If you hit this: you almost certainly do NOT need the feature.
  * **c_variadic** — you only need it to *DEFINE* a C-variadic fn. DECLARING/calling one
    is fine on stable. No-op debug shims (`TRACE(fmt, ...)`) should be `macro_rules!` or
    plain single-arg fns — see `sdk/lib/rust/ole32-rust/src/debug_shims.rs`. If you truly
    must define one, use the C `va_start` trampoline pattern in
    `sdk/lib/rust/varargs-rust/` (a `Name_v` fixed-arity twin + a 3-line C shim).
  * **linkage** — perma-unstable upstream; it will never stabilise. Restructure.
  * A no-op out-of-line fn is also a CLOAK (field manual §9.65 / the cdrom TracePrint
    case): it becomes an optimization barrier and can uncloak latent ABI bugs. Macros are
    both the faithful translation of a C macro AND the safe one.

Usage:
    python3 tools/scan_nightly_surface.py          # one line per finding; exit 0
    python3 tools/scan_nightly_surface.py --check  # exit 1 if any finding
"""

from __future__ import annotations

import argparse
import os
import re
import sys
from pathlib import Path

_FEATURE = re.compile(r"^\s*#!\[feature\(")
_BOOTSTRAP = re.compile(r"RUSTC_BOOTSTRAP")

# Directories that are BUILD OUTPUT, never source. Pruned during the walk.
_PRUNE = {"target", ".git", ".build-tools"}


def repo_root() -> Path:
    return Path(__file__).resolve().parent.parent


def _walk(root: Path, suffixes: tuple[str, ...]):
    """Yield every file under `root` with one of `suffixes`, PRUNING build output.

    Why not `root.rglob("*.rs")` with a `"/target/" in str(p)` filter (which is what
    this did, and which crashed a gate)?

    Because the filter runs on the RESULTS — rglob has already descended into
    `target/` by then. That is (a) needlessly slow, target/ being an enormous tree of
    build artifacts, and (b) RACY: `vibeos-cli gate` runs the per-arch lints
    CONCURRENTLY with the other arches' builds, so a `target/<triple>` directory can
    be created or removed while this walk is inside it. `rglob` then raises

        FileNotFoundError: '.../sdk/lib/rust/target/i686-kernel'

    and the whole lint stage dies — reported as "NEW lint findings", which is exactly
    the wrong diagnosis. (Observed 2026-07-13; a gate failed on lint:amd64 while
    lint:i386 and lint:arm64 passed on the identical tree.)

    os.walk lets us prune the directory list IN PLACE before descending, so we never
    enter build output at all, and it swallows scandir errors by default rather than
    raising — both properties this scanner needs to be safe under a parallel gate.
    """
    for dirpath, dirnames, filenames in os.walk(root):
        dirnames[:] = [d for d in dirnames if d not in _PRUNE and not d.startswith("build-")]
        for fn in sorted(filenames):
            if fn.endswith(suffixes):
                yield Path(dirpath) / fn


def findings(repo: Path) -> list[str]:
    out: list[str] = []

    ws = repo / "sdk" / "lib" / "rust"
    for rs in sorted(_walk(ws, (".rs",))):
        try:
            lines = rs.read_text(errors="replace").splitlines()
        except OSError:
            continue
        for n, line in enumerate(lines, 1):
            if _FEATURE.match(line):
                rel = rs.relative_to(repo)
                out.append(f"{rel}:{n}: [NIGHTLY] {line.strip()} — the tree is on pinned stable")

    for cm in sorted(_walk(repo, ("CMakeLists.txt", ".cmake"))):
        try:
            lines = cm.read_text(errors="replace").splitlines()
        except OSError:
            continue
        for n, line in enumerate(lines, 1):
            # Skip comments — several CMakeLists mention it only to say it's gone.
            if line.lstrip().startswith("#"):
                continue
            if _BOOTSTRAP.search(line):
                rel = cm.relative_to(repo)
                out.append(f"{rel}:{n}: [NIGHTLY] RUSTC_BOOTSTRAP — stable-rustc escape hatch")
    return out


def main() -> int:
    ap = argparse.ArgumentParser(description=__doc__.split("\n")[0])
    ap.add_argument("--check", action="store_true", help="exit 1 if any finding")
    args = ap.parse_args()

    repo = repo_root()
    hits = findings(repo)
    for h in hits:
        print(h)
    if hits and args.check:
        print(
            f"\n{len(hits)} nightly-surface finding(s). The tree builds on PINNED STABLE; "
            "the i386\nkernel's `-Z build-std` cell is the only sanctioned nightly user "
            "(gcc.cmake).\nSee this file's docstring — you almost certainly don't need the "
            "feature.",
            file=sys.stderr,
        )
        return 1
    return 0


if __name__ == "__main__":
    sys.exit(main())
