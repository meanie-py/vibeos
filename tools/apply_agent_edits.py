#!/usr/bin/env python3
"""Apply a subagent's proposed edits after review: JSONL of {id, class, edit:{file, old, new}}.

    python3 tools/apply_agent_edits.py out-A.jsonl out-B.jsonl [--ids A1 A3] [--classes BUG]
        [--dry-run]

The working pattern for a wide mechanical-but-judged change (the fastfat clash residue, the
NTDDI gates): agents stay read-only and write proposed edits as JSONL; the lead reviews them
and applies centrally, so no agent ever writes a shared file. An edit applies only if its
`old` text occurs EXACTLY once in the file at apply time; anything else is reported and
skipped, never guessed. `file` is relative to the repository root.
"""

from __future__ import annotations

import argparse
import json
import sys
from pathlib import Path

REPO = Path(__file__).resolve().parent.parent


def load(paths: list[Path]) -> list[dict]:
    rows = []
    for p in paths:
        rows += [json.loads(ln) for ln in p.read_text().splitlines() if ln.strip()]
    return rows


def apply(rows: list[dict], root: Path, dry_run: bool = False) -> tuple[int, list[str]]:
    """(applied count, skip messages). Edits are applied in order, each against the result
    of the ones before it, so two edits to one file must not overlap."""
    ok, skipped = 0, []
    for r in rows:
        e = r.get("edit")
        if not isinstance(e, dict):
            continue
        f = root / e["file"]
        text = f.read_text()
        n = text.count(e["old"])
        if n != 1:
            skipped.append(f"SKIP {r.get('id')} {e['file']}: old occurs {n}x")
            continue
        if not dry_run:
            f.write_text(text.replace(e["old"], e["new"], 1))
        ok += 1
    return ok, skipped


def main(argv: list[str]) -> int:
    ap = argparse.ArgumentParser(
        description=__doc__, formatter_class=argparse.RawDescriptionHelpFormatter
    )
    ap.add_argument("jsonl", nargs="+", type=Path)
    ap.add_argument("--ids", nargs="*")
    ap.add_argument("--classes", nargs="*")
    ap.add_argument("--dry-run", action="store_true")
    a = ap.parse_args(argv)
    rows = load(a.jsonl)
    if a.ids is not None:
        rows = [r for r in rows if r.get("id") in a.ids]
    if a.classes is not None:
        rows = [r for r in rows if r.get("class") in a.classes]
    ok, skipped = apply(rows, REPO, a.dry_run)
    print("\n".join(skipped))
    print(f"{'would apply' if a.dry_run else 'applied'} {ok}, skipped {len(skipped)}")
    return 1 if skipped else 0


if __name__ == "__main__":
    sys.exit(main(sys.argv[1:]))
