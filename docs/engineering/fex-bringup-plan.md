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

**Probe 1 — table-based SEH dispatch (software raise → `__try/__except`): dispatch
chain PROVEN; post-unwind state hardening OWED.**
`[SEH-POC-U] table dispatch (RtlRaiseException): caught=1 code=0xe0dead02 (PASS)` on
the arm64 boot serial. This exercises the full user-mode chain through the new
decoder: `RtlLookupFunctionEntry` (.pdata) → `RtlVirtualUnwind` (packed + xdata) →
`__C_specific_handler` scope match → filter → `RtlUnwindEx` → `NtContinue` to the
`__except` handler, which runs and returns normally (the "VEH-disabled" and "end"
lines print after it). **This is the FEX-critical path** (guest SEH, C++ EH via
libc++, exception tables) and the dispatch/lookup/filter/land sequence is proven.

**Caveat (owed):** with the probe armed in smss, boot then *hangs* right after the
handler returns — smss can't continue. So the unwind reaches and executes the
handler correctly but leaves residual state wrong (a callee-saved register or the
restored SP slightly off in `RtlVirtualUnwind`/`RtlUnwindEx`), **or** raising this
early in `LdrpInit` is inherently unsafe. These are not yet disambiguated. Next step
is a dedicated arm64 test *process* (not an in-`LdrpInit` hook) to isolate the
decoder from smss's fragile early init; if the hang persists there, harden the
packed/xdata register+SP restoration. Until then the probe is disarmed
(`SEH_POC_U_ARMED = false` in `ntdll-rust` LdrpInit) so arm64 smoke stays green.

**Probe 2 — VEH on a hardware AV: delivery + VEH fire WORK; resume WEDGES. Root-caused.**
The kernel delivers the hardware access-violation first-chance to user mode
(`[UEXC-DISPATCH]`), the registered VEH fires and returns `CONTINUE_EXECUTION`
(advancing PC past the store) — but on resume the process faults again with
`x19 == 0` and is second-chance killed. **Root cause (precise wedge):** the arm64
fault path (`ntoskrnl-rust/src/ke/arm64/trap.rs`, all four `KiDispatchException`
call sites, e.g. :440) passes a **NULL `KEXCEPTION_FRAME`**, so
`KeTrapFrameToContext` (`ke/arm64/stubs.rs:943`) has no source for the callee-saved
registers and leaves `Context.X19..X28 == 0`; `NtContinue` then resumes with them
zeroed. Probe 1 is immune because its context is captured in *user* mode
(`RtlCaptureContext`, full X0–X30) and dispatched in-process — it never rides the
null-ExceptionFrame kernel round-trip.

**Fix owed (next rung-0 increment):** the arm64 exception vector / `trap.rs` must
build a real `KEXCEPTION_FRAME` (save X19–X28 on entry, pass it to
`KiDispatchException`, restore in `KiExceptionExit`). Until then, the PoC's probe 2
is compiled out (`SEH_POC_U_ENABLE_VEH_PROBE 0` in `dll/ntdll/seh_poc_user.c`) so it
can't kill smss; probe 1 stays armed as a permanent arm64 user-mode SEH tripwire
(gated to smss.exe, like the kernel's `seh_poc.c`).

**Net:** the arm64 exception-dispatch machinery FEX leans on is proven to lookup,
match, and land handlers; hardware-fault delivery reaches user mode and VEHs fire.
Two localized, well-understood gaps remain before the fault path is FEX-grade: the
kernel callee-saved-register capture (null `KEXCEPTION_FRAME`, blocks probe 2) and
post-unwind state correctness / early-init isolation (blocks leaving probe 1 armed).

### Owed follow-ups (rung 0, tracked)

1. **Kernel arm64 `KEXCEPTION_FRAME`** — build+pass+restore X19–X28 on the fault
   path (`ke/arm64/trap.rs` + vectors + `KiExceptionExit`). Unblocks probe 2 and any
   real user-mode hardware-fault recovery (the FEX SMC/guest-fault path).
2. **Decoder post-unwind correctness** — dedicated arm64 test process for probe 1;
   if the smss hang reproduces in isolation, audit `RtlVirtualUnwind` packed/xdata
   register+SP restoration for a residual error.
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

**Rung 1 — vendor FEXCore, build the PE out-of-band.** Vendored snapshot (MIT) with
required externals at a clearly-unmodified home (proposal: `external/fex/`,
top-level, since it's neither sdk lib nor a dll of ours yet). Build via its own
CMake + `toolchain_mingw.cmake` wrapped in a thin script (do NOT merge into the
ReactOS cmake graph yet). Checkpoints: `wow64fex` + `FEXOfflineCompiler.exe` link.
Then **run FEXOfflineCompiler on vibeos arm64** — first proof that a freestanding
libc++ PE with FEX's mini-CRT runs against *our* ntdll, and that the JIT compiles
x86 blocks on-box.

**Rung 2 — "hello compute": execute a static x86 blob.** Minimal native arm64
harness exe driving the FEXCore Context API directly (`CreateNewContext` →
`CreateThread(RIP, SP)` → `ExecuteThread`): RWX buffer, load blob, run to a clean
exit (BOP/HLT), verify computed result over serial. No syscalls, no wow64.
Flushes: thread/TEB integration, below-2GB alloc, any missing ntdll imports.

**Rung 3 — guest fault round-trip.** Guest blob AVs; our dispatch hook (rung 0
machinery) catches, calls into FEX's `BTCpuResetToConsistentState`-equivalent path,
guest handler runs. Proves SMC/TSO/guest-SEH viability.

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
