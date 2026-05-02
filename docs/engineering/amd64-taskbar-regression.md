# amd64 Taskbar/Shell UI Regression

## Problem

On amd64, the kernel does NOT successfully boot to a desktop. The "no taskbar"
symptom is the visible end of a deeper kernel-level failure: explorer.exe is
never reached, and on a fraction of boots the kernel BUGCHECKs.

**i386 renders the full desktop correctly** (Start button, taskbar, clock).
ARM64 has separate boot path issues.

## Failure Distribution (commit c1f9350f, May 2 2026)

Across 10+ amd64 boots:
- **~10%**: full 7/7 boot — `[EXPLORER] _tWinMain entered` fires, taskbar may or may not render.
- **~70%**: 6/7 — boot reaches `[WIN32K] shell ready` but **winlogon hangs** between "Switching to Winlogon desktop" (wlx.c:1154) and the next winlogon log line. `[EXPLORER] _tWinMain entered` never fires (explorer is never launched).
- **~20%**: BUGCHECK 0x1E (KMODE_EXCEPTION_NOT_HANDLED) inside `KiUnlinkThread`/`KiUnwaitThread`, plus `KeGetCurrentIrql() <= APC_LEVEL` MM asserts.

i386 is rock solid (5/5 reliable in same probe).

## Failure Trace (BUGCHECK Variant)

```
[WINLOGON] CreateWindowStationW(WinSta0)
(winsta.c:661) err: Initializing input window station
*** ASSERT FAILED: KeGetCurrentIrql() <= APC_LEVEL at miarm.h:1206
*** ASSERT FAILED: KeGetCurrentIrql() <= APC_LEVEL at miarm.h:1283
*** BUGCHECK: 0x0000001E (0xC0000006, 0xFFFFF800004XXXX, <fault VA>, 0)
```

- `0xC0000006` = `STATUS_IN_PAGE_ERROR`.
- PC = `KiUnwaitThread + 0x20` (the inlined `KiUnlinkThreadImpl`).
- The faulting instruction is `movq (%rax), %rcx` where `%rax = Thread->WaitBlockList`.

### What the bad Thread looks like (instrumented capture)

When `KiUnlinkThreadImpl` is hit with the bad value, `Thread` points at a pool
address (`FFFFFA80015C92F0` in one capture) that **is not a valid KTHREAD**:

```
[KIUNLINK] BAD Thread=FFFFFA80015C92F0 WBL=0x1 expected_WB0=FFFFFA80015C93E8 WaitStatus=0x1 State=160
[KIUNLINK] KT[0x80..0xa0]: 0x1 0x1 0x00000C0D00000000 0x000000053A000000
```

Notes:
- `State=160` (0xA0) — not a valid KTHREAD `State` enum value (max ~8).
- `WaitBlockList=1`, `WaitStatus=1` — neither field looks like a real value.
- `expected_WB0` (Thread+0xF8) is where `WaitBlockList` should point if Thread were valid.

So the kernel call path is dispatching `KiUnwaitThread` on **a non-KTHREAD pool block**.
`Thread` came from `WaitBlock->Thread` in `KiWaitTest`, which came from
`Object->WaitListHead.Flink` of the event being signaled. This means a stale or
corrupted KWAIT_BLOCK is in some object's wait list, pointing at a freed/reused
KTHREAD slot.

### Likely culprit chain

1. A thread terminates without removing its KWAIT_BLOCK from some object's wait list.
2. The KTHREAD memory is reused (by another pool allocation).
3. Later, that object is signaled. `KiWaitTest` reads `WaitBlock->Thread` (now garbage), passes it to `KiUnwaitThread`, fault.

Where to investigate: `KeRundownThread` / thread-exit cleanup paths, and any
place where a thread can be unblocked from a wait without going through the
proper unwait mechanism.

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
   For root-causing the stale KWAIT_BLOCK origin, add asserts in
   `KeRundownThread` / wait-list cleanup that verify all of a thread's
   KWAIT_BLOCKs are unlinked before the KTHREAD memory is freed; also
   instrument pool-free for KTHREAD-sized blocks. With breakpoints in those
   paths plus `dump-thread`, the bad-thread origin should fall out within a
   few boots.

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
