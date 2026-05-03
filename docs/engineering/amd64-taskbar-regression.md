# amd64 Taskbar/Shell UI Regression

## Problem

On amd64, the kernel does NOT successfully boot to a desktop. The "no taskbar"
symptom is the visible end of a deeper kernel-level failure: explorer.exe is
never reached, and on a fraction of boots the kernel BUGCHECKs.

**i386 renders the full desktop correctly** (Start button, taskbar, clock).
ARM64 has separate boot path issues.

## Status (2026-05-02 — late session)

**amd64 7/7 smoke tests PASS in 130s.** Three fixes shipped this session:

1. **BUGCHECK 0x1E variant: FIXED** in commit `4ec06ba4`. Same root cause as
   below (missed `WaitBlock->Thread = Thread`), but the actual hot symbol on
   amd64 was the C shim `KxMultiThreadWait` in
   `ntoskrnl/ke/ke_rust_shims.c:985-991`, not the Rust inline at
   `sdk/lib/rust/ntkrnl-sys/src/inline/ke.rs` (which was dead code — the
   linker resolves to the C shim). The earlier afba7bd7 fix landed in the
   wrong file.

2. **conftest clean-build false-passes: FIXED** in same commit. The smoke
   harness was nuking `build-{arch}/sdk/lib/rust/rust-kernel-target/` but the
   actual cargo target dir is `build-{arch}/ntoskrnl/rust-kernel-target/`.
   Stale Rust artifacts went undetected for an unknown number of sessions.

3. **BalanceSet 40% CPU spin: FIXED** in commit `fe4ffb36`.
   `KeBalanceSetManager` (`sdk/lib/rust/ntoskrnl-rust/src/ke/balmgr.rs:164`)
   initialized PeriodTimer with `NotificationTimer (0)` instead of
   `SynchronizationTimer (1)` from the C original. NotificationTimer stays
   signaled forever; the wait loop returned immediately every iteration,
   burning ~40% of one CPU. amd64 boots dropped from 15+ minutes (or hang)
   to ~130s. Found via backtrace sampling — KxMultiThreadWait+KeRaiseIrql
   showed up in 3/4 samples.

**Hang variant: also resolved.** The earlier "winlogon hangs after Switching
to Winlogon desktop" was actually the BalanceSet spin starving the kernel.
With that fixed, winlogon and explorer launch normally.

**Remaining open: amd64 taskbar still doesn't render** (blue desktop only).
Explorer enters `_tWinMain` and continues (3000+ lines of post-explorer
serial), but the screen stays blue with no Start button or clock.

### Status (2026-05-02 evening — narrowed-down hang)

Added per-step `OutputDebugStringA` markers to `StartWithDesktop` →
`CTrayWindow::Open` → `OnCreate` → `CTrayNotifyWnd::OnCreate` →
`CTrayClockWnd::Initialize`/`OnCreate`/`ResetTime`. **Hang point:**
`CTrayClockWnd::CalculateDueTime` — specifically the call to
`GetLocalTime(&LocalTime)` in `dll/win32/kernel32/client/time.c:286`.

`GetLocalTime` reads `SharedUserData->TimeZoneBias` in a do-while that
spins until `High1Time == High2Time`. Inspecting the live amd64 kernel
at `0xFFFFF78000000020` (kernel-side mapping of KUSER_SHARED_DATA):

```
0x14 SystemTime:    LowPart=0x2c6c12fd  High1=0       High2=0       (consistent, but never updates)
0x08 InterruptTime: LowPart=0x2b6c13fd  High1=0x10101 High2=0x10101 (consistent but garbage)
0x20 TimeZoneBias:  LowPart=0x1         High1=0x01010101 High2=0x101 (INCONSISTENT — TimeZoneBias.High1 != High2)
```

The kernel-side KUSER_SHARED_DATA page contains uninitialized garbage on
amd64. `TimeZoneBias.High1 != TimeZoneBias.High2` causes GetLocalTime to
spin forever — that's what blocks `CTrayClockWnd::OnCreate` and
therefore the entire taskbar.

**Bypassing IsThreadSuspended (FreezeCount=1 leak) does NOT fix the
hang** — confirmed by hard-coding `IsThreadSuspended` to return FALSE.
The FreezeCount leak (csrss main thread is reported frozen on both i386
and amd64; doc above) is a real but separate bug.

**Likely root cause of taskbar hang:** the SharedUserData mapping setup
in `mm/mminit.rs` (`MiCreateArm3StaticMemoryArea(KI_USER_SHARED_DATA)`
at line 188 + the PFN mapping at line 263-281) maps the page but never
zeroes it. On i386 the page might be zero-initialized by free-page
zeroing; on amd64 the freeldr mapping path may differ. Check whether
amd64's freeldr/MM init actually puts a zero page at
`KI_USER_SHARED_DATA = 0xFFFFF780_00000000`, or whether it's just
mapped over arbitrary RAM contents.

Investigation paths:
- ~~Add `RtlZeroMemory((PVOID)KI_USER_SHARED_DATA, PAGE_SIZE)` early in
  `MmArmInitSystem`~~ — **tried this 2026-05-02**, both before and right
  after the PTE setup at line 281. Either way the write triggers a page
  fault that the WS-lock-held context can't service, producing
  `MmAccessFault with OwnsExcl=1!` and a cascade of WS-lock asserts.
  The kernel-side mapping VA (0xFFFFF780_00000000) is established but
  the page isn't faulted in yet, and the early init paths can't tolerate
  the fault. Need a later init point (post MM init complete, IRQL <
  APC_LEVEL, no WS lock) — possibly in `Phase1Initialization` before any
  user-mode runs — or fault the page in explicitly before writing.
- Audit `ExpInitializeTimeZoneInformation` / `ExRefreshTimeZoneInformation`
  call order during boot — verify it actually runs and writes
  TimeZoneBias on amd64.
- Compare i386 vs amd64 KUSER_SHARED_DATA bytes at boot — same kernel
  init path should produce same bytes.
- The FreezeCount=1 csrss thread is a separate bug (likely a
  KeFreezeAllThreads/KeThawAllThreads count mismatch via DbgkpSuspendProcess)
  and exists on both i386 (4 spam) and amd64 (135 spam). Worth a
  dedicated session.

### Status (2026-05-03 late — third blocker localized to NtOpenKey called from inside EDWP commit)

**Continued bisection past EDWP into the recursive WM dispatch chain.** Earlier I
suspected `IntEndDeferWindowPosEx`'s loop itself; that was wrong. The real wedge
is one frame deeper, in user-mode → kernel-mode reentrancy.

Path to the wedge (instrumented with `OutputDebugStringA` markers in
`base/shell/explorer/{traywnd.cpp, trayntfy.cpp, traydeskbtn.cpp}` and
`dll/win32/{advapi32/reg/reg.c, uxtheme/system.c}`; markers stashed at
`stash@{0}: amd64 taskbar debug instrumentation`):

```
[TW] Open: before Create
[TW] OnCreate (full sequence completes — StartButton, TrayBandSite,
                CTrayNotifyWnd_CreateInstance, TNWM_GETSHOWDESKTOPBUTTON,
                UpdateFonts, InitShellServices, returning TRUE)
[TW] OnSize entered (post-WM_CREATE WM_SIZE — wParam=SIZE_RESTORED)
[TW] OnSize: RESTORED branch
[TW] OnSize: before/after ResizeWorkArea
   ↓ (ApplyClipping → SetWindowRgn → recursive WM dispatch)
[TW] OnWindowPosChanging entered/done
[TW] OnNcCalcSize entered
[TW] OnSize entered (recursive — non-RESTORED branch)
[TW] AlignControls entered
   ↓ (TNWM_GETMINIMUMSIZE inside parent's BeginDeferWindowPos batch)
[TNW] AlignControls (1st nested call — succeeds, all 3 SWPs commit)
[TW] AlignControls: before EndDeferWindowPos
   ↓ (EDWP iter 1 commits m_TrayNotify position → WM_SIZE → recursive)
[TNW] AlignControls (2nd nested call)
[TNW] AC: before WidthOrHeight
[BTN] WidthOrHeight: before IsThemeActive
[UXT] IsThemeActive: before RegOpenKeyW
[REG] RegOpenKeyExW: before MapDefaultKey / after MapDefaultKey
[REG] RegOpenKeyExW: before NtOpenKey
   ⚠️ HANG — no "after NtOpenKey" marker
```

The wedge is **`NtOpenKey` called from user-mode while win32k holds
`UserEnterExclusive` for an `EndDeferWindowPos` commit**. Earlier `NtOpenKey`
calls in this same explorer process succeed (`[REG] RegOpenKeyExW: after
NtOpenKey` fires N times before the hang). The bug only manifests when
`NtOpenKey` is reached *via the EDWP-commit recursive WM_SIZE handler* —
i.e., the kernel call stack is:

```
NtUserEndDeferWindowPosEx
  → IntEndDeferWindowPosEx (UserEnterExclusive held)
    → co_WinPosSetWindowPos (m_TrayNotify, iter 1 of 3)
      → SendMessage(WM_SIZE) to user-mode
        → CTrayNotifyWnd::OnSize → AlignControls → WidthOrHeight
          → IsThemeActive (uxtheme) → RegOpenKeyW (advapi32)
            → NtOpenKey  ⚠️ NEVER RETURNS
```

**Rules out:**
- The kernel is alive and idle, not bugchecked. `qemu/inspect.sh bt` shows
  CPU0 in HLT (sti/hlt at `0xfffff80000865701`) with stale `KeBugCheckWithTf`
  in the unwound trace — that's the previous frame, not a current bugcheck.
  `run-until.sh --break KeBugCheckEx --timeout 240` does NOT hit, confirming
  no real bugcheck.
- A workaround that hardcodes `IsThemeActive() = FALSE` in
  `CTrayShowDesktopButton::WidthOrHeight` does NOT fix the taskbar — the
  wedge just *moves* to the next IsThemeActive site (e.g., during
  `SetWindowPos(m_hwndShowDesktop)`'s theme query). Multiple call sites in
  the cascade hit registry; the deeper bug is in `NtOpenKey`, not in any
  one explorer caller.

**Hypothesis (most likely):** Cross-thread KCB-lock vs. win32k user-lock
deadlock specific to amd64 timing. Thread A (explorer GUI) holds win32k
`UserEnterExclusive` and wants a KCB; Thread B (Themes service? Service
Control Manager? something doing notifications across processes) holds the
KCB and is parked waiting on either the user lock or some cross-process
ALPC. The same code path is hit on i386 with no wedge, so it's a timing
window or amd64-specific lock-ordering issue.

**Smoke status:** unchanged — 7/7 PASS in 108.68s. Smoke saves snapshot at
"shell ready" (kernel-side marker, fires before explorer wedges).

**Heisenbug warning:** Adding kernel-side `DbgPrint` markers in
`NtOpenKey` is enough to PERTURB the wedge — with kernel-side markers
in place, `NtOpenKey` returns successfully and the wedge moves to
`ResizeWorkArea` (i.e., `SystemParametersInfoW(SPI_SETWORKAREA, ...,
SPIF_SENDCHANGE)` — synchronous WM_SETTINGCHANGE broadcast).

This means the underlying bug is a **timing-sensitive synchronization
issue specific to amd64**, not a single deterministic call site. Sites
observed wedging across runs (with varying instrumentation):

- `NtOpenKey` from inside EDWP commit (registry parse / KCB)
- `SystemParametersInfoW(SPI_SETWORKAREA, SPIF_SENDCHANGE)` (window
  message broadcast through `co_HwndBroadcastMessage`)
- `SetWindowRgn` inside `ApplyClipping` (window region update)

Common pattern: **synchronous cross-thread or cross-process call from a
thread holding a win32k user-lock**. Each "fix" displaces the wedge to
the next such site. User-mode workarounds confirmed band-aid only
(IsThemeActive hardcode → wedge moves to next registry caller).

**Next steps (in order of cost/value):**

1. **Non-disturbing observation.** Stop adding `DbgPrint` and
   `OutputDebugStringA` markers — they perturb timing and move the
   wedge. Use `qemu/start.sh amd64 --gdb` and inspect via `info
   threads` / kernel ETHREAD walking. To find Thread B, walk the kernel
   process/thread lists from `PsActiveProcessHead` and dump each
   thread's stack — look for one stuck on a win32k lock or KCB lock with
   a callstack that crosses subsystems.
2. **Compare i386 vs amd64.** On i386, the same code path runs with no
   wedge. Diff: thread scheduling, lock acquisition primitives,
   FreezeCount/SuspendCount semantics, IRQL handling. A few candidates
   that already exist as differences:
    - `KeFreezeAllThreads` / `KeThawAllThreads` mismatch on csrss main
      thread (see `feedback_kuser_shared_data_garbage` — high frequency on
      amd64 vs. i386). The `Sending to Suspended Thread Msg 121` log
      lines (WM_GETHOTKEY) accumulate during the wedge — possibly a
      symptom of this leak. Note: bypassing `IsThreadSuspended` did NOT
      fix the taskbar in earlier testing, so FreezeCount is not THE
      cause but may be A contributor.
    - amd64-specific `co_IntSendMessageWithCallBack` or `xxxSendMessageEx`
      timeouts. Look for SendMessageTimeout / SMTO_BLOCK paths that
      might behave differently on amd64.
3. **Real fix** is in the kernel. User-mode workarounds will not stick.

### Status (2026-05-03 evening — second blocker localized to win32k EndDeferWindowPos) [SUPERSEDED]

**Bisected via OutputDebugStringA markers in CTrayWindow::Open / OnCreate /
AlignControls and DbgPrint markers in win32k IntEndDeferWindowPosEx.** The
wedge is reproducible in plain `qemu/start.sh amd64` (no smoke-test fixtures).

What we see in serial.log right after explorer's _tWinMain:

```
[TRAY] OnCreate returning TRUE
[TRAY] OnSize entered                     <- co_WinPosSendSizeMove
[TRAY] OnSize entered                     <- second WM_SIZE
[TRAY] AlignControls entered
[TRAY] AlignControls: StartButton.UpdateSize done
[TRAY] AlignControls: before TNWM_GETMINIMUMSIZE
[TRAY] AlignControls: TNWM_GETMINIMUMSIZE done
[TRAY] AlignControls: before EndDeferWindowPos
[EDWP] hdwp=...0040104 ccvr=3 bAsync=0
[EDWP] iter 0 hwnd=...0200BE flags=0x14 bAsync=0   <- StartButton
[EDWP] iter 0 done res=1
[EDWP] iter 1 hwnd=...0300EA flags=0x14 bAsync=0   <- m_TrayNotify (CTrayNotifyWnd)
                                                       *** WEDGES HERE ***
```

So `co_WinPosSetWindowPos(m_TrayNotify, x, y, cx, cy, SWP_NOZORDER|SWP_NOACTIVATE)`
never returns. Flags 0x14 = SWP_NOZORDER (0x4) | SWP_NOACTIVATE (0x10). The
batch contains 3 entries (StartButton, TrayNotify, Rebar) — only StartButton
finishes. TrayNotify is the second iteration, and its
co_WinPosSetWindowPos is what hangs.

When repositioning m_TrayNotify, win32k sends WM_WINDOWPOSCHANGING /
WM_NCCALCSIZE / WM_SIZE / WM_MOVE / WM_WINDOWPOSCHANGED. CTrayNotifyWnd's
WM_SIZE handler calls `Size()` → `AlignControls()` which calls
`::SetWindowPos(m_hwndShowDesktop, ...)` then ditto for clock and pager —
each of which is a *recursive* SetWindowPos that re-enters
co_WinPosSetWindowPos.

Hypothesis paths to investigate next:
1. CTrayNotifyWnd::AlignControls calls SetWindowPos on m_hwndShowDesktop /
   m_hwndClock / m_hwndPager. Add markers there to see how deep the nest
   goes before wedging.
2. If wedge is inside win32k VisRgn calculations or paint synchronization,
   dropping SWP_FRAMECHANGED handling (or skipping invisible windows) might
   help. Look at `co_WinPosDoWinPosChanging`, `co_WinPosFixupOffset`, and
   the paint cleanup at the bottom of `co_WinPosSetWindowPos`.
3. Cross-thread SendMessage in this path: m_TrayNotify is owned by the
   same thread as the tray, but if any of the children (Clock/Pager) ends
   up cross-thread, SendMessage could block indefinitely on amd64.
4. **Workaround candidate**: replace `BeginDeferWindowPos`/`EndDeferWindowPos`
   in `CTrayWindow::AlignControls` with direct `SetWindowPos` calls. This
   avoids the batched commit. If it unblocks the taskbar, the bug is in
   `IntEndDeferWindowPosEx`'s loop interacting with recursive child SWPs.

### Status (2026-05-03 — GetLocalTime hang fixed; taskbar still doesn't render)

**Fix shipped:** zero TimeZoneBias in `ExpInitializeExecutive` right after
`sti` (before `(*SharedUserData).CryptoExponent = 0`). One line:

```rust
(*SharedUserData).TimeZoneBias = core::mem::zeroed();
```

**Confirmed via debug-print bisection:**
- Pre-fix hang point: `CTrayClockWnd::CalculateDueTime` →
  `GetLocalTime(&LocalTime)` in `dll/win32/kernel32/client/time.c:286`,
  spinning on `do { ... } while (Time.High1 != Time.High2)` because
  amd64 freeldr leaves TimeZoneBias with `High1=0x01010101 != High2=0x101`.
- Post-fix: `[CLK] ResetTime: CalculateDueTime done` fires, then
  `[CLK] OnCreate: ResetTime done`, then `[CLK] Initialize done`, then
  `[TNW] OnCreate: pager GetWindow done`. The entire tray notification
  + clock + sys-pager construction completes.

**Earlier "BUGCHECK 0x7F" reading was wrong.** Several screenshots
showed a `*** STOP: 0x0000007F (0x08, 0, 0, 0)` paint on the framebuffer
in 640x480 text mode, which I misread as a real kernel bugcheck. It is
not — `qemu/run-until.sh --break KeBugCheckEx --timeout 180` never hits
the breakpoint, and QMP `query-status` returns `running`. The text on
screen is the cirrus VGA falling back to text mode and exposing whatever
was last written to the legacy 0xB8000 framebuffer (likely a FreeLdr or
early-INBV diagnostic paint that never gets cleared once cirrus enters
graphics mode). Whether that fallback happens is a screenshot-timing
race: smoke runs and 60s/180s probes that catch cirrus in graphics mode
show blue desktop; ones that catch the brief text-mode window show the
stale BSOD-styled bytes. The kernel itself is fine in both cases.

**Smoke status with fix:**
- amd64: 7/7 PASS in ~100s. Screenshot: blue desktop (no taskbar still).
- i386: 7/7 PASS in ~43s. Screenshot: full taskbar with Start + 12:00 AM.
- arm64: untested this session (still 0/7 baseline).

**Remaining: amd64 taskbar still doesn't paint** despite trayclock
construction completing. Explorer makes much more progress than before
(post-fix logs show CTrayClockWnd, CTrayNotifyWnd, CSysPagerWnd all
finishing OnCreate), but the desktop stays blue. There must be a second
blocker after tray construction:

- Possibly `CTrayWindow::ShowWindow` / `SetWindowPos(SWP_SHOWWINDOW)`
  hangs or no-ops. Look at `[TRAY] OnCreate: ...` markers between
  CSysPagerWnd done and the SetWindowPos call.
- The `(\win32ss\user\ntuser\winpos.c:796) err: FIXME: Parent is Desktop,
  Min off screen!` warning that fires shortly after pager init may be
  related — taskbar may be getting placed off-screen.
- `(\win32ss\user\ntuser\msgqueue.c:1096) err: Sending to Suspended
  Thread Msg 121` spam (FreezeCount=1 leak on csrss main thread, present
  on both arches but 135x more frequent on amd64) may also play a role.
  The taskbar paints via SendMessage; if the desktop thread is reported
  as suspended, paint messages get dropped. Worth re-testing the
  `IsThreadSuspended → FALSE` bypass with this fix in place to see if
  the taskbar appears.

Real fix for the underlying TimeZoneBias garbage is still to make
`RtlQueryTimeZoneInformation` succeed (so `ExRefreshTimeZoneInformation`
writes a real bias and we never need this hack). Audit the registry path
`SYSTEM\\CurrentControlSet\\Control\\TimeZoneInformation` setup on amd64.

## Failure Distribution (commit c1f9350f, May 2 2026 — pre-fix)

Across 10+ amd64 boots:
- **~10%**: full 7/7 boot — `[EXPLORER] _tWinMain entered` fires, taskbar may or may not render.
- **~70%**: 6/7 — boot reaches `[WIN32K] shell ready` but **winlogon hangs** between "Switching to Winlogon desktop" (wlx.c:1154) and the next winlogon log line. `[EXPLORER] _tWinMain entered` never fires (explorer is never launched).
- **~20%**: BUGCHECK 0x1E (KMODE_EXCEPTION_NOT_HANDLED) inside `KiUnlinkThread`/`KiUnwaitThread`, plus `KeGetCurrentIrql() <= APC_LEVEL` MM asserts. **(fixed — see Status above)**

i386 is rock solid (5/5 reliable in same probe).

## Failure Trace (BUGCHECK Variant)

```
*** BUGCHECK: 0x0000001E (0xFFFFFFFFC0000006, 0xFFFFF8000047E8D3, 0xFFFFF8000060F940, 0x0000000000000000)
```

- `0xC0000006` = `STATUS_IN_PAGE_ERROR`.
- PC = `KiUnwaitThread + 0x23` (NOT inlined `KiUnlinkThreadImpl` — that runs and returns first).
- Param3 (`0xFFFFF8000060F940`) is the **KTRAP_FRAME** for the fault.

### Actual fault site (gdb-toolkit capture, 2026-05-02)

Disassembly of `KiUnwaitThread` on amd64:

```
KiUnwaitThread:
  +0x00: push %rsi / push %rdi / sub $0x28,%rsp
  +0x09: mov %rcx,%rdi          ; rdi = Thread (saved)
  +0x0c: mov %rdi,%rcx
  +0x0f: call KiUnlinkThread    ; <-- runs, doesn't crash
  +0x14: mov %sil,0x77(%rdi)    ; Thread->AdjustIncrement
  +0x18: movb $0x3,0x76(%rdi)   ; Thread->AdjustReason = AdjustUnwait
  +0x1c: mov %rdi,%rcx          ; pass Thread to inlined KiReadyThread
  +0x1f: mov 0x68(%rcx),%rax    ; rax = Thread->ApcState.Process  (offset 0x68)
  +0x23: cmpb $0x0,0x96(%rax)   ; *** FAULT: rax = 0, faulting VA = 0x96 ***
```

Trap frame at fault time:

| Field | Value | Meaning |
|------|------|---------|
| Rax | `0x0` | `Thread->ApcState.Process` was NULL |
| Rcx | `0xFFFFFA80015C01F0` | the bogus "Thread" pointer |
| FaultAddress | `0x96` | the deref VA (0 + 0x96) |
| Rip | `0xFFFFF8000047E8D3` | KiUnwaitThread+0x23 |

### What the bad Thread actually IS

The bogus `0xFFFFFA80015C01F0` is **the body of an EPROCESS pool block**, not a
KTHREAD. Verified by:

- 16 bytes earlier (`0xFFFFFA80015C01B0`) we find a POOL_HEADER with tag
  `'corP'` (= "Proc" — the EPROCESS pool tag).
- `Header.Type = 0x03` = `ProcessObject` (NOT `ThreadObject = 0x06`).
- At offset `+0x260` from the body start, ASCII `csrss.exe` — this is the
  csrss.exe process EPROCESS.

`KiUnlinkThread` survived the call because its bounds check
(`WaitBlockList ∈ [Thread+0xF8, Thread+0x1B8)`) caught the bad value (which
was `0x1` here) and skipped the unlink loop. The crash happened in the
*inlined* `KiReadyThread` reading `Thread->ApcState.Process` at offset `+0x68`
— which on a KPROCESS is `VdmTrapcHandler` (typically NULL), giving us
`rax = 0`, then the next instruction faults on `[0x96]`.

### Revised culprit chain

Some wait-block setup path is putting an **EPROCESS pointer** into
`KWAIT_BLOCK.Thread`. Then when that wait list is processed by
`KiWaitTest`, the EPROCESS gets passed to `KiUnwaitThread` as if it were a
KTHREAD.

The previously hypothesized "stale KWAIT_BLOCK pointing at a freed KTHREAD
slot" is partially right (the pointer is non-thread) but more specific: the
target is consistently a **live EPROCESS** (csrss.exe in this capture). So
the bug is more likely a **type confusion** at wait-block setup than a
use-after-free.

Where to investigate:
- `KeWaitForSingleObject(processHandle, ...)` paths that set up the wait
  block — verify `WaitBlock.Thread = current_thread`, NOT `= Object`.
- Process-as-dispatcher-object signaling paths (`KeSetProcess`, process
  exit signaling) where wait blocks of waiters are populated.
- The Rust `KiSatisfyObjectWait`/`KiInsertObjectWaitList` translation
  layers — check if anything is swapping `Thread`/`Object` arguments.

The wait block at `0xFFFFFA800176ECC8` (the only entry in csrss EPROCESS's
`Header.WaitListHead` at inspection time) had `Thread = 0xFFFFFA800176EBD0`
(a real thread address) and `Object = 0xFFFFFA80015C01F0` (csrss). That
suggests the wait block being processed at fault time may have been a
*different* one, or the fields are correct but the asm path is wrong (i.e.,
something passed `&object` instead of `WaitBlock->Thread`).

## Failure Trace (Hang Variant — most common)

```
[WINLOGON] Switching to Winlogon desktop                    <- wlx.c:1154
(desktop.c:3401) err: ...didn't have an assigned startup desktop...
(desktop.c:3339) err: Attempted to change thread desktop although the thread has windows!
[WIN32K] shell ready                                        <- desktop.c:1644 (co_IntShowDesktop)
<HANG — no further serial output>
```

The next expected line on i386 is `[WINLOGON] SwitchDesktop OK` (wlx.c:1161),
which never prints. Therefore the `SetThreadDesktop`/`SwitchDesktop` syscall
sequence does not return to user mode, OR returns and immediately hangs.

`co_IntShowDesktop` returns `STATUS_SUCCESS` after `co_WinPosSetWindowPos`. The
hang is between that return and winlogon's user-mode resumption — a syscall
return path bug, possibly the user-mode KiCallback exit on amd64.

## What This Isn't

- **Not** the originally documented "explorer.exe started but didn't render the
  taskbar." Explorer is never launched. The previous version of this doc was
  written from screenshots without correlating to serial logs.

## Suggested Next Steps

The kernel debugging infrastructure (`docs/engineering/kernel-debugging-infra.md`)
is now the preferred entry point for both variants. The standard recipe:

```bash
RUN=$(qemu/start.sh amd64 --gdb)
qemu/wait-for.sh $RUN --pattern "ACPI Timer" --timeout 60
qemu/run-until.sh $RUN --break KeBugCheckEx --timeout 120 \
  --then "bt 30" --then decode-bugcheck
```

This triages the BUGCHECK variant in one tool call (validated against the
`KiUnwaitThread+0x23` fault site documented above). Per-variant guidance:

1. **Hang-variant**: set a breakpoint on `KiServiceExit2` or the user-mode
   return path on amd64. Use `qemu/run-until.sh ... --break KiServiceExit2
   --then "bt 20" --then regs` to capture state at the (non-)return.
   Alternatively keep instrumenting `usercall.rs` directly — the
   `dump-thread` helper makes after-the-fact inspection cheap.

2. **BUGCHECK-variant**: now reproducible end-to-end via the standard recipe.
   With the 2026-05-02 toolkit-driven analysis, root-causing should focus on
   where an EPROCESS pointer ends up in `KWAIT_BLOCK.Thread`. Concrete next
   steps:
   - Add an assertion at `KiUnwaitThread` entry: `Thread->Header.Type ==
     ThreadObject` (= 6). Bugcheck early with the bad pointer + caller saved
     so we can see who set up the bad wait block.
   - Audit Rust translations of `KeWaitForSingleObject`,
     `KiSatisfyObjectWait`, and any process-wait paths for
     `WaitBlock.Thread` vs `WaitBlock.Object` swaps.
   - Audit `KeSetProcess` and the EPROCESS exit signal path — these are
     plausible places where a process body could be passed where a thread
     is expected.

3. **WinDbg / kd64 protocol**: deferred indefinitely. The DWARF/PDB gap and
   Windows-host requirement eat most of the value; the GDB-based toolkit
   gives us symbol-aware inspection across all 3 arches with no extra deps.

## Reference Files

| File | Description |
|------|-------------|
| `tests/qemu/vm.py` | Smoke test boot markers |
| `win32ss/user/ntuser/winsta.c:661` | "Initializing input window station" — start of fault path |
| `win32ss/user/ntuser/desktop.c:1644` | `co_IntShowDesktop` — `[WIN32K] shell ready` print |
| `win32ss/user/ntuser/desktop.c:1572` | DesktopThread signals `gpDesktopThreadStartedEvent` |
| `base/system/winlogon/wlx.c:1154` | "Switching to Winlogon desktop" — last user-mode log before hang |
| `base/system/winlogon/wlx.c:1155` | `SetThreadDesktop` / `SwitchDesktop` syscall pair |
| `sdk/lib/rust/ntoskrnl-rust/src/ke/wait.rs:148` | `KiUnlinkThreadImpl` (where BUGCHECK happens) |
| `sdk/lib/rust/ntoskrnl-rust/src/ke/wait.rs:108` | `KiWaitTestImpl` — caller of `KiUnwaitThread` with bad Thread |
| `sdk/lib/rust/ntoskrnl-rust/src/ke/amd64/usercall.rs` | User-mode return path (suspect for hang variant) |
