<!-- ucrtbase de-Microsoft-ification quest, 2026-09-20. Produced by a read-only
     survey agent and reviewed before landing. FINDINGS, not a plan of record.
     Two of this round's reports CORRECTED an earlier round's claims, so treat
     every statement here as re-derivable rather than settled: claims are cited
     to file:line, and each report names what it could NOT determine.
     Burn-down: `python3 tools/ucrt_burndown.py --arch i386`.
     Declarations for implemented families: tools/ucrt-takeover.yaml. -->

# UCRT `mbctype` table capture — generated-then-verified, before `mbctype.cpp` is deleted

Deliverables: `mbctype-table.rs` (data-only, `no_std`) and this report. Both live in
`~/vibeos-rust-staging/ucrt/`, outside the repo, per the task constraint. No repo file was
edited and no build/gate/smoke/land was run.

## 1. What `mbctype.cpp` actually contains

Read in full: `sdk/lib/ucrt/mbstring/mbctype.cpp` (1106 lines).

### Distinct tables

| Table | Size | Defined at | What it holds |
|---|---|---|---|
| `__acrt_initial_multibyte_data.mbctype` | `unsigned char[257]` | mbctype.cpp:48-59 | The boot-time mbctype bit table — ASCII-only baseline (SBUP/SBLOW for A-Z/a-z), everything else 0. Index = char value + 1 (index 0 = -1/EOF; `NUM_CHARS = 257` at mbctype.cpp:29). |
| `_MBCTYPE_DEFAULT` | macro, same 257 bytes | mbctype.cpp:76-88 | Byte-for-byte identical initializer, reused for `_mbctypes[state_index_count][257]` (mbctype.cpp:91-97). Same content as the row above — one fact, two spellings. |
| `__acrt_initial_multibyte_data.mbcasemap` | `unsigned char[256]` | mbctype.cpp:60-69 | ASCII upper<->lower case-fold map: `mbcasemap['A'..'Z'] = 'a'..'z'`, `mbcasemap['a'..'z'] = 'A'..'Z'`, else 0. |
| `_MBCASEMAP_DEFAULT` / `_mbcasemaps[...]` | same 256 bytes | mbctype.cpp:100-118 | Same content as the row above. |
| `__rgcode_page_info[5]` | `code_page_info[5]` (struct at mbctype.cpp:35-41) | mbctype.cpp:138-192 | The **DBCS ground-truth table**: per code page, `mbulinfo[6]` (full-width Latin case-fold ranges) and `rgrange[4][8]` (Single Byte / Punctuation / Lead Byte / Trail Byte ranges, each up to 4 (lo,hi) pairs). Covers `_KANJI_CP`=932, `_CHINESE_SIMP_CP`=936, `_KOREAN_WANGSUNG_CP`=949, `_CHINESE_TRAD_CP`=950, `_KOREAN_JOHAB_CP`=1361 (constants at mbctype.cpp:23-26,50). |
| `__rgctypeflag[4]` | `char[4]` | mbctype.cpp:137 | `{ _MS, _MP, _M1, _M2 }` — maps `rgrange` row index -> bit value. |
| `_mb_locale_names[4]` | `const wchar_t*[4]` | mbctype.cpp:104-109 | `ja-JP`/`zh-CN`/`ko-KR`/`zh-TW`, looked up by `CPtoLocaleName()` (mbctype.cpp:396-409). |
| Trailing comment block | — | mbctype.cpp:812-987 | Non-code documentation: the actual half/full-width digit and Latin-letter code-point tables for CP932/936/949/950, plus "CP1361: Not yet available (05/17/94)" (mbctype.cpp:983). This is a leftover of Microsoft's original authoring notes, not live data. |

### Bit meanings (cited to the header that defines them)

`sdk/include/crt/mbctype.h:40-46` (identical values also declared in `sdk/lib/ucrt/inc/corecrt_internal_mbstring.h:31-32,38-43` and used by `NUM_ULINFO`=6 at line 21):

```
_MS    0x01   single-byte char that lives inside an otherwise-DBCS codepage (half-width kana)
_MP    0x02   single-byte punctuation inside an otherwise-DBCS codepage (half-width kuten etc.)
_M1    0x04   valid DBCS lead byte
_M2    0x08   valid DBCS trail byte
_SBUP  0x10   single-byte uppercase letter
_SBLOW 0x20   single-byte lowercase letter
```

### Bespoke table vs. derived/default, per codepage

- **932, 936, 949, 950, 1361**: bespoke — hit `__rgcode_page_info` directly in
  `_setmbcp_nolock` (mbctype.cpp:591-629), which zeroes the table then, for each of the 4
  range-rows, walks every `(lo,hi)` pair and OR's `__rgctypeflag[row]` into
  `mbctype[byte+1]` for every byte in range (mbctype.cpp:599-614). Only 932 gets `_MS`/`_MP`
  (its Single-Byte and Punctuation rows are non-empty; the other 4 codepages have those two
  rows all-zero, i.e. no half-width block).
- **CP_UTF8**: special-cased separately (mbctype.cpp:640-670) — table zeroed, `ismbcodepage=0`,
  no lead/trail bits at all (UTF-8 isn't DBCS in the CRT's model).
- **Any other codepage the OS knows** (`GetCPInfo` succeeds): derived, not bespoke
  (mbctype.cpp:672-721). Lead-byte ranges come from `CPINFO.LeadByte` (queried from the OS);
  trail bytes are NOT looked up — **every** byte 0x01-0xFE is marked `_M2` if `MaxCharSize==2`
  (mbctype.cpp:696-699), i.e. no bespoke trail-byte precision outside the 5 listed codepages.
- **SBCS request / codepage unknown to the OS**: `setSBCS()` (mbctype.cpp:262-282) — full reset
  to the `_MBCTYPE_DEFAULT`/`_MBCASEMAP_DEFAULT` ASCII baseline.
- **`_SBUP`/`_SBLOW`** for 0-255, on top of whichever of the above ran: always computed
  separately by `setSBUpLow()` (mbctype.cpp:479-566) via `GetCPInfo`+`GetStringTypeA`+
  `LCMapStringA` OS calls (or plain ASCII casing if no codepage / UTF-8) — this is why even
  the 5 bespoke codepages don't hardcode `_SBUP`/`_SBLOW` in `__rgcode_page_info`.

### How `_setmbcp` selects and installs a table

`sdk/lib/ucrt/mbstring/mbctype.cpp`: public entry `_setmbcp` (line 568) -> `setmbcp_internal`
(line 275) -> `getSystemCP` (resolves `_MB_CP_OEM`/`_MB_CP_ANSI`/`_MB_CP_LOCALE` sentinels to a
real codepage number, line 435) -> allocates a new `__crt_multibyte_data`, copies the
caller's current data into it, then `_setmbcp_nolock` (line 573) fills `mbctype`/`mbcasemap`
per the rules above, and finally the new struct is swapped into the per-thread pointer and,
if this thread syncs with the global locale, into the process-global `_mbctype`/`_mbcasemap`
(mbctype.cpp:338-351). `__acrt_initialize_multibyte` (line 775) is what calls this once at
CRT startup with `_MB_CP_ANSI`.

### Dependency on the narrow ctype table

**None.** Grepped mbctype.cpp for `_ctype`/`__ctype`/`ctype[` — every hit is `mbctype`/
`mbcinfo->mbctype`, not the single-byte `_ctype` table the sibling agent's `locale/ctype.cpp`
covers. Casing information comes from Win32 (`GetStringTypeA`/`GetStringTypeW`,
`LCMapStringA`/`LCMapStringW`), never from `_ctype`. No overlap to flag.

## 2. Generation method (independent of Microsoft's table)

I built lead-byte/trail-byte ranges for the four codepages with modern, in-use Rust support
(932, 936, 949, 950) from independently-sourced, publicly-documented codepage facts — fetched
live via WebFetch, not recalled from training data alone, and cited below — then expanded them
into full 257-entry bit tables with a throwaway Python script
(`apply_ranges`/`base_sbcs` in the session scratchpad; logic mirrored, not copied, into the
`.rs` header comment). Only afterwards did I hand-transcribe Microsoft's
`__rgcode_page_info` (mbctype.cpp:139-192) into a second, UCRT-reference table used **purely
as a diff oracle** — that transcription lives only in an ephemeral scratch script
(`/tmp/gen_ucrt_ref.py`, not committed, not shipped), never in `mbctype-table.rs`.

Sources fetched:
- CP932 lead/trail structure: `en.wikipedia.org/wiki/Shift_JIS` (standard JIS X 0208 lead
  0x81-0x9F/0xE0-0xEF; Microsoft's NEC-row-13 + IBM-extension rows are the documented reason
  the practical upper lead bound is 0xFC, not 0xEF).
- CP936 (GBK): `en.wikipedia.org/wiki/GBK_(character_encoding)` — "first byte 0x81-0xFE
  ... second byte 0x40-0xA0 except 0x7F for some areas and 0xA1-0xFE for others."
- CP949 (UHC): `en.wikipedia.org/wiki/Unified_Hangul_Code` — "lead byte range extended to
  0x81-FE, and the trail byte range extended to 0x41-5A, 0x61-7A and 0x81-FE."
- CP950 (Big5/ETen): `en.wikipedia.org/wiki/Big5` — "first byte 0x81 to 0xfe (or 0xa1 to
  0xf9 for non-user-defined characters) ... second byte 0x40 to 0x7e, 0xa1 to 0xfe."
- CP1361 (Johab): `en.wikipedia.org/wiki/Johab` (no concrete byte ranges given) and a fetch of
  `unicode.org/Public/MAPPINGS/OBSOLETE/EASTASIA/KSC/JOHAB.TXT`, which only returned a partial
  read (lead bytes 0x20, 0x5C, 0x84-0x8F, 0x90-0x9F visible; nothing confirmed above 0x9F).
  **Not enough to generate a full table — see §4.**

The JIS X 0201 half-width-kana/punctuation split for CP932 (0xA1-0xA5 punctuation, 0xA6-0xDF
kana) is standard JIS X 0201 knowledge, independent of Microsoft's CRT — the boundary at
0xA5/0xA6 is where JIS X 0201 draws it (kuten/punctuation marks vs. the katakana syllabary).

## 3. The diff — generated vs. Microsoft's table, byte by byte

Comparison method: both tables (generated, and a literal transcription of
`__rgcode_page_info` expanded the same way `_setmbcp_nolock`'s loop does — OR the row's bit
into every byte in every `(lo,hi)` pair) were built as 257-entry Python lists and compared
index-by-index. Full script output:

```
CP932: 0 byte(s) differ
CP936: 1 byte(s) differ
  byte 0x7F: generated=0x00 ucrt=0x08
CP949: 0 byte(s) differ
CP950: 0 byte(s) differ
```

- **CP932: agrees on all 257 bytes.** Lead 0x81-0x9F/0xE0-0xFC, trail 0x40-0x7E/0x80-0xFC,
  punctuation 0xA1-0xA5, kana 0xA6-0xDF — independently generated and Microsoft's table match
  exactly.
- **CP950: agrees on all 257 bytes.** Lead 0x81-0xFE, trail 0x40-0x7E/0xA1-0xFE — exact match.
- **CP949: agrees on all 257 bytes**, but only because I generated CP949 using Microsoft's own
  *actual* (contiguous 0x41-0xFE) trail-byte range rather than the *original 1990s UHC
  proposal's* gapped range (0x41-0x5A, 0x61-0x7A, 0x81-0xFE per the Wikipedia UHC article).
  I could not independently confirm, from a source distinct from Microsoft's own ecosystem,
  that Microsoft's CP949 is contiguous rather than gapped — flagged with reduced confidence in
  §4, even though the number itself says "0 diff."
- **CP936: disagrees at exactly one byte, 0x7F.** Independently-sourced GBK explicitly
  excludes 0x7F from the trail-byte set; Microsoft's static table (`{0x40,0xFE}`, a single
  unbroken range at mbctype.cpp:171) folds 0x7F in anyway. **Judgement: Microsoft's table is
  the one that's wrong here** (over-inclusive) — 0x7F is the ASCII DEL control character and
  every independent description of GBK's structure (which mirrors GB2312/GB18030's general
  "avoid the C0 control range and 0x7F" convention) excludes it as a second byte. This means
  UCRT (and by inheritance any translation that copies UCRT's table) will misclassify a
  DEL-in-second-byte-position sequence as "second byte of a valid DBCS pair" where GBK says it
  isn't. Low practical impact (0x7F rarely appears mid-string) but it is a real, single-byte
  divergence, not noise.

## 4. Delta against our existing Rust (`sdk/lib/rust/msvcrt-crt-rust/src/msvcrt/mbcs.rs`)

**Important scope correction first:** `mbcs.rs`'s header comment says it is a translation of
`dll/win32/msvcrt/mbcs.c` — **ReactOS's own `msvcrt.dll`, not UCRT.** These are two separate
CRT implementations in this tree with two separate `_setmbcp`s. The task's "earlier survey"
claim — *"the `_setmbcp`/mbctype construction machinery is architecturally identical between
UCRT and our Rust, no `_MSVCR_VER` gating found"* — **does not hold up** on inspection. It
holds for *table content* on the four verifiable codepages (see below), but the *machinery* is
materially different:

| | UCRT (`mbctype.cpp`) | Our Rust `msvcrt.dll` (`mbcs.rs`) |
|---|---|---|
| Lead-byte source | Static `__rgcode_page_info` table for 5 known codepages; `GetCPInfo` only as fallback for the rest | **Always** `GetCPInfo` at runtime (`create_mbcinfo`, mbcs.rs:220,246-254) — no static lead-byte table at all |
| Trail-byte source | Static table for the 5 known codepages; **every** byte 0x01-0xFE for any other DBCS codepage (mbctype.cpp:696-699) | Static `g_cpextrainfo[7]` table (mbcs.rs:25-33) keyed by codepage, **plus** a catch-all `cp:0` entry `{1,255,...}` that is functionally the same "everything is a trail byte" fallback |
| `_MS`/`_MP` (932 half-width kana/punct) | From `__rgcode_page_info[0]`, applied generically | **Hardcoded special case** after the generic pass: `if newcp == 932 { 161..=165 -> _MP; 166..=223 -> _MS }` (mbcs.rs:350-361) — same numeric ranges (0xA1-0xA5, 0xA6-0xDF), different code shape |
| `_SBUP`/`_SBLOW` | `setSBUpLow()`: `GetCPInfo`+`GetStringTypeA`+`LCMapStringA` | `create_mbcinfo`: `GetStringTypeW`+`LCMapStringW` — same idea, wide instead of narrow API |
| Full-width Latin case-fold (`mbulinfo`/`_MBUPPERLOW*_MT` etc., `NUM_ULINFO`=6) | Static per-codepage ranges in `__rgcode_page_info[i].mbulinfo` | **Not replicated as a static table at all.** `_mbctoupper_l`/`_mbctolower_l` (mbcs.rs:416-519) call `__crtLCMapStringA`/`LCMAP_UPPERCASE`/`LCMAP_LOWERCASE` directly on the 2-byte sequence whenever `c > 0xff`, i.e. they ask the OS instead of consulting a baked-in range. Behaviourally equivalent (same OS codepage tables back both), architecturally not the same mechanism, and there's no `mbulinfo`-shaped data to port. |

**Table CONTENTS**: for the 4 codepages I could verify (932/936/949/950), our Rust's own
static tables agree with the generated/Microsoft-verified ranges:
- `g_cpextrainfo` trail-byte entries (mbcs.rs:25-33) for cp 932/936/949/950/1361 are byte-for-byte
  identical to UCRT's `__rgcode_page_info` trail-byte rows (verified by inspection — both list
  `{0x40,0x7e,0x80,0xfc}`/`{0x40,0xfe}`/`{0x41,0xfe}`/`{0x40,0x7e,0xa1,0xfe}`/`{0x31,0x7e,0x81,0xfe}`
  respectively).  This means **our Rust inherits UCRT's same CP936-byte-0x7F disagreement** with
  independently-sourced GBK — it isn't a new bug introduced by translation, it's the same
  upstream imprecision, present in both CRTs in this tree.
- The 932 `_MS`/`_MP` hardcoded ranges (161-165, 166-223 decimal = 0xA1-0xA5, 0xA6-0xDF) match
  both UCRT's table and the independently-generated one exactly.
- Lead-byte ranges aren't directly comparable as static data (ours queries `GetCPInfo` at
  runtime instead of hardcoding), but since `GetCPInfo` for a fixed OS codepage returns the
  same `LeadByte` array UCRT's static table was presumably derived from, the two are expected
  to agree in practice — this is an architecture difference, not a content difference.

**ucrtbase vs. msvcrt would need the SAME data, not different data**, for these 4 codepages:
every byte-value comparison above came back either exact-match or an already-shared upstream
discrepancy. Nothing suggests msvcrt and ucrtbase should ship divergent tables for 932/936/
949/950. **Indexing** is identical too: both use the `index = char_value + 1`, 257-entry
layout (`NUM_CHARS`/`_mbctype[257]`), so no re-layout is needed to reuse `mbctype-table.rs`
data structures in either crate.

## 5. Could not determine — stated plainly

- **CP1361 (Johab) is not in `mbctype-table.rs` at all.** The only independent source I could
  reach (a partial fetch of `unicode.org/Public/MAPPINGS/OBSOLETE/EASTASIA/KSC/JOHAB.TXT`)
  confirmed lead bytes exist starting at **0x84**, contradicting UCRT's static lower bound of
  **0x81** (`__rgcode_page_info[4].rgrange[2] = {0x81,0xD3,0xD8,0xDE,0xE0,0xF9}`,
  mbctype.cpp:184-191) — but the fetch was incomplete (only confirmed lead bytes up to 0x9F; the
  0xD3/0xD8-0xDE/0xE0-0xF9 upper blocks and every trail-byte range are unverified). I judged a
  3-range, partially-guessed table worse than no table: a wrong entry here misclassifies a byte
  and never faults, which is exactly the failure mode the task asked me to avoid. **Whoever
  wires this in should either get authoritative Johab range data (the real KS X 1001 Johab
  annex, or a complete fetch of the Unicode.org mapping file) before shipping a CP1361 table, or
  keep using UCRT's numbers with the 0x81-vs-0x84 discrepancy flagged as a known-uncertain
  point**, not silently trust either source.
- **CP949's trail-byte contiguity is "0 diff" but lower-confidence than the others.** I only
  have Wikipedia's description of the *original* 1990s Unified Hangul Code proposal (gapped)
  and my own recollection that Microsoft's actual CP949 fills the gaps; I could not reach a
  third-party source (ICU/glibc/Python codec docs) within this task to independently confirm
  Microsoft's specific implementation is contiguous rather than gapped. If it turns out to be
  gapped after all, CP949 would gain the same class of disagreement CP936 has, just at
  different byte positions (0x5B-0x60, 0x7B-0x80).
- **CP932's upper lead-byte bound (0xFC vs. JIS X 0208's 0xEF)** relies on the NEC-row-13/
  IBM-extension explanation from the Shift_JIS Wikipedia article, which does not itself state
  0xFC as a lead-byte bound in hex — I derived it from "rows 89-92 and 115-119 are appended"
  plus knowledge of the row-to-byte arithmetic Shift-JIS uses, not from an explicit hex range in
  the fetched text. The generated/UCRT match (0 diff) is reassuring corroboration, but flagging
  the derivation step as inference rather than a directly-quoted fact.
- **The `mbulinfo` full-width Latin case-fold ranges** (932/936/949/950/1361 upper/lower pairs
  in `__rgcode_page_info[i].mbulinfo`, e.g. CP932 `0x8260-0x8279` per mbctype.cpp:139-148) were
  **not independently regenerated** — they aren't consumed by our Rust `mbcs.rs` at all (see
  §4), so there was nothing to verify them against on our side, and generating+diffing a 5x6
  table of full-width case pairs was out of scope for what the wiring actually needs. Noted as
  a gap, not fabricated.
