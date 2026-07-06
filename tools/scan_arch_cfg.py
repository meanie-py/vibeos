#!/usr/bin/env python3
"""scan_arch_cfg.py — targeting filter for per-arch `#[cfg]` divergence audits.

The Rust workspace has ~8.4K `#[cfg(target_arch/os)]` sites, but the vast majority
are LEGITIMATE arch differences the eye should skip: calling-convention/ABI on
`extern` blocks, `global_asm!`/`asm!`, `use`/`type`/`mod` selection, and
pointer-width/layout. Auditing per-arch *logic* divergence (the kind that hides
bugs — see the field-manual §9.15 "PXE 4-level-paging gated to one 4-level arch"
drift class) means first stripping all that legit noise.

This tool classifies each `#[cfg(...target_arch/target_os...)]` site by the item it
attaches to and emits only the CANDIDATES — cfg that gates actual logic (fn bodies,
constants, statements, whole-fn inclusion) — so an audit (human or subagent fleet)
reviews ~2.4K sites instead of ~8.4K. It is a **triage/targeting tool, not a
pass/fail gate**: candidates are "look at these," not "these are wrong" (most are
still legit; the point is a 3-4x smaller, higher-density review surface).

Usage:
    python3 tools/scan_arch_cfg.py                       # summary + per-crate counts
    python3 tools/scan_arch_cfg.py -o candidates.txt     # + write the worklist
    python3 tools/scan_arch_cfg.py --root sdk/lib/rust/ntoskrnl-rust   # scope it

Worklist line format: `path:line<TAB>cfg(predicate)<TAB>attached-code` — feed slices
of it to reviewers. See docs/engineering/per-arch-cfg-divergence-audit.md for the
methodology + the classification rubric (DRIFT-BUG / BRINGUP-SHORTCUT /
UNNECESSARY-DIVERGENCE / ARCH-STUB vs LEGIT).
"""
import argparse
import re
import sys
from pathlib import Path

DEFAULT_ROOT = "sdk/lib/rust"
CFG_RE = re.compile(r'#\[\s*cfg\s*\(')
ARCH_RE = re.compile(r'target_arch|target_os')
# cfg attached to these => legit arch selection, not logic divergence
LEGIT_NEXT = re.compile(
    r'^\s*(pub(\(crate\))?\s+)?(unsafe\s+)?'
    r'(extern\b|use\b|type\b|mod\b|global_asm!|core::arch::asm!|asm!)'
)
LINK_ATTR = re.compile(r'^\s*#\[\s*link')
POINTER_WIDTH = re.compile(r'target_pointer_width')


def cfg_predicate(line):
    m = re.search(r'#\[\s*cfg\s*\((.*)\)\s*\]', line)
    return m.group(1).strip() if m else line.strip()


def classify_file(path):
    try:
        lines = path.read_text(errors="replace").split("\n")
    except OSError:
        return []
    out = []
    for i, ln in enumerate(lines):
        if not CFG_RE.search(ln) or not ARCH_RE.search(ln):
            continue
        # find the item the attribute attaches to (skip blank/comment/attr lines)
        nxt = ""
        j = i + 1
        while j < len(lines):
            s = lines[j].strip()
            if s == "" or s.startswith("//"):
                j += 1
                continue
            if s.startswith("#["):
                if LINK_ATTR.match(lines[j]):  # extern data import => legit
                    nxt = "#[link]"
                    break
                j += 1
                continue
            nxt = lines[j]
            break
        pred = cfg_predicate(ln)
        if POINTER_WIDTH.search(pred) and not re.search(r'target_arch|target_os', pred):
            bucket = "LEGIT"  # pure layout/width
        elif nxt == "#[link]" or LEGIT_NEXT.match(nxt):
            bucket = "LEGIT"  # ABI / asm / module selection
        else:
            bucket = "CANDIDATE"  # fn/const/static/stmt/expr => review
        out.append((bucket, i + 1, pred, nxt.strip()[:80]))
    return out


def main():
    ap = argparse.ArgumentParser(description=__doc__.split("\n")[0])
    ap.add_argument("--root", default=DEFAULT_ROOT, help=f"scan root (default {DEFAULT_ROOT})")
    ap.add_argument("-o", "--out", help="write the CANDIDATE worklist to this file")
    ap.add_argument("--top", type=int, default=30, help="crates to show (default 30)")
    args = ap.parse_args()

    root = Path(args.root)
    per_crate, candidates = {}, []
    for path in root.rglob("*.rs"):
        try:
            crate = str(path.relative_to(DEFAULT_ROOT)).split("/")[0]
        except ValueError:
            crate = path.parts[0]
        for bucket, lineno, pred, nxt in classify_file(path):
            per_crate.setdefault(crate, {"LEGIT": 0, "CANDIDATE": 0})[bucket] += 1
            if bucket == "CANDIDATE":
                candidates.append(f"{path}:{lineno}\tcfg({pred})\t{nxt}")

    tot_c = sum(v["CANDIDATE"] for v in per_crate.values())
    tot_l = sum(v["LEGIT"] for v in per_crate.values())
    print(f"TOTAL: {tot_c} CANDIDATE (logic-gating) / {tot_l} LEGIT (ABI/asm/layout/module)")
    print(f"\n=== CANDIDATE count by crate (top {args.top}) ===")
    for crate, v in sorted(per_crate.items(), key=lambda kv: -kv[1]["CANDIDATE"])[:args.top]:
        if v["CANDIDATE"]:
            print(f"  {v['CANDIDATE']:5d} cand / {v['LEGIT']:5d} legit   {crate}")
    if args.out:
        Path(args.out).write_text("\n".join(candidates) + "\n")
        print(f"\ncandidate worklist -> {args.out} ({len(candidates)} lines)")


if __name__ == "__main__":
    main()
