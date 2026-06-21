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
import json
import os
import shlex
import subprocess

# (rc-stem, rc-path, cmake _res target, consumer data dir)
CELLS = [
    (
        "basesrv",
        "subsystems/win/basesrv/basesrv.rc",
        "basesrv_res",
        "sdk/lib/rust/basesrv-cargo/data",
    ),
    (
        "services",
        "base/system/services/services.rc",
        "services_res",
        "sdk/lib/rust/services-cargo/data",
    ),
    (
        "userinit",
        "base/system/userinit/userinit.rc",
        "userinit_res",
        "sdk/lib/rust/userinit-cargo/data",
    ),
    (
        "winlogon",
        "base/system/winlogon/winlogon.rc",
        "winlogon_res",
        "sdk/lib/rust/winlogon-cargo/data",
    ),
]


def gen(arch):
    db_path = f"build-{arch}/compile_commands.json"
    if not os.path.exists(db_path):
        print(f"  [{arch}] no {db_path} — configure+build that arch first; skipping")
        return
    with open(db_path) as f:
        db = json.load(f)
    for stem, rc, target, datadir in CELLS:
        rc_base = rc.split("/")[-1]
        hits = [e for e in db if e["file"].endswith(rc_base) and f"{target}.dir" in e["command"]]
        if not hits:
            print(f"  [{arch}] {stem}: NO compile_commands entry")
            continue
        e = hits[0]
        os.makedirs(datadir, exist_ok=True)
        out = os.path.abspath(f"{datadir}/{stem}-{arch}.rc.res")
        toks = shlex.split(e["command"])
        toks[-1] = out  # windres output is the last positional token
        r = subprocess.run(toks, cwd=e["directory"])
        ok = r.returncode == 0 and os.path.exists(out)
        print(f"  [{arch}] {stem}: {'OK' if ok else 'FAIL'} -> {datadir}/{stem}-{arch}.rc.res")


def main():
    ap = argparse.ArgumentParser()
    ap.add_argument("--arch", action="append", help="arch(es) to regen (default: i386 amd64)")
    args = ap.parse_args()
    for arch in args.arch or ["i386", "amd64"]:
        gen(arch)


if __name__ == "__main__":
    main()
