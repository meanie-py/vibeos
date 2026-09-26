<!-- Research pass for the ucrtbase de-Microsoft-ification quest, mbstring/ family.
     Produced 2026-09-20 by a read-only survey agent, reviewed before landing.
     This is a FINDINGS document, not a plan of record: it says what the UCRT
     contract is and where our existing msvcrt-graded Rust differs from it. The
     burn-down it feeds is `python3 tools/ucrt_burndown.py --arch i386`; the
     declaration each implemented family lands in is tools/ucrt-takeover.yaml.
     Claims here are cited to file:line -- re-derive before acting on one. -->

# mbstring/ — UCRT-vs-MSVCRT delta research

Repo: `/home/curi/Projects/os-playground/vibeos-msvcrt-homebrew`. Read-only research, no edits made.
Source of the 191-export split: `~/vibeos-rust-staging/ucrt/mbstring-payload.json` (155 `have`, 36 `missing`).

## 1. The `_ismbb*` family (`mbstring/ismbbyte.cpp`, 24 exports)

**Confirmed: one table-driven generic.** Every export in this file is a thin wrapper around a
single static helper:

```
sdk/lib/ucrt/mbstring/ismbbyte.cpp:203
static int __cdecl x_ismbbtype_l (_locale_t plocinfo, unsigned int tst, int cmask, int kmask) throw()
{
    _LocaleUpdate _loc_update(plocinfo);
    tst = (unsigned int)(unsigned char)tst;
    return  ((*(_loc_update.GetLocaleT()->mbcinfo->mbctype+1+tst)) & kmask) ||
            ((cmask) ? ((*(_loc_update.GetLocaleT()->locinfo->_public._locale_pctype + tst)) & cmask) : 0);
}
```

Two tables, ORed:
- **kmask** against `mbcinfo->mbctype[1+tst]` — the per-codepage MBCS classification table
  (bits `_MS`/`_MP`/`_M1`/`_M2`). In our Rust this is `(*mbcinfo).mbctype`, an array on the
  `pthreadmbcinfo` struct, built at runtime by `_setmbcp` — see §5.
- **cmask** against `locinfo->_public._locale_pctype[tst]` — the plain single-byte ctype table
  (bits `_ALPHA`/`_DIGIT`/`_PUNCT`/`_BLANK`/…). In our Rust this is `MSVCRT__ctype`
  (`sdk/lib/rust/msvcrt-crt-rust/src/msvcrt/ctype.rs:52`), reached the same way `_isalpha_l` etc.
  already reach it.

Every one of the 24 exports (`sdk/lib/ucrt/mbstring/ismbbyte.cpp:78`–`214`) is exactly:
`extern "C" int _ismbbXXX_l(unsigned int tst, _locale_t plocinfo) { return x_ismbbtype_l(plocinfo, tst, CMASK, KMASK); }`
with a fixed `(cmask, kmask)` pair per name, plus the no-`_l` twin that just calls the `_l` one
with `nullptr`. Two exceptions that are NOT pure table lookups:
- `_ismbbblank(_l)` (`ismbbyte.cpp:172`): special-cases `tst == '\t'` before falling through —
  mirrors `isblank()`'s own tab special-case (see `_ctype.cpp:169`, and our narrow-table comment
  at `ctype.rs` noting tab is deliberately NOT marked `_BLANK` in the table).
- `_ismbbkana(_l)` (`ismbbyte.cpp:207`): not `x_ismbbtype_l` at all — pure range check
  `0xa1 <= c <= 0xdf` gated on `mbcodepage == 932`. Already implemented in our Rust
  (`mbcs.rs:1822`), and it's the template these 18 do NOT follow.

**What it takes to implement all 18 at once:** one generic Rust helper
`x_ismbbtype_l(locale, tst, cmask, kmask) -> i32` (a direct, mechanical, ~8-line port of the C
above, reusing `get_mbcinfo()`/`(*locale).mbcinfo` exactly as `_ismbblead_l` does at
`mbcs.rs:1990`, and `MSVCRT__ctype` exactly as the existing `isalpha`/`isalnum` family already
does), plus 18 one-line `#[no_mangle]` wrappers supplying the `(cmask, kmask)` constants below,
plus 9 more one-line `_l`→no-`_l` forwarders (the `_l` and non-`_l` are separate exports, so 18
wrapper bodies total, not 9). This is the highest-value, lowest-risk item in the whole family —
mechanical, zero LLM judgment calls, and the constants are read straight off the C.

Missing name → `(cmask, kmask)`, from `ismbbyte.cpp:78`–`182`:

| export (_l twin implied) | cmask | kmask |
|---|---|---|
| `_ismbbkalnum` | 0 | `_MS` |
| `_ismbbkprint` | 0 | `_MS \| _MP` |
| `_ismbbkpunct` | 0 | `_MP` |
| `_ismbbalnum` | `_ALPHA \| _DIGIT` | `_MS` |
| `_ismbbalpha` | `_ALPHA` | `_MS` |
| `_ismbbgraph` | `_PUNCT \| _ALPHA \| _DIGIT` | `_MS \| _MP` |
| `_ismbbprint` | `_BLANK \| _PUNCT \| _ALPHA \| _DIGIT` | `_MS \| _MP` |
| `_ismbbpunct` | `_PUNCT` | `_MP` |
| `_ismbbblank` | special-cased, else `_BLANK` | `_MP` |

Note `_ismbbkalnum`/`_ismbbkprint`/`_ismbbkpunct` (the "k"-for-kana-range trio) use `kmask=_MS`/
`_MS\|_MP`/`_MP` with `cmask=0` — i.e. they are PURELY `mbctype`-table lookups, no ctype
involvement at all, same shape as lead/trail. Do not confuse them with `_ismbbkana`, which is
the CP932-range-check outlier above and already has a body.

**Risk for this sub-family specifically: LOW.** `_M1`/`_M2`/`_MS`/`_MP` bit-setting into
`mbctype` is already fully ported (`_setmbcp`, `mbcs.rs:150`–`365`) and doesn't depend on
`_MSVCR_VER` (see §5). The one real risk is the narrow ctype table's HIGH byte range
(0x80–0xFF), which genuinely differs between MSVCRT and UCRT — see §3, item 1. That risk is
inherited from the ALREADY-existing `MSVCRT__ctype` table, not something the 18 wrappers add.

## 2. The other 18 missing exports (outside `ismbbyte.cpp`)

| export | what it does | difficulty | winetest coverage |
|---|---|---|---|
| `ismbcblank`, `ismbcblank_l` | wide-classification blank test for an MBCS char, delegates via `msvcrt_mbc_to_wc_l` to `_iswblank`-shaped logic; UCRT source at `sdk/lib/ucrt/mbstring/ismbpunc.cpp:57`–`91`, same shape as the existing `_ismbcspace_l`/`_ismbcpunct_l` etc at `mbcs.rs:1918`,`1938` | trivial — copy the `_ismbcpunct_l` pattern, swap `_iswpunct_l` for `_iswblank_l` | none found in `modules/rostests/winetests/{msvcrt,ucrtbase}/string.c` or `misc.c` for this symbol |
| `mbsncoll`, `mbsncoll_l` | locale-collated compare of first N MBCS chars | easy — `_mbsncmp` (`mbcs.rs:1290`) + `_mbscoll_l`/`_mbsnbcoll_l` (`mbcs.rs:1144`,`1176`) already give both halves of the pattern (N-char-aware iteration + `CompareStringW`/`LCMapStringW`-based collation) | none found for `_mbsncoll` specifically in the winetest files present |
| `mbsnicoll`, `mbsnicoll_l` | case-insensitive locale-collated compare of first N MBCS chars | easy — same as above plus `_mbsicoll_l` (`mbcs.rs:1126`) for the case-fold half | none found |
| `mbsninc_l` | `_l` twin of `_mbsninc` (advance N multibyte chars); `_mbsninc` itself already ported at `mbcs.rs:668` | trivial — thread a `_locale_t` through the existing `_mbsninc` body (it already calls `_ismbblead_l`-shaped lead-byte checks internally per line 710) | none found specifically for `_mbsninc_l` |
| `mbsnbset_s`, `mbsnbset_s_l` | bounds-checked `_mbsnbset` (fill first N BYTES with char `c`); non-`_s` `_mbsnbset` exists at `mbcs.rs:2283` | easy — wrap with the `MSVCRT_CHECK_PMT!` + dest-size pattern already used by `_mbsnbcpy_s_l` (`mbcs.rs:961`) and `_mbscat_s_l` (`mbcs.rs:1519`) | none found |
| `mbsnset_s`, `mbsnset_s_l` | bounds-checked `_mbsnset` (char-count-aware fill); non-`_s` exists at `mbcs.rs:2346` | easy, same pattern as above | none found |
| `mbsset_s`, `mbsset_s_l` | bounds-checked `_mbsset` (fill whole string); non-`_s` exists at `mbcs.rs:2237` | easy, same pattern | none found |
| `mbsncat_s`, `mbsncat_s_l` | bounds-checked `_mbsncat`; non-`_s` exists at `mbcs.rs:2614`, and `_mbscat_s_l`/`_mbsnbcat_s_l` already give the `_s` wrapper template | easy | none found |
| `mbsncpy_s`, `mbsncpy_s_l` | bounds-checked, CHAR-count-aware `_mbsncpy` (vs. the already-ported BYTE-count `_mbsnbcpy_s` at `mbcs.rs:961`); non-`_s` `_mbsncpy` exists at `mbcs.rs:882` | easy — same relationship `_mbscpy_s_l` has to `_mbsnbcpy_s_l` (mbcs.rs:971, delegates with `usize::MAX`), just delegate to a char-counting variant instead of byte | none found |
| `mbsstr_l` | `_l` twin of `_mbsstr` (MBCS substring search); non-`_l` exists at `mbcs.rs:1596` | trivial — thread `_locale_t` through, mirroring how every other `_l`/non-`_l` pair in this file is a one-line forward | none found |

None of these 18 were found with dedicated winetest assertions in
`modules/rostests/winetests/msvcrt/string.c` or `modules/rostests/winetests/ucrtbase/{string,misc}.c`
(searched by symbol name). This means there is **no test oracle in-tree** for any of these 36
exports beyond whatever generic `_setmbcp`/`p_mbctype` coverage already exercises the shared
machinery (`modules/rostests/winetests/msvcrt/string.c:211`–`611`). Treat implementations as
unverified by CI until a test is added or run manually against real UCRT/MSVCRT behavior.

## 3. Delta list — where UCRT and MSVCRT genuinely differ (evidence-based)

**1. THE NARROW CTYPE TABLE'S HIGH RANGE (0x80–0xFF) IS NOT THE SAME TABLE. This is the
single biggest risk in the whole family, and it is NOT mbstring-specific — it's inherited
from `ctype.rs`, but every one of the 18 `_ismbb*` alpha/alnum/graph/print/punct/blank
functions in §1 consumes it directly via `cmask`.**

- Our Rust `MSVCRT__ctype` static (`sdk/lib/rust/msvcrt-crt-rust/src/msvcrt/ctype.rs:52`–`86`)
  is zero for every entry from 0x7f (comment `/* 0x7f */`) through 0xff — i.e. the classic
  MSVCRT "C" locale is 7-bit-ASCII-only; every extended/accented Latin-1 byte classifies as
  nothing (not alpha, not digit, not punct, not blank).
- UCRT's equivalent static table, `__newctype[384]`
  (`sdk/lib/ucrt/locale/ctype.cpp:303`), is **populated** for that same range with real
  classifications — e.g. `sdk/lib/ucrt/locale/ctype.cpp:295`: `_UPPER | C1_ALPHA, //100 (LATIN
  CAPITAL LETTER A WITH MACRON)`, and scanning upward from `0xC0`–`0xFF` shows `_UPPER|C1_ALPHA`
  / `_LOWER|C1_ALPHA` for the whole Latin-1 Supplement accented-letter block
  (`sdk/lib/ucrt/locale/ctype.cpp:220`–`296`).
- Confirmed this table is the one actually used for the default "C" locale (not just some
  alternate-codepage table): `__acrt_initial_locale_data._public._locale_pctype = __newctype +
  128` at `sdk/lib/ucrt/locale/nlsdata.cpp:94`, and that struct is explicitly "used until the
  first call to `setlocale()`" (comment at `nlsdata.cpp:87`).
- Below 0x80 the two tables are byte-for-byte identical where checked (spot-checked the full
  control range, space/blank, digit/hex, upper/lower/punct blocks — `ctype.rs:53`–`68` vs.
  `sdk/lib/ucrt/locale/ctype.cpp` control/printable section), including the shared "tab is
  `_SPACE` but deliberately not `_BLANK`" quirk noted in both the Rust comment and the C
  comment at `_ctype.cpp:169`.

  **Consequence for this family:** for a single-byte (non-DBCS) codepage such as 1252, `_MS`/
  `_MP` are never set in `mbctype` for ordinary high bytes (they're set generically only for
  CP932's kana/punct ranges — confirmed by `sdk/lib/ucrt/mbstring/mbctype.cpp:136` and mirrored
  in our Rust `_setmbcp` body at `mbcs.rs:350`–`361`, which only special-cases `newcp == 932`).
  So `x_ismbbtype_l`'s `kmask` term is 0 for e.g. byte `0xE9` under codepage 1252, and the
  entire result falls through to the `cmask` term against `_locale_pctype`. That means
  **`_ismbbalpha(0xE9)` (é) would return false if wired against `MSVCRT__ctype`, but true UCRT
  returns true** — a real, silent, high-byte-only divergence for every `alpha`/`alnum`/`graph`/
  `print`/`punct`/`blank` member of the `_ismbb*` family (9 of the 18 missing exports; the
  3 "k"-prefixed ones and lead/trail are unaffected since they use `cmask=0`).
- **Cannot determine from this research whether the ucrtbase-target build is expected to swap
  in a UCRT-shaped table or keep reusing `MSVCRT__ctype`.** That is a project-level design
  decision (a second table vs. a shared one — see §5), not something this file's exports can
  resolve on their own. No winetest in `modules/rostests/winetests/ucrtbase/{string,misc}.c`
  was found exercising a high byte through `_ismbb*` or plain `isalpha` in the C locale, so this
  is **evidenced from source, not confirmed against a running test** — flagging per the task's
  instruction rather than guessing.

**2. `_setmbcp`/mbctype-table construction itself appears IDENTICAL in shape between UCRT and
our existing Rust** — both dynamically rebuild the table at runtime from `GetCPInfo`/
`MultiByteToWideChar`/`GetStringTypeW`/`LCMapStringW` (compare `sdk/lib/ucrt/mbstring/mbctype.cpp:367`
`setmbcp_internal` with our `mbcs.rs:150`–`365`); no `_MSVCR_VER` gate was found anywhere in
`sdk/lib/ucrt/mbstring/mbctype.cpp`. This table is **not** a delta risk the way the static
ctype table is.

**3. No `_MSVCR_VER` gating found anywhere in `sdk/lib/ucrt/mbstring/`** (`grep -rn
"_MSVCR_VER" sdk/lib/ucrt/mbstring/` returns nothing). By contrast `ctype.rs`'s own header
comment explicitly documents its wide table (`MSVCRT__wctype`) as resolved for
`-D_MSVCR_VER=0`, with named constants `B110`/`D120`/`S140`/`L140`/`C140` for the version-gated
bit differences (`ctype.rs:44`–`49`). **I could not find the analogous `_MSVCR_VER` ifdef site
in UCRT source for the WIDE table** (searched `sdk/lib/ucrt/locale/ctype.cpp` and
`sdk/lib/ucrt/misc/is_wctype.cpp`; the wide-table construction there did not surface in the
greps run). This is a genuine gap in this research — the `_ismbc*` family (already fully
`have`) delegates to `_iswalpha_l` etc., which consume `MSVCRT__wctype`, so if that table also
carries a `_MSVCR_VER=0`-pinned bit difference from UCRT's wide table, it's a second delta of
the same shape as item 1, just not confirmed here. **Flagging as unresolved, not concluding
either way.**

**4. `_ismbbblank`/`_ismbcblank`'s tab special-case is present in BOTH UCRT (`ismbbyte.cpp:172`
`(tst == '\t') ? _BLANK : …`, and `ismbpunc.cpp` for the `c`-flavored one) and, by inspection of
`isblank()`'s C-side comment reproduced verbatim in our Rust, is architecturally consistent —
no delta expected here, but neither function has a Rust body yet to verify against (see §2).**

## 4. Exports checked and found identical (MSVCRT-era body should be safe to reuse as-is)

- `_ismbblead`, `_ismbblead_l`, `_ismbbtrail`, `_ismbbtrail_l` — pure `mbctype`-table `_M1`/`_M2`
  lookups (`mbcs.rs:1990`,`2008`), no ctype-table involvement, no version gating found.
- `_ismbbkana`, `_ismbbkana_l` — pure CP932 range check (`mbcs.rs:1822`), no table dependency at
  all.
- `_setmbcp`, `_getmbcp`, `__p__mbctype`, `mbcasemap`/`__p__mbcasemap` — table-construction
  machinery, confirmed structurally matching UCRT's `setmbcp_internal` (§3.2), no
  `_MSVCR_VER` gate found in either source.
- `_ismbcdigit(_l)`, `_ismbcgraph(_l)`, `_ismbcalpha(_l)`, `_ismbclower(_l)`, `_ismbcupper(_l)`,
  `_ismbcalnum(_l)`, `_ismbcspace(_l)`, `_ismbcprint(_l)`, `_ismbcpunct(_l)`, `_ismbcsymbol(_l)`
  — all delegate to the wide `_isw*` family via `msvcrt_mbc_to_wc_l` (`mbcs.rs:1843`–`1944`); not
  independently re-verified against UCRT source in this pass beyond confirming the delegation
  shape matches UCRT's own equivalent pattern in `sdk/lib/ucrt/mbstring/` — the residual risk,
  if any, lives in the WIDE ctype table (§3 item 3, unresolved), not in these functions
  themselves.

## 5. Where the mbcs tables live, and whether ucrtbase/msvcrt would need separate copies

- **`mbcinfo->mbctype`** (per-thread/per-locale MBCS classification, `_M1`/`_M2`/`_MS`/`_MP`/
  `_SBUP`/`_SBLOW`): built at runtime by `_setmbcp` from Win32 codepage APIs
  (`mbcs.rs:150`–`365`). This is **not a static/shared global** — it's constructed fresh per
  codepage change and is architecturally identical in UCRT and MSVCRT (§3.2). **No separate
  ucrtbase copy needed**; this table is safe to share.
- **`MSVCRT_mbctype`** (`mbcs.rs:17`, the exported `[u8; 257]` global mirroring the current
  thread's `mbcinfo->mbctype`, kept in sync by `_setmbcp` per the loud comment at
  `mbcs.rs:8`–`15` about the two-copies-diverging bug that already bit this file once): this is
  the DLL-exported-by-name global (`msvcrt.spec:715`). **If ucrtbase.dll needs its own
  differently-named export for the same data, that's a second global requiring the same
  write-through discipline — worth flagging loudly to whoever wires ucrtbase, since this is
  exactly the class of bug the existing comment documents.**
- **`MSVCRT__ctype`** (narrow single-byte ctype table, `ctype.rs:52`): **static, compiled-in,
  and — per §3 item 1 — genuinely different from UCRT's `__newctype` in the 0x80–0xFF range.**
  This is the one table where "share it with msvcrt" is actively wrong for ucrtbase's default
  "C" locale behavior. **If ucrtbase reuses `MSVCRT__ctype` unmodified, every `_ismbb*`
  alpha/alnum/graph/print/punct/blank check (and every plain `isalpha` etc., though those are
  outside this family's scope) will silently mis-classify Latin-1 Supplement bytes under a
  single-byte codepage — say so loudly, this changes the design:** ucrtbase needs either (a) a
  second `UCRT__ctype` static generated from `sdk/lib/ucrt/locale/ctype.cpp:220`–`341` the same
  way `tools/gen_ctype_tables.py` currently generates `MSVCRT__ctype`, wired in for the ucrtbase
  build only, or (b) some `#[cfg]`/build-time selection between the two tables if `ctype.rs` is
  to stay a single shared crate module for both DLLs. This research does not recommend which —
  that's a build-architecture call outside a read-only research task's scope — but the fact
  that a naive "just link the same object" reuse is wrong here is the single most important
  takeaway of this brief.
- **`MSVCRT__wctype`/`MSVCRT__pwctype`** (wide ctype table, `ctype.rs:89`–`136`): consumed by
  the entire `_ismbc*` sub-family via `_isw*_l`. Per §3 item 3, whether this table has the same
  kind of UCRT/MSVCRT split as the narrow one is **unresolved** — I could not locate UCRT's wide
  ctype table source to diff it in the time available. Flagging as the most important follow-up
  for whoever picks up the `_ismbc*` reuse question, since if it needs a second copy too, the
  same "generated table, `#[cfg]`-selected or duplicated" design call in the point above applies
  there as well.
