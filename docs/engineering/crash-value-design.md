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

## OPEN: arm64 user-mode SEH first-chance delivery

`sehtest` on arm64: the software raise arrives at `KiDispatchException` with
`FirstChance=0` (second-chance) directly — no first-chance user dispatch ever
happens (`[UEXC] 1st=0` then `[UEXC-KILL]`, process dies `exit=0xe0dead02`).
Root: **`RtlRaiseException` is not implemented for arm64** — `rtl-except`'s
arm64 module only *references* it (`arm64/except.rs:403,420`); the
`@ stdcall -register RtlRaiseException` ntdll export has an amd64/i386 asm
capture-and-`NtRaiseException(…,TRUE)` stub with no arm64 equivalent, so the
user raise never enters first-chance. This is the FIRST time arm64 user SEH
has been exercised (the 2026-07-07 map noted the arm64 unwinder as
"complete translation, never runtime-exercised"). The kernel-side arm64
delivery (`KiDispatchExceptionToUser`, the dispatch.S swallow added this
lane) is in place; the missing piece is the ntdll-side arm64
`RtlRaiseException` (+ likely `RtlpExecuteHandler*` asm trampolines). Fix is
an arm64 asm/naked capture stub mirroring amd64 `except_asm.S`. Field manual
§9.36.

## Validation ladder (repeat before landing)

1. `vibeos-cli build --arch <a>` green.
2. `smoke --arch amd64` green + `[SEH-POC]`/`[SEH-USER]` PASS on serial.
3. `smoke --arch amd64 --tripwire "RUSTPANIC_SELFTEST=1"` → full RUST PANIC
   report + single `0xE0525354` (expected-fail run).
4. Gate ×3 across all arches before landing.
