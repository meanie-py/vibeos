<!-- Research pass for the ucrtbase de-Microsoft-ification quest, convert/ family.
     Produced 2026-09-20 by a read-only survey agent, reviewed before landing.
     This is a FINDINGS document, not a plan of record: it says what the UCRT
     contract is and where our existing msvcrt-graded Rust differs from it. The
     burn-down it feeds is `python3 tools/ucrt_burndown.py --arch i386`; the
     declaration each implemented family lands in is tools/ucrt-takeover.yaml.
     Claims here are cited to file:line -- re-derive before acting on one. -->

# `convert/` family — UCRT vs MSVCRT delta research

Scope: `convert/` (207 exports), the largest ucrtbase family. 169 already have a Rust body in
`sdk/lib/rust/msvcrt-crt-rust/src/` (built/graded at `_MSVCR_VER=0`, i.e. against **msvcrt's**
contract); 38 do not. This file is read-only research — no repo file was edited, nothing was
built.

**Bottom line up front:** the single biggest risk to this family is not a per-function
behavioural nuance, it is **structural**: `msvcrt-crt-rust` is bound (`msvcrt-sys`, see
`generate_bindings.py`) against `dll/win32/msvcrt/include/*.h` compiled at `_MSVCR_VER=0`, and
every `_l`-suffixed / locale-aware / per-thread-buffer function in it dereferences an
**msvcrt-internal struct** (`threadlocaleinfostruct`, msvcrt's `thread_data_t`). Real
`ucrtbase.dll`'s equivalents (`__crt_locale_data`, `__acrt_ptd`) are **different structs with
different field sets**, not just a reordering. This is documented in detail in §0 below and
gates essentially every `_l` export in this family, plus several non-`_l` ones. Read that
section before trusting any individual green diff on a linked ucrtbase build — a clean compile
proves nothing here (`--allow-multiple-definition`, no type checking across the DLL boundary).

---

## §0. The structural risk (applies across nearly all 169 "have" exports with an `_l` twin)

### 0.1 `threadlocaleinfostruct` vs `__crt_locale_data` — NOT layout-compatible at any `_MSVCR_VER`

- Our Rust's `locinfo_of()` (`sdk/lib/rust/msvcrt-crt-rust/src/lib.rs:3457`) resolves a
  `_locale_t` to `*mut sys::threadlocaleinfostruct`, where `sys` = `msvcrt-sys`, bound from
  `dll/win32/msvcrt/include/corecrt.h:274` at `_MSVCR_VER=0` (msvcrt.dll's real build flag —
  `generate_bindings.py` reads it out of msvcrt's own `compile_commands.json`, by design, so it
  is *correct for msvcrt*).
- msvcrt's own header (`dll/win32/msvcrt/include/corecrt.h:274-311`) has a documented
  `#if _MSVCR_VER >= 140` branch that puts `pctype`/`mb_cur_max`/`lc_codepage` at the *front* of
  the struct instead of near the end — this is msvcrt's own acknowledgment that the UCRT
  generation changed the layout, and `msvcrt-sys/generate_bindings.py:9-23` names this exact
  branch as the reason the crate exists.
- Real ucrtbase's locale struct is a **different, unrelated type**:
  `__crt_locale_data` (`sdk/lib/ucrt/inc/corecrt_internal.h:502-518`), reached via
  `_locale_t = __crt_locale_pointers* = {locinfo: *__crt_locale_data, mbcinfo: *...}`
  (`sdk/include/ucrt/corecrt.h:628-634`). The outer two-pointer `_locale_t` shape matches
  msvcrt's `localeinfo_struct{locinfo, mbcinfo}` (`dll/win32/msvcrt/include/corecrt.h:258-262`),
  so a `_locale_t` handle itself is safe to pass around. **The pointee is not.**
- Comparing field-by-field, `__crt_locale_data` has an extra `lc_time_cp` field (right after
  `lc_collate_cp`) that msvcrt's struct never has at any `_MSVCR_VER`, and is **missing**
  msvcrt's `lc_handle[6]` (`__msvcrt_ulong[6]`) and `lc_id[6]` (`LC_ID[6]`) arrays entirely
  (`dll/win32/msvcrt/include/corecrt.h:283-285` vs `corecrt_internal.h:502-518`). Even granting
  the `_MSVCR_VER>=140` reorder puts `pctype`/`mb_cur_max`/`lc_codepage` first (which does match
  UCRT's `_public` sub-struct, `sdk/include/ucrt/corecrt.h:620-625`), **everything after
  `refcount`** — `lconv`, `ctype1`, `pclmap`, `pcumap`, `lc_time_curr`, the locale name array —
  sits at different byte offsets between the two structs, because of the extra/missing fields.
- `msvcrt-sys` has no ucrtbase-specific binding target, no `ucrtbase-sys` crate exists in
  `sdk/lib/rust/` at all, and `generate_bindings.py`'s whole design (§ "WHY THE FLAGS COME FROM
  compile_commands.json") is pinned to **msvcrt's** compile command specifically.
- **What this means concretely:** any `_l` function in this family compiled as-is into ucrtbase,
  given a real (non-null) `_locale_t` from `_create_locale`/`setlocale` on the ucrtbase side,
  reads `li->pctype` (and every other field) at the wrong offset into `__crt_locale_data` —
  silent memory misread, not a crash, not a wrong-but-plausible-looking value necessarily either
  (offsets land on unrelated fields: `lc_time_cp`, parts of `lc_category`, etc.).
- **What I could NOT determine (read-only task, said explicitly not to build):** whether the
  quest's plan is to (a) write a `__crt_locale_data`-shaped adapter/shim before flipping any
  `_l` export, (b) generate a second, ucrtbase-targeted `sys` binding, or (c) this simply hasn't
  been reached yet. `docs/engineering/rust-port-status.md:79` states ucrtbase currently links
  vendored Microsoft `sdk/lib/ucrt` C plus **`msvcrt_shared`** (the `cpp.c`/`except.c`/
  `undname.c` family) — **not** `msvcrt-crt-rust` (the convert/ctype/locale bodies this research
  covers). If that is still accurate, none of the below is live-wired into ucrtbase yet, and
  this whole section is the blocking prerequisite for wiring it, not an active bug. Whoever picks
  up `convert/` next should re-verify current link composition before assuming either way.

### 0.2 The NULL-locale (default/current) path is a *separate* open question

Every non-`_l` wrapper in this family calls its `_l` twin with `locale = nullptr`, which routes
through `sys::get_locinfo()` (`lib.rs:3459`) — an msvcrt-internal helper resolving msvcrt.dll's
*own* global/TLS locale state. If linked into ucrtbase.dll, this symbol either (a) fails to
resolve at all (loud, safe), or (b) resolves via the shared cohab archive to msvcrt's own C
locale globals, decoupled from whatever `setlocale()` calls ucrtbase's own callers made. I could
not determine which from static reading; flagging as unresolved rather than asserting.

### 0.3 `mbstate_t`: `int` (msvcrt) vs an 8-byte 3-field struct (UCRT) — lower severity, still real

- msvcrt: `typedef int mbstate_t;` (`dll/win32/msvcrt/include/wchar.h:30`).
- UCRT: `typedef struct _Mbstatet { unsigned long _Wchar; unsigned short _Byte, _State; } mbstate_t;`
  (`sdk/include/ucrt/corecrt.h:636-642`) — 8 bytes, not 4.
- Our Rust (`sdk/lib/rust/msvcrt-crt-rust/src/msvcrt/mbcs.rs:3055-3260`, `mbrlen`/`mbrtowc`)
  treats `*state` as a **plain scalar** (`s = *str as u8 as mbstate_t;`), never as a struct with
  named fields. Because `_Wchar` (the first UCRT field) is also 4 bytes and sits first, a
  zero-initialized 8-byte UCRT `mbstate_t` passed in and read/written as if it were a 4-byte int
  will, in the common case, only touch `_Wchar` and leave `_Byte`/`_State` at their initial
  (zeroed) value — this likely **works by accident** for state our code itself produces and
  consumes, but:
  - any UCRT-side code (not being replaced) that inspects `_Byte`/`_State` directly will read
    stale/zero values our code never updates.
  - not verified against any test; flagging as "probably survives, not proven."
- Applies to: `mbrlen`, `mbrtowc`, `mbsrtowcs`, `mbsrtowcs_s`, `wcrtomb`, `wcrtomb_s`,
  `wcsrtombs`, `wcsrtombs_s` (all in "have"), and would apply to the missing `mbrtoc16`/
  `mbrtoc32`/`c16rtomb`/`c32rtomb` too if implemented the same way.

### 0.4 Per-thread conversion buffer: msvcrt's `thread_data_t.efcvt_buffer` vs ucrtbase's `__acrt_ptd._cvtbuf`

- Our `_ecvt`/`_fcvt`/`_gcvt` (non-`_s`) Rust (`sdk/lib/rust/msvcrt-crt-rust/src/msvcrt/math.rs:1505-1905`)
  read/allocate `(*data).efcvt_buffer`, a field on msvcrt's internal per-thread-data struct.
- Real UCRT's `_ecvt`/`_fcvt` (`sdk/lib/ucrt/convert/fcvt.cpp:17-31`, `try_get_ptd_buffer`) use
  `raw_ptd->_cvtbuf` on ucrtbase's own `__acrt_ptd`, a different struct entirely.
- Same class of risk as §0.1/§0.3: a third msvcrt-internal struct this crate depends on that has
  no known ucrtbase equivalent binding. `ecvt`/`ecvt_s`/`fcvt`/`fcvt_s`/`gcvt`/`gcvt_s` are all
  in "have" (`atox.cpp`/`fcvt.cpp`/`gcvt.cpp`), so this touches 6 more exports.

**Net:** three separate msvcrt-internal structs (`threadlocaleinfostruct`, plain-int
`mbstate_t`, `thread_data_t`) stand between "compiles and links into ucrtbase.dll" and "behaves
correctly there." Recommend this become its own tracked item before flipping any cohab in this
family — a green `vibeos-cli gate` will not catch any of it (no type across the DLL boundary,
`--allow-multiple-definition` swallows the redefinition silently).

---

## §1. Table: the 38 missing exports

All 38 have **zero winetest coverage** in
`modules/rostests/winetests/ucrtbase/{string,misc}.c` (checked every symbol individually,
underscore-prefixed real name and bare name both — 0 hits across the board). Difficulty below
assumes §0 is resolved first for anything locale/`_l`.

### 1a. Genuinely zero new algorithm — real UCRT exports are `.spec` forwards to bodies we already have

Verified against `dll/win32/ucrtbase/ucrtbase.spec`. These are literally the same function
under a different export name; ReactOS's ucrtbase build resolves them as PE forwarders, not
distinct C bodies.

| Export | Spec forward target | We already have |
|---|---|---|
| `strtoll` | `_strtoi64` | `strtoi64` |
| `_strtoll_l` (missing list: `strtoll_l`) | `_strtoi64_l` | `strtoi64_l` |
| `strtoull` | `_strtoui64` | `strtoui64` |
| `_strtoull_l` (`strtoull_l`) | `_strtoui64_l` | `strtoui64_l` |
| `strtoimax` | `_strtoi64` | `strtoi64` |
| `_strtoimax_l` (`strtoimax_l`) | `_strtoi64_l` | `strtoi64_l` |
| `strtoumax` | `_strtoui64` | `strtoui64` |
| `_strtoumax_l` (`strtoumax_l`) | `_strtoui64_l` | `strtoui64_l` |
| `wcstoll` | `_wcstoi64` | `wcstoi64` |
| `_wcstoll_l` (`wcstoll_l`) | `_wcstoi64_l` | `wcstoi64_l` |
| `wcstoull` | `_wcstoui64` | `wcstoui64` |
| `_wcstoull_l` (`wcstoull_l`) | `_wcstoui64_l` | `wcstoui64_l` |
| `strtold` | `strtod` | `strtod` |
| `_strtold_l` (`strtold_l`) | `_strtod_l` | `strtod_l` |
| `wcstold` | `wcstod` | `wcstod` |
| `_wcstold_l` (`wcstold_l`) | `_wcstod_l` | `wcstod_l` |

Source: `sdk/lib/ucrt/convert/strtox.cpp:120-220` confirms `parse_integer_from_string<T>` is
literally shared between `_strtoi64`/`strtoll`/`strtoimax` (all `T=__int64`/`long long`/
`intmax_t`, which are the same 64-bit type on this ABI). `strtod.cpp:82-93` confirms `strtold`
calls `common_strtod_l<double>` — **UCRT's `long double` is IEEE `double`, not an 80-bit x87
extended value** (contrast with `_atoldbl`/`__STRINGTOLD`, §2, which genuinely is 80-bit — see
the surprise noted in §4).

**Build-system note (out of scope to resolve here):** whether "already forwarded" in the
`.spec` is enough on its own, or still needs a thin `#[no_mangle]` Rust re-export under the new
name depends on how this repo's cohab/link step consumes `ucrtbase.spec` forwards — not
determined by this read-only pass.

### 1b. Trivial one-line-body twins — same algorithm as an existing "have" export, different symbol

| Export | Real UCRT body | Notes |
|---|---|---|
| `atoll` | `sdk/lib/ucrt/convert/atox.cpp:41-44`: `parse_integer_from_string<long long>(str, null, 10, null)` | Identical to our `atoi64` minus the `char**`-endptr param and forced base 10 |
| `_atoll_l` (`atoll_l`) | `atox.cpp:46-49` | Same, `+locale` |
| `_wtoll` (`wtoll`) | `atox.cpp:89-92` | Same, wide |
| `_wtoll_l` (`wtoll_l`) | `atox.cpp:94-97` | Same, wide + locale |
| `_iswcsym_l` (`iswcsym_l`) | `sdk/lib/ucrt/convert/_wctype.cpp:154-157`: `return __iswcsym(c);` — **locale param is ignored** | We already have `iswcsym`; wrap it, ignore `_locale_t` |
| `_iswcsymf_l` (`iswcsymf_l`) | `_wctype.cpp:164-167`, same pattern | Wrap `iswcsymf` |

### 1c. Real, distinct exports but same algorithm as an existing "have" body — moderate, not blocked

`wcstoimax`/`wcstoumax` are **not** `.spec`-aliased (unlike their signed/unsigned `strtoXXX`
cousins) — `dll/win32/ucrtbase/ucrtbase.spec:2658,2668` list them as standalone entries with no
forward target, and `_wcstoimax_l`/`_wcstoumax_l` likewise (`:2115,2125`). The real UCRT source
(`strtox.cpp`) still routes them through the identical `parse_integer_from_string<intmax_t/
uintmax_t>` template used for `wcstoi64`/`wcstoui64`, so the work is "port the same function
under one more name," not new logic.

| Export | Notes |
|---|---|
| `wcstoimax` | same as `wcstoi64` |
| `_wcstoimax_l` (`wcstoimax_l`) | same as `wcstoi64_l` |
| `wcstoumax` | same as `wcstoui64` |
| `_wcstoumax_l` (`wcstoumax_l`) | same as `wcstoui64_l` |

`wctrans`/`wctype`: trivial static string-table lookups, no locale dependency at all.
`sdk/lib/ucrt/convert/wctrans.cpp` (44 lines: a 2-entry table `{"tolower",2},{"toupper",1}`,
`towctrans` dispatches to `towupper`/`towlower` which we already have) and
`sdk/lib/ucrt/convert/wctype.cpp` (56 lines: a 12-entry `{name, mask}` table using the SAME
`_ALPHA`/`_DIGIT`/etc. mask constants we already use, see §3).

| Export | Difficulty |
|---|---|
| `wctrans` | trivial — 2-entry `strcmp` table |
| `wctype` | trivial — 12-entry `strcmp` table, reuses existing mask constants |

### 1d. Moderate — real new logic, self-contained, not locale-dependent

`mbstrnlen`/`_mbstrnlen_l`: `sdk/lib/ucrt/convert/_mbslen.cpp:23-70` is a bounded variant of
`mbstrlen` sharing a `common_mbstrlen_l(string, max_size, locale)` helper — if our existing
`mbstrlen`/`mbstrlen_l` body already factors similarly, adding the `max_size` bound is a small
diff, not new design. Uses `__acrt_MultiByteToWideChar` with `MB_ERR_INVALID_CHARS` for
validation and `_isleadbyte_fast_internal` for the scan — worth checking our `mbstrlen` uses the
equivalent leadbyte check before assuming a trivial port.

| Export | Difficulty | Test |
|---|---|---|
| `_mbstrnlen` (`mbstrnlen`) | moderate (shares logic with `mbstrlen`) | none |
| `_mbstrnlen_l` (`mbstrnlen_l`) | moderate | none |

`c16rtomb`/`c32rtomb`: C11 UTF-16/UTF-32 code-unit → UTF-8 encoders. **Not locale-dependent at
all** (comment in the vendored source: "TODO: Bug 13307590 says this is always assuming UTF-8" —
i.e. deliberately hardcoded, not codepage-driven). Self-contained ~50-90 line functions
(`sdk/lib/ucrt/convert/c16rtomb.cpp`, `c32rtomb.cpp`), surrogate-pair-aware for the 16-bit one.
No §0 exposure since no locale struct or (beyond the state scalar) `mbstate_t` internals are
read as fields.

| Export | Difficulty | Test |
|---|---|---|
| `c16rtomb` | moderate (surrogate pairing across calls, ~90 lines of real logic) | none |
| `c32rtomb` | moderate (~90 lines, single code point, no pairing) | none |

`mbrtoc16`/`mbrtoc32`: the decode direction (UTF-8/multibyte → UTF-16/UTF-32 code unit). Did
not fully read these (88/148 lines respectively) but by symmetry with the encode side expect
similar moderate difficulty, plus partial-multibyte-sequence state tracking across calls (this
is where the `mbstate_t` size mismatch, §0.3, is most likely to actually matter, since more
state needs to survive between calls than a single scalar tag for the trickiest UTF-8 sequences).

| Export | Difficulty | Test |
|---|---|---|
| `mbrtoc16` | moderate-to-hard (partial-sequence state across calls) | none |
| `mbrtoc32` | moderate-to-hard | none |

### 1e. Blocked on the same fix as the strtod/atof delta (§2) — do not implement standalone

`strtof`/`_strtof_l`/`wcstof`/`_wcstof_l` are real, distinct UCRT exports
(`sdk/lib/ucrt/convert/strtod.cpp:53-70,108-125`) that route through the exact same
`parse_floating_point`/hex-float/inf-nan machinery as `strtod`. **Implementing these before the
§2 fpnum gap is closed means shipping something that fails the same winetest cases `strtod`
would fail** (hex floats, "infinity"/"nan" words) — see the one existing test,
`modules/rostests/winetests/ucrtbase/string.c:155-186` (`test_strtof`), which explicitly checks
`ERANGE` overflow-to-`INFINITY` behaviour that needs the same overflow path as `strtod`.

| Export | Difficulty | Test |
|---|---|---|
| `strtof` | moderate, but gated on §2 | `string.c:155` `test_strtof` (10 cases) |
| `_strtof_l` (`strtof_l`) | moderate, gated on §2 | none directly, same code path |
| `wcstof` | moderate, gated on §2 | none |
| `_wcstof_l` (`wcstof_l`) | moderate, gated on §2 | none |

---

## §2. THE central delta — `strtod`/`atof`/`wcstod`/`_atoldbl`/`__STRINGTOLD` family (already flagged in our own Rust, confirmed against both winetest suites and the vendored UCRT source)

This is not a hunch — it is **already documented in our own shipped code** at
`sdk/lib/rust/msvcrt-crt-rust/src/fpnum.rs:34-42`, and this research independently confirms it
against both winetest suites and the vendored Microsoft source.

**Every export that calls `fpnum_parse` is affected** (verified by grepping every call site,
`fpnum.rs`): `strtod`, `_strtod_l`, `atof`, `_atof_l`, `_atoflt_l`, `_atodbl_l`, `_atodbl`,
`wcstod`, `_wcstod_l`, `_wtof`, `_wtof_l`, `__STRINGTOLD`, `__STRINGTOLD_L`, `_atoldbl_l`,
`_atoldbl` — 15 of the 169 "have" exports (plus, per §1e, blocks 4 of the 38 missing ones:
`strtof`/`_l`/`wcstof`/`_l`).

### 2.1 Hex-float literals (`0x1.8p3` style) — MSVCRT never parses them, UCRT always does

- Evidence UCRT requires it: `modules/rostests/winetests/ucrtbase/string.c:118-143` — 19
  distinct `test_strtod_str` cases starting `"0x42"` through
  `"0x1fffffffffffff.80000000000000000001"` (37-char round-off-precision hex mantissa cases),
  all asserting exact parsing (not just "doesn't crash").
- Evidence MSVCRT never had it: `modules/rostests/winetests/msvcrt/string.c` — grep for `"0x`
  near any `strtod`/`atof` test returns nothing; `test__strtod` (`msvcrt/string.c:2078-2114`)
  has zero hex-float cases.
- Evidence in the vendored source: `sdk/lib/ucrt/inc/corecrt_internal_strtox.h:1439-1451`
  (optional `"0x"`/`"0X"` prefix detection) and `:1571-1588` (`'p'`/`'P'` exponent introducer,
  gated on `is_hexadecimal`, vs `'e'`/`'E'` gated on `!is_hexadecimal` — see 2.3 below).
- Our Rust: `fpnum.rs:36` states this outright ("`fpnum_parse16` — hex-float ... does not exist
  here either").

### 2.2 `"infinity"`/`"inf"`/`"nan"` word parsing — MSVCRT rejects, UCRT accepts

- UCRT: `string.c:101-116` — `"infinity"`, `"INFINITY"`, `"InFiNiTy"`, `"INF"`, `"-inf"`,
  `"inf42"` (partial match, stops after `"inf"`), `"NAN"`/`"nan"`/`"NaN"` all parse to
  `INFINITY`/`NAN` with the correctly-truncated `end` pointer.
- MSVCRT: `modules/rostests/winetests/msvcrt/string.c:2088-2089` — `{"INF", 0, 0}` and
  `{"NAN", 0, 0}` are explicit test rows asserting **zero parsed characters, result 0.0** (i.e.
  MSVCRT's `strtod` treats `"INF"`/`"NAN"` as complete garbage, not even a partial match).
- Vendored source: `corecrt_internal_strtox.h:1432-1438` dispatches to
  `parse_floating_point_possible_infinity`/`_nan` the instant the first character is `I`/`i` or
  `N`/`n` — including the parenthesized `NAN(...)`/`NAN(SNAN)`/`NAN(IND)` forms
  (`:1350-1396`), which is even more surface than the plain word match.
- Our Rust: `fpnum.rs:37` states this outright too.

### 2.3 `'d'`/`'D'` exponent character (Fortran/VAX-style, e.g. `"12.1d2"`) — MSVCRT accepts, UCRT drops it

**Not called out in our own code's doc comment — found independently in this research; treat as
higher-confidence-needed than 2.1/2.2 since it's an absence-of-evidence argument, not a positive
UCRT test asserting rejection.**

- MSVCRT: `modules/rostests/winetests/msvcrt/string.c:2091,2098` — `{"12.1d2", 6, 12.1e2}` and
  `{"0.1d238", 7, 0.1e238}` are explicit accepted-parse test rows.
- Our Rust: `fpnum.rs:589-591` implements this — `if nch == b'e' ... || nch == b'd' as u16 ||
  nch == b'D' as u16` — with a comment explicitly noting it as "a Fortran-ism."
- UCRT: `corecrt_internal_strtox.h:1578-1588` — the exponent-introducer `switch` has exactly two
  cases, `'e'/'E'` (decimal) and `'p'/'P'` (hex); **no `'d'`/`'D'` case exists**.
- No `ucrtbase` winetest exercises `'d'`/`'D'` either way (checked — zero occurrences of a
  `d`-exponent literal anywhere in `ucrtbase/string.c`), so this is **inferred from the vendored
  source's exhaustive `switch`, not test-confirmed** — flagging the confidence gap explicitly per
  the task's own instruction not to guess. If someone re-derives this, the fastest check is
  compiling a tiny harness against `parse_floating_point_from_source` directly.

### 2.4 `strtold`/`_atoldbl` precision: UCRT's `strtold` truncates to `double`; `_atoldbl`/`__STRINGTOLD` stays 80-bit — do not conflate the two families

Surprising enough to call out on its own (also in §4): UCRT's *modern* `strtold`
(`strtod.cpp:82-93`, and confirmed by the `.spec` forward `strtold -> strtod`, §1a) discards
80-bit x87 extended precision — `long double` is just `double` on this ABI, full stop. But
`_atoldbl`/`_atoldbl_l` (the legacy MS-internal name, still exported by ucrtbase,
`sdk/lib/ucrt/convert/atoldbl.cpp:492-510`) **is** genuinely 80-bit — it still builds a
`_LDBL12`/`_LDOUBLE` via the same bignum machinery as msvcrt's `MSVCRT__LDOUBLE`
(10-byte struct, matching our `MsvcrtLdouble([u8;10])` at `fpnum.rs:1080`). So: **the ABI
mismatch of §2.1/§2.2/§2.3 (hex float, inf/nan, no `'d'` exponent) applies to `_atoldbl` too**
(it shares `parse_floating_point`/`common_convert_to_ldbl12`, confirmed at
`atoldbl.cpp:492-500`), but the **80-bit-vs-64-bit representation question does not** — our
existing `MsvcrtLdouble`/`__STRINGTOLD_L` 10-byte layout is the right shape for `_atoldbl`, just
missing the parsing-surface delta.

### 2.5 Rounding — could not confirm or refute a *correctly-rounded* claim either way

The task background raises "UCRT is correctly-rounded, older MSVCRT was not" as a known
interesting case. I looked for evidence and could not find a clean confirmation or refutation:

- Both winetest files include the classic hard-to-round boundary cases
  (`"1.7976931348623158e+308"` = `DBL_MAX`, `"2.2250738585072014e-308"` = `DBL_MIN`,
  `"4.9406564584124654e-324"` = smallest denormal) with **identical expected values** in both
  `ucrtbase/string.c:105-110` and `msvcrt/string.c:2101-2104` — so on these specific inputs there
  is no observable delta.
- Our own Rust's doc comment (`fpnum.rs:16-23`) asserts msvcrt's own `string.c` bignum parser
  (which this ported) **disagreed by 14 ULPs** with a naive accumulator on one adversarial input
  — i.e. msvcrt's *own* bignum parser is already claimed correctly-rounded (or at least much
  better than naive) by our existing code, which weakens (but doesn't disprove) the premise that
  UCRT is meaningfully *more* correctly-rounded than what we already ported.
- **Explicitly flagging as UNDETERMINED**: I did not diff UCRT's `parse_floating_point`'s actual
  rounding algorithm (`corecrt_internal_strtox.h`, hundreds of lines of round-to-nearest-even
  bit-twiddling around `assemble_floating_point_value`, lines ~700-950) against msvcrt's
  `bnum.h`-derived one closely enough to assert they diverge on any concrete input. This would
  need either a targeted differential-fuzz harness or a much deeper read than a research pass
  budget allows — recommend whoever implements this family builds exactly the kind of bit-exact
  harness `fpnum.rs`'s own doc comment describes (`classify`, comparing raw `u64` bit patterns
  against a real compiled UCRT reference) rather than trusting a table of edge cases.

---

## §3. Exports checked and found IDENTICAL (examined, not skipped)

- **`_ctype`/`_wctype` mask bit values**: `_UPPER=0x1`, `_LOWER=0x2`, `_DIGIT=0x4`, `_SPACE=0x8`,
  `_PUNCT=0x10`, `_CONTROL=0x20`, `_BLANK=0x40`, `_HEX=0x80`, `_LEADBYTE=0x8000`,
  `_ALPHA=(0x100|_UPPER|_LOWER)` are **byte-for-byte identical** between
  `dll/win32/msvcrt/include/wchar.h:165-176` and `sdk/include/ucrt/corecrt_wctype.h:48-58`. The
  task's "known interesting case" about ctype/wctype table layout differing did **not** pan out
  as an ABI/bitmask issue — see §0.1 for where the *real* ctype-adjacent risk actually is
  (`threadlocaleinfostruct.pctype`'s *offset*, not the mask values the table stores).
- `isalnum`/`isalpha`/`isdigit`/... and their `iswXXX` wide twins and `_l` variants: masking
  logic (`fast_check_*` in `sdk/lib/ucrt/convert/_ctype.cpp:18-80`) is the same table-index +
  mask-and pattern as msvcrt; no semantic delta found in the classification rules themselves
  (only in *which struct* the table pointer is read from, §0.1).
- `strtol`/`_strtol_l`/`strtoul`/`_strtoul_l`/`wcstol`/`wcstoul` and 64-bit twins
  (`strtoi64`/`strtoui64`): algorithm (`sdk/lib/ucrt/convert/strtox.cpp:60-400`,
  `parse_integer_from_string<T>`) — base auto-detection (`0`→octal, `0x`→hex), overflow
  sentinels (`INT_MAX`/`INT_MIN`/`UINT_MAX` etc.) — is the textbook C89 algorithm, unchanged
  across CRT generations; no delta found. (Did not verify the `_locale_t` dereference inside
  these `_l` variants is safe — that's §0.1, applies here too, just not a *semantic* delta.)
- `swab`: trivial byte-pair swap, no locale involvement, identical everywhere.
- `wctrans`/`wctype`-adjacent constants (the 12-entry name table in `wctype.cpp`) use the same
  mask constants as `isctype`/`iswctype` — confirmed consistent, not a separate table.

---

## §4. What surprised me

1. **The `.spec`-forward discovery (§1a) cuts the "38 missing" number of real new bodies roughly
   in half.** 16 of the 38 are literal PE-forward aliases in `ucrtbase.spec` to functions we
   already have under a different name (`strtoll→_strtoi64`, `strtold→strtod`, etc.) — this
   wasn't obvious from the export-count alone and only showed up by reading the `.spec` file
   directly rather than trusting the source-file/export-count breakdown in the payload JSON.
2. **`strtold`/`wcstold` are aliased directly to `strtod`/`wcstod` in real UCRT — `long double`
   is IEEE double, full stop** — while the *separately-named* legacy `_atoldbl`/`__STRINGTOLD`
   family genuinely still produces an 80-bit value via the same `_LDBL12` bignum machinery our
   Rust already ported. These look like they should be "the same feature, C99 name vs MS legacy
   name" and are actually **different precision families that happen to share a parser
   front-end**. Easy trap: implement `strtold` by copy-pasting `_atoldbl`'s logic (or vice
   versa) and get the width wrong.
3. **The ctype/wctype table-layout risk the task flagged turned out to be a red herring at the
   *bitmask* level** (§3 — identical), but there's a real, much larger, and completely different
   ABI risk one layer down that the task didn't name: **which internal struct the `_l` locale
   handle actually points to** (§0.1). This is systemic across the whole family (and likely
   beyond it — anything with an `_l` twin anywhere in ucrtbase-targeted Rust is exposed), not
   specific to `convert/`.
4. **The same "wrong internal struct" risk shows up twice more**, independently, in areas that
   look unrelated at first glance: `mbstate_t` being a scalar `int` in msvcrt vs a real 3-field
   struct in UCRT (§0.3), and the `_ecvt`/`_fcvt` per-thread scratch buffer living on two
   differently-shaped per-thread-data structs (§0.4). Three separate instances of the identical
   underlying problem (msvcrt-sys binds only msvcrt's own headers) strongly suggests this needs
   a repo-level decision (a real `ucrtbase-sys` binding, or an adapter layer) rather than three
   separate per-function patches.
5. **UCRT genuinely drops a corner of MSVCRT's grammar** (§2.3, the `'d'`/`'D'` exponent
   character) rather than only ever being a strict superset — I went in expecting "UCRT adds
   hex-float and inf/nan on top of MSVCRT's grammar" and that's *mostly* true, but the Fortran-
   style exponent letter is a case where UCRT is narrower, not wider. Anyone porting `strtod`
   forward by only "detecting what's missing and adding it" would ship a regression here.
