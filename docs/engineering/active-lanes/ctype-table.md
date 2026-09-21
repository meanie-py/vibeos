<!-- ucrtbase de-Microsoft-ification quest, 2026-09-20. Produced by a read-only
     survey agent and reviewed before landing. FINDINGS, not a plan of record.
     Two of this round's reports CORRECTED an earlier round's claims, so treat
     every statement here as re-derivable rather than settled: claims are cited
     to file:line, and each report names what it could NOT determine.
     Burn-down: `python3 tools/ucrt_burndown.py --arch i386`.
     Declarations for implemented families: tools/ucrt-takeover.yaml. -->

# UCRT ctype classification tables: generated, verified against `ctype.cpp`

Companion to `ctype-table.rs`. Produced while `sdk/lib/ucrt/locale/ctype.cpp` (745 lines,
Microsoft SDK source, about to be deleted for licence reasons) is still in the tree, so its
data can be used one last time as a **diff oracle** — never as the source of truth. The
source of truth for `ctype-table.rs` is the Unicode Character Database (via Python's
`unicodedata` stdlib module) plus the fixed ISO C rules for the ASCII range. See the method
in section 3 and the full generator script in the appendix.

## 1. Which tables exist, where, and which one the default "C" locale uses

`sdk/lib/ucrt/locale/ctype.cpp` defines four data tables plus two pointers:

| Symbol | Line | Shape | What it is |
|---|---|---|---|
| `_wctype` | ctype.cpp:38 | `unsigned short[258]` (unsized decl, size inferred) | WIDE (Unicode codepoint) classification table, index 0 = EOF(-1), index `i` = codepoint `i-1` for `i` in 1..=257 (covers 0x00..=0x100) |
| `__newctype` | ctype.cpp:296 | `unsigned short[384]` | NARROW (byte) classification table. Only 256 of 384 entries are given an initializer; per C aggregate-initialization rules the remaining 128 are zero-initialized. |
| `__newclmap` | ctype.cpp:445 | `unsigned char[384]` | narrow to-lowercase map |
| `__newcumap` | ctype.cpp:539 | `unsigned char[384]` | narrow to-uppercase map |
| `_pctype` | ctype.cpp:744 | `unsigned short const*` | `{ __newctype + 128 }` — pointer used by `isalpha()` etc. |
| `_pwctype` | ctype.cpp:745 | `unsigned short const*` | `{ _wctype + 1 }` — pointer used by `iswalpha()` etc. |

**Which table the default "C" locale (the state before any `setlocale()` call) points at**
is settled directly by `sdk/lib/ucrt/locale/nlsdata.cpp:94`, inside the initializer for
`__acrt_initial_locale_data`:

```
__newctype + 128,    // _locale_pctype
```

So `isalpha(c)` and friends, before any `setlocale()`, resolve through `__newctype`, not
through `_wctype`. `_wctype` backs the **wide** (`iswalpha` etc.) family only, via
`_pwctype`. This is the fact the earlier survey needed and did not establish precisely —
see section 4.

## 2. Bit values and their meanings (cited)

Both the narrow and wide tables share the same eight low bits, defined identically in two
headers:

```
sdk/include/crt/ctype.h:65   #define _UPPER 0x1
sdk/include/crt/ctype.h:66   #define _LOWER 0x2
sdk/include/crt/ctype.h:67   #define _DIGIT 0x4
sdk/include/crt/ctype.h:68   #define _SPACE 0x8
sdk/include/crt/ctype.h:70   #define _PUNCT 0x10
sdk/include/crt/ctype.h:71   #define _CONTROL 0x20
sdk/include/crt/ctype.h:72   #define _BLANK 0x40
sdk/include/crt/ctype.h:73   #define _HEX 0x80
sdk/include/crt/ctype.h:76   #define _ALPHA (0x0100|_UPPER|_LOWER)
```//
identical redeclaration: `sdk/include/ucrt/corecrt_wctype.h:48-58`.

`_wctype`'s entries additionally carry `C1_ALPHA`, which is **not** declared in any ctype
header — it comes from the NLS/`GetStringTypeW` bit set:

```
sdk/include/psdk/winnls.h:266   #define C1_ALPHA 256
```

`C1_ALPHA == 0x0100` is exactly the high bit of `_ALPHA` (`0x0100|_UPPER|_LOWER`, ctype.h:76).
That is not a coincidence: ctype.cpp's own top-of-file comment (ctype.cpp:29-36) explains the
table is built to agree with `GetStringTypeW`, so `C1_ALPHA` was deliberately placed at the
same bit `_ALPHA` already used as its "is a letter" bit — a letter classified `_UPPER|C1_ALPHA`
satisfies `_ischartype_l(c, _ALPHA)` exactly the same way `_UPPER|_LOWER`-only bits would, since
`_ALPHA`'s test only cares that *some* combination of `{0x100,_UPPER,_LOWER}` is set.

## 3. The diff: generated table vs. Microsoft's table

### Method

1. Generate `UCRT_WCTYPE`/`UCRT_NEWCTYPE` independently in Python (`classify(codepoint)`,
   full listing in the appendix), driven only by:
   - `unicodedata.category(ch)` — Unicode general category (Cc, Lu, Ll, Lo, Nd, No, Cf, P*, S*, Zs)
   - `unicodedata.decomposition(ch)` — to detect **compatibility** decompositions (bracketed
     tag, e.g. `<super>`, `<compat>`, `<fraction>`, `<noBreak>`) vs canonical ones (no tag)
   - `ch.isupper()` / `ch.islower()` — Unicode Uppercase/Lowercase properties
   - a hand-verified Unicode `White_Space=Yes` set for 0x00-0xFF: `{0x09,0x0A,0x0B,0x0C,0x0D,
     0x20,0x85,0xA0}` (**not** Python's `str.isspace()` — see the 0x1C-0x1F note below)
   - the fixed ASCII hex-digit set `0-9A-Fa-f`
2. Extract Microsoft's `_wctype` and `__newctype` initializers from `ctype.cpp` with a
   regex + `eval()` of each comma-separated C expression against the bit constants (so the
   oracle values come from the SAME bit definitions, not independently guessed numbers).
3. Zip the two 258-entry / 384-entry arrays index-by-index and print every index where the
   values differ, decoded back to bit names on both sides.
4. Iterate: every disagreement found was investigated against the real Unicode property that
   explained it, the generator was corrected to use that property (never "copy the MS value"),
   and the diff was rerun.

### Result: **zero disagreements**, after two rounds of generator fixes

**Round 1** (initial generator, using `str.isspace()` for the `_SPACE` bit and only a
Numeric_Type-agnostic `No`-category rule) produced 9 diffs on the wide table and matched the
narrow table exactly already:

| Codepoint | Generated (round 1) | Microsoft | Root cause found |
|---|---|---|---|
| 0x1C-0x1F (FS,GS,RS,US) | `_SPACE|_CONTROL` | `_CONTROL` | Python's `str.isspace()` returns `True` for these four C0 controls due to a legacy Bidirectional_Class-based algorithm in CPython — their real Unicode `White_Space` property is `No`. Fixed by using an explicit `White_Space=Yes` set instead of `str.isspace()`. |
| 0xAA, 0xBA (ordinal indicators) | `_LOWER|C1_ALPHA` | `_LOWER|_PUNCT|C1_ALPHA` | Both have Unicode `Decomposition_Type=Super` (`unicodedata.decomposition('\xaa') == '<super> 0061'`) — a compatibility decomposition marking them as symbol-shaped stand-ins for a letter, not plain letters. Fixed by adding `_PUNCT` for any codepoint with a bracketed (compatibility) decomposition tag. |
| 0xBC, 0xBD, 0xBE (vulgar fractions) | `0` | `_PUNCT` | Category `No` but `unicodedata.digit()` raises (no integer digit value, only `numeric()` value 0.25/0.5/0.75) — the round-1 rule only handled `No` codepoints that DO have a digit value. Fixed by giving every `No` codepoint `_PUNCT` via the general P*/S*-category-adjacent rule below, and reserving `_DIGIT` for the ones with a real `Numeric_Type=Digit` (see next row). |

**Round 2** (after adding the general "compatibility decomposition ⇒ `_PUNCT`" rule) introduced
one new diff and the fix generalized correctly:

| Codepoint | Generated (round 2) | Microsoft | Root cause found |
|---|---|---|---|
| 0xA0 (NO-BREAK SPACE) | `_SPACE|_PUNCT` | `_SPACE` | NBSP's decomposition is `<noBreak> 0020` — a compatibility tag, so the round-2 blanket rule added `_PUNCT`. But `<noBreak>` denotes a line-breaking-behavior variant of the *same* base character (space), not a symbol substitution the way `<super>`/`<compat>`/`<fraction>` are. Fixed by excluding the `<noBreak>` tag specifically from the "compat decomposition ⇒ PUNCT" rule. |

**Round 3**: rerun with both fixes → **0/258 wide-table disagreements, 0/384 narrow-table
disagreements.** Every one of the 514 numbers in `ctype-table.rs` was independently derived
from public Unicode/ISO-C facts and landed on the exact bit pattern Microsoft's table has,
with the single exception noted below (which the generated table deliberately does NOT copy).

Superscript digits 0xB2/0xB3/0xB9 (`SUPERSCRIPT TWO/THREE/ONE`) are the case that makes the
`No`-category split meaningful: they have `Decomposition_Type=Super` too, but ALSO have a real
Unicode digit value (`unicodedata.digit('\xb2') == 2`, `Numeric_Type=Digit`), unlike the
fractions. The generator gives them `_DIGIT` (from the digit-value check) `| _PUNCT` (from the
compat-decomposition rule) — matching Microsoft's `_DIGIT|_PUNCT` for all three, for a reason
independent of Microsoft's choice.

The soft hyphen (0xAD) and the micro sign (0xB5) are handled by the same two general rules
(Unicode category `Cf` ⇒ `_CONTROL|_PUNCT`; compatibility decomposition ⇒ `_PUNCT` on top of
whatever letter/digit bits its category already contributed) rather than being hand-listed as
exceptions — five different "weird" Latin-1-Supplement codepoints (0xAA, 0xAD, 0xB2, 0xB3,
0xB5, 0xB9, 0xBA, 0xBC, 0xBD, 0xBE) all fall out of two general, citable Unicode properties
(`Decomposition_Type`, `Numeric_Type`) plus the ordinary category rules, with no per-codepoint
special-casing left in the final generator (see appendix).

## 4. Delta against our existing `MSVCRT__ctype` — the 0x80-0xFF claim, checked

The claim to verify: *"our existing Rust narrow ctype table (`MSVCRT__ctype`,
`sdk/lib/rust/msvcrt-crt-rust/src/msvcrt/ctype.rs:52`) is ZERO for bytes 0x80-0xFF ... while
UCRT's equivalent (`__newctype`) has real Latin-1-Supplement letter classifications in that
range, and `nlsdata.cpp:94` confirms it is the table the default 'C' locale uses."*

**The first two-thirds are correct and the last third is refuted by the actual data:**

- `nlsdata.cpp:94` does confirm `__newctype + 128` is `_locale_pctype` for the default "C"
  locale (section 1, above) — correct.
- `MSVCRT__ctype` (msvcrt.dll's own narrow table, `sdk/lib/rust/msvcrt-crt-rust/src/msvcrt/ctype.rs:52-83`)
  is indeed all-zero for indices corresponding to char 0x80-0xFF — correct
  (`nonzero_80_ff_existing = []`, i.e. empty).
- **But `__newctype` (UCRT's narrow table) is ALSO all-zero for char 0x80-0xFF.** Extracting
  `__newctype`'s C initializer directly from `ctype.cpp:296-441` shows only 256 of the
  declared 384 entries are given values; under C's aggregate-initialization rule (`§6.7.9p21`
  of the C standard: elements not explicitly initialized get zero), the remaining 128 — which
  are exactly the ones `_pctype = __newctype + 128` maps to char 0x80-0xFF — are
  zero-initialized. Nothing in the array literal supplies Latin-1 letter data for that range;
  the source even says so in a trailing comment at ctype.cpp:441, `// and the rest are 0...`.
  Verified programmatically: `narrow_diffs = []` (empty — the generated
  table and `__newctype`, including its implicit-zero tail, agree at all 384 indices) and a
  direct zero-check of indices 256-383 (char 0x80-0xFF) returns no nonzero entries.
- Comparing `MSVCRT__ctype`'s 256 byte-range entries (0x00-0xFF) against the generated/UCRT
  narrow table byte-for-byte: `existing_vs_generated_diffs = []`
  — **empty**. `MSVCRT__ctype` is not merely "also zero at 0x80-0xFF by coincidence"; it is
  **byte-identical to UCRT's `__newctype`-for-the-default-locale across the entire 0x00-0xFF
  range**, not just the top half.

**So the claim as stated is refuted for the table that actually matters: the default "C"
locale's *narrow* (byte) classification table is 7-bit-ASCII-only in UCRT, exactly like
msvcrt.dll's `MSVCRT__ctype`. Wiring `_ismbb*`/`isalpha`-style byte predicates straight to
`MSVCRT__ctype` gets nothing wrong for the default locale** — it already matches UCRT's own
default-locale narrow table bit-for-bit.

**What the earlier survey was actually seeing** is real, but it is a comparison between two
*different* tables, not two versions of the same one: UCRT's **wide** table `_wctype` (used
only by `iswalpha()` and friends via `_pwctype`, never by `isalpha()`) genuinely does carry
Latin-1-Supplement letter data for codepoints 0x80-0x100 (`_UPPER|C1_ALPHA`, `_LOWER|C1_ALPHA`,
etc. — see the `_wctype` excerpt in ctype.cpp:207-296). That table's Rust counterpart already
exists too: `MSVCRT__wctype` (`sdk/lib/rust/msvcrt-crt-rust/src/msvcrt/ctype.rs:88`), and it
already carries the SAME Latin-1 data for 0x80-0xFF (`0x0101`, `0x0102`, etc. in that file
correspond exactly to `_UPPER|C1_ALPHA`/`_LOWER|C1_ALPHA`). So there is no missing translation
here either — comparing `MSVCRT__ctype` (narrow) against `_wctype` (wide) would indeed show a
"0 vs. real data" difference, but that is comparing the wrong two tables to each other, not a
gap in what is already translated.

One genuine, smaller difference DOES exist between msvcrt.dll's existing `MSVCRT__wctype` and
UCRT's `_wctype` (found while cross-checking the above; not implicated in the original claim,
but worth recording since it bears on any future work translating UCRT's wide table
specifically): comparing `MSVCRT__wctype` entry-by-entry against the generated/UCRT `_wctype`
gives 7 disagreements —

| Codepoint | `MSVCRT__wctype` (msvcrt.dll) | UCRT `_wctype` (= generated) |
|---|---|---|
| 0x09 (TAB) | `_SPACE|_CONTROL|_BLANK` | `_SPACE|_CONTROL` |
| 0x85 (NEL) | `_CONTROL` | `_SPACE|_CONTROL` |
| 0xA0 (NBSP) | `_SPACE|_BLANK` | `_SPACE` |
| 0xAA (ordinal ind., f.) | `_PUNCT` | `_LOWER|_PUNCT|C1_ALPHA` |
| 0xAD (soft hyphen) | `_PUNCT` | `_PUNCT|_CONTROL` |
| 0xB5 (micro sign) | `_PUNCT` | `_LOWER|_PUNCT|C1_ALPHA` |
| 0xBA (ordinal ind., m.) | `_PUNCT` | `_LOWER|_PUNCT|C1_ALPHA` |

These are a real generational difference between msvcrt.dll's older table and UCRT's newer
one (msvcrt.dll predates the Universal CRT and, per its own `_MSVCR_VER`-gated `B110`/`D120`/
`S140`/`L140`/`C140` constants already present in that file's header comment, encodes several
Visual-Studio-version-specific behavior changes — e.g. TAB/NBSP losing `_BLANK` and NEL/ordinal
indicators/soft hyphen/micro sign gaining richer classification are exactly the kind of thing
those version gates exist for). **This is not a bug in either table** — `MSVCRT__wctype` is
correctly msvcrt.dll's own data and should not be overwritten with UCRT's; it is flagged here
so that whoever eventually translates UCRT's `_wctype` for `ucrtbase.dll` does not assume
"we already have this table" and reuse `MSVCRT__wctype` unmodified for the different DLL.

## 5. What I could not determine

- **Non-ASCII/non-Latin-1 behavior of `_wctype` above 0x100, and the multi-byte-locale path
  through `_ischartype_l`/`__acrt_locale_data::ctype1`** (a runtime-generated table built by
  `setlocale()` for non-"C" locales via `GetStringTypeA`/`GetStringTypeW` against a codepage,
  referenced by the `ctype1`/`ctype1_refcount` fields in `nlsdata.cpp`'s
  `__acrt_initial_locale_data` struct but not populated until a real `setlocale()` call) is
  OUT OF SCOPE for this capture: `ctype.cpp` itself contains no such table, only the static
  default-locale ones. I did not attempt to generate or verify it. If ucrtbase's Rust port
  eventually needs codepage-specific (e.g. CP1252) `isalpha()` behavior for a non-C locale,
  that is separate runtime-generated data, not a deletion casualty of this file.
- **`__newclmap`/`__newcumap`** (narrow lower/upper case-mapping tables, ctype.cpp:445/539)
  were read and spot-checked (identity everywhere outside ASCII `A-Za-z`, matching the
  narrow classification table's 7-bit-ASCII-only default-locale behavior) but a full
  independently-generated+diffed table for them was NOT produced — out of scope per the task
  (character-*classification*, not case-mapping), flagged here rather than silently omitted.
- I have **not** run this against a real Windows/UCRT binary or ReactOS's own `isalpha`
  test suite — the verification here is a diff against the *source literal* in `ctype.cpp`,
  not against the compiled `.obj`/runtime behavior. `gen_ctype_tables.py` (referenced in
  `ctype.rs`'s own header comment) evaluates against `msvcrt-sys` island mask values and
  REFUSES to emit on mismatch; an equivalent binary-level check for `ctype-table.rs` was not
  built here and would be worth adding before wiring this in for real.
- I did not chase down why `_wctype` is sized 258 (extending one codepoint past 0xFF, to
  U+0100 LATIN CAPITAL LETTER A WITH MACRON, ctype.cpp:296) instead of stopping at 0x100
  exactly at a round boundary — likely just "one full extra row for iswalpha-family callers
  who pass an out-of-byte-range wchar_t immediately above the table's real subject range" but
  I did not find a comment confirming intent, so this is a guess, flagged as such.

## Appendix: generator core (full script produces both files; this is `classify()`)

```python
HEXDIGITS = set('0123456789ABCDEFabcdef')
WHITE_SPACE = {0x09, 0x0A, 0x0B, 0x0C, 0x0D, 0x20, 0x85, 0xA0}

def classify(cp):
    if cp < 0:
        return 0
    ch = chr(cp)
    cat = unicodedata.category(ch)
    decomp = unicodedata.decomposition(ch)
    is_compat_decomp = decomp.startswith('<')
    v = 0
    if cat == 'Cc':
        v |= _CONTROL
    if cp in WHITE_SPACE:
        v |= _SPACE
    if cp == 0x20:
        v |= _BLANK
    if ch.isupper():
        v |= _UPPER
    if ch.islower():
        v |= _LOWER
    if cat.startswith('L'):
        v |= C1_ALPHA
    if cat == 'Nd':
        v |= _DIGIT
    elif cat == 'No':
        try:
            unicodedata.digit(ch)
            v |= _DIGIT
        except (TypeError, ValueError):
            pass
    if cp < 128 and chr(cp) in HEXDIGITS:
        v |= _HEX
    if cat[0] in ('P', 'S'):
        v |= _PUNCT
    if is_compat_decomp and not decomp.startswith('<noBreak>'):
        v |= _PUNCT
    if cat == 'Cf':
        v |= _CONTROL | _PUNCT
    return v
```
