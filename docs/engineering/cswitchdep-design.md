# cswitchdep — a dynamic context-switch verifier

**Status:** LANDED / living reference (2026-07-11). All four phases built and
validated on all three arches; the `cswitchdep-phase1` lane is **closed** and
this doc graduated out of `active-lanes/`. cswitchdep is now an always-on
opt-in tripwire (`vibeos-cli build --cswitchdep`) alongside lockdep/waitdep.
Post-close follow-ons (subset arming; the stable-everything toolchain program
that was scouted from this branch; KeGetPcr x18 dedup) are tracked in the
roadmap Owed-work ledger, not here. Sibling of the lockdep detector
(`sync-instrumentation-design.md` §4) and the planned shadow-IRQL check (§6).
The static counterpart is `execution-context-ownership-model.md`.
v2 incorporates an adversarial review pass: v1's two headline invariants
(held-locks==0 at voluntary switch; resume-IRQL==WaitIrql at the resume tap)
were **false in this kernel as specced** and are corrected below; the mid-swap
asm tap was unsafe and tautological and is deleted; parity modeling, shadow
reentrancy, and ThreadShadow lifecycle are now specified.

## TL;DR

A context switch is the one place where *every* "who may run here" invariant is
handed off at once: IRQL, held locks, per-CPU pointers, the GS base, the address
space, and the kernel stack. lockdep watches locks; the IRQL self-check watches
IRQL; **cswitchdep watches the hand-off itself** — it snapshots the execution
context on both sides of a switch and asserts the transition was legal, pinning
any violation to the exact switch and the culprit PC that caused it, instead of
letting it surface minutes later as an innocent-victim BUGCHECK.

> **What it turned out to be in practice, on the hunt it was built for: a repro
> AMPLIFIER, not a detector.** Under `-smp N` its violation output is noise — the
> held-count accounting uses a single global counter with no per-CPU separation, so
> it mis-fires on most switches. It raised the ready-list flake's reproduction rate
> from ~0.9% to ~35%, which is what actually cracked that bug, but **the violation it
> prints does not pin the culprit.** Treat a `cswitchdep` violation line as "run this
> again under real forensics", never as an attribution. Per-CPU held-counts would be
> needed before the design intent above holds under SMP.

It is built to the **exact shape of lockdep**: a zero-init `.bss` side table
(zero image cost when off), lock-free/allocation-free updates, link-name
instrumentation taps, `returnaddress(0)`/`x30` culprit capture, a PASSIVE-level
serial census, a `debug_block.py` GDB decoder, and a `VIBEOS_CSWITCHDEP` build
feature. It works on the current mixed C+Rust kernel, on all three arches, today.

**Hard dependency: `cswitchdep` requires `lockdep`.** Several checks read
lockdep's per-thread held-set mirror; with lockdep off those reads would see
always-zero counts and the checks would *vacuously pass* — the forbidden
silent-degrade. The cargo feature graph must make `cswitchdep` imply `lockdep`
(or the build must hard-fail on the combination), stated in Build wiring below.

## Why — the bug classes it turns from Heisenbugs into pinpointed reports

cswitchdep is aimed squarely at the crash families the field manual has already
paid for in debug hours, all of which are context-switch/transition invariant
violations that currently corrupt state at the switch and crash somewhere else:

- **Sleep-while-holding-a-spinlock (the general form of §9.13).** A voluntary
  switch while a *dispatch-level* lock is held. Note carefully what is and is
  not a violation: blocking while holding a **pushlock or guarded mutex** is
  legal and routine in NT (a thread holding WS pushlock A legitimately parks
  acquiring pushlock B — that nesting is exactly what populated lockdep's 6K+
  edge pairs on a real boot, `lockdep.rs:56-62`). The violation is a held
  **spinlock** (IRQL ≥ DISPATCH lock) at a wait. lockdep's mirror today covers
  only pushlocks/guarded mutexes (`ntkrnl-sys/src/inline/ex.rs`, `inline/ke.rs`)
  — so this check needs **new spinlock taps** (see Invariants).
- **Leaked-IRQL-raise class (§9.24).** A raise that never lowers, so a later
  lower drops through a critical section. cswitchdep's **tap-1 entry check ("a
  voluntary switch begins at exactly DISPATCH")** catches a leaked-raise thread
  at its next wait — the same signal the arm64 `halstubs.rs` self-check prints,
  but tied to the switch that carried the bad IRQL. (Historical note, 2026-07-06:
  the arm64 TDI 0x1E that motivated this was re-diagnosed — its `raise-DOWN 15→2`
  spam is a *crash artifact* of BugCheck handling, and the real root is
  `KiUnwaitThread+0x28` dispatcher/KWAIT_BLOCK corruption. The IRQL check stays
  — the leak class is real generically — but the *coherence* checks below are
  the ones aimed at that scheduler-corruption family.)
- **arm64 wake-from-exception-context wedge (§9.19, EC=0x0E / 0x1E).** Running
  the thread-wake/DPC machinery from an IRQ exception window instead of genuine
  thread context. The `HalpInterruptContextDepth == 0` gate lives in
  `HalpDispatchSoftwareInterrupts` (`hal-rust/src/arm64/halstubs.rs:816`);
  `KiDispatchSoftwareIntArm64` itself gates on `DpcRoutineActive` (`dpc.rs:658`).
  cswitchdep asserts both at the switch and flags a switch initiated with a
  non-zero interrupt-context depth.
- **Offset-drift coherence bugs (the 0x50/0xBE `KTHREAD.Teb`/MMPFN class).** A
  switch that leaves `KPCR→CurrentPrcb→CurrentThread` inconsistent with the
  KTHREAD actually running (stale current-thread, wrong stack). cswitchdep's
  post-switch coherence check catches the *effect* of an offset bug at the
  switch, complementing the compile-time `offset_of!` asserts that catch the
  *declaration*.
- **swapgs parity (amd64).** A missed or doubled `swapgs` on the user↔kernel
  boundary — a classic silent corruptor. Modeled **per trap frame**, not per
  CPU (see Invariants — the v1 per-CPU bit could not survive nested interrupts
  or NMIs). There is no arm64 analogue: `tpidr_el1` is set once, x18 is
  saved/restored through the trap frame like any GPR, and TTBR0/TTBR1 split
  means nothing swaps at kernel entry — the arm64 check is instead
  "`TfX18 == Thread->Teb` at `eret`", which `KiTrapReturnProbe` half-does today.
- **`ATTEMPTED_SWITCH_FROM_DPC`.** Already a hard BugCheck in
  `KiSwapContextResume` (`amd64/thrdini.rs:232`, gated on `DpcRoutineActive`,
  `:230`); cswitchdep subsumes it into the general "switch legality at this
  context" check and reports it uniformly.

The through-line: these all *corrupt at the switch and crash later*. Arming the
switch itself is the same philosophy as the special-pool tripwire — fault at the
corrupting store, not the innocent victim.

## The model

Two shadow tables, exactly mirroring lockdep's split:

- **Per-CPU shadow** (`CpuShadow`, indexed by processor number — a 1-entry array
  today; the kernel is uniprocessor, `KeGetCurrentProcessorNumber()` ≡ 0,
  `ntkrnl-sys/src/inline/ke.rs:157`. **The UP assumption is load-bearing** for
  the lock-freedom claims here; the SMP generalization is per-CPU arrays, which
  the index already models). Holds the *live* execution-context facts: expected
  IRQL, held dispatch-lock count, interrupt-context depth, and the current
  kernel-stack `[base,limit)`.
- **Per-thread shadow** (`ThreadShadow`, open-addressing hash keyed by KTHREAD
  address, free-on-empty — lockdep's `threads[]` table in shape, plus an
  explicit lifecycle, below). Holds the facts that must be *saved and restored
  with the thread across a switch*: the IRQL observed at its last suspend, its
  held-lock counts at suspend, and a generation counter.

A switch is modelled as **suspend(old) → handoff → resume(new)**. cswitchdep
snapshots the per-CPU shadow into `old`'s `ThreadShadow` at suspend, and checks
`new`'s `ThreadShadow` against the per-CPU shadow at resume. Any field that
should be balanced but isn't is a violation recorded with the initiating PC.

### Reentrancy — the shadow must be saved/restored across nested traps

This is where cswitchdep is categorically more race-sensitive than lockdep, and
v1 under-specified it. Voluntary switches run at DISPATCH via CR8 with **IF=1**
— device interrupts are deliverable throughout the switch path. If taps on trap
entry/exit mutate the per-CPU shadow, an ISR firing between tap 1's snapshot and
the SP swap tears the snapshot and produces false reports. (lockdep is not
exposed the same way: its facts are per-(thread,lock) monotone sets updated only
from thread context; its one acknowledged race is the benign free-on-empty CAS,
`lockdep.rs:472-476`. cswitchdep asserts *equalities across time on shared
per-CPU cells* — a different animal.)

The rule: **nested trap entry/exit save and restore the CpuShadow deltas like a
register.** The trap-entry tap pushes the interrupted context's shadow fields
into the (new) trap frame's tap record and installs its own; the trap-exit tap
restores them. The shadow thus always describes the *currently executing*
context, at every nesting depth. Additionally, tap-1's suspend snapshot is
built in locals and published with a single `Release` store of `gen` — never
read-modify-written in place.

## Instrumentation points

**Five taps** (v1's tap 2 — a call out of `KiSwapContextInternal` at the SP swap
— is deleted: at that point on amd64 the live values are all in caller-saved
registers (`cl`=WaitIrql, `rdx`=OldThread, `r8`=NewThread), and on arm64 a `bl`
destroys `x30` *which is the just-reloaded switch-frame return address* — a
bricked resume; and both checks it proposed were tautologies: the `SwapBusy`
spin sits immediately above the swap, and NewThread is *loaded from*
`Prcb.CurrentThread` on both arches, so "CurrentThread == NewThread" cannot fail
by construction. Taps 1 and 3 bracket the swap on both stacks already, including
first-run threads — the KSWITCH_FRAME return path goes through
`KiSwapContextResume` before `KiThreadStartup`. If a mid-swap probe is ever
genuinely needed, spec it as an asm-local compare + `brk`/`int3`, never a call.)

Rust points get a `#[cfg(feature = "cswitchdep")]` call; the remaining asm
points (trap entry/exit) get a call to a `#[no_mangle]` bridge (the link-name
tap pattern lockdep uses), **placed only where a full kernel stack and trap
frame exist**.

| # | Tap | Site (file:line) | Lang | What cswitchdep checks |
|---|-----|------------------|------|------------------------|
| 1 | `KiSwapThread`/`Impl` — **entry/exit pair** | entry: `ke/thrdschd.rs:443/461`; exit: after the `KeLowerIrql(WaitIrql)` tail (`thrdschd.rs:520-528`) | Rust | **entry:** IRQL ∈ {DISPATCH, SYNCH} (a leaked-HIGH thread trips here — §9.24); snapshot old context; held **spinlock** count == 0 (pushlocks/gmutexes exempt — legal to hold across a park). **exit:** post-lower IRQL == `WaitIrql` (the balance check v1 mis-placed at tap 3 — the switch machinery itself never touches IRQL; the caller lowers *after* KiSwapContext returns) |
| 2 | `KiSwapContextResume` (post-switch, new stack) | `amd64/thrdini.rs:195`, `arm64/thrdini.rs:126` | Rust | **primary hook**: IRQL still == the tap-1 switching level (that *is* invariant across the swap); not in DPC (`DpcRoutineActive`, subsuming `thrdini.rs:230-232`); post-switch coherence (`KPCR→CurrentPrcb→CurrentThread == new`, SP within `new`'s stack bounds); first-run detection via ThreadShadow birth brand (below). Note: `SwapBusy` release happens *inside* this function (`thrdini.rs:242`), after the tap's entry — the tap does not assert it |
| 3 | `KiDispatchInterrupt` / `KiDispatchSoftwareIntArm64` | `i386/thrdini.rs:444`, `dpc.rs:656` | Rust | DISPATCH_LEVEL; DAIF/IF state equal at entry vs exit (the function brackets with `_disable`/`_enable`, `dpc.rs:657-688` — measurable); **arm64:** `HalpInterruptContextDepth == 0` (the §9.19 gate — lives in `HalpDispatchSoftwareInterrupts`, `halstubs.rs:816`) and `DpcRoutineActive == 0` (`dpc.rs:658`) |
| 4 | User→kernel entry | amd64: **after** the trap frame is built and volatiles saved (≈`trap.S:822+`, i.e. after `mov rsp, gs:[PcRspBase]` at `:818` — **not** at the `swapgs` itself, `:812`, where rsp is still the *user* stack and a call would execute kernel code on user-controlled memory); arm64 `TRAP_PROLOG` body (`trap.S:48-50`) + `KiSvcDispatch` `trap.rs:173`; i386 `_KiFastCallEntry` `trap.s:149` after the kernel-stack switch | asm+Rust | swap-parity recorded **by position** (the tap is only reachable post-swapgs on the syscall path) into the *trap frame's* tap record; entry IRQL == PASSIVE for a user syscall; shadow push (Reentrancy rule) |
| 5 | Return→user | amd64 `KiServiceExit3` `trap.S:1008-1022` (before the swapgs at `:1022`), arm64 `KiTrapReturn` `trap.S:167` (`eret` `:241`) | asm | per-frame parity closed exactly once; IRQL == PASSIVE and **held-lock count == 0, all classes** on user return (here the all-classes form *is* the invariant); arm64: `TfX18 == Thread->Teb` (extends `KiTrapReturnProbe`, `trap.rs:80`); shadow pop |

Points 1/2 are the workhorses (thread↔thread); 4/5 cover user↔kernel; 3 covers
the deferred-switch delivery that closed the arm64 timer-freeze landmine.

### Switch classes the tap map must also cover

- **Idle→thread switches bypass tap 1.** arm64 `KiIdleLoop` calls
  `KiSwapContext` directly (`arm64/thrdini.rs:234/243`); the x86 idle loops
  likewise. A resume whose old side has no suspend record is therefore *normal*
  for idle. Rule: **missing suspend record ⇒ skip the balance checks, count the
  occurrence** (a census line, not a violation). Optionally tap `KiSwapContext`
  itself later (Rust-visible on arm64; on amd64 tap its C-visible callers).
- **`KiSwapProcess` / `KeStackAttachProcess`** change CR3/address space with
  *no thread switch*. Any address-space coherence check must treat these as
  legal writers — tap `KiSwapProcess` to update the shadow, or don't check CR3
  at all in phase 1.
- **No lazy-FPU tracking.** v1 carried an `fpu_owner` field; this port has no
  lazy FPU owner to track — amd64 does eager save/restore inside
  `KiSwapContextResume` (`thrdini.rs:211-216`) and arm64 has no FPU handling in
  the switch. The field is deleted. (If anything, a later check is "NpxState
  save/restore executed when NpxState ≠ 0" — out of scope for phase 1.)

### Spinlock taps (P2 — as built, with the coverage reality)

lockdep instruments pushlocks/guarded mutexes only. The sleep-with-spinlock
check is fed by a per-CPU **dispatch-lock counter** tapped at the one choke
point every ntoskrnl Rust funnel delegates to: the `KxAcquireSpinLock`/
`KxReleaseSpinLock` inlines (`ntkrnl-sys/src/inline/ke.rs`, exported via
`shim_exports.rs`), plus the UP try-funnel success path
(`KeTryToAcquireSpinLockAtDpcLevel`, which never reaches Kx). Link-name taps
(`VibeosCswitchdepSpin*`) mirror lockdep's pushlock plumbing exactly.

**Coverage reality, measured on a real armed boot (2026-07-07):**
- `Kx*SpinLock` also exist as **C FORCEINLINEs**
  (`ntoskrnl/include/internal/spinlock.h`) — still-C kernel code inlines its
  own invisible copy. Pure-C pairs are symmetric-invisible; **mixed pairs**
  (C-inline acquire → Rust-funnel release, the split acquire/release shape)
  skew the absolute count: ~−10.4K over an amd64 boot, then stable. The
  counter is therefore a live gauge of the C↔Rust funnel seam, and the
  `SLEEP_WITH_SPINLOCK` check fires on a **delta above a post-boot baseline**
  captured at `cswitchdep_init` — never on the absolute count. The check
  sharpens automatically as translation eliminates mixed pairs.
- The violation is **amd64-only**; on i386/arm64 several funnels route through
  the HAL binary (untapped, pair-symmetric) and the counter is census data.
- **arm64 first light paid immediately (2026-07-07).** The first armed arm64
  boot ever tripped CURTHREAD_INCOHERENT on EVERY switch — and it was a REAL
  latent bug, not tap noise: `KiSwapContextResume` (Rust) resolved
  `ddk::KeGetPcr` to the x18-convention C copy (`mov x0, x18`), and x18 is
  not reloaded by the switch frame, so fresh-off-the-switch it read NULL —
  meaning the `KeContextSwitches` bump, the `DpcRoutineActive` BugCheck
  probe, and tap 2 had been silently dereferencing `NULL+offset` since the
  function was translated. Fixed to the TPIDR_EL1 inline (the invariant
  source ctxswitch.S itself uses). Related first-light calibrations: the
  IDLE thread is exempt from STACK_OOB (boot stack; bounds exclude the
  resume frame), and the tap-3 DAIF check is leaked-masking-only
  (`exit & !entry` — the dispatch legitimately exits unmasked, §9.20).
  Feeds the KeGetPcr cohab-dedup task: an x18 KeGetPcr is unsafe in switch
  context, kill it in favor of TPIDR.
- **All three arches arm on their native channels (2026-07-07).** The
  culprit-PC capture was the only nightly dependency
  (`feature(link_llvm_intrinsics)` → E0554 on arm64's stable builtin target);
  it is now the stable `code_addr!()` inline-asm macro (`ke/lockdep.rs`,
  crate-exported): PC-at-expansion via `lea reg,[rip]` (amd64) /
  `call 2f; 2:; pop reg` (i386) / `adr reg,.` (arm64), threaded as a
  `pc: usize` parameter through every tap signature. **Two prior designs are
  dead — don't resurrect them:** `llvm.returnaddress` (nightly-only), and a
  naked-fn frame-record shim reading `[rbp+8]` — the latter *looked* sound
  but returned 0 on amd64 because Windows-x64 clang does NOT build a classic
  frame chain even with `"frame-pointer":"always"` (it `push`es then
  `lea rbp,[rsp+K]`, so `[rbp+8]` is a spill slot, not the return address;
  the selftest's pc=0 was the tell). `code_addr!()` gives the *call-site* PC
  rather than the caller's return address — one frame lower, equally
  symbolizable, zero ABI assumptions. The old `vibeos-cli build` arm64
  fail-fast is lifted. (Taps 4/5 remain x86-scoped — arm64 has no
  syscall-path taps yet; taps 1/2/3 + census run everywhere.)
- Sacred primitives: the taps sit below the `KeAcquireSpinLock`/
  `KeReleaseSpinLock` named exports (which `verify_sacred_primitives.sh`
  checks), so the named symbols stay intact; the check is re-run on armed
  builds. `KiAcquirePrcbLock` is a UP no-op inline
  (`ntkrnl-sys/src/inline/ke.rs:969`) — the scheduler's own lock is invisible
  and stays so (the scheduler holding its own lock across the switch is the
  design, not a bug).

## Shadow-state layout

Mirror `lockdep.rs` exactly: one exported `#[no_mangle] static VIBEOS_CSWITCHDEP:
CswitchTable` living in `.bss` (all-zero at build → **zero image cost off**),
lock-free CAS/atomic-store updates, publish-last ordering (`Release` store of a
`seq`/generation last) so an external decoder never reads a torn snapshot,
overflow *counted* never silent, and a `violations[16]` ring of recorded
culprits. Compile-time `assert!`s pin the layout so drift breaks the *build*,
not the decoder (lockdep `:173-179`).

```rust
#[repr(C)]
struct CpuShadow {          // per processor (1 today; array indexed by Number)
    expected_irql: u8,      // shadow of CR8 / HalpCurrentIrql
    held_spinlocks: u16,    // NEW dispatch-lock counter (spinlock taps)
    held_passive: u16,      // mirror of lockdep's pushlock/gmutex held count
    intr_ctx_depth: u16,    // HalpInterruptContextDepth snapshot (arm64 gate)
    stack_base: usize,      // current kernel stack [base, limit)
    stack_limit: usize,
    switches: u64,          // KeContextSwitches shadow, for the census
}

// Per-frame parity + the nested-trap shadow save live in the TAP RECORD pushed
// at trap entry (tap 4) and popped at exit (tap 5) — NOT in CpuShadow. A
// per-CPU parity bit cannot model conditional swaps: EnterTrap/ExitTrap
// (trapamd64.inc:148/:221) swapgs conditionally on KTRAP_FRAME.SegCs & 1, so
// correctness is per-trap-frame; and an NMI (KiNmiInterrupt, trap.S:215)
// landing inside the syscall-entry window would desync any global bit with no
// bug present. NMI frames are exempt from the parity check in phase 3.

#[repr(C)]
struct ThreadShadow {       // open-addressing, keyed by KTHREAD*; lifecycle below
    kthread: usize,         // 0 = empty slot
    birth: u32,             // brand written at KiInitializeContextThread
    suspend_irql: u8,       // IRQL observed at last suspend (tap-1 entry)
    held_at_suspend: u16,   // spinlock count captured at suspend
    gen: u32,               // bumped each suspend; publish-last
}

#[repr(C)]
struct Violation {          // ring; decoded by debug_block.py (magic 'CSW1')
    kind: u16,              // SLEEP_WITH_SPINLOCK | IRQL_UNBALANCED_SWITCH |
                            // HIGH_IRQL_AT_SWITCH | FRAME_PARITY |
                            // SWITCH_FROM_DPC | INTR_CTX_SWITCH | STACK_OOB |
                            // CURTHREAD_INCOHERENT | USER_RETURN_DIRTY
    arch_pc: usize,         // culprit: returnaddress(0) (x86) / x30 (arm64)
    kthread_old: usize,
    kthread_new: usize,
    expected: u64,
    actual: u64,
    seq: u32,               // publish-LAST (lockdep Violation.seq pattern,
                            // lockdep.rs:123/:572) — v1 promised this ordering
                            // but omitted the field
}
```

### ThreadShadow lifecycle (v1 had none — the table would leak and misfire)

A ThreadShadow is never "empty" the way lockdep's held-set is (every thread that
ever switched has state forever), and KTHREAD addresses are pool-reused — a new
thread inheriting a dead thread's slot would inherit its `held_at_suspend` and
false-positive with no bug. Two rules:

- **Brand at birth.** `KiInitializeContextThread` (Rust on both arches:
  `amd64/thrdini.rs:94`, `arm64/thrdini.rs:62`) initializes the shadow slot —
  evicting any stale slot at that KTHREAD address first. Tap 2 treats a
  fresh-branded, never-suspended shadow as **first-run**, not a violation.
- **Evict at retirement.** Tap 1 observing `old.State == Terminated` (the final
  switch away from a dying thread) frees the slot; belt-and-braces, a tap in the
  thread reaper does the same. Occupancy is a census line so a leak is visible,
  never silent.

The context-switch-touched field offsets it reads
(`KPCR.CurrentPrcb`→`KPRCB.CurrentThread`→`KTHREAD.{WaitIrql,KernelStack,Teb}`)
are already enumerated with `offset_of!` in `debugblock.rs:110-148` — cswitchdep
reuses that inventory rather than hand-rolling numeric offsets (the
`scan_magic_offsets.py` rule), so the existing `offset_check_*.rs` asserts cover
it automatically. (`KTHREAD.WaitIrql` is bindgen-visible and offset-asserted
today: `offset_check_i386.rs:4214`.)

## Invariants (the assertion catalog)

Grouped by switch class:

**Voluntary thread↔thread (wait-driven, taps 1/2):**
- tap-1 entry IRQL ∈ {DISPATCH, SYNCH} — a thread arriving at a wait with
  IRQL > SYNCH is the leaked-HIGH class (`HIGH_IRQL_AT_SWITCH`, §9.24).
- held **spinlock** count == 0 at the switch (else `SLEEP_WITH_SPINLOCK`,
  culprit = the acquire site from the spinlock-tap held set). Held
  pushlocks/guarded mutexes are **legal** here — recorded in the census, never
  a violation.
- tap-1 exit: post-`KeLowerIrql` IRQL == `WaitIrql` (else
  `IRQL_UNBALANCED_SWITCH`).
- tap 2: IRQL unchanged across the swap (== tap-1 switching level); not in DPC
  (`SWITCH_FROM_DPC`); `KPCR→CurrentPrcb→CurrentThread == new`; SP ∈ `new`'s
  `[KernelStack…StackBase)` (else `CURTHREAD_INCOHERENT` / `STACK_OOB`).
- old side missing (idle-loop entry) ⇒ skip balance, count.

**Involuntary / deferred (tap 3):**
- IRQL == DISPATCH_LEVEL; DAIF/IF equal at entry vs exit.
- **arm64:** `HalpInterruptContextDepth == 0` at delivery (else
  `INTR_CTX_SWITCH` — the §9.19 wedge).

**User↔kernel (taps 4/5):**
- per-frame parity opened exactly once at entry, closed exactly once at return
  (else `FRAME_PARITY`); NMI frames exempt.
- on user return: IRQL == PASSIVE and held-lock count == 0 **across all lock
  classes** (else `USER_RETURN_DIRTY`).
- arm64: `TfX18 == Thread->Teb` at `eret`.

## Reporting & symbolication

Identical to lockdep's contract. The culprit PC is captured as
`llvm.returnaddress(0)` (x86) / `x30` (arm64) *inside* the `#[inline(never)]`
bridge shim (lockdep `:650-667`), stored raw in the `Violation` ring. Loud
cases (`SLEEP_WITH_SPINLOCK`, `INTR_CTX_SWITCH`, `HIGH_IRQL_AT_SWITCH`)
`DbgPrint` immediately — **rate-limited with the SWAP-TRIPWIRE's 8-shot cap**
(`arm64/thrdini.rs:152-161`): a recurring false positive must not flood serial
and wedge the harness (lockdep deliberately keeps its record path print-free
for the same reason, `lockdep.rs:31-32`).

```
[CSWITCH] SLEEP_WITH_SPINLOCK old=%p new=%p held=1 site=%p
[CSWITCH] HIGH_IRQL_AT_SWITCH irql=15 thread=%p pc=%p   (§9.24)
[CSWITCH] INTR_CTX_SWITCH depth=1 pc=%p                  (arm64 §9.19)
```

A PASSIVE-level `cswitchdep_report()` prints the census (mirroring
`lockdep_report()` `:591-643`). Raw PCs resolve offline with `qemu/addr2sym.sh
<arch> --reloc <base> <pc>` or online with `qemu/inspect.sh symbolicate-pc`. A
`qemu/gdb-helpers/debug_block.py` decoder (magic `'CSW1'`, offsets pinned by the
compile-time layout asserts) dumps the live tables: `vibeos-cswitch [cpu|threads|
violations]`.

Per-boot knobs ride the existing fw_cfg tripwire channel (`tripwire.rs`,
`opt/org.vibeos.tripwire`, ports `0x510/0x511` x86 / MMIO arm64): e.g.
`CSWITCH=irql,parity` to arm only a subset, `CSWITCH_SELFTEST=1` to prove the
detector barks (deliberately switch with a held spinlock → expect the report).
No boot-media rebuild; no-op on real hardware.

## Build wiring

Copy lockdep's flag plumbing (`sdk/cmake/gcc.cmake:642-651`): `-DVIBEOS_CSWITCHDEP=ON`
sets cargo env `VIBEOS_CSWITCHDEP=1` for `ntoskrnl-cargo`/`ntkrnlmp-cargo`, whose
`build.rs` add `--features cswitchdep` to the inner `kernel-rust` build; fronted
by `vibeos-cli build --arch <a> --cswitchdep`. The whole module is
`#[cfg(feature = "cswitchdep")]`. **The cargo feature declares
`cswitchdep = ["lockdep"]`** so the held-set mirror is always present (fail-loud
rule: never check against an absent mirror).

## Cost model

- **Off (default):** module `#[cfg]`'d out; table is zero-init `.bss` → **zero
  image, zero runtime**; smoke stays clean.
- **On:** taps 1/2 add a hash probe + a few atomic stores per *switch* — cheap.
  Taps 4/5 fire per **syscall**: hash probe (fib hash, ≤16 probes, atomic loads)
  + several stores + the asm save/restore bracket ≈ order 100–200 guest
  instructions per syscall — likely single-digit-% boot overhead under TCG, but
  that is an **estimate, not a measurement**: phase 3 gates on a measured
  boot-time delta before the taps are considered soak-safe. No allocation, no
  global lock. The per-thread table is bounded by the lifecycle rules above.

## Relationship to existing tripwires

cswitchdep **generalizes and unifies** the point tripwires the kernel already
grew reactively:
- `KiTrapReturnProbe` (arm64 EL0-state validation before `eret`, `trap.rs:80`) →
  becomes cswitchdep tap 5.
- `ATTEMPTED_SWITCH_FROM_DPC` BugCheck (`thrdini.rs:230-232`) → tap 2
  `SWITCH_FROM_DPC` (recorded, not necessarily fatal, so a farm soak can collect
  more than one).
- arm64 DAIF "masked-swap" tripwire (`arm64/thrdini.rs:145-163`) → tap 2
  interrupt-state check (and its 8-shot print cap is the reporting template).
- arm64 IRQL self-check (`halstubs.rs:864-945`, the §9.24 lead) → its
  raise/lower-imbalance signal becomes the tap-1 entry IRQL check, now
  attributable to a specific switch.

Each of those was a one-off `if (bad) print/bugcheck`; cswitchdep is the single
side-table framework they should have been, with a decoder and a census.

## Validation & relationship to the static model

cswitchdep is the *dynamic* half of a two-layer strategy. The *static* half
(`execution-context-ownership-model.md`) makes "no sleep with a spinlock," "no
per-CPU pointer across a switch," and "IRQL balanced" **compile errors** in the
Rust-translated paths. cswitchdep:
1. **Validates the static model's assumptions** on the C/asm remnant the type
   system can't see (the SP swap, the swapgs, the still-C drivers).
2. Is the **runtime backstop** for the cases the static analyzer marks
   `unchecked` (data-dependent lock acquisition — klint's known `try_lock` gap),
   for the ~24 split acquire/release sites the token model bridges but does not
   verify, and for FFI-reminted tokens (see the static doc's minting section).
3. Becomes progressively **redundant** as ntoskrnl Rust coverage grows and the
   invariants move from runtime asserts to "won't compile" — the same trajectory
   as the captures(none) class going from a runtime landmine to a toolchain
   guarantee.

## Phasing

1. **Table + taps 1/2 (thread↔thread) on amd64** — the tap-1 entry-IRQL check
   (`HIGH_IRQL_AT_SWITCH` — immediate §9.24 value), the tap-1-exit `WaitIrql`
   balance, and the tap-2 coherence checks. **No spinlock dependency yet** —
   `SLEEP_WITH_SPINLOCK` waits for phase 2. Includes the ThreadShadow lifecycle
   (birth brand + eviction) and the idle-loop skip rule from day one.
2. **Spinlock taps + `SLEEP_WITH_SPINLOCK`; tap 3 + the arm64
   interrupt-context-depth gate** — folds in the §9.19/§9.24 arm64 leads;
   re-run `verify_sacred_primitives.sh` with the feature on.
3. **Taps 4/5 (user↔kernel) + per-frame parity** — all three arches; includes
   the nested-trap shadow save/restore and the NMI exemption; gated on a
   measured boot-overhead delta.
   **BUILT (2026-07-07), scoped to the SYSCALL paths on x86** (arm64 has no
   4/5 tap sites yet — its other taps arm fine since the stable shim;
   interrupt/exception frames deliberately untapped — the taps
   mutate only per-thread slots + monotone counters, so the reentrancy
   save/restore rule and the NMI exemption are *vacuously* satisfied instead
   of implemented). As-built notes:
   - **The live syscall paths are Rust + cargo-assembled asm, not the cmake
     C/asm.** Tap 4 sits in `traphandler.rs::KiSystemCallHandler` (amd64) and
     `traphdlr.rs::KiSystemServiceHandler` (i386, kernel-origin INT-2E frames
     skipped — Zw at APC_LEVEL is legal); tap 5 in amd64 `trap.S`
     `KiSystemServiceExit` (`#ifdef VIBEOS_CSWITCHDEP`, plumbed through
     `ntoskrnl-cargo/build.rs asm_extra_defines` — cmake's define never
     reaches the live trap.S, which the first armed boot proved with
     `user_entries=0`) and i386 `KiServiceExit`/`KiServiceExit2` user
     branches. The commented-out C copies carry mirror taps for parity.
   - **Violations:** `USER_ENTRY_IRQL` / `USER_EXIT_IRQL` (IRQL != PASSIVE at
     a user-mode syscall edge — the §9.24 leak caught one frame earlier).
   - **Per-frame parity is census-only** (P2 playbook: measure, then
     promote): `ThreadShadow.syscall_depth`/`held_at_entry` +
     `CpuShadow.user_entries/user_exits/exit_no_entry/held_delta_pos`.
     Untapped exits (`KiServiceExit3` first-returns, APC-edited frames,
     termination) make depth diagnostic-only; `exit_no_entry` counts the
     benign orphans.
4. **`debug_block.py` decoder + fw_cfg arming + farm integration** — armed
   overnight soaks cluster context-switch violations the way `farm-report`
   clusters BUGCHECKs.
   **BUILT (2026-07-11).** As-built:
   - **Decoder:** `vibeos-cswitch [threads]` in `qemu/gdb-helpers/debug_block.py`
     (magic `'CSW1'`, offsets pinned by the `ke/cswitchdep.rs` layout asserts) —
     dumps the CpuShadow census, the syscall-parity counters, the violation
     ring (all 11 kinds), and `threads` walks the live ThreadShadow slots.
   - **fw_cfg arming:** `CSWITCH_SELFTEST=1` rides the tripwire channel
     (`tripwire.rs` → `KiStartSyncCensus` → `cswitch_selftest()`); fail-loud if
     armed on a non-`--cswitchdep` build. (Per-check subset arming
     `CSWITCH=irql,parity` was **deliberately NOT built** — it would put a
     runtime-mask branch on every hot-path tap to shave a soak-overhead the cost
     model already rates single-digit-%; the taps stay always-on-when-built.
     Cheap to add later if a measured overhead ever justifies it.)
   - **Farm integration:** `triage.wedge_evidence` counts `[CSWITCH]`/`[LOCKDEP]`
     serial reports; `smoke.py` surfaces `{n} [CSWITCH] on a green boot`;
     `farm.py` carries `cswitch_hits`/`census_lines` into the corpus and
     `farm-report` grew a **"tripwire hits"** section that clusters them by
     (arch, kind) — because a cswitchdep violation pins its bug on an
     *otherwise-green* boot, so it never appears in the BUGCHECK failure-family
     table. The soak now reports context-switch violations even when nothing
     crashed — the whole point.

## Resume checklist — "stable-everything phase 1" landing (parked 2026-07-10)

The changeset is UNCOMMITTED in the `cswitchdep-phase1` worktree
(`~/Projects/os-playground/vibeos-cswitchdep`), rebased-by-merge onto
2026-07-10 master (merge `1c646788398`, clean, no conflicts). Parked only
because the host was oversubscribed on builds — no known defect. Contents:
`code_addr!()` + `link_llvm_intrinsics` removal + `pc: usize` threading
through all taps; arm64 first-light fixes (TPIDR `KeGetPcr`, idle STACK_OOB
exemption, DAIF leaked-masking-only); SwitchSafe rejection + the
`CTX_CAPTURE_ACROSS_WAIT` lint (`tools/scan_irql_balance.py`); doc updates
(this file + execution-context-ownership-model.md v3).

Validation state at park time:
- **amd64: VALIDATED.** Armed boot green; `CSWITCH_SELFTEST` fires with a
  non-zero symbolizable pc (landed in the `cswitch_selftest` region);
  syscall parity clean (~58.5K entries / ~57.7K exits, `held_delta_pos=0`,
  violations=0).
- **arm64: VALIDATED (2026-07-10, on the merged tree).** Armed boot green in
  36s; census `switches=1788 violations=0 first_runs=808 no_record=0
  drops=0` — all three first-light fixes confirmed (the pre-recalibration
  boot had shown violations=210, all INTR_STATE_UNBALANCED under the old
  strict `entry != exit` DAIF check). ~~`user_entries=0` as expected (no
  arm64 taps 4/5)~~ — **taps 4/5 landed on arm64 2026-07-11** (wrapping
  `KiSvcDispatch` in `ke/arm64/trap.rs`, EC=0x15; user-origin =
  `Spsr.M == EL0`, kernel-origin SVC skipped per the i386 INT-2E Zw rule;
  tap 5 pairs at dispatch return, not the shared ERET tail). First light:
  `user_entries=15646 user_exits=15109 exit_no_entry=0 held_delta_pos=0`,
  violations=0 — the entry>exit delta is the documented untapped-exit
  class. All three arches now run all five taps. `held_now` is negative
  census data (arm64 funnels
  through untapped HAL paths — pair-asymmetric; the SLEEP_WITH_SPINLOCK
  check is delta-above-baseline, unaffected).
- **i386: parity validated pre-merge** (58,653/58,653, violations=0); not
  re-run since.

Landing sequence (in order):
1. Armed arm64 build+smoke on the merged tree; read the `[CSWITCH]` census
   in the serial log — expect violations=0 (or near-0, real findings only).
2. If clean: update agent MEMORY.md (arm64 lockdep E0554 blocker CLOSED via
   `code_addr!()`; cswitchdep/lockdep arm on all three arches), then commit
   (staged-paths-only + conflict-marker check — shared-tree discipline).
3. Full `./vibeos-cli gate` ×3 → push on green per pre-authorization
   (report the push).
4. Follow-ups already tracked: KeGetPcr x18 cohab dedup (task #2, urgency
   bumped — x18 copies are unsafe in switch context); arm64 taps 4/5;
   remaining nightly deps (-Z build-std/JSON targets, c_variadic, linkage)
   per the ownership doc v3 toolchain section.
