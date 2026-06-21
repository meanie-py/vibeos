#!/usr/bin/env python3
"""Regenerate the committed blrtl compile-flag templates (cmake-off WS1 DEEP).

The boot loaders (rosload/uefildr) compile the boot-RTL C floor
(`sdk/lib/rtl/{bitmap,compress,crc32,image,largeint,random,unicode,...}.c` +
`boot/rtl/nlsboot.c`) via `reactos-link`'s `c_from_compile_db("blrtl", ...)`,
which replays cmake's recorded clang command per source. That needs a configured
`build-<arch>/compile_commands.json`. For the cmake-off path (cargo build with no
cmake configure), `reactos-link` falls back to a committed per-arch flag template
— this script (re)generates those templates.

The template is byte-faithful by construction: it is cmake's exact recorded blrtl
command with the *inert* bits stripped — the ninja PCH force-include
(`-Xclang -include-pch ...`, droppable since each .c already `#include <rtl.h>`)
and the `build-<arch>/...` `-I` paths (empirically inert for blrtl — see the
roadmap WS1 DEEP note). The compiler, source, and `-o` are supplied by the engine;
the template holds only the flags, one token per line, with the repo root
abstracted to `{REPO}`. Objects built from it are instruction/symbol/reloc
identical to cmake's (only `.debug_*` comp_dir strings differ, runtime-irrelevant).

Run from the repo root after a configured build for each arch:

    python3 tools/gen_blrtl_flags.py            # i386 + amd64
    python3 tools/gen_blrtl_flags.py --arch arm64

A CI diff-check (committed vs freshly-generated) catches drift cheaply.
"""

import argparse
import json
import os
import shlex

OUTDIR = "sdk/lib/rust/reactos-link/cflags"
# One representative blrtl source per arch is enough — every file in the cmake
# `blrtl` target shares the same flags (only the source path / -o differ); the
# script asserts that invariant across all blrtl entries before writing.
MARKER = "blrtl.dir"


def candidate_flags(toks, arch, repo):
    """cmake command tokens -> flag-only template tokens (PCH + build-dir -I +
    compiler + source + -o stripped; repo root -> {REPO})."""
    out = []
    i = 1  # skip toks[0] = the compiler path
    while i < len(toks):
        t = toks[i]
        # PCH machinery — cmake emits BOTH the precompiled binary
        # (`-Xclang -include-pch -Xclang <.pch>`) and the header force-include
        # (`-Xclang -include -Xclang <cmake_pch.h>`). The header just
        # `#include`s rtl.h, which every blrtl .c already does, so dropping all
        # of it is a no-op for codegen (only diagnostics differ). `-Xclang` /
        # `-include` appear ONLY for PCH in the blrtl command.
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


def gen(arch, repo):
    db_path = f"build-{arch}/compile_commands.json"
    if not os.path.exists(db_path):
        print(f"  [{arch}] no {db_path} — configure+build that arch first; skipping")
        return
    with open(db_path) as fh:
        db = json.load(fh)
    # The cmake_pch.h.c TU (cmake's PCH builder) carries different flags and is
    # not a real blrtl source — exclude it.
    hits = [
        e
        for e in db
        if MARKER in e["command"] and e["file"].endswith(".c") and "cmake_pch" not in e["file"]
    ]
    if not hits:
        print(f"  [{arch}] no blrtl entries")
        return
    templates = {}
    for e in hits:
        flags = candidate_flags(shlex.split(e["command"]), arch, repo)
        templates[e["file"]] = flags
    # Invariant: all blrtl sources share one flag set.
    uniq = {tuple(v) for v in templates.values()}
    if len(uniq) != 1:
        print(f"  [{arch}] WARNING: {len(uniq)} distinct flag sets across blrtl sources")
        for f, v in templates.items():
            print(f"    {f}: {len(v)} flags")
        return
    flags = next(iter(uniq))
    os.makedirs(OUTDIR, exist_ok=True)
    out = f"{OUTDIR}/blrtl.{arch}.flags"
    with open(out, "w") as fh:
        fh.write("\n".join(flags) + "\n")
    print(f"  [{arch}] OK -> {out} ({len(flags)} flags, {len(hits)} sources)")


def main():
    repo = os.getcwd()
    ap = argparse.ArgumentParser()
    ap.add_argument("--arch", action="append", help="arch(es) (default: i386 amd64)")
    args = ap.parse_args()
    for arch in args.arch or ["i386", "amd64"]:
        gen(arch, repo)


if __name__ == "__main__":
    main()
