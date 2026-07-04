# Sync/Scheduler Instrumentation — Design Notes

**Why this doc exists.** The expensive kernel hunts (the amd64 Mm-pushlock
service-startup wedge, the i386 BAD-gate corruption, the WHPX CR8 collapse) all
shared three deficits: (1) *invisible state* — nothing records who holds a
pushlock or who was supposed to signal a gate; (2) *invisible history* — a lost
wakeup is an ordering bug, and by the time we attach the interesting event is
hours old; (3) *per-question tooling cost* — DWARF is dead on the Rust kernel,
so every KTHREAD question costs a hand-derived offset that rots on layout
changes. Now that all sync primitives are our Rust source in one workspace, we
can fix this structurally. This doc parks the design so it isn't lost between
sessions; items are tagged with status.

**Hard constraint for everything below:** struct layouts are ABI-frozen
(~1,500 compile-time size assertions, C interop). All instrumentation must be
side-table / per-CPU / global-ring based. Never grow a kernel object.

**Build shape:** one cargo feature (e.g. `sync-trace`) on ntoskrnl-rust with
tiers — Tier 0 always-on (counters, last-owner word: pennies), Tier 1 default
for smoke builds (ring + watchdog + shadow IRQL — the north-star build should
be the instrumented build), Tier 2 opt-in (lockdep).

---

## 1. Sync flight recorder (in-memory ring buffer) — PLANNED, highest value

Global lock-free ring (~64K entries, per-CPU sharded), recording every sync
event: `{TSC, cpu, thread, op, object, caller-RA}` with op ∈ {acquire, release,
wait-start, wait-satisfied, signal, ready-thread, swap-thread}. Relaxed
atomics, ~10–20ns/event, **no serial I/O** (prints perturb timing and mask
races — the arm64 winsrv BAD-gate was a Heisenbug for exactly that reason).

Dump paths: automatically on BUGCHECK; on-demand serial command; offline via
GDB (well-known symbol + versioned self-describing header → one decoder script
in `qemu/gdb-helpers/`, no DWARF).

Payoff: for a wedged gate, filter the ring by object address and read off the
last signal vs. last wait-start with the signaling thread and call site — the
exact question the Mm-wedge lane spent days on.

## 2. Wedge self-census watchdog — PLANNED, do first with #3/#5

Kernel-side hung-task detector (Linux `khungtaskd` analog): periodic
DPC/worker walks threads; anything parked in GateWait/Waiting beyond N seconds
gets a serial census dump: image, TID, state, wait reason, wait object,
frame-pointer backtrace. The wedge self-reports into the smoke log — no GDB
attach, no `walk-procs.py` archaeology. Smoke triage's HANG_TIMEOUT verdict
then carries the stuck-thread list.

## 3. Wait/signal accounting on gates + pushlocks — PLANNED, trivial cost

Side-table per object address: `waits_started, waits_satisfied, signals_sent`
(three relaxed counters, open-addressing hash keyed by object address). A
wedged gate then reads as either *lost wakeup* (`signals == waits-1` present)
or *holder stuck* (no signal ever sent) — the discrimination that took the
Mm-wedge lane days of KTHREAD.State spelunking. Tier 0 candidate.

## 4. Lockdep-lite — PLANNED, biggest lift

Record lock-class acquisition pairs (A held while acquiring B) in a static
table; warn at first B→A inversion with both stacks. Converts "50% flaky
wedge" into "deterministic boot-time warning". Needs lock classes + per-thread
held-stack; do after #1–#3 prove the side-table plumbing.

## 5. Debugger data block — PLANNED, tiny effort, permanent payoff

What NT does with `KDDEBUGGER_DATA64`: export a `#[used]` static populated at
compile time via `offset_of!` — KTHREAD.State/WaitReason/KernelStack/
WaitBlock, EPROCESS links, PRCB fields, ring-buffer address (#1), census
config (#2). `qemu/gdb-helpers/*.py` read the block instead of hardcoding
offsets and never break on layout drift again.

## 6. Shadow IRQL + invariant asserts — PLANNED (user flagged as intriguing)

Debug-tier per-CPU shadow IRQL in memory, cross-checked against CR8 (amd64) at
raise/lower/spinlock boundaries. Would have caught the WHPX `kernel-irqchip`
CR8-zeroing bug at the first VM exit instead of 30s later at uniata's assert
(see `windows-host-qemu-whpx-spike.md`). Plus classic invariants in the Rust
wrappers: no KeWaitFor* at ≥DISPATCH, acquire/release IRQL pairing.

## 7. Loom models for dispatcher primitives — DEFERRED (2026-07-04, user call)

Extract the gate wait/signal handshake (and later pushlock wake) into a loom
model (`slist-loom` pattern) to exhaustively explore interleavings on the
host. Deliberately **not being spec'd now** — parked until the cheaper tools
above have had a shot at the open wedge. Note: the 2026-07-04 dispatcher audit
(below) found the UP primitives faithful to C, which lowers the prior that a
loom pass over them would find the wedge.

## 8. Enum/constant drift gate — PARTIALLY DONE 2026-07-04

New bug class found by the dispatcher audit: **translated files re-declare C
enum members as local `const`s and get the values wrong** (translator guessed
instead of reading the header). One sweep of `verify_constants.py --enums`
over the kernel crates — after adding `sdk/include/ndk/ketypes.h` to its
header catalog — found 15 mismatches, including `AdjustUnwait=3` (header: 1)
which silently killed the unwait priority boost for *every thread wakeup*, and
`ThreadedDpcObject=7` which collided with `GateObject`. All 15 fixed.

The catalog addition also fed `scan_inline_flag_literals` (the *tagged*-literal
checker), which immediately caught 4 more live call-site bugs the const-fixer
can't see: kernel32 console `NtWaitForMultipleObjects(…, 0 /* WaitAny */)`
(0 = WaitAll — waits for ALL init events), `KiSwapEvent` created as
NotificationEvent (arm64), `SeLsaInitEvent` created as SynchronizationEvent
(wake-one instead of wake-all — works today only because there's exactly one
waiter), and a Count=1-masked WaitAll in `MiZeroPageThread`. All fixed
2026-07-04.

Remaining work:
- The `--enums` finder is not a gate (FP-prone on generic names). Consider a
  scoped gate: kernel crates only, dispatcher-relevant enums only.
- Only `const NAME: T = value;` lines and `literal /* Tag */` comments are
  checked — UNtagged inline literals at call sites (`KeInitializeEvent(e, 1,
  0)` with no comment) still escape. A tree-sitter pass mapping
  known-signature call args to enum params would close that.
- Consider more NDK headers in the catalog (pstypes.h, obtypes.h, …) + a
  triaged baseline.

---

## 2026-07-04 dispatcher-primitive audit (context for all of the above)

Audited the Mm-wedge suspects (`docs` ref: MEMORY.md open-items) against the
reference C: **KiSwapThread, KiExitDispatcher, KiTryThreadLock, KiReadyThread,
KiDeferredReadyThread, KiUnwaitThread/KiUnlinkThread, KeWaitForGate/
KeSignalGateBoostPriority, KxQueueReadyThread/KiSelectReadyThread, the
KiSatisfy* family, KxSingleThread/Delay/Multi/QueueThreadWait, the UP lock
no-op family, guarded-mutex acquire/release protocol, amd64 KiIdleLoop, the
KiIdleSummary idle handshake.** Verdict: structurally faithful to C on the UP
configuration — EXCEPT the bugs fixed that day:

1. `ADJUST_REASON` constants (wait.rs setter = 3/invalid; thrdschd.rs reader
   swapped 1↔2) — every wakeup skipped the boost/quantum-recharge block;
   `KeSetEventBoostPriority` wakes took the wrong branch (over-boost to ~15).
2. `KxDelayThreadWait` ignored the caller's `Alertable` (all three copies:
   inline/ke.rs, shim_exports.rs, ke-macros) — non-alertable user Sleeps were
   alertable; alertable kernel delays were not (lost `KeAlertThread` wakes).
3. The 12 other enum-constant drifts from the sweep (some dead consts, some
   live: `InsertApcEnvironment`, `SynchronizationTimer` (po PerfTimer),
   `ThreadedDpcObject`, `EventPairObject`, `CurrentApcEnvironment`).

None of these is a confirmed root cause for the amd64 service-startup wedge
(waiters are GateWait-parked with the gate never signaled; these bugs distort
post-wake scheduling, not signal delivery) — but #1 deformed scheduler timing
on every wakeup since the translation landed, so re-measure the wedge rate
after it. The audit lowers the suspect list for the wedge to: genuine Mm
holder-stuck deadlock, or corruption (the BAD-gate family).
