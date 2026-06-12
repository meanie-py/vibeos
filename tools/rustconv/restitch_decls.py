#!/usr/bin/env python3
"""Re-stitch stitcher-dropped function bodies from the .responses bank.

Finds module-level body-less fn decls (free fn ending in `;`) by name and
either replaces them with the banked body file or deletes them (when the
real body already exists elsewhere in the same .rs file).
"""
import re, sys, pathlib

SRC = pathlib.Path(__file__).resolve().parents[2] / "sdk/lib/rust/fastfat-rust/src"
BANK = pathlib.Path(__file__).resolve().parent / ".responses"

# (rust file, fn name, bank file or None=delete decl)
PLAN = [
    ("deviosup.rs", "FatPagingFileCompletionRoutine", None),
    ("deviosup.rs", "FatSingleAsyncCompletionRoutine", None),
    ("deviosup.rs", "FatSingleNonAlignedSync",
     "20260612-152050-deviosup.c/05_FatSingleNonAlignedSync.rs"),
    ("verfysup.rs", "FatResetFcb", "20260612-145724-verfysup.c/06_FatResetFcb.rs"),
    ("verfysup.rs", "FatMatchFileSize", "20260612-145724-verfysup.c/07_FatMatchFileSize.rs"),
    ("verfysup.rs", "FatDetermineAndMarkFcbCondition",
     "20260612-145724-verfysup.c/15_FatDetermineAndMarkFcbCondition.rs"),
    ("verfysup.rs", "FatDeferredCleanVolume",
     "20260612-145724-verfysup.c/11_FatDeferredCleanVolume.rs"),
    ("verfysup.rs", "FatMarkVolumeCompletionRoutine", None),
    ("volinfo.rs", "FatQueryFsVolumeInfo", "20260612-143211-volinfo.c/01_FatQueryFsVolumeInfo.rs"),
    ("volinfo.rs", "FatQueryFsSizeInfo", "20260612-143211-volinfo.c/02_FatQueryFsSizeInfo.rs"),
    ("volinfo.rs", "FatQueryFsDeviceInfo", "20260612-143211-volinfo.c/03_FatQueryFsDeviceInfo.rs"),
    ("volinfo.rs", "FatQueryFsAttributeInfo", "20260612-143211-volinfo.c/04_FatQueryFsAttributeInfo.rs"),
    ("volinfo.rs", "FatQueryFsFullSizeInfo", "20260612-143211-volinfo.c/05_FatQueryFsFullSizeInfo.rs"),
    ("volinfo.rs", "FatSetFsLabelInfo", "20260612-143211-volinfo.c/06_FatSetFsLabelInfo.rs"),
    ("volinfo.rs", "FatQueryFsSectorSizeInfo", "20260612-143211-volinfo.c/07_FatQueryFsSectorSizeInfo.rs"),
    ("write.rs", "FatDeferredFlushDpc", "20260612-151345-write.c/02_FatDeferredFlushDpc.rs"),
    ("write.rs", "FatDeferredFlush", "20260612-151345-write.c/03_FatDeferredFlush.rs"),
]

def find_decl_span(lines, name):
    """Return (start, end) inclusive line idx of the body-less decl, incl. any
    immediately preceding #[...] attribute lines. None if not found as a decl."""
    pat = re.compile(r'^\s*(pub\s+)?unsafe extern "(C|system)" fn ' + re.escape(name) + r'\s*\(')
    for i, ln in enumerate(lines):
        if pat.match(ln):
            # walk forward to the terminating `;` (decl) or `{` (real body)
            j = i
            depth = 0
            while j < len(lines):
                depth += lines[j].count("(") - lines[j].count(")")
                stripped = lines[j].rstrip()
                if depth <= 0 and (stripped.endswith(";") or stripped.endswith("{")
                                   or "{" in stripped.split("//")[0] and depth <= 0):
                    break
                j += 1
            tail = lines[j].split("//")[0]
            if "{" in tail:
                continue  # real body, not a decl — keep looking
            # include preceding attribute lines
            s = i
            while s > 0 and lines[s-1].lstrip().startswith("#["):
                s -= 1
            return (s, j)
    return None

changed = {}
for fname, fn, bankrel in PLAN:
    p = SRC / fname
    lines = changed.get(fname) or p.read_text().splitlines()
    span = find_decl_span(lines, fn)
    if span is None:
        print(f"SKIP {fname}:{fn} — no body-less decl found")
        changed.setdefault(fname, lines)
        continue
    s, e = span
    if bankrel is None:
        repl = []
        action = "DELETED dup decl"
    else:
        body = (BANK / bankrel).read_text().rstrip("\n")
        repl = [f"// ---- re-stitched from bank: {bankrel} ----"] + body.splitlines()
        action = f"RESTITCHED from {bankrel} ({len(repl)} lines)"
    lines[s:e+1] = repl
    changed[fname] = lines
    print(f"{fname}:{fn}: {action} (was lines {s+1}-{e+1})")

for fname, lines in changed.items():
    (SRC / fname).write_text("\n".join(lines) + "\n")
    print(f"WROTE {fname}")
