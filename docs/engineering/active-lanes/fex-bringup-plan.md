# FEX Bring-up Plan — x86 Windows binaries on ARM64 vibeos

Last updated 2026-07-07 (lane `fex-arm`). Companion to `FEX-ARM-SPIKE-BRIEF.md` (the
mission brief) and `docs/product/vibeos-roadmap.md` §8 (the vision this refines).
This doc records the day-one readiness audit + upstream research and the resulting
ladder. Update it as rungs land.

## Executive summary

- **Target: 32-bit x86 guest first** (`libwow64fex.dll` path), not x64. The WOW64
  module builds with plain `aarch64-w64-mingw32` — our existing llvm-mingw 19.1.7
  works as-is. The x64 guest path (`libarm64ecfex.dll`) requires the ARM64EC-capable
  bylaws llvm-mingw fork and the much hairier ARM64EC ABI — defer it. x86-32-first
  also aligns with the strategic payoff (roadmap §9: retire native i386).
- **JIT memory is fully green** (audited): RWX alloc, protection toggling,
  `NtFlushInstructionCache` with a real arm64 `dc cvau`/`ic iallu` sweep.
- **The critical-path engineering is the arm64 user-mode exception machinery**, not
  FEX itself. Three audited gaps: `.xdata` unwind decoding is a leaf-only stub,
  dynamic function tables are no-op stubs (amd64 impl exists to port), VEH is present
  but never dispatched. The fault-delivery proof (brief's day-one risk) forces this
  work immediately, and all of it is FEX-independent, arm64-smoke-gateable, and
  valuable to the OS regardless.
- **FEX does NOT need Wine.** Wine-specific paths are runtime-gated on
  `wine_get_version` and skipped on a real NT host. It sticks to the xtajit BTCpu
  contract + a modest ntdll import surface, ships its own freestanding mini-CRT over
  ntdll (no msvcrt import), and statically links libc++/libunwind.
- **FEX does no syscall translation.** Guest syscalls hit a "BOP" opcode and exit to
  wow64.dll's `Wow64SystemServiceEx`. The thunk layer is entirely ours (Rust-able),
  and `ntkrnl-sys` bindgen already has per-arch layouts for a future generator.

## Upstream facts (researched 2026-07-07, sources in git history of this doc)

- **License:** MIT. Vendored externals permissive (vixl Apache-2.0, fmt, rpmalloc,
  SoftFloat-3e BSD, unordered_dense, xxhash, zydis, range-v3, tiny-json, cephes).
  jemalloc is Linux-only and force-disabled on MinGW; rpmalloc ("FEX allocator")
  force-enabled. Tracy is a fatal error under MinGW. Catch2/test-bins only with
  `BUILD_TESTING`.
- **Build:** `cmake -DCMAKE_TOOLCHAIN_FILE=Data/CMake/toolchain_mingw.cmake
  -DMINGW_TRIPLE=aarch64-w64-mingw32 -DENABLE_LTO=False -DBUILD_TESTING=False
  -DENABLE_JEMALLOC_GLIBC_ALLOC=False` → `ninja wow64fex`. CI-added flags:
  `-DENABLE_ASSERTIONS=False -DTUNE_ARCH=generic -DTUNE_CPU=none` (the last also
  dodges `.seh directives` assembler errors). `-DUSE_PDB_DEBUGINFO=True` for PDBs.
  Link is `-static -nostdlib -nodefaultlibs -lc++ -lc++abi -lunwind`,
  `--file-alignment=4096`. Skip the `patch_library_wine()` post-step (Wine builtin
  signature stamp) — not wanted on vibeos.
- **Source layout:** `Source/Windows/{WOW64,ARM64EC,Common,Defs}`. `Common/` holds
  the platform layer: mini-CRT (`Common/CRT/`), SMC `InvalidationTracker`, TSO
  handler config, JIT guard pages, allocator. `Defs/ntdll.def` + `Defs/wow64.def`
  define the host import surface (dlltool implibs; only referenced symbols import).
- **BTCpu exports of libwow64fex.dll** (the contract our wow64.dll drives):
  `BTCpuProcessInit/Term, BTCpuThreadInit/Term, BTCpuSimulate,
  BTCpuResetToConsistentState, BTCpuGetContext/SetContext, BTCpuGetBopCode,
  BTCpuSuspendLocalThread, BTCpuIsProcessorFeaturePresent,
  BTCpuUpdateProcessorInformation, BTCpuNotifyMemoryAlloc/Protect/Free/Dirty,
  BTCpuNotifyMapViewOfSection/UnmapViewOfSection,
  BTCpuNotifyProcessExecuteFlagsChange, BTCpuNotifyReadFile,
  BTCpuFlushInstructionCache2, BTCpuFlushInstructionCacheHeavy`
  (+ Wine-only `__wine_get_unix_opcode`).
- **Exception model (corrects an assumption):** FEX does not primarily use VEH. The
  native host exception is caught by **wow64.dll** (dispatcher-side hook) which calls
  `BTCpuResetToConsistentState(EXCEPTION_POINTERS*)`; FEX handles AV (SMC
  invalidation, JIT guard page) and DATATYPE_MISALIGNMENT (TSO unaligned-atomic
  backpatch) there, reconstructs the WOW64_CONTEXT, and guest-visible exceptions go
  to the guest handler. Since we author both ntdll and wow64.dll, we can wire this
  hook directly into our dispatch path.
- **Host NT APIs FEX's WOW64 module actually calls:** `NtAllocateVirtualMemory`
  (incl. **below-2GB** placements for BOP/trampolines — needs ZeroBits support),
  `NtProtectVirtualMemory`, `NtQueryInformationThread(ThreadBasicInformation)`,
  `NtSuspendThread`/`NtResumeThread`, `NtGetContextThread`, `NtContinue`,
  `RtlCaptureContext`, and the WoW64 Rtl family: `RtlWow64GetCurrentCpuArea`,
  `RtlWow64Get/SetThreadContext`, `RtlWow64SuspendThread` (+ cross-process work
  list APIs, gated off under `WOW64_CPUFLAGS_SOFTWARE` — stub-able).
- **Driving FEXCore without wow64.dll:** public API
  (`FEXCore/include/FEXCore/Core/Context.h`): `CreateNewContext`, `InitCore`,
  `CreateThread(InitialRIP, SP, CPUState)`, `ExecuteThread`, `CompileRIP`.
  **FEXOfflineCompiler** is the one standalone FEXCore consumer upstream builds
  under MinGW — ideal first on-vibeos smoke vehicle (JIT-compiles x86 blocks, no
  wow64, no execution glue).

## Readiness matrix (audited 2026-07-07, this tree)

| Area | Status | Evidence |
|---|---|---|
| RWX alloc + protect + `NtFlushInstructionCache`/`KeSweepICache` | **GREEN** | `ARM3/virtual.rs:2041,3171,3121`; `ntkrnl-sys/src/inline/ke.rs:242` |
| Kernel→user fault delivery (arm64) | **GREEN** | `ke/arm64/stubs.rs:1080-1140` → `dll/ntdll/dispatch/arm64/dispatch.S:52` (real `RtlDispatchException`, no DUMMY swallow) |
| arm64 user unwind: `.xdata` decode | **GAP (P1)** | `rtl-except/src/arm64/except.rs` — leaf-only stub (`Pc = Lr`), packed `Flag==2` returns no handler |
| Dynamic function tables (arm64) | **GAP (P2)** | `arm64/except.rs:723-749` no-op stubs; port `rtl-except/src/amd64/dynfntbl.rs` |
| VEH dispatch (arm64) | **GAP (P3)** | impl exists (`rtl-diag/src/vectoreh.rs`) but `RtlDispatchException` never calls it (librtl builds `_NTOSKRNL_`) |
| uwtable for user-mode Rust PEs | **GAP (P0)** | `ntdll-cargo/*.json` etc. still `requires-uwtable: false` — the pseh lane fixed only kernel targets; user-mode `.pdata` walk dies at first ntdll Rust frame |
| TEB via x18, TLS dir + callbacks, `NtCreateThread` | GREEN | `ke/arm64/thrdini.rs:97`, `ldr/ldrutils.rs:506`, `ps-rust/src/thread.rs:278` |
| `NtCreateThreadEx`, `NtWaitForAlertByThreadId`, `RtlWaitOnAddress` | MISSING (shim-sized) | absent from tree |
| `RtlWow64*` family, wow64.dll, BTCpu host side | MISSING (greenfield) | roadmap §8's "partial wow64 skeleton" claim is wrong for this tree |
| ZeroBits / below-2GB user allocs on arm64 | UNVERIFIED | needed for BOP page; check `NtAllocateVirtualMemory` ZeroBits handling |
| Foreign-arch PE mapping | Permissive (good) | `section_pe.rs:951` records Machine, doesn't reject; `LdrpValidateImageForMp` is a stub |
| C++/libc++ PE on arm64 | Toolchain-ready, runtime-unproven | full libc++/libunwind in llvm-mingw; no in-tree arm64 C++ EH precedent (coupled to unwind gaps) |

## Rung 0 results (2026-07-08, lane `fex-arm`)

Rung 0 (0a–0e) is **implemented and validated on arm64**. The exception-machinery
work landed clean and the fault-delivery proof ran on a real boot:

- **0a** uwtable flipped for `ntdll-cargo` (arm64+amd64) — user-mode Rust frames now
  ship `.pdata`.
- **0b** real arm64 `.xdata` unwind-code interpreter + a correct packed-canonical
  unwind (`rtl-except/src/arm64/unwind_decode.rs`, rewritten `RtlVirtualUnwind`).
- **0c** dynamic function tables ported to arm64 (`rtl-except/src/arm64/dynfntbl.rs`),
  consulted by `RtlLookupFunctionEntry` on a static miss.
- **0d** first-chance + continue VEH dispatch wired into arm64 `RtlDispatchException`
  (user-mode-gated); `RtlpCallVectoredHandlers` hardened against a pre-init list.
- **A real dispatch bug fixed**: arm64 `RtlDispatchException` set
  `DispatcherContext.ControlPc` to the *original fault PC* instead of the *current
  frame's* PC, so `__C_specific_handler` searched every non-faulting frame's scope
  table with the wrong RVA and no `__except` ever matched. Now captures per-frame
  `FrameControlPc`. This was the load-bearing fix that made user-mode SEH work.

**Probe 1 — table-based SEH dispatch (software raise → `__try/__except`): PASS,
boot green, ARMED.**
`[SEH-POC-U] table dispatch (RtlRaiseException): caught=1 code=0xe0dead02 (PASS)` on
the arm64 boot serial. Full user-mode chain through the new decoder:
`RtlLookupFunctionEntry` (.pdata) → `RtlVirtualUnwind` (packed + xdata) →
`__C_specific_handler` scope match → filter → `RtlUnwindEx` → `NtContinue` to the
`__except` handler. **This is the FEX-critical path** (guest SEH, C++ EH via libc++,
exception tables).

**Second real dispatch bug found & fixed (the earlier "smss hangs after the handler"
wedge):** arm64 `RtlUnwindEx` restored **`ContextRecord`** — the *raise-point*
context — instead of syncing it to the virtually-unwound state, so the `__except`
landing pad resumed with the deep callees' nonvolatile `x19..x28/fp` (smss later
`ret`'d into `&LdrpProcessInitialized` — an ntdll `.data` address a caller had held
in a callee-saved register — and died, 0x6A). The amd64 reference (`unwind.rs:916`)
does `*ContextRecord = UnwindContext` after every successfully-unwound frame; the
arm64 port had dropped exactly that line. Restored (with the correct
one-frame-behind semantics at the target break). Diagnosed with a callee-saved
snapshot harness around the probe (`SehPocCaptureCalleeSaved` in `seh_poc_user.c`,
kept as a tripwire — any post-unwind register delta prints `[SEH-POC-U] CORRUPT`).

**Probe 2 — VEH on a hardware AV: PASS, boot green, ARMED.**
`[SEH-POC-U] VEH on hardware AV: fired=1 resumed=1 (PASS)`. The kernel delivers the
hardware access-violation first-chance to user mode, the VEH fires, advances PC past
the faulting store, and `NtContinue` resumes with **full integer state**. The
formerly-blocking NULL-`KEXCEPTION_FRAME` wedge is fixed at the vector level:
`ntoskrnl/ke/arm64/trap.S` now has `EXFRAME_SAVE`/`EXFRAME_RESTORE` macros around
`KiSynchronousExceptionHandler` (both EL1 and EL0 sync entries) — callee-saved
x19..x28 are captured into a real `KEXCEPTION_FRAME` below the trap frame, passed as
the handler's second argument, threaded through all four `KiDispatchException` call
sites in `trap.rs`, and re-loaded after the handler returns (which also makes the
in-kernel handled-continue and the Rd>=19 faulting-load fixups actually take
effect). `KeTrapFrameToContext`/`KeContextToTrapFrame`/`KiExceptionExit` already
supported the frame — capture was the only gap.

**Also fixed while validating (build-system):** cargo-module cmake DEPENDS didn't
track the cc-rs `c_shims` C sources (only kernel `.S`/`.inc`), so a
`seh_poc_user.c` edit shipped a **stale ntdll.dll** with zero signal — the same
masking class as the 2026-07-04 kernel-asm incident, one file-type over. The glob in
`sdk/cmake/gcc.cmake` (`_REACTOS_CARGO_KERN_SRCS`) now includes `dll/ntdll/*.c` +
`sdk/lib/{pseh,rtl,crt}/*.c`.

**Net: rung 0 is DONE.** Both probes pass on a green-to-desktop arm64 boot and stay
armed as permanent smss-gated tripwires (a regression prints FAIL/CORRUPT on serial
or kills smss — the arm64 smoke catches either). The fault-delivery "day-one risk"
from the brief is killed: vibeos can deliver a synthesized *and* a hardware fault
into a controlled user context, run VEH + table-based handlers through Rust ntdll
frames, and resume/land with correct full register state.

### Owed follow-ups (rung 0, tracked)

1. ~~Kernel arm64 `KEXCEPTION_FRAME`~~ — **DONE 2026-07-08** (trap.S
   EXFRAME_SAVE/RESTORE; probe 2 green).
2. ~~Decoder post-unwind correctness~~ — **DONE 2026-07-08** (RtlUnwindEx
   ContextRecord sync; probe 1 armed + boot green; the "early-init unsafe"
   hypothesis was wrong — it was the missing sync all along).
3. **verify_signatures / scan_silent_failures baselines** — refresh for the new
   arm64 files + line shifts (both are non-blocking WARNs today).

## The ladder (revised; each rung gates arm64 smoke green)

**Rung 0 — exception-machinery bring-up + fault-delivery proof (start now, FEX-free).**
Builds directly on the `dummy-pseh-question` landing (kernel `requires-uwtable`
fix + kernel SEH PoC — field manual §9.27):
- 0a. uwtable sweep for the user-mode target jsons this lane needs (at least
  `ntdll-cargo`, arm64) — the pseh lane's declared "owed sweep."
- 0b. Real arm64 `.xdata` unwind-code decoding in `rtl-except/src/arm64/except.rs`
  (per the ARM64 exception-handling spec; amd64 unwind.rs is the structural model).
- 0c. Port `dynfntbl.rs` to arm64 + hook `RtlLookupFunctionEntry`.
- 0d. Wire `RtlCallVectoredExceptionHandlers` into arm64 `RtlDispatchException`
  for user-mode builds.
- 0e. **The proof**: a user-mode arm64 PoC exe (native clang `__try/__except`,
  `-fms-extensions` — the seh_poc.c recipe, user-mode sibling): deliberate AV +
  software raise, handler must run and continue; also a VEH variant. Run at boot or
  as a test app; this is the go/no-go the brief demands, and it exercises 0a–0d.

### Rung 1 progress (2026-07-08)

- Upstream **FEX-2607** (`1cc4b93e7a71`) vendored PRUNED at `vendor/fex/`
  (~58MB vs 1.4GB full — test corpora + gated-off externals dropped; prune list
  + provenance: `external/README-fex-vendor.md`). Tree is byte-identical to
  upstream — no vibeos patches. NB upstream's own `CLAUDE.md` forbids
  AI-generated code in *contributions to FEX* — the vendor tree therefore stays
  strictly unmodified, and any future fix goes upstream by a human or lives
  outside the tree.
- **Checkpoint 1 HIT, zero patches:** `libwow64fex.dll` (4.6M) +
  `FEXOfflineCompiler.exe` (3.2M) build with our llvm-mingw (clang 19.1.7,
  `aarch64-w64-mingw32`) via `tools/fex/build-fex.sh` (out-of-band cmake+ninja,
  ~2 min). Proven from the pruned vendor tree.
- `libwow64fex.dll` imports **ntdll.dll + wow64.dll only** (the brief's minimal
  surface, confirmed). `FEXOfflineCompiler.exe` imports KERNEL32/ADVAPI32 + UCRT
  api-sets — for the on-box run (checkpoint 2) vibeos needs `ucrtbase.dll` +
  apiset resolution: our ntdll's `ApiSetResolveToHost` resolves by default
  (version mask ~0), `ucrtbase.dll` builds clean on arm64 (`ninja ucrtbase`) but
  isn't shipped in the arm64 ramdisk yet. Remaining for checkpoint 2: ship
  ucrtbase (+ transitive deps) on the arm64 media, pick an exec vehicle with
  serial-visible output, give FEXOfflineCompiler an input to JIT.

**Rung 1 — vendor FEXCore, build the PE out-of-band.** Vendored snapshot (MIT) with
required externals at a clearly-unmodified home (proposal: `external/fex/`,
top-level, since it's neither sdk lib nor a dll of ours yet). Build via its own
CMake + `toolchain_mingw.cmake` wrapped in a thin script (do NOT merge into the
ReactOS cmake graph yet). Checkpoints: `wow64fex` + `FEXOfflineCompiler.exe` link.
Then **run FEXOfflineCompiler on vibeos arm64** — first proof that a freestanding
libc++ PE with FEX's mini-CRT runs against *our* ntdll, and that the JIT compiles
x86 blocks on-box.

### Rung 2 progress (2026-07-08) — harness built + launches on-box; wedges pre-main

- **Harness authored + links:** `tools/fex/hello-compute/Main.cpp` (+ standalone
  `CMakeLists.txt`, `tools/fex/build-hello-compute.sh`) — a native-arm64 PE
  `fexprobe.exe` (4.5M) that drives the FEXCore Context API directly
  (`Config::Initialize` → `CreateNewContext` → dummy signal/syscall handlers →
  `InitCore` → `EnableExitOnHLT` → RWX blob → `CreateThread(RIP,SP)` +
  64-bit-long-mode GDT → `ExecuteThread` → read back guest RAX). Blob is
  `mov rax,0x2A; add rax,7; hlt`; PASS = RAX reads 0x31. Output via ntdll
  `DbgPrint` → PL011 serial (`[FEX-HELLO]`). Links clean against the prebuilt
  vendored FEXCore archives with our llvm-mingw — first proof the full FEXCore
  link surface resolves into a *standalone* arm64 exe (not just the wow64 dll).
- **Run vehicle wired (doom-pattern):** `winlogon-rust` `TryStartFexProbe()`
  spawns `fexprobe.exe` if staged (gated on `GetFileAttributesW`, silent no-op
  otherwise), `WaitForSingleObject(30s)`. `mkramdisk.sh` stages
  `fexprobe.exe` + `ucrtbase.dll` under `VIBEOS_FEX=1` (off by default).
  `ucrtbase.dll` builds clean on arm64.
- **On-box status: LAUNCHES, wedges pre-`main`.** With `VIBEOS_FEX=1` the boot
  reaches desktop green and `fexprobe.exe` starts as PID 204 —
  `PspUserThreadStartup Image=fexprobe.exe` + `kernel32 DllMain PROCESS_ATTACH
  PID=204` — but never prints `kernel32 ... done OK PID=204`, never reaches the
  first `[FEX-HELLO]` line (the first thing `main` does), and never exits. So it
  wedges during **DLL/CRT init, before `main`** — prime suspects: `ucrtbase.dll`
  DllMain/CRT init, or FEX's static global constructors (rpmalloc/jemalloc
  allocator init, libc++ statics) running against our loader. Imports beyond
  ntdll: KERNEL32/ADVAPI32 + the UCRT api-sets (`ApiSetResolveToHost` resolves
  these by default).
- **Localized (2026-07-08) with a global-ctor tripwire** (`[FEX-HELLO-CTOR]`,
  prints from a static ctor before `main`): it does **not** fire, so the wedge
  is *before* our global ctors. Serial shows fexprobe's kernel32 DllMain
  PROCESS_ATTACH **connects to CSR successfully** (`[CSRSRV] CliCon ServerId=1/2`
  for PID 204) but never reaches `[kernel32] DllMain PROCESS_ATTACH done OK
  PID=204` — so the hang is inside kernel32 DllMain *after*
  `CsrClientConnectToServer` (CsrNewThread / BaseDllInitializeMemoryManager /
  NLS / `ConDllInitialize` console setup), in code shared by every process, yet
  only fexprobe hangs (cmd, nettest pass the same path). Prime remaining
  suspects: `ConDllInitialize` console attach for this import/TLS shape, or a
  CsrNewThread thread-startup interaction. Next: GDB-toolkit backtrace of the
  wedged PID-204 thread (`qemu/start.sh arm64 --gdb`, `VIBEOS_FEX=1` media).

### Rung 2 — remaining blocker (2026-07-08): kernel32 synch export surface + WaitOnAddress

After the three fixes above, fexprobe loads, runs CRT + libc++ static init, and
faults calling `AcquireSRWLockExclusive` — IAT poison `0xffbadd11` (ntdll
LdrpSnapThunk's arm64 tolerant "entrypoint not found"). Root cause is an OS
export-surface gap, NOT a fexprobe bug:

- fexprobe imports the `api-ms-win-core-synch-l1-2-0` set → (apiset) → kernel32.
  It needs `Sleep, WaitForSingleObject` (base kernel32) **plus** SRW locks,
  condition variables, `InitOnceExecuteOnce`, `SleepConditionVariableSRW`
  (Vista) **plus** `WaitOnAddress, WakeByAddressAll/Single` (Win8).
- vibeos splits kernel32: **base `kernel32.dll` (0x502)** has Sleep/Wait* but
  gates out the Vista synch (`-version=0x600+`); **`kernel32_vista.dll`** has the
  SRW/condvar/initonce but NOT Sleep/Wait*. Neither is a superset, so pointing
  the apiset at either leaves poison for the other half.
- `WaitOnAddress`/`WakeByAddressAll`/`WakeByAddressSingle` are **missing
  entirely** — not in either spec, and our ntdll has no `RtlWaitOnAddress`/
  `RtlWakeAddress*`. This is unimplemented synchronization primitive surface.

This blocks ALL FEX exes (rungs 2/3/4 and rung-1 checkpoint 2) since FEX's libc++
uses these threading primitives.

### Rung 2 — synch export surface RESOLVED (2026-07-09): real (Vista-scoped) kernelbase + kernel32 split-surface + CNTKCTL

The blocker split into three independent OS gaps, all fixed. fexprobe now loads
fully, runs CRT + libc++ static init + global ctors, enters `main`, drives
FEXCore setup, and reaches guest-blob execution (`[FEX-HELLO] executing guest
blob @ ...`). arm64 still boots to desktop (`shell ready`).

1. **The missing primitive — `RtlWaitOnAddress`/`RtlWakeAddress{All,Single}`.**
   Implemented in the rtl-sync crate (`sdk/lib/rust/rtl-sync/src/waitaddr.rs`):
   anonymous NT keyed-event + 256-way address-hash bucket table, each waiter
   parked on a UNIQUE key (its own stack entry) so keyed-event pairing is exact
   even through the timeout race (waker/waiter serialize on the bucket spin lock
   + a `Signaled` flag; releases issued OUTSIDE the lock). Corrected for the
   stricter WaitOnAddress contract — a wake is NEVER lost (unlike condvar, whose
   callers re-test a predicate). Exported from base `ntdll.spec` (stripped at
   0x502) AND `ntdll_vista.spec` (the real 0x600 export home; gated
   `any(cabi, vista)` + re-exported via `rtl-vista-rust`).

2. **The apiset surface — real (minimal) `kernelbase.dll`.** Chose option 1
   (real kernelbase) scoped to Vista: a deliberately tiny `dll/win32/kernelbase`
   (NOT the 1746-export wine tree, still stubbed) owning the modern synch
   surface. Almost pure forwarders (condvar/InitOnce → kernel32_vista;
   WakeByAddress* → ntdll_vista.RtlWakeAddress*; Sleep → kernel32); only
   `WaitOnAddress` has local code (DWORD-ms → LARGE_INTEGER shim over
   ntdll_vista!RtlWaitOnAddress). `api-ms-win-core-synch-l1-2-0` re-pointed at it
   (`apisets.table.c`; the arm64 Rust resolver regenerates from that same file).

3. **The split kernel32 surface — the direct-by-name imports.** fexprobe imports
   the SRW `*Exclusive` trio + 4 Vista file APIs (`GetFileInformationByHandleEx`,
   `GetFinalPathNameByHandleW`, `CreateSymbolicLinkW`, `SetFileInformationByHandle`)
   *directly by the name* `KERNEL32.dll` — no apiset/kernelbase redirect can touch
   those, and base kernel32 (0x502) version-gates them out. Exported exactly those
   7 in `kernel32.spec` **`-arch=arm64`**, forwarding SRW → `ntdll_vista.Rtl*` and
   the file APIs → `kernel32_vista.*` (SetFileInformationByHandle an `-arch=arm64
   -stub`).

   **Why `-arch=arm64`, not a plain ungate (a gate-caught regression):** first
   attempt dropped the `-version=0x600+` gate outright. arm64 booted + fexprobe
   ran, but **i386 AND amd64 BUGCHECK 0x4C `(0xc0000139` = STATUS_ENTRYPOINT_NOT_
   FOUND`)` deterministically at csrss's `LoadServerDll winsrv`.** Root cause:
   spec2def assigns export ordinals **sequentially over the *included* entries
   only** (`Name=... @N`), and `DLL_EXPORT_VERSION` is `0x502`, so those 7 were
   version-excluded on x86. Un-gating *inserts* them at their alphabetical
   positions → **shifts every later export's ordinal by +1..+7** → breaks an
   x86 boot DLL that binds kernel32 **by ordinal**. arm64 tolerates the shift (no
   by-ordinal kernel32 importers). Since fexprobe is arm64-only, gating the 7 to
   `-arch=arm64` leaves the x86 kernel32 ordinal table byte-identical to master
   (fix confirmed: i386 back to green) while arm64 still exports them. (x86 FEX,
   if ever wanted, must instead pin explicit ordinals or find+rebuild the
   by-ordinal importer — do NOT plain-ungate.) See field manual §9.43.

Plus a **kernel** fix (`ke/arm64/kiinit.rs`): set `CNTKCTL_EL1.EL0PCTEN|EL0VCTEN`
so EL0 can read the generic-timer counters. vibeos's own user-mode never reads
them directly, so it was never enabled; FEX's `SpinWaitLock` ctor does
`mrs CNTFRQ_EL0` (was trapping `C000001D` illegal-instruction).

### Rung 2 — COMPLETE (2026-07-09): guest RAX = 0x31 PASS

`[FEX-HELLO] guest RAX = 0x31 (expected 0x31) (PASS)` — FEXCore JIT-compiled
and executed the x86-64 blob on vibeos arm64, clean HLT exit, clean process
exit. The "execution-engine data abort" was TWO stacked harness-side bugs, both
found by symbolication + a first-chance VEH register/insn dumper in the harness
(kernel user-exception dispatch works on arm64 — a VEH gets full CONTEXT):

1. **Reserve-only LookupCache, no overcommit handler.** The first fault
   (`llvm-addr2line` → `FEXCore::LookupCache::FindBlock`) was a read into the
   cache's ~272MB VA region: LookupCache allocates it `MEM_RESERVE`-only and
   calls `SyscallHandler->MarkOvercommitRange()`, expecting the host layer to
   commit pages on first-touch AV (the WOW64/ARM64EC modules wire a fault
   handler to `FEX::Windows::OvercommitTracker`). `DummyHandlers`' version is a
   no-op. Fix (harness): `ProbeSyscallHandler::MarkOvercommitRange` eagerly
   `MEM_COMMIT`s the range — byte-for-byte what FEX's own Wine path does.
   Fault-driven overcommit becomes mandatory at rung 4 (wow64fex uses the real
   OvercommitTracker + exception handler).

2. **`EnableExitOnHLT()` called AFTER `InitCore()`.** The dispatcher is emitted
   inside `InitCore` and reads `ExitOnHLTEnabled()` at emission time; enabling
   it after bakes the disabled path, where guest HLT is lowered to a DELIBERATE
   null-load segfault (`mov w1,#0; ldr x1,[x1]` — "Force a SIGSEGV by loading
   zero", `Dispatcher.cpp` `GuestSignal_SIGSEGV`) for a signal delegator we
   don't have. The VEH dump made this unmissable: guest RAX was ALREADY 0x31 in
   x4 (SRA) and x20 held the HLT's RIP — the guest had fully executed and died
   in the exit path. Fix: set the flag before `InitCore()`.

Reusable lessons: (a) any FEXCore embedding on vibeos MUST provide overcommit
handling for `MarkOvercommitRange` regions; (b) Context config flags that the
dispatcher reads are emission-time — set them all before `InitCore()`; (c) a
first-chance `AddVectoredExceptionHandler` + DbgPrint dumper is the go-to tool
for faults inside FEX's runtime-emitted (unsymbolizable) code.

### Rung 2 wedge — RESOLVED (2026-07-08): three stacked bugs, all fixed

Getting fexprobe to `main()` on-box uncovered three independent bugs, fixed in order:

1. **Console-subsystem hang.** A console-subsystem PE makes kernel32 DllMain run
   `ConDllInitialize` (console attach), which wedged for fexprobe. The harness has
   no console I/O (serial via DbgPrint), so it should be GUI-subsystem. Fix:
   `-Wl,--subsystem,windows -Wl,-e,mainCRTStartup` in the harness link. → DllMain
   reaches `done OK`.

2. **api-set redirection built a bad absolute path** (ntdll loader bug, affects
   ALL binaries importing api-sets). `LdrpApplyFileNameRedirection` resolved
   `api-ms-win-crt-*.dll → ucrtbase.dll` correctly, but then prefixed
   `<NtSystemRoot>\System32\` to force an absolute load — which fails to map with
   `STATUS_OBJECT_NAME_INVALID` on this port. **fexprobe was the first binary in
   the whole OS to import the `api-ms-win-crt-*` api-sets**, so this path had
   never run. Fix: emit the bare target name (`ucrtbase.dll`) and let the normal
   DLL search path resolve it from System32 — how every other import already
   loads (`ntdll-rust rtl/libsupp.rs`). Also unblocks rung-1 checkpoint 2. →
   UCRT imports resolve, no more `Skipping missing import`.

3. **ARMv8.3 LDAPR/RCpc illegal-instruction** (the reusable gotcha). fexprobe
   faulted `C000001D` at an `ldaprb` in a libc++ C++ static-init guard. vibeos
   QEMU runs **`-cpu cortex-a72` (ARMv8.0-A, no FEAT_LRCPC)**, and the kernel
   builds `-march=armv8-a` to match (`sdk/cmake/config.cmake` OARCH). FEX's
   default `TUNE_CPU=native` on the x86→aarch64 cross-compile picks a
   high-baseline `-mcpu` that emits RCpc ops → illegal on cortex-a72. Fix:
   `-DTUNE_ARCH=armv8-a -DTUNE_CPU=none` for the FEX vendor build +
   `-march=armv8-a` for the harness. **Any native-arm64 binary run on vibeos
   must target armv8-a**, not the toolchain default.

Debug instrumentation for all three is preserved gated-off in-tree:
`K32_DLLMAIN_TRACE` (kernel32 dllmain.rs), `APISET_TRACE` (ntdll libsupp.rs),
`[FEX-HELLO-CTOR]` ctor tripwire (harness).

**Rung 2 — "hello compute": execute a static x86 blob.** Minimal native arm64
harness exe driving the FEXCore Context API directly (`CreateNewContext` →
`CreateThread(RIP, SP)` → `ExecuteThread`): RWX buffer, load blob, run to a clean
exit (BOP/HLT), verify computed result over serial. No syscalls, no wow64.
Flushes: thread/TEB integration, below-2GB alloc, any missing ntdll imports.

**Rung 3 — guest fault round-trip. COMPLETE (2026-07-09).** Guest blob AVs
(`mov [0], rax` mid-computation); the harness VEH catches the host AV with PC in
the JIT code buffer and replays wow64fex's `BTCpuResetToConsistentState` recipe
(`ReconstructThreadState`: `RestoreRIPFromHostPC` + SRA GPR/FPR spill via
`SignalDelegator::GetConfig()` mappings + `ReconstructCompactedEFLAGS`), then
redirects guest RIP to a recovery routine and resumes at
`Config.AbsoluteLoopTopAddressFillSRA` (with X1/`ENTRY_FILL_SRA_SINGLE_INST_REG`
zeroed). On-box result: reconstructed guest RIP and RAX byte-exact at the fault
point (`RIP=blob+0xB, RAX=0x31`), recovery executed, `guest RAX = 0x99 PASS`,
clean exit. **OS milestone proven en passant:** vibeos arm64 user exception
dispatch round-trips a *modified* CONTEXT — `KiUserExceptionDispatcher` → VEH →
`EXCEPTION_CONTINUE_EXECUTION` → `NtContinue` restores Pc/X-regs, so guest-SEH /
SMC viability holds. All in `tools/fex/hello-compute/Main.cpp`.

**Rung 4 — syscall-boundary mechanism: PROVEN in-harness (2026-07-09).** The
guest blob now ends `mov rax,0x99; mov rdi,0xC0FFEE; syscall; hlt`. The harness
syscall handler sets `OSABI = OS_GENERIC` (the wow64-style ABI: JIT spills ALL
guest regs to the CpuStateFrame before `HandleSyscall`, refills after, C return
value ignored) and services the trap by reading guest RAX/RDI from state,
writing the result into guest RAX, and advancing `State.rip` past the 2-byte
`syscall` (the JIT reloads rip on return — same contract Wow64SystemServiceEx
dispatch relies on). On-box: `num=0x99 arg=0xc0ffee rip=<syscall insn>` exact,
guest resumes and exits with the handler-written RAX. Full ladder in one guest
run: compute → AV → reconstruct/redirect/resume → syscall thunk → clean exit.
Remaining rung-4 scope (the real deliverable) below.

**Rung 4 — the BTCpu boundary + ONE thunk ("hello world").** Author minimal
`wow64.dll`-side host: load libwow64fex.dll by contract, implement
`Wow64SystemServiceEx` with a tiny hand-written thunk table (e.g. `NtWriteFile` to
serial, `NtTerminateProcess`), plus the `RtlWow64*` context helpers. Guest x86 blob
does a real syscall through the BOP. This layer is ours and Rust-preferred.
After this, the thunk *generator* (bindgen → marshalers) is the scaling follow-on,
and full i386-PE process bring-up (32-bit ntdll in-process, wow64 process init)
is the next big phase.

## Known risks / open questions

- **arm64 `.xdata` decode (0b) is the biggest single engineering item in rung 0** —
  a real unwind-code interpreter, not a stub swap. It's also pure OS-quality work
  (correct user-mode SEH on arm64) independent of FEX.
- FEX-on-real-Windows wow64 note: upstream wiki says replacing xtajit with the
  wow64fex module on *real Windows* currently hangs 32-bit apps — the Wine wow64
  path is the production one. We're authoring the wow64 side ourselves, so we
  follow Wine's semantics (CPU area, notify callouts, exception forwarding), which
  is the validated path.
- ZeroBits/below-2GB support in our Mm on arm64 — verify early (rung 2 blocker).
- Vendor size: FEX + needed externals is tens of MB of C++. Confirm repo appetite
  before committing the snapshot (alternative: submodule).
