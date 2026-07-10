# WOW64 on vibeos — running x86 binaries on arm64 (FEX) and x64 (native)

Status: **Phase A COMPLETE on-box** (2026-07-09): `libwow64fex.dll` fully
resolves (146 ntdll imports + 2 wow64.dll imports) and loads in a vibeos arm64
process — `[FEX-WOW64A] libwow64fex.dll loaded, BTCpuGetBopCode=... (PASS)`.
Delivered: arm64-gated ntdll exports (synch family via rtl-sync cabi + the
new-API shims in `ntdll-rust/src/rtl/wow64shim.rs`) and the `dll/win32/wow64`
skeleton. Next: Phase B. Owner: fex-arm lane.
Predecessor: `fex-bringup-plan.md` (rungs 0–3 complete; rung-4 *mechanism*
proven in-harness — this doc IS the real rung 4+ execution plan).

## The model (Win10+ BTCpu architecture)

Modern WOW64 splits into three layers; we adopt it because FEX's wow64 module
(`external/fex/Source/Windows/WOW64/`, prebuilt as `libwow64fex.dll`) already
implements the bottom layer's exact contract:

1. **Kernel + 64-bit loader**: creates the WoW64 process (PEB32/TEB32 beside
   the 64-bit ones, <4GB image/stack/heap for the guest), maps 32-bit ntdll,
   populates `LdrSystemDllInitBlock` (wow64fex reads `ntdll_handle` from it),
   loads wow64.dll into 64-bit userspace.
2. **wow64.dll (OURS, Rust-preferred)**: process/thread bootstrap, the syscall
   thunk layer (`Wow64SystemServiceEx`: 32-bit args → 64-bit Nt* call →
   32-bit result), CPU-area management, loads the CPU emulator by contract.
3. **CPU layer**: `libwow64fex.dll` on arm64 (FEX, vendored, DONE — exports
   the 23 `BTCpu*` entry points); a to-be-written classic `wow64cpu.dll` on
   amd64 (far-jmp 0x33 mode switch, no emulation needed).

### The measured contract (2026-07-09, llvm-objdump on libwow64fex.dll)

wow64fex imports from **wow64.dll** ONLY `Wow64SystemServiceEx` and
`Wow64ProcessPendingCrossProcessItems`. From **ntdll.dll** it imports 146
symbols; diffing against our arm64 ntdll exports leaves this gap:

| Gap | Where it lives today | Plan |
|---|---|---|
| `RtlAcquire/Release/TryAcquireSRWLock*`, `RtlInitializeSRWLock`, `Rtl*ConditionVariable*`, `RtlRunOnceExecuteOnce`, `RtlWaitOnAddress`, `RtlWakeAddress*` | ntdll_vista.dll (0x600 split) | `-arch=arm64` forwarders in base ntdll.spec → ntdll_vista (§9.43 pattern: arch-gating leaves x86 ordinal tables untouched) |
| `RtlWow64GetCurrentCpuArea/GetThreadContext/SetThreadContext/SuspendThread` | nowhere | implement (ntdll, arm64-gated); CPU area lives at `TEB->TlsSlots[WOW64_TLS_CPURESERVED=1]` (Wine-compatible layout: `WOW64_CPURESERVED { USHORT flags; USHORT machine; ctx; ctx_ex }`) |
| `NtAllocateVirtualMemoryEx` | nowhere | wrapper over NtAllocateVirtualMemory (extended params only needed for arm64EC EC_CODE, not wow64) |
| `NtOpenKeyEx` | nowhere | thin syscall (options arg) |
| `LdrGetDllFullName` | nowhere | small ldr helper |
| `RtlQueryPerformanceCounter/Frequency` | kernel32 only | ntdll impl reading shared-user-data / KeQPC path |

Everything else (all the Nt* file/section/thread calls, Rtl heap/unwind/
`RtlRestoreContext`/`RtlVirtualUnwind`, CS, PEB lock, DbgPrint...) already
exports from our arm64 ntdll.

## Phases

**Phase A — ntdll surface (arm64). COMPLETE (2026-07-09).** Landed the table
above; `LoadLibraryW("libwow64fex.dll")` succeeds on-box and `BTCpuGetBopCode`
resolves (fexprobe's `[FEX-WOW64A]` appendix is the regression probe — it runs
on every VIBEOS_FEX boot). Implementation notes: synch family exported from
base ntdll directly (the rtl-sync `cabi` code was always linked in — only the
0x502 version gate hid the exports; arm64-gated duplicate spec lines, x86
ordinal tables byte-identical); new APIs in Rust
(`ntdll-rust/src/rtl/wow64shim.rs`, `#[cfg(target_arch = "aarch64")]`);
`dll/win32/wow64` skeleton exports the two wow64fex imports (loud
`STATUS_INVALID_SYSTEM_SERVICE` until the Phase-B thunk table);
`libwow64fex.dll` + `wow64.dll` stage via mkramdisk under `VIBEOS_FEX=1`.

**Phase B — wow64.dll skeleton + wow64probe.** Author minimal `dll/wow64/`
(ours): `Wow64SystemServiceEx` with a hand thunk table (`NtWriteFile`→serial,
`NtTerminateProcess`, `NtQueryVirtualMemory`), `Wow64ProcessPendingCrossProcessItems`
stub. A `wow64probe.exe` harness (native arm64) that: allocates the CPU area +
TLS slot, maps our real build-i386 `ntdll.dll` into the process (satisfies
`LdrSystemDllInitBlock.ntdll_handle` — populate a minimal init block),
calls `BTCpuProcessInit`/`BTCpuThreadInit`, points a <4GB x86 blob at
`BTCpuGetBopCode`'s bridge for its syscall, `BTCpuSimulate`s it, and verifies
the thunked syscall round-trips. This is fexprobe's methodology applied one
layer up — everything stays out-of-band until it works.

**Phase C — real WoW64 process bootstrap (the OS work).**
- Kernel: accept i386 PE on arm64/amd64 host (`PspCreateProcess` machine
  check), PEB32+TEB32 creation, <4GB default alloc policy for the guest,
  `ZwQueryInformationProcess(ProcessWow64Information)`.
- Loader: map 32-bit ntdll into new WoW64 processes, populate
  `LdrSystemDllInitBlock`, auto-load wow64.dll + CPU dll, `Wow64Transition`
  publishing (32-bit ntdll's syscall stubs dispatch through it → BOP page).
- ntdll32: build variant of our i386 ntdll whose syscall stub calls through
  `Wow64Transition` instead of raw `int 2e` (we own the i386 build — this is
  a macro swap, not a new DLL).
- Image: stage a SysWOW64 set from build-i386 output into the arm64/amd64
  media (mkramdisk/livecd step).
- Minimal filesystem redirection (system32→SysWOW64) later in the phase.

**Phase D — thunk generation at scale.** The hand table in B/C covers hello
world; real apps need hundreds of Nt* thunks + wow64win.dll for win32k. Use
the rustconv/bindgen machinery to GENERATE marshalers from our own syscall
table (in-repo single source of truth) — this is where the Rust story pays.

**Phase E — amd64 classic wow64cpu.** Same phases A–D infrastructure, CPU
layer swapped: `wow64cpu.dll` doing the far-jmp 0x33 CS switch + 32↔64 stack
shuffle. No emulator. (x86-on-x64 "for free" once C+D exist.)

## Ground rules
- FEX vendor tree stays unmodified (external/fex/CLAUDE.md forbids AI-authored
  contributions upstream; we consume its binary contract only).
- wow64.dll/wow64cpu.dll/ntdll changes are vibeos code: Rust-preferred where
  practical (wow64.dll is a clean greenfield candidate).
- Every phase lands behind the existing gates: arm64 boots must stay green;
  x86 arches untouched by arch-gated exports (§9.43 ordinal discipline).
