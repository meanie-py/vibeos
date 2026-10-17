#!/usr/bin/env python3
"""Ratchet: a fully-Rust module is linked by a cargo cell, not by cmake around a stub.

A module whose C is all translated is built and linked by Rust tooling
(docs/engineering/cargo-drive-plan.md). `rustconv port flip` leaves it as a cmake target
around a placeholder `rust_cohab_stub.c`, and nothing used to move it on: 16 modules
translated after cargo-drive wave 11 sat that way until wave 13 swept them by hand.

Every such module is a finding, labelled by what `tools/cargo_cell_gen.py` says it can do:

    <target>: fully-Rust module still linked by cmake (convertible)
    <target>: fully-Rust module still linked by cmake (refused)

The baseline holds today's REFUSED ones. A NEW module, flipped and left linked by cmake,
is a new finding; so is a refused module the generator has since learned to handle
(its label flips to `convertible`). The fix for either is one command:

    python3 tools/cargo_cell_gen.py --convert <target>

Reads build-amd64/build.ninja, so it runs in the deferred partition, after a build.
"""

from __future__ import annotations

import sys
from pathlib import Path

sys.path.insert(0, str(Path(__file__).resolve().parent))

import cargo_cell_gen


def main() -> int:
    if not (cargo_cell_gen.REPO / "build-amd64/build.ninja").is_file():
        print("FATAL: no build-amd64/build.ninja; fix: ./vibeos-cli build --arch amd64")
        return 2
    rows = cargo_cell_gen.survey()
    for target, verdict in rows:
        label = "convertible" if verdict.startswith("OK") else "refused"
        print(f"{target}: fully-Rust module still linked by cmake ({label})")
    if rows:
        print(
            "fix: python3 tools/cargo_cell_gen.py --convert <target>  "
            "(refused: see --survey for why)",
            file=sys.stderr,
        )
    return 1 if rows else 0


if __name__ == "__main__":
    sys.exit(main())
