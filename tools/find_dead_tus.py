#!/usr/bin/env python3
"""Find compiled translation units in a link whose symbols nothing references.

A TU whose COM class went Rust-owned can keep sitting in CMake as pure dead
weight: its method bodies still compile, but the object map activates the Rust
class instead, so nothing links against them. The tail ledger keeps counting
every one of those lines as untranslated work, and the obvious next step looks
like "translate its remaining helpers" — money spent on dead code. CShellLink.cpp
was 3,346 such lines.

The test is link-level and needs no build beyond the objects already on disk: for
each object in the link, take the symbols it DEFINES and ask whether anything else
in the same link leaves any of them undefined. If nothing does, and no .spec
export names one, the object contributes nothing.

The live object set comes from the ninja link edge, NOT from globbing the build
tree: a previously-flipped TU leaves its stale .obj on disk, and globbing reports
every one of those as "dead" — true but useless, since they already left the
build.

This REPORTS; it never edits. Objects can be stale with respect to their source,
so a candidate is a hypothesis to be confirmed by flipping it and linking.

LIMIT, and it is a sharp one: this proves there is no LINK-TIME consumer, which is
not the same as unused. A TU whose symbols are reached through a data table, a
resource, an .inf, or a runtime name lookup has no link-time reference and will be
reported here. The `guid.c` TUs in the USB and storport drivers are exactly that
shape: device-interface GUIDs consumed by the OS, not by the link. Removing one
produces no link error and fails at runtime instead. Treat a candidate as safe only
when its symbols are CODE with a known replacement — CShellLink.cpp qualified
because its class was already Rust-owned and activation went through the Rust
factory.
"""

from __future__ import annotations

import argparse
import re
import subprocess
import shutil
import sys
from pathlib import Path

REPO = Path(__file__).resolve().parent.parent


def find_nm() -> str:
    local = REPO / ".build-tools/llvm-mingw/bin/llvm-nm"
    if local.exists():
        return str(local)
    if shutil.which("llvm-nm"):
        return "llvm-nm"
    sys.exit("no llvm-nm in .build-tools or on PATH")


def syms(nm: str, path: Path, defined: bool) -> set[str]:
    flag = "--defined-only" if defined else "-u"
    try:
        out = subprocess.run(
            [nm, flag, str(path)], capture_output=True, text=True, timeout=180
        ).stdout
    except (OSError, subprocess.TimeoutExpired):
        return set()
    names = set()
    for line in out.splitlines():
        parts = line.split()
        if not parts:
            continue
        name = parts[-1]
        if name.startswith(".refptr."):  # linker indirection stub, not a definition
            continue
        names.add(name)
    return names


def link_inputs(build: Path, target: str) -> tuple[list[Path], list[Path]]:
    """(objects, archives) named by the ninja edge that links `target`."""
    ninja = build / "build.ninja"
    if not ninja.is_file():
        sys.exit(f"{ninja} absent — configure the build first")
    edge = None
    for line in ninja.read_text(errors="replace").splitlines():
        if line.startswith("build ") and re.search(rf"(^|/){re.escape(target)}:", line):
            edge = line
            break
    if edge is None:
        sys.exit(f"no ninja link edge for {target}")
    # Inputs run until the order-only marker; implicit deps after `|` still link.
    body = edge.split(":", 1)[1].split("||")[0].replace("|", " ")
    objs, libs = [], []
    for tok in body.split():
        p = build / tok
        if tok.endswith((".obj", ".o")) and p.is_file():
            objs.append(p)
        elif tok.endswith(".a") and p.is_file():
            libs.append(p)
    return objs, libs


def undecorate(sym: str) -> set[str]:
    """Candidate source names for a symbol, for matching against a .spec.

    i386 adds ONE leading underscore for cdecl and a `@N` suffix for stdcall.
    Stripping every underscore is wrong: msvcrt exports `_CrtCheckMemory`, whose
    i386 symbol is `__CrtCheckMemory`, and lstrip("_") turns that into
    `CrtCheckMemory`, which matches nothing — reporting a live export stub as dead.
    """
    base = sym.split("@")[0]
    out = {base}
    if base.startswith("_"):
        out.add(base[1:])
    return out


def spec_names(spec: Path) -> set[str]:
    """Exported names from a .spec. Entries look like `183 varargs Foo(ptr long)`."""
    if not spec.is_file():
        return set()
    out = set()
    for line in spec.read_text(errors="replace").splitlines():
        line = line.split("#")[0]
        for tok in re.findall(r"[A-Za-z_][A-Za-z0-9_]*", line):
            out.add(tok)
    return out


def main() -> int:
    ap = argparse.ArgumentParser(description=__doc__)
    ap.add_argument("--build-dir", required=True, help="e.g. build-i386 (THIS worktree)")
    ap.add_argument("--target", required=True, help="linked artifact, e.g. shell32.dll")
    ap.add_argument("--spec", help="export .spec whose names count as consumers")
    ap.add_argument(
        "--min-symbols",
        type=int,
        default=10,
        help="ignore objects defining fewer than N symbols (default 10). A fully\n"
        "flipped module keeps an intentional one-line placeholder TU so cmake still\n"
        "has a source (rust_cohab_stub.c); those are by design, not dead weight.",
    )
    args = ap.parse_args()

    build = REPO / args.build_dir
    if not build.is_dir():
        sys.exit(
            f"{args.build_dir} is absent in THIS worktree.\n"
            f"fix: ./vibeos-cli build --arch <a>   (never read a sibling's build dir)"
        )

    objs, libs = link_inputs(build, args.target)
    nm = find_nm()
    print(f"link edge for {args.target}: {len(objs)} objects, {len(libs)} archives")

    defined = {o: syms(nm, o, True) for o in objs}
    undef = {o: syms(nm, o, False) for o in objs}
    lib_undef: set[str] = set()
    for lib in libs:
        lib_undef |= syms(nm, lib, False)
    exports = spec_names(REPO / args.spec) if args.spec else set()

    dead = []
    for o in objs:
        needed = set(lib_undef)
        for other, u in undef.items():
            if other is not o:
                needed |= u
        if defined[o] & needed:
            continue
        # An exported symbol has no in-link consumer but is still load-bearing.
        if any(undecorate(s) & exports for s in defined[o]):
            continue
        if len(defined[o]) < args.min_symbols:
            continue  # placeholder/trivial TU, not a translation target
        dead.append((o, len(defined[o])))

    if not dead:
        print("no dead objects: every TU in this link defines a referenced symbol")
        return 0
    print(f"\nDEAD CANDIDATES ({len(dead)}) — defined symbols with zero consumers:")
    for o, n in sorted(dead, key=lambda x: -x[1]):
        print(f"  {n:5d} symbols   {o.name}")
    print(
        "\nEach is a hypothesis: objects can be stale w.r.t. source. Confirm by\n"
        "flipping the TU out of CMake and LINKING."
    )
    return 0


if __name__ == "__main__":
    sys.exit(main())
