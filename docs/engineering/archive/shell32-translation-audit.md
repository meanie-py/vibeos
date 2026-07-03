# shell32.dll — C→Rust Translation Audit

**Lane:** `shell32` (user32 agent). **Pattern:** per-file cohab flip into a `shell32-rust`
staticlib (`libshell32_rust.a`), mirroring `gdi32-rust` / `user32-rust` / `kernel32-rust`.
**Boot-critical:** yes — explorer.exe links shell32, so the full ×3 boot gate exercises it.

## Scope

shell32.dll source splits sharply in two:

| Kind | Files | LOC | In this lane? |
|------|------:|----:|---------------|
| **wine-derived pure C** (`.c`) | 14 | ~16K | ✅ **yes** — the C→Rust pipeline target |
| **COM vtable classes** (`.cpp`) | 81 | ~64K | ❌ not yet — separate, much heavier effort (ATL/COM C++) |

The `.cpp` files (CDefView / CShellLink / CDefaultContextMenu / folders/* / droptargets/* …)
are ATL-based COM object implementations — out of scope for the C→Rust pipeline. This lane
translates the 14 `.c` files. Imports resolve at runtime via shell32's existing import libs
(`advapi32 gdi32 user32 comctl32 comdlg32 shlwapi msvcrt kernel32 ntdll` + the delay set),
so translated bodies just declare `extern "system"`/`extern "C"` for what they call — no
bindgen `*-sys` substrate needed yet (a `shell32-sys` may be added for the PIDL waves).

## Key risk (per-file, must check during translation)

Many `.c` files call **internal shell32 helpers defined in the `.cpp` side** (e.g. PIDL
helpers `_ILCreate*`, `SHELL_*`). For a cohab flip to link, those callees must have **C
linkage** (declared in `shell32_main.h`/`pidl.h` as plain C, `extern "C"` from the C++ TU).
A `.c` body that calls a **C++-name-mangled** helper will fail to link after the flip. Verify
each file's external callees resolve to C symbols (`nm` the pre-flip `.obj` for the `U` set)
before flipping — same discipline as the gdi32 `GdiEntry16`/`EMFDC_GdiComment` decoration checks.

## Ledger (14 .c files)

| File | LOC | Exp | COM/PIDL | Status | Wave |
|------|----:|----:|---------:|--------|------|
| `wine/shellreg.c`    |  186 |  8 |   0 | ✅ **Rust** (cohab, boots ×3) | **0** |
| `wine/shpolicy.c`    |  215 |  4 |   0 | ✅ **Rust** (cohab, boots ×3) | **1** |
| `wine/shellstring.c` |  271 |  8 |   3 | ✅ **Rust** (cohab, boots ×3) | **1** |
| `appbar.c`           |  128 |  0 |   0 | ✅ **Rust** (cohab, boots ×3) | **1** |
| `vista.c`            |  218 |  0 |   0 | ✅ **Rust** (cohab, boots ×3) | **1** |
| `wine/clipboard.c`   |  252 |  0 |   8 | ✅ **Rust** (cohab, boots ×3) | **1** |
| `wine/control.c`     | 1260 |  8 |   0 | ✅ **Rust** (cohab, links ×3, i386 boots) | **2** |
| `wine/shellole.c`    |  856 | 28 |   9 | ✅ **Rust** (cohab, links ×3, i386 boots) | **2** |
| `wine/classes.c`     |  657 |  0 |   5 | ✅ **Rust** (cohab, links ×3, i386 boots) | **2** |
| `wine/shell32_main.c`| 1370 | 16 |  18 | ✅ **Rust** (cohab, links ×3; smoke pending) | **3** |
| `wine/shlmenu.c`     |  997 | 25 |  18 | ✅ **Rust** (cohab, links ×3; smoke pending) | **3** |
| `wine/pidl.c`        | 2681 | 81 | 193 | ✅ **Rust** (cohab, links ×3; smoke pending) | **4** (PIDL core) |
| `wine/shellpath.c`   | 3398 | 35 |   9 | ✅ **Rust** (cohab, links ×3; smoke pending) | **4** |
| `wine/shellord.c`    | 3108 | 72 |  51 | ✅ **Rust** (cohab, links ×3; smoke pending) | **4** (ordinal grab-bag) |

**Progress: 14/14 .c (100%) · ~16K / ~16K LOC. 🎉 shell32's entire wine-derived C half is Rust.**
(The only remaining build inputs are the auto-generated `shell32_stubs.c` and the 81 `.cpp`
COM-vtable classes — a separate, much heavier ATL/COM effort, out of scope for this lane.)

### Wave 4 notes (pidl + shellcom; shellpath/shellord in flight)
pidl.c (81 exports — 39 WINAPI `IL*@N`/`SH*@N` + 42 cdecl `_IL*`/`__IL*` internals) is the
PIDL foundation. Its canonical shared types (the packed PIDL on-disk format SHITEMID/ITEMIDLIST/
PIDLDATA + PT_* tags, STRRET, the GUID family, and 11 COM vtables) were promoted into a new
shared module **`crate::shellcom`** so pidl/shellpath/shellord share ONE definition (Wave 0-3
keep their file-local copies — retrofitting them is deferred to avoid destabilizing the landed
waves; the residual cross-file `SHGetDesktopFolder`/`PidlToSicIndex`/`StrRetToStrNW` clash
warnings are benign, ABI-identical). pidl links ×3; the C++-side imports
(IFileSystemBindData_Constructor, BindCtx_*, SHELL32_AliasTranslatePidl) all resolve.

### Wave 4b notes (shellpath + shellord — completes 100%)
2 parallel agents against `crate::shellcom`, integrated centrally; links ×3, all exports match
the pre-flip obj decoration. **shellpath** (35 exports): path manipulation + the 63-entry CSIDL
special-folder table (every `__REACTOS__` CSIDL_* row reproduced with registry value / default
path / resource ID pinned from shlobj.h/shresdef.h; the Wine-only rows 0x3f-0x72 omitted).
**shellord** (72 exports, +`g_hShellState` global): the ordinal grab-bag — DAD_* drag image,
DoEnvironmentSubst, ParseField, SHAddToRecentDocs, SHDoDragDrop, prop-sheet ext arrays,
Read/WriteCabinetState, etc.; honest FIXME-stubs kept as stubs. `ShellMessageBoxA` is a genuine
C-variadic export → the crate gains `#![feature(c_variadic)]` + the cmake custom_command gains
`RUSTC_BOOTSTRAP=1` (same pattern as version-rust/scsiport-rust). Fixed one integration bug: a
local `hkey` var shadowed the `hkey()` helper fn in Read/WriteCabinetState (E0618).

### Wave 3 notes (shell32_main/shlmenu)
2 parallel agents (CONTRACT-3), integrated centrally; links ×3, all 43 exports match the
pre-flip obj decoration. shell32_main models IShellFolder/IShellFolder2/IShellDetails/
IExtractIconW vtables + cfg-split PE32/PE32+ headers (ExtractIcon); CommandLineToArgvW is a
literal port of the quote/backslash state machine. shlmenu reproduces the file-private
FMINFO/FMITEM structs + IShellFolder/IEnumIDList vtables. Both reference the still-C `_IL*`
PIDL helpers (resolve from C pidl.c until Wave 4). **Known benign warnings:** shell32_main +
shlmenu each define a file-local `IShellFolder`, so their `extern` decls of `SHGetDesktopFolder`/
`PidlToSicIndex` clash nominally (ABI-identical — same arg count, all pointer-width; resolve
to the one C symbol). Proper fix = hoist the common COM/PIDL types into a shared module; that
lands with the **shell32-sys** substrate the PIDL Wave 4 needs.

### Wave 1 notes (shpolicy/shellstring/appbar/vista/clipboard)
Translated by 5 parallel agents (CONTRACT-1), integrated centrally, boots ×3.
**🔑 Data-table preprocessor-config bug (caught + fixed):** shpolicy's `s_PolicyTable` is
built by `#include "PolicyData.h"`, whose rows are guarded by `WINE_FILEVERSION_MAJOR`/
`NTDDI_VERSION`/`_SHELL32_`. The agent first preprocessed with `_WIN32_WINNT=0x502`
(NTDDI_WS03) → wrong row set (169). The real shell32 build is `_WIN32_WINNT=0x600`
(NTDDI_WIN6, and `NTDDI_LONGHORN==NTDDI_VISTA==NTDDI_WIN6`, so `< NTDDI_LONGHORN` is FALSE)
**plus `-D_SHELL32_`** → **159 rows**, verified byte-for-byte against the compiled
`shpolicy.c.obj`'s UTF-16 strings (the authoritative ground truth, stronger than a
preprocessor count). Lesson now in CONTRACT-2: translate only the branch active under the
real CMake define set; check `strings -el <obj>` when unsure.

(`shellrecyclebin/recyclebin.c` + `shellrecyclebin/guid.c` build into the separate
`recyclebin` static lib, not shell32.dll's TU — tracked separately if pursued.)

## Wave plan (difficulty-ordered, low COM/PIDL first)

- **Wave 0 — `shellreg.c` ✅ DONE.** Validator (like gdi32 linedda): proved crate scaffold,
  cohab link, i386 stdcall `@N` decoration (`_SHRegOpenKeyA@12`…`_SHCreateSessionKey@8` all
  match the pre-flip obj), and advapi32/shlwapi/kernel32 import resolution. Hand-translated.
- **Wave 1 — leaves (~1.1K LOC):** `shpolicy` (policy table + `SHRestricted`), `shellstring`
  (`StrRetToStr*`/`CheckEscapes*`), `appbar` (`SHAppBarMessage`), `vista` (internal helpers),
  `clipboard` (shell clipboard-format helpers). Low/zero COM. Parallel opus agents + CONTRACT.
- **Wave 2 — mid (~2.8K LOC) ✅ DONE:** `control` (control-panel applet loader; the `*16`
  thunk entries are FIXME-stubs in the C, translated verbatim), `shellole` (OLE/COM glue +
  drag-drop; models IClassFactory/IPropertySetStorage/IPropertyStorage vtables — `SHFree`/
  `SHAlloc` keep `@4` decoration for the clipboard.rs cross-ref), `classes` (HKEY_CLASSES_ROOT
  helpers; models IShellFolder vtable). 3 parallel agents, integrated centrally. Links ×3,
  i386 boots; amd64/arm64 self-smoke pending host.
- **Wave 3 — heavy (~2.4K LOC):** `shell32_main` (DllMain-adjacent main exports), `shlmenu`
  (`Shell_MergeMenus`/FileMenu_*). More internal-helper coupling.
- **Wave 4 — heaviest, PIDL-dense (~9.2K LOC):** `pidl` (the ITEMIDLIST core, 193 PIDL refs),
  `shellpath` (`SHGetFolderPath`/`Path*`), `shellord` (76 ordinal exports). Likely needs a
  `shell32-sys` bindgen substrate (ITEMIDLIST/SHITEMID/STRRET/SHFILEINFO layouts) + careful
  C-linkage checks against the `.cpp` PIDL helpers.

## Process

Mirror gdi32: parallel opus subagents translate one `.c`→`.rs` to `~/vibeos-rust-staging/`
(source-only, no shared-file/CMake/build edits), I integrate centrally (wire `mod.rs` +
CMake flip), build + smoke ×3, commit, flag the SHA to meaniepy for the cold ×3 boot gate +
landing. Pin every constant from the SDK headers (avoid the hallucinated-constant class).
