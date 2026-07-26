"""Generate a differential harness for a whole translated C file, from its signatures.

`msvcrt-difftest` proved the idea: hand the genuine `.c` to `cc` as the translation unit,
rename its public symbols to `ref_*` with a generated `-include` prelude, link that against
the SHIPPING Rust crate, and assert parity in one process. Nothing is transcribed, so the
reference cannot drift from the code it grades.

What that harness does NOT have is a way to *scale*. Its 106 symbols are a hardcoded list
carrying its own warning — "regenerate if string.c changes" — and every instrument is
hand-written per family. That is affordable for one file and not for a file-at-a-time
translation track: `mbcs.c` alone is 175 exports.

So this module derives the harness instead. Three things were hardcoded per file, and all
three are already in the C:

* **the public symbol list** — the existing one is hardcoded because deriving it from an
  object file "would make this host crate depend on an OS build having been run". True of
  `llvm-nm`; not true of the scanner, which reads the *source*. Same list, no build, no
  manual regeneration.
* **what each argument is** — 57 distinct signatures across `mbcs.c` looks unmechanizable
  until you stop matching whole signatures. Seven argument KINDS cover 157 of its 175
  functions; the shapes are just those kinds composed.
* **what "equal" means for a result** — per kind, and this is where a hand-written harness
  quietly goes wrong (see `_returns_interior_pointer`).

## Why the generated instrument is STRONGER than a hand-written one, not just cheaper

For this class the dominant input kind is an MBCS character: `unsigned int c` over a domain
of `0..=0xFFFF`. That is small enough to **enumerate**, so 38 of `mbcs.c`'s functions get a
total proof over their input domain rather than a sampled one. A fuzzer that happens not to
try `0x8140` tells you nothing about `0x8140`.

It also closes a blind spot the existing harness documents about itself: its reference runs
in the `"C"` locale only, so "a MATCH from this harness would be a lie the moment an app
calls `setlocale()`" — which is why `_stricmp`/`_strnicmp`/`_memicmp` are graded nowhere and
flipped nowhere. Generating one case per CODEPAGE fixture makes locale a dimension of the
test instead of an assumption behind it.

## What it refuses

A function whose parameters do not all land in the kind table is **reported, never guessed**
— a case that passes wrong arguments is worse than no case, because it reports MATCH. Same
for a function under an arch conditional: the reference TU is compiled for the host, so a
`#if defined(_WIN64)` arm the target would take is not the arm being graded. The existing
harness handles that with a note telling the next human to grep first; this refuses it.
"""

from __future__ import annotations

import re
from dataclasses import dataclass, field
from typing import TYPE_CHECKING

if TYPE_CHECKING:
    from collections.abc import Sequence

    from .scanner import CFunction, ScanResult

# --------------------------------------------------------------------------------------
# argument kinds
# --------------------------------------------------------------------------------------

#: An input character. For the MBCS family this is a 1-2 byte character, so the domain is
#: `0..=0xFFFF` and the generated case ENUMERATES it rather than sampling.
CHAR = "CHAR"
#: A NARROW character — the C says `unsigned char`, so the domain is one byte and the
#: extern must say `u8`. Declaring it `u32` like the MBCS `unsigned int` kind is an ABI
#: lie: msvcrt's `_mbbtype(unsigned char c, int type)` and its Rust twin `(c: u8, …)`
#: then disagree about how much of the register is the argument, and the case reports a
#: divergence that is entirely the harness's own doing.
UCHAR = "UCHAR"
#: A byte count / length. Driven off the corpus string it accompanies, plus the edges.
SIZE = "SIZE"
#: The CRT locale sentinel. `_locale_t` is itself a pointer — see `rustconv.typedefs`.
LOCALE = "LOCALE"
STR_IN = "STR_IN"  # `const unsigned char *` / `const char *`
BUF_OUT = "BUF_OUT"  # `unsigned char *` / `char *` — written through
WSTR_IN = "WSTR_IN"
WBUF_OUT = "WBUF_OUT"
VOID = "VOID"  # C `(void)`, which the scanner reports as one parameter
#: A scalar written THROUGH a pointer: `int *` / `size_t *`, non-const. The whole `_s`
#: family reports its result this way — `_mbccpy_s(dst, n, &copied, src)` returns only an
#: errno_t, and *`copied`* is the answer. Grading the return alone would compare two
#: `EINVAL`s and call it parity while the byte counts disagreed, which is why these were
#: worth a kind rather than a refusal: the `_s` variants carry the richest error contracts
#: in the CRT (EINVAL vs ERANGE vs success-with-truncation) and each contract is only
#: observable in the out-scalar.
INT_OUT = "INT_OUT"  # `int *`, `unsigned *`, `long *` — written through
SIZE_OUT = "SIZE_OUT"  # `size_t *` — written through
#: A `wint_t` return. MSVC's is 16-bit (so `WEOF == 0xFFFF`) where glibc's is 32-bit;
#: the refstub pins the MSVC spelling, so the extern must say `u16` or the two sides
#: disagree about how much of the register is the answer — the UCHAR lesson again.
WINT = "WINT"
#: `mbstate_t *` — the restartable conversions' partial-character state. IN-OUT, which is
#: why it cannot reuse INT_OUT even though msvcrt types it `int`: a sentinel seed would
#: feed the conversion a garbage *input* state. It is seeded from a swept set of real
#: initial states and compared afterwards.
MBSTATE = "MBSTATE"
#: `char **` / `unsigned char **` — an IN-OUT CURSOR the callee advances through a string.
#: `_mbstok_s(str, delim, ctx, locale)` keeps its tokenizer position in `*ctx`; `mbsrtowcs(
#: wcstr, pmbstr, count, state)` reports how far it consumed by writing `*pmbstr`.
#:
#: It cannot be one shared cell handed to both sides, and it cannot be compared as a
#: pointer. Each side walks its OWN copy of the string, so the two raw pointers are always
#: different and always will be — the comparison is the OFFSET each ended up at, exactly as
#: `_returns_interior_pointer` already does for a returned interior pointer. Handing both
#: sides one cell would be worse than useless: the reference would resume from wherever the
#: subject left off and agree with it by construction.
#:
#: Split by the constness of the POINTED-TO characters, which is the one signal that says
#: whether the cursor's string is the subject of the call or merely its input:
#:   * `unsigned char **` — `_mbstok_s` tokenizes the buffer IN PLACE, overwriting delimiters
#:     with NULs (`*(*ctx)++ = 0`). The buffer is the subject and must be seeded, and its
#:     CONTENTS have to be compared afterwards or a wrongly-placed NUL goes unnoticed.
#:   * `const char **` — `mbsrtowcs` only reads through it; the output goes to a separate
#:     wide sink. Nothing to seed, nothing extra to compare beyond where it stopped.
#: Their Rust spellings differ too (`*mut *mut u8` vs `*mut *const u8`), and the extern
#: block has to match the shipping definition or the two disagree about the argument.
STRCUR = "STRCUR"  # `unsigned char **` — cursor into a buffer the callee rewrites
STRCUR_IN = "STRCUR_IN"  # `const char **` — cursor into a source the callee only reads
#: A RETURN that points into the CRT's own state rather than into anything the case
#: supplied — msvcrt's `__p_*` family (`__p__mbctype`, `__p___mb_cur_max`, `__p__fmode`,
#: `__p__daylight`, … a dozen of them across the DLL). They take no arguments at all, so
#: `_returns_interior_pointer`'s "offset from the input base" has no base to work from,
#: and that is what refused them: there IS no input.
#:
#: Each side reads its own fixture slot, so the two addresses differ by construction and
#: always will. Compare what they point AT. Two shapes, split by pointee:
#:   * a scalar cell (`int *`) — dereference and compare.
#:   * a table (`unsigned char *`) — compare the bytes, with the length asked of C by
#:     `sizeof` on the mirrored struct field. Never transcribe that length: too small and
#:     the tail of the table is silently never compared, too large and the read runs off
#:     the struct. Both failures are invisible in the verdict.
STATE_INT = "STATE_INT"  # `int *` / `unsigned *` / `long *` into CRT state
STATE_TABLE = "STATE_TABLE"  # `unsigned char *` into a CRT state table

_INT_SCALARS = {"int", "unsigned int", "unsigned", "long", "unsigned long"}
_SIZE_SCALARS = {"size_t", "unsigned __int64", "__int64"}
_NARROW_CHAR = {"unsigned char", "char"}
_WIDE_CHAR = {"wchar_t", "unsigned short"}

#: Kinds a generated case knows how to SUPPLY. A parameter outside this set is a refusal.
SUPPLIABLE = frozenset(
    {
        CHAR,
        UCHAR,
        SIZE,
        LOCALE,
        STR_IN,
        BUF_OUT,
        WSTR_IN,
        WBUF_OUT,
        INT_OUT,
        SIZE_OUT,
        MBSTATE,
        STRCUR,
        STRCUR_IN,
    }
)
#: Kinds a generated case knows how to COMPARE as a return value.
COMPARABLE = frozenset(
    {CHAR, UCHAR, SIZE, STR_IN, BUF_OUT, WSTR_IN, WBUF_OUT, VOID, WINT, STATE_INT, STATE_TABLE}
)

#: `#if` guards that make the host reference a different function from the target's.
#: The reference TU is compiled for the host (x86_64 linux), so a `_WIN64` arm is graded on
#: whichever side the HOST takes — which is not necessarily the side i386/amd64/arm64 take.
_ARCH_GUARD = re.compile(r"#\s*if.*\b(_WIN64|__i386__|__x86_64__|__aarch64__|_M_[A-Z0-9]+)\b")

#: Win32 NLS. Absent on the host, and the reason 60 of `mbcs.c`'s functions cannot simply be
#: compiled and called: the case/collation/wide-conversion families ask the OS. The surface is
#: small enough to RECORD once and replay from a fixture, which is why reaching it is a
#: separate tier rather than a refusal.
NLS_SURFACE = frozenset(
    {
        "GetCPInfo",
        "GetACP",
        "GetOEMCP",
        "LocaleNameToLCID",
        "GetStringTypeW",
        "LCMapStringW",
        "CompareStringA",
        "MultiByteToWideChar",
        "WideCharToMultiByte",
        "__crtLCMapStringA",
        "locale_to_sname",
    }
)

#: Functions whose RETURN MAGNITUDE is arch-dependent by msvcrt's own design.
#:
#: string.c's `strcmp`/`strncmp` fork on `#if defined(_WIN64) || …`: the 64-bit arm
#: clamps to -1/0/1, the i386 arm returns the raw unsigned byte difference. The harness
#: compiles its reference for the HOST (where `_WIN64` is undefined, so it takes the
#: i386 arm) and links Rust built for a 64-bit target (which clamps) — a mismatch
#: msvcrt-difftest's own lib.rs documents and classify.rs handles by grading
#: `strncmp_raw` directly.
#:
#: mbcs.c reaches them transitively: every comparison entry point falls through to
#: `u_strcmp`/`u_strncmp` or `_strncoll_l` on a single-byte codepage. Both sides are
#: CORRECT for their own arm, so only the SIGN is common ground — comparing magnitudes
#: would report a divergence that is really the harness's inability to hold both arms
#: at once.
SIGN_ONLY_ROOTS = frozenset(
    {
        "strcmp",
        "strncmp",
        "u_strcmp",
        "u_strncmp",
        "u_strcasecmp",
        "u_strncasecmp",
        "_strcoll_l",
        "_strncoll_l",
        "_stricmp",
        "_strnicmp",
    }
)

#: Rebuilding or freeing the per-thread MBCS state, or taking the CRT lock around it. A case
#: cannot run one of these: the whole design pins `threadmbcinfo` as a fixture so that a
#: divergence is provably logic and not state, and a function that REPLACES that state
#: invalidates the fixture for every case that follows it in the same process.
STATE_MUTATORS = frozenset({"create_mbcinfo", "free_mbcinfo", "_lock", "_unlock", "malloc", "free"})

TIER_PURE = "pure"  # host-only: reads the pinned fixture, calls nothing else
TIER_NLS = "nls-replay"  # needs the recorded Win32 NLS answers


def _callees(fn: CFunction) -> set[str]:
    return set(re.findall(r"\b([A-Za-z_]\w*)\s*\(", fn.body_text or ""))


def _reaches(scan: ScanResult, targets: frozenset[str]) -> set[str]:
    """Every function that transitively calls into *targets*."""
    calls = {f.name: _callees(f) for f in scan.functions}
    hit = {n for n, cs in calls.items() if cs & targets}
    changed = True
    while changed:
        changed = False
        for name, cs in calls.items():
            if name not in hit and (cs & hit):
                hit.add(name)
                changed = True
    return hit


def classify_type(c_type: str) -> str:
    """The argument kind of a C type, or `"?:<spelling>"` when it is not one we supply."""
    text = " ".join(c_type.split())
    stars = text.count("*")
    base = " ".join(w for w in text.replace("*", " ").split() if w not in ("const", "volatile"))
    is_const = "const" in text.split()

    if stars == 0:
        if base == "void":
            return VOID
        if base == "wint_t":
            return WINT
        if base in _SIZE_SCALARS:
            return SIZE
        if base == "_locale_t":
            return LOCALE
        if base in _NARROW_CHAR:
            return UCHAR
        if base in _INT_SCALARS:
            return CHAR
        return f"?:{base}"
    if stars == 1 and base in _NARROW_CHAR:
        return STR_IN if is_const else BUF_OUT
    if stars == 1 and base in _WIDE_CHAR | {"wchar_t"}:
        return WSTR_IN if is_const else WBUF_OUT
    if stars == 1 and base == "mbstate_t":
        # BEFORE the _INT_SCALARS arm below would ever see it. msvcrt types mbstate_t as
        # `int`, so resolving the typedef first would classify it INT_OUT and seed it with
        # the not-written sentinel — handing the conversion a garbage partial-character
        # state and grading whatever fell out.
        return MBSTATE
    if stars == 1 and not is_const and base in _SIZE_SCALARS:
        return SIZE_OUT
    if stars == 2 and base in _NARROW_CHAR:
        # The const sits on the CHARACTERS, not on the cell — both forms are written
        # through. What it distinguishes is whether the cursor's string is the SUBJECT of
        # the call (`_mbstok_s` rewrites it) or merely its input (`mbsrtowcs` reads it).
        return STRCUR_IN if is_const else STRCUR
    if stars == 1 and not is_const and base in _INT_SCALARS:
        # `const int *` deliberately falls through to a refusal: an INPUT array of ints is
        # a different thing from a written-through result cell, and guessing which one a
        # signature means is how a harness ends up supplying a 1-element array to a
        # function that wanted 40.
        return INT_OUT
    if stars == 1 and base == "_locale_t":
        # `_locale_t` is ALREADY a pointer; a starred one is a genuine `_locale_t *`.
        return f"?:{text}"
    return f"?:{text}"


def base_kinds(return_kind: str) -> tuple[str, ...]:
    """Which parameter an interior returned pointer is measured against, best candidate first.

    Measuring against the wrong argument compares offsets into two unrelated allocations and
    fails on perfectly correct code, so the order matters:

    * **An output buffer wins when there is one.** `_mbsnbcpy_l(dst, src, n, loc)` returns
      `dst`, not `src`.
    * **Otherwise an input string.** C's search family launders const — `_mbschr_l` takes a
      `const unsigned char *` and returns a plain `unsigned char *` pointing into it, exactly
      as `strchr` does. A rule that matched constness strictly would refuse the entire
      `chr`/`rchr`/`str`/`pbrk` family, which is most of what a string API is.
    """
    if return_kind in (STR_IN, BUF_OUT, WSTR_IN, WBUF_OUT):
        return (BUF_OUT, WBUF_OUT, STR_IN, WSTR_IN)
    return ()


def _returns_interior_pointer(kind: str) -> bool:
    """A returned `char *` from this class points INTO an argument, not to fresh storage.

    So the comparison is on the OFFSET from the input base, never on the address: subject and
    reference are handed separate buffers and their raw pointers can never be equal. This is
    the single easiest way to hand-write a difftest that reports MATCH for the wrong reason —
    compare pointers and every case "passes" only when both return NULL.
    """
    return kind in (STR_IN, BUF_OUT, WSTR_IN, WBUF_OUT)


# --------------------------------------------------------------------------------------
# per-function plan
# --------------------------------------------------------------------------------------


@dataclass(frozen=True)
class Refusal:
    function: str
    reason: str
    detail: str = ""

    def __str__(self) -> str:
        tail = f" ({self.detail})" if self.detail else ""
        return f"{self.function}: {self.reason}{tail}"


@dataclass(frozen=True)
class Case:
    """One generated differential case."""

    name: str
    param_kinds: tuple[str, ...]
    param_names: tuple[str, ...]
    return_kind: str
    exhaustive: bool  # the whole input domain is enumerated, not sampled
    tier: str  # TIER_PURE | TIER_NLS
    sign_only: bool = False  # magnitude is arch-dependent; see SIGN_ONLY_ROOTS

    @property
    def out_buffers(self) -> tuple[int, ...]:
        return tuple(i for i, k in enumerate(self.param_kinds) if k in (BUF_OUT, WBUF_OUT))

    @property
    def out_scalars(self) -> tuple[int, ...]:
        """Written-through cells to compare after the call. For the `_s` family these carry
        the ANSWER — grading only the returned errno_t would call two `EINVAL`s parity while
        the counts they wrote disagreed. `mbstate_t` is here too: it is an in-out, and the
        state left behind is the whole point of a restartable conversion."""
        return tuple(i for i, k in enumerate(self.param_kinds) if k in (INT_OUT, SIZE_OUT, MBSTATE))

    @property
    def cursors(self) -> tuple[int, ...]:
        """Cursor cells to compare by OFFSET after the call — never by value. Each side
        walks its own copy of the string, so the raw pointers differ by construction and an
        equality test would report a divergence on every single iteration."""
        return tuple(i for i, k in enumerate(self.param_kinds) if k in (STRCUR, STRCUR_IN))


@dataclass
class Plan:
    module: str
    cases: list[Case] = field(default_factory=list)
    refusals: list[Refusal] = field(default_factory=list)
    symbols: list[str] = field(default_factory=list)

    def summary(self) -> str:
        ex = sum(1 for c in self.cases if c.exhaustive)
        pure = sum(1 for c in self.cases if c.tier == TIER_PURE)
        return (
            f"{self.module}: {len(self.cases)}/{len(self.cases) + len(self.refusals)} functions "
            f"covered ({pure} host-only, {len(self.cases) - pure} need the NLS replay fixture; "
            f"{ex} exhaustive over their input domain), "
            f"{len(self.refusals)} refused, {len(self.symbols)} symbols to rename"
        )


def _is_arch_guarded(fn: CFunction) -> bool:
    return _ARCH_GUARD.search(fn.body_text or "") is not None


#: `#if _MSVCR_VER>=80` and friends.  The harness compiles the reference TU at
#: `_MSVCR_VER=0` (msvcrt.dll's own version), so a body behind one of these is simply
#: NOT THERE — unlike an arch guard, which selects between two bodies that both exist.
_VER_GUARD = re.compile(r"^\s*#\s*if.*\b_MSVCR_VER\b")
_VER_GUARD_END = re.compile(r"^\s*#\s*(endif|else|elif)\b")


def version_guarded_lines(source: str) -> set[int]:
    """1-based line numbers that sit inside a `#if _MSVCR_VER…` region.

    A function defined in one has no `ref_*` symbol to compare against, so a case for it
    would not merely grade the wrong thing — it would fail to LINK.

    This is a BACKSTOP, not the primary defence. When the scanner can resolve a file's
    include closure it preprocesses at `_MSVCR_VER=0` and such functions never reach the
    plan at all; that is what happens for mbcs.c, whose `_mbsncpy_s`, `_mbsncpy_s_l` and
    `___mb_cur_max_l_func` are behind `_MSVCR_VER>=80` and are correctly absent. On a
    file scanned WITHOUT preprocessing (no compile database — the mode the unit tests
    run in) the directives survive and the functions do reach the plan, and then this is
    the only thing standing between them and a broken link.
    """
    inside: set[int] = set()
    depth = 0  # nesting depth of ordinary #if inside a version region
    active = False
    for i, line in enumerate(source.splitlines(), start=1):
        if not active:
            if _VER_GUARD.match(line):
                active, depth = True, 0
            continue
        if line.lstrip().startswith("#if"):
            depth += 1
        elif _VER_GUARD_END.match(line):
            if depth:
                depth -= 1
            else:
                active = False
                continue
        inside.add(i)
    return inside


def plan_file(scan: ScanResult, module: str) -> Plan:
    """Decide, per function, whether a differential case can be GENERATED — or refused."""
    plan = Plan(module=module)
    mutators = _reaches(scan, STATE_MUTATORS)
    nls = _reaches(scan, NLS_SURFACE)
    ver_guarded = version_guarded_lines(scan.raw_source or "")
    sign_only = _reaches(scan, SIGN_ONLY_ROOTS)
    for fn in scan.functions:
        if fn.is_static:
            continue  # internal linkage: no symbol to rename, nothing for a caller to grade
        plan.symbols.append(fn.name)

        if fn.name in mutators:
            # Rebuilds or frees the pinned `threadmbcinfo`. Running it inside a case would
            # invalidate the fixture for every case after it in the same process — which is
            # far worse than not grading it, because the corruption reports as a DIVERGENCE
            # in an innocent function. `_setmbcp` is the one that matters; grade it on its
            # own, in its own process, by hand.
            plan.refusals.append(Refusal(fn.name, "mutates the pinned MBCS state"))
            continue

        kinds = [classify_type(t) for t, _ in fn.params]
        # C `(void)` reaches the scanner as a single parameter; it means "no arguments".
        if kinds == [VOID]:
            kinds, params = [], []
        else:
            params = list(fn.params)

        bad = [k for k in kinds if k not in SUPPLIABLE]
        if bad:
            plan.refusals.append(Refusal(fn.name, "unsupported parameter", ", ".join(bad)))
            continue
        ret = classify_type(fn.return_type)
        # A pointer returned by a function that takes NO arguments cannot point into
        # anything the case supplied — there is nothing to supply. It points into the CRT's
        # own state, which both sides have their own copy of, so the answer is the contents
        # rather than the address. Reclassifying here rather than in classify_type keeps
        # that inference where the whole signature is visible: the identical `int *` as a
        # PARAMETER is an out-scalar and must stay one.
        if not kinds:
            if ret in (INT_OUT, SIZE_OUT):
                ret = STATE_INT
            elif ret == BUF_OUT:
                ret = STATE_TABLE
        if ret not in COMPARABLE:
            plan.refusals.append(Refusal(fn.name, "unsupported return", ret))
            continue
        if fn.line_start in ver_guarded:
            # Compiled out of the reference TU at _MSVCR_VER=0, so there is no `ref_*`
            # symbol at all and a case here would fail to LINK. Only reachable when the
            # file was scanned without preprocessing — see version_guarded_lines.
            plan.refusals.append(
                Refusal(fn.name, "body is behind #if _MSVCR_VER", "no reference symbol exists")
            )
            continue
        if _is_arch_guarded(fn):
            # The reference TU is compiled for the HOST, so an arch arm the target takes is
            # not the arm being graded. Refuse rather than grade the wrong body.
            plan.refusals.append(Refusal(fn.name, "arch-conditional body", "grade by hand"))
            continue
        if _returns_interior_pointer(ret) and not (set(kinds) & set(base_kinds(ret))):
            # A returned pointer with no argument of matching constness to measure it
            # against: equality is not expressible, so the case would be theatre.
            plan.refusals.append(
                Refusal(
                    fn.name, "returns a pointer into nothing we supply", f"want {base_kinds(ret)}"
                )
            )
            continue

        plan.cases.append(
            Case(
                name=fn.name,
                param_kinds=tuple(kinds),
                param_names=tuple(rust_ident(n) or f"a{i}" for i, (_, n) in enumerate(params)),
                return_kind=ret,
                # Enumerable exactly when every input is a bounded character and the rest is
                # state we pin: no string corpus involved, so there is nothing to sample.
                exhaustive=bool(kinds) and all(k in (CHAR, UCHAR, LOCALE) for k in kinds),
                tier=TIER_NLS if fn.name in nls else TIER_PURE,
                sign_only=fn.name in sign_only,
            )
        )
    return plan


# --------------------------------------------------------------------------------------
# emit
# --------------------------------------------------------------------------------------

#: The Rust spelling of each kind at the FFI boundary. `CHAR` is `u32` because the C is
#: `unsigned int` — the MBCS "character" is a packed 1-2 byte value, not a `char`.
_RUST_TY = {
    CHAR: "u32",
    SIZE: "usize",
    UCHAR: "u8",
    LOCALE: "*mut core::ffi::c_void",
    STR_IN: "*const u8",
    BUF_OUT: "*mut u8",
    WSTR_IN: "*const u16",
    WBUF_OUT: "*mut u16",
    INT_OUT: "*mut i32",
    SIZE_OUT: "*mut usize",
    MBSTATE: "*mut i32",
    STRCUR: "*mut *mut u8",
    STRCUR_IN: "*mut *const u8",
    STATE_INT: "*mut i32",
    STATE_TABLE: "*mut u8",
    WINT: "u16",
    VOID: "()",
}

_PROLOGUE = """//! GENERATED by `rustconv difftest-gen` from `{c_path}` — do not edit.
//!
//! Each case calls the SHIPPING Rust symbol and the genuine C (compiled by build.rs as
//! `ref_*` from that same file) in one process and asserts parity. Nothing here is
//! transcribed from the C, so the reference cannot drift from the code it grades.
//!
//! Regenerate with:
//!     python -m rustconv difftest-gen {c_path} --out <this file>
//!
//! {covered} of {total} functions are graded. The {refused} refusals are listed at the
//! bottom of this file WITH THEIR REASON — a refusal is a function that still needs a
//! hand-written case, not one that is known good.
#![allow(non_snake_case)]

use crate::fixture::{{
    state_table_same, Corpus, Guarded, CODEPAGES, LOCALES, MBSTATES, MbcsFixture,
}};

extern "C" {{
"""


def _extern_decls(cases: list[Case]) -> str:
    out = []
    for c in cases:
        args = ", ".join(
            f"{n}: {_RUST_TY[k]}" for n, k in zip(c.param_names, c.param_kinds, strict=True)
        )
        ret = "" if c.return_kind == VOID else f" -> {_RUST_TY[c.return_kind]}"
        out.append(f"    fn {c.name}({args}){ret};")
        out.append(f"    fn ref_{c.name}({args}){ret};")
    return "\n".join(out)


@dataclass(frozen=True)
class _Bind:
    """How one parameter is supplied: its loop (if any), the two call arguments, and how it
    prints in a failure message."""

    loop: str | None
    sub_arg: str
    ref_arg: str
    ctx: str
    setup: str | None = None  # emitted INSIDE the innermost loop (fresh per iteration)


#: Rust keywords that are legal C identifiers. `_mbbtype_l(int c, int type, …)` is the
#: one that bites in mbcs.c, but the whole set is cheap to cover — a name that only
#: shows up in one file's parameter list is exactly the sort of thing that surfaces as a
#: syntax error in generated code months later.
_RUST_KEYWORDS = frozenset(
    [
        "abstract",
        "as",
        "async",
        "await",
        "become",
        "box",
        "break",
        "const",
        "continue",
        "crate",
        "do",
        "dyn",
        "else",
        "enum",
        "extern",
        "final",
        "fn",
        "for",
        "if",
        "impl",
        "in",
        "let",
        "loop",
        "macro",
        "match",
        "mod",
        "move",
        "mut",
        "override",
        "priv",
        "pub",
        "ref",
        "return",
        "self",
        "static",
        "struct",
        "super",
        "trait",
        "true",
        "false",
        "try",
        "type",
        "typeof",
        "union",
        "unsafe",
        "unsized",
        "use",
        "virtual",
        "where",
        "while",
        "yield",
    ]
)


def rust_ident(name: str) -> str:
    """A C parameter name, made safe to use as a Rust binding.

    Suffixed rather than raw-escaped: these names are pasted into failure messages and
    loop headers as well as declarations, and `type_` reads better than `r#type` in a
    panic string.
    """
    return f"{name}_" if name in _RUST_KEYWORDS else name


def cursor_index(names: Sequence[str], kinds: Sequence[str]) -> int | None:
    """Index of a parameter that must point INTO an earlier string parameter.

    `_mbsdec_l(start, cur)`, `_ismbslead_l(start, str)` and friends take a string and a
    CURSOR WITHIN IT. Supplying two independent corpus buffers violates the
    precondition: the functions do pointer arithmetic between the two, so with unrelated
    allocations the answer is whatever the heap layout happened to be. `_mbsdec_l`
    reported "returned offset 18767 vs ref 122479" — two allocations' distance apart,
    not a bug in the code.

    msvcrt's convention names the base `start`, which is the only signal available; the
    two-independent-strings case (`_mbscmp(str, cmp)`, `_mbsspn(string, set)`) is far
    more common and correct as-is.
    """
    if len(kinds) >= 2 and kinds[0] == STR_IN and kinds[1] == STR_IN and names[0] == "start":
        return 1
    return None


def mutates_in_place(kinds: Sequence[str]) -> bool:
    """Is this function's out-buffer the SUBJECT of the operation rather than a sink?

    `_mbslwr(s)` and `_mbsnbset(s, c, n)` take a `unsigned char*` they READ AND REWRITE.
    Handed a blank buffer they see an empty string, do nothing, return the pointer, and
    both sides agree — the case runs, passes, and grades nothing. `_mbsnbcpy(dst, src, n)`
    is the opposite: `dst` is a sink and must start blank, or stale bytes from a previous
    iteration mask a short write.

    What separates them is whether the function has a source string to read FROM. If it
    does, the buffer is a destination; if it does not, the buffer is the input and has to
    be seeded.

    A narrow CURSOR overrides that reading. `_mbstok_s(str, delim, ctx, locale)` has a
    `const unsigned char *` — so the rule above calls `str` a destination and hands it a
    blank buffer — but `delim` is a delimiter SET, not a source to copy from, and `str` is
    the string being tokenized in place. Blank, both sides tokenize an empty string,
    return NULL, agree, and the case grades nothing while reporting a pass. A function
    that keeps a cursor into a mutable buffer is operating ON that buffer, by definition.
    """
    if STRCUR in kinds and BUF_OUT in kinds:
        return True
    # A read-only cursor IS a source string — it is how `mbsrtowcs(wcstr, pmbstr, …)` names
    # the thing it converts FROM. Without it here the rule sees a lone `wchar_t *` with
    # nothing to read, calls the wide sink an in-place subject, and seeds it: stale content
    # in a destination is exactly what masks a short write.
    has_source = STR_IN in kinds or WSTR_IN in kinds or STRCUR_IN in kinds
    return (BUF_OUT in kinds or WBUF_OUT in kinds) and not has_source


def _bind(name: str, kind: str, exhaustive: bool, seeded: bool = False) -> _Bind:
    if kind == CHAR:
        dom = "0..=0xFFFFu32" if exhaustive else "Corpus::CHARS.iter().copied()"
        return _Bind(f"for {name} in {dom} {{", name, name, f"{name}={{{name}:#06x}}")
    if kind == UCHAR:
        # One byte, always enumerated: 256 values is nothing, and sampling a domain this
        # small would be leaving coverage on the table for no saving.
        return _Bind(f"for {name} in 0..=0xFFu8 {{", name, name, f"{name}={{{name}:#04x}}")
    if kind == LOCALE:
        # A PAIR, like the buffers — not one pointer handed to both sides.
        # A `_locale_t` carries `mbcinfo`, so a shared one would aim the reference and
        # the subject at the SAME fixture: a function that wrongly mutates it would
        # then have the reference read the mutated state and agree with it, hiding
        # exactly the bug the two-slot fixture exists to catch.
        return _Bind(
            f"for ({name}_s, {name}_r) in LOCALES.pairs() {{",
            f"{name}_s",
            f"{name}_r",
            f"{name}={{{name}_s:?}}",
        )
    if kind == SIZE:
        return _Bind(f"for {name} in corpus.sizes() {{", name, name, f"{name}={{{name}}}")
    if kind in (STR_IN, WSTR_IN):
        return _Bind(
            f"for ({name}_s, {name}_r) in corpus.pairs() {{",
            f"{name}_s.as_ptr()",
            f"{name}_r.as_ptr()",
            f"{name}={{{name}_s:?}}",
        )
    if kind in (BUF_OUT, WBUF_OUT):
        # Allocated INSIDE the innermost loop. A buffer hoisted out of the loops keeps the
        # previous iteration's bytes, so a short write leaves stale content that can mask the
        # very difference the out-buffer comparison exists to find.
        #
        # `seeded` when the function has no source string, i.e. the buffer IS the input —
        # see mutates_in_place(). A blank one would make `_mbslwr` fold an empty string.
        setup = (
            f"let (mut {name}_s, mut {name}_r) = Guarded::seeded(&seed);"
            if seeded
            else f"let (mut {name}_s, mut {name}_r) = Guarded::pair();"
        )
        # A wide buffer is `*mut u16`, so it gets its own accessor rather than a cast at
        # the call site — a cast would silently accept a byte-sized buffer for a
        # wchar-sized write and overrun it by 2x.
        acc = "wptr" if kind == WBUF_OUT else "ptr"
        return _Bind(None, f"{name}_s.{acc}()", f"{name}_r.{acc}()", "", setup=setup)
    if kind in (INT_OUT, SIZE_OUT):
        # Seeded with a SENTINEL, not zero. A function that fails to write its out-scalar
        # and one that writes 0 are different behaviours, and zero-initialising makes them
        # indistinguishable — both sides would "agree" on a value neither produced. The
        # sentinel is odd and huge so it cannot be a plausible count or errno.
        ty = "usize" if kind == SIZE_OUT else "i32"
        sen = "0x5A5A_5A5A_5A5A_5A5Ausize" if kind == SIZE_OUT else "0x5A5A_5A5Ai32"
        # Declared per-iteration inside the innermost loop for the same reason the out
        # buffers are: a cell hoisted out keeps the previous iteration's value, so a
        # skipped write silently inherits an answer that happens to be right.
        setup = f"let (mut {name}_s, mut {name}_r): ({ty}, {ty}) = ({sen}, {sen});"
        return _Bind(None, f"&mut {name}_s", f"&mut {name}_r", "", setup=setup)
    if kind == MBSTATE:
        # SWEPT, not just zeroed. Zero is the initial conversion state, and seeding only
        # zero leaves the entire resume-a-partial-character path unreachable — which is the
        # half of a restartable conversion that has any state to get wrong. MBSTATES also
        # carries a lead byte, so `mbrtowc` is actually asked to finish a character.
        # Each side gets its own cell: the call writes back through it.
        return _Bind(
            f"for {name}0 in MBSTATES {{",
            f"&mut {name}_s",
            f"&mut {name}_r",
            f"{name}={{{name}0:#x}}",
            setup=f"let (mut {name}_s, mut {name}_r): (i32, i32) = ({name}0, {name}0);",
        )
    if kind in (STRCUR, STRCUR_IN):
        # Placeholder. A cursor binds to a SIBLING parameter — the buffer it walks, and the
        # base its offset is measured from — and `_bind` is handed one parameter at a time
        # with no view of the rest. emit_case rewrites every one of these entries; nothing
        # downstream ever sees this value.
        return _Bind(None, "", "", "")
    raise ValueError(f"no binding for kind {kind}")  # pragma: no cover - planner refuses first


def _verdict(case: Case, ctx: str) -> list[str]:
    """The comparison, formatting the context ONLY on failure.

    An exhaustive case runs 65,536 × len(CODEPAGES) iterations; building a `format!` per
    iteration would cost more than the calls being graded.
    """
    if case.return_kind == VOID:
        return []
    if _returns_interior_pointer(case.return_kind):
        # Subject and reference hold SEPARATE buffers, so their raw pointers can never be
        # equal — compare the offset into each one's own input, and NULL-ness. Comparing the
        # pointers themselves is the classic way to write a difftest that reports MATCH only
        # because both sides returned NULL.
        return [
            "let g = if got.is_null() { -1 } else { got as isize - sub_base as isize };",
            "let w = if want.is_null() { -1 } else { want as isize - ref_base as isize };",
            "if g != w {",
            f'    panic!("{case.name}: returned offset {{g}} vs ref {{w}} — {ctx}");',
            "}",
        ]
    if case.return_kind in (STATE_INT, STATE_TABLE):
        # Both sides read their OWN fixture slot, so the addresses differ by construction —
        # the contents are the answer. NULL-ness is compared first and separately: two NULLs
        # are a legitimate match, but dereferencing to "compare" them would be a fault, and
        # treating one NULL as equal to a zeroed cell would grade a missing accessor as a
        # working one.
        deref = (
            "unsafe { *got } != unsafe { *want }"
            if case.return_kind == STATE_INT
            else f'!unsafe {{ state_table_same("{case.name}", got, want) }}'
        )
        return [
            "if got.is_null() != want.is_null() {",
            f'    panic!("{case.name}: returned {{}} vs ref {{}} — {ctx}",',
            '           if got.is_null() { "NULL" } else { "non-NULL" },',
            '           if want.is_null() { "NULL" } else { "non-NULL" });',
            "}",
            f"if !got.is_null() && {deref} {{",
            f'    panic!("{case.name}: state behind the returned pointer differs — {ctx}");',
            "}",
        ]
    if case.sign_only:
        # Only the SIGN is common ground — see SIGN_ONLY_ROOTS.
        return [
            "let g = (got as i32).signum();",
            "let w = (want as i32).signum();",
            "if g != w {",
            f'    panic!("{case.name}: sign {{g}} vs ref {{w}} '
            f'(got {{got:?}} ref {{want:?}}) — {ctx}");',
            "}",
        ]
    return [
        "if got != want {",
        f'    panic!("{case.name}: got {{got:?}} ref {{want:?}} — {ctx}");',
        "}",
    ]


def case_fn(name: str) -> str:
    """The Rust name of the case for C function `name`.

    Must not equal `name`: the extern block declares the C symbol at module scope.
    A plain prefix keeps this injective — stripping the leading `_` would collide
    `_mbslen` with a hypothetical `mbslen`.
    """
    return f"case_{name}"


def emit_case(case: Case) -> str:
    """One case function for one C function."""
    seeded = mutates_in_place(case.param_kinds)
    binds = [
        _bind(n, k, case.exhaustive, seeded=seeded)
        for n, k in zip(case.param_names, case.param_kinds, strict=True)
    ]
    # A cursor is an OFFSET into the preceding string, not a second string — see
    # cursor_index(). Both sides walk the same distance into their own copy.
    ci = cursor_index(case.param_names, case.param_kinds)
    if ci is not None:
        base = case.param_names[0]
        cur = case.param_names[ci]
        binds[ci] = _Bind(
            f"for {cur}_at in {base}_s.offsets() {{",
            f"{base}_s.as_ptr().add({cur}_at)",
            f"{base}_r.as_ptr().add({cur}_at)",
            f"{cur}=+{{{cur}_at}}",
        )
    # A cursor cell, and the base each side measures its cursor against. STRCUR aims at the
    # mutable buffer the call rewrites; STRCUR_IN owns its own corpus string, because the
    # only other string in those signatures is the WIDE sink the conversion writes to.
    cur_bases: dict[int, tuple[str, str]] = {}
    for i, k in enumerate(case.param_kinds):
        if k not in (STRCUR, STRCUR_IN):
            continue
        n = case.param_names[i]
        if k == STRCUR:
            j = next((x for x, kk in enumerate(case.param_kinds) if kk == BUF_OUT), None)
            if j is None:  # pragma: no cover - planner refuses first
                raise ValueError(f"{case.name}: {n} is a cursor with no buffer to walk")
            b = case.param_names[j]
            sub, ref = f"{b}_s.ptr()", f"{b}_r.ptr()"
            binds[i] = _Bind(
                None,
                f"&mut {n}_s",
                f"&mut {n}_r",
                "",
                setup=f"let (mut {n}_s, mut {n}_r) = ({sub}, {ref});",
            )
        else:
            sub, ref = f"{n}_str_s.as_ptr()", f"{n}_str_r.as_ptr()"
            binds[i] = _Bind(
                f"for ({n}_str_s, {n}_str_r) in corpus.pairs() {{",
                f"&mut {n}_s",
                f"&mut {n}_r",
                f"{n}={{{n}_str_s:?}}",
                setup=f"let (mut {n}_s, mut {n}_r) = ({sub}, {ref});",
            )
        cur_bases[i] = (sub, ref)

    loops = [b.loop for b in binds if b.loop]
    if seeded:
        # The seed loop is OUTERMOST of the value loops, so every other parameter is
        # swept against every seed rather than against one.
        loops.insert(0, "for seed in corpus.items() {")
    ctx = " ".join(["cp={cp}"] + [b.ctx for b in binds if b.ctx])
    if seeded:
        ctx += " seed={seed:?}"

    # See base_kinds: which argument the returned pointer points INTO.
    base = None
    for want in base_kinds(case.return_kind):  # preference order, not source order
        hit = next(
            (
                (b.sub_arg, b.ref_arg)
                for b, k in zip(binds, case.param_kinds, strict=True)
                if k == want
            ),
            None,
        )
        if hit:
            base = hit
            break

    # `case_` prefix, NOT the bare C name: the extern block declares `fn _getmbcp` at
    # module scope, so a case also called `_getmbcp` is a duplicate definition — and if
    # it did compile it would call ITSELF.
    out: list[str] = [f"pub fn {case_fn(case.name)}() {{"]
    # `loops`, not `binds`: the seed loop is synthesised above rather than coming from a
    # parameter, and checking only the binds left a seeded case referring to a `corpus`
    # it never declared.
    if any("corpus" in loop for loop in loops):
        out.append("    let corpus = Corpus::new();")
    out += ["    for &cp in CODEPAGES {", "        let _fix = MbcsFixture::install(cp);"]
    depth = 2
    for loop in loops:
        out.append("    " * depth + loop)
        depth += 1
    pad = "    " * depth

    for b in binds:
        if b.setup:
            out.append(pad + b.setup)
    call = "" if case.return_kind == VOID else "let got = "
    want = "" if case.return_kind == VOID else "let want = "
    out.append(f"{pad}{call}unsafe {{ {case.name}({', '.join(b.sub_arg for b in binds)}) }};")
    out.append(f"{pad}{want}unsafe {{ ref_{case.name}({', '.join(b.ref_arg for b in binds)}) }};")
    if _returns_interior_pointer(case.return_kind) and base:
        out.append(f"{pad}let (sub_base, ref_base) = ({base[0]}, {base[1]});")
    out += [pad + line for line in _verdict(case, ctx)]
    for i in case.out_buffers:
        n = case.param_names[i]
        # `same()` + panic!, never `assert_same(&format!(..))`: the eager format! would
        # run every iteration (millions, for an exhaustive case) to build a string that
        # is used only on failure.
        out.append(f"{pad}if !Guarded::same(&{n}_s, &{n}_r) {{")
        out.append(f'{pad}    panic!("{case.name}: {n} differs — {ctx}");')
        out.append(pad + "}")
    for i in case.out_scalars:
        n = case.param_names[i]
        out.append(f"{pad}if {n}_s != {n}_r {{")
        out.append(f'{pad}    panic!("{case.name}: {n} = {{{n}_s}} vs ref {{{n}_r}} — {ctx}");')
        out.append(pad + "}")
    for i in case.cursors:
        n = case.param_names[i]
        sub, ref = cur_bases[i]
        # By OFFSET, and NULL is its own answer rather than offset 0: `mbsrtowcs` writes
        # NULL through the cursor to mean "consumed to the terminator" and a real position
        # to mean "stopped here", and collapsing the two would grade those as the same
        # outcome. -1 is unreachable as an offset, so it can stand for NULL.
        out.append(f"{pad}let ({n}_go, {n}_wo) = (")
        out.append(
            f"{pad}    if {n}_s.is_null() {{ -1 }} else {{ {n}_s as isize - {sub} as isize }},"
        )
        out.append(
            f"{pad}    if {n}_r.is_null() {{ -1 }} else {{ {n}_r as isize - {ref} as isize }},"
        )
        out.append(f"{pad});")
        out.append(f"{pad}if {n}_go != {n}_wo {{")
        out.append(
            f'{pad}    panic!("{case.name}: {n} at offset {{{n}_go}} vs ref {{{n}_wo}} — {ctx}");'
        )
        out.append(pad + "}")
    for _ in loops:
        depth -= 1
        out.append("    " * depth + "}")
    out += ["    }", "}"]
    return "\n".join(out)


def emit(plan: Plan, c_path: str, total: int) -> str:
    """The whole generated test file."""
    parts = [
        _PROLOGUE.format(
            c_path=c_path, covered=len(plan.cases), total=total, refused=len(plan.refusals)
        ),
        _extern_decls(plan.cases),
        "\n}\n",
    ]
    for tier, label in ((TIER_PURE, "host-only"), (TIER_NLS, "needs the NLS replay fixture")):
        group = [c for c in plan.cases if c.tier == tier]
        if not group:
            continue
        parts.append(f"\n// ===== {label} ({len(group)}) " + "=" * 40 + "\n")
        parts.extend(emit_case(c) + "\n" for c in group)
    parts.append("\n// ===== REFUSED — each needs a hand-written case " + "=" * 26 + "\n")
    parts.extend(f"//   {r}\n" for r in plan.refusals)
    parts.append(_emit_runner(plan))
    return "".join(parts)


def _emit_runner(plan: Plan) -> str:
    """A plain `fn main()` runner — this crate is `harness = false`.

    libtest is unusable here for the reason Cargo.toml documents: the shipping crate
    exports `#[no_mangle]` shims that shadow glibc's, and libtest reads its
    configuration through them before any case runs.
    """
    lines = [
        "\n// ===== runner " + "=" * 58,
        "//",
        "// A plain `fn main()`, not `#[test]`: see Cargo.toml — libtest reads its config",
        "// through the very libc surface this crate overrides.",
        "",
        "pub fn run_all() -> usize {",
        "    let cases: &[(&str, fn())] = &[",
    ]
    lines += [f'        ("{c.name}", {case_fn(c.name)}),' for c in plan.cases]
    lines += [
        "    ];",
        "    for (name, f) in cases {",
        '        eprint!("  {name} ... ");',
        "        f();",
        '        eprintln!("ok");',
        "    }",
        "    cases.len()",
        "}",
        "",
    ]
    return "\n".join(lines)
