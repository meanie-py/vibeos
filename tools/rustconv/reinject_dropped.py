#!/usr/bin/env python3
"""Recover stitcher-dropped fastfat function bodies from the canonical .responses bank.

The stitcher's _merge_extern_blocks comment-collapse bug absorbed 66 function
DEFINITIONS into extern-block `//` comments, leaving only caller extern-decls
live (which the runaway agent then resolved with STUB_BUGCHECK in the cohab C —
a false-green link). The bodies were paid-for and banked at
tools/rustconv/.responses/fastfat__<file>/<Fn>.rs.

This injects each dropped fn's real body into its .rs, replicating the
assembler's transform: drop the bank's forward-decl extern block, move the
bank's top-level placeholder consts function-local, append the fn definition,
and strip the corrupted comment-decl + any self-referential extern decl.
"""
import re, json, pathlib, sys

ROOT = pathlib.Path(__file__).resolve().parents[2]
SRC = ROOT / "sdk/lib/rust/fastfat-rust/src"
BANK = ROOT / "tools/rustconv/.responses"
dropped = json.load(open("/tmp/dropped_by_file.json"))

def extract_fn_and_consts(bank_text, fn):
    """Return (consts:list[str], fn_block:str) from a bank file for `fn`."""
    lines = bank_text.splitlines()
    # find the fn definition start: a line with 'fn <fn>(' that is a definition
    start = None
    for i, ln in enumerate(lines):
        code = ln.split('//', 1)[0]
        if re.search(r'\bfn\s+' + re.escape(fn) + r'\s*\(', code) and not code.rstrip().endswith(';'):
            # back up over attribute / pub / unsafe / extern lines
            s = i
            while s > 0:
                prev = lines[s-1].strip()
                if prev.startswith('#[') or prev in ('pub','unsafe') or prev.startswith('pub ') or prev.startswith('unsafe ') or prev.startswith('extern'):
                    s -= 1
                else:
                    break
            start = s
            break
    if start is None:
        return None, None
    # find end: brace-match from the first '{' at/after the signature
    depth = 0; seen = False; end = None
    for j in range(start, len(lines)):
        for ch in lines[j]:
            if ch == '{': depth += 1; seen = True
            elif ch == '}': depth -= 1
        if seen and depth == 0:
            end = j; break
    if end is None:
        return None, None
    fn_block = "\n".join(lines[start:end+1])
    # collect top-level placeholder consts (before the fn) that the body may use
    consts = []
    for ln in lines[:start]:
        m = re.match(r'\s*(const\s+\w+\s*:\s*[^=]+=\s*[^;]+;)', ln)
        if m:
            consts.append(m.group(1).strip())
    return consts, fn_block

def inject_consts_into_body(fn_block, consts):
    if not consts: return fn_block
    # insert after the first '{' line of the fn signature
    lines = fn_block.splitlines()
    for i, ln in enumerate(lines):
        if ln.rstrip().endswith('{'):
            indent = '    '
            ins = [indent + c for c in consts]
            return "\n".join(lines[:i+1] + ins + lines[i+1:])
    return fn_block

report = {}
for fname, fns in dropped.items():
    rs = SRC / f"{fname}.rs"
    bankdir = BANK / f"fastfat__{fname}"
    if not rs.exists() or not bankdir.exists():
        report[fname] = f"MISSING rs={rs.exists()} bank={bankdir.exists()}"; continue
    text = rs.read_text()
    injected = []
    add_blocks = []
    fnset = set(fns)
    # Remove LIVE extern decls (`fn X(...);`) for the fns we're about to define,
    # else def + decl in same module = E0428. Comment-stripping preserves the
    # harmless commented-out (corrupted) copies.
    kept = []
    for ln in text.splitlines():
        code = ln.split('//', 1)[0]
        m = re.search(r'\bfn\s+(\w+)\s*\(', code)
        if m and m.group(1) in fnset and code.rstrip().endswith(';'):
            continue
        kept.append(ln)
    text = "\n".join(kept)
    for fn in fns:
        bf = bankdir / f"{fn}.rs"
        if not bf.exists():
            report.setdefault(fname, {})
            continue
        consts, fn_block = extract_fn_and_consts(bf.read_text(), fn)
        if fn_block is None:
            print(f"  ! {fname}::{fn} — could not extract from bank")
            continue
        fn_block = inject_consts_into_body(fn_block, consts)
        # ensure it's no_mangle (so it provides the C symbol)
        if '#[no_mangle]' not in fn_block.split('{',1)[0]:
            fn_block = "#[no_mangle]\n" + fn_block
        add_blocks.append(f"\n// ==== re-stitched from bank (dropped by _merge_extern_blocks) ====\n{fn_block}\n")
        injected.append(fn)
    if not add_blocks:
        continue
    # Insert before the "// ===... ORIGINAL C SOURCE" trailer if present, else append
    marker = "\n// " + "="*76 + "\n// ORIGINAL C SOURCE"
    idx = text.find(marker)
    addition = "".join(add_blocks)
    if idx != -1:
        text = text[:idx] + "\n" + addition + text[idx:]
    else:
        text = text.rstrip() + "\n" + addition
    rs.write_text(text)
    report[fname] = f"injected {len(injected)}: {', '.join(injected)}"

for k in sorted(report): print(f"{k}: {report[k]}")
