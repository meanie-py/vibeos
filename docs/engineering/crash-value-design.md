# Crash Value — design + build log (lane `crash-value`)

Mission (CRASH-VALUE-BRIEF.md): turn every crash into the richest, most
actionable report any OS emits; exception handling from hardware errors
working end-to-end to userland; panic handlers done right; WinDbg working.
Foundation: the DUMMY_PSEH verdict + `requires-uwtable=true` unwinder fix
(dummy-pseh-verdict.md, field manual §9.35).

## Layer 1 — panic-common: the one shared panic body

`sdk/lib/rust/panic-common` — a ZERO-DEPENDENCY leaf crate. ~150 crates'
`#[panic_handler] loop {}` stubs (silent hangs, PanicInfo discarded) are now
two-line forwarders:

```rust
#[panic_handler]
fn panic(info: &core::panic::PanicInfo) -> ! {
    panic_common::panic_common(env!("CARGO_PKG_NAME"), info)
}
```

The shared body: (1) formats `*** RUST PANIC in <crate> at <file>:<line>:<col>`
+ the panic message verbatim into a static 1KB buffer; (2) emits to debugcon
0xe9 (kernel x86) + `DbgPrint`; (3) appends a raw-PC backtrace via
`RtlWalkFrameChain` (message flushes FIRST — the walk is the §9.35-risk step);
(4) halts per the artifact's world. Nested/concurrent-panic safe (level 1 =
skip reporter, level 2+ = dead spin; never ping-pongs with KeBugCheckEx).

**The lang item stays per-crate-graph** (one `#[panic_handler]` per rustc
crate graph; final PEs dedup via `--allow-multiple-definition` first-match) —
only the body is shared. That is why this is a forwarder pattern, not a
single handler crate.

**Halt-policy feature = the final image's link surface** (externs must
resolve per-artifact):

| feature | externs | halt | for |
|---|---|---|---|
| `kernel` | DbgPrint, RtlWalkFrameChain, KeBugCheckEx (ntoskrnl) | `KeBugCheckEx(0xE0525354)` | ntoskrnl, .sys, win32k, kernel DLLs |
| `user` | DbgPrint, RtlWalkFrameChain, NtTerminateProcess (ntdll) | `NtTerminateProcess(-1)` | user .exe/.dll |
| *(none)* | DbgPrint, RtlWalkFrameChain (both worlds export them) | spin | libs linked into BOTH worlds (rtl-rust, utf8) |
| `raw` | NONE | debugcon + spin | constrained links (SCSIPORT miniports — atapi.sys/buslogic link ONLY static scsiport, not even DbgPrint resolves) |

Bugcheck code `RUST_PANIC = 0xE0525354` ("RST", OEM 0xE0 range).
KeBugCheckEx P1 = &message (static, NUL-terminated — the future red-screen /
minidump writer reads it), P2 = buffer cap, P4 = source line.

**Skips (keep their own handlers):** freeldr-rust / rosload-cargo /
uefildr-cargo (bootloader world exports neither DbgPrint nor
RtlWalkFrameChain; freeldr keeps its 0xe9 marker), cmlib-rust / cnt-rust-lib /
cport-rust / hal-rust (co-linked into the bootloaders via `bldr`/`minihal`
features), reactos-link (host tool). Follow-up: gate those four's forwarders
`#[cfg(not(feature = "bldr"/"minihal"))]` to upgrade their kernel builds too.

**Self-test:** `--tripwire "RUSTPANIC_SELFTEST=1"` (fw_cfg, no media rebuild;
tripwire.rs) deliberately panics at Mm Phase 0 arm time — expect the full
report then BUGCHECK 0xE0525354. Destructive by design.

## uwtable sweep (owed by §9.35, DONE here)

All 76 remaining amd64+arm64 `*-kernel.json` target specs flipped
`requires-uwtable: false → true` (ntdll, win32k, every driver/user cargo).
Without it those modules ship ZERO unwind info — user-mode dispatch and any
stack walk dies at their first Rust frame. i686 stays false (fs:[0] chain
SEH, no .pdata).

## Exception chain to userland (Layer "hardware → user SEH")

State per the 2026-07-07 audit: kernel side works and is PoC-validated on
amd64 (`seh_poc.c`); ntdll dispatcher machinery is real but no user module
ships handler tables (all DUMMY_PSEH); software exceptions are swallowed by
the asm `KiUserExceptionDispatcher` shim; hardware faults kill the process.

Changes here:
- **arm64 `dll/ntdll/dispatch/arm64/dispatch.S`**: added the missing
  DUMMY_PSEH swallow branch (amd64/i386 parity): continuable, code <
  0xC0000000 → NtContinue (STATUS_BREAKPOINT steps Pc+4 — arm64 brk points
  AT the instruction, no x86-style rewind); else second-chance.
- **`base/system/sehtest`** — user-mode SEH boot self-test, the userland
  sibling of seh_poc.c. Native clang SEH (`-fms-extensions`), amd64+arm64
  (i386 native SEH doesn't lower — verdict §C). Three tests: software raise
  (RaiseException→caught), __finally on unwind, and the money shot —
  hardware AV in a noinline callee (calls-only coverage!) caught by a real
  user `__except` after the full kernel→KiUserExceptionDispatcher→ntdll
  dispatch→unwind roundtrip. Spawned fire-and-forget by userinit
  (`userinit.rs` wWinMain, cfg amd64/arm64). Serial: `[SEH-USER] … PASS`.

## Layer 3 — flight-recorder sidecar (crash-time dump)

The full Windows minidump-to-disk path stays stubbed (`KeInitializeCrashDumpHeader`
returns STATUS_UNSUCCESSFUL; no dump device). What IS tractable and high-value is
the **rich native sidecar** the brief asks for — dumped to serial/debugcon at
bugcheck time, alongside the backtrace we already emit.

Foundation already in tree: `ke/flightrec.rs` — a lock-free 16K-entry ring
(`VIBEOS_FLIGHTREC`, 512KB .bss) of the last sync events (`gate-park/-signal`,
`pushlock-park/-wake`, `ws-lock-excl`, `lockdep-selfdeadlock`, `lockdep-abba`),
each `{seq, thread, object, op, data}`. Recorded serial-I/O-free (~10ns, relaxed
atomics) so it doesn't perturb the races it captures. It had only a *per-object*
census dump (`flightrec_dump_object`, for GDB/wedge triage) — nothing read it at
crash time.

Added here:
- **`flightrec_dump_recent(limit)`** — dumps the tail of the ring (last N events
  across ALL objects, oldest-first, so the final line is the last event before
  the crash). Torn slots (racing a live writer) are skipped. Runs from the
  bugcheck path — the record-path serial-I/O ban doesn't apply once we're halting.
- **Wired into `KeBugCheckWithTf`** (`ke/bug.rs`), inside the once-only
  `KiBannerDone` banner block, right after the stack backtrace. Every BUGCHECK and
  RUST_PANIC now emits `[FLIGHTREC] --- last 32 of N sync events ---` with the
  thread on each op. For the lost-wakeup / self-deadlock / ABBA bugcheck classes
  (§9.13 WS-pushlock family, §9.24 gate corruption) this is the smoking gun a
  post-mortem backtrace alone can't show: *who* signalled/parked on the object
  last, in what order.
- **`flightrec_op_name`** — the op→name table, now shared by both dumpers (the
  census dump was missing the WS-lock/lockdep names).

Validation: `smoke --arch amd64 --tripwire "RUSTPANIC_SELFTEST=1"` → the RUST
PANIC report is now followed by the `[FLIGHTREC]` tail. (Sidecar content is
boot-path sync traffic; by Mm Phase-0 selftest time the ring holds thousands of
events, so the last-32 window is always populated.)

## WinDbg KD (design, in progress)

Ground truth: `kd64-rust` (~12k LOC, full WinDbg state-manipulate protocol)
is linked and live on all arches, but `KdInitSystem` blanket-forces
`enable_kd=0` on x86/amd64 (kdinit.rs) — the historical "recursive INT3
flood" is the §9.35 unwind bug, now fixed. `KdSendPacket`/`KdReceivePacket`
are defined IN-KERNEL by kd-rust `kdio.rs` (ROSDBG text glue); the real
serial framing exists complete in `kdcom-serial-rust` (unwired spike;
`_WINKD_` is FALSE on llvm-mingw so the shipping kdcom.dll is the kdrosdbg
stub).

Plan (amd64 first) — **no `_WINKD_` flip, no PE import surgery**:
1. `KdInitSystem` opt-in (DONE): `/DEBUG /DEBUGPORT=COM2` enables real KD
   (`KiDebugRoutine=KdpTrap`); anything else keeps today's KdpStub
   text-to-COM1. COM1 stays the console/smoke channel; KD gets COM2 —
   binary packet framing on COM1 would destroy the smoke marker pipeline.
2. Embed `kdcom-serial-rust` in the kernel as an rlib behind renamed entry
   points (feature-gate its `#[no_mangle]` exports off); kd-rust's
   `KdSendPacket`/`KdReceivePacket` delegate to it when the COM2 mode is
   armed, else keep the existing text behavior byte-for-byte.
3. Opt-in paths (both implemented): boot entry `LiveCD (WinDbg KD on COM2)`
   (livecd.ini, `/DEBUG /DEBUGPORT=COM2`) or per-boot `--tripwire "KD=COM2"`
   (fw_cfg; `VibeTripwireHasToken` exported from ntoskrnl-rust tripwire.rs —
   KdInitSystem reads it directly since it runs pre-Mm). Either way
   `KdpWireTransportArm` pins the wire to COM2 @115200 via
   `KdpPortInitialize(2, …)` — deliberately NOT the LoadOptions parse, so a
   tripwire boot carrying default `/DEBUGPORT=COM1` options can never seize
   the console port. Port-init failure falls back to KdpStub (no dead-wire
   boot wedge).
4. QEMU wiring: `VIBEOS_QEMU_EXTRA_ARGS` (vm.py:953) appends a second
   `-serial` → COM2. windows-host backend: `-serial pipe:vibeos_kd` creates
   `\\.\pipe\vibeos_kd`. WinDbg is installed on the Windows host (winget
   Microsoft.WinDbg 1.2606; `kdX64.exe`/`WinDbgX.exe` on PATH via
   WindowsApps): `kdX64.exe -k com:pipe,port=\\.\pipe\vibeos_kd,resets=0,reconnect`.
5. Validate: attach, break-in, `k` (stack walk over Rust frames — needs the
   uwtable sweep above), `lm`, memory read, breakpoint.

### First live-attach attempt (2026-07-09) — wire is ALIVE, handshake incomplete

Fully automatable from WSL: `kdX64.exe` (10.0.29617, WindowsApps PATH) runs
via interop. Recipe used:
1. `kdX64.exe -logo C:\temp\vibeos_kd.log -k com:pipe,port=\\.\pipe\vibeos_kd,resets=0,reconnect -c "..."`
   (launch FIRST; `reconnect` waits for the pipe).
2. `VIBEOS_QEMU_EXTRA_ARGS="-serial pipe:vibeos_kd" ./vibeos-cli smoke --arch amd64 --tripwire "KD=COM2"`
   (windows-host qemu creates `\\.\pipe\vibeos_kd` as pipe server).

Result: **first-ever bidirectional WinDbg↔vibeos packet exchange.** kd
connected, exchanged packets far enough to attempt reading the debugger data
list (`Unable to read head of debugger data list, Win32 error 0n56`, then
`Unable to get target version info, re-init transport and retry`). Two gaps:
- **Boot wedges** right after the tripwire is consumed — KdInitSystem on COM2
  blocks in the packet loop waiting for a handshake our KdReceivePacket side
  doesn't complete. Needs timeout/retry semantics (KdpStub fallback after N
  failed resyncs) so an armed boot without a fully-working debugger still
  reaches desktop.
- **DbgKdGetVersionApi / KdVersionBlock / KdDebuggerDataBlock** responses are
  what kd needs next (error 56 = it never got a valid data-list address).
Cosmetic: tripwire.rs prints `unknown tripwire token (ignored)` for KD=COM2 —
harmless (kd64 reads it via `VibeTripwireHasToken` substring match), but the
parser should learn the token to stop the scary print.

## Validation results (2026-07-08)

- **amd64 — FULLY VALIDATED end-to-end.**
  - Build + smoke green.
  - `[SEH-POC]` (kernel `__try/__except` catches `ExRaiseStatus`) PASS.
  - **`[SEH-USER]` all three PASS**: software `RaiseException` caught,
    `__finally` runs on unwind, and the money shot — a real hardware
    ACCESS_VIOLATION (`0xC0000005`) in a noinline callee caught by a user
    `__except` after the full kernel→`KiUserExceptionDispatcher`→ntdll
    dispatch→`.pdata` unwind→handler roundtrip. `sehtest.exe` exits 0.
    (The spawn must use `DETACHED_PROCESS`; a win32cui child spawned this
    early wedges in console allocation — nothing to do with the exception
    machinery.)
  - `--tripwire "RUSTPANIC_SELFTEST=1"`: the FIRST rich crash report —
    `*** RUST PANIC in ntoskrnl at ntoskrnl-rust/src/tripwire.rs:231:13`,
    the panic message verbatim, a 9-frame `.pdata`-walked backtrace, then a
    single `BUGCHECK 0xE0525354`. No secondary bugcheck.
- **i386** — build + smoke green (no sehtest; native SEH doesn't lower on
  i386, and its user dispatch is the `fs:[0]` chain).
- **arm64** — build + smoke green (desktop boots). SEH self-test surfaced a
  real OPEN gap (below); non-fatal, sehtest is fire-and-forget.

### Crash-path robustness fixes this lane surfaced (Layer 4)

1. **§9.35 fault-amplification, the real fix**: uwtable sweep (all modules)
   + a re-entrancy `PANIC_LEVEL` guard in `panic-common` (nested/concurrent
   panic → skip reporter → halt; never ping-pongs with KeBugCheckEx).
2. **Secondary-0x1E in the bugcheck display path**: `KeGetBugMessageText`
   (`ke/bug.rs`) walks the message-table resource section, which is not
   fault-safe during a Phase-0 bugcheck (`KeLoaderBlock` still live) — it
   took `STATUS_IN_PAGE_ERROR` and cascaded into a second `0x1E` that buried
   the real report. Now: early-boot short-circuit (`KeLoaderBlock != NULL`)
   + `KiBugMsgRangeValid` page-walk guard → degrade to code-only display.
   This is exactly the "crash reporter that can crash" failure mode.

## arm64 user SEH: unwinder rewrite + the latent layout-race it exposed (2026-07-10)

Implemented the real ARM64 `.xdata` unwind-code interpreter in
`rtl-except/src/arm64/except.rs` (`RtlpUnwindXDataCodes`). The prior code was a
`Pc = Lr` **leaf stub** for every `.xdata` (non-packed) frame — so `RtlVirtualUnwind`
could never step past a non-leaf frame, `RtlDispatchException`'s walk stalled at
frame 0, and no user `__except` scope table was ever reached. That is the true
reason arm64 first-chance user SEH never delivered (the "RtlRaiseException missing"
note below was a mis-diagnosis; `RtlRaiseException` IS present via `rtl-diag`).
Two dispatch-loop fixes rode along: per-frame `ControlPc` in `RtlDispatchException`
(was reusing the fault PC for every frame → parent scope tables never matched), and
restoring the target frame's **pre-unwind** context at the `RtlUnwindEx` jump target
(was resuming the faulting callee's x29/sp). Opcode decoder validated byte-for-byte
against clang's emitted `.xdata` in `sehtest.exe`.

**The land was blocked by a pre-existing latent bug this change exposed.** Adding the
unwinder — OR a same-size *dead, never-called* padding function — turned arm64 smoke
red at `BUGCHECK 0x64` (SESSION5/SMSS create failed). Clean HEAD boots **7/7 green**;
the perturbed build is **red 4/4** → deterministically **layout-sensitive**, not a
flake. `rtl-except` compiles into `librtl`, which links into **ntoskrnl.exe**, so any
text-size delta shifts the in-kernel pnpmgr/iomgr code and changes PnP timing.

Failure chain (proven with a `MAPFILE`/component-ladder/`\Device`-enum probe series in
`RtlpMapFile` + `RamdiskCreateDiskDevice`): ramdisk **bus-node**
`QUERY_DEVICE_RELATIONS(BusRelations)` returns `STATUS_INVALID_DEVICE_REQUEST`
(`0xC0000010`) → boot-disk PDO never enumerated → `\Device\Ramdisk{guid}\…` (and thus
`\SystemRoot\System32\smss.exe`) opens fail `STATUS_OBJECT_PATH_NOT_FOUND`
(`0xC000003A`) → `ExpLoadInitialProcess` bugchecks `0x64`.

**Root cause:** `ramdisk-rust/src/ramdisk.rs:2641` (`RamdiskAddDevice`) had
`(*DeviceObject).Flags &= DO_DEVICE_INITIALIZING;` — a faithful copy of an upstream
ReactOS `ramdisk.c:2411` typo that **drops the `~`**. Instead of *clearing*
`DO_DEVICE_INITIALIZING` it masks Flags down to *only* that bit, so the bus FDO stays
permanently "initializing" **and** loses the `DO_DIRECT_IO`/`DO_POWER_PAGABLE` just
set. That stuck state is *necessary-but-not-sufficient*: normally the QDR fires late
enough to tolerate it (clean HEAD green), but a layout shift moves the QDR into the
window where it's rejected. Fix: `&= !DO_DEVICE_INITIALIZING` (matches every other
Rust driver + the C *intent*), which both marks the FDO ready and preserves the I/O
flags — removing the necessary condition for all layouts.

## arm64 user-mode SEH — NOW DELIVERS (2026-07-10)

With the `.xdata` unwinder landed, arm64 `sehtest` first-chance delivery works
(`[UEXC] 1st=1`, `[UEXC-DISPATCH]` to `KiUserExceptionDispatcher`). Results:
- `[SEH-USER] sw-raise: caught=1 (PASS)` — software `RaiseException` caught.
- `[SEH-USER] finally: ran=1 caught=1 (PASS)` — `__finally` unwind runs.
- `[SEH-USER] hw-fault: caught=1 code=0xc0000005 (PASS)` — the hardware
  ACCESS_VIOLATION in a noinline callee is **caught** by the user `__except`.

(The earlier "RtlRaiseException not implemented for arm64" diagnosis was WRONG —
`RtlRaiseException` is present via `rtl-diag`. The real gap was the `.xdata` leaf
stub in `RtlVirtualUnwind`, now fixed.)

### Fix: kernel hardware-fault context must carry callee-saved x19-x28

`ke/arm64/trap.rs` `KiSynchronousExceptionHandler` called `KiDispatchException`
with a **NULL** `ExceptionFrame`. On ARM64 the callee-saved integer regs
(x19-x28) live in the KEXCEPTION_FRAME, not the KTRAP_FRAME, so
`KeTrapFrameToContext` left them at their zero-init values → the CONTEXT handed
to `KiUserExceptionDispatcher` carried **zeroed x19-x28**, and a hardware fault
delivered to a user `__except` resumed with lost callee-saved state (the
`__except`'s `"PASS"` string pointer — a non-volatile reg — resolved to NULL,
printing `(null)`). Fix: `KiCaptureCalleeSaved` (a no-prologue `global_asm!`
shim) snapshots x19-x28 at handler entry — still the faulting code's values,
since `TRAP_PROLOG` only touches x0/x1 — into a local KEXCEPTION_FRAME passed to
all four `KiDispatchException` calls. The `__except` now sees correct x19-x28
(`hw-fault … (PASS)`).

### RESIDUAL (OPEN): hw-fault `__except` leaves `main`'s callee-saved wrong

`sehtest` still prints `RESULT: FAIL`. Instrumented: `after sw-raise r=1 pass=1`,
`after finally r=1 pass=1`, `after hw-fault r=1 pass=0` — `TestHardwareFault`
returns 1 correctly, but `main`'s aggregate `pass` (a callee-saved register) is
corrupted to 0 **across** the hardware-fault `__except`, while the software-raise
`__except` preserves it. FP-relative access is fine (locals, return value, the
`(PASS)` string all correct); the corruption is SP-relative — the `__except`
resume SP is subtly off, so the epilogue's `ldp x19,x20,[sp,#..]` restores
`main`'s registers from a wrong slot. This is a multi-frame ARM64 unwind
SP-restoration detail in `RtlUnwindEx`/`RtlVirtualUnwind` (leaf `FaultingWrite`
→ `main`'s inlined `__except`). Next step: GDB single-step the unwind, compare
the resume SP against the epilogue's expected frame SP. Boot stays green
(sehtest is fire-and-forget); this is the last-mile of arm64 hardware-fault SEH.

## Validation ladder (repeat before landing)

1. `vibeos-cli build --arch <a>` green.
2. `smoke --arch amd64` green + `[SEH-POC]`/`[SEH-USER]` PASS on serial.
3. `smoke --arch amd64 --tripwire "RUSTPANIC_SELFTEST=1"` → full RUST PANIC
   report + single `0xE0525354` (expected-fail run).
4. Gate ×3 across all arches before landing.
