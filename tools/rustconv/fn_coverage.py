#!/usr/bin/env python3
"""fn_coverage.py — does the translated crate define every function its C TUs define?

WHY THIS IS NOT REDUNDANT WITH THE PIPELINE. `translate` works from a PLAN, and the plan is
built by parsing the C. When the parse misses a function, the function is absent from the plan,
absent from the output, and absent from every downstream check — including the assembler's own
accounting, which can only reconcile against the plan it was given. aclui's `PaintControl`
went missing exactly this way: checklist.c closes an `#if SUPPORT_UXTHEME` between an `if` and
its body, which does not parse in one pass, so ~180 lines were silently dropped and nothing in
the pipeline had anything to compare against.

This pass has no such blind spot because its oracle is the C TEXT, not the plan. It is also the
only check that runs BEFORE the link: a private helper that is missing produces no unresolved
symbol at all (the callers went missing with it), so the linker is happy and the behaviour is
simply gone.

THE OTHER DIRECTION MATTERS TOO. A Rust fn with no C counterpart is either a legitimate helper
(a `_v` varargs twin, a macro lowered to a fn) or an INVENTED function — the model writing a
plausible callee that never existed. Both are listed; only the second is a bug, and a human
decides which it is.

    python tools/rustconv/fn_coverage.py aclui
    python tools/rustconv/fn_coverage.py --c-dir dll/win32/foo --crate sdk/lib/rust/foo-rust
"""

from __future__ import annotations

import argparse
import pathlib
import re
import sys

# The C-definition scanner lives in the package so export_audit can use the same oracle:
# an export with no Rust definition is only a defect if the C that defined it is no longer
# compiled. `_inside_if_dbg` is re-exported because the DBG bucket below is this tool's own.
from rustconv.c_defs import _inside_if_dbg, c_functions
from rustconv.function_coverage import rust_definitions

# A C name the linker never actually sees, because a macro renamed it. Each maps to the Rust
# name(s) that DO cover it; defining any one of them is enough.
_C_ALIASES = {
    # rpc.h spells the MIDL allocator hooks lowercase and `#define`s them onto the uppercase
    # pair the RPC runtime actually calls, so a crate that defines MIDL_user_free HAS covered
    # midl_user_free. Without this the alias reads as a dropped function in every RPC server.
    "midl_user_allocate": ("MIDL_user_allocate",),
    "midl_user_free": ("MIDL_user_free",),
    # tchar.h: `_tmain` IS `wmain` under UNICODE and `main` without it. Nothing named `_tmain`
    # ever reaches the linker, so a crate exporting either has covered the entry point.
    "_tmain": ("wmain", "main"),
    "_tWinMain": ("wWinMain", "WinMain"),
    # winbase.h: `#define GetEnvironmentStringsA GetEnvironmentStrings`. The C source spells
    # the A suffix, the object file carries the unsuffixed symbol, and the .spec forwards the
    # A export onto it — so the crate covers it by defining GetEnvironmentStrings.
    "GetEnvironmentStringsA": ("GetEnvironmentStrings",),
}


# Only parsed Rust definitions count; foreign declarations and comments do not.
def rust_functions(paths: list[pathlib.Path]) -> set[str]:
    names: set[str] = set()
    for p in paths:
        t = p.read_text(encoding="utf-8", errors="replace")
        names |= rust_definitions(t)
    return names


_CM_C_FILE = re.compile(r"([\w./-]+\.c)\b")


def active_c_tus(c_dir: pathlib.Path) -> set[str] | None:
    """Basenames of the .c files the CMakeLists still COMPILES, or None if it cannot be read.

    This is the difference between a gate and a noise generator. A cohab flip comments the
    translated TUs out of the source list and leaves the rest; a `<name>.c` that is still
    compiled provides its own functions, so "no Rust definition" is the expected, correct state
    for it. Only a TU that has been flipped OUT has to be covered by the crate — and for those,
    a function with no Rust definition is behaviour that has silently left the build.
    """
    cml = c_dir / "CMakeLists.txt"
    if not cml.is_file():
        return None
    active = set()
    for line in cml.read_text(encoding="utf-8", errors="replace").splitlines():
        code = line.split("#", 1)[0]
        for m in _CM_C_FILE.finditer(code):
            active.add(pathlib.PurePosixPath(m.group(1)).name)
    return active


def c_provided_names(c_dir: pathlib.Path, active: set[str]) -> set[str]:
    """Functions the module's STILL-COMPILED C provides, so the crate does not have to.

    A cohab flip routinely leaves one hand-written C TU behind ON PURPOSE: the C-variadic
    trampoline stable Rust cannot define (`rust_varargs_shim.c`, `acpi_printf_shim.c`). Its
    entry point was DEFINED in a .c that got flipped out — acpi's `AcpiOsPrintf` is in osl.c —
    so without this it reads as a dropped function forever, in every module with a shim.

    Scoped to the still-compiled TUs deliberately: a name only counts as provided if something
    the build actually compiles defines it.
    """
    names: set[str] = set()
    for c in sorted(c_dir.rglob("*.c")):
        if c.name in active:
            names |= c_functions(c.read_text(encoding="utf-8", errors="replace"))
    return names


def _pair(c_dir: pathlib.Path, rs: pathlib.Path) -> pathlib.Path | None:
    """The .c this .rs translates, searched recursively.

    A module can hold two TUs with the same basename in different subdirectories (winsrv has
    both usersrv/init.c and consrv/init.c). The crate mirrors that with matching module dirs, so
    when several candidates share a name the one under a like-named directory wins; a flat
    lookup would silently audit one Rust file against the other's C.
    """
    hits = sorted(c_dir.rglob(rs.stem + ".c"))
    if not hits:
        return None
    if len(hits) > 1:
        same = [h for h in hits if h.parent.name == rs.parent.name]
        if same:
            return same[0]
    return hits[0]


def audit(c_dir: pathlib.Path, crate: pathlib.Path) -> tuple[int, list[str]]:
    src = crate / "src"
    active = active_c_tus(c_dir)
    # The crate's module map mirrors the CMake source list one-for-one, so `<name>.rs` under the
    # module dir IS the translation of `<name>.c`. Pairing that way keeps the audit scoped to
    # what was actually translated instead of flagging files that were never in scope.
    mod_dirs = [d for d in src.iterdir() if d.is_dir()] or [src]
    rs_files = [f for d in mod_dirs for f in sorted(d.glob("*.rs"))]
    # COVERAGE reads the WHOLE crate, not just the paired module dirs. A crate with
    # subdirectories still keeps hand-written modules at src/ top level — kernel32's
    # interlocked.rs holds the four Interlocked* the spec exports — and scanning only the
    # module dirs reported every one of them as dropped while they sat one directory up.
    rfns = rust_functions(sorted(src.rglob("*.rs")))
    # A name the still-compiled C provides is covered, whichever language provides it.
    covered = rfns | (c_provided_names(c_dir, active) if active else set())

    out: list[str] = []
    missing_total = 0
    invented: set[str] = set()
    still_c: list[str] = []
    # Start at C, not existing Rust files: a whole missing translation must fail.
    for c in sorted(c_dir.rglob("*.c")):
        if active is not None and c.name in active:
            still_c.append(c.name)  # cohabitation: the C is still compiled and still provides it
            continue
        paired = [rs for rs in rs_files if _pair(c_dir, rs) == c]
        ctext = c.read_text(encoding="utf-8", errors="replace")
        cfns = c_functions(ctext)
        own = rust_functions(paired)
        # For the reverse direction, "no C DEFINITION" is far too loose: a header macro lowered
        # to a fn (ARGUMENT_PRESENT, Add2Ptr) has no definition here either, and on a big driver
        # that is hundreds of rows of noise. What actually indicates an INVENTED callee is a name
        # the C never mentions at all — as a definition, a call, or a macro.
        ctokens = set(re.findall(r"\b\w+\b", ctext))
        own = {n for n in own if n not in ctokens}
        missing = sorted(
            n
            for n in cfns - covered
            # a variadic entry point becomes a `_v` twin plus a C trampoline
            if f"{n}_v" not in covered and not set(_C_ALIASES.get(n, ())) & covered
        )
        invented |= own - cfns
        # A DBG-only body is a configuration, not a defect: the shipping build the smoke tests
        # gate on never compiled it. Listed, but out of the count that decides CLEAN.
        dbg_only = c_functions(_inside_if_dbg(ctext))
        dbg = [n for n in missing if n in dbg_only]
        missing = [n for n in missing if n not in dbg_only]
        if missing:
            missing_total += len(missing)
            out.append(f"  MISSING from {crate.name} (defined in {c.relative_to(c_dir)}):")
            out += [f"    {n}" for n in missing]
        if dbg:
            out.append(f"  #if DBG only in {c.name} (not in the shipping build): {', '.join(dbg)}")
    if still_c:
        out.append(
            f"  ({len(still_c)} TU(s) still compiled as C, not audited: {', '.join(still_c)})"
        )
    extra = sorted(n for n in invented if not n.endswith("_v"))
    if extra:
        out.append(f"  Rust fns with no C counterpart ({len(extra)}) — helper or INVENTED, review:")
        out += [f"    {n}" for n in extra]
    return missing_total, out


def main(argv: list[str]) -> int:
    ap = argparse.ArgumentParser(
        description=__doc__, formatter_class=argparse.RawDescriptionHelpFormatter
    )
    ap.add_argument("module", nargs="?", help="module name, e.g. aclui (implies the usual paths)")
    ap.add_argument("--c-dir", help="directory holding the module's .c files")
    ap.add_argument("--crate", help="the -rust crate directory")
    args = ap.parse_args(argv)

    repo = pathlib.Path(__file__).resolve().parents[2]
    if args.module and not (args.c_dir or args.crate):
        hits = [p.parent for p in repo.glob(f"*/*/{args.module}/CMakeLists.txt")]
        if not hits:
            print(
                f"cannot locate a source dir for '{args.module}'; pass --c-dir/--crate",
                file=sys.stderr,
            )
            return 2
        c_dir, crate = hits[0], repo / "sdk" / "lib" / "rust" / f"{args.module}-rust"
    elif args.c_dir and args.crate:
        c_dir, crate = pathlib.Path(args.c_dir), pathlib.Path(args.crate)
    else:
        ap.error("give a module name, or both --c-dir and --crate")

    if not crate.is_dir():
        print(f"no such crate: {crate}", file=sys.stderr)
        return 2
    n, lines = audit(c_dir, crate)
    print(f"{c_dir} -> {crate.name}")
    print("\n".join(lines) if lines else "  CLEAN — every C function has a Rust definition.")
    if n:
        print(f"\nFAIL: {n} C function(s) have no Rust definition.")
    return 1 if n else 0


if __name__ == "__main__":
    sys.exit(main(sys.argv[1:]))
