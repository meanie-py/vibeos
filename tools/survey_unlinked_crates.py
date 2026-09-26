#!/usr/bin/env python3
"""
survey_unlinked_crates.py — paid Rust that COMPILES and does not SHIP.

WHY THIS IS A TOOL AND NOT A TABLE. The question "which crates does nothing
build?" was answered once by hand, in a document, with a number. That number
rots the moment a crate is wired, and the next reader cannot tell how old it is
— which is exactly how `recyclebin-rust` came to carry a `Cargo.toml` note
reading "PARKED at 25 errors/arch" while sitting at 204. A typed table is a
claim with an invisible timestamp; a command is a claim anyone can re-date.

TWO DIFFERENT FAILURES, AND ONLY ONE OF THEM ROTS.

  * **Parked** — the crate is not a workspace member. Nobody compiles it, no
    source scanner opens a path that is absent, and its error count drifts
    silently. This is the dangerous one. Measured 2026-09-20 across the whole
    tree: 315 packages, two workspaces, ZERO phantom members, and exactly ONE
    parked crate, already rescued. The surface is closed, and the fix was
    membership rather than vigilance.
  * **Unlinked** — the crate IS a member, so `cargo check` compiles it on every
    lane and it CANNOT drift; nothing links it into a build. That is not rot,
    it is unshipped inventory, and the disposition is decided by reading the
    crate's banner, not by compiling it.

This reports both, and keeps them apart, because conflating them produces a
"rescue effort" aimed at crates that are in no danger.

WHY EACH ONE IS UNREACHED, which is the part a reader cannot reconstruct from a
name and the part that decides rescue-versus-delete:

  no-reference   its name appears in no CMake, script or Python anywhere —
                 nothing could build it even by accident
  docs-only      named only in documentation: somebody wrote about it, nothing
                 builds it. Often the fossil of a plan that was abandoned
  dep-only       something DOES depend on it, but that dependent is itself
                 unreached — transitively dead, and the dependent is the real
                 question
  self-contained a workspace root or standalone host tool; not a finding

ROOTS, and the traps in computing them. This survey over-reported TWICE before
it was right, 13 -> 6 -> 1, and both corrections are modelled below because the
next person will hit them in the same order:

  * a harness can DISCOVER its members rather than name them — `difftest.py`
    selects by directory SUFFIX (`-difftest`), so grepping for
    `portcls-difftest` finds only its own Cargo.toml (13 -> 6);
  * a `build.rs` can hand a crate name to `reactos-link`'s `.rust_lib(...)` /
    `.msvcrtex_crt(...)`, which links it with no Cargo edge and no CMake string
    (6 -> 1). Five shipping crates were reported as orphans this way.

The shape both share: a build mechanism the closure did not model reads as
absence of a build. An unreached crate is therefore a claim about THIS tool's
coverage as much as about the crate, which is why the answer below is worth
confirming by banner before anyone deletes anything.
"""

from __future__ import annotations

import argparse
import re
import sys
from pathlib import Path

sys.path.insert(0, str(Path(__file__).resolve().parent))
from source_walk import source_files

REPO = Path(__file__).resolve().parent.parent
DIFFTEST_SUFFIX = "-difftest"


def _strip_comments(text: str) -> str:
    return "\n".join(ln.split("#", 1)[0] for ln in text.splitlines())


def scan(repo: Path):
    """(packages, deps, workspace-member dirs) for every crate in the tree."""
    pkgs: dict[str, Path] = {}
    deps: dict[str, set[str]] = {}
    members: set[Path] = set()
    roots_self: set[str] = set()

    for ct in source_files(repo, "Cargo.toml"):
        text = ct.read_text(errors="ignore")
        if "[workspace]" in text:
            mm = re.search(r"members\s*=\s*\[(.*?)\]", _strip_comments(text), re.S)
            if mm:
                for n in re.findall(r'"([^"]+)"', mm.group(1)):
                    members.add((ct.parent / n).resolve())
        if "[package]" not in text:
            continue
        m = re.search(r'^\s*name\s*=\s*"([^"]+)"', text, re.M)
        if not m:
            continue
        name = m.group(1)
        pkgs[name] = ct.parent.resolve()
        if "[workspace]" in text:
            roots_self.add(name)  # its own workspace root: a standalone tool
        d: set[str] = set()
        for sec in re.findall(r"^\[(?:.*?)dependencies\](.*?)(?=^\[|\Z)", text, re.S | re.M):
            d |= set(re.findall(r"^\s*([A-Za-z0-9_-]+)\s*=", sec, re.M))
        deps[name] = d
    return pkgs, deps, members, roots_self


def corpora(repo: Path) -> dict[str, str]:
    """The text that can cause a crate to be built.

    THE THIRD MECHANISM, and the one that made this survey wrong twice. A crate
    ships here for one of three reasons, not two: CMake names it, a reached
    crate depends on it in Cargo.toml, or **a `build.rs` hands its name to
    `reactos-link`'s `.rust_lib(...)` / `.msvcrtex_crt(...)`**, which emits a
    linker flag. That third path leaves NO Cargo dependency edge and NO CMake
    string, so a closure over the first two reports live, shipping crates as
    orphans. Measured 2026-09-20: it produced FIVE false positives out of six —
    crt-startup-rust, crt-dllmain-rust, crt0-rust, rtl-vista-rust and
    setjmp-rust all ship, three of them named only inside
    `reactos-link/src/lib.rs` where `.msvcrtex_crt()` expands. Only
    `slist-loom` survived, and it is deliberate scaffolding.

    So `build.rs` and the linker crate's own source are build inputs and belong
    in this corpus. `*.rs` at large deliberately does NOT: a crate name in an
    ordinary comment is prose, and counting it would trade this false-positive
    class for a false-negative one.
    """
    out = {"build": "", "docs": ""}
    for pat in ("CMakeLists.txt", "*.cmake", "*.py", "*.sh", "build.rs"):
        for f in source_files(repo, pat):
            out["build"] += f.read_text(errors="ignore")
    linker = repo / "sdk" / "lib" / "rust" / "reactos-link" / "src"
    if linker.is_dir():
        for f in source_files(linker, "*.rs"):
            out["build"] += f.read_text(errors="ignore")
    for f in source_files(repo, "*.md"):
        out["docs"] += f.read_text(errors="ignore")
    return out


def main() -> int:
    ap = argparse.ArgumentParser()
    ap.add_argument("root", nargs="?", default=None)
    args = ap.parse_args()
    repo = Path(args.root).resolve() if args.root else REPO

    pkgs, deps, members, roots_self = scan(repo)
    text = corpora(repo)

    def named(n: str) -> bool:
        return n in text["build"] or n.replace("-", "_") in text["build"]

    roots = {n for n in pkgs if named(n)}
    roots |= {n for n in pkgs if n.endswith(DIFFTEST_SUFFIX)}
    roots |= roots_self

    seen: set[str] = set()
    stack = list(roots)
    while stack:
        n = stack.pop()
        if n in seen or n not in pkgs:
            continue
        seen.add(n)
        stack.extend(deps.get(n, ()))

    parked = sorted(n for n, d in pkgs.items() if d not in members and n not in roots_self)
    phantom = sorted(d for d in members if not (d / "Cargo.toml").is_file())
    unlinked = sorted(set(pkgs) - seen)

    print(f"packages {len(pkgs)}   workspace members {len(members)}   reachable {len(seen)}")

    print(
        f"\nPARKED — not a workspace member, so NOTHING COMPILES IT and it drifts ({len(parked)}):"
    )
    for n in parked or []:
        print(f"  {n:24s} {pkgs[n].relative_to(repo)}")
    if not parked:
        print("  (none — this is the surface that rots, and it is closed)")

    if phantom:
        print(f"\nPHANTOM MEMBERS — listed in a workspace, no Cargo.toml ({len(phantom)}):")
        for d in phantom:
            print(f"  {d.relative_to(repo)}")

    print(f"\nUNLINKED — a member (compiles, cannot drift), nothing links it ({len(unlinked)}):")
    for n in unlinked:
        dependents = sorted(o for o, dd in deps.items() if n in dd and o != n)
        if dependents:
            why = f"dep-only: depended on by {', '.join(dependents)} — itself unreached"
        elif n in text["docs"]:
            why = "docs-only: named in documentation, built by nothing"
        else:
            why = "no-reference: named in no CMake, script or Python anywhere"
        print(f"  {n:24s} {pkgs[n].relative_to(repo)}\n      why: {why}")

    print(
        "\nUNLINKED is not a defect list. Disposition is decided by reading each crate's"
        "\nbanner — superseded, scaffolding, dead-upstream, or genuinely unfinished — and"
        "\nnot by this tool, which can only say that nothing builds it."
    )
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
