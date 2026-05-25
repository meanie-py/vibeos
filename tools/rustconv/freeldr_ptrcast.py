#!/usr/bin/env python3
"""freeldr_ptrcast — span-driven raw-pointer-signedness cast inserter.

rustconv declares freeldr scratch buffers as `[u8; N]` and emits
`b"..."`/`.as_ptr()`/`.as_mut_ptr()` args, but the C `CHAR` surface
(RtlStringCb*, ERR/TRACE/UiMessageBox, strstr, NtLdrGetOption, ...) is
`*const i8`/`*mut i8` in the keystone (CHAR == i8 on the GNU target).
That produces a large, purely-mechanical class of E0308:

    expected `*mut i8`,  found `*mut u8`
    expected `*const u8`, found `*const i8`     (and the reverse)

Every such site is fixed by wrapping the offending argument expression
in `(<expr> as <expected-type>)`. This reads rustc's JSON diagnostics
(exact byte spans) and rewrites each primary span bottom-up so earlier
edits don't shift later spans. Deterministic + idempotent (a span whose
text already ends in the exact `as <ty>` cast is skipped).

Usage:
  freeldr_ptrcast.py --build-dir sdk/lib/rust [--max-iter N]

Only touches files under freeldr-rust/src/. Re-run freely.
"""
import argparse
import json
import os
import re
import subprocess
import sys
from collections import defaultdict

PTR_RE = re.compile(
    r'expected `(\*(?:mut|const) [iu]8)`, found `(\*(?:mut|const) [iu]8)`')
TARGET = "i686-pc-windows-gnu"


def collect(build_dir):
    env = dict(os.environ, CARGO_TARGET_DIR="/tmp/fldr-winldr")
    p = subprocess.run(
        ["cargo", "build", "--release", "-p", "freeldr-rust",
         "--target", TARGET, "--message-format=json"],
        cwd=build_dir, env=env, capture_output=True, text=True)
    # file -> list of (line, col_start, col_end, expected_ty)
    sites = defaultdict(list)
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
        code = (msg.get("code") or {}).get("code")
        if code != "E0308":
            continue
        # The ptr-signedness mismatch can ride either a span label
        # ("mismatched types" single-arg case) OR a child message
        # ("arguments to this function are incorrect" multi-arg case,
        # where each bad arg is a separate child whose PRIMARY span is
        # the single-line argument expression). Handle both.
        def take(spans, exp):
            for sp in spans:
                if not sp.get("is_primary"):
                    continue
                if sp["line_start"] != sp["line_end"]:
                    continue  # single-line only (safe byte splice)
                sites[sp["file_name"]].append(
                    (sp["line_start"], sp["column_start"],
                     sp["column_end"], exp))
                return

        for sp in msg["spans"]:
            mm = PTR_RE.search(sp.get("label") or "")
            if mm and sp["line_start"] == sp["line_end"]:
                sites[sp["file_name"]].append(
                    (sp["line_start"], sp["column_start"],
                     sp["column_end"], mm.group(1)))
        for ch in msg.get("children", []):
            mm = PTR_RE.search(ch.get("message") or "")
            if mm:
                take(ch.get("spans", []), mm.group(1))
    return sites


def apply(build_dir, sites):
    n = 0
    for fn, lst in sites.items():
        if "freeldr-rust/src/" not in fn:
            continue
        path = os.path.join(build_dir, "..", "..", fn) \
            if not os.path.exists(os.path.join(build_dir, fn)) \
            else os.path.join(build_dir, fn)
        # the json file_name is relative to the cargo workspace (sdk/lib/rust)
        path = os.path.normpath(os.path.join(build_dir, fn))
        if not os.path.exists(path):
            continue
        lines = open(path).read().split("\n")
        # bottom-up (by line desc, then col desc) so splices are stable
        for ln, c0, c1, exp in sorted(set(lst), reverse=True):
            row = lines[ln - 1]
            seg = row[c0 - 1:c1 - 1]
            cast = f" as {exp}"
            # idempotent: already cast to exactly this type
            if seg.rstrip().endswith(cast.strip()):
                continue
            new = f"({seg}{cast})"
            lines[ln - 1] = row[:c0 - 1] + new + row[c1 - 1:]
            n += 1
        open(path, "w").write("\n".join(lines))
    return n


def main():
    ap = argparse.ArgumentParser()
    ap.add_argument("--build-dir", default="sdk/lib/rust")
    ap.add_argument("--max-iter", type=int, default=8)
    a = ap.parse_args()
    total = 0
    for it in range(1, a.max_iter + 1):
        sites = collect(a.build_dir)
        cnt = sum(len(v) for k, v in sites.items()
                  if "freeldr-rust/src/" in k)
        print(f"[iter {it}] ptr-signedness sites~{cnt}")
        if cnt == 0:
            break
        applied = apply(a.build_dir, sites)
        print(f"[iter {it}] applied {applied} cast(s)")
        total += applied
        if applied == 0:
            break
    print(f"freeldr_ptrcast: {total} cast(s) total")


if __name__ == "__main__":
    main()
