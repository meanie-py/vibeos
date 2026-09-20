# Upstream-sync verified findings — wave 2

Continuation of `upstream-sync-verified-findings.md` (leads #3/#4/#10/#11 in the ranked queue,
all fixed and landed already, plus #6 landed separately). This pass works ranks #5, #7, #8, #9,
#12, #13, #14, #15, #16, #17, #18, #19, #20 from `upstream-sync-bug-leads.md`. Ranks #1/#2 are
typing/comment cleanups the queue doc itself flags as low-value and are skipped; ranks #6/#10/#11
were already confirmed+fixed before this pass started (commits `4d7ef310adf`, `f28f4aa70b0`) and
are not repeated here.

Method: read `git show <sha>` directly (never trust the subject line), locate the Rust twin via
`sdk/lib/rust/**` + `pub mod` reachability in the owning crate's `lib.rs`, diff pre-fix C
(`git show <sha>^:<path>`) and post-fix C against the Rust line-for-line, check
`git log --oneline -- <rust file>` for an independent fix.

## Verdict table

| # | sha | what the bug is | our Rust file:line | LIVE? | verdict |
|---|---|---|---|---|---|
| 5 | `640e39c72e` | TIF_MOVESIZETRACKING hack in `ENTRY_ReferenceEntryByHandle` lets a window mid-move/resize bypass the GDI-handle PID-ownership check for ALL its children's DCs — upstream reverted it outright because it crashes (`DceUpdateVisRgn` locks a region it doesn't own, `REGION_vSyncRegion` then touches a user-mode attribute living in another process) | `sdk/lib/rust/win32k-rust/src/gdiobj.rs:701` | yes (`pub mod gdiobj;`) | **CONFIRMED** |
| 7 | `64f1cf3b6d` | `TEXT_PathEllipsify`: when no path separator exists, `lastSlash` is set to the *start* of the string instead of the *end*, so DT_PATH_ELLIPSIS/DT_WORD_ELLIPSIS corrupts plain (no-slash) strings from the front | `sdk/lib/rust/win32k-rust/src/rtl_text.rs:246-248` | yes (`pub mod rtl_text;`) | **CONFIRMED** |
| 8 | `c6410fb0c2` | `QSI_SystemTimeAdjustmentInformation` only writes `*ReqSize` on the too-small failure path; the standard "call once for size" pattern gets `ReturnLength == 0` on success | `sdk/lib/rust/ex-rust/src/sysinfo.rs:1694-1703` | yes (registered in `CALL_QS[28]`; caller sets `*ReturnLength = CapturedResultLength` unconditionally after a successful query) | **CONFIRMED** |
| 9 | `feb0f56da7` | `QSI_SystemInterruptInformation` indexes `DpcData[0]` by magic number instead of the named `DPC_NORMAL` constant; also never sets `*ReqSize` at all (pre-fix C behavior, not itself part of this commit) | `sdk/lib/rust/ex-rust/src/sysinfo.rs:1595-1610` | yes (`CALL_QS[23]`) | **CONFIRMED** (low severity today: `DPC_NORMAL == 0`, so no behavior change yet — pure fragility) |
| 12 | `7f79e1c948` | `co_UserShowCaret`'s `EVENT_OBJECT_SHOW` notification passes `OBJID_CARET` as both 3rd and 4th arg; every sibling caret event correctly passes `CHILDID_SELF` as the 4th | `sdk/lib/rust/win32k-rust/src/caret.rs:407` | yes (`pub mod caret;`) | **CONFIRMED** |
| 13 | `3511472586` | `UserSendMouseInput`'s XBUTTON1-release branch reports `XBUTTON2` in `wParam`'s high word instead of `XBUTTON1` | `sdk/lib/rust/win32k-rust/src/ntmouse.rs:388` | yes (`pub mod ntmouse;`) | **CONFIRMED** |
| 14 | `a94352ba48` | Upstream fixes a C operator-precedence bug (`a < (b ? x : y)` instead of `(a < b) ? x : y`) in `EngTransparentBlt`'s direction pick | `sdk/lib/rust/win32k-rust/src/transblt.rs:197-206` | yes (`pub mod transblt;`) | **REFUTED** — the translation already reads `if OutputRect.left < InputRect.left { CD_RIGHTDOWN } else { CD_LEFTDOWN }` (and the `_UP` sibling identically) |
| 15 | `984657bdf0` | `EngSetPointerShape` computes the exclude rect's `top` from `HotSpot.x` instead of `HotSpot.y` | `sdk/lib/rust/win32k-rust/src/mouse.rs:638` | yes (`pub mod mouse;`) | **CONFIRMED** — sibling `EngMovePointer` (line 675, same file) already has the correct `HotSpot.y`, confirming this is a copy-paste bug, not a deliberate choice |
| 16 | `5b2f79fe07` | `RtlVirtualUnwind`: (a) `UWOP_SAVE_NONVOL_FAR`/`UWOP_SAVE_XMM128_FAR` scale `Offset` by the pointee size via `.add(Offset)` on a typed pointer, when `Offset` is already a raw byte count; (b) `UWOP_EPILOG` advances `i` by 1 instead of 2; (c) `UWOP_SPARE_CODE` advances by 2 instead of 3 | `sdk/lib/rust/rtl-except/src/amd64/unwind.rs:538-558,572-579` | yes (`pub mod unwind;` under `amd64`) | **CONFIRMED** |
| 17 | `7f459b49f0` | (a) `IntGdiFillRgn` paints with `pdcattr->ptlBrushOrigin` instead of the window-adjusted `pdc->ptlFillOrigin`; (b) `NtGdiGetPixel` builds `rcDest` from the *logical* `x,y` instead of the device-translated `ptlSrc.x,ptlSrc.y` used for the actual pixel read | `sdk/lib/rust/win32k-rust/src/ntbitblt.rs:1853` (a); `:2145-2148` and `:2249-2252` (b, both `#[cfg(target_arch = "x86")]` and the `not(x86)` twin) | yes (`pub mod ntbitblt;`) | **CONFIRMED** |
| 18 | `33a19d05f4` | `RtlCreateAtomTable` never initializes `Table->Signature` (`'motA'`) | `sdk/lib/rust/rtl-strings/src/cabi/mod.rs:8195-8225` | yes (`#[no_mangle] pub unsafe extern "system" fn RtlCreateAtomTable`, `pub mod cabi;`) | **CONFIRMED** |
| 19 | `0e64048812` | Multi-part: (a) `Heap->AlignMask` computed via a `(ULONG)` cast, truncating the mask to 32 bits on 64-bit; (b) `PS_JOB_TOKEN_FILTER`'s `pshpack1` scope leaks into `EJOB`, misaligning ARM64 kernel fields; (c) `STACK_TRACE_DATABASE`'s `ERESOURCE` padding/C_ASSERTs were `_M_AMD64`-only, wrong on ARM64 Win64 | `sdk/lib/rust/rtl-heap/src/heap.rs:315,318` (a); `sdk/lib/rust/ntkrnl-sys/src/bindgen_{amd64,arm64}.rs` `_EJOB`/`_PS_JOB_TOKEN_FILTER` (b); no Rust twin found for (c) | (a) not applicable | (a) **REFUTED**, (b)/(c) **INCONCLUSIVE** |
| 20 | `713888355b` | `DPRINT1` → `DPRINT` log-level-only change in `FontLink_PopulateEntries`, no behavioral difference | `sdk/lib/rust/win32k-rust/src/freetype.rs` (not investigated further — moot) | n/a | **REFUTED** (cosmetic, not a bug) |

Counts: **10 CONFIRMED / 2 REFUTED / 1 mixed REFUTED+INCONCLUSIVE (#19)**. Ranks #1, #2, #6, #10,
#11 not repeated (see header).

---

## 5. `640e39c72e` — TIF_MOVESIZETRACKING GDI-handle hack (CORE-20720, CORE-5601)

### Upstream hunk (a revert)
```diff
--- a/win32ss/gdi/ntgdi/gdiobj.c
+++ b/win32ss/gdi/ntgdi/gdiobj.c
@@ -479,13 +479,6 @@ ENTRY_ReferenceEntryByHandle(HGDIOBJ hobj, FLONG fl)
 {
     ULONG ulIndex, cNewRefs, cOldRefs;
     PENTRY pentry;
-    PTHREADINFO pti = PsGetCurrentThreadWin32Thread();
-
-    /* HACK: This may be a hack but it fixes CORE-5601.
-     * Allow a window that is moving or resizing to have access to all of its child
-     * windows dc's even if the dc belongs to another process i.e. 3D Screensaver */
-    if (pti && pti->TIF_flags & TIF_MOVESIZETRACKING)
-        fl = GDIOBJFLAG_IGNOREPID;
```
Upstream's own commit message: the hack causes kernel crashes — `DceUpdateVisRgn` locks a region
it does not own, `REGION_LockRgn` wrongly succeeds, `REGION_vSyncRegion` then dereferences a
user-mode region attribute living in a *different process*. This is not "revert an improvement
we'd rather keep" — it's "delete a landmine."

### Our Rust (`gdiobj.rs:693-702`)
```rust
unsafe fn ENTRY_ReferenceEntryByHandle(hobj: HGDIOBJ, fl: FLONG) -> PENTRY {
    ...
    let pti: PTHREADINFO = PsGetCurrentThreadWin32Thread();
    let mut fl = fl;

    if !pti.is_null() && ((*pti).TIF_flags & TIF_MOVESIZETRACKING) != 0 {
        fl = GDIOBJFLAG_IGNOREPID;
    }
    ...
```
Exact match of the reverted hack. We forked after the hack landed (`ad6cb32fa5c`, #8402) but
before the revert (`640e39c72e`, #9389), so the straight-line translation carried the landmine
forward. `git log --oneline -- gdiobj.rs` shows no independent fix touching this function.

### Proposed diff
```diff
--- a/sdk/lib/rust/win32k-rust/src/gdiobj.rs
+++ b/sdk/lib/rust/win32k-rust/src/gdiobj.rs
@@ -697,12 +697,7 @@ unsafe fn ENTRY_ReferenceEntryByHandle(hobj: HGDIOBJ, fl: FLONG) -> PENTRY {
     let mut cOldRefs: u32;
     let pentry: PENTRY;
-    let pti: PTHREADINFO = PsGetCurrentThreadWin32Thread();
     let mut fl = fl;
 
-    if !pti.is_null() && ((*pti).TIF_flags & TIF_MOVESIZETRACKING) != 0 {
-        fl = GDIOBJFLAG_IGNOREPID;
-    }
-
     ulIndex = GDI_HANDLE_GET_INDEX(hobj);
```
If `pti`/`PsGetCurrentThreadWin32Thread` become unused elsewhere in the function after this,
drop the now-dead `let pti` line entirely (shown already removed above) and check for an unused
`PTHREADINFO` import warning.

---

## 7. `64f1cf3b6d` — DrawText ellipsification, no-slash case (CORE-20581)

### Upstream hunk
```diff
--- a/win32ss/user/rtl/text.c
+++ b/win32ss/user/rtl/text.c
@@ -287,7 +287,11 @@ static void TEXT_PathEllipsify (HDC hdc, WCHAR *str, unsigned int max_len,
     lastFwdSlash = strrchrW (str, FORWARD_SLASH);
 #endif
     lastSlash = lastBkSlash > lastFwdSlash ? lastBkSlash : lastFwdSlash;
+#ifdef __REACTOS__
+    if (!lastSlash) lastSlash = str + *len_str;
+#else
     if (!lastSlash) lastSlash = str;
+#endif
```
(Imported from Wine 11.3, wine-mirror/wine@5909e90.) When the string has no path separator,
`lastSlash` must point at the *end* of the string (so the ellipsis+shrink loop works backward
from there), not the start.

### Our Rust (`rtl_text.rs:241-249`)
```rust
lastSlash = if lastBkSlash as usize > lastFwdSlash as usize {
    lastBkSlash
} else {
    lastFwdSlash
};
if lastSlash.is_null() {
    lastSlash = str;                 // <-- pre-fix bug: should be str + *len_str
}
len_trailing = *len_str as i32 - (lastSlash.offset_from(str) as i32);
```
Exact pre-fix translation. Live (`pub mod rtl_text;`), called from the `DrawText`/ellipsis path
at line 690. `git log --oneline -- rtl_text.rs` shows only the original translation commit
(`68028a7c719`) plus unrelated later ABI-hygiene sweeps.

### Proposed diff
```diff
--- a/sdk/lib/rust/win32k-rust/src/rtl_text.rs
+++ b/sdk/lib/rust/win32k-rust/src/rtl_text.rs
@@ -244,7 +244,7 @@ unsafe fn TEXT_PathEllipsify(
     };
     if lastSlash.is_null() {
-        lastSlash = str;
+        lastSlash = str.offset(*len_str as isize);
     }
     len_trailing = *len_str as i32 - (lastSlash.offset_from(str) as i32);
```

---

## 8. `c6410fb0c2` — SystemTimeAdjustmentInformation ReqSize not set on success

### Upstream hunk
```diff
--- a/ntoskrnl/ex/sysinfo.c
+++ b/ntoskrnl/ex/sysinfo.c
@@ -1719,9 +1719,9 @@ QSI_DEF(SystemTimeAdjustmentInformation)
-    if (sizeof(SYSTEM_QUERY_TIME_ADJUST_INFORMATION) > Size)
+    *ReqSize = sizeof(SYSTEM_QUERY_TIME_ADJUST_INFORMATION);
+    if (Size != *ReqSize)
     {
-        * ReqSize = sizeof(SYSTEM_QUERY_TIME_ADJUST_INFORMATION);
         return STATUS_INFO_LENGTH_MISMATCH;
     }
```
Also tightens the check from "at least big enough" (`>`) to "exact match" (`!=`).

### Our Rust (`ex-rust/src/sysinfo.rs:1693-1703`)
```rust
unsafe extern "C" fn QSI_SystemTimeAdjustmentInformation(Buffer: PVOID, Size: ULONG, ReqSize: *mut ULONG) -> NTSTATUS {
    if (core::mem::size_of::<_SYSTEM_QUERY_TIME_ADJUST_INFORMATION>() as ULONG) > Size {
        *ReqSize = core::mem::size_of::<_SYSTEM_QUERY_TIME_ADJUST_INFORMATION>() as ULONG;
        return STATUS_INFO_LENGTH_MISMATCH;
    }
    let TimeInfo = Buffer as PSYSTEM_QUERY_TIME_ADJUST_INFORMATION;
    ...
    STATUS_SUCCESS
}
```
`*ReqSize` is only ever written on the failure path. The dispatcher
(`NtQuerySystemInformation`, `sysinfo.rs:2493-2528`) initializes `CapturedResultLength = 0`, then
after `query(...)` returns unconditionally does `*ReturnLength = CapturedResultLength` — so a
successful call to `NtQuerySystemInformation(SystemTimeAdjustmentInformation, ...)` with a
correctly-sized buffer returns `ReturnLength == 0`, not the struct size. Every other QSI handler
in the same file that has a size check (`QSI_SystemBasicInformation`,
`QSI_SystemProcessorInformation`, `QSI_SystemPerformanceInformation`,
`QSI_SystemDeviceInformation`, `QSI_SystemFlagsInformation`, `QSI_SystemProcessorPerformanceInformation`)
sets `*ReqSize` unconditionally before or independent of the check — this handler is the outlier,
consistent with a straight-line translation of the pre-fix C rather than a deliberate choice.
Live: registered at `t[28] = si_qs(...)` in `CALL_QS`. No independent fix in
`git log --oneline -- sysinfo.rs`.

### Proposed diff
```diff
--- a/sdk/lib/rust/ex-rust/src/sysinfo.rs
+++ b/sdk/lib/rust/ex-rust/src/sysinfo.rs
@@ -1694,8 +1694,8 @@ unsafe extern "C" fn QSI_SystemTimeAdjustmentInformation(Buffer: PVOID, Size: U
-    if (core::mem::size_of::<_SYSTEM_QUERY_TIME_ADJUST_INFORMATION>() as ULONG) > Size {
-        *ReqSize = core::mem::size_of::<_SYSTEM_QUERY_TIME_ADJUST_INFORMATION>() as ULONG;
+    *ReqSize = core::mem::size_of::<_SYSTEM_QUERY_TIME_ADJUST_INFORMATION>() as ULONG;
+    if Size != *ReqSize {
         return STATUS_INFO_LENGTH_MISMATCH;
     }
```
Note the `!=` tightening (exact size only) is upstream's own behavior change, applied faithfully
here; flag it in review since it is stricter than what we shipped, though it matches upstream's
verified intent for this specific struct (fixed-size, no trailing array).

---

## 9. `feb0f56da7` — SystemInterruptInformation magic DPC index

### Upstream hunk
```diff
--- a/ntoskrnl/ex/sysinfo.c
+++ b/ntoskrnl/ex/sysinfo.c
@@ -1561,7 +1561,8 @@ QSI_DEF(SystemInterruptInformation)
-    if(Size < KeNumberProcessors * sizeof(SYSTEM_INTERRUPT_INFORMATION))
+    *ReqSize = KeNumberProcessors * sizeof(SYSTEM_INTERRUPT_INFORMATION);
+    if (Size < *ReqSize)
     {
         return STATUS_INFO_LENGTH_MISMATCH;
     }
@@ -1572,7 +1573,7 @@ QSI_DEF(SystemInterruptInformation)
-        sii->DpcCount = Prcb->DpcData[0].DpcCount;
+        sii->DpcCount = Prcb->DpcData[DPC_NORMAL].DpcCount;
```

### Our Rust (`ex-rust/src/sysinfo.rs:1595-1610`)
```rust
unsafe extern "C" fn QSI_SystemInterruptInformation(Buffer: PVOID, Size: ULONG, _ReqSize: *mut ULONG) -> NTSTATUS {
    if Size < (KeNumberProcessors as ULONG) * core::mem::size_of::<_SYSTEM_INTERRUPT_INFORMATION>() as ULONG {
        return STATUS_INFO_LENGTH_MISMATCH;
    }
    ...
        (*sii).DpcCount = (*Prcb).DpcData[0].DpcCount as ULONG;   // magic index, matches pre-fix
    ...
```
Both the pre-fix magic index (`[0]`) and the pre-fix "never set ReqSize" behavior (note the
parameter is even named `_ReqSize`, i.e. explicitly marked unused) are present. `DPC_NORMAL` is
defined at value `0` in this tree (`ntkrnl_sys::hal_bindgen_{amd64,i386}::DPC_NORMAL = 0`,
confirmed via `krnlinit.rs`'s own use of `DpcData[DPC_NORMAL as usize]`), so this specific bug is
functionally inert *today* — a latent fragility, not an active wrong-value bug, unless/until
`KPRCB::DpcData` is ever reordered. The `_ReqSize` non-write predates this specific upstream
commit (pre-fix C never set it here either — the fix *adds* the `*ReqSize=` line), so unlike
lead #8 this isn't a regression the fix closes on our side alone; still worth fixing for
consistency with every other QSI handler in the file. Live: `t[23] = si_qx(...)`.

### Proposed diff
```diff
--- a/sdk/lib/rust/ex-rust/src/sysinfo.rs
+++ b/sdk/lib/rust/ex-rust/src/sysinfo.rs
@@ -1595,8 +1595,9 @@ unsafe extern "C" fn QSI_SystemInterruptInformation(Buffer: PVOID, Size: ULONG,
-unsafe extern "C" fn QSI_SystemInterruptInformation(Buffer: PVOID, Size: ULONG, _ReqSize: *mut ULONG) -> NTSTATUS {
-    if Size < (KeNumberProcessors as ULONG) * core::mem::size_of::<_SYSTEM_INTERRUPT_INFORMATION>() as ULONG {
+unsafe extern "C" fn QSI_SystemInterruptInformation(Buffer: PVOID, Size: ULONG, ReqSize: *mut ULONG) -> NTSTATUS {
+    *ReqSize = (KeNumberProcessors as ULONG) * core::mem::size_of::<_SYSTEM_INTERRUPT_INFORMATION>() as ULONG;
+    if Size < *ReqSize {
         return STATUS_INFO_LENGTH_MISMATCH;
     }
@@ -1604,7 +1605,7 @@
-        (*sii).DpcCount = (*Prcb).DpcData[0].DpcCount as ULONG;
+        (*sii).DpcCount = (*Prcb).DpcData[DPC_NORMAL as usize].DpcCount as ULONG;
```
Confirm `DPC_NORMAL` is in scope in `sysinfo.rs` (it's already imported/used transitively via
`ntkrnl_sys::ke::*`/`ntkrnl_sys::ddk::*` elsewhere in the crate — check the glob-use list at the
top of the file before landing).

---

## 12. `7f79e1c948` — Caret EVENT_OBJECT_SHOW copy-paste (#9138)

### Upstream hunk
```diff
--- a/win32ss/user/ntuser/caret.c
+++ b/win32ss/user/ntuser/caret.c
@@ -285,7 +285,7 @@ BOOL FASTCALL co_UserShowCaret(PWND Window OPTIONAL)
-         IntNotifyWinEvent(EVENT_OBJECT_SHOW, pWnd, OBJID_CARET, OBJID_CARET, 0);
+         IntNotifyWinEvent(EVENT_OBJECT_SHOW, pWnd, OBJID_CARET, CHILDID_SELF, 0);
```

### Our Rust (`caret.rs`)
Five `IntNotifyWinEvent` call sites in the file; four correctly pass `CHILDID_SELF` as the 4th
argument (lines 232, 268, 319, 478) and one — `EVENT_OBJECT_SHOW`, line 407 — passes
`OBJID_CARET` twice, matching pre-fix C exactly:
```rust
IntNotifyWinEvent(EVENT_OBJECT_SHOW, pWnd, OBJID_CARET as LONG, OBJID_CARET as LONG, 0);
```
Live (`pub mod caret;`). No independent fix.

### Proposed diff
```diff
--- a/sdk/lib/rust/win32k-rust/src/caret.rs
+++ b/sdk/lib/rust/win32k-rust/src/caret.rs
@@ -404,7 +404,7 @@
-            IntNotifyWinEvent(EVENT_OBJECT_SHOW, pWnd, OBJID_CARET as LONG, OBJID_CARET as LONG, 0);
+            IntNotifyWinEvent(EVENT_OBJECT_SHOW, pWnd, OBJID_CARET as LONG, CHILDID_SELF as LONG, 0);
```

---

## 13. `3511472586` — XBUTTON1 reported as XBUTTON2 on release (#9120)

### Upstream hunk
```diff
--- a/win32ss/user/ntuser/mouse.c
+++ b/win32ss/user/ntuser/mouse.c
@@ -326,7 +326,7 @@ UserSendMouseInput(MOUSEINPUT *pmi, BOOL bInjected)
-            Msg.wParam |= MAKEWPARAM(0, XBUTTON2);
+            Msg.wParam |= MAKEWPARAM(0, XBUTTON1);
```
Bug introduced upstream in r54227, over a decade before this fix.

### Our Rust (`ntmouse.rs:384-389`)
```rust
if (*pmi).mouseData & XBUTTON1 != 0 {
    ...
    Msg.wParam &= !(MK_XBUTTON1 as WPARAM);
    Msg.wParam |= MAKEWPARAM(0, XBUTTON2 as WORD) as WPARAM;   // <-- should be XBUTTON1
}
```
This is the XBUTTON1-release branch (guarded by `mouseData & XBUTTON1`), so every XBUTTON1
release reports XBUTTON2 to the app — exact pre-fix bug. Live (`pub mod ntmouse;`). No
independent fix.

### Proposed diff
```diff
--- a/sdk/lib/rust/win32k-rust/src/ntmouse.rs
+++ b/sdk/lib/rust/win32k-rust/src/ntmouse.rs
@@ -385,7 +385,7 @@
             (*pCurInfo).ButtonsDown &= !(MK_XBUTTON1 as UINT);
             Msg.wParam &= !(MK_XBUTTON1 as WPARAM);
-            Msg.wParam |= MAKEWPARAM(0, XBUTTON2 as WORD) as WPARAM;
+            Msg.wParam |= MAKEWPARAM(0, XBUTTON1 as WORD) as WPARAM;
```

---

## 14. `a94352ba48` — EngTransparentBlt direction pick — REFUTED

### Upstream hunk
```diff
--- a/win32ss/gdi/eng/transblt.c
+++ b/win32ss/gdi/eng/transblt.c
@@ -145,11 +145,11 @@
-                    Direction = OutputRect.left < (InputRect.left ? CD_RIGHTDOWN : CD_LEFTDOWN);
+                    Direction = OutputRect.left < InputRect.left ? CD_RIGHTDOWN : CD_LEFTDOWN;
```
The pre-fix C has misplaced parens: `a < (b ? x : y)` evaluates the ternary first (`b` truthy as
a boolean, not compared against `a`), producing nonsense — `Direction` ends up `0` or `1`
(`CD_RIGHTDOWN`/`CD_RIGHTUP` are apparently 0, from context) whenever `InputRect.left != 0`,
regardless of `OutputRect.left`.

### Our Rust (`transblt.rs:196-207`)
```rust
Direction = if OutputRect.left < InputRect.left {
    CD_RIGHTDOWN
} else {
    CD_LEFTDOWN
};
```
and the `_UP` sibling identically. This is the **correct** logic — `a < b ? x : y`, not the
buggy `a < (b ? x : y)`. The translator resolved the C operator-precedence ambiguity toward the
evidently-intended semantics rather than transcribing the (mis-parenthesized, but
syntactically-valid) literal C. Commented-out pre-translation source at line 577/581 shows the
buggy paren placement was seen and not reproduced. **REFUTED** — no action needed.

---

## 15. `984657bdf0` — EngSetPointerShape exclude-rect Y from HotSpot.x (#9116)

### Upstream hunk
```diff
--- a/win32ss/gdi/eng/mouse.c
+++ b/win32ss/gdi/eng/mouse.c
@@ -543,7 +543,7 @@ EngSetPointerShape(
         if (prcl != NULL)
         {
             prcl->left = x - pgp->HotSpot.x;
-            prcl->top = y - pgp->HotSpot.x;
+            prcl->top = y - pgp->HotSpot.y;
```

### Our Rust (`mouse.rs:636-641`, inside `eng_set_pointer_shape_impl`)
```rust
if x != -1 {
    ...
    if !prcl.is_null() {
        (*prcl).left = x - (*pgp).HotSpot.x;
        (*prcl).top = y - (*pgp).HotSpot.x;      // <-- pre-fix bug, should be .y
        (*prcl).right = (*prcl).left + (*pgp).Size.cx;
        (*prcl).bottom = (*prcl).top + (*pgp).Size.cy;
    }
}
```
Exact pre-fix bug. The sibling function in the same file, `eng_move_pointer_impl`
(`EngMovePointer`, lines 670-677), has the *correct* `(*prcl).top = y - (*pgp).HotSpot.y;` —
confirming this is a copy-paste-and-typo bug local to `EngSetPointerShape`, exactly matching
upstream's description ("crazy this typo survived two decades"). Live (`pub mod mouse;`,
win32k eng module). No independent fix.

### Proposed diff
```diff
--- a/sdk/lib/rust/win32k-rust/src/mouse.rs
+++ b/sdk/lib/rust/win32k-rust/src/mouse.rs
@@ -636,7 +636,7 @@
         if !prcl.is_null() {
             (*prcl).left = x - (*pgp).HotSpot.x;
-            (*prcl).top = y - (*pgp).HotSpot.x;
+            (*prcl).top = y - (*pgp).HotSpot.y;
             (*prcl).right = (*prcl).left + (*pgp).Size.cx;
```

---

## 16. `5b2f79fe07` — RtlVirtualUnwind UWOP mis-handling (amd64)

### Upstream hunk (relevant parts)
```diff
--- a/sdk/lib/rtl/amd64/unwind.c
+++ b/sdk/lib/rtl/amd64/unwind.c
@@ -581,22 +583,24 @@
             case UWOP_SAVE_NONVOL_FAR:
                 Offset = *(ULONG*)(&UnwindInfo->UnwindCode[i + 1]);
-                SetRegFromStackValue(Context, ContextPointers, Reg, (DWORD64*)Context->Rsp + Offset);
+                SetRegFromStackValue(Context, ContextPointers, Reg, (PDWORD64)(Context->Rsp + Offset));
                 i += 3;
                 break;
 
             case UWOP_EPILOG:
-                i += 1;
+                i += 2;
                 break;
 
             case UWOP_SPARE_CODE:
                 ASSERT(FALSE);
-                i += 2;
+                i += 3;
                 break;
 
             case UWOP_SAVE_XMM128_FAR:
                 Offset = *(ULONG*)(&UnwindInfo->UnwindCode[i + 1]);
-                SetXmmRegFromStackValue(Context, ContextPointers, Reg, (M128A*)Context->Rsp + Offset);
+                SetXmmRegFromStackValue(Context, ContextPointers, Reg, (M128A*)(Context->Rsp + Offset));
                 i += 3;
                 break;
```
Two distinct bug classes: (a) `(DWORD64*)Rsp + Offset` and `(M128A*)Rsp + Offset` scale `Offset`
by `sizeof(DWORD64)`=8 / `sizeof(M128A)`=16 via C pointer arithmetic, but `Offset` here is a raw
byte offset read straight out of the unwind code stream (`*(ULONG*)`) — pointer-arithmetic
scaling turns a small byte offset into a wildly wrong address. (b) `UWOP_EPILOG` and
`UWOP_SPARE_CODE` each consume one more slot than the loop advanced past, misaligning every
subsequent opcode in the unwind-code array for the rest of the parse.

### Our Rust (`rtl-except/src/amd64/unwind.rs`)
```rust
UWOP_SAVE_NONVOL_FAR => {
    let Reg = UnwindCode.OpInfo();
    let Offset = (unwind_code_ptr(UnwindInfo, i + 1) as *const ULONG).read_unaligned();
    SetRegFromStackValue(Context, ContextPointers, Reg,
        ((*Context).Rsp as *mut DWORD64).add(Offset as usize));   // <-- .add() scales by 8
    i += 3;
}

UWOP_EPILOG => {
    i += 1;      // <-- should be 2
}

UWOP_SPARE_CODE => {
    i += 2;      // <-- should be 3
}

UWOP_SAVE_XMM128_FAR => {
    let Reg = UnwindCode.OpInfo();
    let Offset = (unwind_code_ptr(UnwindInfo, i + 1) as *const ULONG).read_unaligned();
    SetXmmRegFromStackValue(Context, ContextPointers, Reg,
        ((*Context).Rsp as *mut M128A).add(Offset as usize));    // <-- .add() scales by 16
    i += 3;
}
```
Both bug classes present verbatim. **Additional finding not in the upstream diff**: this same
file has a *second*, independent implementation of "how many slots does opcode X occupy" —
`UnwindOpSlots()` (line 192), used by `GetEstablisherFrame`'s scan loop and by the
"skip ops past our offset" pre-loop in `RtlVirtualUnwind` (lines 425, 485). Its table already
encodes the **post-fix** slot counts (`UWOP_EPILOG` → 1 extra slot = 2 total,
`UWOP_SPARE_CODE` → 2 extra = 3 total — see the table's own inline comments, which literally cite
the old/new opcode names). So the codebase currently disagrees with itself: the *scan/skip*
paths already assume the fixed slot widths, while the *execute* path (the `match` arms actually
run when unwinding through an epilog or hitting a reserved opcode) still uses the old, wrong
widths. This means a real unwind that walks past a `UWOP_EPILOG`/`UWOP_SPARE_CODE` via the match
arm desyncs `i` relative to what the rest of the file (and any future maintainer reading
`UnwindOpSlots`) assumes is correct — worse than plain upstream pre-fix, which was at least
internally consistent in its wrongness.

Live: `pub mod unwind;` under `sdk/lib/rust/rtl-except/src/amd64/mod.rs`. `git log --oneline --
unwind.rs` shows two independent fixes (`RtlWalkFrameChain(NULL)`, general amd64 unwind port)
but neither touches these opcodes.

### Proposed diff
```diff
--- a/sdk/lib/rust/rtl-except/src/amd64/unwind.rs
+++ b/sdk/lib/rust/rtl-except/src/amd64/unwind.rs
@@ -538,7 +538,7 @@
                 UWOP_SAVE_NONVOL_FAR => {
                     let Reg = UnwindCode.OpInfo();
                     let Offset =
                         (unwind_code_ptr(UnwindInfo, i + 1) as *const ULONG).read_unaligned();
                     SetRegFromStackValue(
                         Context,
                         ContextPointers,
                         Reg,
-                        ((*Context).Rsp as *mut DWORD64).add(Offset as usize),
+                        ((*Context).Rsp as *mut u8).add(Offset as usize) as *mut DWORD64,
                     );
                     i += 3;
                 }
 
                 UWOP_EPILOG => {
-                    i += 1;
+                    i += 2;
                 }
 
                 UWOP_SPARE_CODE => {
                     /* ASSERT(FALSE) elided — NDEBUG */
-                    i += 2;
+                    i += 3;
                 }
@@
                 UWOP_SAVE_XMM128_FAR => {
                     let Reg = UnwindCode.OpInfo();
                     let Offset =
                         (unwind_code_ptr(UnwindInfo, i + 1) as *const ULONG).read_unaligned();
                     SetXmmRegFromStackValue(
                         Context,
                         ContextPointers,
                         Reg,
-                        ((*Context).Rsp as *mut M128A).add(Offset as usize),
+                        ((*Context).Rsp as *mut u8).add(Offset as usize) as *mut M128A,
                     );
                     i += 3;
                 }
```

---

## 17. `7f459b49f0` — bitblt.c brush-origin + GetPixel device-coordinate bugs

### Upstream hunk
```diff
--- a/win32ss/gdi/ntgdi/bitblt.c
+++ b/win32ss/gdi/ntgdi/bitblt.c
@@ -1274,7 +1274,7 @@ IntGdiFillRgn(
     bRet = IntEngPaint(&pdc->dclevel.pSurface->SurfObj,
                        (CLIPOBJ *)&xcoClip,
                        pbo,
-                       &pdc->pdcattr->ptlBrushOrigin,
+                       &pdc->ptlFillOrigin,
                        mix);
@@ -1547,10 +1547,10 @@ NtGdiGetPixel(
-    rcDest.left = x;
-    rcDest.top = y;
-    rcDest.right = x + 1;
-    rcDest.bottom = y + 1;
+    rcDest.left = ptlSrc.x;
+    rcDest.top = ptlSrc.y;
+    rcDest.right = ptlSrc.x + 1;
+    rcDest.bottom = ptlSrc.y + 1;
```
Two independent bugs: (a) pattern fills paint with the *unadjusted* brush origin instead of the
window-relative `ptlFillOrigin`, so pattern brushes shift when a window scrolls/moves; (b)
`NtGdiGetPixel` prepares the blit rect from the *logical* `x,y` while the actual pixel copy a few
lines later reads from the *device*-translated `ptlSrc` — `DC_vPrepareDCsForBlit` gets the wrong
rect whenever logical-to-device mapping isn't identity (any non-default viewport/window origin
or mapping mode).

### Our Rust (`ntbitblt.rs`)
(a) `IntGdiFillRgn`, line 1853:
```rust
bRet = IntEngPaint(
    &mut (*(*pdc).dclevel.pSurface).SurfObj,
    &mut xcoClip as *mut XCLIPOBJ as *mut CLIPOBJ,
    pbo,
    &mut (*(*pdc).pdcattr).ptlBrushOrigin,   // <-- should be (*pdc).ptlFillOrigin
    mix,
);
```
`(*pdc).ptlFillOrigin` already exists as a field and is maintained elsewhere in this same crate
(`dcobjs.rs:304-305`, `coord.rs:1739-1740`, `dclife.rs:397-398`, `dcutil.rs:805,864`) — the fix
is a one-line substitution using plumbing that already exists.

(b) `NtGdiGetPixel`, present **twice** (`#[cfg(target_arch = "x86")]` at line 2114 and
`#[cfg(not(target_arch = "x86"))]` at line 2218 — both cfg variants were translated from the
same pre-fix C and both carry the bug identically):
```rust
ptlSrc.x = x;
ptlSrc.y = y;
w32r_IntLPtoDP(pdc, core::ptr::addr_of_mut!(ptlSrc) as _, 1);
ptlSrc.x += (*pdc).ptlDCOrig.x;
ptlSrc.y += (*pdc).ptlDCOrig.y;

rcDest.left = x;          // <-- should be ptlSrc.x  (lines 2145 / 2249)
rcDest.top = y;            // <-- should be ptlSrc.y
rcDest.right = x + 1;      // <-- should be ptlSrc.x + 1
rcDest.bottom = y + 1;     // <-- should be ptlSrc.y + 1
```
Both confirmed against `git show 7f459b49f0^:win32ss/gdi/ntgdi/bitblt.c` (identical pre-fix
logic). Live (`pub mod ntbitblt;`). `git log --oneline -- ntbitblt.rs` shows only warnings/ABI
hygiene sweeps and a WM_CTLCOLOR-unrelated fix — no independent fix of either bug.

### Proposed diff
```diff
--- a/sdk/lib/rust/win32k-rust/src/ntbitblt.rs
+++ b/sdk/lib/rust/win32k-rust/src/ntbitblt.rs
@@ -1850,7 +1850,7 @@
     bRet = IntEngPaint(
         &mut (*(*pdc).dclevel.pSurface).SurfObj,
         &mut xcoClip as *mut XCLIPOBJ as *mut CLIPOBJ,
         pbo,
-        &mut (*(*pdc).pdcattr).ptlBrushOrigin,
+        &mut (*pdc).ptlFillOrigin,
         mix,
     );
@@ -2142,10 +2142,10 @@
-        rcDest.left = x;
-        rcDest.top = y;
-        rcDest.right = x + 1;
-        rcDest.bottom = y + 1;
+        rcDest.left = ptlSrc.x;
+        rcDest.top = ptlSrc.y;
+        rcDest.right = ptlSrc.x + 1;
+        rcDest.bottom = ptlSrc.y + 1;
@@ -2246,10 +2246,10 @@
-        rcDest.left = x;
-        rcDest.top = y;
-        rcDest.right = x + 1;
-        rcDest.bottom = y + 1;
+        rcDest.left = ptlSrc.x;
+        rcDest.top = ptlSrc.y;
+        rcDest.right = ptlSrc.x + 1;
+        rcDest.bottom = ptlSrc.y + 1;
```
(Both `NtGdiGetPixel` copies need the same edit — don't fix only one `#[cfg(...)]` variant.)

---

## 18. `33a19d05f4` — RtlCreateAtomTable never sets Signature

### Upstream hunk
```diff
--- a/sdk/lib/rtl/atom.c
+++ b/sdk/lib/rtl/atom.c
@@ -174,6 +174,7 @@ RtlCreateAtomTable(
     }
     /* initialize atom table */
+    Table->Signature = 'motA';
     Table->NumberOfBuckets = TableSize;
```

### Our Rust (`rtl-strings/src/cabi/mod.rs:8195-8225`)
```rust
pub unsafe extern "system" fn RtlCreateAtomTable(
    table_size: u32,
    atom_table: *mut *mut RTL_ATOM_TABLE,
) -> i32 {
    ...
    let table = RtlpAllocAtomTable(size);
    if table.is_null() { return STATUS_NO_MEMORY; }

    (*table).number_of_buckets = table_size;
    // no `(*table).Signature = ...` anywhere in this function or file
    ...
```
`grep -n "Signature" sdk/lib/rust/rtl-strings/src/cabi/mod.rs` returns zero hits — the
`RTL_ATOM_TABLE.Signature` field (confirmed present in the bindgen struct, and its offset
independently sanity-checked by `offset_check_{i386,amd64,arm64}.rs`) is never written by any
atom-table function in this crate. Exact pre-fix bug: the wine test this commit says it fixes
almost certainly reads back `Signature` and finds garbage/zero instead of `'motA'`
(0x41746F6D). Live: `#[no_mangle] pub unsafe extern "system" fn RtlCreateAtomTable`, reachable
through `rtl-strings`'s public `cabi` module. No independent fix.

### Proposed diff
```diff
--- a/sdk/lib/rust/rtl-strings/src/cabi/mod.rs
+++ b/sdk/lib/rust/rtl-strings/src/cabi/mod.rs
@@ -8210,6 +8210,8 @@ pub unsafe extern "system" fn RtlCreateAtomTable(
+    // RTL_ATOM_TABLE_SIGNATURE = 'motA' (upstream 33a19d05f4 — was never set).
+    (*table).Signature = 0x6d6f7441;
     (*table).number_of_buckets = table_size;
```
(Field name casing — `Signature` vs `signature`/`number_of_buckets`'s snake_case style — needs
to match whatever `RTL_ATOM_TABLE`'s Rust binding actually calls the field; verify against
`bindgen_amd64.rs`'s `_RTL_ATOM_TABLE` before landing, since this file otherwise uses
snake_case for other fields on the same struct (`number_of_buckets`) which suggests a hand-named
wrapper struct, not the raw bindgen type — check whether `RTL_ATOM_TABLE` here is a local
`#[repr(C)]` redefinition and add the field there if it's missing entirely, not just unset.)

---

## 19. `0e64048812` — heap AlignMask / EJOB packing / ARM64 padding — mixed

### (a) Heap->AlignMask truncation — REFUTED
Upstream:
```diff
-        Heap->AlignMask = (ULONG) ~15;
+        Heap->AlignMask = ~(ULONG_PTR)15;
-        Heap->AlignMask = (ULONG) ~(sizeof(HEAP_ENTRY) - 1);
+        Heap->AlignMask = ~(ULONG_PTR)(sizeof(HEAP_ENTRY) - 1);
```
The C bug: `AlignMask` is declared `ULONG_PTR` in the struct, but pre-fix code computed it
through a `(ULONG)` cast, truncating the high 32 bits on a 64-bit build (so bit 63..32 of the
mask read as 0 instead of 1, corrupting large-address alignment math).

Our Rust (`rtl-heap/src/types.rs:263`, `rtl-heap/src/heap.rs:315,318`):
```rust
pub AlignMask: ULONG_PTR,
...
(*Heap).AlignMask = !15usize;
...
(*Heap).AlignMask = !(core::mem::size_of::<HEAP_ENTRY>() - 1);
```
`!15usize` is a `usize` literal, and `core::mem::size_of::<T>() - 1` returns `usize` — both
already pointer-width, with no intermediate 32-bit cast anywhere in the expression. Rust's type
system structurally prevents the bug the C `(ULONG)` cast introduced: there is no narrowing step
to introduce it, because nothing here is ever typed as `ULONG`/`u32`. **REFUTED** — nothing to
fix.

### (b) PS_JOB_TOKEN_FILTER pack(1) leaking into EJOB, and (c) ARM64 STACK_TRACE_DATABASE padding — INCONCLUSIVE
Both are C **struct-layout** bugs (`#include <pshpack1.h>` / `<poppack.h>` placement, and
`#ifdef _M_AMD64` vs `#ifdef _WIN64` in header-only C_ASSERTs). Our Rust does not compile these
headers; struct layouts come from `sdk/lib/rust/ntkrnl-sys/src/bindgen_{amd64,arm64,i386}.rs`,
generated by running bindgen against the NDK headers *at generation time*. Whether our generated
`_EJOB`/`_PS_JOB_TOKEN_FILTER` on ARM64 have the pre-fix (leaked-pack) or post-fix layout depends
entirely on which header revision bindgen consumed when those files were generated — not
something `git show`/`git log` on the Rust tree can answer, since the generated file has no
comment tying it to a header commit, and `_EJOB`'s Rust definition (checked above, no
`#[repr(packed)]` or byte-for-byte layout in the visible fields) doesn't show an obvious
one-byte-packed run the way a leaked `pshpack1` would. Spot check found no `#[repr(packed)]` on
`_EJOB` on either arch and no immediately-adjacent tiny-field runs suggesting mis-packing, but
this is not a substitute for actually diffing generated offsets pre/post header-fix.
`STACK_TRACE_DATABASE`'s ARM64 `ERESOURCE` padding is a debug/diagnostic structure (stack-trace
database for heap/pool debugging) — no Rust twin was found under `sdk/lib/rust` in a quick
search, suggesting it may not be translated at all yet (not a false-negative worth chasing
further in this pass).

**Recommendation**: don't guess. Re-run (or diff) bindgen for `_EJOB`/`_PS_JOB_TOKEN_FILTER`
against `sdk/include/ndk/pstypes.h` at `0e64048812^` vs `0e64048812` and compare generated field
offsets on ARM64; if they differ, the earlier (buggy) offsets are what's frozen in our
`bindgen_arm64.rs` and every kernel-mode consumer of `EJOB` fields past `JobToken` on ARM64 is
reading the wrong offset — that would be a much bigger and more urgent finding than anything
else in this file, but it needs the header-diff step to confirm, which this pass didn't have
time to do safely (regenerating bindgen output is not a read-only operation appropriate for a
verification-only pass).

---

## 20. `713888355b` — freetype.c DPRINT1→DPRINT — REFUTED

Pure log-level change (`DPRINT1` always prints, `DPRINT` is debug-build-only), explicitly
described by its own author as cosmetic ("to avoid unnecessary log spam"). Per
`docs/engineering/agent-reference.md`'s macro model, both lower to the same no-op/print
`macro_rules!` shape in our translation with no functional difference either way. Not
investigated further in `freetype.rs` — there is no behavioral bug to confirm or refute here.
**REFUTED** as a "bug lead"; not worth a diff.

---

## Leads not reached in this pass

Ranks #21 and below (`FREELDR` UEFI/boot-menu fixes, `NDK`/`fltmgr` `OB_CLOSE_METHOD`,
`NtGdiCreateHalftonePalette`, `setupldr.c`, `CSRSRV` signature/doxygen cleanup, `GDI32`
`PolylineTo`/`PolyBezierTo`, `KSYSTEM_TIME` read/write, `USER32` `LookupIconIdFromDirectoryEx`
crash series (#43-45, three commits churning the same function — worth reading as one unit, not
three), the `NtUserConvertMemHandle`/`NtUserCreateAcceleratorTable` apitest-oracle pair (#46-47,
flagged in the queue doc as directly *runnable* against our Rust rather than needing a manual
diff), and the NETCFGX/DHCPCSVC/USER32_WINETEST tail) were not examined in this pass. The queue
doc's own guidance stands: #46/#47 are the highest-leverage remaining items since they ship an
executable oracle test rather than requiring a hand-read diff.
