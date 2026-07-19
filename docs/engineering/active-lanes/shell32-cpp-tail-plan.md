# shell32 C++ tail → zero (lane plan)

**Status:** PLANNED (2026-07-18). Starts after the activatable COM object map reached
33/33 Rust-owned classes. This plan finishes the compiled C++ translation units before any
attempt to make Cargo own the final `shell32.dll` link.

## Outcome and boundary

This lane has two deliberately separate outcomes:

1. **Translation complete:** no `.cpp` is compiled from `dll/win32/shell32/` or its
   `shellmenu`, `shelldesktop`, and `shellrecyclebin` subtargets. CMake/Ninja may still own
   resource generation, typelib/spec processing, and the final PE link.
2. **Cargo-link cutover:** only after outcome 1 is green, move compilation and the PE link
   under Cargo. The ReactOS-wide build graph may retain a thin CMake staging target until a
   broader build-system change; that wrapper is not untranslated shell32 code.

Do not conflate “all object-map classes are Rust” with either outcome. The former is done;
the 70 compiled C++ translation units below are the remaining source frontier.

## Measured baseline

Measured from the four live CMake source lists at `5a03139e92f`:

| Live source kind | Files | Physical lines |
|---|---:|---:|
| `.cpp` | 70 | 60,258 |
| ordinary `.c` (`shellrecyclebin`) | 2 | 377 |
| sanctioned variadic ABI shim | 1 | 60 |
| **total compiled source tail** | **73** | **60,695** |

The `.cpp` count is the landing metric. Files that remain on disk but are absent from every
live source list do not count. Conversely, a hybrid file counts until its final helper,
companion class, and bridge has moved and the whole TU leaves CMake.

The sanctioned C shim is `rust_varargs_shim.c`: stable Rust cannot define the exported
C-variadic `ShellMessageBoxA`. It must ultimately be built from Cargo (or the shared
`varargs-rust` machinery), not used as a reason to retain shell32's CMake compilation.

## Non-negotiable rails

- Preserve paid outputs in `tools/rustconv/.responses/` and durable staging under
  `~/vibeos-rust-staging/shell32-cpp-tail/`. Never regenerate an accepted worker without
  explicit approval.
- A C++ TU leaves CMake only when every live symbol it owned has a Rust owner. No inert
  source hiding, `--allow-partial`, duplicate-definition flags, or lower boot marker.
- `rustconv` owns ABI, object layout, vtables, destruction order, constants, call envelopes,
  and cache replay. The model owns method/function-body judgment only.
- Diff every accepted body against C++ control flow and ownership behavior. Compile-green is
  necessary but never semantic acceptance.
- Bridges are temporary debt. Record a bridge ledger at baseline; after the tooling wave the
  count must decrease monotonically. Do not “finish” a TU by moving its C++ into a new shim.
- i386 is the first integrated link oracle; amd64 and arm64 follow. Each meaningful batch
  ends with candidate-check ×3, builds ×3, smoke ×3, and screenshot inspection. Each cohort
  ends with a clean full `vibeos-cli gate`.
- `shellmenu` and `shelldesktop` are shared by `shell32.dll` and `rshell.dll`; both consumers
  must link and boot before removing shared C++.

## Wave 0 — make whole-TU closure deterministic (no paid calls)

The existing `--cpp-com` path is class-atomic and solved activatable ATL classes. The tail
also contains free functions, private/dynamically-created classes, file statics, overloads,
and several classes per TU. Add a fail-loud **C++ TU mode** before spending on the corpus:

- Extract every defined free function and class method, including header-inline bodies used
  by the TU; recover overload identity without asking the model to invent Rust names.
- Build a source-selected closure over file statics, globals, tables, callbacks, helper
  classes, and same-TU calls. Class layout, ctor/dtor sequencing, RAII members, inheritance,
  and aggregation remain deterministic preflight responsibilities.
- Read `shell32.spec`, CMake consumers, WIDL headers, and existing Rust exports so external
  ownership and calling convention are proven before translation.
- Emit one full-module candidate manifest and compile it on i386/amd64/arm64. Partial body
  extraction is a hard error.
- Add a generated inventory check that compares this plan's ledger with all four live CMake
  lists and fails on an unclassified addition or a claimed-removed TU that is still compiled.
- Add an export/symbol differential oracle: C++ object set versus Rust replacement before
  the source-list flip, with an explicit allowlist only for intentionally private symbols.

Pilot the mode on `CEnumIDListBase.cpp` plus one hybrid wrapper such as
`CDropTargetHelper.cpp`. No bulk paid run is authorized until both go from dry-run preflight
to full-module candidate-check green without guessed substrate.

## Wave 1 — peel every hybrid TU (19 files, 16,378 lines)

These files already contain a Rust-owned headline class. Translate their retained companion
objects/helpers and delete the bridge boundary so each entire TU can leave CMake:

```text
CActiveDesktop.cpp                 CQueryAssociations.cpp
CDropTargetHelper.cpp              CRecycleBinCleaner.cpp
CShellItem.cpp                     CFolderOptions.cpp
dialogs/drvdefext.cpp              dialogs/filedefext.cpp
folders/CDesktopFolder.cpp         folders/CFSFolder.cpp
folders/CDrivesFolder.cpp          folders/CPrinterFolder.cpp
folders/CNetFolder.cpp             folders/CControlPanelFolder.cpp
folders/CRecycleBin.cpp            droptargets/CexeDropHandler.cpp
COpenWithMenu.cpp                  CCopyMoveToMenu.cpp
shellmenu/CMergedFolder.cpp
```

Land in dependency-shaped batches: tiny wrappers first; property-sheet helpers; folder
companions; then OpenWith/copy-move/menu companions. The physical LOC overstates new logic
because many activated class bodies are now dead, but the TU does not count as removed until
the remaining live closure is Rust. `CShellLink.cpp` is intentionally deferred to the
monolith wave: its retained helper surface is still 3.3K lines.

**Exit:** 19 source-list removals, their `*_RustBridge` declarations gone where no longer
needed, shell32 + rshell links green, full gate ×3.

## Wave 2 — private/dynamic object graph (14 files, 6,485 lines)

These are COM-like or C++ object implementations constructed internally rather than through
the activatable object map. Translate them as connected object graphs, not isolated methods:

```text
CIDLDataObj.cpp                    CEnumIDListBase.cpp
CExtractIcon.cpp                  folders/CRecycleBinFolderViewCB.cpp
folders/CRegFolder.cpp            droptargets/CFSDropTarget.cpp
droptargets/CRecyclerDropTarget.cpp
CFileSysBindData.cpp              CDefViewDual.cpp
CDefViewBckgrndMenu.cpp           CDefaultContextMenu.cpp
CFolder.cpp                       CFolderItems.cpp
CFolderItemVerbs.cpp
```

Required deterministic capability is the non-activatable form of the COM scaffold: explicit
constructor/caller ownership, optional aggregation, no assumed `OBJECT_ENTRY`, and vtable
publication only where the C++ actually publishes one.

Suggested batches are data-object/PIDL primitives; drop targets; folder automation; then the
DefView/context-menu cluster.

**Exit:** all 14 TUs absent from CMake, no C++ private object reachable from a Rust vtable,
full gate ×3.

## Wave 3 — shared object libraries (12 files, 7,319 lines)

### `shellmenu` shared by shell32 and rshell

```text
CMenuFocusManager.cpp  CMenuToolbars.cpp  CStartMenu.cpp
```

After Wave 1 removes `CMergedFolder.cpp`, expand `menuband-rust` to own the complete shared
menu surface. Rename it to `shellmenu-rust` once, only after the final C++ source is gone.
Both DLLs retain their own module-lock callbacks.

### `shelldesktop` shared by shell32 and rshell

```text
CChangeNotifyServer.cpp  CDesktopBrowser.cpp  CDirectoryWatcher.cpp
CDirectoryList.cpp       dde.cpp
```

Create a shared `shelldesktop-rust` staticlib linked by both consumers. DDE callback ABI and
window-procedure dispatch require explicit i386 tests in addition to the normal desktop boot.

### `shellrecyclebin`

```text
recyclebin_generic.cpp             recyclebin_generic_enumerator.cpp
recyclebin_v5.cpp                  recyclebin_v5_enumerator.cpp
```

This library is shell32-private today; absorb it into `shell32-rust` unless a second live
consumer is discovered. Translate its two small C files in the later C-tail step.

**Exit:** no C++ `shellmenu`, `shelldesktop`, or `recyclebin` object/static library; shell32
and rshell both gate green.

## Wave 4 — procedural leaf/support TUs (15 files, 6,187 lines)

```text
changenotify.cpp                   debughlp.cpp
dialogs/drive.cpp                 dialogs/folder_options.cpp
dialogs/item_prop.cpp             dialogs/general.cpp
dialogs/recycler_prop.cpp         dialogs/view.cpp
folders.cpp                       propsheet.cpp
shldataobject.cpp                 shlfolder.cpp
stubs.cpp                         systray.cpp
CDefViewUtil.cpp
```

Translate by subsystem, preserving static tables/callback identity and export decoration.
Prefer ordinary Rust modules in `shell32-rust`; create a new crate only for a real shared-link
boundary. Remove each CMake entry in the same commit as its Rust module.

**Exit:** 15 removals and a full gate ×3.

## Wave 5 — monoliths and module capstone (10 files, 23,889 lines)

Order them so shared substrate matures before the hardest control-flow files:

1. `iconcache.cpp`
2. `dialogs/dialogs.cpp`
3. `brfolder.cpp`
4. `dialogs/filetypes.cpp`
5. `utils.cpp`
6. `CShellLink.cpp` (retained helper closure; activated class is already Rust)
7. `shlexec.cpp`
8. `shlfileop.cpp`
9. `CDefView.cpp`
10. `shell32.cpp` (DLL/module/object-map capstone)

Keep translation function-granular and cache-durable. Large Rust outputs may be split into
logical submodules, but C/C++ linkage and file-static semantics must be closed before the
original TU is removed. `shell32.cpp` is last because it owns DLL initialization, module
locks, class-factory routing, and the object map; translating it earlier would churn every
cohab boundary.

**Exit:** the live CMake inventory reports exactly zero `.cpp`; export/symbol differential
is clean; `cpprt`, `atl_classes`, the C++ PCH, and the three obsolete C++ object-library
targets are removable; clean full gate ×3.

## Wave 6 — C tail, then Cargo-owned link

First translate `shellrecyclebin/recyclebin.c` and `shellrecyclebin/guid.c`. Keep the genuine
variadic `rust_varargs_shim.c`, but build it from Cargo/shared `varargs-rust` infrastructure.
At this checkpoint shell32 has no CMake-compiled C or C++.

Only then perform the build-system cutover:

1. Make Cargo produce the final PE DLL for all three targets, including the sanctioned C
   shim, import libraries, module-lock glue, and the exact `shell32.spec` export surface.
2. Feed Cargo the compiled `.rc`/typelib artifacts or move their generation under a
   deterministic build script; compare exported names, ordinals, calling conventions,
   resources, and typelib byte-for-byte where practical.
3. Temporarily keep a thin CMake custom target that asks Cargo for the finished DLL and stages
   it into the ReactOS image. Remove the old `add_library(shell32 MODULE ...)` link path.
4. Remove even that wrapper only as part of a repo-wide build-graph decision; shell32 must not
   invent a private build entry point that the normal image/gate cannot reproduce.

**Final gate:** cold build + full `vibeos-cli gate` on i386/amd64/arm64, export audit, rshell
consumer gate, screenshots, and proof that no `dll/win32/shell32/**/*.{c,cpp}` is compiled by
CMake. Do not push without the resulting clean-tree stamp and explicit user approval.

## Progress ledger

| Cohort | `.cpp` files | Physical lines | Done condition |
|---|---:|---:|---|
| Hybrid peel | 19 | 16,378 | whole TUs removed; bridges retired |
| Private/dynamic objects | 14 | 6,485 | internal construction/vtables Rust-owned |
| Shared libraries | 12 | 7,319 | shell32 + rshell consumers green |
| Procedural leaf/support | 15 | 6,187 | whole TUs removed |
| Monoliths/capstone | 10 | 23,889 | `shell32.cpp` removed last |
| **Total** | **70** | **60,258** | **zero live `.cpp` entries** |

Update this table from the generated live-CMake inventory at every cohort gate. A decreasing
LOC count is informative; the authoritative metric is the live TU count plus executable
three-architecture proof.
