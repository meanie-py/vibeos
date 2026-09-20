# Upstream-sync: win32k deleted-C-tree sweep

Scope: the 49 files that (a) our commit `8d3c33ee41e` ("win32k.sys: retire superseded
kernel-mode C tree — Rust-only") deleted from our tree, and (b) upstream (`reactos/reactos`
master) has continued to modify since our fork point `5213cf717cff6946945922e6a7e03d87b0a58e4f`
(`$FP`). Because the C is gone, no merge and no diff-against-our-C review will ever surface
these fixes; the only way to find them is to read upstream's commits directly and hunt for the
same bug pattern in our Rust translation, matched by **function name**, not filename.

Method: `git log --oneline $FP..upstream/master -- <file>` per file, `git show <sha>` read in
full (never the subject alone), Rust twin located by `#[no_mangle]` function name in
`sdk/lib/rust/win32k-rust/src/`, live/reachable check via `pub mod` in
`sdk/lib/rust/win32k-rust/src/lib.rs` (all functions below are declared and none of the modules
touched are excluded from the module tree — none were spot-checked as unreachable).
**Nothing in this document was applied.** Every proposed diff below is a suggestion for a
follow-up commit.

Already fixed before this pass (do not re-open): `c0f18d059a` (dib_stretchblt mask bounds),
`ae0e4ee305` (dib8bpp stride), `e1bf662d9a` (freetype ftGetFontUnicodeRanges locking),
`e7e1465812` (NtUserSBGetParms probes) — see `upstream-sync-verified-findings.md`.

## Coverage table

Only files with at least one commit examined in enough depth to classify are listed; see
"Not reached" at the bottom for the rest of the 49.

| file | upstream commits since fork | real bug fixes found | CONFIRMED vs our Rust |
|---|---|---|---|
| `win32ss/gdi/ntgdi/freetype.c` | 15 | 4 (+1 already fixed) | 2 clean + 1 large/architectural |
| `win32ss/user/ntuser/desktop.c` | 9 | 3 | 2 clean + 1 large/not fully verified |
| `win32ss/user/ntuser/sysparams.c` | 5 | 2 (+1 shared with desktop.c) | 2 clean |
| `win32ss/user/ntuser/winsta.c` | 5 (3 shared w/ desktop.c) | 1 unique | not verified (out of budget) |
| `win32ss/user/ntuser/menu.c` | 4 | 1 | 1 clean |
| `win32ss/user/ntuser/caret.c` | 3 (1 shared w/ sysparams.c) | 1 unique | 1 clean |
| `win32ss/user/ntuser/scrollbar.c` | 2 (+1 already fixed) | 1 (2 bugs in it) | 1 clean (2 sub-findings) |
| `win32ss/user/ntuser/accelerator.c` | 1 | 1 | 1 clean — **memory corruption** |
| `win32ss/user/ntuser/clipboard.c` | 1 | 1 | 1 clean — **BSOD (null deref)** |
| `win32ss/user/ntuser/painting.c` | 2 | 1 | 1 clean |
| `win32ss/gdi/ntgdi/font.c` | 4 | 0 (all refactors, overlap w/ freetype.c) | — |
| `win32ss/user/ntuser/ime.c` | 5 | 0 confirmed (feature/lock-type work) | not pursued |
| `win32ss/user/ntuser/input.c` | 4 | 0 confirmed (feature work) | not pursued |
| `win32ss/gdi/eng/mouse.c`, `win32ss/user/ntuser/mouse.c`, `win32ss/gdi/eng/transblt.c`, `win32ss/gdi/ntgdi/bitblt.c`, `win32ss/gdi/ntgdi/palette.c`, `win32ss/gdi/ntgdi/gdiobj.c`, `win32ss/gdi/ntgdi/polyfill.c`, `win32ss/gdi/ntgdi/path.c`, `win32ss/gdi/ntgdi/fillshap.c`, `win32ss/gdi/eng/surface.c` | 1 each | subject line reads as a real, self-contained fix in every case | **spotted, not yet verified against Rust** — highest-value queue for the next pass (see below) |

**Totals this pass: 13 CONFIRMED / 0 REFUTED / 3 INCONCLUSIVE (flagged large/architectural,
not fully diffed) / a further ~10 single-commit files identified as likely real fixes but not
yet checked against the Rust twin.**

---

## Verdict table (this pass' new findings)

| # | sha | file | bug | our Rust file:line | verdict |
|---|---|---|---|---|---|
| 1 | `88d06015f61` | freetype.c | `TextIntUpdateSize()` return value ignored at 2 call sites; a size-request failure is not reported to the caller | `freetype.rs:5224` (`ftGdiGetGlyphOutline_impl`), `freetype.rs:5716` (`TextIntGetTextExtentPoint_impl`) | **CONFIRMED** |
| 2 | `4671b481a38` | freetype.c | FontLink sub-font glyphs rendered without requesting the sub-face's point size — wrong-sized fallback glyphs | `freetype.rs:5053-5104` (`FontLink_Chain_FindGlyph`) | **CONFIRMED** |
| 3 | `45a4f95b04f` | freetype.c | Reverts an entire "font look-up cache" subsystem upstream calls buggy (`#8641 introduced some bugs. We revert it.`) | `freetype.rs:83,394-403,1255-1330,6572-6690` (`FONT_LOOKUP_CACHE`, `FontLookUp_*`, live in `TextIntRealizeFont`) | **CONFIRMED, large/architectural** — see detail below |
| 4 | `5d219532335` | desktop.c | `DesktopWindowProc`'s `default:` case never calls `IntDefWindowProc()` for unhandled messages (left disabled since a 2015-era regression that "isn't relevant anymore") | `desktop.rs:1924-1927` | **CONFIRMED** |
| 5 | `9eb3bdef4c7` | desktop.c | `IntResolveDesktop` requests `WINSTA_ACCESS_ALL`/`DESKTOP_ALL_ACCESS` instead of `MAXIMUM_ALLOWED`, so a caller with a restricted token (job/sandbox) can't even open its own startup desktop | `desktop.rs:1247` and `:1429` (`IntResolveDesktop_impl`) | **CONFIRMED** |
| 6 | `529f3c618df` | sysparams.c | `SPI_GETLOWPOWERACTIVE` returns `gspv.iPwrOffTimeout` (copy-paste from the sibling case) instead of `gspv.bLowPwrActive` | `sysparams.rs:1310` (`SpiGetSet_SPI_GETLOWPOWERACTIVE`) | **CONFIRMED** |
| 7 | `7a505166b9f` | sysparams.c | `gpsi` dereferenced unconditionally to set `dtCaretBlink`, with no null check, in a routine that elsewhere in the same function already guards `gbSpiInitialized && gpsi` | `sysparams.rs:432` | **CONFIRMED** |
| 8 | `f17d904d2d9` | menu.c | `NtUserMenuItemFromPoint()` returns the loop index instead of `NO_SELECTED_ITEM` on a miss, because the loop pointer `mi` is never actually null on a miss — callers can index one past `Menu->rgItems` | `menu.rs:7833-7837` | **CONFIRMED** |
| 9 | `c65740eeb3a` | accelerator.c | `NtUserCreateAcceleratorTable`: `EntriesCount * sizeof(ACCEL)` has no overflow check before the pool allocation, so a huge `EntriesCount` wraps to a tiny allocation and the entry-copy loop (bounded only by `EntriesCount`) then writes past it | `accelerator.rs:302-306` | **CONFIRMED — memory corruption** |
| 10 | `feb9febaae3` | clipboard.c | `NtUserConvertMemHandle`: on a failed probe, `pMemObj` is set to null and then unconditionally passed to `UserDereferenceObject(pMemObj)` — null-pointer dereference | `clipboard.rs:1332-1338` | **CONFIRMED — BSOD** |
| 11 | `b38db9dd238` | painting.c | `IntGetNCUpdateRgn` computes the non-client update region's bounding box from the full non-client region instead of the actual dirty region, and never clips the result to `Window->hrgnUpdate` — causes full non-client repaint (title bar/borders) on every unrelated screen update | `painting.rs:468` (missing the extra `RGN_AND`) | **CONFIRMED** |
| 12 | `7f79e1c9483` | caret.c | `co_UserShowCaret`'s `EVENT_OBJECT_SHOW` notification passes `OBJID_CARET` as `idChild` instead of `CHILDID_SELF` (copy-paste from the `idObject` param); the other 4 caret win-events in the same function get it right | `caret.rs:407` | **CONFIRMED** |
| 13 | `b82e2884fca` | scrollbar.c | `co_IntGetScrollBarInfo`: (a) freshly-computed `rgstate` bits are overwritten by the stale cached copy because `RtlCopyMemory` runs *after* `rgstate` is set, not before; (b) the `SB_CTL`+`WS_DISABLED` test is inverted (`== 0` instead of `!= 0`), so a disabled scrollbar control is never reported `STATE_SYSTEM_UNAVAILABLE` and an enabled one always is | `scrollbar.rs:838-853` (order) and `:851` (inverted test) | **CONFIRMED — 2 bugs, both present** |

INCONCLUSIVE (real, upstream-confirmed bug fixes, present in our Rust by the same evidence
standard, but the change is large enough that I did not produce a line-level diff this pass):

| sha | file | why flagged, not diffed |
|---|---|---|
| `e0e659386641` | desktop.c | Full rewrite of desktop wallpaper painting (`IntPaintDesktop`) for Stretch/Fit/Fill modes — 279 lines added, restructures into `IntCalcWallpaperCoordinates`/`IntStretchWallpaper`/`IntPaintWallpaper`. Confirmed the *old*, buggy per-repaint stretch-and-blit-in-one-step is still what our Rust does (no such helper functions exist in `desktop.rs`), but a faithful port of the fix needs the new caching bitmap (`hSystemBM`) plumbed through `IntPaintDesktop`, which is a genuine multi-function feature, not a 1-line patch. Also: our default/smoke-test desktop has no wallpaper bitmap set (`gspv.hbmWallpaper == NULL`), so this path is currently dead for the smoke harness. |
| `33dab072e75` | freetype.c | Rewrites font-face charmap selection at load time (`select_charmap`/`pick_charmap`, ported from Wine) so a charmap is chosen once, correctly, when a face is first loaded, instead of `TextIntUpdateSize` re-deriving one ad hoc on every text operation. Our Rust `TextIntUpdateSize_impl` (freetype.rs:4896-4956) still has the **old** ad hoc charmap search embedded (confirmed: no `select_charmap`/`pick_charmap`/`IntSelectFaceCharmap` anywhere in `freetype.rs`). Real, but restructuring font load (`IntGdiLoadFontsFromMemory`) is out of scope for a quick patch. |
| `1968202840d` | winsta.c | Session-ID retrieval swapped from `NtCurrentPeb()->SessionId` to `PsGetCurrentProcessSessionId()`, plus a global handle for future unload support. Plausible real fix (PEB-based session ID is fragile outside the originating process context) but not checked against our Rust `UserCreateWinstaDirectory` this pass — single-session boot in the smoke harness makes it low-yield to verify first. |

---

## Detail + proposed diffs (CONFIRMED)

### 1. `88d06015f61` — ignored `TextIntUpdateSize()` failure (freetype.c)

Upstream:
```diff
     IntLockFreeType();
-    TextIntUpdateSize(TextObj, FontGDI, FALSE);
+    if (!TextIntUpdateSize(TextObj, FontGDI, FALSE))
+    {
+        IntUnLockFreeType();
+        ExFreePoolWithTag(potm, GDITAG_TEXT);
+        TEXTOBJ_UnlockText(TextObj);
+        EngSetLastError(ERROR_GEN_FAILURE);
+        return GDI_ERROR;
+    }
```
(same shape, minus the `potm`, in `TextIntGetTextExtentPoint`.)

Our Rust already has a `TextIntUpdateSize_impl` that *can* return `0` (`freetype.rs:4953-4956`,
via `IntRequestFontSize` failing), so the failure path is reachable, just not checked, at both
call sites:

```rust
// freetype.rs:5224, ftGdiGetGlyphOutline_impl
IntLockFreeType();
TextIntUpdateSize(dc, TextObj, FontGDI, 0);      // <-- return value dropped
IntMatrixFromMx(&mut mat, w32r_DC_pmxWorldToDevice(dc));
```
```rust
// freetype.rs:5716, TextIntGetTextExtentPoint_impl
IntLockFreeType();
TextIntUpdateSize(dc, TextObj, FontGDI, 0u8 as BOOL);   // <-- return value dropped
Cache.Hashed.matTransform = identityMat;
```

Proposed diff:
```diff
--- a/sdk/lib/rust/win32k-rust/src/freetype.rs
+++ b/sdk/lib/rust/win32k-rust/src/freetype.rs
@@ ftGdiGetGlyphOutline_impl
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
@@ TextIntGetTextExtentPoint_impl
     IntLockFreeType();
-    TextIntUpdateSize(dc, TextObj, FontGDI, 0u8 as BOOL);
+    if TextIntUpdateSize(dc, TextObj, FontGDI, 0u8 as BOOL) == 0 {
+        IntUnLockFreeType();
+        return 0;
+    }
     Cache.Hashed.matTransform = identityMat;
```
(Exact unlock/cleanup order should mirror what's already on each function's existing error
paths above these call sites — both functions already have earlier `return`s that follow this
shape.)

### 2. `4671b481a38` — FontLink sub-font glyph size never requested (freetype.c)

Upstream adds `IntNeedRequestFontSize()` + `IntRequestFontSizeEx(FT_Face, const LOGFONTW*)`
(the latter is `IntRequestFontSize`'s size-computation body, minus the `FontGDI`-cache check
and the `PDC`/`FontGDI` parameters — it works straight off `FT_Face` + `LOGFONTW`) and calls it
from `FontLink_Chain_FindGlyph` right before using the sub-face:

```diff
         pCache->Hashed.Face = *pFace = face;
+        if (IntNeedRequestFontSize(pChain, face, Entry))
+            IntRequestFontSizeEx(face, &pChain->LogFont);
         FT_Set_Transform(face, &pCache->Hashed.matTransform, NULL);
```

Our Rust `FontLink_Chain_FindGlyph` (`freetype.rs:5053-5104`) sets the face and calls
`FT_Set_Transform` but never requests a size for the sub-face — it renders at whatever
point-size FreeType last had active for that `FT_Face` (which could be a completely different
document's request, or FreeType's built-in default), so FontLink-fallback glyphs (e.g. CJK
fallback for a Latin-only font) come out the wrong size.

Proposed diff (mirrors upstream, adapted to the primitives our Rust already has —
`IntRequestFontSize` takes `PDC`/`PFONTGDI` and cannot be reused directly since `FONTLINK` has
no `PFONTGDI`; a `FT_Face`+`LOGFONTW` sibling has to be extracted, same as upstream did):

```diff
--- a/sdk/lib/rust/win32k-rust/src/freetype.rs
+++ b/sdk/lib/rust/win32k-rust/src/freetype.rs
@@
+unsafe fn IntNeedRequestFontSize(pChain: PFONTLINK_CHAIN, face: FT_Face, TargetEntry: PLIST_ENTRY) -> BOOL {
+    let Head: PLIST_ENTRY = core::ptr::addr_of_mut!((*pChain).FontLinkList);
+    let mut Entry: PLIST_ENTRY = (*Head).Flink;
+    while Entry != Head {
+        let pFontLink: PFONTLINK = CONTAINING_RECORD!(Entry, FONTLINK, ListEntry);
+        if FontLink_PrepareFontInfo(pFontLink) != 0 {
+            if (*(*pFontLink).SharedFace).Face == face || Entry == TargetEntry {
+                return 0;
+            }
+        }
+        Entry = (*Entry).Flink;
+    }
+    1
+}
+
+// Extracted from IntRequestFontSize's size-computation body (drops the
+// FontGDI-cache short-circuit and works off a bare FT_Face + LOGFONTW, same
+// as upstream's IntRequestFontSizeEx — see field manual entry for this fix).
+unsafe fn IntRequestFontSizeEx(face: FT_Face, plf: *const LOGFONTW) -> FT_Error {
+    // body: identical arithmetic to IntRequestFontSize (freetype.rs:4767+)
+    // from the lfHeight/lfWidth normalization down through FT_Request_Size,
+    // just parameterized on `face`/`plf` instead of `FontGDI`.
+    todo!("port IntRequestFontSize's body here, see freetype.rs:4767")
+}
+
 unsafe fn FontLink_Chain_FindGlyph(
@@
         *pFace = face;
         (*pCache).Hashed.Face = face;
+        if IntNeedRequestFontSize(pChain, face, Entry) != 0 {
+            IntRequestFontSizeEx(face, core::ptr::addr_of!((*pChain).LogFont));
+        }
         FT_Set_Transform(face, core::ptr::addr_of_mut!((*pCache).Hashed.matTransform), core::ptr::null_mut());
         return index;
```
(`IntRequestFontSizeEx`'s body is a mechanical extraction of the existing `IntRequestFontSize`
arithmetic in `freetype.rs:4767+` — left as `todo!()` above since duplicating ~120 lines of
already-reviewed arithmetic isn't useful to write twice in this doc; the point is the call site
and the missing size-request, which is the actual bug.)

### 3. `45a4f95b04f` — the whole "font look-up cache" is upstream-reverted-as-buggy, still live in our Rust

Upstream's own words: *"#8641 introduced some bugs. We revert it."* (`CORE-20538`). The revert
deletes `FONT_LOOKUP_CACHE`, `s_FontLookupCacheList`, every `FontLookUp_*` function, and reverts
`FontLink_PrepareFontInfo` and `TextIntRealizeFont` to their pre-#8641 shape (159 insertions,
268 deletions in the C).

Our Rust has the **pre-revert** (buggy) state, confirmed present and live:
- `freetype.rs:83` — `static mut s_FontLookupCacheList`
- `freetype.rs:394-403` — `_FONT_LOOKUP_CACHE` struct + `MAX_FONT_LOOKUP_CACHE`
- `freetype.rs:1255-1330` — `FontLookUp_Destroy`/`FontLookUp_Cleanup`/init
- `freetype.rs:6572-6690` — `FontLookUp_LookUp`/`FontLookUp_Add`, called from
  `TextIntRealizeFont` (the font-realization hot path hit on essentially every `SelectObject`
  of a font)

This is exactly the class of finding AGENTS.md calls out as the highest-value seam (a whole
subsystem that upstream considers buggy enough to delete outright, silently carried forward by
translation because the delete happened after our fork). I did not attempt a line-level revert
patch here — the C revert touches ~427 lines across cache lifecycle, `FontLink_PrepareFontInfo`,
and `TextIntRealizeFont`, and a faithful Rust "un-port" of a revert is its own multi-function
task. Flagging it CONFIRMED and out-of-budget for a diff in this pass; recommend a dedicated
follow-up that reads `45a4f95b04f`'s full diff against `freetype.rs`'s
`TextIntRealizeFont`/`FontLink_PrepareFontInfo` side by side.

### 4. `5d219532335` — `DesktopWindowProc` never calls the default window procedure (desktop.c)

Upstream:
```diff
         default:
             TRACE("DWP calling IDWP Msg %d\n",Msg);
-            //*lResult = IntDefWindowProc(Wnd, Msg, wParam, lParam, FALSE);
+            *lResult = IntDefWindowProc(Wnd, Msg, wParam, lParam, FALSE);
```
The call was disabled a decade ago for an unrelated regression that "isn't relevant anymore."

Our Rust (`desktop.rs:1924-1927`):
```rust
_ => {
    TRACE!(b"DWP calling IDWP Msg %d\n\0".as_ptr() as *const i8, Msg);
}
```
matches the disabled pre-fix state exactly — every message `DesktopWindowProc` doesn't
explicitly handle (WM_NCHITTEST, WM_NCPAINT, WM_GETMINMAXINFO, WM_QUERYENDSESSION, etc. all
fall through here) gets no default processing at all. Notably, the sibling function in the
same file, `UserMessageWindowProc` (`desktop.rs:1946-1951`), already calls `IntDefWindowProc`
in its own default arm — the correct idiom exists one function away. There's also a pre-existing
`[DESKWNDPROC]` diagnostic block a few lines above this match arm from an earlier "arm64
black-desktop hunt" — this function is a known area of interest for boot rendering.

Proposed diff:
```diff
--- a/sdk/lib/rust/win32k-rust/src/desktop.rs
+++ b/sdk/lib/rust/win32k-rust/src/desktop.rs
@@ DesktopWindowProc_impl
         _ => {
-            TRACE!(b"DWP calling IDWP Msg %d\n\0".as_ptr() as *const i8, Msg);
+            TRACE!(b"DWP calling IDWP Msg %d\n\0".as_ptr() as *const i8, Msg);
+            *lResult = IntDefWindowProc(Wnd, Msg, wParam, lParam, 0);
         }
     }
     1
```

### 5. `9eb3bdef4c7` — `IntResolveDesktop` requests more access than it needs (desktop.c)

Upstream:
```diff
-                                WINSTA_ACCESS_ALL,
+                                MAXIMUM_ALLOWED,
...
-                                    DESKTOP_ALL_ACCESS,
+                                    MAXIMUM_ALLOWED,
```
Our Rust `IntResolveDesktop_impl` (`desktop.rs:1247` and `:1429`) still requests
`WINSTA_ACCESS_ALL`/`DESKTOP_ALL_ACCESS` verbatim. A caller running with a restricted token
(e.g. a sandboxed/job-restricted process) fails to even open its own startup window
station/desktop, because `ObOpenObjectByName` denies the all-access request outright instead of
granting whatever the token is actually entitled to.

Proposed diff:
```diff
--- a/sdk/lib/rust/win32k-rust/src/desktop.rs
+++ b/sdk/lib/rust/win32k-rust/src/desktop.rs
@@ IntResolveDesktop_impl
         Status = ObOpenObjectByName(
             ObjectAttributes as _,
             ex_windowstation_object_type() as _,
             1i32 as i8,
             core::ptr::null_mut(),
-            WINSTA_ACCESS_ALL,
+            MAXIMUM_ALLOWED,
             core::ptr::null_mut(),
             &mut hTempWinSta as *mut HWINSTA as PHANDLE,
         );
@@
             Status = ObOpenObjectByName(
                 ObjectAttributes as _,
                 ex_desktop_object_type() as _,
                 1i32 as i8,
                 core::ptr::null_mut(),
-                DESKTOP_ALL_ACCESS,
+                MAXIMUM_ALLOWED,
                 core::ptr::null_mut(),
                 &mut hDesktop as *mut HDESK as PHANDLE,
             );
```

### 6. `529f3c618df` — `SPI_GETLOWPOWERACTIVE` returns the wrong field (sysparams.c)

```diff
         case SPI_GETLOWPOWERACTIVE:
-            return SpiGetInt(pvParam, &gspv.iPwrOffTimeout, fl);
+            return SpiGetInt(pvParam, &gspv.bLowPwrActive, fl);
```

Our Rust (`sysparams.rs:1309-1311`):
```rust
unsafe fn SpiGetSet_SPI_GETLOWPOWERACTIVE(pvParam: PVOID, fl: FLONG) -> UINT_PTR {
    SpiGetInt(pvParam, core::ptr::addr_of_mut!(gspv.iPwrOffTimeout) as PVOID, fl)
}
```
Exact match to the pre-fix bug — `SystemParametersInfo(SPI_GETLOWPOWERACTIVE, ...)` returns
the power-off timeout value instead of the low-power-active boolean.

Proposed diff:
```diff
--- a/sdk/lib/rust/win32k-rust/src/sysparams.rs
+++ b/sdk/lib/rust/win32k-rust/src/sysparams.rs
@@
 unsafe fn SpiGetSet_SPI_GETLOWPOWERACTIVE(pvParam: PVOID, fl: FLONG) -> UINT_PTR {
-    SpiGetInt(pvParam, core::ptr::addr_of_mut!(gspv.iPwrOffTimeout) as PVOID, fl)
+    SpiGetInt(pvParam, core::ptr::addr_of_mut!(gspv.bLowPwrActive) as PVOID, fl)
 }
```
(Verify `gspv.bLowPwrActive` is the field name in our `_SYSTEM_PARAMETERS`-equivalent struct —
it's referenced this way at the `SPI_SETLOWPOWERACTIVE` sibling case, so the name should already
exist in scope.)

### 7. `7a505166b9f` — null `gpsi` dereferenced setting caret blink time (sysparams.c)

```diff
-    gpsi->dtCaretBlink = SpiLoadInt(KEY_DESKTOP, VAL_CARETRATE, 530);
+    if (gpsi)
+        gpsi->dtCaretBlink = SpiLoadInt(KEY_DESKTOP, VAL_CARETRATE, 530);
```

Our Rust (`sysparams.rs:432`):
```rust
(*pusi(gpsi)).dtCaretBlink = SpiLoadInt(KEY_DESKTOP.as_ptr(), VAL_CARETRATE.as_ptr(), 530) as _;
```
No null check — `pusi()` just offsets the pointer, so a null `gpsi` here dereferences a
near-null address. The same function later guards a different block with
`if gbSpiInitialized && !gpsi.is_null()` (sysparams.rs, further down), confirming `gpsi` is
known-nullable at this point in the function's own lifecycle (this routine runs during early
per-user parameter initialization, before `gpsi` is necessarily set up).

Proposed diff:
```diff
--- a/sdk/lib/rust/win32k-rust/src/sysparams.rs
+++ b/sdk/lib/rust/win32k-rust/src/sysparams.rs
@@
-    (*pusi(gpsi)).dtCaretBlink = SpiLoadInt(KEY_DESKTOP.as_ptr(), VAL_CARETRATE.as_ptr(), 530) as _;
+    if !gpsi.is_null() {
+        (*pusi(gpsi)).dtCaretBlink = SpiLoadInt(KEY_DESKTOP.as_ptr(), VAL_CARETRATE.as_ptr(), 530) as _;
+    }
```

### 8. `f17d904d2d9` — `NtUserMenuItemFromPoint` returns an out-of-bounds index on a miss (menu.c)

```diff
-   Ret = (mi ? i : NO_SELECTED_ITEM);
+   Ret = (i < Menu->cItems ? i : NO_SELECTED_ITEM);
```
The bug: `mi` is advanced every loop iteration and is **never actually set to null** on a miss
(the loop just runs out because `i == cItems`), so the ternary always took the `i` branch.

Our Rust (`menu.rs:7815-7837`):
```rust
mi = (*Menu).rgItems;
i = 0;
while i < (*Menu).cItems {
    ...
    if RECTL_bPointInRect(&Rect, X as INT, Y as INT) != 0 {
        break;
    }
    i += 1;
    mi = mi.add(1);
}

Ret = if !mi.is_null() {
    i as ::core::ffi::c_int
} else {
    NO_SELECTED_ITEM as ::core::ffi::c_int
};
```
Same bug, faithfully carried over: `mi` is never null here either (`mi.add(1)` never produces a
null pointer), so `Ret` is always `i` — `cItems` on a miss — letting a caller index
`Menu->rgItems[cItems]`, one past the array.

Proposed diff:
```diff
--- a/sdk/lib/rust/win32k-rust/src/menu.rs
+++ b/sdk/lib/rust/win32k-rust/src/menu.rs
@@ NtUserMenuItemFromPoint
-        Ret = if !mi.is_null() {
+        Ret = if i < (*Menu).cItems {
             i as ::core::ffi::c_int
         } else {
             NO_SELECTED_ITEM as ::core::ffi::c_int
         };
```

### 9. `c65740eeb3a` — integer overflow → heap corruption in `NtUserCreateAcceleratorTable` (accelerator.c)

```diff
-    if (!Entries || EntriesCount <= 0)
+    if (Entries == NULL || EntriesCount == 0 ||
+        EntriesCount > MAXULONG_PTR / sizeof(*Entries))
     {
-        SetLastNtError(STATUS_INVALID_PARAMETER);
         goto Exit; // Return NULL
     }
```

Our Rust (`accelerator.rs:284-306`):
```rust
if Entries.is_null() || EntriesCount == 0 {
    SetLastNtError(STATUS_INVALID_PARAMETER);
    break 'exit;
}
...
(*Accel).Table = ExAllocatePoolWithTag(
    PagedPool,
    (EntriesCount as usize * core::mem::size_of::<ACCEL>()) as _,
    USERTAG_ACCEL,
) as *mut ACCEL;
```
No overflow guard on `EntriesCount * sizeof(ACCEL)` before the allocation. On a 32-bit target,
`EntriesCount: ULONG` cast to a 32-bit `usize` times `sizeof(ACCEL)` can wrap, producing a small
allocation; the subsequent SEH-protected entry-copy loop is bounded by `EntriesCount` (the
*un-wrapped* count), not by the allocation size, so it writes past the undersized buffer —
this is the exact heap-corruption shape upstream fixed.

Proposed diff:
```diff
--- a/sdk/lib/rust/win32k-rust/src/accelerator.rs
+++ b/sdk/lib/rust/win32k-rust/src/accelerator.rs
@@ NtUserCreateAcceleratorTable
-        if Entries.is_null() || EntriesCount == 0 {
+        if Entries.is_null() || EntriesCount == 0
+            || EntriesCount as usize > usize::MAX / core::mem::size_of::<ACCEL>() {
             SetLastNtError(STATUS_INVALID_PARAMETER);
             break 'exit;
         }
```
(Upstream drops the `SetLastNtError` call on this path too — but that's `STATUS_INVALID_PARAMETER`
being reported either way via a different mechanism upstream; keeping `SetLastNtError` here is
harmless and arguably better than upstream, so I left it in the proposed diff rather than
matching upstream byte-for-byte on that detail.)

### 10. `feb9febaae3` — null-pointer BSOD in `NtUserConvertMemHandle` (clipboard.c)

```diff
     _SEH2_EXCEPT(EXCEPTION_EXECUTE_HANDLER)
     {
-        pMemObj = NULL;
+        Status = _SEH2_GetExceptionCode();
     }
     _SEH2_END;

     UserDereferenceObject(pMemObj);

-    if (!pMemObj)
+    if (!NT_SUCCESS(Status))
```

Our Rust (`clipboard.rs:1326-1345`):
```rust
Status = SafeProbeForRead(pData, cbData as _, 1);
if !NT_SUCCESS(Status) {
    pMemObj = core::ptr::null_mut();
} else {
    memcpy(...);
}

UserDereferenceObject(pMemObj as _);   // <-- called with pMemObj == null on the failure path

if pMemObj.is_null() {
    UserDeleteObject(hMem, TYPE_CLIPDATA);
    hMem = core::ptr::null_mut();
}
```
Same shape as the pre-fix C, translated bug-for-bug (via `SafeProbeForRead`'s status rather
than SEH, but the null-then-dereference sequence is identical): on a failed probe,
`UserDereferenceObject` is handed a null pointer.

Proposed diff:
```diff
--- a/sdk/lib/rust/win32k-rust/src/clipboard.rs
+++ b/sdk/lib/rust/win32k-rust/src/clipboard.rs
@@ NtUserConvertMemHandle
-        Status = SafeProbeForRead(pData, cbData as _, 1);
-        if !NT_SUCCESS(Status) {
-            pMemObj = core::ptr::null_mut();
-        } else {
+        Status = SafeProbeForRead(pData, cbData as _, 1);
+        if NT_SUCCESS(Status) {
             memcpy(
                 core::ptr::addr_of_mut!((*pMemObj).Data) as *mut _,
                 pData,
                 cbData as _,
             );
         }

         /* Release the extra reference (UserCreateObject added 2 references) */
         UserDereferenceObject(pMemObj as _);

         /* If we failed to copy data, remove handle */
-        if pMemObj.is_null() {
+        if !NT_SUCCESS(Status) {
             UserDeleteObject(hMem, TYPE_CLIPDATA);
             hMem = core::ptr::null_mut();
         }
```
(`Status` needs to be declared `mut` and initialized to `STATUS_SUCCESS` at function top since
the success path through the `if pMemObj.is_null() { break 'cleanup; }` early-out no longer sets
it — check that arm still reads correctly; a `STATUS_SUCCESS` default handles it.)

### 11. `b38db9dd238` — non-client update region not clipped to the dirty region (painting.c)

```diff
-      NcType = IntGdiGetRgnBox(hRgnNonClient, &update);
+      NcType = IntGdiGetRgnBox(Window->hrgnUpdate, &update);

       RgnType = NtGdiCombineRgn(hRgnNonClient, hRgnNonClient, hRgnWindow, RGN_DIFF);
+      if ((RgnType != ERROR) && (RgnType != NULLREGION))
+         RgnType = NtGdiCombineRgn(hRgnNonClient, hRgnNonClient, Window->hrgnUpdate, RGN_AND);
```

Our Rust (`painting.rs:468-471`):
```rust
NcType = IntGdiGetRgnBox(hRgnNonClient, &mut update as *mut _ as _) as u32;

RgnType = NtGdiCombineRgn(hRgnNonClient, hRgnNonClient, hRgnWindow, RGN_DIFF as i32) as u32;
```
Matches the pre-fix code exactly: the bounding box is taken from the whole non-client region
(not the actual dirty/update region) and the DIFF result is never further clipped (AND'd) with
`Window->hrgnUpdate`. Net effect: any repaint anywhere on screen (e.g. dragging another window
across this one) triggers a full non-client repaint of every affected window's title bar and
borders, instead of only the parts actually inside the dirty region.

Proposed diff:
```diff
--- a/sdk/lib/rust/win32k-rust/src/painting.rs
+++ b/sdk/lib/rust/win32k-rust/src/painting.rs
@@ IntGetNCUpdateRgn
-        NcType = IntGdiGetRgnBox(hRgnNonClient, &mut update as *mut _ as _) as u32;
+        NcType = IntGdiGetRgnBox((*Window).hrgnUpdate, &mut update as *mut _ as _) as u32;

         RgnType = NtGdiCombineRgn(hRgnNonClient, hRgnNonClient, hRgnWindow, RGN_DIFF as i32) as u32;
+        if RgnType != ERROR && RgnType != NULLREGION {
+            RgnType = NtGdiCombineRgn(hRgnNonClient, hRgnNonClient, (*Window).hrgnUpdate, RGN_AND as i32) as u32;
+        }
```

### 12. `7f79e1c9483` — wrong `idChild` in caret show win-event (caret.c)

```diff
-         IntNotifyWinEvent(EVENT_OBJECT_SHOW, pWnd, OBJID_CARET, OBJID_CARET, 0);
+         IntNotifyWinEvent(EVENT_OBJECT_SHOW, pWnd, OBJID_CARET, CHILDID_SELF, 0);
```

Our Rust (`caret.rs:407`):
```rust
IntNotifyWinEvent(EVENT_OBJECT_SHOW, pWnd, OBJID_CARET as LONG, OBJID_CARET as LONG, 0);
```
Exact copy-paste bug carried over — 4th argument is `idChild`, and every other caret win-event
in the same file (hide, location-change, etc.) correctly passes `CHILDID_SELF`; only the show
event passes `OBJID_CARET` again. Accessibility clients (screen readers) listening for
`EVENT_OBJECT_SHOW` on the caret get the wrong child id.

Proposed diff:
```diff
--- a/sdk/lib/rust/win32k-rust/src/caret.rs
+++ b/sdk/lib/rust/win32k-rust/src/caret.rs
@@ co_UserShowCaret
-            IntNotifyWinEvent(EVENT_OBJECT_SHOW, pWnd, OBJID_CARET as LONG, OBJID_CARET as LONG, 0);
+            IntNotifyWinEvent(EVENT_OBJECT_SHOW, pWnd, OBJID_CARET as LONG, CHILDID_SELF as LONG, 0);
```

### 13. `b82e2884fca` — stale scrollbar state + inverted `WS_DISABLED` test (scrollbar.c)

```diff
+   RtlCopyMemory(psbi, sbi, sizeof(*psbi));
+
     psbi->rgstate[0] = 0;
     ...
-    if (Bar == SB_CTL && !(Window->style & WS_DISABLED))
+    if (Bar == SB_CTL && (Window->style & WS_DISABLED))
         psbi->rgstate[0] |= STATE_SYSTEM_UNAVAILABLE;

-   RtlCopyMemory(psbi, sbi, sizeof(SCROLLBARINFO));
-
    return TRUE;
```

Our Rust (`scrollbar.rs:838-861`) has the copy at the **end** (after `rgstate` is computed on
`psbi`), so the freshly-computed bits are immediately clobbered by the stale `sbi` snapshot —
and the `WS_DISABLED` test is inverted (`== 0`, i.e. "not disabled", instead of `!= 0`):
```rust
(*psbi).rgstate[0] = 0;
... // rgstate computed into psbi
if Bar == SB_CTL as INT && ((*Window).style & WS_DISABLED as ULONG) == 0 {
    (*psbi).rgstate[0] |= STATE_SYSTEM_UNAVAILABLE as u32;
}

RtlCopyMemory(psbi as *mut _, sbi as *const _, core::mem::size_of::<SCROLLBARINFO>());
1
```
Both bugs present: callers of `GetScrollBarInfo` always get a stale cached `rgstate`
(everything computed above is thrown away), and when they do see rgstate bits, an *enabled*
`SB_CTL` gets `STATE_SYSTEM_UNAVAILABLE` while a *disabled* one doesn't.

Proposed diff:
```diff
--- a/sdk/lib/rust/win32k-rust/src/scrollbar.rs
+++ b/sdk/lib/rust/win32k-rust/src/scrollbar.rs
@@ co_IntGetScrollBarInfo_impl
     IntGetScrollBarRect(Window, Bar, core::ptr::addr_of_mut!((*sbi).rcScrollBar));
     IntCalculateThumb(Window, Bar, sbi, pSBData);

+    RtlCopyMemory(
+        psbi as *mut core::ffi::c_void,
+        sbi as *const core::ffi::c_void,
+        core::mem::size_of::<SCROLLBARINFO>() as usize,
+    );
+
     (*psbi).rgstate[0] = 0;
@@
-    if Bar == SB_CTL as INT && ((*Window).style & WS_DISABLED as ULONG) == 0 {
+    if Bar == SB_CTL as INT && ((*Window).style & WS_DISABLED as ULONG) != 0 {
         (*psbi).rgstate[0] |= STATE_SYSTEM_UNAVAILABLE as u32;
     }

-    RtlCopyMemory(
-        psbi as *mut core::ffi::c_void,
-        sbi as *const core::ffi::c_void,
-        core::mem::size_of::<SCROLLBARINFO>() as usize,
-    );
-
     1
```

---

## Not reached — resume point for the next pass

Files from the 49 not examined at all, or only subject-line-triaged (a `git show` of the sha
would be step 1):

**Not opened at all:** `win32ss/gdi/eng/device.c`, `win32ss/gdi/eng/engmisc.c`,
`win32ss/gdi/eng/pdevobj.c`, `win32ss/gdi/eng/stubs.c`, `win32ss/gdi/ntgdi/gdiobj.c` (note:
`640e39c72e8` is itself a *revert* — "Revert 'Fix 3D-text screensaver not moving in preview
mode'" — worth checking whether the original fix, or its revert, is what our Rust matches),
`win32ss/user/ntuser/class.c`, `win32ss/user/ntuser/hotkey.c`, `win32ss/user/ntuser/kbdlayout.c`,
`win32ss/user/ntuser/display.c`, `win32ss/user/ntuser/defwnd.c`, `win32ss/user/ntuser/main.c`,
`win32ss/user/ntuser/misc.c`, `win32ss/user/ntuser/ntuser.c`,
`win32ss/user/ntuser/simplecall.c`, `win32ss/user/ntuser/object.c`,
`win32ss/user/ntuser/ntstubs.c`, `win32ss/user/ntuser/nonclient.c` (`cf470941031` — "Preserve
WM_QUIT in move/size modal loop" — reads like a real, self-contained fix, good next candidate),
`win32ss/user/ntuser/focus.c`, `win32ss/user/ntuser/layered.c`, `win32ss/user/ntuser/callback.c`.

**Single-commit files with a subject that reads as a real, self-contained fix — spotted but not
checked against the Rust twin (highest-value queue for the next pass, roughly in priority
order):**
- `win32ss/user/ntuser/accelerator.c`-adjacent visual bugs: `bitblt.c` `7f459b49f0a`
  "Fix brush origin for pattern painting"; `win32ss/gdi/eng/transblt.c` `a94352ba486`
  "Fix copy direction in overlapping EngTransparentBlt"; `win32ss/gdi/eng/mouse.c`
  `984657bdf02` "Fix Y coordinate of the software pointer exclude rectangle" (cursor rendering
  — plausibly smoke-relevant).
- `win32ss/user/ntuser/mouse.c` `35114725860` "Fix XBUTTON1 reported as XBUTTON2 on
  WM_XBUTTONUP" and `2dddf480710` (same area, X-button flush checks).
- `win32ss/gdi/ntgdi/polyfill.c` `d3ed29ef065` "Use newx1 as the lower bound in polyfill
  line-touch test".
- `win32ss/gdi/ntgdi/path.c` + `win32ss/gdi/ntgdi/fillshap.c` (shared commit) `049085f1060`
  "Fix pptviewer '97 and calls to PolylineTo and PolyBezierTo".
- `win32ss/gdi/ntgdi/palette.c` `6d8a5fa0bbe` "Allow calling NtGdiCreateHalftonePalette with a
  NULL hDC" (a rejected-valid-input bug, opposite shape from a missing-null-check bug — worth
  confirming which way our Rust gets it wrong).
- `win32ss/gdi/eng/surface.c` `5e76d075f10` "Relax surface parameter validation" (also a
  rejected-valid-input shape).
- `win32ss/user/ntuser/winsta.c` `1968202840d` (flagged INCONCLUSIVE above, not yet diffed).

**Read but classified as refactor/feature/version-bump, not pursued further:**
`freetype.c`'s `8e5bc1b4e56`, `75e5904c057`, `75e91de9a00`, `df7c1fbc6b4`, `713888355b8`,
`0566a0f6998`, `06ae7ec2114`, `f7ef944dcac`, `0c4091cd045`, `e6328fbf1ea` (already known, per
the task brief, to be a pure rewrite); `win32ss/user/ntuser/ime.c`'s and `input.c`'s commits
(feature adds / lock-type changes — `a45e91cb3db`/`6e95a132bd8` "use shared locks" are worth a
second look for an actual race they fix, not just a lock-type swap, but I didn't chase it this
pass); `winsta.c`'s `7e08891a9f3`/`2b8f2020d70`/`2eb28656dab` (shared with desktop.c, feature +
formatting + init-order — `2eb28656dab` "Improve initialization of window station and desktop
objects" is worth reading in full next time, it touches both files and "improve initialization"
can hide a real ordering bug).
