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

### Phase B — broaden the DLL-body flips, family by family
With the pipe proven, flip the remaining pure families cnt-rust already owns: `str*`, `wcs*`, int↔str (`_itoa/_ltoa/_i64toa/…`), path (`_makepath/_splitpath`), float↔str (`_ecvt/_fcvt/_gcvt`), and the whole `math.c`/`sincos.c` transcendental set (cnt-rust `cabi/math.rs`). One family per wave, gate ×3 between. **Semantic watch:** cnt-rust "tracks libcntpr, not textbook libc" — for pure ops they coincide, but any function msvcrt makes locale-aware or errno-setting must be verified against `dll/win32/msvcrt/*.c` before flipping (byte-diff where pointer-free, §1605).

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
