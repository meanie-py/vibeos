# Userland Translation Audit — Part 2 (native residue)

**Sequel to `userland-translation-audit.md`.** Part 1 deep-audited the 10 highest-value
ReactOS-native client DLLs (gdi32, advapi32, ws2_32, …). This Part 2 maps the
**rest of the native userland residue** — every ReactOS-authored, C-only,
not-yet-audited usermode module, excluding the Wine tier (see
`userland-wine-libs-audit.md`), the keyboard/NLS data DLLs, vendored 3rd-party, and
kernel drivers.

**Last updated:** 2026-06-21. Built from a 9-agent family fan-out (full mini-scope
for ≥5K modules, one-liner for the tail).

**Translation: ~3% of this native-residue surface (by LOC).** Methodology: of the
~295K C LOC mapped here, the landed work is **msgina** (~5.5K, done →
`msgina-rust`, 14 .rs / ~7.3K) + **lsasrv** M0–M2 (~1.9K of 14.9K Rust →
`lsasrv-rust`, 9 .rs / ~4.3K); ≈7.4K of ~295K ≈ **~2–3%**. **samsrv is still 0%**
(no `samsrv-rust` crate yet — lsasrv is the active head of the LSA/SAM lane).
Everything else here is unstarted; the bulk (~117 modules) is non-boot UI/applet/COM
code with no smoke payoff.

## How the `.spec` surface decomposes (the residue, settled)

`.spec` files ≈ exported DLLs (~467 deduped dirs) — NOT 1:1 with all modules (EXEs,
`.sys` drivers, static libs use `.def`/nothing). The ~467 exported DLLs split into:

| Pile | Modules | LOC | Status |
|---|---:|---:|---|
| Already Rust (spine) | ~10–14 | — | done |
| **WINE** | 185 | ~1.43M | audited → mostly skip (`userland-wine-libs-audit.md`) |
| **DATA** (keyboard layouts + NLS) | 88 | 40K | not translation targets (tables) |
| **NATIVE, audited (Part 1)** | 10 | ~160K | ranked (`userland-translation-audit.md`) |
| **NATIVE residue (THIS doc)** | **124** | **~295K** | scoped below |
| vendored 3rd-party (mbedtls/libjpeg/…) | 6 | ~236K | never-translate |

So the **whole worth-translating native userland that remains is ~124 modules /
~295K LOC** — and most of it is Low priority (non-boot UI/applets). The actionable
high-value set is small.

## Translate-priority ranking (the actionable output)

### 🟢 HIGH — do these
| Module | LOC | Why |
|---|---:|---|
| **lsasrv** | 15K | 🔄 **IN FLIGHT — M0–M2 landed** (dssetup/privileges/registry/security/utils → `lsasrv-rust`, ~5/16 units, ~13%). The LSA *server* logic behind the already-Rust `lsass.exe` (which links it as a C importlib). Boot-critical. **`lsalib-rust`/`lsass-sys` gave the head start** on LPC + types. Remaining: database/policy/lookup/lsarpc/srm. |
| **samsrv** | 16.5K | ⬜ **0% — not started** (no `samsrv-rust` yet). The SAM *server* (account DB) — the other half of the lsass spine. Boot-critical. Sequence **after lsasrv** (reuse its SD/SID + RPC-glue patterns; the 9.8K `samrpc.c` is the cost driver). |
| **msgina** | 5.5K | ✅ **DONE** (`msgina-rust`, cohab flip landed). The logon UI (GINA) — smoke-visible / boot-relevant. |

These three are the genuine prize: they finish the **boot/logon spine** (smss→wininit→lsass→lsasrv/samsrv; winlogon→msgina→LsaLogonUser), and two of them ride an already-Rust lane.

### 🟡 MEDIUM — good self-contained candidates, no boot urgency
- **apphelp** (5.9K) — app-compat shim engine, pure-data (no COM/UI), good algorithmic rustconv fit.
- **ntmarta** (in win32_misc) — the SD/ACL provider behind advapi32 `GetNamedSecurityInfo`/`SetEntriesInAcl`; compact single-file, clean rustconv job + good ACL-idiom practice.
- **hid.dll** (win32_misc) — HidD_*/HidP_* wrappers; **coordinate with existing `hidparse-rust`/`hidparser-rust`/`hid-sys`** (the HidP parsing half may already be Rust-backed).
- **imm32** (12.9K) — IME client; fully native C but depends on the win32k IME syscall layer; sequence after user32-rust/win32k-rust IME types settle.

### 🔴 LOW / DEFER — the bulk
- **All 20 CPLs** (~50K) — control-panel applets; **zero boot/smoke payoff**, Win32-GUI-shaped (dialog procs/property sheets/registry), the *weakest* rustconv fit. Pure usermode-Rust exercise only.
- **shellext (13)** — mostly C++/ATL in-proc COM servers → **gated on a COM-vtable→Rust pattern** (same dependency as shell32); the few plain-C ones are dialog-only easy wins. None boot-critical.
- **native directx (9)** — ddraw/d3d9/dsound_new are **NOT BUILT** (superseded by the Wine variants); ksproxy is C++ COM KS-proxy. No runtime payoff.
- **printing (5)** — RPC/driver-plugin coupled (winspool/localspl); central to printing but no boot value; gate behind rpcrt4 interop.
- **most win32_misc (58)** — leaf helpers; many are forwarders/stubs, some Wine-derived, some C++/COM. Case-by-case in the chunk, but overwhelmingly Low.

### ⛔ SKIP (not application logic)
- **ucrtbase** — CRT aggregation shim (pulls ucrt/vcruntime static libs); it *is* the C runtime, not translatable app logic.
- **dnsapi** — built on the bundled 3rd-party **adns** C lib + RPC stubs; translate only the thin ReactOS glue if ever, leave adns C.
- **lpk** — Wine-derived BiDi shaping (despite landing in the native sweep).

## Bottom line

The native userland residue is **smaller and far more sharply triaged than the Wine
tier**: of ~124 modules, exactly **3 are High-value** (lsasrv, samsrv, msgina — the
logon spine, two on an already-Rust lane) and ~4 are Medium. The other ~117 are
non-boot UI/applet/COM/forwarder code with no smoke payoff — translate opportunistically
as Rust-usermode practice, not as OS-correctness work.

**Recommended sequence if a native-userland lane opens:** lsasrv → samsrv (complete
the LSA/SAM server spine, leaning on lsass-rust) → msgina (logon UI) → then the Medium
self-contained ones (apphelp/ntmarta/hid). Everything else defers behind a COM-vtable
strategy and the higher-value native client DLLs from Part 1 (gdi32/userenv/…).

> **Status:** **msgina ✅ landed** (`e69a7adaf82`). **lsasrv 🔄 M0–M2 landed**
> (`f9f54798af0` M1 substrate + privileges/utils; `00a7b6afc8f` M2 security.c;
> `af3e0ba7e99` M2 registry.c — 5/16 units / ~13% Rust in `lsasrv-rust`; remaining
> database/policy/lookup/lsarpc/srm/dssetup-stub). **samsrv ⬜ not started.** Both follow
> the detailed execution plan → **`lsa-sam-server-translation-plan.md`** (RPC-server cohab
> via the proven svcctl/`midl-rt` pattern, per-file maps, differential-fuzz strategy for the
> SID/blob math, milestones). Key finding: the MIDL-RPC-server-in-Rust cohab is **already a
> shipping pattern** here (svcctl/winreg), and samsrv has **no in-module crypto** (delegated
> to advapi32).

---

## Per-family detail

*(BIG ≥5K = full mini-scope; TAIL = 2–3 sentences. These are ReactOS-native, so
verdicts assess translate-priority + difficulty + rustconv-fit, not skip-by-default.)*


---

## sec_server

Security servers + security UI. The headline: **lsass.exe is already Rust** (`lsass-rust`/`lsass-sys`/`lsalib-rust`/`lsass-cargo`, full flip + cargo-driven link), but lsass.exe is only the thin exe body (`base/system/lsass/lsass.c`, 87 LOC → calls `LsapInitLsa()`+`ServiceInit()`). It links **lsasrv and samsrv as importlibs** — those two DLLs are still 100% C and contain the actual LSA/SAM logic. So lsasrv + samsrv are the natural next targets in the already-Rust lsass spine, and both are boot-critical (smss → wininit → lsass → lsasrv/samsrv init; winlogon → LsaLogonUser). Strong coordination angle: the lsass lane already owns the LPC/port substrate and `lsass-sys` FFI types, which lsasrv shares.

### lsasrv — dll/win32/lsasrv  🔄 M0–M2 LANDED (~13% Rust)
- **Status:** in flight — `lsasrv-rust` (9 .rs / ~4.3K LOC) has dssetup.rs, privileges, registry, security, utils cohab-flipped (5 source `.c` commented out in the cmake). ~11 C files remain active (database/policy/lookup/lsarpc/srm/authport/…).
- **LOC / files:** 14929 C, 16 files
- **Role:** Local Security Authority server DLL — LSA policy DB, privileges, account/SID lookup, logon-session mgmt, SRM (Security Reference Monitor) channel, auth-package host, the LSA RPC interface (lsarpc.idl) + dssetup. Loaded by lsass.exe.
- **Origin:** ReactOS-native (Eric Kohl et al.)
- **Lang:** C
- **Completeness:** full for the real LSA core (policy/lookup/privileges/session/srm/database); spec has many `@ stub` (DsRoler*, most `LsaI*` internal helpers, Kerberos audit hooks) that are genuinely unimplemented in ReactOS — translating preserves them as stubs.
- **Key deps / consumers:** consumer = lsass.exe (already Rust), winlogon via advapi32 LsaXxx. Imports samsrv, rpcrt4, advapi32, ntdll. **Existing Rust peer: `lsalib-rust`** already translated the LPC client/`lsa.rs` substrate and `lsass-sys` carries the `LSA_API_MSG`/port types — lsasrv reuses the same message + trusted-caller enums. Boot-critical (lsass init path).
- **Problem points:** (1) **RPC server stubs** — built from `lsa.idl`/`dssetup.idl` via `add_rpc_files(server)`; the generated `lsa_s.c`/`dssetup_s.c` MIDL glue stays C, Rust implements the server methods behind it (FFI seam, same pattern lsalib used). (2) Registry-hive backed policy/secret DB (`database.c`, `registry.c`, `policy.c`) — heavy reg I/O + serialization. (3) **lookup.c (2534 LOC)** SID↔name translation — dense SID pointer-math, well-known SID tables. (4) SD/ACL construction in `security.c`; SRM LPC loop in `srm.c`. (5) Uses `wine` + PSEH — DUMMY_PSEH except-body-must-be-dead-code rule applies.
- **Difficulty:** High
- **Verdict:** translate priority **High** — it is the missing middle of an already-Rust lsass spine; finishing it (with samsrv) makes the whole LSA logon path Rust, and `lsalib-rust`/`lsass-sys` give a big head start on types + LPC.
- **rustconv-fit:** good (per-file batchable; lookup.c/lsarpc.c are big-file → use `translate` sync per the >1000-LOC rule; RPC `_s.c` glue stays C)

### samsrv — dll/win32/samsrv
- **LOC / files:** 16512 C, 12 files
- **Role:** Security Account Manager server DLL — the SAM database (users/groups/aliases/domains) backed by the registry SAM hive, the SAM RPC interface (sam.idl), display-info + setup. Loaded/called by lsasrv (`add_importlibs(lsasrv samsrv ...)`) and via SamIConnect.
- **Origin:** ReactOS-native (Eric Kohl)
- **Lang:** C
- **Completeness:** full for the core SAM object/DB/RPC surface; spec has many `@ stub` SamI* DS/GC/foreign-domain helpers (not implemented in ReactOS) — translate-as-stub.
- **Key deps / consumers:** consumer = lsasrv (Rust target above) + advapi32; imports rpcrt4, advapi32, ntdll. **No direct existing Rust peer**, but rides the same lsass spine and shares SD/SID idioms with lsasrv's security.c. Boot-critical (account DB for logon).
- **Problem points:** (1) **samrpc.c is 9824 LOC** — one giant RPC-method file (60% of the module); SAMPR_* marshalled structs, GENERIC_MAPPING access checks, per-object security. (2) **Registry SAM-hive object model** (`database.c`, `registry.c`, `domain.c`, `user.c`, `alias.c`, `group.c`) — fixed/variable-length attribute blobs, RID allocation, lots of byte-offset packing. (3) **Password hashes / crypto** in user.c (LM/NT hash, encrypted-password blobs) — sensitive, must match C bit-for-bit. (4) **security.c (1703 LOC)** SD/ACL construction with RtlAllocateAndInitializeSid + relative-SD packing — exactly the SID/ACL pointer-math the audit warns about. (5) wine + PSEH (DUMMY_PSEH dead-except rule). (6) ABI: stdcall SamI* exports.
- **Difficulty:** Very High (largest module in the family; single 9.8K-LOC RPC file + crypto + SAM-hive packing)
- **Verdict:** translate priority **High** (but sequence after lsasrv) — required to complete the Rust logon path; the 9.8K samrpc.c is the cost driver. Do lsasrv first to establish the shared SD/SID + RPC-glue patterns, then samsrv.
- **rustconv-fit:** good for the DB/object files; **poor-ish for samrpc.c** as one unit — must be `translate` sync (batch silently stubs lazy fns at that size) and likely split-reviewed; crypto needs careful manual review.

### aclui — dll/win32/aclui
- **LOC / files:** 5661 C, 5 files (aclui.c, checklist.c, misc.c, sidcache.c, guid.c)
- **Role:** ACL Editor UI — the "Security" property-sheet page (CreateSecurityPage/EditSecurity), permission checklist control, SID→name cache. Pure shell/explorer UI component.
- **Origin:** ReactOS-native (Thomas Weidenmueller)
- **Lang:** C
- **Completeness:** full
- **Key deps / consumers:** consumers = shell32/explorer property sheets (`ISecurityInformation`); imports user32, gdi32, comctl32, ole32, oleaut32, advapi32, uxtheme. **No relation to lsass Rust work** (it's a GUI client, not a server). **Not boot-critical** (only reached when a user opens a Security tab).
- **Problem points:** (1) **COM** — implements/consumes `ISecurityInformation` (guid.c defines IIDs); COM vtables in Rust are awkward. (2) Win32 dialog/property-sheet message loops + a custom owner-drawn checklist control (checklist.c) — GDI + window-proc heavy. (3) UXTHEME drawing. Pointer-math is light; the difficulty is GUI/COM plumbing, not security logic.
- **Difficulty:** Medium (GUI/COM boilerplate, but no DB/crypto/RPC)
- **Verdict:** translate priority **Low** — not boot-critical, no synergy with the lsass spine, and COM+property-sheet UI is low-value/high-friction for Rust. Defer behind the servers.
- **rustconv-fit:** poor (COM vtables + Win32 window procs translate badly; manual review heavy)

TAIL

### ntmarta — dll/win32/ntmarta (1797 LOC)
The "Marta" provider behind advapi32's `GetNamedSecurityInfo`/`SetEntriesInAcl` family — single-file C (`ntmarta.c`) implementing AccRewrite* / AccGetInheritanceSource / AccTreeResetNamedSecurityInfo (named-object SD get/set, ACL inheritance, explicit-entries→ACL conversion). Full for the implemented exports (spec has many commented-out Acc* it never had). Heavy ACL/SD pointer-math (the audit's warned-about area) but compact and self-contained; imports only user32/advapi32/ntdll, not boot-critical, no lsass-spine synergy. Verdict: **translate priority Low-Med** — small enough to be a clean single-file rustconv job and good ACL-idiom practice, but lower value than the servers; good rustconv-fit.

---

## sys_setup

### apphelp — dll/appcompat/apphelp
- **LOC / files:** 5854 (all C), 10 .c + 5 .h
- **Role:** Application Compatibility shim DB engine — reads/writes the SDB (.sdb) compatibility database, matches apps, applies shims (shimeng).
- **Origin:** ReactOS-native (Hentschel/Blaževic/Jansen; some Wine lineage in early SDB code but heavily ReactOS-rewritten).
- **Lang:** C.
- **Completeness:** full — substantial real implementation (sdbread/write/stringtable/fileattr, hsdb matching, shimeng dispatch). Only `apphelp_stubs.c` (generated) holds spec stubs.
- **Key deps / consumers:** ntdll (ntndk), kernel32, version (delay). Consumed by ntdll loader / shimeng and by the acgenral/aclayers shim DLLs. Not on the GINA/boot-to-desktop critical path; loaded lazily on app launch.
- **Problem points:** (1) binary SDB tag/format parsing — lots of pointer-walked tagged-record I/O (`sdbread.c`/`sdbwrite.c`), good Rust fit but unaligned/cast-heavy; (2) `dbgheap.c` custom alloc tracking + `shimeng.c` function-pointer hooking/IAT patching (unsafe glue); (3) string-table hashing must match byte-for-byte for DB compat.
- **Difficulty:** Medium-High
- **Verdict:** translate priority Med — self-contained, pure-data engine (no COM/UI), but not boot-critical; good algorithmic translation candidate once UI-tier DLLs are done.
- **rustconv-fit:** good (parsing/data tables) except shimeng IAT-patching glue (poor).

### browseui — dll/win32/browseui
- **LOC / files:** 22198 (33 .cpp + 1 .c), 29 headers. Largest in family. (+ shellbars/shellfind subdirs.)
- **Role:** Shell browser UI library — Explorer/IE browser frame, address band, autocomplete, toolbars, travel log, taskbar list, progress dialog.
- **Origin:** ReactOS-native (Giannini/Bertel et al.), built on Wine `wine` compat + ATL.
- **Lang:** C++ heavy (ATL `CComObject`/COM in 19+ files), one C file (dllinstall.c).
- **Completeness:** full (large, mature shell component).
- **Key deps / consumers:** shell32, shdocvw, comctl32, ole32, oleaut32, uxtheme, shlwapi, uuid, wine, cpprt, atl_classes. Consumed by explorer/shell. Not strictly required for `shell ready` smoke marker but part of full desktop shell.
- **Problem points:** (1) pervasive ATL/COM (apartment threading, vtables, CComPtr lifetime) — the single hardest pattern for rustconv; (2) window procs + message loops + drag/drop across many controls; (3) IShellBrowser/IServiceProvider deep COM aggregation; (4) C++ exceptions + RTTI required (cpprt).
- **Difficulty:** Very High
- **Verdict:** translate priority Low — huge ATL/COM C++ surface, low marginal boot value, very poor automation ROI.
- **rustconv-fit:** poor (ATL/COM C++).

### devmgr — dll/win32/devmgr
- **LOC / files:** 10431 (all 15 .cpp), 14 headers.
- **Role:** Device Manager UI + device property sheets (advanced properties, hardware page, resources, problem codes) and the devmgmt tree-view MMC-style snap-in.
- **Origin:** ReactOS-native.
- **Lang:** C++ with ATL/COM, RTTI + exceptions (`set_target_cpp_properties ... WITH_EXCEPTIONS WITH_RTTI`).
- **Completeness:** full (only properties/stubs.cpp is small stub set).
- **Key deps / consumers:** setupapi, newdev, shell32, comctl32, ole32, oleaut32, uxtheme, version, uuid, atl_classes, cpprt. Consumed by control panel / device-manager invocation. Not boot-critical.
- **Problem points:** (1) heavy setupapi device-enumeration (SetupDiGetClassDevs / DEVNODE walking) FFI; (2) ATL property-sheet/dialog window procs + tree-view UI; (3) COM (shell property pages); (4) C++ class hierarchy (Node/DeviceNode/ClassNode/ResourceNode polymorphism → trait objects).
- **Difficulty:** Very High
- **Verdict:** translate priority Low — large C++/ATL UI, zero boot relevance, poor ROI.
- **rustconv-fit:** poor (C++ class tree + ATL + setupapi UI).

### mscoree — dll/win32/mscoree
- **LOC / files:** 5113 (all 8 .c), 1 header.
- **Role:** .NET Common Language Runtime host shim (mscoree.dll) — CLR loading/binding entry points.
- **Origin:** Wine-derived (Wine mscoree port).
- **Lang:** C (COM-style vtable interfaces, no CONST_VTABLE — flagged `set_wine_module_FIXME`).
- **Completeness:** mostly stub — spec is dominated by `stub` entries; mscoree_main + corruntimehost/metahost/metadata are FIXME-laden thin shims (no actual managed runtime). 16 FIXMEs in mscoree_main alone.
- **Key deps / consumers:** dbghelp, ole32, shlwapi. Only matters if a .NET binary runs — never on the boot/smoke path. Effectively dead weight in vibeos.
- **Problem points:** (1) COM vtable interfaces (ICLRRuntimeHost etc.) in C; (2) since it's stubs, faithful translation is mostly returning E_NOTIMPL/S_OK — little real logic; (3) Wine CONST_VTABLE quirk.
- **Difficulty:** Low (because it's stubs) / Medium for the few real config/assembly parsers.
- **Verdict:** translate priority Low — non-functional shim, no consumer in vibeos; skip until everything real is done.
- **rustconv-fit:** good (trivially — mostly stubs), but low value.

### msgina — dll/win32/msgina  ✅ DONE (Rust cohab flip landed)
- **Status:** translated → `msgina-rust` (14 .rs / ~7.3K LOC); C cohab-disabled in `msgina/CMakeLists.txt`. Boots through the winlogon→GINA→desktop path.
- **LOC / files:** 5521 (7 .c + 1 .cpp), 2 headers.
- **Role:** Graphical Identification aNd Authentication (GINA) — winlogon's logon UI: SAS dialogs, logon/lock/shutdown screens, LSA auth glue.
- **Origin:** ReactOS-native (ReactOS GINA, GPL).
- **Lang:** C (gui/tui/lsa/shutdown/msgina) + one small C++ (dimmedwindow.cpp).
- **Completeness:** full — real Wlx* GINA interface implementation (WlxNegotiate/WlxInitialize/WlxLoggedOutSAS etc.).
- **Key deps / consumers:** advapi32, user32, gdi32, userenv, secur32 (delay), uuid, PSEH. **Loaded directly by winlogon** — on the logon → desktop path; relevant to the `shell ready` smoke chain (winlogon must successfully load+call GINA before launching userinit/explorer).
- **Problem points:** (1) the Wlx* dispatch-table ABI to winlogon (exact stdcall signatures, callback table) — ABI-critical on i386; (2) many dialog procs + window message handling (gui.c 1807 / shutdown.c 1450 LOC); (3) LSA logon (LsaLogonUser, token handling) in lsa.c; (4) PSEH usage (DUMMY_PSEH gotcha — any `_SEH2_EXCEPT` body mutating output must be dead code).
- **Difficulty:** High
- **Verdict:** translate priority **High** — it's the boot/logon-relevant module in this family; correctness here is smoke-visible, making it the most valuable target. Watch the Wlx ABI and PSEH patterns.
- **rustconv-fit:** good for lsa/utils/shutdown logic; medium for the dialog/window-proc UI.

### netcfgx — dll/win32/netcfgx
- **LOC / files:** 9100 (all 11 .c), 2 headers.
- **Role:** Network Configuration COM subsystem — INetCfg component installer, binding paths, TCP/IP config notify object, network adapter install property pages.
- **Origin:** ReactOS-native.
- **Lang:** C with COM (7 *_iface.c files implementing vtable interfaces: INetCfg, INetCfgClass, INetCfgBindingPath/Interface, INetCfgComponent).
- **Completeness:** full (only generated stubs file for spec gaps).
- **Key deps / consumers:** rpcrt4, setupapi, advapi32, iphlpapi, dhcpcsvc, ole32, ws2_32, comctl32. Consumed during network adapter install/config (setupapi class installer). Not boot-critical.
- **Problem points:** (1) extensive in-C COM interface implementations (manual Vtbl + refcount + QueryInterface across 7 files); (2) tcpipconf_notify.c (3618 LOC) — large TCP/IP registry/binding config logic; (3) setupapi + registry adapter enumeration; (4) i386 stdcall ABI on COM methods.
- **Difficulty:** High
- **Verdict:** translate priority Low-Med — large COM-in-C surface, no boot relevance; the tcpipconf logic is algorithmic but the COM scaffolding drags ROI down. Defer behind msgina/apphelp.
- **rustconv-fit:** poor for the *_iface COM files; medium for tcpipconf_notify config logic.

### syssetup — dll/win32/syssetup
- **LOC / files:** 8935 (all 8 .c), 3 headers.
- **Role:** System setup library — the GUI-mode (second-stage) install wizard, device/class install, network install, security (SAM/account) setup, log file.
- **Origin:** ReactOS-native.
- **Lang:** C.
- **Completeness:** full (real install logic; generated stubs only).
- **Key deps / consumers:** advapi32, samlib, userenv, setupapi, rpcrt4, ole32, shell32, comctl32, tzlib, uuid, PSEH. Driven during OS install (setup), not during normal LiveCD boot-to-desktop. Not on smoke path.
- **Problem points:** (1) wizard.c (3461 LOC) — many dialog/page procs (PropertySheet wizard); (2) security.c (1805) — SAM/SAdministrator account + ACL setup via samlib (security-sensitive, exact behavior matters); (3) install.c device/inf install via setupapi; (4) PSEH.
- **Difficulty:** High
- **Verdict:** translate priority Low — install-time only, no boot/smoke value; large UI + setupapi/SAM surface. Defer.
- **rustconv-fit:** medium (logic-heavy C) but UI wizard portions poor.

### TAIL

### genral (acgenral.dll) — dll/appcompat/shims/genral (338 LOC)
ReactOS-native app-compat *general shims* DLL (themes, ignore-dbgout/freelib, shimtest), built on shimlib; C, full but tiny. Loaded only by apphelp's shim engine when a matched app needs these fixes — never on the boot path. Verdict: Low priority, translate alongside apphelp (good rustconv fit; trivial size).

### layer (aclayers.dll) — dll/appcompat/shims/layer (796 LOC)
ReactOS-native app-compat *compatibility-layer shims* DLL (version-lie, disable-themes/dispmode, ignore-loadlibrary, VM horizon); C, full, small. Like acgenral it is dispatched by the shim engine for matched apps; zero boot relevance. Verdict: Low priority, bundle with apphelp/acgenral (good rustconv fit, small).

### newdev — dll/win32/newdev (2608 LOC)
ReactOS/Wine-hybrid (Poussineau/von Wittich/Finck) New Device installer DLL — the found-new-hardware wizard + DiInstallDevice/UpdateDriverForPlugAndPlayDevices; C, full (newdev.c 1072 + wizard.c 1463 + stubs). Pulled in by devmgr/PnP install, not by boot-to-desktop. Verdict: Low-Med priority — self-contained C + setupapi/dialog wizard; medium rustconv fit (logic good, wizard UI weaker).

---

## win32ss_misc

### imm32 — win32ss/user/imm32
- **LOC / files:** 12,898 LOC (pure C), 14 .c files (+ 6 headers, .rc, .spec, res/). Biggest: softkbd.c (62K), ime.c (45K), ctf.c (37K), keymsg.c/compstr.c (33K each), imm.c (35K).
- **Role:** The IMM32 client DLL — Windows Input Method Manager. User-mode side of IME (East-Asian text input): input contexts (HIMC), composition/candidate strings, soft keyboard, IME hotkeys, Cicero/TSF (CTF) bridge to msctf.dll.
- **Origin:** ReactOS-native (Wine-derived lineage + heavy ReactOS rewrite by Katayama Hirofumi MZ 2020-2022). LGPL.
- **Lang:** C
- **Completeness:** Full and mature — 132 exports, ReactOS's most-complete win32ss usermode DLL. Real implementations throughout (composition, candidate lists, A/W conversion, registry IME enumeration, soft keyboard). Only the Cicero/CTF path is partial (delegates to msctf.dll, some stubs).
- **Key deps / consumers:** win32k boundary via NtUser IME syscalls — NtUserQueryInputContext (14×), NtUserGetThreadState (9×), NtUserNotifyIMEStatus, NtUserCreate/Destroy/UpdateInputContext, NtUserAssociateInputContext, NtUserBuildHimcList, NtUserGetImeInfoEx/SetImeInfoEx, NtUserGetImeHotKey, NtUserDisableThreadIme, NtUserSetThreadLayoutHandles. Reads gpsi/gSharedInfo (win32k shared section) directly. Imports user32, gdi32, advapi32. Loads IME DLLs (imm-prop) and msctf.dll dynamically. Consumers: user32 (input/keyboard message path), all GUI apps with text input. **Boot relevance: LOW** — IME is not on the desktop-render critical path; smoke boot does not exercise IME. Peers: user32-rust, win32k-rust (the NtUser IME side lives in win32k).
- **Problem points:**
  1. **win32k IME syscall surface** — ~17 distinct NtUser IME syscalls with shared INPUTCONTEXT/IMC structures; FFI must match win32k's IME struct layouts exactly (INPUTCONTEXTDX, IME_STATE, IMEINFOEX, IMEDPI). Cross-thread/cross-process HIMC validation logic.
  2. **Direct shared-section reads** (gpsi, gSharedInfo, ValidateHandle on win32k handle table) — same captures(none)/raw-pointer hazards as user32-rust; reads win32k's USER handle table from user mode.
  3. **A/W (ANSI/Unicode) string conversion** pervasive — codepage-aware CANDIDATELIST/RECONVERTSTRING/COMPOSITIONSTRING marshalling between fixed C structs; lots of pointer-arithmetic into variable-length buffers (cast-before-offset class hazard).
  4. **CTF/Cicero COM bridge** (ctf.c) — light COM (CoInitialize, one ITfLangBarMgr ref), mostly LoadLibrary("msctf.dll")+GetProcAddress thunks; manageable but the GUID-atom/thread-mgr lifecycle is fiddly.
- **Difficulty:** High (large surface, heavy struct-ABI coupling to win32k IME types, A/W marshalling, soft-keyboard bitmap code).
- **Verdict:** translate priority **Low-Med**. Largest module in the family and fully-implemented ReactOS-native C, so it has translation value — but it is off the boot critical path and depends on the win32k IME syscall layer being stable. Sequence it after user32-rust/win32k-rust IME types settle; not a near-term win.
- **rustconv-fit:** good for the leaf algorithm/conversion files (utils.c, candidate.c, guideline.c, regword.c, imepro.c, win3.c, compstr.c A/W converters); **poor** for ctf.c (COM/dynamic-load thunks), keymsg.c and ime.c (deep win32k syscall + shared-state coupling). Translate per-file, easy converters first.

### gdi32_vista — win32ss/gdi/gdi32_vista (31 LOC)
Tiny Vista-era GDI thunk DLL: 3 trivial functions across d3dkmt.c (D3DKMTOpenAdapterFromGdiDisplayName → STATUS_PROCEDURE_NOT_FOUND stub) and dc.c (D3DKMTCreateDCFromMemory/DestroyDCFromMemory → one-line NtGdiDdDDI* syscall passthroughs). Pure C, mostly stub/passthrough. Verdict: trivial translate, but near-zero value (3 thunks) — Low priority, do opportunistically.

### dxapi — win32ss/reactx/dxapi (325 LOC)
Kernel-mode "driver" (dxapi.sys) that is never actually loaded — exists only to export DirectDraw/DxApi dispatch entry points to win32k eng. main.c: DriverEntry (returns SUCCESS), DxApiGetVersion (0x402), DxApi dispatch table router, and ~22 Dx* handlers that are FIXME/UNIMPLEMENTED stubs. Pure C. Essentially a stub export shell. Verdict: Low priority — it's a placeholder driver with no real logic; little to gain from translation.

### dxgthk — win32ss/reactx/dxgthk (26 LOC)
Stub kernel "driver" (dxgthk.sys) — like dxapi, never loaded; only a DriverEntry returning STATUS_SUCCESS plus a .spec re-exporting win32k eng functions (dxgthk.spec is the real content). Pure C, single trivial function. Verdict: Low/skip — nothing of substance to translate (the value is in the .spec, not C).

### win32u — win32ss/win32u (31 LOC)
The Win32 syscall-thunk DLL: a no-op DllMain (main.c) plus sys-stubs.S, with the entire export surface (~1000 syscall thunks, win32u.spec is 31KB) auto-generated as assembly syscall stubs by the build. No hand-written C logic at all. Pure C/asm boilerplate. Verdict: skip translation — it is mechanically-generated syscall trampolines, not algorithmic code; a Rust port would gain nothing.

---

## win32_misc_a

### opengl32 — dll/opengl/opengl32 (3496 LOC) — UNEXPECTEDLY COMPLEX
ReactOS-authored OpenGL 1.x client DLL: ICD loader (`icdload.c`), WGL context/pixel-format management (`wgl.c`, 944 LOC), WGL bitmap-font support (`wgl_font.c`, 570 LOC), and a software GL renderer (`swimpl.c`, 1527 LOC — the bulk). All C. Critically it ships **hand-written x86 asm trampolines** (`glapi_x86.s`) for dispatching to ICD-provided gl* functions, and the CMake passes Mesa/MS-header export-control defines. The asm dispatch layer and the large software rasterizer make this the hardest module in the family. Verdict: LOW priority, HIGH difficulty — defer; the `glapi_x86.s` trampolines are a per-arch porting blocker and not worth early effort.

### wdmaud.drv — dll/win32/wdmaud.drv (2777 LOC) — DEFER
WDM Audio user-mode driver (winmm/MME side). Heavy reliance on external libraries: `mmebuddy` (MME-Buddy framework), `mmixer` (mixer line library), plus legacy/mmixer dual code paths selected by `USE_MMIXER_LIB`. C. Most real work lives in the linked mmebuddy/mmixer libs, not here; this is glue (wodMessage etc. are in mmebuddy). Key deps: mmebuddy, mmixer, ksuser, setupapi. Verdict: LOW priority, MEDIUM difficulty — non-boot audio glue; defer behind the mmebuddy/mmixer libraries it depends on.

### shimgvw — dll/win32/shimgvw (2632 LOC) — C++/COM/GDI+, DEFER
ReactOS Picture and Fax Viewer. **Mixed C and C++**: `loader.cpp` uses GDI+ (`using namespace Gdiplus`, C++ classes deriving from `BITMAPINFOHEADER`) and `comsup.cpp`-style COM support; `shimgvw.c` is a full shell/commctrl/commdlg UI. Deps: gdiplus, shell32, comctl32, comdlg32, windowscodecs. Verdict: LOW priority, HIGH difficulty — the C++/GDI+ image-loader portion is out of the C→Rust mechanical path; defer.

### mmdrv — dll/win32/mmdrv (2070 LOC) — DEFER
Legacy MME user-mode driver (NT-style DriverProc satisfying winmm). Full C implementation split across wave/midi/aux/mixer (`wave.c`, `wave_io.c`, `midi.c`, `auxil.c`, `mme.c`, `session.c`, `kernel.c`). Talks to kernel sound drivers via DeviceIoControl. Deps: winmm, mmddk headers. Verdict: LOW priority, MEDIUM difficulty — self-contained C but non-boot legacy audio; defer.

### psapi — dll/win32/psapi (1675 LOC) — GOOD CANDIDATE
ReactOS-authored Process Status API (EnumProcessModules, GetModuleFileNameEx, GetMappedFileName, QueryWorkingSet, etc.). Full C impl over NDK (`ndk/psfuncs`, `exfuncs`, `mmfuncs`, `rtlfuncs`) — i.e. thin wrappers around NtQuerySystemInformation/NtQueryVirtualMemory with realloc loops. Uses PSEH2 (note DUMMY_PSEH rules). Verdict: MEDIUM priority, MEDIUM difficulty — clean self-contained NDK-wrapping DLL, a solid translatable target.

### ws2help — dll/win32/ws2help (1465 LOC) — CANDIDATE
ReactOS WinSock2 helper DLL (legacy WS2HELP): handle-context tables, APC/notify, upcall context. Full C across `apc.c`/`context.c`/`handle.c`/`notify.c`/`dllmain.c`. Uses critical sections, a global heap, registry reads, and a `wshdrv.h` kernel interface. Deps: ntdll, kernel32. Verdict: MEDIUM priority, MEDIUM difficulty — small, self-contained, network-adjacent (could interest the netstack lane); translatable.

### nfs (nfs41_np) — dll/np/nfs (1101 LOC) — DEFER
NFSv4.1 network provider (MPR np provider, NPGetCaps/NPAddConnection etc.). Third-party (U. Michigan), not ReactOS-authored, but C and self-contained. Talks to the nfs41 RDR driver via DeviceIoControl + registry options parsing (`options.c`). Deps: npapi, advapi32. Verdict: LOW priority, MEDIUM difficulty — niche non-boot net provider, externally-sourced; defer.

### authz — dll/win32/authz (836 LOC) — CANDIDATE (mostly stubs)
ReactOS Authorization Framework (AuthzInitializeResourceManager, AuthzAccessCheck, client-context mgmt across `clictx.c`/`resman.c`). Largely **unimplemented** — 17 `@unimplemented` functions returning failure; only resource-manager/context scaffolding is real. C. Verdict: LOW-MEDIUM priority, LOW difficulty — small, mostly stub bodies make for a quick faithful port, but low value since it's unimplemented anyway.

### ifmon — dll/win32/ifmon (788 LOC) — CANDIDATE
NetSh IF-monitor helper DLL (Eric Kohl, 2025). InitHelperDll + interface/ip/winsock netsh command handlers. C, self-contained over iphlpapi/netsh helper API + GUID tables. Deps: netsh framework. Verdict: LOW-MEDIUM priority, MEDIUM difficulty — clean modern ReactOS C, translatable but non-boot netsh tooling.

### fltlib — dll/win32/fltlib (669 LOC) — CANDIDATE
Filter Manager user library (FilterLoad/Unload, FilterConnectCommunicationPort, FilterSendMessage). Full C over NDK io/ob, DeviceIoControl to fltmgr. `stubs.c` present but 0 @unimplemented in it; real bodies in `fltlib.c`/`message.c`. Deps: ntdll, fltmgr_shared. Verdict: MEDIUM priority, LOW-MEDIUM difficulty — small clean NDK-wrapping lib; good candidate.

### wlnotify — dll/win32/wlnotify (646 LOC) — CANDIDATE
Winlogon notification handlers (Eric Kohl) — scard/schedsvc/senssvc/termserv/wlballoon event callbacks invoked by winlogon. C, uses WINE_DEFAULT_DEBUG_CHANNEL. Mostly thin handlers (service-start/notify). Deps: advapi32, winlogon notify ABI. Verdict: LOW-MEDIUM priority, LOW difficulty — small, simple callbacks; easy faithful port, modest value.

### hid — dll/win32/hid (544 LOC) — CANDIDATE, NOTE PEER
ReactOS HID user library (HidD_* / HidP_* wrappers). Full C over NDK + DeviceIoControl to hidclass. **Existing Rust peers in tree**: `sdk/lib/rust/hid-sys`, `hidparse-rust`, `hidparser-rust`, `hidparse-cargo` — these cover the HID *parser* (HidP descriptor parsing), which hid.dll forwards to. Verdict: MEDIUM priority, LOW-MEDIUM difficulty — small; coordinate with the existing hidparse-rust crates (the HidP_* parsing half may already be Rust-backed; this DLL is mostly the HidD_* ioctl wrappers).

### storprop — dll/win32/storprop (321 LOC) — CANDIDATE
Storage device property-sheet provider (Eric Kohl) — VolumePropPageProvider + disk/dvd/ide property pages. C, UI/setupapi propsheet code. Deps: setupapi, comctl32. Verdict: LOW priority, LOW-MEDIUM difficulty — small but UI-heavy (propsheets), non-boot; defer-ish.

### sndblst — dll/win32/sndblst (265 LOC) — DEFER
Sound Blaster MME user-mode driver — thin shim over the mmebuddy/mment4 libraries (just device-name strings + entry registration). C. All real logic is in mmebuddy. Verdict: LOW priority, LOW difficulty — trivial but depends on mmebuddy; non-boot legacy audio, defer with the audio family.

### streamci — dll/win32/streamci (252 LOC) — CANDIDATE
Streaming device class installer (Johannes Anderwald). C, DeviceIoControl helper (`PerformIO`) + class-installer entry points. Deps: setupapi. Verdict: LOW-MEDIUM priority, LOW difficulty — small self-contained class installer; easy port, modest value.

### acledit — dll/win32/acledit (220 LOC) — STUB
ACL Editor. Effectively a **stub DLL** — `acledit.c` is DllMain + `stubs.c` exported entry points (EditAuditInfo/EditOwnerInfo/EditPermissionInfo etc.), no real impl. C. Verdict: LOW priority, LOW difficulty — trivial stub port, near-zero value.

### wshirda — dll/win32/wshirda (203 LOC) — CANDIDATE
WinSock2 IrDA helper DLL (WSHGetSockaddrType, WSHOpenSocket, address-type helpers for IrDA). Full C, small, self-contained over the WSH helper ABI. Deps: ws2_32 helper interface. Verdict: LOW priority, LOW difficulty — small clean port; niche (IrDA) so low value.

### wing32 — dll/win32/wing32 (143 LOC) — CANDIDATE (Wine-sourced)
WinG (legacy fast-DIB) support, Wine-authored (Dmitry Timoshkov). Tiny: WinGCreateDC→CreateCompatibleDC, WinGRecommendDIBFormat, WinGCreateBitmap/GetDIBPointer, BitBlt wrappers. C over gdi32/user32. Verdict: LOW priority, LOW difficulty — trivial gdi wrapper port; legacy/low value.

### sfc — dll/win32/sfc (130 LOC) — CANDIDATE (forwarder-ish)
System File Checker (WFP) front DLL — mostly forwards/wraps to sfc_os. Small C, DllMain + thin SfcGetNextProtectedFile/SfcIsFileProtected wrappers. Deps: sfc_os. Verdict: LOW priority, LOW difficulty — tiny wrapper; low value.

### srclient — dll/win32/srclient (76 LOC) — STUB
System Restore client. `srclient_main.c` is **unimplemented stubs** (SRSetRestorePointA/W, SRRemoveRestorePoint all DPRINT1 "unimplemented"). C. Verdict: LOW priority, LOW difficulty — trivial stub; near-zero value.

### msxml2 — dll/win32/msxml2 (63 LOC) — WINE/COM TYPELIB, SKIP-ish
MSXML2 entry DLL — `-D_WINE`, links `wine`, builds a COM typelib from `msxml2_tlb.idl`, and DllGetClassObject forwards to **msxml3**. main.c is just DllMain + class-factory forwarding. The substance is the IDL typelib + msxml3. Verdict: LOW priority, LOW-MEDIUM difficulty — the .c is trivial but it's a Wine COM forwarder fronting an IDL typelib (registration/typelib build); not a clean mechanical C→Rust target. Flag: COM/Wine.

### msxml6 — dll/win32/msxml6 (60 LOC) — WINE/COM TYPELIB, SKIP-ish
Same pattern as msxml2: Wine-sourced, IDL typelib (`msxml6_tlb.idl`), DllCanUnloadNow/DllGetClassObject forwarding, manifest + rgs. Verdict: LOW priority, LOW-MEDIUM difficulty — COM/Wine typelib front; not a good mechanical target. Flag: COM/Wine.

### sfcfiles — dll/win32/sfcfiles (59 LOC) — DATA TABLE
Just a static **table of protected system files** (PROTECT_FILE_ENTRY array) + DllMain + SfcGetFiles accessor. C, no logic. Verdict: LOW priority, TRIVIAL difficulty — pure data lift (like the NLS data-table ports); easy but near-zero value.

### msvcrt20 — dll/win32/msvcrt20 (50 LOC) — THIN SHIM
MSVCRT 2.0 compat shim — re-exports msvcrt with a couple of fixups (__getmainargs/__wgetmainargs new_mode-by-value vs by-pointer). Wine-sourced. C, tiny. Verdict: LOW priority, TRIVIAL difficulty — thin ABI-shim; the bulk is forwarders to msvcrt in the .spec.

### comcat — dll/win32/comcat (48 LOC) — STUB COM
Component Categories Manager — `comcat_main.c` is **stub COM exports** (DllCanUnloadNow→S_FALSE, DllRegisterServer→S_OK, DllGetClassObject). 4-line spec. C. Verdict: LOW priority, TRIVIAL difficulty — stub COM shell; near-zero value. Flag: COM (but only stubs).

### xinput1_2 — dll/win32/xinput1_2 (39 LOC) — PURE FORWARDER
Pure forwarder DLL: every export (XInputEnable/GetState/SetState/GetCapabilities/...) forwards to **xinput1_3** in the .spec; `xinput1_2_main.c` is just DllMain. Verdict: SKIP — pure spec forwarder, nothing to translate. Flag: forwarder.

### npptools — dll/win32/npptools (39 LOC) — PURE STUB
`main.c` is only DllMain; the .spec is ~60 `@stub` entries (Netmon NPP blob API). Nothing implemented. Verdict: SKIP — pure stub DLL, no bodies to translate. Flag: stub.

### wsock32 — dll/win32/wsock32 (22 LOC) — MOSTLY FORWARDER
Legacy WinSock 1.1 DLL — `stubs.c` is just DllMain returning TRUE; the .spec forwards 49 entries to **ws2_32** (accept/bind/connect/recv/send/...). A handful of non-forwarded WSA* helpers may exist but the body is essentially empty. Verdict: SKIP/trivial — spec forwarder to ws2_32. Flag: forwarder.

### ufat — dll/win32/ufat (22 LOC) — STUB
FAT format/chkdsk management DLL — `ufat.c` is only DllMain (returns TRUE); real functionality is unimplemented/external. C. Verdict: SKIP/trivial — empty DllMain stub. Flag: stub.

---

## win32_misc_b

Family of 29 native dll/win32 helper DLLs (all TAIL-sized). None has an existing Rust peer. Several are Wine-derived (`target_link_libraries(... wine)` or `set_wine_module`), several are pure C++/COM, and a handful are forwarders/near-empty DllMain shells. Notes below flag each.

### dnsapi — dll/win32/dnsapi (3163 LOC)
DNS resolver API (DnsQuery etc.) built on the bundled 3rd-party **adns** library (`sdk/lib/3rdparty/adns`); ReactOS-authored glue (query.c/context.c/record.c/names.c/memory.c) + RPC client to dnsrslvr (`add_rpc_files` dnsrslvr.idl) + a large `stubs.c` (~105 stub markers). Pure C, but heavily entangled with the C adns lib and generated RPC stubs (dnsrslvr_c.c). Verdict: LOW priority, MEDIUM-HIGH difficulty — the adns interop and RPC stub layer make this the awkward one of the family; not boot-critical. Translate the ReactOS glue only, leave adns as the C lib.

### msports — dll/win32/msports (2771 LOC)
Ports (COM/LPT) class-installer + COM port database (comdb.c) + serial/parallel property pages. Wine-linked but ReactOS-authored C; uses setupapi/comctl32 device-install plumbing and registry. Full implementation, not stubs. Verdict: LOW priority (device-install UI, not boot path), MEDIUM difficulty.

### mswsock — dll/win32/mswsock (2366 LOC)
Microsoft Winsock helper / extensions (AcceptEx, TransmitFile, namespace lookup nsplookup.c). ReactOS-authored C, mostly thin wrappers over ws2_32 + dnsapi; one stubs.c. Full-ish impl. Verdict: MEDIUM priority (companion to the in-tree netstack/winsock work), MEDIUM difficulty — clean FFI surface over ws2_32.

### lpk — dll/win32/lpk (1938 LOC)
Language Pack / complex-script (BiDi) GDI shaping hook DLL — entrypoint LpkDllInitialize. **Wine-derived** (bidi.c/direction.c carry Wine BiDi copyrights), links `wine` + usp10. Full impl + small stub.c. Verdict: LOW priority, MEDIUM-HIGH difficulty — Unicode BiDi algorithm tables, Wine-origin; quality-sensitive, not boot path.

### framedyn — dll/win32/framedyn (1512 LOC)
WMI provider framework (WBEM) — **C++/COM** with exceptions (`set_target_cpp_properties WITH_EXCEPTIONS`, links cpprt/oleaut32). chstring.cpp implements a CHString class (deliberately "BROKEN on purpose" per header note) + provider.cpp COM provider classes. Verdict: SKIP / LOW — C++/COM, not a good early Rust target; defer.

### winsta — dll/win32/winsta (1163 LOC)
WinStation / Terminal Services session API (WinStationEnumerate, logon, query, security). Wine-include + `wine`-linked, ReactOS C. **Largely stubbed** (ws.c ~52 stub markers, query.c ~24, security.c ~17 — most functions return failure). Verdict: LOW priority, LOW-MEDIUM difficulty — many bodies are trivial stubs, so a translation is mostly mechanical but low value (TS not used).

### beepmidi — dll/win32/beepmidi (913 LOC)
Toy MIDI-via-PC-speaker (beep.sys) player; standalone C with threading/polyphony, links winmm. Self-contained, full impl, no deps beyond winmm. Verdict: LOW priority (novelty driver), LOW-MEDIUM difficulty — nice isolated translate candidate if doing easy wins.

### netid — dll/win32/netid (805 LOC)
"Computer Name" / Network ID control-panel property page; single netid.c, GUI (user32/comctl32) + netapi32/dnsapi. ReactOS C, full impl, has a .def (not spec2def-only). Verdict: LOW priority (CPL UI), MEDIUM difficulty — dialog/property-sheet heavy.

### fmifs — dll/win32/fmifs (731 LOC)
File Management IFS utility (Format/Chkdsk/Diskcopy/Extend/Compress) — entrypoint InitializeFmIfs, imports only kernel32/ntdll. ReactOS C, thin wrappers that load the per-FS u*.dll backends (ufat/ucdfs/etc.); format.c has a couple stubs. Verdict: MEDIUM priority (used by installer/format tooling), LOW-MEDIUM difficulty — small, clean FFI.

### iernonce — dll/win32/iernonce (661 LOC)
Extended RunOnceEx processing with UI (registry-driven). **C++** (links cppstl + atl_classes, WITH_EXCEPTIONS) — dialog.cpp/iernonce.cpp/registry.cpp; uses ATL. Verdict: SKIP / LOW — C++/ATL, defer past pure-C targets.

### winfax — dll/win32/winfax (632 LOC)
FAX API (FAXAPI) — single winfax.c that is **almost entirely stubs** (~55 stub/return-error markers); imports only msvcrt/kernel32/ntdll. Verdict: LOW priority, LOW difficulty — trivial near-empty translate, near-zero value.

### serialui — dll/win32/serialui (417 LOC)
COM-port config dialog (drvCommConfigDialog) + get/set default comm config; single C file, user32/shlwapi. Full impl, dialog-driven. Verdict: LOW priority, LOW-MEDIUM difficulty (dialog proc + tables).

### rasadhlp — dll/win32/rasadhlp (285 LOC)
RAS AutoDial helper hooked into ws2_32 (autodial.c/winsock.c/init.c) + small stubs. ReactOS C, thin Winsock-hook layer. Verdict: LOW priority, LOW difficulty — small, clean; companion to netstack/winsock if that lane wants it.

### cards — dll/win32/cards (256 LOC)
Playing-card graphics library (cardlib) used by Solitaire/FreeCell — GDI drawing, user32/gdi32. ReactOS C, full impl, no exotic deps. Verdict: LOW priority (games asset lib), LOW difficulty — self-contained, easy translate.

### batt — dll/win32/batt (243 LOC)
Battery/composite-battery class installer (setupapi device install). Single C file, full impl. Verdict: LOW priority, LOW difficulty.

### sfc_os — dll/win32/sfc_os (215 LOC)
System File Checker / Windows File Protection — single sfc_os.c, mostly stubs (spec is `sfc_os_1..N -stub` ordinals + a couple real exports like SfcFileException). Verdict: LOW priority, LOW difficulty (near-stub).

### verifier — dll/win32/verifier (180 LOC)
Application Verifier provider — entrypoint DllMain, **imports only ntdll**, registers AVrf DLL load/unload/heap callbacks against the RTL verifier infrastructure. Small C. Verdict: LOW priority, MEDIUM difficulty — small but ntdll-RTL-internal coupling (callback registration) needs care.

### ucrtbase — dll/win32/ucrtbase (132 LOC)
Universal CRT shim DLL — SHARED, pulls in ucrt/msvcrt_shared/crtmath/vcruntime static libs; the dll/win32 source here is just printf.c + stubs.c + generated ucrtbase_stubs.c wrapping those CRT libs. **CRT-floor / boot-relevant** (it IS the C runtime). Verdict: SKIP — do not translate; it's a CRT aggregation shim, not application logic.

### profmap — dll/win32/profmap (77 LOC)
User Profile Mapping API — single small C file (DllMain + a few exports), msvcrt/kernel32/ntdll + a stubs.c. Verdict: LOW priority, LOW difficulty — trivial.

### modemui — dll/win32/modemui (76 LOC)
Modem properties CPL — single small C file with a dialog proc; near-stub (+ generated stubs.c). Verdict: LOW priority, LOW difficulty — trivial.

### msxml — dll/win32/msxml (63 LOC)
MSXML version DLL — **Wine-derived COM forwarder** (`set_wine_module`, `-D_WINE`); main.c is just DllMain + the 4 COM registration entrypoints, all **forwarded to msxml3** in the spec. Plus a typelib. Verdict: SKIP — forwarder/COM-reg shim, no real logic to translate.

### msxml4 — dll/win32/msxml4 (60 LOC)
Same as msxml: **Wine COM forwarder** to msxml3, main.c = DllCanUnloadNow/DllMain + typelib registration. Verdict: SKIP — forwarder shim.

### bthci — dll/win32/bthci (56 LOC)
Bluetooth class installer — single tiny C file, one export (BluetoothClassInstaller) that's effectively a switch returning defaults; setupapi. Verdict: LOW priority, LOW difficulty (near-stub).

### getuname — dll/win32/getuname (49 LOC)
Unicode character-name lookup — one export GetUName that LoadStringW's a description from resources. Tiny C, user32. Verdict: LOW priority, TRIVIAL difficulty — but value is mostly in the resource strings, not code.

### xinput9_1_0 — dll/win32/xinput9_1_0 (39 LOC)
XInput version DLL — **Wine forwarder** (`set_wine_module`); main.c is bare DllMain, all 4 XInput* exports **forwarded to xinput1_3** in the spec. Verdict: SKIP — pure forwarder.

### xinput1_1 — dll/win32/xinput1_1 (39 LOC)
Same as xinput9_1_0: **Wine forwarder** to xinput1_3, bare DllMain. Verdict: SKIP — pure forwarder.

### advapi32_vista — dll/win32/advapi32_vista (25 LOC)
Vista advapi32 extension DLL — DllMain.c calls RegInitialize then **compiles in advapi32's own reg/reg.c** (`../advapi32/reg/reg.c`); Wine + PSEH linked. The 25 LOC is just the DllMain glue; real body is shared advapi32 source. Verdict: SKIP as a standalone target — translate alongside advapi32's reg module, not here.

### ufatx — dll/win32/ufatx (22 LOC)
FATX FS-management DLL — **nativedll**, ufatx.c is a do-nothing DllMain; all logic is in the linked `vfatxlib` static lib. Verdict: SKIP standalone — the value is in vfatxlib (separate target); this is just an empty wrapper.

### ucdfs — dll/win32/ucdfs (20 LOC)
CDFS FS-management DLL — **nativedll**, empty DllMain wrapper around the linked `cdfslib` static lib (paired with the in-tree cdfs-rust work). Verdict: SKIP standalone — translate cdfslib, not this empty shell.

---

## printing

The ReactOS print stack (all by Colin Finck, 2015-2018, GPL-2.0+; C; UNICODE). Topology:
`winspool.drv` (client API) → RPC (`ncalrpc`, `winspool.idl`) → `spoolsv.exe` (service, RPC server) →
`spoolss.dll` (router: loads print providers) → `localspl.dll` (local provider, registry-backed) →
print processor (`winprint.dll`, RAW). Heavy shared `marshalling/` lib serializes the PRINTER_INFO_*,
JOB_INFO_*, DRIVER_INFO_*, FORM_INFO_*, PORT_INFO_*, MONITOR_INFO_* level structures across the RPC
boundary. **Boot relevance: none** — Spooler is a non-essential service; the smoke desktop never needs
printing. This whole family is a low-priority, self-contained translation cluster.

### winspool — win32ss/printing/base/winspool

- **LOC / files:** 9,807 LOC, 13 C files (+ shared `../marshalling.c`, generated `winspool_c.c`/`winspool_stubs.c`)
- **Role:** Client-side Win32 spooler API DLL (`winspool.drv`) — the public `OpenPrinter`/`AddPrinter`/`StartDocPrinter`/`EnumPrinters`/`GetPrinterDriver`/etc. surface apps link against; marshals to the spooler over RPC.
- **Origin:** ReactOS-native (Colin Finck); a few helpers Wine-derived (wspool.h header is Wine 2005, devmode/forms conversion).
- **Lang:** C
- **Completeness:** full + spec stubs — `.spec` lists many `stub`/`stub -noname` entries (EnumPrinterPropertySheets, ClusterSpl*, Perf*, AddPrinterConnection, ConnectToPrinterDlg) but the core printing path is implemented.
- **Key deps / consumers:** rpcrt4 (RPC client to spooler `ncalrpc`/`spoolss` endpoint), advapi32, gdi32, user32, marshalling lib, `winspool_c.c` MIDL stubs. Consumers: every printing app, GDI print path, control-panel printer UI. No boot relevance.
- **Problem points:** (1) RPC client binding + MIDL-generated stubs (`_RpcXxx`, `RpcTryExcept`/`RpcExcept` SEH) — must interop with the rpcrt4 runtime; (2) `printers.c` is **127KB** alone — dominated by dynamic loading of driver UI DLLs (DeviceCapabilities, *PropertySheets, ConstructPrinterFriendlyName) via fn-pointer typedefs + GetProcAddress, plus DEVMODE/security-descriptor marshalling; (3) ANSI↔Unicode in-place conversion helpers + DEVMODEA/W conversion (devmode.c); (4) `SPOOLER_HANDLE` bitfield struct + handle protect/unprotect; (5) PSEH (`RpcExcept`) → DUMMY_PSEH dead-code rules apply.
- **Difficulty:** High (sheer size of printers.c, RPC-client + MIDL interop, many function-pointer plug-in call sites)
- **Verdict:** translate priority **Low** — large, no boot value, deep RPC/MIDL + driver-UI plug-in coupling; only worthwhile once the rest of the print stack and rpcrt4 interop are settled.
- **rustconv-fit:** poor — RPC/MIDL stub interop, fn-pointer driver-UI loading, and the 127KB printers.c exceed clean per-file FFI translation; would need heavy manual review.

### localspl — win32ss/printing/providers/localspl

- **LOC / files:** 8,604 LOC, 13 C files (+ generated `localspl_stubs.c`)
- **Role:** The Local Print Provider DLL — exports the single `InitializePrintProvidor`, returns a `PRINTPROVIDOR` vtable of `LocalOpenPrinter`/`LocalEnumPrinters`/`LocalStartDocPrinter`/`LocalScheduleJob`/`LocalAddForm`/etc. Implements actual local printers/jobs/ports/forms/drivers backed by the registry + spool directory.
- **Origin:** ReactOS-native (Colin Finck)
- **Lang:** C
- **Completeness:** full — implements the provider vtable; many vtable slots are `NULL` by design (AddPrinter/DeletePrinter/network connections not done), not stubs of failed work.
- **Key deps / consumers:** registry (HKLM Print/Printers keys), spool directory file I/O, **skiplist16** (sdk/lib/skiplist — custom ordered-map data structure used pervasively for printer/job/form/port tables), spoolss (calls back into router), advapi32_vista, netapi32, secur32, rpcrt4. Consumer: loaded by `spoolss.dll`. No boot relevance. Builds with `_WIN32_WINNT=0x600`.
- **Problem points:** (1) `PRINTPROVIDOR` function-table ABI — the exported vtable must match exactly so spoolss can call through; (2) pervasive **skiplist** dependency (would want the skiplist crate translated first, or FFI to the C skiplist); (3) `jobs.c` (49KB) + `printers.c` (68KB) + `printerdrivers.c` (39KB) are large, registry-heavy with elaborate `precomp.h` data structures (17KB header); (4) a real worker thread (`printingthread.c`) — job scheduling/printing; (5) registry layout + version-quirk constants (mimics Win2003 spooler).
- **Difficulty:** High (large registry-backed state, skiplist dependency, vtable ABI, threading)
- **Verdict:** translate priority **Low** — central to a working print stack but no boot value; gate behind skiplist translation and the spoolss router. Good *eventual* candidate (self-contained, single export) but not soon.
- **rustconv-fit:** poor — skiplist coupling + large registry/data-structure-heavy files + worker thread; per-file translation would leave many cross-file `static` structs and the vtable to wire manually.

### spoolss — win32ss/printing/base/spoolss (2,371 LOC)
ReactOS-native spooler **router** DLL; C; full. Loads print-provider DLLs (`LoadLibraryW` + `GetProcAddress("InitializePrintProvidor")`), keeps a `PrintProviderList`, and routes Enum/Open/etc. across providers; also exposes `DllAllocSplMem`/marshalling memory helpers used by the rest of the stack. 14 small files (largest printers.c 15KB). Verdict: Low priority, but the cleanest mid-tier piece — small files, plain Win32/registry, no MIDL; reasonable rustconv fit once the family is targeted.

### spoolsv — win32ss/printing/base/spoolsv (2,193 LOC)
ReactOS-native print **spooler service** (`spoolsv.exe`); C; full. A Win32 service host that starts the RPC server (`RpcServerUseProtseqEpW`/`RpcServerRegisterIf winspool_v1_0_s_ifspec`/`RpcServerListen`) and dispatches the MIDL-generated server stubs (`winspool_s.c`, rpcstubs.c) into spoolss. Mostly thin RPC-server-side forwarding + service-control plumbing. Verdict: Low priority; translation blocked on rpcrt4 server-side + MIDL `_s` stub interop (the same RPC hurdle as winspool's client side); poor rustconv fit due to MIDL/RPC server glue.

### winprint — win32ss/printing/processors/winprint (378 LOC)
ReactOS-native standard **print processor** DLL (`winprint.dll`, installed to spool/prtprocs); C; full but tiny — implements only the RAW datatype (`OpenPrintProcessor`/`PrintDocumentOnPrintProcessor`/`ClosePrintProcessor`, raw.c spools bytes straight to the port via spoolss). Verdict: Low priority (no boot value) but **smallest and easiest by far** — clean Win32 API, no RPC/MIDL, no skiplist; would be the natural first/sample translation if this family is ever picked up. Good rustconv fit.

---

## cpl

Control Panel applets (`*.cpl` DLLs). Every module exports the standard `LONG CALLBACK CPlApplet(HWND, UINT, LPARAM, LPARAM)` (stdcall) entry and is a usermode leaf loaded on-demand by the CPL host (control.exe / shell). **None are on the boot/smoke path** — the smoke north-star never exercises any of them, so translation has zero OS-correctness payoff. All are pure C (0 C++/ATL), Win32-GUI-shaped (dialog procs, property sheets, registry, FFI-thick), which is the *weakest* fit for rustconv (tuned for ntoskrnl/HAL/driver kernel C with pure-algorithm content). No CPL applet has a Rust peer. Net: the whole family is **translate-priority Low / rustconv-fit poor**, with value only as a usermode-Rust/FFI exercise. Two modules carry Wine markers (input, wined3dcfg) and usrmgr links the `wine` static lib.

### desk — dll/cpl/desk
- **LOC / files:** 11,530 LOC, 20 files, 100% C (0 C++). Largest: `monslctl.c` (1659), `background.c` (1300), `settings.c` (1135), `devsett.c` (1107), `theme.c` (1105), `screensaver.c` (929).
- **Role:** The Display Control Panel applet (`desk.cpl`) — classic Win2k/XP "Display Properties" multi-tab property sheet (Themes/Background/Screensaver/Appearance/Settings).
- **Origin:** ReactOS-native (no Wine markers, no Wine-hybrid files).
- **Lang:** Pure C. COM used only as hand-rolled C vtables (`COBJMACROS`, `lpIDataObjectVtbl`, `impl_from_IDataObject`) — no ATL, no C++.
- **Completeness:** Full — production applet, no stub-out (only an optional `CHECK_SCR_FULL_PATH` feature `#if 0`'d).
- **Key deps / consumers:** user32, advapi32, gdi32, comctl32, comdlg32, ole32, setupapi, shell32, shlwapi, uxtheme, **gdiplus**, + `uuid` static lib. Exports `CPlApplet` + `DisplayClassInstaller`/`MonitorClassInstaller` co-installers + `InstallScreenSaver{A,W}`. No boot/smoke relevance. No Rust peer.
- **Problem points:**
  - Heavy Win32 GUI: ~12 dialog/property-sheet page procs + a full **custom registered window class** (`monslctl.c` "MONITORSELWNDCLASS" — own WndProc, bitfield struct, drag/hit-test/scroll) and a screensaver-preview **subclass** via `SetWindowLongPtr(GWLP_WNDPROC)` — raw HWND/HDC GDI/GDI+ drawing that maps poorly to safe Rust.
  - **Hand-rolled COM object** in `devsett.c`: `CDevSettings` implementing `IDataObject` via manual C vtable — must be reproduced as FFI-correct vtable layout in Rust.
  - **Display-config plumbing:** setupapi (`SetupDiGetClassDevsW` etc.), `EnumDisplayDevices`/`ChangeDisplaySettingsEx`, DEVMODE mode tables, broad registry; all exports stdcall.
- **Difficulty:** Very High — large LOC, GUI-callback-heavy, custom window class, hand-rolled COM, wide setupapi/GDI+ FFI; almost no pure-algorithm content.
- **Verdict:** translate priority **Low** — non-boot user-only applet with the worst-fit profile (UI + callbacks + COM vtables + GDI+); high cost, no smoke payoff.
- **rustconv-fit:** poor — dominated by Win32 message-loop UI, manual COM vtables, FFI-thick device/registry calls.

### intl — dll/cpl/intl
- **LOC / files:** 5,160 LOC, 12 files, 100% C (0 C++). Largest: generalp.c (1521), numbers.c (642), date.c (614), advanced.c (509), currency.c (409).
- **Role:** "Regional and Language Options" applet (`intl.cpl`) — locale/number/currency/time/date/sort/keyboard-layout/language configuration property sheet.
- **Origin:** ReactOS-native (Eric Kohl, Dmitry Chapyshev). No Wine-hybrid files.
- **Lang:** Plain C, Win32 GUI. No ATL/COM/C++.
- **Completeness:** Full — all 8 dialog page procs implemented, no stubs; real logic (sample-string formatting, registry persistence, kb-layout sync).
- **Key deps / consumers:** user32, comctl32, advapi32, setupapi, shlwapi, shell32, msvcrt, kernel32, ntdll. Heavy GetLocaleInfoW (36), Reg*W (~110 calls), setupapi INF parsing (unattended setup), EnumSystemLocales/GeoID/CodePages. Single export `CPlApplet`. Not on boot/smoke path. No Rust peer.
- **Problem points:**
  - Eight `INT_PTR CALLBACK ...PageProc` property-sheet dialog procs — large WM_*/PSN_* switches over combos/edits/listviews; verbose but mechanical, heavy `SendDlgItemMessageW`/`PROPSHEETPAGE` FFI.
  - Pervasive registry read/write (`HKCU\Control Panel\International`, Keyboard Layout\Preload/Substitutes) + Locale-info round-tripping — raw `LPBYTE`/`DWORD dwSize` patterns that map awkwardly to safe Rust.
  - setupapi INF handling for unattended setup + manual wide-string parsing (raw HeapAlloc/wcscpy); small static `GroupingFormats` table; all stdcall/WINAPI.
- **Difficulty:** Medium — large surface but flat C, no COM, no kernel/unsafe-MM; pure user-mode Win32 FFI.
- **Verdict:** translate priority **Low** — non-boot, non-smoke, zero kernel relevance; high LOC for no OS-correctness impact.
- **rustconv-fit:** poor — GUI/dialog-proc Win32 with massive user32/comctl32 FFI surface, no usermode-applet precedent; output needs heavy hand-rework with no smoke validation.

### sysdm — dll/cpl/sysdm
- **LOC / files:** 7,881 LOC, 12 files, 100% C (0 C++). Largest: environment.c (1951), virtmem.c (894), smbios.c (883), userprofile.c (850), startrec.c (840), hardprof.c (821), general.c (797).
- **Role:** "System" applet (`sysdm.cpl`) — System Properties property sheet (General/Hardware/Advanced + dialogs for env vars, virtual memory, user profiles, startup/recovery, hardware profiles, licence).
- **Origin:** ReactOS-native (w3seek/Ged Murphy/Stanislav Motylkov); no Wine markers. hardware.c has a `#ifdef __REACTOS__` shim.
- **Lang:** C only. One COM consumer (performance.c) uses `IRegTreeOptions` via `COBJMACROS` C-style vtable macros — no ATL/C++.
- **Completeness:** Full, shipping — single export `CPlApplet` (NUM_APPLETS=1). A couple cosmetic FIXMEs; no functional stubs.
- **Key deps / consumers:** comctl32, user32/gdi32/comdlg32/shell32/shlwapi/ole32, advapi32, powrprof, + in-tree `udmihelp` static lib (DMI/SMBIOS). Two NDK touches (`ndk/pstypes.h` for SharedUserData/NtProductType — i386 PAE pagefile math). No boot/smoke relevance. No Rust peer.
- **Problem points:**
  - Pervasive dialog/property-sheet UI: 6+ `INT_PTR CALLBACK` DlgProcs with big WM_INITDIALOG/WM_COMMAND/WM_NOTIFY switches, PROPSHEETHEADER/HPROPSHEETPAGE plumbing, MAKEINTRESOURCE templates — thin logic in thick Win32 glue.
  - ~83 registry calls across 7 files + SID/profile work in userprofile.c (LookupAccountSidW, ConvertSidToStringSidW, token enumeration).
  - stdcall ABI; variadic `ResourceMessageBox` (__cdecl + va_list → StringCchVPrintfW). smbios.c has 6 static data tables (vendor/device normalization) — the one genuinely Rust-friendly chunk. Despite setupapi importlib, no SetupDi/CM_ calls actually made.
- **Difficulty:** High — not algorithmically hard, but almost entirely Win32 GUI message-loop code; weakest fit for mechanical C→Rust, dense unsafe FFI with little idiomatic payoff.
- **Verdict:** translate priority **Low** — usermode leaf, zero boot/smoke impact, no kernel consumers.
- **rustconv-fit:** poor — heavy resource/dialog/property-sheet UI + message-pump switches; only smbios.c's data-table normalization translates cleanly.

### access — dll/cpl/access (1,934 LOC)
Accessibility Options applet (`access.cpl`) — keyboard/sound/display/mouse accessibility property-sheet pages (StickyKeys, FilterKeys, ToggleKeys, SoundSentry, high contrast, MouseKeys). ReactOS-native, pure C (6 files), full (no stubs). Deps user32/gdi32/advapi32/comctl32; not boot/smoke-relevant. Verdict **Low** — standard dialog-proc + SystemParametersInfo/registry FFI applet; rustconv-fit poor.

### console — dll/cpl/console (2,998 LOC)
Console (command-prompt) configuration applet — colors/font/layout/options property-sheet pages, links the in-tree `concfg` static lib. ReactOS-native, pure C (6 files), full. Deps advapi32/user32/gdi32/comctl32/ntdll; registry-backed console settings; not boot/smoke-relevant. Verdict **Low** — dialog UI + GDI font preview + registry; rustconv-fit poor.

### hdwwiz — dll/cpl/hdwwiz (990 LOC)
Add Hardware Wizard (`hdwwiz.cpl`) — single-file wizard for manual device install. ReactOS-native, pure C (1 file), full. Exports `AddHardwareWizard`/`InstallNewDevice` + CPlApplet; deps setupapi/rpcrt4/devmgr — heavy SetupDi/device-install FFI. Not boot/smoke-relevant. Verdict **Low** — wizard UI + deep setupapi device-install surface; rustconv-fit poor.

### hotplug — dll/cpl/hotplug (883 LOC)
"Safely Remove Hardware" applet — enumerates/ejects removable devices (eject.c, enum.c, hotplug.c). ReactOS-native, pure C (3 files), full. Deps setupapi/user32/advapi32 — SetupDi device enumeration + eject IOCTLs. Not boot/smoke-relevant. Verdict **Low** — device-enum/setupapi FFI + tray UI; rustconv-fit poor.

### input — dll/cpl/input (2,881 LOC)
Keyboard/text-input (IME/layout) language settings applet — add/edit/key-sequence/advanced dialogs + locale/layout list management (10 files). **Wine-marker present** (Wine-hybrid lineage) though built as a ReactOS applet. Pure C, full. Deps advapi32/user32/comctl32/setupapi; registry Keyboard Layout work. Not boot/smoke-relevant. Verdict **Low** — multi-dialog UI + layout-registry FFI, plus Wine-origin caveat; rustconv-fit poor.

### joy — dll/cpl/joy (375 LOC)
Game Controllers applet (`joy.cpl`) — joystick configuration. ReactOS-native, pure C (1 file), small/largely thin. Links `uuid`; deps ole32/shell32 (COM joystick interfaces). Not boot/smoke-relevant. Verdict **Low** — small but COM/DirectInput-leaning; rustconv-fit poor.

### liccpa — dll/cpl/liccpa (110 LOC)
License Manager applet — its own header says "does almost nothing." Essentially an empty dialog (`DlgMainProc` with no real handling). ReactOS-native, pure C (1 file), effectively a **stub/placeholder**. Deps advapi32/user32/gdi32. Not boot/smoke-relevant. Verdict **Low** — trivial but pointless to port (no behavior); rustconv-fit poor (nothing to translate).

### main — dll/cpl/main (2,446 LOC)
Mouse + Keyboard settings applet (`main.cpl`) — mouse.c + keyboard.c property-sheet pages (buttons/pointers/motion, repeat-rate/speed). ReactOS-native, pure C (3 files), full. Links `uuid`; deps comctl32/devmgr/comdlg32/shell32. Registry + SystemParametersInfo. Not boot/smoke-relevant. Verdict **Low** — standard dialog + SPI/registry FFI; rustconv-fit poor.

### mmsys — dll/cpl/mmsys (3,826 LOC)
Multimedia/Sounds-and-Audio applet (`mmsys.cpl`) — audio/volume/voice/sounds/speaker-volume property-sheet pages (6 files). ReactOS-native, pure C, full. Exports CPlApplet + `mmseRunOnce{,A,W}`/`ShowMMCPLPropertySheet`/`ShowAudioPropertySheet` + **`DllGetClassObject`/`DllCanUnloadNow` (COM class-object surface)**. Deps winmm/devmgr/setupapi/shell32. Not boot/smoke-relevant. Verdict **Low** — large dialog UI + winmm/COM FFI; rustconv-fit poor.

### ncpa — dll/cpl/ncpa (69 LOC)
Network Connections applet — a thin shim that just ShellExecutes explorer to the Network Connections shell folder GUID. ReactOS-native, pure C (1 file), trivial/complete. Deps shell32/user32. Not boot/smoke-relevant. Verdict **Low** — too small to matter (one ShellExecuteW); rustconv-fit poor (nothing of substance).

### odbccp32 — dll/cpl/odbccp32 (90 LOC)
ODBC Data Source Administrator applet shim — `LoadLibrary("odbccp32.dll")` + `GetProcAddress("ODBCCPlApplet")` and forwards. ReactOS-native, pure C (1 file), thin forwarder (real work is in odbccp32.dll). Target name `odbccp32i`. Deps user32/comctl32. Not boot/smoke-relevant. Verdict **Low** — trivial dynamic-forward shim; rustconv-fit poor.

### openglcfg — dll/cpl/openglcfg (295 LOC)
OpenGL configuration applet — general.c settings page (registry-backed OpenGL options). ReactOS-native, pure C (2 files), small/full. Deps user32/comctl32/advapi32. Not boot/smoke-relevant. Verdict **Low** — small dialog + registry FFI; rustconv-fit poor.

### powercfg — dll/cpl/powercfg (2,429 LOC)
Power Options applet (`powercfg.cpl`) — power schemes/advanced/alarms/hibernate/power-meter pages (6 files). ReactOS-native, pure C, full. Deps **powrprof** (Get/SetActivePwrScheme, power policy), setupapi, shell32, shlwapi. Not boot/smoke-relevant. Verdict **Low** — multi-page dialog UI + powrprof/registry FFI; rustconv-fit poor.

### telephon — dll/cpl/telephon (70 LOC)
Phone & Modem (Telephony) applet shim — `LoadLibrary("tapi32.dll")` + GetProcAddress("internalConfig") forwarder. ReactOS-native, pure C (1 file), thin forwarder. Links `uuid`; deps ole32/shell32. Not boot/smoke-relevant. Verdict **Low** — trivial dynamic-forward shim; rustconv-fit poor.

### timedate — dll/cpl/timedate (2,831 LOC)
Date and Time applet (`timedate.cpl`) — date/time, monthcal, clock, timezone, internet-time (SNTP) pages (6 files). ReactOS-native, pure C, full. Links `tzlib`; deps **w32time/ws2_32/iphlpapi** (internet-time sync), comctl32. Has real logic (timezone tables, SNTP). Not boot/smoke-relevant. Verdict **Low** — dialog UI + timezone/SNTP FFI; the tz/clock logic is the only mildly Rust-friendly part; rustconv-fit poor.

### usrmgr — dll/cpl/usrmgr (3,008 LOC)
Users and Groups manager applet — users/groups/userprops/groupprops/extra/misc (7 files). ReactOS-native, pure C, full — but **links the `wine` static lib** (Wine helper code, hybrid lineage). Deps **netapi32** (NetUser*/NetLocalGroup* account management), advapi32. Not boot/smoke-relevant. Verdict **Low** — multi-dialog UI + netapi32 account FFI + Wine-lib caveat; rustconv-fit poor.

### wined3dcfg — dll/cpl/wined3dcfg (265 LOC)
Wine Direct3D configuration applet — general.c settings page. **Wine-marker present (Wine-origin)**, pure C (2 files), small. Deps user32/comctl32/advapi32 (registry-backed D3D options). Not boot/smoke-relevant. Verdict **Low** — small registry/dialog applet, and Wine-origin (not really a ReactOS-native translation target); rustconv-fit poor.

---

## shellext

Shell extensions: mostly C++/ATL in-proc COM servers (`DllGetClassObject`/`DllRegisterServer` exports + IShellExt/IContextMenu/IShellFolder/IShellPropSheetExt vtables). All translatable as ReactOS-authored, but the C++/ATL-COM ones are **gated on a COM-vtable→Rust pattern** (same dependency as shell32). The handful of plain-C modules are dialog/property-sheet only (no COM vtable) and are the easy wins. None are boot-critical: none load during the kernel→csrss→winlogon→explorer→desktop chain; they activate only on demand (Explorer context menus, Control Panel property pages, namespace browsing).

### acppage — dll/shellext/acppage (748 LOC)
"Compatibility" property-sheet extension (the Compatibility tab on .exe/.lnk Properties). **C++/ATL-COM** (`CLayerUIPropPage` implements IShellPropSheetExt/IShellExtInit). Full implementation (only 2 E_NOTIMPL). Not boot/explorer-startup relevant (on-demand property page). Translate verdict: medium, **COM-gated** (property-sheet vtable).

### cabview — dll/shellext/cabview (1171 LOC)
.cab archive folder shell extension (browse CAB contents in Explorer). **C++/ATL-COM** (IShellFolder/IPersistFolder/extract handlers). Partial — 12 E_NOTIMPL + 2 UNIMPLEMENTED stubs in the IShellFolder surface. Not boot relevant (on-demand namespace handler). Translate verdict: medium-hard, **COM-gated** (IShellFolder vtable).

### cryptext — dll/shellext/cryptext (74 LOC)
Certificate viewer helper; exports `CryptExtOpenCER[W]` (rundll-style entry that calls `CryptUIDlgViewCertificateW`). **Plain C, no COM** — just two exported stdcall functions over crypt32/cryptui. Full (tiny). Not boot relevant. Translate verdict: **easy, NOT COM-gated** — ideal first candidate.

### deskadp — dll/shellext/deskadp (848 LOC)
Display-adapter property page for the Screen Resolution / desktop settings dialog. **Plain C, no COM** (dialog proc + `shxiface.c` adds a property sheet via a function pointer interface, not a COM vtable). Full. Not boot relevant (Control Panel display applet). Translate verdict: **easy/medium, NOT COM-gated** (pure Win32 dialog).

### deskmon — dll/shellext/deskmon (1095 LOC)
Monitor property page (companion to deskadp; its own `shxiface.c`, which differs from deskadp's). **Plain C, no COM** (Win32 dialog + property-sheet glue). Full. Not boot relevant. Translate verdict: **easy/medium, NOT COM-gated**.

### devcpux — dll/shellext/devcpux (135 LOC)
CPU device property-page extension (the processor's Device Manager properties tab); reads CPU info via setupapi/powrprof into a dialog. **Plain C, no COM** (DllMain + `ProcessorDlgProc`). Full (tiny). Not boot relevant. Translate verdict: **easy, NOT COM-gated**.

### fontext — dll/shellext/fontext (1781 LOC)
Fonts folder shell extension (the special Fonts namespace in Explorer/Control Panel). **C++/ATL-COM** (CFontExt IShellFolder, CEnumFonts, CDataObject, CFontBackgroundMenu, font PIDLs, font cache). Partial — 17 E_NOTIMPL + 1 UNIMPLEMENTED across the IShellFolder/enumerator surface. Not boot relevant (on-demand namespace). Translate verdict: hard, **COM-gated** (IShellFolder + IDataObject + IContextMenu vtables).

### mydocs — dll/shellext/mydocs (319 LOC)
"My Documents" desktop-icon drop handler + namespace glue. **C++/ATL-COM** (CMyDocsDropHandler implements IDropTarget/IPersistFile). Partial-to-full (4 E_NOTIMPL, 4 FIXME). Not boot relevant (desktop icon drop target). Translate verdict: small but **COM-gated** (IDropTarget vtable).

### netshell — dll/shellext/netshell (4545 LOC)
Network Connections folder + LAN connect/status UI + network-setup helpers. **C++/ATL-COM** (shfldr_netconnect IShellFolder, connectmanager INetConnectionManager, lanconnect/lanstatus UI, enumlist) plus ~25 `stub` exports in the .spec (HrLaunchConnection, NetSetup*, etc.). Largest module; substantial real code with many stubbed Hr*/NetSetup* entry points. Not boot relevant (Network Connections shell folder). Translate verdict: hard/large, **COM-gated** (IShellFolder + connection-manager COM + many flat stub exports).

### ntobjshex — dll/shellext/ntobjshex (2001 LOC)
NT Object namespace + registry browser shell extension (browse \Device, \BaseNamedObjects, registry as Explorer folders). **C++/ATL-COM** (ntobjfolder/regfolder IShellFolder, ntobjenum, foldercommon). Partial — 15 E_NOTIMPL + 1 UNIMPLEMENTED. Not boot relevant (developer/diagnostic namespace handler). Translate verdict: hard, **COM-gated** (dual IShellFolder implementations).

### sendmail — dll/shellext/sendmail (380 LOC)
"Send To > Mail Recipient" / desktop-link drop handler. **C++/ATL-COM** (CDeskLinkDropHandler implements IDropTarget/IPersistFile). Partial-to-full (4 E_NOTIMPL, 4 FIXME). Not boot relevant (SendTo drop target). Translate verdict: small but **COM-gated** (IDropTarget vtable).

### stobject — dll/shellext/stobject (1671 LOC)
System tray (systray) notification-area object: power/battery, volume, mouse, hotplug status icons. **C++/ATL-COM** (CSysTray hosts power/volume/mouse/hotplug handlers; exposed via the tray's COM interface). Mostly full (1 UNIMPLEMENTED, 3 FIXME, 0 E_NOTIMPL). Explorer-relevant in the sense that the running shell loads it for the notification area, but NOT in the boot/desktop-render critical path tested by smoke. Translate verdict: medium, **COM-gated** (systray COM object) + Win32 message handling.

### zipfldr — dll/shellext/zipfldr (3052 LOC)
Compressed (.zip) Folders shell extension: browse/extract/create zip archives in Explorer, plus the Send-To-Zip and extraction-wizard UI. **C++/ATL-COM** (CZipFolder IShellFolder, CZipExtract/CZipCreator, CExplorerCommand IExplorerCommand, enumerators, zip PIDLs); also exports `RouteTheCallW`. Partial — 21 E_NOTIMPL + 3 UNIMPLEMENTED + 6 FIXME across the IShellFolder surface; bundles zlib usage. Not boot relevant (on-demand zip namespace). Translate verdict: hard/large, **COM-gated** (IShellFolder + IExplorerCommand + zlib interop).

---

## directx_native

> **Scope note:** All modules below are confirmed **ReactOS-native** (ReactX / authored by
> Magnus Olsen, Gregor Brunmar, Johannes Anderwald, Dmitry Chapyshev — *not* Wine). The Wine
> ddraw/d3d9/d3d8 live elsewhere under `dll/directx/wine/`. **Critical build-status finding:**
> per `dll/directx/CMakeLists.txt`, **only `ksuser` is actually built**. d3d8, d3d9, ddraw,
> dsound_new, msdvbnp, msvidctl, bdaplgin are all `#disabled in trunk` (the comments say "see
> wine subfolder" for the d3d/ddraw trio); ksproxy is `Not needed at the moment`. This sharply
> lowers translation priority for everything except ksuser — translating disabled code yields
> no boot/runtime value and can't be smoke-validated.

---

### d3d9 — dll/directx/d3d9
- **LOC / files:** ~6951 C code + ~4938 header LOC; 18 .c files (d3d9_impl, d3d9_device, d3d9_haldevice, d3d9_puredevice, d3d9_caps, format, adapter, swapchain, texture, mipmap, etc.). `d3d9_private.h` alone is 98KB.
- **Role:** ReactOS-native Direct3D 9 runtime (IDirect3D9 / IDirect3DDevice9) talking to the kernel D3D HAL.
- **Origin:** ReactOS-native ("PROJECT: ReactOS ReactX", Gregor Brunmar). **DISABLED in trunk** (Wine d3d9 used instead).
- **Lang:** C, COM-style (hand-rolled vtable structs, `IDirect3D9ToImpl` via FIELD_OFFSET).
- **Completeness:** partial+stubs (~10 files carry `UNIMPLEMENTED`); the impl/device/caps/format core is substantial, pure/HAL device paths are real.
- **Key deps / consumers:** d3d9.dll consumers (games/3D apps); kernel-side peer is the win32k/dxg D3D HAL callbacks (`d3d9_callbacks.c`, `D3DHAL_*` tables). No boot relevance. Not currently built.
- **Problem points:** (1) huge hand-written COM vtables + `WINAPI` thunks → Rust must reproduce exact vtable layout/ABI; (2) D3DHAL callback structs and the giant `d3d9_caps.c`/`format.c` data tables (D3DFORMAT enumeration, caps bitfields) are tedious, error-prone constant lifts; (3) FIELD_OFFSET-based iface↔impl back-casting is exactly the cast/offset hazard class that has bitten this repo.
- **Difficulty:** High
- **Verdict:** Low priority — large, COM-heavy, **not built**, superseded by Wine d3d9. No runtime payoff.
- **rustconv-fit:** poor (COM vtables + massive caps/format data tables).

### ddraw — dll/directx/ddraw
- **LOC / files:** ~9177 C code + ~613 header LOC; 39 .c files organized into Clipper/Color/Ddraw/Gamma/Kernel/Palette/Surface/Videoport/Vtable/d3d/thunks subdirs.
- **Role:** ReactOS-native DirectDraw (IDirectDraw 1–7, surfaces, palettes, clippers) over the GDI/DirectDraw HEL + DDRAW HAL.
- **Origin:** ReactOS-native ("PROJECT: ReactOS kernel", Magnus Olsen). **DISABLED in trunk** (Wine ddraw used).
- **Lang:** C, COM-style (per-version Vtable/*.c files: DirectDraw_Vtable … DirectDraw7_Vtable, DirectDrawSurface{,2,3,4,7}_Vtable, DirectD3D{,2,3,7}_Vtable).
- **Completeness:** partial+stubs — many `*_stubs.c` files (color/gamma/videoport/kernel/clipper/surface/ddraw all have stub modules); core create/surface/displaymode/caps paths exist, Surface has the densest `UNIMPLEMENTED` cluster (~7+4).
- **Key deps / consumers:** older DirectDraw apps; HEL/HAL callbacks (`callbacks_dd_hel.c`, `callbacks_surf_hel.c`) bridge to GDI + the kernel DDRAW driver. Links uuid/dxguid/PSEH. No boot relevance; not built.
- **Problem points:** (1) 13 separate version-specific COM vtables to lay out identically + thunk between versions (`thunks/`); (2) DD HAL/HEL callback tables and `DDSURFACEDESC`/`DDCAPS` data-structure plumbing; (3) PSEH usage → DUMMY_PSEH dead-code translation rules apply; (4) iface↔impl casting throughout.
- **Difficulty:** High
- **Verdict:** Low priority — largest in family, heavily stubbed, **not built**, superseded by Wine. No payoff.
- **rustconv-fit:** poor (many parallel COM vtables + thunk layer + HAL callback structs).

### dsound_new — dll/directx/dsound_new
- **LOC / files:** ~5869 C code + ~275 header LOC; 14 .c files (directsound, primary, secondary, capture, capturebuffer, devicelist, notify, property, misc, enum, regsvr, classfactory, dsound, stubs).
- **Role:** ReactOS-native DirectSound (dsound.dll) — primary/secondary buffers, capture, device enumeration over WinMM/KS.
- **Origin:** ReactOS-native. **DISABLED in trunk** (`#disabled in trunk`; the older C dsound or Wine path is used).
- **Lang:** C, COM-style (IDirectSound8/IDirectSoundBuffer8/IDirectSoundCapture vtables); also a self-registration server (regsvr.c, DllRegisterServer).
- **Completeness:** partial+stubs — `UNIMPLEMENTED` scattered across primary(7)/secondary(5)/directsound(5)/property(3)/misc/capturebuffer; substantial real buffer + device-list code.
- **Key deps / consumers:** audio apps; imports winmm, ole32, setupapi, **ksuser** (so it rides KS device control), advapi32. No boot relevance; not built.
- **Problem points:** (1) COM vtables + class factory + COM self-registration (regsvr.c writes CLSID registry trees — data-heavy); (2) KS property-set IOCTL plumbing via ksuser; (3) real-time double-buffer/notify logic with position math; (4) setupapi device enumeration.
- **Difficulty:** Medium-High
- **Verdict:** Low priority — **not built**; if audio is ever wanted, this is the most self-contained of the disabled set, but still no current payoff.
- **rustconv-fit:** poor (COM + regsvr registry tables + KS IOCTLs).

### ksproxy — dll/directx/ksproxy
- **LOC / files:** ~12116 C++ code + ~206 header LOC; 17 .cpp files. Two giants: `proxy.cpp` (84KB) and `output_pin.cpp` (74KB); `input_pin.cpp` (45KB), `interface.cpp`, `allocator.cpp`, `mediasample.cpp`, etc.
- **Role:** WDM Kernel-Streaming → DirectShow ActiveMovie proxy (ksproxy.ax): wraps KS filters/pins as DirectShow IBaseFilter/IPin, marshals media samples and clock/quality across the user/kernel KS boundary.
- **Origin:** ReactOS-native (Johannes Anderwald, "ReactOS WDM Streaming ActiveMovie Proxy"). **NOT built** (`Not needed at the moment`).
- **Lang:** **C++** with real C++ classes (`class CKsProxy : public IBaseFilter, public IAMovieSetup, ...`) — genuine multiple-inheritance COM, not hand-rolled vtables.
- **Completeness:** partial+stubs (cvpconfig/cvpvbiconfig/basicaudio carry many `UNIMPLEMENTED`, ~13–20 each) but proxy/pin core is large and real.
- **Key deps / consumers:** DirectShow graph (would be loaded by quartz/strmiids); links strmiids + **cppstl**; uses `KsSynchronousDeviceControl` + DeviceIoControl against KS drivers (8 files do IOCTL/KS device control). Kernel-side peer is the win32k/KS (`ks.sys`) streaming stack. No boot relevance.
- **Problem points:** (1) **multiple-inheritance C++ COM** — multiple vtable pointers per object, `this`-adjusting thunks — the single hardest ABI shape for Rust translation; (2) heavy kernel-streaming IOCTL marshalling (`KsSynchronousDeviceControl`, KSPROPERTY/KSEVENT/KSMETHOD blobs); (3) media-sample allocator + clock/quality forwarding with lifetime/refcount complexity; (4) two 70–85KB single files.
- **Difficulty:** Very High
- **Verdict:** Low priority — biggest single translation unit, multiple-inheritance C++ COM, **not built**, no boot value. Avoid until a Rust C++-COM story exists.
- **rustconv-fit:** poor (MI C++ COM + KS IOCTL marshalling; rustconv targets C, not C++ classes).

---

### bdaplgin — dll/directx/bdaplgin (2495 LOC)
ReactOS-native (Johannes Anderwald) "BDA Proxy" — Broadcast Driver Architecture plugin exposing device-control/pin/frequency/LNB/signal-statistics COM interfaces over KS for DVB tuners. **C++** COM, partial (real device-control logic + class factory). **Disabled in trunk.** Verdict: Low — C++ COM, niche TV-tuner, not built; skip.

### d3d8 — dll/directx/d3d8 (48 LOC)
ReactOS-native one-file pure-stub: every export (`Direct3DCreate8`, `ValidatePixelShader`, `ValidateVertexShader`, `D3D8GetSWInfo`) is `UNIMPLEMENTED`/`return 0` plus an empty DllMain. C, all-stub. **Disabled in trunk** (Wine d3d8 used). Verdict: Low/none — translating 48 lines of stubs is busywork with zero value; skip.

### ksuser — dll/directx/ksuser (268 LOC)
ReactOS-native (Magnus Olsen / Dmitry Chapyshev / Johannes Anderwald) "KS USER" — the **only module in this family actually built**. Thin user-mode KS helpers (`KsCreateAllocator`, `KsCreatePin`, `KsCreateClock`, `KsCreateTopologyNode` via `KsiCreateObjectType` → `NtCreateFile`/IOCTL). **C, plain procedural** (no COM), full impl. Verdict: **Medium priority / good rustconv-fit** — small, self-contained, NTAPI-style (NtCreateFile + heap + UNICODE_STRING + DeviceIoControl), and the only one that links into the live build so it can actually be smoke-validated. If anything in directx_native gets translated, start here.

### msdvbnp — dll/directx/msdvbnp (2854 LOC)
ReactOS-native (Johannes Anderwald) "Network Provider for MPEG2 based networks" — DVB BDA network provider (IBDA_NetworkProvider, scanning tuner, IPv4/IPv6/ethernet filters, pins). **C++** COM. Partial. **Disabled in trunk.** Verdict: Low — C++ COM, niche DVB, not built; skip.

### msvidctl — dll/directx/msvidctl (1374 LOC)
ReactOS-native "Video Control" — tuning-space / tune-request COM objects (tuningspace, tuningspace_container, tunerequest, enumtuningspaces) for the TV/DVB stack. **C++** COM. Partial. **Disabled in trunk.** Verdict: Low — C++ COM, niche, not built; skip.
