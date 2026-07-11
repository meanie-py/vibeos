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

*Milestone 1 — PASS on-box (2026-07-09): `CreateProcessW("hello32.exe")` (a real
i386 PE) on arm64 runs the whole bootstrap chain and reports
`[FEX-WOW64C] M1 PASS: WoW64 process bootstrap chain complete (image base 00400000)`,
exit 0x600D.*
- [x] Kernel: admit i386 images — `SharedUserData->ImageNumberLow = I386` on
  arm64 (ex-rust `init.rs`; kernel32's CreateProcessInternalW range gate).
  Section/PspCreateProcess needed NO machine gate change (already permissive).
- [x] Kernel: PEB32 below 2GB, mirrored from PEB64, `EPROCESS.Wow64Process =
  PEB32` (5.2-style — exactly what our `ProcessWow64Information` handler
  returns). Detection = image `FileHeader.Machine == 0x14c` in `MmCreatePeb`
  (ntoskrnl-rust `ARM3/procsup.rs`; `MiCreatePebOrTebCapped` low-address VAD).
- [x] Kernel: WoW64 threads get a combined region below 2GB — TEB64 at +0,
  TEB32 at +0x2000 — with the raw Win10 `WowTebOffset` slots (TEB+0x180c /
  TEB32+0xfdc; past our 5.2 struct ends, committed scratch — wow64fex's
  GetWowTEB hardcodes them). `MmDeleteTeb` frees by VAD extent now.
- [x] Loader: `LdrpInitializeProcess` WoW64 branch (ntdll-rust `ldrinit.rs`) —
  queries ProcessWow64Information, skips the i386 image's import walk/TLS/
  kernel32 entirely, loads wow64.dll, hands the thread to
  `Wow64LdrpInitialize` (never returns). Mirrors Win10 LdrpLoadWow64.
- [x] wow64.dll: `Wow64LdrpInitialize` export (ntdll-only DLL now — kernel32
  importlib dropped so a WoW64 process never pulls 64-bit kernel32).
  M1 body verifies PEB32/TEB32 chain and exits 0x600D.
- [x] Harness: `hello32.exe` (no-CRT i386 PE, built by build-hello-compute.sh
  via i686 clang), staged to syswow64 under VIBEOS_FEX=1; winlogon
  `TryStartWow64Hello` launches it through the normal CreateProcessW path.

*Milestone 2 — PASS on-box (2026-07-10): real i386 image code executes under
FEX and exits through the WoW64 service bridge with `0x600D`.* The full CPU-layer
bring-up is now folded INTO `Wow64LdrpInitialize` (wow64.c): maps syswow64
ntdll32 (SEC_IMAGE via NtOpenFile+NtCreateSection), populates
`LdrSystemDllInitBlock.ntdll_handle`, sets the WOW64INFO TLS slot + LOCALAPPDATA
env, loads libwow64fex, resolves + calls BTCpuProcessInit/ThreadInit, builds an
exit stub + guest CONTEXT at the i386 image entry, and enters BTCpuSimulate.
The guest exit stub calls a new `WOW64C_SVC_EXIT` service → M2 PASS.

Two bugs found + fixed along the way:
- **[FIXED] hello32 hung in kernel32 `ConDllInitialize`.** A CUI (console) i386
  image makes kernel32's DllMain allocate a console via csrss, which hangs for
  a winlogon-spawned WoW64 process (no console to inherit; winlogon's 30s wait
  → GetExitCodeProcess returns 0x103/STILL_ACTIVE). Fix: build hello32 as GUI
  subsystem (`--subsystem,windows:5.1`) — the working GUI probes skip console
  init. (K32_DLLMAIN_TRACE pinpointed it: reaches `pre-ConDllInitialize`, never
  `post-`.)
- **[FIXED] The wow64.dll load must happen AFTER `LdrpRunInitializeRoutines`,
  not before.** libwow64fex imports kernel32; loading it from the early
  divert point ran kernel32's DllMain re-entrantly under the loader lock and
  hung. Moved the divert in ldrinit.rs to right after the init routines
  complete (kernel32 already initialized at the normal time). Early detection
  (ProcessWow64Information query) stays; only a flag is set early.

**[FIXED — M2 PASS on-box, 2026-07-10] csrss died (BUGCHECK 0xF4) when hello32 connected to
basesrv.** Once kernel32 DllMain completes, hello32's `CsrClientConnectToServer`
to basesrv (`CliCon ServerId=2 InfoSize=1664`) makes **csrss raise exception
0x6BA (`RtlRaiseException`, ntdll+0x6cf8) inside a CSR API handler**, and with
DUMMY_PSEH the handler's `_SEH2_TRY` is dead code → the exception is unhandled →
csrss (a critical process) dies → BUGCHECK 0xF4 (0x3, csrss EPROCESS, ...).
Serial signature: `[UEXC] Code=0x6BA ... Proc=csrss.exe` then
`0xF4 (0x3, <csrss>, ...)`. This is a csrss/basesrv wow64-connect-handling bug,
amplified by DUMMY_PSEH swallowing being absent in the CSR API loop. Next-step
debug. RULED OUT: the `PebAddressWow64=0` mismatch — kernel32 now supplies the
real PEB32 in the CSR create message (proc_create.rs, queries
ProcessWow64Information; the process reports INTEL arch + a valid PEB32), and the
0x6BA STILL fires identically. So it is NOT the null-PEB32 path. The raw
[UEXC-BT] LRs are UNRELIABLE (arm64 has no frame pointers here — lr0 symbolized
to unrelated console code), so the next step genuinely needs a LIVE gdb unwind:
break on `RtlRaiseException` in csrss (or on the kernel's user-exception KILL
path) once hello32 connects, and read the real stack to find who raises 0x6BA
and why. Candidate: the CSR API request loop's per-message SEH is dead under
DUMMY_PSEH, so ANY handler that raises kills csrss — the fix may be a
DUMMY_PSEH-safe guard around CSR handler dispatch (swallow + reply error)
rather than fixing the specific raise. Live tracing found that the raise was
not the CSR connect handler: usersrv was handling the loader's
`STATUS_APP_INIT_FAILURE`, and its best-effort `UserpLogHardError` called the
unavailable EventLog RPC endpoint. The expected RPC exception killed csrss
under DUMMY_PSEH. Arm64 now skips only that optional event-log copy.

Two later blockers were then exposed and fixed: the native loader interpreted
the i386 optional header with the native 64-bit layout and falsely entered its
relocation path (`STATUS_INVALID_IMAGE_FORMAT`), so WoW64 images now leave
relocation to the guest loader; and wow64.dll allocated its exit stub after
`BTCpuProcessInit`, too late for FEX's initial address-space discovery. Guest
pages are now allocated before ProcessInit. Current-thread
`RtlWow64Get/SetThreadContext` shims copy the CPU-area context (cross-thread
kernel backing remains owed). Verified serial oracle:
`[FEX-WOW64C] M2 PASS: real i386 image code executed under FEX in a WoW64 process`
and hello32 exit `0x600D`. Whole M2 path remains behind VIBEOS_FEX staging.

*Milestone 3 — the real 32-bit loader runs under FEX.*

*M3a — PASS on-box (2026-07-10): a real ntdll32 syscall stub round-trips.* The
i386 syscall stubs (`syscalls.inc` STUBCODE_U) now dispatch through a
`Wow64Transition` data slot (Win10 parity): statically it jumps
`SharedUserData->SystemCall` (native boots unchanged), and wow64.dll patches the
guest ntdll32's slot to FEX's BOP bridge. The guest exit stub calls the *real*
ntdll32 `NtClose(NULL)` stub and gets `STATUS_INVALID_HANDLE` back through
`Wow64Transition → BOP → Wow64SystemServiceEx → generated thunk table → host
NtClose`. wow64gen is now indexed by `ntoskrnl/include/sysfuncs.h` order (the
real service numbers, not `.spec` order) and its output is built + dispatched by
wow64.dll (`wow64_thunks.c`); manual-list slots route to hand marshallers in
`wow64_manual.c` (`Wow64ManualDispatch`).

*M3b — IN PROGRESS (2026-07-10): the 32-bit loader runs, blocked in the guest
heap.* Instead of jumping straight to the image entry, the guest now enters at
`ntdll32!KiUserApcDispatcher → LdrInitializeThunk` (the real loader) with a
down-converted 32-bit `RTL_USER_PROCESS_PARAMETERS` (the kernel leaves PEB32's
NULL). The loader runs its whole early sequence correctly —
`NtQueryVirtualMemory`, `NtQueryInformationProcess(ProcessBasicInformation)`,
IFEO `NtOpenKey`, heap `NtAllocateVirtualMemory ×N`, locale, `\KnownDlls`
dir+symlink+query — every syscall returning `STATUS_SUCCESS` through 11 hand
marshallers (VM in/out-cell trio, the OA-open family, PBI/SBI down-convert,
symlink in/out UNICODE_STRING). **Blocker:** the loader then near-null-writes
building the main image's `LDR_DATA_TABLE_ENTRY` because
`RtlAllocateHeap(ProcessHeap, 8, 0x50)` returns **NULL** (ntdll32 RVA 0x5153
`test eax,eax; je 0x5172` with `edi=0`). RULED OUT via a guest probe stub: FEX
memory is coherent — the guest reads hello32's DOS header at 0x400000 correctly
(`e_magic=0x5A4D`, `e_lfanew=0x78`), matching the file byte-for-byte; and the
FEX-source investigation confirms FEX shares the host address space flat (base 0,
no translation; `BTCpuProcessInit` self-registers the main image from
`Peb->ImageBaseAddress` at Module.cpp:545 — no missing BTCpu call). So the bug is
in the **guest heap** (RtlCreateHeap/RtlAllocateHeap free-list or a
TEB32/PEB32-dependent field the heap reads) running under emulation, *not* the
CPU or memory layer. Next: trace the guest process-heap creation
(PEB32->ProcessHeap / the cached heap-handle global at ntdll32+0x9bb00) — likely
needs guest-side ntdll32 DbgPrint instrumentation or a FEX-context GDB stop.
Landed as `wip(wow64): M3b`; behind VIBEOS_FEX, normal boots unaffected.

*Still owed in Phase C:*
- 32-bit ntdll auto-map + `Wow64Transition` publishing (32-bit ntdll's syscall
  stubs dispatch through it → BOP page); ntdll32 syscall-stub macro swap. **DONE
  in M3a** (`Wow64Transition` slot + wow64.dll patch).
- PEB32 NLS pointers (need a below-4GB NLS view; left 0 in M1),
  `RtlWow64Get/SetThreadContext` kernel `ThreadWow64Context` support,
  `PebAddressWow64` in the csrss create message.
- Full SysWOW64 set staged from build-i386; minimal system32→SysWOW64
  filesystem redirection late in the phase.

**Phase D — thunk generation at scale.** The hand table in B/C covers hello
world; real apps need hundreds of Nt* thunks + wow64win.dll for win32k.

*Tooling built (2026-07-10): `tools/wow64gen/`.* A re-runnable, deterministic
generator (spec2def-class, NOT bindgen/rustconv — it reads a `.spec` *table*
and emits C marshallers; no LLM). It parses `dll/ntdll/def/ntdll.spec` and emits
one 32→64 thunk per `Nt*`: read the guest 32-bit arg dwords, widen to the host
64-bit ABI (a 32-bit guest pointer IS a valid 64-bit pointer — VA < 4GB — so
`ptr` widens by zero-extension), call the real 64-bit `Nt*`, return NTSTATUS.
Current arm64 surface: 296 syscalls, 260 auto / 36 manual (indexed by
`sysfuncs.h` service order). Deep-marshalling cases (structs whose 32/64 layout
differs — OBJECT_ATTRIBUTES, IO_STATUS_BLOCK Information, ...) can't be inferred
from the `.spec`, so a curated `tools/wow64gen/manual.txt` forces them to loud
`WOW64_NEEDS_MANUAL` stubs (never a silently-wrong auto thunk). Tests + docs:
`test_wow64gen.py` (gated by `vibeos-cli check-tooling`), `README.md`. **DONE
(2026-07-10):** generation is wired into the wow64.dll build (CMake custom
command) and dispatched via `Wow64DispatchSystemService` (`wow64_thunks.c`);
service-number alignment solved by keying on `sysfuncs.h` (not `.spec`) order;
11 of the 36 deep marshallers hand-written in `wow64_manual.c` (the loader path).
*Owed:* the remaining ~25 marshallers (file/section/object-create family —
`NtCreateFile`/`NtOpenFile`/`NtCreateSection`/`NtMapViewOfSection`/`NtQueryDirectoryFile`/
the `NtCreate*`/`NtOpen*` object family) for a real app with DLL imports — these
are independent + mechanical and are the ideal **subagent fan-out** target. This
is where the Rust story pays for wow64win.dll (win32k) too.

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
