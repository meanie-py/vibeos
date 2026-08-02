#!/usr/bin/env python3
"""scan_import_cc.py — cross-MODULE import calling-convention, checked against the .spec oracle.

The hole this fills is named, in as many words, by the tool next door. scan_cc_mismatch.py
compares an `extern "X" { fn F; }` declaration against F's `#[no_mangle]` definition in the
SAME crate, and scopes itself out of everything else with:

    Scope: same-CRATE pairs only (cross-crate/DLL pairs go through import libs whose
    decorated names catch drift at link time; same-crate pairs link silently).

That sentence is true and it is the problem: **link time is the gate, not `check`.** `cargo
check` never links a DLL, so `vibeos-cli check`, `check-tooling` and `difftest` can all be
green on a tree that cannot produce a binary. Wave 18c ate exactly that — errno.rs declared

    extern "C" { fn SetErrorMode(uMode: c_uint) -> c_uint; }   // kernel32, WINAPI

which on i386 asks the linker for `_SetErrorMode` while kernel32's import lib provides
`_SetErrorMode@4`. Three green gates, then LINK_FAIL out of the merge queue: 3 builds and
4 boots of fleet load to learn something a text scan knows for free.

WHAT IT COMPARES
    A declaration with no in-crate definition (that half is scan_cc_mismatch's) against the
    convention the OWNING MODULE'S .spec assigns the same name. The spec is not a guess: it
    is the literal input to spec2def, which is what mints the import library — so for a
    cross-module reference the spec IS the link-time truth.

    i386 only, because i386 is the only arch where it can go wrong. spec2def decorates
    exactly two conventions and only on x86 (spec2def.c:719 — `giArch == ARCH_X86` and
    STDCALL or FASTCALL): stdcall -> `_Name@N`, fastcall -> `@Name@N`, cdecl/varargs ->
    `_Name`. On amd64 and arm64 "C" and "system" are the same ABI and the same symbol, so a
    mis-spelled decl there is invisible AND harmless. It is only harmless there.

SCOPING IS THE WHOLE DESIGN, NOT A REFINEMENT
    Run against every .spec in the tree at once this finds 22 sites, and 4 of them are pure
    name collisions between unrelated modules:

      * user32.spec exports `@ stdcall UserRealizePalette(long)`; win32ss/user/ntuser/
        painting.h declares `INT FASTCALL UserRealizePalette(HDC)`. win32k-sys's bindgen
        island transcribes the win32k one and is RIGHT. win32k does not import user32.
      * ntoskrnl.spec exports `@ stdcall RtlAcquireSRWLockExclusive`, and ntoskrnl ALSO
        carries a private cdecl inline-shim symbol of the same name that kernel-rust's
        force-symbols block declares. Both exist in one binary; only one is the export.
      * msvfw32/msvideo.spec's Win16 `3 pascal DllEntryPoint(...)` vs crt-startup's.
      * freeldr's halstub declares IoReadPartitionTable, but freeldr links no import libs
        at all — it is not an import, it is resolved inside the rosload link.

    So the oracle is scoped to the DLLs the consuming module actually names in
    `add_importlibs(<target> ...)`, resolved per crate through the CMakeLists that
    references the crate by name (falling back to the target that shares the crate's stem,
    which is how an unflipped crate like wldap32-rust — referenced by no CMakeLists yet —
    still gets checked before it is wired). A crate whose link set cannot be resolved is
    SKIPPED and counted, never guessed at: an unscoped name match is not evidence, and all
    four false positives above are what guessing looks like.

    Scoping also excludes the module's OWN spec for free (nothing lists itself), which is
    correct and load-bearing. A name the module itself exports is resolved in-module against
    its C, and RtlAcquireSRWLockExclusive proves a module can hold both the exported symbol
    and a private same-named one — the spec cannot arbitrate that. Same-module drift is
    scan_cc_mismatch's job and it already reports it.

WHY verify_ffi's ABI CHECK DID NOT ALREADY DO THIS
    It looks like it should — `verify_ffi.py` compares extern blocks against a .spec too —
    and it misses every finding here for two structural reasons, both visible in one line of
    its source (`if conv == "C" and not skip_abi and fn_name not in local_defs`). It only
    examines declarations spelled `extern "C"`, so fastfat's `extern "system"` against a
    FASTCALL export is outside it; and its oracle is ntoskrnl.spec plus the ntkrnl bindgen
    island, so no usermode DLL is in scope at all — which is why msvcrt's kernel32 import
    sailed through it. Its ABI findings and this tool's are disjoint today (verify_ffi's
    baseline is 93 C_MACRO + 66 FLEX_ARRAY, zero ABI). A kernel crate declaring an ntoskrnl
    stdcall export `extern "C"` would be reported by both, which is fine.

NOT COVERED, deliberately: `-private` exports are absent from the import lib, so a decl of
one fails to link on convention-independent grounds (missing symbol, not wrong symbol) —
they are dropped from the oracle rather than half-judged. `@ stub` carries no convention and
`@ extern` is data.

A finding is a LINK error, not a warning. The one exception is a decl that is never called
and never address-taken: it emits no relocation, so it links green and detonates the day
someone uses it. Both of those shipped in this tree.

Output: one line per finding (stable, repo-relative, no line numbers — the baseline is keyed
on these verbatim and line numbers churn). Baselined by tools/lint-baseline/scan_import_cc.txt
via `vibeos-cli check`. Suppress a verified FP with: // nolint: import-cc (reason)
"""

from __future__ import annotations

import argparse
import os
import re
import sys
from collections import defaultdict

REPO = os.path.dirname(os.path.dirname(os.path.abspath(__file__)))
sys.path.insert(0, os.path.dirname(os.path.abspath(__file__)))

# Reuse the sibling's Rust reader wholesale. Two scanners that parse `extern` blocks
# differently would disagree about which decls exist, and the pair is meant to partition
# the same population: it takes the decls WITH an in-crate def, this one takes the rest.
from scan_cc_mismatch import (  # noqa: E402
    DECL_BLOCK_RE,
    DECL_FN_RE,
    ISLAND_RE,
    NORM_I386,
    cfg_excludes_i386,
    crate_def_conventions,
    decl_symbol,
    excluded_spans,
    in_spans,
    walk_i386_sources,
)

BASE = os.path.join(REPO, "sdk", "lib", "rust")
SKIP_DIRS = {".git", "target", "build-i386", "build-amd64", "build-arm64"}

NOLINT_RE = re.compile(r"nolint:\s*import-cc")

# `@ stdcall -arch=i386 -private Foo(long) target` / `123 cdecl Bar()`.
#
# The convention comes FIRST and the flags follow it — spec2def.c:1026 reads the type, then
# "Go to next token (options or name)". Writing this pattern the other way round (flags, then
# convention) does not fail loudly: every line that HAS a flag simply stops matching, so
# `@ stdcall -norelay GetProcessHeap()` and every `-arch=`/`-stub`/`-private` export drops out
# of the oracle and is silently never checked. Caught by noticing GetProcessHeap sitting
# unflagged in the middle of eight flagged kernel32 imports in wldap32's prelude.
SPEC_LINE_RE = re.compile(r"^\s*(?:@|\d+)\s+(\w+)\s+((?:-\S+\s+)*)([\w?@$.]+)")

# What spec2def does with each token (spec2def.c:1027-1048). `pascal` and `thiscall` are
# NOT in that list — a `pascal` line is Wine-inherited spec text that ReactOS's spec2def
# would reject outright, so treating it as an oracle is reading a file nothing compiles.
# Dropping it is what removes the msvfw32-vs-crt-startup DllEntryPoint collision.
SPEC_CONV = {"stdcall": "stdcall", "cdecl": "C", "varargs": "C", "fastcall": "fastcall"}

# How the linker spells each convention on i386, for the finding message. This is the
# whole point of the tool, so say it in the finding rather than making the reader recall it.
DECORATION = {"stdcall": "_Name@N", "fastcall": "@Name@N", "C": "_Name"}


def _read(path: str) -> str | None:
    """File text, or None if it cannot be read. One home so every walker below skips an
    unreadable file the same way instead of each inventing its own try/except."""
    try:
        with open(path, encoding="utf-8", errors="replace") as fh:
            return fh.read()
    except OSError:
        return None


def _walk_repo(repo: str):
    for root, dirs, files in os.walk(repo):
        dirs[:] = [d for d in dirs if d not in SKIP_DIRS]
        yield root, files


def index_specs(repo: str) -> dict[str, list[str]]:
    """import-lib name -> its .spec path(s). 7 basenames are ambiguous in this tree (two
    ddraw, three fusion, ...); all of them are kept and a name whose specs DISAGREE about a
    convention is treated as no evidence, exactly like a cross-module disagreement."""
    out: dict[str, list[str]] = defaultdict(list)
    for root, files in _walk_repo(repo):
        for fn in files:
            if fn.endswith(".spec"):
                out[fn[:-5]].append(os.path.join(root, fn))
    return out


def _arch_includes_i386(arch_list: str) -> bool:
    """Does `-arch=<list>` include i386? Mirrors spec2def.c:1067-1100.

    A leading `!` NEGATES the entry — msvcrt.spec alone has 60-odd `-arch=!i386` exports
    (`__C_specific_handler`, `_chgsignf`, …), and a plain substring test reads every one of
    them as i386-included, i.e. it imports the exact set of exports that do not exist there.
    `win32` is spec2def's alias for the 32-bit arch (pszArchString2); `win64` is not."""
    included = False
    for entry in arch_list.split(","):
        negated = entry.startswith("!")
        if (entry.lstrip("!") in ("i386", "win32")) != negated:
            included = True
    return included


def read_spec(path: str) -> dict[str, set[str]]:
    """exported name -> convention(s), for the i386 import lib only."""
    out: dict[str, set[str]] = defaultdict(set)
    text = _read(path)
    if text is None:
        return out
    for line in text.splitlines():
        if line.lstrip().startswith(";"):  # commented-out export
            continue
        m = SPEC_LINE_RE.match(line)
        if not m:
            continue
        conv, flags, name = m.groups()
        if conv not in SPEC_CONV:
            continue
        # A -private export is not in the import lib at all; see the module docstring.
        if "-private" in flags:
            continue
        arch = re.search(r"-arch=(\S+)", flags)
        if arch and not _arch_includes_i386(arch.group(1)):
            continue
        # `@ stdcall Foo(long) other.Bar` forwards, but the EXPORT's own decoration is what
        # the import lib carries, so the forward target is irrelevant here.
        out[name.split("@")[0] if not name.startswith("?") else name].add(SPEC_CONV[conv])
    return out


def importlibs_by_target(repo: str) -> tuple[dict[str, set[str]], dict[str, set[str]]]:
    """(target -> import libs, CMakeLists path -> every import lib named in that file)."""
    by_target: dict[str, set[str]] = defaultdict(set)
    by_file: dict[str, set[str]] = defaultdict(set)
    for root, files in _walk_repo(repo):
        if "CMakeLists.txt" not in files:
            continue
        path = os.path.join(root, "CMakeLists.txt")
        text = _read(path)
        if text is None:
            continue
        for m in re.finditer(r"add_importlibs\s*\(([^)]*)\)", text):
            toks = m.group(1).split()
            if not toks:
                continue
            by_target[toks[0]].update(toks[1:])
            by_file[path].update(toks[1:])
    return by_target, by_file


def crate_link_set(
    crate: str,
    cmakelists_for: dict[str, set[str]],
    by_target: dict[str, set[str]],
    by_file,
    specs: dict[str, list[str]],
) -> tuple[set[str], list[str]] | None:
    """(import libs, module source dirs) a crate's symbols resolve against, or None.

    Two rules, in order, and the order matters:

    1. The CMakeLists that NAMES the crate. This is the real consumer — msvcrt-crt-rust is
       referenced only by dll/win32/msvcrt/CMakeLists.txt, which is also where
       `add_importlibs(msvcrt kernel32 kernel32_vista ntdll)` lives. It wins even when it
       yields the EMPTY set, because "this module imports nothing" is an answer: ntoskrnl and
       freeldr link no import libs, and stripping kernel-rust/freeldr-rust down to their
       stems would otherwise invent an oracle for them.
    2. Only if no CMakeLists mentions the crate at all: the target sharing its stem, AND ONLY
       IF that target owns a `<stem>.spec`. This is the unflipped-crate case and it is worth
       the extra rule — wldap32-rust and mscms-rust are translated, parked, and referenced by
       nothing, yet carry 9 real findings between them; catching those BEFORE the flip is the
       point. The spec requirement is what keeps the rule from inventing a link set out of a
       name: `ps-rust` is ntoskrnl's Ps subsystem, but bare `ps` is also a rosapps sysutil
       (`add_importlibs(ps ntdll user32 msvcrt kernel32)`), and matching it reported ps-rust's
       CORRECT kernel FASTCALL InterlockedCompareExchange against kernel32's stdcall one. A
       crate that is the Rust body of a DLL shares that DLL's exports, so it has a spec.
    """
    files = cmakelists_for.get(crate)
    if files is not None:
        libs: set[str] = set()
        for f in files:
            libs |= by_file.get(f, set())
        return libs, [os.path.dirname(f) for f in files]
    stem = crate
    for suffix in ("-rust", "-sys", "-cargo"):
        stem = stem.removesuffix(suffix)
    if stem not in by_target or not specs.get(stem):
        return None
    return set(by_target[stem]), [os.path.dirname(p) for p in specs[stem]]


def c_defines(module_dirs: list[str], name: str) -> str | None:
    """The module's own C file defining `name` out of line, if any.

    A symbol the consuming module DEFINES is resolved inside the link, so the exporting DLL's
    spec is not its oracle — and the tree does this on purpose. fastfat's
    rust_cohab_forceinline.c gives `ExInterlockedPopEntrySList` an out-of-line body because
    the DDK only has it as a FORCEINLINE, so fastfat's Rust rightly calls the local stdcall
    one and not ntoskrnl's `@ fastcall -arch=i386` export of the same name. Without this the
    scan reports a link failure against a symbol sitting in the next file.

    Definition, not call: params containing no `;` `{` `}`, then a brace. `int f(void);` and
    `g(x);` both fail that, which is the whole discrimination needed here."""
    pat = re.compile(r"^[A-Za-z_][\w\s*]*?\b" + re.escape(name) + r"\s*\([^;{}]*\)\s*\{", re.M)
    for d in module_dirs:
        for root, files in _walk_repo(d):
            for fn in files:
                if not fn.endswith((".c", ".h")):
                    continue
                path = os.path.join(root, fn)
                text = _read(path)
                if text is not None and pat.search(text):
                    return os.path.relpath(path, REPO)
    return None


def cmakelists_naming_crates(repo: str, crates: list[str]) -> dict[str, set[str]]:
    """crate -> CMakeLists that mention it by name (absent from the dict if none do)."""
    pats = {c: re.compile(r"(?<![\w-])" + re.escape(c) + r"(?![\w-])") for c in crates}
    out: dict[str, set[str]] = defaultdict(set)
    for root, files in _walk_repo(repo):
        if "CMakeLists.txt" not in files:
            continue
        path = os.path.join(root, "CMakeLists.txt")
        text = _read(path)
        if text is None:
            continue
        for crate, pat in pats.items():
            if pat.search(text):
                out[crate].add(path)
    return out


def scan_crate(
    crate_src: str, oracle: dict[str, dict[str, set[str]]], module_dirs: list[str]
) -> list[str]:
    """`oracle` is import-lib name -> (export -> conventions), already scoped to this crate."""
    defs = crate_def_conventions(crate_src)
    findings = []
    local: dict[str, str | None] = {}
    for path, src in walk_i386_sources(crate_src):
        spans = excluded_spans(src)
        src_lines = src.splitlines()
        for m in DECL_BLOCK_RE.finditer(src):
            cfg, cc, body = m.group("cfg"), m.group("cc"), m.group("body")
            if cfg_excludes_i386(cfg) or in_spans(m.start(), spans):
                continue
            block_line = src.count("\n", 0, m.start())
            for d in DECL_FN_RE.finditer(body):
                name = decl_symbol(d)
                if name in defs:  # same-crate pair — scan_cc_mismatch owns it
                    continue
                line = block_line + body.count("\n", 0, d.start()) + 1
                if NOLINT_RE.search(src_lines[line - 1] if line - 1 < len(src_lines) else ""):
                    continue
                owners = {lib: c[name] for lib, c in oracle.items() if name in c}
                conventions = {v for s in owners.values() for v in s}
                # No exporter, or exporters that disagree with each other: no evidence.
                if len(conventions) != 1:
                    continue
                want = conventions.pop()
                got = NORM_I386.get(cc, cc)
                if got == want:
                    continue
                if name not in local:
                    local[name] = c_defines(module_dirs, name)
                if local[name]:
                    continue
                # A generated bindgen island is not our description of a symbol, it is
                # bindgen's transcript of the C prototype — so a disagreement with the spec
                # is the HEADER against the SPEC, and editing the island destroys the only
                # evidence that could settle it (scan_cc_mismatch pays for that lesson at
                # length: e33a810c9ad rewrote 8 island blocks and silenced 8 real bugs).
                # ntdll's NtCurrentTeb is exactly this: psfuncs.h declares it with no NTAPI
                # while ntdll.spec exports it `@ stdcall -arch=win32`. Reported, tagged,
                # never auto-fixed.
                tag = (
                    "IMPORT_CC_ISLAND"
                    if ISLAND_RE.search(path.replace(os.sep, "/"))
                    else "IMPORT_CC"
                )
                findings.append(
                    f"{os.path.relpath(path, REPO)}: [{tag}] {name} declared "
                    f'extern "{cc}" but {"/".join(sorted(owners))}.spec exports it '
                    f"{want} (i386 link wants {DECORATION.get(want, want)}, "
                    f"we emit {DECORATION.get(got, got)})"
                )
    return findings


def main() -> int:
    ap = argparse.ArgumentParser(description=__doc__)
    ap.add_argument(
        "--coverage",
        action="store_true",
        help="also report, on stderr, which crates were scoped and which were skipped",
    )
    args = ap.parse_args()

    crates = sorted(d for d in os.listdir(BASE) if os.path.isdir(os.path.join(BASE, d, "src")))
    specs = index_specs(REPO)
    by_target, by_file = importlibs_by_target(REPO)
    cmakelists_for = cmakelists_naming_crates(REPO, crates)

    spec_cache: dict[str, dict[str, set[str]]] = {}

    def exports_of(lib: str) -> dict[str, set[str]]:
        if lib not in spec_cache:
            merged: dict[str, set[str]] = defaultdict(set)
            for p in specs.get(lib, []):
                for name, convs in read_spec(p).items():
                    merged[name] |= convs
            spec_cache[lib] = merged
        return spec_cache[lib]

    findings: list[str] = []
    scoped, skipped = [], []
    for crate in crates:
        resolved = crate_link_set(crate, cmakelists_for, by_target, by_file, specs)
        if resolved is None or not resolved[0]:
            skipped.append(crate)
            continue
        libs, module_dirs = resolved
        oracle = {lib: exports_of(lib) for lib in libs if specs.get(lib)}
        if not oracle:
            skipped.append(crate)
            continue
        scoped.append(crate)
        findings += scan_crate(os.path.join(BASE, crate, "src"), oracle, module_dirs)

    for line in sorted(set(findings)):
        print(line)
    # The tally goes to STDERR, and that is not cosmetic: the lint spec captures stdout, so a
    # count-bearing line on stdout IS a finding to the baseline differ. It drifts in two
    # dimensions at once — the finding count and the scoped-crate count, the latter every time
    # any crate anywhere gains or loses an add_importlibs entry — so on its first gate run it
    # reported itself as "1 new" with nothing whatsoever changed in a scanned crate. Same
    # disease and same fix as verify_constants' catalog line (tests/qemu/cli/lints.py).
    print(
        f"IMPORT_CC: {len(set(findings))} across {len(scoped)} scoped crates",
        file=sys.stderr,
    )
    if args.coverage:
        print(f"scoped ({len(scoped)}): {' '.join(scoped)}", file=sys.stderr)
        print(f"skipped ({len(skipped)}): {' '.join(skipped)}", file=sys.stderr)
    return 0


if __name__ == "__main__":
    sys.exit(main())
