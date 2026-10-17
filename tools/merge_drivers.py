#!/usr/bin/env python3
"""Git merge drivers for the two files every translation lane appends to.

    case-arms  tools/rustconv/gen_driver_types.sh: one `case "$DRIVER" in` table, one arm
               per module. Lanes add arms at the same place (before `*)`) and git's line
               merge calls that a conflict; E'+G and A' each resolved one by hand by taking
               both arms. The arms are merged 3-way BY LABEL instead: an arm added, edited
               or deleted on one side only is taken, and an arm both sides changed
               differently is merged with `git merge-file`, its conflict markers left in
               place. Built-in `merge=union` is not this: lanes also EDIT arms (storport's
               re-port rewrote its own), and union keeps both versions of an edited line.
    census     tools/cohab-census.txt: `file<TAB>count<TAB>line`, one row per (file, line),
               sorted as tools/cohab_census.py writes it. Merged 3-way as that map;
               a row both sides changed differently is a conflict.

Usage (git config merge.<name>.driver): merge_drivers.py MODE %O %A %B
Writes the result to %A. Exit 0 = clean, 1 = conflicts left in %A. A file whose shape
the driver does not recognise falls back to `git merge-file`, the ordinary text merge;
it is never guessed at. tools/install-git-hooks.sh configures both drivers.
"""

from __future__ import annotations

import re
import subprocess
import sys
import tempfile
from pathlib import Path

_CASE_OPEN = re.compile(r'^case "\$DRIVER" in\n', re.M)
_ESAC = re.compile(r"^esac\n", re.M)
#: An arm's label line: two-space indent, a pattern, `)`. Comments at that indent are not.
_LABEL = re.compile(r"^  (?![ #])([^\n]*?)\)[^\n]*$", re.M)


class Unrecognised(Exception):
    pass


def text_merge(base: str, ours: str, theirs: str, label: str = "") -> tuple[str, bool]:
    """git merge-file on three strings: (result, clean)."""
    with tempfile.TemporaryDirectory() as tmp:
        paths = []
        for name, text in (("ours", ours), ("base", base), ("theirs", theirs)):
            p = Path(tmp) / name
            p.write_text(text)
            paths.append(str(p))
        names = ["-L", f"ours{label}", "-L", f"base{label}", "-L", f"theirs{label}"]
        cp = subprocess.run(
            ["git", "merge-file", "-p", *names, *paths], capture_output=True, text=True
        )
        if cp.returncode < 0 or cp.returncode > 127:
            raise RuntimeError(f"git merge-file failed: {cp.stderr.strip()}")
        return cp.stdout, cp.returncode == 0


def split_arms(text: str) -> tuple[str, list[tuple[str, str]], str]:
    """(head through `case ... in`, [(label, arm text)], tail from `esac`).

    Comment and blank lines directly above a label belong to that label's arm, so a lane's
    explanatory comment travels with the arm it explains."""
    opens = list(_CASE_OPEN.finditer(text))
    if len(opens) != 1:
        raise Unrecognised(f'{len(opens)} top-level `case "$DRIVER" in` lines')
    start = opens[0].end()
    close = _ESAC.search(text, start)
    if close is None:
        raise Unrecognised("no top-level `esac`")
    body = text[start : close.start()]
    labels = list(_LABEL.finditer(body))
    if not labels:
        raise Unrecognised("no arms")
    cuts = []
    for m in labels:
        cut = m.start()
        # Walk back over the comment/blank lines that sit above this label.
        while cut > 0:
            prev = body.rfind("\n", 0, cut - 1) + 1
            line = body[prev:cut]
            if line.strip() == "" or line.startswith("  #"):
                cut = prev
            else:
                break
        cuts.append(cut)
    if cuts[0] != 0:
        raise Unrecognised("text before the first arm")
    arms = []
    for i, m in enumerate(labels):
        end = cuts[i + 1] if i + 1 < len(cuts) else len(body)
        arms.append((m.group(1).strip(), body[cuts[i] : end]))
    keys = [k for k, _ in arms]
    if len(set(keys)) != len(keys):
        raise Unrecognised("a label appears twice")
    return text[:start], arms, text[close.start() :]


def merge_case_arms(base: str, ours: str, theirs: str) -> tuple[str, bool]:
    try:
        b_head, b_arms, b_tail = split_arms(base)
        o_head, o_arms, o_tail = split_arms(ours)
        t_head, t_arms, t_tail = split_arms(theirs)
    except Unrecognised:
        return text_merge(base, ours, theirs)
    head, clean_head = text_merge(b_head, o_head, t_head, " (head)")
    tail, clean_tail = text_merge(b_tail, o_tail, t_tail, " (tail)")
    clean = clean_head and clean_tail
    b_map, o_map, t_map = dict(b_arms), dict(o_arms), dict(t_arms)

    # Order: ours, then each arm only theirs has, placed just before the next arm (in
    # theirs' order) that ours also has -- so it lands after ours' own new arms and never
    # after the `*)` default, which every side shares.
    order = [k for k, _ in o_arms]
    t_keys = [k for k, _ in t_arms]
    for i, k in enumerate(t_keys):
        if k in order:
            continue
        nxt = next((n for n in t_keys[i + 1 :] if n in o_map), None)
        order.insert(order.index(nxt) if nxt else len(order), k)

    out = []
    for k in order:
        b, o, t = b_map.get(k), o_map.get(k), t_map.get(k)
        if o == t:
            pick = o
        elif o == b:
            pick = t
        elif t == b:
            pick = o
        else:
            pick, ok = text_merge(b or "", o or "", t or "", f" ({k})")
            clean = clean and ok
        if pick is not None:
            out.append(pick)
    return head + "".join(out) + tail, clean


def _census_rows(text: str) -> dict[tuple[str, str], str]:
    rows: dict[tuple[str, str], str] = {}
    for line in text.splitlines():
        parts = line.split("\t", 2)
        if len(parts) != 3 or not parts[1].isdigit():
            raise Unrecognised(f"not a census row: {line[:80]!r}")
        key = (parts[0], parts[2])
        if key in rows:
            raise Unrecognised(f"duplicate census row {key}")
        rows[key] = parts[1]
    return rows


def merge_census(base: str, ours: str, theirs: str) -> tuple[str, bool]:
    try:
        b_map, o_map, t_map = _census_rows(base), _census_rows(ours), _census_rows(theirs)
    except Unrecognised:
        return text_merge(base, ours, theirs)
    out, conflicts = {}, []
    for k in set(b_map) | set(o_map) | set(t_map):
        b, o, t = b_map.get(k), o_map.get(k), t_map.get(k)
        if o == t:
            pick = o
        elif o == b:
            pick = t
        elif t == b:
            pick = o
        else:
            conflicts.append(k)
            continue
        if pick is not None:
            out[k] = pick
    text = "".join(f"{f}\t{n}\t{t}\n" for (f, t), n in sorted(out.items()))
    if conflicts:
        lines = []
        for k in sorted(conflicts):
            lines.append(
                f"<<<<<<< ours\n{k[0]}\t{o_map.get(k)}\t{k[1]}\n=======\n"
                f"{k[0]}\t{t_map.get(k)}\t{k[1]}\n>>>>>>> theirs\n"
            )
        # Regenerating is the real fix; say so where the reader will look.
        text += "".join(lines)
    return text, not conflicts


MODES = {"case-arms": merge_case_arms, "census": merge_census}


def main(argv: list[str]) -> int:
    if len(argv) != 4 or argv[0] not in MODES:
        print(__doc__, file=sys.stderr)
        return 2
    mode, base_p, ours_p, theirs_p = argv
    base, ours, theirs = (Path(p).read_text() for p in (base_p, ours_p, theirs_p))
    result, clean = MODES[mode](base, ours, theirs)
    Path(ours_p).write_text(result)
    if not clean and mode == "census":
        print(
            "census: rows changed on both sides; run `python3 tools/cohab_census.py --write`",
            file=sys.stderr,
        )
    return 0 if clean else 1


if __name__ == "__main__":
    sys.exit(main(sys.argv[1:]))
