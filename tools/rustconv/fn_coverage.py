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

# Words that begin a statement, not a definition, when they sit at column 0 before a `(`.
_NOT_A_FN = {
    "if", "else", "for", "while", "switch", "return", "do", "sizeof", "case", "default",
    "typedef", "struct", "union", "enum", "static", "extern", "const", "void", "break",
    "continue", "goto",
}

_C_COMMENT = re.compile(r"/\*.*?\*/", re.S)
_C_LINE_COMMENT = re.compile(r"//[^\n]*")
_CPP_DIRECTIVE = re.compile(r"^[ \t]*#.*(?:\\\n.*)*$", re.M)
# ReactOS style puts the return type on its own line, so a definition's NAME starts a line;
# the one-line form (`static BOOL Foo(`, `VOID WINAPI Bar(`) puts it last before the `(`.
_C_CANDIDATE = re.compile(r"^(?:[A-Za-z_][\w \t\*]*?[ \t\*])?([A-Za-z_]\w*)[ \t]*\(", re.M)


def _strip_if_zero(text: str) -> str:
    """Drop `#if 0 ... #endif` regions — the C's own "commented out" idiom.

    Not a preprocessor: only the unambiguous case is handled. This tool deliberately does NOT
    evaluate `#if`/`#ifdef` in general, so a function that exists only under a configuration
    this build does not select can still read as missing. That is the known residual false
    positive, and the reason a finding here is a REVIEW item rather than an automatic bug.
    """
    out, depth, lines = [], 0, text.splitlines(keepends=True)
    for line in lines:
        s = line.strip()
        if depth:
            if s.startswith("#if"):
                depth += 1
            elif s.startswith("#endif"):
                depth -= 1
            elif depth == 1 and s.startswith("#else"):
                depth = 0  # the live branch starts here
            continue
        if re.match(r"#\s*if\s+0\b", s):
            depth = 1
            continue
        out.append(line)
    return "".join(out)


def _after_params(text: str, open_paren: int) -> str:
    """The first meaningful characters after the parameter list that opens at `open_paren`."""
    depth = 0
    i = open_paren
    while i < len(text):
        if text[i] == "(":
            depth += 1
        elif text[i] == ")":
            depth -= 1
            if depth == 0:
                break
        i += 1
    else:
        return ""
    return text[i + 1 : i + 400].lstrip()


def c_functions(text: str) -> set[str]:
    """Names of functions DEFINED in this C translation unit.

    A name at column 0 followed by `(` is not enough: SAL annotations
    (`_Requires_lock_held_(..)`, `_Function_class_(..)`) and Wine's
    `WINE_DEFAULT_DEBUG_CHANNEL(..)` sit exactly there, immediately before the definition they
    decorate, and would otherwise be reported as functions the translation dropped. What
    separates a definition from a macro invocation is what FOLLOWS its parameter list: a body.
    A `;` is a prototype; anything else means the line was an annotation and the real
    definition is further down.
    """
    text = _C_COMMENT.sub("", text)
    text = _C_LINE_COMMENT.sub("", text)
    text = _strip_if_zero(text)
    text = _CPP_DIRECTIVE.sub("", text)
    names = set()
    for m in _C_CANDIDATE.finditer(text):
        n = m.group(1)
        if n in _NOT_A_FN:
            continue
        tail = _after_params(text, m.end() - 1)
        # A K&R-ish definition may carry a trailing attribute or annotation before its brace.
        if not tail.startswith("{"):
            continue
        # `CSR_API(SrvFoo)` / `CON_API(SrvBar, ..)` — an ALL-CAPS macro that EXPANDS to a
        # definition. The macro is followed by a body, so it passes the test above, but the
        # function the C actually defines is its first argument. winsrv writes every one of its
        # ~40 server routines this way.
        if n.isupper() and "_" in n:
            arg = re.match(r"\([ \t\n]*(\w+)", text[m.end() - 1 :])
            if arg:
                names.add(arg.group(1))
                continue
        names.add(n)
    return names


# rpc.h spells the MIDL allocator hooks lowercase and `#define`s them onto the uppercase pair
# the RPC runtime actually calls, so a crate that defines MIDL_user_free HAS covered
# midl_user_free. Without this the alias reads as a dropped function in every RPC server.
_C_ALIASES = {"midl_user_allocate": "MIDL_user_allocate", "midl_user_free": "MIDL_user_free"}

_RS_FN = re.compile(
    r"^[ \t]*(?:pub(?:\([^)]*\))?[ \t]+)?(?:unsafe[ \t]+)?(?:extern[ \t]+\"\w+\"[ \t]+)?fn[ \t]+(\w+)", re.M
)
_RS_MACRO = re.compile(r"^[ \t]*macro_rules![ \t]+(\w+)", re.M)


def rust_functions(paths: list[pathlib.Path]) -> set[str]:
    names: set[str] = set()
    for p in paths:
        t = p.read_text(encoding="utf-8", errors="replace")
        names |= {m.group(1) for m in _RS_FN.finditer(t)}
        names |= {m.group(1) for m in _RS_MACRO.finditer(t)}
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
    # The prelude legitimately holds lowered macros and CRT/inline stand-ins.
    substrate = [f for f in (src / "prelude.rs", src / "lib.rs") if f.exists()]
    rfns = rust_functions(rs_files + substrate)

    out: list[str] = []
    missing_total = 0
    invented: set[str] = set()
    still_c: list[str] = []
    for rs in rs_files:
        c = _pair(c_dir, rs)
        if c is None:
            continue
        if active is not None and c.name in active:
            still_c.append(c.name)  # cohabitation: the C is still compiled and still provides it
            continue
        ctext = c.read_text(encoding="utf-8", errors="replace")
        cfns = c_functions(ctext)
        own = rust_functions([rs])
        # For the reverse direction, "no C DEFINITION" is far too loose: a header macro lowered
        # to a fn (ARGUMENT_PRESENT, Add2Ptr) has no definition here either, and on a big driver
        # that is hundreds of rows of noise. What actually indicates an INVENTED callee is a name
        # the C never mentions at all — as a definition, a call, or a macro.
        ctokens = set(re.findall(r"\b\w+\b", ctext))
        own = {n for n in own if n not in ctokens}
        missing = sorted(
            n
            for n in cfns - rfns
            # a variadic entry point becomes a `_v` twin plus a C trampoline
            if f"{n}_v" not in rfns and _C_ALIASES.get(n) not in rfns
        )
        invented |= own - cfns
        if missing:
            missing_total += len(missing)
            out.append(f"  MISSING from {rs.relative_to(crate)} (defined in {c.name}):")
            out += [f"    {n}" for n in missing]
    if still_c:
        out.append(f"  ({len(still_c)} TU(s) still compiled as C, not audited: {', '.join(still_c)})")
    extra = sorted(n for n in invented if not n.endswith("_v"))
    if extra:
        out.append(f"  Rust fns with no C counterpart ({len(extra)}) — helper or INVENTED, review:")
        out += [f"    {n}" for n in extra]
    return missing_total, out


def main(argv: list[str]) -> int:
    ap = argparse.ArgumentParser(description=__doc__, formatter_class=argparse.RawDescriptionHelpFormatter)
    ap.add_argument("module", nargs="?", help="module name, e.g. aclui (implies the usual paths)")
    ap.add_argument("--c-dir", help="directory holding the module's .c files")
    ap.add_argument("--crate", help="the -rust crate directory")
    args = ap.parse_args(argv)

    repo = pathlib.Path(__file__).resolve().parents[2]
    if args.module and not (args.c_dir or args.crate):
        hits = [p.parent for p in repo.glob(f"*/*/{args.module}/CMakeLists.txt")]
        if not hits:
            print(f"cannot locate a source dir for '{args.module}'; pass --c-dir/--crate", file=sys.stderr)
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
