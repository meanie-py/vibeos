#!/usr/bin/env python3
"""Fix E0252 (name reimported) and E0255 (local def conflicts with import).

E0252: primary span points at the duplicate name inside a later `use` list —
remove just that name (and a trailing/leading comma) from the list.
E0255: primary span points at the LOCAL redefinition — delete the local item;
the import (canonical ntkrnl-sys/crate::types) wins, per the
local-shadow-elimination doctrine. Iterates until clean or no progress.
"""
import json, subprocess, pathlib, collections, re

RUST_DIR = pathlib.Path(__file__).resolve().parents[2] / "sdk/lib/rust"
TARGET = "x86_64-pc-windows-gnu"

def cargo_errs():
    p = subprocess.run(
        ["cargo", "check", "-p", "fastfat-rust", "--target", TARGET,
         "--message-format", "json"],
        cwd=RUST_DIR, capture_output=True, text=True)
    out = []
    for line in p.stdout.splitlines():
        try:
            m = json.loads(line)
        except json.JSONDecodeError:
            continue
        if m.get("reason") != "compiler-message":
            continue
        msg = m["message"]
        code = (msg.get("code") or {}).get("code")
        if code in ("E0252", "E0255"):
            for sp in msg.get("spans", []):
                if sp.get("is_primary"):
                    out.append((code, sp["file_name"], sp["line_start"],
                                sp["column_start"], sp["column_end"]))
    return out

def item_span(lines, ident_line):
    i = ident_line - 1
    s = i
    while s > 0 and lines[s-1].strip().startswith(("#[", "#!")):
        s -= 1
    if lines[i].strip().endswith(";"):
        return (s, i)
    depth_p = depth_b = 0
    saw_brace = False
    j = i
    while j < len(lines):
        code = lines[j].split("//")[0]
        depth_p += code.count("(") - code.count(")")
        if "{" in code:
            saw_brace = True
        depth_b += code.count("{") - code.count("}")
        if saw_brace and depth_b <= 0:
            return (s, j)
        if not saw_brace and depth_p <= 0 and code.rstrip().endswith(";"):
            return (s, j)
        j += 1
    return (s, i)

def remove_import_name(lines, ln, cs, ce):
    """Remove the name token at (ln, cs..ce) from a use list, plus one comma."""
    i = ln - 1
    line = lines[i]
    a, b = cs - 1, ce - 1  # 0-based slice bounds
    before, tok, after = line[:a], line[a:b], line[b:]
    # eat a following comma+space, else a preceding comma+space
    if after.lstrip().startswith(","):
        after = after.lstrip()[1:].lstrip()
        new = before + after
    elif before.rstrip().endswith(","):
        new = before.rstrip()[:-1] + after
    else:
        # sole import in the list → drop the whole use stmt if it collapses
        new = before + after
    # collapse `use x::{};` or `use x::;` remnants
    if re.match(r"^\s*use\s+[\w:]*\s*(::)?\s*\{\s*\}\s*;\s*$", new) or \
       re.match(r"^\s*use\s+[\w:]*::\s*;\s*$", new):
        lines[i] = None
    else:
        lines[i] = new

for round_no in range(1, 8):
    errs = cargo_errs()
    if not errs:
        print(f"round {round_no}: clean — done")
        break
    by_file = collections.defaultdict(list)
    for e in errs:
        by_file[e[1]].append(e)
    fixed = 0
    for f, es in by_file.items():
        p = RUST_DIR / f if not pathlib.Path(f).is_absolute() else pathlib.Path(f)
        lines = p.read_text().splitlines()
        # E0255 deletions (multi-line items) first, bottom-up
        e255 = sorted({item_span(lines, ln) for c, _, ln, *_ in es if c == "E0255"},
                      reverse=True)
        last = len(lines) + 1
        for (s, e) in e255:
            if e >= last:
                continue
            del lines[s:e+1]
            last = s
            fixed += 1
        # E0252 name removals: only safe if their line wasn't shifted by E0255
        # deletions above it — restrict to lines above the lowest deletion point
        cutoff = e255[-1][0] if e255 else len(lines) + 1
        for c, _, ln, cs, ce in sorted(
                (e for e in es if e[0] == "E0252"),
                key=lambda x: (x[2], x[3]), reverse=True):
            if ln - 1 >= cutoff:
                continue  # line numbers stale below deletions; next round
            remove_import_name(lines, ln, cs, ce)
            fixed += 1
        lines = [l for l in lines if l is not None]
        p.write_text("\n".join(lines) + "\n")
    print(f"round {round_no}: {len(errs)} conflicts → fixed {fixed}")
    if fixed == 0:
        print("no progress — stopping")
        break
