#!/usr/bin/env python3
"""Regenerate the committed .res files for the cmake-off resource fallback (WS3).

reactos-link's `resource_from_rc` prefers cmake's windres (when a build dir is
configured) and falls back to a committed byte-stable `.res` when there is none
(the cmake-off path). This script (re)generates those committed copies by
replaying the EXACT windres command cmake records in compile_commands.json — so
the committed copy is byte-identical to what the live path produces.

Run from the repo root after a configured `ninja` for each arch you have a build
dir for:

    python3 tools/gen_committed_res.py            # i386 + amd64 (whatever build-* exist)
    python3 tools/gen_committed_res.py --arch arm64

Convention: <consumer crate>/data/<rc-stem>-<arch>.rc.res
(matches reactos-link's committed-.res lookup).
"""

import argparse
import contextlib
import io
import json
import os
import shlex
import subprocess

# (rc-stem, rc-path) — the cmake `<stem>_res` target and the consumer data dir
# `sdk/lib/rust/<stem>-cargo/data` are both pure functions of the stem.
CELLS = [
    ("basesrv", "subsystems/win/basesrv/basesrv.rc"),
    ("services", "base/system/services/services.rc"),
    ("userinit", "base/system/userinit/userinit.rc"),
    ("winlogon", "base/system/winlogon/winlogon.rc"),
]


def committed_path(stem, arch):
    """The committed .res path for a (cell, arch)."""
    return f"sdk/lib/rust/{stem}-cargo/data/{stem}-{arch}.rc.res"


def res_jobs(arch):
    """Yield (stem, committed_rel, toks, cwd) per cell for a configured build,
    where running `toks` in `cwd` with toks[-1] replaced by a dest path emits the
    .res. None when the build dir is absent (skip). Single source of truth shared
    by `gen()` (writes to the committed path) and the drift-check (writes to a
    temp + diffs)."""
    db_path = f"build-{arch}/compile_commands.json"
    if not os.path.exists(db_path):
        return None
    with open(db_path) as f:
        db = json.load(f)
    jobs = []
    for stem, rc in CELLS:
        target = f"{stem}_res"
        rc_base = rc.split("/")[-1]
        hits = [e for e in db if e["file"].endswith(rc_base) and f"{target}.dir" in e["command"]]
        if not hits:
            print(f"  [{arch}] {stem}: NO compile_commands entry")
            continue
        e = hits[0]
        jobs.append((stem, committed_path(stem, arch), shlex.split(e["command"]), e["directory"]))
    return jobs


def gen(arch):
    jobs = res_jobs(arch)
    if jobs is None:
        print(f"  [{arch}] no build-{arch}/compile_commands.json — build that arch first; skipping")
        return
    for stem, committed_rel, toks, cwd in jobs:
        out = os.path.abspath(committed_rel)
        os.makedirs(os.path.dirname(out), exist_ok=True)
        toks = list(toks)
        toks[-1] = out  # windres output is the last positional token
        r = subprocess.run(toks, cwd=cwd)
        ok = r.returncode == 0 and os.path.exists(out)
        print(f"  [{arch}] {stem}: {'OK' if ok else 'FAIL'} -> {committed_rel}")


def main():
    ap = argparse.ArgumentParser()
    ap.add_argument("--arch", action="append", help="arch(es) to regen (default: i386 amd64 arm64)")
    ap.add_argument(
        "--jobs-json", action="store_true", help="emit read-only regeneration recipes as JSON"
    )
    args = ap.parse_args()
    if args.jobs_json:
        if not args.arch or len(args.arch) != 1:
            ap.error("--jobs-json requires exactly one --arch")
        messages = io.StringIO()
        with contextlib.redirect_stdout(messages):
            jobs = res_jobs(args.arch[0])
        print(json.dumps({"jobs": jobs, "messages": messages.getvalue()}))
        return
    # NOTE: arm64 is in the default list. It was not, while check_committed_drift.py has
    # always CHECKED arm64 — so a regeneration after any input change left arm64 stale and
    # only the drift checker noticed, one gate later. Found during the 2026-09-20 upstream
    # merge, when every committed artifact went stale at once.
    for arch in args.arch or ["i386", "amd64", "arm64"]:
        gen(arch)


if __name__ == "__main__":
    main()
