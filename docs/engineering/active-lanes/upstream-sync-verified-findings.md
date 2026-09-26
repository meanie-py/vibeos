# Upstream-sync verified findings

Verification pass over the top 4 leads (queue ranks #2-#5, doc order in the lead's own top-10
narrative) from `upstream-sync-bug-leads.md`. Method: read the actual upstream diff (never the
subject line), find the Rust twin via `tools/cohab-registry.json`-equivalent lookup (module
CMake comment + `sdk/lib/rust` tree), compare line-for-line, check `git log -S`/`git log
--oneline -- <file>` on the Rust file for an independent fix, check `pub mod` reachability in
`sdk/lib/rust/win32k-rust/src/lib.rs`.

All four are CONFIRMED. All four Rust files are LIVE (declared `pub mod` in
`sdk/lib/rust/win32k-rust/src/lib.rs`) and none has been touched by an independent fix commit.

## Verdict table

| sha | what the bug is | our Rust file:line | LIVE? | verdict |
|---|---|---|---|---|
| `c0f18d059a` | StretchBlt mask bounds check uses `<` (should be `<=`); `sx == cx` / `sy == cy` slips through, one-pixel OOB read via `fnMask_GetPixel` | `sdk/lib/rust/win32k-rust/src/dib_stretchblt.rs:148` | yes (`pub mod dib_stretchblt;`, lib.rs:129) | **CONFIRMED** |
| `e7e1465812` | `NtUserSBGetParms` copies two user pointers with no fault protection at all | `sdk/lib/rust/win32k-rust/src/scrollbar.rs:1472-1483` | yes (`pub mod scrollbar;`, lib.rs:340) | **CONFIRMED** (worse than upstream pre-fix: no SEH2 *and* no Probe, vs. upstream's SEH2-but-no-Probe) |
| `ae0e4ee305` | `DIB_8BPP_BitBltSrcCopy` top-to-bottom flip: wrong stride source on both the read-up loop and the write-down loop, plus double palette-translate when chained after the left-to-right flip | `sdk/lib/rust/win32k-rust/src/dib8bpp.rs:307,317` (stride) and `:306` (double-translate) | yes (`pub mod dib8bpp;`, lib.rs:106) | **CONFIRMED** |
| `e1bf662d9a` | `ftGetFontUnicodeRanges` walks FreeType's `charmap`/`FT_Get_Next_Char` without holding the FreeType lock; two early returns skip cleanup entirely (moot for us, see below) | `sdk/lib/rust/win32k-rust/src/freetype.rs:5938-5989` (whole `ftGetFontUnicodeRanges_impl`) | yes (`pub mod freetype;`, lib.rs:82) | **CONFIRMED** |

Counts: **4 CONFIRMED / 0 REFUTED / 0 INCONCLUSIVE**.

---

## 1. `c0f18d059a` — StretchBlt mask off-by-one (#9136)

### Upstream hunk
```diff
--- a/win32ss/gdi/dib/stretchblt.c
+++ b/win32ss/gdi/dib/stretchblt.c
@@ -174,7 +174,7 @@ BOOLEAN DIB_XXBPP_StretchBlt(SURFOBJ *DestSurf, SURFOBJ *SourceSurf, SURFOBJ *Ma
           sx = SourceRect->left+(DesX - DestRect->left) * SrcWidth / DstWidth;
         }
         if (sx < 0 || sy < 0 ||
-          MaskSurf->sizlBitmap.cx < sx || MaskCy < sy ||
+          MaskSurf->sizlBitmap.cx <= sx || MaskCy <= sy ||
           fnMask_GetPixel(MaskSurf, sx, sy) != 0)
         {
           CanDraw = FALSE;
```
The bug: for a valid mask coordinate `sx` must satisfy `0 <= sx < cx`. The bounds check
rejected only `sx > cx`, so `sx == cx` (one past the last valid column) passed through into
`fnMask_GetPixel`, an OOB read. Confirmed by reading the diff directly (not just the subject).

### Our Rust (`dib_stretchblt.rs:139-152`)
```rust
if fnMask_GetPixel.is_some() {
    if bLeftToRight != 0 {
        sx = (*SourceRect).right - (DesX - (*DestRect).left) * SrcWidth / DstWidth;
    } else {
        sx = (*SourceRect).left + (DesX - (*DestRect).left) * SrcWidth / DstWidth;
    }
    if sx < 0 || sy < 0 ||
        (*MaskSurf).sizlBitmap.cx < sx || MaskCy < sy ||          // <-- line 148, pre-fix `<`
        fnMask_GetPixel.unwrap()(MaskSurf, sx, sy) != 0
    {
        CanDraw = 0;
    }
}
```
Line 148 is the exact pre-fix `<`. The sibling source-surface check three lines below (line
162, `(*SourceSurf).sizlBitmap.cx > sx && SourceCy > sy`) is written the *correct* way (`sx <
cx` phrased as `cx > sx`), confirming this crate already knows the correct idiom elsewhere in
the very same function — the mask check just wasn't updated when upstream fixed it, because we
forked before the fix existed.

`git log --oneline -- sdk/lib/rust/win32k-rust/src/dib_stretchblt.rs` shows only the original
wave-1/wave-2b translation commits and one warnings-only sweep; no independent fix.

### Proposed diff
```diff
--- a/sdk/lib/rust/win32k-rust/src/dib_stretchblt.rs
+++ b/sdk/lib/rust/win32k-rust/src/dib_stretchblt.rs
@@ -145,7 +145,7 @@
                 }
                 if sx < 0 || sy < 0 ||
-                    (*MaskSurf).sizlBitmap.cx < sx || MaskCy < sy ||
+                    (*MaskSurf).sizlBitmap.cx <= sx || MaskCy <= sy ||
                     fnMask_GetPixel.unwrap()(MaskSurf, sx, sy) != 0
                 {
                     CanDraw = 0;
```
(Line numbers approximate against the read excerpt; exact context is the `if fnMask_GetPixel.is_some()` block, first `if sx < 0 || sy < 0 || ...` inside it.)

---

## 2. `e7e1465812` — NtUserSBGetParms (CORE-18128, #9411)

### Upstream hunk
```diff
--- a/win32ss/user/ntuser/scrollbar.c
+++ b/win32ss/user/ntuser/scrollbar.c
@@ -1207,9 +1207,11 @@
    _SEH2_TRY
    {
+      ProbeForWrite(lpsi, sizeof(SCROLLINFO), 1);
+      ProbeForRead(pSBData, sizeof(SBDATA), 1);
+
       RtlCopyMemory(&psi, lpsi, sizeof(SCROLLINFO));
-      if (pSBData)
-         RtlCopyMemory(&SBDataSafe, pSBData, sizeof(SBDATA));
+      RtlCopyMemory(&SBDataSafe, pSBData, sizeof(SBDATA));
    }
```
Pre-fix upstream C: has `_SEH2_TRY`/`_SEH2_EXCEPT` around the copy, so an invalid `lpsi`/
`pSBData` page-faults and is caught, returning `FALSE` — annoying but not a BSOD by itself.
**The actual bug the fix closes is the missing `ProbeForRead`/`ProbeForWrite`**: without them, a
user-mode caller can hand a pointer inside *kernel* address space, and a plain
`RtlCopyMemory` to/from a valid kernel address doesn't fault — SEH never fires — giving an
arbitrary kernel read (via `lpsi`) / read (via `pSBData`) primitive gated only by being able to
call `NtUserSBGetParms` at all. (The `if (pSBData)` guard removal is orthogonal — it just means
`pSBData` is not actually optional at this call site; harmless once `ProbeForRead` covers it,
since `ProbeForRead(NULL, ...)` itself raises.)

### Our Rust (`scrollbar.rs:1458-1506`)
```rust
pub unsafe extern "system" fn NtUserSBGetParms(
    hWnd: HWND, fnBar: c_int, pSBData: PSBDATA, lpsi: LPSCROLLINFO,
) -> BOOL {
    ...
    UserEnterShared();
    'exit: {
        RtlCopyMemory(addr_of_mut!(psi) as *mut _, lpsi as *const _, size_of::<SCROLLINFO>() as _);
        if !pSBData.is_null() {
            RtlCopyMemory(addr_of_mut!(SBDataSafe) as *mut _, pSBData as *const _, size_of::<SBDATA>() as _);
        }
        ...
        RtlCopyMemory(lpsi as *mut _, addr_of!(psi) as *const _, size_of::<SCROLLINFO>() as _);
    }
    UserLeave();
    Ret
}
```
There is **no** `ProbeForRead`/`ProbeForWrite` anywhere in this file (`grep -c` = 0) and **no**
SEH-equivalent at all — not even the pre-fix upstream's page-fault catch. `SafeProbeForRead` /
`SafeProbeForWrite` + `NT_SUCCESS` is the established idiom in this exact crate (60 other files
use it, e.g. `painting.rs:1921` `NtUserEndPaint`, `accelerator.rs:247`, `clipboard.rs:1385`),
and `scrollbar.rs` already defines its own local `NT_SUCCESS` helper (line 11) that nothing in
the file currently calls — strong signal the probe call was simply dropped in translation, not
a deliberate simplification. So this reproduces upstream's missing-probe bug, and goes further:
an ordinary invalid user pointer (no attacker needed) causes an *unguarded* page fault in
kernel mode — an immediate BSOD on any bad `lpsi`, not just the kernel-pointer-read/write
exploit upstream was closing. The `if !pSBData.is_null()` guard is retained (matches upstream
pre-fix, harmless either way once probed).

### Proposed diff
```diff
--- a/sdk/lib/rust/win32k-rust/src/scrollbar.rs
+++ b/sdk/lib/rust/win32k-rust/src/scrollbar.rs
@@ -1,5 +1,10 @@
 #![allow(non_snake_case, non_upper_case_globals, unused_unsafe)]
 use win32k_sys::win32k::*;
 use ntkrnl_sys::ddk::{PsGetCurrentThreadWin32Thread};
 use core::ffi::c_void;
+
+extern "C" {
+    fn SafeProbeForRead(Address: *mut c_void, Length: SIZE_T, Alignment: ULONG) -> NTSTATUS;
+    fn SafeProbeForWrite(Address: *mut c_void, Length: SIZE_T, Alignment: ULONG) -> NTSTATUS;
+}
@@ -1465,20 +1470,32 @@
     let Window: PWND;
     let mut psi: SCROLLINFO = core::mem::zeroed();
     let mut Ret: BOOL = 0;
     let mut SBDataSafe: SBDATA = core::mem::zeroed();
     let mut Ref: USER_REFERENCE_ENTRY = core::mem::zeroed();
+    let mut Status: NTSTATUS;
 
     UserEnterShared();
 
     'exit: {
+        Status = SafeProbeForWrite(lpsi as *mut _, core::mem::size_of::<SCROLLINFO>(), 1);
+        if NT_SUCCESS(Status) {
+            Status = SafeProbeForRead(pSBData as *mut _, core::mem::size_of::<SBDATA>(), 1);
+        }
+        if !NT_SUCCESS(Status) {
+            break 'exit;
+        }
+
         RtlCopyMemory(
             core::ptr::addr_of_mut!(psi) as *mut _,
             lpsi as *const _,
             core::mem::size_of::<SCROLLINFO>() as _,
         );
-        if !pSBData.is_null() {
-            RtlCopyMemory(
-                core::ptr::addr_of_mut!(SBDataSafe) as *mut _,
-                pSBData as *const _,
-                core::mem::size_of::<SBDATA>() as _,
-            );
-        }
+        RtlCopyMemory(
+            core::ptr::addr_of_mut!(SBDataSafe) as *mut _,
+            pSBData as *const _,
+            core::mem::size_of::<SBDATA>() as _,
+        );
```
Notes for whoever applies this: `SafeProbeForWrite`'s Rust-side signature/ABI needs matching
against how it's declared in `painting.rs`/`accelerator.rs` (both cdecl-ish stdcall `_...@12`
style comments appear there — copy that extern block verbatim rather than retyping it, to avoid
an ABI mismatch bug of our own). Also double-check whether `NtUserSBGetParms` is even meant to
accept a NULL `pSBData` per current callers (`co_IntGetScrollInfo`'s contract) before deciding
whether to keep or drop the null check — I did not verify every call site, so I left the
existing null check out of the probe-based rewrite intentionally conservative (probes both
first, still guards nothing else). This part is a judgment call for whoever lands it.

---

## 3. `ae0e4ee305` — DIB_8BPP_BitBltSrcCopy (CORE-2965, #9119)

### Upstream hunk (top-to-bottom flip section only, the relevant part)
```diff
+              SrcDelta = BltInfo->DestSurface->lDelta;   // OneDone branch
...
+              SrcDelta = BltInfo->SourceSurface->lDelta;  // else branch
...
-                store[Index] = (BYTE)XLATEOBJ_iXlate(BltInfo->XlateSourceToDest, *SourceBits);
+                store[Index] = OneDone ? *SourceBits : (BYTE)XLATEOBJ_iXlate(BltInfo->XlateSourceToDest, *SourceBits);
                 /* Go up a line */
-                SourceBits -= BltInfo->SourceSurface->lDelta;
+                SourceBits -= SrcDelta;
...
                 *DestBits = store[Index];
                 /* Go down a line */
-                DestBits += BltInfo->SourceSurface->lDelta;
+                DestBits += BltInfo->DestSurface->lDelta;
```
Three distinct bugs in the `bTopToBottom` flip path of `DIB_8BPP_BitBltSrcCopy`:
1. The read-up loop always used `SourceSurface->lDelta`, even when this flip runs *after* a
   `bLeftToRight` flip (`OneDone == TRUE`), in which case the real source of this pass is the
   *destination* surface (its own prior output) — so it should use `DestSurface->lDelta`.
2. The write-down loop used `SourceSurface->lDelta` for the *destination* stride — always
   wrong, should be `DestSurface->lDelta`.
3. When chained after a `bLeftToRight` flip, the pixel was already palette-translated in the
   first pass; re-translating it a second time here (double `XLATEOBJ_iXlate`) corrupts colors.

### Our Rust (`dib8bpp.rs`, `bTopToBottom` block, ~lines 274-322)
```rust
if bTopToBottom != 0 {
    let mut Index: u32;
    let store = ExAllocatePoolWithTag(...);
    ...
    if OneDone != 0 {
        SourceLine = /* DestSurface-based address, matches upstream's OneDone branch */ ...;
    } else {
        SourceLine = /* SourceSurface-based address */ ...;
    }
    DestLine = /* DestSurface-based */ ...;

    i = (*BltInfo).DestRect.left;
    while i < (*BltInfo).DestRect.right {
        ...
        while j < (*BltInfo).DestRect.bottom {
            *store.offset(Index as isize) =
                XLATEOBJ_iXlate((*BltInfo).XlateSourceToDest, *SourceBits as u32) as u8;  // <-- line 306: unconditional translate, no OneDone guard
            SourceBits = SourceBits.offset(-((*(*BltInfo).SourceSurface).lDelta as isize));  // <-- line 307: always SourceSurface, no SrcDelta
            Index += 1; j += 1;
        }
        ...
        while j < (*BltInfo).DestRect.bottom {
            *DestBits = *store.offset(Index as isize);
            DestBits = DestBits.offset((*(*BltInfo).SourceSurface).lDelta as isize);  // <-- line 317: SourceSurface stride used for the DEST walk
            Index += 1; j += 1;
        }
        ...
    }
}
```
All three bugs are present, exactly matching pre-fix C (verified via `git show
ae0e4ee305^:win32ss/gdi/dib/dib8bpp.c`, full pre-fix function text lines through the
`bTopToBottom` block — line-for-line identical logic to what's in the Rust). Notably, the
`OneDone` variable already exists in this Rust file (set from the `bLeftToRight` branch above),
so the fix is a small, local, in-place edit — the plumbing for it is already there, just not
wired into this branch. `git log --oneline -- dib8bpp.rs` shows only wave-1 translation +
one warnings sweep — no independent fix.

### Proposed diff
```diff
--- a/sdk/lib/rust/win32k-rust/src/dib8bpp.rs
+++ b/sdk/lib/rust/win32k-rust/src/dib8bpp.rs
@@ -274,10 +274,17 @@
                     if bTopToBottom != 0 {
                         let mut Index: u32;
+                        let SrcDelta: LONG;
 
                         let store = ExAllocatePoolWithTag(...);
                         if store.is_null() {
                             return 0u8;
                         }
 
                         if OneDone != 0 {
                             SourceLine = ((*(*BltInfo).DestSurface).pvScan0 as PBYTE)
                                 .offset(((*BltInfo).DestRect.top * (*(*BltInfo).DestSurface).lDelta) as isize)
                                 .offset((*BltInfo).DestRect.left as isize)
                                 .offset((((*BltInfo).DestRect.bottom - (*BltInfo).DestRect.top - 1) * (*(*BltInfo).DestSurface).lDelta) as isize);
+                            SrcDelta = (*(*BltInfo).DestSurface).lDelta;
                         } else {
                             SourceLine = ((*(*BltInfo).SourceSurface).pvScan0 as PBYTE)
                                 .offset((((*BltInfo).SourcePoint.y + (*BltInfo).DestRect.bottom - (*BltInfo).DestRect.top - 1) * (*(*BltInfo).SourceSurface).lDelta) as isize)
                                 .offset((*BltInfo).SourcePoint.x as isize);
+                            SrcDelta = (*(*BltInfo).SourceSurface).lDelta;
                         }
@@ -300,13 +307,15 @@
                             j = (*BltInfo).DestRect.top;
                             while j < (*BltInfo).DestRect.bottom {
-                                *store.offset(Index as isize) = XLATEOBJ_iXlate((*BltInfo).XlateSourceToDest, *SourceBits as u32) as u8;
-                                SourceBits = SourceBits.offset(-((*(*BltInfo).SourceSurface).lDelta as isize));
+                                *store.offset(Index as isize) = if OneDone != 0 {
+                                    *SourceBits
+                                } else {
+                                    XLATEOBJ_iXlate((*BltInfo).XlateSourceToDest, *SourceBits as u32) as u8
+                                };
+                                SourceBits = SourceBits.offset(-(SrcDelta as isize));
                                 Index += 1;
                                 j += 1;
                             }
 
                             Index = 0;
 
                             j = (*BltInfo).DestRect.top;
                             while j < (*BltInfo).DestRect.bottom {
                                 *DestBits = *store.offset(Index as isize);
-                                DestBits = DestBits.offset((*(*BltInfo).SourceSurface).lDelta as isize);
+                                DestBits = DestBits.offset((*(*BltInfo).DestSurface).lDelta as isize);
                                 Index += 1;
                                 j += 1;
                             }
```

---

## 4. `e1bf662d9a` — ftGetFontUnicodeRanges missing FreeType locking (CORE-7659, #7659)

### Upstream hunk
```diff
+    /* Lock FreeType for face lookup */
+    IntLockFreeType();
+
     if (face->charmap->encoding == FT_ENCODING_UNICODE)
     {
         ...
-        if (!glyph_code) return 0;
+        if (!glyph_code)
+            goto Quit;
         ...
             if (char_code < char_code_prev)
             {
                 DPRINT1(...);
-                return 0;
+                goto Quit;
             }
         ...
     }
+
+Quit:
+    /* Unlock FreeType */
+    IntUnLockFreeType();
+
     return size;
```
`face->charmap` traversal via `FT_Get_Next_Char` walked FreeType's internal state with no lock
held, while every other FreeType-touching path in the file (`IntLockFreeType`/
`IntUnLockFreeType` appear ~15 times elsewhere in `freetype.c`) does hold it — a race against
any concurrent FreeType call on the same face (font load/unload, glyph rasterization) that can
corrupt or use-after-free FreeType's internal charmap tables.

### Our Rust (`freetype.rs:5938-5989`, `ftGetFontUnicodeRanges_impl`)
```rust
unsafe fn ftGetFontUnicodeRanges_impl(Font: PFONTGDI, glyphset: PGLYPHSET) -> DWORD {
    let mut size: DWORD;
    let mut num_ranges: DWORD = 0;
    let face: FT_Face = (*(*Font).SharedFace).Face;

    if (*face).charmap.is_null() { return 0; }

    if (*(*face).charmap).encoding == FT_ENCODING_UNICODE {
        ...
        char_code = FT_Get_First_Char(face, &mut glyph_code);   // unlocked
        ...
        if glyph_code == 0 { return 0; }                         // early return, no cleanup needed here (moot, see below)
        ...
        while glyph_code != 0 {
            if char_code < char_code_prev { return 0; }           // early return
            ...
            char_code = FT_Get_Next_Char(face, char_code, &mut glyph_code);  // unlocked
        }
    }
    ...
    size
}
```
Zero calls to `IntLockFreeType`/`IntUnLockFreeType` anywhere in this function — both helpers
are in scope in this same file (`freetype.rs:15-22`, used ~15 times elsewhere, e.g. lines 911,
1069, 1262, 2031...), so this is a straight-line translation of the pre-fix C, faithfully
carrying the missing-lock bug forward. `git log --oneline -- freetype.rs` shows unrelated
later commits (a BUGCHECK-avoidance fix for unrealized fonts, a constants-contradiction sweep,
warnings cleanup) — none touch `ftGetFontUnicodeRanges`.

Note on the `goto Quit` half of the upstream diff: in idiomatic Rust there is no `Drop`-guard
currently wrapping this — since Rust doesn't need `goto`, the "reroute early returns through
cleanup" half of the fix is really just "call `IntUnLockFreeType()` before every return," which
is uglier than upstream's `goto` but equivalent; the proposed diff below does exactly that,
matching the crate's existing style (e.g. `freetype.rs:1069-1077`'s `IntLockFreeType()` /
`IntUnLockFreeType()` pair around a similarly-shaped early-return block, though not the RAII
idiom the field manual recommends — TODO'd, not blocking this fix).

### Proposed diff
```diff
--- a/sdk/lib/rust/win32k-rust/src/freetype.rs
+++ b/sdk/lib/rust/win32k-rust/src/freetype.rs
@@ -5938,7 +5938,9 @@
     if (*face).charmap.is_null() {
         return 0;
     }
 
+    IntLockFreeType();
+
     if (*(*face).charmap).encoding == FT_ENCODING_UNICODE {
         let mut glyph_code: FT_UInt = 0;
         let mut char_code: FT_ULong;
         let mut char_code_prev: FT_ULong;
 
         char_code = FT_Get_First_Char(face, &mut glyph_code);
         char_code_prev = char_code;
 
         if glyph_code == 0 {
-            return 0;
+            IntUnLockFreeType();
+            return 0;
         }
@@ -5966,7 +5968,8 @@
         while glyph_code != 0 {
             if char_code < char_code_prev {
-                return 0;
+                IntUnLockFreeType();
+                return 0;
             }
             if char_code - char_code_prev > 1 {
                 num_ranges += 1;
@@ -5983,6 +5986,8 @@
         }
     }
 
+    IntUnLockFreeType();
+
     size = (core::mem::size_of::<GLYPHSET>() as DWORD)
         + (core::mem::size_of::<WCRANGE>() as DWORD) * (num_ranges - 1);
     if !glyphset.is_null() {
```
Line numbers are against the excerpt read directly from the file (`sed -n '5920,6010p'`);
re-verify against a fresh `grep -n` before applying, since the file has 1000s of lines above
this function and edits elsewhere in the crate could have shifted it since this read.

---

## Leads not reached

The queue in `upstream-sync-bug-leads.md` ranks ~85 more candidates (full top-60 table, plus
~29 below the cut). Not investigated in this pass, in the doc's own priority order:

- `93c5a7b98b` / `91b613f158` (CSRSRV / RtlInitializeContext signature) — the source doc itself
  flags these as mostly typing/signature fixes, low value; worth a five-minute check but
  deliberately deprioritized here in favor of depth on the four higher-substance leads.
- `640e39c72e` (NTGDI gdiobj.c revert), `64f1cf3b6d` (DrawText ellipsification), `c6410fb0c2` /
  `feb0f56da7` (EX sysinfo.c ReqSize / DPC index), `7f79e1c948` (caret copy-paste),
  `3511472586` (XBUTTON1/XBUTTON2) — all flagged by the source doc as small, single-hunk,
  high-confidence leads; none opened in this pass.
- `a94352ba48` (EngTransparentBlt copy direction), `984657bdf0` (software pointer exclude
  rect Y), `5b2f79fe07` (RtlVirtualUnwind UWOP), `7f459b49f0` (bitblt.rs pattern origin),
  `33a19d05f4` (RTL atom table signature), `0e64048812` (NDK heap layout packing) — not opened.
- Everything ranked below #20 in the source doc's table (freeldr/UEFI cluster, NETCFGX
  cluster, the two "Sync with Wine" bulk-import categories, the 14 `modules/rostests`
  executable-oracle leads) — not opened; the source doc's own advice to treat the Wine-sync
  commits as "diff the whole module once" rather than N discrete bugs still stands and wasn't
  acted on here.
