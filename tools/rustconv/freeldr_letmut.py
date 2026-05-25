#!/usr/bin/env python3
"""freeldr_letmut — apply rustc's MachineApplicable `let`→`let mut` fix.

rustconv emits `let X;` then assigns/reassigns X later (C declares the
local once, assigns repeatedly). rustc flags E0384 ("cannot assign twice
to immutable variable") and supplies a MachineApplicable suggestion that
inserts `mut ` at an exact span. This reads cargo JSON, collects those
suggestions for files under freeldr-rust/src/, and applies them bottom-up
(idempotent: a decl that already reads `let mut` is skipped).

Usage: freeldr_letmut.py --build-dir sdk/lib/rust [--max-iter N]
"""
import argparse
import json
import os
import subprocess
from collections import defaultdict

TARGET = "i686-pc-windows-gnu"


def collect(build_dir):
    env = dict(os.environ, CARGO_TARGET_DIR="/tmp/fldr-winldr")
    p = subprocess.run(
        ["cargo", "build", "--release", "-p", "freeldr-rust",
         "--target", TARGET, "--message-format=json"],
        cwd=build_dir, env=env, capture_output=True, text=True)
    # file -> set of (line, col) insertion points for "mut "
    pts = defaultdict(set)
    for line in p.stdout.splitlines():
        line = line.strip()
        if not line.startswith("{"):
            continue
        try:
            m = json.loads(line)
        except json.JSONDecodeError:
            continue
        if m.get("reason") != "compiler-message":
            continue
        msg = m["message"]
        if (msg.get("code") or {}).get("code") != "E0384":
            continue
        for ch in msg.get("children", []):
            for sp in ch.get("spans", []):
                if sp.get("suggested_replacement") == "mut " \
                        and "freeldr-rust/src/" in sp["file_name"] \
                        and sp["line_start"] == sp["line_end"]:
                    pts[sp["file_name"]].add(
                        (sp["line_start"], sp["column_start"]))
    return pts


def apply(build_dir, pts):
    n = 0
    for fn, ps in pts.items():
        path = os.path.normpath(os.path.join(build_dir, fn))
        if not os.path.exists(path):
            continue
        lines = open(path).read().split("\n")
        for ln, col in sorted(ps, reverse=True):
            row = lines[ln - 1]
            # idempotent: skip if already `let mut` on this line
            if row.lstrip().startswith("let mut "):
                continue
            lines[ln - 1] = row[:col - 1] + "mut " + row[col - 1:]
            n += 1
        open(path, "w").write("\n".join(lines))
    return n


def main():
    ap = argparse.ArgumentParser()
    ap.add_argument("--build-dir", default="sdk/lib/rust")
    ap.add_argument("--max-iter", type=int, default=6)
    a = ap.parse_args()
    total = 0
    for it in range(1, a.max_iter + 1):
        pts = collect(a.build_dir)
        cnt = sum(len(v) for v in pts.values())
        print(f"[iter {it}] E0384 let-mut sites~{cnt}")
        if cnt == 0:
            break
        applied = apply(a.build_dir, pts)
        print(f"[iter {it}] applied {applied}")
        total += applied
        if applied == 0:
            break
    print(f"freeldr_letmut: {total} fix(es)")


if __name__ == "__main__":
    main()
