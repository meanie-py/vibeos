#!/usr/bin/env python3
"""Remove duplicate module-level definitions (E0428) left by the stitcher.

Strategy: cargo check --message-format json; for each E0428 the primary span
points at the REdefinition's ident. Delete that whole item (const/static/fn
decl/type alias/struct), consuming to the terminating `;` or balanced braces.
Keep-first policy; verify_constants.py is the value-correctness safety net.
Iterates until no E0428s remain or no progress.
"""
import json, subprocess, sys, pathlib, collections

RUST_DIR = pathlib.Path(__file__).resolve().parents[2] / "sdk/lib/rust"
TARGET = "x86_64-pc-windows-gnu"

def cargo_e0428():
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
        if msg.get("code") and msg["code"]["code"] == "E0428":
            for sp in msg.get("spans", []):
                if sp.get("is_primary"):
                    out.append((sp["file_name"], sp["line_start"]))
    return out

def item_span(lines, ident_line):
    """Given 1-based line of the redefinition ident, return (s,e) 0-based
    inclusive span of the whole item."""
    i = ident_line - 1
    # walk back over attribute / visibility-only lines
    s = i
    while s > 0 and lines[s-1].strip().startswith(("#[", "#!")):
        s -= 1
    first = lines[i].strip()
    # single-line const/static/type ending in ;
    if first.endswith(";"):
        return (s, i)
    # multi-line: consume forward; struct/enum/union/impl/fn-with-body use braces,
    # extern-fn-decls and consts use `;` at paren-depth 0 and brace-depth 0
    depth_p = 0
    depth_b = 0
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
    return (s, i)  # fallback: just the ident line

total_removed = 0
for round_no in range(1, 8):
    errs = cargo_e0428()
    if not errs:
        print(f"round {round_no}: 0 E0428 — done")
        break
    by_file = collections.defaultdict(list)
    for f, ln in errs:
        by_file[f].append(ln)
    removed = 0
    for f, lns in by_file.items():
        p = RUST_DIR / f if not pathlib.Path(f).is_absolute() else pathlib.Path(f)
        lines = p.read_text().splitlines()
        spans = []
        for ln in lns:
            spans.append(item_span(lines, ln))
        # delete from bottom up, skip overlapping
        spans = sorted(set(spans), reverse=True)
        last_start = len(lines) + 1
        for (s, e) in spans:
            if e >= last_start:
                continue
            del lines[s:e+1]
            last_start = s
            removed += 1
        p.write_text("\n".join(lines) + "\n")
    total_removed += removed
    print(f"round {round_no}: {len(errs)} E0428 errors → removed {removed} dup items")
    if removed == 0:
        print("no progress — stopping")
        break
print(f"TOTAL removed: {total_removed}")
