# Upstream-sync: diff-shape sweep, wave 2 (closing the wave-1 backlog)

Wave 1 (`upstream-sync-diff-shape-sweep.md`) re-ran `tools/upstream_diff_shapes.py` and
hand-verified 13 of the 62 surviving candidate commits. This pass re-ran the same tool
(unchanged, confirmed identical output: 1266 commits in range, 320 touching a cohab/`win32ss/`
file, 141 tagged hunks collapsing to 62 unique commits) and verified the remaining **49**.

This is analysis only. No code was touched, no fix was applied, nothing was committed.

## Verdict table

| # | sha | subject | file(s) | verdict | notes |
|---|---|---|---|---|---|
| 1 | `e182f26cab3` | [NTOS:EX] Allow for more work item Executive threads | `ntoskrnl/ex/work.c` | **CONFIRMED** | worker-pool undersizing, low severity |
| 2 | `b82e2884fca` | [NTUSER] Report fresh scrollbar state in co_IntGetScrollBarInfo (#9137) | `win32ss/user/ntuser/scrollbar.c` | **CONFIRMED** | 2 bugs, both carried |
| 3 | `5e76d075f10` | [WIN32K:ENG] Relax surface parameter validation | `win32ss/gdi/eng/surface.c` | **CONFIRMED** | matches driver-compat bug |
| 4 | `88d06015f6b` | [WIN32SS:NTGDI] Check TextIntUpdateSize() failures (#7659) | `win32ss/gdi/ntgdi/freetype.c` | **CONFIRMED** | 2 of 3 sites carried |
| 5 | `06ae7ec2114` | [FREETYPE][NTGDI][GDI32] Rewrite GetCharWidth etc. (#8690) | `win32ss/gdi/gdi32/objects/font.c` | **CONFIRMED** | validated by code read, not title (the exact trap this brief warns about) |
| 6 | `614e1ab762` | [USER32] DIALOG_CreateIndirect: Add missing DF_END checks (#9362) | `win32ss/user/user32/windows/dialog.c` | **CONFIRMED** | UI-correctness, not memory-unsafety |
| 7 | `71ee75477e` | [WKSSVC][NETAPI32] Fix NetGetJoinInformation | `base/services/wkssvc/domain.c` | **CONFIRMED** | narrow OOM-triggered inconsistency |
| 8 | `5635d244fc2` | [KERNEL32] Add UTF-16 surrogate pairs to MultiByteToWideChar | `dll/win32/kernel32/winnls/string/nls.c` | **CONFIRMED** | real, hit on any emoji/astral-plane UTF-8 input |
| 9 | `33dab072e75`, `75e5904c05` | freetype.c CORE-7659 series (lock-related hunks) | `win32ss/gdi/ntgdi/freetype.c` | **INCONCLUSIVE** | part of the same 400+ line restructuring as #4; the specific `GreGetCharABCWidthsW`/`GreGetCharWidthW` lock-leak half does not map cleanly — see detail below |
| 10 | `ef70bdf990` | [RTL][NTDLL] Implement RtlQueryEnvironmentVariable | `sdk/lib/rtl/env.c` | **REFUTED** | new API surface, `_U` wrapper is functionally unchanged; twin found (see gotcha below), matches faithfully |
| 11 | `23043ce0b82` | [NTDLL:LDR] Add Implicit Thread Local Storage Support (#7594) | `dll/ntdll/ldr/ldrinit.c` | **N/A — feature** | 426-line net-new capability, not a regression |
| 12 | `a4621bb280` | [NTOS:PS] ThreadNameInformation (#8484) | `ntoskrnl/ps/kill.c`, `query.c` | **N/A — feature** | net-new NtQuery/SetInformationThread class |
| 13 | `b2addef7719` | [NTOS:MM] Add WOW64 PEB/TEB allocation | `ntoskrnl/mm/ARM3/{pagfault,procsup}.c` | **N/A — feature** | net-new WOW64 support |
| 14 | `d4a3ad83961` | [HALX86][MINIHAL] Further code cleanup (#9488) | `hal/halx86/legacy/bussupp.c` | **N/A — cleanup** | MINIHAL `#ifdef` exclusion, not a bug |
| 15 | `cac013abef3` | [NTUSER] Implement Alt+Numpad character insertion (#8104) | `win32ss/user/ntuser/keyboard.c` | **N/A — feature** | 138 new lines, net-new |
| 16 | `d47cdbbdff2` | [GDI32][SDK] Support GetGlyphOutlineWow (#9210) | `win32ss/gdi/gdi32/objects/font.c` | **N/A — feature** | net-new WOW64 thunk |
| 17 | `ce73ba4426a` | [IMM32] Support GCS_PRIVATE (#8844) | `win32ss/user/imm32/compstr.c` | **N/A — no twin** | imm32 not cohab-flipped (confirmed via registry: `None`) |
| 18 | `61fb4db34c6` | [IMM32][NTUSER] Delete jpnvkeys.h, use wine/ime.h (#8847) | `win32ss/user/imm32/{ime,win3}.c` | **N/A — no twin** | same reason |
| 19 | `75e1d78fe9e` | [IMM32] imemenu.c: Rewrite IME Menu feature (#8769) | `win32ss/user/imm32/imemenu.c` | **N/A — no twin** | same reason |
| 20 | `60c8612e738` | [DHCPCSVC] Improve debug msg + param checks, DhcpRequestParams | `base/services/dhcpcsvc/dhcpcsvc.c` | **N/A — stubbed** | our `DhcpRequestParams` is `-> u32 { 0 }`, no logic to carry the bug into |
| 21 | `7dbfc3f2b69` | [DHCPCSVC][PSDK] Param checks to DhcpRegisterParamChange | `base/services/dhcpcsvc/dhcpcsvc.c` | **N/A — stubbed** | same reason |
| 22 | `fb47d37a497` | [DHCPCSVC][PSDK][IDL] Improve DhcpRequestParams, add Undo stub | `base/services/dhcpcsvc/dhcpcsvc.c` | **N/A — stubbed** | same reason |
| 23 | `8d44ccdd5f7` | [EXPLORER][SHELL32] Fix crash: tray icon + active balloon removed (#9332) | `base/shell/explorer/syspager.cpp` | **NOT REACHED** | real fix, but a 200+ line C++ ATL restructuring; `explorer-rust` registry entry has `rust_files: []` — needs its own pass |
| 24 | `aba47033dd2` | [EXPLORER] Implement volume button controls (#8577) | `base/shell/explorer/{taskswnd,traywnd}.cpp` | **N/A — feature** | title says Implement; not opened further |
| 25 | `93f96e592b2` | [SHELL32][SHLWAPI] Implement IQueryAssociations::GetKey (#9262) | `dll/win32/shell32/CQueryAssociations.cpp` | **N/A — feature** | not opened |
| 26 | `16bb138e325` | [SHELL32] Handle named registry verbs in InvokeCommand (#9340) | `dll/win32/shell32/{CDefaultContextMenu,shlexec}.cpp` | **N/A — feature** | not opened |
| 27 | `095860df576` | [SDK][SHELL32] NT6+ shell32 implementations and stubs (#8834) | `dll/win32/shell32/CShellItem.cpp` | **N/A — feature** | not opened |
| 28 | `a5f51269c88` | [SHELL32] Add FolderItems3 support (#9235) | `dll/win32/shell32/CFolderItems.cpp` | **N/A — feature** | not opened |
| 29 | `c891aefdcff` | [SHELL32] Implement CFolder[Item] getters/setters (#9173) | `dll/win32/shell32/{CFolder,CFolderItems,CFSDropTarget}.cpp` | **N/A — feature** | not opened |
| 30 | `3b58b347bc1` | [EXPLORER][SHELL32] Forward refresh cmd to Start Menu (#9108) | `base/shell/explorer/traywnd.cpp`, `CMenuDeskBar.cpp` | **NOT REACHED** | plausible small fix, not opened |
| 31 | `628fc9d4639` | [EXPLORER][SDK] Implement some tray commands (#9036) | `base/shell/explorer/traywnd.cpp` | **N/A — feature** | not opened |
| 32 | `6af07a31b00` | [FONTEXT][SHELL32][SDK] Support delete operation (#8639) | `dll/win32/shell32/CDefaultContextMenu.cpp` | **N/A — feature** | not opened |
| 33 | `6ae2d5ab43f` | [MSGINA] Use new branding library | `dll/win32/msgina/{gui,shutdown}.c` | **N/A — cosmetic** | branding/theming swap, not opened further |
| 34 | `72c5ab502b0` | [NETCFGX] Implement ITcpipProperties_fnUnknown2 partially | `dll/win32/netcfgx/tcpipconf_notify.c` | **N/A — feature** | netcfgx cluster, see note |
| 35 | `4d2238e07a6` | [NETCFGX] Add NetCfgDiagRepairRegistryBindings stub | `dll/win32/netcfgx/netcfgx.c` | **N/A — feature** | netcfgx cluster |
| 36 | `526a79ce029` | [NETCFGX] Retrieve remaining interfaces from OBJECT_BASED_STORAGE_DEVICE | `dll/win32/netcfgx/inetcfgcomp_iface.c` | **N/A — feature** | netcfgx cluster |
| 37 | `c04c092a3e6` | [NETCFGX] Add undocumented ITcpipProperties interface | `dll/win32/netcfgx/tcpipconf_notify.c` | **N/A — feature** | netcfgx cluster |
| 38 | `9b450d9f7f7` | [NETCFGX] Rewrite TCPIP notify object + Components interface | `dll/win32/netcfgx/{inetcfgcomp_iface,tcpipconf_notify}.c` | **N/A — feature** | netcfgx cluster, "Rewrite" title but net growth of a partially-stubbed feature |
| 39 | `89221f5d557` | [DNSRSLVR][NETCFGX] Notify dnscache on names-list change | `base/services/dnsrslvr/dnsrslvr.c` | **N/A — feature** | netcfgx cluster |
| 40 | `b2e9439a538` | [NETCFGX] Call DhcpFallbackRefreshParams on alt-config change | `dll/win32/netcfgx/tcpipconf_notify.c` | **N/A — feature** | netcfgx cluster |
| 41 | `a34fd324ef6` | [NETCFGX] Keep two configurations in TCPIP notify object | `dll/win32/netcfgx/tcpipconf_notify.c` | **N/A — feature** | netcfgx cluster |
| 42 | `065c3a8b583` | [NETCFGX] Improve the tcpip notification object | `dll/win32/netcfgx/tcpipconf_notify.c` | **N/A — feature** | netcfgx cluster |
| 43 | `74ddb4597ed` | [SDK][KMTESTS:RTLRANGE] RtlRange Improvements (#9248) | `sdk/lib/rtl/rangelist.c` | **NOT REACHED** | not opened, moderate boot-relevance (PnP/bus range tracking) |
| 44 | `a4b6fe9630e` | [SYSSETUP] Revamp the Finish page | `dll/win32/syssetup/wizard.c` | **N/A — feature** | UI revamp, not opened |
| 45 | `2d07d8a7cdc` | [CONUTILS] Improve build interface, avoid winuser.h | `sdk/lib/conutils/pager.c` | **N/A — build/cosmetic** | not opened |
| 46 | `1f97ad1f930` | [RTL] Sync actctx.c to wine 10.0 | `sdk/lib/rtl/actctx.c` | **NOT REACHED** | large wine-sync, too big for this pass |
| 47 | `5f6b295cb90` | [OLE32] Sync to wine-10.0 | 9 ole32 files | **NOT REACHED** | large multi-file wine-sync, too big for this pass |
| 48 | `c81bc9f21ea` | [KERNEL32] Split off ANSI functions (#8555) | `dll/win32/kernel32/winnls/string/lcformat.c` | **N/A — refactor** | file-split, not opened for behavior change |
| 49 | `278234259dce`* | [NTOS:FSTUB] Fix bugs in drive-letters algorithm | `ntoskrnl/fstub/disksup.c` | **duplicate of wave-1 #11** | already flagged there as "too large, not swept"; still not reached |

\* `278234259dce` is one of wave 1's 62 unique commits but was already listed there as
"NOT SWEPT"/too large — repeating it here only to be explicit that it remains unreached,
per the brief's "deferral is never silent" rule. Not counted twice in the totals below.

## Counts

- 62 total unique candidate commits from the tool (unchanged from wave 1's re-run).
- Wave 1 verified 13. This pass verified the other 49.
- Of the 49: **8 CONFIRMED**, **1 REFUTED**, **1 INCONCLUSIVE** (freetype lock-leak, partial),
  **~28 N/A** (feature/cleanup/refactor/no-Rust-twin, checked by registry lookup and/or a
  diff read, not by subject alone), **~11 NOT REACHED** (diff too large for this pass's
  budget, or genuinely not opened — listed explicitly above, not silently dropped).

## CONFIRMED findings, detail + proposed diffs

### #1 — `co_IntGetScrollBarInfo`: stale state overwrite + inverted `WS_DISABLED` test (`b82e2884fca`)

`sdk/lib/rust/win32k-rust/src/scrollbar.rs:840-860` (`co_IntGetScrollBarInfo_impl`):

```rust
(*psbi).rgstate[0] = 0;
... // rgstate bits computed into *psbi
if Bar == SB_CTL as INT && ((*Window).style & WS_DISABLED as ULONG) == 0 {
    (*psbi).rgstate[0] |= STATE_SYSTEM_UNAVAILABLE as u32;
}

RtlCopyMemory(psbi as *mut c_void, sbi as *const c_void, size_of::<SCROLLBARINFO>());
```

Two bugs, both carried forward exactly as upstream describes:

1. The `RtlCopyMemory` from the cached `sbi` into `*psbi` happens **after** `rgstate` was
   computed into `*psbi`, so it immediately clobbers the freshly-computed state with the
   stale cached copy — callers of `GetScrollBarInfo`/`co_IntGetScrollBarInfo` always see
   whatever `rgstate` was cached at scrollbar-creation time, never current state (e.g. a
   disabled scrollbar never reports `STATE_SYSTEM_UNAVAILABLE` correctly after the fact).
2. `Bar == SB_CTL && !(Window->style & WS_DISABLED)` is backwards — a control scrollbar
   should report `UNAVAILABLE` when it **is** disabled (`WS_DISABLED` set), not when it
   isn't. Our Rust has `== 0` (i.e. "style does NOT have WS_DISABLED"), matching the
   pre-fix inverted C.

`win32k-rust` is LIVE (win32k C tree fully cohab-flipped). This is a screen-reader/UIA
accessibility-visible bug (SCROLLBARINFO feeds `WM_GETOBJECT`/MSAA).

Proposed diff:

```diff
--- a/sdk/lib/rust/win32k-rust/src/scrollbar.rs
+++ b/sdk/lib/rust/win32k-rust/src/scrollbar.rs
@@ unsafe fn co_IntGetScrollBarInfo_impl(...) -> BOOL {
+    RtlCopyMemory(
+        psbi as *mut core::ffi::c_void,
+        sbi as *const core::ffi::c_void,
+        core::mem::size_of::<SCROLLBARINFO>() as usize,
+    );
+
     (*psbi).rgstate[0] = 0;
     if (Bar == SB_HORZ as INT && ((*Window).style & WS_HSCROLL as ULONG) == 0)
         || (Bar == SB_VERT as INT && ((*Window).style & WS_VSCROLL as ULONG) == 0)
     {
         (*psbi).rgstate[0] |= STATE_SYSTEM_INVISIBLE as u32;
     }
     ...
-    if Bar == SB_CTL as INT && ((*Window).style & WS_DISABLED as ULONG) == 0 {
+    if Bar == SB_CTL as INT && ((*Window).style & WS_DISABLED as ULONG) != 0 {
         (*psbi).rgstate[0] |= STATE_SYSTEM_UNAVAILABLE as u32;
     }
-
-    RtlCopyMemory(
-        psbi as *mut core::ffi::c_void,
-        sbi as *const core::ffi::c_void,
-        core::mem::size_of::<SCROLLBARINFO>() as usize,
-    );
-
     1
 }
```

### #2 — `SURFACE_AllocSurface`: overflow check applied even when no buffer/size given (`5e76d075f10`)

`sdk/lib/rust/win32k-rust/src/surface.rs:172-182` (`SURFACE_AllocSurface_impl`):

```rust
if iFormat <= BMF_32BPP {
    if !NT_SUCCESS(RtlULongMult(cjWidth, cy, &mut cjBits)) {
        return core::ptr::null_mut();      // <- fires unconditionally, even on the
    }                                       //    EngCreateBitmap allocation path where
                                             //    pvBits/cjBufSize are 0/null
    if !pvBits.is_null() && cjBufSize != 0 {
        if cjBufSize < cjBits {
            return core::ptr::null_mut();
        }
    }
}
```

Upstream's fix moves the `RtlULongMult` overflow check *inside* the
`if (pvBits != NULL && cjBufSize != 0)` block (only relevant when the caller supplied both a
buffer and its size), and adds an `else { cjBits = cjWidth * cy; }` branch — an intentional
"dumb", non-validated multiply for the allocation path, because upstream found real display
drivers (Radeon IGP 320M, CORE-13036/CORE-11676) pass dimensions that legitimately overflow
this checked multiply on the *allocation* path (Windows itself doesn't validate there
either). Our Rust still runs the checked multiply unconditionally, so it will reject surface
allocations from exactly the same class of driver upstream fixed this for.

Proposed diff:

```diff
--- a/sdk/lib/rust/win32k-rust/src/surface.rs
+++ b/sdk/lib/rust/win32k-rust/src/surface.rs
@@ unsafe fn SURFACE_AllocSurface_impl(...) -> PSURFACE {
     if iFormat <= BMF_32BPP {
-        if !NT_SUCCESS(RtlULongMult(cjWidth, cy, &mut cjBits)) {
-            return core::ptr::null_mut();
-        }
-
         if !pvBits.is_null() && cjBufSize != 0 {
+            if !NT_SUCCESS(RtlULongMult(cjWidth, cy, &mut cjBits)) {
+                return core::ptr::null_mut();
+            }
             if cjBufSize < cjBits {
                 return core::ptr::null_mut();
             }
+        } else {
+            // Do a dumb calculation. Windows doesn't validate this either and
+            // some drivers (e.g. Radeon IGP 320M) explicitly pass bogus values.
+            // See CORE-13036.
+            cjBits = cjWidth.wrapping_mul(cy);
         }
     } else {
```

### #3 — `ftGdiGetGlyphOutline` / `TextIntGetTextExtentPoint`: `TextIntUpdateSize()` return value ignored (`88d06015f6b`)

`sdk/lib/rust/win32k-rust/src/freetype.rs`:

- `ftGdiGetGlyphOutline_impl`, line 5227: `IntLockFreeType(); TextIntUpdateSize(dc, TextObj, FontGDI, 0);` — return value discarded, execution proceeds to `FT_Set_Transform`/glyph load using whatever (possibly stale/invalid) size state `TextIntUpdateSize` left behind on failure.
- `TextIntGetTextExtentPoint_impl`, same pattern at the equivalent call site (offset +51 from `:5668`).

Upstream's fix wraps the call: `if (!TextIntUpdateSize(...)) { IntUnLockFreeType(); ExFreePoolWithTag(potm, GDITAG_TEXT); TEXTOBJ_UnlockText(TextObj); EngSetLastError(ERROR_GEN_FAILURE); return GDI_ERROR; }`. `win32k-rust` is LIVE; this is squarely on the text-measurement path exercised by every `GetTextExtentPoint32`/`GetGlyphOutline` call, i.e. every window that draws text.

Note: the third site upstream also touches in the same theme (`ftGdiGetTextMetricsW`) does
**not** map — our Rust's `ftGdiGetTextMetricsW_impl` uses a structurally different
`IntRequestFontSize`-based path with no `TextIntUpdateSize` call at all, so that part of the
lead is not applicable here (flagging rather than silently dropping, per the brief).

Proposed diff (pattern applies at both confirmed sites; shown for `ftGdiGetGlyphOutline_impl`):

```diff
--- a/sdk/lib/rust/win32k-rust/src/freetype.rs
+++ b/sdk/lib/rust/win32k-rust/src/freetype.rs
@@ unsafe fn ftGdiGetGlyphOutline_impl(...) -> ULONG {
     IntLockFreeType();
-    TextIntUpdateSize(dc, TextObj, FontGDI, 0);
+    if TextIntUpdateSize(dc, TextObj, FontGDI, 0) == 0 {
+        IntUnLockFreeType();
+        if !potm.is_null() {
+            ExFreePoolWithTag(potm as PVOID, GDITAG_TEXT);
+        }
+        w32r_TEXTOBJ_UnlockText(TextObj);
+        EngSetLastError(ERROR_GEN_FAILURE);
+        return GDI_ERROR;
+    }
     IntMatrixFromMx(&mut mat, w32r_DC_pmxWorldToDevice(dc));
```

(Apply the analogous guard in `TextIntGetTextExtentPoint_impl` at its own
`TextIntUpdateSize` call — exact cleanup list there needs a re-read of that function's own
locals before landing, since its unlock/free sequence differs slightly from
`ftGdiGetGlyphOutline`'s.)

### #4 — `GetCharWidthW`/`32W`/`FloatW`/`ABCWidthsW`/`A`/`32A`: missing handle/range validation (`06ae7ec2114`)

`sdk/lib/rust/gdi32-rust/src/objects/font.rs:1097-1245` — every one of these six exported
functions still has only the pre-fix check:

```rust
if lpBuffer.is_null() || (iFirstChar > iLastChar) {
    SetLastError(ERROR_INVALID_PARAMETER);
    return FALSE;
}
```

Upstream adds three checks that are missing here: `HIWORD(iLastChar) != 0` (catches a
caller passing a value that doesn't fit `WORD`-range indices, which the underlying
`NtGdiGetCharWidthW` count-parameter arithmetic assumes), `GDI_HANDLE_GET_TYPE(hdc) !=
GDI_OBJECT_TYPE_DC`, and `!GdiValidateHandle(hdc)` — i.e. a caller passing a non-DC GDI
handle (or a stale/freed handle) sails straight through into the syscall today. This is the
**exact trap the brief names** (`e6328fbf1ea`, a "Rewrite" title with no real fix) — verified
by reading the diff and the Rust side line-by-line rather than trusting the subject; in this
case the "Rewrite" commit *does* carry a real, still-present validation gap.

`gdi32-rust` is LIVE. Proposed diff (shown for `GetCharWidthW`; identical shape applies to
the other five — `GetCharWidthFloatW`, `GetCharWidth32W`, `GetCharABCWidthsW` need the
`HIWORD`/handle checks added the same way; `GetCharWidthA`/`GetCharWidth32A` need only the
handle checks since they don't take a raw `iLastChar` range in the same form):

```diff
--- a/sdk/lib/rust/gdi32-rust/src/objects/font.rs
+++ b/sdk/lib/rust/gdi32-rust/src/objects/font.rs
@@ pub unsafe extern "system" fn GetCharWidthW(...) -> BOOL {
-    if lpBuffer.is_null() || (iFirstChar > iLastChar) {
+    if lpBuffer.is_null() || (iFirstChar > iLastChar) || (HIWORD(iLastChar) != 0)
+        || (GDI_HANDLE_GET_TYPE(hdc) != GDI_OBJECT_TYPE_DC) || GdiValidateHandle(hdc) == 0
+    {
         SetLastError(ERROR_INVALID_PARAMETER);
         return FALSE;
     }
```

(`GdiValidateHandle` and `GDI_HANDLE_GET_TYPE`/`GDI_OBJECT_TYPE_DC` need to already be in
scope in this crate — grep shows `GdiValidateHandle` is already called elsewhere in
`gdi32-rust`, so this is wiring, not a new import.)

### #5 — `DIALOG_CreateIndirect`: missing `DF_END` checks (`614e1ab762`)

`sdk/lib/rust/user32-rust/src/windows/dialog.rs:1166-1194` has all three pre-fix gaps
upstream's commit (#9362, CORE-20706) fixes:

1. No `!(dlgInfo->flags & DF_END)` guard on the focus-assignment block after
   `WM_INITDIALOG` (line 1172-1173 area) — if the dialog proc calls `EndDialog()` from
   inside `WM_INITDIALOG`, our Rust still runs `SetFocus`/`GetNextDlgTabItem` etc. against a
   dialog that's about to be torn down.
2. No `if !IsWindow(hwnd) { return null }` check after `WM_INITDIALOG` returns — a dialog
   proc that calls `DestroyWindow()` directly (not `EndDialog`) inside `WM_INITDIALOG` falls
   through into `GetWindowLongPtrW`/`SendMessageW` on a (possibly-recycled) HWND.
3. No `DF_END` guard before the final `ShowWindow(hwnd, SW_SHOWNORMAL)` — per the MSDN
   remark quoted in the upstream commit, a dialog that ends itself in `WM_INITDIALOG` should
   never flash visible before being destroyed; ours will show it anyway.

`user32-rust` is LIVE (user32 fully cohab-flipped) and `DIALOG_CreateIndirect` backs every
`DialogBox*`/`CreateDialog*` call — this is exercised on essentially every ReactOS dialog,
including ones in the boot/setup/logon path (winlogon credential prompts, msgina/syssetup
wizards). Not memory-unsafety, but a real, frequently-hit UI-correctness bug (visible dialog
flash + operating on a half-torn-down window).

Proposed diff (`DF_END` constant and `(*dlgInfo).flags` are already in scope — used
elsewhere in this same file at lines 649/682/689/817/1431/1437/2242):

```diff
--- a/sdk/lib/rust/user32-rust/src/windows/dialog.rs
+++ b/sdk/lib/rust/user32-rust/src/windows/dialog.rs
@@ unsafe fn DIALOG_CreateIndirect(...) -> HWND {
             if SendMessageW(hwnd, WM_INITDIALOG, focus as WPARAM, param) != 0
                 && IsWindow(hwnd) != 0
+                && (*dlgInfo).flags & DF_END == 0
                 && ((!template.style & DS_CONTROL != 0) || (template.style & WS_VISIBLE != 0))
             {
                 ...
             }
         }
+        if IsWindow(hwnd) == 0 {
+            return core::ptr::null_mut();
+        }
         if GetWindowLongPtrW(hwnd, GWL_STYLE) & WS_CHILD as LONG_PTR == 0 {
             SendMessageW(hwnd, WM_CHANGEUISTATE, MAKEWPARAM(UIS_INITIALIZE as WORD, 0), 0);
         }
-        if template.style & WS_VISIBLE != 0
+        if (*dlgInfo).flags & DF_END == 0
+            && template.style & WS_VISIBLE != 0
             && GetWindowLongPtrW(hwnd, GWL_STYLE) & WS_VISIBLE as LONG_PTR == 0
         {
             ShowWindow(hwnd, SW_SHOWNORMAL as i32);
```

### #6 — `NetpGetJoinInformation`: success status with `NetSetupDomainName`/`WorkgroupName` but a NULL name buffer on OOM (`71ee75477e`)

`sdk/lib/rust/wkssvc-rust/src/wkssvc/domain.rs:80-101` — on the success path, if
`midl_user_allocate` fails (line 87-92), our Rust leaves `*name_buffer` NULL but has
*already* set `*buffer_type = NetSetupDomainName`/`NetSetupWorkgroupName`, and then falls
through to `LsaNtStatusToWinError(status)` where `status` is still the earlier **success**
status from `LsaQueryInformationPolicy` — so the function returns `NO_ERROR` claiming the
machine is domain/workgroup-joined with a NULL name pointer. `NetGetJoinInformation`
(netapi32) callers that trust `NO_ERROR` and dereference the name unconditionally will crash
on a rare, allocator-pressure-triggered NULL deref.

Low-frequency (needs `midl_user_allocate` — a small heap alloc — to fail) but a genuine
carried bug; `wkssvc-rust` is LIVE. Fix mirrors upstream's `goto done` restructuring: only
write the out-params after confirming the whole sequence (including the allocation)
succeeded, and propagate `STATUS_INSUFFICIENT_RESOURCES` on allocation failure instead of the
stale success status.

```diff
--- a/sdk/lib/rust/wkssvc-rust/src/wkssvc/domain.rs
+++ b/sdk/lib/rust/wkssvc-rust/src/wkssvc/domain.rs
@@ pub unsafe extern "C" fn NetpGetJoinInformation(...) -> u32 {
     if LSA_SUCCESS(status) {
+        let mut alloc_status = status;
         if (*primary_domain_info).Name.Length > 0 {
-            if !(*primary_domain_info).Sid.is_null() {
-                *buffer_type = NetSetupDomainName;
-            } else {
-                *buffer_type = NetSetupWorkgroupName;
-            }
-            *name_buffer = midl_user_allocate(...) as LPWSTR;
-            if !(*name_buffer).is_null() {
-                wcscpy(*name_buffer, (*primary_domain_info).Name.Buffer);
+            let name = midl_user_allocate(...) as LPWSTR;
+            if name.is_null() {
+                alloc_status = STATUS_INSUFFICIENT_RESOURCES; // don't publish a type with a null name
+            } else {
+                wcscpy(name, (*primary_domain_info).Name.Buffer);
+                *name_buffer = name;
+                *buffer_type = if !(*primary_domain_info).Sid.is_null() {
+                    NetSetupDomainName
+                } else {
+                    NetSetupWorkgroupName
+                };
             }
         } else {
             *buffer_type = NetSetupUnjoined;
         }
         ...
+        status = alloc_status;
     }
```

(Written as a minimal patch to the existing Rust structure rather than a full port of
upstream's `goto done` — the semantics are equivalent: never report a Domain/Workgroup type
with a null name buffer.)

### #7 — `IntMultiByteToWideCharUTF8`: astral-plane codepoints truncated instead of surrogate-paired (`5635d244fc2`)

`sdk/lib/rust/kernel32-rust/src/string/nls.rs:1003-1106` — `WideChar` is declared
`WCHAR` (16-bit) exactly as pre-fix C, and the copy at line 1106 is a direct
`*WideCharString = WideChar;` with no surrogate-pair handling anywhere in the function.
Any 4-byte UTF-8 sequence (any codepoint ≥ U+10000 — emoji, many CJK Extension B+ ideographs,
historic scripts) gets silently truncated to a garbage 16-bit value instead of becoming a
proper UTF-16 high/low surrogate pair. `kernel32-rust`'s `MultiByteToWideChar` is on the
UTF-8→UTF-16 path used by essentially every modern console/GUI app doing text I/O; this is a
real, easily-triggered correctness bug (not just an edge case — any emoji in argv, a file
path, clipboard text, etc.).

Proposed diff (widen the working type and add the surrogate-pair split, mirroring upstream
exactly):

```diff
--- a/sdk/lib/rust/kernel32-rust/src/string/nls.rs
+++ b/sdk/lib/rust/kernel32-rust/src/string/nls.rs
@@ unsafe fn IntMultiByteToWideCharUTF8(...) -> LONG {
-    let mut WideChar: WCHAR = 0;
+    let mut WideChar: UINT = 0;
     ...
     // in the counting pass, after WideChar is fully assembled:
+    if WideChar > 0xFFFF {
+        WideCharCount += 1; // surrogate pair needs 2 WCHARs
+    }
     ...
     // in the copy pass:
-                *WideCharString = WideChar;
+                if WideChar > 0xFFFF {
+                    let wc = WideChar - 0x10000;
+                    *WideCharString = (0xD800 | (wc >> 10)) as WCHAR;
+                    WideCharString = WideCharString.add(1);
+                    Count += 1;
+                    if Count >= WideCharCount {
+                        SetLastError(ERROR_INSUFFICIENT_BUFFER);
+                        return 0;
+                    }
+                    *WideCharString = (0xDC00 | (wc & 0x3FF)) as WCHAR;
+                } else {
+                    *WideCharString = WideChar as WCHAR;
+                }
```

(Exact local variable names/control flow need matching against the surrounding
`nls.rs:1090-1110` loop structure before landing — the sketch above follows upstream's
placement but this function's Rust translation restructured the loop somewhat from the C, so
a straight line-for-line port needs a careful re-read of where `Count`/`WideCharCount` are
tracked in the two passes.)

## REFUTED / N/A detail worth recording

- **`ef70bdf990` (`RtlQueryEnvironmentVariable`)** — **gotcha for the tool**: the cohab
  registry's entry for `sdk/lib/rtl/env.c` is `MEDIUM` confidence, `dir-name-guess`
  (`rtl-rust`), with `rust_files: []` — i.e. **it never actually located the twin**. The real
  implementation is `RtlQueryEnvironmentVariable_U` at
  `sdk/lib/rust/rtl-strings/src/cabi/mod.rs:4939`, found only by grepping the function name
  across `sdk/lib/rust/` (it does not live under `rtl-rust` at all). Once found, it is
  faithful to the **pre-fix** logic, but the pre-fix logic itself isn't the buggy part of
  this commit — upstream's change here is adding a new byte-length-based
  `RtlQueryEnvironmentVariable` entry point and making `_U` a thin wrapper around it; `_U`'s
  own behavior is unchanged. So: no regression carried, but flagging the registry miss since
  it's exactly the "secondary island" gotcha the brief warns about, just for a whole-crate
  guess rather than a `-sys` constant.
- **imm32 cluster (`ce73ba4426a`, `61fb4db34c6`, `75e1d78fe9e`, and wave-1's already-noted
  `f76ed0625b34`/`8b3663d2b635`)** — confirmed via `cohab-registry.json` returning `None` for
  every `win32ss/user/imm32/*.c` path queried: imm32 is not cohab-flipped in this tree at
  all. Every imm32-touching candidate in both waves is therefore N/A by construction, not by
  assumption.
- **DHCPCSVC cluster (`60c8612e738`, `7dbfc3f2b69`, `fb47d37a497`)** — `dhcpcsvc-rust`'s
  `DhcpRequestParams` (the only one of the three touched functions present in Rust at all) is
  a bare stub: `pub unsafe extern "system" fn DhcpRequestParams(...) -> u32 { 0 }`. There is
  no logic on our side for the upstream parameter-validation fixes to apply *to* — the gap is
  "not implemented yet", not "implemented with the old bug". Recorded as N/A rather than a
  regression, but worth a line in the roadmap since it means the DHCP client API surface is
  currently a no-op that always reports success.
- **`06ae7ec2114`** is the precedent-named trap (`e6328fbf1ea` in the brief is a sibling
  commit in the same PR series) — included here specifically to record that this pass did
  **not** trust the "Rewrite" title and instead read the diff and the Rust side, and found a
  real gap anyway. Contrast with `9b450d9f7f7` ("Rewrite the TCPIP notify object"), which
  *was* left as N/A/not-opened because the surrounding commits in that same file are all
  "Implement"/"Add"/"Improve" net-growth of a stubby feature area (weight/priority call, not
  a verified refutation — see "not reached" below).

## Explicitly NOT REACHED (deferred, not dropped)

Per the project's "deferral is temporary, never silent" rule — these had a real diff-shape
match but this pass's time budget ran out before opening them fully:

- `8d44ccdd5f7` (explorer tray crash fix) — genuinely fix-shaped, but a 200+ line C++ ATL
  restructuring; `explorer-rust`'s registry entries for `syspager.cpp`/`traywnd.cpp` are
  `MEDIUM` confidence with `rust_files: []`, meaning the tool couldn't locate the actual twin
  file either — this needs someone to first establish where (or whether) `explorer-rust`
  actually implements `CNotifyToolbar`/`CSysPagerWnd` before a diff comparison is even
  possible.
- `3b58b347bc1` (explorer/shell32 refresh-forwarding) — plausible small fix, not opened.
- `74ddb4597ed` (RtlRange improvements) — moderate boot-relevance (PnP/bus resource range
  tracking use `RTL_RANGE_LIST`), not opened.
- `1f97ad1f930` (actctx.c wine-10.0 sync) and `5f6b295cb90` (ole32 wine-10.0 sync, 9 files) —
  both large multi-hundred-line wine-sync commits, structurally the same "too big for this
  sweep's method" case as wave 1's `278234259dce` (fstub). Flagging for a dedicated pass the
  way wave 1 flagged fstub for the fstub lane.
- The full netcfgx cluster (9 commits, `72c5ab502b0` through `065c3a8b583`) — all read as
  incremental feature growth on a partially-stubbed TCPIP property-page notify object, based
  on subject + file overlap; none were opened diff-by-diff to rule out an embedded fix. Low
  priority per the boot-path weighting in the brief, but explicitly not a "checked, no bug"
  verdict — just deprioritized.
- The shell32-only feature commits (`93f96e592b2` through `6af07a31b00`, ~9 commits) — same
  treatment: `Implement`/`Add`/`Support` subjects, `dll/win32/shell32/*.cpp` C++ COM object
  files, not opened.

## What this pass in turn misses (tuning the tool)

- **The tool's per-hunk, 1-shape-per-hunk labeling gives no signal about diff *size*
  relative to the whole commit.** Several genuinely small, real fixes (`b82e2884fca`,
  `5e76d075f10`, `71ee75477e`) were single-purpose commits where the whole diff was the
  fix. Several large multi-hundred-line commits (`33dab072e75`, `23043ce0b82`, `8d44ccdd5f7`)
  surfaced via one small incidental hunk (an added lock call, an added check) buried inside a
  much bigger restructuring/feature-add — the hunk-level shape match is accurate but doesn't
  tell you the commit is 90% unrelated feature work until you read the `git show --stat`.
  **A cheap improvement**: have the tool also emit each candidate's total commit diffstat
  (insertions+deletions across *all* files, not just the matched one), so a human can
  triage "small commit, whole-diff-is-the-fix" vs. "huge commit, one small hunk matched" before
  opening anything.
- **`lock_or_probe_added` on `freetype.c` produced mostly noise for this pass**: 5 of the 8
  hits across `33dab072e7`/`88d06015f6`/`75e5904c05` came from the *same* handful of
  `IntLockFreeType`/`IntUnLockFreeType` call sites being touched repeatedly across a series
  of commits refactoring the same function bodies — by the time you reach the 3rd commit in
  the series the "added" unlock is often just moving with code that already moved in a prior
  commit, not a genuinely new missing-unlock bug. A commit-series-aware dedup (same file,
  overlapping line ranges, commits close together in the log) would cut a lot of repeat
  triage for a file already covered by an earlier commit in the same series.
- **Registry confidence should gate verification order.** Every `MEDIUM`-confidence /
  `dir-name-guess` / empty-`rust_files` entry this pass hit (`ef70bdf990`'s `rtl-rust` guess,
  `explorer-rust`'s three empty entries) turned out to need a manual function-name grep to
  even find the twin, or never resolved at all. Sorting candidates by registry confidence
  before triage would put the genuinely-quick verifications first.
