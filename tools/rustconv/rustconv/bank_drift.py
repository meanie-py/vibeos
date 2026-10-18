"""Banked translations whose C no longer matches what the scanner extracts today.

Each bank manifest records, per function, the sha256 of the C body it was bought from
(response_cache). Re-scan that C now and compare: a function whose body hash moved was
bought from a different text than the tree holds. Two causes, and the report cannot tell
them apart without the old text, so it is a review queue, not a verdict:

- the C changed after purchase (an upstream sync into a flipped file's dead C), or
- the SCANNER changed what it extracts. setupapi's cfgmgr.c is the case that motivated
  this: before RpcTryExcept was masked for tree-sitter, a nested RpcTryExcept ended a
  function early, and the body was bought truncated (2026-10-17). That audit found 7
  functions bought from a cut body; 6 shipped complete anyway (c2rust anchors on the
  whole TU), and winspool's GetMonitorUI was really missing its tail.

To settle which cause applies, check the shipped Rust function against the C (its tail
calls, its returns). A bank with no `.source` claim is matched to its C by name
(`<dir>__<stem>[__<tokens>]`); a name that matches several files is reported, not guessed.

    python -m rustconv.bank_drift                  # this worktree's bank
    python -m rustconv.bank_drift --fleet          # every sibling worktree's bank, read-only
    python -m rustconv.bank_drift --c dll/win32/setupapi/cfgmgr.c --json out.json
"""

from __future__ import annotations

import argparse
import hashlib
import json
import sys
from collections import defaultdict
from dataclasses import asdict, dataclass
from pathlib import Path

from .config import REACTOS_ROOT
from .response_cache import RESPONSES_DIR, SOURCE_CLAIM
from .scanner import scan_file

_C_SUFFIXES = (".c", ".cpp", ".cc", ".cxx")


@dataclass
class Drift:
    c_file: str
    function: str
    bank: str
    state: str  # "moved" (body hash differs) or "gone" (the scanner finds no such function)


def _hash(text: str) -> str:
    return hashlib.sha256(text.encode("utf-8", "replace")).hexdigest()[:16]


def _index_sources(root: Path) -> dict[tuple[str, str], list[Path]]:
    """(dir name, stem) -> the source files that could own a `<dir>__<stem>` bank."""
    index: dict[tuple[str, str], list[Path]] = defaultdict(list)
    for top in (
        "base",
        "dll",
        "drivers",
        "hal",
        "ntoskrnl",
        "sdk",
        "subsystems",
        "win32ss",
        "boot",
        "modules",
    ):
        for path in (root / top).rglob("*"):
            if path.suffix.lower() in _C_SUFFIXES and "build-" not in path.parts[0]:
                index[(path.parent.name, path.stem)].append(path)
    return index


def _source_for(bank: Path, root: Path, index) -> tuple[Path | None, str]:
    claim = bank / SOURCE_CLAIM
    if claim.is_file():
        path = root / claim.read_text().strip()  # an absolute claim stays absolute
        if not path.is_relative_to(root):
            return None, f"claimed source outside the tree: {path}"
        return (path, "") if path.is_file() else (None, f"claimed {path} is gone")
    parts = bank.name.split("__")
    if len(parts) < 2:
        return None, "unparseable bank name"
    # `<grandparent>__<dir>__<stem>` is the second-choice namespace (bank_base).
    for d, stem in ((parts[0], parts[1]), (parts[1], parts[2]) if len(parts) > 2 else ("", "")):
        hits = index.get((d, stem), [])
        if len(hits) == 1:
            return hits[0], ""
        if len(hits) > 1:
            return None, "ambiguous: " + ", ".join(str(h.relative_to(root)) for h in hits[:3])
    return None, "no matching source"


def audit(banks: list[Path], root: Path, only: set[str] | None = None):
    index = _index_sources(root)
    drifts: list[Drift] = []
    unresolved: dict[str, str] = {}
    checked = 0
    scans: dict[Path, dict[str, str]] = {}
    for bank in banks:
        manifest = bank / "manifest.json"
        if not manifest.is_file():
            continue
        src, why = _source_for(bank, root, index)
        if src is None:
            unresolved[str(bank)] = why
            continue
        rel = src.relative_to(root).as_posix()
        if only and rel not in only:
            continue
        if src not in scans:
            try:
                scans[src] = {f.name: f.body_text for f in scan_file(src).functions}
            except Exception as e:  # a file the scanner cannot read is reported, not skipped
                unresolved[str(bank)] = f"scan failed: {e}"
                continue
        bodies = scans[src]
        for name, entry in json.loads(manifest.read_text()).items():
            if name.startswith("__") or not isinstance(entry, dict) or "hash" not in entry:
                continue  # the preamble unit and bookkeeping keys
            checked += 1
            if name not in bodies:
                drifts.append(Drift(rel, name, str(bank), "gone"))
            elif _hash(bodies[name]) != entry["hash"]:
                drifts.append(Drift(rel, name, str(bank), "moved"))
    return checked, drifts, unresolved


def main(argv: list[str] | None = None) -> int:
    ap = argparse.ArgumentParser(description=(__doc__ or "").splitlines()[0])
    ap.add_argument(
        "--fleet", action="store_true", help="read every sibling worktree's bank too (read-only)"
    )
    ap.add_argument("--c", action="append", default=[], help="limit to this C file (repeatable)")
    ap.add_argument("--json", type=Path, help="write every finding here")
    args = ap.parse_args(argv)
    root = Path(REACTOS_ROOT)
    roots = [RESPONSES_DIR]
    if args.fleet:
        roots += sorted(
            p for p in root.parent.glob("*/tools/rustconv/.responses") if p != RESPONSES_DIR
        )
    banks = [d for r in roots if r.is_dir() for d in sorted(r.iterdir()) if d.is_dir()]
    checked, drifts, unresolved = audit(banks, root, set(args.c) or None)
    per_file: dict[str, list[Drift]] = defaultdict(list)
    for d in drifts:
        per_file[d.c_file].append(d)
    for c_file, ds in sorted(per_file.items()):
        names = sorted({f"{d.function}{'' if d.state == 'moved' else ' (gone)'}" for d in ds})
        print(f"{c_file}: {len(names)} — {', '.join(names[:8])}{' ...' if len(names) > 8 else ''}")
    print(
        f"bank_drift: {checked} banked function(s) checked, "
        f"{len({(d.c_file, d.function) for d in drifts})} "
        f"drifted in {len(per_file)} file(s); {len(unresolved)} bank(s) not matched to a source"
    )
    if args.json:
        args.json.write_text(
            json.dumps({"drifts": [asdict(d) for d in drifts], "unresolved": unresolved}, indent=1)
        )
    return 0


if __name__ == "__main__":
    sys.exit(main())
