# WOW64 on vibeos — running x86 binaries on arm64 (FEX) and x64 (native)

Status: **Phase B COMPLETE on-box** (2026-07-09): wow64probe drives the real
`libwow64fex.dll` through the whole BTCpu contract on vibeos arm64 — a 32-bit
x86 guest blob JITs, calls through the BOP bridge, and its syscall marshals
into OUR wow64.dll's `Wow64SystemServiceEx` and back:
`[FEX-WOW64B] guest syscall round-trip PASS`, exit 0x600D. The overcommit
AV was serviced live by `BTCpuResetToConsistentState`+`NtContinue` mid-run.
(Phase A, 2026-07-09 earlier: dll loads, 146 ntdll imports resolve, synch
family + `wow64shim.rs` + `dll/win32/wow64` skeleton.) Next: Phase C.
Owner: fex-arm lane.
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

**Phase B — wow64.dll skeleton + wow64probe. COMPLETE (2026-07-09).**
Author minimal `dll/wow64/`
(ours): `Wow64SystemServiceEx` with a hand thunk table (`NtWriteFile`→serial,
`NtTerminateProcess`, `NtQueryVirtualMemory`), `Wow64ProcessPendingCrossProcessItems`
stub. A `wow64probe.exe` harness (native arm64) that: allocates the CPU area +
TLS slot, maps our real build-i386 `ntdll.dll` into the process (satisfies
`LdrSystemDllInitBlock.ntdll_handle` — populate a minimal init block),
calls `BTCpuProcessInit`/`BTCpuThreadInit`, points a <4GB x86 blob at
`BTCpuGetBopCode`'s bridge for its syscall, `BTCpuSimulate`s it, and verifies
the thunked syscall round-trips. This is fexprobe's methodology applied one
layer up — everything stays out-of-band until it works.

Progress + audited `BTCpuProcessInit` dependency list (from WOW64/Module.cpp):
- [x] `LdrSystemDllInitBlock` DATA export added to arm64 ntdll (zeroed
  96-byte block in `wow64shim.rs`; FEX reads `ntdll_handle` at offset 72 —
  GetProcAddress would have returned NULL and wow64fex derefs it unchecked).
  wow64probe pokes `ntdll_handle` = mapped i386 ntdll base BEFORE
  `BTCpuProcessInit`.
- [x] BOP page ZeroBits: kernel gap CONFIRMED and FIXED (2026-07-09) — our
  `NtAllocateVirtualMemory` (`ntoskrnl-rust/src/ARM3/virtual.rs`) rejected any
  ZeroBits > MI_MAX_ZERO_BITS. Added the 64-bit NT mask semantics: ZeroBits
  > 32 is an address MASK (alloc must land at or below it); FEX passes
  (1<<31)-1 for the `int 2e` bridge. wow64probe re-verifies at runtime (its
  own guest code/stack use the same call shape).
- [x] WOW64INFO: `BTCpuProcessInit` writes `CpuFlags = WOW64_CPUFLAGS_SOFTWARE`
  through `TEB->TlsSlots[WOW64_TLS_WOW64INFO=10]` — probe installs a
  `WOW64INFO{NativeSystemPageSize, NativeMachineType=AA64, Emulated=014C}`
  there before ProcessInit (real bootstrap: lives after the PEB32, Phase C).
- [x] CPU area: probe allocates `WOW64_CPURESERVED{Machine=I386}` + 16-aligned
  WOW64_CONTEXT into `TlsSlots[1]` before `BTCpuThreadInit`; first
  `BTCpuSimulate` flushes it into the JIT via our `RtlWow64GetCurrentCpuArea`
  (the WOW_CPU_AREA_DIRTY control-word bit ThreadInit sets). Guest entry
  context: SegCs=0x23 (GDT index 4 == wow64fex `DefaultWow64CS`), SegFs=0x53,
  EFlags=0x202, XSave FCW=0x27F MXCSR=0x1F80.
- [x] Exit strategy: wow64.dll's `Wow64SystemServiceEx` grew Phase-B probe
  services — ECHO (0xF00D: guest-stack args in, magic 0x5CA11ED back in guest
  EAX) and VERDICT (0xD00E: guest returns the magic, host prints
  `[FEX-WOW64B] ... PASS` and `NtTerminateProcess(0x600D)`). BTCpuSimulate
  never returns by design.
- [x] Guest FS base: `GetWowTEB` reads the LONG at TEB+0x180c (WowTebOffset)
  and adds it to the TEB — zero on our TEB, so guest FS base = TEB64. Fine
  while the probe blob avoids FS; real TEB32 is Phase C.
- [x] `FEX::Config::LoadConfig` + `Logging::Init`: TWO landmines found+fixed.
  (1) The dll's mini-CRT imports `_errno`/`atan2`/`tan` from ntdll — Wine/Win10
  ntdll exports them, ours didn't, and OUR LOADER POISON-SNAPS missing imports
  to `0xffbadd11` instead of failing the load (`ldrpe.rs:849`), so the first
  errno write in LoadConfig's `access()` became a PC-alignment crash at
  PC=0xFFBADD11 (X16=poison → it's a `br x16` through the import veneer;
  diagnose by diffing `llvm-objdump -p` imports vs our exports). Fixed with
  arch-gated msvcrt forwarders in ntdll.spec. (2) `Logging::Init` formats
  `getenv("LOCALAPPDATA")` unchecked (NULL in winlogon-spawned processes) —
  probe defaults the env var; a real session env should provide it (Phase C).
- [x] i386 ntdll mapping: **`NtCreateSection(SEC_IMAGE)` of an i386 PE WORKS
  on the arm64 kernel** — mapped at 0x7E000000 size 0x2FF000, no fallback
  needed. Big Phase-C readiness datum. Staged from build-i386 to
  `\reactos\syswow64\ntdll.dll` (mkramdisk, VIBEOS_FEX=1).
- [x] `HandleImageMap` ran over the arm64 main image + the i386 ntdll — no
  PE-machine assumptions bit.
- [x] Exception routing: the LookupCache overcommit AV inside BTCpuSimulate is
  serviced by the dll's `BTCpuResetToConsistentState` (commits the page,
  `NtContinue`s itself). The probe's VEH routes every exception there first —
  in Phase C this is wow64.dll/ntdll's `Wow64PrepareForException` job.
- [x] Build-infra fix en route: `add_cargo_module` DEPENDS now globs `*.spec`
  (an ntdll.spec edit shipped a stale ntdll.dll — cargo rerun-if-changed
  tracked it but ninja never re-invoked cargo; sdk/cmake/gcc.cmake).
- [x] Kernel fix en route: `NtAllocateVirtualMemory` 64-bit ZeroBits-mask
  semantics (`ARM3/virtual.rs`) — ZeroBits > 32 is a max-address mask.

Harness plumbing (all landed 2026-07-09): `wow64probe.exe` builds via
`tools/fex/build-hello-compute.sh` (second target in the hello-compute
CMake, native arm64, NO FEXCore link — the dll's export surface is the
interface under test); winlogon spawns it after fexprobe when staged
(`TryStartWow64Probe`); verdict = serial `[FEX-WOW64B]` lines + exit code
0x600D.

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
