#!/usr/bin/env python3
"""Regenerate the committed compile-flag templates for cmake-off `c_from_compile_db`.

`reactos-link`'s `c_from_compile_db("<target>", ...)` replays cmake's recorded
clang command per source — which needs a configured `build-<arch>/
compile_commands.json`. For the cmake-off path (cargo build with no cmake
configure/ninja), `reactos-link` falls back to a committed per-arch flag template;
this script (re)generates those templates from a configured build.

Targets that use it (the `c_from_compile_db` consumers whose C is source-tree):
  - `blrtl`    — the boot-RTL floor for rosload/uefildr
  - `libcntpr` — the cntpr C floor (math/printf/string) the loaders reuse

The template is byte-faithful by construction: it is cmake's exact recorded
command for that target with the *inert* bits stripped — the ninja PCH
force-include (`-Xclang -include-pch ...` + the `-include cmake_pch.h` header,
droppable since each .c already `#include`s its real precomp header) and the
`build-<arch>/...` `-I` paths (the one generated header the closure needs,
`psdk/winnt.h`, is regenerated in-build by native-hpp). The compiler, source, and
`-o` are supplied by the engine; the template holds only the flags, one token per
line, with the repo root abstracted to `{REPO}`. Objects built from it are
instruction/symbol/reloc identical to cmake's (only `.debug_*` comp_dir differs).

Run from the repo root after a configured build for each arch:

    python3 tools/gen_cflags.py                          # blrtl + libcntpr, i386+amd64
    python3 tools/gen_cflags.py --target blrtl --arch arm64

A CI diff-check (committed vs freshly-generated) catches drift cheaply.
"""

import argparse
import json
import os
import shlex

OUTDIR = "sdk/lib/rust/reactos-link/cflags"
# The c_from_compile_db targets whose C sources are source-tree (so a committed
# flag template + native-hpp xdk headers fully reproduce cmake's compile).
DEFAULT_TARGETS = ["blrtl", "libcntpr"]


def candidate_flags(toks, arch, repo):
    """cmake command tokens -> flag-only template tokens (PCH + build-dir -I +
    compiler + source + -o stripped; repo root -> {REPO})."""
    out = []
    i = 1  # skip toks[0] = the compiler path
    while i < len(toks):
        t = toks[i]
        # PCH machinery — cmake emits BOTH the precompiled binary
        # (`-Xclang -include-pch -Xclang <.pch>`) and the header force-include
        # (`-Xclang -include -Xclang <cmake_pch.h>`). The header just `#include`s
        # the target's real precomp header, which every .c already does, so
        # dropping all of it is a no-op for codegen (only diagnostics differ).
        # `-Xclang` / `-include` appear ONLY for PCH in these commands.
        if t in ("-Winvalid-pch", "-Xclang", "-include", "-include-pch"):
            i += 1
            continue
        if "cmake_pch" in t:  # the .pch binary AND the cmake_pch.h header
            i += 1
            continue
        if t.startswith("-I") and f"/build-{arch}/" in t:
            i += 1
            continue
        if t == "-o":
            i += 2
            continue
        if t.endswith(".c") and os.path.isabs(t):
            i += 1  # the source positional (engine appends it)
            continue
        out.append(t.replace(repo, "{REPO}"))
        i += 1
    return out


def gen(target, arch, repo):
    db_path = f"build-{arch}/compile_commands.json"
    if not os.path.exists(db_path):
        print(f"  [{target} {arch}] no {db_path} — configure+build that arch first; skipping")
        return
    with open(db_path) as fh:
        db = json.load(fh)
    marker = f"CMakeFiles/{target}.dir"
    # The cmake_pch.h.c TU (cmake's PCH builder) carries different flags and is
    # not a real source — exclude it.
    hits = [
        e
        for e in db
        if marker in e["command"] and e["file"].endswith(".c") and "cmake_pch" not in e["file"]
    ]
    if not hits:
        print(f"  [{target} {arch}] no entries")
        return
    templates = {}
    for e in hits:
        flags = candidate_flags(shlex.split(e["command"]), arch, repo)
        templates[e["file"]] = flags
    # Invariant: all of the target's C sources share one flag set.
    uniq = {tuple(v) for v in templates.values()}
    if len(uniq) != 1:
        print(f"  [{target} {arch}] WARNING: {len(uniq)} distinct flag sets across sources")
        for f, v in templates.items():
            print(f"    {f}: {len(v)} flags")
        return
    flags = next(iter(uniq))
    os.makedirs(OUTDIR, exist_ok=True)
    out = f"{OUTDIR}/{target}.{arch}.flags"
    with open(out, "w") as fh:
        fh.write("\n".join(flags) + "\n")
    print(f"  [{target} {arch}] OK -> {out} ({len(flags)} flags, {len(hits)} sources)")


def main():
    repo = os.getcwd()
    ap = argparse.ArgumentParser()
    ap.add_argument("--target", action="append", help=f"target(s) (default: {DEFAULT_TARGETS})")
    ap.add_argument("--arch", action="append", help="arch(es) (default: i386 amd64)")
    args = ap.parse_args()
    for target in args.target or DEFAULT_TARGETS:
        for arch in args.arch or ["i386", "amd64"]:
            gen(target, arch, repo)


if __name__ == "__main__":
    main()
