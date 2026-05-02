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
