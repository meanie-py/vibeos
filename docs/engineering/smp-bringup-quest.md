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

## Progress log

- 2026-07-10: Lane opened; survey above. Nothing built or booted yet.
