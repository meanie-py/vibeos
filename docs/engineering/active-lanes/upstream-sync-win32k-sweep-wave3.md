# Upstream-sync: win32k deleted-C-tree sweep — wave 3

Continuation of `upstream-sync-win32k-deleted-c-sweep.md` (wave 1) and
`upstream-sync-win32k-sweep-wave2.md` (wave 2). Same seam: `8d3c33ee41e` deleted win32k.sys's
kernel-mode C tree; upstream (`reactos/reactos` master) keeps fixing bugs in files that no
longer exist on our side, so the only way to find them is reading upstream commits directly and
matching by **function name** into `sdk/lib/rust/win32k-rust/src/`. `$FP` =
`5213cf717cff6946945922e6a7e03d87b0a58e4f`. Nothing in this document was applied.

This pass finished wave 2's explicit "not reached" queue (the ~18 files identified there by
subject line only), then re-derived the full list of the 179 deleted files against
`$FP..upstream/master` to check for coverage gaps — and found **5 files neither wave had
listed at all**: `error.c`, `dclife.c`, `keyboard.c`, `window.c`, `winpos.c`. All 5 were opened
this pass.

Already fixed and on master (do not re-open): the 25-sha list in the task brief (includes wave
1's 13 + wave 2's finds already landed, per the brief's own list — not re-verified here).

## Coverage table (files with a `git show` this pass)

| file | commits examined | verdict |
|---|---|---|
| `win32ss/gdi/eng/engmisc.c` | `9d1418fcb6e` | **REFUTED** — pure refactor, mathematically identical formula |
| `win32ss/user/ntuser/kbdlayout.c`, `win32ss/user/ntuser/display.c` | `1cf31524d50` | **REFUTED** — pure `swprintf`→`_swprintf` rename, no behavior change |
| `win32ss/user/ntuser/hotkey.c`, `defwnd.c`, `main.c`, `misc.c`, `ntuser.c`, `object.c`, `callback.c` | `5ab1ff188c3` | **REFUTED** — confirmed comment-only diff (`git show --stat`: pure deletions, no `+` lines except headers) |
| `win32ss/user/ntuser/main.c`, `object.c` | `b272427baa3` | feature (Job UI Restriction, +1765/-21 across 22 files, all new subsystem) — not pursued |
| `win32ss/user/ntuser/object.c`, `ntstubs.c` | `c864b270e06` | feature (raw input device list, all-new `rawinput.c`) — not pursued |
| `win32ss/gdi/eng/stubs.c` | `0566a0f6998` | already classified wave 1 (shared w/ freetype.c/font.c) — refactor, skip |
| `win32ss/user/ntuser/winsta.c` + `desktop.c` + `menu.c` | `2eb28656dab` | **CONFIRMED** — see detail below |
| `win32ss/user/ntuser/winsta.c` | `1968202840d` | **INCONCLUSIVE / not a correctness bug** — see detail |
| `win32ss/user/ntuser/winsta.c` + `desktop.c` | `2b8f2020d70` | **REFUTED** — pure formatting/comment diff, confirmed line-by-line |
| `win32ss/user/ntuser/winsta.c` + `desktop.c` | `7e08891a9f3` | feature (new `WinSta0_DesktopSwitch` event + `EVENT_SYSTEM_DESKTOPSWITCH`, +265/-16) — not pursued |
| `win32ss/gdi/eng/error.c` | `5ab1ff188c3` | **REFUTED** — same comment-removal commit, confirmed |
| `win32ss/gdi/ntgdi/dclife.c` | `aaab5f01981` | **REFUTED as a live bug** — see detail (our `NtGdiCreateCompatibleDC` is already a bare passthrough, no probe/mode-check logic the upstream fix needed to bypass) |
| `win32ss/user/ntuser/keyboard.c` | `e22f4474673`, `cac013abef3` | feature (EnableHexNumpad, Alt+Numpad input) — not pursued |
| `win32ss/user/ntuser/window.c` | `9d287f3c5aa` | **REFUTED as a bug** — confirmed dead-code/hack removal only, not a correctness fix (see detail) |
| `win32ss/user/ntuser/window.c`, `winpos.c` | `6e95a132bd8` | already classified (lock-perf, wave 2) |
| `win32ss/user/ntuser/winpos.c` | `d09a3600606` | **CONFIRMED** — see detail below |
| `win32ss/user/ntuser/ime.c` | `eb9a73c9755` | **REFUTED** — bit-identical refactor (`0x80000000` == `HIWORD(x)&KF_UP`, etc.), confirmed by hand |
| `win32ss/user/ntuser/ime.c` | `215d409f52b`, `61fb4db34c6`, `80bd4608363` | feature/header work — not pursued |
| `win32ss/user/ntuser/input.c` | `36ffee8ea38` | feature (new `LastSystemRITEventTickCount` field, previously always 0) — not pursued |
| `win32ss/user/ntuser/input.c` | `a70008c4379`, `88cffd7b660` | feature (raw input device list, shared w/ `c864b270e06`) — not pursued |

**Totals this pass: 2 CONFIRMED (both real, both cleanly portable), 1 INCONCLUSIVE/not-a-bug,
7 files/commit-groups REFUTED after full-diff reading (not just subject line), remaining
~10 commits classified as feature work and not pursued (consistent with wave 1/2's stance that
new subsystems are out of scope for this seam).**

---

## Verdict table (new findings this pass)

| # | sha | file | bug | our Rust file:line | verdict |
|---|---|---|---|---|---|
| 1 | `2eb28656dab` | winsta.c, desktop.c, menu.c | Window-station and desktop objects have their `dwSessionId` assigned *late* in initialization (after `ObCreateObject()` succeeds but before other init steps that can fail and trigger `ObDereferenceObject()`); on such an early failure, `ExpWin32SessionCallout()` (the Ob delete callback) reads a garbage/zero `dwSessionId` from the object's common header. Also: 3 read sites (`NtUserSwitchDesktop`, `IntDestroyMenuObject`) still read `pdesk->rpwinstaParent->dwSessionId` instead of the now-authoritative `pdesk->dwSessionId` set directly on the desktop | `desktop.rs:486-582` (`IntDesktopObjectParse` — no `RtlZeroMemory`/`dwSessionId` set right after `ObCreateObject`), `desktop.rs:2796-2811` (`UserInitializeDesktop` — `RtlZeroMemory` still here, late), `desktop.rs:3014` (`IntCreateDesktop_impl` — `dwSessionId` set late), `desktop.rs:3370` (`NtUserSwitchDesktop` reads `rpwinstaParent->dwSessionId`), `menu.rs:636` (`IntDestroyMenuObject_impl`, same stale-read pattern), `winsta.rs:854` (`IntCreateWindowStation_impl` — ordering matches pre-fix, though see note below) | **CONFIRMED** |
| 2 | `d09a3600606` | winpos.c | `co_WinPosSetWindowPos` never requests a full non-client repaint when a window's frame *geometry* changes (only on state/visibility changes) — after a resize/frame-change, stale caption-button bits can persist because the NC repaint clip region isn't forced to the full frame | `winpos.rs:2320-2324` (right after the existing `Window->state \|= WNDS_SENDNCPAINT` block for `SWP_STATECHANGED`) | **CONFIRMED** |

INCONCLUSIVE / not pursued as a live bug:

| sha | file | why |
|---|---|---|
| `1968202840d` | winsta.c | `UserCreateWinstaDirectory()` swaps `NtCurrentPeb()->SessionId` for `PsGetCurrentProcessSessionId()` and promotes a local, intentionally-leaked `hWinstaDir` to a global `ghWinStaDir` (for future win32k unload support). This runs once at win32k init in the System process context; there is no smoke-relevant failure mode demonstrated (upstream's own message frames it as "doesn't require a PEB", not "the PEB read was wrong"), and the global-handle change is pure future-proofing, not a fix. Our Rust's `IntCreateWindowStation` (a *different* function, checked under finding #1) still uses `NtCurrentPeb()` too, so if this is ever revisited it's the same pattern, but I found no evidence of a correctness difference to act on now. |

---

## Detail + proposed diffs (CONFIRMED)

### 1. `2eb28656dab` — window station / desktop `dwSessionId` set too late (winsta.c, desktop.c, menu.c)

Upstream's full rationale (from the commit message): as soon as `ObCreateObject()` succeeds,
zero the object and stamp its session ID *before* any further initialization that could fail —
because on a later failure, `ObDereferenceObject()` triggers the Win32 delete callout
(`ExpWin32SessionCallout`, `ntoskrnl/ex/win32k.c`), which needs a valid `dwSessionId` in the
object's common header to route the delete. If init fails before the session ID is set, that
callout reads garbage.

Concretely for **desktop.c**, before the fix: `RtlZeroMemory(pdesk, sizeof(DESKTOP))` happens
inside `UserInitializeDesktop()` (called well after `ObCreateObject`), and
`pdesk->dwSessionId = PsGetCurrentProcessSessionId()` happens even later, inside
`IntCreateDesktop()`, after the desktop *window* has already been created. Between
`ObCreateObject()` succeeding in `IntDesktopObjectParse()` and either of those points sits
`IntAssignDesktopSecurityOnParse()`, which **can fail** — and on failure calls
`ObDereferenceObject(Desktop)` on an object that was never zeroed and never given a session ID.

Our Rust reproduces this exactly:
```rust
// desktop.rs:559-572, IntDesktopObjectParse
Status = ObCreateObject(..., &mut Desktop as *mut PDESKTOP as *mut *mut core::ffi::c_void);
if !NT_SUCCESS(Status) { return Status; }

Status = IntAssignDesktopSecurityOnParse(WinStaObject, Desktop, AccessState);
if !NT_SUCCESS(Status) {
    ObDereferenceObject(Desktop as *mut core::ffi::c_void);   // Desktop: not zeroed, no dwSessionId yet
    return Status;
}
```
```rust
// desktop.rs:2811, UserInitializeDesktop — RtlZeroMemory still here (should move to IntDesktopObjectParse)
RtlZeroMemory(pdesk as *mut core::ffi::c_void, core::mem::size_of::<DESKTOP>() as _);
```
```rust
// desktop.rs:3014, IntCreateDesktop_impl — dwSessionId still set this late
(*pdesk).dwSessionId = PsGetCurrentProcessSessionId();
```
And the two stale-read call sites upstream also fixed (now redundant once `pdesk->dwSessionId`
is trustworthy from creation, but still correct either way once the write-side fix lands):
```rust
// desktop.rs:3370, NtUserSwitchDesktop
if PsGetCurrentProcessSessionId() != (*(*pdesk).rpwinstaParent).dwSessionId {
```
```rust
// menu.rs:636, IntDestroyMenuObject_impl
== (*(*(*Menu).head.rpdesk).rpwinstaParent).dwSessionId
```

`winsta.c`'s `IntCreateWindowStation` (`winsta.rs:801-857`) has the analogous ordering
(zero → `InitializeListHead` → `dwSessionId =` → `RtlCreateAtomTable`, which can fail and
dereference), but in *this specific function* `dwSessionId` is already assigned before the one
fallible step (`RtlCreateAtomTable`) that can trigger `ObDereferenceObject` — so unlike the
desktop case, there is no actual failure window here even though the line order doesn't match
upstream byte-for-byte. Including it in the diff below for parity/robustness (matching upstream
exactly costs nothing and guards against a future added failure point between `ObCreateObject`
and the atom-table call), but flagging that **the desktop.c fix is the one with a real,
demonstrable failure window; the winsta.c part is defense-in-depth, not a currently-exploitable
gap**.

Proposed diff:
```diff
--- a/sdk/lib/rust/win32k-rust/src/desktop.rs
+++ b/sdk/lib/rust/win32k-rust/src/desktop.rs
@@ IntDesktopObjectParse
     Status = ObCreateObject(
         ...,
         &mut Desktop as *mut PDESKTOP as *mut *mut core::ffi::c_void,
     );
     if !NT_SUCCESS(Status) {
         return Status;
     }
+    RtlZeroMemory(Desktop as *mut core::ffi::c_void, core::mem::size_of::<DESKTOP>());
+    (*Desktop).dwSessionId = PsGetCurrentProcessSessionId();
+    debug_assert_eq!((*Desktop).dwSessionId, (*WinStaObject).dwSessionId);
 
     Status = IntAssignDesktopSecurityOnParse(WinStaObject, Desktop, AccessState);
```
```diff
--- a/sdk/lib/rust/win32k-rust/src/desktop.rs
+++ b/sdk/lib/rust/win32k-rust/src/desktop.rs
@@ UserInitializeDesktop
-    RtlZeroMemory(pdesk as *mut core::ffi::c_void, core::mem::size_of::<DESKTOP>() as _);
-
     if pwinsta == InputWindowStation {
```
(the zeroing now happens once, in `IntDesktopObjectParse`, before this function is ever called —
this delete matches upstream removing the *now-redundant* `RtlZeroMemory` call from
`UserInitializeDesktop`.)
```diff
--- a/sdk/lib/rust/win32k-rust/src/desktop.rs
+++ b/sdk/lib/rust/win32k-rust/src/desktop.rs
@@ IntCreateDesktop_impl
-        (*pdesk).dwSessionId = PsGetCurrentProcessSessionId();
         (*pdesk).DesktopWindow = UserHMGetHandle(pWnd);
```
(delete only the now-redundant assignment; `DesktopWindow`/`pDeskInfo->spwnd` stay as-is.)
```diff
--- a/sdk/lib/rust/win32k-rust/src/desktop.rs
+++ b/sdk/lib/rust/win32k-rust/src/desktop.rs
@@ NtUserSwitchDesktop
-        if PsGetCurrentProcessSessionId() != (*(*pdesk).rpwinstaParent).dwSessionId {
+        if PsGetCurrentProcessSessionId() != (*pdesk).dwSessionId {
```
```diff
--- a/sdk/lib/rust/win32k-rust/src/menu.rs
+++ b/sdk/lib/rust/win32k-rust/src/menu.rs
@@ IntDestroyMenuObject_impl
-      if PsGetCurrentProcessSessionId() == (*(*(*Menu).head.rpdesk).rpwinstaParent).dwSessionId
+      if PsGetCurrentProcessSessionId() == (*(*Menu).head.rpdesk).dwSessionId
```
```diff
--- a/sdk/lib/rust/win32k-rust/src/winsta.rs
+++ b/sdk/lib/rust/win32k-rust/src/winsta.rs
@@ IntCreateWindowStation_impl
     WindowStation = WindowStationPtr as PWINSTATION_OBJECT;
-
     RtlZeroMemory(
         WindowStation as *mut core::ffi::c_void,
         core::mem::size_of::<WINSTATION_OBJECT>(),
     );
-
+    (*WindowStation).dwSessionId = (*NtCurrentPeb()).SessionId;
     InitializeListHead(core::ptr::addr_of_mut!((*WindowStation).DesktopListHead));
-    (*WindowStation).dwSessionId = (*NtCurrentPeb()).SessionId;
     Status = RtlCreateAtomTable(37, ...);
```
(Left `NtCurrentPeb()->SessionId` as-is rather than also porting `1968202840d`'s
`PsGetCurrentProcessSessionId()` swap — that part is INCONCLUSIVE, see above; don't conflate the
two shas in one patch.)

**Ordering note for the person applying this**: `RtlZeroMemory` must run before the
`dwSessionId` write in every case (zeroing after would clobber it) — verify the diff context
lines above match current file state before applying, this file gets touched by other lanes.

### 2. `d09a3600606` — no full-frame NC repaint on frame-geometry change (winpos.c)

Upstream, in `co_WinPosSetWindowPos` (right after the existing `SWP_STATECHANGED` block that
sets `WNDS_SENDNCPAINT`):
```c
/*
 * Request a full non-client repaint when the frame geometry changes.
 * Otherwise, copied bits can retain parts of the old frame. CORE-20769.
 */
if ((Window->style & WS_VISIBLE) &&
    ((WinPos.flags & SWP_FRAMECHANGED) ||
     (OldWindowRect.right - OldWindowRect.left) != (NewWindowRect.right - NewWindowRect.left) ||
     (OldWindowRect.bottom - OldWindowRect.top) != (NewWindowRect.bottom - NewWindowRect.top)))
{
   Window->state |= WNDS_SENDNCPAINT;
   Window->state2 |= WNDS2_FORCEFULLNCPAINTCLIPRGN;
}
```
This is explicitly a fix for a regression **introduced by `b38db9dd238`** (wave 1's CONFIRMED
finding #11, the `IntGetNCUpdateRgn` clip-to-`hrgnUpdate` fix) — but note **`b38db9dd238` has
not been applied to our Rust yet** (verified: `painting.rs:469` still reads
`IntGdiGetRgnBox(hRgnNonClient, ...)`, the pre-fix shape, not `(*Window).hrgnUpdate`). The block
being ported here is independently correct regardless of whether `b38db9dd238` has landed — it
forces a full NC repaint on any actual frame-size change, which is a superset-safe behavior — so
it does not depend on that other finding landing first, but **if both land, land `b38db9dd238`
first** since this commit's own stated purpose is to compensate for exactly that tightening.

Our Rust `co_WinPosSetWindowPos` (`winpos.rs:2320-2324`) has the `SWP_STATECHANGED` block and
stops there — the frame-geometry-change block is entirely absent:
```rust
if ((*Window).style & WS_VISIBLE) != 0
    && ((WinPos.flags & SWP_STATECHANGED) != 0
        || (((*Window).state2 & WNDS2_WIN31COMPAT) == 0 && (WinPos.flags & SWP_NOREDRAW) != 0))
{
    (*Window).state |= WNDS_SENDNCPAINT;
}
// <-- upstream's new block belongs here, missing entirely
```
The consuming side already exists and is live: `painting.rs:539-540` already checks and clears
`WNDS2_FORCEFULLNCPAINTCLIPRGN` (`(*pWnd).state2 & WNDS2_FORCEFULLNCPAINTCLIPRGN != 0`) — so the
flag is read but **nothing in the tree ever sets it**, meaning that code path is currently dead.
Both `WNDS_SENDNCPAINT` and `WNDS2_FORCEFULLNCPAINTCLIPRGN` constants are already bindgen'd
(`win32k-sys/src/bindgen_{i386,amd64,arm64}.rs`) and imported in `winpos.rs` already (used a few
lines above), so this is a drop-in addition, no new `use`.

Proposed diff:
```diff
--- a/sdk/lib/rust/win32k-rust/src/winpos.rs
+++ b/sdk/lib/rust/win32k-rust/src/winpos.rs
@@ co_WinPosSetWindowPos_impl (right after the SWP_STATECHANGED WNDS_SENDNCPAINT block)
     if ((*Window).style & WS_VISIBLE) != 0
         && ((WinPos.flags & SWP_STATECHANGED) != 0
             || (((*Window).state2 & WNDS2_WIN31COMPAT) == 0 && (WinPos.flags & SWP_NOREDRAW) != 0))
     {
         (*Window).state |= WNDS_SENDNCPAINT;
     }
 
+    /* Request a full non-client repaint when the frame geometry changes.
+     * Otherwise, copied bits can retain parts of the old frame. CORE-20769. */
+    if ((*Window).style & WS_VISIBLE) != 0
+        && ((WinPos.flags & SWP_FRAMECHANGED) != 0
+            || (OldWindowRect.right - OldWindowRect.left) != (NewWindowRect.right - NewWindowRect.left)
+            || (OldWindowRect.bottom - OldWindowRect.top) != (NewWindowRect.bottom - NewWindowRect.top))
+    {
+        (*Window).state |= WNDS_SENDNCPAINT;
+        (*Window).state2 |= WNDS2_FORCEFULLNCPAINTCLIPRGN;
+    }
+
```
Verify `OldWindowRect`/`NewWindowRect` are the exact local variable names in scope at this point
in `winpos.rs` (they are referenced a few lines below this insertion point in the existing
`hrgnUpdate` offset block, so they should already be live `RECT`/`RECTL` locals — confirm the
field is `.right`/`.left` not a different rect accessor before applying).

---

## Detail (REFUTED, worth recording so nobody re-opens these)

- **`aaab5f01981` (dclife.c)**: upstream replaces a kernel-mode call to the `NtGdi` syscall stub
  (`NtGdiCreateCompatibleDC`) with the internal `GreCreateCompatibleDC`, because calling a
  syscall entry point directly from kernel code is fragile (mode/probe assumptions). In our
  Rust, `NtGdiCreateCompatibleDC` (`dclife.rs:1179`) is already a **bare one-line passthrough**
  (`GreCreateCompatibleDC(hdc, 0)`) with no probing, SEH, or previous-mode logic — so the two
  call paths in our translation are behaviorally identical today. Porting this literally would
  be a no-op; not applying it.
- **`9d287f3c5aa` (window.c)**: removes a decade-old "HACK: Helper for win32csr/desktopbg.c" in
  `IntCreateWindow` that auto-assigned a desktop's first window as `DesktopWindow` — made
  redundant once `IntCreateDesktop()` started doing the same assignment itself (a separate,
  earlier change, already reflected in our Rust's `IntCreateDesktop_impl`). This is dead-code
  removal, not a correctness fix; whether or not our Rust still carries the old hack in
  `IntCreateWindow`, it's provably harmless (both paths write the same value) and out of scope
  for a bug sweep.
- **`eb9a73c9755` (ime.c)**: verified bit-identical before/after (`lParam & 0x80000000` ==
  `HIWORD(lParam) & KF_UP` since `KF_UP == 0x8000`; same for `KF_ALTDOWN`). Readability-only.
- **`2b8f2020d70`, `5ab1ff188c3`**: read in full, confirmed comment/whitespace-only diffs across
  all touched files (`--stat` shows deletions only, or symmetric reformatting with no `+`/`-`
  pairs that change a token).

---

## Not reached

**Nothing from wave 2's explicit worklist remains unopened.** All ~18 files
(`engmisc.c`, `kbdlayout.c`, `display.c`, `hotkey.c`, `defwnd.c`, `main.c`, `misc.c`, `ntuser.c`,
`object.c`, `callback.c`, `ntstubs.c`, `winsta.c`'s 5 commits, `stubs.c`) got a real `git show`
this pass.

**Newly surfaced by the full 179-file re-derivation, also now opened:** `error.c`, `dclife.c`,
`keyboard.c`, `window.c`, `winpos.c`.

**Still open, not pursued (feature work, consistent with wave 1/2's stance — flagging again in
case priorities change):** `ime.c`'s `215d409f52b`/`61fb4db34c6`/`80bd4608363` (IMM32 feature
work), `input.c`'s `36ffee8ea38`/`a70008c4379`/`88cffd7b660` (raw input device list + idle-tick
feature), `keyboard.c`'s `e22f4474673`/`cac013abef3` (Alt+Numpad / hex numpad input methods),
`main.c`+`object.c`+`ntstubs.c`'s `b272427baa3`/`c864b270e06` (Job UI Restriction, raw input —
same feature family), `winsta.c`+`desktop.c`'s `7e08891a9f3` (desktop-switch-by-event feature).
None of these are bug fixes in code that already exists in our Rust; they add new OS surface
that was never translated (correctly out of scope for "find the bug the C fix closed").

**Full-49-file audit note:** the `git log --oneline $FP..upstream/master -- <file>` sweep across
all 179 deleted files (not just the wave 1/2 "49 with commits" claim) turned up exactly the same
file set as wave 1/2 already covered, **plus** the 5 new ones listed above — no other file in
the deleted tree has any upstream commit since `$FP` that wasn't already accounted for. This
closes the file-coverage question for this seam: every deleted C file with upstream activity
since the fork has now had at least one commit `git show`n and classified across the three
waves.
