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

**Second pass 2026-07-04 — the checker had bigger blind spots than the catalog.**
Auditing the checker ITSELF (prompted by "what else is missing?") found the
`parse_rust_const` regex silently ignored:
- **`pub const` / `pub(crate) const`** — the leading visibility kept ~27K
  kernel-crate consts entirely invisible. (The AdjustUnwait bug would have been
  undetectable had the translator written `pub const`.)
- **Path-qualified types** (`const X: ntkrnl_sys::ddk::ULONG = …`) — the `\w+`
  type pattern didn't match `::`.
- **Type-suffixed decimals** (`2u8`, `5usize`) — only hex-suffixed and bare
  decimals parsed.
- Signed C enum values (`-2`) compared against Rust's u32-masked parse.

Fixing those + adding the ndk type headers (pstypes/obtypes/mmtypes/extypes/
lpctypes/cmtypes) surfaced **41 more real constant bugs**, all fixed the same
day — headlined by the **LPC message-type enum** (LPC_DATAGRAM/PORT_CLOSED/
CLIENT_DIED/EXCEPTION/DEBUG_EVENT/ERROR_EVENT all shifted in `lpc/send.rs`,
where they gate `if MessageType < LPC_DATAGRAM || MessageType > LPC_CLIENT_DIED`
— i.e. a live LPC validation window off by the drift, in the csrss-CreateProcess
reply path the amd64 wedge sits in), the **`MEMORY_PRIORITY_FOREGROUND` writer/
reader disagreement** (ps wrote 0, scheduler read `==2` → foreground boost never
applied), swapped Ps pool-quota/priority enums, wrong Vad types (`VadAwe`/
`VadLargePages`/`VadDevicePhysicalMemory` — mixed in the same functions with
hardcoded-correct header literals, so provably the outliers), and ~10 wrong
hand-written STATUS codes in ntkrnl-sys/constants.rs.

**Trap the auto-fix walked into (and how it was caught):** `PsQuotaTypes` — a
PS_QUOTA_TYPE *max sentinel* used as a `QuotaEntry[]` loop bound. The flat
header parse counted the `NTDDI_LONGHORN`-gated members (`PsWorkingSet`,
`PsCpuRate`) that this build's NTDDI level excludes, so the catalog said 5 while
the real value (bindgen ground truth) is 3. `--fix` "corrected" 3→5, which would
have been an OOB read over the quota array. Caught by cross-checking all 41
fixes against `bindgen_amd64.rs` (compiled at the real NTDDI) before gating —
**always validate named enum maxes / version-conditional members against bindgen,
not the raw header.** Reverted + added to `ARCH_DEPENDENT_CONSTANTS`. Lesson for
the tool: the `--enums` catalog should ideally be built from bindgen output (NTDDI-
correct) rather than raw `#ifdef`-laden headers; until then, sentinel/max members
of version-gated enums are a known FP class.

Two more FP classes were identified and guarded, NOT fixed:
- **`TAG_*` pool tags** — Rust `u32::from_le_bytes(*b"…")` vs the checker's
  big-endian `parse_c_multichar` disagree on byte order; every tag
  "mismatches" though both spell the same 4 chars. Skipped (cosmetic pool-dump
  display; a byte-order-aware tag check is the proper follow-up).
- **`MMWSLE_PREVIOUS_FREE_MASK`** — two arch-dependent `#define`s
  (0x7FFFFFFF/0x7FF); added to `ARCH_DEPENDENT_CONSTANTS`.

**Third pass 2026-07-04 — the non-kernel crates + the tagged-literal class.**
Extending the sweep past the kernel (verify_constants now skips `bindgen_*`
files — they're generated ground truth; catalog disagreements there are
`#ifdef`-branch or generic-name-collision FPs) found **86 more const drifts**
across driver/usermode crates (acpi-rust's whole `CM_RESOURCE_*` block —
`CM_RESOURCE_PORT_IO=0` marked port resources as memory; scsiport/storport
DMA widths + inverted `Latched`/`LevelSensitive`; classpnp/ntdll-sys STATUS
codes; disk/cdfs/mountmgr/usbhub IOCTL/FSCTL wire values). All bindgen- or
header-verified before `--fix`; landed `4e175a23dbc`.

Then a two-agent C-reference triage of the 85 `scan_inline_flag_literals`
findings: **60 real value bugs** (38 garbage `TRACE_LEVEL_*` levels in
cdrom-rust, 6 invented SMART/ATA command bytes in disk-rust diskwmi
(`SMART_CMD` as 0/0x85 instead of 0xB0), csrss capture raising
`STATUS_NOT_IMPLEMENTED` instead of `STATUS_DATATYPE_MISALIGNMENT`,
se/srm.rs `KeyValueBasicInformation`-vs-Partial + `OBJ_CASE_INSENSITIVE`
duplicated where `OBJ_OPENIF` intended ×2, lpc/complete.rs `ViewShare` for
`ViewUnmap`, win32k/ldevobj + inflib returning `STATUS_BUFFER_TOO_SMALL`
(error!) where `STATUS_BUFFER_OVERFLOW` (warning) is the contract, hal
BUGCHECK code + `KeyFullInformation`, user32 STRSAFE HRESULTs, kernel32
DATE/TIME flag values, cabinet `_A_EXEC`), 6 misleading comments fixed, and
**three scanner FP classes fixed in the tool**: (1) tree-sitter's
`integer_literal` excludes the unary minus, so every negative NTSTATUS
return was compared by magnitude — now both two's-complement widths match,
on the literal AND catalog side (`HFILE_ERROR=-1`); (2) single-bit flags
contained in a composite OR mask (`GHND=0x42 /* GMEM_MOVEABLE */`) no longer
flag; (3) the name-segment regex's greedy parenthetical made arithmetic
prose (`// NonPagedPoolNx (512) + CacheAligned (4)`) parse as an annotation,
and an |-list annotating ONE literal now compares against the OR of the
names (`const LPTR = 0x40; // LMEM_FIXED | LMEM_ZEROINIT`). Post-fix sweep:
0 findings workspace-wide for BOTH tools.

Running tally for the class: 21 (dispatcher audit) + 40 (kernel 2nd pass) +
86 (driver consts) + 60 (tagged literals) = **207 real constant/enum bugs**.

Remaining work:
- The `--enums` finder is still not a gate (FP-prone on generic names) — but
  note the multi-value ambiguity rule means anything it *does* report resolved
  to exactly one header value, which is a strong real-bug signal. Consider a
  scoped gate: kernel crates only, dispatcher-relevant enums only. (Both tools
  now report 0 across the workspace, so a 0-baseline gate is finally viable.)
- A proper byte-order-aware `TAG_*` check (the one class deliberately skipped).
- Untagged inline literals at call sites (`KeInitializeEvent(e, 1, 0)` with no
  `/* … */`) still escape — `scan_inline_flag_literals` only catches the
  *commented* ones. A tree-sitter pass mapping known-signature call args to
  enum params would close that.
- More NDK/PSDK headers so the catalog covers more names.

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
