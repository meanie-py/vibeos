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
  clean to desktop). Param2=2 localizes it to the FIRST guard in
  `PsLocateSystemDll` (ps-rust/psmgr.rs:328): the `ZwOpenFile` of
  `\SystemRoot\system32\ntdll.dll` returns STATUS_OBJECT_PATH_NOT_FOUND —
  the *directory* path won't resolve. `\SystemRoot` is an ObMgr symlink set
  up during I/O init; a parked CPU1 (which ran KiInitializeCpu / bumped
  KeNumberProcessors to 2 / registered KiProcessorBlock[1] before parking)
  somehow perturbs its resolution. Since CPU1 is parked (no live concurrency),
  suspect a KeNumberProcessors==2-sized allocation or an ObMgr/IO per-CPU
  structure left half-init, NOT a race. Next debug target: GDB watch on the
  ntdll ZwOpenFile path + compare ObMgr `\SystemRoot` link state at -smp 1 vs 2.
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
- 2026-07-11: **M2 COMPLETE — `-smp 2` BOOTS TO DESKTOP** (taskbar + live
  clock + untruncated `X:\reactos` version stamp; `smoke --smp 2` green
  ~1m54s, net suite green). The 0x6B and its successor 0x4C were BOTH
  pre-existing latent bugs the second CPU flushed out — full writeup in
  field manual **§9.51**:
  1. **0x6B root cause:** uniata's `CPU_num > 1` flips
     `g_opt_AtapiSendDisableIntr` → nIEN-gated ATAPI send; QEMU's IDE model
     never replays an IRQ suppressed under nIEN → boot CD INQUIRY dies → no
     CdRom0 → no `\SystemRoot`. Fixed: guard on `g_opt_VirtualMachine <=
     VM_NONE` (uniata-rust id_ata_part3.rs + C reference copy). NOTE: uniata
     and scsiport are RUST translations — the C copies are dead code with
     their DPRINT file:line strings baked into the Rust (a first-round fix
     went into the dead C and no-op'd).
  2. **0x4C root cause:** hand-rolled `KUSER_SHARED_DATA` shadow struct in
     `ke/amd64/kiinit.rs` put `NXSupportPolicy` at USD+0x40 (real +0x2D5) —
     inside `NtSystemRoot`. Harmless on the BSP (runs before the string
     exists); the AP's `KiInitializeCpu` chopped `C:\reactos`→`C:\reacto` →
     smss KnownDlls path c000003a → 0xC000021A. Caught with a TCG GDB
     hardware watchpoint on USD+0x40 (CPU#1 mid-store). Audit swept the
     whole class: i386/kiinit.rs, xstate.rs, thrdini.rs, rtlk misc.rs,
     ex time.rs all had fantasy-offset shadows — ALL swapped to bindgen
     `_KUSER_SHARED_DATA`. Owed: a lint for shadow-struct redeclarations of
     bindgen types.
  The old "suspect a KeNumberProcessors-sized allocation / ObMgr per-CPU
  structure" hypothesis above is DEAD — do not re-open on that lore.
  Kept diagnosability: smss now prints `SmpInit FAILED st=%x` /
  `SmConnectToSm FAILED st=%x` on its terminal failure paths.
  **M3 next:** un-park the AP into KiIdleLoop (set KeActiveProcessors at
  idle entry), TLB-shootdown/IPI send side (unblocks staged wave-2 A), then
  scheduler-on-2-CPUs; staged i386 trampoline (B) + arm64 PSCI (C) after.
- 2026-07-11 (cont.): **M3a — AP SCHEDULES; `-smp 2` boots to desktop at
  1m41s (faster than M2's parked baseline).** Park retired; AP runs
  KiSystemStartupBootStack → hand-built idle thread → KiIdleLoop, sets
  KeActiveProcessors at idle entry. Four root causes fixed en route (field
  manual **§9.52**): Phase-1 ordering (KeStartAllProcessors must precede
  ObInitSystem — per-CPU lookaside loops), ExitTrap IRQL protocol for the
  0xE1 stub, IF=0 sender deadlock (poll-service protocol), and the
  boot-crawl (lazy-idle TLB: senders skip hlt'ed CPUs; flush on wake).
  TLB shootdown is live: KeFlushEntireTb/KeFlushProcessTb/
  KeInvalidateTlbEntry broadcast via vector 0xE1 with pending-mask +
  poll-service + lazy-idle skip. `-smp 2` green 2/2 + `-smp 1` green.
  REMAINING for M3 proper: net suite at -smp 2 flaked once (NET_FLAKE —
  needs repeats), scheduled-on-CPU1 visibility counter, targeted
  single-VA remote flush + packet mailbox (staged A), then M4 soak.
- 2026-07-11 (M3a soak note): `-smp 2` ×3 post-landing: 2/2 fully green
  (1m43/1m46, boot+net), 1× NET_FLAKE — face: `[NETSTACK] TDI dgram
  self-test: datagram upcall TIMED OUT` (~1/3 at -smp 2; never seen on the
  default UP gates). Likely the §9.36 netstack-TDI family meeting real
  concurrency. OPEN — next -smp session: repro under native TCG + gdb,
  look at the TDI upcall completion path for a UP assumption (completion
  DPC targeting / event signal racing the timeout). CPU1-scheduling
  evidence for the record: the pre-fix ObCreateObject AV occurred ON CPU1's
  syscall path (LPC from services), proving user threads execute there.
- 2026-07-11: **M3b — TDI dgram NET_FLAKE ROOT-CAUSED + FIXED** (field manual
  §9.55). Not the §9.36 stack-IRP family — a plain "IRQL raise = mutual
  exclusion" **UP assumption in netstack-rust**. The TDI transport `INBOX`
  (MPSC command queue, many dispatch threads push / one poll worker drains via
  `mem::swap`) was guarded by only `IrqlGuard::raise_to_dpc()` over a bare
  `Vec`. On UP that IS mutual exclusion (DISPATCH masks preemption on the one
  CPU); under `-smp 2` a raise on CPU0 does nothing to a thread on CPU1 → a
  `Vec::push` reallocating races the consumer's `mem::swap` → lost `Cmd`
  (SEND_DATAGRAM / SET_EVENT) → the DNS upcall never fires → TIMED OUT. Fix:
  guard with a **real KSPIN_LOCK** via `SpinLockGuard`→`KeAcquireSpinLock`
  (raises to DISPATCH *and* spins; resolves per-boot — real MP spin on the SMP
  kernel, no-op fence on UP, so no UP regression + no driver feature flag).
  Two more of the same class fixed in the same pass (virtio-net is the smoke
  NIC, so both are live): `hal.rs` `static mut BOUNCES` DMA-bounce table
  (concurrent `share`s claiming the same slot → wrong physaddr/DMA to stale
  mem) and `serial.rs` marker emit (cross-CPU interleave splitting a
  `[NETSTACK]` marker the net suite greps). Workspace-wide audit for the
  "single vCPU" idiom found no other real instances (the rest: dead C
  comments, an ASCII-art string, and the correctly-gated M1a UP lock no-ops).
  **Verified + LANDED** (`89b57370425`, pushed to origin/smp-bringup on a
  ×3-clean-gate-green stamp): `-smp 2` amd64 12/12 green incl. `✓ net` across a
  warm INBOX-only and a clean 3-fix build (pre-fix ~1/3 failed); UP gate ×3 OK
  (i386+amd64+arm64 smoke + net + all builds/lints). One arm64 HANG_TIMEOUT in
  an interim gate was the pre-existing §9.36 netstack-TDI boot flake (3/3 on
  re-run; UP netstack behavior is IRQL-neutral under this change). M3 net-suite
  parity at -smp 2 is now met. Owed next: scheduled-on-CPU1 visibility counter,
  targeted single-VA remote flushes + packet mailbox (staged wave-2 A), then M4
  soak, then M5 (i386 needs the APIC HAL flavor; arm64 PSCI CPU_ON staged).
- 2026-07-11 (cont.): **M3 tail LANDED — visibility milestone, targeted single-VA
  shootdown, amd64 KeIpiGenericCall rendezvous + boot self-test.** All three owed
  M3-tail items done in one pass over the 0xE1 descriptor transport (ke/amd64/ipi.rs):
  1. **Scheduled-on-CPU1 visibility:** KiSwapContextResume counts non-idle switches
     per AP (count-only tap — NEVER DbgPrint mid-swap, see below); idle-exit/DPC-ISR
     flushes one-shot `[SMP] CPUn scheduling live` markers at the 1st/1000th switch.
     `test_milestones` now REQUIRES the marker per AP when `VIBEOS_QEMU_SMP > 1`
     (gate strengthened; UP boots unaffected).
  2. **Targeted single-VA remote flush:** the shootdown descriptor gained a VA slot
     (KiTbFlushVa, FULL sentinel); KeInvalidateTlbEntry → KiTbShootdownSingle(va) →
     awake receivers invlpg exactly that VA instead of a cr3 reload. Full-flush
     callers (KeFlushProcessTb/KeFlushEntireTb) unchanged; lazy-idle CPUs still get
     a deferred FULL flush on wake.
  3. **amd64 KeIpiGenericCall:** real NT rendezvous (targets arrive → sender's go →
     ALL CPUs run the worker at IPI_LEVEL simultaneously → acks) over the same
     lock-serialized descriptor protocol — replaces the DbgBreakPoint stub; the UP
     build now runs the worker at IPI_LEVEL too. A Phase-1 boot self-test (ex-rust
     init.rs, after KeStartAllProcessors) proves the worker ran on every active CPU
     (`[SMP] KeIpiGenericCall self-test: worker ran on 0x3 == active set OK` — seen
     green in-boot) and bugchecks 0x5C/0x4743 loud on mismatch.
  Design note: the staged wave-2 A NT-mailbox receive body (~/vibeos-rust-staging/
  smp/ipi-recv/) was NOT integrated — the validated poll-service descriptor protocol
  was extended instead (inherits the IF=0 deadlock-freedom); staged files remain as
  reference for a future true per-CPU-mailbox upgrade if generic-call concurrency
  ever matters.
  **Lesson (cost: one wedged smoke run):** never DbgPrint from inside
  KiSwapContextResume (mid-swap, SwapBusy still set) — the first marker cut printed
  there and was moved to idle-exit before the wedge theory was even tested.
  **OPEN — the -smp 2 residual flake is PRE-EXISTING and now pinned (field manual
  §9.56):** ~1/4–1/3 of -smp 2 boots creep-wedge (Themes/SamSs/SVCHOST connects
  stall → silence) or crash (0x19 POOL_HEADER_IS_ZERO; 0x1E c0000006 IN_PAGE_ERROR
  w/ cdfs frames). Control experiment: M3b-landed HEAD (stash the whole M3-tail
  delta) reproduces 1/6 under identical conditions → M3-tail exonerated; the
  invlpg narrowing separately bisected clean (full-flush receiver = same rate).
  SPECIALPOOL=* caught nothing at a store. Suspect: device-I/O completion / lost
  wakeup under real concurrency (§9.51 uniata family, not TLB). This is now THE
  gating item for M4 — the soak doubles as its corpus builder.
- 2026-07-12: **M4 first soak DONE — 453 boots amd64 -smp 2, 3 failures, ALL
  during the contended first hour; ~400 consecutive greens once the host went
  quiet** (29s median boot; the evening's ~110s/-25%-flake numbers were
  host-contention-amplified). §9.56 reframed: the residual race is REAL (the
  0x19/0x1E crash faces can't be host artifacts) but its window opens under
  extreme TCG timing skew — which is now the known repro amplifier. Next
  round: farm under deliberate host load + `--cswitchdep` build to pin the
  culprit hand-off. Also landed this session: the whole lane merged to
  master (fast-forward e08f777401a + docs 34d29afda89) — M0–M3 on mainline,
  all opt-in, defaults stay UP.
- 2026-07-12: **NEW finding — amd64 AP-start caps at ONE AP (2 CPUs total).**
  Probing `-smp 4` as a guest-concurrency amplifier for the §9.56 residual:
  3/3 BUGCHECK 0x61 HAL1_INITIALIZATION_FAILED. Deterministic face on serial:
  `[SMP] starting processor 2 … handing processor 2 to HAL … HalStartNextProcessor
  returned FALSE for 2; stopping … successful AP startup count is 1`. So CPU1
  (first AP) starts fine, CPU2 (second AP) fails inside HalStartNextProcessor,
  count falls short of KeNumberProcessors → the M3 `HalAllProcessorsStarted()`
  guard correctly bugchecks 0x61. This is a real multi-AP HAL bringup limit
  (M5-adjacent), NOT the residual flake — but it BLOCKS the "add vCPUs" amplifier
  strategy, so the §9.56 hunt stays on -smp 2 with cswitchdep + (next) kmtest
  guest-load until HalStartNextProcessor handles N>1 APs. Likely a single-AP
  assumption in the amd64 HAL AP-start (SIPI target / AP-startup scratch reused,
  or a HalpProcessorIdentity/APIC-ID-to-index single-slot). Next: read
  hal-rust amd64 HalStartNextProcessor + the AP trampoline scratch alloc
  (HalpApPtScratch / low-1MB SIPI page — is it freed/reused after AP1?).
- 2026-07-12: **§9.56 hunt strategy pivot (was chasing host timing).** 400
  quiet-host greens proved ambient contention is a lottery, not a probe. New
  plan: drive the race from inside the guest, host-independent — (a) more vCPUs
  [BLOCKED, see above: HAL caps at 2], (b) `--cswitchdep` build so the wedge
  face pins the illegal hand-off PC at switch time instead of a silent stall,
  (c) kmtest guest-load (turnkey-ish, needs a rostests build) hammering
  pool/IRP/sync — the suspect subsystems. Current soak: -smp 2 + cswitchdep +
  SPECIALPOOL=*, 5h. If it still ghosts, kmtest guest-load is the escalation.
- 2026-07-12 (cont.): **§9.56 BREAKTHROUGH — cswitchdep amplifier works; SwapBusy
  root cause RETRACTED; real bug = a double-queue.** The `--cswitchdep` build is the
  win of the pivot: it amplifies the residual from ~0.9% organic to ~25-45% per
  16-boot burst (host-independent — global per-switch spinlock widens the window),
  turning a multi-day flake into an on-demand repro. Hand-built ready-list tripwires
  (0x96 invariant at KiSelectReadyThread; 0x52 double-queue guard at KxQueueReadyThread
  entry) did the DETECTING (cswitchdep's OWN detector is useless at -smp 2 — single
  global `held_spinlocks`/`switch_irql`, see owed fix below). Findings:
  1. **RETRACTED the "SwapBusy wait-side unimplemented" root cause** (field manual
     §9.56). The amd64 run-side SwapBusy spin ALREADY EXISTS — `ntoskrnl/ke/amd64/
     ctxswitch.S:179-184` (`ThSwapBusy=0x95`, compiled at ntos.cmake:391). The earlier
     "31 writes, 0 spin-waits" was a `.rs`-only grep that missed the ASM. ReactOS has
     NO readier-side SwapBusy wait anywhere — a Rust spin would be dead code.
  2. **CONFIRMED the real symptom: KxQueueReadyThread called on an already-Ready
     thread (tripwire 0x52, priority 9)** re-inserts a still-linked WaitListEntry →
     tears DispatcherReadyListHead → the 0x1E/0x19/wedge faces. Two 0x22 victims saw
     the already-torn list downstream.
  3. **Caller-pinning IN FLIGHT:** `#[track_caller]` on the inline KxQueueReadyThread
     first reported shim_exports.rs:2069 for ALL callers — the extern-C symbol
     (ke_defs.rs:85) flattened every in-crate call through the wrapper. Repointed
     ke_defs to `pub use` the inline under `#[cfg(feature="smp")]` so track_caller
     reports each true site; re-burst running to read the 0x52 param4 (caller line).
     amd64 suspects: NtYieldExecution (thrdschd 858/882), unwait (wait.rs 379/392),
     KiQuantumEnd (dpc 506), KiDispatchInterrupt preempt (traphandler 105).
- **OWED (in-lane, do at a value-add point — likely alongside the UP→smp1 collapse):
  make cswitchdep per-CPU.** Root defect: `VIBEOS_CSWITCHDEP.cpu` is a SINGLE global
  `CpuState`; `held_spinlocks`/`switch_irql` are lone atomics shared by both CPUs, so
  the SWAP_IRQL/held-count detector reads another CPU's state and false-positives
  (the `expected=2 actual=12` / held-negative spam). Fix: `cpu: CpuState` →
  `cpus: [CpuState; MAX_CPU]` (cache-line padded), index by `Prcb->Number` at the ~15
  tap sites; per-thread shadow fields already thread-keyed and stay; global stats stay
  global. ~1-2h. Payoff: the DETECTOR then works identically at N=1 and N=2 — a
  permanent invariant checker for the unified path. This is a miniature of the whole
  UP-vs-SMP two-regime problem, so it's the natural down-payment when we make "UP =
  SMP with a 1-element processor array" (always take the locks / run the handshake;
  the same -smp 2 smoke becomes the UP test, deleting the "worked-on-UP-broke-on-SMP"
  bug class that hid this very flake). cswitchdep lane is CLOSED — do it here.
- **M5 sub-item (surveyed 2026-07-14): the idle-loop `Prcb->NextThread` lock hole
  exists IDENTICALLY on i386 and arm64 — but DON'T port it in isolation; land it
  with each arch's AP bring-up.** The amd64 §9.56 v2 fix (acquire `KiAcquirePrcbLock`
  inside the `cli` window, re-check NextThread under the lock, consume, `rq_inflight_unmark`,
  release, THEN `sti`) has exact twins waiting:
    - i386  — `ke/i386/thrdini.rs:302` (`KiIdleLoop`, consumes NextThread after `_enable()`,
      lock-free, even under `#[cfg(feature="smp")]`).
    - arm64 — `ke/arm64/thrdini.rs:253` (`KiIdleLoop`, consumes after `daifclr #2`, lock-free).
  These compile into the real `ntkrnlmp.exe` for all three arches (`ntkrnlmp-cargo/build.rs`
  always adds `smp`; `add_subdirectory(ntkrnlmp)` is unconditional), so they are
  compile-real latent code, NOT dead source. **Zero risk to the shipping kernel:** the
  `smp` feature is added only by `ntkrnlmp-cargo`, never `ntoskrnl-cargo`, so UP
  `ntoskrnl.exe` (what the default gate boots) compiles only the
  `#[cfg(not(feature="smp"))]` early-enable and none of the lock block.
  **Why NOT now (the discipline call):** (1) unvalidatable — i386/arm64 have no APs yet
  (M5 IS "start them"), so the lock is uncontended and its correctness can't be
  boot-tested under contention; (2) it's entangled with the per-arch context-switch
  handshake that M5 must complete anyway — specifically the **i386 `ctxswitch.S` has NO
  SwapBusy spin** (separate landmine, this doc's UP-endgame notes), whereas amd64's does
  (`ctxswitch.S:179`). The amd64 idle fix is self-contained (it never SETS `SwapBusy`, so
  amd64's spin is a no-op for the idle-promoted thread — the explicit `KiReleasePrcbLock`
  is the whole handshake), which is exactly why it ports as a clean mirror and NOT a
  double-release hazard — confirmed 2026-07-14. But landing the idle lock without the
  matching SwapBusy/release machinery for that arch is a half-fix that M5 will immediately
  re-touch. **So: do i386's and arm64's idle-loop lock port as the last step of that
  arch's AP bring-up, in the same change that adds the SwapBusy spin + AP start, so the
  whole handshake is validated as a unit on `-smp 2`.**
- 2026-07-14: **M5 arch = arm64 (PSCI), user-chosen. M5a (first AP start + park) IN
  PROGRESS.** Big scope reduction found on survey: the arm64 kernel entry is ALREADY
  SMP-ready — `KiSystemStartup` (kiinit.rs:169) does `Cpu = KeNumberProcessors++` and
  gates ALL global init `if Cpu == 0`, reads the per-AP PCR from `LoaderBlock->Prcb`
  (CONTAINING_RECORD), and `KiSystemStartupBootStack` (kiinit.rs:285) already has the
  alive handshake `(*LoaderBlock).Prcb = 0;` before `KiIdleLoop()`. So M5a is purely the
  HAL mechanism + a KeStartAllProcessors driver — the kernel side needed no changes.
  Written this session (WIP on branch, behind `feature="smp"` = ntkrnlmp only, zero UP
  risk):
    - `hal-rust/src/arm64/mpboot.rs` (NEW): `psci_cpu_on` (SMC64 `0xC4000003`), the AP
      trampoline (`global_asm!`, PC-relative `adr` so it survives being copied to a scratch
      page), a temp identity map, cache maintenance, and the real `HalStartNextProcessor`.
      `halstubs.rs`'s FALSE stub is now `#[cfg(not(smp))]`.
    - `ntoskrnl-rust/src/ke/arm64/mproc.rs` (NEW): `KeStartAllProcessors` — per-AP
      `_KIPCR`+idle-`_ETHREAD`+kernel-stack alloc, `KiInitializePcr` (made `pub(crate)`),
      publish to `KeLoaderBlock`, PSCI via HAL, bounded handshake wait. Auto-discovers the
      AP count by probing MPIDR 1,2,… until PSCI returns INVALID_PARAMETERS (no MADT parser
      needed). Replaced the empty `stubs.rs` `KeStartAllProcessors`.
  **DESIGN CORRECTION to the staged `arm64-ap/AP-ENTRY-DESIGN.md`:** it hand-waved the
  post-MMU-enable PC-continuity problem. The instruction right after `msr sctlr_el1`
  (MMU on) is fetched at the trampoline's LOW physical PC, which the kernel's high-half
  TTBR1 does NOT cover → instruction abort with no vectors installed = silent hang. Fix
  (in mpboot.rs): build a TEMPORARY identity map (TTBR0, one 1GB Normal-WB block covering
  the trampoline page) so the low PC stays mapped across the enable — the arm64 analogue of
  amd64's "identity map is gone" temp-PTE. Deterministic; doesn't gamble on the BSP's
  post-boot TTBR0 state. Second arm64-specific must-do the staged design omitted: **cache
  maintenance** — the scratch page is patched via a cached VA but the AP fetches it by PA
  with caches off, so `dc cvac`+`ic ivau`+`dsb`+`isb` over the page (and the idmap tables)
  is mandatory or the AP runs stale bytes. **REMAINING M5a:** compile-green (in progress),
  then first `-smp 2` arm64 boot + debug (greenfield PSCI — expect iteration on the
  MPIDR encoding, FADT conduit SMC-vs-HVC, and the trampoline). Open per NOTES.md:
  verify FADT `PSCI_USE_HVC` (assumed SMC); confirm QEMU `virt` MPIDR = flat Aff0=index.
  Then M5b (SGI IPI: `HalRequestIpi`→GICD_SGIR@0xF00 + receive dispatch), M5c (AP
  schedules + the arm64 idle-loop PRCB-lock fix lands here), M5d (TLB shootdown + soak).
