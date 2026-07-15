# msvcrt / ucrtbase from pieces — lane plan

> **Lane:** `msvcrt-homebrew` · **Status:** planning (survey done 2026-07-12) · **Size:** M–L
> **Roadmap home:** `docs/product/vibeos-roadmap.md` Tier 2 ("msvcrt / ucrtbase from pieces").
> **Thesis:** the Rust user-mode CRT is not a translation project — it is an *assembly* project.
> Most of the pure-algorithm CRT is **already translated and battle-tested** inside `cnt-rust`,
> it's just wired into the *kernel* CRT (`libcntpr.a`) only. The work is to (a) propagate those
> proven flips out to the *user-mode* CRT, (b) wire the dormant `crt-startup-rust` floor, (c) fill
> the pure-algorithm gaps, and (d) sequence the ABI-bound C++ runtime *last* (deferred, not
> abandoned — §4). No fresh whole-DLL translation, no fighting ConcRT/EH *yet*.

---

## 2026-07-12 (wave 4) — the wide family LANDED, and the three-harness set became four

**Flipped (14, all pure):** `wcslen wcscmp wcsncmp wcschr wcsrchr wcsstr wcsspn wcscspn
wcspbrk wcscpy wcscat _wcsrev _wcsset _wcsnset`. Verdict: **45 MATCH / 0 SIGN-ONLY /
0 DIVERGE**, all bounded fns in-bounds, every flipped primitive within 2× of its C.

**The premise held.** `wchar_t` is unsigned by definition and `Elem::diff for u16` is
already `(a as i32) - (b as i32)`, so the signed-char contract that forced `cmp_u`/`ncmp_u`
into cnt-rust's core in wave 3 **cannot recur** on the wide side. **cnt-rust was not touched
by this wave** — the kernel's libcntpr contract is unchanged.

**`wcs.c` is now a SECOND reference TU** (176 public symbols, disjoint from `string.c`'s 106
— asserted). One *union* rename prelude covers both, so cross-TU calls resolve `ref_*`→`ref_*`
and `wcs.c` reaches `string.c`'s `fpnum_parse`/`memcpy`/`strlen` instead of the host's. The
refstub now pulls glibc's `<wchar.h>`/`<wctype.h>` **under** the 16-bit `wchar_t` remap — the
opposite of the rule for the narrow headers, and it is what makes ~30 of wcs.c's wide
definitions agree with glibc's declarations *for free*. Only five still disagree in **shape**
(`wcstok`'s arity; `wcstol`/`wcstoul`'s LP64 `long`; `wcstombs`/`wctomb`, which live in
`<stdlib.h>` and so are typed *before* the remap) and go through the existing stage-0/stage-3
hide-then-rename.

### What the harnesses caught (all four times, this is the point of the lane)

1. **`wcscmp` CLAMPS to −1/0/1** — unconditionally, no `_WIN64` branch — while its sibling
   **`wcsncmp` returns the RAW difference**. Same family, same file, opposite contracts.
2. **`wcsstr(L"", L"")` → `NULL`**, because its outer `while(*str)` never runs — whereas the
   narrow **`strstr("", "")` → `""`** (it special-cases the empty needle *first*). A real
   narrow/wide asymmetry inside msvcrt itself.
3. **The CRT perf tax (§9.63).** `perf` flagged `wcscmp` at 3.8–4.8×, and the root cause was
   structural: **a slice-based core cannot express a single-pass C string primitive** —
   building a `&[u8]` needs the length, and computing the length is exactly the scan the C
   skips. Which meant **narrow `strcmp` had the identical 3-pass shape and was slow ON MASTER**,
   unmeasured, because the gate benched only `memset/memcpy/memcmp/memchr/strlen`.
   *A perf gate only defends what it BENCHES.* Copy/compare/search are now single-pass
   transcriptions (the same call `memcmp` already made). `wcscmp` 3.8×→0.79×, `strchr`
   2.01×→1.00×, `strcmp` now benched at 1.00×. **The 2× bar was not raised.**
4. **NULL tolerance vanished (§9.64) — and it shipped a boot regression.** The single-pass
   rewrite dropped the NULL guard that `cstr_slice` had always provided (`NULL → &[]`, i.e.
   *NULL behaves as `""`*). `classify` still said 45/45 MATCH — **it never passes NULL**. The
   net suite went 15/15 → 13/15. Found by **A/B control**, not by the harness.

### The harness set is now FOUR orthogonal instruments

| harness | question | blind to |
|---|---|---|
| `classify` | wrong **values**? | access, speed, un-fuzzed inputs |
| `oob_guard` | right answer, wrong **memory**? | values, speed |
| `perf` | right answer, too **slowly**? | values, access |
| `assert_null_tolerant` | did we drop a **tolerance** the last build had? | everything else |

Every one of them was validated by **negative control** (inject the bug → it must fail).
A guard that cannot be shown to fail is worthless.

**The rule that generalizes** (§9.64): when you refactor a wrapper, enumerate the *implicit
guarantees of the helper you are deleting*, not just the contract of the function you are
rewriting. A NULL guard, a bounds clamp, a zero-length early-out — these live in the helper,
are invisible at the call site, and vanish silently when you inline it away. And note the
oracle: **the C is no help here** (it faults on NULL too). The oracle is *the previous build*.

**Deferred, unchanged:** the locale-dependent wide set (`_wcsicmp`, `_wcsnicmp`, `_wcsicoll`,
`_wcsnicoll`, `wcscoll`, `_wcslwr`, `_wcsupr`, `towlower`/`towupper`, the whole `isw*` ctype
block, the NLS `wcstombs` family). The harness pins the "C" locale, so it *would* report
MATCH — that is the documented blind spot (§9.60), not a license to flip. Verified still-C by
PE export RVA (they resolve at 0x045xxx, far from the Rust cluster at 0x0025a0–0x002c90).

### ⛔ WAVE 5 (float↔str) IS CANCELLED — the thesis does not reach this family

Probed before writing any integration (`strtod [PROBE]` in classify, a deliberately
NON-`#[no_mangle]` fn so nothing overrode the C). Result: **DIVERGE, by 14 ULPs.**

```
"0.0000000000000000087519423665610e-32"
  cnt-rust (DJ Delorie naive):  0x35c05f590315f1e3
  msvcrt   (bignum):            0x35c05f590315f1d5
```

Two different algorithms from two different lineages:
- **cnt-rust** = ReactOS libcntpr's **DJ Delorie parser** — a naive `r = r*10 + d`
  accumulator, then `r *= 10^e`. Its own doc says it "was never IEEE correctly-rounded".
- **msvcrt** = Wine's `fpnum_parse` + `bnum.h` — a base-10 **BIGNUM**, correctly rounded.

They were never the same function. **The kernel and msvcrt have genuinely different float
contracts.** Flipping `strtod`/`atof` from cnt-rust would silently corrupt every user-mode
float parse by double-digit ULPs on hard-to-round inputs — invisible in casual use,
lethal in a conformance suite.

**And the float↔str direction is not a separate wave at all.** msvcrt's converters ARE
printf calls:

```c
_ecvt: _snprintf(buf, 80, "%.*le", prec-1, number);
_fcvt: _snprintf(buf, 80, "%.*f",  ndigits, number);
_gcvt:  sprintf(buff,     "%.*g",  ndigit,  number);
```

So the whole float family — both directions — is **part of the printf engine (Phase C)**,
and the honest path to it is *porting msvcrt's own fpnum/bnum machinery*, i.e. a
TRANSLATION, not an assembly. Re-sequenced accordingly.

The probe stays in the tree permanently as an executable DO-NOT-FLIP record.

### Wave 5 (revised) — the `_s` secure-CRT family + the pure leftovers

All gradeable against the EXISTING `string.c` reference TU — **zero new harness work**:
- radix `_s` (10): `_i64toa_s _i64tow_s _itoa_s _itow_s _ltoa_s _ltow_s _ui64toa_s
  _ui64tow_s _ultoa_s _ultow_s` — wave 1's siblings, plus bounds-check + errno.
- string `_s` (5): `strcpy_s strcat_s strncpy_s strncat_s _strnset_s` (cnt-rust's cabi
  already has the wide `wcscpy_s`/`wcsncpy_s`/`wcscat_s`/`wcsncat_s`).
- pure leftovers: `strtok` `strtok_s` (cnt has `strtok.rs`; the ref TU already models
  `thread_data->strtok_next`), `_swab`, `__strncnt`.

The `_s` contract is *errno + the invalid-parameter handler*, not just the buffer — so the
harness must grade `errno` too, which means providing `_errno`/`_invalid_parameter` for the
crate under test (the reference's own are namespaced `msvcrt_ref_*`).

**Also now a real gate:** `classify` used to print `DIVERGE ✗ DO NOT FLIP` and **exit 0**.
It now exits 1 if a FLIPPED function diverges (`[PROBE]` rows exempt by design).
Negative-control validated.

Owed: replace `--whole-archive` with per-symbol `-u` pulling (it currently force-includes the
crate's bundled `compiler_builtins` into msvcrt.dll).

---

## 0. TL;DR — the shape of the win

| | |
|---|---|
| **C we'd otherwise translate** | ~54K LOC `sdk/lib/crt/` + ~44K LOC `dll/win32/msvcrt/` local copies + ~64.5K LOC `sdk/lib/ucrt/` |
| **Already Rust, proven in boot** | `cnt-rust` (10.9K LOC, 208 exports) + `cnt-ctype` (43 exports) + `crt-startup-rust` (3.1K LOC, 121 exports) |
| **Where the Rust is wired today** | (a) `libcntpr.a` (kernel/ntdll CRT) — 37 `REMOVE_ITEM` flips in `libcntpr.cmake`; (b) **the startup floor is ALREADY live** for every **cargo-driven** module via reactos-link's `.msvcrtex_crt()` (`crt-startup-rust`/`crt0-rust`/`crt-dllmain-rust`) — this is NOT dormant (`static-libs-translation-audit` §243: "msvcrtex → Rust DONE 2026-06-13"). |
| **Where it is NOT wired** | The **traditional-cmake** CRT: the `msvcrtex`/`crt` static libs *and* the `msvcrt.dll` body (`msvcrt_static`) are **100% C**. `ucrt`/`ucrtbase` too. |
| **Deferred (not in scope *now*)** | C++ EH / RTTI / ConcRT / undname — ABI-fused to per-arch asm; hardest ROI, so *last*, not *never* (§4) |
| **Smoke gate** | msvcrt is **not** on the early-boot path; it's exercised at `shell ready` via **explorer + shell32** (both import msvcrt), plus the services tier (`dcomlaunch`, `rpcss`). So smoke *does* gate this lane, and a broken flip breaks the desktop — not the kernel. |

The first move (Phase A) is nearly free: `crt-startup-rust` is a 3,135-LOC faithful port of the msvcrtex startup floor that **already exists in the workspace and is referenced by zero cmake**. Turning it on is wiring, not translation.

---

## 1. The terrain — how the CRT is actually built

There are **three** parallel CRT source trees, and the biggest one is compiled **more than once from the same cmake variables**. Getting this wrong is the #1 hazard (field manual §1346/§1348).

### 1a. `sdk/lib/crt/` — the static-lib source of truth (582 files, 54,341 C LOC + 87 asm)

The same `.c` files feed **two** different products via two variable families:

```
sdk/lib/crt/**/*.cmake  set LIBCNTPR_<CAT>_SOURCE   AND   CRT_<CAT>_SOURCE
                                     │                            │
             libcntpr.cmake  ───────┘                            └─────── crt.cmake
             list(APPEND LIBCNTPR_SOURCE ${LIBCNTPR_*_SOURCE})    list(APPEND CRT_SOURCE ${CRT_*_SOURCE})
                     │                                                    │
              add_library(libcntpr …)                            crt static lib  →  msvcrt_static
              KERNEL / ntdll CRT                                 USER-MODE CRT (msvcrt.dll body)
```

`crt.cmake` does `list(APPEND CRT_<CAT>_SOURCE ${LIBCNTPR_<CAT>_SOURCE})` for most categories, so **the user-mode CRT is a superset of the same C the kernel CRT compiles.** The cnt-rust cohab today uses `list(REMOVE_ITEM LIBCNTPR_SOURCE …)` — which only scopes the kernel side. The user-mode `CRT_SOURCE` still has the C. **That asymmetry is the entire "not wired to user-mode" state.**

> ⚠️ **The scoping trap (§1346/§1348):** you cannot flip a family by commenting a `.c` out of `LIBCNTPR_<CAT>_SOURCE` — that variable is *also* consumed by `CRT_<CAT>_SOURCE`, so you'd yank the symbol from msvcrt too and break user-mode (undefined-symbol at csrss/explorer). The kernel-only flip works *because* `REMOVE_ITEM` operates on the aggregated `LIBCNTPR_SOURCE` list after the append, not on the shared per-category var. To extend a flip to user-mode you deliberately `REMOVE_ITEM CRT_SOURCE …` **and** provide the Rust symbol into the msvcrt link.

Static libs built here: `libcntpr` (kernel), `crt`, `msvcrtex`, plus `crtmath`, `chkstk`, `dbgrpt`.

### 1b. `dll/win32/msvcrt/` — msvcrt.dll (1,433 exports, ~44K LOC local C) — **the high-value target, and a distinct THIRD body**

⚠️ **This is the key correction to the naïve "propagate the flips" model.** msvcrt.dll's body (`msvcrt_static`) is **not** built from `CRT_*_SOURCE`. It compiles a **hand-listed set of its own local, Wine-aggregated monolith files** (`dll/win32/msvcrt/{string,wcs,math,file,mbcs,locale,…}.c`) — e.g. `string.c` alone is **3,592 LOC / ~106 functions in one file**. So this body is **not per-function `REMOVE_ITEM`-flippable** the way `libcntpr`/`crt` are (those are one-function-per-file under `sdk/lib/crt/`). Flipping the DLL requires either (i) splitting the monoliths, or (ii) a **`--whole-archive` symbol-override cohab** — link a scoped Rust `.a` under `-Wl,--whole-archive` before `msvcrt_static` so its symbols are DEFINED before the C monolith is pulled (plain link-order is insufficient — the monolith is pulled anyway for its un-flipped symbols and wins first-match; see Phase A). The C copies then become ignored `--allow-multiple-definition` duplicates (dead code in the binary). Since msvcrt.dll *is* the CRT (it doesn't import CRT symbols from ntdll), the `was replaced` import-vs-local landmine (§1277–1279) does **not** bite here — it's the benign local-vs-local dup. This body is what **explorer + shell32 + everything** imports, so it's the highest-value, smoke-gated target. Links:

```
msvcrt  ←  msvcrt_static (legacy CRT body)  +  msvcrt_shared (C++ EH/RTTI/undname, shared w/ ucrtbase)
           + crtmath + dbgrpt + vcruntime + wine + chkstk + PSEH_LIB
           + (INTERFACE) msvcrtex   ← the startup floor, pulled into every msvcrt consumer
```

`msvcrt.spec`: 1,433 exports — **124 `?`-mangled C++** (≈40–50 logical symbols after per-arch variants), 129 `stub`, the rest plain-C. arch qualifiers: 160 i386-only, 58 win64, 33 arm.

### 1c. `sdk/lib/ucrt/` + `dll/win32/ucrtbase/` — the modern CRT (64,533 LOC, real target)

`ucrtbase.dll` is a **real built SHARED target** (`dll/win32/CMakeLists.txt:215`), not a stub. It builds the `ucrt` OBJECT lib (24 category dirs: lowio, stdio, convert, heap, locale, …) + `msvcrt_shared` + `crtmath` + `vcruntime`. It shares **only** msvcrt's C++ runtime support with msvcrt.dll; its CRT body is an independent modern tree. Nothing on our boot/desktop path imports ucrtbase today (it's for VS2015+ apps we don't ship yet) — so it is a **follow-on**, not part of the critical path.

`msvcrt20`/`msvcrt40` are legacy MFC-era shims linking the same `msvcrt_static`; not on any boot path; they ride along for free when `msvcrt_static` gets Rust-fed.

---

## 2. What already exists in Rust (the pieces to assemble)

### 2a. `cnt-rust` (10,873 LOC, 208 `#[no_mangle]`) — the CRT body, minus state and C++

Architecture: `core_rs/` = pure `no_std`/no-unsafe/slice-based logic (Kani-able); `cabi/` = thin `#[no_mangle]` shims. Covered exports (union — the "already have" list):

- **str / wcs** — full `strcat/chr/cmp/cpy/cspn/len/ncat/ncmp/ncpy/nlen/pbrk/rchr/spn/str/tok(+_s)`, `_stri/nicmp _strlwr/upr/rev/set/nset _memicmp`, and the `wcs*` twins incl. `wcscat_s/cpy_s/ncat_s/ncpy_s _wcsi/nicmp _wcslwr/upr/rev/nset wcscoll`.
- **mem** — `memcpy/move/set/cmp/chr _memccpy` (i386 keeps asm; see gap G6).
- **int↔str** — `atoi/atol/_atoi64/_wtoi/_wtol/_wtoi64`, the whole `_itoa/_ltoa/_ultoa/_i64toa/_ui64toa` + `_itow/…/_ui64tow` (+ `_s` variants), `strtol/ul/ull/_strtoi64`, `wcstol/ul/_wcstoi64/ui64`.
- **float↔str** — `_ecvt/_fcvt/_gcvt(_s)` + the engine `format_float/format_floatw`.
- **path** — `_makepath/_splitpath/_wmakepath/_wsplitpath`.
- **mb/wc** — `mbstowcs/wcstombs/mbtowc/wctomb` (+ `mbrtowc/wcrtomb` in crt-startup-rust).
- **stdlib** — `qsort/bsearch/_lfind/rand/srand/rand_s`, env (`getenv/_putenv/_wgetenv/_wputenv/__p__environ/…`), `atexit/_onexit`, `_decode/_encode_pointer`, `_get/set_invalid_parameter_handler/_invalid_parameter`.
- **math / libm** (3,039 LOC `cabi/math.rs`) — `abs/labs/div/fabs/sqrt/pow/exp/exp2/log/log10/sin/cos/tan/asin/acos/atan/atan2/sinh/cosh/tanh` (+ f32 twins), `ceil/floor/fmod/modf/ldexp/scalbn/round/sincos/nextafter/_hypot/_logb`, IEEE predicates `_finite/_isnan/_chgsignf/_dclass/…`, `_matherr`.
- **printf/scanf ENGINE** — `streamout`/`wstreamout` (the printf core the C `va_list` wrappers call), `format_float(w)`, and scanf `cnt_vsscanf/vswscanf/vsnscanf/vsnwscanf`.

### 2b. `cnt-ctype` (353 LOC, 43 exports) — ctype tables + classifiers

`is*`/`to*` narrow set + `iswalnum/…/iswxdigit`, exported data tables `_ctype/_pctype/_wctype/_pwctype`, `__isascii/__toascii/__iscsym(f)`. **Caveat:** wide `isw*`/`towlower` currently *forward to a still-C `iswctype`* (gap G4).

### 2c. `crt-startup-rust` (3,135 LOC, 121 exports) — the msvcrtex floor, **LIVE for cargo modules**

A faithful port of the mingw msvcrtex startup floor. **NOT dormant** — it is wired via **reactos-link's `.msvcrtex_crt(MsvcrtexKind)`** (`sdk/lib/rust/reactos-link/src/lib.rs:742`) into every **cargo-driven** module (winlogon-cargo, services-cargo, lsass-cargo, userinit-cargo, cabinet-cargo, mlang-cargo, …). Personality is selected **per-module by cargo feature** (`crtexe`/`wcrtexe`/`crtdll`) + a sibling (`crt0-rust` ansi/wide bridge for exes, `crt-dllmain-rust` default DllMain for dlls). It is referenced by **zero cmake** because the *cmake* modules (explorer, shell32, the ~700 traditional consumers) still link the **C** `msvcrtex` static lib.

> **Why the floor is hard to extend to cmake modules:** the C `msvcrtex` archive selects personality by **lazy `.o` pull** (the linker drags in `crtexe.o` XOR `wcrtexe.o` XOR `crtdll.o` based on the module's entry symbol). `crt-startup-rust` selects it by **compile-time feature**, and the exe-vs-dll `.CRT$X*` pre-init sentinel (exe = −1, dll = NULL) *must not coexist* in one image. So a **single** cmake `msvcrtex.a` built with one feature set cannot serve exe-ansi + exe-wide + dll modules at once. Extending the Rust floor to cmake modules therefore needs either 3 feature-variant archives with per-module selection, or a personality-neutral restructure — a real design task, **not** a drop-in. (This is why the floor landed on the cargo path.)

Covers `mainCRTStartup/wmain/WinMain/wWinMain`, `DllMainCRTStartup/_CRT_INIT`, `atexit/_onexit`, `_setargv/_wsetargv`, `__main/__do_global_ctors/dtors`, `__security_init_cookie/__report_gsfailure`, `_gnu_exception_handler`, TLS callbacks, PE-section helpers (`_FindPESection*`, `_GetPEImageBase`), `_pei386_runtime_relocator`, `__acrt_iob_func`, `_get_output_format`, CRT-residual libm (`exp2/round/sqrtf/sincos/isblank`), `mbrtowc/wcrtomb`, plus the `.CRT$X*` init-array + `_tls_*` + security-cookie data statics. Siblings `crt0-rust` (54 LOC) and `crt-dllmain-rust` (55 LOC).

### 2d. Adjacent (not CRT, but assembly candidates)

`rtl-rust` umbrella (the 39-crate family) exports the counted-string / charset-conversion / int-conv `Rtl*` surface (`rtl-strings`, 282 exports) — msvcrt's Rtl-backed paths can call these. `vcruntime` (707 LOC C: security cookie, `_purecall`, `__std_terminate`, `mainCRTStartup`) largely overlaps the crt-startup-rust floor and can be subsumed as the floor lands.

---

## 3. Gap analysis — what is NOT Rust yet

| # | Bucket | LOC (approx) | Rust status | Disposition |
|---|---|---:|---|---|
| G1 | **Public printf/sprintf/vsnprintf/swprintf entry symbols** | small (thin `va_list` wrappers) | engine (`streamout`) is Rust; the exported entry points are still C shims | **Phase C.** Keep tiny C variadic trampolines delegating to `streamout` (calling variadics *from* Rust is stable — §873/§1589), OR emit Rust `...`→va_list shims. |
| G2 | **stdio FILE engine** (`stdio/file.c` 4,045 LOC) | C | owns `_iob[]`, `__pioinfo[]` global state | **Phase D.** State-bearing; needs an explicit global-state design (see §7). Highest-risk pure-CRT piece. |
| G3 | **locale** (`locale/locale.c` 1,532) + **mbstring** (2,617) | C | global locale state (`__mb_cur_max`, `__lc_*`, codepage tables) | **Phase D** (locale) / **Phase C** (the pure `mbs*` algorithm leaves). |
| G4 | **Wide ctype** (`iswctype` real classification) | ~part of string/ | cnt-ctype forwards to C `iswctype` | **Phase C.** Finish the wide classification in Rust. |
| G5 | **Thin syscall wrappers** — process/time/conio/direct/signal/sys_stat/misc | ~6,500 | C | **Phase C/deferred.** Easy but numerous; the "shared-C-runtime plumbing" class (§1589) — flip surfaces latent NULL-accessor/PEB bugs, so smoke each wave. |
| G6 | **i386 mem/byteswap** | asm | `mem`/`byteswap` crates guard `cfg(not(x86))` | **Leave asm on i386** unless a reason appears; amd64/arm64 already Rust. |
| G7 | **C++ EH / RTTI / ConcRT / undname** (`wine/cpp.c` 1,603, `except*` ~3K, `concurrency.c` 3,839, `undname.c` 1,655, vcruntime C++ bits) | ~10K + asm | C | **Deferred to a later phase (§4)** — hardest ROI, so last. `undname` is the natural first leaf (pure algorithm, byte-diffable). |

---

## 4. Deferred — what we do *last*, not *never*

Everything here is in eventual scope (the project goal is full Rust); it's sequenced last because it's the hardest ROI and the highest regression blast radius. Nothing below is abandoned — it's parked until the plain-C CRT body is Rust and green. The archived verdict (`docs/engineering/archive/userland-wine-libs-audit.md` §297–305) is the *sequencing* rationale, not a permanent exclusion.

- **`__CxxFrameHandler`(3), `_CxxThrowException`, `type_info`, RTTI, `operator new/delete`, ConcRT** are fused to MSVC's exact per-arch vtable/frame layout and hand-written `.s` thunks. Today a faithful "translation" would be the same asm in `global_asm!` — so we defer until either (a) the surrounding CRT is Rust and these are the only C left, or (b) the SEH/unwinder work (`rtl-except`, the live exception dispatcher — see AGENTS.md SEH section) matures enough to express the frame handlers idiomatically. They live in `msvcrt_shared` (shared with ucrtbase), so translating them is a once-for-both payoff when we get to it.
- **x87/IEEE math corner cases** — the transcendental kernels are *already* Rust in cnt-rust (bit-exact, difftest-gated). Do **not** re-open the ones that landed; the still-C `math.c`/`float/` residue is in scope for a later wave, gated on a byte-diff harness (§1605) proving parity.
- **`undname.c`** (1,655 LOC name demangler) — pure-algorithm and byte-diffable, the natural *first* piece to pick up when we return to this bucket (no ABI entanglement). Not required for the current phases.

---

## 5. Import surface & smoke risk

- **746 distinct modules import msvcrt** (461 production; 222 under `dll/win32`, incl. ole32/oleaut32/rpcrt4/comctl32/shlwapi/setupapi/crypt32/wininet/ws2_32/…).
- **NOT on the early-boot path.** The kernel/subsystem chain (`ntoskrnl`, `ntdll`, `smss`, `csrss`/`winsrv`, `win32k`, `winlogon`, `userinit`, `kernel32`, `user32`, `gdi32`, `advapi32`, `lsass`) does **not** import msvcrt in production — only their apitests do.
- **Smoke exposure begins at `shell ready`:** `explorer` (`base/shell/explorer/CMakeLists.txt:100`) and `shell32` (`dll/win32/shell32/CMakeLists.txt:164`) both import msvcrt, and the boot marker *is* explorer reaching `_tWinMain`. So the ×3 smoke gate genuinely exercises the user-mode CRT — but a broken flip degrades the **desktop/shell**, it does not wedge the kernel. Also gated: the services tier (`dcomlaunch`, `rpcss` under `services.exe`).
- **Practical consequence:** iterate with the normal `./vibeos-cli gate`; failures show up as explorer/shell breakage or a service-host crash, not an early BUGCHECK. This makes the lane *lower-stakes to debug* than a kernel flip — the smoke screenshot + explorer serial tell you immediately.

---

## 6. The plan — phased  *(re-sequenced after the 2026-07-12 topology findings)*

Each phase is independently landable (gate ×3, then merge). Order is by leverage-per-risk. The lead phases attack **msvcrt.dll's body** (`msvcrt_static`) because that's the high-value, smoke-gated target (explorer/shell32 import it) — *not* the startup floor (already Rust for cargo modules, §2c) and *not* the `crt`/`libcntpr` static libs (weakly smoke-gated; deferred to Phase E).

### Phase A — prove the user-mode DLL-body cohab, one pure family  *[✅ DONE 2026-07-12 — ×3 green + exports verified all arches]*
The DLL body is Wine monoliths, so we use a **`--whole-archive` symbol-override cohab** (§1b), *not* per-file `REMOVE_ITEM`. **Landed mechanism** (`dll/win32/msvcrt/CMakeLists.txt`, crate `sdk/lib/rust/msvcrt-crt-rust`):
1. A **scoped** staticlib `msvcrt-crt-rust` depends on `cnt-rust` with `default-features = false` (only `core_rs` — the difftest-pinned logic; cnt-rust's 208 `#[no_mangle]` symbols do NOT leak). It hand-exports exactly the family being flipped, delegating to `cnt_rust::core_rs::*`. This is how we scope one family at a time (the whole `cnt-rust-lib` umbrella would flip all 208 at once, incl. stateful `getenv`/`atexit`/`rand`).
2. Build `libmsvcrt_crt_rust.a` via `add_custom_command` mirroring `libcntpr.cmake:557-627`, per-arch codegen (i386 uses the ReactOS `-sse,-sse2` baseline — wave 1 is integer-only so SSE is moot; `-C no-prepopulate-passes` §378).
3. **Link it under `-Wl,--whole-archive … -Wl,--no-whole-archive`** before `msvcrt_static`. ⚠️ **Plain link-order is NOT enough** (verified — it fails): the C monolith `string.c.obj` is still pulled for its ~100 un-flipped symbols (`strlen`, …) and drags in *its* copies of the flipped functions, which win first-match. `--whole-archive` forces the Rust defs in *first*; the C copies become the ignored `--allow-multiple-definition` duplicates. (Cost: the C bodies stay in the binary as dead code — expected cohab bloat until the family count justifies splitting the monolith.)
4. **VERIFY THE FLIP TOOK** — do not trust the dup-symbol warning or a green smoke alone (a green boot with C still winning is a false positive). Parse the DLL export table and confirm each flipped export's RVA lands in the Rust body, not the C body. (The wave-1 check found all 10 int-conv exports at 0x1580–0x1b90 = the `--whole-archive` Rust cluster, vs the C bodies at 0x80d0.) Then gate ×3; nuke the cohab target dir first (§1266–1268).
> Because msvcrt.dll *is* the CRT (it doesn't import CRT from ntdll), these are benign local-vs-local dups — **not** the `was replaced` import-vs-local landmine (§1277–1279). That landmine only bites *other* user-mode modules that link a Rust CRT `.a` **and** ntdll; msvcrt itself is safe. Keep it that way: only *msvcrt* links `msvcrt-crt-rust`.
>
> **Wave 1 (int↔string radix family):** `_itoa/_ltoa/_ultoa/_i64toa/_ui64toa` + wide twins. Semantics verified identical to msvcrt's `ltoa_helper` (sign only radix==10, unsigned reinterpret otherwise, lowercase digits, no NULL/radix guards). Structurally confirmed live on i386.

### Phase B — broaden the DLL-body flips, driven by the differential harness  *[wave 2 ✅ · wave 3 ✅ 2026-07-12]*

**The harness is the gate, not judgment.** `sdk/lib/rust/msvcrt-difftest` compiles the GENUINE `dll/win32/msvcrt/string.c` for the host (the real file is the TU — 106 symbols renamed `ref_*`) and fuzzes the **shipping** Rust against it, emitting a MATCH / SIGN-ONLY / DIVERGE verdict per function. Run it before flipping anything:
```
cargo test -p msvcrt-difftest --test classify     # verdict table
cargo test -p cnt-difftest                        # kernel contract still pinned
```

**Status: all 31 graded functions MATCH the genuine C exactly** (0 SIGN-ONLY, 0 DIVERGE). msvcrt.dll serves **39** functions from Rust: wave 1 (10, int↔string radix) + wave 2 (23, the pure MATCH set) + wave 3 (6, below).

#### Wave 3 — the unsigned-char contract family

Wave 2's 8 DIVERGE + 1 SIGN-ONLY were **not** "unflippable". They were **one root cause** wearing three masks, and the fix is a *contract adapter*, not a reimplementation:

**Root cause.** cnt-rust's `core_rs` speaks libcntpr's **signed-char** convention (`Elem::diff` for `u8` is `(a as i8) - (b as i8)`; `mem::chr` matches `(b as i8 as i32) == c`). msvcrt speaks **unsigned-char**. They agree on ASCII and **flip sign on any byte ≥ 0x80** — a silent corruption of comparison/search on non-ASCII data (UTF-8 paths, accented filenames, binary buffers).

**The fix, and the rule it establishes:**
- **Parameterize in cnt-rust** when the algorithm is identical and only a *contract knob* differs. `Elem::udiff` (default = `diff`; only `u8` overrides) + `string::cmp_u`/`ncmp_u` give the same loops the unsigned difference. `u16` needs no override — `wchar_t` is unsigned by definition, so **the wide family never had this bug**. The kernel keeps `Elem::diff`; `cmp`/`ncmp` bodies are **byte-for-byte untouched** (deliberately duplicated loops rather than a knob threaded through them — this crate has been bitten by codegen shifting under a hot primitive before, cf. `mem::set`'s DCE'd fill). Both contracts stay pinned, each by its own harness: `cnt-difftest` (kernel) and `msvcrt-difftest` (msvcrt).
- **Port into `msvcrt-crt-rust`** when the *algorithm itself* differs — keeps cnt-rust's libcntpr charter clean. `_memccpy` is one: libcntpr's stops at a NUL and compares signed; msvcrt copies straight through NULs and compares unsigned.
- **Some "divergences" need no code at all.** `memchr` is exactly reconciled by handing cnt's `chr` the sign-extended form of the truncated byte — `(c as u8) as i8 as i32` — which is precisely what msvcrt's `(unsigned char)c` cast means. `_strnset`'s *buffer* effect was already identical (the harness proved it: same bytes, `ret+0` vs `ret+11`); only the **return pointer** differed — msvcrt returns the **advanced** pointer. That is very likely a Wine bug (MS documents "a pointer to the altered string"), but a flip is a *pure refactor of the C it replaces*, so it is reproduced exactly rather than quietly "fixed".

**Flipped:** `strcmp strncmp memcmp memchr _memccpy _strnset`.

> ⚠️ **DEFERRED to Phase D — `_stricmp` / `_strnicmp` / `_memicmp`.** Fixing their signedness makes the harness report MATCH. **Do not believe it.** Their msvcrt bodies consult `get_locinfo()->lc_handle[LC_CTYPE]` and fold via `_tolower_l` — they are **locale-dependent**, while cnt-rust's fold is locale-blind ASCII. They coincide *only* in the "C" locale, which is exactly what the harness pins. Correct under the default locale; wrong the moment an app calls `setlocale()`. They belong with locale (Phase D). The harness prints them as a DEFERRED footer so they are not silently forgotten.

#### ⚠️ BLIND SPOT ZERO — the harness compares VALUES, so it cannot see a bad ACCESS

**This one actually shipped** (caught by the gate's net suite, one cycle later; field manual §9.60). The Rust `_strnset` scanned for NUL **unbounded** and clamped afterwards (`min(cstr_slice(p).len(), len)`), where the C reads at most `len` bytes (`while (*str && len--)`). The `min` still bounds the *fill*, so **every returned value and every written byte was identical to the C** — 20,000 fuzz cases, all MATCH. The only damage was the **read**, running off the end of a buffer that an `n`-suffixed API is explicitly allowed to leave unterminated (`gen_str` always NUL-terminates, so the fuzz could never construct the input that bites). It faulted in the guest via `mbcs.c`'s `u__strnset()` → `_mbsnset`/`_mbsnbset` and silently killed `nettest.exe`.

> **Rule: an `n`-suffixed CRT function must never contain an unbounded scan.** Bound it by `n` (`cstr_slice_bounded`) — never scan-then-`min`.

**Detector (permanent): `msvcrt-difftest/tests/oob_guard.rs`.** Probes the *access*, not the value: a NUL-free buffer placed flush against a `PROT_NONE` guard page, each bounded fn called with `n <= k`, each call in a forked child so the offender is named rather than taking the harness down. Validated both ways (reports `_strnset ✗ OVER-READ` on the bug, green on the fix). **Every new bounded flip gets a row.**
```
cargo test -p msvcrt-difftest --test oob_guard
```

#### ⚠️ The harness's OTHER blind spots (a harness can only falsify what it VARIES)

Both must be settled by **reading the C**, never by a green table:

1. **Locale** — the reference runs in the default "C" locale. Any msvcrt body that reaches `get_locinfo()` is graded on one arm of a branch it doesn't control. This is what nearly shipped the `_stricmp` trio above.
2. **`_WIN64`** — the reference TU is compiled for the **host** (x86_64 linux), where `_WIN64` is *undefined*, so every `#if defined(_WIN64)` in the C is graded on its **32-bit** arm. **Grep any new flip candidate for `_WIN64` / `_UCRT` / `_MSVCR_VER` before trusting a MATCH.** (`_MSVCR_VER` is safe — build.rs defines it as `0`, exactly as the real build does.)

`strncmp` is the one flipped function with such a branch, and it ships **two different contracts**: with `_MSVCR_VER=0` the `#if` reduces to plain `_WIN64`, so **i386 returns the raw unsigned diff while amd64/arm64 clamp to -1/0/1.** Both are graded from the single host reference, because the two C branches run after an *identical* loop and then compare the *same two bytes* — so `_WIN64` is provably `sign(#else)`, and `clamp(ref_strncmp(..))` is an **exact** model of the 64-bit branch, not an approximation. (Generalizable: a `#if` arm that is a pure function of the other arm's value can be modeled in the harness instead of compiled twice.)

#### Grade the artifact, not a proxy

Waves 1–2 graded **cnt-rust's** `cabi` shims — sound only while the wrappers were verbatim copies of them. Wave 3's wrappers deliberately *adapt* cnt-rust, so the proxy stopped being the artifact. The harness now links **`msvcrt-crt-rust`** itself.

That forced a crate split, and it is structurally unavoidable: a crate with `staticlib` in its crate-type is a final no_std artifact, so rustc demands a `#[panic_handler]` **in the same invocation that emits the rlib** — and that handler is then a duplicate `panic_impl` lang item inside the harness's std test binary. `crate-type` is not feature-conditional, so the two roles cannot share a crate. Hence the repo-standard split: **`msvcrt-crt-rust`** (rlib — the code, harness-linkable) + **`msvcrt-crt-cargo`** (staticlib umbrella — `extern crate` + the runtime bits; what cmake links). Same shape as `kernel-rust` over its sibling rlibs.

`assert_shims_live()` is the hard gate that keeps this honest: it asserts `strlen(NULL)==0` (proves the shims aren't **glibc**) *and* `_strnset` returns `str+n` (proves we're linked against the **msvcrt wrapper**, not cnt-rust's cabi shim, whose contract returns the start).

**VERIFY THE FLIP TOOK.** Never trust a green build or a dup-symbol warning: resolve each export's RVA in the built DLL and confirm it lands in the Rust `--whole-archive` cluster (and that the *deferred* ones still land in C). On i386 after wave 3: Rust cluster `0x2dc0–0x8820`; all 6 wave-3 exports inside it; `_stricmp`/`_strnicmp`/`_memicmp` at `0x7a320+` — still C. ✓

**Remaining candidates** (need the same harness treatment first): **`wcs*` is the big one — 33 candidates in `wcs.c`, and cnt-rust's wide core is already the correct unsigned contract, so the signed-char class cannot bite there.** Extending the harness needs a second reference TU (`wcs.c`: 176 public symbols; the extra externals are NLS — `CompareStringW`/`LCMapStringW`/`GetStringTypeW`/`WideCharToMultiByte` — plus heap and `MSVCRT__pwctype`; note it calls `fpnum_parse`, which `string.c` already provides). Then float↔str (`_ecvt/_fcvt/_gcvt`), `math.c`/`sincos.c`, `mbcs.c`.

**Two mechanism gotchas hit in wave 2 (both fixed, both will recur):**
1. **`-C no-prepopulate-passes` (the §378 guard) breaks cross-crate `#[inline]` core generics.** `_strrev`'s `buf.reverse()` pulls `core::ptr::swap_nonoverlapping`, which the flag leaves un-inlined AND un-emitted → `undefined symbol` at link. **Do not drop the guard** (it's why `memset` wasn't DCE'd into a spin); write the loop explicitly instead.
2. **`--whole-archive` force-includes the crate's bundled `compiler_builtins`** (`__chkstk`, `__divdi3`, libm shims) into msvcrt.dll. Benign so far (×3 green), but it means Rust's `__chkstk` can win over ReactOS's. *Owed cleanup:* pull only the flipped objects (per-symbol `-u`) instead of the whole archive — note plain link-order alone does NOT work (§1b).

> ⚠️ **Semantic watch is REAL and per-function — verified, not assumed (2026-07-12).** cnt-rust "tracks *libcntpr*, not textbook libc," and libcntpr's NT variants **do** diverge from msvcrt's Wine-derived bodies. Confirmed divergences found while scoping wave 2:
> - **path (`_splitpath`/`_makepath`) — DO NOT FLIP via cnt-rust.** cnt-rust is the classic **unbounded, non-MBCS** variant ("INT_MAX copy caps unreachable (not modeled)", "_LIBCNT_ strips _ismbblead"); msvcrt's `_splitpath` delegates to the **bounded `_splitpath_s`** (`_MAX_DRIVE`/…) and is MBCS-aware. Coincide for typical paths, but cnt-rust would **overflow** on overlong components where msvcrt truncates (a security regression) and mishandles MBCS. Flipping this needs a *msvcrt-contract* port (bounded + MBCS), not the cnt-rust drop-in.
> - **`_stricmp`/`_strnicmp`/`_memicmp` — CONFIRMED locale-aware (2026-07-12), deferred to Phase D.** cnt-rust is the "ASCII-only A..Z fold (NT variant — NOT the `toupper()` stricmp.c)"; msvcrt's bodies genuinely *are* locale-aware (`get_locinfo()->lc_handle[LC_CTYPE]` + `_tolower_l`). See the Phase B DEFERRED note — and note the harness reports MATCH for them once signedness is fixed, which is a **blind spot, not a license**.
>
> **Process:** for each candidate function, read `dll/win32/msvcrt/*.c` and confirm the contract matches cnt-rust's `core_rs` (byte-diff where pointer-free, §1605) *before* adding it to `msvcrt-crt-rust`. Only genuinely-identical leaves flip via cnt-rust; contract-divergent ones need a msvcrt-specific port (or stay C). Int-conv (wave 1) matched exactly; path did not. A **differential harness** (compile msvcrt's actual `.c` as the reference TU, fuzz cnt-rust against it — the §1605 methodology) would let this scale instead of per-function eyeballing.

### Phase C — fill the pure-algorithm gaps
- **G1 public printf entry points** — tiny C variadic trampolines → `streamout` (`_snprintf/sprintf/vsnprintf/_vsnwprintf/swprintf…`). Pattern is the §873 `_vsnprintf` shim, already used for the kernel `Rtl*Printf` families.
- **G4 wide ctype** — finish `iswctype` classification in cnt-ctype (kill the C forward).
- **G3(part)/G5 mbstring + thin wrappers** — flip the pure `mbs*` leaves and the syscall-wrapper families (process/time/conio/direct) wave-by-wave; the §1589 "shared-C-runtime plumbing" class — each flip lets a previously-unreached path run, so smoke every wave and expect the fault downstream of the flip.

### Phase D — the state-bearing pieces (stdio FILE engine, locale)  *[highest complexity]*
`file.c` (`_iob[]`, `__pioinfo[]`) and `locale.c` (`__mb_cur_max`, `__lc_*`, codepage tables) carry the shared-mutable-statics-with-init-order problem this repo already fights (captures(none) hazards, §378). Design decision up front (§8): model the globals as `#[no_mangle] static mut` exported data (byte-compatible with the C ABI so mixed C/Rust consumers agree), init order preserved from `crt_init.c`. The piece that most resembles a real translation — its own sub-lane with a byte-diff/behavioral harness.

### Phase E — the static-CRT family + the cmake startup floor  *[cleanup, weakly smoke-gated]*
- **`crt`/`libcntpr` static libs** — extend the existing `libcntpr.cmake` `REMOVE_ITEM` flips to `CRT_*_SOURCE` (the literal §1346/§1348 "built twice from the same vars" case — scope the `REMOVE_ITEM` correctly). Low risk (same sources + same cnt-rust symbols already boot-proven), but few boot modules statically link `crt`, so smoke barely exercises it — hence deferred, done for completeness/C-burndown.
- **cmake `msvcrtex` floor** — extend the Rust startup floor to the ~700 traditional-cmake modules (explorer, shell32, …). Blocked on the personality-selection redesign (§2c): needs 3 feature-variant archives with per-module selection, *or* migrating those modules to the cargo link path (the direction the rest of the fleet took). A real design task; sequence when the body is done.

### Phase F — ucrtbase parallel surface  *[follow-on, separate effort]*
Apply the same pure-algorithm flips to the `ucrt` OBJECT lib (`sdk/lib/ucrt/`, independent 64.5K-LOC tree). The Rust symbols already exist; it's cohab wiring against a different C set. Nothing on our boot path needs ucrtbase yet — no smoke urgency; do it after msvcrt is green and stable.

---

## 7. Risks & gotchas — field-manual index

Read these before touching the build; every one has already cost someone a session.

| Risk | Field manual | One-liner |
|---|---|---|
| CRT built twice from shared vars | **§1346/§1348** | `REMOVE_ITEM` the aggregated list, never the shared `*_SOURCE`; grep for the var's second `list(APPEND …)`. |
| **`X was replaced` (import vs local)** | **§1277–1279** | User-mode links Rust CRT **and** ntdll; a local def for an ntdll-imported symbol is fatal, `--allow-multiple-definition` won't save you. The central Phase-B hazard. |
| `was replaced` (stale/member-set) | §384, §1271–1274 | Different bug — stale `.a`. `rm libcntpr*.a && rm -rf cnt-rust-target`; put `--allow-multiple-definition` on EXE linker flags; strong `#[no_mangle] rust_eh_personality`. |
| amd64/arm64 garbage/hang from a cnt shim | §378 | `-C no-prepopulate-passes` (amd64+arm64); i386 `-sse,-sse2`. Suspect this first for arch-specific misbehavior. |
| Variadic printf entry | §873, §1589 | Tiny C shim → `streamout`; calling variadics *from* Rust is stable, defining `...` in Rust is not. |
| Stale target dir → false green | §1266–1268 | Nuke `build-<arch>/sdk/lib/crt/cnt-rust-target` before trusting a cold link; check the `.a` mtime. |
| Flip surfaces a *downstream* latent bug | §1589 | A flip that "advances the marker then crashes" is usually correct — it reached virgin code. Diagnose the crash site, not the flip. Grep flipped accessor shims for `-> … { NULL }`. |
| `.CRT$X*` init-array ordering | §1274 | Strong `#[no_mangle]`, not `global_asm .weak` (its PE weak-external perturbs section emission). |

---

## 8. Open questions / decisions before Phase D

1. **Global-state ABI model** (blocks G2/G3): export `_iob`/`__pioinfo`/locale globals as `#[no_mangle] static mut` with C-identical layout so mixed C/Rust consumers share one instance? Or keep the state in C and translate only the stateless logic around it? (Leaning: keep C-identical exported statics — the whole point is a drop-in.)
2. **Repack vs reuse** `libcntpr_rust.a` for user-mode (Phase B): reuse the same `.a` (kernel + user share it) or build a user-mode-scoped `crt_rust.a`? Repack avoids pulling kernel-only symbols into user-mode links; reuse is less cmake. Decide when the first `was replaced` batch tells us how much overlaps ntdll.
3. **ucrtbase priority** (Phase E): confirm nothing we plan to ship in the near-term roadmap links ucrtbase before investing — it may stay all-C indefinitely.
4. **undname** (§4 optional leaf): translate the demangler as a byte-diffed rlib, or leave it? Only worth it if a consumer surfaces.

---

## 9. Concrete first steps (Phase A, this lane)

1. Confirm a **green baseline** (build + `./vibeos-cli gate` on the untouched tree) so any regression is attributable.
2. Read `libcntpr.cmake:557-627` (the `rust_cnt` custom-target recipe) as the template.
3. In `dll/win32/msvcrt/CMakeLists.txt`, add an `add_custom_command`/`add_custom_target` that builds **`msvcrt_cnt_rust.a`** from `cnt-rust-lib` with **user-mode codegen** (i386 keeps SSE, `--features float`, `-C no-prepopulate-passes`).
4. Link it **before** `msvcrt_static` on the `msvcrt` target (`target_link_libraries(msvcrt msvcrt_cnt_rust.a msvcrt_static …)`), relying on lld first-match + `--allow-multiple-definition`.
5. Restrict the first flip to the **`mem` family** — i.e. don't yet let the Rust `.a` override anything else. (If the umbrella pulls in more symbols than we want to flip in wave 1, gate the extra `cabi` modules behind a cargo feature so wave-1 exports only `mem*`.)
6. `./vibeos-cli build --arch i386 --clean` (prove cold link, nuke the cnt-rust target dir first), diff `msvcrt.dll` exports vs baseline, then `./vibeos-cli gate` (×3, all arches).
7. Land the wave; repeat per family (Phase B). Report each ×3-green push.

---

### Cross-references
- Roadmap line: `docs/product/vibeos-roadmap.md` Tier 2.
- Origin analysis: `docs/engineering/archive/userland-wine-libs-audit.md` §89, §297–305 (the "defer/piecemeal, keep the asm" verdict) · `docs/engineering/archive/static-libs-translation-audit.md` §92 (msvcrtex floor).
- ucrtbase target scoping: `docs/product/api-compat-roadmap.md` §"Universal CRT".
- Cohab/link mechanics: `docs/engineering/rust-in-kernel.md` (HAL cohab pattern, cargo-cutover).
- The proven kernel-CRT wiring to copy: `sdk/lib/crt/libcntpr.cmake`.
- Field-manual recipes: §378, §384, §873, §1266–1279, §1346–1350, §1589 (see §7 table).

## 2026-07-12 (later) — RE-LANDED. Every item on the checklist below is done, and the gate that would have caught it now exists.

The revert was **correct** — mainline was red and the diagnosis (a *performance* regression, invisible to a behavioural harness) was right. Both root causes are fixed and each now has a permanent detector:

| | was | now | pinned by |
|---|---|---|---|
| `memset` (wave 2) | per-byte `write_volatile`; **8.0x slower** than the C | word-wise raw-pointer fill, vectorized — **0.88x** (faster than the C) | `--test perf` |
| `memcmp` (wave 3, found by the new gate) | byte-at-a-time `zip` core; **10.3x slower** | word-wise block compare — **1.02x** | `--test perf` |
| `_strnset` (wave 3) | unbounded NUL scan → OOB read → faulted in `mbcs.c`'s `_mbsnset` | bounded by `n` | `--test oob_guard` |

- `write_volatile` was a crutch against the §378 captures(none) DCE — which came in through the **slice** signature. Storing through a **raw pointer** stays off that path, and the class is toolchain-fixed on LLVM ≥ 21. `#![no_builtins]` is what actually keeps the fill loop from being lowered back into a self-call to `memset`.
- **`-C no-prepopulate-passes` is GONE from msvcrt's RUSTFLAGS.** It disables rustc's optimization pipeline outright — with it, `memset`/`memcpy`/`memcmp` compile to *a function call per byte*. It was the real multiplier on the regression. (The class-guard it provides is toolchain-fixed; ud2 + sacred-primitive scans + the ×3 smoke gate cover the residual risk. Re-check `tools/scan_captures.py` after any rustc/LLVM bump.)
- Verified in the **shipped** windows-gnu build, not just the host bench: `memset`/`memcpy` contain **no calls out** (no self-recursion) and are genuinely vectorized (xmm/ymm).

**The lane now runs three orthogonal harnesses, because each is blind to what the others catch:**

| harness | asks | the bug it caught |
|---|---|---|
| `classify` | wrong **values**? | the signed-vs-unsigned-char contract (would have corrupted non-ASCII compares) |
| `oob_guard` | right answer, wrong **memory**? | `_strnset`'s unbounded scan (§9.61) |
| `perf` | right answer, too **slowly**? | `memset` 8x / `memcmp` 10x (this section) |

A green diff table is not a safety proof, and it never was a performance proof. **Run all three before any flip.**

<details>
<summary>Original revert note (2026-07-12) — kept for the record</summary>

## 2026-07-12 — WAVE 2 REVERTED ON MASTER (net-suite regression). Re-land needs a PERF gate.

`cfdfd3c841a` (wave 2 — the 23 "MATCH" flips) **broke `tests/qemu/test_net.py` on mainline**
and was reverted (code only; this plan doc kept). Not a correctness bug — a **performance**
one, which is exactly the class the differential harness is blind to.

**Symptom.** `test_dns_resolve` + `test_usermode_winsock_roundtrip` fail. On serial,
`nettest.exe` prints `[NETTEST] usermode winsock proof starting` → `[NETTEST] WSAStartup OK`
→ then nothing (marker `[NETTEST] usermode winsock TCP echo OK` never arrives, 120s timeout).
The kernel netstack itself is fine — all 13 in-kernel TDI/UDP/ICMP tests still pass, and
`[NETSTACK] dns OK` does eventually print, just outside the captured boot window.

**Bisect (each a real boot, amd64):**
| tree | net |
|---|---|
| pre-wave-2 | 15/15 green |
| + wave 2 | 2 failed / 13 passed (208s) |
| + wave 2, unrelated kernel commit reverted | 2 failed / 13 passed (control — not that lane) |
| + wave 2 **reverted** | **15/15 green (71s)** |

**Root candidate — semantics-identical but orders of magnitude slower.** The harness grades
`cnt-rust` vs the C bodies on *behaviour*, so a body that returns the right bytes always scores
MATCH no matter how slowly it gets there. But these are `msvcrt.dll`'s `memcpy`/`memset` —
served to **every user-mode caller in the system**:
- `memset` is a **per-byte `core::ptr::write_volatile` loop**. `write_volatile` is *unvectorizable
  by construction* — the compiler may not merge, widen, or elide the stores. That is a byte-at-a-
  time `rep stosb` at best, ~1-2 orders of magnitude off a real `memset`.
- `memcpy` delegates to `raw_move`; check whether that lowers to a real `memcpy`/`memmove` or an
  open-coded loop.

The 71s → 208s wall-clock blow-up on the same 15 boots is consistent with a system-wide CRT
slowdown, and a 120s per-marker timeout is exactly what a big user-mode buffer op would blow.

**Before re-landing:**
1. **Benchmark, don't just diff.** Add a throughput case to `msvcrt-difftest` (bytes/sec vs the C
   body, at 16B / 4KB / 1MB). A flip that is >~2x slower than the C it replaces should FAIL the
   harness, not pass it.
2. **Drop `write_volatile` from `memset`** — nothing here needs volatile semantics; it exists to
   defeat a loop-idiom miscompile, but the cure is `black_box` on the *index* (as `cstr_slice`
   already does), not volatile on every store.
3. **Prefer delegating to the intrinsic**: `core::ptr::copy_nonoverlapping` / `write_bytes` lower
   to the real `memcpy`/`memset`.
4. Re-gate with the net suite (`vibeos-cli gate` runs it; do not `--skip-net` a CRT-wide flip).

*(Cross-lane note: this is the same trap the kernel's `kmsafecopy.c` hit the same day — a
hand-rolled `while (Len--) *Dst++ = *Src++;` compiled to a genuine byte loop; caught by
disassembling the shim. Field manual §9.59.)*

</details>

---

## Wave 6 — COMPLETE (2026-07-12). 24 new flips; msvcrt.dll is **82 functions of Rust**.

Wave 6 split three ways. **6a was worth more than 6b and 6c combined**, and it was not on the
plan: it started as "retro-grade the old flips through wave 5's new errno oracle" and turned
into "the oracle itself was miscompiled."

### 6a — the oracle was lying, and it was hiding 3 shipped bugs (field manual §9.66)

The reference C is compiled for the host with its symbols renamed `ref_*` by a force-included
prelude that lands **before** the system headers — deliberately, so glibc declares the renamed
symbols and we inherit prototypes for free. But a declaration carries **attributes** too, and
glibc's `__nonnull` / `__attribute_pure__` are promises about *glibc's* implementation:

- `__nonnull` let the optimizer **delete msvcrt's `if(!str) return 0;`** from `ref_atoi`.
  Verified in the disassembly. The reference *segfaulted* on an input the real DLL handles.
- `__attribute_pure__` says "no side effects" about a family that **sets errno** — licensing
  the compiler to elide the very calls the errno oracle inspects. That one never crashes.

Fixed by neutering glibc's attribute macros before its headers declare anything (build.rs
"stage 1.5"). `tests/ref_integrity.rs` is the permanent tripwire — a test whose subject is the
*oracle*, negative-controlled.

Then, with a trustworthy oracle, the **already-shipped** integer-parse family was re-run
through it. Three bugs, all live on master:

| fn | what shipped | msvcrt actually |
|---|---|---|
| `_atoi64` | the KERNEL's **wrapping** parser | **saturates** at `_I64_MAX` + `ERANGE` |
| `strtol` | BSD `parse_long_bsd` — no errno at all | `ERANGE` on overflow; `EINVAL`+handler on NULL/bad base; leaves `*endptr` ALONE on a bad base |
| `strtoul` | no errno/handler | same front door |

`_atoi64` was a **value** divergence, not merely an errno one — and `classify` had missed it
because `gen_str` essentially never emits a 19-digit run. **A fuzzer only falsifies what it
varies.** `gen_num_str` now generates the boundaries on purpose.

Fix shape (this is the lane's thesis, stated sharply): **cnt-rust supplies the ALGORITHM;
msvcrt-crt-rust supplies msvcrt's SEMANTICS.** `parse_i64_clamped` already *was* `_strtoi64_l`
faithfully — it just never reported that it saturated, which the caller cannot re-derive from
the value. So the engine now says so (`overflowed`), and `strtoi64_contract()` adds the
`CHECK_PMT` front door + `ERANGE` at the msvcrt boundary. cnt-rust stays kernel-pure.

### 6b — the radix `_s` family: ten functions, FOUR contracts (11 flips)

`_itoa_s _ltoa_s _i64toa_s _ui64toa_s _ultoa_s _itow_s _ltow_s _i64tow_s _ui64tow_s _ultow_s`
+ `_strtoi64`.

They look like one function with ten signatures. They are not:

| group | too-small buffer | handler |
|---|---|---|
| A/B (`_ltoa_s`, `_i64toa_s`, …) | `ERANGE` + the digits written **BACKWARDS** into the caller's buffer | fires |
| C (`_ui64toa_s`, `_ui64tow_s`) | **`EINVAL`**, buffer untouched | fires |
| D (`_ultoa_s`, `_ultow_s`) | `ERANGE` + residue | **never** — hand-rolled `*_errno()` |

Group D is *why 6a had to start grading effect 4*: on a bad radix `_ultoa_s` and `_ltoa_s` are
byte-, return-code- AND errno-identical. The handler bit is the only thing separating them.

The **reversed residue** is bug-for-bug with Microsoft — a clean `str[0]=0; return ERANGE;` is
string-identical and byte-different. `classify` compares the FULL buffer.

### 6c — the wide `_s` family + tokenizers (11 flips)

`wcscpy_s wcsncpy_s wcscat_s wcsncat_s _wcsnset_s _wcsset_s wcsncpy wcsnlen strtok_s wcstok_s
_strtoui64`. Four writers, four *different* handler contracts — proved by the coverage
arithmetic, not by reading:

```
wcscpy_s   EINVAL=1396 ERANGE=7778 handler=9174  (=1396+7778: every error)
wcsncpy_s  EINVAL=632  ERANGE=3366 handler=632   (EINVALs only: ERANGE arm SILENT)
wcscat_s   EINVAL=741  ERANGE=7872 handler=0     (NEVER)
wcsncat_s  EINVAL=3184 ERANGE=2053 handler=5237  (=3184+2053: everything)
```

### The harness got harder (this is the durable output)

- **`ref_integrity.rs`** — audits the ORACLE. New; negative-controlled.
- **Coverage requirements** — `check_buf_fx(name, &["ok","EINVAL","ERANGE","handler"], …)` fails
  a row as **UNEXERCISED CONTRACT** if the reference never produced those outcomes. All 11
  wave-6b rows passed on the first run, which is exactly when a green is least trustworthy.
- **Rename-list drift assertion** — build.rs scrapes both reference TUs for every `(MSVCRT.@)`
  export they define and fails if one is missing from the rename lists. It had already drifted
  (`memmove`, `_mbsdup`, `swprintf`, `_scwprintf_p`, `iswalpha_l` were compiled under their
  PLAIN names, escaping the anti-circularity namespace).
- **`gen_num_str`** — boundary numerals (`_I64_MAX`, `±ULONG_MAX`, 19-digit runs, `0x`/octal).

### What wave 6 REFUSED to flip

- **`memmove`** — *no oracle*. `refstub/msvcrt.h:185` routes it into the private
  `msvcrt_ref_*` namespace (the thing that stops the reference binding to the subject), so the
  compiled reference does not define an msvcrt `memmove`. The body would have been three lines
  (`h::raw_move` already ships under `memcpy`). **Nothing flips without a differential proof,
  and the rule doesn't get suspended because a flip looks easy.** Recorded as
  `NAMESPACED_SYMBOLS` in build.rs. *Owed:* let the refstub expose a namespaced symbol as a
  `ref_*` oracle without opening the namespace to the subject.
- **`strtok` / `wcstok`** — reach into msvcrt's `thread_data_t`. Getting at that field from
  Rust means hardcoding an offset into a struct we don't own (the magic-offset class —
  `scan_magic_offsets`, same shape as the §9.41 CLIENTINFO bug). Wants a real TLS story.
  `strtok_s`/`wcstok_s` take an explicit context and are flipped.

### Known + bounded: the locale caveat (NOT new debt)

The integer parsers skip whitespace with `_isspace_l(c, NULL)` and fold case with
`_tolower_l(c, NULL)` — the thread's **current** locale. We implement the "C" locale (ASCII),
which is what msvcrt.dll starts in; `classify` fuzzes the full 0x00–0xFF byte range against the
genuine C to prove C-locale equivalence. A non-C `LC_CTYPE` could diverge. That is Phase D, and
the already-shipped `strtol` carried the identical caveat before wave 6 — this is *documented*
now, not *introduced*.

Discovered en route: **the WIDE integer parsers are locale-bound where the narrow ones are
not** (`_wcstoi64_l` calls `_iswspace_l`; `_strtoi64_l` calls `_isspace_l` — which lives in
ctype.c and is C-locale ASCII in practice). `wcstol`/`wcstoul`/`_wtoi`/`_wtol`/`_wcstoi64` are
therefore **Phase D**, not a symmetric freebie. Nearly flipped them on a symmetry argument.

### Next

- **Wave 7 candidates:** `_stricmp`/`_strnicmp`/`_memicmp` (case-folding — verify the locale
  boundary FIRST, the way wave 6 did; do not assume ASCII), `_strlwr_s`/`_strupr_s`.
- **Phase C (printf)** is the big one and now includes the whole float family: `_ecvt`/`_fcvt`/
  `_gcvt` are literally `_snprintf` calls, and `strtod` needs msvcrt's own bignum (the wave-5
  probe measured cnt-rust's parser 14 ULPs off — it stays in the tree as an executable
  DO-NOT-FLIP record).

---

## Waves 7–9 (2026-07-13). +37 flips. msvcrt.dll is **119 functions of Rust**.

Three waves, three bugs the harness caught that reading could not. That is the
headline: by wave 7 I had read a lot of msvcrt C and was getting confident, and
confidence is precisely when the differential oracle starts earning its keep.

### Wave 7 — the narrow case-insensitive family (5 flips)

`_stricmp _strnicmp _memicmp _strlwr_s _strupr_s`

**Locale checked FIRST**, per wave 6's lesson. Each consults
`locinfo->lc_handle[LC_CTYPE]` and takes an inline ASCII fast path when unset — the
"C" locale msvcrt.dll starts in. Same bounded caveat the already-shipped
`_strlwr`/`_strupr` carry. No new debt.

**The bug:**

```c
if ((c1 = *s1++) >= 'A' && c1 <= 'Z')
    c1 -= 'A' - 'a';                    /* 'A'-'a' == -32, so this ADDS 32 */
```

Transcribed the `-=` literally as `c1 -= 32`. That maps `'I'` (73) to `')'` (41).
classify convicted it on the first run with the exact arithmetic (`cnt=-131`, which
is `(u8)41 - (u8)172`). `_memicmp` passed the same fuzz because its helper spells
the fold as `c + 32` — two transcriptions of one rule, one wrong, and only the
differential test knew which.

Also: `_strnicmp` and `_memicmp` are NOT the same comparison at different bounds.
The first folds on the sign-extended `char` and returns the `(unsigned char)`
difference; the second folds via `_tolower_l` (non-ASCII passes THROUGH) and returns
that difference raw. On high bytes they return different numbers.

### Wave 8 — the WIDE case-insensitive family (6 flips)

`_wcsicmp _wcsnicmp _wcslwr _wcsupr _wcslwr_s _wcsupr_s`

Deliberately **not** transcribed from wave 7. Three things differ:

1. **No C-locale fast path in the comparer.** `_wcsicmp_l` always calls
   `_towlower_l` — which is fine only because `_towlower_l` has the fast path
   *inside* it. Checked, not assumed. Had it gone through `LCMapStringW`, these
   would be Phase D.
2. **The comparers return the RAW u16 diff.** `wcscmp` (wave 4) CLAMPS. Same file,
   same shape, different contract — and wave 4 already got bitten by exactly this
   pairwise assumption.
3. **The folders MUTATE BEFORE THEY FAIL.** `_strlwr_s_l` scans for the terminator
   first and errors without touching the buffer. `_wcslwr_s_l` folds AS it scans, so
   a string with no terminator inside `n` comes back with its first `n` chars
   already folded, and only then `str[0] = 0`. Its front door also zeroes `*str`
   where the narrow one does not.

### Wave 9 — the wide ctype family (26 flips)

`iswctype` + `iswalnum/alpha/cntrl/digit/graph/lower/print/punct/space/upper/xdigit/blank`,
each with its `_l` variant.

These **look** locale-bound and are not: `_iswctype_l` takes a `_locale_t` and never
uses it. No `locinfo->` field to reach into, so no magic-offset problem, and the
whole family — `_l` variants included — is flippable.

Better: it reads msvcrt's own `MSVCRT__pwctype` global and calls the real
`GetStringTypeW`, so it is **locale-CORRECT, not merely C-locale-correct**. It
inherits msvcrt's locale state instead of modelling it. This is the first family in
the lane with no locale caveat at all.

**EXHAUSTIVE, not fuzzed.** The domain is 65,536 wide with three regimes the C
treats differently (WEOF; the `<256` table; the `>=256` NLS path). Exhaustion proves
all three. It earned that instantly:

```
iswblank  DIVERGE   wc=0x0020  cnt=64  msvcrt=1
```

```c
return wc == '\t' || _iswctype_l( wc, _BLANK, locale );
```

C's `||` **normalises to 0/1** — so `iswblank` is the ONLY member of the family that
returns a boolean. Every other `isw*` returns the raw masked bits (`iswspace(' ')`
is 8, not 1). A fuzz over "interesting wide characters" might well not have included
plain space.

`towupper`/`towlower` are NOT flipped: `_towlower_l` reads
`locinfo->lc_handle[LC_CTYPE]` and calls `LCMapStringW`. That field IS an offset into
a struct we don't own. Phase D.

### Also fixed: a lint that was breaking everyone's gate

`tools/scan_nightly_surface.py` (landed on master that morning) `rglob`'d
`sdk/lib/rust/target/` — build output — and only filtered the *results*. The gate
runs per-arch lints concurrently with the other arches' builds, so a
`target/<triple>` vanished mid-walk and the tool died with `FileNotFoundError`,
reported as "NEW lint findings". Pruned during traversal instead. Not our lane, but
it was in the way, and it was breaking gates for whoever hit it next.

### Where the lane stands

**119 of msvcrt.dll's functions are Rust.** The pure floor of `string.c`/`wcs.c` is
essentially exhausted. What remains is gated on two things, neither of which is
effort:

- **Phase C — printf.** The whole float family rides on it (`_ecvt`/`_fcvt`/`_gcvt`
  are literally `_snprintf` calls; `strtod` needs msvcrt's bignum — the wave-5 probe
  measured cnt-rust's parser 14 ULPs off and stays in the tree as an executable
  DO-NOT-FLIP record).
- **Phase D — locale.** `towupper`/`towlower`, the collation family
  (`strcoll`/`wcscoll`/`strxfrm`), the multibyte conversions, the wide integer
  parsers (locale-bound via `_iswspace_l` where the narrow ones are not), and every
  `_l` variant that actually *reads* its locale argument. All of them want the same
  thing: a way to reach `locinfo` fields without hardcoding struct offsets.

The next wave that isn't Phase C or D is small: `_strdup`/`_wcsdup`/`_mbsdup` (alloc
+ copy). Everything else of substance is behind those two doors.

---

## Wave 10 (2026-07-13) — the LOCALE DOOR. +50 flips (msvcrt.dll is **170 functions of Rust**), and 6 shipped bugs found in already-landed code.

Phase D was recorded as blocked on one thing: *"a way to reach `locinfo` fields without hardcoding
struct offsets."* Wave 10 built that, and it turned out to unlock rather more than the plan expected —
including the discovery that **the harness's own convenience stubs had been the oracle for four waves**.

### The reframe: the "C-locale blind spot" was never about the FUNCTION

The lane had recorded the whole locale family as un-flippable, because the harness pins the "C" locale
(§9.60) and would report MATCH for a `toupper` that is only C-locale-correct. That is true — **of a
locale-INDEPENDENT implementation.** It was never true of a faithful one:

```c
int CDECL _toupper_l(int c, _locale_t locale) {
    if((unsigned)c < 256) {
        if(locinfo->pctype[c] & _LEADBYTE) return c;
        return locinfo->pcumap[c];          /* <-- the LOCALE'S OWN table */
```

A Rust body that reads `locinfo->pcumap[c]` — *the same table* — inherits the C's locale-correctness
**by construction**: the locale-dependence lives in the DATA, `setlocale` repopulates it, and both
bodies follow it. The blind spot forbids hardcoding ASCII. It never forbade reading the real table.
We simply had no way to reach the table without hand-rolling struct offsets — the
`scan_magic_offsets` bug class that cost this repo the `KTHREAD.Teb` hunt and the amd64
`TEB.GdiBatchCount`-at-`0x74` read that killed explorer (§9.41).

### `msvcrt-sys` — the door (and it caught a bug on day one)

`sdk/lib/rust/msvcrt-sys/`: bindgen over msvcrt's internal headers, `offset_of!` everywhere, no
arithmetic. Two properties worth copying:

* **The flags come from `compile_commands.json`, not a hand-copied list.** msvcrt is not built with the
  default include path (its CMakeLists *removes* `sdk/include/crt`, substitutes its own, prepends
  `sdk/include/wine`, sets `_MSVCR_VER=0`). A wrong `-D` makes bindgen emit a *different layout*, and a
  machine-generated wrong offset is worse than a hand-rolled one because it looks trustworthy. So we read
  the real compile command for `dll/win32/msvcrt/ctype.c` out of the build itself.
* **The layout is guarded on two sides by two compilers, on every build** — `layout_<arch>.rs` (rustc) and
  `dll/win32/msvcrt/rust_layout_assert.c` (**clang, with msvcrt's real flags, ×3 arches**), both pinned to
  the same literals. Both negative-controlled. It earned its keep immediately: bindgen had recorded
  `__msvcrt_ulong` as `c_ulong`, which is 32-bit on Windows but **64-bit on the linux host that builds the
  difftest harness** — every field after `lc_handle` would have shifted, *in the oracle*. See §9.74.

### What it found: six functions wrong on master (§9.73)

Making `ctype.c` a real reference TU meant deleting `refimpl.c`'s three convenience stubs for
`_isspace_l`/`_tolower_l`/`_toupper_l`. Those stubs were **the oracle**, and they were an ASCII fold.
The instant the reference started grading itself, six already-shipped functions went DIVERGE:

`_atoi64` · `strtol` · `strtoul` · `_strtoi64` · `_strtoui64` · `_memicmp`

All six because msvcrt hands `_tolower_l` a **raw signed `char`** (`string.c:1651` — no cast, while the
`_isspace_l` two lines above it *does* cast), so a byte ≥ 0x80 sign-extends negative, misses the table,
and takes the NLS arm — **which sets `errno = EILSEQ`**. The values were right; the side effect was
missing. Fixed by *transcribing* the parsers so they call the real `_tolower_l`/`_isspace_l` at the same
points, rather than bolting msvcrt's contract onto `cnt_rust::radix::parse_i64_clamped` (which cannot
express a side effect). **This is the wave-5 float lesson again, in a place we had not looked.**

Also fixed: the harness had silently inherited **glibc's `EILSEQ` (84)** where msvcrt says **42**.
`EINVAL`/`ERANGE` coincide between the two OSes, which is exactly why nobody noticed.

### Flipped (50)

* **ctype/case (44)** — the whole of `ctype.c`: `_isctype`/`_isctype_l`, the 13 `is*`/`_is*_l` pairs,
  `ispunct`, `isblank`/`_isblank_l`, `__isascii`/`__toascii`/`iswascii`/`__iscsym`/`__iscsymf`/
  `__iswcsym`/`__iswcsymf`, `toupper`/`tolower`/`_toupper`/`_tolower`/`_toupper_l`/`_tolower_l`,
  `__pctype_func`/`__p__pctype`/`__pwctype_func`/`__p__pwctype`.
* **thread_data tokenizers (2)** — `strtok`, `wcstok`. **Refused in wave 6c** purely for lack of the
  struct offset; `msvcrt-sys` removed the obstacle.
* **the six re-transcriptions** above.

### What the exhaustive sweep caught

* **`isblank` returns 0/1; `_isblank_l` returns the MASK (64).** Three lines apart in the same file:
  the plain form is `return c == '\t' || _isctype(c, _BLANK);` and C's `||` **normalises**. It is the
  only member of the narrow ctype family that returns a boolean. This is *precisely* the trap wave 9 hit
  on the wide twin (`iswblank`) — and I walked into it again. Caught only because the sweep is exhaustive:
  the divergence exists at exactly one input.
* **`_toupper`/`_tolower` are `c ∓ 0x20`, UNCONDITIONALLY** (`/* sic */` in the C). No range check, no
  table, no locale: `_toupper('0')` is `0x10`. A "reasonable" implementation would be a bug.
* **`toupper` is not `_toupper_l(c, NULL)`** — it short-circuits on the global `initial_locale`.

### Verified

`classify` **125 MATCH / 0 SIGN-ONLY / 1 DIVERGE** (the `strtod` probe, which is the expected
DO-NOT-FLIP record) · `oob_guard`, `perf`, `link`, `ref_integrity` all green · builds ×3 arches, all
lints incl. `scan_magic_offsets` · PE export-RVA proves the Rust bodies won the link on every arch
(wave-10 exports at `0x001d30–0x008480`; the deliberately-refused `memmove`/`strcoll`/`_strdup`/`strxfrm`
still C at `0x072000+`).

### Still open, and now sharply defined

**Waves 7 and 8 shipped locale-INDEPENDENT implementations of locale-DEPENDENT functions.**
`_strnicmp_l` has *two* paths:

```c
if(!locinfo->lc_handle[LC_CTYPE]) { ...inline ASCII fold...; return (unsigned char)c1 - (unsigned char)c2; }
do { c1 = _tolower_l((unsigned char)*s1++, locale); ... } while(...);   /* a REAL locale */
```

Our Rust only implements path 1. It is correct in the "C" locale (all the harness tests) and wrong the
moment `setlocale` is called. Same for `_stricmp`, `_wcsicmp`, `_wcsnicmp`, `_strlwr_s`/`_strupr_s` and
the wide twins. **This is the documented §9.60 blind spot, shipped** — and the fix is not the hard part;
the hard part is that a harness pinned to the C locale *cannot grade path 2*, so adding it would be a
green from an unexercised branch, which is not a green.

**Owed (wave 11): a non-C-locale mode for the harness.** Plant a non-zero `lc_handle[LC_CTYPE]`, and grade
path 2 against the shared NLS stubs — the same argument that already makes `__crtLCMapStringA` gradeable
(both sides call the same stub, so the comparison isolates the Rust's *logic*, and only the NLS *data* is
out of scope, correctly, because that is kernel32's job). That closes the lane's oldest blind spot
structurally rather than by promise.

---

## Wave 11 (2026-07-14) — §9.60 CLOSED. The harness now has a non-"C"-locale mode, and it convicted waves 7–8 the moment it worked.

Wave 10 ended by naming the lane's oldest blind spot precisely: **waves 7 and 8 shipped
locale-INDEPENDENT bodies for locale-DEPENDENT functions.** `_strnicmp_l` has two paths —

```c
if(!locinfo->lc_handle[LC_CTYPE]) { ...inline ASCII fold...; return (unsigned char)c1 - (unsigned char)c2; }
do { c1 = _tolower_l((unsigned char)*s1++, locale); ... } while(...);   return c1 - c2;
```

— and we implemented only the first. Correct in the "C" locale (all the harness ever tested),
wrong the moment `setlocale` is called. Note the two paths do not even agree on their *return
convention*.

### The false green, and why it matters more than the fix

The obvious move — plant a non-zero `lc_handle[LC_CTYPE]` and re-run the rows — **did not work,
and reported that it did.** Sabotaging `_strnicmp` to always take path 1 moved not one row.

Because path 2 folds through `_tolower_l`, which reads `locinfo->pclmap[]` — and that was still
the **C locale's** map, i.e. an ASCII fold. Both paths computed the same answer on every input, so
a path-1-only implementation was *indistinguishable from a correct one*. The branch was being
exercised and proved nothing.

**It is not enough to exercise the branch. The branch has to be able to produce a different
answer.** So the mode also swaps the ctype tables (`pclmap`/`pcumap` gain the Latin-1 case pairs,
`0xC0–0xDE ↔ 0xE0–0xFE`), and `LCMapStringW` obeys the same rule. Now path 1 leaves `0xE9` alone
while path 2 maps it to `0xC9`.

This is §9.73 in miniature, again: **shared data cannot be caught differentially**, so the tables
are one copy (refimpl's), aimed at by both sides — and the NLS entry points (`LCMapStringA/W`,
`CompareStringA/W`, `GetStringTypeW`) are now **un-namespaced and shared**, because kernel32 is the
*environment*, never code under test. What the mode grades is the Rust's LOGIC around the NLS call.
Windows' real NLS *data* stays out of scope, correctly — that is kernel32's job.

### Negative control (the only reason to believe any of this)

| row | subject sabotaged to path-1-only | subject correct |
|---|---|---|
| `_strnicmp` (C locale) | **MATCH** ✓ | MATCH |
| `_strnicmp [NLS]` | **DIVERGE** ✗ | MATCH |

The C-locale row still passes under sabotage — correctly, because path 1 *is* right there. The two
rows test genuinely different things. That is what a working instrument looks like.

### What it caught immediately

**`_wcsicmp [NLS]` DIVERGE** (`cnt=94 msvcrt=126`) — `_wcsicmp` has its own fold loop rather than
delegating to `wcsnicmp_core`, so fixing the core had missed it. Exactly the class of bug the mode
exists for, found the first time it ran honestly.

### Flipped / fixed (+4 flips, 174 total)

* **NEW:** `_towlower_l`, `_towupper_l`, `towlower`, `towupper` — the wide case mappers the whole
  wide family funnels through (`wint_t` is **16-bit** in msvcrt, so `WEOF == 0xFFFF`). Exhaustive
  over all 65,536 inputs, in **both** locale modes.
* **path 2 added** to `_stricmp`, `_strnicmp`, `_wcsicmp`, `_wcsnicmp`, `_strlwr_s`, `_strupr_s`,
  `_wcslwr_s`, `_wcsupr_s`. `_wcslwr_s_l` has no `lc_handle` branch of its own — it folds through
  `_towlower_l` and inherits locale-correctness, so flipping the mapper fixed the family.

### Verified

`classify` **139 MATCH / 0 SIGN-ONLY / 1 DIVERGE** (the `strtod` probe — the expected DO-NOT-FLIP
record) · `oob_guard` / `perf` / `link` / `ref_integrity` green · builds ×3 arches, all lints.

---

## Wave 12 (2026-07-14) — the collation family. +20 flips (**194 total**). Only possible because wave 11 exists.

`strcoll` · `_stricoll` · `_strncoll` · `_strnicoll` · `strxfrm` (+ `_l` forms) and the five wide
twins `wcscoll` · `_wcsicoll` · `_wcsncoll` · `_wcsnicoll` · `wcsxfrm`.

Every one branches on `lc_handle[LC_COLLATE]`:

```c
if(!locinfo->lc_handle[LC_COLLATE])
    return strcmp(str1, str2);                          /* path 1 — the "C" locale */
return CompareStringA(locinfo->lc_handle[LC_COLLATE], SORT_STRINGSORT,
          str1, -1, str2, -1) - CSTR_EQUAL;             /* path 2 */
```

Before wave 11's NLS mode the harness could only see path 1, so flipping these would have been §9.60
all over again. Now both arms are graded, negative-controlled.

### What the harness caught (three real bugs, all in the first run)

1. **`strxfrm` / `wcsxfrm` were missing `errno`** — both set `EILSEQ` (LCMapString refused) and
   `ERANGE` (key longer than the buffer). I never saw those lines: I read the C through a
   comment-stripping `grep -vE "^\s*\*"`, which **also eats `*_errno() = ERANGE;`**. Second time that
   filter has eaten load-bearing C (§9.77).
2. **`_strncoll` inherited `strncmp`'s two contracts.** Its C-locale path is literally
   `return strncmp(...)`, and `strncmp` has a `#if defined(_WIN64)` (i386 → raw byte-diff;
   amd64/arm64 → clamp to ±1/0). The host reference compiles the **i386** arm while our Rust selects
   the **amd64** one, so the row had to model it — exactly as the existing `strncmp[_WIN64]` row does.
   classify's header already warned "any NEW flip must be grepped for `_WIN64` first"; I hadn't.
3. `_wcsxfrm_l` widens `LCMapStringW`'s **byte** sort-key in place **walking backwards** so it never
   clobbers a byte it hasn't read. Transcribe the direction, not just the effect.

### Verified

`classify` **159 MATCH / 0 SIGN-ONLY / 1 DIVERGE** (the `strtod` probe) · all six instruments green ·
builds ×3.

---

## Wave 13 (2026-07-14) — the FLOAT PARSER. msvcrt's own bignum, PORTED. +15 flips (all three arches), the lane's first true translation.

**This is the wave that wave 5 said we could not have.** Wave 5 cancelled "flip `strtod` from
cnt-rust" after a probe showed cnt-rust's DJ-Delorie accumulator disagreeing with msvcrt's bignum by
**14 ULPs**. The recorded honest path was: *port msvcrt's own machinery* — a translation, not an
assembly. Wave 13 does exactly that. `sdk/lib/rust/msvcrt-crt-rust/src/fpnum.rs` is a faithful
transcription of `dll/win32/msvcrt/bnum.h` + `string.c`'s `fpnum_parse`/`fpnum_parse_bnum`/
`fpnum_double`/`fpnum_ldouble` — the base-10 circular-buffer bignum, correctly rounded.

**Flipped (15, all three arches):** `strtod` · `_strtod_l` · `atof` · `_atof_l` · `_atoflt_l` ·
`_atodbl` · `_atodbl_l` · `wcstod` · `_wcstod_l` · `_wtof` · `_wtof_l` · `_atoldbl` · `_atoldbl_l` ·
`__STRINGTOLD` · `__STRINGTOLD_L` (+ the shared engine `fpnum_parse`/`fpnum_double`/`fpnum_ldouble`,
kept internal — not `#[no_mangle]`, so the C engine still serves its C callers).

### The oracle is the strongest the lane has ever had: an exact 64-bit BIT PATTERN

`string.c`/`wcs.c` are already reference TUs, so `ref_strtod` is msvcrt's genuine bignum compiled for
the host. `classify` compares the raw `u64` of every returned `double` (`-0.0 != 0.0` matters), the
endptr displacement, errno, and the invalid-parameter count. There is no "close enough": a 1-ULP slip
is a DIVERGE. **All 13 public rows MATCH, plus `_atoldbl`/`__STRINGTOLD_L` byte-exact on a
16-byte overrun-sentinel buffer.** The wave-5 `strtod [PROBE]` stays the lone red — as designed;
it now records that the *cnt-rust route* is DIVERGE while the *ported engine* is MATCH.

### Everything that mattered was found by trying to break the green (field manual §9.79)

1. **A verbatim Wine bug that is provably INERT.** `fpnum_parse_bnum`'s sticky-scan reads `data[b]`
   instead of `data[i]` — a real upstream bug. Transcribed verbatim; the negative control ("fix" it,
   demand DIVERGE) **did not bite**. That is the signature of an unverified transcription, so I did not
   assume — I built an exact-halfway-**tie** generator (`gen_tie`; random digits are never ties, so the
   whole `FP_ROUND_EVEN` arm was dead in the corpus), still no bite, then **instrumented reachability**
   and proved the discriminating state is *structurally unreachable* (a `LIMB_MAX/2` limb can only sit
   at `b`, forcing `b==e-4` and an empty scan range; whenever the scan runs, `data[b]≠0` — 0 violations
   in 43,712 parses — so both spellings compute the same round). Kept the bug to match msvcrt's bytes;
   the tie-corpus stays as a live tripwire.
2. **The signed-char → WEOF trap, load-bearing.** `strtod_str_get` widens a *signed* `char`, so byte
   `0xFF` becomes `0xFFFF == WEOF`: the parser sees EOF at a byte it already stepped past, and the
   endptr lands one too far. Reproduced with `*p as i8 as i16 as u16`; the negative control reddens
   exactly the endptr-checking rows.
3. **A harness blind spot: it could not see errno being CLEARED.** `strtod_helper` clears errno on
   entry; the wide `_wcstod_l` does not — an observable difference. `Effects` **zeroed errno before
   each call**, so a redundant clear was invisible (sabotage moved 0 rows). Fixed by seeding a
   **non-zero sentinel** (`0x5A5A`) instead of 0; now the sabotage correctly reddens the whole wide
   family while `strtod` stays green. Third variation of §9.73 in this lane: *the harness's own setup
   is part of the oracle.*

### i386 too — via x87 `ST(0)` (I first gated it out on a WRONG premise; §9.79 point 4)

i386 msvcrt is `-march=pentium` (pre-SSE) and the i686 user target is `-sse,-sse2`. I initially read
rustc's "SIMD vector type `f64` … requires the `sse` target feature" as "`extern "C" fn -> f64` is
impossible here" and gated the family `#[cfg(not(target_arch = "x86"))]`. **That was backwards, and
review caught it:** cnt-rust (`cabi/strtod.rs`, `core_rs/floatcvt.rs`) already ships `extern "C" -> f64`
on no-SSE i386 — the C ABI returns `double` in x87 `ST(0)`, no SSE. What actually hard-errors is a
**Rust-ABI** `-> f64` and the SSE-pulling `f64`/`f32` std methods (`is_infinite`/`from_bits`). Fix = three
ABI-portable changes in `fpnum.rs`: mark the one internal `f64`-returning helper `extern "C"`; replace
`is_infinite()` with memory bit-tests (`f64_is_inf`/`f32_is_inf`); replace `f64::from_bits` with a
`store_bits` that writes the `u64` pattern into the destination. None needs reverting if vibeos ever drops
the pre-SSE baseline (it targets Vista-era+, where SSE2 is universal — a candidate simplification, its own
lane). **The family now flips on all three arches.**

### New guardians (the layout/scalar drift defense grew)

`msvcrt-sys` now guards the float parser's two-hop reach `locinfo → lconv → decimal_point` on both
compilers (a genuinely arch-varying offset: `lconv` at 188 on i386, 296 on amd64/arm64 — a hand-number
would have been a §9.41-class bug), and `rust_layout_assert.c` gained scalar pins for the `<float.h>`
exponent limits, `ERANGE`/`_OVERFLOW`/`_UNDERFLOW`, **`char` must be signed** (the WEOF trap depends on
it), and `sizeof(MSVCRT__LDOUBLE)==10` vs `WINE_BROKEN_LDOUBLE==12` (the deliberate 10-of-12-byte copy
that dodges ReactOS's documented overrun). `ref_integrity` cross-checks the new `lconv` offsets subject
vs bindgen.

### Verified

`classify` **172 MATCH / 0 SIGN-ONLY / 1 DIVERGE** (the expected `strtod` probe) · all six instruments
green · three negative controls run (one proved the Wine bug inert, one caught the signed-char trap, one
exposed+fixed the errno-clear blind spot) · `vibeos-cli check` green · builds ×3. **msvcrt.dll is now
209 functions of Rust on all three arches.**

### Next, after wave 13

**The big door is now `printf` (Phase C), and wave 13 unlocked half of it.** msvcrt's float
FORMATTER (`%f`/`%e`/`%g`, and therefore `_ecvt`/`_fcvt`/`_gcvt`, which are literally `_snprintf`
calls) rides on the SAME `bnum.h` bignum this wave ported for the parser — so the hard numeric core
is now Rust and proven. What remains for printf is the `printf.h` templated ENGINE itself
(~1300 lines, instantiated narrow `_a` + wide `_w`): the format-string state machine, flags/width/
precision, positional params, and the digit-generation glue (`pf_output_fp`/`pf_output_hex_fp`).
It is hand-transcribable exactly as `fpnum.rs` was — NOT a paid rustconv run — but it is large and
needs its own **format-string differential harness** (fuzz `%`-specs, grade the emitted bytes +
return + errno against the genuine `pf_printf_a`/`pf_printf_w`, which are already reachable as
reference symbols in `wcs.c`). Scope it as its own sub-lane before starting; it is several waves.

**Small gradeable wins available with ZERO new harness work** (do any time, e.g. as warm-ups):
- `_strdup` / `_wcsdup` (+ `_mbsdup`) — `malloc(strlen+1)` + copy; grade the buffer CONTENTS and the
  NULL disposition (the pointers differ; malloc is shared environment). ~1 short wave.
- `rand` / `srand` — pure LCG on `thread_data->random_seed`; needs a 4th reference TU (`misc.c`) but
  the logic is trivial and the seed field is already in `msvcrt-sys`'s `__thread_data`.

## Wave 14a (2026-07-14) — the printf PROBE. Assemble-vs-port decided: **PORT printf.h**. Bonus: a kernel hang found + fixed.

The wave-13 question ("can cnt-rust's already-Rust `streamout` sit behind the msvcrt printf
exports?") got its answer from a **C-vs-C third-party differential**, no new translation spent:
`tests/printf_probe.rs` drives (a) genuine `printf/streamout.c` compiled under cnt-difftest's
exact proven recipe — the configuration cnt-difftest proved byte-equal to the Rust engine, so
every verdict transfers to streamout-Rust by transitivity — against (b) the genuine msvcrt
`pf_printf_a`, as a SECOND compilation of wcs.c in a `pfflat_*` namespace under a **flat
`char*` va_list** (the Windows-x64 model; the SysV-compiled `ref_*` copy is structurally
uncallable — full story + the glibc-escape oracle lesson in field manual **§9.80**). Both
engines consume the SAME hand-packed 8-byte-slot buffer; an `%I64d` oracle-honesty canary
hard-fails the probe if the reference ever escapes to the host libc again.

**Verdict (111 MATCH / 27 DIVERGE):**
- **Integers/strings/chars/pointers: streamout ≡ msvcrt** — 75/78 rows byte-identical
  (deltas: `%#x` of 0, `%jd`, `%td`). The parser/state-machine half of the engines agree.
- **Floats: msvcrt's bignum formatter is structurally better** than streamout's double-math:
  exact high-precision digits (`%.30f`), denormals, `-0.0`, `%g` trailing-zero strip + form
  threshold, `1.#INF00` padding, real `%a` (streamout's is a FIXME).
- **Decision: PORT `printf.h`'s engine to Rust (wave 14b), do NOT assemble streamout** — that
  would silently downgrade msvcrt.dll's float output (the DLL ships the Wine engine today;
  never-weaken applies). `bnum.rs` (wave 13) is the shared limb engine the formatter needs;
  the probe's `pfflat_*` oracle + flat-va_list drive IS the grading harness for the port.

**Bonus find (the probe's first product, before any comparison):** the kernel's Rust
`streamout` float path hung FOREVER on `%f/%e/%g` of ±Inf — `fl::exp10_floor`'s correction
loop never terminates on Inf (`pow10(309)` saturates; `Inf <= Inf`). Guarded to return 308
(`DBL_MAX_10_EXP`, = NaN's accidental value) in `fl` + every recipe C-twin; ±Inf/NaN rows now
pinned in cnt-difftest (`diff_streamout_float`). Field manual §9.80 finding 0.

## Wave 14m (2026-07-14) — the MATH DOOR. +45 exports flipped for the cost of ONE Cargo feature. msvcrt.dll's libm is the kernel's Rust engine now.

The single highest-leverage flip in the lane so far, and it wrote almost no new code:

**The discovery that made it cheap:** msvcrt.dll's math exports (`sin`/`cos`/`exp`/`pow`/…)
bind DIRECTLY to `crtmath`'s plain-name engine symbols — Wine's `MSVCRT_*` wrapper functions
in `math.c` are dead code (the spec maps no functions through them; its 48 `MSVCRT_` lines are
all data externs). And `cnt-rust/cabi/math.rs` already defines those same plain names in Rust
on all three arches: kernel-flipped, cnt-difftest-graded **against the very
`sdk/lib/crt/math/*.c` sources crtmath compiles** (`USE_MSVCRT_PREFIX` is vestigial — appears
nowhere in math/float sources; only the Bessel files touch errno, and they stay C). So the
user-mode flip is the SAME substitution the kernel already shipped and gated.

**Mechanism (the whole wave):** a new `cabi-math` cnt-rust feature — gen_cnt.py's mod.rs
template now gates every cabi submodule on `cabi` except `math.rs` (compiled under either
feature) — enabled from **msvcrt-crt-cargo** (the staticlib), NOT msvcrt-crt-rust (the rlib):
the difftest links the rlib, and 66 `#[no_mangle]` libm names there would shadow the HOST libm
under the reference TUs (oracle contamination — the probe's §9.80 lesson applied proactively).
`extern crate cnt_rust` forces the objects into the archive; whole-archive + lld first-match
does the rest. Verified in the built DLL by export-RVA clustering: `sin` 0x119A0 / `cos`
0x10020 / `sqrt` 0x12390 sit beside known-Rust `strlen` 0xE410; unflipped `frexp`/`ldiv`/`sinf`
sit in the C region 0x7A640+.

**Count:** +45 exported functions (measured: `llvm-nm` of the Rust archive ∩ the DLL's COFF
export table = **240 Rust-defined msvcrt.dll exports**, amd64). 21 of math.rs's 66 symbols
aren't msvcrt exports (`exp2`/`round`/`scalbn`/`nextafter*`/`sincos`/`_dclass` family…) and
ride along inert. Displaced per arch: i386's x87 asm (`sin_asm.s` etc.), amd64's
`fabs/fmod/sqrt` .S, arm64's portable C — the identical displacement the kernel gate already
validated.

**Trap eaten + template-fixed:** cabi/mod.rs is `@generated` but had drifted (hand-added
`mod strtod/wcs/search`) — the regen silently erased them and broke cnt-difftest. The three
modules live in the gen_cnt.py template now; never hand-edit mod.rs.

**Still C (the follow-up ledger for this family):** the f32 twins msvcrt exports that math.rs
lacks (`sinf cosf expf logf log10f powf fmodf frexp` + `_copysign _scalb _fpclass _cabs`), the
Bessel family (`j0 j1 jn y0 y1 yn` — errno/_handle_error semantics), the i386 `_CI*` compiler
thunks + `__libm_sse2_*` register-convention entries, and the FP-environment block
(`_control87/_statusfp/fe*` — stateful, shared with the C runtime init).
