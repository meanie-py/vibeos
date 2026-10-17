"""Bring a module's paid response banks over from a sibling worktree.

Each worktree keeps its own `tools/rustconv/.responses/`, so a module ported in one lane is a
total cache miss in the next: cicero's 2026-10-16 regeneration in `vibeos-cpp-cluster` found
every one of its 15 units empty, because its 2026-10-10 purchase lives in
`vibeos-msvcrt-homebrew`. The banks are keyed by content (each entry carries the hash of the
C body it answers), so they are safe to share; what is not safe is WRITING into another
worktree (AGENTS.md "fleet isolation"). So this only ever reads the sibling and writes here,
copies only what is missing, and never replaces a file that differs -- that is reported.
"""

from __future__ import annotations

import json
import os
import subprocess
from dataclasses import dataclass, field
from pathlib import Path

MANIFEST = "manifest.json"
SOURCE_CLAIM = ".source"


@dataclass
class ImportReport:
    namespaces: list[str] = field(default_factory=list)
    files: int = 0
    entries: int = 0
    conflicts: list[str] = field(default_factory=list)
    foreign: list[str] = field(default_factory=list)  # claimed by a different source file


def namespace_bases(units: list[Path]) -> dict[str, Path]:
    """`<dir>__<stem>` (the prefix of every bank namespace a unit can own) -> the unit.
    The flags suffix (`__factseal_v1__callseal_v1`) varies with the run, so it is matched."""
    return {f"{u.parent.name}__{u.stem}": u for u in units}


def _owns(ns: str, base: str) -> bool:
    return ns == base or ns.startswith(base + "__")


def _claim(d: Path) -> str | None:
    f = d / SOURCE_CLAIM
    return f.read_text().strip() if f.is_file() else None


def _same_source(a: str | None, b: str | None, repo: Path) -> bool:
    if a is None or b is None:
        return True

    def norm(s: str) -> str:
        p = Path(s)
        return str((p if p.is_absolute() else repo / p).resolve())

    return norm(a) == norm(b)


def import_banks(
    units: list[Path], src: Path, dst: Path, repo: Path, *, dry_run: bool = False
) -> ImportReport:
    """Merge the namespaces of *units* from the bank dir *src* into *dst*."""
    rep = ImportReport()
    bases = namespace_bases(units)
    if not src.is_dir():
        return rep
    for d in sorted(p for p in src.iterdir() if p.is_dir()):
        base = next((b for b in bases if _owns(d.name, b)), None)
        if base is None:
            continue
        mine = dst / d.name
        unit_claim = str(bases[base])
        if not _same_source(_claim(d), unit_claim, repo) or not _same_source(
            _claim(mine), _claim(d), repo
        ):
            rep.foreign.append(d.name)
            continue
        theirs_manifest = _load(d / MANIFEST)
        mine_manifest = _load(mine / MANIFEST)
        copied: set[str] = set()
        for f in sorted(d.iterdir()):
            if f.name == MANIFEST or not f.is_file():
                continue
            target = mine / f.name
            if target.exists():
                if target.read_bytes() != f.read_bytes() and f.name != SOURCE_CLAIM:
                    rep.conflicts.append(f"{d.name}/{f.name}")
                continue
            rep.files += 1
            copied.add(f.name)
            if not dry_run:
                mine.mkdir(parents=True, exist_ok=True)
                target.write_bytes(f.read_bytes())
        # An entry is taken only when this bank has none for the name, and its file is
        # here now with the sibling's bytes (copied above, or already identical).
        merged = dict(mine_manifest)
        for name, entry in theirs_manifest.items():
            if name in merged:
                continue
            body = entry.get("file", "")
            here, there = mine / body, d / body
            if not body or not there.is_file():
                continue
            if body in copied or (here.is_file() and here.read_bytes() == there.read_bytes()):
                merged[name] = entry
                rep.entries += 1
        if len(merged) != len(mine_manifest) or copied:
            rep.namespaces.append(d.name)
            if not dry_run and len(merged) != len(mine_manifest):
                mine.mkdir(parents=True, exist_ok=True)
                tmp = mine / (MANIFEST + ".tmp")
                tmp.write_text(json.dumps(merged, indent=1, sort_keys=True))
                os.replace(tmp, mine / MANIFEST)
    return rep


def _load(p: Path) -> dict:
    try:
        return json.loads(p.read_text())
    except (OSError, ValueError):
        return {}


def sibling_bank_dirs(repo: Path) -> list[Path]:
    """Every OTHER worktree's `.responses`, from `git worktree list`."""
    try:
        out = subprocess.run(
            ["git", "-C", str(repo), "worktree", "list", "--porcelain"],
            capture_output=True,
            text=True,
            check=True,
        ).stdout
    except (OSError, subprocess.CalledProcessError):
        return []
    roots = [Path(ln.split(" ", 1)[1]) for ln in out.splitlines() if ln.startswith("worktree ")]
    here = repo.resolve()
    return [
        r / "tools" / "rustconv" / ".responses"
        for r in roots
        if r.resolve() != here and (r / "tools" / "rustconv" / ".responses").is_dir()
    ]


def banked_elsewhere(units: list[Path], names: set[str], repo: Path) -> dict[Path, int]:
    """For a refusal message: which sibling banks hold how many of *names* for *units*."""
    bases = namespace_bases(units)
    found: dict[Path, int] = {}
    for bank in sibling_bank_dirs(repo):
        hit: set[str] = set()
        for d in bank.iterdir():
            if d.is_dir() and any(_owns(d.name, b) for b in bases):
                hit |= names & set(_load(d / MANIFEST))
        if hit:
            found[bank.parent.parent.parent] = len(hit)
    return found


__all__ = [
    "ImportReport",
    "banked_elsewhere",
    "import_banks",
    "namespace_bases",
    "sibling_bank_dirs",
]
