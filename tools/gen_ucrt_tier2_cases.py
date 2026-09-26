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
    # iswctype's mask. Unknown here, so iswctype/_iswctype_l -- the core every isw*
    # export runs through -- were never generated; the ctype takeover had to record
    # them as covered by construction instead of graded.
    "wctype_t": "I",
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

#: Comparators contract only the SIGN of their result, never its magnitude. C says
#: strcmp returns a value "greater than, equal to, or less than zero"; Microsoft's
#: returns the byte difference while ours returns -1/0/1, so grading the RAW return
#: marks perfect agreement as failure.
#:
#: This is not hypothetical. It manufactured 292 failures out of 392 on each of
#: strncmp, _mbsncmp(_l) and _mbsnbcmp(_l), and that cluster read as the sweep's single
#: largest group of real bugs -- five exports, identical counts, a tidy shared root
#: cause -- until someone looked at what the functions actually return. The
#: hand-written runner in ucrtdiff.c always knew this and wrapped every comparator in
#: sgn(); the generator did not, and nothing cross-checked the two.
_COMPARATOR = re.compile(r"(cmp|coll)(_l)?$")

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
#:      qemu/ramdisk.payloads records what the last stage wrote, and entries absent
#:      from the current selection are deleted before the new set is staged.
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
    "these exports wedge the ENTIRE GUEST, not just their own thread: all serial output "
    "stops, kernel logging included, so neither run_guarded's per-export timeout nor "
    "the harness's own deadline can fire (the deadline is cooperative, checked between "
    "exports). Two rules: QUARANTINE_EXACT lists exports individually WITNESSED killing "
    "a boot; the *coll* rule holds out a family by inference from two known members. "
    "This is OWED WORK, not a decision that any of them are fine."
)


#: Exports PROVEN to end the boot, each by a run that died entering it. Distinct from
#: the *coll* family rule below, which is a family held out by inference from two known
#: members; these are individually witnessed and named with their evidence.
QUARANTINE_EXACT = {
    # 2026-09-23: the full 194-export single-boot run graded 110 exports and the guest
    # died entering this one -- all serial stops, kernel [PS] spam included. Note the
    # neighbour: _mbsnbcmp_l (plain compare, same shape, same corpora) graded 392 cases
    # immediately before it, so this is the case-INSENSITIVE path, not multibyte
    # compare in general.
    "_mbsnbicmp",
    "_mbsnbicmp_l",
    # 2026-09-23, the 8-shard sweep. Each of these killed the shard it sat in, and the
    # rule they share is exact: a LENGTH-BOUNDED, narrow-or-multibyte, case-INSENSITIVE
    # compare. The controls matter as much as the cases -- _stricmp_l (unbounded, same
    # family) graded MATCH n=49, and _wcsnicmp (bounded but WIDE, so no lead-byte path)
    # graded clean in shard 3. That is what the wrapping_sub over-read in
    # mbcs.rs _mbsnbicmp_l needs: a `len` to underflow, and mbcs lead-byte logic to
    # reach `len == 1` with `clen == 2`. Case folding then makes mismatches rare enough
    # that the walk runs off into unmapped guest memory instead of stopping at a NUL.
    "_strnicmp",  # shard 1, after 16 exports
    "_strnicmp_l",  # shard 2, after 16
    "_mbsnicmp",  # shard 4, after 14
    "_mbsnicmp_l",  # shard 5, after 14
    # 2026-09-23, the corrected full run: died entering this after 122 exports. It had
    # never graded in ANY run -- in every shard it sat behind another killer -- so it
    # stayed invisible until the others were held out.
    #
    # It also BREAKS the tidy rule the four above suggested. _mbstrnlen is not a
    # comparator at all, and _mbsnbcmp (bounded AND multibyte, case-sensitive) grades
    # fine, so "length-bounded narrow/mb case-insensitive compare" was over-fitted to
    # the cases then in hand. What the six share is only "length-bounded, multibyte or
    # locale-aware"; what separates them from the bounded multibyte SURVIVORS is NOT
    # established. Treat this as a list of witnessed facts, not a proven predicate.
    "_mbstrnlen",
    "_mbstrnlen_l",
    # 2026-09-23: killed the tail run after 173 exports. NOTE it is not deterministic --
    # earlier runs caught it as a clean HANG row (run_guarded's 8s timeout fired and the
    # sweep carried on), and this time it took the guest down instead. Same body, two
    # outcomes, so whatever it does is timing-dependent. It is the only killer so far
    # that is neither bounded nor multibyte.
    "rand",
}


def is_quarantined(name: str) -> bool:
    """Exports held out of the sweep because grading them ends the boot.

    Two rules, deliberately kept separate. QUARANTINE_EXACT names exports individually
    witnessed killing a boot. The *coll* rule holds out a whole FAMILY by inference:
    every *coll* export routes through the same locale-collation path and the two
    originally known deadlockers (_mbsnbcoll, _mbsnbcoll_l) are members, so rather than
    discover the rest one dead boot at a time -- each costs ~25 minutes and hides every
    export after it -- the family goes out together.

    Holding an export out is never a verdict that it is FINE. Both rules are owed work,
    and the generator announces the full list on stderr every run so it cannot go quiet.
    """
    return "coll" in name or name in QUARANTINE_EXACT


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
        cmpf = bool(_COMPARATOR.search(name))
        out.append("#undef NORM")
        if pred:
            norm = "(!!(x))"
        elif cmpf:
            norm = "(sgn(x))"
        else:
            norm = "(x)"
        out.append("#define NORM(x) " + norm)
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


#: Exports graded by HAND-WRITTEN runners in ucrtdiff.c rather than generated ones
#: (the generator has no driver for buffer writers or pointer returns). Once a
#: takeover deletes one's Microsoft object, the reference set stops defining it and
#: a strong reference would stop the harness linking. A WEAK one does not work
#: either: a weak undefined never pulls an archive member, so it came back NULL even
#: while the object was right there. So the question "does the reference have it"
#: is answered HERE, from the archive, as a macro the harness compiles against.
HAND_GRADED = (
    "strcat",
    "strcpy",
    "strncat",
    "strncpy",
    "strpbrk",
    # run_ctype's is* rows: the ctype takeover deleted their Microsoft objects.
    "isalpha",
    "isdigit",
    "isspace",
    "isupper",
    "ispunct",
    # ...and its tolower/toupper rows: the tocase takeover deleted theirs.
    "tolower",
    "toupper",
    # run_strtox and run_strtox_family: the whole of convert/strtox.cpp.
    "strtol",
    "strtoul",
    "_strtol_l",
    "_strtoul_l",
    "_strtoi64",
    "_strtoi64_l",
    "strtoll",
    "_strtoll_l",
    "strtoimax",
    "_strtoimax_l",
    "_strtoui64",
    "_strtoui64_l",
    "strtoull",
    "_strtoull_l",
    "strtoumax",
    "_strtoumax_l",
    "wcstol",
    "_wcstol_l",
    "wcstoul",
    "_wcstoul_l",
    "_wcstoi64",
    "_wcstoi64_l",
    "wcstoll",
    "_wcstoll_l",
    "wcstoimax",
    "_wcstoimax_l",
    "_wcstoui64",
    "_wcstoui64_l",
    "wcstoull",
    "_wcstoull_l",
    "wcstoumax",
    "_wcstoumax_l",
    # run_atox_family and run_xtoa(_s)_family: convert/atox.cpp and xtoa.cpp.
    "atoi",
    "_atoi_l",
    "atol",
    "_atol_l",
    "atoll",
    "_atoll_l",
    "_atoi64",
    "_atoi64_l",
    "_wtoi",
    "_wtoi_l",
    "_wtol",
    "_wtol_l",
    "_wtoll",
    "_wtoll_l",
    "_wtoi64",
    "_wtoi64_l",
    "_itoa",
    "_itoa_s",
    "_ltoa",
    "_ltoa_s",
    "_ultoa",
    "_ultoa_s",
    "_i64toa",
    "_i64toa_s",
    "_ui64toa",
    "_ui64toa_s",
    "_itow",
    "_itow_s",
    "_ltow",
    "_ltow_s",
    "_ultow",
    "_ultow_s",
    "_i64tow",
    "_i64tow_s",
    "_ui64tow",
    "_ui64tow_s",
    # run_findfile_family: filesystem/findfile.cpp.
    "_findfirst32",
    "_findnext32",
    "_wfindfirst32",
    "_wfindnext32",
    "_findfirst32i64",
    "_findnext32i64",
    "_wfindfirst32i64",
    "_wfindnext32i64",
    "_findfirst64i32",
    "_findnext64i32",
    "_wfindfirst64i32",
    "_wfindnext64i32",
    "_findfirst64",
    "_findnext64",
    "_wfindfirst64",
    "_wfindnext64",
    "_findclose",
)


def emit_refs(arch: str) -> str:
    have = _renamed_exports(arch, "libucrt_ref.a", ("_ref_", "ref_"))
    lines = [
        "/* GENERATED by tools/gen_ucrt_tier2_cases.py -- do not edit. 1 when the reference",
        " * set defines ref_<name> for a hand-graded export, 0 once a takeover removed it. */",
    ]
    for name in HAND_GRADED:
        lines.append(f"#define UCRTDIFF_REF_HAS_{name} {int(name in have)}")
    return "\n".join(lines) + "\n"


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
    (out.parent / "generated-refs.h").write_text(emit_refs(args.arch))
    print(f"gen-ucrt-tier2-cases {args.arch}: {len(rows)} exports -> {out}")
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
