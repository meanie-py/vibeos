#!/usr/bin/env python3
"""How much of ucrtbase.dll is still Microsoft's, by export and by source file.

ucrtbase.dll is the last place vendored Microsoft C is the LIVE implementation
rather than inert flipped-out source: `sdk/lib/ucrt` is 455 .c/.cpp and 64,353
lines from the Windows SDK, and it is what answers the 15 `api-ms-win-crt-*`
apisets. The plan is to retire it the way every other module was retired -- our
Rust takes the bodies, the C leaves -- and a plan like that needs a number that
anyone can reproduce, not a number in a commit message.

So this reports, per arch:

    MICROSOFT   an export whose body is defined by a `sdk/lib/ucrt` object and
                by nothing of ours. The burn-down target.
    OURS        an export our side of the link line already defines. The Rust
                archive is whole-archived ahead of the rest, so a name here is
                answered by our code whatever ucrt also contains.
    UNBACKED    an export no object on the link line defines: a spec-generated
                stub or forward. 1,003 of these are `_o_*` aliases, which are
                the same body under a second name and cost nothing to provide.

WHY NOT just diff the two .spec files: because a spec says what is EXPORTED, not
whose body answers it, and the whole hazard here is that those differ silently.
ucrtbase links with `--allow-multiple-definition`; `verify_ucrtbase_cohab.py`
exists because putting msvcrt's bodies behind ucrtbase's exports builds, boots
and behaves subtly like MSVCRT forever, with no diagnostic. A count taken from
the spec would call that job finished. This one reads the objects.

Grouping by defining source file is the point of the report rather than a
nicety: it is what makes the work sliceable. A file whose every export has moved
to Rust can be deleted outright, and until then it cannot.

Needs a build (it reads `build-<arch>/`), and says so rather than guessing.
"""

from __future__ import annotations

import argparse
import collections
import re
import sys
from pathlib import Path

REPO = Path(__file__).resolve().parent.parent
sys.path.insert(0, str(Path(__file__).resolve().parent))

from verify_ucrtbase_cohab import (  # noqa: E402
    OURS_REFERENCE,
    _nm_defined,
    _spec_exports,
)

SPEC = REPO / "dll" / "win32" / "ucrtbase" / "ucrtbase.spec"
UCRT_OBJ_DIR = Path("sdk/lib/ucrt/CMakeFiles/ucrt.dir")
#: ucrtbase's own TUs, and the Rust archive, which live outside OURS_REFERENCE.
OWN_OBJ_DIR = Path("dll/win32/ucrtbase/CMakeFiles/ucrtbase.dir")
RUST_ARCHIVE = Path("dll/win32/ucrtbase/libcrt_shared_cargo.a")


def _undecorate(name: str) -> str:
    """i386 cdecl/stdcall decoration off, so `_malloc`/`_foo@8` compare to the
    spec's `malloc`/`foo`. amd64 and arm64 are undecorated already, and stripping
    a leading underscore there would corrupt genuinely-underscored names -- so
    this is applied to BOTH sides of every comparison or to neither, never one."""
    return name.lstrip("_").split("@")[0]


def classify(arch: str) -> tuple[dict[str, set[str]], dict[str, set[str]]]:
    """(buckets, ms_by_file) for one arch. Raises SystemExit if there is no build."""
    build = REPO / f"build-{arch}"
    if not build.is_dir():
        raise SystemExit(
            f"ucrt_burndown: no build tree at {build}\n  fix: ./vibeos-cli build --arch {arch}"
        )

    # The LINK LINE, not a glob of the object dir: since 2026-10-03 there is no ucrt
    # target, so a glob only ever finds objects a past build or a `--takeover-from`
    # sweep's pre-build left behind -- Microsoft C counted as live that nothing links.
    # No ucrt object on the link line means none is linked: MICROSOFT is 0, not an error.
    ninja = build / "build.ninja"
    if not ninja.is_file():
        raise SystemExit(f"ucrt_burndown: no {ninja}\n  fix: ./vibeos-cli build --arch {arch}")
    linked = set(re.findall(re.escape(str(UCRT_OBJ_DIR)) + r"/\S+?\.obj", ninja.read_text()))
    ucrt_objs = sorted(build / o for o in linked if (build / o).is_file())

    ours_paths: list[Path] = []
    for rel in OURS_REFERENCE.values():
        ours_paths += sorted((build / rel).rglob("*.obj"))
    ours_paths += sorted((build / OWN_OBJ_DIR).rglob("*.obj"))
    if (build / RUST_ARCHIVE).is_file():
        ours_paths.append(build / RUST_ARCHIVE)

    ours = _nm_defined(ours_paths)
    if ours is None:
        raise SystemExit("ucrt_burndown: no usable nm; cannot read the archives")
    ours = {_undecorate(s) for s in ours}

    ms_by_file: dict[str, set[str]] = collections.defaultdict(set)
    ms: set[str] = set()
    for obj in ucrt_objs:
        syms = _nm_defined(obj)
        if syms is None:
            continue
        src = str(obj.relative_to(build / UCRT_OBJ_DIR)).removesuffix(".obj")
        for s in {_undecorate(x) for x in syms}:
            ms.add(s)
            ms_by_file[src].add(s)

    spec = {_undecorate(s) for s in _spec_exports(SPEC, arch)}
    buckets = {
        "MICROSOFT": spec & ms - ours,
        "OURS": spec & ours,
        "UNBACKED": spec - ms - ours,
    }
    # Only the exports still owed, per file -- a file's internal helpers are not work.
    owed = {f: (s & buckets["MICROSOFT"]) for f, s in ms_by_file.items()}
    return buckets, {f: s for f, s in owed.items() if s}


def main() -> int:
    ap = argparse.ArgumentParser(description=__doc__)
    ap.add_argument("--arch", default="i386", choices=("i386", "amd64", "arm64"))
    ap.add_argument("--top", type=int, default=20, help="source files to list (0 = all)")
    ap.add_argument("--file", help="list the exports still owed by one source file")
    args = ap.parse_args()

    buckets, owed = classify(args.arch)
    total = sum(len(v) for v in buckets.values())

    if args.file:
        names = sorted(owed.get(args.file, ()))
        print(f"{args.file}: {len(names)} export(s) still Microsoft's")
        for n in names:
            print(f"    {n}")
        return 0

    print(f"ucrtbase.dll exports ({args.arch}): {total}")
    for k in ("MICROSOFT", "OURS", "UNBACKED"):
        n = len(buckets[k])
        print(f"  {k:<10} {n:>5}   {100 * n / total:5.1f}%")
    print()
    owe = sum(len(v) for v in owed.values())
    print(f"{len(owed)} source file(s) in sdk/lib/ucrt still own {owe} export(s):")
    rows = sorted(owed.items(), key=lambda kv: -len(kv[1]))
    run = 0
    for f, s in rows[: args.top or len(rows)]:
        run += len(s)
        print(f"  {len(s):>4}  {100 * run / max(owe, 1):5.1f}% cum   {f}")
    if args.top and len(rows) > args.top:
        print(f"  ... and {len(rows) - args.top} more file(s)")
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
