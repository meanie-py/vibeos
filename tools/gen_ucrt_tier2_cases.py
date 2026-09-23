#!/usr/bin/env python3
"""Generate the Tier-2 ucrt difftest cases: every export we can grade without a hand-written row.

The hand-written harness reached 17 families. This reaches 221 on i386, and it is the
221 that matter -- convert/, string/ and mbstring/ are exactly where the reuse question
lives (see docs/engineering/active-lanes/ucrt-tier2-difftest-design.md).

HOW THE DECLARATIONS ARE RIGHT BY CONSTRUCTION. The renamed copies are declared with
`extern __typeof__(foo) ref_foo;`, so their types come from the SAME header the bare
side is declared by. Nothing re-spells a signature, so nothing can drift from it, and
the ~188 declarations mentioning bare `long` -- the reason the host tier cannot grade
this family at all -- are simply whatever Windows says they are.

WHAT IS EXCLUDED, AND WHY BOTH FILTERS ARE NEEDED:

  * Not declared in the public headers (135 of 999): no prototype, no call.
  * Shape we have no driver for: anything returning or taking something other than a
    scalar, a string, or a `_locale_t`. FILE*, va_list, callbacks and out-params all
    need a hand-written row.
  * NOT from a side-effect-free source directory. This filter is not tidiness -- shape
    says nothing about behaviour, and the shape-gradable set contains `exit`, `system`,
    `_getch` and `abort`. Auto-calling those kills or hangs the harness. Purity cannot
    be read off a signature, so it is taken from the source file the burn-down already
    maps each export to.

Locale params are passed NULL (= the current locale). Both sides start in "C", so that
is consistent; a row that CHANGES locale would have to change it on both sides and is
not generated.
"""

from __future__ import annotations

import argparse
import collections
import json
import re
import subprocess
import sys
from pathlib import Path

REPO = Path(__file__).resolve().parent.parent
sys.path.insert(0, str(REPO / "tools"))

#: Public UCRT surface. A header missing here costs coverage, never correctness.
HEADERS = [
    "windows.h",
    "ctype.h",
    "wctype.h",
    "stdlib.h",
    "stdio.h",
    "string.h",
    "wchar.h",
    "time.h",
    "math.h",
    "locale.h",
    "mbstring.h",
    "mbctype.h",
    "process.h",
    "io.h",
    "direct.h",
    "errno.h",
    "float.h",
    "search.h",
    "signal.h",
    "conio.h",
    "fcntl.h",
    "malloc.h",
    "memory.h",
    "sys/stat.h",
    "sys/timeb.h",
    "sys/utime.h",
]

#: Source directories whose exports are pure computation over their arguments.
#: exec/, lowio/, conio/, stdio/, env/, filesystem/, heap/, startup/ and time/ are all
#: excluded -- they spawn, block, allocate, touch the filesystem or read the clock.
PURE_DIRS = ("convert/", "string/", "mbstring/", "math/", "stdlib/")

#: mbstring/ is pure by the same standard and is 96 more exports -- but it is EXCLUDED,
#: and the reason is the harness's real boundary rather than anything about those bodies.
#:
#: Our msvcrt Rust reaches per-thread state through `get_mbcinfo()` ->
#: `msvcrt_get_thread_data()`, which msvcrt.dll's DllMain sets up. The harness links a
#: PRIVATE copy for which no DllMain ever runs, so the pointer is null and dereferencing
#: it takes the whole process down -- silently, with no fault on serial. That is not one
#: bad export: `_ismbchira` ended a run at 41 output lines, and excluding it by name only
#: moved the wall to `_mbscoll_l` at 167. The dependency is transitive through helpers,
#: so a name list is whack-a-mole; the directory is the honest boundary.
#:
#: A body that dies hides every finding AFTER it, which is why this is drawn wide rather
#: than narrow. Lifting it needs the Rust side's per-thread state initialised in-harness
#: -- the "state isolation in practice" item the design doc still lists as unverified,
#: and the single highest-value thing to do next.

_INT_RET = {
    "int",
    "unsigned int",
    "long",
    "unsigned long",
    "short",
    "unsigned short",
    "char",
    "signed char",
    "unsigned char",
    "wint_t",
    "wchar_t",
    "size_t",
    "__int64",
    "long long",
    "unsigned long long",
    "unsigned __int64",
    "errno_t",
    "intptr_t",
    "uintptr_t",
}
#: Parameter kinds, keyed by the exact qualType clang prints.
_PARAM_KIND = {
    "_locale_t": "L",
    "const char *": "SN",
    # NOT `char *`. A non-const pointer is an OUT parameter, not an input string:
    # _wctomb_l WRITES through it, and feeding it a shared string literal would
    # corrupt the corpus for every later row (and is undefined besides). Out-params
    # need a hand-written row with a real buffer.
    "const wchar_t *": "SW",
    "const unsigned char *": "SU",
}
for _t in _INT_RET:
    _PARAM_KIND.setdefault(_t, "I")

_SIG = re.compile(r"^(.*?)\s*\((.*?)\)(?:\s*__attribute__.*)?$")

#: Classification predicates. C contracts a NONZERO value for a true `is*`, not a
#: specific one, and MSVC returns the raw ctype bitmask -- so `_ismbcgraph_l` answering
#: 258 where the reference answers 4 is the SAME answer. Comparing raw values here
#: manufactures failures that no caller could observe, which is worse than missing a
#: real one: it buries the real ones. Both sides are normalised to 0/1 instead.
_PREDICATE = re.compile(r"^_?(is[a-z]|isw|_?ismb|_?iswcsym)")

#: Exports excluded because the sweep cannot get past them.
#:
#: `_mbsnbcoll` was the first suspect and is NOT the cause -- excluding it (verified in
#: the generated file) did not move the stopping point at all. Kept only as a marker.
#:
#: WHAT THE WALL IS, AS FAR AS MEASURED. The sweep stops partway and never resumes, and
#: `--reverse` separated the explanations: FORWARD the old single-boot sweep stopped
#: after 114 exports, REVERSED after 53. So it was NOT a fixed count (114 != 53) and NOT
#: a single bad body (one would sit at reverse index ~106). Counting the corpus products
#: for those two prefixes killed the last cheap theory too: ~5,263 cases vs ~2,182, so
#: not a fixed case budget either.
#:
#: RESOLVED 2026-09-23, and it was two independent faults:
#:
#:   1. STALE PAYLOAD STAGING. The ramdisk is built incrementally and nothing removed a
#:      payload a later build did not select, so a kmtest-sweep.exe from 2026-09-22 sat
#:      in the All Users StartUp folder beside ucrtdiff.exe. Explorer autostarts that
#:      folder, so every Tier 2 run shared its boot and its serial channel with a full
#:      kernel test corpus. Fixed in qemu/mkramdisk.sh, which now purges every dest ANY
#:      payload could own (compile.py all-dests) BEFORE staging the selected set.
#:
#:   2. THE COLLATION FAMILY WEDGES THE WHOLE GUEST. With the media clean, shard 0/8 ran
#:      16 exports and died entering `_mbsnicoll`. It is not a hung thread: ALL serial
#:      stops, kernel [PS] spam included, so run_guarded's 8s WaitForSingleObject never
#:      returns because nothing in the guest runs again. That also defeats the harness's
#:      own deadline, which is cooperative and only advances between exports.
#:
#: The note that the guest "keeps logging, so the HARNESS is wedged" was wrong -- it was
#: read from a log that had stopped for an unrelated reason (see 1).
QUARANTINE_REASON = (
    "the multibyte/locale collation family wedges the ENTIRE GUEST, not just its "
    "thread: all serial output stops, so neither run_guarded's per-export timeout nor "
    "the harness's own deadline can fire. Quarantined to unblock the other ~194 "
    "exports; this is OWED WORK, not a decision that these are fine."
)


def is_quarantined(name: str) -> bool:
    """Exports held out of the sweep because grading them ends the boot.

    Every *coll* export routes through the same locale-collation path, and the two
    originally known deadlockers (_mbsnbcoll, _mbsnbcoll_l) are members of it. Rather
    than discover the rest one dead boot at a time -- each costs ~11 minutes and hides
    every export after it -- the whole family is held out and recorded as owed.
    """
    return "coll" in name


class GenError(RuntimeError):
    """Raised instead of emitting a harness that grades less than it claims."""


def ast_functions(arch: str, cache: Path | None = None) -> dict[str, str]:
    """name -> clang qualType, for every function the public UCRT headers declare."""
    if cache and cache.is_file():
        return json.loads(cache.read_text())
    cc = {
        "i386": "i686-w64-mingw32-clang",
        "amd64": "x86_64-w64-mingw32-clang",
        "arm64": "aarch64-w64-mingw32-clang",
    }[arch]
    tool = REPO / ".build-tools" / "llvm-mingw" / "bin" / cc
    if not tool.is_file():
        raise GenError(f"{tool} not found")
    src = "\n".join(f"#include <{h}>" for h in HEADERS) + "\n"
    tu = (cache.parent if cache else REPO) / f".ucrt-tier2-{arch}.c"
    tu.parent.mkdir(parents=True, exist_ok=True)
    tu.write_text(src)
    try:
        res = subprocess.run(
            [str(tool), "-fsyntax-only", "-Xclang", "-ast-dump=json", str(tu)],
            capture_output=True,
            text=True,
            timeout=1800,
        )
    finally:
        tu.unlink(missing_ok=True)
    if res.returncode != 0:
        raise GenError(f"clang could not parse the UCRT headers: {res.stderr[:1500]}")
    out: dict[str, str] = {}

    def walk(n):
        if isinstance(n, dict):
            if n.get("kind") == "FunctionDecl" and "name" in n:
                out.setdefault(n["name"], n.get("type", {}).get("qualType", ""))
            for c in n.get("inner", []) or []:
                walk(c)

    walk(json.loads(res.stdout))
    if cache:
        cache.write_text(json.dumps(out))
    return out


def shape(sig: str) -> list[str] | None:
    """Parameter kinds, or None if this signature has no driver.

    Returns e.g. ["I"], ["SN", "SN", "I", "L"]. The return type must be integer-like:
    the pure set contains no pointer returns, and a pointer return could not be compared
    across sides anyway (the two sides scan two different copies of the same literal).
    """
    m = _SIG.match(sig)
    if not m:
        return None
    ret, params = m.group(1).strip(), m.group(2).strip()
    if ret not in _INT_RET:
        return None
    if params in ("void", ""):
        return []
    kinds = []
    for p in (x.strip() for x in params.split(",")):
        k = _PARAM_KIND.get(p)
        if k is None:
            return None
        kinds.append(k)
    return kinds


def _renamed_exports(arch: str, lib: str, prefixes: tuple[str, ...]) -> set[str]:
    """Export names an archive carries a renamed copy of, read from the archive itself.

    THE NAME MUST COME FROM HERE, never from guessing decoration. `stricmp` and
    `_stricmp` are both real: the first is a deprecated oldname alias the headers also
    declare, and picking it produced calls to `ref_stricmp`, which nothing defines
    because the refset renamed the export `_stricmp`. The archive knows which one it
    renamed; nothing else does.
    """
    path = REPO / f"build-{arch}" / lib
    if not path.is_file():
        raise GenError(f"{path} not found -- run tools/ucrt_tier2_refset.py --arch {arch} first")
    nm = REPO / ".build-tools" / "llvm-mingw" / "bin" / "llvm-nm"
    res = subprocess.run(
        [str(nm), "--defined-only", str(path)], capture_output=True, text=True, timeout=600
    )
    if res.returncode != 0:
        raise GenError(f"llvm-nm failed on {path}: {res.stderr.strip()[:500]}")
    out = set()
    for line in res.stdout.splitlines():
        parts = line.split()
        if len(parts) < 3:
            continue
        sym = parts[2]
        for pre in prefixes:
            if sym.startswith(pre):
                out.add(sym[len(pre) :])
                break
    return out


def rust_bodies(arch: str) -> set[str]:
    """Export names the renamed msvcrt Rust archive defines a `rust_*` body for.

    Not every MICROSOFT export has one: ~200 of the 999 need genuinely new code, and a
    generated call to `rust_foo` that nothing defines is a link error, not a finding.
    Those get a two-sided row (shipping DLL vs Microsoft) instead.
    """
    return _renamed_exports(arch, "libucrt_rust.a", ("_rust_", "rust_"))


def select(arch: str, cache: Path | None = None) -> list[tuple[str, str, list[str], bool]]:
    """(export, declared-name, param kinds, has-rust-body) for everything we can grade."""
    import ucrt_burndown as bd

    funcs = ast_functions(arch, cache)
    buckets, byfile = bd.classify(arch)
    src: dict[str, str] = {}
    for f, exports in byfile.items():
        for e in exports:
            src[e] = f

    have_rust = rust_bodies(arch)
    # Iterate the REFERENCE SET's exports, so every generated `ref_<name>` is a symbol
    # that provably exists. The burn-down's names are undecorated, which is exactly the
    # ambiguity that broke this.
    chosen = []
    quarantined: list[str] = []
    for name in sorted(_renamed_exports(arch, "libucrt_ref.a", ("_ref_", "ref_"))):
        if name not in funcs:
            continue
        kinds = shape(funcs[name])
        if kinds is None:
            continue
        export = bd._undecorate(name if name.startswith("_") else "_" + name)
        if export not in buckets["MICROSOFT"]:
            continue
        if not src.get(export, "").startswith(PURE_DIRS):
            continue
        if is_quarantined(name) or is_quarantined(export):
            quarantined.append(export)
            continue
        chosen.append((export, name, kinds, name in have_rust))
    # Held-out exports are ANNOUNCED, never silently dropped: a sweep that quietly
    # grades less than the burn-down thinks it does is the same false-completeness
    # failure the shard `partial=` field exists to prevent.
    if quarantined:
        print(
            f"gen-ucrt-tier2-cases: QUARANTINED {len(quarantined)} export(s) -- "
            f"{QUARANTINE_REASON}\n  " + " ".join(sorted(quarantined)),
            file=sys.stderr,
        )
    return chosen


def emit(rows: list[tuple[str, str, list[str]]]) -> str:
    """The generated .inc: declarations, per-shape driver macros, then one call per export.

    The macros are generated too, so a shape that appears for the first time gets a driver
    without anyone editing C.

    CORPUS SIZING. A function with one scalar argument gets the FULL 0..255 sweep plus the
    edges -- that is the ctype families, and the 0x80-0xFF half is exactly where the narrow
    ctype table and the codepage decide anything. Anything with more than one looping
    argument uses the small corpora instead, because the product is what explodes: at the
    full sweep a four-argument shape would be 4.5 billion cases.
    """
    shapes = {tuple(k) for _e, _n, k, _r in rows}
    out: list[str] = (
        [
            "/* GENERATED by tools/gen_ucrt_tier2_cases.py -- do not edit.",
            " *",
            " * Declares each graded export's two renamed copies with __typeof__ of the REAL",
            " * declaration, so no signature is ever re-spelled and none can drift from the",
            " * header the bare side is declared by.",
            " */",
            "",
            "/* The SAME header set the generator parsed. Emitting it here rather than trusting",
            " * the harness to include the right ones is what keeps __typeof__ resolvable: a",
            " * header the harness omitted took out every mbstring declaration first time. */",
        ]
        + [f"#include <{h}>" for h in HEADERS]
        + [""]
    )
    for _e, name, _k, has_rust in rows:
        out.append(f"extern __typeof__({name}) ref_{name};")
        if has_rust:
            out.append(f"extern __typeof__({name}) rust_{name};")
    out.append("")

    # arg kind -> (corpus array, count macro, element expression)
    big = {"I": ("g_int_full", "G_INT_FULL_N")}
    small = {
        "I": ("g_int_small", "G_INT_SMALL_N"),
        "SN": ("g_narrow", "G_NARROW_N"),
        "SW": ("g_wide", "G_WIDE_N"),
        "SU": ("g_mbstr", "G_MBSTR_N"),
    }
    for kinds in sorted(shapes):
        macro = "GRADE_" + ("VOID" if not kinds else "_".join(kinds))
        loops = [i for i, k in enumerate(kinds) if k != "L"]
        use_big = len(loops) == 1 and kinds[loops[0]] == "I"
        lines, args, depth = [], [], 0
        for i, k in enumerate(kinds):
            if k == "L":
                args.append("(_locale_t)0")
                continue
            arr, cnt = (big if use_big else small)[k]
            # !g_fault_flag in EVERY loop condition: the handler can only skip the one
            # faulting instruction, so without this the body keeps being re-entered and
            # the export never ends. This is what turns a fault into a FAULT row.
            lines.append(
                "    " * (depth + 1)
                + f"for (size_t i{i} = 0; i{i} < {cnt} && !g_fault_flag; i{i}++) \\"
            )
            args.append(f"{arr}[i{i}]")
            depth += 1
        # The label names the FIRST looping argument's index, which is what identifies a
        # failing row; a bare family name could not be acted on.
        label = f"label_idx(i{loops[0]})" if loops else '"()"'
        # NORM is `!!` for a predicate and empty otherwise; see _PREDICATE.
        argl = ", ".join(args)
        call = f"grade(#fn, {label}, (long long)NORM(fn({argl})), \\"
        body = "    " * (depth + 1) + call
        rust_c = f"(long long)NORM(rust_##fn({argl})), \\"
        body2 = "    " * (depth + 2) + rust_c
        ref_c = f"(long long)NORM(ref_##fn({argl})));"
        body3 = "    " * (depth + 2) + ref_c
        out.append(f"#define {macro}(fn) \\")
        out.extend(lines)
        out.append(body)
        out.append(body2)
        out.append(body3)
        out.append("")
        # Two-sided variant for an export with no Rust body yet: shipping DLL vs
        # Microsoft only. Passing the DLL's own answer as the rust side would count a
        # free pass for a body that does not exist.
        out.append(f"#define {macro}_NORUST(fn) \\")
        out.extend(lines)
        call2 = f"grade2(#fn, {label}, (long long)NORM(fn({argl})), \\"
        out.append("    " * (depth + 1) + call2)
        out.append("    " * (depth + 2) + ref_c)
        out.append("")

    # ONE FUNCTION PER EXPORT, so each can be run on its own thread and timed out.
    # A hang is otherwise unrecoverable: the fault guard handles a body that crashes,
    # but `_mbsnbcoll` LOOPS, and it stopped the first complete arm64 sweep at export
    # 114 of 224 -- hiding the other 110 exactly the way a crash used to.
    for _e, name, kinds, has_rust in rows:
        macro = "GRADE_" + ("VOID" if not kinds else "_".join(kinds))
        pred = bool(_PREDICATE.match(name))
        out.append("#undef NORM")
        out.append("#define NORM(x) " + ("(!!(x))" if pred else "(x)"))
        out.append(f"static void ucrtdiff_run_{name}(void)")
        out.append("{")
        out.append(f"    {macro}{'' if has_rust else '_NORUST'}({name});")
        out.append("}")
        out.append("")
    out.append("#undef NORM")
    out.append("")

    # export_begin GATES the export: it returns 0 when this shard does not own the
    # export, or when the harness's own deadline has passed. Emitting the call inside
    # the `if` is what keeps sharding and the deadline in ONE place -- the driver
    # stays a flat list and neither concern leaks into the generated bodies.
    out += ["static void run_generated(void)", "{", "    int outcome;", ""]
    for _e, name, _kinds, has_rust in rows:
        out.append(f'    if (export_begin("{name}")) {{')
        out.append(f"        outcome = run_guarded(ucrtdiff_run_{name});")
        out.append(f"        export_end(outcome, {1 if has_rust else 0});")
        out.append("    }")
        out.append("")
    out += ["}", ""]
    return "\n".join(out) + "\n"


def main(argv: list[str] | None = None) -> int:
    ap = argparse.ArgumentParser(description=__doc__.splitlines()[0])
    ap.add_argument("--arch", required=True, choices=["i386", "amd64", "arm64"])
    ap.add_argument("--out", type=Path, help="generated .inc (default: beside the harness)")
    ap.add_argument("--cache", type=Path, help="cache the parsed header AST here")
    ap.add_argument("--stats", action="store_true", help="print the shape histogram and stop")
    ap.add_argument(
        "--reverse",
        action="store_true",
        help="emit the exports in reverse order. A DIAGNOSTIC: the sweep stops after a "
        "fixed COUNT (114 of 224) that survives excluding the next export, dropping the "
        "threading and shrinking the corpus. Reversing separates the three explanations "
        "-- stopping at the same COUNT means an accumulated resource, stopping at the "
        "same NAME means that body, and stopping at neither means position in the list.",
    )
    args = ap.parse_args(argv)

    try:
        rows = select(args.arch, args.cache)
        if args.reverse:
            rows = list(reversed(rows))
    except GenError as exc:
        print(f"gen-ucrt-tier2-cases: FAIL ({args.arch}): {exc}", file=sys.stderr)
        return 1
    if not rows:
        print(f"gen-ucrt-tier2-cases: FAIL ({args.arch}): selected nothing", file=sys.stderr)
        return 1

    if args.stats:
        hist = collections.Counter("_".join(k) or "VOID" for _e, _n, k, _r in rows)
        n_rust = sum(1 for r in rows if r[3])
        print(
            f"{args.arch}: {len(rows)} gradable exports "
            f"({n_rust} with a Rust body to grade, {len(rows) - n_rust} DLL-vs-ref only)"
        )
        for sh, n in hist.most_common():
            print(f"  {n:4d}  ({sh})")
        return 0

    out = args.out or (REPO / "tools" / "payloads" / "ucrtdiff" / f"generated-{args.arch}.inc")
    out.write_text(emit(rows))
    print(f"gen-ucrt-tier2-cases {args.arch}: {len(rows)} exports -> {out}")
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
