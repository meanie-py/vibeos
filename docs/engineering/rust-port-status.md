# ReactOS → Rust Port — Current As-Built Status

**A multi-architecture port of ReactOS (Windows-compatible OS) translated from C to
Rust, booting to a graphical desktop on i386, amd64, and ARM64.**

*Live status as of 2026-07-03, re-measured against `master` (the single mainline) by a
fresh 5-surface tree audit (kernel, win32k, userland DLLs, boot-userland exes, static
libs/RTL, drivers). This is THE single source of truth for Rust-vs-C coverage — the seven
per-surface `*-translation-audit.md` files were superseded by this pass and moved to
[`archive/`](archive/) (their per-file counts were ~12 days stale). Historical snapshots +
project timeline live in the [old-details rollup](archive/rust-port-status-rollup.md). Re-measure
before trusting later.*

> **Cohabitation caveat.** Convention is *never delete a translated C file*: a flipped
> `.c` is commented out in CMake (`# replaced by Rust`) and kept on disk as dead source;
> the Rust body (`#[no_mangle]` symbol or cargo-driven link) takes over. So
> `find -name '*.c' | wc -l` counts **on-disk** C, not **compiled** C. Every "C remaining"
> figure below is compiled/active C unless marked *on-disk*.

## TL;DR

**The entire boot-to-desktop path is Rust.** Kernel, win32k, the usermode boot spine, the
core client DLLs, and the driver fleet are all ~90–100% translated with the replaced C
*removed from the compile* (not overridden-in-place). The OS boots to a graphical desktop
on all three arches with an essentially all-Rust runtime.

The remaining C is four well-bounded categories, **none on the boot-critical path**: (1) the
bulk userland **CRT** (msvcrt/ucrt leaf library — the assemble-from-`cnt-rust`+`rtl-rust`
track), (2) **vendored/by-policy** C (ACPICA, FreeType), (3) **legacy dead** C superseded by
Rust (videoprt, old NDIS/tcpip), and (4) **irreducible glue** (variadic trampolines, arch
`.S`, generated RPC/syscall stubs). *(The desktop shell `explorer.exe` is NO LONGER a
remnant — it's ~99% Rust as of the crate-split era; see the ledger below.)* The broad
still-C userland frontier is the **~140 ROS-native DLLs (~240K LOC)** + the **Wine-derived
bucket (~154 DLLs, ~1.05M LOC)** catalogued in the roadmap's Phase 0 Tier 3.

Tree inventory (2026-07-25): **2,908 `.rs` files · 280 crates** under `sdk/lib/rust/`. Re-derive
rather than trusting this line: `find sdk/lib/rust -name '*.rs' | wc -l` and
`find sdk/lib/rust -name Cargo.toml | wc -l`.

## Headline component ledger

| Surface | Status | Coverage |
|---|---|---|
| **ntoskrnl.exe** (kernel) | 344 subsystem `.c` removed from build; 8-file Rust-glue floor | **~100%** |
| **win32k.sys** (GUI) | 166 `.rs`/256K LOC; 159 `.c`/`.cpp` flipped; glue = shims/floatobj/gendib | **~100%** |
| **ntdll.dll** | body Rust; cargo-driven link; per-arch `dispatch` + gen stubs | **~97%** |
| **Boot spine** smss/csrss/csrsrv/winlogon/services/lsass/userinit | bodies 100% Rust, cargo-driven, zero C in link | **~100%** |
| **user32 / gdi32 / secur32 / msafd** | 0 active body C; cohab-flipped | **~100%** |
| **kernel32.dll** | body Rust; only NLS `locale.c` + resources left | **~95%** |
| **advapi32.dll** | only `wine/security.c` + generated RPC stubs left | **~90%** |
| **msgina.dll** | 7 `.c` flipped; one C++ leaf (`dimmedwindow.cpp`) | **~88%** |
| **ole32.dll** (COM runtime) | `ole32-rust`: **the entire DLL is Rust — all 29 TUs.** The COM core (compobj + marshal/stubmanager/rpc/ftmarshal/git); the mechanical ring (task allocator + streams + lockbytes, the Running Object Table + all 6 monikers, `bindctx`/`errorinfo`/`enumx`/`dictionary`, `ole2`/`ole2impl`/`ole32_main`, `clipboard`, `comcat`, `oleproxy`, the NDR `usrmarshal`); and the storage/embedding cluster (`storage32` — the compound-file implementation, `stg_stream`, `stg_prop`, `datacache`, `defaulthandler`, `oleobj`). The C `SOURCE` list in the CMakeLists is EMPTY. Boots ×3. `guid.c` stays C permanently (an `initguid.h` TU whose job is emitting the moniker CLSIDs as data) | **done** |
| **oleaut32.dll** (OLE automation) | `oleaut32-rust`: **the entire DLL is Rust — all 14 TUs.** The automation core: BSTR (`oleaut`), VARIANT (`variant`), the VarXxxFromYyy coercion matrix (`vartype`), SAFEARRAY (`safearray`), the typelib reader/writer + ITypeInfo/ITypeLib (`typelib` + submodules), IDispatch (`dispatch`), user-marshal (`usrmarshal`), `hash`. Plus the OLE-control cluster (2026-07-13): `olefont` (IFont/IFontDisp), `olepicture` (IPicture + OleLoadPicture*), `varformat` (the VarFormat engine), `connpt` (connection points), `recinfo` (IRecordInfo), `olepropframe`. The C `SOURCE` list in the CMakeLists is EMPTY. Boots ×3. `export_audit` clean: all 398 spec exports have a `#[no_mangle]` Rust definition with the spec's convention | **done** |
| **ws2_32.dll** (winsock client) | `ws2_32-rust`: **the entire DLL is Rust — all 34 compiled TUs.** The socket API (`socklife`/`sockctrl`/`send`/`recv`/`select`/`ioctl`/`event`/`qos`/`dupsock`), the transport + namespace catalogs (`dcatalog`/`dcatitem`/`nscatalo`/`nscatent`/`nspinstl`/`spinstal`), the object model (`dprocess`/`dprovide`/`dsocket`/`dthread`/`nsprovid`/`nsquery`), the WSP upcall table (`sputil`), async getXbyY (`async`/`getxbyxx`/`getproto`), name resolution (`rnr`/`qshelpr`/`addrinfo`/`addrconv`/`enumprot`/`rasdial`), and startup/dllmain/wsautil/bhook. The C `SOURCE` list in the CMakeLists is EMPTY. Boots ×3, and `test_usermode_winsock_roundtrip` (the M5 capstone) passes ON the Rust: TCP echo + UDP DNS + a **real HTTP request to the public internet**, all via ws2_32→msafd→AFD→netstack→smoltcp. `verify_lost_exports` + `verify_c_signatures` clean | **done** |
| **winsock/net satellite DLLs** | Cohab-flipped to Rust alongside ws2_32 (2026-07-17/18): **wshtcpip** (`wshtcpip-rust`, both TUs — the TCP/IP WSH provider), **ws2help** (`ws2help-rust`, dual-role static lib), **mswsock** (`mswsock-rust`), **wshirda** (`wshirda-rust` — the IrDA WSH provider, all-stub), **userenv** (`userenv-rust`). **netapi32** (`netapi32-rust`, **all 21 TUs** — FULL flip completed 2026-07-18; see the MIDL-island note below), **dhcpcsvc** (`dhcpcsvc-rust`, **all 12 TUs** — FULL flip, the DhcpC* API + the BSD/ISC dhclient state machine). **iphlpapi** (`iphlpapi-rust`, **all 11 TUs** — FULL flip 2026-07-18; only the generated `iphlpapi_stubs.c` remains C, and the `iphlpapi_rust_shims.c` out-of-line no-op TRACE/FIXME hazard was deleted with it, field manual §9.111). **Partial flips:** **dnsapi** (`dnsapi-rust`, all 8 TUs — FULL flip, 2026-07-18; the single `dnsrslvr_c.h` MIDL interface + the `adns` staticlib stay C). netapi32's 7 RPC-stub TUs (browser/dssetup/misc/netlogon/schedule/srvsvc/wksta_new) landed 2026-07-18. The long-standing "6 conflicting MIDL interfaces" blocker was MISDIAGNOSED: the conflict was the `<lm.h>` master header vs the WIDL stubs, not the stubs against each other (only browser↔srvsvc genuinely collide, over `_SERVER_INFO_100`) — `netapi32.h` itself pulls only the narrow lmaccess/lmapibuf/lmerr trio and lets each `*_c.h` supply its own wire types. `gen_driver_types.sh` grew `TYPES_STEM` so one crate can carry several bindgen islands; netapi32-rust now has `types` (lm.h, the 14 c2rust modules), `types_rpc` (5 interfaces, 6 modules) and `types_browser`, with a `prelude_common` + per-island prelude split (377 type names overlap, so the islands must never share a glob scope). Field manual §9.108; the integration bugs a green cargo check cannot see are §9.109. | **full** |
| **hal** (x86 + arm64 stub) | 42+ cohab markers; `.S` NT-ABI + setjmp glue *(not re-verified this pass)*. **i386 default flavor is now APIC** (LAPIC/IOAPIC/TPR, lazy-IRQL, RTC clock — i386-apic-hal lane, 2026-07-16); the legacy 8259 PIC HAL is preserved in-tree, gated off, behind `vibeos-cli build --hal-pic` (`VIBEOS_PIC_HAL`). amd64 was already APIC | **~97%** |
| **rtl** static lib | 80/82 `.c` flipped; `dbgprint_variadic.c` + asm floor | **~100%** |
| Cohab helper libs (cmlib, rossym, evtlib, inflib, …) | real `.c` flipped for kernel/cross consumers | **~100%** |
| **freeldr + boot DLLs** | modules + cmlib/cport/cmboot; 16-bit `.S` floor *(not re-verified this pass)* | **~99%** |
| **Driver fleet** (storage, FS, input, bus, netstack) | see below — netstack is a from-scratch Rust rewrite; rest cohab-flipped | **~90–100%** |
| **LSA/SAM server** (lsasrv / samsrv) | active frontier — samsrv ~90%, lsasrv ~50%+ | **partial** |
| **explorer.exe** (desktop shell) | `explorer-rust` 22 `.rs`/22K LOC; 22 cohab markers; 2 glue `.cpp` left (settings_shim, util_vararg) | **~99%** |
| **CRT** (msvcrt/ucrt body) | **483 of 1,087 non-stub C exports Rust-backed — 44%** (2026-07-26 wave 16 finished `string.c` + `wcs.c`; the denominator here excludes the `?`-mangled C++ surface, which is a separate lane. Was 431 of 1,165 counting C++) (2026-07-26, re-measured amd64 as `llvm-nm libmsvcrt_crt_cargo.a` ∩ the spec's implementation names; was 240 on 2026-07-14, moved by wave 15's `mbcs.c` and wave 14c's printf shells). Wave 14m flipped the **whole core libm** (+45 exports: `sin`/`cos`/`exp`/`pow`/`sqrt`/`fmod`/`abs`/`div`/… — the DLL's math exports bind straight to `crtmath`'s plain names, and cnt-rust's kernel-proven `cabi/math.rs` supplies them via one `cabi-math` feature; no new code, cnt-difftest already grades that engine against the same sources). Waves 1–13 cover — the pure string/mem/wide/radix/`_s` floor, the whole locale-table **ctype/case** family (`ctype.c`), the **collation** family, and the **bignum float parser** (`strtod`/`atof`/`wcstod`/`_atoflt`/`_atodbl`/`_atoldbl`/`__STRINGTOLD` — msvcrt's own `bnum.h` PORTED, wave 5's DO-NOT-FLIP reversed the honest way). Differentially proved against the genuine `string.c`/`wcs.c`/`ctype.c` by **six** instruments (`classify` values incl. **exact 64-bit float bit patterns** · `oob_guard` access · `perf` speed · `hot_path` per-element env lookups · `ref_integrity` the oracle · `link`), in **both** the "C" and a real NLS locale. The float family flips on all three arches — i386 via x87 `ST(0)` extern-C returns (I first gated it out on a wrong premise; field manual §9.79). Phase D's locale door is OPEN (`msvcrt-sys` = bindgen + `offset_of!` + a two-compiler layout guardian). The 581 `.c` still compile; what remains is gated mainly on **printf** (Phase C). | **~13–19%** |

### CRT (msvcrt) detail — the assemble-from-`cnt-rust` track

**Wave 15 (2026-07-25) opens a SECOND track: whole-C-file rustconv translation.** Waves 1–14
assemble a family at a time from `cnt-rust`'s difftest-pinned engines; wave 15 translates a
`.c` outright into `msvcrt-crt-rust/src/msvcrt/<stem>.rs`, kept 1:1 with its source. First
file: **`mbcs.c` — 3,624 lines, 175 `#[no_mangle]` exports, $0.95**, `cargo check` clean with
zero warnings on all three arches. The two tracks coexist because the cohab rule is unchanged:
whatever Rust defines wins by lld first-match and the C copy goes dead, so a partial file is a
safe state and no CMake edit is needed. The crate gained `src/prelude.rs` (the standard
`*-rust` import surface — every `extern` transcribed from a named C header, not inferred).

**Wave 14c (2026-07-26) — `wcs.c`'s printf ENTRY POINTS, and the export total re-measured.**
Wave 14b flipped the printf *engine* and swung file.c's stream family with it, but `wcs.c`'s own
shells kept calling the C: clang had already IPA-specialized the same-TU engine into them, and
intra-TU inlining never consults the linker (**symbol replacement flips a TU's callers, not its
housemates**). The 30 `va_list`-taking entry points are now `msvcrt-crt-rust/src/printf_shells.rs`
— `_vsnprintf`, `vsprintf`, `_vswprintf`, the `_s` bounded family, the `_p` positional family and
the `_vsc*` counting sinks — graded by a tenth harness, `printf_shells`, at **1,260 rows** against
the genuine `wcs.c`. The `...` shells (`sprintf`, `_snprintf`, `_swprintf`, …) still need a
`va_start` trampoline and are the remaining half of the wave.

**Export total, re-measured against the DLL** (the method waves 1–13 used, not arithmetic on 240):
**431 of 1,165 non-stub amd64 exports are Rust-backed, 37%** — 400 before wave 14c, +31 from it
(30 shells; `vsnprintf`/`vswprintf` are spec aliases onto two of them). The remaining C is
concentrated: `file.c` 5,897 LOC (stdio, Phase D), `concurrency.c` 3,840 (C++), `scanf.c`'s 40
exports (engine not ported), `math.c` 13 of 144 functions flipped.

**Wave 16 (2026-07-26) — `string.c` and `wcs.c` FINISHED. 483 of 1,087 non-stub C exports
Rust-backed, 44%** (was 433/39% after wave 14c). Both files now have zero unflipped exports;
what remains attributed to them is only the printf `...` trampolines, which are C by language
limit. string.c's last 17 and wcs.c's last 29 (+4 private callees the family needed —
`_wtoi_l`, `_wcstoul_l`, `wctomb_s`, `_wcstombs_l`, all of which would have left a split family
had the callers flipped without them).

Thirteen of string.c's seventeen were not new code: the crate already shipped those bodies with
the locale hardwired to NULL, so the work was threading an explicit `_locale_t` through
`strtoi64_contract`, `strtoui64_contract`, `strnicmp_core`, `strcase_s_core` and a newly
extracted `memicmp_core`. `locinfo_of(NULL)` **is** `get_locinfo()`, so every already-graded
plain form is bit-identical afterwards. Genuinely new: `_strdup`, `_atoflt`, `I10_OUTPUT`.
wcs.c contributed the wide integer engines and clamps, the case folders, the comparers, and the
whole wchar→multibyte family including two file-statics that stay private exactly as they are
file-static in the C.

Deliberately preserved C misbehaviours (this is a differential port — "fixing" one is a
failure): `_strtoul_l`/`_wcstoul_l` underflow clamping to **1**, `_wtoi64_l`'s missing overflow
check, `wctob`'s strict `< 0xff` so U+00FF reports EOF, and `wcsrtombs_s_l`'s genuine
`mbstr[-1]` touch at `conv == size == 0`.

**A real ABI bug in already-landed code fell out of it.** `prelude.rs` declared the nine
`_isw*_l` predicates taking `wint_t` (`unsigned short`) while this crate DEFINES all nine taking
`u32` — so every call through the prelude wrote 16 bits into an argument read as 32, upper half
undefined per the Windows x64 ABI, benign only because rustc happens to zero-extend.
`scan_extern_arity` cannot see it: the arity matches. Field manual carries the class under
ABI/FFI — never `extern`-declare a symbol your own crate defines.

**Wave 14c part 2 (2026-07-26) — the 30 `...` shells, in `dll/win32/msvcrt/rust_varargs_shim.c`.**
Stable Rust cannot DEFINE a C-variadic, so these stay C permanently as three-line `va_start`
trampolines over the Rust engine — but that is now the *whole* of msvcrt's C printf: every byte
that actually formats anything is Rust. Their own TU (in wcs.c clang re-inlines the C engine into
them and undoes the flip) and compiled into the msvcrt **MODULE**, not `msvcrt_static`, so module
objects beat archive members at first-match. Verified on the built DLL: export `_snprintf` → the
shim's block → `callq _vsnprintf` → the Rust archive; wcs.o's copies sit dead. **The export total
does not move — 431 — and that is honest**: these 30 are C. What moved is which engine 30 exports
*execute*. Graded by `tools/scan_varargs_shim_parity.py` (in `vibeos-cli check`, baseline empty)
rather than a difftest, because a differential is *impossible* here: neither instantiation can run
a variadic shell on a SysV host (field manual §9.66). Four negative controls fire; it caught the
shim defining `_sprintf_p`/`_scwprintf_p`, which are `#if _MSVCR_VER>=80` and exist at no vintage
this DLL builds. **Next**: the ~225 host-gradeable exports (`scanf.c` 39 — 820 LOC, zero Win32,
16 template instantiations collapsing to one generic Rust engine; then `time.c` 60, `wcs.c` 32,
`math.c` 30), then `file.c`'s **170** — 26% of all remaining exports, gated on a Win32-shim
difftest substrate that does not exist yet.

**240 Rust-defined `msvcrt.dll` exports on all three arches** as of 2026-07-14 (waves 1–13: 209 hand-flipped wrappers; wave 14m: +45 libm exports assembled from cnt-rust's `cabi/math.rs` via the `cabi-math` feature — engine grading lives in cnt-difftest; wave 14a decided printf is a PORT and wave 14b DELIVERED it: printf.h's engine — 2,312 lines, bignum floats over the shared bnum.rs — graded 142 narrow + 30 wide rows byte-identical vs the genuine pf_printf_a; the DLL's stream-printf family (printf/fprintf/vfprintf) runs the Rust engine now, the sprintf-family shells await wave 14c's varargs trampolines. See the lane plan + field manual §9.83)
(waves 1–13). Verified by parsing the DLL's export table: every flip lands in the Rust cluster,
every deliberate refusal is still C (`memmove`, `strtok`, `_ecvt`/`_fcvt`/`_gcvt` — the
printf-backed converters — at the high RVAs). Wave 13 added the **bignum float parser** (`strtod`
and the whole `atof`/`wcstod`/`_atoflt`/`_atodbl`/`_atoldbl`/`__STRINGTOLD` family) on **all three
arches** — including i386, where an `extern "C" fn -> f64` returns via x87 `ST(0)` and needs no SSE
(§9.79 point 4 corrects an earlier wrong read that had gated i386 out).

Flipped: the mem/str floor (`memcpy` `memset` `memchr` `memcmp` `str{len,cmp,cpy,cat,chr,
rchr,str,spn,cspn,pbrk,nlen,ncmp,ncpy,ncat}` `_str{lwr,upr,rev,set,nset}` `_memccpy` `_swab`
`__strncnt`), the wide twins (`wcs{len,cmp,ncmp,chr,rchr,str,spn,cspn,pbrk,cpy,cat,ncpy,
nlen}` `_wcs{rev,set,nset}`), the integer parsers (`atoi` `atol` `_atoi64` `strtol` `strtoul`
`_strtoi64` `_strtoui64`), the radix family (`_itoa` `_ltoa` `_ultoa` `_i64toa` `_ui64toa` +
wide + all ten `_s` variants), the `_s` secure family (`strcpy_s` `strcat_s` `strncpy_s`
`strncat_s` `_strnset_s` + the six wide), the tokenizers (`strtok_s` `wcstok_s`), the
case-insensitive family (`_stricmp` `_strnicmp` `_memicmp` `_strlwr_s` `_strupr_s` + the six
wide), the ctype/case + collation families (`ctype.c`'s 44 + `strcoll`/`strxfrm` and wide
twins), and the **bignum float parser** (`strtod` `_strtod_l` `atof`
`_atof_l` `_atoflt_l` `_atodbl` `_atodbl_l` `wcstod` `_wcstod_l` `_wtof` `_wtof_l` `_atoldbl`
`_atoldbl_l` `__STRINGTOLD` `__STRINGTOLD_L`).

**The method is the product.** Nothing flips without a differential proof. Ten orthogonal
harnesses (`sdk/lib/rust/msvcrt-difftest/`), each negative-controlled:

| harness | asks | blind to |
|---|---|---|
| `classify` | wrong **values**? (20k fuzz cases vs the genuine C, grading bytes + **exact 64-bit float bit patterns** + return + **errno** (seeded with a non-zero sentinel so a spurious *clear* is visible) + **invalid-parameter handler**, with per-row coverage requirements) | access, speed |
| `oob_guard` | right answer, wrong **memory**? (guard-page probes; 36 bounded fns) | values, speed |
| `perf` | right answer, too **slowly**? (>2× the C fails) | values, access |
| `hot_path` | a per-element loop doing per-element **environment lookups**? (counts `get_locinfo()` at n=8 vs n=512 — the cost no *ratio* can see, §9.76) | values, access |
| `ref_integrity` | **is the oracle itself honest?** (see §9.66 — glibc's attributes had been deleting the reference C's guards; also cross-checks every `msvcrt-sys` layout offset subject-vs-bindgen) | the subject |
| `link` | do the shipping symbols actually **resolve** to the Rust bodies? | everything else |
| `mbcs_fixture` | is the **DBCS state** the mbcs cases are graded against actually multi-byte? (asserts non-zero lead/trail bytes per codepage — without it every case takes the single-byte path and the suite grades none of the DBCS logic) | the subject |
| `mbcs` | **173 of 175** generated cases over `mbcs.c` on **7 codepages** (932/936/949/950/1361 DBCS + 1252/20127 single-byte), enumerating each function's whole input domain rather than sampling | speed, access; `create_mbcinfo` |
| `mbcs_state` | does the SUBJECT's **`create_mbcinfo`** build the same tables? Its own process, because every `mbcs` case is pinned against a fixture built from the *reference's* copy — so the subject's never runs there and a wrong one passes all 173 | everything the fixture holds still |
| `printf_shells` | the 30 printf ENTRY POINTS vs the genuine `wcs.c` — **1,260 rows**, four channels each (return · full buffer image incl. a guard prefix · errno · invalid-parameter count). `printf_port` grades the engine and is structurally blind to everything wrapped *around* it: which `options` bits go in, how a `size_t` buffer size truncates into the C's `int len`, which of `ret<0`/`ret==len` decides a `-1`, whether the `-1` path zeroes the buffer or just terminates it | the `...` shells (no `va_start` on stable) |

`mbcs.c` was graded on 2026-07-24 and the first run found **three real bugs** in the shipping
Rust: `_mbsnset_l` dropped its truncation (C's `while (a && b && len--)` wraps `len` to
`SIZE_MAX` on exit, and the epilogue reads that), and `_mbslwr_s_l`/`_mbsupr_s_l` walked *copies*
of their parameters — so the buffer-too-small `EINVAL` was unreachable and the terminator landed
at the start of the string. Both classes are in the field manual under "Semantic rewrites that
flip meaning". Coverage then went 160 -> 167 -> **173 of 175** as the generator learned the
`_s` out-scalars, the restartable `mbstate_t` sweep, the `char **` cursors (`_mbstok_s`,
`mbsrtowcs`) and the `__p_*` state accessors. `create_mbcinfo` is graded separately by
`mbcs_state` — and diverged on its first run, which is what a fixture built from one side
cannot show you. **`_setmbcp` is the last ungraded function**: it installs through
`MSVCRT_locale->mbcinfo`, which the harness defines as NULL, so grading it means modelling
`MSVCRT_locale` and the thread-data `locale_flags` on both sides rather than writing a case.

All host harnesses now run under **`vibeos-cli difftest`**, which `gate` invokes and which gates
the push stamp — see `tests/qemu/cli/README.md`. They were previously reachable only by typing
`cargo test` in a crate dir, and `rtl-difftest` sat 2/87 red on master as a result.

**What it has refused to flip, and why** — these are the interesting entries:
- **`strtod`-from-`cnt-rust`** — cnt-rust's float parser is the ReactOS naive accumulator;
  msvcrt's is a Wine bignum. Measured **14 ULPs apart**. The `strtod [PROBE]` row stays in the
  tree as an executable DO-NOT-FLIP record for *that route*. **Wave 13 flipped `strtod` the
  honest way instead** — by PORTING msvcrt's own bignum (`fpnum.rs`), bit-exact, amd64/arm64.
  `_ecvt`/`_fcvt`/`_gcvt` (the float→string direction) are literally `_snprintf` calls and
  remain a **Phase C (printf)** deliverable.
- **`memmove`** — no oracle. The refstub routes it into the private anti-circularity
  namespace, so the compiled reference doesn't define an msvcrt `memmove` to compare against.
  The body would be three lines. It stays C.
- **`strtok`/`wcstok`** — reach into msvcrt's `thread_data_t`; getting at that field means
  hardcoding a struct offset we don't own (the magic-offset class). Wants a real TLS story.
- **The wide integer parsers** (`wcstol`/`wcstoul`/`_wtoi`/…) — locale-bound via `_iswspace_l`
  where the *narrow* ones are not. Nearly flipped on a symmetry argument. Phase D.

Lane doc: `docs/engineering/active-lanes/msvcrt-from-pieces-plan.md`. Field manual: §9.60,
§9.61, §9.63, §9.64, **§9.66**.

### Driver fleet detail
Essentially the entire boot/smoke driver fleet is Rust. **`netstack.sys` is the only
from-scratch Rust rewrite (100%, zero C, replacing the legacy NDIS/tcpip stack).**
Everything else is a cohab flip (body in a `*-rust` crate, thin C residue —
`guid.c`/data-TU/stub — so ~90–100% each): all storage class/port/ide
(uniata, scsiport, classpnp, disk, cdrom, ramdisk, storport, storahci, **buslogic**,
atapi, pciide(x), mountmgr, partmgr), all boot-path filesystems (fastfat, cdfs, fs_rec,
mup, msfs, npfs), input (i8042prt, kbdclass, mouclass, arm64input), bus (pci, pcix, acpi
glue), display (framebuf, arm64vid, bootvid), **afd**, **ks**, ksecdd, fltmgr, and the USB
HC/hub/function stack. **Still pure C (not targets):** `videoprt.sys` (legacy INT10, dark
under UEFI), the vendored **ACPICA** core (155 `.c` inside acpi.sys, kept by policy), and
the legacy **NDIS/tcpip/tdi** stack (dead, superseded by netstack).

## Beyond a port — Rust-native capabilities & correctness (the differentiators)

*Added 2026-07-10. Coverage above is "how much C became Rust." This section is the part
that isn't just a port — capabilities and correctness properties that go past upstream
ReactOS, made tractable because the kernel is now our Rust in one workspace. Landed-vs-in-
flight is marked; in-flight lanes live in [`active-lanes/`](active-lanes/).*

- **x86-on-ARM64 — a homegrown Rosetta (WoW64 + FEX).** 32-bit x86 PE binaries launch as a
  **WoW64 process on the ARM64 NT kernel** — Phase B complete + Phase C M1 landed (real
  i386 PE runs through the full WoW64 stack: `wow64.dll`/`wow64cpu`/`wow64win`, generated
  syscall thunks). M3b (ntdll32 loader → `RtlCreateActivationContext`) is in flight on
  `fex-arm`, currently blocked on a foundational arm64 unaligned-access fault. The x86→arm64
  execution engine is **FEX-emu** (vendored FEX-2607, `wow64fex`/`FEXOfflineCompiler` link
  with zero patches; `fexprobe` executes an x86 guest blob) — the "run real x86 Windows
  binaries on an ARM64 Rust kernel" thesis. Docs: `active-lanes/wow64-plan.md`,
  `active-lanes/fex-bringup-plan.md`.
- **SMP / multi-processor bring-up.** The `CONFIG_SMP` kernel (`ntkrnlmp.exe`) **links on
  all three arches and boots `-smp 1` to desktop (amd64)** — landed. The **first amd64 AP
  starts and runs `KiSystemStartup`** (CPU 1 parks) on the `smp-bringup` lane; M2 is pinned
  on an AP-path `ZwOpenFile(ntdll)` 0x6B. This is the pressure test that makes every Rust
  spinlock/queued-spinlock/pushlock/interlocked path run under *real* concurrency for the
  first time (UP made them ≈ IRQL raises). Doc: `active-lanes/smp-bringup-quest.md`.
- **Compile-time & RAII correctness the C never had.** IRQL/spinlock lifetimes are
  **RAII-guarded** (`ntkrnl-sys::guards` — `IrqlGuard`/`SpinLockGuard`, released on `Drop`,
  the idiomatic replacement for `__finally`); a 504-site audit found **0 leaks**. SEH
  catch-at-a-distance is being replaced by **validate-first `Result<T>` probes** (`probe.rs`;
  SE + OB migrated) and callee-side `try_*` contract conversion, per `dummy-pseh-verdict.md`
  — the honest Rust shape of `__try/__except`. arm64 carries **hardware-backed IRQL** via
  `GICC_PMR` (lazy — no MMIO in the PASSIVE..DISPATCH band, so spinlocks stay register-only).
  A planned **typestate CPU-ownership model** (`archive/execution-context-ownership-model.md`)
  would make context-switch-safety a compile-time property.
- **Runtime correctness tripwires unique to a Rust kernel.** Bespoke always-on / opt-in
  instrumentation no stock NT has, enabled because we own every symbol: **waitdep**
  (dispatcher wait-block chain integrity at `KiUnlinkThread`/`KiWaitTest`, BUGCHECKs 0x109
  with a full chain dump at the corruption point — field manual §9.27/§9.31/§9.36),
  **lockdep** (pushlock self-deadlock + ABBA lock-order graph, `--lockdep`), **cswitchdep**
  (context-switch safety), per-boot **special-pool** tripwires (`--tripwire SPECIALPOOL=*`),
  and a post-opt **captures(none) IR audit** (`scan_captures.py`). See
  `kernel-debugging-infra.md`.
- **ABI-drift elimination (in flight).** The **exported-symbol trampoline refactor** (roadmap
  Phase 0 Tier 2) splits every export into a native snake_case `nt_foo()` (plain Rust ABI,
  holds the logic) + a generated `NtFoo` calling-convention trampoline — so no hand-written
  `extern` decl is left to drift, killing the ABI-mismatch `ud2`/`noreturn` cascade class at
  the source (today caught reactively by `verify_ffi` + `scan_cc_mismatch`).

## Remaining C translation frontiers (the real work)

- **ROS-native userland DLLs — ~140 components, ~240K LOC.** The real resume-translation
  queue (Winsock/net, setup/PnP/device, ~15 control panels, services, security/auth, shell
  exts, audio/ACM). Batched in the roadmap Phase 0 Tier 3. *(`explorer.exe` is DONE — ~99%
  Rust, `explorer-rust`; only 2 glue `.cpp` remain.)*
- **CRT floor — ~0–5%.** `sdk/lib/crt/` = 581 `.c` (printf/scanf/string/math/locale/
  mbstring/wine) still compiled; only `crt-startup/dllmain/crt0` entry-glue + the
  `libcntpr` kernel subset (`cnt-rust`) are Rust. Largely irreducible variadic.
- **LSA/SAM server lane — in flight.** samsrv (~90%, `samsrv-rust` 16 `.rs`) and lsasrv
  (~50%+, `lsasrv-rust` 20 `.rs`) are the active server-side frontier.
- **kernel32/advapi32 tails.** kernel32 NLS `locale.c` + resources; advapi32 `wine/
  security.c` + generated RPC.
- **shell32 activatable COM surface — 33 of 33 classes Rust-owned; compiled C++ tail is next.**
  Rust-owned activated classes now span `CDropTargetHelper`, `CActiveDesktop`,
  `COpenControlPanel`, `CExeDropHandler`, `CCopyAsPathMenu`, `CShellItem`, `CFontsFolder`,
  `CAdminToolsFolder`, `CMyDocsFolder`, `CNewMenu`, `CSendToMenu`, `CPrinterFolder`,
  `COpenWithMenu`, `CNetFolder`, `CMenuBand`, `CFolderOptions`, `CDrivesFolder`,
  `CQueryAssociations`, `CShellLink`, `CUserNotification`, `CControlPanelFolder`, `CRecycleBin`,
  the folder-cohort anchor pair (2026-07-18) — **`CFSFolder`** (the file-system folder)
  and **`CDesktopFolder`** (the desktop root) — and the capability-unlocked pair (2026-07-18 PM):
  **`CDrvDefExt`** (drive-properties sheet; first `--local-raii` hand-verified RAII lowering),
  **`CMergedFolder`** (second menuband-rust shared-staticlib class; `IAugmentedShellFolder`
  promoted to `sdk/include/reactos/shobjidl_undoc.idl` so WIDL owns its vtable ABI), and
  **`CMoveToMenu` + `CCopyToMenu`** (first uses of the concrete-local-base inherited-layout
  mapper — the shared `CCopyMoveToMenu` base is flattened into each translation). Per-class
  history + the reusable integration playbook live in
  `docs/engineering/active-lanes/shell32-com-plan.md`. `CMenuBand` + `CMergedFolder` are
  shared-infrastructure (the `menuband-rust` staticlib linked by both shell32 and rshell).
  The final five (`CFileDefExt`, `CShellDispatch`, `CMenuSite`, `CMenuDeskBar`, and the inline
  `CStartMenuDummy`) landed on 2026-07-18. The activatable object map now has zero C++ entries.
  `CCopyAsPathMenu` was the first out-of-sample cheap-model
  class: two deterministic workers plus eight unchanged cached DeepSeek workers went from
  62 diagnostics per architecture to zero after deterministic support closure, then passed
  the full integrated three-architecture gate. rustconv now
  parses ATL COM maps, reads
  flattened interface slots/signatures from WIDL-generated or legacy `DECLARE_INTERFACE_` SDK
  headers, and generates interface structs,
  vtables, pointer recovery, QueryInterface/refcounting, the Rust factory, and ATL activation
  glue deterministically; the model sees method bodies only. Same-file free-helper closures are
  limited to internal linkage, selected transitively, overload-bound by proven-distinct arity,
  and renamed before prompting; wide literals are emitted as deterministic UTF-16 statics. Plain
  C scalar/pointer object state and `CComPtr<T>` ownership are now laid out and initialized
  deterministically; source-selected bridges expose required internal C++ helpers without
  asking the model to invent linkage. Exact prepared-body translation memory reused 19/22
  workers between the Fonts/Admin pair, while source-derived scope guards now restore
  body-local `CComPtr<T>` cleanup on every exit path. `CExeDropHandler` is the first
  scalar-state pilot, `CCopyAsPathMenu` is the first object-field RAII-state pilot, and
  `CMyDocsFolder` is the first six-interface class with a deterministic `FinalConstruct`
  activation/failure-cleanup path.
  This does **not** mean the DLL is wholly Rust: the audited live CMake lists still compile 70
  `.cpp` files (60,258 physical lines), including retained companions/bridges, private dynamic
  objects, shared shellmenu/shelldesktop objects, procedural helpers, and monoliths such as
  `CDefView.cpp`, `shlexec.cpp`, and `shlfileop.cpp`. The exhaustive zero-C++ plan and ledger are
  in `docs/engineering/active-lanes/shell32-cpp-tail-plan.md`.
  **Free-function TUs: 7 translated and PARKED, not wired** (2026-07-25) —
  `sdk/lib/rust/shell32-rust/src/shell32/{shlexec,shlfileop,brfolder,iconcache,shlfolder,stubs,
  shldataobject}.rs`, 21,626 lines, each carrying an `//! ORPHAN-BY-DESIGN:` banner. Five of them
  had been blocked behind seal defects that made the envelope unsatisfiable — a rejected worker
  assembles *no* output, so 5 rejects were withholding ~19.8K lines of already-paid Rust; see the
  field manual's `[§9.127]` RESOLVED note. They still need substrate closure (SDK externs, Win32
  constants, resource IDs) before they compile, so the C++ tail count above is unchanged: nothing
  is cohab-flipped yet. Paid artifacts — do not delete to tidy up.
- **Vendored, by policy (NOT targets):** ACPICA (~155 `.c`), FreeType (~208K LOC).

## Open items / flagged regressions

- ✅ **arm64 kernel stack-size bump — FIXED/re-landed** (`7e9d4644e53`, 12KB→48KB;
  `KERNEL_STACK_SIZE 0xC000` / `KERNEL_LARGE_STACK_SIZE 0x12000`, amd64 parity).
- ✅ **amd64 taskbar BUGCHECK 0x50 — FIXED** (2026-07-02, `d74739332d9`).
- **Live open bugs & flakes** are now tracked in one place — the roadmap **Known bugs &
  flakes registry** (`docs/product/vibeos-roadmap.md`) — with field-manual § pointers:
  the arm64 0x1E/`[WAITDEP]` ALL-ONES residual (§9.36 addendum), the winlogon 0x4C
  basesrv-population race, the win32k `DialogBoxParamW` race, and the amd64 start-menu
  render residue. (The old `triage.py` flake catalog was largely retired as its entries
  were root-caused — see the field manual.)

## Scale

~3.58M LOC of translated Rust across kernel + drivers + usermode chain, + ~1.43M LOC of
generated FFI (`-sys` bindgen: ntkrnl-sys 444K, win32k-sys 379K, winsrv-sys 192K, ntdll-sys
128K, … — declarations, not translation). The two *hot* monoliths dwarf everything for
edit-churn: **win32k 257K** and **the kernel** (see crate-split note). The driver wave is
big but cold: cdrom 211K, disk 170K, classpnp 166K, kernel32 161K, buslogic 138K, uniata
125K, scsiport/atapi/storahci ~120K each, fastfat 113K, freeldr 75K, cdfs 65K, storport 65K,
+ ~130 smaller crates.

## Kernel crate topology (build-architecture, not coverage)

`ntoskrnl-rust` is being split from one ~456K-LOC rlib monolith into per-subsystem crates to
cut the edit→rebuild loop (rustc frontend is whole-crate). **As of 2026-07-07 it is ~63%
extracted:** the monolith is down to **170K LOC**, with **~286K across 21 crates** (iomgr, se,
config, pnpmgr, ps, ob, fsrtl, kdbg, fstub, kd64, rtlk, cc, po, dbgk, lpc, kd, wmi, inbv, vdm,
vf, dbgprint) severed and talking over the FFI symbol boundary. Remaining inside the monolith:
the dense center **ke (58K) + mm/ARM3 (59K)**, plus **ex (41K) extracted-but-parked** on an
x86 boot hang. win32k (257K, flat single crate) is the planned follow-on. Full as-built +
recipe + forward plan: [`ntoskrnl-crate-split-plan.md`](ntoskrnl-crate-split-plan.md). This
is orthogonal to C-vs-Rust coverage — the linked `.exe` is functionally identical.
