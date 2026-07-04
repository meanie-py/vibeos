# explorer.exe → Rust Translation Plan

**Status:** ACTIVE (2026-07-04). util.cpp swap BOOTED (i386 smoke green); scanner
extracts C++ methods. See the Status Log at the bottom for the current edge.
*(§1–§6 below are the 2026-07-03 scoping — still the design of record, but two
facts have moved: the scanner now extracts C++ methods (§2's "blindness" and
§5's tier-B/C extraction blocker are FIXED), and the tier-A substrate
(`explorer-sys`) + cohab link exist and boot. Tier C's remaining blocker is the
COM-vtable/msgmap *object* design, not extraction.)*

**TL;DR / GO-NO-GO:**
- **Tier A (6 procedural files, ~1.3K LOC): GO today** after a small, bounded
  substrate prep. One file (`util.cpp`) already translated for $0.09 — output is
  ~70% useful, same ballpark as HAL.
- **Tier B (6 OO-but-tractable files, ~2K LOC): CONDITIONAL** — needs the COM
  substrate + a per-class cohab recipe first; translation quality unproven.
- **Tier C (10 ATL/WTL/COM files, ~11.9K LOC = 78% of the code): NO-GO today.**
  rustconv has no `BEGIN_MSG_MAP`/`BEGIN_COM_MAP`/`CWindowImpl` expansion path; the
  per-function C path emits garbage on these. This tier is the real project and
  needs a pipeline extension (or hand-translation).

---

## 1. Source inventory & tier triage

`base/shell/explorer/`: 22 `.cpp` + 4 `.h`, 15,240 LOC. Tiers below are by
translation tractability with **today's** rustconv (the proven "C-as-C++" L2 path:
`extern "C"` free functions, per-function translation — precedent `uniata`/`udfs`).

"msgmap/COM macros" = count of `BEGIN_MSG_MAP|MESSAGE_HANDLER|BEGIN_COM_MAP|
COM_INTERFACE_ENTRY|STDMETHODIMP|CWindowImpl|CComObject|DECLARE_|CHAIN_MSG_MAP|
MSG_WM_|…`. "OO methods" = out-of-line `Class::method(` defs (class body in a `.h`).

| File | LOC | macros | OO | Tier | Notes |
|---|---:|---:|:--:|:--:|---|
| **util.cpp** | 239 | 0 | no | **A** | ✅ TRANSLATED (PoC). 10 free fns. 1 COM call (`IsSameObject`→`QueryInterface`). |
| settings.cpp | 98 | 0 | no | **A** | Reg get/set helpers. |
| startup.cpp | 592 | 0 | no | **A** | RunOnce/startup-folder launcher. Largest procedural; pulls shell32 COM (`IShellDispatch`). |
| rshell.cpp | 203 | 0 | no | **A** | ReactOS-shell DLL loader / fallbacks. |
| shellservice.cpp | 141 | 0 | no | **A** | Starts shell services (SENS/hotplug). |
| notifyiconscust.cpp | 50 | 0 | no | **A** | Tiny; notify-icon customization glue. |
| desktop.cpp | 207 | 2 | 1 cls | **B** | `CDesktopThread` small class + `DesktopThreadProc`. |
| explorer.cpp | 296 | 1 | 1 cls | **B** | `_tWinMain` + `CExplorerModule` (ATL module). **Entry point** — needed to actually own the process. |
| startmnucust.cpp | 308 | 0 | 2 cls | **B** | Start-menu customize dialog; inline methods. |
| startmnu.cpp | 111 | 7 | — | **B** | Small; a few `MESSAGE_HANDLER`s. |
| appbar.cpp | 612 | 0 | 56 :: | **B/C** | `CAppBarManager` out-of-line methods (class in `appbar.h`). No COM/msgmap → tractable OO but large. |
| traydeskbtn.cpp | 395 | 0 | 41 :: | **B/C** | `CTrayShowDesktopButton` out-of-line methods. |
| startctxmnu.cpp | 263 | 11 | — | **C** | `IContextMenu` COM. |
| trayprop.cpp | 391 | 14 | — | **C** | Taskbar property sheet (dialog + `IShellPropSheetExt`-ish). |
| startmnusite.cpp | 374 | 25 | — | **C** | `IBandSite`/`IOleCommandTarget` COM site. |
| trayclock.cpp | 786 | 25 | — | **C** | `CWindowImpl` clock control, dense msgmap. |
| tbsite.cpp | 748 | 29 | — | **C** | `ITrayBandSite` COM. |
| trayntfy.cpp | 558 | 30 | — | **C** | Notification area (`CWindowImpl` + COM). |
| syspager.cpp | 1630 | 31 | — | **C** | Systray icon pager. **Monster.** |
| taskband.cpp | 351 | 35 | — | **C** | `IDeskBand`/`IObjectWithSite` — densest COM per LOC. |
| taskswnd.cpp | 2299 | 37 | — | **C** | Task-switch / window list. **Monster.** |
| traywnd.cpp | 3762 | 88 | — | **C** | The taskbar core (`CTrayWindow`, 40+ msg/COM handlers). **Mega-monster.** |

**Totals:** Tier A = 6 files / ~1.3K LOC · Tier B = 4–6 files / ~2K LOC ·
Tier C = 10 files / ~11.9K LOC (**78% of the code, and 100% of the hard part**).

---

## 2. util.cpp PoC — what the existing pipeline does on a real explorer file

**Pipeline reach (all FREE stages):**
- `scan` — ✅ clean. Extracts all 10 functions with correct signatures/line ranges.
  The C++ tree-sitter front-end handles this procedural file fine.
- `resolve` / `prompt` / `translate` — ❌ **failed initially**: explorer was not a
  registered rustconv *target*, so `detect_target` errored. **Fixed in this lane**
  (`tools/rustconv/rustconv/config.py`): added an `explorer` Target (winsrv-sys
  substrate, generic user-mode addendum — modeled on `userinit`/`winlogon`) + a
  `/base/shell/explorer` branch in `detect_target`. After that, `resolve` builds the
  catalog (23K winsrv symbols), `prompt` builds a 1081-line prompt, `estimate` works.
  Run rustconv against the worktree with `PYTHONPATH=$W/tools/rustconv` (pip -e
  shadows to the main checkout otherwise).
- `estimate` — util.cpp: ~9.5K in / 5.7K out; Sonnet sync ~$0.11 (@5x ~$0.57).
  **Gap:** `estimate base/shell/explorer/` (directory) returns **0 files** — the
  batch/dir file-discovery does not enumerate `.cpp`. Per-file works; bulk `batch`
  would need a discovery fix (or drive it file-by-file).

**Paid PoC:** `translate base/shell/explorer/util.cpp` → **$0.09**, 10/10 functions,
assembled to `sdk/lib/rust/explorer-rust/src/explorer/util.rs`. Archive:
`tools/rustconv/.responses/20260703-234423-util.cpp`.

**Rust quality — ~70% useful (matches the HAL "review before integrating" bar).**
- ✅ Control flow faithful; `#[no_mangle] extern "C"` symbols preserved (cohab-ready
  for the free-function files). Registry/menu/version API calls mapped sensibly.
  `IID_PPV_ARG` macro hand-expanded; COM `QueryInterface`/`Release` recognized.
- ❌ **Won't compile as-is.** Concrete defects (representative of the whole tier):
  1. **Undeclared file-scope globals** referenced: `hkExplorer`, `hProcessHeap`.
     These are explorer globals (in `precomp.h`/`explorer.cpp`) — need an `extern`
     block or a Rust home. **Every OO file will hit this** (shared `g_*` statics).
  2. **Inconsistent wide-string literals** — the real correctness risk. Some are
     correctly interleaved UTF-16 (`b"L\0o\0g\0..."`), but `GetAdvancedBool` and the
     `VerQueryValueW` sub-blocks emit **narrow** `b"Software\\..."`/`b"\\StringFileInfo..."`
     cast to `LPCWSTR` — wrong encoding, silent bug.
  3. **Missing extern decls** for non-substrate APIs: version.dll
     (`GetFileVersionInfoSizeW`, `VerQueryValueW`) and shlwapi (`wnsprintf`) are
     **not in winsrv-sys**; only 3 externs were emitted. `LOBYTE` used as a fn (C macro).
  4. **`IUnknown`/`IID_IUnknown` don't exist in the substrate** — `IsSameObject`
     calls `(*punk).QueryInterface(...)` as if IUnknown were a Rust type with methods.
     It isn't (see §3). Even this "procedural" file has a COM dependency.
  5. **Variadic** `FormatMenuString` → `mut args: ...` + `libc::vswprintf` — needs
     `#![feature(c_variadic)]` and `libc`; not wired for this target.

**Takeaway:** the *scanner/prompt/translate* mechanics work on procedural explorer
files today (with the target registration done here). The blocker is **substrate**,
not the front-end — the translated code references COM/Win32/global symbols that the
winsrv-sys catalog doesn't provide.

---

## 3. COM / WTL → Rust approach

**Decision: hand-rolled `#[repr(C)]` vtable structs, NOT windows-rs.** windows-rs
assumes std + its own COM runtime and does not fit our no_std, cohab, per-arch
PE-COFF link. We already have a **working in-tree precedent**:
`sdk/lib/rust/ks-rust/src/filterfactory.rs` implements a COM interface the C way —

- `#[repr(C)] I…Vtbl { QueryInterface: fn-ptr, AddRef, Release, … }` (bindgen-shaped),
- a `static mut vt_…: I…Vtbl` populated with `core::mem::transmute`d `extern "system"`
  handlers,
- objects whose **first member is `lpVtbl`**, dispatched as
  `(*(*iface).lpVtbl).Method.unwrap()(iface, …)`.

This is exactly the ABI ATL's `CComObject`/`BEGIN_COM_MAP` compiles down to, so a
Rust object with the same `#[repr(C)]` `lpVtbl`-first layout is drop-in for shell32/
comctl32 callers. **This is the model for Tier B/C.**

What's missing (must be built — none exists in the Rust tree today; grep found no
`IUnknown`/`IID_IUnknown`/`IUnknownVtbl`):
1. **`explorer-sys` FFI crate** (mirror `winlogon-sys`): `#[repr(C)]` vtable +
   object-header types for `IUnknown` and the shell interfaces explorer implements/
   consumes (`IShellDesktopTray`, `ITrayWindow`, `IDeskBand`, `IBandSite`,
   `IContextMenu`, `IOleWindow`, `IOleCommandTarget`, `IShellDispatch`, …), the
   `IID_*` GUID constants, and the version.dll/shlwapi/shell32/comctl32 externs the
   files call. Size-assert per arch (×3) like winlogon-sys.
2. **`BEGIN_MSG_MAP` → Rust.** ATL message maps are the WTL half. There is **no
   translation support** for them — a `CWindowImpl` message map must become a Rust
   `match uMsg { WM_… => … }` window-proc. rustconv's per-function path cannot do
   this; it's a macro/class-shaped transform. Options: (a) extend rustconv with an
   ATL-aware C++ pass that lowers msg/COM maps to Rust dispatch, or (b) hand-translate
   the window procs. Given only ~13 files need it, **(b) hand-translation of Tier C,
   assisted by per-method `translate` for the handler bodies, is likely cheaper than
   building (a)** — but that's a real, multi-week effort.

---

## 4. Crate & link plan (cohab, like HAL)

- **New crate `sdk/lib/rust/explorer-rust/`** (scaffolded: the PoC wrote
  `src/explorer/util.rs`). Model on `userinit-rust`/`winlogon-rust`: winsrv-sys
  substrate + `explorer-sys` (new, §3), builds a `libexplorer_rust.a` for all 3 arches.
- **Cohab granularity differs by tier:**
  - **Tier A (free functions):** per-**file**, per-**symbol** cohab exactly like HAL.
    Comment the `.cpp` out of `base/shell/explorer/CMakeLists.txt` with a
    `# replaced by Rust (…)` line; the `#[no_mangle] extern "C"` symbol takes over;
    `--allow-multiple-definition` during swap-in (lld first-match wins).
  - **Tier B/C (classes):** cohab is per-**class / whole-object**, NOT per-symbol — a
    C++ object's vtable cannot be split across languages. A `CTrayWindow` is either
    fully C++ or fully Rust. So each monster is an atomic swap once its whole class +
    all its interfaces are translated. Its C++ mangled method names never cohab
    individually.
- **Build integration:** add `explorer-rust` (+ `explorer-sys`) as workspace members;
  add the CMake `-rust` target + link `libexplorer_rust.a` into `explorer.exe`
  (same pattern as winlogon/userinit). No shared-file edits done in this scoping lane.
- **Validation = the smoke North Star.** explorer.exe *is* the boot marker
  (`[EXPLORER] _tWinMain entered` + `shell ready`). Any swap is validated by the
  existing smoke ×3 — a Rust explorer that boots to desktop is the proof.

---

## 5. Recommended ordering & cost

**Order: substrate → leaf-procedural → OO → COM monsters last.**

1. **Substrate prep (no per-file API $):** build `explorer-sys` (IUnknown + the
   handful of interfaces/IIDs the leaf files touch, version/shlwapi/shell32 externs,
   the `g_*`/`hk*` globals home). This unblocks Tier A actually *compiling*.
2. **Tier A (6 files, ~1.3K LOC):** `util.cpp` done; finish settings, startup, rshell,
   shellservice, notifyiconscust. Per-file est. Sonnet sync ~$0.10–0.15
   (@5x ~$0.5–0.75). **Whole tier ≈ $3–4 sync** incl. re-runs. Each needs manual
   review (~70% quality). Land one file first (`util.cpp`), prove it links + boots.
3. **Tier B (4–6 files, ~2K LOC):** after the per-class cohab recipe is proven. Start
   with `explorer.cpp` (own the entry point) + `desktop.cpp`.
4. **Tier C (10 files, ~11.9K LOC):** the ATL/WTL monsters. **Blocked on §3** (msg-map
   lowering). Order within: leaf controls (`trayclock`, `traydeskbtn`) → COM sites
   (`taskband`, `tbsite`, `startmnusite`) → the two big ones (`taskswnd`, `syspager`)
   → `traywnd` (mega, last — everything else plugs into it). Cost here is dominated by
   per-function fan-out on 3–4K-LOC files at 2–5×: realistically **$15–40 sync** for
   the tier *if* rustconv could do it — but it can't yet, so this is a
   hand-translation budget, not an API budget.

**Honest split of blocked-vs-doable:**
- **Doable today:** target registration (done), Tier A translation once `explorer-sys`
  exists. `explorer-sys` itself is hand-written FFI (bindgen-assisted), not API $.
- **Blocked on substrate:** Tier A *compiling* (needs `explorer-sys`).
- **Blocked on pipeline:** Tier B per-class cohab recipe; **all of Tier C** (no ATL
  msg-map/COM-map translation path — the 78%).

---

## 6. Single highest-value next step

**Build `explorer-sys` (IUnknown vtable types + IIDs + version/shlwapi/shell32 externs
+ the shared globals) and wire `explorer-rust` as a cohab into `explorer.exe`, then
land `util.cpp` alone and prove smoke ×3 still boots to desktop.** That closes the
loop end-to-end (translate → substrate → cohab → link → boot) on the cheapest possible
surface, and is the prerequisite for everything else. Only after that green should any
further explorer files be translated (and Tier C not until the msg-map path exists).

---

## Status Log

- **2026-07-03** — Scoped (this doc). util.cpp PoC translated ($0.09). `explorer`
  target registered in rustconv config.
- **2026-07-04 — util.cpp swap BOOTS.** `explorer-sys` created (com::IUnknown
  lpVtbl types + IID_IUnknown + S_OK/S_FALSE, shlwapi/version.dll/msvcrt externs,
  cohab globals extern block) and `explorer-rust` staticlib (both in the cargo
  workspace, `cargo check` clean ×3 arches). util.rs integration fixes vs the raw
  translation: BOOL returns u8→i32 (ABI), narrow-string→`w!` (encoding bug),
  variadic `FormatMenuString` kept as C++ `va_start` shell (`util_vararg.cpp`) →
  Rust `FormatMenuStringV` (stable Rust can't define C-variadic; rtl-diag
  pattern). Cohab: util.cpp commented out of CMakeLists (advapi32-rust cmake
  pattern: cargo custom command + `--allow-multiple-definition`, rust lib first),
  util decls `extern "C"` in precomp.h. **i386 smoke green (1m19s, boot ✓).**
  amd64/arm64 owed at the landing gate (amd64 baseline is ~50% red from an
  unrelated Mm-wedge — another lane owns it).
- **2026-07-04 — scanner extracts C++ methods** (user-directed "improve rustconv
  rather than gate"). `_parse_function` now handles `qualified_identifier`
  (out-of-line `Cls::m`, incl. glued STDMETHODCALLTYPE), `field_identifier`
  (in-class), `destructor_name`, ctor detection, and ascends to the enclosing
  class/struct for attribution; operators stay unextracted. Explorer coverage:
  taskband 2→24, appbar 0→29, traywnd 21→149 (4 macro-mangled leftovers),
  taskswnd 6→91, syspager 4→62; kernel .c files unchanged. `FunctionDep.name`
  for methods is the unique target symbol (`Cls_method`, `Cls_ctor`/`_dtor`,
  `_2` overload suffix) = the Rust `#[no_mangle]` name. Per-method prompts get a
  method→free-fn contract block + the owning class definition with bodies
  stripped (`extract_class_context`, sibling-.h aware). Fail-loud gate:
  `translate`/`submit`/`batch` HARD FAIL on unextracted bodies in `.cpp`
  (`--allow-partial` to override), warn-only on `.c` (PSEH parse artifacts were
  always dropped; kernel pipeline stays unblocked). `estimate`/dir discovery now
  globs `.cpp` (full dir: ~$3.4 Sonnet sync floor). rustconv pytest 78/78 green.
  **Translation quality of method bodies is still unproven** — first live test =
  a tier-B file (settings/appbar); the Rust-side `#[repr(C)]` class structs must
  be authored/verified per class before its methods can link.
- **2026-07-04 — Tier A translated + 6/7 files integrated** (total API spend
  ~$0.42). explorer-sys grew the consumed-COM surface: vtables (orders verified
  from psdk/reactos idl+headers via subagent research) for IMenuPopup chain /
  IBanneredBar / IMenuBand / IInitializeObject / IBandSite / IOleCommandTarget /
  ITrayWindow / IShellDesktopTray, IID/CLSID extern statics resolved from the
  cmake `uuid` lib at link (verified real data symbols; IID_IShellDesktopTray
  comes from traywnd.cpp), DPA_*, SHRestricted + REST_* values, strsafe/
  HRESULT_FROM_WIN32/MAKEINTRESOURCEW/DialogBoxParamW gap-fill, wine TRACE!/
  WARN!/ERR!/FIXME! no-op macros. precomp.h: extern "C" on all swapped decls +
  TrayProcessMessages/TrayMessageLoop/CStartMenuSite_CreateInstance (called
  from Rust). Raw-output defect classes fixed in integration (add to review
  checklist): u8-vs-BOOL ABI on exported fns; narrow-vs-wide literals; zeroed
  GUID stubs (CLSID_BandSiteMenu was [0,0,0,0] — silent bug); resource IDs
  stubbed to 0 (IDD_NOTIFICATIONS_CUSTOMIZE); DialogBoxW/MAKEINTRESOURCEW
  macros emitted as externs; FARPROC-Option call shapes. cargo check 0 errors
  ×3 arches; notifyiconscust/rshell/shellservice/startmnu/startup .cpp swapped
  out in CMakeLists. **startmnucust.rs translated but NOT integrated** (needs
  TaskbarSettings/TW_STUCKRECTS2 bitfield mirror + treeview substrate;
  startmnucust.cpp stays compiled — phased swap, tracked).
- **2026-07-04 — i386 LINK gotcha (cargo-check can't catch it).** The 6-file
  tier-A livecd link FAILED with 8 undefined symbols (`_SHGetFolderPathW`,
  `_ExpandEnvironmentStringsW`, `_ShellExecuteExW`, `_IsUserAnAdmin`,
  `_GetBinaryTypeW`, `_SUCCEEDED`, `_StringCbCatW`, `_ZeroMemory`). Root cause:
  rustconv emits Win32 API imports as `extern "C"` (cdecl) blocks; on i386 that
  decorates `_Name` but the import libs export stdcall `_Name@N` → unresolved.
  cargo-check on the rlib PASSES (the reference is unresolved-but-legal until the
  final exe link), so **the defect only appears at the explorer.exe link — always
  do a targeted `ninja base/shell/explorer/explorer.exe` after integrating a
  file, not just cargo check.** Fix: Win32 stdcall APIs → `extern "system"`;
  drop re-declared inlines (`SUCCEEDED` from winsrv_sys::consts, `StringCbCatW`
  from explorer_sys::externs); `ZeroMemory` is a macro → `RtlZeroMemory`; keep
  genuine cdecl (`wcslen`/`wcscmp`) and DCE'd wine `TRACE`/`WARN` as `extern "C"`.
  amd64/arm64 don't care (cdecl==stdcall decoration), so this is i386-only — same
  family as the kernel's #1 i386 crash cause. Every tier-B/C file has this same
  `extern "C"` Win32 block and will need the fix at integration.
- **2026-07-04 — first TIER-B file integrated: settings.cpp (the shared-global +
  method-shim pattern, the template for ALL tier C).** settings.cpp defines
  `g_TaskbarSettings` (read by 8 still-C++ files) and `TaskbarSettings::Save/Load`
  (called mangled from explorer.cpp wWinMain + traywnd.cpp). The cohab recipe,
  proven end-to-end in the i386 binary:
  1. **Shared mutable global**: Rust `#[no_mangle] pub static mut g_TaskbarSettings:
     explorer_sys::types::TaskbarSettings` (exact C++ layout). Itanium ABI leaves
     namespace-scope globals unmangled, so the 8 C++ `extern TaskbarSettings
     g_TaskbarSettings;` referencers bind to the one Rust `_g_TaskbarSettings`
     data symbol. Confirmed via nm.
  2. **Mangled method entry points**: a tiny `settings_shim.cpp` defines
     `TaskbarSettings::Save/Load` (the `__ZN15TaskbarSettings4SaveEv` symbols the
     C++ callers need) forwarding to the flat Rust `TaskbarSettings_Save(this)`.
     Same shim pattern as util_vararg.cpp — the general answer for "C++ method
     called from still-C++ code."
  Integration fixes on the raw translation: the LLM FABRICATED the TaskbarSettings
  struct layout in the preamble global (no header → hallucinated
  ScreenResolution/ShellSettings fields) — replaced with the real
  explorer_sys::types mirror; bitfield writes (`.sr.SmSmallIcons = 0`) → the
  `dwFlags` word (`= F_ALWAYSONTOP`); `dwValue as TrayIconsMode` (not a type) →
  `as i32`; added shlwapi `SHGetValueW`/`SHSetValueW` externs (LSTATUS typed as
  DWORD so `== ERROR_SUCCESS` needs no cast). cargo-check + explorer.exe link
  clean ×3. **The fabricated-struct-when-header-absent defect is systemic** — every
  tier-B/C class whose fields the LLM can't see will hallucinate; the fix is
  authoring the real `#[repr(C)]` mirror (or, better, feeding the class header to
  the translator — the scanner's extract_class_context already does this for
  methods, so a preamble-context extension would close it).
- **2026-07-04 — second tier-B file: desktop.cpp (CDesktopThread + the
  arch-split callback-ABI pattern every tier-C proc needs).** desktop.cpp runs
  the desktop's own message-loop thread; DesktopCreateWindow/DestroyShellWindow
  are its extern "C" entry points (no shim — free functions, not methods).
  Reusable fixes:
  1. **C++ `new`/`delete` of a now-Rust class** → Rust `CDesktopThread_new` =
     HeapAlloc + `_ctor`, `_delete` = `_dtor` + HeapFree (the raw translation
     used `Box::from_raw` — std, and mismatched the allocator: a UAF/no_std
     bug). The operator-new/delete template for any heap-allocated tier-C class.
  2. **Callback ABI is arch-split and needs cfg.** A fn passed to CreateThread
     (or SetTimer/RegisterClass) must MATCH winsrv's bindgen fn-pointer type
     EXACTLY, and bindgen spells it `extern "C"` on amd64/arm64 but
     `extern "stdcall"` on i386 — and Rust treats "C"/"system"/"stdcall" as
     distinct TYPES even where the ABI coincides, so no single spelling
     type-checks on all three. Pattern: `#[cfg(not(target_arch="x86"))] extern
     "C"` + `#[cfg(target_arch="x86")] extern "stdcall"` on the proc (thin
     trampoline into a shared worker). **Every tier-C WndProc / timer / thread
     proc hits this** — the flat match-based WndProc that replaces
     BEGIN_MSG_MAP is exactly such a callback.
  3. Local redefs with ZEROED IIDs: the translation re-declared IShellDesktopTray
     + IID_IShellDesktopTray locally with an all-zero GUID (QI-always-fails bug)
     — deleted; use explorer_sys::com. Added E_FAIL/E_INVALIDARG/… to the
     substrate. Links + boots i386.

## Refined remaining-work map (2026-07-04, after clearing the self-contained files)

8/22 files are Rust + booting i386 (util, notifyiconscust, rshell, shellservice,
startmnu, startup, settings, desktop) across 3 committed checkpoints. The
remaining 14 all need MORE than the tier-A/B patterns — re-triaged by their
actual entanglement:

- **appbar.cpp** — `CTrayWindow : public CAppBarManager`, so CAppBarManager is a
  BASE CLASS of the still-C++ CTrayWindow. Its methods are Rust-translated but
  the class LAYOUT must stay in appbar.h (C++ needs it to lay out CTrayWindow),
  and C++ needs mangled entry points for the ctor, the *virtual* dtor, and
  OnAppBarMessage (the 3 things CTrayWindow touches). New pattern: **Rust method
  bodies on a C++-inherited base subobject** (appbar_shim.cpp with 3 forwards).
  Blocked-ish: cleanest once CTrayWindow itself is Rust. Translated, inert.
- **trayprop.cpp** — 3 `CPropertyPageImpl<>` WTL classes (message maps + dialog
  procs). Needs the WTL substrate (below), not a tier-B pattern. One clean entry
  point (`DisplayTrayProperties`, called from traywnd.cpp:562). Translated, inert.
- **explorer.cpp** — wWinMain (the process ENTRY POINT) + `CExplorerModule :
  CComModule` (ATL module). Self-contained but highest-risk (owns the process;
  if wrong, nothing boots). Translated, inert. Do with care, probably last of
  tier B, or after a couple tier-C wins build confidence.
- **Tier C (10 files: traydeskbtn, startctxmnu, startmnusite, taskband, trayntfy,
  tbsite, trayclock, syspager, taskswnd, traywnd)** — all ATL/WTL COM servers.
  NOT yet translated (a ~$3–4 paid run once the substrate exists). Blocked on the
  one genuinely-new piece:

### The tier-C COM-server / WTL substrate (the last unbuilt foundation)

Each tier-C class is `CComObjectRootEx + CWindowImpl<T> + CComCoClass + I<iface>`.
Replacing ATL (per the decision in §3, hand-rolled #[repr(C)], no windows-rs):
- **CComObjectRootEx → refcount+QI**: object header `{ lpVtbl, ref_: LONG, … }`;
  AddRef/Release interlocked on ref_; a per-class QueryInterface walking a static
  IID→interface-pointer table (the BEGIN_COM_MAP replacement). ks-rust
  `filterfactory.rs` is the in-tree precedent to mirror.
- **CWindowImpl → WndProc**: `Create()` registers the window class once + creates
  the HWND; a per-class `WndProc` recovers `this` from GWLP_USERDATA (set on
  WM_NCCREATE) and dispatches `match uMsg { … }` — the flat replacement for
  BEGIN_MSG_MAP/MESSAGE_HANDLER (a Rust match is *more* natural than the macro).
  **The WndProc is an arch-split callback** → the exact
  `#[cfg] extern "C"/"stdcall"` pattern desktop.cpp already proved.
- **CreateInstance factory**: `ShellObjectCreatorInit<T>` → HeapAlloc the object,
  set lpVtbl + ref_=1, call Initialize(param), QI to riid (the new/delete pattern
  desktop.cpp already proved).

So tier C's hard parts (WndProc ABI, new/delete, QI vtables) are each already
proven in isolation; the remaining work is assembling them into a reusable
`explorer_sys::server` module + one proof file (**trayclock.cpp** — smallest full
CWindowImpl+CComObjectRootEx+IOleWindow), then grinding the other 9 (traywnd
last — everything plugs into it).

### 2026-07-04 — tier-C substrate BUILT + first class (trayclock) BOOTS i386

`explorer_sys::server` exists and is proven end-to-end (`0beb484e653`):
- `register_class_once` (DECLARE_WND_CLASS_EX), `set/get_window_this` +
  `wndproc_nccreate_this` (GWLP_USERDATA `this`-recovery — the CWindowImpl thunk
  equivalent, chosen over ATL's machine-code thunk per the substrate research;
  ARCH-SPLIT because i386 `SetWindowLongPtrW` is a macro→`SetWindowLongW` with no
  import), `interlocked_increment/decrement` (core-atomic; `Interlocked*` are
  intrinsics never in bindgen), uxtheme/ShellExecuteW/CTooltips-TTM surface +
  theme/tooltip/locale constants. `IOleWindow` vtable + `IID_IOleWindow` added to
  `explorer_sys::com`; `GetHideClock` → settings.rs (owns g_TaskbarSettings).
- **trayclock.rs is the TEMPLATE** for the other tier-C classes: flat
  `#[repr(C)]` lpVtbl-first object (`lpVtbl`/`ref_`/`m_hWnd` + members; layout is
  FREE — the object is only ever touched via its interface ptr or its WndProc, so
  ATL's multi-base layout is irrelevant); QI walks {IUnknown,IOleWindow}→the one
  vtbl; `BEGIN_MSG_MAP`→flat `match uMsg` in an arch-split `extern "C"/"stdcall"`
  WndProc (verified: links `_CTrayClockWnd_WindowProc@16` = correct i386 stdcall);
  `ShellObjectCreatorInit<T>`→the factory (new ref=0 → AddRef → Initialize(args)
  → QI → Release). cargo-check ×3 clean; explorer.exe links; **i386 smoke ✓ boot**.

**Entanglement re-triage (which files are ATOMIC swaps vs blocked on traywnd):**
A tier-C class is an independent atomic swap **iff** it's reached only through a
`*_CreateInstance` COM factory (never embedded as a C++ member/base of a still-C++
class). By that test:
- **INDEPENDENT (fan-out targets, any order, traywnd last):** trayclock ✅DONE ·
  trayntfy (`CTrayNotifyWnd_CreateInstance`) · syspager (`CSysPagerWnd_…`) ·
  taskswnd (`CTaskSwitchWnd_…`) · taskband (`CTaskBand_…`) · tbsite
  (`CTrayBandSite_…`) · startctxmnu (`CStartMenuBtnCtxMenu_…`) · startmnusite
  (`CStartMenuSite_…`).
- **ENTANGLED — embedded in the still-C++ CTrayWindow, so BLOCKED until traywnd
  is Rust (or need the appbar-style mangled-method-shim):** traydeskbtn
  (`CTrayShowDesktopButton` is a *member* of CTrayWindow — plain CWindowImpl, no
  factory) · appbar (`CAppBarManager` is a *base* of CTrayWindow) · CStartButton
  (member, lives in traywnd.cpp).

**Per-file cost is dominated by central substrate expansion, not the class body:**
each independent file needs its consumed/implemented COM interface vtables added
to `explorer_sys::com` (e.g. startctxmnu → IShellFolder + IContextMenu + the PIDL/
shell32 surface + CMINVOKECOMMANDINFOEX) — shared-file work that must be done
centrally (subagents can't touch the substrate). The class body itself follows the
trayclock template mechanically. Recommended order: trayntfy (consumes the Rust
clock — validates the Rust-from-Rust layer) → the COM-only sites (taskband,
tbsite, startmnusite, startctxmnu) → the monsters (syspager 1.6K, taskswnd 2.3K)
→ traywnd (3.7K, last — unblocks the 3 entangled files).
