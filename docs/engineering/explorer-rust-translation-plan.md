# explorer.exe → Rust Translation Plan

**Status:** SCOPING (2026-07-03). No explorer code translated to Rust before this
lane except the single PoC below (`util.cpp`). explorer.exe is the last boot-path
component still fully C++ (`docs/engineering/rust-port-status.md` lists it at 0%).

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
