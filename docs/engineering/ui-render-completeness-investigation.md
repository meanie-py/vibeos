# UI Render-Completeness — the desktop reaches a *stable but incomplete* render

**Status: ROOT CAUSE FOUND + FIXED 2026-07-05.** One shared root, exactly as predicted:
`win32k-rust/src/defwnd.rs` `IntDefWindowProc` had a `WM_CTLCOLOR =>` match arm whose
constant **did not exist** in win32k-sys (windowsx.h macro, 0x19, never bindgen'd) — in
Rust an unresolvable identifier in a match pattern silently becomes an **irrefutable
catch-all binding**, so every message after that arm in the match (WM_SETCURSOR,
WM_ERASEBKGND, WM_NCHITTEST, WM_PAINT, WM_SYNCPAINT, WM_WINDOWPOSCHANGING/CHANGED,
**WM_NCCALCSIZE, WM_NCACTIVATE, WM_NCPAINT**) was swallowed by the kernel DefWindowProc
and returned a garbage brush handle. WM_NCCALCSIZE never inset `rcClient`, so
`rcClient == rcWindow` for every window and win32k (correctly) skipped all NC painting;
the tray children starved on the same swallowed message tail. Fix: define the const
(`defwnd.rs`), value 0x0019. Post-fix amd64 renders the Start button + tray clock and
NC_DoNCPaint runs. Full defect-class writeup + oracle (`grep "matches any value"` on a
clean-build log) + same-day sibling instances (pnpmgr/devaction.rs, ke/balmgr.rs,
ke/bug.rs): field manual → "Match arm on an UNDEFINED constant".

Diagnosis method (for the record): serial-marker bisection of the WM_NCCALCSIZE
round-trip, one DbgPrint per hop across the kernel→user32→kernel boundary — plus five
parallel subagent semantic diffs of the translated paint chain vs the retired C
(painting/nonclient/winpos/defwnd/windc/vis), which cleared every layer and cornered
the bug in the one unaudited function, and found six adjacent real bugs (wrong DC_NC
constant ×3 files, i386 `co_WinPosGetMinMaxInfo` export hijacked by a stray attribute,
`IsChildVisible` null-parent inversion, snap bitwise-NOT, Win-key 2-bit keystate
indexing, `IntGetNCUpdateRgn` region leak) — all fixed in the same change.

Remaining follow-ups tracked at the bottom of this doc. The original investigation
brief follows unchanged.

---

**Why this is a big one:** it is the gap between *"boots to the desktop marker"* and
*"usable desktop."* Every top-level window comes up **chromeless** — no title bar, no
border, no min/max/close — so windows can't be moved, resized, or closed; and the
taskbar has **no Start button and no clock**, so the shell can't be driven. The smoke
North-Star marker (`shell ready`) fires and a screenshot is saved that *looks* desktop-ish,
so **the harness is currently blind to all of this** (green boots ship a half-painted
shell). Closing it is the next frontier after "boots to desktop," and because it
reproduces identically on all three arches it is almost certainly **one shared root** in
the win32k/user32 paint path — high leverage.

---

## TL;DR

After a full boot to `shell ready`, the desktop renders **client areas and top-level
window *backgrounds*, but not non-client chrome or taskbar child controls**:

| Renders ✅ | Does NOT render ❌ |
|---|---|
| Desktop wallpaper (solid fill) | **Window caption / title bar** (non-client) |
| Version watermark (GDI text on desktop) | **Window border / sizing frame** (non-client) |
| Taskbar *strip* (tray window background) | **Start button** (taskbar child) |
| Notepad **client area** (white) + edit control | **Tray clock** (taskbar child) |
| Notepad **status bar** ("Line 1, column 1 \| Windows (CR + LF) \| UTF-8") | Any taskbar band content |

The pattern is sharp: **client-area painting works; non-client (frame/caption) painting
does not; and the taskbar's own window paints but its child bands do not.**

---

## Evidence

### 1. It is NOT a screenshot-timing / "captured too early" artifact — PROVEN
The obvious first hypothesis (the smoke screenshots before the shell finishes painting)
is **disproven empirically**:

- The CLI smoke captures its screenshot **after** the desktop-settle window
  (`tests/qemu/cli/smoke.py:170`, `_screenshot()` runs after the settle block at
  153–168). Default arm64 settle is **15 s** (`Arm64Config.desktop_settle_seconds`,
  `tests/qemu/vm.py:280`).
- Re-ran the arm64 smoke with the settle bumped to **120 s** (8× longer). The resulting
  screenshot is **byte-for-byte identical** to the 15 s capture:
  ```
  8aac9a93053077cb94b53ee499dc0c53  qemu/arm64_smoke.png            (15 s settle)
  8aac9a93053077cb94b53ee499dc0c53  arm64_smoke_15s.png (120 s run) (identical md5)
  ```
- Conclusion: the render reaches a **stable steady state** well before 15 s and never
  advances. The missing elements are missing *permanently*, not *yet*.

### 2. The windows/controls ARE created — serial proves it
From the arm64 boot serial (`[EXPLORER]`/`[WINLOGON]` markers):
```
[EXPLORER] _tWinMain entered
[EXPLORER] taskbar: OnCreate begin
[EXPLORER] taskbar: child controls created      <-- start button / bands / clock created
[EXPLORER] taskbar: creating tray window
[WINLOGON] ARM64: notepad.exe created (PID 200, TID 204)   <-- notepad launched OK
[WINLOGON] explorer.exe created (PID 184, TID 188)
```
So this is **"created but not painted,"** not "failed to create." explorer builds the
tray window and its child controls; winlogon launches notepad; the screenshot shows the
tray strip and notepad client — but none of the chrome or tray children paint.

### 3. Cross-arch — all three show the same tray gap
- **i386** (`qemu/i386-uefi_smoke.png`): wallpaper + watermark + bare taskbar strip; **no
  Start button, no clock.** (x86 does not spawn notepad, so only the tray gap is visible.)
- **amd64** (`qemu/amd64-uefi_smoke.png`): identical to i386.
- **arm64** (`qemu/arm64_smoke.png`): same tray gap, **plus** a notepad window whose
  **client + status bar paint but whose caption/frame do not**.

Only arm64 currently spawns notepad (winlogon `#ifdef _M_ARM64`), which is why the
non-client gap is only *visible* on arm64 — but it is a win32k/user32 path shared by all
arches, so x86 windows would show the same missing chrome.

---

## Code-grounded analysis

### Notepad — a `WS_OVERLAPPEDWINDOW` window with no non-client area
`base/applications/notepad/main.c:603` creates the main window with
`WS_OVERLAPPEDWINDOW` = `WS_OVERLAPPED | WS_CAPTION | WS_SYSMENU | WS_THICKFRAME |
WS_MINIMIZEBOX | WS_MAXIMIZEBOX`. So it *requests* a title bar, system menu, sizing
border, and min/max/close buttons. On screen: the **client** renders (white background +
edit control + status bar child), but **none of the non-client chrome** does. The caption
paints on the first `WM_NCPAINT` at window-show time — essentially immediately — so a
30 s+ settle rules out timing (see Evidence #1).

**This localizes to the non-client paint path:** `WM_NCPAINT` / `WM_NCCALCSIZE` handling
in `DefWindowProcW` → `NC_DoNCPaint` / caption + frame drawing (win32k `ntuser` +
user32). The status bar (a *client-area child control*) paints, and the edit control
paints, so general `WM_PAINT` / `BeginPaint` / blitting works — it is specifically the
**non-client frame** that does not.

### Taskbar — tray window paints, child bands do not
`base/shell/explorer/explorer.cpp:212 CreateTrayWindow`; the tray builds its children
(`CTaskBand`, `CTrayNotifyWnd` → `CTrayClockWnd_CreateInstance` at `trayntfy.cpp:114`,
the Start button, `CTrayShowDesktopButton`, etc. — all ATL `CWindowImpl` windows). Serial
confirms "child controls created." On screen the tray **strip** (the tray window's own
background) paints, but the **band children** (Start button, clock, task band) do not.

Two sub-shapes possible, to be distinguished by instrumentation:
- **Same root as notepad** — if these bands rely on non-client paint or a shared
  `RedrawWindow`/invalidation path that is broken.
- **A tray-band-specific paint/visibility gap** — the ATL band windows are created but
  never get `WS_VISIBLE` / a `ShowWindow` / a `WM_PAINT`, or the deskband/rebar host
  never asks them to paint (COM band-site `IDeskBand::GetWindow`/`ShowDW` path).

---

## Hypotheses (ranked)

**H1 — Non-client painting is broken/stubbed (most likely, explains the most).**
`WM_NCPAINT` → `DefWindowProc` caption+frame draw is not producing pixels for any
top-level window. Single best explanation for the missing notepad caption **and** window
border **and** (if the bands inherit any NC) part of the tray gap. First suspects:
`DefWindowProcW` `WM_NCPAINT`/`WM_NCCALCSIZE`, `NC_DoNCPaint`/`NC_DoButton`, `DrawCaption`,
and whether a theme/uxtheme path is expected (classic vs themed NC).

**H2 — Tray child bands never paint (distinct or shared).** The ATL band windows are
created but not shown/invalidated/painted. Suspects: `WS_VISIBLE`/`ShowWindow` on the
band children, the rebar/band-site host paint propagation, `CTrayClockWnd`'s
`WM_PAINT`/`WM_TIMER`, the Start button's owner-draw/paint.

**H3 — A shared invalidation/`RedrawWindow` clipping issue** that spares client areas but
drops non-client + certain children. Less likely (client paint clearly works), but a
`RedrawWindow`/`RDW_FRAME`/update-region bug could present exactly this way.

The two observed gaps (top-level chrome; tray children) may be **one root** (H1 reaching
both) or **two** (H1 for chrome + H2 for bands). Instrumentation will tell.

**H4 — GUI threads on amd64/arm64 skip their client-thread-setup callback (fresh lead,
2026-07-05).** `co_IntClientThreadSetup()` — the per-GUI-thread `KeUserModeCallback(
USER32_CALLBACK_CLIENTTHREADSTARTUP)` that runs user32's client-side thread init — is
**called only on i386**; the call site at `win32k-rust/src/main.rs:673` is gated
`#[cfg(not(any(x86_64, aarch64)))]` (empty `skip_client_thread_setup` block on amd64/arm64).
The function itself is fully implemented on all arches — *only the call is skipped*. This
was found by the per-arch cfg-divergence audit (Tier 2) and is suggestive because **the skip
is on exactly the two arches with the render gap** and it's per-*GUI-thread* client init.
If user32's `ClientThreadSetup` primes any per-thread state the NC/band paint path depends
on, the missing callback would present as incomplete paint on amd64/arm64 only. Cheap to
test: drop the cfg gate (call it on all arches) and see if chrome/tray paint appears —
but note re-enabling adds a kernel→user round-trip per GUI thread and may have been skipped
because that path was fragile in bring-up, so gate ×3. See
`per-arch-cfg-divergence-audit.md` Tier 2. *(Ranking uncertain vs H1/H2 — verify the
`ClientThreadSetup` body's actual side-effects first; it may be a near-no-op in our user32,
in which case demote.)*

---

## Diagnostic plan — render milestones (the boot-marker philosophy, applied to UI)

Turn "the screenshot looks incomplete" into "each UI element self-reports whether it
painted," which both localizes the bug and gives the smoke a hard signal.

1. **Non-client paint probe.** Marker when `DefWindowProc`'s `WM_NCPAINT` path runs for a
   top-level window, and when `NC_DoNCPaint`/`DrawCaption` actually blits caption pixels.
   - Marker never fires → NC paint is not *invoked* (invalidation/`RDW_FRAME` issue).
   - Fires but no pixels → the caption/frame draw itself is a no-op/clipped/wrong-brush.
2. **Notepad "caption painted" vs "client painted" split.** The status bar paints and the
   caption does not — instrument both so the *difference* (client-child vs non-client) is
   attributable, not inferred from a screenshot.
3. **Tray child probe.** For the Start button + `CTrayClockWnd`: are they `WS_VISIBLE`? do
   they receive `WM_PAINT`? Add markers in their paint handlers.
4. **Wire into smoke as render-completeness assertions.** Emit
   `[NOTEPAD] window shown + painted` (after `UpdateWindow`, `main.c:631`),
   `[EXPLORER] start button shown`, `[EXPLORER] clock painted`, and have the smoke assert
   them. A missing marker localizes exactly which element failed — and guards the
   regression so a half-painted shell can never green again.

### Supporting change (requested, low-risk): notepad on all arches
Lift winlogon's notepad spawn out of `#ifdef _M_ARM64` (`base/system/winlogon/winlogon.c:850`)
so i386/amd64 also launch it (keep the arm64-only `cmd.exe` scaffold as-is). Gives a
uniform cross-arch GUI-app render proof and makes the non-client gap visible on x86 too,
where it can be debugged with the faster x86 toolkits.

---

## What this is NOT
- **Not a timing/settle artifact** (Evidence #1 — 15 s ≡ 120 s, byte-identical).
- **Not a create/launch failure** (Evidence #2 — windows + controls are created).
- **Not the win32k session wild-pointer 0x50 flake** (field-manual §9.17) — that is an
  intermittent crash; this is a deterministic, stable *under*-render with no fault.

## Cross-references
- Screenshots: `qemu/{i386-uefi,amd64-uefi,arm64}_smoke.png`.
- Smoke capture timing: `tests/qemu/cli/smoke.py:148-170`; settle config
  `tests/qemu/vm.py` (`desktop_settle_seconds` per arch).
- Tray construction: `base/shell/explorer/explorer.cpp`, `trayntfy.cpp`, `trayclock.cpp`.
- Notepad window: `base/applications/notepad/main.c:603,631`.
- win32k coverage history: `docs/engineering/archive/win32k-translation-audit.md`.

---

## Post-fix follow-ups (2026-07-05, owed — not silent deferrals)

1. **amd64/i386 notepad exit 0x80 — separate desktop-heap AV (notepad kept arm64-only).**
   Enabling notepad cross-arch (the brief's "supporting change") surfaced a *pre-existing*
   amd64/i386 bug unrelated to NC paint: notepad paints its caption then takes a user-mode
   `0xC0000005` at user32 `ValidateHandleNoErr+0xed` (`user32.dll` VA 0x…b79d) — the faulting
   read is `*(pci->pDeskInfo)` where `pci->pDeskInfo` is a bogus small value (~0x80010) in
   notepad's TEB `CLIENTINFO`, **even though** the kernel's `IntSetThreadDesktop` computed and
   wrote the correct user pointer (verified: `[DESKMAP] setdesk img=NOTEPAD.EXE kDeskInfo=…dd0
   delta=… uDeskInfo=0x15e0dd0` — a valid mapped VA). csrss then reaps it via
   `LPC_EXCEPTION → NtTerminateProcess(STATUS_ABANDONED=0x80)` (`smss-csrss-rust csrsrv/api.rs`).
   So between `IntSetThreadDesktop` and the `UpdateWindow`-triggered paint, notepad's
   user-side `CLIENTINFO.pDeskInfo` reads wrong — a desktop-heap/`CLIENTINFO`/`gSharedInfo`
   mapping issue specific to a *freshly-CreateProcess'd* GUI app on x86 (explorer/winlogon
   validate handles fine, so it is not universal). arm64 notepad is unaffected (ran fine
   pre-fix, renders full chrome now). **Notepad is therefore kept arm64-only** (winlogon-rust
   `#[cfg(target_arch = "aarch64")]`) until this AV is root-caused; the NC-paint fix is proven
   on x86 without it (Start button + clock + dialog chrome). Next step: trace whether the
   wrong value is written by the kernel on a *second* pass, or notepad's TEB `Win32ClientInfo`
   region differs from the one the kernel wrote (offset/section-mapping mismatch).
2. **Workspace `unreachable_patterns = "deny"` lint** — the mechanical tripwire for the
   catch-all-binding class. Size it against the clean-build ×3 warning sweep; if legit
   unreachable patterns are few, flip to deny + per-site `#[allow]`.
3. **Render-completeness smoke markers** (diagnostic-plan item 4): emit
   `[EXPLORER] start button painted` / `clock painted` / `[NOTEPAD] painted` and assert
   in smoke, so a half-painted shell can never green again.
4. **user32-rust DefWindowProc GoSS arm order** — while auditing, noted RealDefWindowProcW
   is faithful; no action. The five paint-chain diffs cleared painting/nonclient/winpos/
   defwnd/windc/vis (kernel) — reusable clearance record.
5. **Tray top-edge never painted → desktop-version "ghost" band (2026-07-07 forensics).**
   The doubled version string in every screenshot is NOT a double-draw: pixel forensics
   (640×480 smoke shots, all arches) show the tray window's rect starts at row 452 but its
   painted face starts at 455 — the 3px top band is the tray's NC frame (unthemed
   Shell_TrayWnd = WS_POPUP|WS_THICKFRAME|WS_BORDER) and it is NEVER painted, so the
   pre-taskbar full-screen `IntPaintDesktop` version text (drawn with workarea=480 before
   the tray's SPI_SETWORKAREA) shows through forever. `IntPaintDesktop` + its callers and
   `NC_DoNCPaint`'s frame path (UserHasWindowEdge → DrawEdge EDGE_RAISED) were audited
   faithful vs C — so the suspect is NCPAINT *delivery/timing* for the tray window
   specifically (e.g., its final SetWindowPos/frame-change not queuing a WM_NCPAINT, or
   the erase happening before the workarea move with no later invalidate of the frame
   band). Next step is a runtime trace: DbgPrint in explorer's OnNcPaint + win32k
   NC_DoNCPaint entry filtered to the tray hwnd, and check whether it ever runs post-
   final-position. Cosmetic but it's the last visible artifact on otherwise-clean
   desktop screenshots.

   **Runtime trace narrowed it (2026-07-07, i386, diagnostics since stripped):**
   the kernel DOES deliver WM_NCPAINT to the tray. Traced the full hop:
   `IntSendNCPaintImpl` sends `WM_NCPAINT` (0x85) to the tray hwnd (style
   `0x96840000` = the exact `WS_POPUP|WS_VISIBLE|WS_CLIPSIBLINGS|WS_CLIPCHILDREN|
   WS_THICKFRAME|WS_BORDER` tray creation style), and `User32CallWindowProcFromKernel`
   fires with `Msg=0x85`, `Proc=0x00402E70` (an explorer.exe .text address). BUT
   both a `[TRAYNC]` DbgPrint in `CTrayWindow`'s `BEGIN_MSG_MAP` prologue AND one in
   `OnNcPaint` stayed **completely silent** — the message reaches user32's
   kernel-callback but never enters CTrayWindow's ATL message map. Also: NO
   `IntSetWindowProc` ever ran for the tray hwnd, even though the tray clearly
   processes WM_SIZE/paint (taskbar face + Start + clock render). So the tray's
   window proc is a static explorer proc (`0x00402E70`), not the expected ATL
   thunk, and WM_NCPAINT dispatched to it is dropped before the map. **Next step:**
   identify `0x00402E70` (is it `CWindowImplBaseT::StartWindowProc`/`WindowProc`,
   or a class-registered default that swallows NCPAINT?) and why the ATL thunk
   subclass isn't installed/hit for the tray on this build. This is an explorer/
   ATL-side dispatch bug, not a win32k paint bug (win32k did its job).
