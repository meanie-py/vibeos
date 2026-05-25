#!/usr/bin/env python3
"""freeldr_widen — compiler-span-driven integer-width cast inserter.

The C→Rust translations carry C's implicit integer-promotion everywhere:
packed-struct narrow fields (u8/u16) freely mixed with ULONG in arithmetic
and assignment. Rust rejects each (E0308). The *correct* fix at every such
site is a width `as` cast of the coerced expression to the expected type —
and rustc tells us the EXACT byte span of that expression and the exact
expected type via --message-format=json. So this is deterministic and
directional, NOT a blind source regex (cf. the #if/#elif rewrite hazard):
we only ever wrap the precise span rustc flagged, only when BOTH expected
and found are primitive integer scalars, and idempotently.

Pointer-type E0308 (e.g. *mut _FAT_BOOTSECTOR vs *mut _FAT32_BOOTSECTOR),
c_void field access, and LARGE_INTEGER union shape are NOT touched here —
those need human reinterpret/accessor judgement.

Loop: build(json) -> collect safe E0308 -> apply non-overlapping span
edits -> repeat until no progress. Reusable for the ui unit.

Usage:
  python3 freeldr_widen.py --build-dir sdk/lib/rust [--max-iter 12]
"""
import argparse
import json
import re
import subprocess
import sys
from pathlib import Path

PRIM = {"i8", "u8", "i16", "u16", "i32", "u32", "i64", "u64",
        "i128", "u128", "isize", "usize", "c_int", "c_uint",
        "c_long", "c_ulong", "c_char", "c_uchar", "c_schar"}
# normalize a few alias spellings rustc may print
ALIAS = {"c_int": "i32", "c_uint": "u32", "c_long": "i32",
         "c_ulong": "u32", "c_char": "i8", "c_uchar": "u8",
         "c_schar": "i8"}

MSG_RE = re.compile(r"expected `([A-Za-z0-9_]+)`, found `([A-Za-z0-9_]+)`")


def run_build(build_dir: str):
    cmd = ["cargo", "build", "--release", "-p", "freeldr-rust",
           "--target", "i686-pc-windows-gnu",
           "--message-format=json"]
    env = {"CARGO_TARGET_DIR": "/tmp/freeldr-rust-check",
           "RUSTFLAGS": "-C target-feature=-sse,-sse2"}
    import os
    e = dict(os.environ)
    e.update(env)
    p = subprocess.run(cmd, cwd=build_dir, env=e,
                        capture_output=True, text=True)
    diags = []
    for line in p.stdout.splitlines():
        line = line.strip()
        if not line.startswith("{"):
            continue
        try:
            j = json.loads(line)
        except json.JSONDecodeError:
            continue
        if j.get("reason") != "compiler-message":
            continue
        m = j.get("message")
        if m:
            diags.append(m)
    return diags


def resolve_path(build_dir: Path, fp: str) -> Path:
    p = Path(fp)
    if p.is_absolute() and p.exists():
        return p
    for c in (build_dir / fp, build_dir / "freeldr-rust" / fp):
        if c.exists():
            return c
    return build_dir / fp


def collect_edits(build_dir, diags):
    """-> {path: [(line, col_start, col_end, expected), ...]} (1-based,
    char columns, single-line spans only)."""
    edits = {}
    n_e0308 = n_safe = 0
    for d in diags:
        code = (d.get("code") or {}).get("code")
        if code != "E0308":
            continue
        n_e0308 += 1
        for sp in d.get("spans", []):
            if not sp.get("is_primary"):
                continue
            mm = MSG_RE.search(sp.get("label") or "")
            if not mm:
                break
            exp, found = mm.group(1), mm.group(2)
            if exp not in PRIM or found not in PRIM:
                break
            if sp["line_start"] != sp["line_end"]:
                break
            exp_r = ALIAS.get(exp, exp)
            path = resolve_path(build_dir, sp["file_name"])
            edits.setdefault(str(path), []).append(
                (sp["line_start"], sp["column_start"],
                 sp["column_end"], exp_r))
            n_safe += 1
            break
    return edits, n_e0308, n_safe


def apply_edits(edits):
    applied = 0
    for fp, spans in edits.items():
        path = Path(fp)
        if not path.exists():
            continue
        lines = path.read_text().split("\n")
        # group by line; apply right-to-left within a line; one edit/line
        by_line = {}
        for (ln, c0, c1, exp) in spans:
            by_line.setdefault(ln, []).append((c0, c1, exp))
        for ln, sps in by_line.items():
            if ln - 1 >= len(lines):
                continue
            # pick the widest span on the line, apply once (avoids
            # nested/overlap churn; remaining re-surface next iter)
            c0, c1, exp = max(sps, key=lambda s: s[1] - s[0])
            line = lines[ln - 1]
            i, j = c0 - 1, c1 - 1
            if not (0 <= i < j <= len(line)):
                continue
            seg = line[i:j]
            tail = line[j:j + 24]
            if re.match(r"\s*\)?\s*as\s+" + re.escape(exp) + r"\b", tail):
                continue
            if re.search(r"\)\s*as\s+[A-Za-z_0-9]+\s*$", seg):
                continue
            if seg.strip() == "":
                continue
            # Never wrap inside a match-arm pattern: `as` casts are not
            # valid in pattern position (rustc: "expected a pattern, found
            # an expression"). Heuristic: the text following the flagged
            # span (after optional close-parens/ws) starts with `=>`.
            if re.match(r"[\s)]*=>", line[j:]):
                continue
            # Never wrap a fn-parameter binding: for a call-arg E0308 rustc
            # also emits a primary span at the callee param decl ("expected
            # due to this type"); wrapping it yields `((x) as T): Ty` which
            # is an expression in pattern position (rustc: "expected a
            # pattern, found an expression"). Heuristic: the line is a `fn`
            # signature and the flagged span is immediately followed by `:`
            # (a `name: Type` parameter binding).
            if re.match(r"\s*:", line[j:]) and re.search(
                r"\bfn\s+[A-Za-z_][A-Za-z_0-9]*\s*(<[^>]*>)?\s*\(", line
            ):
                continue
            lines[ln - 1] = line[:i] + f"(({seg}) as {exp})" + line[j:]
            applied += 1
        path.write_text("\n".join(lines))
    return applied


def main():
    ap = argparse.ArgumentParser()
    ap.add_argument("--build-dir", default="sdk/lib/rust")
    ap.add_argument("--max-iter", type=int, default=12)
    a = ap.parse_args()
    bd = Path(a.build_dir).resolve()
    last = None
    for it in range(1, a.max_iter + 1):
        diags = run_build(str(bd))
        edits, n308, nsafe = collect_edits(bd, diags)
        total = len({(d.get("rendered") or d.get("message"))
                     for d in diags if d.get("level") == "error"})
        print(f"[iter {it}] errors~{total}  E0308={n308}  "
              f"safe-widen={nsafe}", flush=True)
        if nsafe == 0:
            print("no more safe widen edits.")
            break
        applied = apply_edits(edits)
        print(f"[iter {it}] applied {applied} cast(s)", flush=True)
        if applied == 0 or last == applied and nsafe == last:
            break
        last = applied
    return 0


if __name__ == "__main__":
    sys.exit(main())
