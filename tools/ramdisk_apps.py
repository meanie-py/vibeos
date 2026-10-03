#!/usr/bin/env python3
"""ramdisk_apps.py -- the stock apps for the arm64 ramdisk, with their whole import closure.

The x86 livecds carry every app the tree builds; the arm64 QEMU ramdisk was hand-curated in
qemu/mkramdisk.sh and carried notepad, cmd and explorer, so appstorm reported 36 of its 43
entries ABSENT on arm64 and app bugs seen on the Pi had no QEMU repro. The roots now live in
qemu/ramdisk-apps.txt as livecd destinations, and everything else is derived:

  * WHERE a module goes and WHICH build output it is come from the livecd manifest
    (build-arm64/boot/livecd.<config>.lst, generated at configure time), so a module is
    resolvable, and nameable as a ninja target, before it has ever been built.
  * WHAT a root needs is the same closure the SysWOW64 staging uses (tools/fex/wow64-closure.py):
    import table, delay imports, the INF-declared runtime loaders, and module names in a
    root's string literals. A partial closure leaves a module half-bound and crashing at its
    first call, so an unresolvable name is a hard failure, never a skip.

Modes:
  --targets   build-relative paths ninja must build (roots, plus closure members not built
              yet). `vibeos-cli build` repeats this until it prints nothing new, since each
              round of built modules can reveal the next round's imports.
  --stage     a `dest=src` graft list for mkramdisk.sh (strip_cd_media.py --lst reads it).
              Modules mkramdisk.sh already stages by hand are left out: the curated copy
              wins. Fails if any closure member is unbuilt or not in the manifest.
"""

from __future__ import annotations

import argparse
import importlib.util
import re
import sys
from pathlib import Path

REPO = Path(__file__).resolve().parent.parent
CANON = "/tmp/vibeos/"

_spec = importlib.util.spec_from_file_location("wow64_closure", REPO / "tools/fex/wow64-closure.py")
assert _spec and _spec.loader
closure = importlib.util.module_from_spec(_spec)
_spec.loader.exec_module(closure)


def roots_list(path: Path) -> list[str]:
    out = []
    for line in path.read_text().splitlines():
        line = line.split("#", 1)[0].strip()
        if line:
            out.append(line)
    return out


def manifest_path(build_dir: Path) -> Path:
    """The generator-evaluated livecd list. livecd.cmake.lst still holds $<TARGET_FILE:...>
    expressions; file(GENERATE) writes the evaluated one as livecd.<config>.lst."""
    found = sorted(
        p for p in (build_dir / "boot").glob("livecd.*.lst") if p.name != "livecd.cmake.lst"
    )
    found = [p for p in found if ".stripped." not in p.name]
    if len(found) != 1:
        sys.exit(
            f"ramdisk_apps: expected one evaluated livecd.<config>.lst in {build_dir}/boot, "
            f"found {[p.name for p in found]} -- configure it: ./vibeos-cli build --arch arm64"
        )
    return found[0]


def load_manifest(build_dir: Path) -> dict[str, tuple[str, Path]]:
    """Lowercase module name -> (preferred dest, build output path).

    A module listed at several dests (notepad.exe in reactos/ and reactos/system32/) keeps
    its system32 one: that is where the loader's search path finds a DLL."""
    mods: dict[str, tuple[str, Path]] = {}
    for line in manifest_path(build_dir).read_text().splitlines():
        if "=" not in line:
            continue
        dest, src = line.split("=", 1)
        if not src.startswith(CANON):
            continue
        name = Path(dest).name.lower()
        if not name.endswith((".exe", *closure.PE_SUFFIXES)):
            continue
        path = build_dir.parent / src[len(CANON) :]
        old = mods.get(name)
        if old is None or (
            dest.startswith("reactos/system32/") and not old[0].startswith("reactos/system32/")
        ):
            mods[name] = (dest, path)
    return mods


def hand_staged(repo: Path) -> set[str]:
    """Module names mkramdisk.sh copies into ::/reactos/system32 itself."""
    text = (repo / "qemu/mkramdisk.sh").read_text(errors="ignore")
    return {
        m.lower() for m in re.findall(r"::/reactos/system32/([A-Za-z0-9_.\-]+)\s*$", text, re.M)
    }


def compute(build_dir: Path, roots_file: Path):
    """(closure: name -> (dest, path), unbuilt: names in the manifest with no file yet,
    unknown: name -> who wanted it, for names the manifest does not list at all)."""
    mods = load_manifest(build_dir)
    objdump = closure.find_objdump(REPO)
    # The literal scan's index: only modules that exist, keyed the way string_referenced wants.
    index = {n: p for n, (_d, p) in mods.items() if p.is_file()}
    out: dict[str, tuple[str, Path]] = {}
    unbuilt: set[str] = set()
    unknown: dict[str, set[str]] = {}
    stack: list[Path] = []

    def admit(name: str, why: str) -> None:
        if name in out or name in unbuilt or name in unknown:
            if name in unknown:
                unknown[name].add(why)
            return
        if name not in mods:
            unknown[name] = {why}
            return
        dest, path = mods[name]
        if not path.is_file():
            unbuilt.add(name)
            return
        out[name] = (dest, path)
        stack.append(path)
        spec = closure.DYNAMIC_LOADERS.get(name)
        if spec:
            for target in sorted(closure.declared_modules(REPO, *spec)):
                admit(target, f"{name} (INF-declared runtime load)")

    for dest in roots_list(roots_file):
        name = Path(dest).name.lower()
        if name not in mods:
            unknown.setdefault(name, set()).add(str(roots_file.relative_to(REPO)))
            continue
        admit(name, "root")
        if name in out:
            for lit in sorted(closure.string_referenced(out[name][1], index)):
                admit(lit, f"{name} (string literal)")
    while stack:
        cur = stack.pop()
        for dep in closure.imports_of(objdump, cur) + closure.delay_imports_of(cur):
            admit(dep, cur.name)
    return out, {n: mods[n] for n in unbuilt}, unknown


def main() -> int:
    ap = argparse.ArgumentParser(description=__doc__.split("\n\n")[0])
    mode = ap.add_mutually_exclusive_group(required=True)
    mode.add_argument("--targets", action="store_true")
    mode.add_argument("--stage", action="store_true")
    ap.add_argument("--build-dir", type=Path, default=REPO / "build-arm64")
    ap.add_argument("--roots", type=Path, default=REPO / "qemu/ramdisk-apps.txt")
    args = ap.parse_args()
    build_dir = args.build_dir.resolve()

    out, unbuilt, unknown = compute(build_dir, args.roots.resolve())
    if unknown:
        for name, why in sorted(unknown.items()):
            print(
                f"ramdisk_apps: {name} is not in the livecd manifest "
                f"(wanted by {', '.join(sorted(why))})",
                file=sys.stderr,
            )
        print(
            "ramdisk_apps: refusing a PARTIAL closure -- a module whose import is missing loads "
            "half-bound and crashes at its first call",
            file=sys.stderr,
        )
        return 1
    if args.targets:
        for _name, (_dest, path) in sorted(unbuilt.items()):
            print(path.relative_to(build_dir))
        return 0
    if unbuilt:
        for name, (_dest, path) in sorted(unbuilt.items()):
            print(f"ramdisk_apps: {name} is not built ({path})", file=sys.stderr)
        print("ramdisk_apps: build it first: ./vibeos-cli build --arch arm64", file=sys.stderr)
        return 1
    skip = hand_staged(REPO)
    staged = {n: v for n, v in out.items() if n not in skip}
    for _name, (dest, path) in sorted(staged.items()):
        print(f"{dest}={path}")
    total = sum(p.stat().st_size for _d, p in staged.values())
    print(
        f"ramdisk_apps: {len(staged)} module(s), {total / 1e6:.1f} MB before strip", file=sys.stderr
    )
    return 0


if __name__ == "__main__":
    sys.exit(main())
