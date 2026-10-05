#!/usr/bin/env python3
"""Which archive members a module's link actually PULLS, and for which symbol -- from lld.

WHY THIS IS A TOOL
------------------
"Does anything still need library X?" decides whether X can come off a link line, and
reading source lists cannot answer it: an archive member is pulled only when something
references a symbol it defines first, and `--allow-multiple-definition` plus link order
decide which copy wins. lld knows: `--verbose` logs every member it loads and the symbol
that pulled it. Getting that meant hand-editing a ninja link command three times in one
session (msvcrt_static, crtmath, libm_sse2), so the recipe is this command instead. (Not
`-Map`: lld's COFF map names a member without its archive.)

It re-runs the module's real link step (from `ninja -t commands`, inside the canonical-path
mount the build used) with `-o` redirected to a scratch file and `-Wl,--verbose` added, so
the build's own artifact is never touched. Only the link segment of the command runs, never the
`&&`-chained post-link steps, which act on the real output path.

FLEET ISOLATION: links THIS worktree's build-<arch> only.

USAGE
    link_pulls.py --arch amd64 --target win32ss/gdi/gdi32/gdi32.dll
    link_pulls.py --arch amd64 --linking libcrtmath.a          # every module linking it
    link_pulls.py --arch amd64 --linking libcrtmath.a --match libm_sse2

Each row is `<archive or object> for <symbol>`. CMake's archives are thin, so their members
print as the object path (`sdk/lib/crt/CMakeFiles/atan2.dir/math/libm_sse2/atan2.c.obj`),
which names the library through its `<target>.dir`.
"""

from __future__ import annotations

import argparse
import re
import subprocess
import sys
import tempfile
from pathlib import Path

ROOT = Path(__file__).resolve().parent.parent
MODULE_EXT = r"(?:dll|exe|sys|cpl|ocx|drv|acm|ax|scr|efi|tlb|mui)"
#: `ld.lld: Loaded libfoo.a(member.o) for sym` or, from a thin archive, `Loaded path.obj for sym`.
LOADED = re.compile(r"Loaded (\S+) for (\S+)")


def linking(build: Path, archive: str) -> list[str]:
    """Every module whose link line names `archive` (by basename)."""
    out, target = [], None
    for line in (build / "build.ninja").read_text().splitlines():
        m = re.match(rf"^build (\S+\.{MODULE_EXT}):", line)
        if m:
            target = m.group(1)
        elif (
            target
            and line.lstrip().startswith(("LINK_LIBRARIES", "LINK_FLAGS"))
            and re.search(rf"(^|[ /]){re.escape(archive)}( |$)", line)
        ):
            out.append(target)
            target = None
    return sorted(set(out))


def link_step(build: Path, target: str) -> str:
    res = subprocess.run(
        ["ninja", "-C", str(build), "-t", "commands", target],
        capture_output=True,
        text=True,
    )
    lines = [ln for ln in res.stdout.splitlines() if ln.strip()]
    if res.returncode != 0 or not lines:
        sys.exit(f"link_pulls: no commands for {target}: {res.stderr.strip()[:500]}")
    steps = [s for s in lines[-1].split(" && ") if f" -o {target}" in f"{s} "]
    if len(steps) != 1:
        sys.exit(f"link_pulls: cannot isolate the link step of {target} ({len(steps)} candidates)")
    return steps[0]


def pulls(arch: str, target: str, scratch: Path) -> list[tuple[str, str]]:
    """`(loaded input, symbol it was loaded for)`, in lld's load order."""
    build = ROOT / f"build-{arch}"
    out = scratch / Path(target).name
    cmd = link_step(build, target).replace(f" -o {target}", f" -o {out} -Wl,--verbose", 1)
    script = scratch / "link.sh"
    script.write_text(f"cd /tmp/vibeos/build-{arch} && {cmd}\n")
    res = subprocess.run(
        ["bash", str(ROOT / "tools" / "canon-run.sh"), "bash", str(script)],
        cwd=build,
        capture_output=True,
        text=True,
    )
    loaded = LOADED.findall(res.stdout + res.stderr)
    if res.returncode != 0 or not out.is_file() or not loaded:
        # No `Loaded` line at all means the flag did not reach lld, not that nothing was pulled.
        sys.exit(
            f"link_pulls: relinking {target} failed or logged no loads (exit {res.returncode}):\n"
            f"{(res.stdout + res.stderr).strip()[-2000:]}"
        )
    return loaded


def main() -> int:
    ap = argparse.ArgumentParser(
        description=__doc__, formatter_class=argparse.RawDescriptionHelpFormatter
    )
    ap.add_argument("--arch", required=True, choices=["i386", "amd64", "arm64"])
    group = ap.add_mutually_exclusive_group(required=True)
    group.add_argument("--target", action="append", help="module output path, as ninja names it")
    group.add_argument("--linking", help="every module whose link line names this archive")
    ap.add_argument("--match", help="report only loads whose input path contains this")
    args = ap.parse_args()

    build = ROOT / f"build-{args.arch}"
    if not (build / "build.ninja").is_file():
        sys.exit(
            f"link_pulls: {build} has no build.ninja.\n  fix: ./vibeos-cli build --arch {args.arch}"
        )
    targets = args.target or linking(build, args.linking)
    if not targets:
        print(f"no module links {args.linking}")
        return 0
    with tempfile.TemporaryDirectory(prefix="link_pulls-") as tmp:
        for target in targets:
            # A module in build.ninja that the default build never produces (its objects
            # were never compiled) has no link to replay; say so rather than fail the sweep.
            if not (build / target).is_file():
                print(f"{target}: not built here, skipped")
                continue
            rows = [
                (what, sym)
                for what, sym in pulls(args.arch, target, Path(tmp))
                if args.match is None or args.match in what
            ]
            print(f"{target}: {len(rows)} load(s)")
            for what, sym in rows:
                print(f"    {what}  for {sym}")
    return 0


if __name__ == "__main__":
    sys.exit(main())
