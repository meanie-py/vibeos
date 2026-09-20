# Upstream-sync: win32k deleted-C-tree sweep — wave 2

Continuation of `upstream-sync-win32k-deleted-c-sweep.md` (wave 1: 10 files in depth, 13
CONFIRMED). Same method, same seam: `8d3c33ee41e` deleted win32k.sys's kernel-mode C tree;
upstream (`reactos/reactos` master) keeps fixing bugs in files that no longer exist on our
side, so the only way to find them is reading upstream commits directly and matching by
**function name** into `sdk/lib/rust/win32k-rust/src/`. `$FP` =
`5213cf717cff6946945922e6a7e03d87b0a58e4f`. Nothing in this document was applied.

Already fixed and on master (do not re-open, wave 1 + earlier verification pass): `c0f18d059a`,
`ae0e4ee305`, `e1bf662d9a`, `e7e1465812`, `640e39c72e`, `feb9febaae3`, `c65740eeb3a`,
`7f79e1c948`, `3511472586`, `984657bdf0`, `64f1cf3b6d`.

Already CONFIRMED in wave 1 (do not re-open): `88d06015f61`, `4671b481a38`, `45a4f95b04f`
(freetype.c), `5d219532335`, `9eb3bdef4c7` (desktop.c), `529f3c618df`, `7a505166b9f`
(sysparams.c), `f17d904d2d9` (menu.c), `b38db9dd238` (painting.c).

## Coverage table (files touched this pass)

| file | commits examined | verdict |
|---|---|---|
| `win32ss/gdi/ntgdi/bitblt.c` | `7f459b49f0a` | **CONFIRMED** — 2 bugs |
| `win32ss/gdi/eng/transblt.c` | `a94352ba486` | **REFUTED** — already fixed in our Rust |
| `win32ss/user/ntuser/mouse.c` | `2dddf480710`, `35114725860` | 1 **CONFIRMED**, 1 **REFUTED** |
| `win32ss/gdi/ntgdi/polyfill.c` | `d3ed29ef065` | **CONFIRMED** |
| `win32ss/gdi/ntgdi/fillshap.c` + `win32ss/gdi/ntgdi/path.c` | `049085f1060` | **CONFIRMED** (fillshap, small) + **CONFIRMED, large/architectural** (path.c) |
| `win32ss/gdi/ntgdi/palette.c` | `6d8a5fa0bbe` | **CONFIRMED** |
| `win32ss/gdi/eng/surface.c` | `5e76d075f10` | **CONFIRMED** |
| `win32ss/user/ntuser/nonclient.c` | `cf470941031` | **CONFIRMED** |
| `win32ss/gdi/eng/device.c`, `pdevobj.c`, `win32ss/user/ntuser/simplecall.c` | `6e95a132bd8` | not a bug fix (exclusive→shared lock perf change) — not pursued |
| `win32ss/user/ntuser/class.c`, `focus.c`, `layered.c` | `a45e91cb3db` | not a bug fix (same shape, lock perf) — not pursued |
| `win32ss/gdi/eng/engmisc.c`, `win32ss/user/ntuser/kbdlayout.c`, `display.c` | `9d1418fcb6e`, `1cf31524d50` | subject-line only, not opened this pass |
| `win32ss/user/ntuser/hotkey.c`, `defwnd.c`, `main.c`, `misc.c`, `ntuser.c`, `object.c`, `callback.c` | `5ab1ff188c3` | comment-removal only per wave-1's classification of the same sha elsewhere — not opened |
| `win32ss/user/ntuser/main.c`, `object.c`, `ntstubs.c` | `b272427baa3`, `c864b270e06` | feature work (Job UI restriction, raw input) — not opened |
| `win32ss/user/ntuser/winsta.c` | `2eb28656dab`, `7e08891a9f3`, `2b8f2020d70` | not opened this pass (still open from wave 1's flag) |
| `win32ss/gdi/ntgdi/gdiobj.c` | `640e39c72e8` | already fixed (= wave 1's `640e39c72e`) |

**Totals this pass: 6 files with a real CONFIRMED finding (8 distinct bugs across them,
counting bitblt.c's 2 and fillshap+path as 2), 1 REFUTED-outright (transblt), 1 partially
REFUTED (mouse.c's XBUTTON1/2 sha; the sibling sha in the same file is CONFIRMED), 2 files
found to be pure lock-type perf changes (not bugs, not pursued further).**

---

## Verdict table (new findings this pass)

| # | sha | file | bug | our Rust file:line | verdict |
|---|---|---|---|---|---|
| 1 | `7f459b49f0a` | bitblt.c | `IntGdiFillRgn` passes `pdc->pdcattr->ptlBrushOrigin` (raw logical brush origin) to `IntEngPaint` instead of `pdc->ptlFillOrigin` (the already-window-adjusted origin) — pattern-brush fills paint with the wrong origin | `ntbitblt.rs:1765-1799` (`IntGdiFillRgn`), line ~1839 `&mut (*(*pdc).pdcattr).ptlBrushOrigin` | **CONFIRMED** |
| 2 | `7f459b49f0a` | bitblt.c | `NtGdiGetPixel` builds `rcDest` from the raw logical `x`/`y` instead of the DC-origin-translated `ptlSrc.x`/`ptlSrc.y` it just computed — `GetPixel()` samples the wrong pixel whenever the DC has a non-zero origin | `ntbitblt.rs:2145-2148` (`#[cfg(target_arch = "x86")]` variant) and `:2249-2252` (`#[cfg(not(target_arch = "x86"))]` variant) — **both arch variants have the bug** | **CONFIRMED** |
| 3 | `a94352ba486` | transblt.c | (upstream fixes a misplaced `?:` operator precedence bug — brackets picked the wrong operand — in `EngTransparentBlt`'s direction calc) | `transblt.rs:197-206` already reads `if OutputRect.left < InputRect.left { CD_RIGHTDOWN } else { CD_LEFTDOWN }` — the correct, already-fixed shape | **REFUTED** (our translation post-dates or independently avoided the bug) |
| 4 | `2dddf480710` | mouse.c | `UserProcessMouseInput`'s X-button-4/5 "flush" checks test `mi.dwFlags & (MOUSE_BUTTON_4_DOWN\|MOUSE_BUTTON_4_UP)` (and the `_5_` sibling) — but `mi.dwFlags` only ever holds `MOUSEEVENTF_*` bits, never the raw `MOUSE_BUTTON_N_*` bits that come from `mid->ButtonFlags`, so the flush test checks the wrong bit namespace entirely (should be `MOUSEEVENTF_XDOWN\|MOUSEEVENTF_XUP`) | `ntmouse.rs:208` and `:222` | **CONFIRMED** |
| 5 | `35114725860` | mouse.c | (upstream fixes `UserSendMouseInput`'s XBUTTON1-up event reporting `XBUTTON2` in `wParam`'s high word instead of `XBUTTON1`) | `ntmouse.rs:388` already reads `MAKEWPARAM(0, XBUTTON1 as WORD)` — the correct, already-fixed shape | **REFUTED** |
| 6 | `d3ed29ef065` | polyfill.c | `POLYGONFILL_FillScanLineWinding`'s scanline-merge test has `x2 >= newx2 && x2 <= newx2` (always false unless `x2 == newx2`) where it should be `x2 >= newx1 && x2 <= newx2` — an active-edge merge case that should combine is missed, degrading (not crashing) polygon fill quality/correctness for winding-mode fills | `polyfill.rs:538` | **CONFIRMED** |
| 7 | `049085f1060` | fillshap.c | `NtGdiPolyPolyDraw`'s `GdiPolyBezierTo` case calls `IntGdiPolyBezierTo` unconditionally with no validation that `*SafeCounts` is `3n+1` (the only valid point count for a Bezier chain with an implicit start point) — malformed input (e.g. from pptviewer '97-style callers) is accepted and fed straight into the Bezier renderer instead of being rejected with `ERROR_INVALID_PARAMETER` | `fillshap.rs:846-848` (the `x if x == 5` arm) | **CONFIRMED** |
| 8 | `049085f1060` | path.c | The whole `add_log_points`/`add_log_points_new_stroke` family is missing the new `bExtraPt`/`newStroke`-conditioned "prepend the current position as an implicit extra point" logic that `PATH_PolyBezierTo`/`PATH_PolylineTo` need — a faithful port touches `add_log_points`, `add_log_points_new_stroke`, `PATH_LineTo`, `PATH_PolyBezierTo`, `PATH_PolyBezier`, `PATH_PolyDraw`, `PATH_PolylineTo`, `PATH_PolyPolygon` (8 call-site signature changes) | `path.rs:638` (`add_log_points`, 2-arg-short of upstream's new signature), `:705` (`add_log_points_new_stroke`, same), plus the 6 call sites at `:760`, `:1350`, `:1374`, `:1428`, `:1548`, `:1588` | **CONFIRMED, large/architectural** — see detail below, not diffed |
| 9 | `6d8a5fa0bbe` | palette.c | `NtGdiCreateHalftonePalette(NULL)` unconditionally calls `DC_LockDc(NULL)`, fails, and returns `NULL` — but Windows/Wine and the MS SDK headers document `hDC` as optional; real apps (IE8, Encarta per the commit) call it with `NULL` and get nothing from us | `palette.rs:573-581` | **CONFIRMED** |
| 10 | `5e76d075f10` | surface.c | `SURFACE_AllocSurface` runs the `RtlULongMult(cjWidth, cy, &cjBits)` overflow-checked size calculation **unconditionally**, even when no caller-supplied buffer/size was given (the `EngCreateBitmap`-style allocation path) — some real display drivers (Radeon IGP 320M per the commit, `CORE-13036`/`CORE-11676`) pass `cjWidth`×`cy` values that legitimately look like they'd overflow the checked multiply but are harmless in the "we're allocating our own buffer" path, and Windows itself never validates this case | `surface.rs:145-184`: the overflow-checked `RtlULongMult` call happens before the `if !pvBits.is_null() && cjBufSize != 0` branch, instead of only inside it | **CONFIRMED** |
| 11 | `cf470941031` | nonclient.c | Both the move/size modal message loops (`DefWndStartSizeMove` and `DefWndDoSizeMove`) call `co_IntGetPeekMessage(..., PM_REMOVE, ...)` and process the message without ever checking for `WM_QUIT` — a `WM_QUIT` posted while the calling thread is tearing itself down (e.g. right after `PostQuitMessage`) gets silently eaten by the modal loop instead of reaching the thread's outer `GetMessage()` loop, so the thread never sees its own quit and hangs. Upstream's own repro: "Boot to first-boot setup, drag the install/status window while it is finishing" — this is a real first-boot/setup hang, smoke-adjacent if any smoke scenario ever drags a window during shutdown | `nonclient.rs:508` (`DefWndStartSizeMove_impl`'s loop) and `:731` (`DefWndDoSizeMove_impl`'s loop) — neither checks `msg.message == WM_QUIT` | **CONFIRMED** |

INCONCLUSIVE / not pursued (real subject, but out of budget or not a correctness bug):

| sha | file | why |
|---|---|---|
| `6e95a132bd8` | device.c, pdevobj.c, simplecall.c, menu.c, window.c, winpos.c | Pure `EngAcquireSemaphore`→`EngAcquireSemaphoreShared` swap on read-only paths. Described by upstream as a contention/perf optimization ("avoid serializing read-only display queries"), not a correctness fix — no bug is being closed, so not a candidate for this seam. Confirmed by reading the full diff, not the subject. |
| `a45e91cb3db` | class.c, focus.c, ime.c, layered.c | Same shape as above (exclusive→shared lock for read-only NTUSER queries: foreground window lookup, IME hotkey/context queries, layered window attributes, title bar info). Not pursued. |

---

## Detail + proposed diffs (CONFIRMED, small)

### 1+2. `7f459b49f0a` — wrong origin used in `IntGdiFillRgn` and `NtGdiGetPixel` (bitblt.c)

Upstream:
```diff
     bRet = IntEngPaint(&pdc->dclevel.pSurface->SurfObj,
                        (CLIPOBJ *)&xcoClip,
                        pbo,
-                       &pdc->pdcattr->ptlBrushOrigin,
+                       &pdc->ptlFillOrigin,
                        mix);
@@ NtGdiGetPixel
-    rcDest.left = x;
-    rcDest.top = y;
-    rcDest.right = x + 1;
-    rcDest.bottom = y + 1;
+    rcDest.left = ptlSrc.x;
+    rcDest.top = ptlSrc.y;
+    rcDest.right = ptlSrc.x + 1;
+    rcDest.bottom = ptlSrc.y + 1;
```
`ptlFillOrigin` already exists on `PDC` in our tree (used correctly elsewhere: `dcobjs.rs:304`,
`dcutil.rs:805,864`, `coord.rs:1739`) — this is a call site that was never updated, not a
missing field.

Proposed diffs:
```diff
--- a/sdk/lib/rust/win32k-rust/src/ntbitblt.rs
+++ b/sdk/lib/rust/win32k-rust/src/ntbitblt.rs
@@ IntGdiFillRgn
     bRet = IntEngPaint(
         ...,
         pbo,
-        &mut (*(*pdc).pdcattr).ptlBrushOrigin,
+        &mut (*pdc).ptlFillOrigin,
         ...,
     );
```
```diff
--- a/sdk/lib/rust/win32k-rust/src/ntbitblt.rs
+++ b/sdk/lib/rust/win32k-rust/src/ntbitblt.rs
@@ NtGdiGetPixel (both #[cfg(target_arch = "x86")] and #[cfg(not(...))] copies)
-        rcDest.left = x;
-        rcDest.top = y;
-        rcDest.right = x + 1;
-        rcDest.bottom = y + 1;
+        rcDest.left = ptlSrc.x;
+        rcDest.top = ptlSrc.y;
+        rcDest.right = ptlSrc.x + 1;
+        rcDest.bottom = ptlSrc.y + 1;
```
Apply to **both** arch-gated copies of `NtGdiGetPixel` (x86 at `:2114-2160`ish, non-x86 at
`:2217-2260`ish) — they are near-duplicate functions in this file, both carry the bug
independently.

### 4. `2dddf480710` — X-button flush check tests the wrong bit namespace (mouse.c)

Our Rust (`ntmouse.rs:199-224`):
```rust
if (*mid).__bindgen_anon_1.__bindgen_anon_1.ButtonFlags as u32 & MOUSE_BUTTON_4_DOWN != 0 {
    mi.dwFlags |= MOUSEEVENTF_XDOWN;
    mi.mouseData |= XBUTTON1 as DWORD;
}
if (*mid).__bindgen_anon_1.__bindgen_anon_1.ButtonFlags as u32 & MOUSE_BUTTON_4_UP != 0 {
    mi.dwFlags |= MOUSEEVENTF_XUP;
    mi.mouseData |= XBUTTON1 as DWORD;
}

if mi.dwFlags & (MOUSE_BUTTON_4_DOWN | MOUSE_BUTTON_4_UP) != 0 {   // <-- wrong bits
    UserSendMouseInput(&mut mi, FALSE as BOOL);
    RtlZeroMemory(&mut mi as *mut _ as *mut _, core::mem::size_of::<MOUSEINPUT>());
}
```
`mi.dwFlags` is only ever assigned `MOUSEEVENTF_*` constants in this function (see the
`MOUSEEVENTF_MOVE`/`_ABSOLUTE`/`_LEFTDOWN`/etc. assignments above it) — `MOUSE_BUTTON_4_DOWN`/
`_UP` are a completely different bit-value namespace belonging to `mid->ButtonFlags`
(`MOUSE_INPUT_DATA`), not `MOUSEINPUT::dwFlags`. Whether this test evaluates true or false
depends entirely on incidental bit-pattern overlap between the two enums, not on whether a
button-4/5 event actually occurred — the same class of bug the sibling `35114725860` fix (in
the same file) closed, just not yet fixed upstream-for-us here since it's a separate sha.

Proposed diff:
```diff
--- a/sdk/lib/rust/win32k-rust/src/ntmouse.rs
+++ b/sdk/lib/rust/win32k-rust/src/ntmouse.rs
@@ UserProcessMouseInput
-    if mi.dwFlags & (MOUSE_BUTTON_4_DOWN | MOUSE_BUTTON_4_UP) != 0 {
+    if mi.dwFlags & (MOUSEEVENTF_XDOWN | MOUSEEVENTF_XUP) != 0 {
         UserSendMouseInput(&mut mi, FALSE as BOOL);
         RtlZeroMemory(&mut mi as *mut _ as *mut _, core::mem::size_of::<MOUSEINPUT>());
     }
@@
-    if mi.dwFlags & (MOUSE_BUTTON_5_DOWN | MOUSE_BUTTON_5_UP) != 0 {
+    if mi.dwFlags & (MOUSEEVENTF_XDOWN | MOUSEEVENTF_XUP) != 0 {
         UserSendMouseInput(&mut mi, FALSE as BOOL);
         RtlZeroMemory(&mut mi as *mut _ as *mut _, core::mem::size_of::<MOUSEINPUT>());
     }
```
(`MOUSEEVENTF_XDOWN`/`MOUSEEVENTF_XUP` are already imported/used earlier in the same function,
so this is a drop-in constant swap, no new `use`.)

### 6. `d3ed29ef065` — inverted lower-bound test in polygon scanline merge (polyfill.c)

Our Rust (`polyfill.rs:532-542`):
```rust
let newx1 = (*pLeft).XIntercept[0];
let newx2 = (*pRight).XIntercept[1];
if (newx1 >= x1 && newx1 <= x2) ||
   (newx2 >= x1 && newx2 <= x2) ||
   (x1 >= newx1 && x1 <= newx2) ||
   (x2 >= newx2 && x2 <= newx2)          // <-- should be newx1
{
    x1 = if x1 < newx1 { x1 } else { newx1 };
    x2 = if x2 > newx2 { x2 } else { newx2 };
    ...
```
`x2 >= newx2 && x2 <= newx2` can only be true when `x2 == newx2` exactly — it is not a range
test at all, so this disjunct is nearly always false, dropping the one case it's meant to catch
(`x2` sits inside `[newx1, newx2]`). Net effect: some overlapping/touching active edges in a
winding-mode polygon fill that should merge into one span don't, degrading fill correctness at
polygon self-intersections.

Proposed diff:
```diff
--- a/sdk/lib/rust/win32k-rust/src/polyfill.rs
+++ b/sdk/lib/rust/win32k-rust/src/polyfill.rs
@@ POLYGONFILL_FillScanLineWinding
         if (newx1 >= x1 && newx1 <= x2) ||
            (newx2 >= x1 && newx2 <= x2) ||
            (x1 >= newx1 && x1 <= newx2) ||
-           (x2 >= newx2 && x2 <= newx2)
+           (x2 >= newx1 && x2 <= newx2)
         {
```

### 7. `049085f1060` (fillshap.c half) — unvalidated Bezier point count (`NtGdiPolyPolyDraw`)

Our Rust (`fillshap.rs:840-850`, the `x if x == 5 =>` arm for `GdiPolyBezierTo`):
```rust
x if x == 5 => {
    Ret = IntGdiPolyBezierTo(dc, SafePoints as *mut POINTL, *SafeCounts);
}
```
Upstream now requires, per Wine 10.0's `NtGdiPolyPolyDraw`, that a single-polygon Bezier
request have `UnsafeCounts[0] != 1 && UnsafeCounts[0] % 3 == 1` (a valid Bezier chain with an
implicit start point is `3n+1` points), decrements the count by one to drop the now-implicit
start point, and rejects anything else with `ERROR_INVALID_PARAMETER` instead of handing a
malformed count straight to the renderer.

Proposed diff:
```diff
--- a/sdk/lib/rust/win32k-rust/src/fillshap.rs
+++ b/sdk/lib/rust/win32k-rust/src/fillshap.rs
@@ NtGdiPolyPolyDraw
-        x if x == 5 => {
-            Ret = IntGdiPolyBezierTo(dc, SafePoints as *mut POINTL, *SafeCounts);
-        }
+        x if x == 5 => {
+            // From Wine 10.0 dlls/win32u/painting.c NtGdiPolyPolyDraw:
+            // UnsafeCounts[0] must be 3*n + 1 (n >= 1).
+            if Count == 1 && *SafeCounts.offset(0) != 1 && (*SafeCounts.offset(0)) % 3 == 1 {
+                *SafeCounts.offset(0) -= 1;
+                Ret = IntGdiPolyBezierTo(dc, SafePoints as *mut POINTL, *SafeCounts);
+            } else {
+                EngSetLastError(ERROR_INVALID_PARAMETER);
+                Ret = 0;
+            }
+        }
```
(`SafeCounts` indexing/mutability should mirror however this function already reads
`*SafeCounts` a few lines above — check whether `SafeCounts` is a `*mut ULONG` or a slice
reference in this translation before applying; the exact deref syntax here is illustrative, not
copy-paste-verified against the file's actual pointer type.)

---

## Detail (CONFIRMED, large/architectural — flagged, not diffed)

### 8. `049085f1060` (path.c half) — `add_log_points` missing "prepend current position" support

Upstream's `add_log_points`/`add_log_points_new_stroke` (Wine-10.0-derived) gained a `BOOL
bExtraPt` parameter: when set (and, in the `add_log_points_new_stroke`-driven callers, when
`path->numEntriesUsed == 1`), the function copies **one extra point** — the path's current
position — ahead of the caller-supplied points, and calls `IntLPtoDP` over `count + 1` points
instead of `count`. This closes the actual "pptviewer '97" bug: `PolylineTo`/`PolyBezierTo`
were building the point path without ever anchoring it at the figure's current position, so
Bezier/line strokes rendered detached from where the pen actually was.

Our Rust (`path.rs`) has:
```rust
unsafe fn add_log_points(dc: *mut DC, path: PPATH, points: *const POINT, count: DWORD, type_: BYTE) -> *mut BYTE { ... }
unsafe fn add_log_points_new_stroke(dc: *mut DC, path: PPATH, points: *const POINT, count: DWORD, type_: BYTE) -> BOOL { ... }
```
— both **missing** the new parameter entirely, and all 6 call sites (`PATH_LineTo:760`,
`PATH_PolyBezierTo:1350`, `PATH_PolyBezier:1374`, `PATH_PolyDraw:1428`, `PATH_PolylineTo:1548`,
`PATH_PolyPolygon:1588`) call the old 5-argument shape. This is the pre-fix state, confirmed
present and unmodified since translation (`git log --oneline -- sdk/lib/rust/win32k-rust/src/path.rs`
shows only the original wave translation commits, no independent path.c-area fix).

A faithful port needs, per upstream's diff:
1. `add_log_points(dc, path, points, count, type, bExtraPt: BOOL)` — when `bExtraPt &&
   path->numEntriesUsed == 1`, `memcpy`/`IntLPtoDP` over `count + 1` points starting at index 0
   instead of `count` points at `numEntriesUsed`.
2. `add_log_points_new_stroke(..., bExtraPt: BOOL)` threading the same flag through.
3. Call-site updates: `PATH_LineTo`/`PATH_PolyBezier`/`PATH_PolyDraw`/`PATH_PolyPolygon` pass
   `FALSE` (unchanged behavior); `PATH_PolyBezierTo`/`PATH_PolylineTo` pass `TRUE`.
4. `PATH_PolylineTo` additionally gets `if pPath->newStroke { cbPoints--; }` before the call —
   check whether our `path.rs`'s `PPATH` struct exposes a `newStroke` field before porting this
   part; not verified this pass.

Not diffed here: 8 call sites plus 2 function-signature changes is a genuine multi-site,
multi-function port (same shape as wave 1's freetype font-lookup-cache revert and desktop
wallpaper rewrite) rather than a 1-2 line patch. Recommend a dedicated follow-up that reads
`049085f1060`'s full `path.c` diff side-by-side with `path.rs`'s `PATH_*` family.

---

## Not reached — resume point for the next pass

**Still fully unopened** (from wave 1's list, no commits examined this pass beyond the
subject-line triage already done in wave 1 or the coverage table above):
`win32ss/gdi/eng/engmisc.c` (`9d1418fcb6e` "Improve tick count to ms calculation" — worth a
look, tick-count arithmetic bugs are a real class), `win32ss/user/ntuser/hotkey.c`,
`win32ss/user/ntuser/kbdlayout.c` + `win32ss/user/ntuser/display.c` (both only touched by
`1cf31524d50` "Stop using non-conforming swprintf/vswprintf" — worth one `git show` to check if
it's a pure API swap or fixes a real format-string/buffer bug), `win32ss/user/ntuser/defwnd.c`,
`win32ss/user/ntuser/main.c`, `win32ss/user/ntuser/misc.c`, `win32ss/user/ntuser/ntuser.c`,
`win32ss/user/ntuser/object.c`, `win32ss/user/ntuser/ntstubs.c`, `win32ss/user/ntuser/callback.c`
(all only touched by `5ab1ff188c3` "Remove needless comments" per wave 1's classification of
that sha in other files — likely cosmetic everywhere, but not independently verified for these
specific files this pass), `win32ss/user/ntuser/main.c` + `object.c` also touched by
`b272427baa3` (Job UI Restriction — feature) and `object.c`/`ntstubs.c` by `c864b270e06` (raw
input device list — feature, both skippable), `win32ss/user/ntuser/winsta.c`'s
`2eb28656dab` "Improve initialization of window station and desktop objects" (still flagged
from wave 1 as worth a full read — "improve initialization" can hide a real ordering bug, not
opened this pass either) and `7e08891a9f3`/`2b8f2020d70` (desktop-switch-by-event feature +
formatting).

**Read this pass, not real bugs (do not re-open):** `6e95a132bd8`, `a45e91cb3db` (both pure
exclusive→shared lock perf changes on read-only paths, no correctness fix).

**Read this pass, CONFIRMED already-fixed or already-correct in our Rust (do not re-open, but
also don't re-count as new findings if seen again):** `a94352ba486` (transblt.c, REFUTED),
`35114725860` (mouse.c, REFUTED).

**Files from the original 49 not mentioned in wave 1 or wave 2 at all** — cross-check against
the full 49-file list in the lane doc's header before assuming this covers everything; this
document did not re-derive the full file list, only continued from wave 1's explicit "Not
reached" section plus its "single-commit spotted" queue (device.c, engmisc.c, pdevobj.c,
stubs.c, gdiobj.c, class.c, hotkey.c, kbdlayout.c, display.c, defwnd.c, main.c, misc.c,
ntuser.c, simplecall.c, object.c, ntstubs.c, nonclient.c, focus.c, layered.c, callback.c,
bitblt.c, transblt.c, mouse.c, polyfill.c, path.c, fillshap.c, palette.c, surface.c, winsta.c —
29 files). Of those 29: **9 got a real `git show` this pass** (bitblt.c, transblt.c, mouse.c,
polyfill.c, path.c, fillshap.c, palette.c, surface.c, nonclient.c); 2 got a `git show` but
turned out to be lock-perf, not bugs (device.c/pdevobj.c/simplecall.c via `6e95a132bd8`,
class.c/focus.c/layered.c via `a45e91cb3db`); the remaining ~18 files' commits were only
identified by subject line (`git log --oneline`), never `git show`n — that's the resume point.
`win32ss/gdi/ntgdi/stubs.c`'s `0566a0f6998` was already classified as a refactor in wave 1
(shared with freetype.c/font.c) — skip.
