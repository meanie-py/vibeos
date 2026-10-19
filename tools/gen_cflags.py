#!/usr/bin/env python3
"""Regenerate the committed geninc flag template, `cflags/genincdata.<arch>.flags`.

`reactos-link`'s `generate_ks_inc` self-hosts geninc: it compiles `genincdata.c` with these
flags and reads the `.asmdef` section out of the object. The template is cmake's recorded
command for `genincdata` with the *inert* bits stripped -- the ninja PCH force-include and
the `build-<arch>/...` `-I` paths (the generated headers it needs are made in-build by
native-hpp) -- and the repo root abstracted to `{REPO}`, one token per line.

The libraries `c_from_compile_db` names (blrtl, libcntpr, win32k_residue) once had
templates here too; a build with no cmake now compiles them from their recipes
(`tools/cargo_cell_gen.py --clib-recipes`).

Run from the repo root after a configured build for each arch:

    python3 tools/gen_cflags.py
    python3 tools/gen_cflags.py --arch arm64

`vibeos-cli check`'s drift lint compares the committed copy with a fresh one.
"""

import argparse
import json
import os
import shlex

OUTDIR = "sdk/lib/rust/reactos-link/cflags"
DEFAULT_TARGETS = ["genincdata"]

# The canonical build mountpoint (tools/canon-run.sh). Builds run at /tmp/vibeos
# via a bind mount, so compile_commands.json bakes /tmp/vibeos paths regardless
# of the real worktree location. The committed template must be portable, so
# BOTH the real repo root AND this canonical root templatize to {REPO} — else a
# canonical-built tree reads every -I flag as DRIFT vs the committed {REPO} copy
# (a false-positive that silently broke the committed_drift guard; 2026-07-04).
CANON_ROOT = "/tmp/vibeos"


def candidate_flags(toks, arch, repo):
    """cmake command tokens -> flag-only template tokens (PCH + build-dir -I +
    compiler + source + -o stripped; repo root -> {REPO}).

    Both the real `repo` root and the canonical build root (CANON_ROOT) map to
    {REPO}: a build may bake either depending on whether it ran under canon-run."""
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
        out.append(t.replace(repo, "{REPO}").replace(CANON_ROOT, "{REPO}"))
        i += 1
    return out


def committed_path(target, arch):
    """The committed flag template path for a (target, arch)."""
    return f"{OUTDIR}/{target}.{arch}.flags"


def gen_content(target, arch, repo):
    """Regenerate the flag-template TEXT for (target, arch) from a configured
    build, or None when the build dir / entries are absent (skip). This is the
    single source of truth shared by `gen()` (writes it) and the drift-check
    (compares it) — keep all generation logic here."""
    db_path = f"build-{arch}/compile_commands.json"
    if not os.path.exists(db_path):
        return None
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
        return None
    templates = {}
    for e in hits:
        flags = candidate_flags(shlex.split(e["command"]), arch, repo)
        templates[e["file"]] = flags
    # Invariant: all of the target's C sources share one flag set.
    uniq = {tuple(v) for v in templates.values()}
    if len(uniq) != 1:
        raise SystemExit(
            f"[{target} {arch}] {len(uniq)} distinct flag sets across sources — "
            "the single-flag-set invariant broke"
        )
    flags = next(iter(uniq))
    return "\n".join(flags) + "\n"


def gen(target, arch, repo):
    content = gen_content(target, arch, repo)
    if content is None:
        print(f"  [{target} {arch}] no build / entries — configure+build that arch first; skipping")
        return
    os.makedirs(OUTDIR, exist_ok=True)
    out = committed_path(target, arch)
    with open(out, "w") as fh:
        fh.write(content)
    print(f"  [{target} {arch}] OK -> {out} ({content.count(chr(10))} flags)")


def main():
    repo = os.getcwd()
    ap = argparse.ArgumentParser()
    ap.add_argument("--target", action="append", help=f"target(s) (default: {DEFAULT_TARGETS})")
    ap.add_argument("--arch", action="append", help="arch(es) (default: i386 amd64 arm64)")
    ap.add_argument(
        "--stdout", action="store_true", help="emit JSON generated content without writing"
    )
    args = ap.parse_args()
    if args.stdout:
        if not args.target or len(args.target) != 1 or not args.arch or len(args.arch) != 1:
            ap.error("--stdout requires exactly one --target and --arch")
        print(json.dumps(gen_content(args.target[0], args.arch[0], repo)))
        return
    for target in args.target or DEFAULT_TARGETS:
        # NOTE: arm64 is in the default list. It was not, while check_committed_drift.py has
        # always CHECKED arm64 — so a regeneration after any input change left arm64 stale and
        # only the drift checker noticed, one gate later. Found during the 2026-09-20 upstream
        # merge, when every committed artifact went stale at once.
        for arch in args.arch or ["i386", "amd64", "arm64"]:
            gen(target, arch, repo)


if __name__ == "__main__":
    main()
