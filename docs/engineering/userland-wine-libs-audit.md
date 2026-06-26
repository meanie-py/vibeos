# Userland Wine / non-ROS-native Libraries — Translation Audit

**Companion to `userland-translation-audit.md`** (which maps the ReactOS-**native**
client DLLs). This doc maps the **complement**: the Wine-imported / non-native
usermode library surface — everything that is *not* kernel, *not* the native DLLs
already audited, *not* the done spine, and *not* vendored 3rd-party C libs
(`dll/3rdparty/*`). It exists so agents stop re-flagging this low-value tier as
"remaining userland C."

**Last updated:** 2026-06-21. Built from a 14-agent fan-out (one per family); per
module: role, LOC, origin (Wine / ReactOS-modified-Wine / native), completeness
(full / stub / forwarder), problem points, difficulty, translate-verdict.

**Translation: ~0% of this Wine tier (by design).** Methodology: this doc is the
**deliberately-skipped** low-value tier (~1.23M LOC of Wine-imported code — translating
it is re-doing CodeWeavers' work, not *our* OS). The only Wine-tier DLLs with a Rust
crate are small utility ones already absorbed into the done spine: **cabinet** (DONE —
`cabinet-rust`/`cabinet-cargo`, cohab flip) and **version** (DONE — `version-rust`,
cohab flip; the once-"partial" note is now resolved → fully landed). **mlang** is also
DONE (`mlang-cargo`, cargo-driven full-body link). An **mscms-rust** crate exists in
the tree but is **not yet wired into the mscms link** (the mscms cmake has no Rust
ref) — treat as in-progress, not landed. Everything else here remains C and should stay C.

## Methodology / canonical set

Deterministic enumeration: every `.spec`-bearing module under `dll/` + `win32ss/`,
minus the done spine, the 10 native-audited DLLs, and `dll/3rdparty/*`. Wine
classification signal = fraction of `.c` files including a `wine/` header (≥50% =
Wine-lib, 15–49% = mixed) plus the `dll/directx/wine/` path and `media/doc/
WINESYNC.txt`. (`mshtml` is a known false-negative on the header signal — it uses
a precomp header — and is force-classified Wine.)

**Canonical set: 186 modules, ~1.23M LOC.** For scale that's ~5× the entire
ntoskrnl Rust translation — and it is the **low-value tier** (translating Wine =
re-doing CodeWeavers' work, not *our* OS).

| Family | modules | of which "big" (≥5K) |
|---|---:|---:|
| directx (3D + A/V) | 22 | 12 |
| crypto_sec | 20 | 4 |
| net | 20 | 4 |
| multimedia | 22 | 3 |
| ole_com | 15 | 2 |
| text_font_codec | 12 | 5 |
| browser_script | 9 | 7 |
| crt_dbg | 6 | 2 |
| install | 6 | 1 |
| wmi_auto | 7 | 2 |
| comctl | 2 | 2 |
| printing | 6 | 0 |
| misc (rpcrt4/kernelbase/xmllite/cabinet/hhctrl + ~34 small) | 39 | 5 |

---

## Cross-cutting findings (read this first — they change the picture)

**Bottom line: skip essentially the entire tier for translation-value reasons.**
The exceptions below are the only things worth remembering.

### Already done / not built / no-op (remove from any "todo")
- **cabinet — already Rust** (`cabinet-rust`/`cabinet-cargo`, cohab flip done). Not a candidate.
- **version — DONE** (`version-rust`, cohab flip landed: the whole `version.c` body
  GetFileVersionInfo*/VerQueryValue*/VerFindFile*/VerInstallFile* → Rust, cmake links
  `libversion_rust.a`). No longer "partial."
- **mlang — DONE** (`mlang-cargo`, cargo-driven full-body link replacing the win32dll C link).
- **mscms** — `mscms-rust` crate exists but is **not yet wired** into the mscms link
  (mscms cmake has no Rust ref); in-progress, not landed.
- **kernelbase/wine (41K) — NOT BUILT.** `#add_subdirectory(wine)` is commented out
  ("ReactOS isn't ready for Kernelbase.dll"). Dormant dead source — ignore entirely.
- Pure **stubs/forwarders** (nothing to translate): the RAS family (rasapi32 has 63
  stubs, rasdlg/rasman/mprapi), crtdll + msvcrt40 (forward to msvcrt), dpnhpast
  (85-LOC no-op), initpki/mscat32/mssign32/mssip32/sccbase/slbcsp/cryptdll, and a
  long tail of leaf DLLs. Listed per family below; none are real work.

### The one keystone if RPC clients ever go Rust
- **rpcrt4** (`dll/win32/rpcrt4`, 28K) — the RPC runtime under advapi32 / setupapi /
  netapi32 / the service clients. Wine-derived, **Very High** (NDR marshalling
  engine + stubless asm thunks). Not worth translating for its own sake, but it's
  the foundation the native RPC-client translations (advapi32 SCM/LSA, netapi32,
  setupapi cfgmgr) sit on — they currently call its generated `*_c.c` MIDL stubs via
  FFI, which is fine. Only revisit rpcrt4 itself if a full-Rust RPC stack is ever a goal.

### Foundational-but-special: msvcrt
- **msvcrt (44K)** is foundational to *usermode* but **not on the early-boot path**
  (FreeLdr has its own CRT; only host tools link it). It is **orthogonal to the
  existing Rust CRT crates** — `crt0-rust`/`crt-startup-rust`/`crt-dllmain-rust` are
  the mingw startup floor (`.CRT$X*` init array + main→WinMain + default DllMain)
  that link into every PE and *call* msvcrt; they don't overlap the library. Verdict:
  **defer / piecemeal**, never a whole-DLL target — Very High, dominated by shared
  CRT global state (`_iob`/`__pioinfo`/errno/locale) + ~30% MSVC C++ runtime
  (ConcRT/EH/RTTI) fused to per-arch asm.

### "Surprisingly-native" exceptions — belong in the NATIVE audit, not this skip-list
These slipped into the Wine sweep by family but are ReactOS-authored; if any get
translated they're native-target work, not Wine-port work:
- **msv1_0** (NTLM auth package in LSASS), **winscard** (smartcard stubs) — crypto_sec
- **ntlanman** (LanMan WNet provider), **wshtcpip** (WinSock helper directly on the
  TCP/IP transport — netstack-lane-relevant) — net
- **samlib**, **powrprof**, **user32_vista** — misc
- **msdmo** has a ReactOS-native `stubs.c` — directx_av
- **winmm** is partly ReactOS-native — multimedia

### 3rd-party-wrapping (don't re-implement the algorithms)
- **bcrypt / schannel** are thin shims over the vendored **mbedtls** lib.
- **rsaenh** bundles public-domain **TomCrypt/LibTomMath** (pure-math shape, but
  re-implementing crypto is the highest-risk thing in the tree — leave it).

### Wine-but-UI-critical (the only Wine ports a full-Rust *desktop* would ever need)
- **comctl32** (87K), **comdlg32** (20K), **uxtheme** (10K), **usp10** (16K) — Wine,
  but on the explorer/desktop render path. A truly 100%-Rust desktop would eventually
  need them, but they're huge Wine ports with per-control window-proc/message-callback
  + GDI/USER coupling. Far lower ROI than the native targets; revisit only after the
  native client surface (user32/gdi32/…) is done.

---

## Per-family detail

*(Generated per-module: BIG modules get a full mini-scope; TAIL modules get a
2–3 sentence summary. Verdicts default to "skip — Wine, low value" unless flagged.)*


---

## misc

The grab-bag bucket: 5 BIG + 34 TAIL. Overwhelmingly upstream-Wine. Two notable
status facts found during scoping:
- **cabinet is ALREADY translated to Rust** (cargo-driven `cabinet-cargo` +
  `cabinet-rust`, including the hand-ported LZX/QTM/MSZIP engine). Effectively DONE.
- **kernelbase/wine is NOT BUILT** — its parent `CMakeLists.txt` has
  `#add_subdirectory(wine)` with the comment "TODO: ReactOS Isn't ready for
  Kernelbase.dll". The 41K LOC under `kernelbase/wine/` is dormant/dead source,
  not part of any boot artifact. Translating it would be wasted effort.

---

### cabinet — dll/win32/cabinet
- **LOC / files:** 5065 (C), ~7 files (cabinet_main.c, fci.c 58K, fdi.c 98K, stubs.c)
- **Role:** Microsoft Cabinet (.cab) compression/decompression — FCI (create) / FDI (extract); LZX/QTM/MSZIP codecs.
- **Origin:** Wine, ReactOS-modified.
- **Completeness:** **ALREADY TRANSLATED TO RUST** (cabinet-cargo/cabinet-rust on disk; cohab flip done — C kept per never-delete rule but not the link path; uses pure-Rust libz-rs-sys for deflate).
- **Key deps / consumers:** setupapi, advpack, installers. Self-contained codecs.
- **Problem points:** LZX/QTM/MSZIP decompress bit-exactness; FDICopy callback model. (All already solved in Rust.)
- **Difficulty:** N/A (done).
- **Verdict:** **DONE — do not re-translate.** Already Rust; verify it's in the build/smoke, nothing to scope.

### rpcrt4 — dll/win32/rpcrt4
- **LOC / files:** 27671 (C), ~30 .c files. Heavyweight: ndr_marshall.c (7306 LOC), rpc_transport.c (3970), rpc_binding.c, rpc_server.c, rpc_message.c, ndr_stubless.c (84K bytes), ndr_typelib.c.
- **Role:** The RPC runtime — NDR marshalling engine, /Oicf stubless proxy/stub dispatch, RPC bindings, EPM (endpoint mapper), named-pipe/LRPC/TCP transports, context handles, full pointers.
- **Origin:** Wine, with ReactOS additions (`ros_extra.c`, `arm64_rpc_stubs.c`, `thunks.c`/`msvc.S` asm). NOT native.
- **Completeness:** full (real implementation; arm64 delay-import workaround stubs only for the csrss-stack-overflow chain).
- **Key deps / consumers:** **Foundational RPC infra** — every RPC client links it: advapi32 (SCM/eventlog/lsa), setupapi, netapi32, secur32, spoolss, the service control manager. A regression here cascades into winlogon/services boot.
- **Problem points:** (1) NDR marshalling engine — format-string interpreter, pointer/array/union/struct marshalling, alignment, byte-exact wire format; (2) stubless proxy dispatch with raw asm thunks (msvc.S, arm64_rpc_stubs.c) + per-arch calling conventions; (3) transport layer (123K-byte rpc_transport.c) with named pipes / sockets / async; (4) COM/typelib NDR (ndr_typelib.c, ndr_ole.c).
- **Difficulty:** **Very High** (largest real logic in the bucket; asm-heavy ABI surface; foundational so bugs are catastrophic).
- **Verdict:** **Translate-worth: MEDIUM-but-deferred.** Foundational and Wine — the strongest "principled" candidate in misc — but very high risk/effort and asm-laden. Not low-priority *importance*-wise, but a late, careful lane (after the kernel/win32k frontier), with the NDR engine as a self-contained pure-logic sub-target if attacked.

### xmllite — dll/win32/xmllite
- **LOC / files:** 5825 (C), reader.c (3742 LOC) + writer.c + xmllite_main.c + guid.c.
- **Role:** Lightweight pull-model XML reader/writer (IXmlReader/IXmlWriter COM interfaces).
- **Origin:** Wine. NOT native.
- **Completeness:** full.
- **Key deps / consumers:** SxS / manifest parsing, some setup/installer paths. Moderate.
- **Problem points:** (1) hand-rolled XML tokenizer/state machine (encoding detection, entity/CDATA/DTD handling) in reader.c; (2) COM interface plumbing (IUnknown/QueryInterface vtables); (3) UTF-8/16 buffer handling.
- **Difficulty:** Medium-High (mostly self-contained parsing logic + a COM shell; pure-algorithm core is Rust-friendly, COM vtables are the friction).
- **Verdict:** **Translate-worth: LOW-MEDIUM.** Clean parser core makes it a *nicer* candidate than most, but low consumer-criticality. Defer.

### hhctrl.ocx — dll/win32/hhctrl.ocx
- **LOC / files:** 5182 (C), ~12 files (help.c 59K bytes, chm.c, webbrowser.c, content.c, index.c, search.c, stream.c) + IDL typelib.
- **Role:** HTML Help (.chm) ActiveX control — CHM parsing, help-window UI, embedded WebBrowser (Trident) host, TOC/index/search.
- **Origin:** Wine (Jacek Caban/CodeWeavers). NOT native.
- **Completeness:** full (Wine impl), but depends on a working WebBrowser/MSHTML host to actually render — largely non-functional on ReactOS in practice.
- **Key deps / consumers:** itss (CHM storage), comctl32, shdocvw/WebBrowser, ole32/oleaut32. End-user help only — zero boot relevance.
- **Problem points:** (1) heavy COM/OLE (ActiveX control, IObjectWithSite, WebBrowser embedding); (2) CHM ITSS storage access; (3) full Win32 UI (window/treeview/listview).
- **Difficulty:** High (COM + OLE-embedding + UI), low payoff.
- **Verdict:** **Translate-worth: LOW (skip).** Leaf end-user app, COM-heavy, not exercised by boot/smoke.

---

### TAIL

### dll/cpl/appwiz (1261 LOC)
Add/Remove Programs control panel applet. Wine (Jacek Caban/CodeWeavers). Real UI impl. Leaf end-user CPL, no boot relevance. Verdict: skip.

### dll/shellext/netplwiz (248 LOC)
"User Accounts" shell extension. ReactOS-native (Jared Smudde 2018) but uses wine/debug.h. Small, leaf UI shellext. Verdict: skip.

### dll/win32/activeds (279 LOC)
ADSI (Active Directory Service Interfaces) helper. Wine. Thin wrapper/forwarder around adsldp/COM, mostly stubs. Verdict: skip.

### dll/win32/avrt (136 LOC)
Multimedia Class Scheduler (MMCSS) "AvSetMmThreadCharacteristics" API. Wine, near-total stub. Verdict: skip.

### dll/win32/clusapi (136 LOC, 115 spec entries)
Failover-cluster API. Wine. ~135 LOC of body for 115 exports = essentially all stubs/FIXME. Verdict: skip (stub forwarder).

### dll/win32/dwmapi (338 LOC)
Desktop Window Manager API. Wine. Mostly stubs (DwmIsCompositionEnabled=FALSE etc.); no DWM on ReactOS. Verdict: skip.

### dll/win32/faultrep (129 LOC)
Fault Reporting (Dr. Watson / ReportFault). Wine, stub. Verdict: skip.

### dll/win32/inseng (3544 LOC)
Internet Install Engine (INF download/install, ICifFile COM). Wine. Real-ish impl but legacy IE installer infra, dead in practice. COM-heavy. Verdict: skip.

### dll/win32/itircl (72 LOC)
IT Information Retrieval Control Library (CHM full-text index helper). Wine, tiny stub. Verdict: skip.

### dll/win32/kernelbase/wine (41320 LOC)
Wine's kernelbase.dll (file/locale/path/registry/sync/process/security/loader). **NOT BUILT** — `#add_subdirectory(wine)` is commented out ("ReactOS isn't ready for Kernelbase.dll"); only an empty `kernelbase_static` is configured. Dormant/dead source; ReactOS routes these APIs through kernel32 instead. Verdict: **skip — not in any artifact; translating it is wasted effort.** (If kernelbase were ever activated, this would be a huge, foundational, high-difficulty lane — but that's not the current architecture.)

### dll/win32/loadperf (247 LOC)
Performance-counter ini loader (LoadPerfCounterTextStrings). Wine, mostly stub. Verdict: skip.

### dll/win32/mapi32 (4782 LOC)
Simple MAPI (mail) + util functions. Wine. Real util.c/prop.c logic but leaf email API, no boot relevance. Verdict: skip.

### dll/win32/mscms (2681 LOC) 🔄 crate exists, not wired
Color Management System (ICC profiles, WCS). Pure Wine (Hans Leidekker — despite the audit-brief flag, header says "MSCMS … for Wine", not native). Leaf imaging API. **An `mscms-rust` crate exists in the tree but the mscms cmake has no Rust ref yet (not linked).** Verdict: skip / in-progress (low value).

### dll/win32/msimg32 (37 LOC, 5 exports)
GradientFill/TransparentBlt/AlphaBlend forwarders to gdi32. Wine, trivial forwarder. Verdict: skip.

### dll/win32/msisys.ocx (75 LOC, 4 exports)
MSI system OCX. Wine, trivial stub OCX. Verdict: skip.

### dll/win32/mstask (1780 LOC)
Task Scheduler COM service (ITaskScheduler/ITask). Wine. COM impl, leaf service. Verdict: skip.

### dll/win32/nddeapi (346 LOC)
Network DDE API. Wine, stub (NDDE is obsolete). Verdict: skip.

### dll/win32/ntdsapi (255 LOC)
NT Directory Services helper. Wine, mostly stub/thin. Verdict: skip.

### dll/win32/objsel (403 LOC)
Object Picker COM dialog (IDsObjectPicker). Wine. Leaf COM UI. Verdict: skip.

### dll/win32/odbc32 (3126 LOC)
ODBC Driver Manager (proxyodbc.c). Wine. Real proxy logic but leaf data-access API, depends on unixODBC backend absent here. Verdict: skip.

### dll/win32/odbccp32 (1644 LOC)
ODBC installer/config. Wine. Leaf. Verdict: skip.

### dll/win32/pdh (1313 LOC)
Performance Data Helper (perf-counter query API). Wine. Real-ish impl, leaf diagnostics. Verdict: skip.

### dll/win32/pidgen (60 LOC, 7 exports)
Product-ID generation/validation. Wine, tiny stub. Verdict: skip.

### dll/win32/powrprof (1471 LOC)
Power Profile / power-scheme API. **ReactOS-native** (Benjamin Cutler / Dmitry Chapyshev — not Wine; uses wine/debug.h only). Real registry-backed impl. Leaf power API, no boot relevance. Verdict: skip (native, low value).

### dll/win32/query (106 LOC, 46 exports)
Indexing Service / content query. Wine, stub. Verdict: skip.

### dll/win32/regapi (46 LOC, 84 exports)
Registry API helper. Wine/thin, near-all forwarder/stub. Verdict: skip.

### dll/win32/resutils (50 LOC, 71 exports)
Cluster resource utilities. Wine, stub forwarder. Verdict: skip.

### dll/win32/samlib (2368 LOC)
SAM (Security Account Manager) client RPC stubs. **ReactOS-native** (ReactOS Team 2004 — uses wine/debug.h but is native). Real SAM RPC client used by lsass/netapi32/SAM management. Foundational-ish for auth but small and a client of rpcrt4. Verdict: defer; LOW (native, would follow rpcrt4 if ever).

### dll/win32/tapi32 (2148 LOC)
Telephony API client. Wine. Leaf TAPI. Verdict: skip.

### dll/win32/url (128 LOC, 21 exports)
URL invocation helper (InetIsOffline etc.). Wine, stub forwarder to shdocvw/shell. Verdict: skip.

### dll/win32/version (1687 LOC) ✅ DONE
Version-resource API (GetFileVersionInfo/VerQueryValue). Upstream-Wine-derived (Marcus Meissner et al.) but **widely used** (installers, loader, shell). **Fully translated → `version-rust` (1 .rs / ~2.8K LOC); `version.c` cohab-disabled, cmake links `libversion_rust.a`.** Verdict: DONE — not a candidate.

### dll/win32/vssapi (113 LOC, 85 exports)
Volume Shadow Copy API. Wine, stub (85 exports, 113 LOC). Verdict: skip.

### dll/win32/wtsapi32 (826 LOC)
Terminal Services / session API (WTSEnumerateSessions etc.). Wine. Mostly single-session stubs. Verdict: skip.

### win32ss/printing/monitors/localmon/ui (694 LOC)
Local print-port monitor UI dialog (localui.c). Wine (Detlef Riekenberg). Leaf printing UI. Verdict: skip.

### win32ss/user/user32_vista (3879 LOC)
Vista+ user32 APIs (GetDpiForSystem/Window, plus wine/ subdir: input.c/misc.c/sysparams.c/win.c). **ReactOS-native** shim (Carl Bialorucki 2024) over a vendored Wine subset. Small DPI/Vista API surface that win32k/user consumers call. Verdict: LOW (native shim, small, not boot-critical).

---

## crt_dbg

### msvcrt — dll/win32/msvcrt
- **LOC / files:** ~44,130 (C, no C++; ~43,860 in 57 top-level .c) across 113 files incl. headers + per-arch asm (`i386/`, `amd64/` .s). Spec = 1,540 exports.
- **Role:** The Microsoft C runtime library DLL — stdio (`file.c` 5.9K), strings (`string.c`/`wcs.c`/`mbcs.c` ~11K), math (`math.c`/`mathf.c`/`sincos.c` ~3K), locale (`locale.c` 2.3K), heap, time, dir, process/exec, errno, ctype, plus the **MSVC C++ runtime** (`cpp.c`, `cppexcept`, `concurrency.c` 3.8K = ConcRT, `exception_ptr.c`, `undname.c` name demangler 1.8K) and SEH (`except*.c`, `handler4.c`).
- **Origin:** ReactOS-modified-Wine mix. ~33/57 .c reference Wine (`wine/debug.h`, Wine copyrights); the rest are ReactOS-native CRT glue (`crt_main.c`, `crt_winmain.c`, `crt_gccmain.c`, `data.c`, `reactos/` dir). Heavy `__MSVCRT_VERSION__` / `__REACTOS__` ifdef divergence from upstream Wine.
- **Completeness:** Full — production CRT, not stubs. Real stdio, real math, real C++ EH/RTTI/ConcRT.
- **Key deps / consumers:** Imported by ~everything in usermode (299 cmake refs across dll/). **Relationship to existing Rust crt crates: orthogonal, NOT overlapping.** `crt0-rust` / `crt-startup-rust` / `crt-dllmain-rust` are the *mingw msvcrtex startup floor* (the `.CRT$X*` init array, `main`→WinMain forwarders, default DllMain, `__mingw_winmain_*` globals) that every PE links statically — they are the *consumer side* that ultimately calls into this DLL. msvcrt.dll itself is the actual library those startup stubs and all apps call. So translating msvcrt does not subsume or get subsumed by them. **Boot-criticality: not on the early kernel/freeldr boot path** (freeldr uses its own CRT; only `fdebug`/`install` host tools reference it), but it *is* foundational to the usermode userland — many win32 DLLs/apps link it, so a broken msvcrt = broad usermode breakage at/after `shell ready`.
- **Problem points:** (1) **CRT global state** — `_iob`/`MSVCRT__iob` FILE table, `__pioinfo` fd blocks, errno, locale, `_pctype`, onexit tables: shared mutable statics with subtle init order; Rust `static mut` + the captures(none) hazards this repo fights. (2) **C++ EH/RTTI/ConcRT** (`concurrency.c`, `cppexcept`, hand-written per-arch vtable/alias `.s`) — deeply ABI-bound to MSVC layout, essentially untranslatable without keeping the asm. (3) **Math** — `math.c` uses x87/asm aliases (`asm_sqrt`, `x87_asin`) and IEEE corner cases; bit-exact behaviour matters. (4) **stdio/locale/mbcs** — `FILE*` semantics, multibyte codepage tables, printf/scanf format engines (`printf.h`/`scanf.h`) are large and fiddly. (5) **ABI**: i386 `__ASM_USE_THISCALL_WRAPPER`, `__wine_RtlUnwind` shim, stdcall/cdecl/thiscall per export.
- **Difficulty:** Very High.
- **Verdict:** **Real assessment: defer / translate-piecemeal at most, not a whole-DLL target.** msvcrt is foundational but it is the *worst-shaped* Rust target here: enormous, full of shared CRT global state, and ~30% is MSVC C++ runtime fused to hand-written per-arch asm (ConcRT, EH, vtables) that has no Rust equivalent. The clean wins, if any, are the *pure-algorithm* leaves (string/wcs/mbcs ops, ctype, math kernels, undname demangler) factored as rlibs in the project's established pattern — and even those compete with simply reusing Rust's own libcore. Whole-module translation is poor ROI and high regression risk to all of usermode. Recommend: leave as C; cherry-pick algorithm files only if a consumer needs them. Note it is distinct from the already-done crt0/startup Rust crates (those are the startup floor, already translated).

### dbghelp — dll/win32/dbghelp
- **LOC / files:** ~28,168 (C) across 32 files. Spec = 210 exports.
- **Role:** Debug help library — symbol engines: DWARF (`dwarf.c` 3.6K), PDB/CodeView (`msc.c` 3.5K), STABS (`stabs.c`), COFF (`coff.c`); module loaders (`pe_module`, `elf_module`, `macho_module`); `symbol.c`/`type.c` symbol+type stores; `stack.c` stackwalk + per-cpu unwinders (`cpu_{i386,x86_64,arm,arm64}.c`); `minidump.c`; bundled zlib `inflate.c` (1.9K) for compressed debug sections.
- **Origin:** Pure Wine, winesync-tracked (the `dbghelp_ros.diff` records the exact Wine→ReactOS delta, e.g. `DBGHELP_STATIC_LIB` guards). `rosstubs.c` (788) + `compat.c` are the ReactOS shims.
- **Completeness:** Full (real DWARF/PDB/ELF/Mach-O parsers), plus `rosstubs.c` for the bits ReactOS doesn't wire.
- **Key deps / consumers:** Used by debuggers/crash tooling (imagehlp forwards to it; dbgeng builds on it). Not boot-critical; not on the smoke path.
- **Problem points:** (1) Massive **binary-format parsers** (DWARF, PDB/CodeView, STABS, ELF, Mach-O) — exactly the unsafe pointer-walking-over-mmapped-bytes code that is risky to port and where Rust's safety buys little because it stays `unsafe`. (2) Bundled **zlib inflate** — would want to swap for a Rust crate, not translate. (3) Per-arch **unwinders/CONTEXT** ABI. (4) Big internal data model (`storage.c` hash/vector pools, `symt` type graph).
- **Difficulty:** Very High.
- **Verdict:** **Not worth translating (default LOW).** It is a developer/debug-tooling DLL, not runtime-critical, it's clean upstream Wine kept in sync via winesync (so a Rust fork would diverge and lose free upstream fixes), and the bulk is unsafe format-parsing where Rust's value-add is minimal. Leave as C.

### imagehlp — dll/win32/imagehlp (1,705 LOC)
- **LOC / files:** 1,705 (C), 5 files (`imagehlp_main`, `access`, `integrity`, `modify`, `precomp`). Wine-origin (Patrik Stridvall 1998).
- Real (not pure forwarder) but **thin**: most symbol APIs forward to dbghelp; the native part is PE image checksum/integrity/cert (`integrity.c`, `modify.c`, `access.c`) with ~22 STUB/FIXME markers. Image-on-disk PE manipulation — modest unsafe binary munging.
- **Verdict:** Low value, Low–Medium difficulty; depends on dbghelp staying C anyway. Skip / translate only if dbghelp is.

### dbgeng — dll/win32/dbgeng (4,105 LOC)
- **LOC / files:** 4,105 (C), single `dbgeng.c`. Spec = 5 exports. Wine-origin (Volodymyr Shcherbyna 2010), COM-heavy (`COBJMACROS`, IDebug* interfaces).
- Real implementation of the Microsoft Debug Engine COM API (`IDebugClient`/`IDebugControl`/`IDebugSymbols` etc.), built atop dbghelp. Debugger-only, not runtime/boot-critical. The whole thing is one big COM-vtable C file — translating to Rust means reimplementing COM vtable boilerplate by hand.
- **Verdict:** Low value, High difficulty (COM vtable plumbing in 1 file). Skip.

### crtdll — dll/win32/crtdll (324 LOC)
- **LOC / files:** 324 (C), 2 files (`dllmain.c` + precomp). Spec = 528 entries. ReactOS public-domain shim.
- Legacy NT3.x/4 C runtime DLL. **Almost entirely a forwarder to msvcrt** — the spec re-exports msvcrt functions; `dllmain.c` only holds a handful of CRTDLL-specific version globals + DllMain. Not boot-critical.
- **Verdict:** Trivial/Low. Real value is the spec forwarding, not C logic; translate only as a byproduct of msvcrt (which it depends on). Skip standalone.

### msvcrt40 — dll/win32/msvcrt40 (43 LOC)
- **LOC / files:** 43 (C), single empty-ish `msvcrt40.c` (no real bodies). Spec = 1,178 entries. Wine-origin (Louis Lenders 2007).
- **Pure forwarder DLL** — the 1,178-line spec forwards everything to msvcrt/msvcp; the .c file is effectively empty (no DllMain body). Zero translatable logic.
- **Verdict:** Nothing to translate (it's a .spec, not code). Skip.

---

## install

The install family is the Windows Installer / side-by-side / patch-engine cluster. With one exception (msi) these are small Wine ports or pure forwarders; **none are boot-critical** — the boot-relevant SxS/activation-context logic lives in ntdll/kernel32, not in this `sxs.dll`. Default verdict across the whole family is LOW translate-worth.

### msi — dll/win32/msi
- **LOC / files:** 55,015 total (44 .c/.h, ~53,391 in .c), plus 2 Bison grammars (cond.y 884, sql.y 1018) and 2 IDLs (msiserver.idl, winemsi.idl).
- **Role:** The full Windows Installer engine — MSI database, SQL-ish query engine, install action sequencing, UI dialogs, custom-action host, COM automation, RPC server.
- **Origin:** Wine (CodeWeavers — Aric Stewart / Mike McCormack / Hans Leidekker), with ReactOS `#ifdef __REACTOS__` shims. `-D__WINESRC__`, `target_link_libraries(... wine)`.
- **Completeness:** full (real, heavy implementation; a handful of stub spec entries like `MsiConfigureFeatureFromDescriptor*`, but the engine is largely complete).
- **Key deps / consumers:** ole32, oleaut32, cabinet, shell32, shlwapi, rpcrt4, urlmon, wininet, **sxs**, imagehlp, mspatcha, comctl32, version, crypt32, wintrust, apphelp; uses PSEH + `wine_dll_register`. Consumed by msiexec and any MSI-based installer. Not in any boot path.
- **Problem points:** (1) two Bison/yacc grammars (`sql.y`, `cond.y`) — generated C parsers that have no clean Rust auto-translation path; (2) a relational database + SQL query engine (table.c 3530, select/where/insert/update/distinct/where) — large, stateful, pointer-heavy; (3) deep COM (IDispatch automation 2505 LOC, msiserver.idl typelib) + RPC (winemsi.idl client/server) ABI surfaces; (4) action.c (7853) + dialog.c (4576) are enormous procedural state machines tied to comctl32/GDI.
- **Difficulty:** Very High
- **Verdict:** **No / LOW.** Huge Wine port, zero boot relevance, grammar-generated parsers + SQL engine + COM/RPC — worst-case translation ROI in this family.

### advpack — dll/win32/advpack (3,678 LOC)
INF-driven advanced install helper (DelNode, LaunchINFSection, AdvInstallFile, RegisterOCX, RunSetupCommand). Pure Wine (`wine` lib, `set_wine_module_FIXME`), 82 spec entries, real implementation across advpack.c/files.c/install.c/reg.c — thin wrappers over setupapi/ole32/version. Wine, not native, not boot-relevant. Verdict: LOW (defer to C).

### fusion — dll/win32/fusion (3,600 LOC)
.NET Fusion GAC / assembly-cache + assembly-name API (CreateAssemblyCache/Enum/NameObject, GetCachePath). Pure Wine (James Hawkins, CONST_VTABLE COM); spec is largely real but several `stub` entries (CreateHistoryReader, PreBindAssembly, InstallCustomAssembly). Heavy COM (IAssemblyCache/IAssemblyName vtables) and only matters for .NET 1.1/2.0 — irrelevant to ReactOS boot. Verdict: LOW.

### mspatcha — dll/win32/mspatcha (1,524 LOC)
PatchAPI delta-patch applier (ApplyPatchToFile*, GetFilePatchSignature*); main + an LZX decompressor (lzx.c/lzx.h). Wine base (David Hedberg) with substantial ReactOS-native work (Mark Jansen 2018); several Ex variants are stubs. Real LZX bit-stream decompressor is the only interesting part but it's small and a consumer of msi only. Verdict: LOW.

### sxs — dll/win32/sxs (1,535 LOC)
SxS assembly-cache shim DLL: IAssemblyCache install/query + IAssemblyName/SxsLookupClrGuid (cache.c 858, name.c 404, sxs.c 194, guid.c 16). **Despite the family note, this file is pure Wine** (EA Durbin / Hans Leidekker copyrights, COM IAssemblyCache) with only `#ifdef __REACTOS__` glue — it is NOT the native, boot-relevant SxS path. The boot/loader-critical side-by-side + manifest/activation-context logic lives in **ntdll (ldr/sxs) and kernel32 (actctx)**, which this DLL does not implement. So no boot exception applies here. Verdict: LOW.

### updspapi — dll/win32/updspapi (35 LOC)
Update-service setup API. **Pure forwarder** — main.c is 35 lines and the entire .spec forwards every entry to `setupapi.*` (UpdSpCloseFileQueue → setupapi.SetupCloseFileQueue, etc.). Nothing to translate. Verdict: SKIP (forwarder, no body).

---

## comctl

### comctl32 — dll/win32/comctl32
- **LOC / files:** 86,798 C (39 .c files) + ~417 LOC headers (comctl32.h, precomp.h, resource.h). All C, no C++.
- **Per-control breakdown (top):** listview 12,173 · toolbar 7,463 · treeview 6,059 · edit 5,323 · imagelist 4,475 · propsheet 4,020 · rebar 3,865 · tab 3,501 · listbox 3,207 · monthcal 3,057 · tooltips 2,407 · header 2,307 · button 2,244 · comboex 2,191 · combo 2,144 · trackbar 2,088 · commctrl 2,013 (init/glue) · syslink 1,773 · datetime 1,740 · pager 1,584 · taskdialog 1,420 · status 1,363 · updown 1,202 · comctl32undoc 1,184 · dpa 1,030 (+ animate, string, static, progress, dsa, ipaddress, hotkey, flatsb, draglist, nativefont, smoothscroll, theming, theme_scrollbar, stubs). 177 exports.
- **Role:** Windows Common Controls 6.0 — every standard UI widget (list/tree/tab/toolbar/header/etc.), used by virtually every GUI app + the shell.
- **Origin:** ReactOS-modified Wine. Diff header is literally `e:\wine\dlls\comctl32 → e:\reactos\dll\win32\comctl32`; a large 2,507-line `comctl32_ros.diff` shows heavy ReactOS patching (theming, A/W, resource IDs moved to resource.h, ARRAY_SIZE FIXME). Core control logic is Wine.
- **Completeness:** full (one small `stubs.c`; near-complete control set). Production-grade, mature.
- **Key deps / consumers:** imports user32, gdi32, advapi32, usp10, imm32, msvcrt; delay-loads winmm + uxtheme. Consumed by explorer, the shell, comdlg32, and essentially every GUI process. **Boot/smoke-critical:** desktop render (taskbar/explorer) drives common controls — directly on the `shell ready` path.
- **Problem points:**
  1. **Massive per-control window procs** — each file is a giant `WindowProc` switch over WM_* + control-specific messages, all stdcall `CALLBACK` ABI; pervasive `GetWindowLongPtrW/SetWindowLongPtrW(hwnd, 0/GWLP_*)` to stash the per-window `INFO*` struct (raw ptr ↔ window-word punning, hostile to Rust ownership).
  2. **Deep GDI/USER coupling + owner-draw/subclassing** — DrawFrameControl, custom-draw NM_CUSTOMDRAW notifications, edit-label subclassing (listview EditLblWndProc swaps GWLP_WNDPROC), CallWindowProcA/W A↔W thunking everywhere.
  3. **Notification callbacks (WM_NOTIFY/NMHDR) + global state** (COMCTL32_hModule, brushes, theming) and the sheer surface (177 exports, 12K-line listview) — translation is many independent large grinds, not one unit.
- **Difficulty:** Very High (size + window-proc/window-word idioms + GDI coupling).
- **Verdict:** **Not worth translating now — defer.** It is UI-critical and on the boot path, but it is mature, stable Wine C that already works, with no kernel-memory-safety payoff (pure usermode UI). 86K LOC of callback/owner-draw/window-word code is the single largest and most translation-hostile target in the family; the project's safety wins are in the kernel, not here. If ever attempted, do it one control file at a time (start small: progress/updown/static), never as a unit.

### comdlg32 — dll/win32/comdlg32
- **LOC / files:** 19,838 (19,723 C across 9 .c + 115 LOC autocomp.cpp). Files: filedlg 4,898 · itemdlg 4,674 · printdlg 4,305 · colordlg 1,418 · fontdlg 1,312 · filedlg31 1,191 · filedlgbrowser 1,062 · finddlg 577 · cdlg32 286 (init) · autocomp.cpp 115. 25 exports.
- **Role:** Common Dialogs — GetOpenFileName/GetSaveFileName, IFileDialog (Vista-style), ChooseColor, ChooseFont, PrintDlg, Find/Replace.
- **Origin:** ReactOS-modified Wine (diff `e:\wine\dlls\comdlg32 → e:\reactos\...`), but only a tiny 54-line `comdlg32_ros.diff` — far closer to upstream Wine than comctl32.
- **Completeness:** full (Wine-complete incl. the modern IFileDialog COM object in itemdlg.c).
- **Key deps / consumers:** imports shell32, shlwapi, **comctl32**, winspool, user32, gdi32, advapi32; delay-loads ole32. Heavy COM/shell coupling (filedlg: ~68 IShellFolder/IShellBrowser/IShellView hits; itemdlg: ~387 IFileDialog/IShellItem/QueryInterface/vtbl hits). **Not on the minimal boot path** — dialogs are invoked on user action (open/save), not at desktop bring-up; lower smoke relevance than comctl32.
- **Problem points:**
  1. **Dialog-template window procs + WM_INITDIALOG message handling** — same stdcall CALLBACK dialog-proc pattern, owner-draw color/font preview.
  2. **Dense COM object implementation** (itemdlg's IFileDialog is a full vtbl/QueryInterface/AddRef/Release object graph) layered on shell32's IShellFolder/IShellView browsing — translating COM vtbls + aggregation to Rust is its own hard problem.
  3. **shell32 + winspool integration** (filedlgbrowser drives the shell view; printdlg talks to the spooler) — a long dependency chain that must exist first.
- **Difficulty:** High (COM-heavy, shell-coupled), smaller than comctl32 but not easy.
- **Verdict:** **Not worth translating now — defer, lower priority than even comctl32.** It works as Wine C, is off the boot/smoke critical path, and its value (COM file dialogs) gives zero kernel-safety benefit while requiring shell32+ole32 COM infrastructure in Rust first. Leave as C indefinitely.

---

## ole_com

OLE/COM core + automation + ATL + accessibility. Every module in this family is
**Wine-derived** (each `CMakeLists.txt` links the `wine` compat lib; most carry
`set_wine_module_FIXME(... CORE-5743: No ARRAY_SIZE / CONST_VTABLE)`). These are the
canonical Wine COM stack imported almost verbatim into ReactOS — heavy on
`CONST_VTABLE` IUnknown/IDispatch dispatch tables, NDR/usrmarshal proxy code, typelib
parsing, and apartment/threading global state. This is the lowest-value translation
family in the audit: the C is mature, portability-engineered, idiom-saturated Wine, and
none of it is on the kernel boot path. **Family verdict: do not translate** (with the
single exception flagged below being merely "least-bad", not "worth it").

---

### ole32 — dll/win32/ole32
- **LOC / files:** 55,164 LOC, 37 C files (35 carry Wine debug-channel headers)
- **Role:** Core COM runtime — CoCreateInstance/apartments, structured storage
  (`storage32.c` 10.7K), marshalling/RPC/stub-manager, monikers (file/item/class/
  composite/anti/pointer), clipboard, data cache, default handler, GIT.
- **Origin:** Wine (pure; `target_link_libraries(ole32 wine uuid ...)`, `WINE_REGISTER_DLL`)
- **Completeness:** full (real implementation, not stubs)
- **Key deps / consumers:** COM infrastructure — consumed by virtually every usermode
  COM client (shell32, explorer, oleaut32, propsys, etc.). Not kernel-boot-critical but
  desktop-functionality-critical. Links `uuid`, `wine`, `PSEH_LIB`.
- **Problem points:**
  1. Pervasive `CONST_VTABLE` IUnknown/IMoniker/IStorage/IDataObject vtables — Rust has
     no native COM vtable model; every interface needs hand-built `#[repr(C)]` vtable
     structs + thunk glue.
  2. Marshalling: `marshal.c`/`usrmarshal.c`/`rpc.c`/`stubmanager.c`/`ftmarshal.c` are
     NDR/proxy-stub + cross-apartment RPC — intricate ABI/pointer-rewriting code.
  3. Apartment & threading global state (STA/MTA, GIT, message pumping) — shared mutable
     globals that resist safe-Rust ownership modelling.
  4. `storage32.c` (10.7K) compound-file format with on-disk layout invariants.
- **Difficulty:** Very High
- **Verdict:** No. Core Wine COM, enormous, vtable/marshalling-saturated, zero boot value.

### oleaut32 — dll/win32/oleaut32
- **LOC / files:** 41,610 LOC, 14 C files (all Wine debug-channel)
- **Role:** OLE Automation — typelib (`typelib.c` 11.6K), VARIANT engine
  (`vartype.c` 8K + `variant.c` 5.9K), SAFEARRAY, IDispatch, connection points,
  OLE font/picture, var formatting, recordinfo, usrmarshal.
- **Origin:** Wine (pure; `target_link_libraries(oleaut32 wine wineldr uuid ...)`)
- **Completeness:** full
- **Key deps / consumers:** Automation clients/scripting (VBScript/JScript hosts, MSI,
  controls). Desktop-relevant, not boot-critical. Links `wine`, `wineldr`, `uuid`.
- **Problem points:**
  1. `typelib.c` — binary TLB/MSFT/SLTG typelib parsing with format-specific structs;
     huge and pointer-heavy.
  2. VARIANT/SAFEARRAY: `vartype.c`+`variant.c`+`safearray.c` ~15.7K LOC of tagged-union
     coercion across dozens of VT_* types + locale-aware conversions — a combinatorial
     coercion matrix.
  3. IDispatch + connection points (`dispatch.c`/`connpt.c`) — late-binding vtable +
     event sink lists.
  4. `usrmarshal.c` NDR user-marshalling; `msvc.S` hand asm for thunks (ABI per-arch).
- **Difficulty:** Very High
- **Verdict:** No. Core Wine automation; VARIANT/typelib are the worst-fit-for-Rust code
  in the family.

---

### actxprxy — dll/win32/actxprxy (259 LOC)
Wine ActiveX proxy/stub DLL — almost entirely IDL-generated (12 `.idl` + `.rgs` pairs:
comcat, docobj, hlink, objsafe, ocmm, servprov, shobjidl, urlhist, etc.); the only hand
C is `usrmarshal.c` (259 LOC). Pure Wine, real (generated) marshalling code. Verdict:
**No** — the bulk is MIDL-generated proxy code, not hand-translatable source; the C stays.

### atl — dll/win32/atl (3,844 LOC)
Wine ATL (Active Template Library) runtime: `atl.c`/`atl30.c`/`atl_ax.c` (ActiveX host) +
`registrar.c`. **Written in C, not C++** (Wine's ATL-as-C-DLL), with `atl_ros.diff`
showing local ReactOS modifications → ReactOS-modified-Wine. Real implementation, COM
vtable + registrar/window-host glue. Verdict: **No** — modest size but vtable/registrar
COM glue with ReactOS-local patches; low value, not boot-critical.

### atl80 — dll/win32/atl80 (69 LOC)
Tiny Wine ATL 8.0 shim (`atl80.c`) forwarding to atl runtime; carries the
`set_wine_module_FIXME`. Essentially a thin version-specific forwarder. Verdict: **No**
(trivial Wine shim).

### hlink — dll/win32/hlink (2,499 LOC)
Wine hyperlink (IHlink) implementation: `hlink_main.c`, `link.c`, `browse_ctx.c`,
`extserv.c` (+ `hlink_classes.idl`). Real Wine COM-object implementation (IHlink/
IHlinkBrowseContext vtables), not a stub. Pure Wine. Verdict: **No** — small but pure COM
vtable code, no boot value.

### msimtf — dll/win32/msimtf (1,082 LOC)
Wine Active IMM / Text Services proxy (`activeimmapp.c`, `main.c`) with `msimtf_ros.diff`
→ ReactOS-modified-Wine. Real IActiveIMMApp COM object. Verdict: **No** (small Wine IME
COM glue, niche).

### ole32 — see BIG above

### oleacc — dll/win32/oleacc (1,992 LOC)
Wine accessibility (MSAA / IAccessible): `client.c`, `window.c`, `main.c`,
`propservice.c` (+ `oleacc_classes.idl`). Real Wine implementation of default
IAccessible proxies. Pure Wine. Verdict: **No** — small COM accessibility glue, not boot
or core-COM critical.

### oleaut32 — see BIG above

### olecli32 — dll/win32/olecli32 (192 LOC)
Wine OLE-client 16→32 compat stub (`olecli_main.c`); ~10 FIXME/stub markers in 192 LOC —
mostly-stub legacy OLE 1.0 client API. Verdict: **No** (legacy stub, nothing to gain).

### oledlg — dll/win32/oledlg (1,803 LOC)
Wine OLE common dialogs (`insobjdlg.c` insert-object, `pastespl.c` paste-special,
`oledlg_main.c`). Real dialog implementations (UI/resource-driven). Pure Wine. Verdict:
**No** — UI dialog code, low value.

### olepro32 — dll/win32/olepro32 (66 LOC)
Wine OLE property-frame forwarder: `olepro32stubs.c` is mostly stubs
(`DllRegisterServer`/`DllUnregisterServer` return `S_OK` with `FIXME("stub")`); the real
property-frame APIs forward to oleaut32 (`olepro.idl`). **Mostly stub/forwarder.**
Verdict: **No** (forwarder; nothing to translate).

### olesvr32 — dll/win32/olesvr32 (220 LOC)
Wine OLE-server 16→32 compat stub (`olesvr_main.c`); ~9 FIXME/stub markers — legacy OLE
1.0 server API, mostly stub. Verdict: **No** (legacy stub).

### olethk32 — dll/win32/olethk32 (39 LOC)
Wine 16/32-bit OLE thunk DLL (`main.c`) — essentially just `DllMain`/`ThkManager`
skeleton, 39 LOC. Pure Wine, trivial. Verdict: **No** (trivial thunk skeleton).

### propsys — dll/win32/propsys (2,204 LOC)
Wine property system: `propvar.c` (PROPVARIANT helpers), `propstore.c`
(IPropertyStore), `propsys_main.c` (+ `propsys_classes.idl`). Real Wine implementation;
PROPVARIANT is a VARIANT-like tagged union (smaller-scale oleaut32 coercion problem).
Pure Wine. Verdict: **No** — modest size but PROPVARIANT coercion + COM vtables, low
value.

### xolehlp — dll/win32/xolehlp (712 LOC)
Wine MS-DTC distributed-transaction helper (`xolehlp.c`); ~28 FIXME/stub markers in
712 LOC — DtcGetTransactionManager etc. largely stubbed. Mostly stub/forwarder. Pure
Wine. Verdict: **No** (stub transaction helper, no use here).

---

## browser_script

Family of Wine-derived browser/script DLLs (Trident/HTML, JS/VBScript engines, XML, IE frame, URL moniker). Almost all are large, near-verbatim Wine ports (CodeWeavers/Jacek Caban copyrights, `wine/debug.h`, `WINE_DEFAULT_DEBUG_CHANNEL`). They are dense COM/IDispatch vtable + OLE-automation code with zero kernel/boot relevance — uniformly LOW Rust-translation value. The one structural exception is **shdocvw**, which is a thin Wine forwarder shell wrapping ~4K LOC of *native ReactOS* C++ ATL Explorer-bands. The smoke desktop does not require any of these to boot (IE/Trident are launched on demand, not in the SMSS→explorer chain).

### ieframe — dll/win32/ieframe
- **LOC / files:** 13,206 LOC, 19 files (all C)
- **Role:** Internet Explorer browser frame / WebBrowser OLE control host (DocHost, frame window, navigation, shell browser integration).
- **Origin:** Wine (CodeWeavers copyrights, `wine/debug.h` in all 19 files).
- **Completeness:** Full implementation, no `.spec` stubs. Exports are just the COM registration quartet (`DllGetClassObject`/`DllCanUnloadNow`/`Register`/`Unregister`) + `IEWinMain` (`-noname` 101); everything else is COM CLSID-dispatched internally.
- **Key deps / consumers:** mshtml, shdocvw, urlmon, ole32/oleaut32/shell32. Consumed by `base/shell/explorer` (rshell.cpp) and iexplore.exe — IE-launch-time, not boot-time.
- **Problem points:** Heavy COM aggregation (DocHost ↔ WebBrowser ↔ shellbrowser vtable webs); OLE in-place activation / `IOleObject`/`IOleInPlaceSite` callbacks; window-message + event sink plumbing; deep client-site reentrancy.
- **Difficulty:** High
- **Verdict:** LOW. Pure Wine COM host; no boot/kernel value; translation cost dwarfs benefit.

### jscript — dll/win32/jscript
- **LOC / files:** 28,123 LOC, 25 files (all C; has `lang/`)
- **Role:** JScript (ECMAScript) engine — lexer, compiler, bytecode interpreter (`engine.c`), and the full builtin object library (array/date/math/string/regexp/json/...).
- **Origin:** Wine (Jacek Caban / CodeWeavers).
- **Completeness:** Full engine, no `.spec` stubs (only COM-registration exports).
- **Key deps / consumers:** oleaut32 (VARIANT/BSTR), the IActiveScript host protocol; consumed by mshtml and any IActiveScript host.
- **Problem points:** Hand-written lexer + compiler producing a bytecode VM (stack machine in engine.c) — exactly the kind of stateful, goto/jump-table-heavy code that translates badly; `IDispatchEx`/`IActiveScript` vtables; GC-ish reference management; pervasive VARIANT juggling.
- **Difficulty:** Very High
- **Verdict:** LOW. A scripting-language VM is the worst-ROI translation target in the set; Wine, no boot relevance.

### mshtml — dll/win32/mshtml
- **LOC / files:** 70,936 LOC, 73 files (all C) — largest module in the family.
- **Role:** Trident / the HTML document engine (DOM tree, element/style objects, layout host, editor, Gecko/nsapi embedding shim via `nsiface.idl`).
- **Origin:** Wine. (Note: source files lack the `wine/debug.h` grep hit because they include it indirectly via `mshtml_private.h`; CMake sets `COM_NO_WINDOWS_H` and the code is unmistakably Wine Trident.)
- **Completeness:** Full, with a handful of `.spec` stubs (5: e.g. `CreateHTMLPropertyPage`, `DllEnumClassObjects`). Biggest files: htmldoc.c (4878), htmlelem.c (4853), nsio.c (4096), htmlstyle.c (3245), htmlwindow.c (3109).
- **Key deps / consumers:** urlmon, jscript/vbscript (script-engine host), oleaut32, an embedded Gecko/nsapi interface layer. Consumed by ieframe/iexplore.
- **Problem points:** Enormous COM/IDispatchEx vtable surface (every DOM node is a COM object); the nsapi/Gecko embedding bridge (`nsiface.idl`, nsembed/nsio) is an FFI-over-FFI nightmare; DOM tree global/mutable state; OLE document/editor reentrancy.
- **Difficulty:** Very High
- **Verdict:** LOW. By far the costliest target; Wine Trident, no kernel/boot value. Do not translate.

### msxml3 — dll/win32/msxml3
- **LOC / files:** 34,879 LOC, 33 files (all C; `res/`)
- **Role:** MSXML 3.0 — XML DOM/SAX/XSLT/XPath document services (DOMDocument, schema, httprequest), a COM facade over libxml2.
- **Origin:** Wine (Mike McCormack / Adam Martinson, CodeWeavers).
- **Completeness:** Full COM surface; `.spec` has 7 `stub MSXML3_N` ordinal placeholders (registration only). 22 of 33 files call into libxml2 (`LIBXML_STATIC`, links 3rdparty libxml + libwin-iconv).
- **Key deps / consumers:** **libxml2** (static), oleaut32, the msxml typelib (`msxml3_v1.idl`). Consumed by IE/mshtml and any MSXML automation client.
- **Problem points:** It is a thin-ish COM/IDispatch wrapper but every method threads through libxml2 C structs (`xmlNode`/`xmlDoc`) — translating the wrapper without translating libxml2 yields a Rust shell still doing raw libxml FFI; VARIANT/BSTR marshalling; XSLT/XPath callback plumbing.
- **Difficulty:** High
- **Verdict:** LOW. Wine COM-over-libxml2; no boot value. The libxml2 dependency makes a "pure Rust" port impossible without also porting libxml2.

### shdocvw — dll/win32/shdocvw  ⚠️ EXCEPTION (hybrid)
- **LOC / files:** 5,089 LOC, 9 files = **2 C (Wine)** + **7 C++ (native ReactOS)**.
- **Role:** Legacy shdocvw forwarder DLL. Today mostly a compatibility shim that forwards to ieframe, **plus** the native ReactOS Explorer namespace **bands** (Folders/Favorites/History tree-view bands) implemented in C++ ATL.
- **Origin:** **Hybrid.** shdocvw_main.c / shlinstobj.c are Wine; the C++ (CExplorerBand.cpp, CFavBand.cpp, CNSCBand.cpp, mrulist/objects/utility/winlist, ~4,035 LOC) is **native ReactOS** (Sylvain Deverre, Katayama Hirofumi MZ, Whindmar Saksit — 2016-2025, ATL-based).
- **Completeness:** `.spec` is mostly **stub/forwarder** — 91 stub ordinals out of 128 (37 live). Many marked `# FIXME: Inspect`. The C++ bands are real, maintained implementations.
- **Key deps / consumers:** ieframe (forward target), shell32/browseui, ATL. **Boot-relevant coupling:** consumed by `base/shell/explorer` (the bands appear in the Explorer sidebar) — but not on the smoke desktop critical path.
- **Problem points:** The C++ side is ATL COM (CComObject/templated vtables, `IDeskBand`/`IDockingWindow`/`INamespaceTreeControl` callbacks) — ATL has no clean Rust analogue; the Wine side is pure forwarder glue.
- **Difficulty:** Medium (small) / High (ATL idioms)
- **Verdict:** LOW overall, but **flag the split**: do NOT lump the native C++ bands in with "Wine cruft." If anything here is ever revisited it's the native ATL bands, and even those are low-priority (Explorer cosmetic UI, not boot). The Wine forwarder half is near-worthless to translate.

### urlmon — dll/win32/urlmon
- **LOC / files:** 22,922 LOC, 21 files (all C)
- **Role:** URL moniker / asynchronous pluggable protocols — `IMoniker`/`IBindCtx` URL binding, protocol handlers (http/ftp/file/mk), MIME sniffing, security/zone manager.
- **Origin:** Wine.
- **Completeness:** Full; no `.spec` stubs; 73 named exports (rich public API: BindAsyncMoniker, CoGetClassObjectFromURL, CoInternetCombineUrl, zone APIs, etc.) + RPC proxy (`urlmon_urlmon.idl`, PROXY_DELEGATION).
- **Key deps / consumers:** wininet, ole32/oleaut32, RPC proxy stubs. Consumed by mshtml/ieframe and any pluggable-protocol client.
- **Problem points:** Async bind state machines (`IBindStatusCallback` callback storms, threaded completion); pluggable-protocol COM vtables; RPC proxy/delegation marshalling; URL-parsing security edge cases; pervasive global protocol/zone registries.
- **Difficulty:** High
- **Verdict:** LOW. Wine async-COM networking glue; no boot value.

### vbscript — dll/win32/vbscript
- **LOC / files:** 15,118 LOC, 10 files (all C) + a **bison** grammar (`parser.y`).
- **Role:** VBScript engine — bison parser, compiler, interpreter (`interp.c`), builtin global functions, and a regexp engine.
- **Origin:** Wine.
- **Completeness:** Full engine; no `.spec` stubs (COM-registration exports only).
- **Key deps / consumers:** oleaut32, IActiveScript host protocol; consumed by mshtml / WSH-style hosts.
- **Problem points:** **Bison-generated parser** (`parser.tab.c`) — generated LALR tables that won't translate meaningfully; the bytecode interpreter (interp.c) and `IDispatch`-based VBScript object dispatch (vbdisp.c, 116 IDispatch refs); VARIANT-everywhere; a second regexp VM.
- **Difficulty:** Very High
- **Verdict:** LOW. Another script-language VM (with a bison front-end); worst-ROI class. Wine, no boot relevance.

---
### TAIL modules

### itss — dll/win32/itss (4,762 LOC)
ITSS / CHM (compiled HTML help) archive provider: an `IStorage`/`IStream` COM facade over the ITSS container format, including an **LZX decompressor** (lzx.c) and the public-domain chm_lib.c. Real, full implementation — mixed-origin (Wine COM shell + Jed Wing's chm_lib + LZX codec); 1 `.spec` stub (`DllInstall`). Verdict: LOW — niche help-file codec, no boot relevance; the LZX/chm core is the only algorithmically-interesting (but isolated, low-priority) part.

### jsproxy — dll/win32/jsproxy (629 LOC)
Tiny WPAD auto-proxy (PAC) DLL — a single `main.c` exposing `InternetInitialize/GetProxyInfo` by hosting jscript to evaluate the proxy auto-config script. Mostly real (3 live exports) with 3 `.spec` stubs (the `*Ex` variants); CMake even flags it `set_wine_module_FIXME` (CORE-5743). Wine. Verdict: LOW — trivially small but pure Wine networking glue with no boot value; not worth a dedicated port.

---

## wmi_auto

All seven modules are Wine COM-automation ports (every CMakeLists links `wine` + `uuid`/`oldnames`, and 6/7 carry `set_wine_module_FIXME(... CORE-5743: No ARRAY_SIZE and CONST_VTABLE)`). They are IDispatch/WBEM/COM-server DLLs — heavy on vtables, BSTR/VARIANT/SAFEARRAY marshalling, and typelib registration. Across the board this is **low Rust value**: idiomatic Rust gains nothing on COM vtable plumbing, and the ABI/CONST_VTABLE/SAFEARRAY surface is exactly where a C→Rust port is most error-prone. Default verdict LOW for all; the only non-trivial *algorithmic* core is wbemprox's WQL parser + class tables (still LOW-to-translate because it is bison-generated + giant static data).

### scrrun — dll/win32/scrrun
- **LOC / files:** 5,239 (C), 5 files (filesystem.c 4005, dictionary.c 915, scrrun.c 304, guid.c, +precomp/headers). Generates `scrrun_stubs.c`.
- **Role:** Microsoft Scripting Runtime — FileSystemObject (drives/folders/files/text streams) + Scripting.Dictionary, for VBScript/JScript hosts.
- **Origin:** Wine (CodeWeavers), ReactOS-modified (`__ROS_LONG64__`, `oldnames`).
- **Completeness:** full+partial — filesystem.c is a real implementation (19 Win32 file-op calls: CreateFileW/FindFirstFile/CopyFile/DeleteFile); ~80 stub/NOTIMPL and 53 FIXME mark unimplemented IDispatch members and edge methods.
- **Key deps / consumers:** oleaut32, version, advapi32; consumed by script hosts (cscript/wscript, IE/HTA). Ships typelib (scrrun.tlb).
- **Problem points:** (1) large IDispatch surface — FSO + Dictionary + Drive/Folder/File/TextStream each a full dual-interface vtable; (2) VARIANT/BSTR marshalling for Dictionary keys/values + enumerators (IEnumVARIANT); (3) generated `scrrun_stubs.c` forwarders must keep exact stdcall decoration/ABI.
- **Difficulty:** High (volume + COM dual interfaces).
- **Verdict:** LOW. Big but pure COM-automation; no kernel value, high ABI risk.

### wbemprox — dll/win32/wbemprox
- **LOC / files:** 10,612 total (10,278 C + 831-line wql.y bison grammar), 13 .c + wql.y. Dominated by builtin.c (4,612) and query.c (1,439), class.c (1,050), services.c (945).
- **Role:** WBEM provider — the actual WMI backend behind `\\root\cimv2`: enumerates built-in CIM classes (Win32_Process, _OperatingSystem, _NetworkAdapter, etc.) and runs WQL `SELECT` queries against them.
- **Origin:** Wine (CodeWeavers, Hans Leidekker), ReactOS-modified (forces `_WIN32_WINNT=0x600`, `__WINESRC__`, drops d3d10 dep).
- **Completeness:** full — the most genuinely-implemented module here (only ~20 stub markers vs 76 FIXME). builtin.c is a huge static table of CIM class schemas + property-fill callbacks pulling from iphlpapi/setupapi/winspool/ws2_32/winsvc; wql.y is a working bison SQL-subset parser → view/query engine.
- **Key deps / consumers:** iphlpapi, ws2_32, setupapi, winspool, advapi32, rpcrt4; consumed by wbemdisp, wmic, and any WMI client. Pairs with wmiutils (path parsing) + wbemdisp (scripting facade).
- **Problem points:** (1) WQL parser is bison-generated (`wql.tab.c`) — a Rust port means hand-porting the grammar or keeping the C/bison file; (2) builtin.c property callbacks are deeply tied to many Win32 enumeration APIs (FFI-heavy, low Rust benefit); (3) IWbemServices/IWbemClassObject/IEnumWbemClassObject vtables + VARIANT/SAFEARRAY result marshalling; (4) per-table allocation/lifetime + global class registry.
- **Difficulty:** Very High (largest, parser + schema tables + broad Win32 FFI).
- **Verdict:** LOW. Despite being the most "real" module, it is COM + bison + Win32-FFI glue — the worst translation-ROI shape. Leave in C.

### qmgr — dll/win32/qmgr (2,810 LOC) — TAIL
BITS (Background Intelligent Transfer Service) — the COM service behind Windows Update downloads (IBackgroundCopyManager/Job/File), built around winhttp + ole32. Real but partial Wine port (Google-authored, 2007-08; 40 FIXME / 55 stub markers — many job/notification methods unimplemented). Verdict: **LOW** — COM service + winhttp glue, no kernel value.

### wbemdisp — dll/win32/wbemdisp (3,045 LOC) — TAIL
WBEM scripting facade — the `WbemScripting.SWbemLocator`/`SWbemServices`/`SWbemObject` IDispatch automation layer that script hosts use, thin wrapper that forwards to wbemprox. Wine port (CodeWeavers 2013); heavily stub-ish (93 FIXME / 79 stub) — mostly vtable + VARIANT marshalling. Verdict: **LOW** — pure IDispatch-over-WMI shim.

### wmiutils — dll/win32/wmiutils (1,613 LOC) — TAIL
WMI helper objects — WbemDefPath (WMI object-path parser/formatter, IWbemPath) + WbemStatusCode strings; path.c is a real parser (46 WBEM_E_/FIXME branches). Wine port. Smallest of the family and the most self-contained (a string-path parser), but still a COM-object DLL. Verdict: **LOW** — small, but COM-wrapped string parsing, not kernel-relevant.

### wshom.ocx — dll/win32/wshom.ocx (2,128 LOC) — TAIL
Windows Script Host Object Model — `WScript.Shell`/`Network`/`Shell` automation (Run/Exec, env vars, special folders, registry read/write, shortcuts) for VBScript/JScript. Wine port; real-ish core (shell.c calls ShellExecute/CreateProcess/RegOpenKey) but 35 FIXME / 49 stub. Verdict: **LOW** — COM-automation over Win32 shell/registry APIs.

### wuapi — dll/win32/wuapi (1,754 LOC) — TAIL
Windows Update Agent API — IUpdateSession/Searcher/Downloader/Installer/SystemInformation COM objects. Mostly **stub/forwarder**: nearly every interface method returns E_NOTIMPL (95 FIXME, 82 stub; installer.c 25× E_NOTIMPL, searcher.c 21×) — it's a vtable skeleton, not a working updater. Verdict: **LOW** — largely a stub COM surface; nothing to gain from a Rust port.

---

## crypto_sec

CryptoAPI / cert / SSP / smartcard family. Overwhelmingly Wine ports. Crypto correctness is high-risk to re-implement and these are mature Wine code; default verdict is LOW translate-worth. Notable exceptions: **rsaenh** (bundled TomCrypt algos — pure-math, but re-impl risk), **msv1_0** (ReactOS-native NTLM authpkg), **winscard** (ReactOS-native smartcard stubs), and the **mbedtls-wrapping** modules (bcrypt, schannel — thin shims over a 3rd-party lib).

### crypt32 — dll/win32/crypt32
- **LOC / files:** 40142 LOC, 24 C files (cert.c, chain.c, decode.c, encode.c, msg.c, oid.c, store.c, str.c, serialize.c, rootstore.c, ...) + crypt32_private.h
- **Role:** Core CryptoAPI: certificate stores, cert/CRL/CTL contexts, ASN.1 DER encode/decode, PKCS#7 messages, cert chain verification, OID registry, base64.
- **Origin:** Wine (every source has `WINE_DEFAULT_DEBUG_CHANNEL`; `-D_WINE`, links `wine`; `set_wine_module_FIXME`).
- **Completeness:** full — the largest, most complete Wine crypto DLL; generated `crypt32_stubs.c` covers a few unimplemented exports.
- **Key deps / consumers:** imports bcrypt, advapi32(+_vista), delay-imports cryptnet; consumed by rsaenh, schannel, wintrust, cryptui, cryptnet — the keystone of the whole family.
- **Problem points:** (1) huge hand-rolled ASN.1/DER encode+decode engine (encode.c/decode.c) — fiddly, security-sensitive byte parsing; (2) cert chain-building/trust logic (chain.c) correctness; (3) PKCS#7/message crypto (msg.c) + OID data tables; CSP-store provider model.
- **Difficulty:** Very High
- **Verdict:** LOW — too large, security-critical ASN.1/cert parsing, mature Wine. Not worth a Rust re-impl.

### cryptui — dll/win32/cryptui
- **LOC / files:** 7607 LOC, 1 C file (main.c) + resources (cryptui.rc, lang)
- **Role:** Certificate UI: cert-view/import/export wizards and property dialogs (CryptUIDlg* exports).
- **Origin:** Wine (`WINE_DEFAULT_DEBUG_CHANNEL`, `-D_WINE`, links `wine`, `set_wine_module_FIXME`).
- **Completeness:** full (dialog-heavy); `cryptui_stubs.c` for the few unimplemented exports.
- **Key deps / consumers:** imports crypt32, comctl32, comdlg32, ole32, user32, gdi32; delay-imports urlmon, wintrust. Pure UI leaf consumer.
- **Problem points:** (1) heavy Win32 dialog/resource + comctl32 wizard glue — Rust value is low for GUI plumbing; (2) tightly coupled to crypt32 cert-context APIs; (3) resource/.rc table dependencies.
- **Difficulty:** High (volume + dialog plumbing, not algorithmic)
- **Verdict:** LOW — GUI shell over crypt32; minimal Rust upside, large mechanical surface.

### rsaenh — dll/win32/rsaenh
- **LOC / files:** 14524 LOC, 11 C files (aes.c, des.c, rc2.c, rc4.c, md2.c, sha2.c, mpi.c, rsa.c, implglue.c, handle.c, rsaenh.c) + tomcrypt.h
- **Role:** Microsoft RSA/Enhanced base cryptographic provider (CSP) — the default software crypto provider behind CryptAcquireContext etc.
- **Origin:** Wine, but the algorithm files are **bundled public-domain TomCrypt/LibTomMath** (AES/DES/RC2/RC4/MD2/SHA2/bignum by Tom St Denis), wrapped by Wine's CSP glue (rsaenh.c/implglue.c/handle.c).
- **Completeness:** full.
- **Key deps / consumers:** imports crypt32, advapi32; consumed by anything doing classic CryptoAPI symmetric/RSA crypto.
- **Problem points:** (1) the math is the whole point — cipher/hash/bignum correctness is unforgiving (a translation bug is a silent miscompute, not a crash); (2) MPI bignum (mpi.c) constant-time / overflow subtleties; (3) CSP handle table + provider key-container model (handle.c/rsaenh.c).
- **Difficulty:** Very High
- **Verdict:** LOW (notable exception worth flagging) — the algorithm files *are* pure, self-contained, cfg-free math (the Rust sweet-spot shape), and Rust crates for AES/RSA exist — BUT re-implementing crypto primitives is the highest-risk thing in the whole audit and these are battle-tested. If ever attempted, do the pure cipher/hash files in isolation with vector tests, never the CSP glue. Default: leave in C.

### wintrust — dll/win32/wintrust
- **LOC / files:** 7633 LOC, 5 C files (asn.c, crypt.c, register.c, softpub.c, wintrust_main.c) + wintrust_priv.h
- **Role:** Trust verification: WinVerifyTrust, Authenticode/software-publisher trust providers, catalog & SIP registration.
- **Origin:** Wine (`WINE_DEFAULT_DEBUG_CHANNEL`, `-D_WINE`, links `wine`, `set_wine_module_FIXME`).
- **Completeness:** full; `wintrust_stubs.c` for remaining exports.
- **Key deps / consumers:** imports crypt32, imagehlp, advapi32; delay-imports cryptui. Consumed by setup/installer/driver-signing paths; mscat32/mssign32/msisip forward here.
- **Problem points:** (1) Authenticode signature-verification trust logic (softpub.c) — security-critical correctness; (2) its own ASN.1 chunk (asn.c) for SPC structures; (3) trust-provider/action-GUID registration & callback model (register.c) + COM-ish dispatch.
- **Difficulty:** High
- **Verdict:** LOW — security-critical trust decisions on mature Wine; not worth the re-impl risk.

### bcrypt — dll/win32/bcrypt (1152 LOC)
Wine CNG (Cryptography Next Generation) provider (`BCrypt*`), Copyright CodeWeavers; single bcrypt_main.c with ~214 references to **mbedtls/gnutls/gcrypt** backends — it is a thin shim that **wraps a 3rd-party crypto lib** (mbedtls here). Wine port, partial+stubs (49 FIXME/stub markers). Verdict: LOW — wraps mbedtls; little Rust value in the shim, none in re-doing the backend.

### credui — dll/win32/credui (1006 LOC)
Wine "Credentials User Interface" (CredUIPromptForCredentials etc.), Copyright CodeWeavers; single credui_main.c, dialog/UI code. Wine port, partial+stubs (23 markers). Verdict: LOW — GUI leaf, mechanical, low Rust upside.

### cryptdlg — dll/win32/cryptdlg (1203 LOC)
Wine certificate-policy/trust dialog helpers; single main.c, Wine origin. Real-ish (8 FIXME) but UI/dialog glue over crypt32. Verdict: LOW.

### cryptdll — dll/win32/cryptdll (111 LOC)
Tiny Wine DLL (Copyright CodeWeavers): a few `CD*` checksum/RNG/CSystem helpers, with most exports **forwarded to advapi32** (MD5*/etc.) in the .spec. Mostly forwarder + thin glue. Verdict: LOW (trivial).

### cryptnet — dll/win32/cryptnet (1871 LOC)
Wine network-retrieval object DLL (CryptRetrieveObjectByUrl — fetches certs/CRLs over HTTP/LDAP/file); single cryptnet_main.c, Wine origin, real implementation (26 markers). Delay-imported by crypt32. Verdict: LOW — network + URL-scheme dispatch over crypt32/wininet, Wine-mature.

### initpki — dll/win32/initpki (61 LOC)
Tiny Wine PKI-init/registration stub DLL; main.c only, near-empty (DllRegisterServer plumbing). Pure stub/forwarder. Verdict: LOW (trivial).

### mscat32 — dll/win32/mscat32 (42 LOC)
Wine catalog-API shim; main.c does `DLL_WINE_PREATTACH return FALSE` (prefer-native) and forwards catalog functions to wintrust. Pure forwarder to wintrust. Verdict: LOW (trivial).

### msisip — dll/win32/msisip (254 LOC)
Wine MSI Subject Interface Provider (SIP) — registers itself for Authenticode of .msi files via crypt32/wintrust SIP API; main.c, Wine origin, small real impl (1 stub). Verdict: LOW.

### mssign32 — dll/win32/mssign32 (93 LOC)
Wine signing-API DLL; `prefer-native` PREATTACH + every export is a FIXME stub (`SignError`/`SignerSign*` → E_NOTIMPL). Mostly stub. Verdict: LOW (trivial).

### mssip32 — dll/win32/mssip32 (60 LOC)
Tiny Wine SIP-registration DLL; main.c near-empty (register/unregister plumbing). Pure stub/forwarder. Verdict: LOW (trivial).

### msv1_0 — dll/win32/msv1_0 (3329 LOC)
**ReactOS-native** NTLM authentication package (LICENSE GPL-2.0, Copyright Andreas Maier 2019-2020 "PROJECT: Authentication Package DLL") — lsa.c/sam.c/user.c/usercontext.c + ntlm/ subdir; runs in LSASS as an SSP/AP. NOT a Wine port (native ReactOS, though pulls `wine/debug.h` for TRACE). Real but with NOT_IMPLEMENTED entry points (e.g. SpInitialize). Verdict: LOW — security-critical auth package in LSASS; native code, high risk, not a translation target now (flagged as the family's main native exception).

### pstorec — dll/win32/pstorec (399 LOC)
Wine Protected Storage (IPStore COM) DLL; single pstorec.c + .idl/.rgs, COM-object plumbing, mostly stub (42 FIXME). Wine origin. Verdict: LOW — deprecated COM API, stubby.

### sccbase — dll/win32/sccbase (60 LOC)
Tiny Wine smartcard base-CSP stub DLL; main.c near-empty register plumbing. Pure stub/forwarder. Verdict: LOW (trivial).

### schannel — dll/win32/schannel (2977 LOC)
SSL/TLS SSP. Hybrid: Wine glue (schannel_main.c/usermode.c/lsamode.c/secur32_wine.c/schannel_wine.c, all `_WINE`) over a ReactOS-authored **mbedTLS backend** (schannel_mbedtls.c, "Lightweight mbedTLS-based implementation", Copyright 2015 Peter Hater / Ismael Ferreras); links the **mbedtls** importlib + mbedtls include dir. So: **wraps a 3rd-party lib** (mbedtls) with Wine+ReactOS SSP glue, plus stubs.c. Verdict: LOW — wraps mbedtls; re-doing TLS in Rust is extreme risk, the shim has little value.

### slbcsp — dll/win32/slbcsp (60 LOC)
Tiny Wine Schlumberger smartcard CSP stub DLL; main.c near-empty register plumbing. Pure stub/forwarder. Verdict: LOW (trivial).

### winscard — dll/win32/winscard (757 LOC)
**ReactOS-native** SmartCard API (PROJECT/FILE/PROGRAMMER headers, Thomas Weidenmueller) — winscard.c (149 LOC, ~12 FIXME) + stubs.c covering the SCard* PC/SC surface. Mostly stub/forwarder (smartcard subsystem largely unimplemented). NOT Wine. Verdict: LOW — native stub layer; nothing real to translate yet.

### wrote crypto_sec: 20 modules (4 big) → out/crypto_sec.md

---

## net

Usermode networking clients layered above ws2_32 / the TCP stack. Mostly Wine ports
(wininet/winhttp/wldap32/inetcomm/inetcpl/hnetcfg/mpr/snmp), a few ReactOS-native
net providers / WinSock helpers (ntlanman, wshtcpip), and many thin stub DLLs (RAS
family, mprapi, msnet32, traffic, mgmtapi, sensapi, wlanapi). Default verdict for the
Wine ports is LOW translate-worth (mature C, frozen upstream, no Rust safety win);
the native exceptions and netstack touch-points are flagged below.

### inetcomm — dll/win32/inetcomm
- **LOC / files:** 8481 (C only), 9 .c + 2 .h
- **Role:** Internet messaging — MIME OLE message objects + SMTP/POP3/IMAP transports (mlang/mime parsing for mail clients).
- **Origin:** Wine (Robert Shearman / CodeWeavers, 2006).
- **Completeness:** full — real MIME parser and transport state machines, not stubs.
- **Key deps / consumers:** heavy COM (IMimeMessage/IMimeBody/IStream, 45+ vtbl/QueryInterface refs in mimeole.c); transports ride ws2_32 via internettransport.c. Consumer = legacy mail clients (Outlook Express-style).
- **Problem points:** (1) urlmon-style COM object soup — many vtbls, aggregation, ref-counting; (2) async socket transport state machines (SMTP/POP3/IMAP line protocols); (3) MIME charset/intl conversion (mimeintl) depends on mlang.
- **Difficulty:** High
- **Verdict:** LOW — Wine COM-heavy port, no consumer in the boot/smoke path, no Rust payoff.

### winhttp — dll/win32/winhttp
- **LOC / files:** 9321 (C only), 8 .c + 2 .h
- **Role:** WinHTTP — server/service-side HTTP(S) client API (request.c is the engine, 4798 LOC).
- **Origin:** ReactOS-modified-Wine (Jacek Caban / CodeWeavers 2007; has `winhttp_ros.diff`).
- **Completeness:** full — complete HTTP request/response engine, cookies, sessions, proxy autoconfig (jsproxy).
- **Key deps / consumers:** **ws2_32 (netstack coordination)** for sockets in net.c; schannel for TLS; jsproxy for PAC. Consumers = updater/service HTTP clients.
- **Problem points:** (1) HTTP/1.1 protocol + chunked transfer state machine in request.c; (2) async I/O + callback model (WINHTTP_STATUS_CALLBACK); (3) TLS via schannel handshake; (4) global session/handle table + cookie jar shared state.
- **Difficulty:** High
- **Verdict:** LOW — Wine port; only relevant once netstack TLS path matters; not boot-critical.

### wininet — dll/win32/wininet
- **LOC / files:** 24622 (C only), 11 .c + 4 .h — the largest module in this family.
- **Role:** WinINet — the classic HTTP/FTP/Gopher client + URL cache + cookie store (IE-era networking).
- **Origin:** ReactOS-modified-Wine (`wininet_ros.diff`). Bundles a private zlib (inflate.c, 1916 LOC).
- **Completeness:** full — http.c (6351), internet.c (4714), urlcache.c (4148), ftp.c (3955) all real implementations.
- **Key deps / consumers:** **ws2_32 (netstack coordination)** via netconnection.c; schannel TLS; mpr, shlwapi, shell32, normaliz, kernel32_vista. Consumer = anything doing InternetOpen/HTTP downloads, IE shell.
- **Problem points:** (1) three protocol stacks (HTTP/FTP/Gopher) each with own state machine; (2) on-disk URL cache + container/index format (urlcache.c) with global persistent state; (3) cookie persistence; (4) bundled zlib inflate (decompression); (5) async handle/callback model + per-handle object hierarchy.
- **Difficulty:** Very High
- **Verdict:** LOW — huge mature Wine port, no safety payoff vs cost; leave as C. If ever touched, slice urlcache/cookie before the protocol engines.

### wldap32 — dll/win32/wldap32
- **LOC / files:** 8625 (C only), 20 .c + 3 .h
- **Role:** LDAP client (winldap API) — bind/search/modify/add/delete/compare/parse + BER encoding (ber.c).
- **Origin:** Wine (Hans Leidekker, 2005); thin A/W/legacy shims over an external LDAP backend.
- **Completeness:** full as a *shim*, but **delegates to OpenLDAP** (`ldap_initialize`/`ldap_connect`) — the actual LDAP protocol lives in an external libldap, not here.
- **Key deps / consumers:** OpenLDAP (libldap) at runtime; otherwise standalone (imports only user32/msvcrt/kernel32/ntdll — **no ws2_32 directly**, sockets are inside libldap). Consumers = AD/directory clients.
- **Problem points:** (1) external libldap dependency dominates — translating the shim buys little without the backend; (2) A/W string + struct marshalling across the boundary; (3) BER encode/decode (ber.c) is the only self-contained protocol logic; (4) option/control/page result plumbing.
- **Difficulty:** Medium (shim) / High (with backend)
- **Verdict:** LOW — mostly a marshalling shim over OpenLDAP; no in-repo consumer, no Rust win.

### inetcpl — dll/cpl/inetcpl (1473 LOC)
Internet Options control-panel applet (Wine, Detlef Riekenberg 2010) — connections/content/general/security property pages. Real UI code, GUI-only, no protocol logic. Wine port. Verdict: LOW (control-panel UI, irrelevant to kernel/translation goals).

### hnetcfg — dll/win32/hnetcfg (3500 LOC)
Home Networking Configuration (Windows Firewall / ICS) COM server (Wine, Jeff Latimer 2007) — INetFwMgr/INetFwPolicy/port/profile/service objects via IDL/typelib, 7 .c files with vtbls. Real COM scaffolding but functionally stubby (no actual firewall backend on ReactOS). Wine port. Verdict: LOW (COM façade, no backend, no consumer).

### inetmib1 — dll/win32/inetmib1 (1462 LOC)
SNMP MIB-II extension agent (Wine, Juan Lang 2008) — maps the internet MIB to the SNMP extension API; ~10 FIXME/stub sites but substantive table logic. Wine port, rides snmpapi. Verdict: LOW.

### mgmtapi — dll/win32/mgmtapi (48 LOC)
SNMP management API (Wine, Stefan Leichter 2012). Pure stub (1 stub, 48 lines). Forwarder-tier. Verdict: LOW (skip — trivial stub).

### mpr — dll/win32/mpr (4099 LOC)
Multiple Provider Router — WNet network-redirector dispatch (Wine, Ulrich Weigand 1999; `mpr_ros.diff` → ReactOS-modified-Wine). Real provider-routing + credential cache (pwcache) logic, dispatches to net providers (ntlanman). Consumer of provider DLLs. Verdict: LOW (Wine, no Rust payoff), though it is a genuine functional module not a stub.

### mprapi — dll/win32/mprapi (83 LOC)
RAS server admin API (Wine, Dmitry Timoshkov 2006). 2 stubs, 83 lines. Stub DLL. Verdict: LOW (skip).

### msnet32 — dll/win32/msnet32 (56 LOC)
MS Network API (Wine, Klazes/Julliard). 1 stub, 56 lines. Stub DLL. Verdict: LOW (skip).

### ntlanman — dll/win32/ntlanman (199 LOC)
**surprisingly-native** — ReactOS LanMan network provider (Dmitry Chapyshev, dmitry@reactos.org). Implements the NP* WNet provider entry points (NPGetCaps/NPAddConnection3/...) that mpr dispatches to; uses wine/debug only for logging. ReactOS-native, but tiny and mostly thin provider shims over the redirector. Verdict: LOW (native but small/shim; flag as native for the audit — not a Wine port).

### rasapi32 — dll/win32/rasapi32 (555 LOC)
RAS dial-up API (Wine, Marcus Meissner 1998/2001). **63 stubs** — almost entirely a stub surface (RasDial/RasEnumConnections etc. return errors). Stub DLL. Verdict: LOW (skip — pure stub façade).

### rasdlg — dll/win32/rasdlg (106 LOC)
RAS dial-up UI dialogs (Wine, Dmitry Chapyshev 2007). 8 stubs, 106 lines. Stub DLL. Verdict: LOW (skip).

### rasman — dll/win32/rasman (94 LOC)
RAS connection manager service stub (ReactOS Team 2007). 6 stubs, 94 lines. Mostly-stub; ReactOS-authored header but trivial. Verdict: LOW (skip).

### sensapi — dll/win32/sensapi (61 LOC)
System Event Notification connectivity API (Wine, Steven Edwards/CodeWeavers 2005). 2 stubs — comment literally "we just report that we are always connected." Stub DLL. Verdict: LOW (skip).

### snmpapi — dll/win32/snmpapi (652 LOC)
SNMP utility/API library (Wine, Stridvall/Leidekker) — SnmpUtil* helpers, OID/varbind manipulation, real logic (only 1 FIXME). Wine port, consumed by inetmib1/mgmtapi. Verdict: LOW (real but no Rust payoff).

### traffic — dll/win32/traffic (67 LOC)
QoS Traffic Control API (Wine, Austin English 2009). 2 stubs, 67 lines. Stub DLL. Verdict: LOW (skip).

### wlanapi — dll/win32/wlanapi (582 LOC)
WLAN (native Wi-Fi) API (Wine, Christoph von Wittich 2009) — main.c (handle/RPC client scaffolding) + profile.c (WLAN profile XML). No FIXME stubs but no Wi-Fi backend on ReactOS/QEMU. Wine port. Verdict: LOW (no hardware/consumer).

### wshtcpip — dll/win32/wshtcpip (1258 LOC)
**surprisingly-native** — ReactOS WinSock Helper DLL for TCP/IP (WSH provider). iflist.c + wshtcpip.c implement the WSH* socket-helper entry points (address/option translation for the TCP/IP transport). **Native, and netstack-relevant** — it is the WinSock helper sitting directly on the TCP/IP transport (\Device\Tcp), so it coordinates with the netstack lane's transport. Real code, 4 minor stubs. Verdict: LOW-MEDIUM — native + on the live socket path; flag for netstack-lane awareness rather than a Wine-style translation candidate.

---

## printing

ReactOS printing stack is mixed: the `dll/win32/*` modules (compstui, sti, twain_32) are imported Wine code (`set_wine_module`), while the `win32ss/printing/*` modules are ReactOS-native in origin but ntprint/printui were *also* taken from Wine (`set_wine_module`); only localmon is genuinely ReactOS-native. All six are TAIL/small.

### compstui — dll/win32/compstui (93 LOC)
Common Property Sheets UI (CommonPropertySheetUIA/W, Get/SetCPSUIUserData) — single `compstui_main.c` with DllMain plus a handful of exports. Wine-imported (`set_wine_module`, `target_link_libraries(... wine)`), mostly stubbed (4 FIXME/stub markers; real logic is thin). Verdict: trivial Wine stub DLL — low-value, defer.

### sti — dll/win32/sti (562 LOC)
Still Image (scanner) API + WIA COM proxy: `sti.c` (15 stub markers), `sti_main.c` (class-factory/DllGetClassObject delegation), `guid.c`, plus generated RPC proxy from `sti_wia.idl`. Wine-imported and flagged `set_wine_module_FIXME` (CORE-5743: missing ARRAY_SIZE/CONST_VTABLE) — heavy COM/RPC + IDL, largely stubbed. Verdict: Wine COM-proxy DLL, mostly stubs — skip for translation (COM/IDL surface unsuitable).

### twain_32 — dll/win32/twain_32 (732 LOC)
TWAIN Data Source Manager (single export `DSM_Entry`): `dsm_ctrl.c` (DSM control/state machine) + `twain32_main.c` (global DSM state, source list). Wine-imported (`set_wine_module`). Real-ish DSM dispatch logic but niche (scanner acquisition). Verdict: self-contained Wine DLL, plausible translation candidate but very low priority.

### ntprint — win32ss/printing/base/ntprint (158 LOC)
Spooler Setup API (PSetup* monitor-info helpers). `ntprint.c` implements only PSetupCreate/Destroy/EnumMonitor; the spec is ~30 `@ stub` entries (ClassInstall32, driver install, ICM, etc.) generated into `ntprint_stubs.c`. Wine-imported (`set_wine_module`), overwhelmingly stub. Verdict: mostly stub Wine DLL — minimal real impl, defer.

### printui — win32ss/printing/base/printui (280 LOC)
Printer UI dialogs / `rundll32 printui.dll,PrintUIEntry` command-line driver. `printui.c` parses the option/flag string syntax (real impl); nearly all dialog/COM exports are `@ stub`/`-stub` in the spec → `printui_stubs.c`. Wine-imported (`set_wine_module`). Verdict: partial Wine impl (cmdline parser real, dialogs stubbed) — low priority.

### localmon — win32ss/printing/monitors/localmon (3061 LOC)
ReactOS-native Local Port Monitor (the only non-Wine module here): full MONITOR2 vtable wired in `main.c` (EnumPorts/Open/Write/Read/XcvData etc.), with real implementations in `ports.c` (1228 LOC), `xcv.c` (612), `tools.c` (260) plus a `ui/` subdir. No `set_wine_module` — Colin Finck 2015 ReactOS code, substantial real impl. Verdict: genuine ReactOS-native module with the most real logic of the family — the only serious translation candidate here (still TAIL-sized).

---

## multimedia

Family covers the Windows multimedia stack: winmm (the MM core), the MSACM/ACM audio-codec layer + individual .acm codec drivers, MCI command drivers, the VfW (Video-for-Windows) layer (msvfw32/avifil32 + ICM codec drivers), capture (avicap32), and the modern audio/media-foundation shells (mmdevapi/wmvcore). Almost everything is a direct Wine port (LGPL, Eric Pouech / Marcus Meissner / Maarten Lankhorst lineage) linking the `wine` compat lib; the only true ReactOS-native member is avicap32. Most members are codec/MCI driver DLLs with tight data tables and audio/video buffer math — low Rust value. Overall verdict: LOW translate-worth across the board; winmm is the only structurally interesting candidate and even it is deep-Wine.

### avifil32 — dll/win32/avifil32
- **LOC / files:** 10193 C, 13 source files (11 .c + 2 .h, plus guid.c/IDL proxy)
- **Role:** AVIFile API — COM interfaces (IAVIFile/IAVIStream/IGetFrame/IEditStream) for reading/writing/editing AVI RIFF files; bridges to ACM (audio) and ICM (video) streams.
- **Origin:** Wine (Marcus Meissner 1999, Michael Günnewig 2002-03). `set_wine_module_FIXME` (CORE-5743).
- **Completeness:** partial+stubs — own TODO header lists IAVIStreaming missing, FIND_INDEX unsupported, fnDelete/fnSetInfo are stubs, formatchanges unread.
- **Key deps / consumers:** imports msacm32, msvfw32, winmm, ole32, rpcrt4; uses RPC proxy (avifil32.idl). Consumed by media apps / mciavi.
- **Problem points:** (1) heavy COM/vtable surface across many files (CONST_VTABLE issues per FIXME); (2) RIFF/AVI chunk parsing + extrachunk index handling (extrachunk.c, tmpfile.c); (3) ACM/ICM stream-codec marshalling (acmstream.c/icmstream.c) with audio+video buffer format conversion; (4) RPC proxy / IDL marshalling glue.
- **Difficulty:** High
- **Verdict:** LOW — large COM-heavy Wine port with no kernel relevance; not translate-worth.

### msacm32 — dll/win32/msacm32
- **LOC / files:** 5563 C, 7 .c + 2 .h
- **Role:** Audio Compression Manager — driver enumeration/loading, format/filter selection, stream open/convert dispatch; ships a built-in PCM converter.
- **Origin:** Wine (Patrik Stridvall 1998, Eric Pouech 1999). `set_wine_module_FIXME`. Uses generated `msacm32_stubs.c`.
- **Completeness:** full+stubs — core driver/format/filter/stream paths implemented; stub spec entries auto-generated; pcmconverter is real.
- **Key deps / consumers:** imports winmm, advapi32, user32; consumed by avifil32, mciwave, and every .acm codec (which register as ACM drivers).
- **Problem points:** (1) ACM driver model — driverProc dispatch, message marshalling, driver enumeration via registry (driver.c/internal.c); (2) format/filter tag negotiation tables (format.c/filter.c); (3) PCM conversion sample math (pcmconverter.c) — bit-depth/channel/rate conversion buffers; (4) registry-backed driver priority list.
- **Difficulty:** High
- **Verdict:** LOW — central audio-codec manager, all usermode Wine; not translate-worth.

### winmm — dll/win32/winmm
- **LOC / files:** 10339 C (incl midimap subdir), 10 .c + 2 .h
- **Role:** Multimedia core DLL — waveform/midi/aux/mixer low-level device API, MCI command interface, MMIO file I/O, timers, joystick, PlaySound, driver loader.
- **Origin:** Wine (Martin Ayotte 1993, Eric Pouech 1998-2002), with ReactOS modifications — lolvldrv.c (Andrew Greenwood 2007) and playsound.c (Thamatip Chitpong 2023). `-D_WINE`. Uses generated `winmm_stubs.c`.
- **Completeness:** full+stubs — broad real implementation across wave/midi/mci/mmio/timer; some 16-bit thunk leftovers (winemm16.h).
- **Key deps / consumers:** imports userenv, advapi32, user32; the hub consumed by msacm32, avifil32, all mci* and .acm drivers, msvfw32.
- **Problem points:** (1) low-level driver loader + message dispatch to wave/midi/mixer kernel drivers (lolvldrv.c/driver.c) — ABI-sensitive driverProc thunking; (2) MCI command string parser + table-driven command dispatch (mci.c, large); (3) MMIO buffered file I/O + RIFF walking (mmio.c); (4) multimedia timer thread/callback model (time.c); 16-bit legacy thunks.
- **Difficulty:** High
- **Verdict:** LOW (with a caveat) — the one structurally meaty member and partly ReactOS-touched, but it's a deep Wine usermode service DLL with no kernel-translation payoff; not a priority.

### TAIL

### avicap32 — dll/win32/avicap32 (324 LOC)
ReactOS-NATIVE thin video-capture API shell (Dmitry Chapyshev, PROJECT header — not Wine). Mostly capCreateCaptureWindow / capGetDriverDescription wrappers over registry-enumerated capture drivers; effectively forwarder/thunk-grade. Verdict: LOW (trivial native shell, no value).

### iccvid — dll/win32/iccvid (1063 LOC)
Wine port of the Cinepak (CVID) video decompressor codec (Dr. Tim Ferguson algorithm). Real decoder: bitstream parse + YUV codebook pixel reconstruction in one .c. Pure data-table/pixel-loop codec, usermode ICM driver. Verdict: LOW.

### imaadp32.acm — dll/win32/imaadp32.acm (990 LOC)
Wine IMA ADPCM audio codec driver (Eric Pouech). Real ADPCM encode/decode step-tables + ACM driverProc dispatch in one .c. Codec data-table arithmetic. Verdict: LOW.

### mciavi32 — dll/win32/mciavi32 (2615 LOC, 4 files)
Wine MCI driver for AVI video playback (Eric Pouech). Real-ish: open/play/window/output across mciavi.c/info.c/mmoutput.c/wnd.c, drives avifil32 + msvfw32 + winmm. MCI command dispatch + playback timing. Verdict: LOW.

### mcicda — dll/win32/mcicda (1374 LOC)
Wine MCI CD-audio driver (Martin Ayotte 1994). Single .c — MCI command dispatch over IOCTL_CDROM to play/seek redbook audio tracks. Driver-model + MCI dispatch, no algorithm. Verdict: LOW.

### mciqtz32 — dll/win32/mciqtz32 (1209 LOC)
Wine MCI digitalvideo driver backed by DirectShow/quartz (Christian Costa 2009). Heavily stubbed (~35 FIXME/stub markers) — mostly an MCI→quartz shim, much unimplemented. Verdict: LOW (mostly stub).

### mciseq — dll/win32/mciseq (1626 LOC)
Wine MCI MIDI sequencer driver (Martin Ayotte 1994). Single .c — parses/plays MIDI files via winmm midiOut, MCI command dispatch + event timing. Verdict: LOW.

### mciwave — dll/win32/mciwave (1784 LOC)
Wine MCI waveaudio driver (Martin Ayotte 1994). Single .c — record/play WAV via winmm waveOut/waveIn, MCI dispatch + buffer queueing. Verdict: LOW.

### mmdevapi — dll/win32/mmdevapi (2290 LOC, 4 files)
Wine MMDevice API (Vista audio endpoint enumeration; Maarten Lankhorst 2009). COM device enumerator + IAudioVolume, IDL-registered classes. Real but COM-shell over an audio backend (largely inert without a driver). Verdict: LOW.

### msacm32.drv — dll/win32/msacm32.drv (1244 LOC)
Wine wave-mapper driver (Eric Pouech). Single wavemap.c — maps app wave format requests to an installed device via ACM, format negotiation + buffer relay. Driver-model glue. Verdict: LOW.

### msadp32.acm — dll/win32/msadp32.acm (849 LOC)
Wine MS-ADPCM audio codec driver (Eric Pouech 2002). Single .c — ADPCM coefficient/step tables + encode/decode + ACM driverProc. Codec data tables. Verdict: LOW.

### msg711.acm — dll/win32/msg711.acm (1142 LOC)
Wine G.711 (A-law/µ-law) audio codec driver (Eric Pouech 2002). Single .c — companding lookup tables + ACM dispatch. Trivial codec. Verdict: LOW.

### msgsm32.acm — dll/win32/msgsm32.acm (644 LOC)
Wine GSM 06.10 audio codec driver (Maarten Lankhorst 2009). Single .c — small ACM shim over GSM encode/decode. Codec driver. Verdict: LOW.

### msrle32 — dll/win32/msrle32 (1924 LOC)
Wine Microsoft RLE video codec driver (Michael Günnewig 2002-03). Single .c — RLE4/RLE8 compress/decompress + ICM driverProc. Pixel-loop codec. Verdict: LOW.

### msvfw32 — dll/win32/msvfw32 (3665 LOC, 3 files)
Wine Video-for-Windows core (Eric Pouech 2000). ICM codec manager (msvideo_main.c), DrawDib blitting (drawdib.c), MCIWnd control (mciwnd.c). Real VfW layer — codec enumeration/dispatch + GDI blit + a windowed control. Two exported specs (msvfw32 + msvideo). Verdict: LOW (usermode video shell, no kernel payoff).

### msvidc32 — dll/win32/msvidc32 (681 LOC)
Wine MS-Video1 (CRAM) video codec driver (ffmpeg-derived 2003). Single msvideo1.c — decode/encode + DriverProc (CRAM_DriverProc). Pixel-loop codec. Verdict: LOW.

### winemp3.acm — dll/win32/winemp3.acm (1039 LOC)
Wine MP3 decoder ACM driver (mpglib-based; Eric Pouech 2002). Single mpegl3.c wraps an MP3 decode core behind ACM driverProc. Codec driver with heavy decode tables. Verdict: LOW.

### winmm/midimap — dll/win32/winmm/midimap (579 LOC)
Wine MIDI mapper driver (subdir of winmm). Single midimap.c — routes midiOut messages to a configured physical MIDI device per registry mapping. Small driver-model shim. Verdict: LOW.

### wmvcore — dll/win32/wmvcore (2931 LOC, 3 files)
Wine Windows Media Format SDK core (Austin English 2012). reader.c/writer.c/wmvcore_main.c — COM IWMReader/IWMWriter surface, largely stub/FIXME (~14 in main alone), thin shell over a media backend. Verdict: LOW (mostly stub COM shell).

---

## text_font_codec

Family scope: rich edit, Uniscribe shaping, theming, GDI+, image codecs, IDN/normaliz, font embedding/subset. Five large Wine ports (gdiplus/riched20/usp10/uxtheme/windowscodecs) dominate the LOC; the rest are thin forwarders, stubs, or large generated data tables. **mlang is already cohab-translated to Rust** (the one exception in this family). uxtheme + usp10 are on the desktop render path (themed nonclient drawing, text shaping); the others are app-facing and not in the boot/smoke chain.

---

### gdiplus — dll/win32/gdiplus
- **LOC / files:** ~29,383 LOC (pure C), 14 .c + gdiplus_private.h. Dominated by graphics.c (225K bytes), image.c (182K), metafile.c (142K), graphicspath.c (73K), brush.c (61K), region.c (47K).
- **Role:** GDI+ 2D vector/raster graphics, imaging, EMF+ metafiles, font/text rendering on top of GDI.
- **Origin:** Wine (near-verbatim; tiny ReactOS contid tweak in `gdiplus_ros.diff` — `InterlockedExchangeAdd` for unique container IDs).
- **Completeness:** full (large mature Wine implementation; `gdiplus_stubs.c` generated for a handful of unimplemented spec entries).
- **Key deps / consumers:** delay-imports windowscodecs (image decode); imports gdi32/user32/ole32/oleaut32/shlwapi. Consumed by uxtheme (PNG/themed glyphs in vista mode) and apps. NOT on the core boot path — desktop renders without it; uxtheme only pulls it at DLL_EXPORT_VERSION >= 0x600.
- **Problem points:** (1) 630-entry stdcall spec — huge flat-C float/REAL math API surface; (2) heavy GDI HDC/HBITMAP interop and float geometry (Bezier/curve flattening, region combine); (3) EMF+ metafile record parse/playback (binary format, metafile.c); (4) COM-ish GpImage/codec bridging to WIC.
- **Difficulty:** Very High
- **Verdict:** LOW (do not translate). Large mature Wine port, off boot path; cost/value poor. Keep as C.

### riched20 — dll/win32/riched20
- **LOC / files:** ~25,339 LOC (C), ~20 .c + headers. editor.c (140K), richole.c (166K, TOM COM), reader.c/writer.c (RTF, 78K/37K), wrap.c/paint.c/caret.c/para.c/run.c/table.c/style.c/txthost.c/txtsrv.c.
- **Role:** RichEdit 2.0+ control engine + Text Object Model (ITextServices/ITextHost/ITextDocument) + RTF reader/writer.
- **Origin:** Wine (riched20_ros.diff is trivial — keeps an old .cur cursor file).
- **Completeness:** full (mature). Generates riched_tom typelib; uses uuid/CONST_VTABLE.
- **Key deps / consumers:** imports usp10 (shaping), imm32, user32, gdi32, ole32, oleaut32. Consumed by msftedit/riched32 (forwarders) and apps (editors, dialogs). NOT on smoke/boot path.
- **Problem points:** (1) full COM vtable surface (TOM: ITextServices/Host/Document/Range/Font/Para) — CONST_VTABLE, raw-dylib glue, the riched_tom.idl typelib; (2) RTF parser/writer state machines (rtf.h is 37K of tokens); (3) line wrap/paint/caret geometry tied to GDI + usp10; (4) i386 __thiscall wrapper for COM.
- **Difficulty:** Very High
- **Verdict:** LOW (do not translate). Big COM-heavy Wine port off boot path.

### usp10 — dll/win32/usp10
- **LOC / files:** ~15,868 LOC (C), 12 .c + headers. shape.c (129K), usp10.c (148K), opentype.c (106K), linebreak.c (62K), bidi.c/direction.c/mirror.c/indic*.c/breaking.c/bracket.c/shaping.c.
- **Role:** Uniscribe — complex-script text shaping (OpenType GSUB/GPOS), BiDi, line breaking, glyph runs. `ScriptString*`/`ScriptShape`/`ScriptItemize`/etc.
- **Origin:** Wine. ReactOS diff (`usp10_ros.diff`) only re-implements `LpkPresent` (was a stub) and a flag.
- **Completeness:** full (mature shaping engine).
- **Key deps / consumers:** imports gdi32/user32/advapi32. **Consumed by riched20 and by the GDI/text render path** — used when complex scripts / BiDi are drawn. On the desktop render path indirectly, but core Latin desktop boot does not exercise the heavy shaping code.
- **Problem points:** (1) OpenType table binary parsing (GSUB/GPOS/GDEF/cmap, big-endian) in opentype.c; (2) huge embedded Unicode/script property data tables (direction.c, mirror.c, indicsyllable.c — generated arrays); (3) BiDi + Indic reordering state machines; (4) ScriptCache HDC/HFONT lifetime.
- **Difficulty:** Very High
- **Verdict:** LOW (do not translate). Despite render-path relevance, it's a large, table-heavy, mature Wine algorithm port — low Rust value, high risk. Keep as C.

### uxtheme — dll/win32/uxtheme
- **LOC / files:** ~10,123 LOC (C, +1 small .cpp pngsup). 17 sources. stylemap.c (70K, generated property maps), draw.c (66K), system.c (50K), nonclient.c (45K), msstyles.c (37K), themehooks.c, ncscrollbar.c, property.c, uxini.c.
- **Role:** Visual Styles / theming — `.msstyles` loading, themed control part/state drawing, themed nonclient (caption/scrollbar) rendering, theme hooks.
- **Origin:** ReactOS-modified Wine — substantially diverged from upstream Wine. ReactOS adds its own nonclient/scrollbar/themehooks (`ncscrollbar.c`, `nonclient.c`, `themehooks.c`, `uxini.c`) not in Wine; `-D__WINESRC__` but heavily ros-specific.
- **Completeness:** full (real implementation; some vista-gated PNG support via pngsup.cpp).
- **Key deps / consumers:** imports user32/gdi32/advapi32 (+ gdiplus/shlwapi in vista mode), delay-imports msimg32. **On the desktop render path** — themed window frames/controls when a theme is active. Consumed by user32/comctl32/explorer.
- **Problem points:** (1) `.msstyles` PE-resource theme-data parsing (binary, msstyles.c); (2) huge generated style/property name→id maps (stylemap.c 70K); (3) GDI-heavy part/state drawing + nonclient region math; (4) cross-DLL theme hook installation (themehooks.c) into user32.
- **Difficulty:** High
- **Verdict:** LOW (do not translate). It IS on the render path, but it's a large, GDI-bound, ros-diverged theming engine — translation value is low and regression risk to the visible desktop is high. Keep as C. (If anything, only a tiny leaf helper would be a candidate, not the module.)

### windowscodecs — dll/win32/windowscodecs
- **LOC / files:** ~31,385 LOC (C), ~35 .c. Largest: image.c-class decoders — gifformat.c (70K), ddsformat.c (74K), regsvr.c (77K), info.c (78K), converter.c (62K), imgfactory.c (55K), libtiff.c (41K), bmpdecode.c (38K), metadatahandler.c/metadataquery.c, palette.c, plus libpng/libjpeg/libtiff bridges and ungif.c.
- **Role:** Windows Imaging Component (WIC) — COM image decoders/encoders (BMP/GIF/ICO/PNG/JPEG/TIFF/TGA/DDS), pixel-format converters, scaler, metadata readers, color transforms.
- **Origin:** Wine (uses Wine WIC + bundled libjpeg/libpng/libtiff/zlib from ReactOS libs).
- **Completeness:** full (mature; rpcproxy for WIC interfaces, regsvr registration).
- **Key deps / consumers:** imports libjpeg/libpng/libtiff/ole32/oleaut32/rpcrt4/propsys/shlwapi. Consumed by gdiplus (delay), uxtheme PNG, shell/explorer thumbnails. NOT on the core boot path.
- **Problem points:** (1) image-format binary parsing across many codecs (GIF LZW, DDS, TIFF tags, PNG/JPEG via C libs); (2) full WIC COM surface (decoder/frame/encoder/converter/metadata vtables) + RPC proxy; (3) pixel-format conversion matrix (converter.c); (4) FFI to three external C image libraries — translating those bridges to Rust gains little.
- **Difficulty:** Very High
- **Verdict:** LOW (do not translate). Large COM + external-C-lib codec port off boot path. Keep as C.

---

### normaliz — dll/nls/normaliz (~2,048 LOC)
IDN/Unicode normalization forwarder DLL: IdnToAscii/IdnToUnicode/IdnToNameprepUnicode are real Wine punycode+nameprep (nameprep.c is ~91K of *generated* RFC3454 data tables — `nameprep_char_type[4432]` etc.), but `NormalizeString`/`IsNormalizedString` are **ReactOS stubs returning ERROR_CALL_NOT_IMPLEMENTED**. Wine-derived data + ReactOS stub mix; off boot path. Verdict: LOW — mostly generated tables + stubs, no Rust value.

### fontsub — dll/win32/fontsub (~67 LOC)
Font subsetting DLL. `CreateFontPackage` is a **Wine stub that just memcpy's the whole font** (no real subsetting); `MergeFontPackage` is a spec `stub`. Wine. Off boot path. Verdict: LOW — trivial stub, nothing to translate.

### mlang — dll/win32/mlang (~4,102 LOC) — ALREADY RUST
MIME/codepage/font-link COM service (IMultiLanguage, IMLangFontLink, INet string conversion). **EXCEPTION: already cohab-translated to Rust** — `add_cargo_module(mlang ... BODY_CRATE mlang-rust)`; mlang.c kept on disk per never-delete; `rust_cohab_stub.c` supplies wine-debug/IsEqualGUID/list_* shims. Originally Wine, now Rust body. Verdict: N/A (done) — do not re-translate; off boot path.

### msftedit — dll/win32/msftedit (~79 LOC)
RichEdit 4.1 facade — `msftedit.spec` **forwards almost everything to riched20** (CreateTextServices, RichEdit*WndProc, IID_* externs); only a small DllGetVersion/DllMain in C. Wine/ReactOS. Off boot path. Verdict: LOW — pure forwarder, no value.

### riched32 — dll/win32/riched32 (~98 LOC)
RichEdit 1.0 compatibility shim — registers a window class whose WndProc forwards to riched20's `RichEdit10ANSIWndProc`. Spec exports only DllGetVersion. Wine. Off boot path. Verdict: LOW — thin wrapper, no value.

### t2embed — dll/win32/t2embed (~301 LOC)
Font embedding (TTLoadEmbeddedFont/TTEmbedFont/TTCharToUnicode...). main.c is thin; **stubs.c is ReactOS UNIMPLEMENTED stubs** for the whole TT* surface (spec lists both decorated i386 and undecorated names). ReactOS stub (not real Wine impl). Off boot path. Verdict: LOW — all-stub, nothing to translate.

### windowscodecsext — dll/win32/windowscodecsext (~77 LOC)
WIC color-transform extension shim — `WICCreateColorTransform_Proxy` creates a real WIC factory object, `DllGetClassObject` is a FIXME stub. Wine. Off boot path. Verdict: LOW — tiny proxy, no value.

---

## directx_3d

All 12 modules are Wine DirectX ports (LGPL-2.1, CodeWeavers / Wine authors: Stefan Dösinger, Henri Verbeet, Christian Costa, Marcus Meissner, etc.). ReactOS markers are limited to `#ifdef __REACTOS__` build-glue shims (precomp.h, config.h, wingdi includes) — none are ReactOS-native logic. None are boot/smoke-relevant (DirectX 3D graphics is not on the desktop-boot path). **Blanket verdict: LOW translate-worth across the family** — this is translating CodeWeavers' code, not ReactOS's. Exceptions called out per entry below (only the two trivial-stub tails are even arguably worth a glance).

### d3d8 — dll/directx/wine/d3d8
- **LOC / files:** 7301, 11 C files (buffer, device, directx, shader, surface, swapchain, texture, vertexdeclaration, volume, d3d8_main, guid)
- **Role:** Direct3D 8 API DLL; thin COM wrapper translating IDirect3D8 calls down to wined3d.
- **Origin:** Wine (pure). Only ROS marker = build precomp glue.
- **Completeness:** Full impl (spec exports = 5 entry points, 0 stubs); real work is the COM object layer.
- **Key deps / consumers:** wined3d (does all the real GPU work); consumed by D3D8 games.
- **Problem points:** Dense COM/IUnknown vtable boilerplate (every object has a vtbl + QueryInterface/AddRef/Release), thunk-down-to-wined3d pattern, stdcall ABI, global state in directx.c.
- **Difficulty:** Medium
- **Verdict:** LOW — pure Wine wrapper over wined3d; no ROS value.

### d3d9 — dll/directx/wine/d3d9
- **LOC / files:** 9467, 13 C files (adds query, stateblock vs d3d8)
- **Role:** Direct3D 9 API DLL; COM wrapper over wined3d (newer/larger than d3d8).
- **Origin:** Wine (pure). ROS markers = build glue only.
- **Completeness:** Full impl; spec = 14 exports, 3 stubs (minor).
- **Key deps / consumers:** wined3d; consumed by D3D9 games/apps.
- **Problem points:** Same COM vtable density as d3d8 plus stateblock recording and query objects; stdcall ABI; thunking convention to wined3d.
- **Difficulty:** Medium
- **Verdict:** LOW — pure Wine wrapper.

### d3dcompiler_43 — dll/directx/wine/d3dcompiler_43
- **LOC / files:** 10325, 8 C files + 6 lex/yacc grammar files (hlsl.l/.y, asmshader.l/.y, ppl.l/ppy.y)
- **Role:** HLSL / shader-assembly compiler — parses HLSL & shader asm, emits D3D bytecode.
- **Origin:** Wine (pure); 3 files touch ROS build markers only.
- **Completeness:** Full impl; spec = 17 exports, 5 stubs.
- **Key deps / consumers:** wpp preprocessor; consumed by d3dx9 and apps compiling shaders at runtime.
- **Problem points:** **Flex/Bison generated lexer+parser** (6 .l/.y grammars) — these are generated C that does not translate to Rust mechanically; full HLSL/asm compiler with AST, bytecode writer, reflection. Among the hardest in the family.
- **Difficulty:** Very High
- **Verdict:** LOW — lex/yacc grammar core makes this near-impossible to faithfully port; pure Wine compiler tech.

### d3drm — dll/directx/wine/d3drm
- **LOC / files:** 15288, 11 C files (frame, mesh/meshbuilder, face, light, material, texture, viewport, device, math)
- **Role:** Direct3D Retained Mode — legacy high-level 3D scene API (frames/meshes/lights).
- **Origin:** Wine (pure); 1 file has a ROS marker (build glue).
- **Completeness:** Full impl; spec = 23 exports, 0 stubs.
- **Key deps / consumers:** ddraw / d3d; obsolete API used by very old apps.
- **Problem points:** Many COM interfaces (IDirect3DRMFrame/Mesh/etc. with versioned vtables), 3D math (vectors/matrices/quaternions in math.c), mesh-builder data structures, global state.
- **Difficulty:** High
- **Verdict:** LOW — pure Wine, legacy/obsolete API.

### d3dx9_36 — dll/directx/wine/d3dx9_36
- **LOC / files:** 34328, 21 C files (effect, mesh, font, sprite, animation, skin, shader, texture, surface, math, preshader, txc_compress/fetch_dxtn, core, line, render, xfile)
- **Role:** D3DX9 utility/helper library — the big "everything else" D3D9 helper DLL (effects framework, mesh loaders, font/sprite, texture compression, math).
- **Origin:** Wine (pure); 22 files reference ROS markers (build glue — txc_dxtn carries its own non-Wine S3TC license header but is still upstream third-party, not ROS-native).
- **Completeness:** Full impl; spec = **336 exports, 71 stubs** (~21% stub) — large real surface.
- **Key deps / consumers:** d3dcompiler_43, wined3d, d3d9; consumed by many games via D3DX.
- **Problem points:** Huge surface; effects framework (state-machine .fx parser/interpreter in effect.c + preshader.c), DXT texture compression (txc_*), big math tables, mesh loaders, COM. Heterogeneous and large.
- **Difficulty:** Very High
- **Verdict:** LOW — largest non-engine module, all pure Wine helper code.

### d3dxof — dll/directx/wine/d3dxof (3525 LOC)
DirectX .X File parser (IDirectXFile COM + binary/text .x parsing + bundled MSZIP decompressor mszip.c). Pure Wine (Christian Costa); spec = 5 exports, 0 stubs; full impl. COM vtables + a parser + a compression codec. Verdict: LOW — pure Wine, niche file-format DLL.

### ddraw — dll/directx/wine/ddraw
- **LOC / files:** 24761, 12 C files + 1 IDL (surface, device, palette, clipper, viewport, material, light, executebuffer, vertexbuffer, ddraw, utils, main)
- **Role:** DirectDraw + legacy Direct3D (D3D1–7 immediate mode) on top of wined3d.
- **Origin:** Wine (pure); 3 files have ROS build markers.
- **Completeness:** Full impl; spec = 28 exports, 16 stubs (many are COM class-factory class IDs).
- **Key deps / consumers:** wined3d; consumed by old DirectDraw/early-D3D games.
- **Problem points:** Very heavy COM (multiple interface versions per object: IDirectDraw1/2/4/7, IDirect3DDevice1–7), executebuffer interpreter, surface/blit logic, IDL-defined coclasses, large global state.
- **Difficulty:** High
- **Verdict:** LOW — pure Wine, legacy API over wined3d.

### dinput — dll/directx/wine/dinput
- **LOC / files:** 13288, 11 C files + 1 IDL (device, keyboard, mouse, joystick + joystick_linux/linuxinput/osx, effect_linuxinput, config, data_formats, dinput_main)
- **Role:** DirectInput — keyboard/mouse/joystick input device API.
- **Origin:** Wine (pure); 1 ROS marker file.
- **Completeness:** Full impl; spec = 7 exports, 0 stubs.
- **Key deps / consumers:** user32/win32k for kbd+mouse; joystick backends.
- **Problem points:** COM device objects; **Linux/OSX joystick backends (joystick_linux*.c, effect_linuxinput.c)** are host-OS-specific and inapplicable to ReactOS — dead weight; data-format descriptor tables; callbacks for device enumeration.
- **Difficulty:** Medium-High
- **Verdict:** LOW — pure Wine; the joystick backends are Linux/OSX-only and irrelevant to the OS.

### dplayx — dll/directx/wine/dplayx
- **LOC / files:** 11510, 9 C files + 1 IDL (dplay, dplaysp, dplobby, lobbysp, name_server, dplayx_global, dplayx_messages, dpclassfactory, dplayx_main)
- **Role:** DirectPlay — legacy multiplayer networking/lobby/session API.
- **Origin:** Wine (pure); 0 ROS markers in source.
- **Completeness:** Spec = 11 exports, 0 stubs, but DirectPlay is famously a thin/partial Wine impl (much internal logic incomplete).
- **Key deps / consumers:** networking; obsolete API, near-zero real-world use.
- **Problem points:** COM, custom message queue (dplayx_queue.h), name-server/session global state, IDL coclasses.
- **Difficulty:** High
- **Verdict:** LOW — pure Wine, dead networking API.

### wined3d — dll/directx/wine/wined3d
- **LOC / files:** 79904, 27 C files (the engine: device, context, cs, shader + shader_sm1/sm4, glsl_shader, arb_program_shader, ati_fragment_shader, nvidia_texture_shader, gl_compat, state, stateblock, surface, texture, buffer, view, sampler, query, resource, swapchain, vertexdeclaration, palette, utils, directx, dxtn)
- **Role:** The actual D3D-over-OpenGL translation engine — the heart of the whole DirectX stack; every other d3dN/ddraw module thunks into it.
- **Origin:** Wine (pure, the canonical CodeWeavers engine); 5 files ROS build markers; note `roshacks.txt` documenting ROS-specific build hacks.
- **Completeness:** Full impl; spec = 317 exports, 0 stubs — by far the largest module (80K LOC).
- **Key deps / consumers:** OpenGL driver (opengl32); consumed by d3d8/d3d9/ddraw/d3dx9.
- **Problem points:** Massive; **GLSL/ARB shader codegen** (string-building GL shader source from D3D bytecode), GL state machine + extension dispatch tables, multiple shader-model backends (sm1/sm4), heavy `unsafe`/pointer-arithmetic-laden GPU resource management, command-stream (cs.c) threading, enormous global format/capability tables, asm fragments.
- **Difficulty:** Very High (the single hardest module in the family)
- **Verdict:** LOW — 80K LOC of pure-Wine GPU engine; translating this is translating CodeWeavers' flagship code with zero ReactOS value.

### dciman32 — dll/win32/dciman32 (63 LOC)
DCI Manager (legacy "Device Context Interface" video acceleration). Wine (Marcus Meissner, 1 ROS wingdi-include shim). Almost entirely a **stub DLL**: spec = 21 exports, **17 stubs**, only ~4 trivial real functions (DCIOpenProvider/CloseProvider/CreatePrimary). Verdict: LOW (so trivial it's not worth a port; if anything, a near-empty stub — easy but pointless).

### xinput1_3 — dll/win32/xinput1_3 (165 LOC)
XInput 1.3 gamepad API (Wine, Andrew Fenn). Tiny: 1 C file, spec = 9 exports, 0 stubs but a **thin/skeleton real impl** (XInputGetState/SetState/Capabilities/etc.) — small enough to translate trivially. Pure Wine, not boot-relevant. Verdict: LOW, though it's the one genuinely small/easy candidate if a gamepad DLL were ever wanted (165 LOC, no COM).

---

## directx_av

> Family: Wine-derived DirectX A/V media DLLs (DirectShow / DirectSound / DirectMusic / DPlay).
> ALL modules are unmodified-or-lightly-ROS-wrapped Wine ports linking `wine` + `strmbase`/`strmiids`/`dxguid`/`uuid`.
> NONE are boot-relevant (no kernel/win32k/smss/csrss path; all are on-demand `reactos/system32` user-mode COM DLLs loaded only when a media app runs).
> Each carries `set_wine_module_FIXME(...)` (CORE-5743: no CONST_VTABLE) — i.e. ROS already treats them as upstream Wine to be re-synced, not maintained natively.
> Overall verdict: **LOW translate value across the entire family** — huge COM filter-graph / DirectShow-pin vtable surface, callback/threading-heavy, pure desktop-app territory. Exceptions called out individually below (the two genuinely native/stub modules, msdmo + dpnhpast).

### dsound — dll/directx/wine/dsound
- **LOC / files:** ~10712 LOC, 13 C files (+ .spec/.idl/.rgs/.h). Biggest: dsound.c (58K), primary.c (37K), mixer.c (36K), sound3d.c (35K), duplex.c (25K), propset.c (21K), capture.c (47K incl. blank), buffer.c (42K incl. blank), dsound_convert.c (14K).
- **Role:** DirectSound — audio mixing/playback/capture, 3D positional audio, property sets.
- **Origin:** Wine (Marcus Meissner / TransGaming / CodeWeavers headers). `-D_WINE`.
- **Completeness:** Full implementation (real mixer + format conversion + 3D math), with TODOs noted in header (3D hardware, focus handling).
- **Key deps / consumers:** links `dxguid uuid wine`, imports `winmm advapi32 msvcrt`; consumers = quartz dsoundrender, games/media apps.
- **Problem points:** (1) COM vtables across IDirectSound/IDirectSoundBuffer/IDirectSound3DBuffer/IKsPropertySet; (2) real-time mixing threads + critical-section locking (race conditions flagged in source); (3) DSP/format-conversion inner loops (dsound_convert.c) with per-sample bit-twiddling and unaligned 24-bit access — perf-sensitive unsafe Rust; (4) global device arrays `DSOUND_renderer[MAXWAVEDRIVERS]`.
- **Difficulty:** High.
- **Verdict:** LOW. Large, threading + COM + perf-critical DSP, zero boot relevance.

### qcap — dll/directx/wine/qcap
- **LOC / files:** ~7293 LOC, 10 C files (+ generated qcap_stubs.c). Biggest: avimux.c (78K), capturegraph.c (30K), v4l.c (28K), vfwcapture.c (23K), smartteefilter.c (22K), avico.c (22K).
- **Role:** DirectShow capture (QCAP.DLL) — AVI mux, capture-graph builder, smart-tee, VFW capture, video4linux source.
- **Origin:** Wine (Dominik Strasser / Maarten Lankhorst). `#include "config.h"`; v4l.c is literally Linux Video4Linux glue — irrelevant on ReactOS/QEMU.
- **Completeness:** Full (Wine), but v4l backend is a no-op on non-Linux host; uses generated stubs file.
- **Key deps / consumers:** links `strmbase strmiids uuid wine`, imports `ole32 oleaut32 gdi32`, delay-imports `msvfw32`; consumers = capture apps.
- **Problem points:** (1) DirectShow filter/pin COM vtables (BaseFilter/IPin) via strmbase; (2) AVI mux state machine + buffering/threads (avimux.c huge); (3) callback-driven sample delivery; (4) v4l ioctl glue is platform-foreign (dead weight to port).
- **Difficulty:** High.
- **Verdict:** LOW. Capture stack no one exercises in the QEMU target; v4l portion is pointless to translate.

### quartz — dll/directx/wine/quartz
- **LOC / files:** ~23531 LOC, 22 C files (+ generated proxy/typelib). Biggest: filtergraph.c (166K!), vmr9.c (103K), filtermapper.c (52K), avisplit.c (50K), filesource.c (45K), dsoundrender.c (43K), videorenderer.c (34K), regsvr.c (34K), mpegsplit.c (30K), parser.c, memallocator.c, systemclock.c, waveparser.c, acmwrapper.c, avidec.c, nullrenderer.c.
- **Role:** DirectShow core runtime — the filter graph manager, filter mapper, source/splitter/parser/renderer filters, memory allocator, system clock.
- **Origin:** Wine. `-DENTRY_PREFIX=QUARTZ_ -DWINE_REGISTER_DLL -DPROXY_DELEGATION`; uses RPC proxy (quartz_strmif.idl) + typelib (control_tlb.idl). Built `_WIN32_WINNT=0x600`.
- **Completeness:** Full, heaviest DirectShow implementation in the tree.
- **Key deps / consumers:** links `strmbase strmiids uuid dxguid wine ${PSEH_LIB}`; imports a long list incl. `dsound msacm32 msvfw32 ole32 oleaut32 rpcrt4 user32 gdi32`; deps `dxsdk stdole2`. The graph manager is the hub all other A/V filters plug into.
- **Problem points:** (1) filtergraph.c alone is 166K of COM-aggregation across ~dozen interfaces (IGraphBuilder/IMediaControl/IMediaSeeking/IMediaEvent/IBasicAudio/IBasicVideo...); (2) PSEH usage (`${PSEH_LIB}`) — SEH semantics under DUMMY_PSEH; (3) RPC proxy/typelib (PROXY_DELEGATION) generated marshalling glue; (4) worker threads + state machines per filter; (5) vmr9.c (103K) D3D video mixing renderer; (6) pervasive CONST_VTABLE FIXME (vtable layout assumptions).
- **Difficulty:** Very High.
- **Verdict:** LOW (strongly). Single largest, most COM/RPC/SEH-entangled module in the family, pure desktop multimedia.

---

### amstream — dll/directx/wine/amstream (2858 LOC)
Wine MultiMedia Streams (AMSTREAM.DLL, Christian Costa). Real Wine implementation — mediastream.c (52K) is the bulk; full IAMMultiMediaStream/IMediaStream/IAudioData COM filters linking strmbase/strmiids/wine. Not native, not boot-relevant. Verdict: LOW (COM media-stream vtables, on-demand only).

### devenum — dll/directx/wine/devenum (2390 LOC)
Wine Device Enumeration (John Hohm/Robert Shearman) — DirectShow system device enumerator + moniker/property-bag classes (createdevenum.c 28K, mediacatenum.c 30K). Real Wine COM impl linking strmiids/uuid/wine; `set_wine_module_FIXME`. Used by capture/filtergraph device discovery; not boot path. Verdict: LOW (moniker/enumerator COM, no value).

### dmusic — dll/directx/wine/dmusic (4328 LOC)
Wine DirectMusic core (Rok Mandeljc) — ports, instruments, DLS collection loader, synth clock (port.c 41K, collection.c 27K, dmusic.c 19K, instrument.c 15K). Full Wine COM impl linking dxguid/uuid/wine. Music synthesis/sequencing, no consumer in the target, not boot-relevant. Verdict: LOW.

### dpnhpast — dll/directx/wine/dpnhpast (85 LOC)
DirectPlay NAT Helper (Past). **Pure forwarder/stub:** main.c is the entire DLL — DirectPlayNATHelpCreate returns E_NOTIMPL, DllGetClassObject returns E_FAIL, DllRegisterServer/Unregister return S_OK FIXME stubs. Wine-origin but essentially empty. Trivial to "translate" (~5 stubs) but ZERO value — it does nothing. Verdict: LOW / skip (it's already a no-op).

### dxdiagn — dll/directx/wine/dxdiagn (2494 LOC)
Wine DxDiag provider (Raphael Junqueira) — IDxDiagProvider/IDxDiagContainer building the diagnostic info tree. provider.c (71K) is almost the whole module: enumerates system/display/sound/input info into a property container. Full Wine COM impl linking strmiids/dxguid/uuid/wine. Diagnostics tool, on-demand only. Verdict: LOW.

### msdmo — dll/directx/wine/msdmo (1138 LOC)
DirectX Media Object runtime/registry. **Mixed ReactOS-native + Wine:** dmoreg.c (25K, DMO registration/enum) + dmort.c (5K, Mo* media-type helpers) are Wine; **stubs.c is ReactOS-native** (UNIMPLEMENTED DMOGuidToStr/StrToGuid using reactos/debug.h, not Wine). Links `uuid wine mediaobj_guid`. The Mo* media-type helpers (MoCreate/Copy/Free/InitMediaType) are small leaf functions — the only mildly tractable bit, but still no boot relevance. Verdict: LOW (note the native stubs.c if ever touched — keep it C/native, don't Wine-ify).

### qedit — dll/directx/wine/qedit (3183 LOC)
Wine DirectShow Editing Services (Google/Lei Zhang) — Sample Grabber filter (samplegrabber.c 38K), DES timeline (timeline.c 32K), MediaDet (mediadet.c 17K). Full Wine COM filter impl linking strmbase/strmiids/uuid/wine; `set_wine_module_FIXME`. Editing/grabber filters, not boot path. Verdict: LOW.
