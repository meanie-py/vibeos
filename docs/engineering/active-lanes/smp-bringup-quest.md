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
Found + fixed en route (§9.101 for the full stories): (1) `KeFlushEntireTb`'s
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

## UP→smp1 collapse — the gating experiment (2026-07-24)

Re-opening the "UP endgame" item above (decided 2026-07-10, never scheduled). The
question the collapse turns on is narrow: **does the MP kernel on ONE cpu behave
like the UP kernel?** If yes, `ntoskrnl.exe` becomes the MP kernel, the UP flavor
retires, and the "worked-on-UP-broke-on-SMP" bug class stops existing.

### Harness capability landed first (no default changed)

Three false-green shapes had to be closed before any measurement could be trusted
— an SMP run that isn't actually SMP looks *identical* to one that is, screenshot
included:

1. **`conformance --smp 1` was a no-op experiment.** `--smp-kernel` was appended
   only when `N > 1`, so `--smp 1` swept the **UP** kernel and reported a
   meaningless all-green. Now any `--smp N` builds MP media; `--smp 1`
   deliberately ratchets against the **UP baseline** (`kmtest.txt`) because
   MP-at-one-cpu matching UP test-for-test IS the collapse criterion.
   `--smp 1 --write-baseline` is refused — it would overwrite the UP oracle with
   MP results and destroy the only thing the experiment is measured against.
2. **`smoke --smp N` never checked which kernel booted.** freeldr already
   announces it (`[FLDR] load …\ntkrnlmp.exe`), so this is an assertion, not new
   instrumentation → `UP_KERNEL_ON_SMP_RUN`. Verified by running it against UP
   media: a boot that reached the desktop and would have been reported GREEN now
   fails with the actionable fix line.
3. **`smoke --smp N` had no AP-liveness check** — `test_smoke.py::test_milestones`
   requires `[SMP] CPU<n> scheduling live` per AP, but the CLI smoke fast path
   does not run pytest, so `smoke --smp 2` greened on the BSP's desktop alone.
   Mirrored into the CLI path (`SMP_AP_NOT_SCHEDULING`), plus explicit detection
   of the arm64 **MP-kernel-on-UP-hal.dll** trap (`SMP_UP_HAL`) — arm64 has no
   MISMATCHED_HAL handshake, so that mis-pair boots to a desktop on 1 cpu and
   looks perfect. All three live in `_smp_check()` (tests/qemu/cli/smoke.py).

### Result: MP kernel at `-smp 1` boots to desktop on all three arches

Same host, same session, UP numbers from the immediately-preceding gate:

| arch  | UP    | MP @ `-smp 1` | note |
|-------|-------|---------------|------|
| i386  | 37.2s | **33.5s**     | `[SMP] KeIpiGenericCall self-test: worker ran on 0x1 == active set OK` |
| amd64 | 38.6s | **~40s** (86.9s incl. the net suite, which also passed) | |
| arm64 | 44.2s | **46.0s**     | no `UP HAL stub` marker — real SMP HAL |

Each run is kernel-verified (`[FLDR] load …\ntkrnlmp.exe`). **No boot-time
regression from running the MP kernel on one cpu** — the real spinlock bodies and
the SYNCH_LEVEL raise on every context switch are inside the noise. That answers
the "UP might be faster" objection at the boot level; conformance is the semantic
half.

### Structural blocker found: the media cannot carry both flavors

`[LiveCD_SMP]` (`/KERNEL=ntkrnlmp.exe`) **already ships in every livecd.ini** —
`VIBEOS_SMP_DEFAULT_BOOT` only rewrites `DefaultOS=`. And `ntkrnlmp.exe` is in the
livecd file lists unconditionally, so **every default gate build already builds,
links and stages the MP kernel on all three arches**; it is only never *booted*.
What is NOT interchangeable is the HAL: `hal/halx86/CMakeLists.txt` says it
outright — "There is exactly ONE hal.dll (the halacpi/halapic/halaacpi names are
install aliases)" — and its UP/MP flavor is a build-dir-global cmake cache var
(`VIBEOS_SMP_DEFAULT_BOOT` → `VIBEOS_SMP_HAL=1` → hal-cargo `smp` feature →
`HalpBuildType` + `SYNCH_LEVEL`). So switching kernels costs a full reconfigure +
relink (measured: **9m25s i386 / 12m22s amd64, warm**), which is what makes an
MP gate leg expensive today.

**Proposed fix, symmetric with what the kernel already does:** build the SMP HAL
as a second output (`halmp-cargo` → `halmp.dll`, mirroring `ntoskrnl-cargo` →
`ntkrnlmp-cargo`) and ship both. freeldr already supports the override —
`winldr.c:719` parses `/HAL=`, and `[LiveCD_Macpi]` already uses
`/HAL=halmacpi.dll /KERNEL=ntkrnlmp.exe`. Then ONE media boots either flavor, an
MP gate leg costs one extra boot instead of a rebuild, and the mis-pair class
(0x79 on x86, silent-1-cpu on arm64) stops being reachable by construction.

### …and DONE (2026-07-25). Both flavors now ship; `/HAL=` picks one per boot

Built as proposed, user-approved as *"a separate entry, but not default until the
rest of the bugs are fixed"* — so `DefaultOS` is untouched and every default boot
is still the UP pair.

**Why the loader needed nothing.** `winldr.c:780` is
`LoadModule(LoaderBlock, DirPath, HalFileName, "hal.dll", LoaderHalCode, …)` —
the 4th argument is the name the module is *registered* under in the loader
block. So freeldr loads whatever `/HAL=` names and every import still resolves
against `hal.dll`. This is not new machinery; it is the mechanism
`[LiveCD_Aacpi]`'s `/HAL=halaacpi.dll` has always used. The one serial line that
reveals which file was actually loaded is freeldr's own
`[FLDR] load \reactos\system32\halmp.dll` stamp — nothing downstream of the
loader can tell you the flavor, which is exactly why the smoke check below reads
that line.

**`halmp.dll` is the upstream name, and txtsetup.sif was already asking for it.**
`boot/bootdata/txtsetup.sif` `[Files.x64_mp]` has always mapped
`halmp.dll → hal.dll` for an MP install; the file simply never existed. The
i386 `[Files.acpi_mp]` equivalent names `halmacpi.dll` (a *different* flavor
axis — ACPI, not just MP), so it is untouched here.

**One recipe, two cells — the flavors cannot drift.** The whole of
`hal-cargo/build.rs` moved verbatim into a new `sdk/lib/rust/hal-link` crate
parameterized by `Flavor::{Up,Mp}`; both cells' `build.rs` are now three lines
over it, and `halmp-cargo` even shares `hal-cargo`'s `src/main.rs` (a GC anchor
for `HalInitSystem` + the loud panic handler) via `[[bin]] path`. This was the
whole point of the exercise: two hand-maintained link recipes for binaries that
must be identical apart from one cargo feature is the same shape as the swapped
`PRCB_BUILD_*` constants below — a single-copy divergence that no name-based
check can see. The only genuine duplicate is `i686-kernel.json`, and that is
forced: cmake builds the i386 `--target` path as
`${workspace}/${crate}/i686-kernel.json`, so every one of the 42 cargo cells
carries its own copy already.

`reactos-link` keys its inner staticlib target dir on `CARGO_PKG_NAME`, so the
two builds of `hal-rust` (with and without `smp`) land in
`target/hal-cargo-hal_rust-rustlib` and `target/halmp-cargo-hal_rust-rustlib` and
never thrash each other's cache — the same isolation `ntoskrnl-cargo` and
`ntkrnlmp-cargo` already rely on for `kernel-rust`.

**What changed, in full:**

| file | change |
|---|---|
| `sdk/lib/rust/hal-link/` | NEW — the shared link recipe, `Flavor::{Up,Mp}` |
| `sdk/lib/rust/halmp-cargo/` | NEW — 3-line `build.rs`, shared `main.rs`, own target json |
| `hal/halx86`, `hal/halarm64` CMakeLists | second `add_cargo_kernel_dll(halmp …)`; no `spec2def` (identical export surface, `.def` self-hosted) |
| `boot/bootdata/livecd.ini` | `[LiveCD_SMP]` += `/HAL=halmp.dll` |
| `qemu/mkramdisk.sh` | `--smp` stages `halmp.dll` and injects `/HAL=` (arm64 stages a curated set, so the twin must be named as well as built) |
| `sdk/cmake/gcc.cmake` | `VIBEOS_SMP_DEFAULT_BOOT → VIBEOS_SMP_HAL` RETIRED; `VIBEOS_PIC_HAL` now reaches **both** cells |
| `tests/qemu/cli/smoke.py` | `_smp_check` asserts the loaded HAL *name* → `SMP_UP_HAL`, all three arches |
| `archcfg.py`, `cargo_orchestrate.py` | `halmp` joins the arm64 optional targets + the cell list |

**Two footguns closed, not one.** The obvious win is cost: `--smp-kernel` now
rewrites `DefaultOS=` and nothing else, so flipping flavors no longer relinks the
HAL. The subtler one is that `VIBEOS_SMP_HAL=1` used to produce a file named
`hal.dll` containing an MP HAL — a binary that passes every name-based check
while being the wrong thing. The flavor is now the file name, which is also what
let the smoke check become a positive assertion (`loaded halmp.dll`) instead of
the old negative one (the `UP HAL stub called` runtime marker, kept as a backstop
for a UP body wearing an MP name).

**Cost accepted:** every build now links `hal-rust` twice on every arch, whether
or not the MP entry is booted. That is the same bargain `ntkrnlmp.exe` has always
had, and it is the price of one media carrying both.

#### Validation

| arch | `hal.dll` | `halmp.dll` | `--smp 2` boot | freeldr loaded |
|---|---|---|---|---|
| i386 | 2,744,320 | 2,756,608 | green, 38s | `halmp.dll` + `ntkrnlmp.exe` |
| amd64 | 868,352 | 880,640 | green + net, 1m53s | `halmp.dll` + `ntkrnlmp.exe` |
| arm64 | 389,120 | 401,408 | green, 51s | `halmp.dll` + `ntkrnlmp.exe` (ramdisk) |

Every arch's MP binary is the larger one, which is the right direction (real
spinlock bodies, the SMP paths). amd64 `--smp 1` is green + net too, and the UP
default is untouched — a plain smoke still reports `[FLDR] load …\hal.dll` +
`…\ntoskrnl.exe`. Both x86 arches show
`KeIpiGenericCall self-test: worker ran on 0x3 == active set OK`; arm64 adds
`PSCI CPU_ON mpidr=…1`, `CPU 1 online`, `CPU1 clock tick live`, and
`CPU1 scheduling live`. arm64's `opt:halmp` ninja target and the mkramdisk
staging both report green in the build summary, so the twin is built AND staged
on the arch that has no handshake to catch a mis-pair.

The two PEs export the **identical 57 names** (parsed out of the export
directories directly), which is the check that matters: `halmp.dll` is a drop-in
for `hal.dll` by construction, not by inspection.

**The relink really is gone, proven by mtime rather than by stopwatch.** After a
plain build followed by a `--smp-kernel` build, `hal.dll` and `halmp.dll` still
carry the *earlier* build's timestamps — the flavor flip relinked neither. The
`--smp-kernel` build ran 34 ninja steps: the two kernel links and the ISO.

**But "an MP gate leg costs one extra boot" — the claim made when this was
proposed — is only HALF delivered, and the earlier note above overstated it.**
The 9-12 minute HAL relink is gone. What remains is a cmake reconfigure + ISO
repack (**8m30s measured on amd64**, dominated by the ~44 lints and mkisofs, not
by compilation), because *which entry boots* is `DefaultOS=` inside the single
generated `freeldr.ini` on the media. One media, one `freeldr.ini`, one default.
So a gate cannot hold UP and MP media simultaneously today, and
`vibeos-cli gate` is deliberately UNTOUCHED here.

#### …and the second media turned out to be unnecessary. Patch the ini IN the ISO.

The paragraph above (written hours earlier) proposed building a second
`livecd-smp.iso` to escape the repack. **That was overbuilt.** The question
"why can't it just be a freeldr flag on the same ISO?" has a better answer than
either option, and the answer is measured:

ISO9660 stores each file's bytes **once, contiguously, with no per-file
checksum**, and `freeldr.ini` is plain text. So flipping the default entry is an
**equal-length in-place write of one line** into a copy of the image:

```
occurrences of "DefaultOS=" in livecd.iso : exactly 1   (0x1a800e, plain text)
DefaultOS=LiveCD_Debug    -> 22 bytes
DefaultOS=LiveCD_SMP\r\n  -> 22 bytes   (the spare \r\n becomes a blank line)
cp of the 221 MB ISO      -> 0.153s warm
```

`vm.py:smp_selected_iso()` does exactly that into the per-run work dir the
harness already stages media through, so `--smp N` boots the MP pair off
**unmodified UP-default media**. Every failure path RAISES — not exactly one
`DefaultOS=`, no `[LiveCD_SMP]` section, a value too short to overwrite, or a
1-byte remainder that cannot terminate the line. A silent fallback here would
manufacture precisely the false-green `_smp_check` exists to catch.

**Boot-proven end to end (amd64, 2026-07-25):** with `build-amd64/livecd.iso` on
disk still reading `DefaultOS=LiveCD_Debug`, `smoke --arch amd64 --smp 2` gave
`[FLDR] load …\halmp.dll` + `[FLDR] load …\ntkrnlmp.exe`,
`[SMP] CPU1 scheduling live`, boot + net green in 2m26s. No rebuild, no
reconfigure, no second media.

So **an MP gate leg now genuinely costs one extra boot** — the claim originally
made for the halmp.dll twin, walked back when the `DefaultOS=` repack surfaced,
and now backed by measurement. What made it possible was still the twin: both
HALs have to EXIST on the media before selecting between them can be a 22-byte
poke.

arm64 never had this problem — `mkramdisk` `mcopy`s the modified ini into
`boot.img`, no repack — so this is purely an x86 ISO-packing artifact.

**Pre-existing cruft noticed, not touched:** `[LiveCD_Macpi]` passes
`/HAL=halmacpi.dll`, which this tree has never built — that menu entry has always
been dead. Harmless (`TimeOut=0`, so automated boots never see the menu), and
`halmacpi` denotes the ACPI-MP flavor, a different axis from ours, so aliasing it
to `halmp.dll` would be wrong rather than helpful.

### Conformance at `-smp 1`: x86 PASSES, arm64 has a REAL blocker

| arch  | result |
|-------|--------|
| i386  | 74 launched / 70 ok. Only delta vs the UP ratchet: **KeEvent 42733→42601** = the documented MP-skip (§9.97), exactly the value in `kmtest-smp2.txt`. Plus 2 NEW passes (RtlStack, RtlStackKM — absent from the UP baseline entirely). |
| amd64 | 87 launched / **87 ok, 0 bad**. Deltas: KeEvent (same MP-skip), IoDeviceObject ±1 (known flicker, already in the smp2 baseline), and **ExHardError 30→24 / 0→6**. |
| arm64 | **BUGCHECK 0x9 `IRQL_NOT_GREATER_OR_EQUAL` at the start of the KeSpinLock kmtest** — 34 tests in, sweep never completes. |

**ExHardError is NOT ours — control-proven.** A UP-kernel sweep on the same tree reports the
identical `pass 30 → 24 / fail 0 → 6`, so it regressed on master since the baselines were seeded
(2026-07-17) and is orthogonal to the MP kernel. Registered in the roadmap's known-bugs registry.
Subtracting it: **MP-at-one-cpu differs from today's UP kernel by exactly the two already-baselined
MP-build deltas on both x86 arches.** That is the collapse criterion met on i386 + amd64.

**arm64 0x9 — root-caused statically; it is the OWED arm64 SYNCH alignment coming due.**
Bugcheck args `(lock, 0, 0, 0)` — arg2 is `KeGetCurrentIrql()` = **PASSIVE**, i.e. an
`…AtDpcLevel` lock routine ran at IRQL 0. Three facts pin it:

| source | arm64 `SYNCH_LEVEL` |
|---|---|
| `sdk/include/ndk/arm64/ketypes.h:29` — what **kmtest_drv compiles against** | `IPI_LEVEL - 2` = **12** |
| our kernel `ke/ke_defs.rs:267` (the `aarch64` + `feature="smp"` arm) | **2** |
| our arm64 HAL `hal-rust/src/arm64/halstubs.rs:23` | **2** (`DISPATCH_LEVEL`) |

`KeSpinLock.c:384` is `SynchIrql = KmtIsMultiProcessorBuild ? IPI_LEVEL - 2 : DISPATCH_LEVEL`, so
the test asks for 12 the moment it detects an MP build, while every raise-to-synch in our arm64
kernel and HAL goes to 2 — and the several `…AtDpcLevel` IRQL asserts in `ke/spinlock.rs` are
`#[cfg(any(debug_assertions, feature = "smp"))]` / `#[cfg(feature = "smp")]`, so they only *exist*
in the MP kernel. Hence: MP-only, arm64-only, and it fires before the test emits a single result.

**Correct the earlier note in this doc** ("ndk arm64 defines no MP SYNCH variant"): `ketypes.h:27-31`
carries the same `#ifndef CONFIG_SMP` pair as i386 and amd64 — UP `DISPATCH_LEVEL`, MP
`IPI_LEVEL - 2`. All three arches vary. Our arm64 value (2) was right for UP and wrong for MP, and
`ke_defs.rs:258`'s comment ("unchanged on arm64") encoded the wrong premise. *(A second correction:
the paragraph above this one, written the same day, claimed the header has no UP variant at all —
also wrong. Read the header, twice.)*

### …and the 0x9 was NOT the SYNCH bug. The BuildType constants were SWAPPED. (2026-07-25)

The SYNCH misalignment above is real and is fixed below, but it was **not** what bugchecked. Under
instrumentation (a temporary `spindiag` naming each of the 14 anonymous `0x9` sites — the banner
carries no PC) the trip was:

```
[SPINDIAG] KeAcquireSpinLockAtDpcLevel irql=0 lock=0000000000000000 buildtype=2
```

`buildtype=2` on a kernel that was demonstrably running SMP (PSCI markers, `CPU 1 online`). The
cause, `ke/arm64/kiinit.rs:55-56`:

```rust
const PRCB_BUILD_UNIPROCESSOR: USHORT = 1;   // ndk says 2
const PRCB_BUILD_DEBUG:        USHORT = 2;   // ndk says 1
```

**The two constants were swapped**, against `ndk/arm64/ketypes.h:46-47` — where DEBUG=1 and
UNIPROCESSOR=2 on *all three* arches. i386 and amd64 both had them right. So every arm64 boot ever
published a `BuildType` meaning the opposite of what the source says it wrote:

| kernel | wrote | published | kmtest read it as |
|---|---|---|---|
| arm64 UP  | `\|= UNIPROCESSOR`(1) | **1** | checked **MP** build |
| arm64 MP  | (nothing)             | **0** | free **MP** build |
| after fix, UP | `\|= UNIPROCESSOR`(2) `\|= DEBUG`(1) | **3** | checked UP ✓ |
| after fix, MP | `\|= DEBUG`(1) | **1** | checked MP ✓ |

Compounding it, arm64 gated `PRCB_BUILD_DEBUG` on `#[cfg(debug_assertions)]` — **off in
`--release`, which is how the kernel is built.** i386 and amd64 both set it unconditionally and
each carries a comment saying exactly that; arm64 never got the fix.

The chain to the bugcheck: `BuildType` with no DEBUG bit ⇒ `KmtIsCheckedBuild == FALSE` ⇒
`KeSpinLock.c:312`'s `(OriginalIrql >= DISPATCH_LEVEL || !KmtIsCheckedBuild)` opens the
`AcquireNoRaise` block at **PASSIVE**, which is legal on a free build — real NT compiles those
`ASSERT`s out. Our `KeAcquireSpinLockAtDpcLevel` (`spinlock.rs:118`) asserts **ungated**, so it
bugchecks. It was never MP-specific: the arm64 **UP** kernel would have done the same, and the only
reason we hadn't seen it is that nobody had run an arm64 UP kmtest sweep.

**Why the lint didn't catch it.** `scan_const_contradictions` finds one name with two values, but
buckets **by arch** — and `PRCB_BUILD_DEBUG` is 1/1/2 across i386/amd64/arm64, so no single bucket
disagrees with itself. A constant that is arch-**invariant** in the C headers yet wrong in exactly
one arch's bucket is structurally invisible to it. That is the gap; see the field manual entry.

**Fix (all landed together, each validated):**

1. `ke/arm64/kiinit.rs` — un-swap the constants; set `PRCB_BUILD_DEBUG` unconditionally.
2. arm64 `SYNCH_LEVEL` 2 → 12 under `smp`, at all three definition sites (`ntkrnl-sys/inline/ke.rs`,
   `ke/ke_defs.rs`, `hal-rust/arm64/halstubs.rs`). 12 lands in the gap the arm64 IRQL↔GIC map
   already leaves free — above every DIRQL (max 11), below `CLOCK_LEVEL`(13) — so a SYNCH section
   masks device lines but never the scheduler tick or an IPI. `gic_priority(12)=0x30` vs clock
   `0x20` and IPI `0x10`. The cost is that a SYNCH raise/lower now leaves the no-MMIO software band
   (two GIC PMR writes per pair) — correct, and the price of IRQL being hardware-real.
3. `ke/arm64/thrdini.rs` — the idle loop now raises to SYNCH around `KiSwapContext` in both `smp`
   branches, mirroring amd64. While SYNCH read DISPATCH the raise was a no-op, so it had been
   skipped.
4. `ke/cswitchdep.rs` — taps 1 and 2 compared the switch IRQL against `DISPATCH` outright, right
   only by coincidence on UP; on **any** MP kernel every swap would have reported a violation. Now
   `SWITCH_LEVEL` (= `SYNCH_LEVEL`). Not arm64-specific.

**No `KiSynchIrql` export is needed for arm64** (the earlier fix list said otherwise):
`ke_rust_shims.c` gates that shim `#elif defined(_M_AMD64)`, i386 goes through the HAL IAT, and
arm64's `KeRaiseIrqlToSynchLevel` is a HAL export in `halstubs.rs`. Likewise no arm64 HAL build-type
handshake exists to break — `buildtype.rs`/`MISMATCHED_HAL` is x86-only.

**Result: arm64 MP boots to desktop green at both `-smp 1` and `-smp 2`, zero assert trips.**

#### Conformance: arm64 now meets the collapse criterion, and UP got measurably better

The sweep completes for the first time on an arm64 MP kernel — `launched=86 ok=83 sweep=done` in
~70s, where it used to BUGCHECK 34 tests in. Run it on the UP kernel and you get **the same
86/83**. Test-for-test, MP-at-one-CPU differs from UP by exactly three lines:

| test | UP | MP@1cpu | what it is |
|---|---|---|---|
| `KeEvent` | 42733 | 42601 | the documented MP-skip (§9.97) — **the same value i386 and amd64 report** |
| `IoDeviceObject` | 181/8 | 180/9 | the known ±1 flicker, already in the amd64 smp2 baseline |
| `KeSpinLock` | 281/42 | **286/37** | MP passes *five more* — the `SynchIrql` rows working now |

That is the same shape x86 showed, so **the collapse criterion is met on all three arches.**
`ZwWaitForMultipleObjects` tripped `NO-END` (`0xC0000005`) on one of three sweeps and passed on the
others — it is a known member of the hostile-user-pointer / `memcpy+0x40` cluster
(`conformance-ratchet-quest.md`), not a new MP effect. A fourth run under a saturated host (another
lane's 3-arch gate) launched only 76 of 86 with widespread timeouts: **conformance numbers taken
while the host is busy are not comparable** — check `vibeos-cli ps` first.

The BuildType fix also repaired the **UP** kernel, because the old arm64 UP baseline had been
recorded by a kernel telling kmtest it was MP. Re-seeded (`--write-baseline`), improvements only:

- `KeIrql` 125/3 → **128/0** — every failure was a `SynchIrql` expectation the UP kernel could
  never satisfy while claiming to be MP.
- `KeSpinLock` 278/45 → 281/42, `MmSection` 291/40 → 292/39, `IoDeviceObject` 180/9 → 181/8.
- `KeEvent` 42601 → 42733 — the baseline had been storing the *MP* value for the UP kernel.
- Four tests join the baseline: `ExFastMutex`, `ExHardError`, `SeInheritance`, `RtlStack`.

**Bonus datum for the `ExHardError` bug (roadmap registry / task #18):** arm64 reports **24/6** —
the identical split amd64 regressed to (30/0 → 24/6). The same six subtests fail on both arches, so
that regression is arch-independent and lives in the shared `ex-rust`/`winsrv-rust` hard-error path,
not in anything amd64-specific.

**Also spotted, deliberately NOT changed here:** `ke/arm64/kiinit.rs`'s `KERNEL_STACK_SIZE = 0x6000`
against `xdk/arm64/ke.h`'s `0xC000`. It is self-consistent (the `KiP0BootStackData` array and the
top-of-stack computation use the same const), so it is a half-size boot stack rather than an
overrun — but it is a real divergence and deserves its own change plus validation.

### Harness gap worth fixing (found the slow way)

The arm64 bugcheck cost ~25 minutes of drain instead of an immediate verdict: the conformance
sweep's wait loop (`test_payload.py::test_payload_launched`) does break early on `*** BUGCHECK`,
but the run still burned to the outer timeout, and `conformance` reports neither a classification
nor a symbolicated PC the way `smoke` does. A guest BUGCHECK mid-sweep should abort the verb loud
with the stop code — same fail-loud shape as the three `--smp` checks above.

## arm64 clears its last conformance blocker: `-smp 2` sweeps end-to-end (2026-07-26)

The BuildType fix landed and the arm64 `-smp 2` kmtest sweep — which had never once
completed, dying at the KeSpinLock `0x9` 34 tests in — now **runs the whole corpus in 95
seconds**: `launched=89 ok=86 bad=3 sweep=done`.

Against the freshly-seeded arm64 UP baseline, all 89 rows present, no coverage loss, and
**not one regression**:

| test | UP baseline | `-smp 2` | reading |
|---|---|---|---|
| `KeSpinLock` | 281/42 | **286/37** | +5 — the SYNCH_LEVEL/BuildType alignment paying off |
| `IoInterrupt` | 937/158 | **939/156** | +2 |
| `ZwMapViewOfSection` | 120/14 | **121/13** | +1 |
| `KeEvent` | 42733 | 42601 | the documented MP-skip (§9.97), expected |
| `IoDeviceObject` | 181/8 | 180/9 | the known ±1 flicker, already in amd64's smp2 baseline |

The three `bad` rows are byte-identical to the UP baseline's three (`HalPortIo`,
`RtlCaptureContextKM`, `RtlCaptureContextUM`, all `NO-END`) — pre-existing, not MP.

### Two harness bugs the run surfaced first

**A UP-tuned `boot_timeout` reads a healthy `-smp N` boot as a wedge.** The first attempt
failed the boot fixture at arm64's 180s with the log still *talking* — it had reached
`[WINLOGON] explorer.exe created (PID 216)` and was mid-SCM startup. That budget is sized
against a ~40s UP boot; under MTTCG each vCPU is its own host thread and the guest does more
work, so wall-clock grows roughly linearly in N. `vm.smp_boot_timeout()` scales it. **Scale
the clock, never the marker** — `hang_silence_seconds` is deliberately left alone, because
the silence watchdog is what actually catches a wedge, so a genuine MP hang still fails in
~120s instead of burning the stretched budget.

**`--write-baseline` could silently delete tests from the ratchet floor.** It replaces the
baseline wholesale with the run's rows, so a sweep cut short by a boot flake or an early
BUGCHECK removes every test it never reached — permanently, with no signal. That is the
ratchet form of "never weaken a test to make it pass", and catching it was a hand-check
before every re-seed. Now `baseline_coverage_refusal()` refuses and names the tests that
would vanish (`--force-baseline` is the deliberate retire-a-test escape hatch), and
`baseline_lowering_note()` says out loud which rows a re-seed is making worse — that one is
allowed, because absorbing a known regression is sometimes intended (i386 and arm64 both took
`ExHardError` 30/0 → 24/6 on purpose).

### The MP gate leg got cheap on x86, and conformance stopped rebuilding for it

`conformance` no longer passes `--smp-kernel` on x86. Since `halmp.dll` became its own cell
the ISO carries both kernels and both HALs plus a `[LiveCD_SMP]` entry, and
`vm.smp_selected_iso()` patches `DefaultOS=` inside a *copy* of the media at boot time —
so `--smp-kernel` would only rewrite that same line via a cmake cache var, i.e. a reconfigure
and an ISO re-roll to change 22 bytes we change for free. Consequence worth having: **one x86
build now serves the UP, `--smp 1` and `--smp 2` sweeps**, via `--no-build`. arm64 still needs
the flag — its `boot.img` does not carry the MP pair at all unless `mkramdisk.sh --smp` stages
it, so there is nothing in the media to boot-select. Closing that asymmetry (always stage the
MP pair, patch the ini in the FAT image the same way) is the remaining follow-up.

## amd64 collapse: CLEAN, once the UP baseline stopped lying (2026-07-26)

Re-seeded amd64's UP baseline first, then ran the collapse gate against it. All three sweeps
came off **one build** — the payoff from retiring `--smp-kernel` on x86 — and all three were
`launched=89 ok=89 bad=0 sweep=done`.

| run | verdict |
|---|---|
| UP (re-seed) | 89/89, 0 bad |
| `--smp 1` | 89/89, 0 bad. **Single delta vs the UP baseline: `KeEvent` 42733 → 42601** — the documented MP-skip, and nothing else |
| `--smp 2` (re-seed) | 89/89, 0 bad, plus 2 NEW passes (`RtlExceptionKM`, `NtCreateSection`) |

`--smp 1` and `--smp 2` both printed `MP pair confirmed on serial: ntkrnlmp.exe + halmp.dll`
— the new conformance assertion, which is the first time an x86 conformance sweep has *proved*
it graded the MP kernel rather than assuming it.

**The stale UP baseline was doing real damage.** amd64's `ExHardError` row was still the
untouched 2026-07-17 `30/0` while the measured value had been `24/6` since 07-24, so every
amd64 sweep reported a phantom regression that had to be mentally subtracted before the real
signal was visible. Re-seeding is why the `--smp 1` line above is a single clean delta instead
of three.

### A pass-count DROP that was an improvement — why the guard announces instead of refusing

The re-seed lowered three rows, and `baseline_lowering_note()` printed all three. Two were
known (`ExHardError` 30/0 → 24/6; `IoDeviceObject` ±1 flicker). The third looked alarming:

    ExInterlocked: pass 662 -> 180, fail 40 -> 0

Total assertions 702 → 180 is a big drop, and a rule that refused count regressions would have
blocked it. Cross-arch check says it is a **convergence, not a loss**:

| arch | ExInterlocked | SEH path |
|---|---|---|
| amd64 (old baseline) | 662 / **40 fail** | pre-`-fasync-exceptions` |
| amd64 (now) | 180 / **0 fail** | native SEH |
| arm64 | 180 / **0 fail** | native SEH |
| i386 | 718 / 120 fail | DUMMY_PSEH — a genuinely different path |

amd64 landed on arm64's exact value with zero failures. That is `d3b1c045c5d`
("clang was DELETING every `__except` around an in-body fault") reaching amd64's measured
numbers. **Generalisable:** a falling assertion count is not evidence of lost coverage — it can
mean the test stopped re-running a broken path. Coverage loss is the thing worth refusing
(a test that never launched); count changes are the thing worth *showing* to a human.

## Collapse criterion MET on all three arches (2026-07-26)

i386 completed the same one-build / three-sweep pattern. Every arch's `--smp 1` run now differs
from its UP baseline by **exactly the documented `KeEvent` MP-skip and nothing else**:

| arch | `--smp 1` result | delta vs UP baseline |
|---|---|---|
| i386  | 89 launched, 86 ok, 3 bad, done | `KeEvent` 42733 → 42601 — only |
| amd64 | 89 launched, **89 ok, 0 bad**, done | `KeEvent` 42733 → 42601 — only |
| arm64 | 89 launched, done | `KeEvent` only, systematically; plus page-reuse flake noise |

All x86 MP runs printed `MP pair confirmed on serial: ntkrnlmp.exe + halmp.dll`.

Baselines re-seeded: **i386** (UP + smp2), **amd64** (UP + smp2). **arm64 deliberately NOT
seeded** — see the page-reuse entry in the roadmap registry; a ratchet floor cannot come from a
non-deterministic sweep.

The i386 re-seed was pure improvement, no lowering note at all: `RtlUnicodeString`
**NO-END → OK 77/0** (one fewer permanently-bad test) and `ExInterlocked` 718/120 → **922/52**.
Its smp2 baseline went 72 → 89 rows with **zero tests dropped** — it had been stuck at the
2026-07-18 shape, missing 17 tests the corpus had since gained, so it was silently not ratcheting
them at all.

### Remaining before the default can flip

1. **arm64 late-run page-reuse / stack corruption** — the one real blocker. Non-determinism at
   both `-smp 1` and `-smp 2`, so it is not an SMP defect; it blocks the arm64 baseline seed and
   any trustworthy arm64 ratchet. First symbolicated kernel-side instance and the next
   distinguishing test are in the roadmap entry.
   **CLOSED 2026-08-01** (`1266bb6c8477`) — stale FreeLDR identity PTEs in recycled user page
   tables; `MiPurgeStaleUserPtes` at `MmCreateMemoryArea`. See the roadmap entry.
2. **BUILD_INCOMPLETE** (roadmap) — instrumented, awaiting an occurrence to name itself.
   **CLOSED** — root-caused as the post-main ninja-optional passes re-emitting shared cohab
   staticlibs and stranding the main pass's binaries stale; fixed by a ninja-reaffirm pass.

`ExHardError` is NO LONGER on this list: the failure count proves the LPC round-trip succeeds,
so the six are the OS becoming more correct, and all three UP baselines now record 24/6.

## arm64 re-measured post-page-reuse-fix (2026-08-01): one NEW blocker, and it is real

With the page-reuse corruption gone the arm64 sweep is finally deterministic enough to seed and
to reason about, so this is the first *trustworthy* arm64 collapse measurement. Three UP rounds
on the merged tree: **91 launched / 88 ok / 3 bad**, and rounds 1 and 2 were byte-identical
results files. The single varying row is `IoDeviceObject` (below).

### The arm64 UP baseline was four days stale and under-ratcheting badly

Re-seeded off that run. Nine rows moved, eight of them **up**. Split the movement before
reading it (field manual, "A conformance score is not a defect count"): eight rows kept an
unchanged `pass+fail` total — pure fail→pass redistribution — and **`CmSecurity` alone gained
coverage, 389 → 910 assertions**, because it had been bailing early. Its failures fell 89 → 4
across that same +521 of newly-reached ground, which is luck as much as merit: a test that gets
further can just as easily surface new failures, and would then read as a regression while
being an improvement. Note also that zero is not the target — many of these only pass cleanly
on real Windows and stock ReactOS fails them too, so the counts below are a change detector
against our own history, not a distance from correct:

| test | was (2026-07-26) | now | note |
|---|---|---|---|
| `CmSecurity` | 300 / 89 | **906 / 4** | total assertions 391 → 910: it had been bailing early |
| `IoInterrupt` | 937 / 158 | **1093 / 2** | the spinlock owner stamp — see below |
| `KeGuardedMutex` | 1566 / 87 | **1653 / 0** | |
| `FsRtlLegal` | 2496 / 32 | **2528 / 0** | |
| `KeSpinLock` | 281 / 42 | **321 / 2** | |
| `ExSingleList` | 201 / 51 | **240 / 12** | |
| `ExDoubleList` | 110 / 36 | **130 / 16** | |
| `ZwMapViewOfSection` | 120 / 14 | **123 / 11** | the page-reuse fix |
| `IoDeviceObject` | 181 / 8 | **180 / 9** | LOWERED on purpose — bistable, see below |

`IoDeviceObject` is the one lowering, and it is a bistable read of uninitialised memory rather
than noise: `KmtGetPoolTag(ServiceKeyName.Buffer)` reads the POOL_HEADER in front of a pointer
that is not a tagged pool allocation, so it returns `0x00000000` on some boots (which also fails
the follow-on `KmtGetPoolType` check) and a garbage pointer-high-half on others (which passes it
by luck). Eight of the nine failures are fully deterministic. Seating the floor at the worse
value stops the arm64 ratchet reporting a phantom regression on 2 of every 3 runs; the honest
fix is the arm64-only `IoDeviceObject` divergence (9 failures vs i386's 4 / amd64's 5).

### `--smp 1` has TWO deltas, and only one of them is the documented MP-skip

| delta | verdict |
|---|---|
| `KeEvent` 42733 → 42601 | the documented MP-skip. Expected. |
| `IoInterrupt` 1093/2 → 937/158 | **not an SMP defect** — an upstream test gap. Explained below. |

All 156 are one assertion, `IoInterrupt.c:42`, fired once per iteration:

    *(TestContext->Interrupt->ActualLock) = 18446737476741476673, expected 1

`0xfffffa00060fb141` is `KeGetCurrentThread() | 1` — the spinlock **owner stamp**, which
`KI_TRACK_SPINLOCK_OWNER` writes on any checked build (upstream gates it on plain `#if DBG`,
and `ntkrnl-sys inline/ke.rs` mirrors it under `feature = "dbg"`). The test's `CheckSpinLock`
macro asks `KmtIsMultiProcessorBuild` FIRST and then demands `*Lock == 1` exactly; its
checked-build branch (`*Lock != 0`) is only reachable when the MP test already failed. **There
is no MP∧checked case**, so a correct DBG+SMP ReactOS kernel cannot pass it — upstream's own
would fail identically.

The kernel does the same thing under both kernels; only the test's expectation branch changes.
Four independent data points fit, which is what makes this a conclusion rather than a story:

| kernel | branch taken | `*Lock` holds | observed |
|---|---|---|---|
| pre-stamp UP | checked → "nonzero" | `0` | fails 156 ✓ arm64 937/158, amd64 939/156 |
| pre-stamp MP | MP → "== 1" | `1` (interlocked bit) | passes ✓ amd64 smp2 1095/0 |
| post-stamp UP | checked → "nonzero" | `thread\|1` | passes ✓ arm64 today 1093/2 |
| post-stamp MP | MP → "== 1" | `thread\|1` | fails 156 ✓ arm64 today 937/158 |

**Consequence worth acting on: all five x86 baselines are pre-stamp.** They were seeded
2026-07-26; `cfdff72b41d` ("restore the spinlock owner STAMP — a checked build that behaved
free") landed 2026-07-30. The next x86 sweep will therefore show a 156-assertion swing in *both*
directions — UP improving, MP regressing — with no source change behind either. Re-seed i386
and amd64 (UP + smp2) before reading anything into an x86 `IoInterrupt` delta. This is also the
trap that nearly cost this analysis: comparing today's arm64 numbers against those stale x86
baselines makes the delta look *inverted* by arch, which it is not.

### THE new blocker: arm64 MP wedges at lsass, ~30% of boots, and UP never does

Round 2 of 3 never reached the desktop. No BUGCHECK, no `[MMGUARD]`, no Rust panic — the serial
simply stops after:

    [SERVICES] ScmStartUserModeService(SamSs): waiting for connect

A good round continues `connect result=0` → `START returned=0`. So SCM launched lsass and lsass
never connected to its control pipe. (`CreateProcessW returned pid=0` and the
`NtControlPipe0 (disposition=0)` line both appear on GOOD rounds too — neither is the anomaly.)

The durable ledger (`~/vibeos-flake-farm/conformance.jsonl`, 430 records) makes the attribution
clean, because the UP:=smp1 experiment holds the CPU count fixed at 1 and varies only the
kernel/HAL pair. **Filter to rounds that actually booted first** — see the classifier caveat
below, without which every number here is wrong:

| arch | smp | booted | clean | `B:no-launch` | stalled | bugcheck | 1s non-runs |
|---|---|---|---|---|---|---|---|
| arm64 | **0 (UP)** | 28 | **28** | **0** | **0** | **0** | 1 |
| arm64 | **1 (MP)** | 126 | 88 | **14** | 11 | 13 | 46 |
| arm64 | 2 (MP) | 2 | 2 | 0 | 0 | 0 | 0 |
| amd64 | 0 (UP) | 14 | 14 | 0 | 0 | 0 | 0 |
| amd64 | 2 (MP) | 63 | 59 | 4 | 0 | 0 | 0 |
| i386 | 0 (UP) | 22 | 20 | 0 | 0 | 2 | 0 |
| i386 | 2 (MP) | 130 | 130 | 0 | 0 | 0 | 0 |

**arm64 UP is 28/28 clean; arm64 MP fails 38 of 126 booted rounds (~30%), across three faces.**
The 13 bugchecks were the page-reuse faces and should now go to zero, which leaves
`B:no-launch` (14) and `stalled` (11) as the live MP-only failures. 13 of the 14 `B:no-launch`
are pre-fix (2026-07-29); today's is the first post-fix one, so the post-fix rate is one
occurrence, **not** a measured rate — do not quote 1-in-3 as a frequency off n=3.

**Classifier caveat that changes every number: `shape` is derived from `launched == 0`, so it
lumps "the kernel never booted" together with "the harness never ran".** 46 of arm64's 92
`-smp 1` records have `elapsed_s` of 1 (or −1) — instant aborts, the `SMOKE_NO_SUMMARY` face,
not kernel failures at all. Taking `B:no-launch` at face value gives 52 and an apparent 41%
lsass-wedge rate; the real figure is 14 and ~11%. **Discriminate on `elapsed_s > 60` before
reading this ledger**, and note the separate historical face-B (roadmap) that *reached the
desktop* and then swept nothing for 87 minutes — a third thing again, also filed under `B:`.
Teaching the classifier to split these three is owed.

**This blocks the `DefaultOS` flip on arm64** — and it is squarely an MP-kernel defect, not
media, not CPU count, not the page-reuse bug. Next step is a repro loop that keeps the serial
per round (`--repeat` overwrites `qemu/arm64_smoke_live.log`): the wedge is silent, so the
discriminator is *where* lsass stops, which needs lsass-side markers or a debugger attach at
the pipe connect, compared against a UP round's lsass trace that completes.

## The wedge is RARE, not endemic — 20-round soak, instruments armed (2026-08-02)

The paragraph above ends on "next step is a repro loop." That loop ran, on a build carrying
both new instruments, and it **did not reproduce the wedge at all**.

| arm64 `--smp 1`, 20 rounds | result |
|---|---|
| clean sweeps | **19** — byte-identical every time: `launched=91 ok=88 bad=3`, 71–90 s |
| silent no-launch wedges | **0** |
| `A:bugcheck 0x000000E3` | 1 (task #43 — a *different*, loud failure at 52 s) |

Combined with the earlier post-fix rounds that is **1 wedge in 23, ~4%**. The "~30%" that
opened this hunt was measured on the pre-page-reuse-fix corpus and is retracted; even the
"1 of 3" that motivated the soak now reads as small-n bad luck. It remains a `DefaultOS`
blocker — a 4% silent freeze is not shippable — but it is a rare-flake hunt now, not a
repro-on-demand one. Use `farm`, not a hand-rolled loop, and stop theorizing until one lands.

**The collapse criterion held across all 19 rounds**, which is the stronger result. The only
ratchet deltas were the two already explained, and no third ever appeared:

    IoInterrupt 1093/2 -> 937/158   test-branch artifact of the restored owner stamp, not a defect
    KeEvent     42733  -> 42601     documented MP-skip

Nineteen consecutive identical sweeps is also the best evidence yet that the page-reuse fix
bought real determinism.

**Two instruments are now armed, so the next occurrence cannot be silent** — which is the
actual deliverable here, since the bug is too rare to corner by staring at it:

- `hang-cpustate.txt` (`34b44687a974`) — on *any* boot timeout the harness asks QEMU where every
  vCPU is parked and symbolicates it, before the kill. Wired into `wait_for_boot`, so the
  conformance sweep gets it too, not just smoke. Both backends previously ran `-monitor none`;
  we had never once asked a wedged box where it stopped.
- `SPIN_LOCK_ALREADY_OWNED` in `KxAcquireSpinLock` (`e4a0cc63c17`) — an MP self-deadlock
  bugchecks instead of spinning forever at `DISPATCH_LEVEL` with the clock stopped. It did
  **not** fire in 20 rounds. That is a weak negative for the spinlock hypothesis and nothing
  more: with zero recurrences of the wedge itself, the instrument was never given the chance.

Also settled here: the wedge is **not** a regression of the §9.89 SamSs handshake bug
(`74c08f68e68`). Same last serial line, but that one prints `connect result=1053` when its 30 s
elapses and the box keeps running. Ours prints no `result=` at all and every subsystem stops
within a few lines of each other — the 30 s `PipeTimeout` never expiring is what indicts the
clock rather than the service.

## arm64 media asymmetry CLOSED — the MP pair is now a boot pick, not a build (2026-08-02)

The last place where "which kernel runs" was a property of *how the media was built* rather
than *which entry freeldr selects*. x86 has shipped both pairs in every `livecd.iso` since the
halmp twin landed, with `vm.smp_selected_iso` repointing `DefaultOS=` in a copy of the image.
arm64 did it the other way round: `mkramdisk.sh --smp` staged `ntkrnlmp.exe` + `halmp.dll`
**only then**, and rewrote the single `[ReactOS_Arm64]` entry's `Options=` in place. So a
`--smp N` run on media built without `--smp-kernel` silently booted UP — identical-looking
media, no error, one CPU — which is precisely the false-green `_smp_check` exists to catch,
and it is why `smoke.py` carried an `arch.startswith("arm64")` branch telling you to go
rebuild. That branch is gone.

Now: `mkramdisk.sh` stages the MP pair unconditionally (and mkramdisk *requires* the two
outputs, so a build that could not produce them fails there instead of shipping media whose
MP entry names files that are not on it), and `qemu/freeldr.ini` ships
`[ReactOS_Arm64_UP]` + `[ReactOS_Arm64_MP]`. `DefaultOS=` stays on UP.

**The names are the same length on purpose.** `DefaultOS=ReactOS_Arm64_UP` and
`..._MP` are both 26 bytes, so selecting the MP pair is a one-byte `U`→`M` poke into a copy of
`boot.img` — no padding, no mtools at boot time, and the *same* `smp_selected_iso` all three
arches already used. Measured: 0.68 s for a 294 MB image, exactly 1 byte different, idempotent.
Keep them equal-length; `test_mp_entry_names_are_equal_length_to_the_defaults_they_replace`
guards it so renaming one half fails in pytest rather than at boot.

Proven on ONE build of ONE media, built with **no** `--smp`:

| run | freeldr loaded |
|---|---|
| `smoke --arch arm64`            | `ntoskrnl.exe` + `hal.dll` |
| `smoke --arch arm64 --smp 1`    | `ntkrnlmp.exe` + `halmp.dll` |

**A bug the unit tests could not have caught, and the guard did.** The first real `boot.img`
built from the two-entry ini refused to patch: `expected exactly one DefaultOS=, found 2`. The
second hit was the new ini's own *comment* explaining the equal-length rule, which says
`DefaultOS=` in prose. The search is now anchored to a line start — which is where freeldr
reads the key anyway — and the fixture carries that comment plus uefildr's bare `DefaultOS`
token so the distinction stays tested. Worth remembering as a shape: a synthetic fixture
agreed with the code, and only the real artifact disagreed. The same pass replaced a
per-offset `startswith` comprehension that walked all 294 MB in Python with `bytes.find`.

Not changed, deliberately: `DefaultOS=` still selects UP on every arch. The flip waits on the
remaining MP defects (the rare `--smp 1` wedge and the `0xE3`), not on media plumbing.

### The tail of it: `conformance` was still paying for the old asymmetry (2026-08-02)

`conformance.py` kept its own copy of the retired rule — `needs_smp_media_build = smp and base
== "arm64"`, justified by a comment saying arm64's `boot.img` "does not carry halmp.dll /
ntkrnlmp.exe at all". True when written, false since mkramdisk started staging both pairs. The
cost was not just a redundant media re-roll to change one byte: it meant the `--smp 1` and
`--smp 2` sweeps **of the same commit** needed two different builds, because one wanted MP-
default media and the other did not. Removed — one build now serves both, which is what makes
taking a collapse measurement and an MP baseline in the same session affordable.

Generalizable: when a capability stops being special, grep for callers still branching on the
special case. The branch does not fail — it just quietly keeps charging the old price.

## The UP floor moved under the collapse criterion (2026-08-02)

Merging `origin/master` brought in the ratchet lane's `bfda94a3ce1`, which removed **765
baselined failures** from the UP `kmtest.txt` floor on all three arches — `CmSecurity`
900/2 → 1048/1 everywhere, i386 `IoInterrupt` 1839/306 → 2145/0, `KeGuardedMutex` 1566/87 →
1653/0, `FsRtlLegal` 2496/32 → 2528/0.

**That is the oracle the UP:=smp1 collapse is measured against.** `--smp 1` ratchets the MP
kernel on one CPU against `kmtest.txt`, and a ratchet only asserts *no worse than baseline*, so
every collapse result this lane has banked — including "holds 19/19" — was graded against a
floor carrying that slack. Any residual MP-at-1-CPU divergence sitting inside those 765 rows
was invisible by construction.

The collapse is a `DefaultOS`-flip gate, so it is being re-taken against the tightened floor.
If the MP kernel at one CPU really is UP, it reproduces the tightened numbers and the collapse
still holds; this is a sharper test, not a redo. Worth stating as a rule: **when a baseline is
re-seeded, every criterion defined as "ratchets against that baseline" needs re-taking, even
though nothing in the criterion's own code changed.**

The two MP baselines are stale for the same reason and are being re-seeded alongside:
`i386/kmtest-smp2.txt` and `amd64/kmtest-smp2.txt` are both from 07-26 and predate
`cfdff72b41d` (07-30, spinlock owner STAMP restored). They also *lack* `IoFilesystem` and
`NpfsFileInfo` entirely (91 rows against UP's 93), and bank `CmSecurity` at pass=300 fail=89
where the sweep now launches 1049 assertions — the smp2 sample was taken when the sweep died
early. arm64's smp2 baseline was already re-seeded on 08-01 and is fine.

### Result: the collapse holds on amd64 + arm64, and i386 has ONE real defect

Re-measured on all three arches, `--repeat 2`, every row below identical across both rounds.

| arch | rows the tightened UP floor flags at `--smp 1` |
|---|---|
| i386  | `IoInterrupt` 2145/0 → 1839/306 · `KeEvent` −132 pass · **`KeSpinLock` 710/75 → 706/79** · `IoDeviceObject` 185/4 → 184/5 |
| amd64 | `IoInterrupt` 1095/0 → 939/156 · `KeEvent` −132 pass |
| arm64 | `IoInterrupt` 1093/2 → 937/158 · `KeEvent` −132 pass |

**Two of these are MP-conditional by construction and can never go to zero.**
`IoInterrupt`'s `CheckSpinLock` branches on `KmtIsMultiProcessorBuild`; the MP kernel
truthfully reports MP, takes the strict arm, and fails on the upstream owner stamp (see the
baseline files for the full reasoning). `KeEvent` launches exactly 132 fewer assertions on
every arch, with zero new failures — the MP-fatal sections the test skips on an MP build. The
ratchet flags a dropped pass count even when nothing failed, which is correct conservatism.

So **the criterion itself needed restating**: "UP:=smp1" cannot mean *zero ratchet regressions
against the UP baseline* while kmtest branches on the build flavour. It means **zero
UNEXPLAINED deltas**, against an enumerated list of MP-conditional rows.

**The old "holds 19/19" was true against a floor that agreed with MP for the wrong reason.**
Pre-stamp, a held lock read 0 on *both* kernels, so UP's `ok_eq_bool(*Lock != 0, TRUE)` failed
306 and MP's `ok_eq_ulongptr(*Lock, 1)` failed the same 306 — identical counts, different
causes. The UP floor banked `1839/306`, the MP run produced `1839/306`, and the comparison
reported no delta. Restoring the stamp fixed UP only, and the re-seed made the floor honest,
which is what let the coincidence show itself. Generalize: **a ratchet compares numbers, not
reasons, so two rows can agree while measuring different things** — and a criterion built on
"no delta" inherits that blindness.

**The one genuine find is i386 `KeSpinLock` +4**, isolated against a UP control run on the same
media (histograms sum exactly: UP 75, MP 79). The delta is `+5 "TryAcquire returned FALSE,
expected TRUE"` and `−1 "Ret = TRUE, expected FALSE"`. `KeSpinLock.c:280-289` releases a lock,
asserts it reads 0, then try-acquires expecting TRUE. The `expected 0` assertion fails 5× on
*both* kernels — the lock is left DIRTY after release either way, and that half is already
banked on UP. Only MP acts on it: its try-acquire is a real `InterlockedBitTestAndSet`, sees
bit 0 still set, and returns FALSE, where UP's has no `CONFIG_SMP` body and returns TRUE
unconditionally. A lock word that survives release is not cosmetic on MP — a genuine acquire on
it spins forever at IRQL ≥ DISPATCH_LEVEL. Tracked as its own task; `cfdff72b41d` already named
this hazard (`KeTryToAcquireSpinLockAtDpcLevel` stamping under plain `#if DBG`) and believed it
closed, so some path still leaks.

**Flip status: unchanged, `DefaultOS=` still selects UP.** amd64 and arm64 collapse cleanly
modulo the two enumerated rows; i386 does not until the leaked lock word is understood, and it
is the same shape as the two open MP blockers (#39's silent stall, #43's `0xE3`).

## i386's "one real defect" was the HAL: halmp.dll had no spinlocks at all (2026-08-02)

The i386 `KeSpinLock` delta is closed, and it was bigger than the row. **`hal-rust`'s x86
`kx_acquire_spinlock` / `kx_release_spinlock` were unconditional compiler fences** — the UP arm
of `spinlock.h` frozen into the source, under a module docstring that said so out loud:
*"We're a UP-only build."* True when written; false since the halmp.dll twin landed, because
`hal-cargo` and `halmp-cargo` now build that same crate one `smp` feature apart and neither
funnel looked at the feature.

They were also wrong on the **UP** HAL, which is why this is not merely an MP bug.
`KxReleaseSpinLock` clears the lock under `CONFIG_SMP || KI_TRACK_SPINLOCK_OWNER`, and
`KI_TRACK_SPINLOCK_OWNER` is plain `#if DBG` — the C half stamps and clears the owner on every
checked build. So ntoskrnl stamped locks the HAL never cleared, exactly the failure
`spinlock.h`'s own comment predicts: *"the two MUST agree, or a lock stamped by one side is
never cleared by the other."*

Past the test row: on i386 the whole `Ke*SpinLock*` / `Ke*QueuedSpinLock*` surface is a **HAL**
export (`hal.spec`, `-arch=i386,arm,arm64`), so halmp.dll has been giving `--smp 2` no mutual
exclusion beyond the IRQL raise. That is not a conformance nit; it is the MP kernel's locking.

**What actually named the layer was the cross-arch comparison.** i386 UP `KeSpinLock` sat at
`710/75` where amd64 and arm64 both sat at `321/2` — and amd64 is precisely the arch that takes
these from `ntoskrnl.spec` (`-arch=x86_64`) instead of the HAL. One outlier row, one
spec file, and the layer falls out. Worth doing before theorising about the failures themselves.

### Correcting the previous entry's mechanism

The section above inferred that the 5 `TryAcquire returned FALSE` failures were the same locks
as the 5 `expected 0` ones at `:336`. They are not. Ordering the serial by `Test SpinLock run`
separates them cleanly: the `:336` dirt is runs 2/5/8/14/20, and the try-acquire failures are
runs **22–26**, a contiguous block that is the two `CheckQueue` rows (`LockQueuePfnLock`).
Those call `KeTryToAcquireQueuedSpinLock`, which funnels into ntoskrnl's real interlocked
`KeTryToAcquireSpinLockAtDpcLevel` and pairs it with the HAL's no-op `KeReleaseQueuedSpinLock`
— so run 21 acquired the PFN lock once and every later attempt failed forever. Same root, but
"the lock is left dirty by release" was the wrong sentence: the *reason* the pairing is
mismatched is that the two halves come from different modules, and only one of them was
translated with the flavour axis intact. `cfdff72b41d` was not wrong about its own path.

### Result

One implementation now, not two: both HAL funnels call
`ntkrnl_sys::inline::ke::Kx{Acquire,Release}SpinLock`, with `hal-rust/smp` forwarding to
`ntkrnl-sys/smp`. FreeLDR's mini-HAL keeps the bare fence — there is no KPCR to read
`KeGetCurrentThread()` from pre-kernel, and it is single-threaded by construction.

| i386 `KeSpinLock` | before | after |
|---|---|---|
| UP kernel   | 710 / 75 | **753 / 32** |
| MP kernel   | 706 / 79 | **754 / 31** |

Deterministic: `--smp 1` and both `--smp 2` rounds all read `754/31`. Both rows re-seeded with
`--write-baseline --only KeSpinLock`, so no bistable row got re-rolled in the process.

The 31/32 residual are two *other* upstream shapes, present on both kernels and so not collapse
blockers: the queued-spinlock HACK (`KeAcquireInStackQueuedSpinLock` sets no `LQ_WAIT` bit, and
the lock word carries the owner stamp where a real queued lock carries the queue-handle
pointer), plus one try-acquire that sets bit 0 without stamping the owner — the latter is task
#47, `KeTryToAcquireSpinLockAtDpcLevel` gating its stamp on cargo's `debug_assertions` instead
of `feature = "dbg"`, and it is the entire remaining MP-vs-UP difference in this test.

**The i386 collapse now holds on the same terms as amd64 and arm64.** `--smp 1` flags exactly
three rows: `IoInterrupt` and `KeEvent` (both enumerated MP-conditional, unchanged) and
`IoDeviceObject`, which is the known bistable one — re-confirmed directly this session by two
back-to-back `--smp 2` rounds on identical media reading `185/4` then `184/5`, the extra
failure being `IoDeviceObject_drv.c:300`.

**Flip status: still unchanged, `DefaultOS=` selects UP.** The collapse criterion is met on all
three arches now, but #39 (arm64 `--smp 1` stall) and #43 (`0xE3`) are still open, and #46 says
arm64's HAL has the same no-exclusion hole i386 just had — self-consistent, so no kmtest row
catches it, but halmp.dll on arm64 is in the state halmp.dll on i386 was in this morning. The
default should not move onto a HAL whose locks are decorative.

## The two surviving `--smp 1` rows are now PROVEN enumerated, not just asserted (2026-08-02)

With i386's `KeSpinLock` fixed, all three arches reduce to the same residual, and it is the
same two rows everywhere:

| row | i386 | amd64 | arm64 |
|---|---|---|---|
| `IoInterrupt` fail | 0 → 306 | 0 → 156 | 2 → 158 |
| `KeEvent` pass | −132 | −132 | −132 |

(i386 additionally shows `IoDeviceObject`, which is the bistable row, not an MP delta.)

Both had been *called* MP-conditional. Neither had been demonstrated. They are now.

**`KeEvent` is the test declining to run, and the arithmetic closes exactly.** `KeEvent.c:83`
guards a block behind `!KmtIsMultiProcessorBuild`, with upstream's own reason on the line above:
`KeSetEvent(..., Wait=TRUE)` and `KePulseEvent(..., Wait=TRUE)` leave the dispatcher lock held,
which is only legal to exercise on UP. The block is two `CheckEvent`s plus four scalar checks;
`CheckEvent` with `ThreadCount=0` is 9 assertions, so 22 per call. `TestEventFunctional` is
called twice per IRQL over `{PASSIVE, APC, DISPATCH}` = 6 calls. 6 × 22 = **132**, and the
failure count does not move. This row can never be driven to zero — it is not our kernel being
wrong, it is the suite refusing a UP-only experiment on any MP kernel, real Windows included.

**`IoInterrupt` is upstream ReactOS diverging from Windows, not vibeos diverging from ReactOS.**
`CheckSpinLock` asserts a held lock reads *exactly* 1 on an MP build and merely non-zero on a UP
checked build. That is correct Windows semantics: on MP the word IS the lock. ReactOS stamps
`KeGetCurrentThread() | 1` into it under a plain `#if DBG`
(`ntoskrnl/include/internal/spinlock.h`, `KI_TRACK_SPINLOCK_OWNER`) with no `CONFIG_SMP` term,
so an MP checked build fails this assertion upstream too. Benign for exclusion — bit 0 is set
either way and release is an unconditional clear.

Fixing `IoInterrupt` therefore means the owed item: keep the word at exactly 1 on MP and track
the owner out of band. Scoping that turned up a hazard worth recording before anyone tries it.
A per-CPU held-lock table is the obvious shape, and it is **wrong here**, because it is only
sound if every acquire path releases through the same funnel. `KeTryToAcquireSpinLockAtDpcLevel`
(#47) and the queued-lock HACK both set bit 0 without going through `KxAcquireSpinLock`, so the
table leaks entries, and a leaked entry is a *false* `SPIN_LOCK_ALREADY_OWNED` bugcheck on three
boot paths. The instrument the stamp currently feeds is better replaced by a **bounded spin** —
it needs no bookkeeping, cannot desynchronise, and catches inter-CPU deadlock cycles that an
owner check structurally cannot see. That is its own change with its own soak, deliberately not
folded into the i386 fix.

**Collapse status: the criterion is met on all three arches** — zero unexplained deltas, with
both survivors now mechanically accounted for rather than taken on trust.

## The wedge hunt was blind, in three separate ways (2026-08-08)

`--smp 2` reproduced the SamSs wedge (#39) on 2026-08-03 and the run dir held serial, results
and `summary.json` — nothing else. Chasing that produced three findings about the *instruments*
rather than the bug, and all three are the same shape: an observation that could never have been
made, mistaken for an observation that came back empty.

1. **The capture existed and was thrown away.** `wait_for_boot` wrote `hang-cpustate.txt` into
   the VM's `work_dir`, which under the pytest `vm` fixture is a `tmp_path_factory` dir. pytest
   keeps the last three runs. By the time anyone read the 08-03 run dir the file was gone, and a
   whole-disk search found no `hang-cpustate.txt` anywhere. Fixed: `save_hang_state()` writes to
   `work_dir` **and** `$VIBEOS_ARTIFACT_DIR`, which `conformance` points at its run dir.

2. **The wedge does not enter the path that captures.** A drained-out sweep boots fine — 2002
   serial lines — so it never raises the boot-wait `TimeoutError` the capture hangs off. The
   comment there claimed the placement covered "the conformance sweep — the very place the arm64
   `--smp 1` stall shows up". It did not. `test_payload_launched` now captures when its
   launch-marker drain exhausts without a crash, which is the path this bug actually takes.

3. **One dump cannot answer the question the dump exists for.** A dead clock and a thread
   blocked forever both park every vCPU in `KiIdleLoop`. The capture now samples twice, 5s apart,
   and writes **FROZEN** or **EXECUTING** into the artifact — the fork the whole hunt turns on,
   and one that cannot be recovered after the guest is torn down.

And a fourth, which is not fixed: **the `[CENSUS]` watchdog cannot witness this class at all.**
`CensusThread` loops on `KeDelayExecutionThread(10s)` — a timed wait. The wedge's own signature
is a 30s timed wait (`PipeTimeout`, services-rust `database.rs:166`) that had not expired after
300s. If timed waits stop expiring, the watchdog stops first, so its silence during the wedge is
not evidence of "nothing stuck". A detector must not depend on the mechanism it is watching.

What the serial does now say, which it did not before: good runs print
`[SCTRL] pid=184 connecting pipe=\\.\pipe\net\NtControlPipe0` about 100 lines after SCM starts
waiting, and the wedge never prints it. So SCM created lsass, resumed it, and blocked; lsass's
dispatcher never reached the pipe. The `NtControlPipe0 (disposition=0)` line that looks alarming
is byte-identical in the good runs — not a lead.

## #47 closed the KeSpinLock MP-vs-UP divergence on all three arches (2026-08-08)

`KeTryToAcquireSpinLockAtDpcLevel` gated the `KI_TRACK_SPINLOCK_OWNER` stamp on cargo's
`debug_assertions`, which is OFF for `ntoskrnl-rust` — the per-crate override in
`sdk/lib/rust/Cargo.toml` is written out but commented, staged behind the arm-the-asserts
migration. So the stamp never happened, while `KxAcquireSpinLock` wrote it under
`feature = "dbg"`. On UP the `smp` body compiles out too, leaving the entire function a no-op
that returned TRUE against a lock that could never read as held.

Measured before and after, `KeSpinLock` rows:

| arch  | UP before | UP after | MP@1cpu before | MP@1cpu after |
|-------|-----------|----------|----------------|---------------|
| arm64 | 321 / 2   | 323 / 0  | 322 / 1        | 323 / 0       |
| i386  | 753 / 32  | 755 / 30 | 754 / 31       | 755 / 30      |
| amd64 | 321 / 2   | 323 / 0  | —              | 323 / 0       |

The same shape three times: two failures fixed on UP, one on MP, and **UP and MP now agree
exactly**. This row no longer contributes anything to the collapse delta on any arch. All three
UP baselines were re-seeded with `--write-baseline --only KeSpinLock`, one line each — the
improvement is real and explained, and the ratchet only flags regressions, so leaving it would
have silently banked slack.

Note what made this findable at all: the row was a *one-assertion* MP-vs-UP difference, which is
exactly the size of signal the collapse criterion exists to surface. Two of the three arches
were also carrying a second UP-only failure that no MP-vs-UP comparison could ever have shown —
it took reading the cfg to see that the whole function was dead.

i386's `IoDeviceObject` baseline turns out to be seeded on the LUCKY side of a coin flip
(line 300, `STATUS_INVALID_DEVICE_REQUEST` vs `0xC0000034`, alternates across runs), so it
reports a phantom regression on about half of all i386 sweeps. That is the inverse of arm64,
whose floor deliberately absorbs the worse value. Left alone deliberately — lowering a shared
gate floor wants a decision, not a quiet edit. Evidence and a nine-run sample are on the task.
**Decided 2026-09-05:** the i386 `IoDeviceObject` floor is lowered to 184/5, the worse face,
matching what arm64's floor already does. The ratchet only flags regressions, so the lucky face
now reads as slack rather than as a phantom regression.

## #39 caught: the arm64 wedge is an EL1 fault loop at a user PC, not a dead clock (2026-09-05)

A 300-round `--smp 1` kmtest soak on fixed media (`~/vibeos-soak/smp39-arm64/`) hit the wedge
**5 times (1.7%)**, and the rebuilt instrument fired on every one. What it showed:

| round | verdict | sample 1 | sample 2 | last serial line |
|-------|---------|----------|----------|------------------|
| 03:45 | FROZEN | `PC=0x10000` EL1h | same | `ScmStartUserModeService(DcomLaunch)` |
| 04:19 | FROZEN | `PC=0x10000` EL1h | same | same |
| 05:38 | FROZEN | `PC=0x10000` EL1h | same | same |
| 06:26 | FROZEN | `PC=0x10000` EL1h | same | same |
| 07:47 | EXECUTING | `ntoskrnl+0x2824` = **`KiTrapReturn+0x24`**, DAIF all masked | `PC=0x10000` EL1h | same |

One shape, five times: the vCPU is at **EL1 with a user-range PC**, every timed wait is dead
(`boot_timeout=180s` drained with the SCM one `DbgPrint` short of `CreateProcessW`), and the odd
sample caught it inside the eret path. That is not a frozen clock. It is a tight loop: instruction
abort at `0x10000` → `KiSynchronousExceptionHandler` → `MmAccessFault` says success →
`KiTrapReturn` → `eret` to `0x10000` → abort. The register file is byte-identical across samples
because every iteration restores it from the same trap frame.

Why the kernel calls it success: `MmArmAccessFault`'s valid-PTE path (`ARM3/pagfault.rs`, the
user-address branch) bugchecks an instruction fetch only when `MI_IS_PAGE_EXECUTABLE` is false
(UXN, bit 54). A user page that is user-executable passes, and the function returns
`STATUS_SUCCESS` — spurious, retry. On AArch64 an EL0-writable page is implicitly PXN, so EL1
cannot execute it no matter what UXN says, and the retry can never succeed.

**Fixed (this lane):** in `trap.rs`, a same-EL instruction abort (EC 0x21) whose VA is below
`MmUserProbeAddress`, or whose IFSC is a permission fault, no longer goes to `MmAccessFault` — it
is dispatched as a kernel access violation, which bugchecks `0x1E` with `[KEXC]`/`[KSTACK]`, plus
a new `[KSTKW]` dump of the kernel-range words above the faulting SP. The trap frame alone cannot
name the jumper: the captured frame had `LR=0` and **every callee-saved register zero**, with
`X18` a TEB and `X28` a user start-context — a user thread's register file, at EL1, with the kernel
about to fetch from the lowest user granule. Whatever jumped there did it with `br`, or by an eret
from a frame that said `Pc=0x10000, Spsr=EL1h`. The stack words are how the next occurrence names
it.

**What this changes about #39:** the ~2% wedge is no longer a hang. The next soak round that hits
it will BUGCHECK with a symbolicated frame and a stack, which is a bug we can read. The
first-cause (who sets PC to `0x10000` at EL1) is still open and is the actual flip blocker; the
loop that hid it for five weeks is closed.

The instrument learned the same lesson: `save_hang_state` now writes **FAULT-LOOP** when a vCPU
is at kernel EL with a user PC, and `capture_cpu_state` runs `x /6i` at every PC, so the next
same-PC pair cannot masquerade as a dead clock.

Two side notes from the same soak. The `0xE3` (`#43`) fired once **on the MP kernel at
`--smp 1`** (`20260905-052822`), so "UP-only" was a sample-size artefact, not a property — it is
still not MP-conditional, but it is not UP-conditional either. And 294/300 clean rounds showed
exactly the two enumerated MP-conditional rows (`KeEvent` MP-skip, `IoInterrupt` upstream gap)
and nothing else: the collapse delta is stable at zero unexplained across 294 samples.

### The one probe built to catch this could never have fired — three separate reasons

`KiTrapReturnProbe` (trap.rs) exists precisely to catch a bad EL0-return state at the `eret`,
and it was in the tree for the whole five weeks this wedge went unexplained. It missed for three
independent reasons, each sufficient on its own:

1. **It is not compiled.** The call site is `#ifdef ARM64_TRAP_DEBUG` in `trap.S:251`, and
   `ARM64_TRAP_DEBUG` is defined **nowhere in the repo** — not a CMakeLists, not a header, not a
   cmake cache. The probe is dead source on disk, which reads exactly like an armed instrument.
2. **It returns early on the frame that matters.** `if (spsr & 0xF) != 0 { return; }` — it looks
   only at EL0t returns. The wedge's frame says **EL1h**, so even compiled in, it would have
   declined to look.
3. **Its PC bound is off by one at exactly the observed value.** `pc < 0x1_0000` is a strict
   less-than and the wedge's PC is `0x10000` exactly. Even had it looked, it would have passed
   the frame as good.

The frame it declined to inspect is the whole answer: `Spsr` says **EL1h** while the register
file is a **user thread's** — `X18` a TEB (`0x7ffffffe7000`; in kernel mode x18 is the KPCR, per
`KeGetPcr` = `mov x0, x18`), `X28` a user start-context, `LR` and every callee-saved register
zero. A frame carrying user register content with a kernel `Spsr` is not a wild branch. It is a
**corrupted or mis-built trap frame**, and `KiInitializeContextThread` writes `Spsr = 0` (EL0t)
for exactly this shape of thread — so something changed it, or built it somewhere else.

**Owed, deliberately deferred:** fix (2) and (3) so the probe also flags a KERNEL return whose
`Pc` is a user VA, and decide whether to compile it in for the MP kernel. Not done in this pass
because editing any `.rs` while the 400-round verification soak runs on fixed media makes every
remaining round abort on a staleness check — the soak is the verification of the fix that is
gating now, and it outranks a second instrument.

Note the shape, which is the same one this lane has now hit three times: **an instrument that is
present, plausible, and structurally incapable of firing.** Grep for the guard, not the code.

### Lead for the first cause: arm64 decides a frame's target EL from the CALLER's mode

Not proven to be this wedge — the stack dump is what will settle that — but a real defect found
while reading the frame, and the only mechanism so far that produces the observed state without
invoking corruption.

`KeContextToTrapFrame` (arm64 `stubs.rs`, and the C it came from) picks the frame's exception
level like this:

```rust
if PreviousMode == UserMode { Spsr = Cpsr & !0xF; }        // EL0t
else                        { Spsr = (Cpsr & !0xF) | 0x5; } // EL1h
```

`PreviousMode` is the **caller's** trust level, not the **frame's** target level, and those are
different questions. `KiContinue` (`ke/except.rs`, the core of `NtContinue`) branches on
`KeGetPreviousMode()` and passes `KernelMode` straight through on the kernel path. So a
kernel-mode caller continuing a trap frame that returns to **user** code stamps **EL1h** onto a
frame whose `Pc` is a user VA. The `eret` then lands at EL1 with a user PC — the wedge's state
exactly, including a user TEB in `X18` and every callee-saved register zero.

The asymmetry is the tell: **i386 and amd64 do not do this.** They ask the FRAME, via
`KiUserTrap(TrapFrame)`, and `arm64 has no `KiUserTrap` at all` — grep finds none in
`ke/arm64/` or `ntoskrnl/ke/arm64/`. arm64 substituted the caller's mode for a frame property
that only the frame knows.

**Owed, same deferral as the probe fix** (editing `.rs` aborts the running soak on staleness):
give arm64 a frame-based target-mode test and use it here, rather than the caller's
`PreviousMode`. Then re-soak. If the wedge survives that, the cause is corruption and the
`[KSTKW]` dump is the instrument for it.

## #39 root-caused: a dropped `ASSERT`, not a memory barrier (2026-09-06)

Two overnight soaks on the four fixes plus the `[FAULTLOOP]` tripwire settled the lane's two
open questions and falsified my leading hypothesis.

### The UP question is closed

| kernel | rounds | wedges | `[FAULTLOOP]` reports |
|---|---|---|---|
| UP (`ntoskrnl.exe`) | 400 | 0 | **none, on any boot** |
| MP (`ntkrnlmp.exe`, `--smp 1`) | 128 | 3 (2.3%) | every boot |

The UP kernel's clean record used to rest on 35 samples, where a 1.3% rate yields zero 63% of
the time — worth nothing. At 400 rounds it means something, and the discriminator is sharper
than the wedge count: UP does not print a single retry report, ever. This is not a race that
MP loses more often. It is a code path MP takes and UP does not. **The MP-conditional list for
the UP:=smp1 criterion now has a real entry.**

Note also that `--smp 1` means one core, so nothing here is a cross-CPU race. The earlier
memory-ordering story — arm64's page-table walker as a separate observer needing `dsb ishst` —
is **not** this bug. That barrier (`MI_WRITE_VALID_PTE`) is defensible on its own terms and
stays, but it was not what wedged the box, and I had already noted that TCG models no store
buffer. Recording it as falsified rather than quietly dropping it.

### What the tripwire caught

The wedge is not an infinite loop at one address. The faulting VA marches forward 4 bytes at a
time through the user stack while the PC sits still at `KiDispatchException+0x458` — the kernel
writing an exception frame onto the user stack — with each word taking 64+ pager round-trips:

```
[FAULTLOOP] VA=0x0000000000138118 PC=0xFFFFF800421A44B0 ESR=0x96000007 DFSC=0x00000007
[FAULTLOOP] TTBR0=0x0000000042B5B000 PTE@0xFFFFF680000009C0=0x000000004738C041
```

`0x...041` is the whole bug: bit 0 (Valid) and bit 6 (Owner) set, **bit 1 (`NotLargePage`) clear**.
A level-3 descriptor of type `0b01` is reserved, so the MMU raises a translation fault (DFSC
0x07) that software cannot see — `MmAccessFault` reads Valid=1, concludes the page is already
resolved, returns `STATUS_SUCCESS`, and the trap returns to the same instruction. With DAIF
masked there is nothing left to break the loop.

### Where the malformed entry comes from

`MmProtectToPteMask` is the only source of bit 1 and bit 10 (AF) on arm64, and it is zero at
exactly four rows — 0, 8, 16, 24, the no-access protections. The C original of
`MI_MAKE_HARDWARE_PTE_USER` opens with three ASSERTs whose comment says precisely this:

```c
/* Check that we are not setting valid a page that should not be */
ASSERT(ProtectionMask & MM_PROTECT_ACCESS);          /* == 7: selects those four rows */
ASSERT((ProtectionMask & MM_PROTECT_SPECIAL) != MM_GUARDPAGE);
ASSERT(ProtectionMask != MM_OUTSWAPPED_KSTACK && ...);
```

**Both Rust implementations dropped all three** (`ARM3/helpers.rs`, `ntkrnl-sys/inline/mm.rs`).
On x86 the loss hides, because `Valid | User | PFN` is a complete present PTE there — you get a
wrongly readable page, not a hang. On arm64 it is a frozen machine.

The bit pattern also names the writer, which matters because there is a second candidate. The
transition-PTE resolve path (`ARM3/pagfault.rs:1122`) ORs the same table in and then calls
`MI_MAKE_ACCESSED_PAGE`, so a zero mask there would leave AF **set** (`0x...401`). The observed
entry has AF clear, which only `MI_MAKE_HARDWARE_PTE_USER` produces. That sibling path carries
the same latent hole and is owed a guard.

### Fix, and what is still open

`ntkrnl_sys::mm::mi_user_pte_may_be_valid` restores the assert as a runtime guard, centrally so
both implementations and any future one inherit it: a protection granting no access leaves the
entry not-present — which is what the mask asked for — so the fault takes the invalid path and
the thread gets a clean access violation instead of taking the box down. It reports once per
distinct mask as `[NOACCESSPTE]`.

Still open, and the reason the guard reports the mask rather than just fixing the bits:

- **Why does only the MP kernel reach this call with a no-access mask?** The `smp` feature
  changes `SYNCH_LEVEL` (2 on UP, 12 on arm64 MP), `KeGetCurrentProcessorNumber` and the
  spinlock bodies. Which of those steers a no-access protection into a make-this-valid helper
  is not yet known. The mask value in the next boot's `[NOACCESSPTE]` line should name it —
  16 means a guard page, 8 no-access, 0 zero-access.
- **The guard treats a symptom at the callee.** The caller has no error return, so leaving the
  entry not-present is caller-invisible and may leave PFN share counts inconsistent. It is
  strictly better than a frozen box and it is the diagnostic that finds the caller, but the
  real fix belongs where the mask is chosen.
- **A design fix worth considering:** bit 1 is required by *every* valid L3 entry on arm64, so
  smuggling it through the protection table is fragile by construction. Setting it in
  `set_valid()` / `MI_WRITE_VALID_PTE` would make the whole class impossible.

### Owed tooling

`scan_assert_to_ub` does not cover this class. It catches an assert lowered to
`unreachable_unchecked()` — present and wrong — not one deleted outright, and its sibling
`scan_dropped_prints` relies on rustconv's trailing C comments as an oracle, which hand-written
bridge modules like `ARM3/helpers.rs` do not have. A scanner pairing C `FORCEINLINE` helpers in
`miarm.h` against their same-named Rust twins and flagging dropped ASSERTs would have caught
this before a single boot.
