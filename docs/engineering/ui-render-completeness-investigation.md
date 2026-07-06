# UI Render-Completeness — the desktop reaches a *stable but incomplete* render

**Status:** OPEN investigation, opened 2026-07-05. Characterized + proven-not-timing;
root cause not yet found. Cross-arch (i386/amd64/arm64). Likely a shared win32k/user32
non-client-paint + tray child-paint gap.

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
