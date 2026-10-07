# WOW64 on vibeos — running x86 binaries on arm64 (FEX) and x64 (native)

**Status (2026-07-26): the capability arc is complete on both hosts.** Stock,
unmodified i386 PEs run on arm64 (JIT-emulated, via the vendored FEX CPU layer)
and on amd64 (natively, in x86-64 compatibility mode). They load through the
genuine Windows loader, draw through native win32k, create threads, take
hardware faults into their own SEH, and generate and execute code at runtime.

- **Probe ladder — 9 guests, green on both arches** (`tools/fex/hello-compute/`):
  hello32, gui32, win32, file32, async32, marshal32, thread32, fault32,
  selfmod32.
- **Stock i386 applets on the desktop:** winmine, calc, notepad, sol, charmap,
  mspaint, regedit, winver, taskmgr, wordpad, clipbrd, eventvwr, magnify, osk,
  spider — plus x86 DOOM, rendering and playable.

What remains is depth, not capability: guest *processes* and the marshalgen audit
backlog. Registry redirection was the third item here until E24 landed it. See
**What is deliberately not built**.

Owner: `wow64-amd64` lane.
Predecessor: `fex-bringup-plan.md` (rungs 0–3; this doc was its rung 4+).

## Ground rules

- FEX vendor tree stays unmodified (`vendor/fex/CLAUDE.md` forbids AI-authored
  contributions upstream; we consume its binary contract only). One exception has
  been taken since: `6afc128553f`, recorded in `vendor-patches/fex.md` and marked
  in-source with a `VIBEOS PATCH` comment. That is the bar — a patch here is
  allowed only if it is both places.
- wow64.dll / wow64cpu.dll / ntdll changes are vibeos code.
- Every phase lands behind the existing gates: all three arches stay green; x86
  arches stay untouched by arch-gated exports (§9.43 ordinal discipline).

---

# The as-built design

*This section describes the system as it exists. The chronological phase log
below it records how it got here, and is kept because nearly every entry carries
a bug and its root cause.*

## The layer split, and the decision that shaped everything else

We adopted the Win10+ BTCpu three-layer model because FEX's WOW64 module already
implements the bottom layer's exact contract. As built:

| Layer | What it is here |
|---|---|
| **1. Kernel + 64-bit loader** | PEB32/TEB32 beside the 64-bit ones, i386 image admission, `<2GB` guest allocations, i386 ntdll mapping, the handoff to layer 2. Deliberately small: a handful of Rust files, most of them touched in one or two places. |
| **2. `wow64.dll` — ours** | The whole 32↔64 boundary: syscall thunk tables, marshallers, the callback direction, CPU-area management, exception routing, filesystem redirection. ~330KB of C across 7 TUs, and **exactly three exports**. |
| **3. CPU layer** | `libwow64fex.dll` on arm64 (vendored FEX, binary contract only). `wow64cpu.dll` on amd64 (ours — heaven's-gate mode switch, no emulation; the silicon runs the 32-bit instructions). |

**The decision that paid for itself: layer 2 knows nothing about the host arch,
layer 3 is the only thing that does.** The 32→64 widening problem is a property
of the *guest*, not of the host — a `UNICODE_STRING32` is the same shape whether
the thing that will consume it is a JIT or a CPU in compat mode. So when amd64
came along, the entire marshalling investment transferred untouched: E0 ungated
the shared layer, E1 wrote a CPU layer, and E3 had `hello32.exe` passing. Every
marshalling bug found on one arch was already fixed on the other.

The three exports are the whole interface the CPU layer sees
(`dll/win32/wow64/wow64.spec`): `Wow64SystemServiceEx`,
`Wow64ProcessPendingCrossProcessItems`, `Wow64LdrpInitialize`. Keeping that
surface tiny is what made the second CPU layer cheap: E0 (un-gate) landed
2026-07-17 and E3 (`hello32.exe` passing on amd64) landed 2026-07-18 — the whole
port from "arm64 only" to "a real i386 PE runs on amd64" took two days, because
everything above layer 3 was already arch-neutral and already debugged.

## The bootstrap chain

1. **Kernel admits the image.** `SharedUserData->ImageNumberLow = I386` on the
   64-bit host clears kernel32's `CreateProcessInternalW` range gate. Sections
   and `PspCreateProcess` needed no machine gate — they were already permissive.
2. **Kernel builds the 32-bit process view.** `MmCreatePeb` detects
   `FileHeader.Machine == 0x14c` and creates a PEB32 below 2GB mirrored from
   PEB64, with `EPROCESS.Wow64Process = PEB32` (the 5.2 shape our
   `ProcessWow64Information` handler already returned). Each thread gets one
   combined below-2GB region: TEB64 at +0, TEB32 at +0x2000, plus the raw Win10
   `WowTebOffset` slots at TEB+0x180c / TEB32+0xfdc that FEX's `GetWowTEB`
   hardcodes.
3. **The 64-bit loader diverts.** `LdrpInitializeProcess` queries
   `ProcessWow64Information`, skips the i386 image's import walk / TLS / kernel32
   entirely, and — critically, **after** `LdrpRunInitializeRoutines`, not before —
   hands the thread to `wow64!Wow64LdrpInitialize`, which never returns. (Before
   meant loading a CPU layer that imports kernel32 re-entrantly under the loader
   lock, which hung.)
4. **`Wow64LdrpInitialize` brings up the guest world**, in an order where two
   steps are load-bearing:
   - map `syswow64\ntdll.dll` as `SEC_IMAGE`, publish
     `LdrSystemDllInitBlock.ntdll_handle`, install `WOW64INFO` in TLS slot 10;
   - **allocate every guest page** (exit stub, guest stack) — *before*
     `BTCpuProcessInit`, because that is when the CPU layer sweeps the address
     space to discover what is executable;
   - load the CPU layer, `BTCpuProcessInit`, then **publish the memory
     notification contract** — *after*, because a notification delivered before
     ProcessInit has no address-space model to land in;
   - `BTCpuThreadInit`, build the guest CONTEXT at ntdll32's entry, enter
     `BTCpuSimulate`, which never returns by design.
5. **The guest's own `LdrInitializeThunk` runs**, under emulation, and does all
   the real loader work — imports, relocations, TLS, DllMain. We do not
   reimplement the 32-bit loader; we make the real one able to run.

## Syscall dispatch: two tables, four tiers

The guest's i386 ntdll stubs dispatch through a **`Wow64Transition` data slot**
(Win10 parity). Statically it points at `SharedUserData->SystemCall`, so native
i386 boots are completely unaffected; wow64.dll rewrites it to the CPU layer's
BOP bridge. gdi32/user32 embed their *own* copy of the stubs, so every mapped
image that exports `Wow64Transition` gets patched as it maps.

Guest EAX selects the table. ntdll services carry their raw service number;
win32k services carry the `0x1000` tag bit, and `(n & 0xF000) == 0x1000` routes
to the win32k table (ntdll's max is ~0x127, so there is no overlap).

Both tables are **generated at build time** by `tools/wow64gen/wow64gen.py`,
indexed by real service number — ntdll from `sysfuncs.h` order, win32k from
`w32ksvc32.h` — with the `.spec` contributing argument types only:

| Table | Auto | Manual |
|---|---|---|
| ntdll (`ntdll.spec` × `sysfuncs.h`) | 218 | 78 |
| win32k (`win32u.spec` × `w32ksvc32.h`) | 624 | 113 |

Every service lands in one of four tiers:

1. **Flat auto-thunk** — zero-extend the guest's 32-bit argument dwords and call
   the real 64-bit `Nt*`. Sound because a 32-bit guest pointer *is* a valid
   64-bit pointer (guest VA < 4GB). This covers most of win32k, whose args are
   largely pointer-free `RECT`/`POINT`/`COLORREF`.
2. **Generated deep marshaller** (`wow64_automarshal.c`) — field-by-field in/out
   copies emitted by `marshalgen`, from **clang record-layout diffs** taken at
   `-target i686-w64-mingw32` and `aarch64-w64-mingw32` through the real header
   stack. Layout truth comes from the compiler, never from a human reading a
   struct.
3. **Hand marshaller** (`wow64_manual.c`, 101 functions in ~186KB) — the shapes no layout
   diff can express: embedded pointers, count-aware arrays, filesystem
   redirection, the `CSR_API_CONNECTINFO` width thunk, and rules like
   "`NtContinue` must hand the guest CONTEXT32 to the CPU layer and never to the
   host kernel."
4. **Loud unimplemented** — `WOW64_NEEDS_MANUAL` / `WOW64_MANUAL_UNIMPL`. A gap
   announces itself instead of returning a plausible wrong answer.

**The audit gate (E10) exists because tier 1 is invisible when it is wrong.** A
flat thunk over a divergent payload compiles, links, and silently truncates.
`marshalgen audit` derives from checked-in sources which services provably
diverge 32↔64 and **hard-fails the build** if a flat thunk covers one that is not
in `tools/lint-baseline/marshalgen_audit.txt` (209 entries, being burned down).
It also hard-fails on a stale built table, so the audit cannot be quietly
outrun.

## The callback direction (64→32)

win32k calls back into user mode via `KeUserModeCallback`, which on a WoW64
process targets a **64-bit** dispatcher. So `wow64_callback.c` marshals the
payload *down* into guest memory, enters the guest through the CPU layer, and
copies the OUT payload back *up* on `NtCallbackReturn`. Twenty entries in
`Wow64pUserCallbackTable`; the payload structs are generated the same way as the
syscall marshallers (`wow64_cb_automarshal.c`). This is what makes window
procedures — and therefore every real GUI applet — work.

## The CPU layer's view of the address space

On a JIT host this is a **capability, not a cache hint**. FEX's decoder refuses
to translate any guest range it has not been told is executable
(`CheckRangeExecutable` → `HitNonExecutableRange`), and its executable set is
populated only by notifications plus that one sweep at `BTCpuProcessInit`. Until
E17, everything a guest allocated *after* startup was un-executable — no ATL/MFC
thunks, no VB6/Delphi trampolines, no packers, no JIT.

`wow64_btcpu.h` wires all eight entry points (alloc / protect / free / dirty /
map / unmap / read-file / execute-flags). Two rules the header states because
getting them wrong is silent:

- **`Before`/`After` is a lock/unlock pair, not a status report.** `After=FALSE`
  takes the CPU layer's `ThreadCreationMutex`; `After=TRUE` releases it. An
  unmatched *Before* deadlocks the entire guest process.
- Notifications are **current-process only**, and a NULL entry point is a
  legitimate silent no-op (amd64's `wow64cpu` exports none of these — it has
  nothing to invalidate, which makes it a perfect control for the arm64 work).

## Exceptions, which arrive completely differently per arch

The guest-visible result is identical; the two paths share only the frame
builder.

- **amd64:** a guest fault is a genuine compat-mode trap, caught in
  `Wow64pPrepareForExceptionHook`.
- **arm64:** a guest fault is a *host* arm64 fault inside FEX's JIT.
  `BTCpuResetToConsistentState` gets first refusal, and when it can service the
  fault itself (self-modifying code, overcommit, unaligned atomic, suspend) it
  `NtContinue`s and never returns. When it declines, it hands the fault over *in
  guest terms* — guest registers reconstructed into the CPU area, the
  `EXCEPTION_RECORD` rewritten to describe the guest fault — and
  `Wow64pRedirectGuestFault` takes it from there.
- **Shared:** `Wow64pBuildGuestExceptionFrame` builds the i386
  `KiUserExceptionDispatcher` frame (a `WOW64_EXCEPTION_RECORD32` + CONTEXT32
  pair) on the *guest* stack, so the guest's own `fs:[0]` chain runs.

Two details that decide whether the arm64 half works at all: the handed-over
fault is told apart from a genuine host fault by
`ExceptionAddress < 2GB && ExceptionAddress == CpuArea->Eip` (FEX always rewrites
it to the guest `Eip` before declining), and **breakpoints are deliberately
excluded** — that same test matches a DebugService `int 2d` (reported at the
already-advanced `Eip`) and *misses* a real `int3` (reported at `Eip - 1`), i.e.
exactly backwards. Guest `int3` dispatch is its own change.

## Threads, redirection, and the kernel's share

**Threads.** Each guest thread gets its own CPU area and WoW64 entry path.
`ThreadZeroTlsCell` has to reach TEB32, not TEB64 — a 32-bit guest reads its TLS
through the 32-bit TEB, so writing the 64-bit one zeroes a cell nobody reads.

**Filesystem redirection** happens inside the file marshallers, exactly where
real WOW64 does it: `\system32\` → `\syswow64\`, same length, rewritten into host
scratch. The guest also opens `\KnownDlls32` rather than `\KnownDlls`, because
the latter holds **host-arch** sections — a guest mapping an arm64 kernel32 gets
`ILLEGAL_DLL_RELOCATION`, which is a spectacular way to learn this.

**Registry redirection lives in CM, not here** — see phase E24. It is the one part
of the redirection story that is deliberately not in wow64.dll, because registry
opens are overwhelmingly relative and a user-mode rewrite would need a
handle-to-path syscall per open.

**The kernel's share is small on purpose** — `ARM3/procsup.rs` (PEB32/TEB32),
`ke/amd64/except.rs` and `ke/amd64/thrdini.rs` (compat-mode entry/exit), plus
scattered enablers: 64-bit `ZeroBits`-is-a-mask semantics in `ARM3/virtual.rs`
and `ARM3/section.rs`, and on arm64 delivering DFSC 0x21 as
`STATUS_DATATYPE_MISALIGNMENT` so FEX's own unaligned-access backpatcher can see
it. Everything else stays in user mode where it can be debugged without a
BUGCHECK.

## How it is tested

Each ladder guest is a small i386 PE spawned by winlogon
(`TryStartWow64*`), staged under `VIBEOS_FEX=1` on arm64 and from
`wow64cpu/CMakeLists.txt` on amd64, and each reports a machine-checkable verdict:
**`0x600D` = PASS** (`0x600E` for gui32), with a distinct `0xBADx`/`0xBAFx` per
failing rung so a red boot names the rung without a debugger.

Two properties make the ladder worth more than its size suggests:

- **Guests check values, not survival.** Most of these failures are silently
  wrong rather than fatal — a truncated pointer, a stale code page, a callback
  payload read at the wrong offset. A guest that only proves it did not crash
  proves almost nothing.
- **The same guest runs on both arches.** Since the arrival paths share nothing
  below layer 2, a guest passing on both is real evidence, and a guest passing on
  one is an immediate bisect.

Stock applets are the other half of the story: they are written by people who
never heard of this port, so they exercise surface our own guests structurally
cannot reach. Nearly every marshalling bug from E6 onward was found by one.

## What is deliberately not built

- ~~**Guest processes**~~ — **`CreateProcess` WORKS** (2026-09-19). What was
  "the largest remaining hole" took five stacked defects, each invisible until
  the one before it was fixed: a dropped `_SEH2_EXCEPT` (bugcheck), an invented
  `STATUS_PARTIAL_COPY` constant, a missing TEB32 `RemotePeb` channel, an
  unmarshalled `BASE_CREATE_PROCESS` CSR message, and PEB64 process parameters
  that only a 64-bit parent would have filled. `spawn32.exe` reports `0x600D`
  across all five rungs and is now a graded LADDER guest (11/11 on amd64).
  Full chain in `x64-primary-plan.md` gap #1.

  **`CreateRemoteThread` closed the same night**, and it needed no channel
  either: the new thread's start routine is the TARGET's own
  `Wow64pGuestThreadStart`, so the per-thread setup runs in the target, on the
  new thread, by the target's own code — which is what Win10's work list exists
  to arrange. `Wow64ProcessPendingCrossProcessItems` is therefore legitimately
  empty rather than pending. The one thing that could NOT be assumed is the
  start address: `wow64.dll` loads at DIFFERENT bases in the two processes
  (measured on the live boot), so the creator walks the target's loader list and
  translates through the delta. spawn32 rungs 6-7 guard it.
- ~~**Registry redirection**~~ — **BUILT** (phase E24, task #66). Listed here as a
  silent correctness bug until then: a 32-bit guest read and wrote the *native*
  hive. Now redirected in `CmpParseKey` (`config-rust/src/cmwow64.rs`), with
  `reg32.exe` as the runtime guard. Two documented deviations from Windows remain,
  both recorded under E24 and both reversible.
- **The 209 remaining audit-baseline entries** — services whose payloads provably
  diverge but that still flat-thunk. Each is a latent version of bugs already
  paid for. E19 closed the pointer-width **out-cell** slice (which was corrupting
  guest memory) and E20 the bare **`PUNICODE_STRING`/`OBJECT_ATTRIBUTES`** slice
  (which was bugchecking the box). What is left is dominated by win32k's
  kernel-graphics-driver surface (`SURFOBJ`/`BRUSHOBJ`/`XLATEOBJ`, ~81 entries,
  unreachable from a user-mode guest), `PORT_MESSAGE` (the LPC server side,
  which belongs with the CSR-32 layer), and `TOKEN_GROUPS`.
- **IOCTL embedded-pointer buffers** (task #42) and **guest `int3` dispatch**.
- ~~**Guest user APCs**~~ — **BUILT** (2026-10-07): QueueUserAPC, I/O completion routines
  and waitable-timer routines run in the guest (`Wow64ApcRoutine`, wow64_callback.c), graded
  by `apc32.exe`. Still refusing async: `NtQueryDirectoryFile` and `NtLockFile`, which complete
  on a host IOSB.

## Things that turned out to matter more than expected

1. **Fail loud beat fail safe, every single time.** The expensive bugs were all
   quiet ones: imports poison-snapped to `0xffbadd11` instead of failing the
   load; flat thunks truncating without a word; a `switch` over constants missing
   from a bindgen island collapsing into a catch-all. Each fix was cheap; each
   *diagnosis* was not.
2. **The generator outranks the marshaller.** Hand-writing ~100 marshallers was
   affordable only because 842 services auto-thunked, and the audit gate is worth
   more than any individual marshaller because it finds the ones nobody thought
   to write.
3. **Layout truth belongs to the compiler.** Every struct-width bug we shipped
   came from a human reading a header; none came from a clang record-layout diff.
4. **Keeping the CPU-layer interface at three exports** is what made the second
   host arch cheap, and it was not obvious at the time that it would be.

---

# Historical record — the phase log

*Everything below is the chronological bring-up record, kept as written at the
time. It is not a plan any more: every phase here is complete unless it says
otherwise. Read it for the bugs — most entries carry a symptom, a root cause, and
the reasoning that got from one to the other, and several record a hypothesis
that turned out to be wrong (which is the part worth having). The as-built
picture is above; durable, reusable lessons are cross-referenced into
`translation-debugging-field-manual.md`.*

## The model (Win10+ BTCpu architecture)

Modern WOW64 splits into three layers; we adopt it because FEX's wow64 module
(`vendor/fex/Source/Windows/WOW64/`, prebuilt as `libwow64fex.dll`) already
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
   *(As built: `wow64cpu.dll` landed in E1 — `dll/win32/wow64cpu/`.)*

### The measured contract (2026-07-09, llvm-objdump on libwow64fex.dll)

*Historical. Every gap in the table below was closed in Phase A.*

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

*Index of the log, so a 2000-line record stays navigable. All complete.*

| Arc | What it covers |
|---|---|
| **A–B** | arm64 ntdll surface; wow64.dll skeleton driven out-of-band by `wow64probe` |
| **C (M1–M4)** | the real process bootstrap: PEB32/TEB32, loader divert, ntdll32 under FEX, first win32k syscall |
| **D** | `wow64gen` — the thunk-table generator |
| **M5–M10** | win32k client connect, wide GDI handle entries, DC_ATTR, windows + callbacks, the USER shared-struct view, file I/O |
| **M11–M14** | the first *stock* i386 binary, then x86 DOOM rendering and playable |
| **M15–M18** | callback OUT-payloads, async IOSB completion, desktop-heap WND reads, and `marshalgen` — bulk marshaller generation from clang layout diffs |
| **E0–E4** | the amd64 port: shared layer un-gated, `wow64cpu.dll`, compat-mode kernel support, hello32 + gui32 passing |
| **E5–E13** | depth on both arches: guest SEH, the stock-applet robustness ladder, heap corruption, the `marshalgen audit` build gate, the `NtUserMessageCall` boundary |
| **E14–E18** | guest threads, arm64 thread parity, the memory-notification contract (guests can generate code), arm64 guest-fault → guest SEH |

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

**Phase C — real WoW64 process bootstrap (the OS work). COMPLETE (M1–M4,
2026-07-09 → 2026-07-11).**

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

*M3b — IN PROGRESS (2026-07-10): the 32-bit loader runs deep, blocked on an
arm64 unaligned-access fault.* The guest now enters at
`ntdll32!KiUserApcDispatcher → LdrInitializeThunk` (the real loader) with a
down-converted 32-bit `RTL_USER_PROCESS_PARAMETERS` (the kernel leaves PEB32's
NULL) and `SystemArgument1 = ntdll32 base` (the initial-thread APC contract).
The loader runs its whole early sequence correctly — `NtQueryVirtualMemory`,
`ProcessBasicInformation`, IFEO `NtOpenKey`, heap `NtAllocateVirtualMemory ×N`,
locale, `\KnownDlls` dir+symlink+query — through 11 hand marshallers, then
builds the image + ntdll `LDR_DATA_TABLE_ENTRY`s and reaches
`RtlCreateActivationContext`.

**Two bugs found + fixed here, plus the current blocker:**
- **[FIXED] `SystemArgument1 = 0` in the APC frame.** The loader builds ntdll's
  LDR entry via `LdrpAllocateDataTableEntry(SystemArgument1)` (ldrinit.rs:2257);
  a NULL base made it call `RtlImageNtHeader(NULL)` → NULL → near-null-write of
  the entry's Flags → AV at ntdll32+0x5172. Fixed by passing the ntdll32 base.
  *(The earlier "guest heap" hypothesis was WRONG — a host-side VEH dump of the
  guest x86 regs proved the heap was healthy (ProcessHeap=0x820000, free list
  populated) and `ebx=PEB32->Ldr` showed the `RtlImageNtHeader` `je` was taken,
  not the `RtlAllocateHeap` one. Don't re-chase the heap.)*
- **Diagnostic infra landed:** `Wow64pDumpGuestHeap` in the VEH dumps the guest
  x86 register file + PEB32 fields + heap state at any unhandled guest fault
  (FEX shares the host AS, so wow64.dll reads 32-bit guest memory directly). This
  is how both bugs were pinned without a guest-ntdll rebuild or GDB.
- **[SOLVED 2026-07-10] The "GLOBAL unaligned fault" was NOT Device memory —
  it was FEX's x86-TSO atomics.** The `[ALIGN]` trap-handler diagnostic
  (trap.rs data-abort arm: dump raw self-map PTE + MAIR_EL1 + faulting opcode
  on DFSC 0x21) settled the contradiction in one boot: `PTE=0x0040000045C8A4C3`
  (AttrIndx[3:2]=0 → **Normal-WB**; memory attributes were fine all along) and
  `OPC=0x88DFFCA7` = **`LDAR w7,[x5]`**. FEX emits LDAR/STLR (acquire/release)
  for x86-TSO emulation, and those REQUIRE alignment even on Normal memory.
  FEX's own solution is a fault-handler backpatcher
  (`FEXCore::ArchHelpers::Arm64::HandleUnalignedAccess`, reached through
  `BTCpuResetToConsistentState`) keyed on `EXCEPTION_DATATYPE_MISALIGNMENT`.
  Fix: the arm64 kernel now delivers DFSC 0x21 as
  `STATUS_DATATYPE_MISALIGNMENT` (0x80000002, correct Windows semantics)
  instead of ACCESS_VIOLATION (trap.rs). With that one change the already-wired
  VEH → BTCpuResetToConsistentState path backpatches every unaligned LDAR/STLR
  and execution resumes. The `[ALIGN]` print stays as a permanent per-fault
  diagnostic (rare, one line).
- **M3b progress after the TSO fix (one boot-debug ladder, all landed):**
  1. `NtMapViewOfSection` deep marshaller (in/out 32-bit cells) + **ZeroBits
     mask semantics** in the kernel (`ARM3/section.rs`: NtMapViewOfSection
     validation + MiMapViewOfDataSection ceiling accept the 64-bit
     ZeroBits>32-is-a-mask form, same as NtAllocateVirtualMemory already did).
  2. **KnownDlls32**: ldrinit.rs (x86-gated) queries `ProcessWow64Information`
     and opens `\KnownDlls32` under WOW64 — `\KnownDlls` holds HOST-arch
     sections (the guest was mapping an *arm64* kernel32 → not-at-base →
     ILLEGAL_DLL_RELOCATION hard error). KnownDlls32 absent on arm64 → clean
     file-path fallback.
  3. **Filesystem redirection** in the wow64.dll file marshallers:
     `\system32\` → `\syswow64\` (same length, rewritten into host scratch,
     exactly where real wow64 does it). Marshallers added this pass:
     NtOpenFile, NtCreateFile, NtCreateSection, NtQueryAttributesFile,
     NtQueryFullAttributesFile, NtQuerySection (SECTION_IMAGE_INFORMATION
     48↔64), NtCreateKeyedEvent (PHANDLE out-cell).
  4. **DllPath** down-copy (+ synthesized `syswow64;system32` default) in
     `Wow64pBuildProcessParams32` — an empty DllPath NULL-crashed
     `RtlDosSearchPath_U` during the loader's mandatory kernel32 load.
  5. **`BTCpuNotifyMapViewOfSection` on every guest map** (called from the map
     marshaller) — without it FEX's dispatcher NULL-faults instead of
     translating code from newly mapped images. This was the difference
     between "DLLs map" and "DLLs execute".
  6. mkramdisk stages i386 `kernel32.dll` + `ntdll_vista.dll` into syswow64.
  7. **Guest DbgPrint forwarding**: ntdll32's `DebugService` (`int 2d`, eax=1)
     surfaces as a breakpoint exception; the VEH prints the guest's message as
     `[GUEST32] ...`. First forwarded line: `[GUEST32] [kernel32] DllMain
     PROCESS_ATTACH PID=236` — **the i386 kernel32 executes under FEX on
     arm64.**
- **M3b PASS (2026-07-11): `hello32.exe exit = 0x600D` — a real i386 PE runs
  end-to-end under FEX on arm64 through the genuine Windows loader.** Final
  ladder after the DLL-execution rung:
  1. **NtSecureConnectPort/NtConnectPort marshallers** (PHANDLE +
     PORT_VIEW/REMOTE_PORT_VIEW widen; SECURITY_QOS is pointer-free and passes
     through). This removed the BUGCHECK 0x1E (the guest's 4-aligned
     `_CsrApiPort` cell hit `ProbeForWriteHandle`'s align-8 RAISE, fatal under
     DUMMY_PSEH — `[PROBE-MISALIGN-R/W]` in ex-rust exintrin.rs is the
     permanent breadcrumb printer for this class). csrss still REFUSES the
     connect (c0000041: 32-bit CSR_API_CONNECTINFO vs 64-bit csrss) — that's
     the deferred CSR/LPC-32 layer, below.
  2. **kernel32 no-CSRSS leniency under WOW64** (kernel32-rust dllmain.rs,
     x86 + runtime `ProcessWow64Information` gate): same fallback the arm64
     native bring-up uses (continue + synthesized BASE_STATIC_SERVER_DATA).
     Native i386 keeps strict terminate-on-failure.
  3. **PEB32 NLS pointers** (the "left 0 in M1" debt): MmCreatePeb's PEB32
     branch maps a SECOND below-2GB view of the NLS section (ZeroBits mask
     form) and points PEB32's Ansi/Oem/CaseTable fields at it —
     `RtlInitCodePageTable(NULL)` AV'd in kernel32's NlsInit otherwise.
  4. **NtContinue → BTCpuSetContext** (the loader's final jump-to-entry):
     the host kernel must never see a guest CONTEXT32 (it rebuilt a garbage
     arm64 context and resumed at a wild PC). WOW64_CONTEXT == i386 CONTEXT,
     so the marshaller passes the guest pointer straight to the CPU layer;
     FEX reloads guest state on simulate reentry. NtRaiseException needs the
     same treatment (still auto-thunked — returns c000000d harmlessly today).
  5. Convenience marshallers: NtCreateDirectoryObject/NtCreateEvent/
     NtOpenEvent (open-by-OA shape).

*M4 — win32k thunk table PROVEN ON-BOX (2026-07-11, branch `fex-arm-m4`).* A
32-bit GDI syscall from a real i386 GUI guest (gui32.exe) routes through the
new win32k thunk table to native arm64 win32k. The boot oracle:
```
[FEX-WOW64C] Wow64Transition slot ...7C0CA000 (module ...7C060000) patched -> BOP   # gdi32
[FEX-WOW64C] Wow64Transition slot ...77ABB054 (module ...77A20000) patched -> BOP   # user32
[WOW64] NtGdiInit(0x10db) => ffffffff                                                # auto-thunk fires
[GUEST32] [gui32] start: CreateDCA(DISPLAY)                                          # guest code runs
[WOW64]   NtGdiOpenDCW(dev='DISPLAY' type=0 disp=1) => hdc=0                          # manual marshaller fires
```
hello32.exe still PASSes 0x600D in the same boot (no regression). gui32 exits
0xBAD1 (CreateDCA→NULL) only because the guest has no win32k process
connection yet (NtGdiInit returns 0xffffffff) — the next milestone, below.

Bugs found + fixed here:
- **CurrentDirectory NULL in the down-converted 32-bit process params.**
  `Wow64pBuildProcessParams32` copied DllPath/ImagePathName/CommandLine but NOT
  CurDir.DosPath, so `RtlGetFullPathName_Ustr` faulted at `cmpw $':',0x2(esi)`
  (esi=CurDir.Buffer=NULL) the moment the loader resolved a DLL path. hello32
  imported nothing so never hit it; gui32's gdi32/user32 closure did. Fixed by
  down-converting CURDIR (UPP64 +0x38 DosPath / +0x48 Handle).

Two independent M4 fronts (all landed on `fex-arm-m4`; a fresh gate is owed
before landing to master — these touch i386 gdi32/user32 specs + csrss):

1. **wow64win — the win32k (NtGdi*/NtUser*) 32→64 thunk table.** The Doom
   path needs win32k, not just ntdll. Reused wow64gen with a `--symbol-prefix`
   flag so a SECOND table (`Wow64Win32kThunkTable`, generated from
   `win32ss/win32u/win32u.spec` ordered by `win32ss/w32ksvc32.h`) coexists in
   wow64.dll in its own TU (`wow64_win32k.c`). Guest EAX carries `0x1000+index`
   (the win32k service-table tag bit), so `Wow64DispatchSystemService` routes
   `(n & 0xF000)==0x1000` to `Wow64DispatchWin32kService` (ntdll max is ~0x127,
   no overlap). Host side resolves through the native arm64 `win32u.dll`
   importlib (pure syscall stubs, no csr). 738 win32k syscalls, 654 auto / 84
   manual (GDI/USER structs are mostly pointer-free RECT/POINT/COLORREF, so
   auto covers far more than the ntdll table). `manual-win32k.txt` seeds the
   pointer-bearing ones (NtGdiOpenDCW, the MSG pump family).
2. **Guest-DLL Wow64Transition patching.** gdi32/user32 embed their own copy
   of the win32ksys syscall stubs (per-module `Wow64Transition` slot), so the
   NtMapViewOfSection marshaller now calls `Wow64pPatchTransitionSlot(base)` on
   every mapped image — walks the PE32 export dir, and if it exports
   `Wow64Transition`, points it at the BOP. Added the `-arch=i386`
   `Wow64Transition` export to gdi32.spec + user32.spec (arch-gated, x86-only).
3. **CSR_API_CONNECTINFO width thunk** (the c0000041 blocker's real fix). The
   NtConnectPort/NtSecureConnectPort marshaller detects a 0x24-byte connect
   blob aimed at an `*ApiPort` and widens it to the 64-bit 0x38-byte
   `CSR_API_CONNECTINFO` csrss expects, narrowing the reply back down. A loud
   print fires if csrss's returned shared-section base is >4GB (guest-invisible
   — the next sub-layer: csrss must map the shared section below 4GB for a
   WoW64 client). csrss-side refusal forensics added (`[CSRSRV] ConnReq:` /
   `AttachSharedSection FAILED` in csrsrv api.rs/server.rs).
4. **gui32.exe harness** — an i386 GUI guest (real kernel32+gdi32 imports) that
   opens a display DC (CreateDCA→NtGdiOpenDCW) + PatBlt + GetPixel, exercising
   the win32k table without user-mode callbacks. Staged under VIBEOS_FEX beside
   the gdi32/user32/win32u guest set; winlogon `TryStartWow64Gui` launches it.
   Verdict = exit 0x600D + a magenta rect on the arm64 desktop.

## M5 (2026-07-11): win32k client connect — gui32 DRAWS through native win32k

**The NtGdiInit=0xffffffff mystery is SOLVED and closed** (task #29). Root
cause was never in the wow64 table at all: `win32ss/win32u/sys-stubs.S`
selected its service-number list by `#ifdef _M_AMD64` alone, so **arm64
win32u.dll numbered its stubs from w32ksvc32.h while the kernel table
(napi.h, `_WIN64`) uses w32ksvc64.h** — every diverging index called the
WRONG win32k routine (NtGdiInit@219-in-32 landed on
NtGdiModifyWorldTransform@219-in-64; real GdiInit is 0x1E0). Native
gdi32/user32 were immune (win32ss/sys-stubs.S has a correct `_M_ARM64`
branch) — only win32u callers, i.e. exactly the wow64 host, saw it. Moral:
*a stub library nothing native exercises is unproven; compare its numbers
against a consumer that works.*

With routing fixed, the actual client-connect layer (all landed, gate ×3):
- **win32k `W32Wow64ZeroBits`** (init.rs): every user-visible mapping win32k
  hands a WoW64 process now lands below 4GB via the ZeroBits mask form —
  GDI handle table (`GDI_MapHandleTable`), global user heap
  (`MapGlobalUserHeap`), desktop-heap view (`IntMapDesktopView`), GdiPool
  DC_ATTR/BRUSH_ATTR slabs (`GdiPoolAllocateSection`).
- **`GdiProcessCreate` mirrors `GdiSharedHandleTable`/`GdiDCAttributeList`
  into PEB32** (typed `ntkrnl_sys::ddk::_PEB32`, cfg'd off x86).
- **`NtUserProcessConnect` manual marshaller** (USERCONNECT 0x124↔0x240;
  truncated `ulSharedDelta` stays correct in the guest's modular 32-bit
  pointer arithmetic since the user VAs are <4GB).
- **`NtQueryValueKey` manual marshaller** (PUNICODE_STRING widen) — its raw
  auto-thunk fed the kernel a 32-bit UNICODE_STRING whose garbage Buffer
  probe-raised C0000005 → BUGCHECK 0x1E under DUMMY_PSEH (user32 init reads
  registry right after NtOpenKey).

On-box result: `NtUserProcessConnect => 0` (psi=001c0dd0 ahe=001da000 —
below 4GB), user32 DllMain attach completes, `NtGdiOpenDCW` returns a real
HDC (18010112), **PatBlt draws through native arm64 win32k**. gui32 exit
**0x600E** = the designed draw-OK/readback-WARN rung. hello32 0x600D
unchanged.

## M6 (2026-07-11): wide GDI handle-table entries — guest GetPixel round-trips

The M5 readback WARN was NOT surface readback: 32-bit gdi32 read the shared
GDI handle table with 16-byte GDI_TABLE_ENTRY stride, but the 64-bit kernel
writes 24-byte ENTRYs — so `GdiValidateHandle`/`GdiGetHandleUserData` failed
client-side and SelectObject/GetPixel never issued syscalls (the boot-log
tell: no `NtGdiSelectBrush`/`NtGdiGetPixel` traces after `NtGdiPatBlt`).
Fix in **gdi32-rust** (dllmain.c/misc.c/region.c are cohab-flipped DEAD
source — a first fix went into the C and did nothing; the parallel C edits
are kept aligned for reference): `GdiProcessSetup` detects wow64 once
(ProcessWow64Information != 0 → `g_GdiWow64`), all shared-entry reads route
through `GdiReadSharedEntry` (wide layout when set; field ORDER matches, only
KernelData/UserData widen), and `GdiDevCaps` rebases past 0x10000 wide
entries. On-box: **NtGdiGetPixel fires from the guest and returns the true
framebuffer pixel** — the full 32-bit-guest GDI readback path works.

## M7 (2026-07-11): gui32 0x600D PASS — WoW64 DC_ATTR shim, guest draws MAGENTA

M6's white pixel was DC_ATTR layout divergence (guest wrote hbrush/ulDirty_
at 32-bit offsets into the kernel's 64-bit DC_ATTR). Fixed the real-Windows
way — the wow64 CLIENT knows the 64-bit layout (approach (b); (c)
layout-invariance is impossible because FLOATOBJ diverges semantically:
i386 8-byte EFLOAT vs FLOAT elsewhere):
- **`gdi32-rust/src/dcattr64.rs`**: DC_ATTR64 mirror on the 32-bit compiler.
  Wide pointer/handle slots split lo(native-typed)+`_hi` pad (kernel zeroes
  the struct and never stores nonzero upper halves, so lo-half writes
  compose); explicit `__padN` where repr(C)-on-64 pads; const-asserted
  offsets + 416-byte size. MATRIX/FLOATOBJ fields are opaque filler —
  client code never touches them (swept).
- **`dcattr_f!`** (lib.rs): field-projection macro → correctly-offset,
  correctly-typed `*mut`; serves reads/writes/compounds/sub-fields; native
  branch byte-identical, non-x86 passthrough.
- **All 240 `(*pdcattr).f` sites rewritten by script** (regex → 
  `(*dcattr_f!(pdcattr, f))`, per the >50-edits automation rule).

On-box: `NtGdiPatBlt => 1`, `NtGdiGetPixel => 00ff00ff`, **gui32 exit
0x600D PASS** — an x86 GUI binary under FEX completes draw+readback through
the native arm64 win32k (magenta rect on the desktop). Gate ×3 green.

## M8 (2026-07-11): win32.exe — an x86 WINDOW on arm64 + kernel→user callbacks

gui32 used a *direct display DC* (no window, no callbacks). M8 is the first
real window: `RegisterClassA` → `CreateWindowExA` → message pump, driven by
`win32.exe` (i386, user32 imports), landing `WM_NCCREATE`+`WM_CREATE` in the
guest's own 32-bit wndproc. Two new pieces:

- **`dll/win32/wow64/wow64_callback.c` — the kernel→user callback bridge.**
  win32k delivers callbacks via the 64-bit ntdll `KiUserCallbackDispatcher` →
  `PEB64->KernelCallbackTable`. A WoW64 process loads no native user32, so
  *wow64.dll owns that 20-entry table*. Each entry (a) snapshots the guest
  CONTEXT (`BTCpuGetContext`), (b) marshals the 64-bit callback arg struct
  **down to 32-bit** (WINDOWPROC = the CREATESTRUCTW repack: 80→48 bytes, the
  packed lpszName/lpszClass *offsets* shift by the 32-byte header shrink), (c)
  pushes an i386 `KiUserCallbackDispatcher` frame on the guest stack and
  **re-enters `BTCpuSimulate` nested** (FEX supports this — wine's
  `Wow64KiUserCallbackDispatcher` shape). The guest wndproc runs and exits via
  `ZwCallbackReturn` → BOP → our manual `NtCallbackReturn` marshaller, which
  translates the 32-bit result back up and `longjmp`s (hand-rolled aarch64
  setjmp/longjmp, no CRT) to the table entry, which issues the real 64-bit
  `NtCallbackReturn`. The kernel resumes the original syscall.
- **9 new win32k marshallers** in `wow64_manual.c`: `NtUserRegisterClassExWOW`
  (WNDCLASSEX + CLSMENUNAME + 3× UNICODE_STRING), `NtUserCreateWindowEx`
  (LARGE_STRING ×3, atom-in-class-slot passthrough), the pump family
  (`NtUserGetMessage`/`PeekMessage`/`DispatchMessage`/`TranslateMessage` —
  MSG 28↔48), `NtUserBeginPaint`/`EndPaint` (PAINTSTRUCT).

Also: the guest window must be spawned on the **input desktop**
(`WinSta0\Default` via STARTUPINFO.lpDesktop) — winlogon's own threads sit on
the Winlogon desktop, and win32k's paint scan only walks the input desktop
(§9.22 class). On-box: `WM_NCCREATE`/`WM_CREATE` reach the guest wndproc, the
window is created, the pump runs — **an x86 program owns a real window on the
arm64 desktop.** win32.exe exits `0x600E` (window+callbacks OK; WM_PAINT is
M9). Native i386/amd64/arm64 unaffected (arch-gated / runtime-flagged).

## M8.5 (2026-07-12): default-cursor marshalling — win32.exe gets its FULL message set

Fixed a real BUGCHECK 0x1E on the way to M9 (field manual §9.58a). `win32.exe`
registers with `hCursor=NULL`, so window creation loads `IDC_ARROW`:
`NtUserSetCursorIconData` + `NtUserFindExistingCursorIcon` take module/resource
`UNICODE_STRING`s that were **passthrough** thunks → the guest's 32-bit
`UNICODE_STRING` (4-aligned) hit win32k's `ProbeForRead(.,16,8)` →
`STATUS_DATATYPE_MISALIGNMENT` → 0x1E. Fix: manual marshallers widening both
strings 8→16 + repacking `CURSORDATA` 0x58→0x88 (`wow64_manual.c`), added to
`manual-win32k.txt`. Post-fix the window creates and the guest wndproc receives
its **full** message set through the callback bridge (WM_NCCREATE, WM_NCCALCSIZE,
WM_CREATE, WM_SHOWWINDOW, WM_NCPAINT, WM_ERASEBKGND, WM_ACTIVATE, WM_SIZE,
WM_MOVE, …). Also added an **on-entry** win32k syscall trace (`[WOW64>]`, gated
on `Wow64TraceSyscalls`) so a thunk that *faults* the kernel still names itself.

## M9 (DONE — `win32.exe` 0x600D): the 32-bit client view of USER's shared structures

**win32.exe now creates a window, receives WM_PAINT, and paints its client rect
through the native arm64 win32k.** `0x600E → 0x600D`; `gui32` stays `0x600D`.
Full writeup + the reusable lessons: field manual **§9.58b**.

The symptom was that `PeekMessageA` returned `WM_PAINT` with the right hwnd 200×
(win32k's paint path was always correct) but the guest never dispatched it —
`user32!DispatchMessageA` calls `ValidateHwnd()` first and silently `return 0`s on
NULL, with no syscall.

**Root cause — a layout problem, not a mapping one.** user32 reads three things
*straight out of shared memory with no syscall*, and all three are laid out by a
64-bit kernel:
1. the shared **USER handle table** (`gSharedInfo.aheList`) — what `ValidateHwnd` indexes,
2. the desktop heap's **WND / CLS** objects, via `DesktopPtrToUser()`,
3. **DESKTOPINFO** (`pvDesktopBase`/`pvDesktopLimit`/`fsHooks`/`spwnd`).

They are already *addressable* by the guest (win32k maps them <4GB for WoW64 procs
via `W32Wow64ZeroBits`) — just not *parseable*. Shadowing only (1) kernel-side (the
first instinct, and what the earlier attempt did) fixes validation and then walks
straight into (2): the guest dereferences the now-validated WND and AVs. So the fix
is **client-side**, mirroring the M6/M7 GDI pattern exactly:

- **`user32-rust/src/wow64.rs`** (new) — `g_UserWow64` (detected at connect via
  `ProcessWow64Information`, like `GdiProcessSetup`), the `*64` shapes (`Wnd64`,
  `Cls64`, `DesktopInfo64`, `UserHandleTable64`/`Entry64`), and a normalizing view
  over each. WND/CLS become cached **snapshots** — sound because the client's
  desktop-heap mapping is `PAGE_READONLY`, so user32 never writes through a `PWND`.
  Typed `desktop_wnd_to_user()`/`desktop_cls_to_user()` replace the ~25 raw
  `DesktopPtrToUser(x->spwnd*/pcls)` sites.
- **`win32k-rust/src/wow64.rs`** (new) — win32k writes CLIENTINFO into the **64-bit**
  TEB, but a WoW64 guest reads it from **TEB32**, which was therefore all zeros
  (`ulClientDelta == 0`, `pDeskInfo == NULL` — the actual reason the guest computed a
  raw kernel pointer). `Wow64PublishClientInfo()` projects it into a `ClientInfo32`
  in TEB32 at thread init + desktop assignment.
- **Layout tripwire** — `user32-rust` also builds natively for amd64/arm64, so a
  `#[cfg(not(target_arch = "x86"))] mod layout_guard` asserts each `*64` shape still
  *is* the real struct (size + per-field `offset_of!`). It caught a bogus padding word
  in `Cls64` on the first compile. `ntuser.h` drift now breaks the 64-bit build instead
  of silently corrupting WoW64 clients.

**Owed, but LOUD:** `MENU`/`ITEM`, `HOOK`, `SBINFO` are desktop-heap objects with
pointers and have no `*64` shape yet (nothing on the M9 path reaches them).
`wow64_unsupported()` prints a one-shot warning naming the type instead of letting a
32-bit client silently walk a 64-bit `MENU`. First WoW64 app that needs menus turns
that print into the work order.

**Gotcha worth repeating:** the guest's `user32.dll`/`gdi32.dll` are staged from
**`build-i386/`**. An arm64-only rebuild ships a *stale* client — and the kernel half
alone against a stale client is actively harmful (`0x600E` → `0xC000001D`, wild jump),
because a non-zero `ulClientDelta` makes the unmodified client start translating
pointers with 32-bit offsets into 64-bit structs. Rebuild **both** arches.

*Same divergence class still owed elsewhere (next layers toward Doom):*
- **TEB GdiTebBatch GDIBS\* batch commands** — gui32 didn't batch; the
  kernel-side `KeGdiFlushUserBatch` reads 64-bit GDIBS from a wow64 TEB.
- **user32 HANDLEENTRY / SERVERINFO / desktop-heap structs** — the M8 pump path
  exercises SHAREDINFO reads; deeper window ops (subclassing, prop lists) will
  surface more 32/64 divergence in the user handle table.

## M10: file I/O — the `IO_STATUS_BLOCK` family

The first thing Doom does is `ReadFile` a WAD, and that path was quietly broken.

`IO_STATUS_BLOCK` ends in `ULONG_PTR Information` — **4 bytes in the guest, 8 on
the host**. So the guest's struct is 8 bytes and the host kernel writes 16. Yet
`NtReadFile`, `NtWriteFile`, `NtQueryInformationFile`, `NtSetInformationFile`,
`NtQueryVolumeInformationFile` and `NtFlushBuffersFile` were never listed in
`tools/wow64gen/manual.txt`, so wow64gen **auto**-thunked them: it handed the
guest's 8-byte IOSB straight through. Two bugs at once — an 8-byte overflow past
the guest struct, and an `Information` read from the wrong offset (i.e. every
`ReadFile` byte count was garbage). This is exactly the failure wow64gen's manual
list exists to prevent; the names were simply missing from it. *Lesson: the
deep-marshal list is only as good as its audit — the generator cannot infer
struct-ness from a `.spec`, so a syscall taking a divergent struct is silently
wrong until somebody puts it on the list.*

Fixed in `dll/win32/wow64/wow64_manual.c`: all six, plus `NtQueryDirectoryFile`
(which additionally takes a `PUNICODE_STRING` search mask). The remaining IOSB
takers (`NtLockFile`, `NtQueryEaFile`, `NtCancelIoFile`, `NtNotifyChangeKey`,
`NtDeviceIoControlFile`, …) are now on the manual list too, so they fail **loud**
instead of silently corrupting — implement on demand.

**Async is the hazard, and we do not take it.** The host IOSB we marshal into
lives on the thunk's stack frame. Returning `STATUS_PENDING` would leave the
kernel holding a pointer into a frame that is about to die — the dead-frame-IOSB
corruption class (field manual §9.36, which cost this project real days). So:

- an explicit async request (guest passed an `Event` or an APC routine) is
  **refused loudly before the call is issued** — no IRP, no corruption;
- if the host pends anyway (async handle, guest polling its IOSB), we **block on
  the file handle** until completion rather than return over a live pointer into
  a dead frame.

*Still true only for the functions listed below that use `Wow64pFinishIosb`.*
`NtReadFile`/`NtWriteFile` (M15b) and the IOCTL family (M15c) no longer marshal
through a host IOSB at all and accept an `Event`; see the two notes below.

*~~Owed~~ — **LANDED 2026-07-14 (M15b)**: the kernel-side IOSB32 fix is in.*
Design as planned, with the per-IRP gate solved by a **per-call TEB latch**:
wow64.dll's `NtReadFile`/`NtWriteFile` thunks store the guest IOSB pointer in
`TEB.Instrumentation[0]` right before the syscall; the kernel entry
(`IopCaptureWow64Iosb32`, iofunc.rs) consumes it (read-and-clear) only when the
process is WoW64 AND the latch equals the IoStatusBlock argument exactly, then
probes 8/4 instead of 16/8 and marks the IRP (`IRP_WOW64_IOSB32`, kernel-
internal bit 0x8000_0000). Both `IopCompleteRequest` UserIosb writes (irp.rs)
and both fast-IO direct writes emit the 8-byte `{Status, Information}` layout
when marked — sync AND async. The thunks now pass the guest IOSB straight
through (no host stack IOSB → the dead-frame hazard is gone by construction)
and accept Event-based/polled overlapped I/O; only a guest APC routine still
refuses loudly.

*What the APC refusal actually needs — the old note here said "no 64→32
user-APC dispatch yet" and that was wrong.* The guest-entry half exists and is
load-bearing: `Wow64pBuildLoaderApcFrame` builds a `KiUserApcDispatcher`-ABI
frame, `Wow64pKiUserApcDispatcher32` is resolved from ntdll32 at startup, and
the pair runs on every guest thread start. Missing is (a) INTERCEPTION — the
kernel's `KiInitializeUserApc` builds a 64-bit `UAPC_FRAME` targeting the 64-bit
dispatcher, so a guest's 32-bit routine passed through would be `call`ed as
native x64; wow64.dll must substitute a host thunk as the `NormalRoutine` and
carry the guest routine/context as arguments — and (b) the frame builder taking
its routine as a parameter instead of hardcoding the loader thunk. Nothing on
the ladder needs it (msafd uses Events), so it stays refused by choice. 64-bit host code in the same
process is unaffected: its calls never latch. i386 kernels compile the capture
out (no WoW64-of-32-bit; its bindgen EPROCESS has no Wow64Process).
**M15c (2026-09-06) extends the same mechanism to the IOCTL family**
(`NtDeviceIoControlFile`/`NtFsControlFile` via `IopDeviceFsIoControl`), which
was the last thing keeping E25 unexercised: msafd always passes an `Event`
because AFD is inherently async, so the refusal blocked every socket call a
32-bit guest could make. Three obligations per consumer, each **silent** if
skipped: probe the guest block at 8 bytes, mark the IRP, and narrow the write
on the fast-I/O path (it completes inline, never builds an IRP, so the flag
cannot reach it). One asymmetry against M15b is deliberate: a **marshalled**
IOCTL still completes synchronously, because the hazard there is the PAYLOAD,
not the IOSB — the widened input, the WSABUF array and the AFD_POLL_INFO shadow
are host-heap blocks the thunk frees on the way out, and METHOD_BUFFERED copies
SystemBuffer back into that shadow *at completion*. `Wow64pIoctlCall` therefore
blocks, re-signals the guest's event (msafd's `SockEvent` is a SYNCHRONIZATION
event, so our own wait consumes the signal `MsafdWaitForAlert` is about to wait
for — missing that is a silent guest hang), then copies back and frees. The
unmarshalled path carries no shadow and is genuinely async. Oracle:
**`marshal32.exe`**'s select rung, `0xbb04` -> `0x600d`, 9/9 guests PASS on
ladder `20260906-132106-smoke-amd64`.

**M15c addendum (2026-09-06): `IOCTL_AFD_SEND_DATAGRAM` is marshalled**, which
retires the last refusal in the AFD family. `AFD_SEND_INFO_UDP` is 56 → 104 and
audited SEMANTIC for its nested `TDI_REQUEST_SEND_DATAGRAM` /
`TDI_CONNECTION_INFORMATION`, but reading both ends collapses most of that:
`AfdPacketSocketWriteData` consumes only `BufferArray`, `BufferCount` and
`TdiConnection.RemoteAddress`; `WSPSendTo` fills only those plus `AfdFlags` and
`RemoteAddressLength`. `TdiRequest` is touched by neither, so it is carried as
opaque zero padding rather than translated — writing a host-shaped
`TDI_REQUEST` there would manufacture content no one supplies. `RemoteAddress`'
pointee is pointer-free and width-identical, so the pointer widening is the
whole job (same shape as `RECV_DATAGRAM`'s `Address`). **The rung's assertion is
the implicit bind, not the send.** A successful `sendto` proves little; AFD only
reaches `WarmSocketForBind` by dereferencing `RemoteAddress` for
`Address[0].AddressType`, so a socket that comes back bound AF_INET with a real
port is evidence the driver read the family through the widened pointer. The
byte count covers the WSABUF stride separately, and the rung sends UNBOUND on
purpose — binding first would skip the very dereference under test.

Oracle: **`async32.exe`** (`Async32.c`, winlogon `TryStartWow64Async`) —
overlapped ReadFile with event: `OVERLAPPED.Internal/InternalHigh` (the IOSB32)
must show SUCCESS/512, **`.Offset` must survive** (a 16-byte write would clobber
it — the layout-drift tripwire), bytes must match a sync control read, plus an
eventless poll rung. Verified 0x600D with all five sibling guests green on the
same boot.

Oracle: **`file32.exe`** (`tools/fex/hello-compute/File32.c`, launched by
winlogon's `TryStartWow64File`) — opens a real file, checks `GetFileSize`, reads
it and **asserts the returned byte count**, seeks and re-reads, round-trips a
write, and enumerates a directory. The byte count *is* `IOSB.Information`, so a
regression in this marshalling cannot pass the test.

*Still deferred (owed, the next milestones):*
- **advapi32.dll / usp10.dll (i386) staging** — user32 imports them; the loader
  currently skips-missing (`LDR: Skipping missing import ... c0000135`), which
  works for gui32 but leaves user32 partially unresolved. Stage them for a real
  windowed app.
- **CSR shared-section below-4GB mapping** — the connectinfo thunk widens the
  message, but csrss's `CsrSrvSharedSectionBase` is a 64-bit VA; a WoW64 client
  needs it mapped below 4GB (the loud >4GB print is the tripwire). Likely a
  `NtMapViewOfSection` ZeroBits-mask in `CsrSrvAttachSharedSection` when the
  target process is WoW64.
- CSR_API_MESSAGE/PORT_MESSAGE body layouts for actual API calls past connect
  (console I/O, real CreateProcess from guests).
- **Kernel robustness:** misaligned user pointer at any PHANDLE-probing
  syscall = probe RAISE + DUMMY_PSEH = BUGCHECK 0x1E from user input.
  Long-term: Stage-2 `Result<T>` probe migration for syscall entry paths.
- NtRaiseException marshaller (32-bit EXCEPTION_RECORD + CONTEXT), guest
  exception dispatch parity.
  All landed behind VIBEOS_FEX; normal boots unaffected (i386+amd64+arm64
  smoke green with these kernel changes).

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

**Phase E — amd64 classic wow64cpu (native x86-on-x64).** Same phases A–D
infrastructure, CPU layer swapped: a native `wow64cpu.dll` heaven's-gate mode
switch (no emulator — the silicon runs the 32-bit code in compatibility mode).
The thesis "x86-on-x64 for free once C+D exist" held: the arch-shared OS layer
(process bootstrap, the 296-syscall thunk table, wow64win) dropped onto amd64 by
un-gating, and only the CPU layer + one kernel context-switch line were new.
Rungs (branch `wow64-amd64`):

*E0 — ungate the shared layer + kernel i386 admission for amd64. DONE
(2026-07-17, commit 6816923480e).* Flipped the `#[cfg(target_arch="aarch64")]` /
`-arch=arm64` WoW64 gates to include `x86_64`: ldrinit.rs (WoW64 detect / IAT-
skip / reloc-skip / wow64.dll handoff), rtl/mod.rs (`wow64shim`), procsup.rs
(MmCreatePeb PEB32 + MmCreateTeb TEB32 + below-2GB NLS; `ARM64_UART_TRACE_HEX`
→ portable `MiWow64TraceHex`), ex-rust init.rs (`ImageNumberLow/High=[0x014c,
0x8664]` so CreateProcessInternalW admits i386), ntdll.spec (10 exports widened;
dead `-stub -arch=x86_64 RtlWow64*` duplicates removed), wow64.dll CMake/spec +
wow64gen amd64→x86_64 spec-arch alias, `WOW64_HOST_PC` portability. All 3 arches
build; amd64 native smoke boot+net green; arm64 FEX path byte-unchanged.

*E1 — wow64cpu.dll, the native heaven's-gate CPU layer. DONE (2026-07-17, commit
6816923480e).* `dll/win32/wow64cpu/` (amd64-only) exports the SAME `BTCpu*`
contract wow64.dll resolves by name (so wow64.c only `#ifdef`-selects the DLL
name + `WOW64INFO.NativeMachineType`). `wow64cpu.S`:
  - **EnterGuest** builds an `iretq` frame from the CPU-area WOW64_CONTEXT and
    drops into compat mode (CS 0x23, SS/DS/ES 0x2B, FS 0x53); gs is left
    untouched so its base stays TEB64 for the dispatcher.
  - **BTCpuProcessInit** emits a 22-byte heaven's-gate sequence into a below-2GB
    page: `EA <off32> 0033` (32-bit `jmp far 0x23→0x33`) → `mov r10d,eax;
    movabs rax,Wow64CpuSyscall64; jmp rax`. `BTCpuGetBopCode` hands its 32-bit
    entry to wow64.c's `Wow64Transition` patch.
  - **Wow64CpuSyscall64** (the 64-bit landing) saves guest GPRs into the context,
    switches to the dispatcher stack, calls the C `Wow64CpuDispatch` (→
    `Wow64SystemServiceEx(eax, esp+8)`, i386-stdcall stack shape), then
    re-enters via EnterGuest. **r15 pins the CPU-area context across guest
    execution** — compat-mode code can't touch r15, so no gs/TEB re-derivation
    is needed after the mode switch. `BTCpuNotifyMapViewOfSection` is a no-op
    (native, no JIT cache). Builds on amd64; trampolines disassemble byte-exact.

*E2 — amd64 per-thread CMTEB base (32-bit guest fs). DONE (2026-07-17, commit
a0c900d2a37).* The amd64 boot GDT (freeldr `Amd64SetupGdt`) already populates
CMCODE(0x23)/CMTEB(0x53)/R3-data/64-bit-code descriptors with correct type
flags, and context.rs/except.rs/usercall.rs already special-case compat-mode CS —
so E2 was tiny: `KiSwapContextResume` (ke/amd64/thrdini.rs) now rewrites the
CMTEB descriptor **base** = TEB32 (TEB64+0x2000) on switch-in to any
`Wow64Process` thread (Flags1/Flags2 preserved). Native threads untouched; amd64
smoke boot+net green.

*E3 — hello32.exe 0x600D on amd64. DONE (2026-07-18, commit 0811e1c2ab0).* The
first 32-bit binary the amd64 build ever ran: the real i386 ntdll32 loader
(LdrInitializeThunk, ~200 syscalls, kernel32 DllMain PROCESS_ATTACH) executed in
hardware compatibility mode through the E1 heaven's-gate, handed off to the image
entry, and exited through wow64.c's verdict service with **0x600D**. Staging:
`add_cd_file FILE` in `wow64cpu/CMakeLists.txt` injects the i386 syswow64 set
from the sibling `build-i386`/`build-fex-hello` trees onto `livecd.iso` (the
amd64 mirror of mkramdisk's VIBEOS_FEX staging; all-or-nothing, loud skip).
Launch: winlogon `TryStartWow64Hello` widened to x86_64. One code fix from first
boot to pass: **wow64cpu `BTCpuSetContext`** (the guest NtContinue at loader end)
copies CONTEXT32 into the CPU-area WOW64_CONTEXT and sets a CPU-area+4 "resume
replaced" flag so `Wow64CpuDispatch` resumes the new state instead of the
return-to-stub fixup. As predicted, none of the arm64 JIT/TSO/RCpc pitfalls
applied.

*E4 — gui32 (win32k GDI) 0x600e on amd64. DONE (2026-07-18, commit f64d06e236e).*
A 32-bit x86 app draws through the 64-bit win32k: gui32 opens a DISPLAY DC,
creates a pattern brush, PatBlts a magenta rectangle (real NtGdi* handles),
exits 0x600e (draw OK, GetPixel readback soft-mismatch, same verdict as arm64).
The one native-only root cause: gdi32/user32's win32k client stubs do NOT use an
exported `Wow64Transition` slot (unlike ntdll32) — they dispatch through a
per-module `_KiIndirectSystemCall` (`mov edx,0x7FFE0300; jmp [edx]` =
`&SharedUserData->SystemCall`). On arm64/FEX the JIT intercepts the syscall
regardless of that pointer; on amd64 the guest literally `jmp`s to `[0x7FFE0300]`
== 0 (KUSER_SHARED_DATA is one global page, can't hold the per-process BOP) →
rip=0 in gdi32 DllMain. `Wow64pPatchTransitionSlot` (amd64-only) now scans each
mapped guest module for that 7-byte dispatcher and rewrites it in place to jump
straight to the process BOP. Also landed on the way: the M8 callback bridge's
x86-64 setjmp/longjmp (`wow64_callback.c` was aarch64-only) + `BTCpuGetContext`.
E0–E4 are on branch `wow64-amd64`; amd64 smoke green (hello32 0x600D + gui32
0x600e run in every smoke as coverage).

*E4b — testgui/doom (window creation + kernel→user callbacks) on amd64.
**DONE 2026-07-18** — x86 DOOM renders E1M1 in a window on the amd64 desktop
(3,000+ `NtGdiStretchDIBitsInternal` frame blits and climbing, message pump
alive); testgui PASSes its 20s still-running check. Four distinct fixes, in
debug order:*

1. **Stale-`Esp` callback stack smash (the deep one — `wow64cpu.S`).**
   `Wow64CpuSyscall64` saved only the guest GPRs into the CPU-area context at
   BOP entry; `Esp`/`Eip` were written only by the *return* fixup. So while a
   win32k service was in flight the context held the PREVIOUS syscall's
   `Esp`, and a kernel→user callback fired mid-service built the guest-side
   dispatcher frame at that stale address — smashing everything the guest had
   pushed since (doom died in `RtlInitUnicodeString` reading a clobbered
   string pointer during the WM_NCCREATE W→A conversion). FEX tracks live
   guest state internally, which is why only amd64 needed the fix: save the
   live `Esp` + `[esp]` return address at BOP entry.
2. **`NtUserGetClassInfo` manual marshaller** (`wow64_manual.c` + gen list).
   The auto-thunk fed win32k an 8-byte `UNICODE_STRING32` read as 16 bytes
   (garbage `Buffer` → probe-raise → BUGCHECK 0x1E under DUMMY_PSEH) and let
   the kernel write an 80-byte `WNDCLASSEXW` over the guest's 48-byte struct.
   arm64 only ever survived this by stack-garbage luck — the marshaller fixes
   both arches.
3. **Callback 8 (CLIENTLOADLIBRARY) 64→32 marshaller** (`wow64_callback.c`).
   `CLIENT_LOAD_LIBRARY_ARGUMENTS` 40→24 header squeeze (both UNICODE_STRING
   Buffers are self-relative offsets; shift by the 16-byte shrink, blob
   verbatim) + 4→8-byte result widen on the return. First exercised by
   win32k's `LoadUserApiHook(uxtheme.dll)` during doom's first window op;
   uxtheme isn't in the closure so the guest returns FALSE gracefully (theme
   api hooks stay off — fine, arm64 never even ran this callback before).
4. **Stale guest binaries (the embarrassing one).** After fixes 1–3 doom's
   WM_NCCREATE *completed* but the guest returned Result=0: the staged
   `build-i386` DLLs predated the last master merge. The syswow64 set on the
   amd64 livecd is snapshotted from `build-i386` at `ninja livecd` time —
   **always rebuild i386 before re-mastering the amd64 livecd when
   guest-visible shared source changed.** (Cross-build staging has no
   staleness gate — the smoke staleness check covers only same-arch media.)

Bonus hardening on the way: failure-branch `ERR()` prints in user32's
`MsgiUnicodeToAnsiMessage` (NC)CREATE conversion (fail-loud), completion-path
result tracing behind `Wow64TraceSyscalls`, and `MmCreatePeb`/`MmCreateTeb`
out-pointer writes converted to `write_volatile` (lint_kernel clean).

*E4c — full probe-ladder parity + stock-binary robustness. **DONE
2026-07-18.*** The amd64 ladder now runs every arm64 probe guest —
hello32/gui32/win32/file32/async32/marshal32 all exit 0x600D — plus a NEW
robustness rung: **winmine.exe, a stock i386 OS applet, runs alongside doom**
(menus, WM_TIMER, LED counters all render; "still running after 10s" PASS).
What it flushed out:

* **`NtUserThunkedMenuItemInfo` marshaller** (winmine's menu bar): the
  auto-thunk handed win32k the guest's 8-byte UNICODE_STRING32 caption; the
  align-8 ProbeForRead raised STATUS_DATATYPE_MISALIGNMENT → BUGCHECK 0x1E
  under DUMMY_PSEH. MENUITEMINFOW 48→80 + caption widen fixes it (this is the
  exact "4-aligned WoW64 buffer into an align-8 probe" class the
  `wow64_win32k.c` on-entry trace was built to catch — it named the syscall
  on serial).
* **file32/async32 hardcoded `C:\ReactOS\...`** — the amd64 livecd's system
  volume is X: (arm64's ramdisk is C:), so both guests failed their open
  rung. They now derive paths from `GetWindowsDirectory` at runtime, and
  file32's write/round-trip rung SKIPs loudly on a read-only volume (the
  livecd is a CD; the arm64 ramdisk still exercises the write path).

*E4c owed (non-blocking):* guest 32-bit `GetWindowsDirectoryA` returns
`\reactos` with NO drive letter on amd64 (rooted path still resolves, but the
CSR/static-server-data marshalling that should carry `X:` is incomplete);
uxtheme.dll not in closure (theme api hooks disabled); guest hardware-fault
context flush (the CPU-area is fresh at syscall boundaries now, but a fault at
an arbitrary guest instruction still reads stale values — Phase E5 below).

*E4d — doom INPUT on amd64: VERIFIED PLAYABLE (2026-07-18).* The M14 oracle
replayed against the amd64 livecd boot: QMP-injected ESC opened DOOM's main
menu, ENTER advanced to episode select, ENTER again to skill select — three
distinct screen transitions, each caused by an injected key, with WineMine
alive alongside. Keys ride the `NtUserPeekMessage` return path (~1,800 pump
iterations traced) and dispatch inside guest code, exactly as on arm64.
Harness: the session's `doom-play-amd64.sh` (start.sh amd64 + QMP send-key +
screendump), a straight port of `tools/doom/play.sh`.

*E4d finding — the "[GUEST32] int-2d silent on amd64" note was MISDIAGNOSED:*
guest DbgPrints are NOT lost on amd64. The compat-mode `int 2d` is consumed
in-kernel by the KD path (KdpPrint → serial) and the guest resumes — the
prints appear WITHOUT the `[GUEST32]` prefix (e.g. winmine PID 648's
`[kernel32] DllMain PROCESS_ATTACH` lines). wow64.dll's VEH forwarder never
sees them, which is fine; only arm64/FEX needs the VEH branch (FEX surfaces
int 2d as a host exception instead of reaching the kernel's IDT).

**Phase E5 — guest fault-context flush + exceptions dispatched INTO guest SEH
(2026-07-18).** Before E5, a hardware fault at an arbitrary guest instruction
was a death sentence: the CPU-area WOW64_CONTEXT was fresh only at syscall
boundaries (the E4b fix), the host VEH read stale guest registers for its
diagnostics, and the guest's own 32-bit SEH never ran — the 64-bit
RtlDispatchException walked 64-bit unwind info from a guest RIP and
second-chanced. Software raises were worse: the generated flat thunk for
`NtRaiseException` passed the guest's 32-bit EXCEPTION_RECORD/CONTEXT
pointers straight to the 64-bit kernel to read at 64-bit layout. Three
mechanisms close it (all landed together):

1. **`BTCpuResetToConsistentState` in wow64cpu (the flush).** The heaven's-
   gate property makes it trivial: when compat-mode code faults, the kernel's
   trap CONTEXT captures the full register file — the low 32 bits of the
   captured GPRs/Rsp/Rip ARE the guest registers, and `SegCs == 0x23`
   identifies the frame as guest code. The VEH already called this entry
   point by contract (FEX exports it on arm64); wow64cpu just never
   implemented it.
2. **VEH hardware-fault redirect into guest SEH (amd64, wow64.c).** After the
   flush, a `SegCs == 0x23` exception converts EXCEPTION_RECORD 64→32, builds
   the i386 `KiUserExceptionDispatcher` frame on the guest stack BELOW the
   deepest 64-bit frame (the kernel dispatched the 64-bit handler ON the
   guest stack; building below guest-Esp would be trodden on by the host
   dispatch path before its NtContinue), redirects the host CONTEXT
   (Rip=guest dispatcher, Rsp=frame, DF cleared) and returns
   EXCEPTION_CONTINUE_EXECUTION → ZwContinue → the kernel's context sanitizer
   (`ke/amd64/context.rs`) keeps CS 0x23 → guest dispatcher →
   `RtlDispatchException`(i386) walks the REAL fs:[0] chain (guest apps'
   handlers are genuine x86 SEH — DUMMY_PSEH only neuters our own C's
   `_SEH2_TRY`). Handler-continue resumes via the guest's `NtContinue`
   (already marshalled); handler-not-found second-chances via
   `NtRaiseException(...,FALSE)` — see 3. arm64 hardware redirect is OWED
   (FEX's simulate loop owns fault recovery there).
3. **`NtRaiseException` manual marshaller (both arches, wow64_manual.c).**
   With no debugger a first-chance user raise comes straight back to user
   dispatch, so skip the kernel: build the same guest dispatcher frame below
   the LIVE guest Esp (from BTCpuGetContext — NOT the captured Ctx32->Esp;
   the raiser's frames and the very structs being copied live between the
   two) and enter it via BTCpuSetContext. `DBG_PRINTEXCEPTION(_WIDE)_C` are
   consumed here (print `[GUEST32]` + resume at the captured context) exactly
   as the kernel's KD does natively — dispatching them to guest SEH would
   kill every `OutputDebugString` caller, since guest kernel32's `__except`
   is DUMMY_PSEH. Second chance (guest dispatcher found no handler) =
   loud process termination with the exception code.

Two mechanism corrections found while bringing E5 up (both were dead-wrong
first guesses that fault32 exposed):

* **VEH is NOT the dispatch path.** This build compiles ntdll with
  `-D_NTOSKRNL_`, so `RtlDispatchException` (amd64) elides the
  vectored-handler block entirely — a `RtlAddVectoredExceptionHandler` in
  wow64.dll never fires. The real dispatch point is the
  `Wow64PrepareForException` slot that `KiUserExceptionDispatcher` already
  calls unconditionally before `RtlDispatchException`; wow64.dll installs its
  handler there (ntdll exports the slot as data — dispatch.S + ntdll.spec).
  The hook ends in `NtContinue`, so `RtlDispatchException` never runs on the
  host frame and the guest's own i386 dispatcher takes over cleanly.
* **TEB32 stack bounds were never set — guest SEH had NEVER worked on amd64.**
  The kernel's `MmCreateTeb` deliberately leaves `TEB32.NtTib.StackBase/
  StackLimit` zero for a WoW64 TEB (comment: *"Guest stack bounds are set by
  wow64.dll at thread init"*), but wow64.dll's bootstrap never did. i386
  `RtlDispatchException` validates every fs:[0] registration against those
  bounds and rejects all of them when they're 0 — so every guest exception
  silently second-chanced. fault32 is the first guest to install a real SEH
  handler, which is why it surfaced only now. wow64.dll now writes the
  GuestStack bounds into TEB32 (+0x04/+0x08) during process bootstrap.

Oracle: **fault32.exe** (tools/fex/hello-compute/Fault32.c), the E5 ladder
rung (amd64): rung 1 hardware AV caught by a raw fs:[0] handler that verifies
the LIVE faulting context (marker regs ebx/esi/edi, AV parameters), retargets
Eip, and resumes; rung 2 `RaiseException` round-trip with parameters intact;
rung 3 `OutputDebugStringA` must return WITHOUT hitting the guest handler.
Exit 0x600D; 0xBAF1..5 name the failing rung. Gotcha for future guests: the
fs:[0] registration record MUST live on the thread stack —
RtlDispatchException validates against the TEB stack limits.

**Phase E6 — stock-binary robustness ladder on amd64 (2026-07-18).** Beyond the
authored probe guests and winmine, the amd64 winlogon ladder now launches more
unmodified stock i386 applets via a generic `TryStartWow64StockApp` helper
(same inverted "still running after Ns = PASS" verdict). Results:

* **notepad.exe — PASS.** Real window with a *working EDIT control* (the §9.67
  GlobalLock/LocalLock fix carries through WoW64 — the M11-era "dies in
  EDIT_LockBuffer" note is obsolete). Comdlg32/comctl32 in the closure.
* **sol.exe (Solitaire) — PASS.** cardlib is a *static* C++ lib (not a runtime
  cards.dll), so its import closure is already complete; heavy GDI + window.
* **calc.exe — was RESIDUAL, ROOT-CAUSED + FIXED in E7 (below).** The
  `ntdll32+0x2c5bb` first-chance AV was `RtlpRemoveFreeBlock` unlinking a
  guest-heap block with a NULL free-list `Blink` — guest-heap corruption from a
  WoW64 marshaller (`NtUserBuildHwndList`), not a shared native bug. See E7.

**The UNICODE_STRING NtUser* marshaller class (E6).** calc flushed out a whole
class the earlier guests never hit: `NtUser*` syscalls taking a
`PUNICODE_STRING` that win32u.spec types *flat* (`ptr`), so the generated flat
thunk hands 64-bit win32k the guest's 8-byte `UNICODE_STRING32` read as a
16-byte `UNICODE_STRING64` — a garbage `Buffer` (kernel reads/writes through
it) or an align-8 `ProbeForWrite` on the 4-aligned guest struct. Two bugchecked
calc directly before the fix (`NtUserGetAtomName` → 0x1E via garbage Buffer;
`NtUserGetClassName` → 0x80000002 DATATYPE_MISALIGNMENT → 0x1E), and two more
(`NtUserRegisterWindowMessage`, `NtUserGetKeyboardLayoutName`) were added
proactively — every GUI app hits them. All four are single-`UNICODE_STRING`
in/out marshallers (widen US32→US64, Buffer zero-extended as a guest <4GB
pointer, narrow Length back) in wow64_manual.c + manual-win32k.txt. This is the
exact M17 "flat-typed divergent payload" trap; the remaining un-marshalled
`PUNICODE_STRING` takers (FindWindowEx, EnumDisplayDevices, CreateDesktop,
SetWindowsHookEx, …) are owed as apps reach them.

**Phase E7 — calc guest-heap corruption root-caused + fixed: the OUT
handle-array narrowing gap (2026-07-18).** The E6 calc residual was a genuine
*guest-heap* corruption, not the contained-cascade curiosity it looked like.
Chased it by enriching the E5 guest-fault hook to dump the AV address + guest
GPRs, then a one-shot guest-memory window around the faulting block (guest
low-4GB is directly host-readable from wow64.dll):

* The `ntdll32+0x2c5bb` fault is `rtl_heap::heap::RtlpRemoveFreeBlock` on the
  `RemoveEntryList(&FreeEntry->FreeList)` unlink — `Blink->Flink = Flink` with a
  **NULL Blink** (`ecx=0`, AV write to addr 0). `FreeEntry`'s `FreeList.Flink`
  read back an **HWND** (`0x00020414`), not a heap pointer — i.e. a *busy*
  allocation holding `{HWND, 0}` was mis-coalesced as free.
* The memory dump was dispositive: a contiguous array of **`{HWND:4, 0:4}`
  8-byte entries** running *past* its heap block's end (`Size=0x11`=136 B, 16
  entries) into the adjacent block. That's **64-bit HWNDs written into a
  32-bit-per-entry buffer** — a 2× overrun.
* Root: **`NtUserBuildHwndList`** (behind `Enum{,Child,Thread}Windows`) fills a
  caller `HWND[]` and had **no WoW64 marshaller**. win32u.spec types the array
  slot flat (`ptr`), so the auto-thunk handed 64-bit win32k the guest buffer
  verbatim; the kernel wrote 8-byte HWNDs while the guest (user32
  `BuildHwndList`) sized it at `sizeof(HWND)=4`·count. calc's button enumeration
  overran the adjacent block; the damage surfaced minutes later as the NULL
  free-list link. **Native i386 is unaffected** (kernel writes 4-byte handles),
  so this is strictly a WoW64 marshalling gap — no A/B needed once the `{HWND,0}`
  8-byte stride was visible.
* **Fix:** `Wow64Manual_NtUserBuildHwndList` (wow64_manual.c + manual-win32k.txt)
  gives the kernel a host 64-bit scratch array (`cHwnd`·8 B), then narrows each
  handle's low 32 bits into the guest's 4-byte-per-entry buffer; the count slot
  (`pcHwndNeeded`, a `ULONG`) is width-identical and flat-passes; the count-query
  call (`phwndList==NULL`) passes straight through. The enriched one-line guest
  AV register dump stays in the E5 hook as fail-loud hygiene; the memory-window
  dump was removed.
* **General lesson (owed sweep):** any win32k/ntdll syscall that fills a caller
  **array of handles/pointers** typed flat in the .spec has this same 2×-overrun
  shape under WoW64 (`NtUserBuildNameList`, `NtUserBuildPropList`,
  `NtGdiGetXXX` handle arrays, `NtQueryDirectoryObject`, …). They're silently
  fine until a guest passes a non-empty buffer, then corrupt the guest heap far
  from the write. Add each to manual-win32k.txt with an array-narrowing
  marshaller as coverage reaches it.

**Phase E8 — new-surface stock-applet ladder + NtUserUnregisterClass (2026-07-18).**
Extended the amd64 winlogon ladder with four unmodified stock i386 applets chosen
to exercise win32k surface a prior guest never reached — **charmap** (font
enumeration + EnumFontFamilies callback), **mspaint** (heavy GDI regions/DIBs +
comdlg32 dialogs), **regedit** (treeview/listview common controls + registry-enum
arrays), **winver** (shell32 ShellAbout modal). Staged via the same
`wow64cpu/CMakeLists.txt` closure roots + `add_cd_file` mechanism (closure
recomputed complete: 59 modules, no new DLL), launched through the generic
`TryStartWow64StockApp` with the inverted "still running = PASS" verdict.

*Result:* charmap/mspaint/winver PASS on the first boot; **regedit BUGCHECKed
0x1E** and its fix is the E8 payload:

* **`NtUserUnregisterClass` was a silent-wrong flat thunk** (the M17 trap). The
  spec types it `(ptr ptr ptr)`; the real prototype is
  `NtUserUnregisterClass(PUNICODE_STRING ClassNameOrAtom, HINSTANCE, PCLSMENUNAME)`.
  The flat auto-thunk fed 64-bit win32k the guest's 8-byte `UNICODE_STRING32` read
  as a 16-byte `UNICODE_STRING64`; `ProbeAndCaptureUnicodeStringOrAtom` picked up
  `Buffer` from the wrong offset (garbage) and copied the "class name" from that
  wild address → C0000005 in win32k's `raw_move` → BUGCHECK 0x1E under DUMMY_PSEH.
  Symbolicated cleanly: fault at `win32k+0x258EC` (`cnt_rust::cabi::raw_move`),
  caller `NtUserUnregisterClass+0x117`. regedit/comctl32 hit it during class
  teardown.
* **Fix:** `Wow64Manual_NtUserUnregisterClass` (wow64_manual.c + manual-win32k.txt)
  widens arg0 US32→US64 via `Wow64pUS32to64` (the atom form — `Buffer` < 0x10000,
  `Length`=0 — survives the zero-extend + the kernel's `IS_ATOM` check). `hInstance`
  flat-passes; `pClassMenuName` is **ignored kernel-side** (`NtUserUnregisterClass`
  passes NULL to `UserUnregisterClass` — class.rs), so it needs no OUT marshalling.
  Post-fix: `NtUserUnregisterClass(0x123a) [manual] => 00000001` ×2, **regedit PASS**,
  all seven stock apps + all six probe guests green on one boot, zero faults.

* **Systematic insight (owed E9):** `marshalgen report` classifies only syscalls
  already in the NEEDS_MANUAL list — a **spec-flat-typed** divergent-payload
  syscall (typed `ptr`/`long`, so wow64gen emits a flat thunk and never stubs it)
  is **invisible** to the report. So the report cannot find the very class most
  likely to be silently wrong. The remaining spec-flat `PUNICODE_STRING` NtUser*
  takers from the E6 note (`FindWindowEx`, `EnumDisplayDevices`, `CreateDesktop`,
  `SetWindowsHookEx`, …) share this shape and are owed. The durable fix is a
  spec-vs-prototype audit tool (scan win32u.spec `Nt*` `ptr` args against the real
  ntuser.h/ntgdi.h prototype; flag any pointing at a divergent payload not on the
  manual list) — that would mechanically surface them all *before* a guest crashes.

**Phase E9 — the flat-typed PUNICODE_STRING NtUser* sweep + more applets
(2026-07-18).** Extended the ladder with five more stock i386 applets —
**taskmgr** (listview + `NtQuerySystemInformation` process array), **wordpad**
(richedit/riched20), **clipbrd** (clipboard), **osk** (on-screen keyboard),
**magnify** (timer-driven screen-capture BitBlt). taskmgr/wordpad/clipbrd/magnify
PASS on the first boot with zero new marshallers (the layer is robust across
listview, richedit, clipboard, and screen-BitBlt surfaces); **osk BUGCHECKed
0x1E** on `NtUserFindWindowEx` — the same `win32k+0x258EC` `raw_move` PC as
regedit's E8 crash, the same flat-typed-PUNICODE_STRING trap.

Rather than fix one syscall, this closed the **whole class** the E8 insight
predicted. The spec-vs-prototype scan proposed there (every `NtUser*` `ptr` arg
in win32u.spec whose real ntuser.h prototype is a `PUNICODE_STRING`, minus the
manual list) found **11 uncovered members**, each a latent kernel BUGCHECK:

* **9 pure US-widen** (the proven `Wow64pUS32to64` pattern — widen the US arg(s),
  flat-pass the rest, narrow handle returns): `NtUserFindWindowEx` (2×US),
  `NtUserGetWOWClass`, `NtUserSetWindowsHookEx`, `NtUserSetWinEventHook`,
  `NtUserRegisterUserApiHook` (2×US), `NtUserDrawCaptionTemp`,
  `NtUserLoadKeyboardLayoutEx`, `NtUserEnumDisplaySettings`,
  `NtUserEnumDisplayDevices` (the last two also take a DEVMODEW/DISPLAY_DEVICEW
  OUT struct — non-divergent, no pointers, identical 32/64, so the guest buffer
  flat-passes). All in wow64_manual.c + manual-win32k.txt.
* **2 that also carry a divergent struct** — `NtUserCreateDesktop`
  (OBJECT_ATTRIBUTES) and `NtUserGetIconInfo` (ICONINFO HBITMAP widen) — are
  listed in manual-win32k.txt **without** a dispatch entry, so they hit the loud
  `WOW64_MANUAL_UNIMPL` stub (a clean error, **not** a bugcheck) instead of the
  silent-wrong flat thunk. No ladder app reaches them; a marshaller lands when
  one does. (This respects the M18 "don't ship a marshaller no guest can
  disprove" rule — the 9 pure-US ones are instances of an already-verified
  pattern, and osk verifies `NtUserFindWindowEx` directly.)

*Result:* all **12 stock apps** (calc/notepad/sol/charmap/mspaint/regedit/winver/
taskmgr/wordpad/clipbrd/osk/magnify) + all **8 probe guests** (hello32→fault32,
winmine) green on one boot, zero faults. The flat-typed-PUNICODE_STRING class —
the "silently wrong until a guest hits it, then a kernel bugcheck" hazard that bit
E6 (GetAtomName/GetClassName), E8 (UnregisterClass), and E9 (FindWindowEx) — is
now closed across the win32k surface.

**Phase E10 — `marshalgen audit`: the flat-thunk blind spot becomes a BUILD-TIME
GATE (2026-07-24).** E6, E8 and E9 each found one member of the same class the
expensive way — boot an applet, watch the box BUGCHECK, symbolicate, marshal one
syscall. E8 named the reason that keeps happening and E9 hand-ran the fix once;
E10 makes it mechanical.

*Why the existing tooling could not see it.* wow64gen types its thunks from
`win32u.spec` / `ntdll.spec`, and a `.spec` arg says only **"this is a pointer"**
— never what it points AT. So a syscall whose real prototype takes a pointer to a
divergent payload is never flagged `WOW64_NEEDS_MANUAL`, gets a flat thunk, and
therefore **never enters the list `marshalgen report` iterates over**. The report
is structurally incapable of finding the class most likely to be silently wrong.

*The audit inverts the question.* Instead of "are the stubs we already flagged
marshalled correctly?", `marshalgen audit` asks: **of every syscall production
dispatches through a LIVE flat thunk, which have a prototype whose payload
provably diverges 32↔64?** Ground truth is the GENERATED table (an entry is `0`
when a hand/auto marshaller owns it, `wow64_thunk_<Name>` when the flat thunk is
live), cross-checked against the real header prototype through clang's own
`-fdump-record-layouts` for i686 and the host — the same layout engine M17
already trusts. Two verdicts are kept apart on purpose: PROVEN divergence is a
finding, while "marshalgen could not resolve that layout" is a hole in the TOOL,
reported separately so it can be driven down instead of burying the real hits.
Wired into `vibeos-cli check` beside `verify_svc_numbering`, baseline gates NEW.

*What it found on the win32k table* (737 entries, 628 live flat thunks) — **16
syscalls**, every one a latent repeat of a bug we already paid for. The clang
layout diff caught far more than a regex over the headers would: it resolves
**typedef'd** struct pointers (`LPMSG`, `PCOMBOBOXINFO`, `PGUITHREADINFO`, …)
that a textual scan cannot see.

* **`NtGdiCreateDIBSection`** — the E7 overrun shape on a **hot** path. Guest
  gdi32 passes `&bmBits`, a **4-byte** stack local (`gdi32-rust/src/objects/
  bitmap.rs`), and the flat thunk let 64-bit win32k store an **8-byte** `PVOID`
  there, smashing the next 4 bytes of guest stack on *every* DIB section —
  mspaint, magnify, any double-buffered app. Marshalled (host-width slot,
  narrowed back). A mapping above 4GB is unrepresentable to the guest, so that
  case now fails the call loudly instead of truncating the pointer.
* **`NtUserResolveDesktop`** — both halves at once: `PUNICODE_STRING` path +
  `HWINSTA*` OUT cell. **`NtUserOpenDesktop` / `NtUserOpenWindowStation` /
  `NtUserCreateWindowStation`** — `OBJECT_ATTRIBUTES` (24→48 bytes, nested
  `UNICODE_STRING`). All four marshalled, reusing `Wow64pMarshalOa` — the helper
  the entire ntdll open family already runs on.
* **The rest of the MSG family** — `tagMSG` is 28 bytes on i386 and 48 on the
  host, and the pump three (Get/Peek/DispatchMessage) have been marshalled since
  M8, but `NtUserTranslateAccelerator` (any app with an accelerator table),
  `NtUserCallMsgFilter` (modal loops / `MSGF_*` hooks) and
  `NtUserRealInternalGetMessage` were still flat. Same `Wow64pMsg32To64` pair.
* **8 more** (`NtUserSetCursorContents`/ICONINFO, `NtGdiAddEmbFontToDC`,
  `NtGdiFONTOBJ_cGetGlyphs`, the `NtGdiDd*`/`NtGdiDvp*`/`NtGdiDxg*` DirectDraw
  handle-array calls) carry a divergent struct no ladder guest reaches, so per
  the M18 rule they became loud `WOW64_MANUAL_UNIMPL` stubs — a clean error
  instead of the silent-wrong flat thunk they had.

**All 16 verified cleared** by re-running the audit against the rebuilt table.

*What it found overall*: **294 divergent params across 201 live flat thunks**
(win32k 165/120, ntdll 129/81), plus 140 unknown-layout (tool blind spots,
reported but never gated) and 3 no-prototype. After the 16 fixes, **291 remain
and are BASELINED** (`tools/lint-baseline/marshalgen_audit.txt`) as the visible
work queue — the same treatment `scan_silent_failures` (~2.1K) and
`scan_cc_mismatch` (~330) get. The gate blocks anything NEW. Burning the
baseline down is E12; prefer `marshalgen gen` codegen over hand-written
marshallers for the plain-WIDENING cases, and note that the driver DDI families
(XLATEOBJ/STROBJ/PATHOBJ/FONTOBJ) are not guest-reachable at all.

*What it found on the ntdll table* (296 entries, 230 live flat thunks) is a much
larger cold-path residual — ~70 struct-divergent and ~44 pointer-to-handle
params. That is the audit doing its job: the baseline captures it as a visible
work queue (house pattern — `scan_silent_failures`, `scan_cc_mismatch`) and the
gate blocks any NEW one, rather than pretending the surface is clean. One entry
in it is not a line item but a **milestone**:

* **`NtCreateThread` is an unmarshalled flat thunk** — `OBJECT_ATTRIBUTES` +
  `PCONTEXT` + `PINITIAL_TEB` + `PHANDLE` + `PCLIENT_ID`, and an i386 `CONTEXT`
  bears no resemblance to an amd64/arm64 one. There is no thread-creation path
  in wow64.dll at all, so **a multithreaded 32-bit guest is unsupported today**
  (our ladder applets simply never create a thread inside their launch window).
  Fixing it is not a struct marshal — a new guest thread needs its whole CPU
  context set up the way process startup does. Owed as its own milestone.

**Phase E11 — six more stock applets; they immediately found E13 (2026-07-24).**
Added **spider** (cardlib DIB blitting — the disproof rung for E10's
`NtGdiCreateDIBSection` fix), **eventvwr** (listview + the first 32-bit guest to
drive real RPC to a 64-bit service), **devmgmt** (setupapi enumeration),
**msconfig** (comctl32 property sheets), **sndrec32** (winmm) and **kbswitch**
(shell tray-icon IPC). All six are **STAGED on the livecd** (closure recomputed
complete: 60 → 61 modules, one new DLL).

*Result — the ladder did exactly its job, which this time means it found a bug
rather than a clean sweep.* On a quiet host the twelve existing applets and all
eight probe guests PASS (the earlier five "exits" were pure host-contention
artifacts of a native-KVM boot taken while four builds and two gates were
running — a re-run on an idle host was clean; AGENTS.md warns about exactly
this). **spider PASSed on one boot and BUGCHECKed the next**, and eventvwr
BUGCHECKed too — both with the identical stack:

```
0x1E / C0000005 at win32k+0x258EC
  cnt_rust::cabi::raw_move <- memcpy <- win32k_rust::message::g_MsgMemory
                                     <- co_IntDoSendMessage
```

That is **E13**, a real pre-existing gap this lane had never reached:
`g_MsgMemory` (win32k-rust/src/message.rs:407) sizes SendMessage pointer-lParam
payloads with the **host's** `size_of::<T>()` — MINMAXINFO, STYLESTRUCT,
WINDOWPOS, RECT, MEASUREITEMSTRUCT, DRAWITEMSTRUCT, HELPINFO, MDINEXTMENU — so a
32-bit sender's buffer is over-read. M15a/M16/M18 covered the **callback**
direction (kernel→user) and `NtUserMessageCall`'s WINDOWPOS lParam; the
**SendMessage** direction (user→kernel→window) was never covered. The fix is
win32k-side and needs conversion, not just a size swap — DRAWITEMSTRUCT and
MEASUREITEMSTRUCT carry HWND/HDC/ULONG_PTR fields that shift.

*E13 is PRE-EXISTING — proved by control experiment, not inferred.* With this
whole batch stashed out and amd64 rebuilt from pristine master, the boot
BUGCHECKs with the **byte-identical** stack (same `win32k+0x258EC`, same
`+0x2597D`/`+0x181B24`/`+0x6AC82` frames) at the doom/testgui rung, with the same
12/12 applet PASS. So master's amd64 boot already dies ~3.5 minutes in today;
E11 merely reaches the fault sooner. (Worth doing that control: the fault PC is
shared with the E6/E8/E9 class and three of E10's own new marshallers are on
message paths — "my code looks uninvolved" was not good enough to land on.)

*Disposition:* the six applets stay staged (launch `spider.exe` or
`eventvwr.exe` by hand to repro E13 in seconds) but their winlogon
**auto-launch block is held back**, because auto-launching a timing-dependent
box-killer would stop the twelve existing rungs from ever reporting — spider and
eventvwr reach the fault ~6 applets earlier than doom does, so auto-launching
them would COST the coverage the ladder already has. Re-enabling that block is
the regression test for E13. Tracked, not dropped.

*Durable lesson (field manual §9.115):* when a generator's type source is
narrower than the truth (a `.spec` describing pointers but not pointees), the
generator cannot flag what it cannot see, and **a report that iterates the
generator's own flagged set inherits exactly that blind spot**. The check has to
cross the artifact it generated against the independent source of truth.

**Phase E13 — the `NtUserMessageCall` boundary: three divergences, one syscall
(2026-07-24).** E11's ladder found a box-killer (§9.116): win32k's `g_MsgMemory`
sizes SendMessage pointer-lParam payloads with the HOST's `size_of::<T>()`, so a
32-bit sender's smaller struct is over-read — `0x1E`/`C0000005` at win32k's
`raw_move`. E11 recorded it as "the fix is win32k-side"; **that was wrong**, and
one grep overturned it: every `co_IntDoSendMessage` call — the only consumer of
`CopyMsgToKernelMem`/`CopyMsgToUserMem` — is inside **`NtUserMessageCall`**. The
whole class sits on one thunk boundary, which is where real Windows puts it
(wow64win.dll owns message thunking; the kernel only ever sees host-width structs).

*Auditing that boundary properly found three divergences, not one* — and the one
nobody had reported was the worst:

| Arg | i386 | host | Consequence |
|---|---|---|---|
| lParam payload (`g_MsgMemory` msgs) | varies | varies | over-read → BUGCHECK (the reported bug) |
| `ResultInfo` / `FNID_SENDMESSAGE` | `ULONG_PTR` **4** | **8** | **every** SendMessage smashes 4 bytes of guest stack |
| `ResultInfo` / `FNID_SENDMESSAGE{FF,WTOOPTION}` | `DOSENDMESSAGE` **12** | **16** | over-read *and* over-write |

The middle row had been live since the first 32-bit GUI guest: user32's
`SendMessageW` passes `&Result`, a 4-byte `LRESULT` local, and win32k stores 8
bytes into it. It never crashed because the adjacent stack was usually dead — the
E7 shape on a far hotter path.

*Implementation.* `Wow64Manual_NtUserMessageCall` now marshals the payload through
the **generated** `wow64_cb_automarshal.c` table (keyed by `Msg`, from clang
record-layout diffs) that the callback direction already used, so the two
directions cannot disagree about a layout. `WINDOWPOS` had been an "extra struct"
with no `Msg`→pair row, so the **callback** direction had been copying 40 host
bytes into a buffer the guest reads as 28 — an unfiled bug, fixed by adding the row
the SendMessage side needed anyway. M16's hand-written `FNID_DEFWINDOWPROC`
special case is now one row of that table.

Widths were confirmed with marshalgen's own layout engine, not arithmetic —
which also proved `MINMAXINFO`/`RECT`/`STYLESTRUCT`/`DEV_BROADCAST_HDR` are
width-IDENTICAL, so "helpfully" marshalling one would have corrupted it.
Marshalling is scoped by what the callee *does* with the arg: deliberately not
`FNID_CALLWNDPROC*` (repacked into a `CWPSTRUCT` the callback path handles) and
not the async posts (a host stack buffer handed to an async post is a
use-after-return). `WM_NCCALCSIZE`/`WM_COPYDATA`/`WM_CREATE` embed a pointer or a
packed tail; on the `g_MsgMemory` FNIDs they now **fail loudly** rather than
flat-pass — losing a message beats losing the machine.

*Result.* The old signature is **gone** from the log entirely; **spider and
eventvwr — the two applets that had been killing the box — PASS**, and the E11
block is re-enabled for them. **21 applets + 7 probe guests green on amd64.**

*And the ladder immediately moved to the next bug*, which is exactly what it is
for: `devmgmt` (never reached before) and `doom` now BUGCHECK 0x1E via a
**different** chain — `ProbeForWrite` → `ExRaiseAccessViolation` →
`RtlRaiseStatus`, unhandled under DUMMY_PSEH (task #67). A control run with the
callback-direction change reverted reproduced that chain byte-identically, which
is what proves it is not E13's doing. devmgmt/msconfig/sndrec32/kbswitch stay
staged with their launch held, on the same coverage argument as before.

*Also fixed in passing (§9.118a):* `rtl-diag`'s `[RAISE]` helper reported a
**stack** address as the raise site — its "kernel-text-looking" window
(`0xFFFFF800..`–`0xFFFFF8FF..`) also covers kernel stacks. It now rejects
candidates within 1MB of `rsp`. The tell was that the known-good `seh_poc` raise
lied the same way.

*Durable lessons:* field manual **§9.118** (fix width bugs at the thunk boundary,
audit the whole boundary not just the reported arg, reuse the generator's table,
verify widths with the layout engine) and **§9.118a** (a triage tool that lies
costs more than no tool — validate it against a case whose answer you know).

## M11: the first STOCK binary — `testgui.exe` (i386, unmodified) under WoW64

**Why.** Every guest through M10 was written by us, and a guest you author can only
test what you thought of — worse, it silently *avoids* whatever the layer doesn't
support, because you never wrote the call that would have tripped it. M11 stops
authoring guests: it runs `base/applications/testgui` built for i386 exactly as the
LiveCD ships it (real `msvcrt` CRT startup, real import table, real window + child
EDIT control). Full writeup + the five bugs it flushed out: field manual **§9.67**.

**Result.** The stock binary reaches `wWinMain` → `RegisterClassExW` →
`CreateWindowExW` → `WM_CREATE`, and then dies in *exactly the same place as the
NATIVE arm64 build of the same source* — a pre-existing user32 EDIT-control bug
(`EDIT_LockBuffer` → `LocalLock` returns NULL), not a WoW64 bug. That identical
failure IS the milestone: the emulation layer is now faithful.

`hello32`/`gui32`/`win32`/`file32` all still 0x600D with the full closure staged.

### The SysWOW64 DLL closure (`tools/fex/wow64-closure.py`)

A CRT-linked binary drags in a real import graph: `testgui → user32 → advapi32 →
rpcrt4 → wininet → …`, 53 modules. The old M8 rule ("do NOT stage advapi32/rpcrt4/
msvcrt — they leave unbound IATs") was right about the hazard and wrong about the
cure: a DLL that is staged but whose *own* imports are missing loads **half-bound**,
and the first call through an unbound slot jumps to garbage. Not-staging only worked
because nothing on the M8 path called into those DLLs.

The cure is the **complete** closure — no module is ever left half-bound. A bigger
PARTIAL set has exactly the same landmine, just later. `wow64-closure.py` computes it
at stage time (stripped variants; 30 MB) and **hard-fails rather than emit a partial
set**. The loader's `LDR: Skipping missing import` line is the standing oracle that
the closure is still complete — it must never fire for a guest module.

### The A/B control (`TryStartNativeTestGui`)

winlogon also spawns the **native arm64** build of the same testgui.exe. Without a
control, every crash inside a shared library looks like a WoW64 bug. With it, one
boot partitions the entire search space. Keep it.

### OWED (M11)

- **The callback OUT-payload copy-back.** `WM_GETMINMAXINFO` / `WM_NCCALCSIZE` /
  `WM_WINDOWPOSCHANGING` pass their payload as an OUT parameter — the window proc
  writes into the struct win32k then reads back. We currently DROP the guest's edits
  (restoring the kernel's original bytes), which is what we have shipped since M8.
  Implementing the copy-back regressed `win32.exe` 0x600D → 0x600E: WM_PAINT /
  WM_NCPAINT / WM_ERASEBKGND vanished and the size-negotiation messages each went
  1× → 3× (win32k never converged on the geometry the guest handed back). Evidence
  and the disabled code are in `wow64_callback.c`. Root-cause why the guest's
  DefWindowProc returns geometry win32k won't converge on, then re-enable.
- **The remaining loud refusals** (`wow64_manual.c`): `NtNotifyChangeDirectoryFile`,
  `NtNotifyChangeKey(MultipleKeys)` — all inherently async, blocked on the kernel-side
  `IO_STATUS_BLOCK32` fix (M10's owed item); `NtSetSystemInformation` — class-dependent
  pointer-bearing buffers.
- **`NtDeviceIoControlFile`/`NtFsControlFile` pass their IOCTL buffers through
  untranslated.** Most are flat; a minority embed pointers. Fine until a guest sends
  a pointer-bearing IOCTL, at which point the failure lands inside the driver.
- ~~user32 EDIT control is broken NATIVELY~~ **FIXED 2026-07-13 (field manual §9.70).**
  Root cause was in kernel32, not user32: `GlobalLock` had its SEH *filter expression*
  translated into a runtime value (`let __except_val = EXCEPTION_EXECUTE_HANDLER;` then
  `if __except_val != 0`), so the exception HANDLER ran on every call and the real result
  was dead code — `GlobalLock` (and therefore `LocalLock`) returned NULL unconditionally.
  Every edit box in the OS, on every arch, had a NULL text buffer. With it fixed, the
  stock testgui.exe **PASSES under WoW64**: EDIT control created, WM_PAINT, message pump
  alive, still running — same as native.

### M11 FINAL: PASS (2026-07-13)

`testgui.exe` — a **stock, unmodified, CRT-linked i386 GUI binary** — now runs under
WoW64/FEX on arm64 to completion:

```
[TESTGUI] wWinMain entered (hInstance=00400000)
[TESTGUI] RegisterClassExW succeeded
[TESTGUI] WM_CREATE received
[TESTGUI] EDIT control created (hWnd=00020188)
[TESTGUI] CreateWindowExW succeeded (hWnd=00020186)
[TESTGUI] WM_PAINT received
[TESTGUI] Window shown, focus set, entering message loop
[WINLOGON] testgui.exe still running after 20s = PASS
```

Both it and the native arm64 build of the same source sit in the taskbar side by side.
hello32/gui32/win32/file32 all 0x600D; shell ready ×2; zero `NEEDS MANUAL` stubs; no
BUGCHECK. The A/B control (`TryStartNativeTestGui`) is what made the last bug findable:
when native and WoW64 failed *identically* the bug was shared (kernel32's GlobalLock);
the moment native passed and WoW64 didn't, the remaining delta was a stale guest
kernel32 in the ramdisk. Keep the control.

## M12: x86 DOOM under WoW64 on arm64 — the north star

`tools/doom/build.sh i386` → doomgeneric's Win32/GDI backend, a genuinely foreign
mingw binary that has never heard of ReactOS. Its import closure is **exactly the
same 53 modules testgui needs**, so M11's closure work covered it with nothing added.
Staged into `syswow64/` (+ the 4 MB shareware IWAD) and spawned by
`winlogon-rust::TryStartWow64Doom` (silent no-op unless staged; it also suppresses
the testgui pair and notepad, which would otherwise sit on top of DOOM's window —
doomgeneric creates its window at `CW_USEDEFAULT` and never calls
`SetForegroundWindow`).

**It runs.** Under FEX on arm64, the x86 binary:
- starts through msvcrt's CRT and binds its whole import graph,
- opens and reads the 4 MB IWAD (`NtReadFile` ×934),
- registers its class, creates its window, and **`NtUserShowWindow` shows it**
  (it appears in the taskbar as "DOOM Shareware"),
- pumps messages (`NtUserPeekMessage` ×4000+) and paces frames (`NtDelayExecution`),
- and **renders**: `NtGdiStretchDIBitsInternal` ×4000+, each copying 400 scanlines.

### The bug it found: 195 misrouted syscalls (field manual §9.71)

DOOM was the first guest to call a win32k syscall above index 541 — and there the
thunk table was **off by one**. `w32ksvc32.h` gates `NtUserSetWindowLongPtr` on
`#ifdef _WIN64`; `wow64gen.py` parsed the file as flat text, included that entry on
the 64-bit host, and shifted every service number after it. The guest's
`NtUserShowWindow` (0x1227) dispatched to the host's `NtUserShowScrollBar`. Nothing
faulted — the neighbours have plausible signatures — so DOOM's window simply never
became visible while it happily rendered thousands of frames into a DC nobody could
see. Fixed by making `parse_sysfuncs` expand the list **as the 32-bit guest does**,
and permanently guarded by `tools/wow64gen/verify_svc_numbering.py` (wired into
`vibeos-cli check`), which compares the generated table against the REAL i386 syscall
stubs disassembled out of the build.

### OWED: DOOM's frames don't reach the screen

DOOM renders, but the pixels don't land in its window. **This is NOT a WoW64 bug** —
the A/B control proves it: the NATIVE arm64 build of the same doomgeneric
(`VIBEOS_DOOM_AB=1` stages it into system32) misrenders the same way, painting its
framebuffer *outside* its own window (over whatever else is on the desktop).


**Investigation notes (2026-07-13, instrumented then reverted — findings only).**
Traced with one-shot DbgPrints in `NtUserShowWindow` and `DceResetActiveDCEs`.
What is RIGHT: the show path reaches `co_WinPosSetWindowPos -> DceResetActiveDCEs`;
DOOM's cached DCE **is** found and revisited on show; its visible region is
recomputed correctly to `bbox=(0,0,640,400)` complexity=2 (NOT empty); the window
is at (0,0), DC origin (0,0), `StretchDIBits` returns 400 scanlines. So the clip
is not the problem. Two real anomalies remain for the win32k lane to chase:
  1. DOOM's window has `rcWindow == rcClient == (0,0,640,400)` — **no non-client
     inset at all**, i.e. `WM_NCCALCSIZE`'s client-rect writeback is being dropped
     (compare testgui, which also shows rcWindow==rcClient). A window with a
     caption should have a smaller client rect.
  2. gui32's magenta (a DIRECT display DC) DOES reach the framebuffer, while
     DOOM's window-DC blits do not — with a provably-correct visrgn. So the
     divergence is between a window DC and a direct DC at blit time (surface
     target / screen offset), not the visible region. Start there, not at DCE.

The shape: doomgeneric does

```c
    s_Hdc = GetDC(hwnd);        /* window not yet visible */
    ShowWindow(hwnd, SW_SHOW);  /* becomes visible AFTER the DC was cached */
    ...
    StretchDIBits(s_Hdc, ...);  /* forever, into that one cached DC */
```

so the DC's visible region is computed while the window is hidden and must be
recomputed when it is shown (`DceResetActiveDCEs` / DCE visrgn invalidation). That
is win32k/GDI territory, not the WoW64 layer — it belongs to a win32k lane. The
WoW64 side of DOOM is done.

## M14: x86 DOOM under WoW64/FEX on arm64 — DONE (renders + playable, 2026-07-15)

Status: **CLOSED. DOOM renders full gameplay on every boot AND takes keyboard
input.** Two independent root-caused fixes closed it:
- **Rendering (M13, commit 6213b784ad7):** `DC_vPrepareDCsForBlit` checked the
  wrong `fs` bit (0x04 DC_CANCELED vs 0x10 DC_DIRTY_RAO), so the blit clip
  object was never rebuilt and cached DCs drew through a stale region. 12-boot
  soak → 10/10 render (2 host-contention reaps, not misrenders). Field manual §9.86.
- **Input (M14, commit 525084ded8b):** the guest's `TEB32.Win32ThreadInfo` was
  never populated, so user32's window-ownership check in `IntCallMessageProc`
  dropped every keystroke with ERROR_MESSAGE_SYNC_ONLY. `Wow64PublishClientInfo`
  now writes it. Verified: QMP-injected ESC/ENTER navigate DOOM's main menu →
  skill-select; DROP count 8→0; WM_KEYDOWN reaches the guest WndProc.

Reproduce with the session's `/tmp/doom-play.sh` (restage FEX ramdisk, boot arm64,
drive keys via QMP, save `/tmp/play-{0..4}.png`) — or `tools/doom/play.sh`.
The historical "rendering INTERMITTENT" text below is kept for the debugging
narrative; it is SUPERSEDED by the §9.86 fix.

**Post-fix 6-boot render soak (2026-07-15, clean probe-free build, 6-way
contended):** the render fix is **4/4 on every boot where doom's process
actually ran** (~2200 StretchDIBits each, full gameplay). The two misses were
NOT render failures: one boot was reaped at the 420s lifetime cap before doom's
frame loop ramped (only 20 blits — host CPU contention from 6 concurrent arm64
TCG VMs), and one was a genuine **doom guest-code crash** — `C0000005` NULL-read
at guest PC `0x5BFE18` (in doom.exe's own image, `ExAddr=0`), i.e. doom
dereferencing a NULL it got back from some WoW64 syscall without checking. That
last one is a **residual startup flake** (startup-death family, cousin of §9.85),
exacerbated by contention, and is INDEPENDENT of the render/input fixes (both
run long after doom init). Characterized (not yet fixed):
- It is a **NULL function-pointer CALL**, not a data deref: `[UEXC-KILL] PC=0x0
  LR=0x5BFE18` — doom's code at `0x5BFE18` called through a pointer that was 0.
- `0x5BFE18` is between doom.exe's image (`~0x400000`, top data symbol
  `__tls_end` @ `0x4A8004`) and the first SysWOW64 DLL base (`0x6CD80000`) — so
  it's in doom's own later sections (`.idata`/thunk region), i.e. doom calling
  an IAT/callback slot that was 0.
- It is **NOT a loader closure gap**: no `LDR: Skipping missing import` /
  `c0000135` on the crashed boot OR the rendering boots (checked both).
- Only reproduced under 6-way host contention (sibling agents' VMs loaded the
  host); ~8 single-VM doom boots this session never hit it.
OWED: repro under controlled contention with a per-slot syscall/callback trace
to catch which slot doom read as 0 (a win32k callback-table entry or a lazily-
bound thunk are the leads). Not a regression; not chased further this session.

**Rendering — works on some boots, magenta on others (M13 is NOT closed).** A boot
with only the WoW64 x86 DOOM (no A/B native build) *sometimes* shows DOOM's
attract-mode demo animating in its window on the desktop — the E1M1 corridor, "DOOM
Shareware" in the taskbar, correct compositing around it — and *sometimes* shows only
the window's magenta background with no game frame at all. Measured 2026-07-13: 3 of 4
recent boots rendered, 1 (a cold-cache boot) stayed magenta. On the magenta boot DOOM
still called `StretchDIBits` **2598 times** (same order as the rendering boots'
3229–6456) — so DOOM is blitting continuously; the pixels just don't reach the screen.
That is the M13 window-DC-blit / visrgn-on-show bug below, timing-dependent (worse on
cold-JIT boots), **still owed to the win32k lane**. So M14 rendering is a coin-flip,
not reliably playable yet.

The earlier "frames don't reach the window" read was ALSO muddied by two red herrings
(distinct from the real intermittent bug), which is why it took so long to localize:
  1. **win32k draws the mouse cursor (32×32, a fresh DC each time) in the foreground
     thread's context.** Instrumenting `NtGdiStretchDIBitsInternal` gated on
     `PsGetProcessImageFileName == "doom"` caught *those* cursor blits, not DOOM's
     frame — they saturated any bounded trace before the 640×400 blit was sampled.
     Gate additionally on `cxDst >= 320` to see the real frame. (A `pSurface`-null
     early-return theory was also refuted: DOOM's window DC is `DCTYPE_DIRECT` with a
     valid surface.)
  2. **`VIBEOS_DOOM_AB=1` stages TWO `doom.exe`** (native arm64 in system32 + WoW64
     x86 in syswow64). The native one crash-loops on `CsrClientConnectToServer
     0xc0000041` and both match a `"doom"` image gate — its noise muddied the trace.
     Drop A/B (plain `VIBEOS_FEX=1`) to diagnose the WoW64 build alone.

**Input works, proven at every layer.** Injected keys travel arm64 virtio-keyboard →
win32k `ProcessKeyEvent` (not eaten, `bPostMsg=1`) → posted to doom.exe's focus queue
→ `co_MsqPeekHardwareMessage` returns them (`gate=1`) → the manual WoW64
`NtUserPeekMessage` thunk down-converts `WOW64_MSG64→32` and delivers correct
`WM_KEYDOWN/UP` to the guest with the **correct VK + scancode** (verified: ESC=0x1B,
ENTER=0x0D, CTRL=0x11, UP=0x26, each with matching `lParam` scancode/transition
bits). `NtUserPeekMessage` returns non-zero to the guest — DOOM's `wndProc` retrieves
them. So the WoW64 keyboard marshalling is correct; nothing is mangled.

**One playability enabler was the boot config.** `qemu/start.sh` (the agent/manual
boot tool) did not attach virtio input devices, so a GUI guest rendered but received
no input. `vm.py`/smoke and `run.sh --gui` already attach them; `start.sh` now does
too (virtio kbd+mouse + modern virtio-mmio, mirroring vm.py). That fixed *input*
availability — but rendering reliability is a separate, still-open win32k bug (next).

**PARKED — the remaining M14 blocker: the intermittent window-DC render (M13).** DOOM
blits its 640×400 frame thousands of times per boot into a DC it cached via
`GetDC(hwnd)` *before* `ShowWindow` (see the doomgeneric shape above). It works most
boots but not all — on a miss the window shows only its magenta background despite
`StretchDIBits` running continuously. This is a timing-dependent win32k race (worse
cold-JIT), **NOT a WoW64 bug** — the native arm64 A/B DOOM misrenders identically.

**2026-07-13 — the "stale empty visrgn / blit clips to nothing" hypothesis is DISPROVEN.**
Three instrumented soaks (probes parked in git stash `ca1bdab8` —
`m13-render-probes`) settled the clip path completely. On *every* boot, rendered and
magenta alike:
- **visrgn is correct**: `DceResetActiveDCEs`→`DceUpdateVisRgn` on ShowWindow recomputes
  the cached DC's visrgn to `(0,0,640,400)c2` (M13-RST probe in `windc.rs`).
- **surface is correct + identical**: `fffffa80001efe50` on every boot (M13-VIS in `dibobj.rs`).
- **the blit clip (RAO) is correct**: `CLIPPING_UpdateGCRegion` (reached via DOOM's real
  path `StretchDIBits`→SRCCOPY branch→`NtGdiBitBlt`) computes `rao=(0,0,640,400)c2`,
  clip/meta/api all null, org=(0,0), on rendered *and* magenta boots (M13-UGR in `cliprgn.rs`).

So clip/surface/visrgn are all correct 100% of the time; the magenta is **not** a
clipping failure. What the screenshots show is that DOOM's `NtGdiBitBlt` intermittently
deposits **no pixels** into a correctly-clipped, correct surface — the window's
erase-background magenta persists. That points downstream of the clip, at the **DIB→surface
pixel copy** (`IntSetDIBits` / `EXLATEOBJ` color translation / `EngBitBlt`) or a **ramfb
scanout / dirty-rect timing** race — deep native win32k/framebuffer, off the WoW64 mission.
Next probe for whoever resumes: instrument inside `NtGdiBitBlt`/`EngBitBlt` (bytes actually
written to `pdc.dclevel.pSurface`) or dump the surface bits right after the blit, gated on
process=doom, to catch the no-op copy in the act. The related small-magenta patch at the
window's top-left on *rendering* boots (leftover un-painted window-background fill) and the
`rcWindow == rcClient` no-non-client-inset anomaly (`WM_NCCALCSIZE` client-rect writeback
dropped) are lesser follow-ups in the same win32k lane.

**2026-07-14 overnight — REFRAMED with the owed probe armed. Evidence chain:**

- **The old probe placement was wrong for doom's path.** Doom's `StretchDIBits` is
  SRCCOPY, which takes GreStretchDIBitsInternal's *memory-DC* branch
  (`IntSetDIBits` → `NtGdiBitBlt`, dibobj.rs:2109), NOT the `IntEngStretchBlt`
  branch. The new `M13-BLT` probe checksums guest DIB / converted tmp bitmap /
  dest strip pre+post around that real path.
- **12-boot instrumented soak (`/tmp/m13pix`)**: every boot shows `ret=1`,
  `tmpmid==dibmid` (conversion pipeline delivers the frame), and `pre==post`
  at the dest-rect mid-strip — the blit deposits nothing at the sampled row.
  `dstD=(4,23,644,423)` constant: DC origin says doom's client starts (4,23),
  window at (0,0) outer 640×400 (AdjustWindowRect added nothing under wow64 —
  separate small bug).
- **Screendump ground truth**: doom's window is *entirely absent* on all 12
  boots — no caption, no borders, no fill; only its taskbar button exists. But
  doom CONTENT (title screen on boot1, live demo gameplay on boot10!) shows in
  a ~192×123 patch at exactly **win32.exe's dead window's client rect**
  ((44,63)-(236,186)) — doom's 640×400 blits are being clipped to a stale
  region belonging to a destroyed window.
- **Dead lore killed #1**: the "small magenta patch at top-left" is
  `gui32.exe`'s DESIGNED output (`PatBlt(dc, 8,8, 160,120)` magenta on a direct
  display DC) — never a doom artifact.
- **Dead lore killed #2**: the copy-back family is NOT the culprit — value
  traces show NCCALCSIZE returns the correct client (44,63,236,186), the
  WINDOWPOSCHANGING copy-backs carry NOMOVE|NOSIZE don't-cares, and
  WINDOWPOSCHANGED confirms correct final geometry for both win32 (40,40
  200×150) and doom ((0,0) 640×400). The M9 snapshot is also fresh-per-call
  (`normalize_wnd` re-reads the kernel WND every access) — no reuse poisoning.
- The "second doom window" (semi-random tall geometry, garbage-looking NCCALC
  rects) is user32's hidden **Default IME window** — cosmetic, ignore.

**Working hypothesis (one mechanism, all three faces): recycled DCE with stale
`prgnVis`.** doomgeneric caches `GetDC(hwnd)` *before* `ShowWindow`. win32.exe
(and gui32/file32/marshal32) run and exit first; their window destruction frees
DCE entries. If doom's `GetDC` reuses a freed DCE whose vis region is never
recomputed for doom's window, the blit clip stays stuck at the *previous
owner's* region: a fresh/reset DCE → full render (historic "rendered" boots),
win32's DCE → the 192×123 crop observed tonight, a degenerate/empty one → no
pixels at all (historic "magenta", now fully invisible since M15a's NCCALC
fix changed the erase path). This is a **native win32k windc.rs bug** (matches
the old note that native arm64 A/B doom misrenders identically) — likely a
dropped DCX_DCEDIRTY/vis-invalidate side-effect in the windc.c → windc.rs
translation (§9.53 dropped-side-effect class). A clip-bounds probe
(`cplx`/`rclBounds` in M13-BLT v2) is the confirm; then diff windc.rs's DCE
reuse path against windc.c.

**CLOSED 2026-07-15 — ROOT CAUSE FOUND + FIXED (commit 6213b784ad7; field
manual §9.86).** The clip probe confirmed the stale-clip mechanism but moved
the blame one stage later: the DCE/vis/Rao pipeline was updating CORRECTLY
(as the parked probes had said) — what never refreshed was **`dc.co`, the
clip object blits actually consume**. `DC_vPrepareDCsForBlit` (dclife.rs)
translated `fs & DC_DIRTY_RAO` as `fs & 0x04` — which is `DC_CANCELED`;
`DC_DIRTY_RAO` is `0x10` — so `CLIPPING_UpdateGCRegion` (the only co rebuild)
never ran on the blit path, and a cached DC kept the recycled DCE's previous
region forever. One wrong nibble = the entire historic intermittency (render
depended on whether an unrelated correctly-checking path touched the DC
first). Post-fix boot: clip snaps to doom's real client `(4,23,636,396)` at
frame 0, dest-strip checksums flip to the frame content, and the screendump
shows **full DOOM gameplay** on the arm64 desktop. All six guests 0x600D on
the same media. The M13-BLT/clip probe (dibobj.rs) is temporary and parked
uncommitted; determinism soak + M14 input drive are the follow-ups.

## M15a: callback OUT-payload copy-back (2026-07-13) — GETMINMAXINFO + NCCALCSIZE landed

Window messages that pass their `lParam` struct as an OUT parameter (the window
proc writes into it and win32k reads it back — `co_IntCallWindowProc`,
callback.rs, copies `lParamBufferSize` bytes from the returned buffer into the
64-bit `lParam`) had their guest edits **silently dropped** since M8: the WoW64
callback-return path (`Wow64pCompleteUserCallback`, wow64_callback.c) restored the
kernel's ORIGINAL 64-bit payload and only copied back the proc's return value.

Enabled the copy-back incrementally, **oracle-gated on win32.exe staying 0x600D**:
- **WM_GETMINMAXINFO** — MINMAXINFO = 5 POINTs = 40 bytes, identical 32/64 layout,
  no pointers → verbatim copy-back. Proven clean.
- **WM_NCCALCSIZE** — `rgrc[]` rects (48b for wParam=TRUE, 16b for FALSE), all
  width-identical RECTs → copy back the rects, PRESERVE the 64-bit `lppos` pointer
  from the baseline restore. Proven clean.

**WM_WINDOWPOSCHANGING is deliberately NOT copied back — and it's a GUEST bug, not
a marshalling one.** A 2026-07-13 down-vs-up trace settled the historical
regression (the 2026-07-12 attempt that took win32.exe 0x600D → 0x600E with a
size-renegotiation loop). Ground truth:
- **DOWN** (win32k → guest): a mix of real-geometry frames
  `(cx=640 cy=400, fl=0x1803)` and no-op frames `(cx=0 cy=0, fl=0x3 =
  SWP_NOSIZE|SWP_NOMOVE)`.
- **UP** (guest → win32k, with clobber on): the guest's 32-bit DefWindowProc
  returns the no-op frame with **SWP_NOSIZE CLEARED** (`fl=0x6c`) while leaving
  `cx=cy=0`. Copying that back tells win32k "size to 0×0 AND do size it" → the
  window collapses to zero client area → WM_PAINT never fires.

Field translation is verified correct (handles preserved from baseline, geometry
per-field). The defect is the guest DefWindowProc returning divergent flags — most
likely because its 32-bit view of the window's `rcWindow` is stale/zero (the M9
shared-struct layer). **Owed WINDOWPOS continuation:** confirm the guest's
`rcWindow`/`rcClient` shadow under WoW64 and fix the source so DefWindowProc stops
clearing SWP_NOSIZE; then the field-correct copy-back can be re-enabled. This is
also plausibly connected to the M13 doom offset-content / `rcWindow==rcClient`
anomaly.

## M16 (owed, newly scoped 2026-07-13): the 32-bit desktop-heap WND view

The WINDOWPOSCHANGING finding above generalises to a real, named WoW64 gap that
several parked items share a root with.

**The problem.** win32k (64-bit) owns the desktop heap; every `tagWND` in it is in
**64-bit layout** (pointer fields — `spwndNext`, `spwndParent`, `spwndChild`,
`ppcls`, `pcls`, … — are 8 bytes each). The guest's 32-bit `user32.dll` maps that
same desktop heap and dereferences `tagWND` at **32-bit offsets**. Any field past
the first pointer is therefore read at the WRONG offset. `rcWindow`/`rcClient` (the
RECTs `DefWindowProc` reads to answer WM_WINDOWPOSCHANGING / WM_NCCALCSIZE) sit well
past several pointers, so the guest reads garbage/zero.

**Why paint still works.** Everything that goes through a **syscall** is marshalled
correctly (M6–M10): GetDC/BeginPaint/GetClientRect/etc. thunk into win32k and the
32/64 translation happens in the thunk. Only code that reads the shared desktop-heap
`tagWND` **directly** in user-mode hits the layout mismatch — and that is exactly
the `DefWindowProc` geometry helpers. Hence: window creation, the message pump, and
WM_PAINT are green, but DefWindowProc's WINDOWPOS/NCCALCSIZE geometry is wrong.

**Suspected blast radius (all consistent with this root):**
- The M13 DOOM "offset content + magenta band" and `rcWindow == rcClient`
  (no non-client inset) anomalies — DOOM's client-origin/geometry likely computed
  off the same mis-read `tagWND` (e.g. `GetWindowRect`, window.c:1282, reads
  `Wnd->rcWindow` DIRECTLY from the heap).

**UPDATE 2026-07-13 — the WINDOWPOSCHANGING half is FIXED and landed** (M16 Fix A +
Fix B, commits `b8263e6a22d` + `3cf1faee89b`). It was the tractable mechanism below,
confirmed by an in→out trace: pre-fix win32k returned garbage (cx=cy=0, SWP_NOSIZE
cleared); post-fix it returns the WINDOWPOS correctly (SWP_NOSIZE preserved,
cx=640→640). The callback copy-back (M15a) is re-enabled on top. win32.exe stays
0x600D. **What REMAINS of M16 is only the direct desktop-heap `tagWND` reads**
(`GetWindowRect` window.c:1282 reads `Wnd->rcWindow` straight from the heap) — that
still needs one of the three fixes below, and is the likely DOOM geometry lead.

**The WINDOWPOSCHANGING mechanism (now fixed), for the record (defwnd.c:783):** The 32-bit user32
DefWindowProc does NOT read the heap for WM_WINDOWPOSCHANGING — it forwards to
`NtUserMessageCall(hWnd, Msg, wParam, lParam, &lResult, FNID_DEFWINDOWPROC, ...)`,
a **syscall**. `lParam` is the guest's 28-byte 32-bit `WINDOWPOS`. That syscall is
NOT manually thunked in wow64 (only the `SHAREDINFO` struct is — wow64_manual.c),
so it goes through the flat generated thunk: the pointer is zero-extended but the
**payload is not marshalled**, and 64-bit win32k's `IntDefWindowProc` reads the
28-byte guest struct as a 40-byte `WINDOWPOS` → every field past the two leading
handles is misaligned → garbage `cx/cy/flags` written back. This explains the
`SWP_NOSIZE`-cleared / `cx=cy=0` return exactly. **Fix: a manual WoW64 thunk for
`NtUserMessageCall` that marshals the `lParam` payload 32→64 (in) and 64→32 (out)
keyed on (FNID, Msg)** — the syscall-direction mirror of the callback marshaller in
wow64_callback.c. Confirm first with a win32k-side trace of the `WINDOWPOS`
`IntDefWindowProc` receives from a WoW64 guest (expect garbage today). This is
smaller than the full desktop-heap-WND-view and is the right next step for
WINDOWPOS specifically; the direct-heap-read reframe above still stands for
`GetWindowRect`-class functions and the DOOM geometry.

**Fix options (real Windows solves this; pick one):**
1. **Client-side thunked WND** — win32k publishes a per-process 32-bit-layout shadow
   of each `tagWND` the client can read (what genuine WoW64 `gSharedInfo` +
   `SERVERINFO`/`DISPLAYINFO` do); the 32-bit user32 reads the shadow. Most correct,
   most work.
2. **Force the reads through syscalls** — patch the 32-bit user32's DefWindowProc
   geometry helpers to call `NtUserGetWindowRect`-class syscalls instead of touching
   the heap. Smaller, but a user32 fork.
3. **Layout-compatible `tagWND` prefix** — reorder `tagWND` so the client-read
   fields (state flags, rcWindow, rcClient) share identical offsets in 32 and 64
   bit. Fragile; touches win32k core.

Verification oracle already exists: win32.exe's WINDOWPOSCHANGING round-trip
(re-enable the copy-back and require 0x600D) plus DOOM's window geometry. This is
the next substantial WoW64/USER milestone.

**CLOSED 2026-07-14 — the "remaining half" was already fixed; the scoping was done
against dead C source.** The direct-heap-read framing above cites
`window.c:1282` — but that file is **inert cohab source**; the live i386
`GetWindowRect` is `user32-rust/src/windows/window.rs:1367`, whose
`ValidateHwnd` → `ValidateHandle` routes TYPE_WINDOW through
`wow64::desktop_wnd_to_user` → the **M9 normalizing snapshot**, which already
fills `rcWindow`/`rcClient` (wow64.rs:431). The truncated `spwnd*` snapshot
fields are also sound: `(kernelVA − ulClientDelta) mod 2^32` lands on the
correct <4GB client mapping, so chain walks translate correctly. Proof: the
win32.exe **geometry rung** (0xBADB/0xBADC/0xBADD) asserts
GetWindowRect == requested 40,40 200×150, client strictly inset (the
rcWindow==rcClient doom detector), and ClientToScreen consistency — green on
first boot: `wnd 40,40-240,190 client 192x123 origin 44,63` (classic metrics
exact: 4px borders + 19px caption). The M13 doom `rcWindow == rcClient`
observation predates the M15a/M16-A landings and is superseded by this rung;
doom's remaining render issue is the M13 DIB→surface pixel-copy lane, not
geometry. Options 1–3 above are moot; option-1-lite (M9 snapshots) had
already won.

## M17 — marshalgen: bulk marshaller GENERATION (2026-07-14)

The manual-marshaller era is over as a scaling strategy. `tools/wow64gen/marshalgen.py`
(wow64gen stage 2) mechanizes the whole pipeline that M8–M16 did by hand:

1. **Layouts**: `clang -Xclang -fdump-record-layouts-complete` through the real
   header stack at BOTH `-target i686-w64-mingw32` and `aarch64-w64-mingw32`
   (flags stolen from each build's compile_commands.json). Authoritative
   per-field offsets from the same compiler that builds the DLLs — no
   hand-maintained struct tables, no drift.
2. **Prototypes**: Nt* param lists + SAL direction extracted textually from
   ntuser.h / ntgdi.h / ntgdibad.h (pre-preprocessing — mingw no_sal strips SAL).
3. **Classify** (`report` subcommand): per-syscall verdict —
   FLAT-OK (flat thunk was already correct) / WIDENING (auto-generatable
   field-copy marshaller) / SEMANTIC (honest manual residue) / SCALAR_WIDEN.
4. **Generate** (`gen` subcommand): `dll/win32/wow64/wow64_automarshal.c`
   (checked in) — flat thunks + `WowAmIn_/WowAmOut_<tag>` field-copy pairs,
   dispatched from `Wow64ManualDispatch`'s UNIMPL tail (hand-written always wins).

**Win32k residue results** (93 NEEDS_MANUAL stubs):
- 45 were FLAT-OK all along (no marshalling needed — the .spec just lacked type info)
- 32 WIDENING (auto-generatable); 15 SEMANTIC (12 D3DKMT unions + LARGE_STRING/
  USERCONNECT already hand-done); 1 no-proto (DxEng)
- **Validation**: the WIDENING set independently rediscovers the exact set of
  hand-written manual marshallers from M8–M10 (MSG, PAINTSTRUCT, UNICODE_STRING,
  CURSORDATA, WNDCLASSEXW, CLSMENUNAME). The classifier reproduces months of
  hand-derived knowledge from first principles.

**Landed coverage**: +18 win32u.spec exports (stubs always existed in sys-stubs.S —
only export lines were missing; arg counts from w32ksvc32.h for i386 @N decoration)
→ 17 new stage-1 flat thunks; +1 generated deep marshaller (NtUserChangeDisplaySettings,
UNICODE_STRING). `tools/wow64gen/automarshal-win32k.txt` force-stubs spec-flat-typed
syscalls whose payloads diverge (wow64gen --manual-list is now comma-separated) —
this is the guard against the silent-wrong-flat-thunk trap the spec additions opened.

**Trap for posterity**: typing a divergent-payload syscall as all-`long` in
win32u.spec makes stage 1 emit a silently-WRONG flat thunk (it can't see inside
`ptr`). Any spec export added for automarshal MUST be checked against
`marshalgen report` and force-listed if not FLAT-OK.

**Next targets**: the ntdll surface (64 NEEDS_MANUAL in wow64_thunks.gen.c) — same
machinery, NTDLL_TU already sketched; then reverse-direction callback payload
generation (mechanize M15a); then arrays/count-aware shapes (EnumDisplayMonitors).

## M18 — scaling marshalgen: callbacks + last ntdll gaps + arrays (2026-07-14)

Three fronts, all landed + oracle-green:

**1. gen-callback (`wow64_cb_automarshal.c/h`)** — the KeUserModeCallback
direction is now generated from the same layout DB. Direction semantics differ
from syscalls and are encoded type-driven: Down (64→32) converts everything;
Up (32→64) writes into the kernel's ORIGINAL packed buffer, so dereferenceable
pointer fields (lppos) are PRESERVED while handles/uintptr value fields
(hmenuNext, itemData) sign-extend back. Fixed silent verbatim-copy corruption
for: WM_NCCALCSIZE wParam≠0 (NCCALCSIZE_PARAMS+appended WINDOWPOS, 96→80 —
the guest's lppos fixup used to land mid-lppos64), WM_COPYDATA (24→12 header +
blob), WM_DRAWITEM / WM_MEASUREITEM / WM_HELP / WM_NEXTMENU (table-driven both
directions). Authoritative message set = win32k g_MsgMemory. Hand-written
CREATESTRUCT/WINDOWPOS paths keep precedence.

**2. ntdll audit + last app-relevant manuals** — 58/64 ntdll NEEDS_MANUAL
stubs already had M10-era hand dispatch. Of the 6 gaps, wrote the 2 real apps
hit: NtQueryInformationThread (TBI 48→28 with TEB32 via WowTebOffset;
identical-class whitelist; unknown classes fail loud) and NtRemoveIoCompletion
(IOCP dequeue; out-slots + IOSB via Wow64pCopyIosbDown). Remaining 4
(scatter/gather/quota) stay loud stubs — no real caller.

**3. Count-aware array** — NtUserEnumDisplayMonitors HMONITOR out-array
marshalled via local ULONGLONG[16] + narrow copy-back.

**Startup-death flake family — ROOT-CAUSED + FIXED 2026-07-14 (field manual
§9.85).** Face A (5 corpus hits, the 0xC000001D deaths at deterministic
PC): SCTLR_EL1.UCI was never set, so FEX's rare inline `dc cvau` flush
(Arm64JITCore::ExitFunctionLink back-patch) trapped as EC 0x18 →
STATUS_ILLEGAL_INSTRUCTION. Fixed in kiinit (UCI|UCT|DZE, per-CPU) +
deterministic fexprobe tripwire (`mrs CTR_EL0`/`dc cvau`/`dc zva` at EL0
every boot). Face B (1 hit, wow64probe 0xC0000005 mid-ProcessInit):
pre-ThreadInit exception amplified into recursion-until-stack-overflow by
the NULL THREAD_STATE deref in BTCpuResetToConsistentStateImpl — vendored
guard landed (passes pre-thread exceptions to the next handler); the seed
AV (NULL node in the invalidation Threads walk, n=1) stays open but will
now self-report truthfully if it recurs.

Commits: 0c4cf1de7a5 (gen-callback), b6a22290788 (ntdll manuals).

### M18 addendum — marshal32.exe, the self-verifying oracle (2026-07-14)

M17/M18 landed marshallers that **no guest exercised**. `tools/fex/hello-compute/Marshal32.c`
closes that hole: an i386 guest that checks each marshalled value **against itself**
(its own TEB32, its own posted sentinels), so a wrong marshal fails deterministically
rather than "looking fine". Launched by winlogon (`TryStartWow64Marshal`), staged by
mkramdisk; verdict `[marshal32] …` + exit 0x600D.

It found two real bugs on its first run — both invisible to every other guest:

1. **NtCreateIoCompletion / NtOpenIoCompletion were flat-thunked.** The .spec types the
   PHANDLE out-slot as a bare `ptr`, so the flat thunk let the kernel's **8-byte** handle
   write smash the guest's **4-byte** slot *and its neighbor* — `CreateIoCompletionPort`
   returned NULL. This is precisely the silent-wrong-flat-thunk trap M17 documented,
   caught live. Fixed: manual marshallers (NtCreateKeyedEvent shape) + force-listed in
   `manual.txt`.

2. **`ntuser.h` keeps a DEAD prototype in a block comment.** The MSDN-shaped 4-arg
   `NtUserEnumDisplayMonitors` sits commented directly above the REAL 5-arg ROS one
   (extra OUT `monitorRectList`). A marshaller written from the dead declaration reads
   every subsequent arg at the wrong guest-stack slot. Fixed the marshaller (+ fail-loud
   on >16 monitors instead of silent truncation) and hardened marshalgen:
   `strip_comments()` now runs before prototype extraction, so **no commented-out
   declaration can ever reach the classifier**. (Only last-definition-wins dict ordering
   had hidden this in the generator; the hand-written path had no such luck.)

**Durable lesson:** a marshaller without a guest that can *disprove* it is an untested
assertion. Every future marshalgen batch should land with a marshal32 check that verifies
the value round-trips, not merely that the call returns success.

Oracle arm64-20260714-183400: all SIX guests 0x600d. Commit 44e6ab10649.

**Phase E14 — the ladder's next bug was NOT where the stack said, and the
`NtCreateThread` ceiling drew first blood (2026-07-25).** E13 let the ladder run
far enough to reach a `BUGCHECK 0x1E` whose stack read
`ProbeForWrite → ExRaiseAccessViolation → RtlRaiseStatus`, attributed to doom and
devmgmt (task #67).

*The attribution was wrong on every axis, and one diagnostic settled it.* The
ACCESS_VIOLATION arm of `ProbeFor*` printed **nothing** — the misalignment arm had
a trace and its caller-LR helper is aarch64-only — so a 0x1E named the probe and
nothing about what was probed. Adding `probe_av_trace` (reporting `WRAP` vs
`ABOVE-USERPROBE` apart, because they point at opposite bugs) produced:

```
[PROBE-AV-W] ABOVE-USERPROBE Addr=fffffffffffffb00 Len=1280 (0x500) Align=16
```

`Addr` is exactly `-1280 = -sizeof(UAPC_FRAME)`, i.e. `Rsp == 0` through
`(Rsp - sizeof) & !15`. **Size and alignment identified the struct; the arithmetic
identified the caller** — `KiInitializeUserApc`, not anything on the reported
stack. It also *refuted* the leading hypothesis (E12: a flat thunk passing a
wrong-width `Length`) in a single boot, which is exactly why the instrument went
in before any fix.

*Two bugs, stacked.* (1) `KiInitializeUserApc`'s C `_SEH2_TRY`/`_SEH2_EXCEPT` had
been translated to a **bare block with the handler body dropped** — the
`exception_occurred` flag was declared and tested but never assigned, so recovery
was dead code and the raise escaped (§9.133; `ke/i386/usercall.rs` already carried
the correct conversion, so amd64 was simply missed). (2) The reason `Rsp` was 0 at
all: **`osk.exe` called `NtCreateThread` through the flat thunk, which returned
`STATUS_SUCCESS` while the kernel built a thread with a zeroed trap frame**
(§9.134) — the only `NtCreateThread` in the whole boot, ~400 log lines earlier,
same process.

*So the culprit was a third applet.* doom and devmgmt were downstream victims of
the box-kill; they merely outlived the process that killed it.

*Result* (same boot command, before → after): BUGCHECK `1 → 0`, PROBE-AV raises
`1 → 0`, doom frames rendered `died → 2022`.

*Disposition on the thread gap:* `Wow64Manual_NtCreateThread` now **refuses**
loudly with `STATUS_NOT_IMPLEMENTED`. Marshalling its five divergent structs would
be *worse than nothing* — a more convincing corpse, still missing the CPU-context
setup only a per-thread WoW64 entry provides. Task #64 carries the scope, read off
the existing process bootstrap: guest + APC stacks, TEB32 `NtTib` StackBase/Limit
(which `MmCreateTeb` deliberately leaves 0 for a WoW64 TEB — "set by wow64.dll at
thread init", a per-thread entry the design already expects and we never built),
a `WOW64_CONTEXT`, and the per-thread wow64cpu/FEX state.

*Bonus find, filed not fixed (§9.135, task #72):* all **1962** Rust-kernel
`debug_assert!` are compiled out — `[profile.release]` never sets
`debug-assertions` — while the C `ASSERT` they came from is live under `-DDBG=1`.
`usercall.rs:96` asserts `(SegCs & MODE_MASK) != 0`; the boot printed `SegCs=0x0`.
That assertion, live, would have caught this at its source.

**Phase E15 — 32-bit guests can create THREADS (2026-07-25, task #64).** E14 left
`NtCreateThread` as a loud `STATUS_NOT_IMPLEMENTED` refusal, which is the right
place to stop but not a place to stay: essentially every real Win32 binary
creates a thread, so "runs single-threaded toys" was the honest description of
the port. This closes it.

*The kernel needed no changes at all, which was worth checking first.*
`MmCreateTeb` keys the combined TEB64+TEB32 low region off
`EPROCESS.Wow64Process` — per **thread**, not per first-thread — and amd64's
`KiSwapContextInternal` re-bases the `KGDT64_R3_CMTEB` descriptor from the
incoming thread's TEB on every context switch. Both were already right. The
entire gap was in wow64.dll, which had no per-thread entry: `Wow64LdrpInitialize`
was process-init and first-thread-init fused into one function, and it kept the
genuinely per-thread pieces — the CPU area holding the guest register file, the
run `CONTEXT` — in function-local `static`s. That is exactly one thread's worth
of storage; a second guest thread would have run on the first one's registers.
`MmCreateTeb` even carries the comment "Guest stack bounds are set by wow64.dll
at thread init", i.e. the design had been waiting for this entry all along.

*The marshaller does not marshal the CONTEXT — it reads it.* The 64-bit kernel
cannot build a 32-bit thread, so widening the guest's five structs would have
produced a more convincing corpse. What it *can* build is a native thread, so
`Wow64Manual_NtCreateThread` reads the guest's `CONTEXT32` as the description it
is (i386 `BaseInitializeContext`: `Eip` = kernel32's `BaseThreadStartupThunk`,
`Eax` = the thread routine, `Ebx` = its parameter, `Esp` = the guest stack the
guest already allocated in `BaseCreateStack`), takes the guest stack bounds from
`INITIAL_TEB32`, and calls `RtlCreateUserThread` with a native start routine
that establishes the guest side. Two fields must **not** be taken from the guest:
`BaseInitializeContext` fills in the native i386 GDT selectors (CS `0x18`, DS/ES/SS
`0x20`, FS `0x38`) because on a real i386 that is what they are, while compat mode
needs `0x23`/`0x2B`/`0x53`; and it asks for `EFlags` IOPL 3, which user mode cannot
have. New threads then enter through the guest's **own** `LdrInitializeThunk` (the
same APC-frame shape the initial thread uses, built on the guest's own stack), so
`LdrpInitializeThread` runs and the thread gets TLS slots and `DLL_THREAD_ATTACH`.
Cross-process creation stays refused — it needs the *target's* wow64.dll.

*Four PRE-EXISTING bugs stood between here and a working thread, each sufficient
on its own.* None were in the new code; all four are the kind that can only exist
while a capability is missing:

1. **`RtlpThrImageStackReserve` read a PE32 image at native width.** A WoW64
   process's image is 32-bit, so `SizeOfStackReserve`/`SizeOfStackCommit` are
   adjacent 4-byte fields where the 64-bit layout has one 8-byte field: the read
   returned `(commit << 32) | reserve` = **16 TB**, and `ZwAllocateVirtualMemory`
   rejected it with `STATUS_INVALID_PARAMETER_4`. Every `RtlCreateUserThread` in
   every WoW64 process failed. Fixed by dispatching on `OptionalHeader.Magic`.
   (The `.c` is cohab-dead — the live copy is `ntdll-rust/src/rtl/libsupp.rs`.)
2. **`LdrpProcessInitialized` never left 1.** `LdrpInitializeProcess` does not
   return in a WoW64 process, so its caller's `_InterlockedIncrement` never ran,
   and every later thread spun forever in `LdrpInitialize`'s "another thread is
   still initializing" wait. The symptom was a thread that created and resumed
   cleanly and then printed **nothing**. Fixed by publishing completion before
   the handoff.
3. **`CreateRemoteThread`'s csrss notify.** A WoW64 guest runs with no CSRSS
   connection at all (`WOW64: Continuing without CSRSS connection` — csrss
   refuses the 32-bit `CSR_API_CONNECTINFO`), so the notify handed
   `NtRequestWaitReplyPort` a port that was never connected, the kernel returned
   `STATUS_INVALID_PARAMETER`, and the cleanup path tore down an otherwise
   perfectly built thread → `ERROR_INVALID_PARAMETER` from every `CreateThread`.
   Now tolerated for exactly the case DllMain already documents, and only that
   case; the real fix remains the deferred CSR-LPC-32 layer.
4. **`NtWaitForMultipleObjects` was a flat thunk.** Its wait ARRAY is 4-byte
   handles guest-side against the host's 8-byte ones → `STATUS_INVALID_HANDLE`.
   It had been sitting in the audit baseline as a *cold* `SCALAR_WIDEN` entry and
   went live the moment threads existed. Baseline 279 → 278.

*Also:* `NtTerminateThread` is now intercepted so a self-terminating guest thread
releases its CPU area — the guest's exit path kills the native thread outright,
so there is no native `DLL_THREAD_DETACH` to hook and the area would leak per
thread for the life of the process.

*Verification — `thread32.exe`, staged on BOTH arches.* Nine rungs, each chosen
to fail loudly for one specific missing piece rather than all riding one code
path: create/join, the thread parameter (`CONTEXT32.Ebx`), the exit code, a
distinct TID, **TLS** (proves the guest loader's per-thread init ran, not a
direct jump to the entry), **guest SEH on the new thread** (proves TEB32's
`NtTib` stack bounds were set — i386 `RtlDispatchException` validates every
`fs:[0]` record against them and silently skips out-of-stack ones), and **four
concurrent threads** verified by result and by distinct stacks, yielding between
rounds so every `Sleep(0)` is a full 32→64→32 mode switch through that thread's
own CPU area. **amd64: `thread32.exe exit = 0x600d`, all nine rungs, and all 8
probe guests still green.**

**arm64 was PARTIAL at E15 and is now GREEN — see Phase E16 below.** At E15 rungs
1-6 passed but rung 8 (*four concurrent* guest threads) died `0xC0000005`, logged
as task #73. Rung 7 is skipped via an explicit `noseh` argument — the
hardware-fault redirect into guest SEH is still owed on arm64, pre-existing and
unrelated to threading, the same reason `fault32.exe` is amd64-only.

E15's recorded diagnosis of #73 — "FEX's per-thread bring-up is not
concurrency-safe", based on two repeated fault PCs inside libwow64fex — was
**wrong**, and wrong in an instructive way: those PCs were reported by an
exception handler that had itself faulted, so they described the handler's own
crash, not the original fault. E16 has the real story. FEX was never at fault.

*This also closed the arm64 coverage debt E-phase had been carrying:* the amd64
livecd stages probe guests from `dll/win32/wow64cpu/CMakeLists.txt`, which is
entirely `if(ARCH STREQUAL "amd64")`, while arm64 stages from
`qemu/mkramdisk.sh`. The per-thread entry is arch-neutral code whose only other
exercise is amd64's `wow64cpu`, so staging it on arm64 is the point, not a bonus.

*Durable lessons (field manual, WoW64 section):* "In a WoW64 process the IMAGE is
a PE32 — native code reading its headers must dispatch on `Magic`" and "An init
that never returns leaves its caller's completion flag unset — and only thread #2
notices". Plus an addendum to the live-set triage rule: **"cold" is a property of
today's capabilities, not of the syscall** — landing a capability re-sorts the
baseline under you, and the newly-live entries are precisely the ones that
capability depends on.

*Still owed after E15:* guest **processes** (`CreateProcess` from a guest) and
cross-process `CreateRemoteThread`, both of which need the CSR-LPC-32 layer;
`NtCreateThreadEx` (nothing in our i386 kernel32 calls it yet — `CreateThread`
goes straight to `NtCreateThread`).

---

**Phase E16 — arm64 guest threads GREEN; the whole arm64 probe ladder is green
(2026-07-26, task #73 CLOSED).** `thread32.exe exit = 0x600d` on arm64: all nine
rungs, rung 7 skipped as designed. The full arm64 WoW64 ladder is now **7/7** —
hello32 `0x600d`, gui32 `0x600e`, win32/file32/async32/marshal32/thread32
`0x600d` — i.e. probe-ladder parity with amd64 (fault32 stays amd64-only,
pending the arm64 guest-SEH redirect).

Three independent root causes, none of them FEX, each one masking the next:

1. **`MiInsertSharedUserPageVad` was `#[cfg(target_arch = "x86_64")]`.** It
   reserves `MM_SHARED_USER_DATA_VA` (0x7FFE0000) with a real VAD so the
   allocator knows the page is taken. amd64 had it, arm64 did not — which is
   exactly why arch-neutral threading code passed 9/9 on amd64 and died on
   arm64. WoW64 PEB32/TEB regions are capped at the 32-bit
   `MM_HIGHEST_USER_ADDRESS` and descend 0x3000 apart, so the **5th** thread's
   TEB landed on the shared page. Its `NtTib.StackBase`/`StackLimit` read back
   as `InterruptTime`/`SystemTime` — the clock, not corrupt bounds — and
   `MmCreateTeb`'s zeroing wiped KUSER_SHARED_DATA for the *whole process*,
   killing threads whose own TEBs were perfect. Fixed by extending the `cfg`,
   plus a tripwire in `MiCreatePebOrTebCapped` that refuses a PEB/TEB VAD
   straddling the shared page.

   *Rejected first attempt, recorded so nobody retries it:* lowering the WoW64
   cap to `0x7FFEFFFF - 16 * PAGE_SIZE` (the native `MM_HIGHEST_VAD_ADDRESS`
   shape). It moves PEB32/TEB below 0x7FFE0000 and **breaks WoW64 bring-up
   outright** — `Wow64pMapNtdll32` reads `NtSystemRoot` from `0x7FFE0000+0x30`,
   and the run died with `NtOpenFile(\??\\syswow64\ntdll.dll)`. The reservation
   is the fix; the cap is not.

2. **`Wow64pVeh` / `Wow64pPrepareForExceptionHook` dereferenced the CPU area
   unguarded.** `(WOW64_CONTEXT *)(CpuArea + 16)` with a NULL `CpuArea` faults at
   the field offsets themselves — `Eax` is `CpuArea+0xC0` — *inside the exception
   handler*, which re-enters the VEH and buries the original exception. This is
   what produced E15's bogus "fault PCs inside libwow64fex" and sent the first
   diagnosis down the wrong path for a session. Now guarded and logged.

3. **`NtSetInformationThread(ThreadZeroTlsCell)` zeroed the wrong TEB.** It walks
   every thread in the process doing `Teb->TlsSlots[i] = NULL` on the **64-bit**
   TEB. A 32-bit guest's TLS cells live in TEB32, and TEB64's low slots are
   WoW64's own — slot 1 is `WOW64_TLS_CPURESERVED`, the per-thread guest register
   file. So an ordinary guest `TlsFree(1)` wiped wow64.dll's CPU-area pointer for
   *every* thread, and the guest then took a breakpoint with no CPU area (cause
   #2's crash). Now zeroes `TEB32.TlsSlots` (0xE10) / `TlsExpansionSlots` (0xF94)
   in a WoW64 process. Note this needed **no marshalling** — the argument is a
   plain `ULONG` at the same offset in both worlds, so no layout-diff tool would
   ever flag it; what differs is which structure the side effect lands in.

*Durable lessons (field manual, WoW64 section):* "A per-arch `#[cfg]` on an
address-space reservation = a TEB allocated on top of KUSER_SHARED_DATA" and
"`ThreadZeroTlsCell` from a 32-bit guest zeroes the WRONG TEB — and wipes WoW64's
own TLS". The transferable one: **when a bug reproduces on one arch and not
another in arch-neutral code, diff the `cfg`-gated setup the two arches ran, not
the code that crashed.**

*Not this bug after all, still owed on its own merits:* we wire only
`BTCpuNotifyMapViewOfSection` of FEX's memory-notification contract, though it
also exports `BTCpuNotifyMemoryAlloc`/`Free`/`Protect`/`Dirty`. **Closed by E17
below — and it was not a cache hint, it was a missing capability.**

---

**Phase E17 — the memory-notification contract: 32-bit guests can GENERATE CODE
(2026-07-26, task #74).** E16 left this as tidy-up owed "on its own merits".
Reading FEX's decoder first changed the estimate completely: `CheckRangeExecutable`
(`FEXCore/Source/Interface/Core/Frontend.cpp`) **refuses to decode any guest
address outside the executable set the CPU layer has been told about**, and that
set is filled only by these notifications plus a one-time `VirtualQuery` sweep at
`BTCpuProcessInit`. With just the map-view hook wired, images worked and
**everything the guest allocated afterwards was un-executable** — ATL/MFC thunks,
VB6/Delphi trampolines, packers, self-extractors, any JIT. Not slow: impossible.

*The disguise is worth naming.* `wow64.c` already carried a comment instructing
itself to "allocate every raw guest page first; pages created afterward are not
visible until the full memory-notification contract is wired." A workaround that
constrains allocation ORDER is a missing notification wearing a disguise — it
holds exactly until something allocates outside your control, i.e. until the
guest runs.

Wired, with the pairing discipline the contract actually demands (`After` is a
lock/unlock flag inside FEX, not a status report — an unmatched *Before*
deadlocks the guest process): `BTCpuNotifyMemoryAlloc` / `MemoryProtect` /
`MemoryFree` / `MemoryDirty` / `UnmapViewOfSection` / `ReadFile` /
`FlushInstructionCache2` / `ProcessExecuteFlagsChange`, behind `Wow64Notify*`
wrappers in the new `wow64_btcpu.h` so no call site touches a raw pointer. They
resolve optionally and are published only *after* `BTCpuProcessInit` — the layer
builds its address-space model there, so an earlier notification has nothing to
land in. All notifications are gated on the target being the CURRENT process:
the protect path really does reprotect pages, so a `VirtualAllocEx` range
belonging to someone else would mutate our own pages at those addresses.

Five syscalls became manual to carry the side effects — `NtUnmapViewOfSection`,
`NtReadVirtualMemory`, `NtWriteVirtualMemory`, `NtFlushInstructionCache`,
`NtSetInformationProcess`. The Read/Write pair had a **second, independent bug**
the flat thunk was hiding: the trailing `*ReturnSize` is a `ULONG_PTR`, so the
host wrote 8 bytes into the guest's 4-byte cell. `NtSetInformationProcess`
intercepts only `ProcessExecuteFlags`, and forwards it: DEP has two enforcers
here — the kernel owns the host page protections (which on a native compat-mode
host ARE the guest's) and the CPU layer owns whether the emulator will run a page
the host would. The kernel's verdict is the one the guest gets back.

*Verification — `selfmod32.exe`, seven rungs, staged on BOTH arches.* Each
isolates one notification, and most fail **silently wrong** rather than loudly,
which is the whole reason the guest checks returned values instead of merely
surviving: (1) execute code generated into a fresh RWX allocation — the only
rung that dies rather than lying; (2) rewrite it + `FlushInstructionCache`;
(3) write as data into an RW region then `VirtualProtect` to RX — the commonest
real pattern and one the alloc notification structurally cannot cover;
(4) free the page and take the same base back, so the next tenant does not
inherit the previous tenant's translation; (5) `ReadFile` straight into live
code memory, i.e. the kernel writing behind the layer's back; (6)
`WriteProcessMemory` hot-patch, which exercises protect + dirty + flush together
because kernel32 wraps the write in `NtProtectVirtualMemory`; (7) `NtWrite`/
`NtReadVirtualMemory` called **directly**, with a canary immediately after the
count cell.

Rung 7 had to bypass kernel32 to mean anything, and noticing that mattered: the
obvious version calls `WriteProcessMemory` with a canary, but kernel32 hands the
syscall its **own** local `nSize` and copies the result out, so the 8-byte
clobber lands in kernel32's stack frame where no caller-side canary can ever see
it. A rung that cannot fail is worse than no rung.

*amd64 is the CONTROL, not a second copy of the test.* `wow64cpu` exports none of
these notifications — execution is native, there is nothing to invalidate — so
amd64 runs the same guest with **0/8 entry points wired** and every rung must
still pass. arm64 exercises the real contract; the serial line
`memory-notification contract: N/8 entry points wired` states which mode a boot
is in.

*Running it found two KERNEL bugs, both pre-existing, neither in the new code.*
This is the payoff of wiring a contract rather than stubbing it: the
notifications drove code paths nothing had reached before.

1. **FIXED — the PFN `CacheAttribute` was never initialised.** `MiInitializePfn`
   (and the `AndMakePteValid` / `ForOtherProcess` variants) never set
   `u3.e1.CacheAttribute`, faithfully to upstream ReactOS. `MiNonCached` **is
   0**, so a page off the free list reads as *deliberately uncached* when nobody
   ever decided anything — and `MiFlushTbAndCapture` consults it on **every**
   protection change and force-ORs `MM_NOCACHE`. On x86 that is a perf loss,
   which is why it survived for years; on arm64 `MM_NOCACHE` selects MAIR
   AttrIndx 1 = **Device-nGnRnE**, where every unaligned access alignment-faults.
   FEX write-protects RWX pages for SMC detection, un-protects on the write
   fault, and the retried unaligned store then faulted forever with the PC
   pinned — the rung-2 livelock. Now sets `MiCached`, as Windows does.

   *It was not a selfmod32-only bug.* Across the arm64 WoW64 ladder `[ALIGN]`
   went from **6–14 per boot to 0–1**, and the character changed: the faults it
   removed were plain `str` on `PTE=…447` (Device), while the survivor is `ldar`
   on `PTE=…4C3` (Normal) — an ordinary unaligned atomic, which is exactly the
   case FEX's own handler exists to absorb.

2. **FIXED (task #76, 2026-09-19) — `MiDoPoolCopy` reported success without
   writing.** Closed by the validate-first probe conversion in `ARM3/virtual.rs`,
   which was made for a different bug entirely (the dropped `_SEH2_EXCEPT` that
   BUGCHECKed guest `CreateProcess`) and gave this one the observability its
   entry below says it needed. The kernel now answers `STATUS_PARTIAL_COPY` with
   0 written where it used to answer `STATUS_SUCCESS` with 6 — the truth instead
   of the lie.

   *And the fix made `selfmod32` go RED, because rung 7 had encoded the bug.* It
   read `st < 0` as failure, which was right only while the kernel was wrong. The
   rung now accepts a refusal, but verifies WHY — `VirtualQuery` must report the
   page unwritable (measured: `0x20`, `PAGE_EXECUTE_READ`, exactly the SMC
   write-protection FEX installs) — and checks the out-cell canary on both paths,
   since a refused write may not clobber the guest's 4-byte count cell either.
   amd64 has no SMC trap, so the write succeeds there and the original assertion
   still runs. **11/11 on both arches.** Field manual: "A test that ENCODES a bug
   goes red the day the bug is fixed".

   *Original filing, kept because it is why the entry existed:* It
   translated the C's `_SEH2_TRY/_SEH2_EXCEPT` as straight-line code, so
   `Status` is initialised to `STATUS_SUCCESS` and never assigned anywhere in
   the function; both `if Status >= 0 { copy }` guards always pass and the tail
   unconditionally does `*ReturnSize = BufferSize`. Proven by rung 7: a write
   onto a page FEX had trapped (`VirtualQuery` → `PAGE_EXECUTE_READ`) returned
   `0x00000000` with 6 bytes written and the bytes unchanged. **Refusing that
   write is correct** — the emulator's SMC trap is a real page protection and
   the kernel honours it, which is precisely why `WriteProcessMemory` wraps its
   write in `NtProtectVirtualMemory`. Reporting success is not. The real fix
   needs probe/copy faults to be observable in translated Rust
   (`dummy-pseh-verdict.md`), so `selfmod32` prints `rung7 BUG: …` on serial and
   deliberately does **not** gate on it — a WoW64 ladder that goes red for an MM
   bug stops being a WoW64 signal.

*Durable lessons (field manual):* "On a JIT host, the CPU layer's memory
notifications are a CAPABILITY, not a cache hint" and "An uninitialised PFN
`CacheAttribute` turns every `VirtualProtect`ed page into DEVICE memory". Three
transferable ones:

- **A workaround that constrains ORDER is a missing notification in disguise.**
- **A zero-valued enum member is not a default, it is an assertion** —
  `MiNonCached == 0` made "uninitialised" and "the caller chose uncached"
  indistinguishable, so every consumer believed a decision nobody made.
- **A rung that cannot fail is worse than no rung.** Rung 7's obvious form calls
  `WriteProcessMemory` with a canary — but kernel32 hands the syscall its *own*
  local `nSize` and copies the result out, so the 8-byte clobber lands in
  kernel32's frame where no caller-side canary can ever see it. Bypassing
  kernel32 is what made it an oracle.

---

**Phase E18 — arm64 dispatches a guest hardware fault into the guest's own SEH
(2026-07-26).** The last arm64 gap in the ladder, and the reason `fault32.exe`
was amd64-only and `thread32.exe` rung 7 ran with an explicit `noseh` skip.
Both are now closed: **arm64 `fault32.exe exit = 0x600D`**, and thread32 runs
all nine rungs on both arches.

*The work was small because the CPU layer already does the hard part; what was
missing was on our side.* There is no compat-mode trap to catch on arm64 — a
guest fault arrives as a **host** arm64 fault inside FEX's JIT, and
`BTCpuResetToConsistentState` gets first refusal. When it can deal with the
fault (SMC, overcommit, unaligned atomic, suspend) it `NtContinue`s and never
returns. When it cannot, it hands the fault over *in guest terms* and returns:
the guest register file is reconstructed into the CPU area (FEX calls
`BTCpuSetContext` itself), the `EXCEPTION_RECORD` is rewritten to describe the
guest fault, and the host `CONTEXT` is replaced with the one captured before
JIT entry. So `Wow64pRedirectGuestFault` only has to build the i386
`KiUserExceptionDispatcher` frame on the **guest** stack — reusing
`Wow64pBuildGuestExceptionFrame` verbatim from E5 — point the guest context at
the dispatcher, and return `EXCEPTION_CONTINUE_EXECUTION`. That resumes the
host at the pre-JIT-entry context, i.e. back inside `BTCpuSimulate`, which
re-enters the guest from the CPU area — now at the guest's own dispatcher, so
its real `fs:[0]` chain runs.

Two details that decide whether it works at all:

- **Telling a handed-over guest fault from a genuine host one.** FEX always
  rewrites `ExceptionAddress` to the guest `Eip` before it declines, and a
  host-side fault it never touched still carries a host address. So the test is
  `ExceptionAddress < 2GB && ExceptionAddress == CpuArea->Eip` — cheap, and it
  cannot false-positive on a host fault.
- **Breakpoints deliberately keep the pre-E18 behaviour.** That same
  discriminator matches exactly the wrong one: FEX reports a DebugService
  `int 2d` with `ExceptionAddress ==` the (already advanced) guest `Eip`, but a
  genuine `int3` with `Eip - 1`. Redirecting on it would dispatch the guest
  loader's own debug channel into SEH while still missing real breakpoints.
  Guest `int3` dispatch is its own change, not a freebie.

*Also settled:* `EXCEPTION_CONTINUE_EXECUTION` from a VEH **is** honoured on
arm64 — `rtl-except/src/arm64/except.rs` calls
`RtlCallVectoredExceptionHandlers` first and returns on a non-zero result.
Worth stating because the amd64 hook exists precisely *because* that build's
`RtlDispatchException` elides the vectored block, and it would be reasonable to
assume the same here.

*Verification:* the redirect fires exactly once for `fault32` rung 1 —
`guest fault c0000005 eip=00401002 -> 32-bit SEH dispatch` — and the remaining
rungs (RaiseException round-trip, `DBG_PRINTEXCEPTION` consumption,
Get/SetThreadContext) reach the guest by their own paths, unchanged. amd64 is
untouched: the whole redirect is `#ifdef _M_ARM64`, and amd64 keeps catching
the real compat-mode fault in `Wow64pPrepareForExceptionHook`. Running the same
guest on both arches is worth it precisely because the arrival paths share
nothing.

**Phase E19 — the out-cell class: 19 syscalls were writing four bytes past the
guest's variable (2026-07-26).** The first real burn-down of the E10 audit
baseline, and it turned out not to be tidy-up: these were **live guest memory
corruption**.

Each of the 19 takes a guest pointer to a cell the kernel fills with a
POINTER-SIZED value — a `HANDLE`, a base address, a region size. Flat-thunked,
the guest's 4-byte cell went straight to a host kernel that stores 8, so every
*successful* call scribbled over the next four bytes of guest memory: the
adjacent local, or the next field of the struct. `NtOpenProcessToken` was on the
list, which means every `OpenProcessToken()` from a 32-bit guest did it.

*Why it survived this long is the interesting part.* The call succeeds. The
value the guest asked for arrives correctly, in the right place. Only the
collateral write is wrong, so the damage surfaces later and somewhere else —
the same signature as E7's calc heap corruption and the `IO_STATUS_BLOCK`
overflow M10 fixed. Reading the thunk table teaches you nothing, because a flat
thunk *is* the absence of code. The audit gate exists exactly for defects that
have no textual presence, and this is the first time it was mined rather than
merely kept green.

The clincher for "don't fix these from memory": `NtOpenProcessTokenEx` and
`NtOpenThreadTokenEx` were already marshalled while the plain
`NtOpenProcessToken`/`NtOpenThreadToken` sitting next to them in the same table
were not. Someone fixed the variant they were looking at. The list has to come
from the tool.

*Scope, and what was deliberately left.* Fixed: the 19 whose divergence is
purely out-cells (plus, where present, an `OBJECT_ATTRIBUTES` the existing
helper already handles) — the token opens, `NtDuplicateObject`/`Token`, the
job/timer/event-pair/debug-object/port creates, `NtCreateProcess`(`Ex`),
`NtCreateProfile`, and the `Lock`/`Unlock`/`Flush`VirtualMemory in/out trio.
**Not** fixed here, and named rather than quietly skipped: the AWE family
(`NtAllocateUserPhysicalPages` and friends) plus `NtGetWriteWatch`,
`NtCompactKeys` and `NtRaiseHardError`, whose payloads are *arrays* of
pointer-width elements needing a bounded scratch — and whose kernel side
currently returns `STATUS_NOT_IMPLEMENTED`, so there is nothing to corrupt
today and no way to test a marshaller for them. Also left: the five with genuine
`SEMANTIC` divergence (`PORT_MESSAGE`, `TOKEN_GROUPS`), which belong with the
CSR-LPC-32 layer. Baseline 278 → **246**, re-blessed with **+0 added**.

*A latent bug found en route:* `Wow64pMarshalOa` dereferenced the guest's
OBJECT_ATTRIBUTES pointer unconditionally. Several of the new callers take it
`_In_opt_`, and a guest is free to pass NULL where it is mandatory too — that
would have faulted inside wow64.dll. It now returns an empty OA for the
mandatory case, and a new `Wow64pMarshalOaOpt` passes a real NULL through for
the optional one (a pointer to a zeroed OA is not the same thing to the object
manager as no OA at all).

*Verification:* `marshal32.exe` gained an out-cell rung — a canary parked
immediately after each handle cell, with the call required to SUCCEED, because a
failing syscall writes nothing and would pass the check while completely
unmarshalled. The canary is `volatile` for a specific reason: we only hand the
callee `&cell.Handle`, so a conforming compiler may assume the rest of the
struct is untouched and fold the comparison away — deleting the bug from the
test but not from the OS. Green on both arches.

**Also fixed, and it is a lesson about the harness rather than the OS:**
`selfmod32.exe` was failing `0xBAD9` on amd64 — and had been since E17 landed.
Its rung-5 fixture writes a scratch file under `%WINDIR%`, which is a writable
ramdisk on arm64 and a **read-only CD** on the amd64 livecd. E17 verified it on
arm64 and staged it for amd64 without ever watching it there, because **the
ladder runs after the desktop marker and the ordinary smoke stops at that
marker**. An amd64-only failure in a late rung is invisible to the gate; it took
`smoke --arch amd64 --until "selfmod32.exe exit"` to see it. Rung 5 now skips
loudly with the errno on a read-only volume (the file is the rung's *fixture*,
not its subject — file32 already established this precedent for its own write
rungs) and rungs 6-7 still run. arm64 keeps full coverage of rung 5, so nothing
was traded away. Both arches now report the whole ladder green: **9/9 on amd64
and 9/9 on arm64.**

*And the blind spot itself is closed, not just this instance of it.* Hand-typing
`--until "<guest>.exe exit"` on two arches is precisely the "remembering a
recipe" smell AGENTS.md calls a bug report against the tool — and the gap had
already been written down once (the roadmap's `_MmCopyFromCaller` entry notes
"the smoke gate stays GREEN through this: the boot marker fires long before the
ladder runs") and still cost a day. So it is a flag now:

```
./vibeos-cli smoke --arch amd64 --ladder     # arm64 needs VIBEOS_FEX=1 media
```

It keeps draining serial after the desktop marker until all nine guests report,
and **fails the boot** on a wrong or missing verdict. `tests/qemu/cli/ladder.py`
owns the expected-verdict table (gui32 is `0x600E`, the rest `0x600D`; arm64-only
extras like `wow64probe` are reported but not required, so the table stays the
cross-arch *parity* claim rather than "whatever arm64 happens to stage"). A boot
with nothing staged reports `LADDER_NOT_STAGED` **with the build recipe** — a
staging mistake must not read as an OS regression. Seven unit tests pin the
parser, including the selfmod32 `0xBAD9` case and the one that matters most:
**a guest that never reports is a FAILURE, not a pass** — treating absence as
success is the exact failure mode the check exists to prevent. The extra watch
is bounded and strictly additive: with the flag off, the boot path is unchanged,
so no other lane pays for it.

**Phase E20 — every `RegSetValueEx` from a 32-bit guest BUGCHECKed the box
(2026-07-26).** The second slice of the E10 audit baseline, and it turned out to
be a **local denial of service**, not a correctness wart.

`UNICODE_STRING` is 8 bytes in the guest — `{USHORT Length; USHORT
MaximumLength; ULONG Buffer}` — and 16 on the host, because the pointer is
8-aligned: `{USHORT Length; USHORT MaximumLength; ULONG pad; ULONGLONG Buffer}`.
`Buffer` therefore lives at offset 4 in the guest and offset 8 on the host, so a
flat thunk makes the kernel read the string pointer **from past the end of the
guest's struct**. Not a truncation — a wholly unrelated address, assembled from
whatever two words happened to follow. `Length` and `MaximumLength` share
offsets, so they arrive intact and make the whole thing look plausible.

`NtSetValueKey` was flat. `NtQueryValueKey`, one line away in the same table,
was already marshalled. **Registry reads worked and registry writes killed the
machine** — the same "fix the variant you happen to be looking at" pattern that
left plain `NtOpenProcessToken` behind its `Ex` sibling in E19, which is the
argument for driving this off the tool instead of off inspection.

*The proof, in the order it was obtained.* The guest rung was written and run
**against unfixed media first**, so the test is known to be able to see the bug:

```
[WOW64] NtCreateKey(0x2b) [manual] => 00000000
[PROBE-AV-R] ABOVE-USERPROBE Addr=0040400000580056 Len=38 (0x26) Align=2
[RAISE] RtlRaiseStatus(0xC0000005)
*** BUGCHECK: 0x0000001E
```

Read the operands: `Len=38` is 19 WCHARs, exactly `"Wow64ValueNameProbe"`, so the
lengths came through correctly and only `Buffer` is wrong —
`0x0040400000580056` is two adjacent 32-bit guest words glued together. The
address is above the user probe boundary, so `ProbeForRead` raises, in kernel
mode, unhandled → `0x1E`. After the fix the same media reports
`[marshal32] registry value-name round-trip verified (set/query/delete)` and the
ladder is 9/9 on both arches.

*Scope.* All 27 ntdll services whose only divergence is a bare
`PUNICODE_STRING` or `OBJECT_ATTRIBUTES`: the registry value/rename calls, the
hive load/unload/replace family, `NtDeleteFile`, `NtDisplayString`, the driver
load/unload pair, the system-environment quartet, `NtCreatePagingFile`, and the
six audit-alarm entry points. `PRIVILEGE_SET` and `GENERIC_MAPPING` pass through
unmarshalled because they are pointer-free (LUIDs and ACCESS_MASKs) — which is
what the audit says, and why they are not on the list. Baseline **246 → 209**,
re-blessed with **+0 added**; the ntdll table is now 172 auto / 124 manual.

*Rung design.* `marshal32` does a full round trip — set a value, read it back
**by the same name**, delete it, confirm it is gone — because a wrong `Buffer`
cannot survive that: either the set faults, or it lands under a garbage name and
the query misses, or the delete removes nothing. Checking only that the set
returned success would have passed on the broken build right up until the
machine died.

**Phase E21 — `NtOpenKey` denied every `KEY_WOW64_32KEY` open, native callers
included (2026-08-01, task #79).** `KEY_WOW64_64KEY` (0x0100) and
`KEY_WOW64_32KEY` (0x0200) are not access bits — no key's DACL grants them — so
the kernel must strip them before the object-manager check. The C does that
with `DesiredAccess &= ~KEY_WOW64_RES` (0x0300) in both `NtCreateKey` and
`NtOpenKey`. `NtOpenKey`'s Rust port had lost the name for a bare `0x0100`, so
it stripped only the 64KEY bit and `RegOpenKeyEx(.., KEY_WOW64_32KEY)` returned
`ACCESS_DENIED` while the identical `RegCreateKeyEx` succeeded.

Two things about this are worth keeping. It is **not WoW64-specific** — a
native 64-bit `reg query /reg:32` hits it too; the lane only found it because
it was surveying the WoW64 registry story. And task #66 had recorded the
premise "the `KEY_WOW64_*` flags are unhandled", which was **wrong**: they were
handled, and one of the two had been mistranslated. A task description is not
evidence. The `marshal32` rung opens one key four ways so a failure names the
offending bit instead of saying "registry broken".

**Phase E22 — the self-describing size: a field-perfect marshaller is still a
broken one (2026-08-01, task #81).** Third slice of the E10 audit baseline
after E19 (out-cells) and E20 (`UNICODE_STRING`), and the first one that is not
about *widths* at all.

A struct that carries its own `sizeof` gets version-checked against the
**host's** `sizeof`. So for this family the marshalling problem is not that the
guest's bytes land in the wrong places — it is that the call never gets that
far:

```
win32k-rust/src/ntmouse.rs     SafeTME.cbSize != size_of::<TRACKMOUSEEVENT>()  -> ERROR_INVALID_PARAMETER
win32k-rust/src/cursoricon.rs  (*pci).cbSize  == size_of::<CURSORINFO>()       -> else ERROR_INVALID_PARAMETER
win32k-rust/src/misc.rs        SafeGui.cbSize != size_of::<GUITHREADINFO>()    -> ERROR_INVALID_PARAMETER
win32k-rust/src/window.rs      (*pcbi).cbSize <  size_of::<COMBOBOXINFO>()     -> ERROR_INVALID_PARAMETER
win32k-rust/src/input.rs       cbSize as usize != size_of::<INPUT>()           -> ERROR_INVALID_PARAMETER
```

| struct | guest | host | the API a 32-bit app loses |
|---|---|---|---|
| `TRACKMOUSEEVENT` | 16 | 24 | `TrackMouseEvent` — no `WM_MOUSELEAVE`/`WM_MOUSEHOVER` at all |
| `CURSORINFO` | 20 | 24 | `GetCursorInfo` |
| `GUITHREADINFO` | 48 | 72 | `GetGUIThreadInfo` |
| `COMBOBOXINFO` | 52 | 64 | `GetComboBoxInfo` |
| `MENUINFO` | 28 | 40 | `GetMenuInfo`/`SetMenuInfo` |
| `INPUT` | 28 | 40 | `SendInput` — all synthesized input |

None of these crash. They return `FALSE` and set `ERROR_INVALID_PARAMETER`, for
every 32-bit process, forever — which is exactly why six applets' worth of
ladder never noticed. The audit found them because the audit does not need a
guest to call something before it will look at it.

*What the generator learned.* `marshalgen`'s `_gen_struct_marshaller` copied
every same-width field verbatim, which is correct for every field except this
one: the guest wrote a number that is only true on the guest. It now emits
`WowAmSizeIn`/`WowAmSizeOut` for a 4-byte leading field with a size-ish name,
**guarded on the value**: rewrite it only when it is exactly the 32-bit
`sizeof`, so a struct with more than one legal size (`MONITORINFO` vs
`MONITORINFOEXW`) still reaches the kernel with the size the guest chose, and
garbage still gets rejected as garbage. The copy-out undoes the rewrite — the
kernel restamps `cbSize` with the host size, and handing that back would leave
the guest holding a number that does not describe its own struct.

The hand-written `NtUserThunkedMenuItemInfo` thunk had `M64.cbSize =
sizeof(M64)` in it from the day it was written. The generator did not, and the
five syscalls that were waiting on the generator got the broken version. That
is E19's `NtOpenProcessToken`-vs-`Ex` and E20's `NtSetValueKey`-vs-
`NtQueryValueKey` a third time: **the sibling that a human looked at is right
and the one the tool produced is wrong**, which is the argument for teaching
the tool rather than fixing the instance.

*`SendInput` is hand-written*, because its size is a syscall **argument**, not
a leading field — no struct diff can associate arg 2 with arg 1. It diverges
twice over anyway: the union is at offset 4 in the guest and 8 on the host, and
each arm's `dwExtraInfo` widens 4→8, so the array stride is 28 vs 40.
`dwExtraInfo` is **zero**-extended, unlike every handle in this layer, because
it is app-defined data the guest reads back through `GetMessageExtraInfo` and
sign-extension would corrupt any value with bit 31 set.

*`NtUserThunkedMenuInfo` is hand-written too, and it is the more instructive
one — the generator was about to introduce a bug.* Its prototype says
`LPCMENUINFO` and the layout diff is a clean 28→40 WIDENING, so `marshalgen`
generated a marshaller for it happily. But win32k casts the pointer to
`PROSMENUINFO` and `UserMenuInfo` reads **`cbSize` bytes** out of the caller,
accepting anything in `[sizeof(MENUINFO), sizeof(ROSMENUINFO)]` — ROSMENUINFO
being MENUINFO plus a dozen internal fields. The real payload is therefore a
**caller-declared variable size**, and a generated thunk hands the kernel a
40-byte stack bounce and then lets it read `cbSize` bytes out of it: a guest
declaring a ROSMENUINFO reads ~36 bytes past the end of wow64.dll's own buffer.
The flat thunk it was replacing at least passed a real, correctly-sized guest
pointer. So: marshal exactly the MENUINFO size class, pass everything else
through untouched. **A prototype is a claim about the declared type, not about
what the callee reads** — check what the kernel does with the pointer before
believing a WIDENING verdict on it. The other four were checked the same way
and are clean: `TrackMouseEvent`, `GetCursorInfo` and `GetGUIThreadInfo` all
read and write a fixed `sizeof`, and `GetComboBoxInfo` writes fields
individually within its 64 bytes.

*Deliberately NOT fixed: `NtUserGetMouseMovePointsEx`.* It is in the same class
(`cbSize` as an argument, `MOUSEMOVEPOINT` 16→24) but the kernel side is a
stub — the `GMMP_USE_DISPLAY_POINTS` branch probes the output buffer and
returns `nBufPoints` without ever filling it. Marshalling it would convert an
honest `ERROR_INVALID_PARAMETER` into a success handing the app an
uninitialized buffer. It stays in the audit baseline, and the kernel-side stub
is filed separately.

*Rung design.* `marshal32`'s new check asserts three things per struct, and
they fail in different directions. The call must **succeed** (the size reached
the kernel host-sized) — that is the one that fires on unmarshalled media. The
size must come back **guest-sized** (the copy-out undid the rewrite). And a
canary parked immediately after the struct must **survive** — which is what
separates "fixed the size check" from "fixed the size check and let the kernel
write 72 bytes onto the guest's 48-byte stack slot". A naive rewrite with no
bounce buffer passes the first and fails the third.

---

**Phase E23 — the kernel's error codes never reached the guest (2026-08-02,
task #82).** Found by E22's *control* run, not by the fix: the unfixed-media
boot printed

```
[marshal32] cbSize: TrackMouseEvent(TME_QUERY) failed, err = 0x00000000
```

on a path where win32k had just executed `EngSetLastError(ERROR_INVALID_PARAMETER)`.
The zero is the whole bug.

`EngSetLastError` writes `NtCurrentTeb()->LastErrorValue`, and in kernel mode
that is the thread's **64-bit** TEB. A 32-bit guest's `GetLastError()` reads
`TEB32->LastErrorValue` — a different structure at `WowTebOffset`. So every
code the entire USER/GDI surface produced landed in a TEB no 32-bit process
ever reads. Apps got the `FALSE` and never the reason: no
`ERROR_CLASS_ALREADY_EXISTS`, no `ERROR_ALREADY_EXISTS`, no
`ERROR_NO_MORE_ITEMS` to end an enumeration loop, no
`ERROR_INVALID_WINDOW_HANDLE`. The ntdll surface was never affected — those
syscalls return an NTSTATUS and the guest's own ntdll32 converts it into its
own TEB32.

*The fix is one rule at four sites.* **Crossing into the kernel copies the
guest's last-error UP; crossing into the guest copies it DOWN.** That makes
TEB32 and TEB64 behave as the single variable a native 64-bit process has,
which is the only definition of "correct" here that does not require deciding
case-by-case whose value wins.

| site | direction | file |
|---|---|---|
| win32k syscall entry | TEB32 → TEB64 | `wow64_win32k.c` `Wow64DispatchWin32kService` |
| win32k syscall exit | TEB64 → TEB32, *if changed* | ″ |
| kernel→guest callback entry | TEB64 → TEB32 | `wow64_callback.c` `Wow64pEnterGuestCallback` |
| callback return (longjmp landing) | TEB32 → TEB64 | ″ `Wow64pUserCallbackCommon` |

*Why the copy-UP, when the bug is only about codes coming down.* win32k's own
save-and-restore idiom — `Error = EngGetLastError(); …; EngSetLastError(Error);`
in `window.rs` and `menu.rs` — exists to leave the **caller's** last-error
untouched. Without the copy-up it faithfully preserves a stale value from some
earlier syscall instead of the caller's, so the idiom does the opposite of what
its author intended for every WoW64 caller.

*Why the callback pair exists at all — it is a hole the syscall pair CREATES.*
Once syscall entry copies up, a guest window proc that makes any win32k
syscall overwrites `TEB64->LastErrorValue` in the middle of the outer call.
That is not hypothetical: `co_IntSendMessageTimeout` sets `ERROR_TIMEOUT`
(`message.rs:2225`) and four `BroadcastSystemMessage` sites read it back with
`EngGetLastError()` *after* the send — and `co_` is precisely the prefix for
functions that leave the critical section and call back into user mode. Mirror
at the callback boundary too and the value the kernel reads back is always the
one the kernel last wrote, exactly as on a native host.

*Why "if changed" on the way out but unconditional on the way in.* With both
boundaries in place an unconditional copy-down is provably a no-op when win32k
did not write — the value going down is the value that came up. The change
test is kept anyway as cheap insurance against a boundary not enumerated here
(guest exception dispatch re-enters guest code without passing through the
callback path), where an unconditional copy-down would clobber a value the
guest set and the kernel never saw. One compare.

*Rung design: two different codes out of ONE api.* `marshal32`'s check drives
`TrackMouseEvent` three ways — a rejected `cbSize` must answer
`ERROR_INVALID_PARAMETER`, a rejected flag bit must answer
`ERROR_INVALID_FLAGS`, and a call that SUCCEEDS must leave the guest's own
sentinel alone. One expected code proves much less than it looks: a bridge that
hardcoded, guessed, or merely zeroed would satisfy it. Two codes out of two
branches of the same kernel function is what makes the claim "the code
travelled". `TrackMouseEvent` is the probe because `user32.spec` forwards it
STRAIGHT to `NtUserTrackMouseEvent` with no client-side validation, so no
asserted code can have come from user mode; and because E22's size rewrite is
value-guarded, a `cbSize` of 1 passes through untouched and genuinely reaches
the kernel's version check. The third assertion is the one that rejects the
naive bridge: an unconditional copy-down with no copy-up hands the guest
whatever an earlier syscall left in the 64-bit TEB, turning a fix for failing
calls into a corruption of succeeding ones.

*The meta-lesson, and it is the reason this was filed separately instead of
being folded into E22.* E22's `SendInput` check originally excused a failure
unless `GetLastError()` was `ERROR_INVALID_PARAMETER` — a discriminator that
**could never fire**, because the code it tested for could not reach the guest.
An assertion that cannot fail is worse than no assertion. Any guest-side test
that branches on a code the kernel sets is dead on arrival until this bridge
exists; prove the code arrives before trusting the branch.

---

**Phase E24 — Wow6432Node registry redirection (2026-08-02, task #66).** The
last structurally-missing WoW64 feature: a 32-bit process shared the 64-bit
`HKLM\Software`, so nothing it wrote was isolated and a COM `InprocServer32` it
read named a 64-bit DLL it could not load.

*Where it went, and why not where the task proposed.* Filesystem redirection
lives in `wow64.dll` (`wow64_manual.c` rewrites `\system32\` → `\syswow64\`),
so the obvious symmetry was to rewrite key paths there too. That does not work:
registry opens are overwhelmingly RELATIVE — advapi32 caches a handle to
`\REGISTRY\MACHINE` and passes `RootDirectory` + `"Software\Foo"` — so a
user-mode rewrite would have to resolve the root handle to a path on every
open, a syscall per open and stateful besides. It went into CM instead, as one
wrapper around `CmpParseKey` (`sdk/lib/rust/config-rust/src/cmwow64.rs`).
`NtOpenKey` and `NtCreateKey` both funnel into `ObOpenObjectByName` →
`CmpParseKey`, and Ob's only parse-procedure bypass (`obname.rs`) is for
*directory* objects, which a key never is — so that wrapper sees every key open
in the system, including callers that reach `NtOpenKey` directly and never
touch advapi32.

*The rule is one line.* When the parse crosses `\REGISTRY\MACHINE\SOFTWARE`
into a child, splice `WOW6432NODE` in. Everything else falls out:

- **The view is sticky per handle, for free.** Redirect only when the SOFTWARE
  boundary lands INSIDE the remaining name. If the caller's root handle is
  already at or below SOFTWARE it was resolved under some view already, so
  "do nothing" is correct — and a 32-bit app that opened SOFTWARE with
  `KEY_WOW64_64KEY` keeps the 64-bit view beneath it with no per-handle state.
- **`\REGISTRY\USER` is untouched**, so HKCU never redirects, which is right.
- **The native boot path cannot be affected.** The gate is
  `EPROCESS.Wow64Process != NULL`; smss, csrss, winlogon and explorer are all
  64-bit. On i386 the whole thing compiles to `false`.

*Two design traps, both hit and both reversed before writing code.* The first:
shared-key exemptions look like they should be registry SYMLINKS — CM already
resolves them and Windows really does keep `Software\Wow6432Node\Classes` as
one. They cannot be. A link target is an absolute path Ob re-parses, and the
re-parse redirects it AGAIN, so a link from `Software\Wow6432Node\Policies` to
`Software\Policies` bounces straight back to itself. Exemptions have to be a
table the redirect consults — which is exactly what Windows' redirection table
is, and why it is one.

The second is the task's own hard rule, *redirection without population is a
regression*. Nothing populates `Wow6432Node` here, so redirecting reads into it
would hide every OS key a 32-bit app needs — `Microsoft\Windows NT\CurrentVersion`
and the shell's `Explorer` keys among them. Windows' answer is to mirror the
subtree from the installer; we have no installer and a 1199-key mirror would
drift. So this takes Wine's answer instead: a redirected open that returns
`OBJECT_NAME_NOT_FOUND` is retried unredirected. Writes get a private node,
reads still find the 64-bit keys, and no population is owed. The fallback is
**reads only** — a create that fell back would have no isolation, which was the
entire point. `HKLM\Software\Wow6432Node` is still created in `hivesft.inf`,
because `NtCreateKey` creates only the LAST path component and a create
redirected into a missing intermediate would fail.

Two deliberate deviations to record, both reversible:

1. **Real Windows HIDES 64-bit-only keys from a 32-bit process; we fall back to
   them.** The precise consequence is that isolation comes from CREATE, not
   from the access mask: a 32-bit process that OPENS an existing 64-bit-only
   key for write falls back and writes the SHARED key, where Windows would have
   failed the open. That is the safe direction — refusing it instead breaks
   every 32-bit app that updates an OS key — but it is a real gap. Revisit if
   we ever ship 32-bit-only components with their own registrations, at which
   point the fallback starts masking a genuine "this component is 64-bit only"
   answer.
2. **`Classes` lands at `Software\Wow6432Node\Classes`, where Windows puts it
   at `Software\Classes\Wow6432Node`.** Windows reconciles the two by making
   the former a symlink to the latter; we cannot, per the re-parse trap above.
   Nothing here reads the Windows location, and with the fallback a 32-bit COM
   lookup resolves to the shared 64-bit registration exactly as it did before
   E24 — so this is invisible today. It becomes real if a 32-bit-only COM
   server is ever registered, and the fix then is a second redirect root for
   `Software\Classes` plus the `CLSID`/`Interface`/`DirectShow`/`Media Type`
   subset Windows actually redirects, rather than a symlink.

*E21's strip had to move.* `KEY_WOW64_32KEY`/`64KEY` are view selectors, not
access bits: no key's ACL grants either, so one left set denies the open, which
is why E21 stripped them in `NtOpenKey`/`NtCreateKey`. That is correct only
while the two views are identical. The moment they differ, `KEY_WOW64_64KEY` is
a WoW64 process's ONLY route back to the 64-bit view — ole32 uses exactly that
for HKCR. So the strip moved down into `CmpParseKey`, where the bits first pick
a view and are then cleared before any access check. Same invariant E21
established, now enforced at the one choke point every key open passes through
instead of at two callers. marshal32's rung 6 is the regression guard on it.

*Testing it: prove the two views DISAGREE.* "The write succeeded" is not
evidence — with no redirection at all a create still succeeds and a read still
finds it. marshal32 rung 9 asserts two independent things: the default-view
handle's own `NtQueryKey(KeyNameInformation)` must say `Wow6432Node` (where it
went, from the kernel's mouth), and the identical path opened with
`KEY_WOW64_64KEY` must NOT see the value just written (the views are genuinely
distinct objects). Faking either one still fails the other. A third check then
pins the property the whole design rests on — a 64-bit-only key must still
resolve from the 32-bit view — which passes before AND after, so it is a guard
on the fallback rather than a detector of the original bug.

*The control run, done the E20/E22/E23 way — rung first, against media that
does NOT have the fix.* Redirection was disabled with a one-line early return
(leaving the flag consume in place, so the media is exactly "before E24"), and
the ladder reported:

```
[WINLOGON] marshal32.exe exit = 0xbafb
[marshal32] redirect: default view is not in Wow6432Node, status = 0x00000000
```

`status = 0x00000000` is the part that makes this a control rather than a
coincidence: `NtQueryKey` SUCCEEDED and handed back a real key name that simply
has no `Wow6432Node` in it. Had the rung failed because the query itself broke,
that field would carry the error and the assertion would prove nothing. The
other eight rungs stayed green, so the failure is isolated to rung 9 rather
than a sick guest.

Fixed media, amd64, ladder **9/9**:

```
[marshal32] Wow6432Node redirection: private 32-bit view, 64-bit view distinct, miss falls back
[WINLOGON] marshal32.exe exit = 0x600d
```

*The first cut of this was INERT, and the story is the most useful part of the
phase.* It built clean, passed every lint, regressed nothing — and redirected
nothing. Ob hands the parse procedure a remaining name in TWO shapes: an
ABSOLUTE-path open arrives with a LEADING separator (parent `\REGISTRY`,
remaining `\Machine\Software\...`), a RELATIVE open without one (parent
`\REGISTRY\MACHINE`, remaining `SYSTEM\...`). Both happen constantly. The code
assumed one separator sat *between* parent and remaining, so every absolute
open compared `\Machine\Softwar` against `MACHINE\SOFTWARE`, failed, and
silently declined. Fix: skip leading separators before comparing and carry the
offset into the splice.

*The offline harness passed 16/16 and was still wrong, which is the lesson.*
The splice is pure string arithmetic, so it was lifted into a host harness and
run against parent-at-`\REGISTRY` vs `\REGISTRY\MACHINE` vs at/below SOFTWARE;
`SOFTWARE` with and without a tail; lowercase; `SOFTWAREX` (must NOT match);
`SOFTWARE\WOW6432NODE\FOO` and `SOFTWARE\Wow6432Node` (idempotence, both
cases); `SOFTWARE\WOW6432NODEX` (a prefix of the node name is a DIFFERENT key
and must still redirect); HKCU; and names stopping short of the boundary. All
16 passed — because all 16 had been INVENTED in the relative shape, the shape
the author had in mind. **A hand-built test inherits the implementer's blind
spot by construction.** What actually found it was a kernel trace printing the
real arguments, and the harness only became trustworthy once its cases were
transcribed from that trace (now 22/22, six of them real). Seed a pure-logic
harness from observed inputs, not imagined ones.

*And the rung did not say what it saw.* It reported "default view is not in
Wow6432Node" while holding the name the kernel had just given it. Answering
"then what IS it?" cost a full build plus a boot for a string already in hand.
marshal32 gained a `ReportWide` helper and now prints it; an assertion should
name what it found, not only what it wanted.

*One non-obvious kernel requirement.* `CmpConstructName` walks the KCB parent
chain and reads hive cells, so it needs `CmpLockRegistry()` + the KCB shared +
a `Delete` check between them — the sequence `CmpQueryKeyName` uses. The
redirect builder runs BEFORE `CmpParseKey`'s own locking
(`CmpBuildHashStackAndLookupCache` is what takes the registry lock), so it
acquires and drops its own. Calling the inner parse twice for the fallback is
safe for the same reason: each invocation is lock-balanced, acquiring in the
cache lookup and releasing on every exit.

## M19 — CSR/LPC scoping (2026-09-11): measured, and two of my own claims corrected

The LPC family (`NtRequestWaitReplyPort`, `NtReplyPort`, `NtReplyWaitReceivePort`,
`NtRequestPort`, `NtReplyWaitReplyPort`) is the last big refusal in the lane. This is the
scoping pass, re-derived from the tree rather than from carried-over notes — which is how two
of the carried-over claims turned out to be wrong.

**The failure is real and deterministic.** `lpc-rust/src/send.rs:413` rejects a request when
`DataLength + sizeof(PORT_MESSAGE) > TotalLength`. A 32-bit guest sets
`TotalLength = DataLength + 24`; the amd64 kernel wants `+ 40`. Always true, so every call
fails `STATUS_INVALID_PARAMETER` before it reaches a port.

**Correction 1 — "there is no marshaller" was wrong.** There is one: a *generated flat thunk*.
The name is absent from `manual.txt` and from the automarshal table, so wow64gen derived a
thunk from the `.spec` that widens each pointer ARGUMENT and passes the 24-byte struct through
untouched. The bug is not an absent marshaller, it is a thunk that marshals the pointer but not
what it points at — the same shape as the `NtCreateIoCompletion` PHANDLE bug (M18).

**Correction 2 — `USE_LPC6432` does not solve this,** though it looks purpose-built to.
`ndk/lpctypes.h:106` carries an unused switch that swaps `LPC_CLIENT_ID`/`LPC_SIZE_T`/
`LPC_PVOID` for their 64-bit forms, exactly to make a 32-bit and 64-bit side agree. Measured on
the host rather than reasoned about:

| build | `sizeof` | `ClientId` | `MessageId` | `ClientViewSize` |
|---|---|---|---|---|
| i386, plain | 24 | 8 | 16 | 20 |
| i386, `USE_LPC6432` | **32** | 8 | 24 | 28 |
| amd64 (either) | **40** | 8 | 24 | 32 |

It narrows the gap and does not close it: the i386 ABI aligns `long long` to 4 where amd64
aligns to 8, so the trailing `ULONGLONG` lands at 28 instead of 32. The header needs a real
marshaller. (Had this not been measured it would have shipped as "ReactOS already has a switch
for this.")

**The payload is much smaller than "~100 per-API layouts".** `CSR_API_MESSAGE` is a
`PORT_MESSAGE` header plus a `ULONG_PTR ApiMessageData[39]` union blob that each subsystem
server casts to its own struct — nominally ~20 BASE + ~9 USER + ~74 CONSOLE layouts. But:

- **13 of the 20 BASE APIs contain no pointer at all**, and the pointer-heavy ones
  (`BASE_CHECK_VDM`, `BASE_GET_NEXT_VDM_COMMAND`, `BASE_GETSET_VDM_CURDIRS`) are 16-bit
  DOS/Win16 support a WoW64 guest never reaches. The realistic pointer-bearing set is
  `BASE_CREATE_PROCESS` (`PebAddressNative`), `BASE_SXS_CREATEPROCESS_MSG` and
  `BASE_NLS_GET_USER_INFO` — one pointer each.
- A pointer-free struct that only needs field-copy under a layout diff is precisely what
  **marshalgen already generates**, from real clang i686-vs-aarch64 record layouts (M17).
- `CSR_CAPTURE_BUFFER` carries a `PointerOffsetsArray`: CSR already enumerates its own pointers
  because it already relocates them between client and server address space
  (`csrsrv/api.rs:768`, `+= BufferDistance`). The offsets are validated to lie inside
  `ApiMessageData`, so they are a ready-made table of where the pointers are — in 32-bit
  coordinates, which is the part that still needs translating.

So the job is: hand-write the PORT_MESSAGE header widen, hand-write the capture-buffer
relocation, and point existing generation machinery at the message structs. There is precedent
in the lane already — `Wow64pConnectPort` (`wow64_manual.c:1880`) widens the 0x24-byte
`CSR_API_CONNECTINFO` today, discriminated by sniffing a port name ending in `ApiPort`. For the
message APIs the discriminator is cleaner: `ApiNumber` is in the message.

**Blocked on a diagnostic that is itself broken — fix this first.** `WOW64_MANUAL_UNIMPL` is
`#define`d FOUR times, once per file, with no shared header anywhere in the module:
`0x7FFFFFFF` in `wow64_manual.c`, `wow64_thunks.c` and `wow64_win32k.c`, but **`0xC0BBBBBB`**
in generated `wow64_automarshal.c`. `Wow64ManualDispatch` tails unconditionally into
`Wow64AutoMarshalDispatch`, so the value that actually comes back is `0xC0BBBBBB` — and both
callers test `Status == WOW64_MANUAL_UNIMPL` against `0x7FFFFFFF`. **Both of those branches are
dead.** An unhandled syscall returns `0xC0BBBBBB` to the guest instead of printing the
`NEEDS MANUAL MARSHALLER` line that names it, and two comments still promise the old contract
(`wow64_thunks.c:25`, `wow64_manual.c:4550`). That line is exactly the instrument needed to
learn which LPC calls a guest actually makes, so it gets fixed before the probe: one shared
header, two distinctly-named sentinels (a hand marshaller declining is genuinely different from
no thunk existing anywhere), and `marshalgen.py` emitting the include instead of its own
`#define` so the drift cannot recur.

## M19 — CSR/LPC works: a 32-bit binary reaches PASS (2026-09-12)

The scoping above held up, and the job turned out to be one bug repeated at four levels of the
same structure. Every fix below was proven by measurement on a live boot, not by reading the
headers and reasoning.

| rung | what was wrong | measured result |
|---|---|---|
| `PORT_MESSAGE` header | 24 bytes guest / 40 host, so `DataLength + sizeof(PORT_MESSAGE) > TotalLength` rejected every call before it reached a port | transport `c000000d` → `00000000` ×25 |
| capture buffer | dereferenced a pointer that is a **server** address by design | host AVs 26 → 1 |
| reply `Data` union | copied verbatim, so the guest read the 64-bit padding word as `ConnectionInfo` | fault left ntdll32 |
| `ReadOnlyStaticServerData` | 8-byte array indexed with a 4-byte stride: `[1]` is the high half of `[0]` | `hello32.exe` **0x600D PASS** |
| `BASE_STATIC_SERVER_DATA` | opens with three `UNICODE_STRING` (8 vs 16), so `WindowsDirectory.Buffer` came out of padding | guest AVs 5 → **0** |

Current amd64 state, stable across three consecutive boots: **0 guest faults, 0 asserts, 0
above-4GB warnings**; `hello32`/`win32`/`file32`/`async32` all exit `0x600D`, and `gui32` exits
`0x600E` — `GUI32_WARN_GETPIXEL`, which its own source documents as a soft rung ("the draw
already proves the win32k path"). `marshal32`/`fault32`/`thread32`/`selfmod32` are created but
the smoke reaches the desktop marker before they report, so their current status is **unmeasured,
not passing** — that is the next thing to look at.

### Three traps this rung set, all of the same shape

**The shared LPC view cannot be grown.** Asking for `ViewSize + 1` page so the tail could be
wow64 scratch fails the connect outright with `c000001f` (`STATUS_INVALID_VIEW_SIZE`) — the port
section caps it. Carve the scratch from *inside* the granted view and report the reduced
`ViewSize` to the guest.

**That view's pages are reserved, not committed** (`state=0x2000`). The first write to the
scratch faulted exactly like the bug it was meant to fix. Commit it explicitly at connect.

**Deriving a layout instead of measuring it costs a boot each time.** `SYSTEM_BASIC_INFORMATION`'s
`ULONG_PTR` block starts at **32** on amd64, not 28 — the seven `ULONG`s end at 28 and a
`ULONG_PTR` wants 8-byte alignment. Deriving 28 put `MinimumUserModeAddress = 0x1000000000000` on
serial. Worse, a host measurement program is only trustworthy if it is validated: the first
version typedef'd `LONG` as C `long`, which is 8 bytes on LP64 Linux while Windows `LONG` is
always 32-bit, and it silently inflated `LUID`/`KSYSTEM_TIME`/`TIME_ZONE_INFORMATION`. Checking
the replica against the real headers' own `C_ASSERT`s (`NLS_USER_INFO` 0x1870,
`BASE_STATIC_SERVER_DATA` 0x1AC8 on i386) is what caught it.

**A narrowed value is not always a truncated one.** `MaximumUserModeAddress` is the host ceiling
`0x7FFFFFFEFFFF`; truncating hands the guest `0xFFFEFFFF`, which reads as a *kernel* address. It
is clamped to the i386 user ceiling instead.

The `BASE_STATIC_SERVER_DATA` shadow is a **snapshot** taken at connect, because there is no hook
to re-narrow on change. csrss fills it during server-DLL registration, before any client
connects, so what the guest reads at startup is stable — but `NlsUserInfo.ulCacheUpdateCount` is
one field csrss may bump later, and a 32-bit client will not see that bump.

## Third-party stress guests

The nine ladder guests each probe ONE mechanism on purpose, which is what makes them good
regression detectors and bad discovery tools: they only look where we already knew to look.
The stress guests are the opposite — large third-party C programs that never heard of us,
doing what they normally do. Both are built on demand (they fetch their own source, nothing
is vendored), so they are absent from a normal tree; that is why they live in `ladder.py`'s
`OPTIONAL` table rather than `LADDER`, and why `media_optional_staged()` asks the MEDIA
whether they are present instead of inferring it from serial.

| guest | source | axis it stresses |
|---|---|---|
| `sqlite32` | `tools/stress/sqlite/build.sh` (SQLite 3.45) | DATA path: allocator churn under a real pager, Win32 file I/O with locking and truncation, long compute between syscalls |
| `lua32` | `tools/stress/lua/build.sh` (Lua 5.4.7) | CONTROL path: `setjmp`/`longjmp` through error propagation, deep recursion after value-stack reallocation, and coroutine switching |

**Why Lua specifically.** `fault32` already covers SEH, and it is easy to assume that covers
non-local control flow — it does not. SEH unwinding and C `setjmp`/`longjmp` are different
machinery, and only the latter is what an emulator's block cache has to invalidate correctly.
Lua drives it constantly: `pcall`/`error` is a longjmp out of an arbitrarily deep interpreter
frame, coroutine resume/yield switches between independent stacks, and rung 4 raises from a
frame whose value stack has MOVED since it was entered. Nothing else in the ladder reaches any
of that.

**Ordering rule: the guest that can BUGCHECK goes LAST — and it is the GRADED
guests the rule protects.** spawn32 is the worked example in both directions: it
ran dead last while it could kill the machine, and moved ahead of the stress
guests the day it was promoted into the graded set, because a crash in sqlite32
costs every guest sequenced after it its verdict.

 `sqlite32` is the registered arm64
on-disk instability (roadmap: 0x1E / `RtlRaiseStatus+68`), and a guest that takes the machine
down costs every guest launched after it its verdict — silently, because the harness sees a
clean 9/9 parity pass and simply no line about the others. That is not hypothetical: it is what
happened on lua32's first arm64 run. While that bug is open, `sqlite32` stays last in
`winlogon.rs`.

**Verdict contract**, same as every other guest: `0x600D` on PASS, a distinct `0xBADn` per rung
so a red run names the rung on serial rather than just "it died". Verify a new guest's rung
arithmetic on the HOST first — build the same third-party sources natively against a few lines
of Win32 shim and run it. lua32's rung 1 shipped a hand-computed expected value that was off by
one; a 17ms host run caught it, where a boot would have cost minutes and blamed the emulator.
