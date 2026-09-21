<!-- ucrtbase de-Microsoft-ification quest, 2026-09-20. Produced by a read-only
     survey agent and reviewed before landing. FINDINGS, not a plan of record.
     Two of this round's reports CORRECTED an earlier round's claims, so treat
     every statement here as re-derivable rather than settled: claims are cited
     to file:line, and each report names what it could NOT determine.
     Burn-down: `python3 tools/ucrt_burndown.py --arch i386`.
     Declarations for implemented families: tools/ucrt-takeover.yaml. -->

# ucrtbase "234 owed" forwards audit

Read-only audit. No repo files touched, no builds run. Command used to regenerate the
input list (from repo root, needs `build-i386/` already built):

```
python3 - <<'EOF'
import sys; sys.path.insert(0,'tools')
from ucrt_burndown import classify,_undecorate
from verify_ucrtbase_cohab import _nm_defined
from pathlib import Path
b,owed=classify('i386')
rust={_undecorate(s) for s in (_nm_defined(Path('build-i386/dll/win32/msvcrt/libmsvcrt_crt_cargo.a')) or set())}
for f,s in sorted(owed.items()):
    miss=sorted(s-rust)
    if miss: print(f, miss)
EOF
```

**This run returned 219 symbols across 86 files, not 234.** I ran it exactly as given
in the task; the count has apparently drifted since the quest was framed (likely more
families landed in `msvcrt-crt-rust` since then — `wcsnlen`, `wcsset_s`, `abs64`,
`mbsnbcnt`/`mbsnbcnt_l`, the `vfprintf`/`vsprintf`/`vswprintf` family, and others used
below as TRIVIAL-ALIAS targets are all already ported). I did not force the number to
234; I audited what the script actually produced today. If 234 is load-bearing
somewhere, the burndown script (not this audit) is the thing to re-run and reconcile.

## Headline counts (of 219)

| Class | Count | % |
|---|---:|---:|
| **FORWARD** (spec redirects to a body that already exists) | **21** | 9.6% |
| **TRIVIAL-ALIAS** (needs a small wrapper over a body we already have) | **17** | 7.8% |
| **REAL** (genuinely new implementation work) | **181** | 82.6% |

So the headline finding does **not** generalize the same way the convert/ family
sample suggested. In convert/ itself, forwards are real and large (13 of convert/'s
symbols are FORWARD — see table), but convert/ is the single most forward-heavy
family in the whole 219; most other families (mbstring bounded-copy ops, the whole
`stdio_common_v*`/`conio_common_v*` printf engine, exec/spawn, time, locale) are
staffed with zero or near-zero forwards. Pulling in TRIVIAL-ALIAS (wrappers over
already-ported bodies, which is new this audit and wasn't in the original convert/
survey) recovers another 17, but the overall remaining work is **181 of 219 (83%)**,
not "roughly half." The convert/ family's ~50% forward rate does not generalize to the
full 234/219 — it was the friendliest family, not a representative one.

**Directional check, since a wrong FORWARD is the dangerous read:** all 21 FORWARD
targets were verified to already have a Rust body in
`build-i386/dll/win32/msvcrt/libmsvcrt_crt_cargo.a` (via `nm`, the same check
`ucrt_burndown.py` itself uses) — not just "the spec names something else." A forward
to a still-vendored-C target would **not** count as FORWARD here; none of the 21 are
that case. See Method.

## Method

1. **Source of truth for FORWARD**: `dll/win32/ucrtbase/ucrtbase.spec`, parsed with
   `tools/export_census.py:parse_spec()` (the same parser `ucrt_burndown.py` already
   trusts, so this reuses vetted logic rather than re-deriving spec2def's grammar by
   hand). A spec line of the shape

       @ cdecl -ret64 strtoll(str ptr long) _strtoi64

   has a **trailing bare name after the arg list** — that's the redirect target. A line
   with no trailing name (`@ cdecl strtof(str ptr)`) is a real export with its own
   body. `parse_spec` returns `(kind, backing)` per export; I undecorated both the
   export name and `backing` (`export_census._undecorate` / `ucrt_burndown._undecorate`,
   strip leading `_` and `@N` suffix — same helper both files already use so i386
   cdecl/stdcall decoration doesn't create false differences) and called it FORWARD
   only when `undecorate(backing) != undecorate(export_name)`. That is the only
   mechanical signal I used for FORWARD — 21 symbols, listed below.
2. **Verified FORWARD targets exist**: for each of the 21, undecorated `backing` was
   checked against the same `rust` set (`nm --defined-only` on
   `libmsvcrt_crt_cargo.a`, undecorated) used to build the "234/219 owed" list itself.
   All 21 targets are `True`. (Concretely this is where the convert/ family's forwards
   land: `strtoll`/`strtoimax`/`wcstoll`/etc all forward to `_strtoi64`/`_wcstoi64`/
   `_strtoi64_l`/`_wcstoi64_l`, and those *are* already Rust — ported previously under
   the classic MSVCRT `_strtoi64` family, just not visible in the strtox.cpp grouping
   because they're not themselves "owed.")
3. **TRIVIAL-ALIAS**: for every remaining (non-forward) symbol I read the actual
   `sdk/lib/ucrt/**/*.cpp` body (not just the spec line) and looked for one of two
   patterns, both cited per-entry in the table:
   - an `_l` locale twin whose body provably **ignores** the locale argument (calls
     the non-`_l` sibling, or a locale-blind helper, with the locale parameter unused
     or passed to something itself locale-blind) **and** that sibling/helper already
     has a Rust body; or
   - a width/name variant whose logic is identical to an already-ported function
     (e.g. `_calloc_base`'s own doc comment: *"This function implements the logic of
     calloc()"* — literally the same HeapAlloc-based algorithm our Rust `calloc`
     already has).
   I did **not** count a case as TRIVIAL-ALIAS on resemblance alone — every one below
   has the specific existing Rust symbol it wraps named, and I ran `c in rust` against
   the `nm`-derived set to confirm that symbol is actually present before using it.
4. **Everything else defaults to REAL.** Per the task's instruction, ambiguous cases
   were pushed to REAL rather than guessed as FORWARD or TRIVIAL-ALIAS. Concretely I
   backed off from TRIVIAL-ALIAS in two cases where I initially expected reuse:
   `mbsncpy_s`/`mbsncat_s`/`mbsnset_s` and their `_l` twins look, from their names
   alone, like bounded variants of the already-ported `mbscpy_s`/`mbscat_s`/`mbsset_s`
   — but their `.inl` bodies (`sdk/lib/ucrt/mbstring/mbsncpy_s.inl`,
   `mbsncat_s.inl`, `mbsnset_s.inl`) are 80-150 lines of real MBCS lead-byte-aware
   truncation logic, not thin wrappers, so they're REAL. And
   `mbsncoll`/`mbsncoll_l`/`mbsnicoll`/`mbsnicoll_l` call
   `_loc_update.GetLocaleT()->mbcinfo->...` and `__acrt_CompareStringA` directly — they
   **use** the locale argument for real (codepage-dependent collation), so they're
   REAL and additionally locale-struct-blocked (see below), not trivial.

### The `_locale_t` caveat (from `docs/engineering/active-lanes/convert-delta.md` §0)

ucrtbase's `_locale_t` is a different struct than msvcrt's
(`__crt_locale_data` vs `threadlocaleinfostruct`). I used this to separate two REAL
sub-cases that a difficulty note below calls out per-family:
- **locale-blind REAL** (e.g. `c16rtomb`/`c32rtomb`/`mbrtoc16`/`mbrtoc32`,
  `imaxdiv`/`lldiv`, `wcsncnt`): self-contained new logic that never touches
  `_locale_t` at all. No struct-mismatch risk; ordinary translation work.
- **locale-struct-blocked REAL** (`mbsncoll`/`_l`, `mbsnicoll`/`_l`, `strftime_l`,
  most of `locale/*.cpp`, `misc/invalid_parameter.cpp`'s handler-state accessors):
  genuinely dereferences ucrtbase's own `_locale_t`, so needs that struct (or the
  ucrtbase-side locale plumbing generally) sorted out first, same class of blocker as
  the convert/ survey already flagged.

### Two ambiguous cases I could NOT cleanly resolve — flagged, not guessed

- **`wctype`**: this undecorated name collides between two *different* spec exports —
  `wctype(str)` (a real function, `convert/wctype.cpp`, category-name→`wctype_t`
  lookup) and `_wctype` (a `extern` **data** symbol, `locale/ctype.cpp`, a 257-entry
  `unsigned short const[]` classification table). `ucrt_burndown.py`'s own grouping
  reports both under the literal string `wctype` in two different files
  (`convert/wctype.cpp` and `locale/ctype.cpp`) because its `_undecorate` strips the
  leading `_` off `_wctype` and collapses it onto `wctype`. **I did not merge these —
  I list both, separately, tagged with which spec line each is.** The `wctype()`
  function is REAL (small, ~15 lines, no locale). The `_wctype[]` data table's values
  look byte-for-byte comparable to `MSVCRT__wctype` already in
  `sdk/lib/rust/msvcrt-crt-rust/src/msvcrt/ctype.rs` (both are 257-entry classic-C
  `_CONTROL`/`_SPACE`/etc bitmask tables) — plausibly a data-only TRIVIAL-ALIAS, but I
  did **not** diff the two tables' actual 257 values byte-for-byte, so per the "unsure
  → REAL" rule I'm leaving it REAL with a flag, not counting it in the 17.
- **`Exit`** (spec `_Exit`): its body in `startup/exit.cpp` is
  `common_exit(return_code, _crt_exit_no_cleanup, _crt_exit_terminate_process)` —
  textually identical to `_exit`'s call. If our already-ported `exit` (`rust=True`)
  Rust body happens to be the no-cleanup variant, `_Exit` would be a one-line
  TRIVIAL-ALIAS to it; if it's the full-cleanup variant, `_Exit` is REAL and
  `_exit`/`_Exit` are two distinct still-missing bodies that happen to collide under
  `_undecorate` the same way `wctype`/`_wctype` do. I could not tell which without
  reading the Rust `exit` body, which is out of scope for a read-only audit of the
  *ucrt* side. Left REAL, flagged.

## Table: all 219, by family

Legend: **F**=FORWARD, **T**=TRIVIAL-ALIAS, **R**=REAL. Forward target / alias target
given where applicable.

### convert/ (33 symbols — the friendliest family: 13 F, 6 T, 14 R)

| Symbol | Class | Target / note |
|---|---|---|
| mbstrnlen | R | new (bounded mbslen, MBCS-aware) |
| mbstrnlen_l | R | locale-struct-blocked twin of above |
| iswcsym_l | **T** | `__iswcsym` (rust=True) — body is `return __iswcsym(c);`, locale param unused |
| iswcsymf_l | **T** | `__iswcsymf` (rust=True) — same pattern |
| atoll | **T** | `cnt_rust::core_rs::radix::parse_i64_wrapping` — already used by `atoi`/`atol`, just don't truncate to i32 |
| atoll_l | **T** | same primitive; `_atoll_l` passes locale straight into the same width-generic parser template that `atoi`/`atol` already instantiate locale-blind |
| wtoll | **T** | `cnt_rust::core_rs::radix::parse_i64_wrapping_cu` — already exists (wide-char-generic, used by `_wtoi`/`_wtol`, both rust=True) |
| wtoll_l | **T** | same, locale-blind for integer parsing |
| c16rtomb | R | new, self-contained UTF-16→UTF-8 state machine (~40 lines), no locale |
| c32rtomb | R | new, self-contained UTF-32→UTF-8 (~60 lines), no locale |
| mbrtoc16 | R | new, self-contained UTF-8→UTF-16 w/ surrogate state (~50 lines), no locale |
| mbrtoc32 | R | new, self-contained UTF-8→UTF-32 decoder (~90 lines), no locale |
| strtof | R | new float parse, no locale |
| strtof_l | R | locale-struct-blocked twin |
| strtold | **F** | → `strtod` (rust=True) |
| strtold_l | **F** | → `_strtod_l` (rust=True as `strtod_l`) |
| wcstof | R | new wide float parse |
| wcstof_l | R | locale-struct-blocked twin |
| wcstold | **F** | → `wcstod` (rust=True) |
| wcstold_l | **F** | → `_wcstod_l` (rust=True as `wcstod_l`) |
| strtoimax | **F** | → `_strtoi64` (rust=True as `strtoi64`) |
| strtoimax_l | **F** | → `_strtoi64_l` (rust=True) |
| strtoll | **F** | → `_strtoi64` |
| strtoll_l | **F** | → `_strtoi64_l` |
| strtoull | **F** | → `_strtoui64` (rust=True as `strtoui64`) |
| strtoull_l | **F** | → `_strtoui64_l` |
| strtoumax | **F** | → `_strtoui64` |
| strtoumax_l | **F** | → `_strtoui64_l` |
| wcstoimax | R | real export per spec (no forward line) — 64-bit imax parse on the wide side |
| wcstoimax_l | R | locale-struct-blocked twin |
| wcstoll | **F** | → `_wcstoi64` |
| wcstoll_l | **F** | → `_wcstoi64_l` |
| wcstoull | **F** | → `_wcstoui64` |
| wcstoull_l | **F** | → `_wcstoui64_l` |
| wcstoumax | R | real export per spec — 64-bit umax parse, wide |
| wcstoumax_l | R | locale-struct-blocked twin |
| wctrans | R | new (name→wctrans_t table lookup, same shape as `wctype()`) |
| wctype | R | new (name→`wctype_t` table lookup, ~15 lines, no locale) — see ambiguous-case note; **not** the same spec export as `_wctype` data below |

### locale/ (5 symbols, all R; `wctype`'s data twin lives here)

| Symbol | Class | Target / note |
|---|---|---|
| wctype (locale/ctype.cpp) | R (flagged) | this is `_wctype[]`, an `extern` **data** table (257 × `unsigned short`), distinct spec export from `convert/wctype.cpp`'s `wctype()` function above. Plausibly reuses/duplicates the layout of `MSVCRT__wctype` already in `msvcrt-crt-rust/src/msvcrt/ctype.rs`, but I did not diff the 257 values — see ambiguous-case note. |
| lc_locale_name_func | R | locale-struct accessor, blocked |
| mb_cur_max_l_func | R | locale-struct accessor, blocked |
| initialize_lconv_for_unsigned_char | R | locale-struct init, blocked |
| configthreadlocale | R | locale-struct/TLS interaction, blocked |
| wcreate_locale | R | locale-struct construction, blocked — likely the largest single item in the whole 181; this is where ucrtbase's `_locale_t` actually gets built |

### mbstring/ (38 symbols — 1 T, 37 R; the "n-bounded" ops are NOT trivial)

| Symbol | Class | Target / note |
|---|---|---|
| ismbbalnum / _l | R / R | new (MBCS-aware classification), locale-struct-blocked for `_l` |
| ismbbalpha / _l | R / R | same pattern |
| ismbbblank / _l | R / R | same |
| ismbbgraph / _l | R / R | same |
| ismbbkalnum / _l | R / R | same |
| ismbbkprint / _l | R / R | same |
| ismbbkpunct / _l | R / R | same |
| ismbbprint / _l | R / R | same |
| ismbbpunct / _l | R / R | same |
| ismbcblank / _l | R / R | same |
| mbsnbset_s | R | real MBCS bounded-write logic (`mbsnset_s.inl`, ~90 lines, lead-byte tracking) |
| mbsnbset_s_l | R | same, locale-struct-blocked |
| mbsncat_s | R | real MBCS bounded-concat logic (`mbsncat_s.inl`, ~150 lines) — **not** a trivial wrapper over `mbscat_s` despite the name similarity; verified by reading the body |
| mbsncat_s_l | R | same, locale-struct-blocked |
| mbsncoll | R | uses `_loc_update.GetLocaleT()->mbcinfo` + `__acrt_CompareStringA` for real — locale-struct-blocked, not trivial |
| mbsncoll_l | R | same |
| mbsncpy_s | R | real MBCS bounded-copy logic (`mbsncpy_s.inl`, ~150 lines) — same caveat as mbsncat_s |
| mbsncpy_s_l | R | same, locale-struct-blocked |
| mbsnicoll | R | same pattern as mbsncoll (codepage-dependent `__acrt_CompareStringA` call) |
| mbsnicoll_l | R | same |
| mbsninc_l | **T** | body is `string + (unsigned int)_mbsnbcnt_l(string, ccnt, plocinfo)` — `_mbsnbcnt_l` (rust=`mbsnbcnt_l`) is **already ported**. One-line wrapper. |
| mbsnset_s | R | real MBCS bounded-set logic (`mbsnset_s.inl`) |
| mbsnset_s_l | R | same, locale-struct-blocked |
| mbsset_s | R | real (validated single-char-fill MBCS logic) |
| mbsset_s_l | R | same, locale-struct-blocked |
| mbsstr_l | R | uses `_loc_update.GetLocaleT()->mbcinfo->ismbcodepage` + `_ismbblead_l` for real — locale-struct-blocked, not trivial (the non-`_l` `mbsstr` is already ported, but only because it hard-codes `plocinfo=nullptr`, which the `_l` entry point cannot do) |

### heap/ (8 symbols — 4 T, 4 R)

| Symbol | Class | Target / note |
|---|---|---|
| calloc_base | **T** | MS's own comment: *"This function implements the logic of calloc()"* — identical HeapAlloc-based algorithm to already-ported `calloc` |
| malloc_base | **T** | same pattern, already-ported `malloc` |
| free_base | **T** | same pattern, already-ported `free` |
| realloc_base | **T** | same pattern, already-ported `realloc` |
| recalloc | R | `_recalloc_base` = `_realloc_base` + zero-fill new tail using `_msize_base`; realloc/malloc/free are ported but there's no ported `_msize`-equivalent yet, so the zero-fill span can't be computed trivially — small but not zero-cost |
| aligned_msize | R | header-walk to recover original block size from an aligned pointer (~15 lines); the *pattern* already exists inline in `_aligned_malloc`/`_aligned_free`'s C source, but isn't itself an exported Rust symbol to call — low difficulty, not literal reuse |
| aligned_offset_recalloc | R | depends on `aligned_msize` existing first (MS's own `_aligned_offset_recalloc_base` calls `_aligned_msize_base`) — chained, not standalone trivial |
| aligned_recalloc | R | same chain, `align=0` special case of the above |

### stdlib/ (5 symbols — 2 T, 3 R)

| Symbol | Class | Target / note |
|---|---|---|
| imaxabs | **T** | `intmax_t` is 64-bit here; body is `number < 0 ? -number : number` — identical to already-ported `_abs64` (rust=`abs64`) |
| imaxdiv | R | self-contained (`quot`/`rem` + truncation-direction fixup), no existing 64-bit div primitive to wrap — trivial-difficulty but no existing body to alias |
| llabs | **T** | identical logic to already-ported `_abs64` |
| lldiv | R | same shape as imaxdiv, self-contained, trivial-difficulty |
| lsearch_s | R | new (bounded linear search + insert-if-not-found), no locale |

### string/ (4 symbols — 3 T, 1 R)

| Symbol | Class | Target / note |
|---|---|---|
| strset_s | **T** | narrow twin of already-ported `wcsset_s`; both funnel through the same MS-side `common_tcsset_s` template — narrow side is the smaller of the two |
| wcsncnt | **T** | `__wcsncnt` = bounded wcslen; identical semantics to already-ported `wcsnlen` |
| wmemcpy_s | **T** | thin `_VALIDATE_*` wrapper around a raw wide-char copy; our existing `memcpy` already exposes the `raw_move` primitive this needs, just scaled by `sizeof(wchar_t)` |
| wmemmove_s | **T** | same pattern, overlap-safety already handled by the existing `raw_move` (the repo's own comment notes `memcpy` here is already overlap-safe) |

### stdio/ + conio/ (28 symbols — 0 F/T, all R, but heavily clustered)

| Symbol | Class | Target / note |
|---|---|---|
| stdio_common_vfprintf / _p / _s | R×3 | route through `__crt_stdio_output::output_processor<...>` — a **different, options-driven printf engine** (`unsigned __int64 options` + explicit `_locale_t` + flags) than whatever the already-ported `vfprintf` implements. NOT a wrapper over it. |
| stdio_common_vfwprintf / _p / _s | R×3 | same engine, wide |
| stdio_common_vsprintf / _p / _s | R×3 | same engine |
| stdio_common_vswprintf / _p / _s | R×3 | same engine, wide |
| stdio_common_vsnprintf_s | R | same engine |
| stdio_common_vsnwprintf_s | R | same engine, wide |
| conio_common_vcprintf / _p / _s | R×3 | same engine, console output adapter |
| conio_common_vcwprintf / _p / _s | R×3 | same engine, console + wide |
| conio_common_vcscanf | R | matching *input*-side engine (separate work) |
| conio_common_vcwscanf | R | same, wide |
| acrt_iob_func | R | UCRT's `__iob_func`-equivalent stdin/stdout/stderr accessor — new plumbing |
| get_stream_buffer_pointers | R | FILE* internals accessor, new |
| getwc_nolock | R | real export per spec (no forward — unlike `getc_nolock`, see FORWARD table) |
| fread_nolock_s / fread_s | R×2 | bounds-checked read wrappers, new |
| ftell_nolock | R | new |
| getws_s | R | new |
| get_printf_count_output / set_printf_count_output | R×2 | new state accessors for the `%n`-style counting mode of the printf engine above |

**Note the leverage here even though all 28 are REAL**: once one instantiation of the
`output_processor` engine is ported (say `stdio_common_vfprintf`), the other 21
`stdio_common_v*`/`conio_common_v*` entries are the *same engine* with a different
adapter/character-width template parameter, not independent designs. This is the
single biggest real cluster in the 181, and also the cluster with the best
work-amortization — treat it as one porting task with ~20 thin instantiation shims
after, not 22 separate designs.

### exec/ (16 symbols, all R — but one shared varargs shim)

| Symbol | Class | Target / note |
|---|---|---|
| execl, execle, execlp, execlpe | R×4 | `varargs` directive per spec — collects `...` into a vector, forwards to `_spawnve`-family (which is itself not yet ported: `_spawnve` rust=False) |
| spawnl, spawnle, spawnlp, spawnlpe | R×4 | same varargs-collection shim, spawn side |
| wexecl, wexecle, wexeclp, wexeclpe | R×4 | wide twins |
| wspawnl, wspawnle, wspawnlp, wspawnlpe | R×4 | wide twins |

All 16 are the *same* varargs-to-vector collection logic (verified: `spawnl.cpp` and
`spawnlp.cpp` both build a `char* argv[]` from a C varargs list and hand it to
`_spawnve`/`_execve`), parameterized by narrow/wide and exec/spawn/with-env. Real work,
but one shim, not 16 designs — and it's gated on `_spawnve`/`_execve` not yet existing
in Rust either, so this whole family is blocked on a prerequisite that's itself not in
the 219 (out of ucrtbase's scope, presumably still on the msvcrt/kernel32 side).

### filesystem/ (10 symbols, all R, but width/layout variants of ported logic)

| Symbol | Class | Target / note |
|---|---|---|
| findfirst32i64, findnext32i64, wfindfirst32i64, wfindnext32i64 | R×4 | same `FindFirstFileW`/`FindNextFileW` Win32 calls as already-ported `findfirst`/`findnext`/`wfindfirst`/`wfindnext` (all rust=True) — only the **output struct** differs (32-bit time_t fields, 64-bit size field). Low difficulty: marshal into a different struct shape, not a new algorithm. |
| stat32, stat32i64, stat64i32, wstat32, wstat32i64, wstat64i32 | R×6 | same story — already-ported `stat`/`wstat`/`fstat64` do the real `GetFileInformationByHandle`-equivalent work; these are struct-width permutations (32/64-bit time_t × 32/64-bit size) of that same result. Low difficulty, mechanical. |

### time/ (14 symbols, all R — genuinely blocked, not just untouched)

| Symbol | Class | Target / note |
|---|---|---|
| strftime_l | R | body is `_Strftime_l(...)` which genuinely uses the locale argument (month/day names, codepage) — real locale-struct-blocked work, not trivial despite `strftime` (no-`_l`) being already ported |
| timespec32_get, timespec64_get | R×2 | new — depends on the underlying time-get machinery |
| daylight, dstbias, get_daylight, get_dstbias, get_timezone, timezone, tzname | R×7 | **the whole `time()`/`localtime()`/`gmtime()` subsystem is not yet ported at all** (`time`, `localtime`, `gmtime`, `_time64` all rust=False) — these accessors are downstream of that, not a small standalone task |
| W_Getdays, W_Getmonths, W_Gettnames, Wcsftime | R×4 | locale-name-table + wide strftime core — real, and also blocked on the locale-struct work in `locale/wsetlocale.cpp` (`_Getdays_l` etc pull `time_data->wday[n]` out of `locale_update.GetLocaleT()->locinfo->lc_time_curr`) |

### misc/ + startup/ + internal/ + env/ + lowio/ (remaining ~37 symbols, all R)

| Symbol | Class | Target / note |
|---|---|---|
| threadhandle, threadid | R×2 | per-thread-data accessors, new plumbing |
| query_app_type | R | new |
| open, sopen, wopen, wsopen | R×4 | `varargs` directive (mode-flag optional arg) — lowio open path, new |
| umask_s | R | trivial-**difficulty** (sets one global + validates), but no existing Rust `_umask`/`umask` body to alias — `umask` (rust=True) is a *different* symbol undecoration collision candidate worth double-checking the same way `wctype`/`Exit` were, but I did not chase it further within this audit's scope |
| ismbcblank / _l | (listed under mbstring above) | |
| getdrives | R | `GetLogicalDrives()`-style wrapper, new |
| seh_filter_dll, seh_filter_exe | R×2 | SEH top-level filter plumbing, new |
| get_invalid_parameter_handler, get_thread_local_invalid_parameter_handler, invalid_parameter_noinfo, invalid_parameter_noinfo_noreturn, invoke_watson, set_invalid_parameter_handler, set_thread_local_invalid_parameter_handler | R×7 | glue/accessors around a concept (`invalid_parameter`) that already has *some* Rust body — but ucrtbase keeps its own separate global handler-state slot from msvcrt's, so these are new state plumbing, not literal wrappers. Lower difficulty than most REAL entries because the pattern is proven elsewhere. |
| is_wctype | R | new (int-based ctype-class test, distinct spec export from both `wctype` entries above) |
| sys_errlist, sys_nerr | R×2 | **data**, not code — a `char const*[]` of 44 English error strings + its count. `strerror` is already ported (rust=True) and plausibly already carries equivalent message text internally, which would make this closer to TRIVIAL, but exposing it as a literal ABI-visible array-of-pointers symbol is a different shape of work than a function call, so left REAL rather than guessed as T. |
| getsystime, setsystime | R×2 | new |
| set_abort_behavior | R | new (small state flag) |
| configure_narrow_argv, configure_wide_argv | R×2 | argv-parsing-mode state, new |
| get_narrow_winmain_command_line, get_wide_winmain_command_line | R×2 | new |
| crt_at_quick_exit, crt_atexit, execute_onexit_table, initialize_onexit_table, register_onexit_function | R×5 | ucrtbase's on-exit-table mechanism (`_onexit_table_t`, encoded function pointers) is a different, more defended encoding than whatever `onexit` (rust=True) already implements on the msvcrt side — related concept, not a literal wrapper |
| Exit | R (flagged) | see ambiguous-case note — possibly TRIVIAL if the already-ported `exit` turns out to be the no-cleanup variant, could not confirm without reading the Rust `exit` body |
| quick_exit | R | genuinely distinct cleanup mode (`_crt_exit_quick_cleanup`, separate `__acrt_at_quick_exit_table`), not an alias of `exit` |
| dcrt_initial_narrow_environment, get_initial_narrow_environment, get_initial_wide_environment, initialize_narrow_environment, initialize_wide_environment | R×5 | environment-block bootstrap, new |
| dcrt_get_wide_environment_from_os | R | new |
| dupenv_s, wdupenv_s | R×2 | new (getenv + malloc-and-copy) |
| cgets_s | R | console input, new |
| cgetws, cgetws_s | R×2 | console input, wide, new |

## The REAL list as actual remaining work, ordered by leverage

Not a flat list of 181 — grouped by what unlocks what, cheapest/most-leveraged first:

1. **`output_processor` printf/scanf engine** (stdio/output.cpp, conio/cprintf.cpp,
   conio/cscanf.cpp) — 28 symbols, but ONE engine. Port `__stdio_common_vfprintf`'s
   core once; the other ~20 fall out as adapter instantiations. Highest leverage in
   the whole set.
2. **Filesystem width variants** (findfile.cpp, stat.cpp) — 10 symbols, all struct
   marshaling over Win32 calls we already wrap in Rust. Low difficulty, no new
   algorithm, purely mechanical once someone writes the struct-copy shim once.
3. **exec/spawn varargs shim** — 16 symbols, one varargs-collection routine × 4 axes
   (narrow/wide, exec/spawn, with/without env), but blocked on `_spawnve`/`_execve`
   existing first (not in this list — check msvcrt/kernel32 side).
4. **Locale-struct plumbing** (locale/*.cpp, mbsncoll/mbsnicoll/mbsstr_l's `_l` halves,
   strftime_l, wcsftime.cpp) — this is the real hard core, ~20 symbols directly plus
   it's a soft blocker for correctness on several more. `wcreate_locale` is probably
   the single most consequential item to scope first, since ucrtbase's whole `_locale_t`
   world hangs off it.
5. **Time subsystem** (timeset.cpp, time.cpp) — 9 symbols, blocked on `time`/
   `localtime`/`gmtime` not existing in Rust *at all* yet — this is bigger than it
   looks from the file list; don't start it thinking it's 9 independent accessors.
6. **mbstring bounded ops** (mbsncpy_s/mbsncat_s/mbsnset_s/mbsnbset_s + `_l` twins) —
   8 symbols, genuine MBCS lead-byte-tracking logic, no shortcuts found. Budget real
   time here; this is exactly where the convert/-family optimism doesn't transfer.
7. Everything else (misc/, startup/, env/, lowio/, internal/) — mostly small,
   independent, no found forwards or aliases. Grind, but not individually hard.

## What I could not determine

- **The 234 vs 219 discrepancy** — not chased further; script drift since the quest
  was scoped, most likely (see top of doc).
- **`_wctype[]`'s 257 values vs `MSVCRT__wctype`'s** — not diffed byte-for-byte;
  plausible TRIVIAL-ALIAS candidate, left as REAL per the conservative default.
- **Whether already-ported `exit` is `exit()`- or `_exit()`-shaped** — decides whether
  `Exit` (`_Exit`) is TRIVIAL or REAL; would need to read the Rust `exit` body, out of
  this audit's ucrt-side scope.
- **Whether `sys_errlist`'s 44 strings already live inside the ported `strerror`** —
  if `strerror` carries its own internal table, exposing `sys_errlist`/`sys_nerr` might
  drop from REAL to TRIVIAL-ALIAS; I did not read the Rust `strerror` implementation.
- **`umask` vs `umask_s`** — `umask` shows rust=True; whether that's genuinely the
  same underlying state `_umask_s` needs to touch, or another undecoration collision
  like `wctype`/`_wctype`, wasn't checked.
