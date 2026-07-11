# SMP Bring-up — boot `-smp 2` to desktop and make our locks real

**Status: QUEST (2026-07-10, lane `smp-bringup`, worktree
`~/Projects/os-playground/vibeos-smp`). Handoff doc — read fully before starting.**

## Mission

vibeos is uniprocessor on every arch. That means **every spinlock, queued spinlock,
pushlock, and interlocked path in our Rust translations has never run under real
concurrency** — on a UP x86 kernel, spinlock acquire is ≈ an IRQL raise; the interlocked
contention paths are dead code. The dispatcher-lock family (§9.13), the pushlock
lost-wakeup lane, the waitdep-caught stack-scribbles (§9.36) — all were found *without*
an actual second CPU. SMP is the pressure test that makes the whole locking surface real.

Goal: **amd64 guest boots `-smp 2` to `shell ready`**, then survives the flake farm.
i386 and arm64 follow. Expect to unleash a pile of latent bugs — that is the point;
budget for a long tail of dispatcher/TLB/per-CPU bugs and fix them one root cause at a
time (no invented time budgets — AGENTS.md).

## Current state (surveyed 2026-07-10 — verified in-tree)

- **UP everywhere.** `hal/halx86/generic/up.c`: `HalRequestIpi` = `__debugbreak()`,
  `HalStartNextProcessor` returns FALSE, `HalpSetupProcessorsTable` = NOTHING. No SMP
  HAL variant dir; the `apic/` HAL flavor exists (amd64 uses it) and is where IPI +
  AP-start support belongs.
- **Upstream AP-start scaffolding exists on i386**: `ntoskrnl/ke/i386/mproc.c`
  (2023, Justin Miller / Victor Perevertkin) — real `KeStartAllProcessors` with per-AP
  `APINFO` (IDT/GDT/PCR/TSS pages). **amd64 `ke/amd64/mproc.c` is `UNIMPLEMENTED`.**
  Upstream ReactOS (our 2024 fork point) could *start* APs experimentally but not
  schedule on them — treat the scaffolding as a reference, not a working feature.
- **31 ntoskrnl C files carry `#ifdef CONFIG_SMP`** blocks (dispatcher, spinlocks,
  IPI, freeze). Nothing in our cmake defines `CONFIG_SMP` today for the kernel we ship.
- **`sdk/lib/rust/ntkrnlmp-cargo/` already exists**: a cargo link manifest for the MP
  kernel (`ntkrnlmp.exe`) that assembles the kernel `.S` with `CONFIG_SMP` (mirroring
  cmake's `target_compile_definitions(ntkrnlmp PRIVATE CONFIG_SMP)`). Verify what it
  currently produces and whether the *Rust* side gets a matching cfg — grep shows
  CONFIG_SMP-conditional Rust in `ntoskrnl-rust/src/ke/{freeze,thrdschd}.rs` and
  `ntkrnl-sys/src/inline/ke.rs`; find the cfg plumbing (cargo feature? env?) and audit
  which one the shipped kernel builds with.
- **QEMU side is trivial**: `-smp 2` on the command line. Plumb as an opt-in knob
  (e.g. `VIBEOS_QEMU_SMP=N` in `tests/qemu/vm.py`/`backend.py` + a `vibeos-cli smoke
  --smp N` flag). Note WHPX/KVM vs TCG: TCG multi-vCPU is MTTCG (real host threads) —
  fine; all three backends exercise real concurrency.

## Why this is mostly a *Rust-correctness* lane, not a C-porting lane

The C `#ifdef CONFIG_SMP` code came from upstream and is at least design-reviewed. The
risk concentrates in **our translations**, which were written and only ever tested UP:

- `ke` spinlock/queued-spinlock Rust (in `ntoskrnl-rust` ke + `ke-inlines`): do the
  CONFIG_SMP contention paths exist at all, or were they translated from the UP
  expansion? Audit against the C originals with CONFIG_SMP defined.
- **Per-CPU state**: `KeGetCurrentPrcb`/PCR access patterns in Rust — anything cached in
  a static that's per-processor in NT is a bug under SMP. Sweep for statics in ke/mm
  translations that shadow PRCB fields.
- **IRQL vs interlocked assumptions**: UP code may rely on "DISPATCH_LEVEL == mutual
  exclusion" (true UP, false SMP). The §9.13 family shows how these read.
- **TLB shootdown**: `MiFlushTlb` and friends must become IPI-based (`HalRequestIpi` +
  `KiIpiSendRequest`). Mm/ARM3 Rust is the dense center — expect work here.
- The slist crate is the *one* piece already proven concurrent (loom + Kani + real
  LDXP/CMPXCHG16B asm). Everything else: unproven.

## Suggested phasing

- **M0 — audit + build.** Get the `CONFIG_SMP` kernel *building* ×3 arches (cmake
  `ntkrnlmp` target or the existing ntkrnlmp-cargo path — decide which drives; verify
  the Rust cfg plumbing matches the asm/C side). Deliverable: an SMP-compiled kernel
  binary + a written audit of Rust UP-assumptions found (this doc, progress log).
- **M1 — SMP kernel, one CPU.** Boot the CONFIG_SMP kernel with `-smp 1` to desktop,
  gate-parity on the default configs. Proves the SMP-compiled code paths (real spinlock
  bodies!) are UP-safe. This alone may flush bugs (the spinlock fast paths change).
- **M2 — AP start (amd64 first).** Implement `HalStartNextProcessor` in the apic HAL
  (INIT-SIPI-SIPI via APIC ICR), port the i386 `APINFO` approach to amd64
  `KeStartAllProcessors` (per-AP PCR/PRCB/GDT/IDT/TSS/stack, `KiSystemStartup` AP
  entry), `HalRequestIpi`/IPI dispatch, `HalpSetupProcessorsTable`. Target: AP reaches
  its idle loop, `KeNumberProcessors == 2`, system still boots to desktop (scheduler may
  still keep everything on CPU0 via affinity at this stage).
- **M3 — schedule on both CPUs.** Release affinity; dispatcher/ready-queue/IPI paths go
  live. This is where the latent-bug tail lives (§9.13's cousins, TLB shootdown, DPC
  cross-CPU, timer expiry on the wrong CPU). Milestone: `-smp 2` desktop + the 28-test
  kernel suite green + net suite parity.
- **M4 — soak + gate.** `vibeos-cli farm` overnight on `-smp 2` with
  `SPECIALPOOL=*`; fix the flake families `farm-report` ranks. Then an opt-in smoke
  config (`--smp 2`, like whpx: opt-in until it earns reliability history). Default
  gate stays `-smp` unset/UP until the user flips it.
- **M5 — i386, then arm64** (PSCI `CPU_ON` on QEMU virt; GICv2 IPIs/SGIs).

## Oracles & tooling (all exist — use them from day one)

- **waitdep** (`ke/waitdep.rs`) always-on; **lockdep** via `vibeos-cli build --lockdep`;
  **cswitchdep** (see `docs/engineering/cswitchdep-design.md`).
- `--tripwire "SPECIALPOOL=*"` per-boot; `farm`/`farm-report` for the flake tail.
- The GDB toolkit (`qemu/start.sh --gdb`, `inspect.sh`) is SMP-aware at the QEMU level
  (`info cpus`, per-thread bt) — extend `inspect.sh` verbs for per-CPU dumps early; it
  pays for itself immediately.
- Field manual §9 is the crash-hunting playbook; §9.13–9.16 are the closest priors.

## Ground rules (binding)

- AGENTS.md in full: $W path discipline, no push without gate, never weaken smoke,
  Rust stays Rust (fix the translation, never revert to C), fail loud.
- **Default configs stay UP until the user opts otherwise.** All SMP boots/gates are
  opt-in flags. Never let an SMP experiment change the default smoke behavior.
- Kernel asm changes (`ks*.inc` CONFIG_SMP blocks) affect the cargo link — coordinate
  with the ntkrnlmp-cargo manifest, don't fork the truth.
- One root cause at a time; a hang/BUGCHECK under SMP is ~1 root cause. Persistence
  rule applies.

## M0 audit findings (2026-07-10 — supersedes parts of the survey above)

**The C side is a non-issue: there is NO still-compiled kernel C file with a live
`#ifdef CONFIG_SMP` block.** All 24 of them (ke/spinlock,thrdschd,procobj,ipi,dpc,
freeze,bug,processor, ex/pushlock,init, config/cmsysini, kd64/kdinit, mm/ARM3
sysldr+virtual, per-arch cpu/kiinit/context/irqobj/thrdini/stubs/spinlock, +
mproc.c both arches) are translated to Rust with the C commented out of
`ntos.cmake`/`ntkrnlmp/CMakeLists.txt`. The only compiled C among the grep hits
(`ke_rust_shims.c`) has CONFIG_SMP in a doc comment only. `CONFIG_SMP` reaches
only the assembler (`ks*.inc` via ntkrnlmp-cargo `.asm_extra_defines`). So SMP
correctness lives entirely in **asm + Rust** — the quest doc's framing confirmed,
and stronger: there is no design-reviewed C fallback in the build at all.

**Rust cfg plumbing was missing and inconsistent (fixed in this lane):**
- Three cfg spellings coexisted, none wired: `feature = "smp"` (42 sites, the
  majority — spinlock/ipi/freeze/kiinit/pushlock/resource), `feature =
  "CONFIG_SMP"` (13 sites), bare `#[cfg(CONFIG_SMP)]` (config-rust). No crate
  except ex-rust even declared the feature → **every Rust SMP path was
  unconditionally compiled out**, including in `ntkrnlmp.exe` (whose build.rs
  comment wrongly claimed "the Rust body is identical UP/MP — only the asm
  differs").
- Fixed: unified on `feature = "smp"`; declared in ntkrnl-sys, ke-inlines,
  ntoskrnl-rust, ex-rust, kd64-rust, config-rust (each forwarding
  `ntkrnl-sys/smp`); kernel-rust `smp` umbrella enables all; ntkrnlmp-cargo
  build.rs passes `rust_features(["smp"])` always (accumulated with
  lockdep/cswitchdep — NB `rust_features` REPLACES, doesn't append).

**Rust SMP-path reality (what `--features smp` actually turns on today):**
- `ke/spinlock.rs`, `ke/ipi.rs`, `ke/freeze.rs`, `ex/pushlock.rs`,
  `ex/resource.rs`, thrdschd/procobj/irqobj: real translations of the C SMP
  sides exist behind the gates (routed through `KxAcquireSpinLock` etc.). Never
  compiled until now — expect a compile-fix pass.
- **The keystone gap: the Kx/Ki lock inlines are UP-only with no smp variant.**
  `ntkrnl-sys/src/inline/ke.rs` (`KxAcquireSpinLock`/`KxReleaseSpinLock` =
  compiler fence; the "UP no-op lock functions" section: KiAcquire/Release
  ThreadLock/PrcbLock/DispatcherObject/TimerLock/DispatcherLockAtSynchLevel,
  KiTryThreadLock, KiRequestApcInterrupt, KiRescheduleThread,
  KiSetThreadSwapBusy, KiCheckDeferredReadyList, KiInsertDeferredReadyList) and
  the mirrored `_rs_*` exports in `ke-inlines/src/lib.rs`. These need
  `#[cfg(feature = "smp")]` bodies translated from `ke_x.h` lines 305–603 +
  `spinlock.h` (M1a). Queued spinlocks delegate to Kx* (ReactOS's simplified
  queued-lock semantics), so the Kx pair + ke_x.h helpers cover the whole lock
  surface.
- `ke/amd64/mproc.rs` `KeStartAllProcessors` is the faithful `UNIMPLEMENTED`
  upstream stub; `ke/i386/mproc.rs` is the real 424-line APINFO translation (M2
  ports it to amd64).

## Progress log

- 2026-07-10: Lane opened; survey above. Nothing built or booted yet.
- 2026-07-10: M0 audit (above). `smp` feature plumbed end-to-end (7 Cargo.tomls +
  ntkrnlmp-cargo build.rs); cfg spellings unified on `feature = "smp"`.
  Gotcha for posterity: editing workspace Cargo.tomls while a `vibeos-cli build`
  is in flight makes cargo's resolver see a half-applied feature graph
  (LINK_FAIL/"failed to select a version" — benign, rebuild after the edits).
- 2026-07-10 (cont.): **M1a lock bodies written + M0 build green on i386+amd64.**
  Real SMP bodies for the whole ke_x.h/spinlock.h lock family in BOTH copies
  (ntkrnl-sys inline/ke.rs + ke-inlines _rs_*): Kx spin, dispatcher/dispatcher-
  object/thread/PRCB/timer locks, KiRequestApcInterrupt/KiRescheduleThread IPIs,
  KiInsertDeferredReadyList, KxQueueReadyThread affinity gate. SYNCH_LEVEL made
  smp-aware (UP=2; SMP i386=27/amd64=12; 3 definition sites). ~8 compile-fix
  rounds on the never-compiled gated code (bindgen union paths, bindgen-local
  vs base ULONG_PTR widths, `extern "system"` vs `"stdcall"` fn-pointer types,
  ke_defs' KfRaiseIrql was declared cdecl → fastcall on i386).
  **M1b harness knob landed**: `VIBEOS_QEMU_SMP=N`/`smoke --smp N` (vm.py both
  backends), `build --smp-kernel` → `-DVIBEOS_SMP_DEFAULT_BOOT=ON` → livecd
  freeldr.ini defaults to the new `LiveCD_SMP` entry (/KERNEL=ntkrnlmp.exe, no
  /HAL override — amd64-safe). Defaults stay UP.
  **Subagent wave (4 agents → ~/vibeos-rust-staging/smp/, all integrated except
  hal-apic)**: ke IPI core (KiIpiSend/SendPacket/SignalPacketDone/
  GenericCallTarget real bodies + KiIpiServiceRoutine aarch64-cfg and 4-arg
  worker-ABI fixes); amd64 KeStartAllProcessors APINFO port (mproc.rs, +
  the AP handshake `LoaderBlock->Prcb = 0` + park added to amd64
  KiSystemStartup); i386 KxFreeze/Thaw/SwitchKdProcessor parity port (+ new
  i386 KiRestoreProcessorState in cpu.rs); HAL apic INIT-SIPI-SIPI +
  HalRequestIpi module **staged only** (needs real trampoline bytes — its
  placeholders must not ship; ~/vibeos-rust-staging/smp/hal-apic/).
  **M2 debts recorded by the agents**: IPI_FREEZE receive side unwired on
  i386/arm64 (KiProcessorFreezeHandler not reachable from trap/IPI dispatch);
  KiIpiSendPacket i386 extern-"system"-vs-FASTCALL audit; amd64 mailbox
  generic-call gap (upstream parity); trampoline low-page (<1MB) allocation
  (HalpLowStub equivalent); arm64 freeze snippet staged not integrated.
- 2026-07-10 (cont.): **M1 GREEN — amd64 ntkrnlmp.exe + MP HAL boots `-smp 1` to
  desktop** (`shell ready` in 1m31s, full taskbar+clock in the screenshot, zero
  bugchecks on serial). First-ever boot of the SMP-compiled lock family: real Kx
  spinlocks, queued dispatcher locks, SYNCH_LEVEL=12, IPI plumbing (BSP-only).
  Two-step lesson en route: first `-smp 1` boot died BUGCHECK **0x79
  MISMATCHED_HAL (2, 1, 3, 0)** — the MP kernel (BuildType=DEBUG only) met the
  UP HAL (DEBUG|UNIPROCESSOR); the exact-equality handshake in halinit worked
  as designed. Fix = MP HAL pairing: hal-rust `smp` feature (buildtype.rs drops
  PRCB_BUILD_UNIPROCESSOR, HalpIrqlSynchLevel 2→IPI_LEVEL-2), plumbed
  VIBEOS_SMP_DEFAULT_BOOT → gcc.cmake `_cargo_env` VIBEOS_SMP_HAL=1 (hal-cargo
  only) → build.rs (`rerun-if-env-changed`). Second 0x79 "recurrence" was a
  STALE-MEDIA boot racing the rebuild — check artifact mtimes vs smoke start
  before believing a re-fail.
- 2026-07-10 (cont.): **M0+M1 LANDED on master** (`36e076feaba`, pushed after
  ×3-green gates on both the branch tree and the post-merge tree — origin
  master had moved: fex-arm lane; one lint_kernel tally conflict, resolved by
  regenerating the baseline on the merged tree, 767 = both lanes' fixes).
  **M2 progress:** amd64 AP trampoline is REAL machine code —
  `~/vibeos-rust-staging/smp/hal-apic/aptramp64.S` (16→32→64-bit, PIC via
  runtime GDTR/far-ptr patching, TSS busy-bit clear before ltr, LoaderBlock→
  rcx), 274-byte flat blob via system GNU as/ld (`regen-trampoline.sh`),
  disassembly-verified, include_bytes!'d with offset_of! const-asserts pinning
  the param-block ABI (packed contiguous GDTR/IDTR — the first draft's padded
  layout would have lgdt'd garbage). Discovery: `HalpLowStub` ALREADY exists
  (halacpi.rs Phase-0 allocates 1 page <1MB + maps it) — subagent note was
  wrong; only 1 page though. **GDB probe on the live amd64 kernel: the boot
  identity map is GONE** (PML4[0] is normal user space), so the trampoline
  can't flip paging on the kernel CR3 as-is → M2 design locked to (b): a
  temporary VA==PA PTE for the stub page in the System process around the
  SIPI window (self-map PTE write + invlpg, torn down after APs start);
  mini-page-tables-in-stub is the hardening fallback. Full decision tree +
  corrections in the staging NOTES.md.
- 2026-07-10 (cont.): **M2 — FIRST AP START. amd64 CPU 1 takes the SIPI, runs
  the real trampoline, executes KiSystemStartup end-to-end, and parks**
  (`[SMP] AP 1 up, parked pre-scheduler` + `processor 1 online` on serial).
  First application processor ever started in this codebase. Path to green,
  each a distinct root cause found by GDB on the AP thread:
  1. **Trampoline was real bytes but not LINKED** — `aptramp64.bin` is
     `include_bytes!`'d, not a `.rs`, so cmake's cargo-module DEPENDS glob
     (`*.rs *.toml`) never saw the regenerated blob and shipped a STALE
     hal.dll (0xF4 debug-cycle). Fixed the glob to include `*.bin`
     (gcc.cmake) — permanent infra fix for any include_bytes! asset.
  2. **#DF at KiSystemStartup's first `movaps`** — the trampoline `lretq`'d
     onto a 16-aligned stack; the amd64 ABI entry state is rsp≡8 (mod 16)
     (caller's CALL pushed a return addr), so every aligned xmm spill was
     off by 8 and #GP'd. Fixed: trampoline pushes a fake NULL return addr
     before the LoaderBlock frame (aptramp64.S).
  3. **Low-stub alloc FAILED under UEFI** — mainline reserves 1 low page but
     I'd bumped it to 4 for the identity-map page tables; UEFI leaves no
     4-page run <1MB (measured: one 2-page FirmwareTemporary at 0x37000).
     Split: low stub stays 1 page (only the SIPI target must be <1MB); the
     3 PDPT/PD/PT scratch pages allocate separately with a 4GB cap
     (HalpApPtScratch, halacpi.rs).
  4. **BSP idle-wedged post-park** — the AP set `KeActiveProcessors |= 1<<Cpu`
     then parked with interrupts off; the dispatcher routed a ready thread /
     DPC to CPU 1's queue and it never ran, so the BSP went idle forever
     (hung at PnP). Fixed: a parked AP does NOT advertise in
     KeActiveProcessors (kiinit.rs, smp+Cpu!=0 gated) — M3 sets it when the
     AP actually enters the idle loop.
  Also landed from the M3-audit wave: `KeStartAllProcessors` call site wired
  into ex-rust Phase 1 (+ HalAllProcessorsStarted check), and
  `KeGetCurrentProcessorNumber` reads `Prcb->Number` under smp (was hardcoded
  0 — would have silently poisoned IPI completion / dispatcher / freeze).
  **OPEN (M2 tail / M3 boundary): `-smp 2` now BUGCHECKs 0x6B
  PROCESS1_INITIALIZATION_FAILED (0xc000003a STATUS_OBJECT_PATH_NOT_FOUND, 2)
  at PsLocateSystemDll** — 2-CPU-specific (`-smp 1` of the same build boots
  clean to desktop). Boot runs far into Phase 1 before failing; likely a
  per-CPU init the parked AP left half-done (KeNumberProcessors==2 now, but
  CPU1's PRCB parked before full init) surfacing in ntdll section mapping.
  Next debug target.
- **Subagent wave 2 (4 Sonnet agents, all delivered):** A = amd64 IPI 0xE1
  receive body (staged; corrected premise — amd64 freeze already works via
  NMI, 0xE1 dead both ways, needs an amd64 mailbox send-side design = M3);
  B = i386 `aptramp32.S` (real 212B blob, disasm-verified, `lret`-onto-
  prebuilt-stack contract verified vs C — staged); C = arm64 PSCI CPU_ON
  survey (`smc #0`, EL1; GIC SGI plumbing absent, Rust IPI layer already
  arch-generic; freeze snippet splice-ready); D = M3 scheduler-entry audit
  (4 BROKEN / 4 GAP / 6 READY — 2 spine-critical BROKENs already fixed above).
  All staged under ~/vibeos-rust-staging/smp/.
- **UP endgame (decided 2026-07-10, not yet scheduled):** once M4 soaks clean,
  flip the gate default to the MP kernel, deprecation-window the UP build, then
  delete the `cfg(not(feature="smp"))` sides entirely (NT precedent: Vista
  dropped the UP kernel). "Defaults stay UP" is a bring-up rule, not forever.
