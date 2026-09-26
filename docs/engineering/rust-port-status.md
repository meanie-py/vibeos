# ReactOS → Rust Port — Current As-Built Status

**A multi-architecture port of ReactOS (Windows-compatible OS) translated from C to
Rust, booting to a graphical desktop on i386, amd64, and ARM64.**

*This is THE single source of truth for Rust-vs-C coverage — the seven per-surface
`*-translation-audit.md` files were superseded and moved to [`archive/`](archive/).
Historical snapshots + project timeline live in the
[old-details rollup](archive/rust-port-status-rollup.md).*

> **The core ledger below is GENERATED — run `python3 tools/port_audit.py` rather than
> trusting this page's date.** The hand audit that used to produce it was last run on
> 2026-07-03 and there was no command to refresh it, so it read as current for twelve
> weeks. The narrative ledger further down is still hand-written and still dated; treat
> it as commentary on the generated numbers, not as a second measurement of them.

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

> The paragraphs below are the 2026-07-03 estimates, kept for their per-category framing.
> Where a figure here and the generated ledger disagree, **the ledger is the measurement**
> — e.g. this section's "~140 ROS-native DLLs (~240K LOC)" is 424,078 live lines across
> `dll/win32` as actually compiled, ~124K of which carries a Wine copyright header the
> registry does not yet cover.

The remaining C is four well-bounded categories, **none on the boot-critical path**: (1) the
bulk userland **CRT** (msvcrt/ucrt leaf library — the assemble-from-`cnt-rust`+`rtl-rust`
track), (2) **vendored/by-policy** C (enumerated in `vendor.yaml` — see below), (3) **legacy dead** C superseded by
Rust (videoprt, old NDIS/tcpip), and (4) **irreducible glue** (variadic trampolines, arch
`.S`, generated RPC/syscall stubs). *(The desktop shell `explorer.exe` is NO LONGER a
remnant — it's ~99% Rust as of the crate-split era; see the ledger below.)* The broad
still-C userland frontier is the **~140 ROS-native DLLs (~240K LOC)** + the **Wine-derived
bucket (~154 DLLs, ~1.05M LOC)** catalogued in the roadmap's Phase 0 Tier 3, plus — added
by the 2026-08-08 classification sweep, and not covered by either of those counts — the
**Microsoft DDK storage class drivers** (`classpnp`/`cdrom`/`disk`, ~90.5K lines of
bodies), now explicit in `vendor.yaml`'s `ours:` half rather than uncounted.

Tree inventory (2026-09-25, `python3 tools/port_audit.py --rust`): 7.05M lines of `.rs`
under `sdk/lib/rust/`, of which **4.01M are generated** (checked-in `bindgen_*.rs`;
`win32k-sys` alone is 4.4M bytes per arch), **47K are host test harnesses**
(`*-difftest`), and **2.99M are hand-written or paid translation output**. Quoting the 7M
as "lines of Rust written" overstates the port by more than half. And **the 2.99M is Rust,
not C**: the ledger below counts the C it replaced (1.45M lines with a Rust twin), and the
ratio of about 2:1 is the normal cost of explicit casts, `unsafe` blocks and FFI shapes.
Both numbers describe the same progress.

## Core OS ledger — GENERATED, 2026-09-25 (`tools/port_audit.py`)

Measured at commit `1a825d6d19a` against this worktree's three build graphs. **The oracle is the
link, not the source tree**: a surface's "live C" is the translation units some arch
actually compiles, after every `if(ARCH)` and every commented-out CMake entry, taken as the
union across i386/amd64/arm64.

**What "core" means (owner's ruling, 2026-09-25):** the base Windows XP/Vista OS — kernel,
HAL, drivers, in-box DLLs, services, shell, control panel; what a user expects out of the
box. **Provenance does not decide it.** Everything not in `vendor.yaml` `trees:` is ours,
Wine-derived code included; vendored trees are third-party libraries we keep in C and bump
as versioned dependencies. Wine is still reported (the `of which Wine` column and the
provenance table) because it predicts cost — a Wine module has an upstream to diff and a
winetest suite to grade against. In-box applications get their own table; tests, host
tools and the unbuilt rosapps are out. The 2026-09-24 cut excluded Wine as "a separate
tier" and so reported **60%**; on this definition the same tree is **45%**.

**Read `Covered` as an upper bound, and `Live C (lines)` as the thing that must reach
zero.** `Out of build` counts every `.c` the link does not contain, which includes upstream
C this port never compiled; `Rust twin` narrows that to files whose module has a crate in
`sdk/lib/rust/`, and `Covered` is built from that column, never from `Out of build`.

| Surface | Live C (TUs) | Live C (lines) | of which Wine | Out of build | Rust twin | Covered |
|---|---:|---:|---:|---:|---:|---:|
| Win32 DLLs | 1,023 | 1,012,731 | 933,768 | 520,185 | 433,645 | 30% |
| DirectX | 201 | 267,951 | 267,683 | 40,884 | 0 | 0% |
| Support libraries | 692 | 114,929 | 17,820 | 93,559 | 82,615 | 42% |
| Driver fleet | 176 | 55,895 | 0 | 425,597 | 388,912 | 87% |
| Setup | 45 | 42,721 | 0 | 132 | 0 | 0% |
| Keyboard layouts | 86 | 37,899 | 0 | 0 | 0 | 0% |
| Control Panel applets | 72 | 28,183 | 1,473 | 24,863 | 24,753 | 47% |
| Shell | 69 | 27,189 | 0 | 14,826 | 14,513 | 35% |
| Text services (CTF/IME) | 31 | 27,175 | 8,218 | 0 | 0 | 0% |
| Print spooler | 66 | 27,069 | 694 | 0 | 0 | 0% |
| win32k: USER | 25 | 20,079 | 3,749 | 87,009 | 79,550 | 80% |
| Shell extensions | 57 | 18,940 | 0 | 9 | 0 | 0% |
| System spine | 61 | 18,176 | 1,516 | 28,526 | 28,505 | 61% |
| Services | 41 | 16,169 | 1,152 | 21,495 | 21,495 | 57% |
| win32k: display drivers | 26 | 11,322 | 0 | 1,798 | 1,798 | 14% |
| win32k: GDI | 13 | 9,522 | 9,299 | 29,465 | 24,038 | 72% |
| AppCompat | 24 | 7,339 | 0 | 0 | 0 | 0% |
| OpenGL (opengl32) | 6 | 3,496 | 0 | 0 | 0 | 0% |
| SDK inline TUs | 13 | 2,704 | 0 | 724 | 724 | 21% |
| NLS codepages | 4 | 2,048 | 0 | 23 | 0 | 0% |
| Boot (freeldr) | 5 | 1,407 | 0 | 38,552 | 31,356 | 96% |
| win32k: ReactX | 3 | 362 | 0 | 5,077 | 1,884 | 84% |
| ntdll.dll | 2 | 38 | 0 | 12,115 | 12,115 | 100% |
| win32k: syscall stubs | 1 | 31 | 0 | 0 | 0 | 0% |
| CSR subsystem | 1 | 17 | 0 | 12,258 | 11,284 | 100% |
| Kernel (ntoskrnl.exe) | 0 | 0 | 0 | 273,724 | 273,107 | 100% |
| HAL | 0 | 0 | 0 | 17,766 | 15,773 | 100% |
| **CORE TOTAL** | **2,743** | **1,753,392** | **1,245,372** | **1,648,587** | **1,446,067** | **45%** |

| Core by provenance | Live C (lines) | Rust twin | Covered |
|---|---:|---:|---:|
| ReactOS-native | 508,020 | 1,140,823 | 69% |
| Wine-derived | 1,245,372 | 305,244 | 20% |

| Surface | Live C (TUs) | Live C (lines) | of which Wine | Out of build | Rust twin | Covered |
|---|---:|---:|---:|---:|---:|---:|
| In-box applications | 337 | 153,232 | 24,806 | 15,952 | 7,860 | 5% |
| **APPS TOTAL** | **337** | **153,232** | **24,806** | **15,952** | **7,860** | **5%** |

| Not ours to translate | Live C (TUs) | Live C (lines) | Why |
|---|---:|---:|---|
| Vendored (vendor.yaml) | 1,451 | 948,216 | third-party, kept in C as a versioned dep |
| Tests | 141 | 41,008 | apitests + winetests |
| Host tools | 1 | 642 | build-time only, never ships |
| rosapps | 0 | 0 | optional extras, not configured into the build |

*Compile edges per arch: i386 4,515, amd64 4,597, arm64 4,566; the tables are their union.*

### What the table says

**The boot path is done.** ntoskrnl and HAL compile zero lines of C, ntdll 38, CSR 17;
win32k USER/GDI/ReactX are 72–84% with ~30K live lines left. Nothing in the
boot-to-desktop chain is waiting on a translation.

**Core is 1.75M live lines, and 71% of what is left is Wine-derived** — 508K
ReactOS-native at 69% covered, 1.25M Wine-derived at 20%. Provenance comes from two
sources: vendor.yaml's `ours:` Wine entry (24 paths) and upstream's own import ledger,
`media/doc/WINESYNC.txt` (~316 paths and files). Reading only the first, as the 2026-09-25
morning cut did, labelled gdiplus, shlwapi, riched20, dbghelp, comdlg32, usp10, rsaenh
and ~150 others "native" and reported the split as 50/50. `Forked` entries count as Wine
(shell32 forked in 2007, which is why the Wine twin column includes it).

**The ReactOS-native remainder, by module** (live lines): browseui 22K, cmd 22K, setup
(lib 21K + reactos 20K + usetup 14K), portcls 18K, imm32 16K, spooler base 15K, ndis 13K,
wow64 11K, the sound/rdbss/rxce driver libs ~30K, devmgr 10K (C++), diskpart 9K, cmlib
9K, videoprt 8K, the CTF cluster (msutb/cicero/msctfime ~18K), apphelp 6K,
the static CRT (~12K across stdio/string/math) and keyboard layouts (38K of scancode
tables). Kernel, HAL, ntdll and CSR are at or near zero.

**Where the Wine half sits** (live lines): comctl32 87K, wined3d 80K, mshtml 71K, msi 53K,
msvcrt 45K, crypt32 42K, msxml3 35K, jscript 34K, d3dx9_36 34K, setupapi 32K,
windowscodecs 31K, rpcrt4 27K, ddraw 25K, wininet 24K, quartz 24K, urlmon 23K. Two
clusters are worth naming because they are big and could be argued either way under "out
of the box": the **DirectX stack** (268K, all Wine, all live, 0% — wined3d plus the
d3d8/9/ddraw/dsound/dinput/quartz front ends) and the **IE engine** (mshtml + jscript +
vbscript + ieframe + urlmon + wininet, ~177K). XP shipped both, so both are core here.

**Compile edges overcount a first-match cohab — msvcrt is the example.** The ledger
shows 45K live Wine lines under `dll/win32/msvcrt`, and `msvcrt.dll` itself is done:
`python3 tools/msvcrt_export_coverage.py` reports 1,192 of 1,280 exports Rust-backed, the
88 left being asm and deliberate (`memmove`), and the C++ runtime (`cpp.c`/`except.c`/
`undname.c`) is Rust in `crt-shared-rust`. The C is still COMPILED because
`msvcrt_static` is linked behind the Rust archive as a fallback, and its copies lose
first-match. The same Wine C also ships as the only implementation in **`msvcrt20`,
`msvcrt40` and `crtdll`** (1,139 / 1,175 / 520 exports), which link `msvcrt_static` and
not the Rust archive. So of those 45K lines, the part that really runs is those three
legacy DLLs. They can be relinked against the existing Rust for no translation cost.
Anywhere else that cohabs with `--allow-multiple-definition` inflates `Live C` the same
way; the per-module export tools are the finer measure there.

**The never-started native tail** is unchanged: Setup (56K), keyboard layouts (38K),
CTF/IME (27K), print spooler (27K), shell extensions (19K), AppCompat (7K) — 0% covered,
no crate.

### Registry notes from this audit

- **Mesa was unregistered third-party — REGISTERED 2026-09-24 (owner approved).**
  `dll/opengl/mesa` (Mesa 2.6, 53,679 lines, LGPL-2.0-or-later) was in neither half of
  `vendor.yaml`; `port_audit.py --check` caught it by asking what the LINK contains, which
  `--suspects` (corporate copyright holders only) structurally cannot. That ratchet sees
  only UNCLASSIFIED top-level areas: third-party code nested inside a classified directory
  passes silently.
- **Eleven more trees registered 2026-09-25** by diffing upstream's own ledger,
  `media/doc/3rd Party Files.txt`, against `vendor.yaml` — a cross-check nobody had run.
  Vendored: dosfstools (`vfatlib/check`), the C-Media CMIDriver, Red Hat's netkvm,
  rdesktop (`mstsc` + rosapps `tsclient`), setup's `treelist.c`, the libdxg
  `d3dkmthk.h`, the GL SDK headers, and three host tools (chmc, lzx_compress,
  schilytools). Ours (already Rust): buslogic, cryptlib. ~31K live lines left the
  native core. `hdaudbus` was deliberately NOT registered: the files it compiles are
  ReactOS-authored, and CoolStar's BSD sources sit beside them unbuilt.
- The 2026-09-24 note that ~124K Wine-headered lines sat outside the `ours:` Wine entry
  (usp10, imm32, winmm, msctf, …) no longer moves the headline — they are core either way.
  It only affects the `of which Wine` column, which is labelling, not scope.

## Headline component ledger — narrative, hand-written (2026-07-03)

| Surface | Status | Coverage |
|---|---|---|
| **ntoskrnl.exe** (kernel) | 344 subsystem `.c` removed from build; 8-file Rust-glue floor | **~100%** |
| **win32k.sys** (GUI) | 166 `.rs`/256K LOC; 159 `.c`/`.cpp` flipped; glue = shims/floatobj/gendib | **~100%** |
| **ntdll.dll** | body Rust; cargo-driven link; per-arch `dispatch` + gen stubs | **~97%** |
| **Boot spine** smss/csrss/csrsrv/winlogon/services/lsass/userinit | bodies 100% Rust, cargo-driven, zero C in link | **~100%** |
| **user32 / gdi32 / secur32 / msafd** | 0 active body C; cohab-flipped | **~100%** |
| **kernel32.dll** | body Rust; only NLS `locale.c` + resources left | **~95%** |
| **advapi32.dll** | only `wine/security.c` + generated RPC stubs left | **~90%** |
| **msgina.dll** | 7 `.c` flipped; one C++ leaf (`dimmedwindow.cpp`). Its strsafe printf wrappers were macros that DROPPED the caller's bound and forwarded to `wsprintfW` (1024-wchar internal bound) — a stack overrun on a `WCHAR[256]`, plus a `[WCHAR; 256]` passed to the variadic by value. Both fixed 2026-08-05 by the shared `strsafe-rust` crate + msgina's first `rust_varargs_shim.c`, itself retired 2026-09-25 for strsafe-rust's `StringC*PrintfW!` macros | **~88%** |
| **ole32.dll** (COM runtime) | `ole32-rust`: **the entire DLL is Rust — all 29 TUs.** The COM core (compobj + marshal/stubmanager/rpc/ftmarshal/git); the mechanical ring (task allocator + streams + lockbytes, the Running Object Table + all 6 monikers, `bindctx`/`errorinfo`/`enumx`/`dictionary`, `ole2`/`ole2impl`/`ole32_main`, `clipboard`, `comcat`, `oleproxy`, the NDR `usrmarshal`); and the storage/embedding cluster (`storage32` — the compound-file implementation, `stg_stream`, `stg_prop`, `datacache`, `defaulthandler`, `oleobj`). The C `SOURCE` list in the CMakeLists is EMPTY. Boots ×3. `guid.c` stays C permanently (an `initguid.h` TU whose job is emitting the moniker CLSIDs as data) | **done** |
| **oleaut32.dll** (OLE automation) | `oleaut32-rust`: **the entire DLL is Rust — all 14 TUs.** The automation core: BSTR (`oleaut`), VARIANT (`variant`), the VarXxxFromYyy coercion matrix (`vartype`), SAFEARRAY (`safearray`), the typelib reader/writer + ITypeInfo/ITypeLib (`typelib` + submodules), IDispatch (`dispatch`), user-marshal (`usrmarshal`), `hash`. Plus the OLE-control cluster (2026-07-13): `olefont` (IFont/IFontDisp), `olepicture` (IPicture + OleLoadPicture*), `varformat` (the VarFormat engine), `connpt` (connection points), `recinfo` (IRecordInfo), `olepropframe`. The C `SOURCE` list in the CMakeLists is EMPTY. Boots ×3. `export_audit` clean: all 398 spec exports have a `#[no_mangle]` Rust definition with the spec's convention | **done** |
| **ws2_32.dll** (winsock client) | `ws2_32-rust`: **the entire DLL is Rust — all 34 compiled TUs.** The socket API (`socklife`/`sockctrl`/`send`/`recv`/`select`/`ioctl`/`event`/`qos`/`dupsock`), the transport + namespace catalogs (`dcatalog`/`dcatitem`/`nscatalo`/`nscatent`/`nspinstl`/`spinstal`), the object model (`dprocess`/`dprovide`/`dsocket`/`dthread`/`nsprovid`/`nsquery`), the WSP upcall table (`sputil`), async getXbyY (`async`/`getxbyxx`/`getproto`), name resolution (`rnr`/`qshelpr`/`addrinfo`/`addrconv`/`enumprot`/`rasdial`), and startup/dllmain/wsautil/bhook. The C `SOURCE` list in the CMakeLists is EMPTY. Boots ×3, and `test_usermode_winsock_roundtrip` (the M5 capstone) passes ON the Rust: TCP echo + UDP DNS + a **real HTTP request to the public internet**, all via ws2_32→msafd→AFD→netstack→smoltcp. `verify_lost_exports` + `verify_c_signatures` clean | **done** |
| **winsock/net satellite DLLs** | Cohab-flipped to Rust alongside ws2_32 (2026-07-17/18): **wshtcpip** (`wshtcpip-rust`, both TUs — the TCP/IP WSH provider), **ws2help** (`ws2help-rust`, dual-role static lib), **mswsock** (`mswsock-rust`), **wshirda** (`wshirda-rust` — the IrDA WSH provider, all-stub), **userenv** (`userenv-rust`). **netapi32** (`netapi32-rust`, **all 21 TUs** — FULL flip completed 2026-07-18; see the MIDL-island note below), **dhcpcsvc** (`dhcpcsvc-rust`, **all 12 TUs** — FULL flip, the DhcpC* API + the BSD/ISC dhclient state machine). **iphlpapi** (`iphlpapi-rust`, **all 11 TUs** — FULL flip 2026-07-18; only the generated `iphlpapi_stubs.c` remains C, and the `iphlpapi_rust_shims.c` out-of-line no-op TRACE/FIXME hazard was deleted with it, field manual §9.111). **Partial flips:** **dnsapi** (`dnsapi-rust`, all 8 TUs — FULL flip, 2026-07-18; the single `dnsrslvr_c.h` MIDL interface + the `adns` staticlib stay C). netapi32's 7 RPC-stub TUs (browser/dssetup/misc/netlogon/schedule/srvsvc/wksta_new) landed 2026-07-18. The long-standing "6 conflicting MIDL interfaces" blocker was MISDIAGNOSED: the conflict was the `<lm.h>` master header vs the WIDL stubs, not the stubs against each other (only browser↔srvsvc genuinely collide, over `_SERVER_INFO_100`) — `netapi32.h` itself pulls only the narrow lmaccess/lmapibuf/lmerr trio and lets each `*_c.h` supply its own wire types. `gen_driver_types.sh` grew `TYPES_STEM` so one crate can carry several bindgen islands; netapi32-rust now has `types` (lm.h, the 14 c2rust modules), `types_rpc` (5 interfaces, 6 modules) and `types_browser`, with a `prelude_common` + per-island prelude split (377 type names overlap, so the islands must never share a glob scope). Field manual §9.108; the integration bugs a green cargo check cannot see are §9.109. | **full** |
| **network SERVICES** (`base/services/*`) | Cohab-flipped 2026-07-19, ~6.7K LOC, six FULL flips: **dnsrslvr** (`dnsrslvr-rust`), **wkssvc** (`wkssvc-rust`), **browser** (`browser-rust`), **wlansvc** (`wlansvc-rust`), **tcpsvcs** (`tcpsvcs-rust`), **telnetd** (`telnetd-rust`). dnsrslvr/wkssvc close the dnsapi/netapi32 MIDL loops from the row above. Only wkssvc/browser/wlansvc actually RUN at boot (hivesys `Start=2`); tcpsvcs/telnetd are `Start=3`, and **dnsrslvr has no hivesys service entry at all** — it ships to system32 but nothing starts it, so dnsapi's `R_Resolver*` client calls reach no server. That is a pre-existing upstream gap, NOT introduced by the flip. Field manual §9.112 (catch-all bindings recur in usermode services; the upstream cause is an incomplete bindgen island — allowlist the header owning any constant a `switch` dispatches on). | **full** |
| **network client DLLs** (Tier 3) | Cohab-flipped 2026-07-25, 56 functions for $0.50 through the deterministic-first loop: **wsock32** (`wsock32-rust` — the only compiled C was a `DllMain` returning TRUE, lowered deterministically for $0; the rest of the DLL is `.spec` forwarders `spec2def` turns into thunks), **wlanapi** (`wlanapi-rust`, 28/28 — the CLIENT half of `wlansvc.idl` whose server half is `wlansvc-rust`, closing that loop), **snmpapi** (`snmpapi-rust`, 27/27 — `AsnAny` is a tagged union whose `ASN_*` tags are macro EXPRESSIONS, so island completeness is a correctness property here, §9.112). `wlansvc_c.c` and the generated `*_stubs.c` stay C. Three integration bugs a green `cargo check` cannot see: `midl_user_allocate` is a **preprocessor rename** (`rpcndr.h:94` → `MIDL_user_allocate`, invisible to a source-reading translator, surfaces only as an unresolved symbol at link); a `DWORD`→`SIZE_T` cast that compiled on i386 and failed on amd64/arm64; and bindgen's `__BindgenUnionField` emulation for `AsnAny` (cleared by `bindgen_unions_to_real.py`). | **full** |
| **network client DLLs** (Tier 3 batch B) | Cohab-flipped 2026-07-25, 81 functions for ~$0.55: **mprapi** (`mprapi-rust`, 3/3 — RAS/router admin, near-all stubs), **netid** (`netid-rust`, 19/19 — the "Computer Name" system-properties page), **samlib** (`samlib-rust`, 58/58 + dllmain — the CLIENT half of `sam.idl`, whose server half is samsrv, closing that loop as wlanapi-rust does for wlansvc.idl). `sam_c.c` (generated MIDL client stubs) stays C; its 68 prototypes are **bindgen-derived** from `sam_c.h`, not hand transcribed. `netid_varargs_shim.c` is NOT a translation left in C — stable Rust cannot DEFINE a C-variadic, so it owns `FormatMessageBox`'s entry point and hands straight back to Rust with a pointer to the `va_list` (see `varargs-rust`). Bugs worth remembering: the dead `RpcExcept` handler shipped as LIVE code in 4 samlib functions, unconditionally clobbering the RPC status — the seal envelope checks token presence and multiplicity, never control-flow POSITION (§9.117); `SamLookupIdsInDomain` lost a level of indirection on `PSID_NAME_USE *Use` and would still have linked; and the `L` prefix of a wide literal was translated as CONTENT three separate ways. | **full** |
| **RAS client family** (Tier 3 batch C) | Cohab-flipped 2026-07-26, 81 functions for **$0.13**: **rasapi32** (`rasapi32-rust`, 58/58), **rasman** (`rasman-rust`, 7/7), **rasdlg** (`rasdlg-rust`, 9/9), **rasadhlp** (`rasadhlp-rust`, 7/7 across autodial/init/winsock). The cheapest batch yet because **66 of the 81 were lowered deterministically for $0** — Wine `FIXME(); return ERROR_NOT_SUPPORTED;` shims, 53 of them in rasapi32 alone. That is the stub-heavy end of the distribution, not a repo-wide rate (9% repo-wide, ~0% on real driver code); rasadhlp, the only one with real logic, paid for all 7. rasadhlp is the winsock NSP autodial hook — ws2_32 calls `WSAttemptAutodialAddr/Name` on a failed resolution and it IOCTLs `\Device\RasAcd` — so it rides the ws2_32 surface already translated. Its island is the first to use `REAL_UNIONS=1` (AUTODIAL_ADDR/AUTODIAL_CONN are tagged unions over an ANONYMOUS union). Bugs worth remembering: `RTL_CONSTANT_STRING(L"\\Device\\RasAcd")` came back with the `L` dropped — a UNICODE_STRING over ANSI bytes compiles, and only fails when it cannot open the device; and `nsp_dns.h` DEFINES its six service GUIDs in the header, so flipping winsock.c — the only TU that included it — left them undefined at link. | **full** |
| **wldap32** (Tier 3) | **TRANSLATED + GUARD-GATED, NOT YET FLIPPED — `cargo check` clean on all 3 arches (was 1087 errors).** 162 functions + wldap32.h's 62 `static inline` marshalling helpers, $2.05. In THIS tree wldap32 is a **stub DLL**: wine's config.h `#undef`s HAVE_LDAP (ReactOS does not vendor OpenLDAP), so every entry point compiles to `return WLDAP32_LDAP_NOT_SUPPORTED;`. The translation bought that disabled arm and emitted it UNCONDITIONALLY. **Fixed 2026-07-26** by carrying the C's guards across instead of deleting the arm: new `rustconv/cfg_gate.py` derives the boundary from the C (164 fns gated, 115 `#[cfg(not(...))]` twins) across the EIGHT `HAVE_LDAP*` probes config.h `#undef`s independently. Surfaced two silent bugs — a live §9.112 catch-all in `map_error` (all 16 `case` labels are ldap.h constants, so arm 1 bound everything and it returned SERVER_DOWN for every input) and 7 functions nested inside `ber_printf`'s body by an assembly-order slip. ber_printf/ber_scanf use the varargs-rust trampoline (`rust_varargs_shim.c`) since stable Rust cannot define a C-variadic. **FLIPPED 2026-08-02**, and the flip is what found the module was **missing 24 functions** — every one of them pointer-returning, i.e. the `TYPE * CDECL fn()` scanner hole (`0b2d3521e0a`) that landed after wldap32 was translated. Bought for $0.21 (the resume cache replayed the other 47 bodies for $0) and spliced into the gated files rather than re-assembled, so the hand fixes survived. `export_audit` reported only 15 of the 24 and called the rest "forwarded" — a bare `.spec` target is a LOCAL RENAME, not a DLL forward, and `-version=0x600+` rows are not in the `.def` at all; both fixed, and the repaired audit then caught 4 fresh `extern "system"`-for-cdecl exports in the new code before they could corrupt the i386 stack. `profile_derive` warns `MOSTLY_DEAD_BODIES` before payment so the stub-DLL trap cannot recur silently. | **FLIPPED (all 20 TUs)** |
| **ntmarta** (Tier 3, batch D) | **FULL FLIP — the one compiled `.c` → ntmarta-rust, `cargo check` clean on all three arches, and NO C left in the module at all** (no variadic entry point, no inline asm; netcfgx's strsafe shim, the comparison once drawn here, retired 2026-09-25). 1,797 LOC / 28 functions for **$0.40**; 287 cargo-check errors to zero without needing `crate-repair`. The MARTA provider — the ACL/SID/trustee back end advapi32's Get/SetNamedSecurityInfo calls through. **The harness fix it forced is the durable part**: `AccpGetTrusteeSid` repeated all five of its call tokens and was rejected, correctly per the envelope and wrongly in fact — the C falls `TRUSTEE_IS_OBJECTS_AND_NAME` through into `TRUSTEE_IS_NAME`, and Rust `match` has no fall-through, so the only faithful lowering writes the second arm's body into the first. `_replicable_after_goto`'s positional concession now covers a falling-through `case` label too, and the body was then ADOPTED from its stored reject for $0. The second rejection was real and worth the whole check: `AccRewriteGetNamedRights` shadowed the outer `Ret` with a `let` inside the `if`, so it would have returned the OPEN result rather than the query's, silently, on the success path. **17 enum-member aliases were written BEFORE paying, fifteen of them `case` labels** — bindgen prefixes an enumerator with its enum tag, so the bare name the C switches on is absent from the island (§9.112); `profile_derive` flagged one of the seventeen, because it checks whether the owning HEADER is allowlisted and accctrl.h already was. winsvc.h had to be allowlisted by hand for the same reason (SC_HANDLE and SC_MANAGER_CONNECT are ordinary uses, not case labels). One real signature bug, the netcfgx class again: `PEXPLICIT_ACCESS_W*` came back as `*mut *mut PEXPLICIT_ACCESS_W`. | **flipped** |
| **authz** (Tier 3, batch D) | **FULL FLIP — all 3 compiled `.c` → authz-rust, `cargo check` clean on all three arches.** 836 LOC / 29 functions across authz.c + clictx.c + resman.c for **$0.23**, no audit rejections, 99 errors to zero. The authorization API: resource managers, client contexts, the audit-event surface. Like ntmarta it needs **no C shim** — and the interesting part is why the three C-VARIADIC exports do not force one. `AuthzInitializeObjectAccessAuditEvent{,2}` and `AuthzReportSecurityEvent` are `UNIMPLEMENTED; return FALSE;` stubs that never read their varargs; `authz.spec` declares all three **cdecl with only the FIXED parameters**; and under cdecl the CALLER cleans the stack. So a fixed-arity callee is ABI-identical to the variadic one for every caller on all three targets, and stable Rust's inability to DEFINE a C-variadic costs nothing here. Each carries a note saying that implementing it later means adding the varargs trampoline. | **flipped** |
| **msv1_0** (Tier 3, batch D) | **FULL FLIP — all 7 compiled `.c` → msv1_0-rust, `cargo check` clean on all three arches, and NO C left in the module** (no variadic entry point, no inline asm; just the generated `.def` + stubs). 2,983 LOC / 61 functions for **$0.73**; 323 cargo-check errors to zero. The MSV1_0 authentication package — the LSA-side and user-side halves of NTLM/interactive logon that lsasrv dispatches through `SECPKG_FUNCTION_TABLE`. **`ntlm/global.c` never went through `translate`**: it defines six globals and no functions, so there was nothing to buy, and its absence was invisible to `cargo check` (the island declares all six `extern`) right up to the link. Hand-carried into `src/ntlm/global.rs`, with the 27- and 14-slot dispatch tables emitted **mechanically** from the C's designated initializers — the risk in 41 near-identical `LsaSp*`/`UsrSp*` names is transposition, not translation, and a swapped pair compiles, links, boots and then misauthenticates. Three substrate decisions did most of the work: the profile moved to `BLOCKLIST_FUNCS_FROM_CRATE` (a thin module over samsrv/lsasrv delay imports would otherwise need ~60 hand-written RPC-shaped prototypes whose pointer levels nothing checks) and allowlists winbase.h for the four kernel32 calls; `NT_SUCCESS`/`RtlEqualMemory` are `bool` in Rust so the C's `== 0` idiom was rewritten rather than the predicates re-typed. **Four real bugs the compiler could not have found:** `*(UINT64*)&UserInfo->All.AccountExpires` came back as `.QuadPart as *const UINT64` — a *timestamp value dereferenced as an address*, loud here only because OLD_LARGE_INTEGER has no `QuadPart`; `LogonListResource`'s initializer invented six RTL_RESOURCE field names (recalled, not read) where the C is a bare tentative definition; `sizeof(MSV1_0_PROTOCOL_MESSAGE_TYPE)` and `sizeof(LSA_STRING)` lost their `sizeof` entirely, leaving the bare type name as the operand; and three wide literals were hand-encoded UTF-16 byte strings. Two tool fixes fell out: `normalize_island_abi.py` now projects the i386 witness THROUGH aliases (a C function *type* is a named alias on the 64-bit twins but inlined on i386, so 40 SECPKG slots stayed `extern "C"` on amd64/arm64 only), and the field manual gained the `let`-binding-captured-by-a-glob-const entry (`TokenGroups` is both the C's local and a `TOKEN_INFORMATION_CLASS` member). | **flipped** |
| **aclui** (Tier 3, batch D) | **FLIPPED — all 4 translatable `.c` → aclui-rust, `cargo check` clean on all three arches.** The ACL editor: the "Security" property page plus the owner-drawn check-list control it is built around. `guid.c` stays C on purpose (it is `#include <initguid.h>` and says "NO CODE HERE" — macro-generated data, nothing to buy), and `rust_varargs_shim.c` carries `LoadAndFormatString`'s `va_start` because stable Rust cannot DEFINE a C-variadic. **628 cargo-check errors to zero.** What actually moved the number: `BLOCKLIST_FUNCS_FROM_CRATE` (~290 errors, replacing ~200 hand-written user32/comctl32 prototypes); **collapsing the 5 per-arch `#[cfg]` twin pairs** — 1,369 duplicated lines, and while they existed `crate-repair` withheld **119 of 119** repairs because two targets can never agree about a site that exists on only one of them (105 applied on the next run); and `UNBLOCK`ing DLGPROC/WNDPROC/LPTHREAD_START_ROUTINE into this island so one `extern "system"` definition of each callback serves all three arches. **Four bugs the compiler could not have found:** `PaintControl` (~180 lines) was never in the plan at all — checklist.c closes an `#if SUPPORT_UXTHEME` between an `if` and its body, which does not parse in one pass; one of the three `GetKeyState` sites read `VK_BACK` where the C reads `VK_SHIFT`, for a variable named `Shift`; `AddSelectedPrincipal`'s first parameter lost its `IDsObjectPicker` type to `c_void`; and the whole TCHAR type family was resolving to the ANSI one out of ntdll-sys's glob, in a module compiled `-DUNICODE`. | **flipped** |
| **netcfgx** (Tier 3) | **FULL FLIP — all 11 compiled `.c` → netcfgx-rust, `cargo check` clean on all three arches, links.** 9,100 LOC for **$0.36 total**. Stuck at 6 of 11 since 2026-07-26 on "one or two functions per TU the call-seal keeps rejecting across two samplings"; finished 2026-08-03 for $0.08 once the three real causes were found, and all three were in the harness. (1) `_audit_paid_body` unsealed the plain-C reply TWICE, and the second pass — which sees no tokens — rejected bodies the first had accepted, four of them here. (2) The first pass then ACCEPTED a SHIFTED envelope: `ApplyOrCancelChanges` spent CALL_0000 on `IsEqualGUID`, a function-like macro that by design gets no token, and every later token slid one call to the left, so unsealing restored `swprintf`'s name onto `StringFromCLSID`'s site. A shift is a permutation and passes every count-based check, so the envelope now also compares the ARGUMENT COUNT at each token's C and Rust site (0 disagreements over all 164 banked bodies; catches this on the first pass). The envelope additionally NAMES the untokenized macros now instead of only disclaiming them in general — three interleaved windowsx.h macros were enough to make `ReadParameterValue` invent three token numbers. (3) `netcfgx.c` was refused outright: its three `STDAPI` exports are unextractable, because `STDAPI` is `EXTERN_C HRESULT STDAPICALLTYPE` and the scanner normalized it to a bare `__stdcall`, deleting the return type — same class as the `TYPE * CDECL fn()` drop. `KW_EXPANSION` now carries the eleven aliases that are really `[linkage] <TYPE> <conv>`. The 2,758-error `cargo check` came down in the documented order — 236 missing symbols closed by a generated substrate (the 110 Win32 externs are bindgen output from the same header set and `-D` flags the `.c` compiles with, checked identical across all three arches; the 40 COBJMACROS dispatch shims are generated from the island's own `IFACEVtbl` structs, so a parameter list cannot drift from the vtable it dispatches through), then `MACRO_SIGNED=1` on the profile (bindgen's u32 default cost 715 E0308s on `rc != ERROR_SUCCESS` and `GetDlgItem(hDlg, IDC_*)` alone), then `crate-repair` ×2 (1,186 → 375), then the residue by hand. `UNBLOCK="DLGPROC"` is what lets one definition of each of the twelve dialog procs serve all three targets — Rust compares fn-pointer ABIs nominally, and ntdll-sys spells DLGPROC `stdcall` on i386 and `C` on the 64-bit targets, so without it every proc needs a `#[cfg]` twin (field manual, the fn-pointer-typedef entry). **Six real bugs the compiler could not have found**, all caught by checking the output against the C: 157 wide literals spelled as Rust BYTE strings (plain ASCII cast to a wide pointer, hand-encoded UTF-16 with a half-width terminator, and the C `L` prefix carried into the content) — of which `L"\Device\"` had silently lost its trailing separator; four `RegSetValueExW` calls whose wide-literal DATA argument was dropped entirely, leaving the bare type name `LPBYTE`; `SUCCEEDED(hr)`/`FAILED(hr)` expanded and then compared against zero a second time; `ZeroMemory` given memset's three arguments; `CheckInfFile` returning `u32` where the C returns `BOOL`; and `MergePropPages`'s `pszStartPage` typed `*const LPCWSTR` against netcfgn.h's `LPCWSTR *`. Two bodies needed judgement rather than sampling and both were the model IMPROVING on the C: `CreateInstanceKey` hoisted the `goto done` tail per path and lost `if (ret == FALSE)`, fixed by feeding the audit finding back into the retry; `ScanForInfFile` refused three samplings to reproduce a `continue` that skips its own `FindNextFileW` (an infinite loop, unreachable because `\inf\*.inf` cannot match `.` or `..`), repaired by hand in its stored reject and adopted for $0 through the new re-audit path. | **flipped** |
| **umpnpmgr** (Tier 3, batch E) | **FULL FLIP — all 4 compiled `src/*.c` → umpnpmgr-rust, `cargo check` clean on all three arches.** The user-mode PnP manager service: the SERVER half of pnp.idl that cfgmgr32 and setupapi speak as clients, owning device enumeration/installation, the kernel PnP event pump (`NtGetPlugPlayEvent`) and the device-install worker. 7,038 LOC, 1,150 cargo-check errors to zero for **$0.47**. `pnp_s.c` (MIDL server stubs) and `umpnpmgr_stubs.c` (spec2def) stay C — build products, nothing to translate. **`MACRO_SIGNED=0` was MEASURED, and the resemblance argument was wrong**: netcfgx and aclui both want it ON because their constants are `int` control ids and window messages, and umpnpmgr looks the same — but it spends its time passing DBT_/WM_/CM_/SERVICE_ constants into UNSIGNED Win32 parameters, so signing them cost 90 extra E0308s (165 signed against 76 unsigned). Generate both islands and count; it is one cargo check. | **flipped** |
| **syssetup** (Tier 3, batch F) | **FULL FLIP — all 8 compiled `.c` → syssetup-rust, `cargo check` clean on all three arches, links.** The system setup DLL: the second-stage GUI wizard (welcome/product/owner/computer/locale/date-time/theme/process/finish pages), the account and security-policy writer that drives LSA and SAM, the network-component installer, the setup action log, and the PS/2 mouse property page. ~20K lines for **$2.94**; 1,392 → 0 cargo-check errors. **Parked RED at 176/177/177 for a session** because the residue had no width answer — of the 17 integer sites erroring on all three targets exactly ONE had three-way agreement, since the parameters genuinely differ per arch (SIZE_T, WPARAM, UINT_PTR). Cross-target agreement is a filter, not a repair. Unstuck by `crate-repair`'s new **`width_alias_cast`** rule, which reads the callee's DECLARED parameter type out of the crate's own island and casts to the NAME (34 sites; 55 more claimed so the width-baked rule stood down). `syssetup_stubs.c` stays C (spec2def), the only C left: `rust_varargs_shim.c` (the three variadic entry points) retired 2026-09-25 — `pSetupDebugPrint` (`@ varargs` in the .spec, a real export) and `FatalError` RECEIVE their varargs the va-slots way, and `StringCchPrintfW` is a strsafe-rust macro. **strsafe.h's other bounded string ops now come from the shared `strsafe-rust` crate** (they were ported here first, in `src/syssetup/strsafe.rs`, which is now a re-export): they are `static __inline`, so no object file in the tree ever defined them and the prelude's `extern` block named a strsafe static lib the module does not link — invisible until the last C TU was flipped. Profile: `REAL_UNIONS=1` (PROPSHEETPAGE/PROPSHEETHEADER are half anonymous union and the C flattens them), `UNBLOCK` of the DLGPROC/PROPSHEET family, and `MACRO_SIGNED=0` measured (483 signed against 432 unsigned) — the second module running where the shape argument lost to the measurement. **Six bugs the compiler could not have found**: `CreateTimeZoneList` dropped its callback entirely (`EnumerateTimeZoneList(RetrieveTimeZone, SetupData)` became `(SetupData, NULL)`, so the time-zone list would have been empty); `PFNREQUESTWIZARDPAGES` lost a pointer level; ADMIN_INFO's three LPWSTR fields were initialised as `[0; 256]` arrays; proppage stored the PMOUSE_INFO through a 32-bit truncation on 64-bit; five of wizard.c's six registry-path statics had the wrong declared `[WCHAR; N]`; and 54 wide literals arrived spelled out elementwise as `['S' as u16, …]`, which dangles in a function body. | **flipped** |
| **desk.cpl** (Tier 3, batch G) | **FULL FLIP — all 19 compiled `.c` → desk-rust, `cargo check` clean on all three arches, links.** The Display control panel: the Background/Screen Saver/Appearance/Settings/Desktop-items property sheets, the classic-and-visual theme reader and applier, the monitor-selection and preview custom controls, the GDI+ wallpaper loader, the display class installer, and the `CDevSettings` IDataObject carrying the desk.cpl ↔ display-driver shell-extension protocol. ~14K lines; 2,261 cargo-check errors to zero. `guid.c` stays C (an `initguid.h` TU that emits CLSIDs as data — zero functions); `rust_varargs_shim.c` (strsafe's three variadic entry points) retired 2026-09-25 for strsafe-rust's `StringC*PrintfW!`/`StringCbPrintfExW!` macros; the rest of strsafe.h now comes from the shared `strsafe-rust` crate (task #48; desk's copy was the superset the crate was built from). **desk was reported FLIPPED-READY at 20/20 TUs when it was 17** — `settings.c` and `theme.c` had never been translated, and nothing could see it: `lib.rs` declared modules for files that did not exist, a crate out of `workspace.members` is never compiled, and no source scanner opens a path that is absent. `scan_orphan_modules` now reports that as `[PHANTOM]`, the dual of its orphan finding, with no banner escape — an orphan can be parked on purpose, a `mod` pointing at nothing never can. **`MACRO_SIGNED=0` MEASURED** (1,517 unsigned against 1,820 signed): the third property-sheet module where the resemblance argument would have lost. **Two island bugs that compiled clean:** three constant families desk uses by bare name (DVASPECT_*, TYMED_*, DATADIR_*) were absent because `--allowlist-file` matches where an item is DEFINED and objbase.h only `#include`s the widl-GENERATED objidl.h/wtypes.h — so the island carried `IDataObject` but none of the constants its own call sites pass; and `TCHAR` resolved to ntdll-sys's `c_char`, making desk.h's `TCHAR szPath[MAX_PATH]` structs HALF-SIZED, hidden behind `.cast()`s the translation had sprinkled to make the narrow pointers fit. Both fixed in the profile (allowlist + `UNBLOCK` of the TCHAR family), never in the prelude. **Bugs the compiler could not have found:** `REGSTR_PATH_CONTROLSFOLDER TEXT("\\Desk")` is C string-literal CONCATENATION and came across as a COMMA, turning a 4-argument call into a 5-argument one; the `DesktopIcons` table shipped four invented resource ids (1001/1004/1000/1347 where the header says 3901–3904), which would have wired every desktop-icon checkbox to a control that does not exist; `devsett.c`'s `vtblIDataObject` was dropped entirely, so every COM slot came across but not the table wiring them together; 104 wide literals arrived in four broken spellings (narrow bytes behind a wide pointer, the C source text `L"…"` quotes and all, hand-interleaved UTF-16 at alignment 1, and decimal character codes typed `i32`); the two bitfield structs lost their accessors; `SwitchDisplayMode` narrowed BOOL to `u8` and `SettingsPageCallbackProc`'s third parameter LPPROPSHEETPAGE to `*mut u32`; and **two `#if` directives were read as ARCHITECTURE tests** — classinst.c's `#if _WIN32_WINNT >= 0x502` became `#[cfg(target_pointer_width)]` on two argument expressions, and theme.c's `#if (WINVER >= 0x0600)` became `#[cfg(target_arch)]`. A version test and an architecture test look alike and are never interchangeable. | **flipped** |
| **hal** (x86 + arm64 stub) | 42+ cohab markers; `.S` NT-ABI + setjmp glue *(not re-verified this pass)*. **i386 default flavor is now APIC** (LAPIC/IOAPIC/TPR, lazy-IRQL, RTC clock — i386-apic-hal lane, 2026-07-16); the legacy 8259 PIC HAL is preserved in-tree, gated off, behind `vibeos-cli build --hal-pic` (`VIBEOS_PIC_HAL`). amd64 was already APIC | **~97%** |
| **rtl** static lib | 80/82 `.c` flipped; `dbgprint_variadic.c` + asm floor | **~100%** |
| Cohab helper libs (cmlib, rossym, evtlib, inflib, …) | real `.c` flipped for kernel/cross consumers | **~100%** |
| **freeldr + boot DLLs** | modules + cmlib/cport/cmboot; 16-bit `.S` floor *(not re-verified this pass)* | **~99%** |
| **Driver fleet** (storage, FS, input, bus, netstack) | see below — netstack is a from-scratch Rust rewrite; rest cohab-flipped | **~90–100%** |
| **LSA/SAM server** (lsasrv / samsrv) | active frontier — samsrv ~90%, lsasrv ~50%+ | **partial** |
| **explorer.exe** (desktop shell) | `explorer-rust` 22 `.rs`/22K LOC; 22 cohab markers; 2 glue `.cpp` left (settings_shim, util_vararg) | **~99%** |
| **CRT** (msvcrt/ucrt body) | **The legacy CRT DLLs are Rust now too (2026-09-25, i386, the only arch they build on): msvcrt20 669/681 (98%), msvcrt40 659/673 (97%), crtdll 469/505 (92%)**, up from 0% -- they link msvcrt_static's same `_MSVCR_VER=0` C and now take the same cohab archive and varargs shims as msvcrt.dll (`msvcrt_rust_cohab.cmake`), so msvcrt-difftest's grading is their evidence too; `tools/payloads/oldcrt` loads each in the guest and checks a spread of exports (all PASS). `--dll` measures any of the four; the tool's i386 figures were ~0% until the same day (it never stripped i386's leading underscore). **1,192 of 1,280 implemented C exports Rust-backed — 93%** (2026-09-13, amd64. **Re-measure with `python3 tools/msvcrt_export_coverage.py`** — this figure was hand-derived three times and rotted every time: the previous entry said 44% while the true value was already 91%, because eight waves landed and nobody re-ran the recipe written out beside it. The tool refuses a stale or missing archive rather than reporting the previous wave's number.) The denominator excludes 24 `-stub` entries (msvcrt has no body at all — and `-stub` is a FLAG on `@ varargs -stub`, not a directive, which is how the first draft counted them as implemented), 5 DLL forwards, and the 124 `?`-mangled C++ exports, which are a separate lane. **None of the 88 still-C exports is msvcrt's own C**: 40 are crtmath (16 x87 `_CI*`, 21 `__libm_sse2_*`, 3 `_ftol*` — all assembly; the 18 float/bessel/misc that used to sit here are Rust as of 2026-09-13, see the crtmath section below), 18 are i386 FDIV errata fixups in asm, 17 are SEH/setjmp asm plus `except_i386`, 6 are dbgrpt, 6 are `i386/inout.s` port I/O, and one is `memmove`, which is deliberately not Rust (§9.65: LLVM folds a byte-copy loop back into a call to itself). crtmath and dbgrpt are a shared-library lane serving five DLLs, not an msvcrt one. Wave 14m flipped the **whole core libm** (+45 exports: `sin`/`cos`/`exp`/`pow`/`sqrt`/`fmod`/`abs`/`div`/… — the DLL's math exports bind straight to `crtmath`'s plain names, and cnt-rust's kernel-proven `cabi/math.rs` supplies them via one `cabi-math` feature; no new code, cnt-difftest already grades that engine against the same sources). Waves 1–13 cover — the pure string/mem/wide/radix/`_s` floor, the whole locale-table **ctype/case** family (`ctype.c`), the **collation** family, and the **bignum float parser** (`strtod`/`atof`/`wcstod`/`_atoflt`/`_atodbl`/`_atoldbl`/`__STRINGTOLD` — msvcrt's own `bnum.h` PORTED, wave 5's DO-NOT-FLIP reversed the honest way). Differentially proved against the genuine `string.c`/`wcs.c`/`ctype.c` by **six** instruments (`classify` values incl. **exact 64-bit float bit patterns** · `oob_guard` access · `perf` speed · `hot_path` per-element env lookups · `ref_integrity` the oracle · `link`), in **both** the "C" and a real NLS locale. The float family flips on all three arches — i386 via x87 `ST(0)` extern-C returns (I first gated it out on a wrong premise; field manual §9.79). Phase D's locale door is OPEN (`msvcrt-sys` = bindgen + `offset_of!` + a two-compiler layout guardian). Every `.c` in `msvcrt_static` still compiles and is inert — Rust wins by lld first-match — and every one now has a Rust counterpart except `sincos.c`, whose two symbols are `#pragma makedep implib` and never reach the DLL export table at all. `concurrency.c` (3,839 lines) and `handler4.c` (775) compile to **ZERO defined symbols** at this `_MSVCR_VER=0` config — entirely inside version-gated blocks, confirmed by `nm` and independently by `profile_derive --target msvcrt_shared`, which flags handler4 `DEAD_TU` at 97% dead with "DO NOT PAY TO TRANSLATE IT". Neither is porting debt; both would have been money burned. **ucrtbase now links Rust** (2026-09-05): it is vendored Microsoft UCRT (`sdk/lib/ucrt`, see vendor.yaml) plus OUR `msvcrt_shared` plus 132 lines of its own C, so the shared TUs -- cpp.c, cpp_rtti, except.c, except_<arch>.c, seh_i386, undname.c -- moved into their own crate `crt-shared-rust`, and `crt-shared-cargo` is what ucrtbase links under `--whole-archive` ahead of `msvcrt_shared`. The split is the point, not the packaging: ucrtbase.spec and msvcrt's cohab archive collide on 891 names of which only 35 are msvcrt_shared's, so pointing msvcrt's rail at ucrtbase would have silently replaced 856 genuine Microsoft UCRT bodies with msvcrt-contract ones -- and with `--allow-multiple-definition` that raises no diagnostic at all. Measured, not argued: the ucrtbase archive defines 128 symbols where msvcrt's defines 1,320, and `tools/verify_ucrtbase_cohab.py` (in `vibeos-cli check`, empty strict baseline) fails the build if the archive ever defines something ucrtbase EXPORTS that `msvcrt_shared`'s C does not -- negative-tested at 899 findings by putting msvcrt's archive in the slot. The `ucrtsupport` cargo feature carries the `-D__UCRTSUPPORT__` live set, which swaps in BOTH directions: 7 bodies out (set_terminate, _get_terminate, terminate, unexpected, __pxcptinfoptrs, signal, raise -- ucrt owns those) and 15 in that msvcrt's `_MSVCR_VER=0` never compiled. The arch-SEH family is no longer owed on i386: `msvcrt-seh-difftest` (2026-08-07) is a 32-bit host harness that grades `except_i386.c` at 9,982 rows, and it caught a shipping panic — a C flexible array indexed as a Rust array. `except_x86_64.c` and `except_arm64.c` stay ungraded on purpose: between them the ten symbols are one-line calls through a function pointer, a `Rip` read, a two-line FIXME stub, and one `RtlUnwindEx` wrapper, so rows over them would grade nothing anyone would miss. | **~13–19%** |

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

**Wave 21 (2026-08-03) — process.c FLIPPED. All 50 functions.**
`msvcrt-crt-rust/src/msvcrt/process.rs`. The whole `_exec*`/`_spawn*`/`_popen` family, plus
`_cwait`, `_loaddll`/`_unloaddll`/`_getdllprocaddr` and `_getpid`. 41 bodies replayed from the
resume cache for $0, 7 lowered deterministically, and **2 hand-written**: `msvcrt_spawn` and
`_wpopen` failed the deterministic CALL envelope on nine consecutive runs, which is a
restructured-lowering rejection rather than a model error and does not improve with retries.

That pair was blocking the other 48, so `rustconv translate` grew **`--skip-fn`**: it drops
named bodies from the worklist, assembles the rest, and records each omission as an audit
finding so the gap cannot go quiet. (`--exclude` filters FILE stems and could never do this.)
The whole TU then reassembled for $0.10.

**The sixteen `WINAPIV` entry points** (`_execl`, `_execle`, … `_wspawnlpe`) are `va_start`
trampolines in `dll/win32/msvcrt/rust_process_varargs_shim.c` over `_v`-prefixed Rust twins,
the same shape as file.c's twenty and for the same pinned-stable reason: receiving a `va_list`
is fine, starting one is not. The four `_execle`-family twins need to walk the list TWICE (once
to build the command line, once to skip past it and read `envp`), and they do it from one
snapshot — sound only because `va_list` is a flat pointer on every Windows target here and
`msvcrt_valisttos` takes it by value. That reasoning is written out at the twins rather than
assumed.

`system` and `_wsystem` are `#[cfg(any())]` — `#ifndef __REACTOS__`, confirmed against all
three objects. `msvcrt_get_comspec` is NOT gated despite being absent from `nm`: it is inlined
into its sole caller, which is the trap the object-table rule has to be read carefully to
avoid. The module sits behind the `stdio` feature for the same reason file.c does — `_popen`/
`_wpopen`/`_pclose` stand on file.c's fd table, and an ungated `pub mod` broke the scanf
difftest, which turns `stdio` off to own the character-source names itself.

**Two defects found by audit**, both the wide-literal class rather than a permuted constant:
`msvcrt_search_executable`'s `L"PATH"` was one byte short of a UTF-16 NUL, so `_wgetenv` read
the final code unit's high half out of bounds and PATH search silently matched nothing for
every `use_path=1` spawn; and `_wsystem`'s `L" /c "` was plain ASCII cast to `*const wchar_t`.
Both are now caught by `scan_wide_literals.py` — see the field manual.

**GRADED 2026-08-07 — reference TU #12, 9,101 rows, and process.rs came back CLEAN.**
`tests/process.rs` over `processimpl.c`, a RECORDING `CreateProcessW`. That shape is the whole
design: process.c never creates a process, it MARSHALS — argv array or `...` list into one
command line, envp array into a double-NUL block, a bare name into a full path via `PATH` and
four extensions, a `_P_*` mode into creation flags plus a wait-or-return decision — and then
calls Win32. A harness that compared only return values would grade two `-1`s and report green
over the entire module, so every row compares the RECORDED CALL: application name, command
line, the environment block including its embedded NULs, creation flags, `bInheritHandles`, and
the `STARTUPINFOW` fields msvcrt fills (including the fd-inherit block file.c builds).

Three things had to be solved before a row could exist.

* **The two `va_list` models never meet.** Sixteen exports are WINAPIV and the subject cannot
  supply them — they are C trampolines over `_v`-prefixed Rust twins that walk a flat `*mut u8`,
  and this host's `va_list` is a SysV register-save-area struct. The scanf TU solved its version
  by compiling the reference against refstub_printf's flat `<stdarg.h>`; that FAILS here,
  because these bodies genuinely start their own lists and the flat header's
  `va_start(ap, last)` is `ap = NULL`. So the reference is called with real host varargs, the
  subject's twin is handed the same logical arguments hand-packed into 8-byte slots, and what
  is compared is the recorded spawn.
* **`_exec*` cannot be graded on success, and that is a finding rather than a gap.** They all
  pass `_P_OVERLAY`, whose arm is `_exit(0)` — the harness would terminate mid-row and exit
  ZERO. Exec rows preset the spawn to FAIL, which still grades everything through the errno
  mapping; and `_exit` is redirected to a terminator that exits 70 with a message, so the
  silent-green can never happen by accident.
* **The three std handles are compared as ROLES, not numbers.** `_popen`'s only real decision
  is which end of the pipe the child gets, and the two sides' fd tables cache different OS
  handles for the same CRT descriptor — `(4, 14, 8)` vs `(5, 14, 9)` for identical behaviour.

Nothing in process.rs diverged. Two findings landed in the harness instead, both of the class
where both sides fail identically and the row still says MATCH: fileimpl.c's path converter
passed `\` through to `stat`, so **every** `use_path` search found nothing on both sides; and
the four `msvcrt.h` codepage inlines that `msvcrt_argvtos_aw`/`msvcrt_valisttos_aw` are built
out of were a hand copy in `refstub/msvcrt_file_prelude.h` claiming to be "VERBATIM" with
nothing checking it — build.rs now diffs it against the genuine header. Both are in the field
manual under §9.73.

**The substrate closed behind it.** process.rs was written by a lane that owned neither
msvcrt-sys nor prelude.rs, so it carried six names as local `extern` blocks and `const`s under
`// TODO(hoist)`. All six are derived now and the locals are gone —
`GetEnvironmentVariableW`, `CREATE_UNICODE_ENVIRONMENT` and the bare
`STARTUPINFOW`/`PROCESS_INFORMATION` aliases from the bindgen island, `realloc`/`_exit`/
`intptr_t` from prelude.rs (the last three had already been hoisted by another lane; the TODOs
outlived their premise). `CREATE_UNICODE_ENVIRONMENT` is the one that mattered: it was absent
from the island because the allowlist said `CREATE_[A-Z]+`, which matches `CREATE_NEW` and
silently drops every multi-word member of the family — the LOCALE_SMONTHNAME1 bug again, in a
constant `msvcrt_spawn` ORs into the creation flags of **every** spawn. The hand-written value
was right; nothing was checking that. `get_aw_cp`, `convert_acp_utf8_to_wcs` and `HIWORD` stay
local on purpose: each mirrors the C's own `static inline` or macro, so a per-TU copy is the
faithful shape.

`STARTUPINFOW` and `PROCESS_INFORMATION` joined `msvcrt_crt_layout.h`'s guarded set. The second
is not bookkeeping: msvcrt_spawn's `_P_WAIT` arm calls
`GetExitCodeProcess(pi.hProcess, &pi.dwProcessId)` — it writes the exit code into the PID field
and returns THAT — so the field order is program logic, and a pin asserts the returned value is
the exit code and not the pid.

**GRADED 2026-08-07 — reference TU #10, 2,247 rows, and every finding was in the ORACLE.**
`tests/math.rs` drives the 37 exports whose reference twin exists on the host (the rest —
`_isnan`, `_finite`, `abs`, `div`, `ldexp`, … — are `#[cfg(any())]` here because cnt-rust's
`cabi` owns them in the DLL, and cnt-difftest grades them; a row for one does not fail, it
fails to LINK). Most bodies here are `if (isnan(x)) return math_error(…); return <libm>(x);`
and both sides reach the same host libm, so what is actually graded is the wrapper: THREE
views of one call — the return value compared BITWISE (a float `==` is blind to both NaN and
`-0.0`, which is most of what this file computes), errno through the correct per-side
accessor, and the whole `struct _exception` captured by installing a recording handler through
msvcrt's own `__setusermatherr`. The third view is the one that sees a permutation, which is
the class this file is known to carry — and a $0 cross-check ran first: all 11 `math_error`
call sites tabulated on both sides, type/name/arg1/arg2/retval, 0 divergent.

Four things it turned up, none of them a port bug:
- **`sqrt(-1.0)`'s NaN sign is a COMPILER FOLD, and the reference's compiler is not the
  shipping one.** `ret_nan(TRUE)` is `(x - x) / (x - x)`; host gcc folds it to `-NaN`,
  llvm-mingw clang (measured, at msvcrt's own flags) and rustc to `+NaN`. The subject matches
  the binary that ships. The row pins the subject to clang's constant absolutely and lets the
  reference differ in that one bit.
- **The reference's `_snprintf` and the subject's were two different host stand-ins**, drifted
  in opposite directions — one truncating a digit early, one writing its NUL one byte PAST the
  caller's buffer. Now one shared body: the host's formatter under msvcrt's buffer contract.
- **`_gcvt_s`'s reference path segfaulted** the first time anything drove it, on the flat-vs-
  SysV `va_list` incoherence; fixed with a per-TU `-include`, because a shared redirect would
  rename wcs.c's own definition.
- **`ldexp`/`_fpclass`/`div`/`ldiv` sit in `#ifndef __REACTOS__`** and the textual scrape put
  them on the rename wall anyway, so `_scalb`'s one-line body called a `ref_ldexp` that nothing
  defines. The guarded set is now derived and subtracted, and pinned.

Two domains are skipped by predicate and counted (190 cases), both the C's own: inputs that
overflow its 80-byte scratch and then read the buffer back past the terminator, and `_s`
destinations smaller than the conversion, where msvcrt's copy loop has no `else` and spins
forever — faithfully, in the Rust too. All four findings are in the field manual under "The
oracle can be MISCOMPILED by the host's own headers".

**Wave 20 (2026-08-03) — math.c FLIPPED, and the object symbol table is why it is correct.**
`msvcrt-crt-rust/src/msvcrt/math.rs` is now `pub mod math;`. The banked translation from
2026-08-02 (91 exports, $2.29) needed four kinds of work before it could go in, and only the
first was the one the header predicted.

**The inline asm.** Of the 64 asm sites, 48 turned out to be inside `#ifndef __REACTOS__` —
the `__libm_sse2_*` wrappers, not compiled here at all. The 16 that remain are the real FP
control state, and the aarch64/arm half (`mrs`/`msr` on FPSR/FPCR, `vmrs`/`vmsr` on FPSCR)
was already correct, because those genuinely take register operands. Only the x86 half was
wrong: `stmxcsr`, `ldmxcsr`, `fstsw`, `fstcw`, `fnstenv`, `fldenv`, `fldcw` have no register
form. Rust's `asm!` has no `m` constraint, so the shape is a register holding the ADDRESS,
dereferenced in the template, written in `options(att_syntax)` so the instruction text is the
C's character for character. `fldenv`'s eight `st(n)` clobbers cannot be transcribed — Rust's
x86 asm has no `st` register class — which is safe here only because the function is
integer-only and returns immediately after; that reasoning is written at the site.

**What the object said and the source did not.** Cross-referencing `llvm-nm --defined-only`
on `math.c.obj` against the Rust caught three flip-breaking bugs the C reads past:
`MSVCRT__rotl64`/`MSVCRT__rotr64` had been translated under a bare `_rotl64`/`_rotr64`
(Rust defines a symbol nothing imports, the C body stays live, and the flip is a silent
no-op); `math_error` must be *exported* as `_invoke_matherr`, because CMakeLists carries
`-Dmath_error=_invoke_matherr`; and the same `-D` list retargets `asm_sqrt`/`asm_sqrtf`/
`x87_asin` to libm, which rewrites the call sites but not the `__ASM_GLOBAL_FUNC` labels, so
those three stubs are defined-but-dead and the Rust follows the call sites instead. The i386
`_CI*` family, `_ftol` and the eight `__stdcall` FDIV stubs ARE referenced (they are
`.spec` exports), so they are defined here — the `_CI*` and `_ftol` as `global_asm!`
transcribed verbatim from the C, the FDIV stubs with `#[export_name = "__adj_fdiv_m16i@4"]`
because `#[no_mangle]` cannot spell an i386 `__stdcall` decoration.

**Twenty-two `__libm_sse2_*`, `_fpclass`, `ldiv`, `fenv_encode` and `fenv_decode`** are
`#[cfg(any())]` for `#ifndef __REACTOS__` / `_MSVCR_VER` reasons, alongside the nine already
gated because cnt-rust's difftest-pinned `cabi/math.rs` exports them into the same archive.
The `#ifndef __i386__` and `#ifdef __i386__` families are now arch-gated rather than defined
everywhere.

**The audit found four real defects in sixteen flagged bodies**, plus one in a body that was
never flagged. `_fcvt_s` was the worst: all three `size--` sites had been translated as
`let mut size = size; size -= 1;` *inside* the enclosing `if size > 1` block, so every
decrement mutated a shadow that died at the closing brace and the caller's buffer bound never
moved — an out-of-bounds write in the one API whose entire job is to prevent one. `_ecvt_s`
never decremented `len` after the sign strip, shortening both `memmove`s. `_setfp`'s aarch64
arm copied `~(old_fpcr >> 8) & fpsr & 0x9f != fpsr & 0x9f` across verbatim, which is a
different expression in Rust: C binds `!=` tighter than `&`. And `MSVCRT_exp` — *not* on the
flagged list — had `_DOMAIN` demoted to `0`, silently turning a NaN domain error into no
errno at all. Every other flagged body diffed clean, which is the calibration: the list is a
review queue, not a defect list.

**Wave 19 (2026-08-02) — time.c FLIPPED. 52 of 52 exports at `_MSVCR_VER=0`.**
`msvcrt-crt-rust/src/msvcrt/time.rs`, the whole file: the calendar (`_mktime32/64`,
`_mkgmtime32/64`, `_localtime32/64(_s)`, `_gmtime32/64(_s)`), the clock (`_time32/64`,
`_ftime32/64(_s)`, `clock`, `_difftime32/64`), the zone (`_tzset`, `_get_tzname`, the four
`__p__*` accessors and the four exported globals), the string forms (`_strdate`, `_strtime`,
`asctime`, `_wasctime`, `_ctime32/64`, `_wctime32/64` and their `_s` twins) and `strftime` /
`wcsftime` / `_Strftime` / `_wcsftime_l`. Whole-file, like errno.c: `_tzset`'s four exported
globals plus the file-static `tzi` are ONE contract that `is_dst`, `mktime_helper` and
`_ftime64` all read, so a partial flip leaves some readers on the C's copy of that state.

**Seven names that read like exports of this file are not compiled at `_MSVCR_VER=0`** and
so have no Rust counterpart: `_get_dstbias`/`_get_timezone`/`_get_daylight` (`#if >= 80`),
`_Wcsftime` (`>= 110`), `_timespec32_get`/`_timespec64_get`/`_strftime_l` (`>= 140`). The
OBJECT is the authority on that, not the source's apparent shape — the same rule wave 18
used for file.c.

**GRADED 2026-08-07 — reference TU #11, 375,839 rows, and one REAL PORT BUG.** All 52
exports, over five zone settings x 18 boundary instants, plus the `TZ`-string branch that no
preset can reach and a three-hourly sweep of 2024 in every zone (which finds a transition
disagreement without depending on knowing where the transitions are).

`strftime("%c")` HUNG. msvcrt's picture formatter counts the run of the current field with
`while(format[0] == format[count]) count++;`; the translation emitted `format[count] ==
format[count]` — an element compared to itself, so `count` ran away until the read left the
mapping. It reaches every `%c`, `%x` and `%X`, i.e. every locale-formatted date and time in
the DLL. It survived the flip because those three conversions read
`locinfo->lc_time_curr`, and that slot was NULL in both sides' harness locinfo until this
lane filled it — so the first row that ever executed the line is the one that found it.
Three more were debug-profile checked-arithmetic aborts where the C relies on hardware
wraparound (`_difftime32/64`'s subtraction, `_localtime64_s`'s deliberate
signed-to-unsigned multiply); `vibeos-cli difftest` builds `--release`, so that class is
invisible to the gate and only a plain `cargo test` shows it. Detail in the field manual
under "Faithful C unsigned underflow" and "Semantic rewrites that flip meaning".

Everything else matched, including the two things this file exists to get wrong: the
half-hour bias (India) and the year-wrapping southern-hemisphere DST window (Sydney).

Two pieces of substrate arrived with the rows. `cloc_time_data` — the C locale's 43 calendar
strings — is lifted out of locale.c by build.rs into a TU of its own and shared by both
sides, on the same terms as `MSVCRT__ctype`: it is what the locale IS, neither side's code
writes it, and what the rows grade is time.c's INDEXING into it. And `__lc_time_data` stopped
being a bare forward declaration in `locale_layout.h`; `ref_integrity` now checks all six of
its offsets and its size against msvcrt-sys, because the wide union arm nothing reads at
`_MSVCR_VER=0` is 344 bytes that everything after it depends on.

Two host-type-model traps were pinned in the reference TU's prelude, both in `clock()`:
`clock_t` is 32 bits on Windows and 64 here (so msvcrt's `clock()` wraps after 24.86 days and
an unpinned reference does not), and `CLOCKS_PER_SEC` is 1000 in msvcrt's `time.h` against
glibc's 1000000 (so an unpinned reference answers in microseconds — every row off by exactly
1000x). The constant is derived by `gen_msvcrt_difftest_consts.py`, not transcribed.

**`timeimpl.c` — the Win32 clock and zone layer, built because the SUBJECT needed it.**
Wiring time.rs into the harness revealed that nothing there answered `GetTimeZoneInformation`,
`NtQuerySystemTime`, `SystemTimeToFileTime`, `FileTimeToSystemTime`,
`SystemTimeToTzSpecificLocalTime`, `GetLocalTime`, `GetDateFormatW`, `GetTimeFormatW` or
`lstrcpynA`. Two design points are the reusable part:

* **The instant is a KNOB, not the host clock** (`ref_time_set_now`). Both sides run in one
  process one after the other, so a real clock makes every `_time64`/`clock`/`asctime` row a
  coin flip that *usually* agrees — green in CI, red at midnight. Freezing it also buys what
  a real clock cannot: a row can sit on the epoch, on a leap day, or on `_MAX__TIME64_T`.
* **The zone is a knob too** (`ref_time_set_zone`), with four presets chosen for the branches
  they reach: UTC (no DST), US Pacific (post-2007 rules), India (a HALF-HOUR bias), and Sydney
  (southern hemisphere, so the DST window WRAPS the year end — a rule engine that assumes
  start &lt; end gets this backwards all year). A fifth setting makes `GetTimeZoneInformation`
  FAIL, which is the only way `_tzset`'s early-out arm is reachable. A shim that always
  succeeded with Bias 0 would make `is_dst` dead code and the harness would report green over
  the half of time.c that has the arithmetic in it.

**`_time64` had to leave the plain namespace.** fileimpl.c has answered file.c's one
`_time64` call under that name since wave 18; msvcrt EXPORTS `_time64`, so the moment time.rs
defines it that is a duplicate-symbol failure. The stub keeps its job and changes name to
`ref__time64` (file prelude stage 4.5). This is the standing tax on wiring a new module into
the harness, and it is also the seam where time.c becomes reference TU #8: that stage points
at time.c's own body and the stub is deleted, exactly as errno.c's arrival deleted the
hand-written `msvcrt_set_errno`. **That seam is closed as of 2026-08-07** (it turned out to
be reference TU #11, not #8 — undname.c, misc.c and math.c landed first). The stub is gone
and `ref__time64` is time.c's genuine body, which is a real upgrade for file.c's rows too:
`_wutime`/`_futime` used to stamp modification times from the HARNESS's idea of "now" and now
read the frozen instant the test set.

**Two limits stated rather than papered over.** `asctime`'s `_snprintf` is answered by the
harness over the host's `vsnprintf` (subjectimpl.c) — the shipping name is a `va_start`
trampoline in the msvcrt MODULE, and that TU cannot enter this harness because the Rust
`_vsnprintf` walks a Windows va_list while the host's `va_start` builds a SysV one. So
`asctime`/`ctime` rows would grade the ASSEMBLY of the string, not msvcrt's formatting of it;
the formatting is graded by `printf_port.rs` against the genuine `ref_pf_printf_a`. And
`GetDateFormatW`/`GetTimeFormatW` model the picture grammar `_wstrdate`/`_wstrtime` actually
pass (repeated-letter fields plus quoted literals) and HARD FAIL on anything else, rather
than emitting a best guess that would turn a future caller's wrong output into a passing row.

**One real bug was found and fixed in the translation, and it is a NEW class** — see the
field manual's sealing section. `_tzset` came back with all 35 sealed constants in the wrong
slots (`_lock(0)` for `_lock(_TIME_LOCK)`, `*tz == 3600` for `*tz == '-'`,
`tzi.Bias * CP_ACP` for `tzi.Bias * 60`) and `--seal-facts` passed it, because its invariant
is a closed SET and a permutation satisfies that. `unseal_facts` now refuses a body that
preserved no tokens at all. Two more were fixed by hand in the same pass: the
`#if _MSVCR_VER == 0` arm of `MIN/MAX_GMTIME64_TIME` was taken from the `#else` (which widens
`_gmtime64_s`'s accepted range at both ends), and `MSVCRT__tzname` was initialised to two
NULLs instead of the two 64-byte buffers `_tzset` immediately writes through.

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
cannot show you. `_setmbcp`, the other function no generated case can reach, is now graded by
`mbcs_setmbcp`: **mbcs.c is 175 of 175.**

`_setmbcp` needed the harness to grow rather than another case, and what it needed is the
lesson. It INSTALLS an mbcinfo instead of building one, so a generated case would move the very
fixture the other 173 are pinned against — hence its own binary, hence its own process. And its
non-thread arm dereferences `MSVCRT_locale`, which the harness defined as a single shared NULL.
Both halves of that mattered: NULL is why it was refused, and SHARED is why fixing NULL alone
would have been worse than useless. That arm ends in
`memcpy(MSVCRT_mbctype, MSVCRT_locale->mbcinfo->mbctype, …)`, so one `MSVCRT_locale` would have
had both sides installing into the same slot and reading back each other's writes — a row
comparing a table against itself, green whatever either side did. So `nlsimpl.c` keeps one
`_locale_tstruct` per side, the reference's reached through the `msvcrt_ref_*` import namespace
like `get_mbcinfo`.

Three arms and a refusal, swept over all 7 codepages: the thread arm (which must NOT publish
its table globally), the `LOCALE_FREE` arm (whose only observable effect is the release call, so
`free_mbcinfo` records its argument and each side is read immediately after its own call), and
the global arm — where the exported `_mbctype` is compared both against the other side's and
against the table that side just installed. Both controls were checked by sabotage: pointing the
memcpy at a second array reproduces the shipped bug and the row fails on cp932; dropping the
`LOCALE_FREE` release fails the arm that exists for it.

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

**Wave 23 (2026-08-04) — console.c, misc.c, exit.c and locale.c FLIPPED (93 exports, $0.80
for locale.c; the other three were banked).** All four are whole-file.

| file | exports | note |
|---|---|---|
| `console.c` | 25 of 29 | the other 4 are WINAPIV `...`; `rust_console_varargs_shim.c` |
| `misc.c` | 17 of 17 | `qsort`/`qsort_s`/`bsearch`/`bsearch_s`/`_lsearch`/`_lfind`/`rand` |
| `exit.c` | 18 of 18 | the atexit/onexit registry, `abort`, `_wassert`, `_exit` |
| `locale.c` | 29 + 8 globals | `setlocale`/`_create_locale`/`localeconv`/`_Getdays`… |

**Six defects the paid output shipped that no compiler or boot would have reported.** Worth
reading as a set, because four are the same shape — a value that is REAL but belongs
somewhere else, which is what recall produces and what a closed-SET seal cannot see:

* `_EXIT_LOCK1` came back **8**, which is `_CONIO_LOCK` two lines earlier in the same
  mtdll.h table. `_lock(8)` compiles, links and runs; `_cexit`/`_onexit`/`atexit` would have
  serialized on the CONSOLE lock — racing each other while deadlocking against console.c.
* `cloc_time_data.lcid` came back `0x0409`, a perfectly real LCID —
  `MAKELANGID(LANG_ENGLISH, SUBLANG_ENGLISH_US)` — where the C says
  `MAKELCID(LANG_ENGLISH, SORT_DEFAULT)` = `0x0009`. The same initializer also named a
  `wrefcount` field that does not exist at `_MSVCR_VER=0`, shifting `unk` and `refcount` one
  slot each; `refcount = -1` is the "statically allocated, never free me" sentinel.
* `_aexit_rtn` (exit.c) and locale.c's **eight** globals arrived as PRIVATE Rust statics.
  Every one is a global in its object and `_aexit_rtn` is an msvcrt.spec export, so each
  would have left the C object as the sole provider and dragged the whole TU back in.
* `_country_synonyms` was declared `[*const i8; 72]` against 70 initializers.
* `_exit` was DEFINED returning `()` while prelude.rs and process.rs both DECLARE it `-> !`
  (stdlib.h:229 is `__declspec(noreturn)`); `scan_extern_arity` reported it as `[RETVOID]`.
* `swprintf` would have been a one-slot-off ABI bug. locale.c's call is corecrt_wstdio.h's
  `static inline` over `_vsnwprintf` — `llvm-nm --undefined-only locale.c.obj` imports
  `__vsnwprintf` and no `swprintf` at all — while the DLL's exported `swprintf` is the
  NON-conforming **2-argument** `_swprintf` (msvcrt.spec:1466). A 3-argument call through
  that export puts the format string in the count slot. Call sites use `_snwprintf`.

The generalisable lesson is the first four: **diff every translated constant against the
header, not just the ones the seal flags.** The seal's invariant is a closed SET, so a
permutation satisfies it, and the `UNVERIFIED CONSTANTS` audit fires only when NO sealed
token survived. locale.c's run reported zero findings and still carried two.

Three classes are now mechanised instead of remembered. The msvcrt-sys island derives the
mtdll.h lock numbers, the `LC_`/`LANG_`/`SORT_` families, wincon.h's `INPUT_RECORD`
vocabulary, and the digit-suffixed `LOCALE_S*` LCTYPEs — the pre-existing `LOCALE_[A-Z_]+`
matched **none** of the 42 month/day-name constants, silently. And `wstr!` (prelude) builds
a wide literal at compile time from its readable spelling, so the four hand-interleaving bug
classes `scan_wide_literals.py` grades have no way in for new code.

**Feature gates.** `conio-exit` (console + exit) and `locale`, both DEFAULT-ON and dropped
only by the host harness. exit.c defines `exit`/`abort`, which would replace the test
runner's own with bodies calling `ExitProcess`; the harness DEFINES eight of locale.c's
symbols as the codepage-aware NLS oracle mbcs.c's 160 graded functions stand on (see
`nlsimpl.c`'s header for why a codepage-blind model makes every DBCS branch unreachable).
**misc.c deliberately does not ride a gate** — its sort/search family is among the most
gradeable code in msvcrt, so the harness instead grew the four Win32 stubs it actually
needed (`Beep`, `DebugBreak`, `RtlGenRandom`, `VirtualProtect`) and dropped its now-redundant
`_get_output_format` stub in favour of the real body.

**GRADED 2026-08-07 as reference TU #9** — `tests/misc.rs`, 1,430 rows. Four of its exports
take a CALLBACK and the callback is the harness's, so a row can watch the algorithm rather
than only check its answer — and for `qsort` that IS the contract. misc.c says so above
`qsort_s`: *"trying to sort data doing identical comparisons as native does."* The
comparison LOG is compared per side across sorted, reverse, all-equal, heavy-duplicate and
scattered datasets at sizes 0–64, and it matches exactly. Interior pointers are never
compared across sides (two different buffers) — each becomes `(ret - base) / elem_size`; and
`_lsearch` APPENDS on a miss, so its array and `*array_size` are part of the answer.

Three findings, none in the sorting, and two of them in the HARNESS:

* **glibc's `RAND_MAX` had leaked into the reference by inclusion.** msvcrt's is `0x7FFF`,
  the host's `0x7FFFFFFF`, and refstub/msvcrt.h pulls `<stdlib.h>`. `rand()` ends in
  `(seed >> 16) & RAND_MAX`, so the *reference* returned 16-bit numbers where msvcrt returns
  15-bit ones. Same class as a hand-typed refstub constant and harder to see, because nobody
  typed it.
* **`rand_s`'s entropy stub was named `RtlGenRandom`**; the Rust binds `SystemFunction036`,
  which is what advapi32 exports. It never mattered because no test binary referenced
  `rand_s`, so the undefined symbol never reached a link — a class `cargo check` cannot see.
* The subject's `rand()` **panicked under debug overflow checks**: the LCG's wrap IS the
  algorithm, so `wrapping_mul`/`wrapping_add`, not `*`/`+`. Right in release, unusable in the
  debug build the difftest uses — i.e. ungradeable. Same class as mbcs.rs's underflow.

`Beep` and `Sleep` also stopped being inert. That was right while misc.c was only the
SUBJECT's; once graded, `_beep` is a passthrough and `_sleep` is `Sleep(timeout ? timeout : 1)`
— functions whose entire body is the call, which an inert stub makes unobservable.

**Wave 24 (2026-08-04) — data.c FLIPPED (58 exports, $0.18).** The CRT's process-global
state: `__argc`/`__argv`/`__wargv`/`_environ`/`_wenviron`/`_pgmptr`/`_wpgmptr`/`_acmdln`, the
version words (`_osver`/`_winmajor`/`_winminor`/`_winver`/`_osplatform`), `MSVCRT_app_type`,
the whole `__p__*` accessor family, `__getmainargs`/`__wgetmainargs`, and the `_initterm` /
`_initterm_e` initializer walkers.

`build_expanded_wargv` is HAND-WRITTEN: the paid translation damaged its sealed call envelope
("not exactly once") and was refused, so `--skip-fn` dropped it, the other 39 replayed from
cache for $0.02, and the body was transcribed from data.c:471. Two traps are recorded at the
site — `path_len` walks BACKWARDS as the C's `int` (`wcslen(x) - 1` on an empty string is
`-1` in C and an underflow panic in `usize`), and `len` is only assigned INSIDE the
`do`/`while`, so an argument matching only `.`/`..` must fall through to the unexpanded
fallback.

**The private-static defect recurred for the third consecutive file, and that makes it
systematic.** All 22 of data.c's exported globals arrived as private Rust statics — as
`_aexit_rtn` did in exit.c and eight did in locale.c. It compiles, links, and silently leaves
the C object as the only definition, dragging the whole TU back in. `llvm-nm --defined-only`
on the object is the only thing that says so; the `static` keywords in the source do not.

Also fixed: `_INITTERMFUN` and `_INITTERM_E_FN` are TU-LOCAL typedefs, so the translation
carried them as bare (non-nullable) Rust fn pointers — but `_initterm` tests every table slot
against NULL before calling it, which a bare fn pointer cannot express. `Option<fn>` is the
right shape, and it is exactly what bindgen gives the sibling `_PVFV` for the identical C
typedef.

Behind a `data` feature that IMPLIES `dir-environ` — `msvcrt_init_args` calls environ.c's
`env_init`, so the feature graph states the dependency rather than leaving it to a link error
nobody sees until the DLL is built. Dropped by the host harness, which defines `_get_fmode`
and `msvcrt_wstrdupa` itself.

Lane doc: `docs/engineering/active-lanes/msvcrt-from-pieces-plan.md`. Field manual: §9.60,
§9.61, §9.63, §9.64, **§9.66**.

**Wave 28 (2026-08-05) — lock.c + thread.c FLIPPED ($0.13). The lock table and per-thread state.**

6 spec exports (`_lock`, `_unlock`, `_beginthread`, `_beginthreadex`, `_endthread`,
`_endthreadex`) plus the three internals main.c calls (`msvcrt_init_mt_locks`,
`msvcrt_free_locks`, `msvcrt_get_thread_data`). 434 lines, 14 live functions; small, but the
most-executed code in the crate — `_lock`/`_unlock` run on every `malloc` through heap.rs's
`LOCK_HEAP`, and `msvcrt_get_thread_data` is the per-thread accessor nearly every other
translated module calls. That is the point of taking it now: unlike except.c, a defect here
fails the boot loudly instead of hiding.

**One real defect, and it is the CALL-side twin of the `_tzset` constant permutation.**
`_beginthread_trampoline` came back with its call set slid one slot: `msvcrt_get_thread_data`
— a zero-argument function — wearing the `memcpy`'s three arguments, `memcpy` wearing the
`free`'s one, and `free` in place of the `start_address(arglist)` call that is the entire
point of the trampoline. The thread would have leaked its argument block and never run the
user's entry point. `--seal-calls` accepted it because every callee appeared exactly once and
executable, which is all a closed SET can check.

`rustconv`'s arity oracle exists for exactly this and had the evidence, but skipped the site:
it discarded any call whose argument text mentioned `::<`, and the third argument was
`core::mem::size_of::<T>()` — which is how essentially every `memcpy` lowering spells its
length, so one over-broad guard had disabled arity checking across a whole call family. Fixed
by blanking only the turbofish's own commas (`_mask_turbofish_commas`, length-preserving and
`->`/nesting aware); replaying the real body through the seal now rejects it with
`msvcrt_get_thread_data wants 0, got 3`. Three regression tests.

**Substrate.** The two thread-entry typedefs are DERIVED, not hand-written, and the reason is
visible in the output: `_beginthreadex_start_routine_t` is `extern "stdcall"` on i386 and
`extern "C"` on amd64/arm64, because `__stdcall` is a real convention only on x86 — a
hand-written pair would have been right on one arch and silently wrong on two. Also added
`SetLastError` (the other half of the `GetLastError` save/restore `msvcrt_get_thread_data`
goes out of its way to do) and `ResumeThread`. `msvcrt_tls_index` stays `extern`: main.c is
still C and owns the `TlsAlloc()` that fills it, and a second Rust definition would give the
reader an always-zero index.

Both trampolines' per-arch convention duals are thin wrappers over one shared `_body`, because
only half a `cfg` dual is ever compiled — a later hand-edit to the wrong twin is invisible to
`cargo check` on every target that does not select it.

Behind a `threading` feature, default-on, dropped by the host harness for a DUPLICATE
definition rather than a missing one: `msvcrt-difftest/refimpl.c:310` defines
`msvcrt_get_thread_data` and `fileimpl.c:1220` defines `_lock`/`_unlock`, since the harness
needs its own single-threaded thread_data it can inspect between cases. PORTED, NOT GRADED —
the substrate is Win32 (TLS slots, CRITICAL_SECTIONs, CreateThread), not an algorithm a host
harness can reproduce.

**What is actually left in msvcrt (census, 2026-08-05).** Measured, not estimated: for each
compiled TU, the symbols it defines minus everything that wins first-match — the Rust archive
plus the `rust_*_shim.c` trampolines, which live in the msvcrt MODULE and so beat
`msvcrt_static`. Forget the shims and the answer is wrong by 70 symbols (wcs.c reads as 33
still-C when it is 3).

| file | defines | still reached in C |
|---|---|---|
| `except.c` | 21 | **0** — flipped in wave 29, GRADED 2026-08-08 (reference TU #15, with cpp.c, 848 rows) |
| `except_i386.c` | 13 | **0** — flipped in wave 30, GRADED 2026-08-07 (reference TU #13, 9,982 rows) |
| `except_x86_64.c` | 5 | **0** — flipped in wave 31 |
| `except_arm64.c` | 5 | **0** — flipped in wave 31 |
| `handler4.c` | 0 | **0** — 97% dead, compiles to an object with no symbols |
| `concurrency.c` | 0 | **0** — 98% dead, same |
| `ucrt_tls_sup.c` | 4 | 3 — but see below: NOT msvcrt.dll's surface |
| `wcs.c` | 176 | **0** — the last 3 flipped in wave 32 |
| `string.c` | 106 | 3 (the `fpnum_*` bignum internals) — defined but UNREFERENCED, see below |
| `main.c` | 1 | **0** — flipped in wave 33 |
| `sincos.c` | 2 | 1 (`sincosf`) — implib-only, see below |
| `scanf.c` | 44 | **0** — fully flipped |

Three corrections to that table, all from re-measuring it against the linker on 2026-08-05:

* **`ucrt_tls_sup.c` is not in msvcrt.dll at all.** It is in `msvcrt_shared` only, and
  `MSVCRT_SOURCE` does not list it — it belongs to **ucrtbase**, which links `msvcrt_shared`
  and no Rust archive whatsoever. Flipping it needs a ucrtbase cohab that does not exist yet,
  so it is a different lane, not a straggler of this one.
* **string.c's three `fpnum_*` are already Rust** — `fpnum_double`, `fpnum_ldouble` and
  `fpnum_parse` live in `msvcrt-crt-rust/src/fpnum.rs` as private `fn`s, because the Rust
  `strtod` family is their only caller. The C copies stay linker-visible because nothing
  overrides those *names*, and stay linked because string.c's own dead `strtod` still
  references them. Dead weight, not a divergence.
* **`sincos.c` is `#pragma makedep implib`** — its two symbols are in the import library, not
  the DLL export table, and they exist so a compiler folding `sin`+`cos` into `sincos` resolves.
  `sincos` is already Rust (cnt-rust `cabi/math.rs`); `sincosf` is not, and flipping it would
  buy a Rust wrapper over the still-C `sinf`/`cosf`. Left alone deliberately.

So msvcrt.dll's genuinely-still-C **exported** surface is now **`DllMain` and nothing else**.

So the remaining real surface was **the exception dispatcher and nothing else of size**:
except.c + except_i386.c were 34 of the 45 symbols still served by C. Waves 29 and 30 took both.
What is left is a handful of stragglers — `ucrt_tls_sup.c`'s 3, `wcs.c`'s 3, `string.c`'s 3,
`DllMain`, `sincosf` — and nothing in msvcrt is now bigger than one function.

**And two of the biggest-LOOKING files left were never work at all.** `handler4.c` (775 lines)
and `concurrency.c` (3,839) each sit wholly inside one outermost `#if` decided by the module's
own `-D_MSVCR_VER=0` — `#if defined(__x86_64__) && _MSVCR_VER>=140` and `#if _MSVCR_VER >= 100`.
Each compiles to a **396-byte object with zero defined symbols** on all three arches
(`llvm-nm --defined-only`, not inference). 4,614 lines that every prior census carried in the
"still to translate" column. `profile_derive`'s `DEAD_TU` finding had been saying so
pre-payment; it was read straight past, because in a module whose constants are named
`ERROR_ACCESS_DENIED` a `grep ERROR` of the report shows forty innocent lines and buries it.

**Wave 34 — reactos/misc.c + reactos/system.cpp, $0. No C bodies remain in msvcrt's own
directory.** The two files ReactOS added that Wine never had, 189 lines between them, and both
hand-written: every callee they need had been Rust since earlier waves, so a paid translation
would have been buying the easiest code in the module.

`reactos/misc.c` (7 exports) is two arrays and three accessors, and its only trap is the one
`_mbctype` had just demonstrated — `_mbcasemap` and `_fileinfo` are bare `@ extern` lines in
the spec, so their Rust twins need `#[no_mangle]` or the export keeps naming the C copy while
`__p__mbcasemap` hands out Rust's. `_mbcasemap` is written as a const-eval loop with boundary
assertions rather than 257 transcribed bytes: all it is is an ASCII case swap, and an
assertion that fails the build beats a literal nobody rechecks.

`reactos/system.cpp` had been carried for months as "the C++ substrate lane" and **it is not
one** — worth stating plainly, because that label alone kept it unported. There is no RTTI, no
vtable, no exception object and no C++ ABI on the boundary; both exports are `extern "C"` and
`throw()` is an empty exception specification, not a use of exceptions. The C++-ness is a
`template <typename Character>` instantiated at exactly `char` and `wchar_t` and resolved at
compile time, which is a Rust generic over a trait with four methods. Its one real hazard is a
naming collision, not a language one: process.rs already holds *Wine's* rival `system`/
`_wsystem`, correctly `#[cfg(any())]` because process.c wraps them in `#ifndef __REACTOS__`.
Two live definitions in one archive would be settled by member order.

Both modules had to be named for their DIRECTORY (`reactos_misc`, `reactos_system`) because
both basenames are taken, and that is what surfaced a hole in `verify_cohab_coverage.py`: it
derived the C object as `<mod>.c.obj` and treated a miss as a bare `continue`. A renamed or
mistyped module was therefore compared against nothing and reported clean — the same
fail-open shape as the `extern` parser bug, in the adjacent place. It now takes an explicit
`objects` map, and a missing object is a finding unless the C source exists (arch-conditional,
like `except_i386.c` on arm64) or the module is declared in `no_c_twin` with a reason.

**Wave 33 — main.c's `DllMain`, $0. The WINE-derived half of msvcrt.dll is Rust.** The last C
function in that half, and the one every prior wave routed around: `set_module_type(msvcrt
win32dll ENTRYPOINT DllMain 12)` means this is the *entry point*, and it is what decides the
order in which the heap, the locks, the locale, the fd table, the signal table, argv and the
clock come up. Every one of those was already Rust before this file existed — what was left in
C was the sequencing, and the TLS slot index they all reach through.

`msvcrt_tls_index` had to move in the SAME commit, and prelude.rs's old comment is why: it
warned that duplicating the definition on the Rust side would give the reader *"a second,
always-zero index"*. That was exactly right while the C's `DllMain` was the only WRITER — a
Rust definition would have been a slot nobody filled in. It stops being true the moment the
writer is Rust too, which is the whole reason these two things are one change.

Two checks that cost nothing and are worth repeating on any entry-point flip:

* **`nm` the decorated name before building anything.** `extern "system"` + `#[no_mangle]`
  through `rustc --emit=obj --target i686-pc-windows-gnu` gives `_DllMain@12` — the same
  stdcall decoration the C emits, and the same `12` the CMake `set_module_type` line names.
  `msvcrt_tls_index` comes out `B` (BSS) on both sides too. A mismatch there is a linker error
  at best and a silently-unused entry point at worst, and one `rustc --emit=obj` settles it
  without a build.
* **The feature has to name what it calls.** `dllmain` lists `heap`, `stdio`, `locale`,
  `data`, `cpp`, `conio-exit` and `threading` rather than assuming the default set. A
  `--no-default-features --features dllmain` build would otherwise compile — the C copies
  satisfy the names — and boot msvcrt with half its init sequence pointed at implementations
  that are no longer in the link.

Absent on purpose, all `_MSVCR_VER = 0`: the three `handler4` calls, the three `concurrency`
ones, and `_set_printf_count_output(0)`. The `#if _MSVCR_VER == 0` arm — `LdrAddRefDll`
pinning msvcrt so it can never be unloaded and re-initialised over a live fd table — is the
one that IS live, and is the branch a translation working from the wine original rather than
this tree's flags would drop.

**The census that headline needed — and the bug it found.** "Exported surface is Rust" was
first written as *zero C exports*, and that was measured over `MSVCRT_SOURCE`'s Wine-derived
`.c` only. `dll/win32/msvcrt/reactos/` is in the same source list and also exports. Re-measured
properly on amd64 — `llvm-nm` over every `msvcrt_static` object against the providers that link
ahead of it, joined to `msvcrt.spec` — **8 of 1,119 non-stub, non-C++-mangled exports are still
backed by a live C body**: five in `reactos/misc.c` (`_fileinfo`, `_mbcasemap`, `_get_fileinfo`,
`_set_fileinfo`, `_get_winver`, plus `__p__fileinfo`/`__p__mbcasemap` on i386), two in
`reactos/system.cpp` (`system`, `_wsystem`), and one regression in `mbcs.c`.

Three categories that *look* like remaining work and are not, each checked against the binaries
rather than assumed: **111** symbols across scanf/wcs/file/process/console are the deliberate
`...` trampolines in `rust_*_shim.c`, because stable Rust cannot DEFINE a C-variadic; **7** are
non-exported internals (`asm_sqrt`, `fpnum_parse`, `sse2_supported`) whose Rust twins are
private on purpose; and a further set of exports is resolved from other archives entirely, with
no `msvcrt_static` object defining them at all.

**That measurement is now `tools/export_census.py`, and building it corrected the hand-rolled
answer four times.** Doing it by hand a second time was the detection event: the same question,
the same reconstruction, a different wrong answer each time, and no way for a reader to see the
error without redoing the work. The tool reads the `.spec` for what is exported, `llvm-nm` for
who defines it, and **the linker's own command line out of `build.ninja`** for the order it
would pick — nothing about the link is restated where it could drift. Run on amd64 after
wave 34:

| backing | exports |
|---|---:|
| Rust | 1,060 |
| module objects — the `...` trampolines, in `rust_*_shim.c` | 109 |
| `stub` — spec2def generates a failing body; no implementation in any language | 24 |
| `libcrtmath.a` | 18 |
| `libdbgrpt.a` | 6 |
| `libvcruntime.a` — the `setjmp`/`longjmp` asm | 4 |
| forwarders to kernel32/ntdll | 4 |
| `libntdll.a` — `memmove`, an import thunk | 1 |
| **msvcrt's own C** | **0** |
| | 1,226 |

Four parser traps account for the earlier miscounts, and every one of them *inflates or
deflates a category silently* rather than failing: the backing symbol comes AFTER the argument
list (`@ cdecl time(ptr) MSVCRT_time` — a first-word regex files 22 live Rust exports as
unresolved); `-arch=` restricts a line to some builds (ignoring it charges i386's x87 and
port-IO surface to amd64); `-arch=!i386` is a NEGATION, and reading it as positive inverts a
whole family of C++ ABI thunks; and `@ cdecl _endthread ()` has a space before its parenthesis,
which lets `()` fall into the trailing-symbol slot. `tools/test_export_census.py` pins all four
plus the `extern` two-name case `_mbctype` shipped through.

That `memmove` row is worth reading twice. It is not C and never was ours: it resolves through
the ntdll IMPORT library, so msvcrt.dll's exported `memmove` is a six-byte
`jmp *__imp_memmove` — into ntdll, where the implementation is already Rust. (This corrects an
earlier reading of the same row as compiler-rt's; the member name in the archive is
`ntdll.dll`.)

The zero row is what wave 34 bought, and the rest is the honest shape of what is left: 109
trampolines and 4 setjmp bodies stable Rust cannot express, 24 stubs and 4 forwarders with
nothing to port, one import thunk — and **6 exports, dbgrpt's, that live in
`sdk/lib/crt` and are shared by five DLLs** (crtmath's 18 were the other half of that
group and are DONE — 2026-09-13, below). That group is a different lane, not the tail of
this one: msvcrt, msvcrt20, msvcrt40, crtdll and ucrtbase all link `libcrtmath.a` and only
msvcrt has a cohab. Doing it means a lib-level `crtmath-rust` archive linked ahead of it in all
five consumers — which is also the rail the ucrtbase lane needs. Note `sinf`/`cosf`/`powf` are
in that 18, so the `sincosf` implib question is a crtmath question too.

#### crtmath's 18 exports are FLIPPED — 2026-09-13

`msvcrt_export_coverage` grouped the remainder as "crtmath: float/bessel/misc libm". That
bucket is now EMPTY: **1,174 -> 1,192 of 1,280, 91% -> 93%**, and everything still C in
msvcrt.dll is assembly (58), C++ dbgrpt (6), or `memmove`.

  sinf cosf expf logf log10f powf fmodf _copysign _copysignf frexp ldiv _fpclass
  _j0 _j1 _jn _y0 _y1 _yn

**Not by the route the 2026-09-06 entry below proposed, and the difference is a real
limitation, not a shortcut.** That entry called for a lib-level `crtmath-rust` archive
linked ahead of `libcrtmath.a` in all five consumers. These went into `cabi/math.rs` under
the existing `cabi-math` feature instead — no new archive, no new rail — which is what the
same entry recommends two paragraphs later ("the remaining 35 files belong in the same place
by the same method"). The cost: **only msvcrt.dll and ucrtbase.dll link that cohab**
(verified — they are the only two CMakeLists referencing `msvcrt_crt_cargo` /
`crt_shared_cargo`). msvcrt20, msvcrt40 and crtdll linked the pure C until 2026-09-25, when
they took msvcrt's whole cohab archive (see the CRT row); the math comes with it.

Three things worth keeping:

* **The file names lie about what compiles.** `math.cmake` lists `libm_sse2/sinf.asm` &c.
  for x86, which reads as hand-written SSE2 assembly. That arm is the `else()` of
  `if(USE_LLVM_MINGW)`, and we build WITH llvm-mingw, whose clang cannot assemble MASM. The
  arm that actually compiles (math.cmake:161-166) names `math/sinf.c`. Ordinary C, on all
  three arches. Check which arm is live before sizing anything else in this tree.
* **The Bessel bodies are vendored and were nearly paid for.** `_j0`..`_yn` are 102 lines of
  ours sitting on 1,346 lines of Sun FDLIBM under `math/ieee754/`, which `vendor.yaml`
  registered 2026-09-06 precisely so nobody translates them. Only the errno/`_fpclass` shell
  was ported; cnt-difftest compiles the FDLIBM once and has BOTH sides call it, so a row can
  only ever report a divergence in the part that is ours.
* **`_j0`..`_yn` are `cabi-math` ONLY, not `cabi`.** They call `_errno`, which the kernel
  does not have (not in ntoskrnl.spec). cnt-rust is one codegen unit, so an ungated `_errno`
  would make the whole math object fail the kernel link. Verified by `nm`: the i386
  `libcnt_rust_lib.a` defines zero `_j0` and carries no undefined `_errno`. Consequence —
  `cabi-math` is no longer a strict subset of `cabi`, and cnt-difftest enables BOTH so these
  six are still graded.

Two bugs fell out of writing the rows, which is the argument for writing them:
`div(7, i32::MIN)` panics in Rust where C wraps (`denom = -denom` overflows; the old corpus
dodged `INT_MIN / -1` but never put `INT_MIN` in the DENOMINATOR), and the `ldiv` reference
TU was LP32-vs-LP64 — Windows `long` is 32-bit, the host's 64, so a correct `quot=1/rem=-1`
came back as `0/0`.

Still owed here: the 5 BLOCKING `SWITCH_CONST_IN_BASE_HEADER` findings on
`float/i386/cntrlfp.c` (below) are untouched — the FPU control word was deliberately left
alone.

#### ucrtbase's own 132 lines — what actually blocks them — 2026-09-13

The goal above is "everything in ucrtbase that is ours is Rust", and `printf.c` +
`stubs.c` are the remainder. Two things blocked them; one is now fixed and the other is a
design decision, so nobody should treat this as a drive-by.

**Fixed:** `verify_ucrtbase_cohab` modelled "our C" as ucrtbase's five link LIBRARIES and
omitted the DLL's own TUs — 205 symbols on amd64, six of them exported (`fma`, `fmaf`,
`log2`, `log2f`, `lrint`, `lrintf`, stubs.c's placeholders for intrinsics spec2def cannot
stub). A Rust twin of any of those read as displacing a genuine Microsoft body. Now in the
authority; verified that none of the 205 is also defined by vendored ucrt, so the assertion
is not weakened.

**NOT fixed, and it is the real cost: these cannot be flipped by lld first-match.** Every
other cohab in this tree works because the C sits in a static ARCHIVE, so forcing the Rust
in under `--whole-archive` makes the C copies ignored duplicates. `printf.c`/`stubs.c` are
SOURCES of `add_library(ucrtbase SHARED …)`, so their objects are on the link line as
`$in`, ahead of `LINK_LIBRARIES` — and with the tree-wide `--allow-multiple-definition`
the FIRST definition wins, which is the C. A Rust `fma` added today would link, raise no
diagnostic, and be dead. Flipping therefore means removing the C from SOURCES, and
`stubs.c` bundles the six math stubs with `atexit`, `operator_new`, `operator_delete`,
`__acrt_initialize_sse2` and `_chkesp_failed` — so it is port-all-twelve (including the C++
operators) or split the file first. That is a deliberate change, not a translation.

Also note what is NOT owed here: `sprintf` must stay C — stable Rust cannot define a
C-variadic — and `lrint`/`lrintf` are `__debugbreak()` stubs, where a faithful port buys
arch-specific inline asm and no behaviour. `_vsnprintf` IS translatable (it takes a
`va_list`), but it is not a ucrtbase export; it exists because the legacy CRT references it,
and it needs the ucrt header's `_CRT_INTERNAL_*` printf option constants as substrate.

**And dbgrpt, the other half of that "shared-library lane", does NOT share this blocker —
the two are grouped by ownership, not by mechanics.** `add_library(dbgrpt misc/dbgrpt.cpp)`
produces a static `libdbgrpt.a` and msvcrt links it as a LIBRARY
(`msvcrt_static crtmath dbgrpt vcruntime wine chkstk`), so its six `_Crt*Report*` exports are
shadowable by the existing `--whole-archive` cohab rail exactly as crtmath's were — no CMake
surgery, no new rail. What blocks it is the translation itself: 500 lines of genuine C++,
`template <typename char_t>` with `char`/`wchar_t` specialisations feeding a traits-based
narrow/wide dual instantiation, plus MessageBox UI and OutputDebugString paths. c2rust cannot
anchor C++, so it is pure-LLM or hand, and it belongs to the C++ substrate lane rather than
here. Do not assume the ucrtbase link-order finding above applies to it.


#### crtmath sized, and the island question answered — 2026-09-06

`profile_derive sdk/lib/crt --target crtmath --usermode` derives cleanly (159 compiled TUs,
155 headers). Sorting those 159 by who actually owns them is the point of running it:

| bucket | files | lines |
|---|---|---|
| AMD LibM — `math/libm_sse2/`, vendored | 48 | 8,900 |
| Sun FDLIBM — `math/ieee754/`, vendored | 3 | 1,346 |
| i386-only (x87 asm shims, `_CI*`, `libm_sse2.c`) | 13 | 772 |
| already has a `cnt-rust` Rust body | 60 | 2,234 |
| **ours, still C** | **35** | **1,259** |

So the lane is ~1,259 lines, not the ~4,100 the raw file sizes suggest, and both vendored
trees are now in `vendor.yaml` — registering them is most of what made the number move.

**It does NOT want a bindgen island, and that is the design decision.** An island buys exact
struct layouts and header constants; crtmath has essentially no structs (`_exception` is
already in msvcrt-sys) and the surface is pure numerics plus the FPU control word. The 61
libm bodies already in `cabi/math.rs` were done without one, are graded per-symbol in
cnt-difftest against the same C `crtmath` compiles, and now ship in the kernel, msvcrt.dll
and ucrtbase.dll alike. The remaining 35 files belong in the same place by the same method.

What the derive DOES owe, and what the next lane must not skip:

* **5 BLOCKING `SWITCH_CONST_IN_BASE_HEADER` findings** — `_PC_24`, `_PC_53`, `_PC_64`,
  `_RC_DOWN`, `_RC_UP`, all `switch` labels in `float/i386/cntrlfp.c`, all owned by
  `float.h`, none in any `-sys` crate. Untranslated they become a silent catch-all arm
  (§9.112). Transcribe them into the crate, per the finding's own instruction; do not
  allowlist `float.h`, which re-emits the platform.
* **423 `.c` under `sdk/lib/crt` that nothing compiles** (`DEAD_SOURCE`). Not translation
  debt; do not pay for any of them.
* **Two islands would be needed IF one were ever generated** — `libm_inlines.h` and
  `rtlfuncs.h` both define `return`, and no TU includes both. That only matters for the
  AMD tree, which is vendored, so it is a fact to record rather than act on.

The five-consumer rail the paragraph above asks for is BUILT: msvcrt.dll (wave 14m),
ucrtbase.dll (2026-09-05), and msvcrt20/msvcrt40/crtdll (2026-09-25, via
`dll/win32/msvcrt/msvcrt_rust_cohab.cmake`) all whole-archive `cabi-math` ahead of
`libcrtmath.a`.

The regression is the one worth carrying forward: `_mbctype` was **two arrays**. `msvcrt.spec`
exports it as `@ extern _mbctype MSVCRT_mbctype`, mbcs.rs defined `MSVCRT_mbctype` without
`#[no_mangle]`, and the mangled Rust static therefore did not satisfy the export — mbcs.c's
copy did. `_setmbcp` wrote one; the exported `_mbctype` named the other, permanently zero —
`__p__mbctype` is not involved, it returns the per-thread `mbcinfo->mbctype`. Green everywhere,
because that table IS zero in every single-byte locale.
`verify_cohab_coverage.py`, the tool that exists to catch exactly this, took only the first
name on a spec line and so never saw the backing symbol — `extern` is the one directive where
the two differ. Fixed there first, which made the finding appear, and only then in mbcs.rs.
Scope measured, not guessed: 41 renamed data exports in msvcrt, this the only wrong one
(`_adjust_fdiv` already carried the same fix and a comment explaining it), and no other
Rust-flipped module in the tree has a renamed data export at all.

**Wave 32 — wcs.c's last three, $0.** `_wcsdup`, `towctrans` and `wcstombs`, hand-written into
`wcs_rest.rs`. Wave 16 left them because each delegates to something that was still C at the
time; all three delegates (`_towupper_l`/`_towlower_l`, and wcs_rest's own private
`wcsrtombs_l`) are Rust now, so these are three wrappers, not a paid re-translation of a file
that already has output. Two details kept rather than tidied: `_wcsdup` sizes its own copy and
`memcpy`s where `_strdup` calls `strcpy`, and `towctrans` folds *every* category that is not 1
to lower, including ones this msvcrt never issues. `wctrans_t` is spelled against
`msvcrt/include/wctype.h` in the file — it is a typedef, so it has no symbol for the island to
snapshot.

**Wave 31 — except_x86_64.c + except_arm64.c (10/10), $0.04.** The other two arms of the arch
gate wave 30 opened, and the end of the except family: `except.rs`'s `unsafe extern "C"` block
for `call_catch_handler` / `call_unwind_handler` / `get_exception_pc` / `handle_fpieee_flt` is
now answered by Rust on every arch instead of by a surviving C sibling. Both files are tiny —
four bodies and one `__ASM_GLOBAL_FUNC` each — because that is genuinely all the 64-bit
dispatcher cannot say portably: how to enter a handler on somebody else's frame, and where this
architecture keeps the faulting PC. Neither earns a `seh_*` companion module; one thunk stays
inline.

Worth carrying forward:

* **The asm proof generalises, including the unwind data.** `-d` agreement is necessary and not
  sufficient for a `.seh_*`-carrying thunk: a dropped `.seh_save_regp` leaves the instructions
  identical and the unwind info wrong. Comparing `.xdata` **and** `.pdata` against the C's own
  object closes it — arm64 `call_exc_handler` matches its 16-byte `.xdata` record exactly, and
  `_setjmp` and `get_exception_pc` have no `.pdata` entry on *either* side, which is the
  agreement that says the packed-vs-`.xdata` decision came out the same way.
* **A one-line `RtlUnwind` needed an island change on two arches and none on the third.** The
  `-DRtlUnwind=__wine_RtlUnwind` rename is inside `if(ARCH STREQUAL "i386")`, so bindgen sees
  the plain declaration on amd64/arm64 and the renamed one on i386. The allowlist now carries
  both spellings; they are never both present in one island.
* **A wine channel's ARGUMENTS can be the dangerous part.** `handle_fpieee_flt`'s FIXME prints
  `*(ULONG64*)ep->ContextRecord->Rip` — a load through the faulting PC, which the C performs
  only when the seh channel is on. Our sink is unconditional, so the arguments are deliberately
  left unevaluated; evaluating them to silence an unused-parameter warning would take a second
  fault inside a filter that is about to return CONTINUE_SEARCH.

**GRADED 2026-08-07 — reference TU #13, 9,982 rows, and it found a shipping bug.**

The first 32-bit harness: `sdk/lib/rust/msvcrt-seh-difftest` builds for
`i686-unknown-linux-gnu`, compiles the genuine `except_i386.c` beside the shipping
`except_i386.rs` + `seh_i386.rs`, and grades seven exports plus six of the TU's `static`
bodies (`get_scopetable_v4`, `msvcrt_local_unwind2`, `msvcrt_local_unwind4`,
`cxx_local_unwind`, `check_noexcept`, `MSVCRT_nested_handler`). The task that owed these rows
said the blocker was "a whole second harness variant"; it was three environment questions, and
all three answered yes — `clang -m32` builds and runs 32-bit ELF, `rustup target add
i686-unknown-linux-gnu` is one command, and `set_thread_area` + a `%fs` load gives a real
Windows-shaped TEB, so `fs:0` (the SEH chain head) and `fs:0x18` (`NtCurrentTeb`) mean what
both sides think they mean. That is what Wine does, and it is why the harness can exist.

What it caught, on the first run:

* **A C flexible array translated as a Rust array is a panic, not a read.** `SCOPETABLE_V4`
  ends in `SCOPETABLE entries[1]` — the flexible-array idiom, sized by the compiler to the
  function's `__try` count — and the translation indexed it as `entries[trylevel as usize]`.
  That is a Rust ARRAY index with a bounds check against the declared length of 1, and array
  bounds checks are not debug-only. So the shipping i386 DLL panicked inside its own SEH
  handler for **any** function with more than one `__try` level: eight sites across
  `_except_handler4_common` and `msvcrt_local_unwind4`. The sibling `msvcrt_local_unwind2`,
  reached through a `*mut SCOPETABLE` rather than a struct member, had it right — the two
  idioms sat six lines apart. Fixed with a `scope_entry()` helper that does what the C
  compiles to. **Nothing else in the tree could have found this**: it needs the module
  EXECUTED on its own arch, and until this harness there was no way to execute it at all.
* **`extern "system"` and `extern "stdcall"` diverge on a 32-bit Linux host, and the
  difference is silent.** `"system"` means stdcall only on 32-bit *Windows*, so the seven
  graded exports are cdecl here — but msvcrt-sys spells its Win32 imports `extern "stdcall"`
  explicitly, and rustc honours that on x86 whatever the OS. A harness stand-in written cdecl
  therefore left four bytes on the stack per call, and the following `ret` went to a
  stack-resident value: SIGSEGV with the PC *inside* the stack, nowhere near the cause. Same
  family as errno.rs's `_SetErrorMode@4`, from the other direction.

Three things the harness does that the next 32-bit one should copy: a `setjmp`/`longjmp`
recovery trampoline, because `call_finally_block` is `DECLSPEC_NORETURN` and the `__finally`
arm cannot otherwise be graded at all; an alarm + SIGSEGV watchdog that names the ROW, because
a scope chain that never reaches its sentinel spins with nothing to observe and a bare
segfault with no output is the one failure that tells you nothing; and `refstub/seh_layout.h`,
generated from the real i386 headers by `tools/gen_msvcrt_seh_layout.py`, because everything
under test is a walk BY OFFSET and an unchecked transcription would grade correct Rust as
wrong (§9.66).

Still owed, and recorded rather than skipped: `call_catch_block`,
`catch_function_nested_handler` and `se_translation_filter` — the catch-dispatch half, which
needs a real thrown C++ object matched by `find_catch_handler` and ends in
`continue_after_catch` restoring `%esp` from the throwing frame. The rows drive
`cxx_frame_handler` up to that boundary and stop.

**Wave 30 — except_i386.c (13/13), $0.21.** 32-bit SEH: an exception frame is a node in a
linked list rooted at `fs:0`, and the frame handler walks that list, where amd64/arm64 look the
frame up by PC in a table. Same exports, different implementations — which is what the
`#ifdef __i386__` around the whole TU and the `#ifndef __i386__` inside except.c mean, and why
both modules are `#![cfg(target_arch = "x86")]`-shaped rather than sharing bodies. rustconv
emitted the same **convention dual** here (`se_translation_filter` as a stdcall/cdecl pair);
this time the `not(x86)` twin was unreachable by construction and was deleted.

Three things generalise:

* **A dead preprocessor arm was on the paid bill.** `translate` charges per function and was
  quoting for the 7 bodies inside `#ifndef __REACTOS__` — a 30% surcharge for code in no object
  we build, on every wine-derived module. `dead_regions.never_compiled_lines` now drops them
  before anything is bought, and records each as an audit finding that says *absent on purpose,
  nothing owed* (as opposed to `--skip-fn`'s *hand-write it*).
* **`__ASM_GLOBAL_FUNC` has no C text, so it never reaches the translation** — it arrives only
  as commented-out C. The six thunks live in a hand-written `seh_i386` companion (the
  relationship `cpp_rtti` has to `cpp`), transcribed **verbatim in AT&T syntax** via
  `options(att_syntax)` rather than converted to Intel, so "transcribed" can be checked rather
  than trusted: `llvm-objdump -d` of the rlib's asm object against `except_i386.c.obj` is
  instruction-for-instruction identical at every offset, with the same
  `IMAGE_REL_I386_REL32 _cxx_frame_handler` relocation at 0x50. **Do this on any asm
  transcription** — it is a total check and costs one objdump.
* **A bitfield the island exposes as `set_X()` is a lucky compile error.** `handle_fpieee_flt`
  is a positional transcription of the x87 control and status words into `_FPIEEE_RECORD`, in
  which each of five flags appears three times (Status from the exception, Enable from the
  mask, Cause = Enable & Status). Plain field writes would have compiled if the fields were
  plain; because they are bitfields the body had to be rewritten against the C by hand. A pair
  swapped between two of those three slots is invisible to every other check we run.

**GRADED 2026-08-08 — reference TUs #14 (cpp.c) and #15 (except.c), 848 rows, and the two
divergences it found were both the ORACLE.** `tests/cpp.rs`, over the shipping
`msvcrt-crt-rust` with the `cpp` feature ON for the first time in a host harness.

The feature's own comment said that was impossible — `cpp_rtti`'s `global_asm!` vtables are
COFF-only, the RTTI walk needs `RtlPcToFileHeader`, "the module cannot link there at all" —
and it was wrong on both counts. The x86-64 arm of `asm_vtable!` emits `.data`/`.balign 8`/
`.quad`/`.globl`, valid ELF as written and the same text cxx.h's `__ASM_VTABLE` emits for the
reference TU beside it; only the i386 arm's leading `_` is COFF-specific. `RtlPcToFileHeader`
was never a platform property, only an undefined symbol. **The one genuinely PE-only thing is
`except_x86_64.rs`'s `call_exc_handler` thunk** — COFF `.def`/`.endef` plus four `.seh_*`
records over a Win64-ABI body — and unlike `seh_i386.rs` it has no honest ELF twin, because
re-spelling the directives would leave an invented calling sequence. It is gated to Windows
and left undefined elsewhere for the harness to supply as a loud out-of-reach sink.

Three instruments this file needs and no other suite here uses: cppimpl.c's **`RaiseException`
sink**, which records a throw's marshalling and longjmps back (the alternative, a fake unwind,
would let a row pass having graded a control flow that does not exist); **`fork()`** for the
arms that terminate, comparing the child's RAW wait status so `_exit(3)` and death by SIGABRT
are distinguishable; and **structural** comparison of handler pointers, since the two sides
install different functions and only "which sentinel, or the one I installed" carries across.

Both divergences it reported were in the reference, and both are the §9.66 class — the oracle
rewritten on the way in by the host's own model:

* **`typedef long NTSTATUS` is 64 bits on an LP64 host and 32 on Windows.** except.c's
  `float_exception_map[].status` therefore sign-extended every `((int)0xC000008D)` to
  `0xFFFFFFFFC000008D`, while the `DWORD ExceptionCode` it is compared against widens to a
  POSITIVE long — so the mapping loop matched nothing and every FP fault came out of
  `msvcrt_exception_filter` carrying the default `_FPE_INVALID` instead of its own subcode.
  Seven rows read DIVERGE with the subject correct. It also silently widened `_XcptFilter`'s
  first parameter.
* **`-Wl,--wrap=signal` redirects EVERY reference in the link, including the test's.** The
  wrap exists so Rust's startup `signal(SIGPIPE, SIG_IGN)` does not land in msvcrt's `signal`,
  which has no SIGPIPE and answers SIG_ERR. A row spelling `signal` was therefore grading
  cppimpl.c's `sigaction` shim against the genuine C. The subject is reached as
  `__real_signal`, the linker's name for the definition itself.

Two harness bugs the LINKER found, both of which a symbol-set comparison had already passed:

* **`_WIN64` is predefined for x86_64-w64-mingw32 and not for x86_64-unknown-linux-gnu.** The
  first reference TU to branch on it. Without it cxx.h's `__ASM_VTABLE` built the five vtables
  with i386's four-byte slots on an LP64 host, and cppexcept.h's `catchblock_info` came out
  four bytes short of the one the DLL walks.
* **A rename wall and `#` interact in exactly one place.** `#` suppresses expansion of its own
  operand and nothing else, so `VTABLE_ADD_FUNC(name) -> THISCALL_NAME(name)` sees a name the
  wall has *already* rewritten, and prefixing again produced `ref_ref_exception_what`. The
  `__ASM_NAME(#name ...)` uses in the same macro DO need the prefix. Same macro, two jobs.

**The `__UCRTSUPPORT__` config has no subject to grade, and that is the finding.** Both TUs are
compiled TWICE — into `msvcrt_static` without it and `msvcrt_shared` with it — and the second
build has a *complementary* live set: `signal`, `raise`, `terminate`, `unexpected`,
`set_terminate`/`_get_terminate` and `__pxcptinfoptrs` are all inside `#ifndef
__UCRTSUPPORT__` and vanish, while the eleven `#if _MSVCR_VER >= 80/140 ||
defined(__UCRTSUPPORT__)` bodies (`__AdjustPointer`, the four `__std_type_info_*`,
`_is_exception_typeof`, …) come alive. Wave 27/29 deliberately did not translate those eleven,
and they are still present in `cpp.rs` only as commented-out original C. So a second reference
TU under `-D__UCRTSUPPORT__` would compile a reference and compare it against nothing. It is
owed to the ucrtbase lane, not to this one — see the `ucrt_tls_sup.c` entry below.

Ungraded on purpose and listed by the suite itself rather than faked: `__CxxFrameHandler`,
`find_catch_handler`, `dump_function_descr`, `_fpieee_flt`, the three `__RT*` casts, and
`__CxxExceptionFilter`'s type-matching walk. The first six need a real PE unwind table and a
real stack; the last needs a LIVE thrown object's descriptor graph, which the sink's longjmp
out of the throw is what makes unavailable.

**Wave 29 — except.c (21/21), behind the `cpp` feature.** The signal layer (`signal`/`raise`,
the `sighandlers` table, the SEH filter that maps a hardware fault to a C signal) and the C++
frame handler, in one TU because they share one thread_data block. $0.28 sealed; no
`UNVERIFIED CONSTANTS` findings, and `find_catch_block`'s EIGHT positional arguments were
diffed against both C call sites by hand regardless.

Two things here generalise beyond msvcrt:

* **A convention dual is not an arch gate, and it looks like one.** rustconv emitted
  `#[cfg(target_arch = "x86")] extern "stdcall"` / `#[cfg(not(...))] extern "C"` pairs for the
  callbacks — correct, and unrelated to the fact that this file's whole 64-bit frame handler
  is inside `#ifndef __i386__`. Left ungated, the Rust `__CxxFrameHandler` would have won
  first-match over except_i386.c's completely different 32-bit SEH implementation, and
  **linked clean**. The object is the oracle: except.c.obj defines `__CxxFrameHandler`,
  `call_catch_block` and `cxx_local_unwind` on amd64/arm64 and none of them on i386.
* **A TU compiled TWICE with different `-D` has two different live sets.** except.c is built
  into `msvcrt_static` (msvcrt.dll, no `__UCRTSUPPORT__`) *and* `msvcrt_shared`
  (ucrtbase.dll, with it). Its `#if _MSVCR_VER>=140 || defined(__UCRTSUPPORT__)` block is
  undecidable from source — correctly so, since neither `profile_derive` nor rustconv can
  settle a disjunction with an unknown term — but the *object* settles it per target. Five
  bodies were `--skip-fn`'d on that evidence and recorded as audit findings.

**Two files that look meaty and are phantoms — do not target them.** `concurrency.c` (3,839
lines, the largest untranslated file in the module) sits entirely inside
`#if _MSVCR_VER >= 100`, and `handler4.c` (775) inside
`defined(__x86_64__) && _MSVCR_VER>=140`. msvcrt compiles at `-D_MSVCR_VER=0`, so **both build
to a 396-byte object with zero defined symbols.** `profile_derive` was blind to this — its
dead-code audit only knew wine's `config.h` `#undef`s, not a module's own `-DNAME=<int>` — and
the two together are 10.6% of the module, under the 15% module-wide threshold, so nothing
would have warned. It now evaluates integer gates and reports `DEAD_TU` at ERROR for any
single TU ≥90% dead; both are flagged.

**Wave 27 (2026-08-05) — cpp.c FLIPPED ($0.21). The C++ RTTI/exception subsystem.**

47 spec exports, 51 compiled functions: the `type_info` / `exception` / `bad_cast` /
`bad_typeid` / `__non_rtti_object` family, the three RTTI entry points (`__RTtypeid`,
`__RTDynamicCast`, `__RTCastToVoid`), `_CxxThrowException`, and the terminate/unexpected/
se_translator handler slots. The largest single block of C left in msvcrt. 49 functions
translated; the macro-generated half is hand-written in `src/msvcrt/cpp_rtti.rs`.

**Two feasibility questions, both answered NO-BLOCKER, and both worth recording because the
opposite is widely assumed:**

* `extern "thiscall"` **is stable** on our pinned rustc 1.92, and lowers exactly like the C on
  i386: `this` in ECX, `retl $N` callee-cleanup, undecorated symbol. It is **x86-only**
  (`E0570` on amd64), so every method needs the per-arch dual — which is what msvcrt.spec
  already spells out, `thiscall -arch=i386` beside `cdecl -arch=win64`/`-arch=arm`.
* The `__TRY`/`__EXCEPT_PAGE_FAULT` in the three RTTI entry points is **not real SEH here**.
  msvcrt compiles with DUMMY_PSEH, where `_SEH2_EXCEPT(f)` is `if (_SEH2_Volatile0 || (0 && f))`
  — compiled, never taken (the tell: `_SEH2_Volatile0` undefined in the object, no
  `__C_specific_handler`, no scope table). So they translate to straight-line Rust plus a
  runtime-false branch. **Rust needing SEH was not the blocker anyone expects it to be.**

**Substrate: cxx.h and cppexcept.h joined the msvcrt-sys island**, and deriving rather than
transcribing mattered more here than anywhere so far. cxx.h defines every RTTI struct TWICE
under `#ifndef RTTI_USE_RVA`, and the two arms differ in **field TYPE, not field name**: i386
holds real `const type_info *` pointers where amd64/arm64 hold 32-bit RVAs off the image base.
A hand-written Rust struct necessarily picks one arm and is silently wrong on the other two,
and "wrong" means dereferencing an image offset as an address. `CXX_EXCEPTION_PARAMS` came back
**3 on i386 and 4 elsewhere** — simultaneously the length of `_CxxThrowException`'s argument
array and the count handed to `RaiseException`, i.e. exactly the constant a transcription gets
wrong. Five new `GUARDED` pairs (`__exception.vtable/name/do_free`, `__type_info.name/mangled`),
27 total.

`cpp_rtti.rs` holds what rustconv **structurally cannot see** — not functions it missed, but
text that is not in the file: five `global_asm!` vtables (the RTTI pointer lives at
`vtable[-1]`, so a Rust `static`, which gives one symbol at its START, cannot express it), the
RTTI and throw descriptors with their RVA fixups, and the seven bodies
`CREATE_EXCEPTION_OBJECT`/`CREATE_TYPE_INFO_VTABLE` expand to.

**Four tool bugs, three of them silent, all fixed rather than worked around:**

* **The scanner dropped 25 of 62 bodies without saying so.** `TYPE * __thiscall name(...)` is
  the position tree-sitter cannot parse in any spelling — the documented `FILE * CDECL
  fopen(...)` class that cost file.c 15 functions — and `__thiscall` was simply never in the
  convention table the repair regex is built from. The split fell along the RETURN TYPE, so
  `void __thiscall bad_typeid_dtor` parsed while `bad_typeid * __thiscall bad_typeid_ctor`
  beside it did not, and the plan read as *partly* translatable rather than broken. Audited the
  tree: no shipped translation lost anything to it.
* **`thiscall` was collapsed to `extern "system"`** in three places. Not a spelling of the same
  ABI — stdcall takes `this` on the stack, thiscall in ECX — and every downstream check still
  agrees (same export name, arity, cleanup). Only the register differs, only on i386.
* **`fix_fastcall_exports` rewrote `-> *mut exception` as `-> *mut`** (return-type regex stopped
  at the first space) and could not see a parameter containing parens, i.e. every `Option<fn>`.
  Both invisible until now because ntoskrnl's fastcall exports return single tokens.
* **`--skip-fn` silently kept only the last name** when the flag was repeated — ten of eleven
  skips dropped, from the one flag whose stated contract is "never silent".

**Three defects in the translated output:**

* **An INVENTED `#if` again** (wave 26's class, next file): `get_obj_locator_base` got
  `all(target_pointer_width = "32", feature = "rtti_rva")` for `#ifdef RTTI_USE_RVA` — inverted
  AND gated on a feature that does not exist, so the arm was dead on all three arches and every
  64-bit RTTI lookup would resolve against base 0. An undeclared feature is not an error; it is
  a permanent `#[cfg(false)]`. `msvcrt_init_exception` keyed the same condition on pointer
  width — accidentally right on the three arches we build, still the wrong term.
* **The dead `__EXCEPT_PAGE_FAULT` arms shipped as unconditional code**, so all three RTTI
  queries would throw `__non_rtti_object` on every call. This is `--seal-calls` pushing the
  wrong way: its invariant is "every call appears and is executable", and the model satisfied it
  by deleting the branch condition. All three were rejected once and came back this way — so
  **when a rejected body returns accepted, diff its control flow, not its calls.**
* `InterlockedCompareExchangePointer` was missing from `prelude.rs` — same intrinsic story as
  the rest of its family, and it needs its own lowering rather than a cast (64-bit CAS).

**Verification beyond `cargo check` ×3:** every `#[cfg]` diffed against every `#if`; the
intra-family call set of all 43 bodies present on both sides diffed C-vs-Rust (zero divergence);
and the vtable argument each ctor hands the shared `__exception_ctor` checked as a table, since
that is the permutation class the fact-seal cannot see — a swap there compiles, links, and
constructs an object of the wrong class.

The 11 `_MSVCR_VER >= 80/140 || defined(__UCRTSUPPORT__)` bodies are deliberately untranslated:
msvcrt compiles at `_MSVCR_VER = 0` without `__UCRTSUPPORT__`, so nothing builds them. Each is
recorded as an audit finding. **Also: `concurrency.c` is 3,839 lines that compile to ZERO
functions here — the largest dead file in the module. Do not pay to translate it.**

**Wave 26 (2026-08-04) — heap.c FLIPPED ($0.18). The allocator is Rust; 87.8% of msvcrt.**

37 functions, 855 lines, and the file every other translated module in this crate already
stands on. `heap.c.obj` is now fully covered — zero exports left C — which took amd64 from
1,060 to **1,083 of 1,234** non-C++ exports (85.9% → 87.8%), and reached **8 `?`-mangled
exports** (`operator_new`, `operator_delete`, `operator_new_dbg`, the `_set_new_handler`
family) that had been counted as the separate C++ lane.

Substrate first, and it was most of the work. Nothing in msvcrt-sys covered this file: the
island gained `RtlAllocateHeap`/`RtlReAllocateHeap`/`RtlFreeHeap`/`RtlSizeHeap`/
`RtlValidateHeap` + `HeapCreate`/`HeapDestroy`/`HeapValidate`/`HeapCompact`/`HeapWalk`, the
`_heapinfo`/`PROCESS_HEAP_ENTRY` pair, `TEB`/`PEB`, and 15 constants.

**Read the header that WINS.** msvcrt does `include_directories(BEFORE sdk/include/wine)`, so
`#include "winternl.h"` resolves to sdk/include/wine's copy — the one with real fields
(`PEB.ProcessHeap` /* 018/030 */, `TEB.Peb` /* 030/0060 */). The psdk header of the SAME NAME
spells those bytes `PVOID Reserved4[3]` and calls the TEB field `ProcessEnvironmentBlock`.
Picking the wrong one is not a compile error; it is an allocator reading its heap handle from
the wrong offset. Both hops are in `GUARDED`, so the two guardians now fail the build if
either struct moves.

Four defect classes the compiler could not see, all in this one file:

* **An INVENTED `#if` condition, and it is the one that cost a gate.** The C guards
  `msvcrt_init_heap` with `#if _MSVCR_VER <= 100 && !defined(__REACTOS__)` — false here, so
  the `#else` (`heap = NtCurrentTeb()->Peb->ProcessHeap`) compiles on all three arches. The
  translation mapped that onto `target_pointer_width`. i386 ran the DEAD `HeapCreate` arm and
  **booted green** (a private heap is still a valid heap — that is what hid it); amd64 and
  arm64 got the PEB pointer as their heap handle, so every allocation failed and user-mode
  starved with 15 threads parked on `WrFreePage`/`WrQueue`/`WrLpcReceive`. `msvcrt_destroy_heap`
  had the same invented condition, which on i386 would `HeapDestroy` the process heap.
  The free check, now standing practice: **diff the `#[cfg]` list against the `#if` list** and
  require correspondence. Field manual, the build-configuration `#ifdef` entry.
* **`NtCurrentTeb()->Peb->ProcessHeap` appears 5 times and was translated 5 different ways,
  4 of them wrong** — two stopped one hop short and used the PEB pointer itself as the heap
  handle; a third walked the PEB by a hard-coded `0x18`, which is ProcessHeap's **i386**
  offset, so it read a neighbouring field on amd64 and arm64. The guarded offsets cannot
  catch this: they prove where the fields LIVE, not how many times a body dereferenced.
* **An absent constant is not sealed, it is INVENTED.** `_heapset` shipped `!= 1` for
  `!= _HEAPOK` (-2) and `== 5 { 1 }` for `== _HEAPEND ? _HEAPOK` (-5, -2) — as written the
  loop body never executes. `--seal-facts` was on; it can only seal a constant the ISLAND
  CARRIES, and the `_HEAP*` family was not in it at call time. Distinct from the recall
  PERMUTATION entry: that needs the constant present, this needs it absent. It also falls in
  `profile_derive`'s blind spot, whose switch-constant audit reads `case` labels only —
  every one of these is in a `while`/`?:`.
* **The assembler split `_set_sbh_threshold`**, emitting its signature into the file-scope
  preamble with no body (whose `{` then swallowed every following item) and leaving the body
  headerless 400 lines down. The parse failure MASKED name resolution: the first `cargo
  check` reported 21 errors, the real count was 40.

Verification that came back clean, and is worth repeating on the next file: diffing the C's
whole constant vocabulary against the Rust showed `EINVAL` 13→9, `ENOMEM` 3→2, `ERANGE` 8→4,
which reads as dropped parameter checks. Stripping the uncompiled `#if _MSVCR_VER>=X` arms
first gives 9/9, 2/2, 4/4 — nothing dropped. Compare against the COMPILED C, never the file.

Status is **PORTED, NOT GRADED**, and the `heap` feature is why: msvcrt-difftest is a Linux
host binary, and linking `malloc`/`free`/`calloc`/`realloc` into it replaces glibc's allocator
with one calling `RtlAllocateHeap`. Lifting that needs a heap shim in `refstub/` mapping the
Rtl* family onto the host allocator, the way `fileimpl.c` does for the fd table.

Also closed here: `InterlockedCompareExchange`/`InterlockedCompareExchangePointer` had been
sitting in the island's `--allowlist-function` emitting NOTHING, silently — the family is
`__INTRIN_INLINE`, a compiler intrinsic with no symbol. `prelude.rs` documented that and
nobody removed the dead entries. An allowlist entry that matches nothing reads as coverage.

**GRADED 2026-08-07, and it was wrong twice.** undname.c is reference TU #8 —
`tests/undname.rs`, 63,216 rows over a corpus build.rs scrapes from every `.spec` in the
repo (878 genuine mangled names; a hand-written list would grade the cases we thought of and
would rot). It was the cheapest file in msvcrt to grade and the last one anybody worried
about, which is the point: **the demangler had been shipping two wrong answers, and the first
row that reached each one failed.**

*Every identifier was one character short.* `sym->current - 1 - ptr` parses as
`(current - 1) - ptr`; the translation folded `current - ptr` into `offset_from` and kept the
`- 1` as well. `CBaseUnknown` demangled as `CBaseUnknow`, `_GUID` as `_GUI`.

*Every function signature parsed its return type from the calling-convention letter.* The C
is `get_calling_convention(*sym->current++, …) || demangle_datatype(sym, …)` — the `++` is a
side effect inside the first ARGUMENT, so the cursor moves before the second operand reads
the same `sym`. The translation hoisted it to after the whole `if`. `A` reads as a reference,
so `void * (__cdecl*)(long)` came out ` & (__cdecl*)(void,long)` with the real return type
shifted into the argument list. Both forms advance the cursor exactly once — nothing leaks,
nothing faults, no scanner can see it. Field manual, "Semantic rewrites that flip meaning".

The rows compare more than text: the returned POINTER's identity (with a buffer both sides
must hand it back, and `if (buffer && buflen)` means a zero length takes the allocate path
instead — both sides must choose the same), a per-side ALLOCATION LEDGER (memget/memfree are
the harness's, so `und_free_all` releasing everything is directly observable and is the half
no text comparison reaches), and truncation over five buffer sizes around the exact fit with
a `0xAA` tail past the terminator.

**Wave 25 (2026-08-04) — undname.c FLIPPED (1,788 lines, $0.55), and ctype.c CLOSED.**

`undname.c` is the Visual C++ symbol demangler behind `__unDName`/`__unDNameEx`, and the
ratio is the story: 1,788 lines, 27 functions, and exactly **two** exports. Everything else
is `static`, so defining those two means the C object is never pulled from the archive and
all 27 helpers go dead in one step — no partial-flip subtlety of any kind. It is also the
most self-contained file in msvcrt: no locale, no fd table, no global state, no Win32 beyond
the caller-supplied malloc/free in `parsed_symbol`. A recursive-descent parser with its own
bump arena.

Three rejected workers (`str_array_push`, `get_args`, `symbol_demangle` all damaged their
fact envelope) passed on `--retry-rejects 2` for $0.17, with the other 24 replayed from the
resume cache for $0. `str_printf` is HAND-WRITTEN and is the only one: it is a `static
WINAPIV` variadic, so the C-shim trampoline answer that file.c and process.c use for their
twenty cannot reach it — a shim cannot name a `static`, and it calls the equally-static
`und_alloc`. The variadic list becomes an explicit `&[PArg]` slice behind a `str_printf!`
macro, so the 52 call sites still read as they do in the C.

**Two literal permutations, one of them in already-landed code.** `case 'C'/'D'` shipped as
`"__stdcall"` where the C says `"pascal"`; and time.c's `GetTimeFormatW` picture string
`L"HH':'mm':'ss"` had shipped as `HH:'mm:'ss` — `'` quotes the character it surrounds, so
that asks for the locale's separator instead of a forced colon. Neither is invented, neither
fails to build, and neither is reachable by the constant-vs-header diff, because both are
STRING literals. `tools/verify_translation_literals.py` compares them as a multiset per file
and is now in `vibeos-cli check` with a baseline; field manual, the sealing/RECALL entry.
Also made inert: undname.c's two `#ifdef _UCRT` match arms, which this DLL does not compile.

`ctype.c` is now fully Rust. Its 44 functions were already hand-written in waves 9/10 — a
rustconv pass over the file re-derived 23 of them independently and agreed with every mask,
which is a free confirmation rather than a replacement, so it is archived and not assembled.
What was left was `_ispunct_l` (an object export that a comment here claimed did not exist —
the DLL export list is not the object's symbol table, and it is the object that decides
whether the C body stays live) and the three DATA symbols. Those two 257-entry tables are
**generated, not copied**, by `tools/gen_ctype_tables.py`: it parses ctype.c's initializers,
resolves the `_MSVCR_VER`-gated arms, evaluates every entry against the msvcrt-sys island's
masks, and refuses to emit unless all 514 match the bytes in `ctype.c.obj`'s `.data`. Two
independent derivations agreeing is a stronger check than a differential test, which is why
the `ctype-tables` feature costs no gradeable coverage when the harness drops it.

`UNDNAME_*` is now derived in the island (imagehlp.h added to the bindgen wrapper) — 17
constants that are pure bit positions, where an invented value demangles with the wrong
options rather than failing.

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
- **CRT floor — ~0–5%.** *(`sdk/lib/crt/mbstring` + `misc`: **CLOSED PERMANENTLY**, owner
  decision 2026-09-13, on the measurement below. Nothing that ships links the `crt` target and
  `libcrt.a` is not produced by a normal build, so translating them improves exactly one
  apitest. Do not re-cost this; if you are about to, read the paragraph after this one first.)* `sdk/lib/crt/` = 581 `.c` (printf/scanf/string/math/locale/
  mbstring/wine) still compiled; only `crt-startup/dllmain/crt0` entry-glue + the
  `libcntpr` kernel subset (`cnt-rust`) are Rust. Largely irreducible variadic.
  **But read that 581 carefully before costing any of it: much of the tree belongs to the
  `crt` TARGET, and nothing that ships links `crt`.** Measured 2026-09-13 — the only
  `target_link_libraries` naming it is `modules/rostests/apitests/crt/static_crt_apitest.cmake:52`,
  and `sdk/lib/crt/libcrt.a` is not even produced by a normal build (it is a ninja target with
  no built artifact in `build-amd64`). crtdll, which is the plausible consumer, links
  `msvcrt_static crtmath vcruntime chkstk wine` instead. Concretely for the two directories
  that look biggest: `CRT_MBSTRING_SOURCE` and `CRT_MISC_SOURCE` feed `crt.cmake` ALONE, while
  shipping gets exactly `mbstring/mbstrlen.c` and `misc/fltused.c` through `libcntpr` — so
  translating the other ~55 mbstring and ~16 misc files would improve one apitest and nothing
  in the OS. The parts of `sdk/lib/crt` that DO ship are the ones already named elsewhere:
  `libcntpr` (kernel, Rust via cnt-rust), `crtmath` (done 2026-09-13), `msvcrtex`, and the
  `msvcrt_static` set.

  **Object-level measurement of the three shipping static libs (2026-09-13, all three
  arches).** Read the `CMakeFiles/<target>.dir` object list, not the `list(APPEND …)` source
  vars: parsing the cmake source lists says `libcntpr` has **107** live `.c`, and the truth is
  **21** (amd64) / **20** (arm64) / **28** (i386). The 4.6x over-count is arch `if()` guards the
  parse ignores — the `LIBCNTPR_*_SOURCE` families are shared with the user-mode `CRT_*` vars.
  Pairing those 107 against `cnt-rust`'s 216 `#[no_mangle]` exports by filename produced **37
  apparent double-definitions (`sinf`, `frexp`, `ldiv`, `strtod`, …) and every one was a
  phantom** — none of those `.c` compiles into `libcntpr` on any arch. Pair by compiled object,
  never by filename.

  What actually remains, and why each is not a translation target:
  - `libcntpr` — 12 C-variadic `printf/` wrappers (irreducible), plus the documented kept-C
    (`wctype.c` dead data table, `winesup.c`, `atol.c`, `mbstrlen.c`, `fltused.c`, `exp2f.c`,
    `_invoke_matherr.c`, `scanf_trampoline.c`) and `except/` SEH. **i386 carries 8 more, all
    structurally blocked:** `i386/ci.c` + `ci{cos,log,pow,sin,sqrt}.c` are x87-stack-argument
    thunks (`FPU_DOUBLE` = `fstpl`) whose math is *already* Rust — the thunk is inline asm glue,
    and `cisin`/`cisqrt` were already settled as headerless cc-rs shims; `i386/libm_sse2.c` is 32
    `__m128d`/`__m128` by-value functions needing `target("sse2")`, which cannot exist in
    `cnt-rust` because i386 builds `-sse,-sse2` (the same ABI-required-target-feature error that
    forced every `cabi/math.rs` helper to `extern "C"`); `nexttowardf.c` takes an x87 80-bit
    `long double`, which Rust has no type for. So `libcntpr` is done.
  - `msvcrtex` — **translated** (`crt-startup-rust` + `crt0-rust` + `crt-dllmain-rust`, ~190
    symbols, commit `94f2d421748`) but wired through **`reactos-link`'s `.msvcrtex_crt()`**, not
    CMake — so grepping CMakeLists for the crate name finds nothing and it looks orphaned. It
    ships into *cargo* modules; the 44-TU C archive still ships into every module that links
    `libmsvcrt` the C way (`dll/win32/msvcrt/CMakeLists.txt:311`). Retiring the C archive is an
    architectural call, not a transcription.
  - **`user32_wsprintf` — 12 TUs, and the one genuine unclaimed CRT frontier left.**
    It is the LAST target in the tree still compiling `printf/streamout.c` +
    `wstreamout.c` as C: the 708-line engine is already Rust in `cnt-rust`
    (`cabi/streamout.rs`) and flipped for `libcntpr` (`libcntpr.cmake:125`). The other
    10 TUs are 4 trivial leaves (`strnlen` 5 lines, `wcsnlen` 6, `mbstowcs_nt` 66,
    `wcstombs_nt` 57) and 8 irreducible variadic shells.

    **It is not free wiring, because `-D_USER32_WSPRINTF` (`sdk/lib/crt/CMakeLists.txt:47`)
    is a different CONFIGURATION of that engine**, and `cabi/streamout.rs:294` says those
    paths are deliberately not compiled. The delta is bounded and fully enumerable — six
    points, mostly subtractive:
      1. no `_IOSTRG`/`_base == NULL` early-return in `streamout_char`/`_astring`/`_wstring`
         (3 sites, `streamout.c:231/257/285`);
      2. the buffer path is `defined(_USER32_WSPRINTF) || defined(_LIBCNT_)` — ALREADY
         what the Rust implements, so this one costs nothing;
      3. `USE_MULTISIZE 0` (`:318`) — no size modifiers;
      4. `*` field width and precision emit the literal `*` instead of consuming a
         `va_arg` (`:376`, `:407`);
      5. no float conversions — the `G/E/A/g/e/a/f` cases are `#ifndef`-ed out (`:548`);
      6. `fieldwidth += 2` for the `FLAG_SPECIAL` `0x` prefix (`:608`).

    **Do NOT implement this as a cargo feature on `cnt-rust`.** Cargo features are an
    additive UNION and this one is not additive: it changes the behaviour of symbols that
    already exist rather than adding new ones. `cnt-rust` has 10 dependents and is one
    codegen unit, so unification would hand `libcntpr` — the KERNEL's printf — the user32
    variant, silently and with no diagnostic. Its own Cargo.toml already documents this
    hazard for `cabi`/`cabi-math` ("The msvcrt DIFFTEST must NOT see this feature").
    The right shape is a separate crate, exactly the precedent `crt-shared-rust` set when
    ucrtbase needed its own configuration of the shared TUs.

    Link mechanics, checked: there is no `--whole-archive` rail in `sdk/cmake`, and
    `win32ss/user/user32/CMakeLists.txt:126` lists `user32_wsprintf` BEFORE
    `${USER32_RUST_LIB}`, so a Rust twin loses to the C by lld first-match. Unlike
    ucrtbase this is fixable: `user32_wsprintf` is a plain static lib whose source list
    can be edited, which is the ordinary cohab pattern. Scoped 2026-09-13, not started —
    user32 is on the boot path, so this wants its own session with a 3-arch smoke, not
    the tail of one.


  Recorded because the obvious check misleads: `grep -rl '\bcrt\b' --include=CMakeLists.txt`
  "finds" msvcrt, ucrtbase, ntdll and services, every one of them a substring match on
  `msvcrt`/`crtmath`/`_crt`. File-mentions-it is not links-it, and I asserted the wrong version
  of this before checking the link graph.
- **LSA/SAM server lane — in flight.** samsrv (~90%, `samsrv-rust` 16 `.rs`) and lsasrv
  (~50%+, `lsasrv-rust` 20 `.rs`) are the active server-side frontier.
- **kernel32/advapi32 tails.** kernel32 NLS `locale.c` + resources; advapi32 `wine/
  security.c` + generated RPC.
- **shell32 — every translatable TU is Rust (2026-09-23).** `shell32.cpp`, the last one, went in
  three phases: its free functions, the COM server (class-object table, cached factories,
  `DllGetClassObject`/`DllCanUnloadNow`), then `DllMain` and registration — which carries a Rust
  port of ATL's static registrar (`shell32/statreg.rs`), proven byte-identical to the C++ build's
  DllRegisterServer/DllUnregisterServer output on all three arches by `tools/payloads/regdiff`.
  **Since 2026-09-24 shell32 compiles no C or C++ at all** (`rustconv shell32-tail-inventory`:
  0 cpp, 0 c). The last "sanctioned glue" went too:
  - the three ATL window classes (`CStubWindow32`, `CMSGlobalFolderOptionsStub`,
    `CChangeNotifyBroker`) are on the crate's `atlwin` substrate
    (`shell32-rust/src/atl_stub_windows.rs`), now that every constructor is Rust;
  - `ShellMessageBoxA` receives its varargs as fixed slots (va-slots-rust);
  - `StringCchPrintfW` is `strsafe_rust::StringCchPrintfW!`, which calls msvcrt's variadic
    `_snwprintf` directly.
  They were A/B-checked against a baseline shell32 in-guest on all three arches by
  `tools/payloads/shelltail`. The paragraphs below are the history of getting here.
- **shell32 activatable COM surface — 33 of 33 classes Rust-owned.**
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

  **The `shellmenu/` cluster is DONE except `CMenuToolbars.cpp` (2026-09-20).**
  `CMenuFocusManager.cpp`, `CStartMenu.cpp` and `CMergedFolder.cpp` all left CMake together;
  `CMenuToolbars.cpp` is the only shellmenu source the build still compiles, blocked on gap (2)
  below (`CWindowImplBaseT<CToolbar<DWORD_PTR>>`, plus a derived and a base class that both
  define a non-trivial constructor, which flattening cannot sequence). Two things in that flip
  generalise beyond shellmenu:

  * **A `*_RustBridge` shim does not disappear when its class is translated — it MOVES.** Each
    flipped .cpp had been publishing shims that already-Rust modules call, so the symbols were
    re-published from Rust under identical names and ABI (`menu_focus_manager_support.rs`,
    `enum_merged_folder_support.rs`, `merged_folder_support.rs`). Only one needed thought:
    `CEnumMergedFolder_CreateInstance_RustBridge` was `new CComObject<T>()`, which starts at
    refcount **0** and is AddRef'd to 1 by the assign helper, whereas the generated
    `_CreateInstance` follows the COM activation contract and returns **1**. Forwarding naively
    leaks the enumerator for the life of the process.
  * **The module-lock symbol belongs to the output CRATE, not to `--target`.** `menuband-rust`
    links into shell32.dll AND rshell.dll, so it must call `MenuBandRustModuleLock` (both DLLs
    define it) and not shell32's private `Shell32RustObjectLock`. A `--target shell32` run
    emitted the latter, which linked shell32 perfectly and broke rshell — and only at the flip
    that first made an rshell-reachable object pull the module out of the archive, several TUs
    later. Now derived per-crate by `reactos.targets.module_lock_for_crate()`.
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
  (`shlexec.cpp` and `shlfileop.cpp` left that list on 2026-08-01, and `dialogs/drive.cpp`,
  `dialogs/folder_options.cpp` and `dialogs/item_prop.cpp` on 2026-08-02; the tail is tracked by
  `rustconv shell32-tail-inventory`.) The exhaustive zero-C++ plan and
  ledger are in `docs/engineering/active-lanes/shell32-cpp-tail-plan.md`.
  **The remaining tail is gated on three SCAFFOLD gaps, not on translation budget** (2026-08-05,
  measured by running `rustconv cpp-tu` over all 22 live TUs). Only four pass preflight outright;
  every other failure names one of: (1) an activation shape the scaffold does not model — it knows
  `OBJECT_ENTRY` and `ShellObjectCreator<T>`, and the tail is full of `new CComObject<T>()`
  (`CEnumMergedFolder`) and plain internal construction; (2) an inherited layout it will not guess
  — `CWindowImpl<>`/`CWindowImplBaseT<>` (`CDefView`, `CStubWindow32`, `CMenuToolbars`) and
  `IDispatchImpl<>` (`CFolder`, `CFolderItems`, `CFolderItemVerbs`); (3) object-field C++ RAII
  (`CAtlList<>`, `CComHeapPtr<> m_pidls[2]`, `CContainedWindow`). The SDK-enum blocker that used
  to sit alongside these is **gone**: `island_integer_aliases` now resolves alias chains into the
  `-sys` crate (176 -> 417 proven types), which alone moved `CRegFolderEnum` from UNSUPPORTED to
  translatable and left `CEnumMergedFolder` with only its activation shape.
  **Re-measured 2026-08-05 after three more flips, now per-TU** (`rustconv cpp-tu` over every
  live TU). Gap (3), object-field RAII, is the cheapest to close and blocks exactly three:
  `CIDLDataObj` (`CSimpleArray<FORMATETC>`/`<STGMEDIUM>`), `CDefaultContextMenu`
  (`CAtlList<>` ×2, 1,995 lines) and `CRecycleBinFolderViewCB` (`CComHeapPtr<ITEMIDLIST>
  m_pidls[2]`). The leverage is that `atl.rs` **already implements** CComPtr / CHeapPtr /
  CComHeapPtr / CSimpleArray with `Drop` restoring the dropped destructors, and
  `--local-raii NAME=PATH` already performs exactly this substitution for BODY-locals — there
  is simply no equivalent for a class's STATE fields.
  **Closed for two of the three, 2026-08-05, and the third turned out to be a different gap.**
  The scaffold now keeps an ATL container's TYPE in the Rust owner (it cannot be flattened to
  a pointer the way `CComPtr` is, because its destructor walks storage the scaffold does not
  own), initialises it with `new()` instead of `zeroed()`, and emits a `drop_in_place` per
  container in the generated `Release` — after the explicit dtor body, which is C++'s own
  order. `CComHeapPtr<T> name[N]` is accepted too, freeing each element. `CIDLDataObj` is
  preflight-green; `CRecycleBinFolderViewCB` has its state field accepted and is now blocked
  only on activation. **`CDefaultContextMenu` is not an object-field-RAII case at all** —
  `CAtlList<DynamicShellEntry>` and `<StaticShellEntry>` hold C++ objects with their OWN
  destructors (`CComPtr<IContextMenu>`, `CStringW`), and `atl.rs`'s container `Drop` runs one
  `drop_in_place` per element, which reproduces the C++ only when the element is trivially
  destructible. So the element type must be PROVEN trivially destructible or the container is
  refused, and the proof is the bindgen island: bindgen only ever renders C declarations, so a
  name it emits as a struct came from a C header. A C++ class is never in that set.
  **CLOSED 2026-08-05.** The island is one of TWO safe families; the other is a
  source-discovered POD **rendered owning**. Two defects sat behind it: the element type was
  never discovered at all (the candidate scan only matched a plain `Type name`, so an element
  inside a template argument was invisible — which made the refusal accidentally-correct rather
  than a proof of anything), and the POD renderer spelled ATL members with the COM *owner's*
  type. That is right for an owner, whose generated `Release` releases explicitly, and fatal in a
  POD, whose field types are its entire destructor. `PlainStateField.owned_rust_type` keeps the
  owning spelling (`CComPtr<T>`/`CComHeapPtr<T>`/`CStringW`), all `repr(transparent)`, so
  layout is unchanged and Rust's field-drop order *is* the C++ implicit destructor. The
  free-function renderer had the same bug and the shipped `brfolder.rs` is what proved it — it
  already had the owning spelling while the generator did not, so a regeneration would have
  downgraded a correct file. Repo sweep: 12 C++ structs hold ATL members, only `BrItemData` has
  a Rust port, so nothing shipped carried the leak. `CAtlList<T>` itself is implemented in `atl.rs` (the
  measured surface: GetHeadPosition/GetNext ×7, IsEmpty ×5, RemoveAll/GetAt/FindIndex/AddTail
  ×2, AddHead ×1) and returns a null POSITION on OOM where ATL throws, which is stated at the
  type because it silently drops an entry rather than abandoning the operation.
  **`CIDLDataObj.cpp` is flipped on the back of it** (2026-08-05, $0.16) — the shell's
  `IDataObject`/`IAsyncOperation` for a PIDL selection, its `IEnumFORMATETC`, and the TU's four
  constructors, as three modules: the assembler writes one file per class slice, and a TU only
  leaves CMake when every slice has an owner. No bridge was needed (both constructors sit inside
  `wine/shell32_main.h`'s own `extern "C"`; both `SHCreate*DataObject` are WINAPI in shlobj.h).
  The container contract held — the model wrote `m_Formats.GetAt(n)` from the sealed usage
  mapping instead of inventing a wrapper. Three tool fixes fell out of the grind, each of which
  had been paid by hand before: `--seal-facts` was sealing the function's OWN parameter TYPES
  (unsatisfiable for a COM method, whose signature comes from the vtable authority — the whole
  parameter list is excluded now); the SDK-record renderer emitted a template-erased SHADOW of
  `CSimpleArray` that beat the real one by scope; and the stitcher's substrate globs collide
  with the scaffold's prelude glob on every shared name, which is what those 339/339/399 opening
  error counts were. Defects worth naming in the paid bodies: the free-function slice INVENTED a
  `CIDLDataObj` struct (it cannot see the slice that owns the real one), `Clone` used the
  generated worker name as a VTABLE SLOT, and `SHAlloc(size)` passed a DWORD to a SIZE_T
  parameter — invisible on i386, caught by both 64-bit targets.
  **`CDefaultContextMenu.cpp` is BOUGHT but NOT FLIPPED** (2026-08-05, $0.41, 43/45 workers) —
  the 2,024-line TU behind every right-click. Preflight is fully green now: 5 interfaces, 33
  vtable slots, both `CAtlList` fields resolved, and the element PODs rendering owning. The
  assembly is parked at `~/vibeos-rust-staging/CDefaultContextMenu-cppcom/` with a RESUME.md,
  and it re-assembles for $0 from the resume cache. Three more tool fixes fell out, two of which
  recovered already-paid bodies at $0 on replay: a free helper and a method may share a name and
  C++ resolves it by SCOPE (`MapVerbToCmdId/5` static vs `/3` member — the binder knew only the
  free one and hard-failed the whole TU); `--seal-calls` hid sibling call sites from the pass
  that inserts the implicit `this`, so the envelope demanded a receiver the responsible pass
  could not see (5 of the 7 rejects); and cpp_com's dropped-constant audit demanded back a
  `const_cast<T>` type argument that `fact_seal` already knew has no Rust landing site. The two
  bodies the workers could not produce are now hand-written in the parked file:
  `DoCreateNewFolder`, because every sample silently FIXED a real ReactOS leak (the C frees
  `pidl` once and leaks it when `SelectItem` fails), which the envelope is right to refuse and
  which retrying cannot change; and `InvokeRegVerb`, whose three missing constructs now exist —
  **`CRegKey` ported in `atl.rs` and `CScopedSetObjectWithSite` in `shellutils.rs` (LANDED)**,
  and the `static_cast<IContextMenu*>(this)` is the file's own `QueryInterface` answer,
  `addr_of_mut!((*this).IContextMenu3_iface).cast()`. That is the same `--local-raii` substrate
  that blocks `COpenWithMenu`, so that lane is unblocked too.
  **Second session (2026-08-05): the free-function half is bought as well — $0.13, 22/22, so
  $0.54 for the whole TU.** `--cpp-com --class` translates the CLASS ONLY; this TU's other 14
  top-level definitions (the `DCMA_*` menu-array API, both `*_RustBridge` entry points,
  `SHCreateDefaultContextMenu`, `CDefFolderMenu_Create2`, the `g_StaticInvokeCmdMap` verb
  table) needed a separate `--free-functions` run — the same two-run shape `CNetFolder` used.
  Both runs derive their output path from the TU, so the second **requires `-o`**: rustconv
  refuses to write rather than discard the first, which is fail-loud working as designed.
  **FLIPPED (2026-08-05, third session): 201 errors -> 0 on all three arches, `.cpp` out of
  CMake.** The residue was the three-way substrate conflict this crate is known for
  (`crate::shellcom` vs the bindgen island vs `com_sys`/`ntdll_sys`), and it resolved as three
  mechanical classes plus a hand tail. `IContextMenu` is declared ONCE, by the class half, with
  every slot typed, and both the TU half and the support module import that one — which is what
  keeps `DCMENTRY.pCM` and `DynamicShellEntry.pCM` the same nominal type; the TU half's
  generated duplicates of `DynamicShellEntry`/`StaticShellEntry` were dead AND typed `pCM` as a
  raw pointer where the C++ has `CComPtr<IContextMenu>`, so they were deleted rather than
  reconciled. `IShellExtInit` and `IShellFolderView` are declared file-locally in the
  hand-written support module, as shellcom.rs's own header directs. `DCMENTRY` is re-declared
  there too, for one reason: the island types its `pCM` as the bindgen C++ class with an opaque
  `vtable_`, which no call site can go through.
  Defects worth naming beyond the substrate: `_InsertMenuItemW` lost the `LOWORD(dwTypeData)`
  argument to `LoadStringW` and shifted the rest left (a `/* i32 */` placeholder, so it failed
  loudly); `AddStaticEntry` rendered C's `!_wcsicmp(...)` as Rust's BITWISE `!`, which would
  have inverted the "open verb goes first" rule; `QCMINFO`'s 5th field was named `NULL` after
  its initializer; the ctor transposed `m_iIdSCMFirst(0)` and `*m_DefVerbs = UNICODE_NULL`;
  `StrIsNullOrEmpty` is C++-OVERLOADED and the wide overload was chosen for the ANSI
  `lpParameters`; and both `*_RustBridge` wrappers came back as `#[no_mangle]` GENERICS
  returning `CY` (tagCY, the CURRENCY union) instead of `UINT`/`HRESULT` — CFSDropTarget imports
  those two by name, so they had to be right.
  The flip itself needed only two linkage changes: `MapFcidmCmdToVerb` and `MapVerbToDfmCmd`
  are declared with C++ linkage in precomp.h and still called from the C++ `CDefView.cpp` and
  `utils.cpp`, so both declarations became `EXTERN_C`. Everything else already matched —
  `SHCreateDefaultContextMenu` and `CDefFolderMenu_Create2` are `extern "C" WINAPI` in
  shlobj.h, and `_InsertMenuItemW` sits inside wine/shell32_main.h's `extern "C"` block.
  Three PHANTOM symbols only the link could find, all macros: `SID_IShellBrowser` and
  `SID_SFolderView` are `#define`s ALIASING `IID_IShellBrowser`/`IID_IFolderView`, and
  `DSA_GetItemCount(hdsa)` is `(*(int *)(hdsa))`. `cargo check` is green with all three
  declared `extern`; only the linker knows they are not symbols.
  Gap (2), inherited layout, blocks
  `CFolder`/`CFolderItems` (`IDispatchImpl<>`), `CStubWindow32`/`CDefView` (`CWindowImpl<>`)
  and `CShell32Module` (`CAtlModuleT<>`). The rest are assorted activation shapes
  (`CFileSysBindData`, `CExtractIcon`, `COpenWithList`/`COpenWithDialog`, `CRegFolderInfo`,
  `CChangeNotifyBroker`).
  **Three more whole-TU flips landed 2026-08-05** — `droptargets/CFSDropTarget.cpp`,
  `droptargets/CRecyclerDropTarget.cpp` and `CDefViewBckgrndMenu.cpp` — and between them they
  settled the per-module type-ownership question that had CFSDropTarget parked
  `ORPHAN-BY-DESIGN`. The answer is three-way and none of it is optional:
  **`shellcom` owns the COM vtable ABI** (imported explicitly, which outranks the prelude's
  island glob, and the GUID family has to move WITH it — taking some and not the rest merely
  relocates the mismatch from the vtable slots to the `IID_*` statics);
  **the island owns data layouts**, so every PIDL / POINT / STRRET / GUID crossing INTO a
  shellcom slot takes a `.cast()` at the call site (shellcom.rs:91 already says why it keeps
  layout-identical copies and that the cast is the price);
  **the file owns whatever neither has** — e.g. `IShellFolderView`, whose island form is
  bindgen's opaque `vtable_: *const c_void` with no callable slot, and `_DoDropData`, whose
  island form names the island's ATL-shaped `CFSDropTarget` and would read `data->This` at the
  wrong offsets on the receiving thread.
  A fourth rule fell out of CDefViewBckgrndMenu and is worth knowing before the next flip: the
  island renders IDropTarget's vtable slots as `Option<fn>` where shellcom uses raw fn pointers,
  so an island-typed interface pointer cannot be CALLED at all (E0618, not a mismatch).
  Two shellcom vtables grew trailing slots to make this work, and both are additive for the same
  reason — a `usize` slot is the same SIZE as a fn pointer and cannot be called, so typing one
  changes the vtable's type and not its layout, and nothing in the tree builds either as a
  literal: `IDataObject` got QueryInterface/AddRef/Release/QueryGetData typed, and
  `IFolderViewVtbl`, which stopped dead at `GetFolder`, got the remaining eleven from
  shobjidl.idl:866.
  **shell32 has NO orphan modules left (2026-09-20).** `utils.cpp` was cohab-flipped, which
  was the last one. What the flip actually needed was far smaller than the guard on that module
  predicted: measured against the LIVE CMake source list plus libshell32_rust.a, 36 symbols
  leave the TU and utils.rs publishes 32. The guard said all 16 `*_RustBridge` wrappers needed
  rehoming; 14 were already published by utils.rs itself, and only the CStubWindow32 pair moved
  (to `rust_stubwindow32_stub.cpp`) because only that pair reaches a C++ ATL class propsheet.cpp
  and CDrivesFolder.cpp still construct. Two mangled helpers took EXTERN_C in utils.h instead.
  Conformance: launched=91 ok=91 on all three arches, and the `[EXPLORER] taskbar: OnCreate
  begin` -> `child controls created` gap it used to freeze in is traversed.
  **The whole `recyclebin` STATIC LIBRARY is Rust (2026-09-20)** — the first whole-library
  swap rather than a per-TU cohab. Its six sources (`recyclebin.c`, `guid.c`,
  `recyclebin_generic{,_enumerator}.cpp`, `recyclebin_v5{,_enumerator}.cpp`, 2,083 lines)
  WERE the entire library, so the `recyclebin` target goes away and shell32 links
  `${RECYCLEBIN_RUST_LIB}` instead; `add_subdirectory(shellrecyclebin)` is commented out and
  that CMakeLists is now a tombstone recording the per-file mapping.
  Cost: **$0** — the translations had been bought earlier and were sitting in
  `recyclebin-rust` unlinked, which is the thing worth checking for before buying anything:
  the crate compiled and was in `workspace.members`, but nothing referenced it, so it had
  never been linked and its exports had never been proven against a real link.
  Capture set at the swap, measured against the LIVE consumers (shell32's objects plus
  libshell32_rust.a): **7 symbols, all published** — `CloseRecycleBinHandle`,
  `DeleteFileToRecycleBinW`, `EmptyRecycleBinW`, `EnumerateRecycleBinW`,
  `GetDefaultRecycleBin`, `GetRecycleBinPathFromDriveNumber`, `RemoveFromRecycleBinDatabase`.
  The rest of the 138 symbols the C++ library defined are C++-mangled class methods and ATL
  guard variables that never left it — which is why the raw "138 defined vs 75 published"
  gap is NOT the number that matters.
  Verified: builds and links on all three arches; conformance kmtest amd64
  launched=91 ok=91 bad=0, ratchet holds.

  **CDirectoryList + CDirectoryWatcher are a CIRCULAR pair — translated, cannot flip.**
  `shelldesktop/CDirectoryList.cpp` (both its plain classes, CDirectoryList and
  CDirectoryItem) IS translated and wired at
  `shell32-rust/src/shelldesktop/CDirectoryList.rs` + `CDirectoryItem.rs`, compiling on all
  three arches. It stays in CMake anyway, and the cycle is why:
  `CDirectoryWatcher` holds `CDirectoryList m_dir_list` BY VALUE (CDirectoryWatcher.h:35) and
  calls its methods through CDirectoryList.h's inline constructor, so flipping CDirectoryList
  alone leaves four C++-MANGLED undefined symbols — and a bridge cannot stand in for a
  by-value member, which needs the real class layout. Flipping the consumer first does not
  work either: `CDirectoryWatcher` fails preflight on exactly that member
  (`state field type is not proven C scalar/pointer state: CDirectoryList m_dir_list`).
  **Breaking it needs the deterministic layer to model a translated SIBLING CLASS as
  by-value state.** Wiring the Rust without flipping is safe here and is NOT a partial cohab
  flip: the Rust publishes unmangled `#[no_mangle]` names while the live C++ defines mangled
  ones, so there is nothing for the linker to silently choose between.
  Note for whoever flips it: `CSimpleArray<CDirectoryItem>` in C++ frees each element through
  `~CDirectoryItem` (`free(m_pszPath)`); confirm the Rust side frees it too before trusting
  the flip, because the module is dead code today and no test would notice a leak.

  **CFolderItemVerb / CFolderItemVerbs are still C++ — OWED, not done, and one hand-written
  bridge stands in for them.** `CFolder.cpp` and `CFolderItems.cpp` are Rust (CFolder 24 fns,
  CFolderItem 25, CFolderItems 13); `CFolderItemVerbs.cpp` is untranslated original C++,
  blocked on TWO named rustconv gaps, both deterministic and free to close: (1) its activation
  is `new CComObject<T>`, and the only internal shapes the layer maps are `ShellObjectCreator`,
  `ShellObjectCreatorInit` and `_CreatorClass::CreateInstance`; (2) `CFolderItemVerb` carries
  `CComBSTR m_name`, and `CComBSTR` has ZERO mentions anywhere in rustconv — it is unmodelled,
  unlike `CComPtr`. Because `CFolder`'s `CFolderItem::Verbs` is Rust and still has to build
  that C++ object, `rust_folderitemverbs_bridge.cpp` publishes the four symbols rustconv kept
  symbolic: `CFolderItemVerbs_new` / `_delete` / `_Init_body` / `_AddRef_body`. **That file is
  scheduled to die**: the names are exactly the ones the Rust already calls, so when the two
  classes translate, Rust defines them, the bridge is deleted, and `CFolderItem.rs` does not
  change. It is ABI-sensitive in one specific way worth knowing before touching it — every
  entry point traffics in `FolderItemVerbs*`, not `CFolderItemVerbs*`, because the C++ it
  replaces ended in an implicit upcast that adjusts the pointer by a non-zero offset.
  **A stale ORPHAN banner is not cosmetic**: `scan_extern_arity`, `scan_cc_mismatch`,
  `scan_silent_failures` and `verify_lost_exports` all SKIP a file on sight of
  `ORPHAN-BY-DESIGN:`, so it must come off in the same commit that wires the module or four
  checks go quiet on it.
  The paragraph below is kept for the CDefView half of the story and for the measurement
  recipe, which grew a third clause: an intra-DLL undefined-symbol sweep does not see `.spec`
  exports, because those are consumed by other modules entirely.

  **`CDefView.cpp` is COHAB-FLIPPED (2026-09-19); `utils.rs` was the last orphan and is now
  flipped too.**
  `shell32/CDefView.rs` (11.9K lines) is wired and owns the shipping implementation. What
  unblocked it was a measurement: the TU is a link LEAF — `llvm-nm --undefined-only` over every
  object in shell32.dll (shell32.dir, dialogs/, folders/, shelldesktop, shellmenu, recyclebin)
  references NOTHING defined in `CDefView.cpp.obj`, so the post-flip capture set is empty. Its
  only outward surface is the spec export `SHCreateShellFolderView` (ordinal 256).
  An empty capture set was necessary and NOT sufficient: the flip then needed three real
  translation bugs fixed before the desktop worked — an inverted `m_hWnd` truthiness guard, a
  bare `QueryInterface` given `m_pSFParent` as receiver instead of `this`, and a dropped ATL
  `GetWindowProc` override in `crate::atlwin` (whose window path had never executed in a
  shipping build, CDefView being its only user). Details in the field manual under "Semantic
  rewrites that flip meaning". Verified `launched=91 ok=91` on amd64 (3/3) and i386.
  `shell32/utils.rs` (95 free functions) is at zero rustc errors on i686/amd64/arm64 and stays
  NOT wired; `ORPHAN-BY-DESIGN` on it is a GUARD rather than a to-do — an earlier revision of
  this entry claimed both were wired, which was the mistake, not the reporting of it.
  **Wiring a translated TU whose `.cpp` is still live is a PARTIAL COHAB FLIP.** utils.rs
  defines 58 `#[no_mangle]` names the live C++ objects also define as unmangled externs (16 of
  them the `*_RustBridge` wrappers other Rust modules call, plus `SHGetShellStyleHInstance` and
  `SHGetUserDisplayName`); the linker resolves archive members against object files without a
  duplicate-symbol error and silently reroutes those entry points into Rust that has never
  executed. Measured: wired, the guest booted and passed smoke on all three arches, then froze
  inside explorer's taskbar `OnCreate`, so `ProcessStartupItems` never ran and the kmtest sweep
  launched **0 of 91** tests. Un-wiring utils alone cleared that and exposed a second,
  independent freeze from CDefView after `shell window ready` — one blocker per wired module,
  which is what symbol capture predicts and a single bad body does not. Both un-wired:
  `launched=91 ok=88`, ratchet holds.
  So wiring and the cohab flip are ONE change, gated together. For utils.cpp the flip
  additionally needs its 16 bridges rehomed first (the `rust_*_stub.cpp` files are the pattern; the
  CStubWindow32 pair must stay C++ because that ATL class needs exactly one owner and
  CDrivesFolder.cpp still uses it). Prove the capture set is empty with the `llvm-nm` recipe in
  the field manual under "Cohab link plumbing" — `scan_duplicate_symbols` does NOT cover this
  direction. `shell32-tail-inventory` still counts 28 `.cpp` + 4 `.c` live.
  That work also **narrowed the "island owns data layouts, so every crossing takes a
  `.cast()`" rule above**: the reason a cast was needed at all was that `types.rs`/`shellcom.rs`
  kept layout-identical COPIES. VARIANT, SAFEARRAY, POINT, SIZE, RECT, MSG, POINTL,
  LARGE_INTEGER and ULARGE_INTEGER are now `pub use crate::island::…`, so those crossings need
  no cast at all and cannot drift. Two of the copies were not even layout-identical: the local
  VARIANT was a flattened four-member union missing `pintVal` (the slot `VT_INT_PTR` uses), and
  the local SAFEARRAY stopped at `pvData` and left the flexible `rgsabound[]` implicit. A
  `.cast()` remains the right answer only where the two sides are genuinely different types —
  a `shellcom` interface pointer handed to an island function that takes the com-sys one.
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
  `dialogs/item_prop.cpp` is flipped (2026-08-02) — the filesystem property-sheet entry points,
  and the first **split-path** TU: nine free functions bought through `--free-functions`, and
  four hand-written. Its `ShellPropSheetDialog` is a C++ `struct` used as a NAMESPACE (only
  statics, no vtable, no `BEGIN_COM_MAP`, no `OBJECT_ENTRY`), so `--cpp-com` refuses it on the
  deterministic preflight while `--free-functions` skips its methods as "class methods" — a
  shape neither mode owns, and one that `--free-functions` drops SILENTLY. `--local-raii` does
  reach the free-function path, which is what kept `CDataObjectHIDA`'s drop semantics out of the
  model's hands. The flip also retired the last `SHELL_ShowItemIDListProperties_RustBridge`
  trampoline: that shim existed only to escape C++ mangling, so `EXTERN_C` on the four
  `precomp.h` declarations replaced it with a direct call.
  Alongside it, `CRegKeyHandleArray` moved to a single definition in `shell32-rust/src/shfldr.rs`.
  It had been re-derived independently in `cfs_folder.rs` and `desktop_folder.rs` and BOTH copies
  had lost the destructor (the HKEY leak fixed on 2026-08-02); `item_prop` would have been the
  third. The rule that fell out: a re-declared `extern` fn is cheap to repeat and this crate does
  it 530 times, but a `Drop` gets exactly one home — a missing one is a leak no build can see.
  `changenotify.cpp` is flipped (2026-08-02) — the CLIENT half of shell change notification
  (`SHChangeNotify`, `SHChangeNotifyRegister`, the shared-memory delivery ticket and handbag);
  the SERVER half in `shelldesktop/CChangeNotifyServer.cpp` is untouched C++. 22 of its 24
  functions bought, 2 hand-written. Its `CChangeNotifyBroker` is an ATL **window** class, not a
  COM one, so no C++ mode applies — it and `CreateNotificationBroker` stay in
  `rust_changenotify_broker_stub.cpp` (second entry in `sanctioned.atl_window_class`). First TU
  in the wave needing **no `EXTERN_C` lockstep**: every declared name was already inside an
  `extern "C"` block and the rest have no header declaration. The two hand-written functions are
  the interesting part — the seal envelope for each was *unsatisfiable*, and fixing that closed
  three sealer defects (preprocessor directives, array-parameter bounds, and `-E -dM` not
  reporting a macro the TU `#undef`s). Field manual: "A DIRECTIVE is not code, and an array
  PARAMETER's bound is not a value".
- **Vendored, by policy (NOT targets):** enumerated in **`vendor.yaml`** at the repo root —
  **25 trees, ~2.07M lines**, gated by `tools/vendored.py` (a stage of `vibeos-cli check`).
  Run `python3 tools/vendored.py --emit` for the table; do not restate it here, and do not
  hand-maintain a second list anywhere. This entry used to read "ACPICA (~155 `.c`),
  FreeType (~208K LOC)" and was one of three disagreeing lists — the others being
  `external/README-fex-vendor.md` (one tree) and a hardcoded 6-tuple in
  `verify_conv_macros.py` that covered 5 of the 19 directories under
  `sdk/include/reactos/libs/` and carried one entry matching nothing at all.

  Two properties worth knowing. **The list cannot be a glob:**
  `sdk/include/reactos/libs/` holds `zlib` and `mbedtls` beside `audiosrv`, `samsrv`,
  `umpnpmgr`, `syssetup`, `fslib`, `fmifs`, `epsapi`, `cportlib` and `fast486`, all of
  which are ours, and no path convention anywhere in the tree separates foreign code from
  ours (ACPICA under `drivers/`, FreeType under `sdk/lib/3rdparty/`, FEX under `external/`,
  ucrt under `sdk/lib/`). **Versions are probes, not copies:** each entry records the file
  and regex where its version lives and the tool derives it, so a re-vendor cannot leave a
  stale literal behind. `--check` fails on a probe that stops matching, on a stale
  `VENDORED` stamp, and — the ratchet — on any unregistered directory appearing inside a
  vendor container.

  **`sdk/lib/ucrt` (78K lines, 459 TUs) is registered**, which is what makes ucrtbase's
  goal sayable: ucrtbase.dll is vendored Microsoft ucrt plus **our** `msvcrt_shared`
  (5.7K lines — `cpp.c`, `except.c`, `undname.c`, `except_<arch>.c`,
  `reactos/ucrt_tls_sup.c`). Counting ucrt as debt makes the DLL read as permanently ~8%
  reachable; excluding it makes the target "everything in ucrtbase that is ours is Rust"
  — 0% today, 100% after task #55. Four entries carry a `review:` note flagging a
  classification that is NOT settled (ucrt's unknown SDK version; `cardlib`'s unestablished
  provenance; `libwine` and `strmbase`, which are Wine-derived and therefore arguably
  translation TARGETS under the roadmap's Tier 3 — they sit in the registry only because
  upstream files them under `3rdparty`). `--emit` prints those under NEEDS REVIEW.

  **The registry has a second half: `ours:`** (2026-08-08, task #62). `--suspects` found
  ~400K lines of foreign-copyright code in no list at all, and half of it turned out to be
  ours — but a verdict of "ours" had nowhere to live, so it would have stayed a commit
  message and been re-derived by the next reader. Both verdicts are now recorded, which is
  what makes the scan a **ratchet**: `--check` fails on a foreign-copyright directory that
  is in NEITHER half, so newly-imported foreign code must be classified before it lands.
  The six calls that cleared all 30 hits:

  - **WDF → vendored** (290K lines / 488 files, the second-largest foreign tree here after
    FEX). Vendored for the ACPICA reason, not the "it is Microsoft's" reason: third-party
    KMDF/UMDF driver binaries bind to this framework's DDI table by version, so the tree
    has to *be* the framework Windows ships, and upstream is public. ReactOS carries it
    near-unmodified (9 of 488 files mention ReactOS). Two facts that do NOT decide it, so
    nobody re-litigates on them: it is 81% C++, which is a scheduling obstacle rather than
    an ownership argument, and it is built, which is equally true of FreeType.
  - **Microsoft compiler-tests (EH/SEH) → vendored**, for a reason no other entry has:
    they are an **oracle**, and an oracle's worth is entirely that someone else wrote it.
  - **classpnp / cdrom / disk → OURS**, +90.5K lines of bodies to the denominator. The
    contrast with WDF is what the call turns on: these are drivers this OS runs, with no
    external ABI contract, shipped under MS-PL (which grants derivative works), and
    already diverged — 13 of classpnp's 24 files carry ReactOS edits.
  - **`sdk/include/psdk` → ours**, and not a target either way: 5 of its ~700 headers are
    verbatim SDK headers, and the directory contains no C bodies at all.

## Orphan audit, 2026-09-20 — 18 items sorted, ZERO rescuable

The `research` lane's half of the fleet orphan sweep, re-derived from evidence
rather than inherited. Complements the `ORPHAN-BY-DESIGN` inventory; where the
two disagree this one carries the re-derivation.

**The headline is that nothing was abandoned-but-valuable.** Of 18 items — 4
orphan modules, 6 candidate orphan crates, 8 inherited "upstream-dead" — not one
came back RESCUABLE. The bank is cleaner than the survey feared.

| item | verdict | basis |
|---|---|---|
| `classpnp/class_gap3.rs`, `class_gap4.rs` | SUPERSEDED — **trashed** | all 11 exports live in `class_part_d.rs` |
| `ntoskrnl/global_barrier.rs` | not a translation — **trashed** | `gen_global_barrier.py` output; technique `nm`-verified dead, root cause toolchain-fixed on LLVM ≥ 21 |
| `explorer/appbar.rs` | SUPERSEDED — **trashed** | ~16 of 29 exports have an `ab_*` counterpart in `traywnd.rs`; the other 13 do not, so trashed on cost not certainty (see below) |
| `rtlk-rust/path.rs` | SUPERSEDED — **trashed** | all 31 exports live in `rtl-strings/src/cabi/mod.rs`; overturns its "upstream-dead" prior |
| `afd/event.rs`, `bootvid/vga.rs`, `bootvid/bootdata.rs`, `dhcpcsvc/tree.rs`, `freeldr/directui.rs`, `win32k/bitblt_new.rs`, `win32k/drvdbg.rs` | DEAD-UPSTREAM — keep | confirmed against the C, not the banner |
| 6 candidate crates | **5 SHIP**, `slist-loom` is scaffolding | see `tools/survey_unlinked_crates.py` |

**Three findings worth more than the sort itself.**

1. **`bootvid/vga.rs` and `bitblt_new.rs` must NEVER be wired.** They export the
   same symbols as the live `bootvid.rs` / `bitblt.rs`+`copybits.rs`. These are
   mutually-exclusive platform backends (INT10h BIOS vs GOP framebuffer;
   `USE_DIBLIB` on vs off) implementing one spec surface. Wiring one is a
   duplicate-symbol LINK FAILURE, not a feature — unwired is the only legal
   state, and "orphan" undersells it.
2. **`dhcpcsvc/dhcp/tree.rs` is genuinely unrecoverable**, and it is the only
   one. It references `struct tree`, `struct dns_host_entry` and
   `do_host_lookup`, none of which exist anywhere in the tree — an unfinished
   ISC import that cannot compile even standalone. Reviving it is a new
   translation, not a rescue.
3. **A banner was factually WRONG, and banners are this audit's whole evidence
   base.** `win32k-rust/lib.rs` claimed enabling `drvdbg` requires "defining
   `DbgLookupDHPDEV`"; it is defined at `pdevobj.rs:182` and `drvdbg.rs` calls
   it nine times. The C only forward-declares it (`drvdbg.c:14`) and defines it
   nowhere, so that Rust body has **no C original** and is unverified against
   any oracle. Corrected in place. **An orphan with a wrong banner is worse than
   one with none**, because it is sorted confidently and wrongly — which is the
   argument for making a banner mandatory when a module is unwired.

**The last two were trashed on a COST argument, not a certainty one, and the
argument was checked.** Neither was SUPERSEDED-certain — `appbar.rs` had 13 of
29 exports with no named counterpart, and `rtlk/path.rs` overturned a prior
classification rather than confirming it. What makes being wrong survivable is
that both C originals are still on disk (`base/shell/explorer/appbar.cpp`, 612
lines; `sdk/lib/rtl/path.c`, 3,052), so rustconv can retranslate either for
roughly a dollar. **Retranslatable output is cheap to be wrong about; output
whose C is gone is not** — which is exactly why `dhcpcsvc/tree.rs` stays,
despite also shipping nothing: its types exist nowhere, so there is no redo.
Verify the source still exists before reusing this reasoning.

Trashed files are moved to `tools/rustconv/.responses/trash/`, which is
GITIGNORED and therefore a local convenience only. **Git history is the
preservation mechanism**: `git log --diff-filter=D -- <path>` finds the removal
and `git show <commit>^:<path>` prints the content back.

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

## Paid Rust that does not ship — the ORPHAN-BY-DESIGN inventory

Surveyed fleet-wide 2026-09-19 by the shell32 lane. **17 modules across 9 crates hold
translated Rust that no build compiles.** Recorded here because an inventory that lives only
in a chat log is the thing this file exists to prevent; nothing below is a task list.

**Sixteen of the seventeen are dead on purpose.** Only `shell32/utils.rs` is unfinished work.
The distinction is not visible from "is it wired?" — it is written in each file's banner, and
the banner has to be read.

| category | modules | what it means |
|---|---|---|
| **Genuinely blocked (0)** | ~~`shell32-rust/src/shell32/utils.rs`~~ — **COHAB-FLIPPED 2026-09-20** (`fb07f381d99a`), `launched=91 ok=91` on all three arches | It was the only one of the seventeen that was unfinished work, and it is done. Kept in the table because the *reason* it was stuck is the reusable part: 58 live C++ symbols would have been captured, and 16 `*_RustBridge` wrappers plus the `CStubWindow32` pair had to be rehomed first. See the shell32 entry above. |
| **Superseded / gap artifacts (8)** | `recyclebin-rust`: `recyclebin_generic{,_enumerator}.rs`, `recyclebin_v5{,_enumerator}.rs` · `classpnp-rust`: `class_gap3.rs`, `class_gap4.rs` · `explorer-rust`: `appbar.rs` · `ntoskrnl-rust`: `global_barrier.rs` (**generated**, see below) | Dead weight, superseded by class-atomic modules, rustconv file-split gaps whose exports are all defined live elsewhere, or retired code. **Deletion candidates — owner's call, since they are paid artifacts; the bodies replay from `.responses/` for $0.** |
| **Translations of upstream-dead source (8)** | `afd-rust` `event.rs` · `bootvid-rust` `vga.rs`, `bootdata.rs` (legacy INT10h) · `dhcpcsvc-rust` `dhcp/tree.rs` · `freeldr-rust` `directui.rs` (`#if 0` upstream) · `win32k-rust` `bitblt_new.rs` (`USE_DIBLIB FALSE`), `drvdbg.rs` (in no CMake list) · `rtlk-rust` `path.rs` | Correctly orphan and kept deliberately under the never-delete rule. Nothing owed. |

**An orphan ROTS, in every category, because nothing compiles it.** `recyclebin-rust` carried a
`Cargo.toml` note reading "PARKED at 25 errors/arch (from 256)"; when the lane picked it up it
was at **204**. Drift with no diagnostic, because a crate outside `workspace.members` is never
built. CDefView reached **799** the same way before it was flipped. The fix is membership, not
vigilance: `recyclebin-rust` is now at 0 on i686/amd64/arm64 and has joined the workspace
(`1fc79782b4f4`), which puts it in every lane's `cargo check`. **Nothing links it, so no boot
can regress** — that asymmetry is what makes joining-on-green safe and is the general rule for
a parked crate.

**`global_barrier.rs` is not a translation** — `tools/gen_global_barrier.py` generates it. The
never-delete-paid-artifact rule does not attach to it the way it does to the other seven, and
it is trivially regenerable; it is `nm`-verified dead-stripped and its root cause is fixed in
the toolchain on LLVM ≥ 21. Worth stating because "orphan `.rs` in a crate" looks identical on
disk whether it cost money or not, and the deletion rule turns on which.

**AN ORPHAN ALSO ROTS BECAUSE THINGS DELIBERATELY DO NOT LOOK AT IT.** Four source checks —
`scan_extern_arity`, `scan_cc_mismatch`, `scan_silent_failures` and `verify_lost_exports` —
skip a file on sight of the `ORPHAN-BY-DESIGN:` token. That is correct while the file is
orphan and wrong the instant it is wired, so **the banner must come off in the SAME commit
that wires the module**, or four scanners go quiet on live code (4,949 lines of it, in the
`utils.rs` case). `scan_doc_sections` catches the stale banner, but only once the file is
reachable. So the rot has two independent sources: nothing compiles it, and several things
refuse to read it.

**The crate-level version of this question is closed, and smaller than it looked.** A
reachability closure over both workspaces (315 packages) found **zero phantom members**,
exactly **one** genuinely parked crate — `recyclebin-rust`, already rejoined — and **zero**
member-but-unlinked crates. An earlier pass reported 13, then 6, then 1: each cut came from
modelling a build mechanism the tool had missed, first difftest's directory-suffix discovery
and then `reactos-link`'s `build.rs` linker-flag path, which links a crate with no Cargo
dependency edge and no CMake string. The general lesson is worth more than the number: **a
build mechanism the tool does not model reads as absence of a build**, so "unreached" is a
claim about the tool's coverage at least as much as about the crate. `slist-loom` is the one
remaining unlinked member and is deliberate — a Loom concurrency model run as
`cargo test -p slist-loom`.

One question is parked for the owner, and it is now the only one: whether the eight
superseded/gap files should be deleted. The join-on-green survey that used to sit beside it is
answered — the surface was one crate, and it is closed.

## Scale

~3.58M LOC of translated Rust across kernel + drivers + usermode chain, + ~1.43M LOC of
generated FFI (`-sys` bindgen: ntkrnl-sys 444K, win32k-sys 379K, winsrv-sys 192K, ntdll-sys
128K, … — declarations, not translation). The two *hot* monoliths dwarf everything for
edit-churn: **win32k 257K** and **the kernel** (see crate-split note). The driver wave is
big but cold: cdrom 211K, disk 170K, classpnp 166K, kernel32 161K, buslogic 138K, uniata
125K, scsiport/atapi/storahci ~120K each, fastfat 113K, freeldr 75K, cdfs 65K, storport 65K,
+ ~130 smaller crates.

## Shared substrate crates (ported once, consumed by many islands)

Some header material is invisible to bindgen and therefore gets re-ported by hand into every
module that needs it. Three crates now own that material, and all three exist because the hand
copies had measurably drifted — or, for `va-slots-rust`, because they were about to:

| Crate | What it owns | Why it exists |
|---|---|---|
| `strsafe-rust` | `strsafe.h`'s `static __inline` bounded string ops | `STRSAFEAPI` is `static __inline`, so **no object file in the tree defines any of them** — the last `.c` a module flips takes the module's only definition with it. Four copies existed (msgina, syssetup, desk, sysdm), no two the same subset, and msgina's `StringCchPrintfW` had DROPPED the caller's bound and forwarded to `wsprintfW` — a stack overrun on a `WCHAR[256]`. Consumers: desk, syssetup, sysdm, msgina, rosload, uefildr. |
| `va-slots-rust` | Receiving C varargs on stable Rust: the slot-capture helpers (`format_slots`, `format_slots_z`) and the measured `VA_SLOTS`/`VA_STAGE` bounds | Stable Rust cannot **define** a C-variadic, so every printf-family entry point in the tree took a fixed parameter list and printed its format string LITERALLY, dropping every argument. It can **receive** one: all three targets pass varargs as a flat ascending slot sequence and `va_list` is `*mut c_char` on all three, so the trailing slots become ordinary fixed parameters replayed as the va_list, and no call site changes. Extracted at the THIRD copy rather than the fourth: the bounds are only safe **because they were measured** (widest live format = 11 slots, counted in slots not specifiers), so each copy is a fresh chance to guess them. Exports no `#[no_mangle]` symbol, which is what lets `drvtrace-shim` keep the LLVM-module isolation it exists for (§9.99). Consumers: freeldr-rust, kd-rust, drvtrace-shim. |
| `win32macros-rust` | The function-like Win32 macros (`LOWORD`/`HIWORD`/`MAKELONG`/`MAKELANGID`/`MAKELCID`/`MAKEINTRESOURCEW`/`RGB`/`Get?Value`/`GET_?_LPARAM`/`ZeroMemory`/`_countof`/`ARRAYSIZE`) plus the constants bindgen folds away (`DWLP_MSGRESULT`/`DWLP_DLGPROC`/`DWLP_USER`, `INVALID_HANDLE_VALUE`, `UNICODE_NULL`) | bindgen emits **no** function-like macro and drops cast/`sizeof`/pointer-literal constants. Before extraction: `LOWORD` defined **32× across 12 crates in 8 spellings**, `HIWORD` in 8, `ZeroMemory` in 6, `DWLP_USER` in 9 — and all four `RGB` copies had dropped `wingdi.h`'s per-component `BYTE` mask, so an out-of-range component bled into the next channel. Transcribed from the headers, not from the copies, with 13 known-answer tests. Consumers: desk, sysdm, intl, syssetup, aclui, netcfgx, netid, umpnpmgr, samlib, ntmarta, authz, msv1_0 (+ recyclebin, parked). |

**The sharing boundary is narrower than "core primitives."** A shared crate cannot depend on any
consumer's bindgen island, so everything it exports must be a fixed-width primitive that is the
*same on all three targets*. Two Win32 types look primitive and are not: `HWND` is `*mut HWND__`,
a nominal opaque struct each island emits for itself; and `LPARAM`/`WPARAM`/`LRESULT` alias
`LONG_PTR`/`UINT_PTR`, which bindgen emits as `c_long`/`c_uint` on i386 but
`c_longlong`/`c_ulonglong` on the 64-bit pair — never `isize`/`usize`. Anything pointer-width or
nominal stays in the consumer prelude. Details and the full stay-local list:
`translation-debugging-field-manual.md` → "A header MACRO re-ported per module drifts".

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
