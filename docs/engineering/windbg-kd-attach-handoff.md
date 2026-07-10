# Handoff: WinDbg live-attach corruption on the KD-armed boot (amd64)

**Status:** The KD-armed boot is now *safe* (reaches desktop with no debugger),
and the packet transport is protocol-correct. The remaining blocker is a
**processor/stack-state corruption in the heavy KD break-in path** that faults
the target when a debugger actually attaches. This doc is a self-contained brief
to pin and fix that corruption.

## What already works (landed on branch `crash-value`)

- **Armed no-debugger boot → desktop.** `--tripwire "KD=COM2"` with the COM2
  wire backed but no kd listening boots to `[WIN32K] shell ready` +
  `SwitchDesktop OK`, no BUGCHECK. (Was: BUGCHECK 0x1E flood.)
- **Transport protocol fixes** (kd64 `kdapi.rs`, kdcom-serial `kddll.rs`):
  - `KdpSendWaitContinue` sends the state change ONCE then loops on receive
    (was re-sending per serviced request → WinDbg desync = the `error 0n56`).
  - `KdSendPacket` fast-bails (1 attempt) until a debugger has ever acked
    (`KdpWireEverAcked`); bounded `KdpWireStallCycles` for vanished debuggers.
  - `KdpReportExceptionStateChange` returns FALSE when the send bailed.
- **`KdpWireTransportArm` sets `KdDebuggerNotPresent = 1` on arm** so the heavy
  paths self-skip until a real debugger clears the flag. **This is what makes
  the no-debugger boot safe — it does NOT fix the attach (see below).**

## The remaining bug

When a debugger is present (or a break-in triggers), the target enters the heavy
KD path: `KdpTrap`/`KdpReport`/`KdpSymbol` → `KdEnterDebugger` →
`KeFreezeExecution` + `KiSaveProcessorControlState` → state-change send →
`KiRestoreProcessorControlState`. After this returns, the **`ret` of
`DebugService2` faults with `0xC0000005` (AV)**, cascading into a recursive
AV↔breakpoint bugcheck flood ("crash reporter crashes").

### Evidence (amd64, kbase `0xfffff80000400000`)

- BUGCHECK 0x1E, P1=`0xffffffffc0000005`, **P2=`0xfffff800004b696a` =
  `DebugService2+6` (the `ret`)**. Second-chance: `0xC0000006` in
  `MmArmAccessFault ← MmAccessFault ← DebugService+0x13` (the bugcheck's own
  DbgPrint re-faulting).
- A single-letter debugcon wire trace (since removed) showed: the first
  load-symbols `KdpSymbol` runs to completion with **stable rsp
  (`0x…6027c8`)**, then the flood begins — i.e. the corruption manifests at the
  `ret` *after* `KdpSymbol` returns and `KdpTrap` advances the PC to the `ret`.
- PC advance is CORRECT (verified against C): int 2Dh pushes `+5` (the int3);
  `KiDebugServiceTrap` `inc`s to `+6`; `KiDispatchException` does `Rip -= 1`
  → `+5`; `KdpTrap` load-symbols adds `KD_BREAKPOINT_SIZE(1)` → `+6` (the ret).
  Not a double-advance. The `ret` address is right; it faults reading `[rsp]`,
  which points at CR3/stack-state corruption.

### Ruled out

- `KiSaveProcessorControlState` / `KiRestoreProcessorControlState`
  (`ke/amd64/cpu.rs`): **faithful match to `ntoskrnl/ke/amd64/cpu.c`** — CR0-8,
  DR, GDTR/IDTR (`&Gdtr.Limit` idiom with the KDESCRIPTOR `Pad[3]` is correct),
  MXCSR, MSRs all symmetric. Not the corruptor.
- The PC-advance double-skip theory (above).

### Prime suspects (unpinned)

1. **`KdEnterDebugger` → `KeFreezeExecution`** (`kd64 kdapi.rs:1727`,
   `ntoskrnl-rust ke/amd64/freeze.rs`). The owner-side UP freeze path. (Note: a
   sibling PRCB-cast bug in `KiSaveProcessorState`/`KiRestoreProcessorState` was
   already fixed here this pass — look for another in the freeze owner path.)
2. The **int-2Dh trap-frame ⇄ CONTEXT round trip** for this specific gate
   (`KeTrapFrameToContext`/`KeContextToTrapFrame`) writing a bad rsp back into
   the trap frame, so the post-`iret` `ret` uses a corrupt rsp.

## How to pin it

The corruption is in a well-scoped window: enter `KdEnterDebugger`, exit at the
`DebugService2` `ret`. Single-step the owner-side freeze on UP and diff
CR3/GDTR/rsp/trap-frame across `KeFreezeExecution` and across the
CONTEXT⇄trap-frame round trip.

**Repro (native KVM + gdb):**
```
VIBEOS_QEMU_EXTRA="-fw_cfg name=opt/org.vibeos.tripwire,string=KD=COM2 -serial file:/tmp/com2.bin" \
  RUN=$(bash qemu/start.sh amd64 --gdb | tail -1)
# gdb: connect, then PASS THE KNOWN KBASE so find-kbase does not rescan the
# crashing address space (this was the main gdb pain point this pass):
#   find-kbase 0xfffff80000400000
# break KeFreezeExecution / KdEnterDebugger / *0xfffff800004b696a (the ret),
# stepi across, dump cr3/rsp/gdtr and the trap frame.
```
Runtime addrs (this build; recompute per build via `llvm-nm` + `+0xfffff80000000000`):
`DebugService2 0xfffff800004b6964`, `KdpSymbol 0xfffff8000040cf80`,
`KdEnterDebugger` (nm `KdEnterDebugger`), `KeFreezeExecution` (nm),
`KiRestoreProcessorControlState 0xfffff800004ac710`.

**Repro (windows-host pipe + real WinDbg):** `scratchpad/kd-attach.sh <tag>`
launches `kdX64.exe` (WindowsApps PATH) at the tripwire marker against a
windows-host smoke boot. With the fixes, kd connects to the pipe but shows
`no_debuggee` (target stays passive by design until the heavy path is safe).

## Update (2026-07-10, second investigator) — attach WORKS; continuation is the blocker

The KD *session* now establishes: WinDbg attaches and `version`, `lm`, `k`, and
direct memory reads all succeed against the running target. So the transport +
initial handshake + state-manipulate reads are proven — **the compatibility
milestone (WinDbg speaks KD to our ntoskrnl) is substantially met.**

Remaining blocker is **debugger continuation** (resume after a break): the
**amd64 same-CPL `int 2Dh` machine-frame layout still produces an invalid
resumed context** — i.e. when KD resumes the target after servicing a break,
the reconstructed trap/machine frame for the same-privilege-level `int 2Dh` gate
is wrong, so the resumed context is corrupt. This is the same int-2Dh
trap-frame⇄context suspect flagged above, now narrowed to the **same-CPL
machine-frame layout on the resume path** specifically. Investigation touched
`sdk/include/asm/trapamd64.inc`, `kd-rust/src/kdio.rs`,
`kd64-rust/src/kdprint.rs`, `ntoskrnl-rust/src/ke/amd64/context.rs` (left
uncommitted on `crash-value`). Next: verify the same-CPL vs cross-CPL machine
frame (SS:RSP presence) on the `int 2Dh` iret path against what
`KeContextToTrapFrame` reconstructs.

## Definition of done

`kdX64.exe -k com:pipe,…` attaches, breaks in, and `version` / `lm` / `k` /
memory read succeed against the running target — then flip
`KdpWireTransportArm`'s `KdDebuggerNotPresent=1` guard OFF (or make it clear on
first real byte) so the target proactively announces to WinDbg at boot.

Full context: `docs/engineering/crash-value-design.md` § "Second pass
(2026-07-10)".
