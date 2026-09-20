# shell32: oracle triage wave 2 — the 26 subject-level-only commits, opened

**From** the upstream-sync lane · **written 2026-09-20** · **owner: the shell32 lane**

The wave-1 handoff (`shell32-upstream-oracle-handoff.md`) triaged 48 upstream commits
touching `dll/win32/shell32` since the fork (`5213cf717cff6946945922e6a7e03d87b0a58e4f`),
verified 2 CONFIRMED + 7 INCONCLUSIVE, and left **~26-31 commits triaged at subject level
only** — flagged as unread, not cleared, given a measured 25-40% subject-only error rate in
both directions. This document opens every one of those commits (`git show`, full diff, not
just the subject line) and checks each against `sdk/lib/rust/shell32-rust/` by function name.

**Method note on scope**: the handoff's own count of "~26" undercounted. Subtracting the 11
commits the handoff actually verified (4fecc908e7d, 68a03b2e1ae, a790beb22ba, 6730ec5508f,
fd683dbba45, 02f23141395, 7cba1c80a75, 26f920582be, 9342ed43d9a, 39ad1b6824f, 8d44ccdd5f7) and
6 translation/cosmetic-only commits (see "Skipped" below) from the full 48-commit log leaves
**31 commits**, all opened below.

## Result summary

| verdict | count |
|---|---|
| CONFIRMED (Rust carries the pre-fix bug) | 6 |
| INCONCLUSIVE (real behavior change, scope too large to pin down in this pass) | 1 |
| REFUTED (checked, no behavior difference or Rust already matches) | 6 |
| N/A-FEATURE (pure addition or unimplemented-stub-stays-unimplemented, nothing regressed) | 10 |
| N/A-BUILD/SPEC (linkage, ordinals, forwarding — no shell32-rust logic touched) | 8 |
| Skipped (translation-only / pure cosmetic, per task instructions) | 6 |
| **Total** | **37** (31 opened + 6 skipped) |

(Table corrected 2026-09-20 after a self-check found `07d544ff07e` — `SHSetLocalizedName` —
missing from the first pass; it is now filed under N/A-FEATURE, so N/A-FEATURE reads 10
rather than 9 as originally drafted. Counts above already include the fix.)

Six commits the wave-1 handoff's "genuine feature additions, nothing to regress" framing
would have covered turned out to carry **real, demonstrable bugs in our Rust** — see
"What changed vs the handoff's assumptions" at the bottom.

---

## CONFIRMED — Rust has the pre-fix shape, verified by function name

### 1. `a83411cfefe` — `SHRunControlPanel` / `SHELL32_RunControlPanel`: no NULL check before `wcscat`
- **Upstream**: adds `if (!commandLine) return FALSE;` to `SHELL32_RunControlPanel` before
  `wcscat(parameters, commandLine)` (`dll/win32/shell32/wine/shellord.c`).
- **Ours**: `sdk/lib/rust/shell32-rust/src/wine/shellord.rs:1986-2002`. `SHRunControlPanel`
  builds `parameters` then does `wcscat(parameters.as_mut_ptr(), commandLine)`
  unconditionally — no null check, byte-for-byte the pre-fix shape.
- **Reachable how**: `CShellDispatch_ControlPanelItem_body`
  (`sdk/lib/rust/shell32-rust/src/shell_dispatch.rs:1425-1430`) calls
  `SHRunControlPanel(szDir, core::ptr::null_mut())` directly with the caller-supplied `szDir`
  BSTR — this is `IShellDispatch::ControlPanelItem`, a public COM automation method reachable
  from a WSH/VBScript client. A null/empty BSTR argument (e.g. `ControlPanelItem("")` producing
  a null `szDir`, or a script passing `vbNullString`) drives `wcscat` with a NULL source →
  crash.
- **What to check / correct behaviour**: `SHRunControlPanel` (and its `__REACTOS__` inner
  `SHELL32_RunControlPanel`) should return `FALSE` immediately when `commandLine` is NULL,
  before touching `parameters`.

### 2. `16bb138e325` — `CDefaultContextMenu`: named registry verbs never reach the registry
This is the "high suspicion" commit from the task brief, and it is real. Three separate defects
converge:
- **`DoOpenOrExplore` is an unconditional stub.** Upstream deletes it entirely (folds "open"
  into generic registry-verb dispatch). Ours: `CDefaultContextMenu_DoOpenOrExplore_body`
  (`sdk/lib/rust/shell32-rust/src/shell32/CDefaultContextMenu.rs:1997`) is
  `{ E_FAIL }` — exactly the pre-fix `UNIMPLEMENTED; return E_FAIL;`.
- **`g_StaticInvokeCmdMap` still contains the dead pre-fix entries.**
  `sdk/lib/rust/shell32-rust/src/shell32/CDefaultContextMenu_tu.rs:43-58` has all 14 pre-fix
  rows, including `"runas"→IntVerb 0`, `"print"→IntVerb 0`, `"preview"→IntVerb 0`, and
  `"open"→FCIDM_SHVIEW_OPEN`. Upstream's fix removes all four (comment: *"Verbs from the
  registry should not be listed here"*).
- **`MapVerbToCmdId` never checks the real registry-verb list.** Ours
  (`CDefaultContextMenu.rs:2383-2420`) walks `g_StaticInvokeCmdMap` then only
  `m_DynamicEntries` — it never walks `m_StaticEntries` (the actual per-file-type registry
  verbs), which is exactly the gap upstream's fix closes.
- **Net effect**: `IContextMenu::InvokeCommand(lpVerb="open")` on `CDefaultContextMenu` —
  a very common explicit-verb invocation (double-click default action, `ShellExecute` with an
  explicit verb, shell extensions that re-invoke a named verb) — dispatches to
  `FCIDM_SHVIEW_OPEN` → `CDefaultContextMenu_DoOpenOrExplore_body` → **always `E_FAIL`**,
  instead of resolving through the file type's real registered "open" command. `"runas"` and
  `"print"` similarly collide on `IntVerb == 0` instead of resolving via the registry.
- **What to check / correct behaviour**: `MapVerbToCmdId` should also search
  `m_StaticEntries` (registry verbs) by name and return an id resolving to
  `InvokeRegVerb`; `g_StaticInvokeCmdMap` should drop the `"runas"/"print"/"preview"/"open"`
  rows; `DoOpenOrExplore` should be removed and its case folded into the registry-verb path.

### 3. `763ce84cf55` — `CExeDropHandler`: leaked allocation + missing OOM check
- **Upstream**: `~CExeDropHandler()` gained `SHFree(m_PathTarget);` (was empty — leaked every
  time); `Load()` gained `if (!m_PathTarget) return E_OUTOFMEMORY;` before the `memcpy`.
- **Ours**: `sdk/lib/rust/shell32-rust/src/exe_drop_handler.rs`.
  - `release()` (lines 344-353) frees `this` via `HeapFree` on last-ref but never frees
    `(*this).sPathTarget` — the exact pre-fix leak, once per drop-handler instance (one per
    drag-drop of an exe/bat/cmd file after this port completes CORE-20745's item).
  - `load()` (lines 479-486): `SHAlloc(...)` result is stored into `(*this).sPathTarget` and
    immediately used in `copy_nonoverlapping(file, target, len + 1)` with **no null check** —
    on allocation failure this writes through a null pointer.
- **What to check / correct behaviour**: free `sPathTarget` in `release()`'s last-ref branch
  before/alongside `HeapFree(this)`; check the `SHAlloc` result in `load()` and return
  `E_OUTOFMEMORY` on null before the copy.

### 4. `a5f51269c88` — two existing collection helpers swallow failure as success
- **`CFolderItemVerbs::Item` out-of-range index returns `S_OK` with an unset out-param.**
  Upstream changes `if (index > m_count) return S_OK;` to `return S_FALSE;`. Ours:
  `sdk/lib/rust/shell32-rust/src/shell32/CFolderItemVerbs.rs:586-588` still has
  `if index > (*this).m_count { return S_OK; }` — and `*ppid` is only ever written at the very
  end of the function (line 653, `*ppid = pVerb;`), so this early return leaves the caller's
  `*ppid` **uninitialized/garbage** while reporting success. Reachable from script automation
  (`FolderItemVerbs.Item(i)` via `IDispatch`/`IEnumVARIANT`-style iteration past the true
  count) — an uninitialized-pointer dereference in the caller, not just a logic wart.
- **`CEnumIDListBase::AppendItemsFromEnumerator` ignores `AddToEnumList`'s return.** Upstream
  wraps the call: on `AddToEnumList` failure, `ILFree(pidl)` + return `E_OUTOFMEMORY`. Ours:
  `sdk/lib/rust/shell32-rust/src/shell32/CEnumIDListBase.rs:215-228` calls
  `CEnumIDListBase_AddToEnumList_body(this, pidl);` and discards the `BOOL` result, always
  returning `S_OK` — leaks `pidl` on OOM and silently drops the enumerated item instead of
  reporting the failure.
- **What to check / correct behaviour**: `Item` should return `S_FALSE` (and leave `*ppid`
  NULL) past the last index; `AppendItemsFromEnumerator` should check `AddToEnumList`'s return,
  free `pidl` and return `E_OUTOFMEMORY` on failure.

### 5. `b60f66cbbfa` — `CPrinterFolder::GetUIObjectOf` missing `IQueryAssociations`
- **Upstream**: adds an `else if (IsEqualIID(riid, IID_IQueryAssociations)) hr = _AssocCreate(...)`
  branch, needed because "ShellExecute implementation requires a valid IQueryAssociations
  interface" (commit message, citing the NSE implementation guide).
- **Ours**: `CPrinterFolder_GetUIObjectOf_body`
  (`sdk/lib/rust/shell32-rust/src/printer_folder.rs:873-899`) only handles
  `IID_IExtractIconA`/`IID_IExtractIconW`, `else { hr = E_NOINTERFACE; }` — exactly pre-fix.
  `ShellExecute` on a printer-folder item that needs to resolve verbs through
  `IQueryAssociations` gets `E_NOINTERFACE` instead of a working association object.
- **What to check / correct behaviour**: add the `IID_IQueryAssociations` branch, creating a
  `QueryAssociations` object initialized against the `"Printers"` progid (mirrors upstream's
  `_AssocCreate`).

### 6. `d6cc284435e` — `LinkWindow_RegisterClass` lies about success
- **Upstream**: implements real window-class superclassing; the pre-fix body was
  `FIXME("()\n"); return TRUE;` — claiming success while registering nothing.
- **Ours**: `sdk/lib/rust/shell32-rust/src/wine/shellord.rs:2846-2849` —
  `pub unsafe extern "system" fn LinkWindow_RegisterClass() -> BOOL { TRUE }` — same lie.
  Lower severity than the others: `LinkWindow_UnregisterClass` is presumably a matching no-op
  stub, so a caller that only round-trips register/unregister sees no visible symptom; a
  caller that then tries `CreateWindow` against the (never-registered) class would fail with
  `ERROR_CANNOT_FIND_WND_CLASS` despite the earlier "success". Not traced to a concrete caller
  in this pass — flagging as CONFIRMED-but-low-severity rather than escalating further.
- **What to check**: whether anything in the current tree actually calls
  `LinkWindow_RegisterClass` and depends on the return value; if so, either implement real
  registration or return `FALSE` honestly.

---

## INCONCLUSIVE

### 7. `93f96e592b2` — `CQueryAssociations` near-total rewrite (777+/313- on one file)
This is not an incremental patch to existing functions — `Init`, `GetString`, and the internal
`hkeySource`/`hkeyProgID` storage model are replaced wholesale with a new
`AssocCreateElement`/evalcmd-backed design, alongside the new `GetKey` method and more
`ASSOCSTR_*` support. Comparing "does our Rust have the pre-fix shape" is not well-posed at
that granularity: our Rust translated the *old* architecture completely, and upstream replaced
the architecture, not a bug within it.
- **What's real and worth a dedicated pass**: the already-CONFIRMED handoff finding #1
  (`7cba1c80a75`, `ASSOCSTR_DEFAULTICON` not returning `E_OUTOFMEMORY` on allocation failure)
  is exactly the class of defect this rewrite's `GetString` continues to touch — the new
  `GetString` implementation should be read function-by-function against our
  `sdk/lib/rust/shell32-rust/` `CQueryAssociations` (function names: `Init`, `GetString`,
  `GetKey`) rather than assumed clean because it's "new architecture." Not done here due to
  size; flagging rather than guessing.

---

## Skipped (per task instructions)

- **Translation-only**: `d8b87733dc3` (it-IT), `d4e4a4a7f29` (pt-PT), `7d08042343a` (tr-TR) —
  `.rc` string-table translations only, no code.
- **Pure cosmetic**: `dbaf14d4a1c` (indentation reduce-diff-to-wine), `46121f46437` (disable a
  GCC 13 warning), `1aab8a87c4e` ("accessability" typo, touches `NTUSER` + one `.h` comment).

---

## REFUTED — opened, no behavior difference or Rust already correct

- **`1cf31524d50`** ("Stop using non-conforming swprintf/vswprintf") — renames call sites from
  `swprintf`/`vswprintf` to `_swprintf`/`_vswprintf` across 11 shell32 files (COpenWithMenu.cpp,
  dialogs/filedefext.cpp, shlexec.cpp, etc.). Checked directly: under
  `_CRT_NON_CONFORMING_SWPRINTFS` (globally defined at the fork point — see `b13e568d991`
  below), `swprintf` **is** `_swprintf` (no length argument, non-ISO-conforming). The rename is
  purely decoupling call sites from that global macro ahead of removing it project-wide; it is
  the *same function* before and after. No truncation/argument-count/return-value difference.
  Not a signature change at any of the sampled call sites.
- **`b13e568d991`** ("Remove global `_CRT_NON_CONFORMING_SWPRINTFS`") — the CMake half of the
  above; shell32's own `CMakeLists.txt` gets a comment-only-equivalent change (the global
  define moves to per-target opt-in elsewhere in the tree; shell32 already uses the
  underscored calls from `1cf31524d50` so it needs no opt-in). Build-only, no runtime effect.
- **`ba7a6fc7d9a`** (`SHELL_LINK_HEADER::nIconIndex` `DWORD`→`INT`) — checked every consumer in
  `CShellLink.cpp`: line 1755 assigns through an `INT*` out-param, line 2960 immediately casts
  to a local `INT`; there is no comparison anywhere that depends on the field's declared
  signedness. The struct is also unchanged in size (both are 32-bit). Our Rust
  (`sdk/lib/rust/shell32-rust/src/shell_link.rs:518`, still `DWORD`) matches this: every read
  site already casts to `INT`/`i32` (lines 3468, 4496), so the field's declared type is
  cosmetic/documentation-only in both C and Rust. No behavior difference either side.
- **`2cbce1c043e`** (`SHUpdateImageW`) / **`98bebe868b0`** (`SHUpdateImageA`) — both replace a
  `FIXME(...); ` no-op stub with a real icon-cache-invalidation broadcast. Ours
  (`wine/shellord.rs:2668-2690`) is still the stub, matching the **pre-fix** behavior exactly —
  since the pre-fix behavior was itself a documented no-op, there is nothing to regress; this
  is an unported feature, not an inherited bug.
- **`9e5774e75cd`** (`CDefFolderMenu_MergeMenu`) — same pattern: pre-fix was a documented
  `FIXME`/no-op (`stubs.cpp`); ours (`sdk/lib/rust/shell32-rust/src/shell32/stubs.rs:91-97`)
  is the same no-op. Unported feature, not a regression.

---

## N/A-FEATURE — pure additions, nothing pre-existing touched (checked, not assumed)

- **`546297fe862`** (`SHEvaluateSystemCommandTemplate`) — entirely new `evalcmd/` static lib;
  the one pre-existing-file touch (`wine/shpolicy.c`) is a two-line `#include` reorder, no
  behavior change.
- **`79761d88fdf`** (`AssocCreateElement`) — entirely new `evalcmd/elements.cpp`; the touch to
  the (week-old, itself-post-fork) `evalcmd.cpp` only changes `static` → `EXTERN_C` linkage on
  three helpers, no logic change.
- **`095860df576`** (NT6+ bundle: `CShellItemArray` split-off, `CShellLibrary`/
  `CUserEventTimer` stubs, `SHCreateItemFromIDList`/`SHCreateItemFromParsingName`/
  `SHGetItemFromDataObject`) — verified against the fork-point tree: none of these three
  `SHCreateItemFrom*`/`SHGetItemFromDataObject` functions existed anywhere in shell32 before
  this commit (checked `CShellItem.cpp` and `stubs.cpp` at the fork SHA). The only removed
  code is the `CShellItemArray` class body, relocated verbatim to its own new file. No
  pre-existing `CShellItem` method changed.
- **`c891aefdcff`** (`CFolder[Item]` getters/setters) — the one non-additive-looking hunk
  (`CFSDropTarget::_CopyItems`/`_DoDrop` reading a new `"FileOpFlags"` private clipboard
  format via `GetDefaultFileOpFlags(pDO, fDefault=FOF_ALLOWUNDO|FOF_NOCONFIRMMKDIR)`) keeps the
  exact old hardcoded flags as its default, so callers that don't set the new format see
  identical behavior. `CFSFolder::GetDetailsEx`'s `static_cast<IShellFolder2*>(this)` is a
  C++ overload-disambiguation cast only.
- **`07d544ff07e`** (`SHSetLocalizedName`) — upstream implements the function for real in
  `utils.cpp` (resolves the path, builds an indirect `@module,-id` name string, calls
  `IShellFolder::SetNameOf`) and gates the old `wine/shellord.c` stub out with
  `#ifndef __REACTOS__`. Before this commit the function was a pure `return S_OK;` no-op stub
  in `wine/shellord.c` — it never had a real ReactOS implementation, in `utils.cpp` or
  anywhere else (`grep` at the fork SHA confirms no prior `utils.cpp` definition). Ours
  (`sdk/lib/rust/shell32-rust/src/wine/shellord.rs:2836-2844`) is still the `S_OK`-only stub —
  matches pre-fix behavior exactly, and pre-fix behavior was itself a documented no-op. Unported
  feature (renaming "My Computer" etc. silently no-ops today), not an inherited bug.
- **`0533a67b184`** (`SHRestrictedMessageBox`) — the function itself lives entirely in
  `dll/win32/shlwapi/`, out of `shell32-rust` scope. The shell32 touch
  (`dialogs/folder_options.cpp`) adds a brand-new `REST_NOFOLDEROPTIONS` policy check that
  didn't exist pre-fix on either side — a genuinely new Group-Policy enforcement point, not a
  fix to previously-broken enforcement (there was no enforcement before). Our Rust
  (`sdk/lib/rust/shell32-rust/src/dialogs/folder_options.rs`) lacks it — a real feature gap
  (a restricted user can currently open Folder Options where real Windows/upstream would
  block them), but not an inherited *bug* in the oracle sense; flagged here for the shell32
  lane's feature backlog rather than as a CONFIRMED regression.
- **`a83411cfefe`** — the CONFIRMED null-check bug is listed above; the surrounding
  Vista-compat rewrite of `CShellDispatch::ControlPanelItem` itself (branching on
  `GetVersion()`) is additive/feature, not itself a second bug.
- **`3b58b347bc1`** (Explorer refresh forwarding) — touches `CMenuBand`, `CMenuDeskBar`,
  `CStartMenu` (none ported to `shell32-rust`; confirmed via
  `grep -rln "CMenuDeskBar\|InvalidateItem" sdk/lib/rust/shell32-rust/` → only bindgen
  `island_*.rs` islands, no translation) and `base/shell/explorer/*` (out of shell32 scope
  entirely). `CShellDispatch::RefreshMenu` itself is untouched behaviorally (gains a comment
  and a `C_ASSERT`, still `PostTrayCommand(TRAYCMD_REFRESH_MENU)`).
- **`fe2b8ba41b1`** (`CShellDispatch::EjectPC`) — `Suspend`/`EjectPC`/`RefreshMenu` go from
  `E_NOTIMPL` to real `PostTrayCommand(...)` calls. Ours
  (`sdk/lib/rust/shell32-rust/src/shell_dispatch.rs:1401-1423`) still has all three as
  `E_NOTIMPL` — an unported feature (E_NOTIMPL is a valid, non-buggy "not implemented" answer),
  not a regression.
- **`afd1ef71031`** (branding library) — relocates the About-dialog logo bitmaps from
  shell32's own resources to an external `rosbrand.dll`; our Rust
  (`sdk/lib/rust/shell32-rust/src/wine/shell32_main.rs:1984-2028`) still loads
  `IDB_REACTOS`/`IDB_LINEBAR` from shell32's own resources — correct as long as shell32 still
  ships those resources (it does; this repo never took the branding-library split). Asset
  sourcing only, zero logic difference.
- **`6af07a31b00`** (FONTEXT delete support) — the shell32-side touch adds two new
  early-exit-if-handled callback hooks (`_DoInvokeCommandCallback` in `DoCopyOrCut`,
  `_DoCallback(DFM_GETVERBA/W)` in `GetCommandString`) that only change behavior for a folder
  whose callback actually intercepts `DFM_CMD_COPY`/`DFM_CMD_MOVE`/`DFM_GETVERBA`/`DFM_GETVERBW`
  (i.e. the new `CFontExt`) — for every existing folder that doesn't answer those messages the
  callback returns non-`S_OK` and control falls through to the unchanged original logic.
  Extension-point addition, not a fix.

## N/A-BUILD/SPEC — linkage, ordinals, forwarding; no shell32-rust logic involved

Checked full diffs on all eight; none touch a function body reachable from `shell32-rust`:
- `f9e3448f168` — `"StartUp"` → `"Startup"` typo fix, `.rc` string tables + `boot_images.cmake`
  path text only.
- `da8f604da06`, `aaf231f00f0` — forward `SHSetInstanceExplorer`/`SHGetInstanceExplorer` to
  `shcore.SetProcessReference`/`GetProcessReference` in `shell32.spec`. **Not actionable as-is**:
  this tree has no `shcore` module at all (`find . -iname "*shcore*"` — nothing), so our
  `shell32.spec` still points at shell32's own implementations
  (`sdk/lib/rust/shell32-rust/src/wine/shellord.rs:2004-2021`, `SHSetInstanceExplorer`/
  `SHGetInstanceExplorer`), which is the only option available and functionally equivalent for
  a single-process-explorer model. Revisit only once/if a `shcore` module exists.
- `0e245f97b5c`, `4f329a03b71`, `bd6d9b73ee3` — `shell32.spec` ordinal/export-table
  bookkeeping (Vista stub exports, moving auto-ordinals to the bottom, `@`-marking auto
  ordinals with 2k3/Vista ordinal comments). Export table metadata only.
- `c3d6516f20c` — `Shell_GetCachedImageIndexW` forward-declared/defined `EXTERN_C` (linkage
  attribute for C++ name mangling only).
- `c61d98f9392` — deletes a duplicate `SHMultiFileProperties` declaration from `precomp.h`
  (the real one already lives in `<shlobj.h>`). Declaration-only.

---

## What changed vs the handoff's assumptions

The handoff characterized most of this 26-31-commit block as "genuine feature additions with
no prior C behavior to regress against," while explicitly warning that this exact reasoning
had once cleared a real bug (`addrconv.c`'s stack overflow) elsewhere. That warning was
justified: **six of these "feature" commits carry real, verifiable bugs in `shell32-rust`**,
found by reading the diff instead of the subject:

- `a83411cfefe` ("Avoid SHRunControlPanel to be Vista compatible") reads as a compat shim; it
  also silently added a NULL-pointer guard to a function our Rust still lacks.
- `16bb138e325` ("Handle named registry verbs...") reads as new-capability plumbing; it is
  actually the fix for a standing bug where invoking the literal `"open"` verb on
  `CDefaultContextMenu` **always fails** in our Rust.
- `763ce84cf55` ("Implement bat/cmd drop target menu") reads as pure feature; it also fixed a
  leak and a missing OOM check in an existing, already-shipping class (`CExeDropHandler`) that
  our Rust still carries.
- `a5f51269c88` ("Add FolderItems3 support") reads as new-collection-methods; two of its
  changed hunks are silent-failure-as-success fixes in the **existing**
  `CFolderItemVerbs::Item` and `CEnumIDListBase::AppendItemsFromEnumerator`, both reachable
  independently of FolderItems3.
- `b60f66cbbfa` ("CPrinterFolder: Support IQueryAssociations") is a real functional gap in an
  existing dispatch function (`GetUIObjectOf`), not just a new capability.
- `d6cc284435e` ("Implement LinkWindow_RegisterClass") is filed as "implement" but the
  pre-fix/current-Rust stub actively **lies** (`return TRUE` while registering nothing) rather
  than honestly reporting unimplemented.

Net: the "feature bundle, nothing to regress" heuristic held for roughly 25 of the 31 opened
commits, and failed on the other 6 — consistent with the handoff's own measured 25-40% error
rate. The common shape in every miss: a feature commit that, in the course of wiring the new
capability in, also touched a helper or dispatch branch used unconditionally by existing
callers.
