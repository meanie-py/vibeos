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
- 2026-07-14 (cont.): **M5a code COMPILES INTO arm64 ntkrnlmp.exe** (commits
  1da2142fdd3 + 84b08a8bf40). First `-smp 2` arm64 boot reached desktop but the SMP
  code NEVER RAN — critical discovery: **the arm64 ntkrnlmp MEDIA PIPELINE is unwired**,
  so `--smp-kernel` boots the UP `ntoskrnl.exe`, not ntkrnlmp. Three gaps, each now
  pinned:
    1. **Build:** `archcfg.ninja_targets["arm64"]` lists `ntoskrnl` but NOT `ntkrnlmp`,
       so `build --arch arm64 --smp-kernel` never builds `ntkrnlmp.exe`. (amd64 builds it
       via the `livecd` DAG + VIBEOS_SMP_DEFAULT_BOOT; arm64's hand-picked staged set omits it.)
    2. **Stage:** `qemu/mkramdisk.sh` copies only `ntoskrnl.exe` (lines 76/266), never `ntkrnlmp.exe`.
    3. **Boot-select:** `qemu/freeldr.ini` `[ReactOS_Arm64]` Options has no `/KERNEL=`, so
       it defaults to `ntoskrnl.exe`. Needs `/KERNEL=ntkrnlmp.exe` when SMP (like
       `livecd.ini`'s `[LiveCD_SMP]`).
  **A NON-OBVIOUS TRAP this flushed out:** the `smp` feature is set ONLY by ntkrnlmp-cargo,
  so `mproc.rs`'s `#[cfg(feature="smp")] smp_impl` is compiled ONLY in the ntkrnlmp build —
  a plain `build --arch arm64 --smp-kernel` links `ntoskrnl.exe` (smp OFF) GREEN while
  `ntkrnlmp` silently fails to relink and keeps a stale binary. **Always compile-check
  SMP-gated kernel code with `ninja ntkrnlmp` (or a build that actually builds ntkrnlmp),
  NOT the default `--smp-kernel` build** — the latter's green ntoskrnl.exe hides
  smp_impl errors (cost: NonPagedPool/TAG_KERNEL imports + `as _` size cast, invisible
  until `ninja ntkrnlmp`). **NEXT (M5a-media):** wire the 3 gaps above (conditional on
  the existing VIBEOS_SMP_DEFAULT_BOOT env, mirroring the x86 livecd path), then re-boot
  `-smp 2` to finally exercise mpboot/mproc and debug the real PSCI/trampoline path.
- 2026-07-14 (cont.): **M5a COMPLETE — arm64 CPU 1 boots via PSCI to KiIdleLoop; BSP
  reports "2 processor(s) online" (commit adf7e452fdb).** Media pipeline wired
  (archcfg builds ntkrnlmp for arm64; mkramdisk `--smp` stages it + injects
  `/KERNEL=ntkrnlmp.exe`; commit df353be6e20). Three greenfield bugs, each pinned
  before the next surfaced (reusable arm64-AP-bringup gotchas — will recur for any
  future arm64 secondary-core work):
    1. **PSCI conduit = HVC, not SMC.** `smc #0` is UNDEFINED on QEMU virt (EL1-only,
       no EL3) → the BSP took an EC=0 abort and wedged *right after* the CPU_ON call
       (MPIDR=0x80000000 ⇒ it was the BSP, not the AP; "post-PSCI SUCCESS" never
       printed). QEMU's generated DTB carries `method = "hvc"` (verified by
       `-machine dumpdtb` → the /psci node has "hvc", never "smc"). Fixed: `hvc #0`.
       (Consuming the FADT ARM_BOOT_ARCH PSCI_USE_HVC bit is TODO for real EL3 firmware.)
    2. **AP MMU-enable needs a purpose-built idmap.** The BSP's live TTBR0 does NOT
       identity-map the contiguous scratch trampoline page (MmAllocateContiguousMemory
       lands it hundreds of MB into RAM), so the AP's first instruction fetch after
       `msr sctlr_el1` faulted — it printed the MMU-off 'A' trace then died before 'B'.
       Fixed: `build_trampoline_idmap` now maps a Normal-WB 1GB block for the trampoline
       page + a Device 1GB block for the PL011 UART (MAIR Attr1=Device-nGnRnE, PTE
       AttrIndx 1) so post-MMU trampoline traces still reach hardware.
    3. **AP FP/SIMD trap → silent vector re-entry (the hard one).** The BSP inherits an
       FP-enabled CPACR_EL1 from UEFI firmware; a PSCI-started AP comes up at the
       trapping reset value (FPEN=00). The first NEON-vectorized store the AP executed
       (KiInitSpinLocks' LockQueue block) took an EC=0x07 SIMD-disabled abort, and the
       arm64 exception path re-entered the vector on a corrupted SP → silent infinite
       re-entry (no [KEXC]). **GDB was decisive** where UART bisection wasn't (the two
       CPUs' serial writes interleave char-by-char, making "how far did the AP get"
       unreliable — a memory-only stage beacon fixed that and pinned the hang inside
       KiInitSpinLocks; then GDB on the native `-smp 2` boot showed CPU1
       PC=KiArm64VectorTable+0x200, "FPU disabled", garbage SP, and the BSP deadlocked
       in KeIpiGenericCall). Fixed: set CPACR_EL1.FPEN=0b11 at the top of KiSystemStartup
       (idempotent on the BSP; runs on all CPUs).
  **Method note for the next debugger:** for arm64 SMP hangs, reach for GDB on a native
  `qemu-system-aarch64 -smp 2 -s` boot early — `info threads` + `monitor info registers -a`
  gives each CPU's PC/SP and `$ESR_EL1`/`$FAR_EL1` directly. UART tracing from an AP is
  actively misleading because it interleaves with the BSP; use a memory beacon
  (`KiArm64ApStage`, read by the BSP on handshake timeout with a hung-vs-slow re-read).
  Default UP arm64 unaffected (boots to desktop; all AP code is smp-gated, CPACR OR is
  idempotent).
  **M5b (NEXT, exact starting point):** `-smp 2` boot now hangs *after* "2 processor(s)
  online" — the BSP's first cross-CPU IPI (KeIpiGenericCall self-test / Phase-1 sync)
  never delivers because the AP's **GIC CPU interface / SGI receive path is unwired**
  (HalInitializeProcessor is an empty stub, so the AP's GICC/redistributor is never
  enabled). Wire: AP-side GIC CPU-interface enable in HalInitializeProcessor + SGI
  receive dispatch (trap.rs → KiIpiServiceRoutine) + HalRequestIpi → GICD_SGIR@0xF00.

- **2026-07-14 — M5b COMPLETE: arm64 GIC SGI IPI wired; `-smp 2` boots to DESKTOP
  (3/3 green ~50s; UP regression green).** Commit `8439ceca339`. The self-test proof:
  `[SMP] KeIpiGenericCall self-test: worker ran on 0x3 == active set OK` — SGI path
  works end to end (HalRequestIpi → GICD_SGIR → AP IRQ → KiIpiServiceRoutine → packet
  worker → signal-done), and the boot then continues clean through SMSS to
  `shell ready` + explorer `_tWinMain`. Zero IRQL violations / WAITDEP / BUGCHECK.
  What it took (three pieces, all in the commit):
    1. **SGI transport (hal clock.rs).** SGI 0 == the NT IPI at IPI_LEVEL(14) → GIC prio
       0x10 (above CLOCK 0x20, below only HIGH). Send: `gic_send_sgi` (`dsb ish` then
       GICD_SGIR@0xF00, target-list filter 0). Receive: new `intid < 16` branch in
       HalClockInterruptArm64 — HalpEnterInterruptIrql(IPI_LEVEL) → KiIpiServiceRoutine
       (new arm64 spec export) → HalpExitInterruptIrql; EOI writes back the full IAR
       (source-CPU field included). Per-CPU enable: `gic_percpu_init` — the GICv2
       IPRIORITYR/ISENABLER0 copies for INTIDs 0-31 and the whole GICC are **banked per
       accessing core**, so each AP programs its own in HalInitializeProcessor (was an
       empty stub) and the BSP gets its bank at clock init.
    2. **The early-window mapping problem.** KeStartAllProcessors (+ the IPI self-test)
       runs **before MmInitSystem(1)** — MmMapIoSpace can't exist yet, so the GIC is
       reached via boot-identity PAs (`gic_ensure_early_bases`, first-writer-wins CAS;
       clock init later overwrites with the real VAs). The identity mapping itself is
       now **guaranteed by the loader**: ported `MempMapDevicePage` to the live Rust
       winldr_arm64.rs and mapped UART/fw_cfg/GICD/GICC with Device attrs. (Side find:
       the 2026-07-05 tripwire commit had added that function to the by-then-INERT C
       winldr — classic edited-the-dead-C trap — so the fw_cfg mapping never shipped and
       early device access worked only by accident of the UEFI memory-map walk.)
    3. **Per-CPU IRQL + softint state (halstubs.rs) — the silent prerequisite.** The
       arm64 HAL kept ONE global soft IRQL + ONE softint pending mask + ONE
       interrupt-context depth. With 2 CPUs that's lost-update city: CPU0's clock tick
       would gate CPU1's softint dispatch, and CPU1's lower-drain could consume (and
       clear) a DISPATCH request CPU0 posted for itself — lost-DPC hangs. Soft IRQL now
       lives in **KPCR.CurrentIrql via tpidr_el1** (KiInitializePcr already initializes
       it per CPU; boot-global fallback for the pre-tpidr window), and pending/
       in-progress/depth live in a cache-line-padded per-CPU array.
  **M5c (NEXT):** prove the AP actually *schedules* (desktop boots, but nothing yet
  measures CPU1 picking up threads — wire the CPU1-visibility counter from the M3-tail
  list, arm64 leg). Then M5d: KiTbShootdown via SGI + `-smp 2` soak. AP-local vtimer
  (per-CPU clock ticks) is also still un-armed — the AP gets no CNTV interrupts; fine
  for idle+IPIs, needed for real per-CPU quantum/timer work.

- **2026-07-14 (cont.) — M5c CORE COMPLETE: arm64 AP provably SCHEDULES.** Commit
  `e257ec33d80`. Proof: `[SMP] CPU1 scheduling live (first non-idle context switch)`
  on 3/3 arm64 `-smp 2` boots (45–55s to desktop, 2 CPUs online, self-test 0x3 OK,
  zero WAITDEP/BUGCHECK); UP arm64 regression green (47s); amd64 `-smp 2` regression
  green (marker still prints post-hoist). The `-smp` milestone gate (test_milestones
  requires `CPU<n> scheduling live` per AP when `VIBEOS_QEMU_SMP > 1`) is now
  satisfiable on arm64 — note the CLI `smoke` fast path does NOT run test_milestones,
  which is why the M5b smokes passed without it; only pytest/gate runs enforce it.
  Three pieces:
    1. **Visibility tap hoisted arch-neutral** (amd64/ipi.rs → ke/ipi.rs, pure
       atomics), parameterized by CPU number so each arch derives it from its own
       invariant PCR source (arm64's linked `KeGetPcr` is the stale-x18 copy —
       thrdini.rs comment). arm64 leg: count tap in KiSwapContextResume (Number!=0 &&
       new != IdleThread), flush at idle-loop top (idle context — never print from
       the mid-swap tap).
    2. **arm64 KiIdleLoop SMP promote fixes** (mirrors amd64/stubs.rs, §9.56): consume
       NextThread UNDER THE PRCB LOCK with IRQs still masked + re-check under the lock
       (the inherited lock-free consume races KiDeferredReadyThread's standby
       displacement → a Running thread set DeferredReady → union-alias list tear);
       the KiSelectReadyThread branch gets the same lock treatment; and
       `rq_inflight_unmark` at BOTH promote sites. **The missing unmark was why the
       M5b `-smp 2` settle logs spammed `[RQDEFER2] DOUBLE-DEFER` with
       `stale_by == caller_line`** — every arm64 idle-loop promote (BSP and AP)
       leaked a stale in-flight mark, so each subsequent wake false-positived the
       §9.56 oracle. 0 occurrences across all post-fix boots. UP shape byte-identical
       (cfg-gated). arm64 SMP `SYNCH_LEVEL == DISPATCH_LEVEL` (ke_defs) and idle is
       pinned at DISPATCH, so no SYNCH raise around the swap (amd64 raises to 12).
    3. **Fail-loud UP HalStartNextProcessor stub** — the trap that cost this session a
       debug round: the arm64 SMP HAL flavor rides the **VIBEOS_SMP_DEFAULT_BOOT cmake
       CACHE var** (gcc.cmake → `VIBEOS_SMP_HAL=1` env → hal-rust `smp` feature), so a
       plain UP `vibeos-cli build --arch arm64` reconfigure flips hal.dll back to UP,
       and a later raw `ninja ntkrnlmp` inherits the UP cache — **MP kernel + UP HAL
       boot fine on arm64 (no MISMATCHED_HAL 0x79 handshake here!) and silently
       degrade to 1 CPU**: HalStartNextProcessor is the cfg(not(smp)) FALSE stub, and
       mproc.rs reads FALSE as "no more CPUs". The stub now DbgPrints the rebuild
       hint. **Rule: after ANY plain arm64 build, re-run `build --arch arm64
       --smp-kernel` before trusting an -smp boot** (it reconfigures the cache,
       rebuilds the smp HAL, and runs `mkramdisk --refresh --smp` itself — no manual
       mkramdisk needed).
  **M5c tail (owed):** AP-local vtimer arming — CPU1 still gets no CNTV interrupts, so
  it never reaches 1000 switches by boot-marker time (threads land there only via
  explicit dispatch IPIs); per-CPU quantum-end/timer work needs it. Candidate design:
  arm each CPU's vtimer + banked PPI-27 enable from a KeIpiGenericCall broadcast worker
  at HalInitializeClockArm64 time (generic call runs on every CPU at IPI_LEVEL — the
  banked GICC/PPI registers REQUIRE running on the target CPU); gate KeUpdateSystemTime
  to Prcb.Number==0, others do KeUpdateRunTime. Then **M5d:** KiTbShootdown via SGI +
  `-smp 2` soak. (arm64 currently has NO cross-CPU TLB shootdown — KeFlushEntireTb et
  al are still UP-shaped; correctness has ridden on low cross-CPU VM churn so far.)

- **2026-07-14 (cont.) — M5c tail COMPLETE (AP vtimer, commit `503ba99aa99`) + M5d
  COMPLETE (TLB, commit `27f632fc0f1`): arm64 SMP is at FEATURE PARITY with amd64.**
  Evidence chain on 3/3 `-smp 2` boots (~47s, zero WAITDEP/BUGCHECK/DOUBLE-DEFER; UP
  regression green): `[SMP] CPU1 vtimer armed (banked PPI-27 + CNTV)` → `[SMP] CPU1
  clock tick live (per-CPU CNTV delivering)` → `[SMP] CPU1 scheduling live`.
  - **Vtimer design that landed (simpler than the generic-call candidate above):** APs
    arm LAZILY — HalClockInterruptArm64 arms this CPU's banked PPI-27 + CNTV on its
    first IRQ once TIMER_INTERVAL is published, and clock init kicks all APs with one
    SGI (`GICD_SGIR` target-filter 01 = all-but-self, no CPU enumeration, no new
    kernel→HAL imports). Self-healing off any later dispatch IPI if the kick is lost.
    Tick routing: CPU0 keeps KeUpdateSystemTime (sole wall-clock owner, NT rule); APs
    call KeUpdateRunTime — which turned out to be 100% per-PRCB-shaped already (thread
    time, DPC-rate, quantum-end via the M5b per-CPU softint state): zero kernel changes.
  - **M5d verdict: arm64 needs NO software shootdown protocol.** Full audit of every
    `tlbi` in the kernel: KeInvalidateTlbEntry (`vale1is`), KeFlushProcessTb /
    KeFlushEntireTb / KiSwapProcess (`vmalle1is`) are all INNER-SHAREABLE — the
    hardware (and QEMU MTTCG) broadcasts the invalidate to every core, and the
    trailing `dsb ish` completes only when all cores have dropped the entries. That IS
    the shootdown + ack. The amd64 0xE1 descriptor machinery has no arm64 twin BY
    DESIGN. KeFlushCurrentTb's non-IS `vmalle1` is correct per NT contract (current
    CPU only; callers audited: freeze + sysldr post-reloc, both local-scoped).
  - **Real latent bug fixed:** every broadcast flush site was missing the leading
    `dsb ishst` that publishes the caller's PTE store to REMOTE table walkers before
    the invalidate — a remote walk could refill from the stale PTE. TCG's strong
    memory model forgives it, real ARM hardware does not. Added everywhere (KiSwap-
    Process already had its dsb).
  - **Owed (perf, not correctness):** KiSwapProcess's `vmalle1is` full-flushes every
    core's TLB on each process switch (no ASIDs on this port yet — TTBR writes carry
    ASID 0). ASID-based switching would localize it. Roadmap Owed-work item.
  - **Overnight `-smp 2` soak launched** (farm, arm64+amd64, 7h, `VIBEOS_QEMU_SMP=2`
    env — farm has no --smp flag; the harness reads the env var directly, vm.py:974).
  **NEXT:** i386 M5 (APIC HAL — plan discussed 2026-07-14: PIC cannot do SMP at all,
  8259 has no IPIs; the Rust APIC HAL exists but is cfg-gated x86_64 — un-gate + add
  i386 halves + TPR IRQL as Phase A "i386 UP on APIC", then Phase B AP start reusing
  aptramp32 + ke/ipi.rs, landing the i386 idle-lock + ctxswitch SwapBusy in the same
  change). Or land the arm64 lane on master first (full ×3 gate → merge → push).

## Conformance-soak triage (2026-07-16) — amd64 `-smp 2` kmtest guest-load

  Working the bugs the 20-sweep kmtest guest-load soak surfaced (task #10; full writeup field-manual §9.88). Amplifier =
  a temporary MmMdl-dense sweep list (`kmtest-tests-amd64.txt` warmup + MmMdl ×25/boot;
  RESTORE from `scratchpad/kmtest-tests-amd64.txt.REAL` before landing) + a background
  boot/capture loop (`scratchpad/mmMdl_capture.sh` → `~/vibeos-flake-farm/mmMdl-evidence/`).

  - **MmMdl 0x1E — ROOT-CAUSED + FIXED (`ARM3/mdlsup.rs` `MmUnmapLockedPages`).** The
    kernel-path unmap carried an i386-bringup band-aid (commit `15434cd8dea`, "restore
    explorer.exe"): `if ByteCount > 16MB { return; }`. That FALSE-FIRED on kmtest MmMdl's
    legitimate **32MB** MDL maps, so unmap bailed WITHOUT `MiReleaseSystemPtes` — and,
    critically, without ZEROING the PTEs. `MmFreePagesFromMdl` runs right after, so the
    freed pages stayed mapped by **valid** system PTEs with live TLB entries → a
    use-after-free alias: on `-smp 2`, CPU1 reallocates a page CPU0 still aliases → the
    ~10% 0x1E. The same leak steadily starved `MmMapLockedPagesSpecifyCache`→NULL (the
    `fail=4` soft face: MmMdl.c:68/100/103/104, with `MappedSystemVa` left holding a stale
    VA). **Fix:** a precise use-after-free guard instead of a size cap — a genuinely
    system-mapped MDL can never span more PTEs than the whole pool, so bail only when
    `PageCount > MmTotalSystemPtes`, else release properly (zeroes the PTEs, closes the
    alias, returns the pool). `MiReleaseSystemPtes` bounds-checks StartingPte itself as a
    backstop. **Validated:** dense MmMdl ×25/boot went `fail=4`→`fail=0`, boot completes
    (`done total=61`). Crash-rate confirmation (0 across many exposed runs) + gate ×3 in
    progress before landing. (Residual `pass 569→563` across the 25× dense runs is benign
    system-PTE fragmentation from the unnatural repeat, not a leak.)
  - **KeIrql `fail 0→3` — NOT a kernel bug; kmtest build-config artifact.** amd64
    `KeRaiseIrqlToSynchLevel` is a `FORCEINLINE` (xdk/amd64/ke.h:117) with a *compile-time*
    `#ifdef CONFIG_SMP` → `KfRaiseIrql(12)` else `KfRaiseIrql(2)`. kmtest_drv is compiled
    WITHOUT CONFIG_SMP → returns 2, but runs on ntkrnlmp (Prcb->BuildType clears the
    UNIPROCESSOR bit) so `KmtIsMultiProcessorBuild=TRUE` and the test expects SYNCH=12
    (KeIrql.c:32,111-116). Our `KfRaiseIrql(12)` (CR8=12) is fine. Disposition: compile
    kmtest_drv with CONFIG_SMP for SMP conformance runs, or seed a separate SMP baseline
    (conformance-harness scope, not the kernel). KeEvent's `pass` drift is the same class
    (MP-build-detected sub-test expectations). Neither is a kernel change.
  - **0xF4 boot-phase (2/20 soak) — pre-existing §9.56-class, still OPEN.** Separate from
    MmMdl; not chased this pass.

## i386 M5 (Phase B) — AP-start bringup (2026-07-16, in progress)

Unblocked by the i386 APIC HAL landing as the default (Phase A). Working
`build --arch i386 --smp-kernel` + `smoke --smp N`. Progress this pass — three
root-caused i386 SMP bugs FIXED + LANDED (commit on `smp-bringup`; full writeup
field-manual **§9.91**):

1. **HAL SMP gating** keyed on `hal_apic`/`hal_pic` (not `target_arch`) so
   i386-on-APIC gets the real `smp_processor`/`trampoline` symbols + a new i386
   (2-level non-PAE) `identity_map_stub`. amd64 provably no-op.
2. **0x79 MISMATCHED_HAL** — swapped i386 `PRCB_BUILD_UNIPROCESSOR/DEBUG` consts
   (masked on UP, unmasked SMP). Fixed to canonical DEBUG=1/UNIPROCESSOR=2.
3. **KeStartAllProcessors GDT + loader-block corruption** — `KDESCRIPTOR` missing
   `Pad` (→ `sgdt` base misaligned → 0x1E in memcpy) + AP-handshake `offset(0/1/2)`
   clobbering `LoadOrderListHead`/`MemoryDescriptorListHead` (→ 0x50 + a masking
   module-list-walk cascade). Now named-field writes + `addr_of_mut!` matching
   `ke/amd64/mproc.rs`.

4. **Bug D (2026-07-16) — the M3 KeIpiGenericCall self-test worker was cdecl,
   not NTAPI.** `ex-rust/src/init.rs`'s `gc_selftest_worker` + the local
   `KeIpiGenericCall` `Function` param were `extern "C"`; `PKIPI_BROADCAST_WORKER`
   is `NTAPI` (stdcall on i386), so the stdcall call site left ESP 4 bytes low →
   rendezvous frame skew → `ret` onto the stack → deterministic `0x50` (EIP =
   stack VA `0xf7bbbba0`). NOT a `KeStartAllProcessors` bug (that returns clean —
   a GDB `watch` on its saved-return slot proved the frame intact). Invisible on
   amd64/arm64 (`extern "C"==extern "system"`). Fix = both → `extern "system"`.
   Full writeup: field-manual **§9.91 Bug D**.

**Boot state after ALL FOUR fixes (2026-07-16): i386 SMP kernel boots to a
fully-rendered desktop at BOTH `-smp 1` and `-smp 2`** — `_tWinMain entered` +
`shell ready`, 0 bugchecks in 3385 serial lines. At `-smp 2` the BSP starts CPU 1
(`HalStartNextProcessor` INIT-SIPI-SIPI on the first call) and the AP completes
the `KeLoaderBlock->Prcb` handshake in `KiSystemStartup` (the BSP has no timeout
on that spin, so boot-to-desktop *proves* the AP ran). The KeIpiGenericCall
self-test now prints `worker ran on 0x1 == active set OK`. **This is i386 M5
Phase B DONE — boot parity with amd64's M0–M3.**

**Phase B LANDED (2026-07-17):** merged origin/master (warnings-burndown lane, no SMP
validation) + re-verified `-smp 1`/`-smp 2` boot on the merged tree + gate ×3 green
(stamp `58d57943d8a7`) → pushed to origin `master` (`b095876cd3f`).

## i386 M5 Phase C — AP schedules (2026-07-17, in progress)

At `-smp 2` after Phase B the AP was *started + handshaked* but parked in a spin-stub
(`ke/i386/kiinit.rs` `KiSystemStartup`, `if Cpu != 0 { … loop { YieldProcessor() } }`),
so its `KeActiveProcessors` bit stayed 0 and it never reached the idle loop. Phase C
flows the AP through the full per-CPU init, exactly as ReactOS does (the i386
`KiInitializeKernel` is already AP-aware — `Number != 0` branches, sets Prcb
CurrentThread/IdleThread, `KiIdleSummary`, clears `LoaderBlock->Prcb` as the handshake).
Three changes:

1. **Remove the M5a spin-stub** so the AP runs `KiGetMachineBootPointers` →
   `Ki386InitializeTss` → `KiInitializePcr` → `KeInitExceptions` → freeze-lock →
   `KeActiveProcessors` publish → `HalInitializeProcessor` → `KiSwitchToBootStack` →
   `KiSystemStartupBootStack` → `KiInitializeKernel(AP)` → `KiIdleLoop`.
2. **Freeze-lock release drop — found by disassembly BEFORE building** (the reusable
   catch). `KiFreezeExecutionLock` (i386) is acquired in `KiSystemStartup`
   (`InterlockedBitTestAndSet` bit 0) but the compiled kernel *never writes it back to
   0* — `objdump -d ntkrnlmp.exe | grep 0x5f2014` shows only the acquire's `cmpl`/two
   address-pushes, no store-0. amd64 releases it explicitly (`InterlockedAnd64(&lock,0)`,
   `ke/amd64/../kiinit.c:566`); the i386 `KiSwitchToBootStack` asm translation (now a
   Rust rlib symbol that just switches ESP + jmps `KiSystemStartupBootStack`) dropped the
   upstream release. Latent while the AP parked *before* the acquire; a hard deadlock the
   moment the AP flows the full path (AP spins forever at the acquire → P0's
   `KeStartAllProcessors` handshake on `LoaderBlock->Prcb==0` never completes → whole
   boot hangs). **Fix:** atomic `store(0, Release)` right after `KeNumberProcessors += 1`,
   mirroring amd64's release point. (i386-only fatal; amd64/arm64 already release.)
3. **§9.56 idle-loop lock discipline** ported to `ke/i386/thrdini.rs` `KiIdleLoop`
   (was a lock-free consume + `_enable()`-first — inert until the AP scheduled). SMP now
   consumes `NextThread` under `KiAcquirePrcbLock` with IRQs disabled, re-checks, promotes,
   `rq_inflight_unmark`, releases, `sti`, `KfRaiseIrql(SYNCH)` for the swap — the idle
   thread is never queued ready (no `KxQueueReadyThread`), so the lock is released
   explicitly before `KiSwapContext`, same as amd64 `stubs.rs`. Plus a one-shot per-CPU
   `[SMP] CPU%u scheduling live` marker (Phase C visibility). No `KiTbLazyIdleEnter/Exit`
   on i386 — that's the separate i386-TLB-shootdown parity item.

**Bringup progress (2026-07-17, greenfield — ReactOS's i386 AP `KiInitializeKernel`
branch is a `/* FIXME */ "you are brave"` STUB, never implemented, so every step below
was untested).** Removing the spin-stub exposed a cascade; each root-caused via GDB /
QEMU `-d int` (full writeups in the field-manual candidate + `phasec-findings.md`). Five
fixes, each moving the AP strictly further:
4. **AP ran `ExpInitializeExecutive` at HIGH_LEVEL → 0xC2 BAD_POOL_CALLER** (Arg2=0x1f).
   `KiInitializeKernel`'s `KeLowerIrql(APC_LEVEL)` is inside `if Number==0`. Fix: fill the
   empty AP `else{}` with `KeLowerIrql(APC_LEVEL)` (mirrors arm64). Also unblocks the later
   `KeRaiseIrql(DISPATCH)` (can't raise-to-lower from HIGH_LEVEL).
5. **AP `HalInitSystem` deadlock** — AP's `ExpInitializeExecutive(Cpu!=0)` re-ran GLOBAL
   Phase-1 HAL init → `MmAllocateContiguousMemory`→`MmMapIoSpace`→`KeFlushEntireTb`→TLB-flush
   IPI to P0, but P0 spin-waits in the `KeStartAllProcessors` handshake with IPIs unserviced
   → deadlock. Fix: gate the `HalInitSystem` call on the explicit `Cpu` arg in `ex-rust`
   `ExpInitializeExecutive` (AP HAL is done by `HalInitializeProcessor`). **Reusable rule:
   the AP MUST issue no cross-CPU IPI during init while P0 is in the handshake.**
6. **AP idle-thread pointer** — `mproc.rs` set `LoaderBlock->Thread = &Prcb->IdleThread`
   (address of the pointer FIELD in the PRCB), not the real KTHREAD `&APInfo->Thread`;
   `KiInitializeKernel`→`KeInitializeThread` scribbled a KTHREAD onto the PRCB → garbage
   `CurrentThread` → `(*Thread).Priority=0` faulted → double-fault (0x7F/8). Fix: point at
   `&APInfo->Thread` (matches P0's `&KiInitialThread`).
7. **AP IDT unwired for HAL vectors** — the AP's first clock IPI (0xd2, from P0's M3
   self-test now reaching the active AP) hit `KiUnexpectedInterrupt210` → #PF@0x8048 → #DF.
   `mproc.rs` copies P0's LIVE, fully-wired IDT, but `KiSystemStartup` re-copies the bare
   `KiIdtDescriptor` template on top (HAL vectors → unexpected stubs) and the AP never
   re-runs HAL wiring. Fix: gate BOTH the recopy block AND `KeInitExceptions` (the P0-only,
   unguarded KiIdt Selector/ExtendedOffset flip that preps it) with `if Cpu == 0`.

**STATE: the i386 AP now boots through its COMPLETE init and idles healthy in
`KiIdleLoop`** (GDB `-smp 2`: CPU1 `[halted]` at `thrdini.rs:369`; CPU0 in the M3
`KeIpiGenericCall` self-test). This is the core Phase-C achievement — the AP-init path
works end-to-end.

**Bugs 6–8 (2026-07-17): the 0xE1 wiring + two latent MP bugs it uncovered — i386
`-smp 2` NOW BOOTS TO DESKTOP** (`[SMP] KeIpiGenericCall self-test: worker ran on 0x3` +
`[SMP] CPU1 scheduling live` + shell ready, ~38 s).

6. **Kernel IPI receive (0xE1) wired — the blocker.** i386 IDT[0xE1] was
   `KiUnexpectedInterrupt225` on both CPUs (amd64 wires 0xE1→`KiIpiInterrupt` in the
   KERNEL; i386 only ever had the send side). Fix exactly as planned: apictrap.S
   `TRAP_ENTRY HalpIpiInterrupt` (i386 half only!) → fastcall
   `HalpIpiInterruptHandler` (rtctimer.rs: `KiEnterInterruptTrap` →
   `HalBeginSystemInterrupt(IPI_LEVEL=29, 0xE1)` → `KiIpiServiceRoutine(TrapFrame,
   NULL)` → `KiEndInterrupt` + `KiEoiHelper`), wired in apic.rs
   `HalpInitializePICs` i386 block (`HalpVectorToIndex[0xE1]=RESERVED` +
   `register_interrupt_handler`). The AP inherits it via the mproc.rs live-IDT copy.
   **amd64 must NOT wire this** — its kernel owns IDT[0xE1] (descriptor transport in
   `ke/amd64/ipi.rs`); the HAL stub would clobber it. `KiIpiServiceRoutine` is an
   ntoskrnl.spec stdcall export (i386/arm/arm64), so the HAL links it directly.
7. **i386 `HalpClockIpiHandler` dropped `KiEndInterrupt`** (latent until an AP existed —
   only APs receive the clock IPI). The C (rtctimer.c:226) closes the non-spurious path
   with `KiEndInterrupt(Irql, TrapFrame)` = cli + `HalEndSystemInterrupt` (LAPIC EOI +
   IRQL restore) + `KiEoiHelper`; the Rust closed with bare `KiEoiHelper`, so the AP's
   FIRST clock IPI would leave soft-IRQL at CLOCK_LEVEL + an un-EOI'd ISR bit forever,
   blocking every class≤0xD vector (clock/DPC/APC) on that AP.
8. **i386 SwapBusy protocol was INVERTED — the post-wiring `-smp 2` 0x1E.** First run
   past the self-test bugchecked 0x1E (`STATUS_IN_PAGE_ERROR`) at
   `KiSwapContextExit+0x20` with OldThread = a STACK VA (0xf7a29c40): CPU1 resumed a
   migrating thread while CPU0 was still executing on its stack, CPU1's frame pushes
   scribbled CPU0's live locals (the affinity-bouncing SMP self-test thread made the
   migration window hot — `[RQAFF]` prints of the same thread flipping setmember 1↔2
   immediately precede the crash). Root: NT's MP protocol (2k3 i386 `SwapContext`) is
   (a) SPIN until `NewThread->SwapBusy == 0` before adopting its stack, (b) clear
   `OldThread->SwapBusy` only AFTER ESP leaves the old stack; our i386
   `KiSwapContextEntry` had NO spin and cleared the NEW thread's flag (a lazy self-heal
   that destroyed the guard). Fix: Acquire spin in `KiSwapContextEntry` (before
   `KiSwitchThreads`, interrupts enabled so IPIs stay serviceable), Release clear of
   `OldThread->SwapBusy` in `KiSwapContextExit` (after the DPC-active check, mirroring
   amd64 thrdini.rs:271), plus `KiSetThreadSwapBusy(OldThread)` in i386
   `KiDispatchInterrupt` before `KxQueueReadyThread` (NT does; without it the flag is
   never raised on the preempt path). **Cross-arch bonus: amd64
   `KiDpcInterruptHandler` had the SAME missing set-busy** (traphandler.rs — its
   ctxswitch.S spin existed but the flag was never raised on this path), a prime
   suspect for the open amd64 "kernel-stack teardown race" 0x1E (task-ledger Bug2).
   Fixed identically. Audit of every other queue-current site came back covered
   (wait paths, `KiQuantumEnd`, `NtYieldExecution` all call `KiSetThreadSwapBusy`;
   arm64 has the full protocol in ctxswitch.S:159 + thrdini.rs:225 and has no
   queue-current dispatch-interrupt path). The same audit closed §9.88 Bug 2
   (amd64 kernel-stack teardown race, task #10): `PspReapRoutine`/`PspDeleteThread`
   freed the dying thread's stack with NO wait for its final context swap —
   `KeTerminateThread` queues the reaper entry while still running on that stack.
   Fix: Acquire spin on `Tcb.SwapBusy` before `MmDeleteKernelStack` (all arches;
   field manual §9.94).

This also unblocks i386 TLB-shootdown-to-AP (same 0xE1 packet path — `KeFlushEntireTb`'s
`KiFlushTargetEntireTb` packets now execute on the AP; single-VA remote flush remains the
M-wave-2 parity item shared with amd64).

**Phase C VALIDATION (2026-07-17, landed `b19871654ed`):** i386 `-smp 2` 19/19 boots to
desktop (~40 s, up to 4 parallel) + `-smp 1` green; amd64 `-smp 2` 2/2 + net; arm64
`-smp 2` 2/2 (reaper spin is cross-arch); full ×3 gate green.  **§9.88 Bug 2 closure
evidence:** `conformance --arch amd64 --smp 2` sweep ran to completion — 86/87 tests,
**MmMdl pass=575 fail=0, zero 0x1E** (was: boots died ~33–50% under this exact load).
NEW SMP-vs-UP-baseline deltas from that sweep (wave-2 ledger, not boot-affecting):
KeGuardedMutex TIMEOUT, KeSpinLock fail 2→31, KeIrql fail 0→3 — real MP semantic gaps
to burn down when the SMP conformance baseline lane opens.

GDB recipe that cracked Bug D (banked for Phase C): **`hbreak`** the suspect fn
(a plain `break` at `--gdb-wait` reset is clobbered when freeldr copies the kernel
over the `0xCC`), `add-symbol-file build-i386/ntoskrnl/ntkrnlmp/ntkrnlmp.exe
0x804c4000` (kbase-stable `0x804c3000`; symbolicate ntkrnlmp, NOT ntoskrnl — the
toolkit's `find-kbase`/`addr2sym` default to the UP binary), and **read the gdb
port from `qemu/runs/<run-id>/manifest.json` (`gdb_port`), never hardcode 12001**
— auto-alloc drifts across runs and a stale QEMU on the assumed port silently
eats your breakpoints.

**Wave-2 burn-down 1/3 (2026-07-17): KeGuardedMutex TIMEOUT ROOT-CAUSED + FIXED —
translation-dropped `#ifdef CONFIG_SMP` deferred-ready drain in `KiRetireDpcList`**
(field manual §9.95). An idle CPU re-entered `hlt` with runnable threads parked on its
own `DeferredReadyListHead` — the idle loops check that list but rely on the retire
loop's drain (the dropped block) to empty it; timer-driven wakes on an idle CPU
stranded ~60s each until an unrelated cross-CPU wake drained them as a side effect.
The census watchdog cracked it (owner `state=7` persisting across scans) + a live GDB
catch (CPU1 halted, `DeferredHead` loaded, ReadySummary=0). Fix: restore the C block
in `KiRetireDpcList_body` + the same drain in arm64's `KiDispatchSoftwareIntArm64`
(both `cfg(smp)`; UP untouched). Validated: amd64 `-smp 2` conformance **87/87 ok,
0 timeouts** (was 86/1), **IoDeviceObject delta also recovered**, sweep wall-clock
**679s→348s** (the strands taxed every test's wakeups); i386 `-smp 2` smoke 3/3;
arm64 `-smp 2` smoke 2/2. REMAINING wave-2 conformance deltas (real MP semantic
gaps, next up): KeSpinLock pass 321→292/fail 2→31 (queued-lock handle `Lock`/`Next`
not cleared MP-style — residues are a global lock pointer at byte-shifted offsets —
plus `KmtIsMultiProcessorBuild` try-acquire semantics), KeIrql 131→128/fail 3,
KeEvent pass dip 42733→42601.

**Wave-2 burn-down 2/3 + 3/3 (2026-07-17): KeIrql + KeSpinLock + KeEvent deltas ALL
CLOSED** (field manual §9.97). Three distinct roots, none of them the guessed
"queued-lock handle not cleared MP-style":
- **KeIrql fail 3 → 0**: SYNCH_LEVEL's UP/MP flavor-variance was frozen at the UP
  value in FIVE translation sites — the amd64 `ke_rust_shims.c` CR8 shim (hardcoded
  DISPATCH), hal-rust's apic/pic/spinlock consts (i386 exports), the xdk amd64
  `KeRaiseIrqlToSynchLevel` FORCEINLINE (baked `2` into every driver binary incl.
  kmtest — no kernel fix could reach it), and two stray i386 kernel constants (28 =
  pre-WS03 value in irqobj; 0x0C = the **amd64** value at the thrdini context-switch
  raise). Fixed: kernel exports `KiSynchIrql` (per-flavor via `feature="smp"`) + the
  shim reads it; xdk header switched to `NTKERNELAPI` import + `-arch=x86_64` spec
  export (matching real NT amd64); hal-rust sites all take `buildtype::SYNCH_LEVEL`;
  i386 strays aligned to 27. UP kernels still raise 2 by construction.
- **KeSpinLock fail 31 → 1**: the storm was kmtest's OWN latent bug — `CHECK_DATA`
  unioned the `Queue` pointer with the `Irql` byte, so queued-acquire wrappers
  stomped the pointer's low byte and `CheckSpinLockQueue` dereferenced the garble
  ("byte-shifted MmPfnLock" reads = misaligned neighbor entries; fail count varied
  31↔1 across identical binaries). Latent on UP only because the garble happened to
  read zeroed memory — validation showed `KiInitSystem` initializes LockQueue on UP
  kernels too (faithful to C/real NT), so the test's NULL expectation was never
  checkable on any flavor. Fixed the union + made the check a universal
  captured-compare (`Lock & ~7 == captured initial`, `Next == NULL`, no flavor
  branch). Remaining 1 fail = the known DUMMY_PSEH checked-build divergence
  (no `KTHREAD|1` owner store) that UP also carries (2 fails there).
- **KeEvent pass dip**: legitimate — the test skips an MP-fatal section by design.
  Handled at the harness: `conformance --smp N` now ratchets against
  `tools/conformance-baseline/<arch>/kmtest-smp<N>.txt` (seeded via
  `--smp N --write-baseline`).
- **BONUS build-system fix**: `sdk/include/xdk/CMakeLists.txt` GLOB → GLOB_RECURSE —
  per-arch fragment edits (amd64/ke.h!) were NOT dependencies of the `native-hpp`
  header generation, so wdm.h stayed silently stale (bit this lane: first validation
  ran against a Jul-10 wdm.h).
Validated: amd64 `-smp 2` full sweep **87/87 ok, 258s**, `kmtest-smp2.txt` baseline
seeded; KeIrql **131/0** (UP parity), KeSpinLock **322/1** (beats UP's 321/2),
KeGuardedMutex 1566/87 (exact UP parity), IoInterrupt 1095/0 (UP: 939/156 — MP
kernel outright better); the only SMP-worse entry is a known IoDeviceObject ±1
flicker (184/5 vs 185/4), baselined. i386 `-smp 2` smoke 3/3 with the SYNCH=27
context-switch raise. arm64 untouched behaviorally (halstubs still DISPATCH — its
SYNCH alignment is OWED for when arm64 -smp conformance opens; ndk arm64 defines no
MP SYNCH variant, real NT arm64 uses IPI-2=12).

**Wave-2 TLB item CLOSED + i386 SMP conformance lane OPENED (2026-07-18, overnight):
i386 single-VA remote flush landed over the packet path — and the first-ever i386
`-smp 2` kmtest sweep flushed out three protocol bugs (field manual §9.98).**
The owed "targeted single-VA remote flush + packet mailbox (i386+amd64)" item: amd64
half had landed 2026-07-11 (descriptor transport); the i386 half is now in —
same-named remote-only `KiTbShootdownSingle`/`KiTbShootdownBroadcast` in
`ke/i386/cpu.rs` riding the validated `KiIpiSendPacket` 0xE1 path (raise-to-SYNCH →
send → `KiIpiStallOnPacketTargets` drain → lower), dispatched from the
`KeInvalidateTlbEntry`/`KeFlushProcessTb` inlines (mm.rs cfg widened
amd64-only → any(x86); arm64 stays hardware-broadcast TLBI by design). The "packet
mailbox" wording resolves to the amd64 per-CPU-mailbox upgrade that was deliberately
NOT integrated (staged reference in ~/vibeos-rust-staging/smp/ipi-recv/) — the
descriptor protocol covers it; item retired.
Found + fixed en route (§9.99 for the full stories): (1) `KeFlushEntireTb`'s
"wait for IPI completion" block was EMPTY — sender packet-slot reuse race, now a
real drain; (2) `IPI_FREEZE` corrupted the `IpiFrozen` state machine (bit-OR over a
staged state) and `KiIpiServiceRoutine` had no freeze branch — the
KdSystemDebugControl `-smp 2` instant wedge; now state-dispatched to
`KiProcessorFreezeHandler` (kd freeze/thaw WORKS on i386 SMP); (3) the §9.56
double-defer oracle's unmark was missing at i386 `KiDispatchInterrupt`'s standby
promote — eternal `[RQDEFER2]` spam on every i386 SMP boot, a material slowdown
(the pre-fix sweep attempt: 25 tests/77 min, wedged; post-fix: **72 launched /
68 ok / done in 99s**, within 15% of the UP control's 86s — so NO lazy-idle port
needed on i386 for now). Also cleared four pre-existing landmines on the
first-ever i386 rostests build + pointed the conformance error at the
`ENABLE_ROSTESTS_KMTESTS=ON` scalpel (full ROSTESTS drags the unported apitest
estate in).
**Validation:** i386 `-smp 2` boots 5/5; i386 smp2 sweep 99s done, KdSystemDebugControl
40/2 = exact UP parity, `kmtest-smp2.txt` baseline SEEDED — MP is at-or-better than
UP everywhere (ExUuid 3000/0 vs 2000/1000, IoInterrupt 2145/0 vs 1839/306,
FsRtlLegal 2504/24 vs 2496/32; KeEvent 42601 = the same legitimate MP-skip as
amd64; KeSpinLock 697/88 and IoDeviceObject ±1 = known classes). i386 UP control
sweep: ratchet holds, no regressions. amd64 post-change: `-smp 2` 2/2, `-smp 4`
2/2. Separately: the 2026-07-14 `-smp 4` 0x4C c0000139 residual NO LONGER
REPRODUCES (6/6 desktop) — killed by the intervening corruption fixes; task #14
closed.
