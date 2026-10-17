#!/usr/bin/env python3
"""Ratchet: a fully-Rust module is linked by a cargo cell, not by cmake around a stub.

A module whose C is all translated is built and linked by Rust tooling
(docs/engineering/cargo-drive-plan.md). `rustconv port flip` leaves it as a cmake target
around a placeholder `rust_cohab_stub.c`, and nothing used to move it on: 16 modules
translated after cargo-drive wave 11 sat that way until wave 13 swept them by hand.

Every such module is a finding, labelled by what `tools/cargo_cell_gen.py` says it can do,
and so is a cargo cell whose build.rs still names an archive cmake builds:

    <target>: fully-Rust module still linked by cmake (convertible)
    <target>: fully-Rust module still linked by cmake (refused)
    <crate>: links cmake-built <archive> (engine-buildable | not engine-buildable)

The baseline holds today's REFUSED ones. A NEW module, flipped and left linked by cmake,
is a new finding; so is a refused module the generator has since learned to handle
(its label flips to `convertible`). The fix for either is one command:

    python3 tools/cargo_cell_gen.py --convert <target>

Reads build-amd64/build.ninja, so it runs in the deferred partition, after a build.
"""

from __future__ import annotations

import re
import sys
from pathlib import Path

sys.path.insert(0, str(Path(__file__).resolve().parent))

import cargo_cell_gen


def _source_facts(repo: Path) -> tuple[set[str], set[str]]:
    """(targets handed to add_cargo_module, targets whose defining file has a stub).

    The survey reads build.ninja, which only a configure refreshes, so it can describe a
    tree that no longer exists in either direction: a module converted since (its cmake
    edge is still there) or one whose flip is not in this tree at all (an integration
    batch re-created without the lane that flipped it: blue.c and font.c compiled again,
    while the graph still had blue's stub-only link). A finding needs the source to agree:
    not a cargo cell, and defined in a cmake file that still compiles rust_cohab_stub.
    """
    cells: set[str] = set()
    stubbed: set[str] = set()
    for cm in [*repo.glob("**/CMakeLists.txt"), *repo.glob("**/*.cmake")]:
        if cm.parts[len(repo.parts)].startswith(("build-", ".")):
            continue
        text = cm.read_text(errors="replace")
        if "add_cargo_module" in text:
            cells.update(re.findall(r"add_cargo_module\(\s*([\w.+-]+)", text))
        if "rust_cohab_stub" in text:
            stubbed.update(re.findall(r"add_(?:library|executable)\(\s*([\w.+-]+)", text))
    return cells, stubbed


def main() -> int:
    if not (cargo_cell_gen.REPO / "build-amd64/build.ninja").is_file():
        print("FATAL: no build-amd64/build.ninja; fix: ./vibeos-cli build --arch amd64")
        return 2
    cells, stubbed = _source_facts(cargo_cell_gen.REPO)
    rows = [(t, v) for t, v in cargo_cell_gen.survey() if t not in cells and t in stubbed]
    for target, verdict in rows:
        label = "convertible" if verdict.startswith("OK") else "refused"
        print(f"{target}: fully-Rust module still linked by cmake ({label})")
    # A cargo cell that still links an archive cmake builds is still cmake-bound. The
    # engine builds any cmake C static library and any rust_staticlib_variant archive
    # (`cargo_cell_gen.engine_lib`); `--remap` moves a cell onto them.
    raw = []
    for build_rs in sorted((cargo_cell_gen.REPO / "sdk/lib/rust").glob("*-cargo/build.rs")):
        # Comments name archives a cell USED to link (the loader cells keep the cmake
        # path beside the engine call that replaced it); only live code counts.
        code = re.sub(r"//[^\n]*", "", build_rs.read_text())
        for lib in sorted(set(re.findall(r'"([^"]+\.a)"', code))):
            label = "engine-buildable" if cargo_cell_gen.engine_lib(lib) else "not engine-buildable"
            raw.append(f"{build_rs.parent.name}: links cmake-built {lib} ({label})")
    for line in raw:
        print(line)
    if rows or raw:
        print(
            "fix: python3 tools/cargo_cell_gen.py --convert <target>, or --remap <crate> "
            "for a cell on a cmake-built archive (refused: see --survey for why)",
            file=sys.stderr,
        )
    return 1 if rows or raw else 0


if __name__ == "__main__":
    sys.exit(main())
