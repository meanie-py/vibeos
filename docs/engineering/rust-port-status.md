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
| **network SERVICES** (`base/services/*`) | Cohab-flipped 2026-07-19, ~6.7K LOC, six FULL flips: **dnsrslvr** (`dnsrslvr-rust`), **wkssvc** (`wkssvc-rust`), **browser** (`browser-rust`), **wlansvc** (`wlansvc-rust`), **tcpsvcs** (`tcpsvcs-rust`), **telnetd** (`telnetd-rust`). dnsrslvr/wkssvc close the dnsapi/netapi32 MIDL loops from the row above. Only wkssvc/browser/wlansvc actually RUN at boot (hivesys `Start=2`); tcpsvcs/telnetd are `Start=3`, and **dnsrslvr has no hivesys service entry at all** — it ships to system32 but nothing starts it, so dnsapi's `R_Resolver*` client calls reach no server. That is a pre-existing upstream gap, NOT introduced by the flip. Field manual §9.112 (catch-all bindings recur in usermode services; the upstream cause is an incomplete bindgen island — allowlist the header owning any constant a `switch` dispatches on). | **full** |
| **network client DLLs** (Tier 3) | Cohab-flipped 2026-07-25, 56 functions for $0.50 through the deterministic-first loop: **wsock32** (`wsock32-rust` — the only compiled C was a `DllMain` returning TRUE, lowered deterministically for $0; the rest of the DLL is `.spec` forwarders `spec2def` turns into thunks), **wlanapi** (`wlanapi-rust`, 28/28 — the CLIENT half of `wlansvc.idl` whose server half is `wlansvc-rust`, closing that loop), **snmpapi** (`snmpapi-rust`, 27/27 — `AsnAny` is a tagged union whose `ASN_*` tags are macro EXPRESSIONS, so island completeness is a correctness property here, §9.112). `wlansvc_c.c` and the generated `*_stubs.c` stay C. Three integration bugs a green `cargo check` cannot see: `midl_user_allocate` is a **preprocessor rename** (`rpcndr.h:94` → `MIDL_user_allocate`, invisible to a source-reading translator, surfaces only as an unresolved symbol at link); a `DWORD`→`SIZE_T` cast that compiled on i386 and failed on amd64/arm64; and bindgen's `__BindgenUnionField` emulation for `AsnAny` (cleared by `bindgen_unions_to_real.py`). | **full** |
| **network client DLLs** (Tier 3 batch B) | Cohab-flipped 2026-07-25, 81 functions for ~$0.55: **mprapi** (`mprapi-rust`, 3/3 — RAS/router admin, near-all stubs), **netid** (`netid-rust`, 19/19 — the "Computer Name" system-properties page), **samlib** (`samlib-rust`, 58/58 + dllmain — the CLIENT half of `sam.idl`, whose server half is samsrv, closing that loop as wlanapi-rust does for wlansvc.idl). `sam_c.c` (generated MIDL client stubs) stays C; its 68 prototypes are **bindgen-derived** from `sam_c.h`, not hand transcribed. `netid_varargs_shim.c` is NOT a translation left in C — stable Rust cannot DEFINE a C-variadic, so it owns `FormatMessageBox`'s entry point and hands straight back to Rust with a pointer to the `va_list` (see `varargs-rust`). Bugs worth remembering: the dead `RpcExcept` handler shipped as LIVE code in 4 samlib functions, unconditionally clobbering the RPC status — the seal envelope checks token presence and multiplicity, never control-flow POSITION (§9.117); `SamLookupIdsInDomain` lost a level of indirection on `PSID_NAME_USE *Use` and would still have linked; and the `L` prefix of a wide literal was translated as CONTENT three separate ways. | **full** |
| **RAS client family** (Tier 3 batch C) | Cohab-flipped 2026-07-26, 81 functions for **$0.13**: **rasapi32** (`rasapi32-rust`, 58/58), **rasman** (`rasman-rust`, 7/7), **rasdlg** (`rasdlg-rust`, 9/9), **rasadhlp** (`rasadhlp-rust`, 7/7 across autodial/init/winsock). The cheapest batch yet because **66 of the 81 were lowered deterministically for $0** — Wine `FIXME(); return ERROR_NOT_SUPPORTED;` shims, 53 of them in rasapi32 alone. That is the stub-heavy end of the distribution, not a repo-wide rate (9% repo-wide, ~0% on real driver code); rasadhlp, the only one with real logic, paid for all 7. rasadhlp is the winsock NSP autodial hook — ws2_32 calls `WSAttemptAutodialAddr/Name` on a failed resolution and it IOCTLs `\Device\RasAcd` — so it rides the ws2_32 surface already translated. Its island is the first to use `REAL_UNIONS=1` (AUTODIAL_ADDR/AUTODIAL_CONN are tagged unions over an ANONYMOUS union). Bugs worth remembering: `RTL_CONSTANT_STRING(L"\\Device\\RasAcd")` came back with the `L` dropped — a UNICODE_STRING over ANSI bytes compiles, and only fails when it cannot open the device; and `nsp_dns.h` DEFINES its six service GUIDs in the header, so flipping winsock.c — the only TU that included it — left them undefined at link. | **full** |
| **wldap32** (Tier 3) | **TRANSLATED + GUARD-GATED, NOT YET FLIPPED — `cargo check` clean on all 3 arches (was 1087 errors).** 162 functions + wldap32.h's 62 `static inline` marshalling helpers, $2.05. In THIS tree wldap32 is a **stub DLL**: wine's config.h `#undef`s HAVE_LDAP (ReactOS does not vendor OpenLDAP), so every entry point compiles to `return WLDAP32_LDAP_NOT_SUPPORTED;`. The translation bought that disabled arm and emitted it UNCONDITIONALLY. **Fixed 2026-07-26** by carrying the C's guards across instead of deleting the arm: new `rustconv/cfg_gate.py` derives the boundary from the C (164 fns gated, 115 `#[cfg(not(...))]` twins) across the EIGHT `HAVE_LDAP*` probes config.h `#undef`s independently. Surfaced two silent bugs — a live §9.112 catch-all in `map_error` (all 16 `case` labels are ldap.h constants, so arm 1 bound everything and it returned SERVER_DOWN for every input) and 7 functions nested inside `ber_printf`'s body by an assembly-order slip. ber_printf/ber_scanf use the varargs-rust trampoline (`rust_varargs_shim.c`) since stable Rust cannot define a C-variadic. **Owed:** the CMake cohab flip + gate ×3. `profile_derive` warns `MOSTLY_DEAD_BODIES` before payment so the stub-DLL trap cannot recur silently. | **translated, compiles, unflipped** |
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

**Wave 18c (2026-08-01) — `errno.c`, and the two tool bugs it surfaced. 18/18 exports,
$0.14.** `msvcrt-crt-rust/src/msvcrt/errno.rs` — the file that owns `msvcrt_set_errno`, the
Win32-error → errno mapping `file.c` reaches from ~60 call sites. `_MSVCR_VER` is 0 here, so
the `>= 80` / `>= 140` blocks never compile and 18 is the whole file.

**It could not have been bought correctly the day before.** Two silent tool defects sat
directly on the path, both found by the mandated pre-payment loop and both fixed first:

* **`profile_derive` audited ZERO of errno.c's 42 `case` labels** and reported a clean bill.
  Its scan walks the unexpanded AST for `case_statement` nodes, and every arm here is
  `ERR_CASE(x)` / `ERR_MAPS(x, y)` — a function-like macro. That is the §9.112 catch-all
  detector missing its own case, and it is 31 TUs repo-wide, not one. 77 → 120 constants on
  msvcrt after the fix.
* **The scanner silently dropped `msvcrt_set_errno` and `_errno` from the paid catalog and
  put `ERR_CASE` — a preprocessor directive — IN it.** A bare `IDENT(ARG)` where a case
  label belongs is unparseable, and tree-sitter's recovery swallowed lines 132–451 into one
  `ERROR` node. `unextracted` was empty, so the plan looked healthy at 16 functions.
  `_expand_case_macros` now expands such macros before the parse; `preamble_audit` gained
  the same pass, having drifted from the catalog exactly as its own comment warned.

**And a third, one layer down: msvcrt-sys carried 13 of errno.h's 39 codes.** The list had
grown per-wave on demand, so 9 of `msvcrt_set_errno`'s 18 targets were absent — including
`EDEADLK` (36 in msvcrt, **35 in glibc**) and `ENOTEMPTY` (41 vs 39). Nothing would have
failed to compile; the model would have supplied a plausible host value. The allowlist now
carries errno.h's whole non-POSIX family.

With all three fixed, sealing did what it exists for: `msvcrt_set_errno`'s 60 constants went
to the model as `__RUSTCONV_FACT_00NN__` and came back as 42 correctly-grouped arms, with no
`unreachable_patterns` warning on any arch.

**The error-string table is GENERATED, not translated** —
`tools/gen_msvcrt_errlist.py`. Two of the 40 strings came back sized to the text with **no
NUL** (`str_ENOTTY`, `str_ENOSYS`), which `strerror`'s `strcpy` and `perror`'s `strlen` read
off the end of; eleven more were padded with surplus NULs. C sizes a `char[]` from a literal
to include the terminator and a Rust byte-array literal does not — and `--seal-facts`
protects CONSTANTS, so a string literal is outside it. Pure data should not be a claim.

**Grading status, stated precisely.** The subject's errno storage moved out of
`subjectimpl.c`'s two file-static `__thread` variables into `msvcrt_get_thread_data()`'s
`thread_errno`/`thread_doserrno`, where the Rust reads it — so file.c's 32 existing rows now
exercise the REAL mapping on the subject side while the reference still uses `fileimpl.c`'s
transcription. **That half-closes the shared-substrate blindness those rows had**: the
mapping is differential for the first time. It is not yet CLOSED — the reference's copy is
still hand-written, and errno.c's own 18 exports have no rows. Both need errno.c as
reference TU #7.

`_invalid_parameter` is live in msvcrt.dll on all three arches and excluded from the host
link by the `invalid-parameter` feature, the same shape and the same reason as `stdio`:
`subjectimpl.c` defines the plain name as the harness's CALL COUNTER, which 42 assertion
sites read as `Effects.pmt`. At `_MSVCR_VER=0` the handler slot can never be written
(`_set_invalid_parameter_handler` is in the uncompiled `>= 80` block), so the Rust body and
the harness's are identical in effect and nothing is hidden.

**Wave 18c part 3 (2026-08-02) — file.c is GRADED END TO END. 104 rows, 22,180 cases, 0
DIVERGE / 0 UNEXERCISED.** file.c's graded spec exports **116 → 146 of 146**. The last
thirty were left for last because each of the three families needed substrate that did not
exist, and both pieces of it are reusable.

**The eleven std-stream aliases needed each side to OWN its standard streams.** `putchar`
is `fputc(c, stdout)` and takes no FILE, so the only way to compare is for `stdout` to be a
different file on each side. It half was already — `stdout` is `&MSVCRT__iob[1]`, and the
`_iob` rename wall gives each side its own array — but the fd underneath was NOT:
`msvcrt_init_io` binds CRT fd 1 from `GetStdHandle`, which the substrate answered with host
fd 1. Both sides were writing into the harness's own stdout, interleaved with each other
and with `println!`. `ref_set_std_fds` (fileimpl.c) points the three standard handles at
caller-supplied host fds, called once per side immediately before that side's init — the
only reader of `GetStdHandle`. **Not `freopen`**, which reaches the same place by CLOSING
host fd 1; the next `open` would then hand fd 1 to a temp file and silently swallow every
`println!` the harness makes.

**The sixteen `v*printf` shells needed a THIRD instantiation of file.c.** They are not
callable in the `ref_*` one and never were: every one forwards `&valist` to the engine as a
`va_list*`, coherent only where a `va_list` IS a plain pointer, so under the host's SysV
register-save-area model `*valist` reinterprets the decayed pointer variable as the tag
struct and reads wild memory. That is the same diagnosis `pfflat_` was built for on wcs.c,
and the fix is the same shape: `fileflat_*` is file.c compiled again under refstub_printf's
flat `char*` `<stdarg.h>`, which is the Windows x64 / aarch64 / i386 model — both the
callable instantiation and the target-faithful one. build.rs's file prelude is now a
closure over the prefix, and the stage-5 protos header is re-emitted with its thirteen
`__asm__` labels retargeted (an asm label is a string baked into the header, so the second
instantiation cannot share it). The `%I64d` and `1.000000e+010` cases in the corpus are the
oracle-honesty canary: glibc prints `I64d` literally and a two-digit exponent.

Two properties of the flat TU are by construction rather than oversight, and the rows are
built around them: its four variadic openers (`_open`, `_wopen`, `_sopen`, `_wsopen`) are
broken, because the flat `va_start` yields NULL — so the rows reach its stream layer
through `_sopen_s` + `_fdopen`, neither of which is variadic. And `add_std_buffer` is
unreachable on any redirected stream (it requires `_isatty`), which is the same answer a
Windows CI job with redirected output gets.

`_lock_file`/`_unlock_file` have two arms and neither is visible in a return value — both
are `void`. The critical-section arm leaves `RecursionCount` in the FILE's own memory; the
global-lock arm leaves nothing, so fileimpl.c's `_lock`/`_unlock` now record the slot
number they were handed (`ref_get_last_lock`, same read-it-immediately discipline as
`ref_get_last_share_mode`). The number is `_STREAM_LOCKS + index`, so it catches a wrong
index as well as a wrong arm. Reaching `RecursionCount` went in as `(f as *const
u8).add(48 + 12)` and **`scan_magic_offsets` rejected it, correctly** — a hand-computed
struct offset is the same claim whether it is in shipping code or in the harness that
grades it. It is a `#[repr(C)]` `FileCrit { file: Iobuf, crit: Crit }` now, with the sizes
`const _: () = assert!(…)`ed to match msvcrt_crt_layout.h's `_Static_assert`s.

**Wave 18c part 2 (2026-08-02) — the temp-name family, and THREE more shipping bugs. 93
rows, 19,960 cases, 0 DIVERGE / 0 UNEXERCISED.** file.c's graded spec exports **104 → 116 of
146**. The twelve exports split on whether their names are comparable at all, and the split
is the design: `tmpnam` and `_mktemp` build from `GetCurrentProcessId()` plus a per-side
static counter and PROBE for a free name without creating one, so the two sides must produce
identical strings and the rows compare them byte for byte; `tmpfile`, `tmpfile_s`, `_tempnam`
and `_wtempnam` reach `GetTempFileNameW`, whose shim is `mkstemp` — random suffix, reserved
by creation — so they cannot receive the same name even in principle, and those rows compare
the class of answer, the errno, a write/read round trip, and (for `_tempnam`) that the `TMP`
override beats the `dir` argument.

**Three bugs, none of them a compile error, two of them sitting beside a CORRECT translation
of the same C statement:**
* **Five `L"..."` literals were byte strings cast to `*const wchar_t`** — `b"TMP\0"` is four
  bytes, which as `wchar_t` reads `0x4D54, 0x0050, <rodata>`. `_wtempnam`'s TMP override
  never fired (so `tmpfile()` failed on every call, using the literal `","`), and
  `msvcrt_get_flags`' four `ccs=` comparison strings were all garbled — closing the entire
  `_wfopen(path, L"rt,ccs=UTF-8")` Unicode-mode door with EINVAL. Now a `wlit!` that encodes
  at compile time and COMPUTES the length. kernel32-rust already carried comments warning
  about this shape.
* **`msvcrt_int_to_base32_w` never shifted** — `let num = num_local;` is a fresh binding
  scoped to the loop body, so every base-32 digit came from the same value (`\s1lu6v.`
  became `\svvvvv.`). The narrow twin, thirty lines up, gets the same C right.
* **`tmpnam_helper`/`wtmpnam_helper` compared a `size_t` as a truncated `i32`** — the C
  promotes to unsigned; `tmpnam` passes `(size_t)-1`, whose low 32 bits are −3, so the guard
  fired always and `tmpnam` returned NULL/ERANGE on every call. The second bounds check in
  the same function was already correct.

Also closed a shim-fidelity gap the first bug exposed: `WideCharToMultiByte` dereferenced a
NULL source where real kernel32 returns 0 with ERROR_INVALID_PARAMETER, turning msvcrt's
graceful failure into a SIGSEGV that reads like a translation crash and is not one.

**Wave 18c part 1 (2026-08-02) — timestamps and the process-global stream ops. 89 rows,
19,080 cases, 0 DIVERGE / 0 UNEXERCISED.** file.c's graded spec exports **93 → 104 of 146**.
The utime family is six exports over one body (`_futime64` does the work; the path and `*32`
variants delegate), so the axis that matters is the VALUE — whether a time survives
`time_to_filetime`, the 100ns-since-1601 conversion, `SetFileTime` and the read back through
`_stat64` unchanged. Graded across the epoch, 1, and `INT32_MAX`. **No NULL-`t` case**:
`_futime64` stamps `_time64(NULL)` for it, so the two sides read their own wall clocks and
straddle a second boundary at random — a flake, not a test. `_flushall` and `_fcloseall`
return COUNTS, which makes them the only rows that assert the two FILE rings have stayed in
step; `_setmaxstdio` restores what it found, because every later row's `msvcrt_get_file`
reads it. The 42 still ungraded are the std-stream aliases, the `v*printf` shells, the
temp-name family, and `_lock_file`/`_unlock_file`/`__iob_func`.

**Wave 18b (2026-08-01) — the wide door, stat, and the rest of the plumbing. 83 rows,
17,760 cases, 0 DIVERGE / 0 UNEXERCISED, both profiles.** file.c's graded spec exports went
**36 → 93 of 146** (the commit message says 40 → 95; that count matched export names
anywhere in the test file, including the section banners that merely NAME what a family
covers. Measured against comment-stripped code it is 36 → 93). 18a took the narrow fd and stream layers and nothing else, which left the
entire UTF-16 surface ungraded — and that is backwards from how it reads: msvcrt IMPLEMENTS the
wide call and reaches Win32's `*W` directly, while the narrow entry points convert and delegate.
The wide side is where the bodies are. Added: the wide path (`_wfopen`/`_s`, `_wfsopen`,
`_wfreopen`/`_s`, `_wsopen_s`, `_wcreat`, `_wfdopen`, `_wremove`, `_wunlink`, `_wrename`,
`_waccess`/`_s`, `_wchmod`), the narrow openers 18a missed (`_fsopen`, `freopen`/`_s`,
`_sopen_s`, `_creat`, `_chmod`, `_access_s`, `_fdopen`), the nine-export **stat family**, the
64-bit positioning twins, descriptor plumbing (`_dup2`, `_pipe`, the fd↔HANDLE pair, `_locking`,
`_umask`, `_chsize_s`), the wide stream characters, and `setvbuf`/`setbuf`/`_filbuf`/`_flsbuf`/
`getc`/`putc`/`_getw`/`_putw`/`clearerr_s`.

**One genuine shipping bug**: `_fstat*` reported `st_mode` `0o776`. `file.c` computes
`_S_IFREG | 0444` and the translation carried the literal across verbatim — C reads a leading
zero as octal, Rust as part of a **decimal** literal, with no warning because `0444` is a valid
way to write 444. All three arches, six exports. A repo-wide sweep for the class found zero
other instances.

**And the harness's most important tripwire was off.** `cc::Build::warnings(false)` emits `-w`,
which DISARMS `-Werror=` diagnostics outright — in either order, and even with the warning
re-enabled afterwards. build.rs carried a comment saying "NO `-w` here, on purpose" directly
below the call that was emitting one, on ten builders. With it armed the reference TUs had **57
implicitly-declared functions**, typed `int(...)`, truncating every returned pointer to 32 bits;
it survived only because `fd_to_handle` hands back small integers. `msvcrt_wstrdupa` returns a
real malloc pointer and SIGSEGV'd the first row to reach it. `refstub/msvcrt_shim_decls.h` now
declares that surface and fileimpl.c includes it, so a drifting shim is a compile error.
`BUFSIZ` joined the DISPLACE set beside `_IONBF` (msvcrt 512, glibc 8192 — `setbuf` passes it
straight to `setvbuf`). Two shim inputs that were silently DISCARDED — `dwFlagsAndAttributes`
and `dwShareMode` — are modelled and recorded respectively; see the field manual's §9.73 entry
for that class. The 51 exports still ungraded (std-stream aliases, the `v*printf` family,
temp names, process-global stream ops, utime) are wave 18c.

**Wave 18a (2026-08-01) — file.c GRADED. `msvcrt-difftest/tests/file.rs`, 32 rows,
7,040 cases, 0 DIVERGE / 0 UNEXERCISED.** Wave 17 shipped file.c's 189 exports with no host
oracle at all: the flip was gated by a link, `scan_ud2` and three boots, none of which can see a
return value that is right by one or a byte written to the wrong offset. This is that
instrument, and it covers the fd layer (`_open`/`_read`/`_write`/`_lseek`/`_lseeki64`/`_eof`/
`_filelength`/`_filelengthi64`/`_dup`/`_commit`/`_chsize`/`_setmode`/`_isatty`/`_close`), the
stream spine (`fopen`/`fopen_s`/`fclose`/`fread`/`fwrite`/`fgetc`/`fputc`/`fgets`/`fputs`/
`ungetc`/`fseek`+`ftell`/`rewind`/`feof`+`clearerr`/`fflush`) and the path layer (`remove`/
`_unlink`/`rename`/`_access`). The rest of the 189 are owed.

Each row runs over TWO real files, one per side, and the compared image is the scalars (return,
errno, **doserrno**, invalid-parameter count) + the CRT's own state (position, EOF/error flags,
fd number, and the FILE's `_flag`/`_cnt`/`_bufsiz`/`_ptr - _base`) + **the file on disk, read
back through `std::fs`** rather than through the implementation that wrote it. Four substrate
changes were needed before any of that could mean anything, all of them in the same shape — a
shared thing that quietly joined the two sides:

* the fd table, `_iob` and `__badioinfo` were un-renamed (both implementations shared ONE table);
* `msvcrt_set_errno` had one definition writing through the reference's `_errno()`, so the
  subject's failures were invisible *and* clobbered the slot the next reference call was graded
  on — the store is per-side now, the mapping table still one transcription (retires with
  errno.c as a reference TU);
* `fileimpl.c` called POSIX `rename`, which became msvcrt's once file.rs was in the link →
  unbounded recursion. Now `renameat`/`dprintf`, and build.rs intersects that object's undefined
  symbols with file.c's exports so the next one fails at build time;
* `_IONBF`: at `_MSVCR_VER < 140` msvcrt.h aliases `MSVCRT__NOBUF` to it, and the reference was
  taking **glibc's `2`** — bit-identical to `_IOWRT` — so it treated every write stream as
  unbuffered. Derived and displaced now. Field manual has both this and the `rename` class.

**One genuine shipping bug found and fixed** (`file.rs` `_fread_nolock`): the C clamps `i` with
`if (i > rcnt) i = rcnt;` and that clamped `i` is what the loop tail credits to `pread`,
subtracts from `rcnt` and advances `ptr` by. The translation put the clamp in a separate
`actual_i` used only for the memcpy, so a buffered read smaller than the buffer reported the
whole BUFFER FILL as bytes delivered — `fread(p, 1, 1, f)` on a six-byte file returned 6, moved
the caller's pointer six bytes past one byte of data, and underflowed `rcnt`. It survived the
boot because the callers on that path ask for at least a buffer's worth and never take the
branch. Two more rows (`fseek+ftell`, `feof+clearerr`) were downstream of it and went green with
the fix.

**Wave 17 (2026-07-31) — `file.c`, the whole stdio layer. 189/189 exports, 5,897 LOC**, the
single largest remaining C file in msvcrt and 26% of what was left. `msvcrt-crt-rust/src/msvcrt/
file.rs` defines 173 `#[no_mangle]` bodies; the other 16 are the variadic entry points below.

**The flip is `pub mod file;` and nothing else** — worth stating because it misled me. msvcrt's
cohab is by LINK ORDER, not by commenting the `.c` out of CMake the way the driver flips do:
`libmsvcrt_crt_cargo.a` is linked ahead of `msvcrt_static`, so lld first-match takes the Rust
symbol and file.c's copy goes dead in place under `--allow-multiple-definition`. file.c is still
compiled and still listed in `dll/win32/msvcrt/CMakeLists.txt`; it is simply no longer reachable.
No CMake edit was owed for the flip and none was made.

It had to go in **atomically**, and did. The general cohab property that makes a partial file
safe — a function the Rust does not define just stays C — is *false for this file*: its entry
points share one fd table and one FILE ring, so a C `fopen` beside a Rust `fclose` would be split
state rather than a partial port.

The 20 `...` entry points are `dll/win32/msvcrt/rust_file_varargs_shim.c` — 16 printf shells plus
`_open`/`_wopen`/`_sopen`/`_wsopen`, whose `...` carries an optional `pmode` rather than a format.
Same language limit and same placement as wcs.c's and scanf.c's (no `va_start` on stable; in the
MODULE, not `msvcrt_static`, so they win first-match; in their own TU because **symbol replacement
flips a TU's callers, not its housemates**). `tools/scan_varargs_shim_parity.py` now diffs all
three pairs — 90 shells, 0 findings — and covering file.c is what exposed that the scanner had
been walking past a leading `va_arg` to find the delegate, which it now skips explicitly.

**An i386-only ABI bug, caught by the first link and worth the class it belongs to.** The port
called `GetLastError` 39 times through a hand-written `extern "C"` block in `prelude.rs`.
`GetLastError` is WINAPI — stdcall on i386, cdecl elsewhere — so that block is correct on
amd64/arm64 and emits `_GetLastError` with no `@0` on i386, where nothing defines it. It sat
three lines under a comment saying that getting this convention wrong is the project's #1
historical i386 crash cause, and directly contradicted the `msvcrt-sys` allowlist comment that
these signatures are "DERIVED from the real headers and never hand-written as `extern` blocks in
the consuming crate." Fixed where it belonged: `GetLastError` joined the island allowlist, so
bindgen emits `extern "stdcall"` on i386 and `extern "C"` on the other two, from winbase.h, with
no judgement call left to make. This is the sibling of wave 16's `_isw*_l` finding — that one was
*never `extern`-declare a symbol your own crate defines*; this one is *never `extern`-declare a
WINAPI import by hand*. Both are invisible to `scan_extern_arity`, since the arity matches.
An audit of every other `extern "C"` block in the crate found no second instance.

**Then four `ud2` traps, which the first *successful* link surfaced.** `scan_ud2` flagged
`_fcloseall`, `_fflush_nolock`, `_rmtmp` and `msvcrt_free_io` in msvcrt.dll. It files unexpected
driver `ud2` under §9.99 (the CC/sig-mismatch fold), but the disassembly said otherwise — `call
panic_bounds_check` immediately before the trap, so this was the *array-index panic* class. One
root, inherited by the other three through inlining: `msvcrt_get_file` guards
`i >= MSVCRT_max_streams` before indexing `MSVCRT_fstream[i/BLOCK]`, and `MSVCRT_max_streams` is
a runtime `static mut`, so the bound is unprovable and the check survives. The index is in fact
always in range — `_setmaxstdio` refuses any `newmax > MSVCRT_MAX_FILES` — but that invariant is
global and nothing local can see it. Fixed with pointer arithmetic matching the C, behind one
documented helper. The control that made the diagnosis cheap: `get_ioinfo_nolock` indexes an
identically-shaped array and emitted no check at all, because *its* guard is the constant
`MSVCRT_MAX_FILES` — so the difference is provability, not safety, and only the variable-guarded
site needed touching. `msvcrt_free_io` mattered most: it runs at process exit for everything that
links this DLL.

That cleared `msvcrt_free_io` and left three, on a **second and different** shape — the
disassembly had changed to `call __errno` then `ud2`, with no panic symbol. `msvcrt_get_file`
returns NULL on `calloc` failure, and `_fcloseall`, `_rmtmp` and `msvcrt_flush_all_buffers` all
dereference the result immediately without checking, which is exactly what file.c does. Inlined,
LLVM sees NULL reach a deref, folds the path to `unreachable`, and emits one trap per caller.
The fix is `#[inline(never)]` on the callee, **not** a null check: adding the check would be a
behaviour change on the path the C faults on, i.e. a divergence in a differential port, while out
of line the caller emits the ordinary load and faults exactly where C faults. The path is
unreachable regardless — those callers walk `3..MSVCRT_stream_idx` and such slots are allocated
at stream-creation time, so the `calloc` arm never runs with those indices. Both shapes, and the
one-look disassembly tell that separates them from the genuine §9.99 CC/sig fold that `scan_ud2`
reports them all under, are now a table in the field manual.

**And then the one that actually broke the OS: an inverted compare-exchange.**
`alloc_pioinfo_block` publishes a newly-allocated fd block into `MSVCRT___pioinfo` with a CAS,
and the C frees the block only when it *loses* the race —
`if (InterlockedCompareExchangePointer(&pioinfo[i], block, NULL)) { …; free(block); }`, because
the CAS returns the PREVIOUS value and non-NULL means someone else published first. The
translation rendered the condition as `!= block`, which reads plausibly and is backwards: the
winning path returns NULL, so **every successful allocation freed the block it had just
published**. Every fd block was a dangling pointer, critical sections and all, and i386 wedged
inside `LdrLoadDll` — deterministically, 3 boots for 3 — at the first msvcrt load of the boot.

Two things about how it was found are worth keeping. The search space collapsed on one
observation: **basesrv and csrsrv do not import msvcrt and winsrv does** (via shell32/ole32), so
the wedge was the first msvcrt initialization in the whole boot and the frame was `DllMain` →
`msvcrt_init_io` → the two functions under it. And the bug is invisible to everything upstream of
the boot — single-threaded it never manifests, and `cargo check` cannot see it because both
spellings are well-typed pointer comparisons. Cleared along the way, so nobody re-walks them:
`_lock_file`/`_unlock_file` pairing, `init_ioinfo_cs`, the `_STREAM_LOCKS`/`_IOB_ENTRIES`
constants (28/20/47/48 per `mtdll.h` — ucrt's `_IOB_ENTRIES 3` is not on msvcrt's include path),
every `_MSVCR_VER` gated helper arm, and the `_iob` type arm. All faithful.

Substrate closure ran to 0 errors on all three arches before any of this: the island grew from 3
functions to 50 and 3,032 constants to 3,790. Bugs the closure surfaced that would have compiled
and booted while being wrong are in the field manual — the `tmpnam_helper` counter (the C calls it
with two different ones, and a shadowing fix silently redirected `tmpnam_s` to the wrong one), the
`_stat` time width (`time_t` is 32-bit on our i386 and 64-bit elsewhere; i386 alone reported zero
errors with the wrong cast baked in), `Interlocked*` having no symbol to import at all in usermode,
and a `LARGE_INTEGER.HighPart` reached by pointer arithmetic rather than by its union field.

**Not yet graded, and the substrate is FURTHER ALONG than task #20 recorded.** msvcrt-difftest
already compiles the genuine file.c as a full reference TU — rename wall, an asm-label escape
list for the 19 `#undef`s that would otherwise silently delete a `#define` rename, and generated
prototypes (it has to: file.c owns 23 symbols the other reference TUs call, four of which would
bind to GLIBC and convert through the HOST locale — a wrong oracle that links, runs and says
nothing). What is missing is grading ROWS, not the substrate. The boot is the only oracle file.c
has today, which is exactly how the inverted CAS above reached a wedged OS.

**Two things are owed before those rows can exist**, both discovered when the flip bounced out of
the merge queue as `DIFFTEST_FAIL` — which was not a divergence at all but ten duplicate symbols
at link:

1. **`tests/scanf.rs` owns the plain stdio names.** It DEFINES `fgetc`/`fgetwc`/`ungetc`/
   `ungetwc`/`_lock_file`/`_unlock_file`/`__iob_func`, and those are not link-satisfying stubs —
   each calls `model_get`, so they *are* the harness's character source, injected through the
   plain `extern "C" { fn fgetc(...); }` seam in the shipping `scanf.rs`. Once file.rs defines
   them too, both land in one test binary. Interim: msvcrt-difftest takes msvcrt-crt-rust with
   `default-features = false`, dropping the new `stdio` feature (file.rs) for the HOST build
   only — all three arches keep it, and `libmsvcrt_crt_cargo.a` was checked with `llvm-nm` to
   confirm `__fcloseall`/`__rmtmp`/`_msvcrt_init_io`/`__lseeki64` are still in the shipping
   archive. Removing this interim means giving scanf's FILE rows real FILE-backed stdio.
2. **The reference rename wall covers FUNCTIONS only.** `object_text_symbols` filters `llvm-nm`
   to `p[1] == "T"`, so file.c's file-scope DATA — `MSVCRT___pioinfo`, `MSVCRT__iob`,
   `MSVCRT___badioinfo` — was never renamed into `ref_*`/`msvcrt_ref_*` and collides with the
   subject's copies the moment file.rs is linked for the host. Latent until now because nothing
   else defined them. Widening the wall to data symbols is a prerequisite for (1), not optional.

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
check, and `wcsrtombs_s_l`'s genuine `mbstr[-1]` touch at `conv == size == 0` — that last one
caught only by the harness's **guard prefix**, since a compare that stops at the NUL cannot see
a write before the buffer.

**`wctob`'s strict `< 0xff` is NOT one of them, and the distinction is worth keeping.** It looks
like a preserved quirk and it is unfalsifiable: the only input that could distinguish `<` from
`<=` is U+00FF, where `<` returns `EOF` (-1) and `<=` returns `(signed char)0xFF` — also -1.
Relaxing the comparison moves no row and *cannot*. A differential test can pin the row (it is
exhaustive over all 65,536 values × 5 modes, and a control returning the raw value instead of
the signed char is caught at once) but it cannot pin that comparison, so nobody should read a
green `wctob` as evidence about it.

**Grading: 122 rows, 122 MATCH, 0 DIVERGE, 0 UNEXERCISED, ~2.0M cases** (`msvcrt-difftest/tests/wave16.rs`),
under both the "C" and a real NLS locale. Nine negative controls, eight fired; the ninth is the
`wctob` non-result above. The `_wctomb_s_l` ERANGE arm's reachability is **measured, not
assumed** — a probe showed it at **0 of 0 hits** because the harness's `WideCharToMultiByte`
model truncated where real kernel32 returns 0 + `ERROR_INSUFFICIENT_BUFFER`; the arm was
ungraded in every mode while every row read MATCH. Model corrected (scoped to single-character
conversions), arm now 1299/1299, probe kept permanently so a regression names its own cause.

**A real ABI bug in already-landed code fell out of it.** `prelude.rs` declared the nine
`_isw*_l` predicates taking `wint_t` (`unsigned short`) while this crate DEFINES all nine taking
`u32` — so every call through the prelude wrote 16 bits into an argument read as 32, upper half
undefined per the Windows x64 ABI, benign only because rustc happens to zero-extend.
`scan_extern_arity` cannot see it: the arity matches. Field manual carries the class under
ABI/FFI — never `extern`-declare a symbol your own crate defines.

**Wave 15 (2026-07-26) — `scanf.c`: 16 template instantiations collapse to ONE Rust engine.**
`scanf.h` (743 lines) is `#include`d sixteen times under WIDE_SCANF/SECURE/STRING/STRING_LEN/
CONSOLE, generating ~9,600 lines of C; `msvcrt-crt-rust/src/scanf.rs` (1,514 lines) is one engine
generic over `W: Width`, `S: Source` and `const SECURE: bool`, calling the crate's existing Rust
`fpnum_parse`/`fpnum_double` so `%f` is correctly rounded and shares the bignum with strtod and
printf. The 40 `...` shells are C trampolines in `rust_scanf_shim.c` — and here the MODULE
placement is REQUIRED, not merely preferable: scanf.c's 16 engines are `static`, so there is no
symbol for a Rust engine to win by first-match.

Graded by `tests/scanf.rs` — **36 rows, 439,100 cases, 0 DIVERGE, 0 UNEXERCISED**, nine controls,
nine fired. It found a **real shipping bug**: the port snapshotted the float parser's cursor once,
where scanf.h re-assigns `ctx.file = file` AFTER the whitespace skip, so `swscanf(L" 1.50",
L"%g", &f)` returned **11.5** and a stale cursor could turn a valid parse into an EOF return.
Invisible on 8 of the 16 instantiations (FILE/CONSOLE contexts carry a `FILE*` or a ZST, which a
cursor skip cannot move), so nothing else — cargo check, the DLL link, the 3-arch build, the
parity scanner — could see it.

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
  objects, shared shellmenu/shelldesktop objects, procedural helpers, and the remaining
  monoliths — `CDefView.cpp`, `dialogs.cpp`, `filetypes.cpp`, `utils.cpp`, `shell32.cpp`.
  (`shlexec.cpp` and `shlfileop.cpp` left that list on 2026-08-01 and `dialogs/drive.cpp` on
  2026-08-02; the tail is now 45 `.cpp` / 40,798 lines, tracked by
  `rustconv shell32-tail-inventory`.) The exhaustive zero-C++ plan and
  ledger are in `docs/engineering/active-lanes/shell32-cpp-tail-plan.md`.
  **Free-function TUs — the island-first recipe is proven; 2 wired, 7 still parked** (2026-07-26).
  `dialogs/general.cpp` and `dialogs/recycler_prop.cpp` are cohab-flipped and green on all three
  arches: they are the first shell32 TUs to take their substrate from the **bindgen island**
  (`crate::prelude`, which globs `crate::island`) rather than the hand-written
  `types.rs`/`win32.rs`. That is the whole unlock — measured on these two files, globbing the
  island *instead of* the hand-written substrate dropped unresolved names from **228 to 77**,
  and the rest was a per-TU tail (bitfield accessors, C-uninitialised locals, variadic promotion,
  nominal alias splits) rather than a substrate wall. Recipe and every trap: prelude.rs's module
  header plus the field manual's "Adopting an island in a crate that already has hand-written
  substrate".
  **The C++ SUBSTRATE gate is now built** (2026-07-26). `stubs.cpp` and `shldataobject.cpp` are
  also flipped, and the blocker behind the remaining five was one shared gap, not a per-file
  tail: `CComPtr` appears in **25 of the 38** still-C TUs and `CComHeapPtr` in 19. Two new
  modules close it — `shell32-rust/src/atl.rs` (CComPtr / CHeapPtr / CComHeapPtr / CSimpleArray
  and the CRT/Local/COM allocator policies, with `Drop` restoring the destructors the
  translation dropped) and `shell32-rust/src/shellutils.rs` (the `shellutils.h`/`utils.h` header
  inlines, which live in no TU and so reach no translated file). Both are written against
  *measured* usage in the emitted Rust; `atl.rs`'s header records the three invariants the call
  sites depend on. Neither is shell32-specific in spirit — any ATL-using C++ module wants them.
  `shlfolder.cpp` is flipped too (2026-07-26). It needed the widest `EXTERN_C` surface of any
  shell32 TU so far — 19 of its 83 mangled definitions have live callers (the other ~64 are
  COMDAT template/inline emissions each TU makes for itself), 18 declared in `shfldr.h` and 2 in
  `precomp.h`. One of the 19 was an OVERLOAD PAIR of `SHELL32_CoCreateInitSF`, which `EXTERN_C`
  cannot express, so the csidl-taking form is now `SHELL32_CoCreateInitSFWithCsidl` on both
  sides; its C++ `REFIID`/`const GUID&` parameters were never an obstacle (`extern "C"` changes
  linkage, not the type system). The `Shell32RustCoCreateInitSF` trampoline that five already
  translated folder classes call stays — it is `WINAPI` where the renamed function is cdecl, so
  repointing it is an ABI change for no gain — but `cpp_com.py` and `scan_extern_arity` both
  needed fixing alongside; see the field manual's `EXTERN_C` entries.
  `iconcache.cpp` is flipped (2026-07-31), and it is the cheap counter-example to shlfolder: its
  four shell32-internal exports needed **no** `EXTERN_C` at all, because `wine/shell32_main.h`
  already wraps itself in `extern "C"` for C++. What it needed instead was three things the
  island cannot supply — the function-local `static int s_imgListIdx` (C++ gives it a hidden
  global; the translation kept the uses and dropped the storage), a local mirror of
  `RTL_CRITICAL_SECTION_DEBUG` whose trailing union is pinned to the `WineDebugString` pointer
  arm, and `SHELL_OsIsUnicode`, a header inline ported into `shellutils.rs`. `MAKELONG`,
  `MAKEROP4` and `ImageList_AddIcon` joined the prelude. Two link failures closed it: three
  globals `wine/shell32_main.h` publishes needed `#[no_mangle]`, and `brfolder.cpp` declares
  `Shell_GetCachedImageIndexW` bare at its own file scope — a declaration outside any header,
  which a header-only grep misses.
  `brfolder.cpp` is flipped (2026-07-31), 86 errors → 0. One root was shared substrate and pays off
  beyond this file: the prelude declared `SetWindowLongPtrW`/`GetWindowLongPtrW` as
  raw externs when on i386 winuser.h makes them macros for the non-`Ptr` entry points, so they
  are now cfg-split wrappers that `layout.rs` and `shlfileop.rs` inherit. A second candidate
  root was REJECTED and is worth knowing about: `shellcom.rs` duplicates the island's
  `ITEMIDLIST`/`SHITEMID`, and collapsing that to a re-export passes `cargo check -p
  shell32-rust` on all three arches while breaking `menuband-rust`, which `#[path]`-includes
  the file and has no island. The duplication is deliberate; the crossings `.cast()` instead.
  The rest were defects
  a green C++ build cannot show: amd64 widths baked into `LRESULT`/`UINT_PTR`/`SIZE_T` casts
  (i386-only failures), `isize` where i386 wants `INT_PTR`/`LPARAM` (same width, different Rust
  type), one of two `BrFolder_GetName` **overloads** dropped, a C++ **default argument** lost at
  its call site, C's `!` on a BOOL translated as Rust's **bitwise** NOT, and a `COleInit` RAII
  local left as an uninitialised binding so neither `OleInitialize` nor `OleUninitialize` ran —
  silent, and the reason the C++-construct checklist is now in the field manual. Link closure
  needed `EXTERN_C` on `SHELL_GetIDListTarget` (declared in `precomp.h`, `__cdecl` — so the Rust
  side is `extern "C"`, not `extern "system"`) and `_RustBridge` shims for `SHGetAttributes` and
  `GetDfmCmd`, matching the ones `utils.cpp` already had.
  **Nothing is parked any more — all six free-function TUs are flipped** (2026-08-01).
  `shlfileop.cpp` (2,868 lines of Rust) and then `shlexec.cpp` (3,900) closed the set; the
  `CStringW`-needs-a-variadic-`Format` blocker that had deferred both turned out to be wrong
  for shlexec (zero `Format` calls) and solvable for shlfileop (the 7 sites are CONCRETE
  arities — `_scwprintf` to size, `swprintf` to print — not one variadic macro). What each
  file was hiding is in that directory's `mod.rs`, beside its `pub mod`; the pattern across
  all six is that the defects cluster wherever C's type system declines to have an opinion —
  a `sizeof` folded away, `!` on an int, `SUCCEEDED` on an unsigned cast, a `goto` that jumps
  to itself, a `BOOL` narrowed to a byte, a `#define` that renames its callee, an fn-pointer
  typedef given a calling convention the C never had. Every one compiled clean in C and
  shipped. Field manual: "Semantic rewrites that flip meaning", "Element count vs byte count",
  "A file-local fn-pointer TYPEDEF given a convention the C never had", "Phantom symbols".
  `dialogs/drive.cpp` is flipped (2026-08-02) — the format-drive dialog, 786 lines, and the TU
  that made the **island grow a disk IOCTL surface**. `ioapiset.h` and `psdk/winioctl.h` were
  missing from the shell32 allowlist in `gen_driver_types.sh`, so `DeviceIoControl` and every
  `GET_LENGTH_INFORMATION` / `DISK_GEOMETRY` / `PARTITION_INFORMATION{,_EX}` /
  `PARTITION_STYLE_*` a translation needs resolved to nothing. Adding the two headers is the
  fix; the four `IOCTL_DISK_*` codes still have to be COMPUTED in the prelude, because
  `CTL_CODE` is a function-like macro bindgen cannot emit. Two shipped-class defects the paid
  bodies carried: the `FormatExCB` callback matched on the BARE C enumerators (`PROGRESS`,
  `DONE`, …) where the island spells them `CALLBACKCOMMAND_*` — §9.112, and the ordinals are
  not what the C order suggests (`DONE` is 11, `DONEWITHSTRUCTURE` is 1) — and the
  `SHCreateThread` thread proc lost its `CALLBACK`/stdcall. Regenerating the island also
  normalized 236 fn-pointer slots to `extern "system"`, which let four `#[cfg]`-duplicated
  stdcall/C callback twins in `iconcache`/`shlexec`/`shlfileop` collapse to one body each.
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
