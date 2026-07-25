# §9.56 HANDOFF — the `-smp 2` ready-list corruption (amd64)

> **RESOLVED — this handoff is closed; read it as history, not as a work item.** The hole was
> found on 2026-07-13 (`466d66bf550`) and fixed on all three arches: **`KiIdleLoop` consumed
> `Prcb->NextThread` LOCK-FREE**, inherited from ReactOS's never-validated `CONFIG_SMP` C. The
> standby-displacement branch of `KiDeferredReadyThreadImpl` (correctly under the PRCB lock) read
> `NextThread = T` while the idle CPU promoted T to Running, then re-deferred the *running* T →
> the double-link tear. Every other promote site already held the lock; idle was the single hole.
> Fix: acquire the PRCB lock, **re-check `NextThread` under it**, promote, release, then `sti`
> (`ke/amd64/stubs.rs`, `ke/i386/thrdini.rs`, arm64 mirror). Zero occurrences post-fix.
> The reusable rule: **a field guarded by lock L at N−1 of N sites is not guarded**, and treat
> every inherited `CONFIG_SMP` primitive as unreviewed. Full writeup: field manual §9.56.
>
> The refuted-theory table below is still worth reading before touching this code — each entry
> cost ~40 minutes to disprove, and theory 2 in particular ("a thread with `State == Ready` was
> re-queued") was the headline explanation for weeks and is **wrong**: `State` is not a valid
> proxy for list membership.

**Status when written:** OPEN. Mechanism **reclassified** 2026-07-12 from "scheduler logic bug" to
**"wild write / use-after-free; the ready list is a VICTIM."** (Partly right: the tear was indeed
a victim of an upstream writer — but the writer was a scheduler locking hole, not a stray write.)
**Branch:** `smp-bringup` (worktree `/home/curi/Projects/os-playground/vibeos-smp`).
**Ruling doc:** field manual §9.56 (`docs/engineering/translation-debugging-field-manual.md`).
**Quest log:** `docs/engineering/active-lanes/smp-bringup-quest.md`.

Read this whole file before touching code. Several "obvious" theories are **already
disproven with evidence** — re-deriving them costs ~40 min each.

---

## 1. TL;DR

Booting the SMP kernel with 2 CPUs flakes: ~0.9% organically, **~35% under the
cswitchdep amplifier**. Faces: `0x1E` IN_PAGE_ERROR, `0x19 BAD_POOL_HEADER`,
`0xF4`, and silent wedges.

Custom tripwires proved a `Prcb->DispatcherReadyListHead[pri]` (pri 8/9) gets
**torn**. But every precise test of the *scheduler's own* logic comes back **clean**:
the list is found broken **immediately after a legitimate insert, while the correct
PRCB lock is held**, and the node being inserted was **not** already linked.

That plus `0x19 BAD_POOL_HEADER` in the same corpus ⇒ **something is scribbling
kernel memory**, and the PRCB's ready list is one of the things it lands on.

**Your job: catch the corrupting STORE.** Do **not** add more tripwires to the
scheduler — you will just keep instrumenting victims.

---

## 2. Reproduce (exact)

```bash
cd /home/curi/Projects/os-playground/vibeos-smp
./vibeos-cli build --arch amd64 --smp-kernel --cswitchdep     # ~5-15 min
./vibeos-cli smoke --arch amd64 --smp 2 --repeat 16 --allow-stale   # ~30 min, ~35% fail
```

- **`--cswitchdep` is the amplifier and it is essential.** It is *host-independent*:
  a global per-switch spinlock serializes both CPUs at every context switch, widening
  the race window (0.9% → ~35%). Without it you are waiting days for a hit.
- Read the verdict from `summary.json` / `full.log` under `/tmp/vibeos-cli/<ts>-smoke-amd64/`,
  **not** stdout. Bugcheck params: `grep -E "BUGCHECK: 0x" full.log`.
- **cswitchdep's OWN detector output is GARBAGE under SMP** — ignore the
  `[CSWITCH] SWAP_IRQL expected=2 actual=12` / held-count-negative spam. Its state
  (`VIBEOS_CSWITCHDEP.cpu`) is a **single global** `CpuState`, so with 2 CPUs the
  `held_spinlocks`/`switch_irql` counters read each other's state. It is an
  **amplifier, not a detector**. (Owed fix: make it `cpus[Prcb->Number]`, ~1-2h —
  logged in the quest doc.)

---

## 3. What is ELIMINATED (do not re-derive)

All of these were tested with purpose-built tripwires under full amplified bursts.

| # | Theory | Verdict | Evidence |
|---|---|---|---|
| 1 | **SwapBusy handshake wait-side unimplemented** | **REFUTED** | The amd64 run-side spin **already exists in ASSEMBLY**: `ntoskrnl/ke/amd64/ctxswitch.S:179-184` (`cmp byte ptr [r8+ThSwapBusy],0 / pause`), `ThSwapBusy=0x95`(=149, matches `offset_check_amd64.rs`), compiled at `ntos.cmake:391`. An earlier `.rs`-only grep missed it. ReactOS has **no readier-side SwapBusy wait anywhere** — the only consumer is this asm spin. A Rust spin would be dead code. |
| 2 | **Double-queue (`State==Ready` thread re-queued)** — tripwire `0x52` | **REFUTED as the mechanism** | Fired once early, but later bursts tore the list with `0x52` **silent**. ⇒ **`State` is the WRONG PREDICATE** (a path can change State without unlinking). |
| 3 | **Union-alias clobber** — tripwire `0x53` | **REFUTED** | `KTHREAD.__bindgen_anon_3` really *is* a union (`WaitListEntry: LIST_ENTRY` / `SwapListEntry: SINGLE_LIST_ENTRY`), so `KiInsertDeferredReadyList`'s `SwapListEntry.Next` write really *does* alias `WaitListEntry.Flink` — and it would tear a list **with no lock on the victim PRCB**. Beautiful theory. **It just never happens**: a precise "still-linked?" guard at `KiInsertDeferredReadyList` was **silent** across a full burst. |
| 4 | **Already-linked node re-inserted** — tripwire `0x54` | **REFUTED** | State-independent linkage test ("do my neighbours point back at me?") at `KxQueueReadyThread` entry: **silent**. No still-linked node is ever re-inserted. |
| 5 | **Missing / wrong PRCB lock** | **REFUTED (audited)** | `KiAcquirePrcbLock` is a real SeqCst test-and-set on `(*prcb).PrcbLock` (`inline/ke.rs`, `smp_simple_lock_acquire`). Every mutator pairs with the **correct** PRCB's lock: `KxQueueReadyThread`, `KiDeferredReadyThread`, `KiSelectReadyThread`, `KiSetPriorityThread`, `KiSetAffinityThread`, balmgr (holds both dispatcher + PRCB). `KiQuantumEnd`'s release-inside-callee hand-off into `KxQueueReadyThread` is **correct NT semantics** (the release at `dpc.rs:491` is inside the early-return branch). All `KiSelectNextThread`/`KiSelectReadyThread` callers hold the lock. The **dispatcher lock is real too** (queued spinlock, `LOCK_QUEUE_DISPATCHER_LOCK`). |

**What still fires:** `ki_rq_integrity` sites `0x11` (post-insert in `KxQueueReadyThread`)
and `0x22` (post-insert in `KiDeferredReadyThread`) — i.e. the list is **broken right
after a correct, locked insert of a clean node**.

---

## 4. Current hypothesis

**A wild write / use-after-free is scribbling kernel memory; `Prcb->DispatcherReadyListHead`
is collateral.** Supporting:

- The scheduler is clean under every precise, state-independent test (§3).
- Corruption is observed **under a held lock**, which no amount of lock-discipline
  bugs can explain.
- The same corpus carries **`0x19 BAD_POOL_HEADER` (×2)** and **`0xF4` (×4)** —
  independent memory-corruption faces.
- It is SMP-only / timing-sensitive ⇒ likely a race that produces a stale pointer
  (freed object written after free, or a per-CPU/thread pointer used cross-CPU).

**Why `SPECIALPOOL=*` came back empty and that's CONSISTENT:** special pool guards
**pool allocations**. The PRCB is **not pool**. A writer scribbling through a stale
pointer into the PRCB (or into a freed non-pool object) is invisible to it. Don't
read the null result as "no corruption."

---

## 5. NEXT STEP — catch the store, not the victim

**Use a GDB hardware watchpoint on the torn list head.** This reports the corrupting
PC directly and ends the guessing.

Toolkit (see `docs/engineering/kernel-debugging-infra.md`, `qemu/README.md`):

```bash
RUN=$(qemu/start.sh amd64 --gdb)
qemu/wait-for.sh $RUN --pattern "ACPI Timer" --timeout 60
# then: watch the DispatcherReadyListHead[pri] of the PRCB that tears (pri 8 and 9 seen)
qemu/run-until.sh $RUN --break KeBugCheckEx --timeout 300 --then triage-bugcheck
```

Practical notes:
- **The GDB flow needs the native/KVM backend**, not the default `windows-host` TCG.
  It **serializes** (no parallel bursts), so budget for a slower loop.
- **Getting a hit under GDB is the hard part** — the amplifier matters even more here.
  Build with `--cswitchdep`.
- You need the PRCB address at runtime. `KiProcessorBlock[n]` gives it; the tripwire
  bugchecks already print the offending `prcb` (param1 of site `0x11`/`0x22`, e.g.
  `0xfffffa80018b6180`) and the priority (`param2 & 0xff` — 8 or 9 observed). The
  ready-list head address is `&Prcb->DispatcherReadyListHead[pri]`.
- **Leave the existing tripwires armed.** They are cheap, `#[cfg(feature="smp")]`-gated,
  and they convert a silent wedge into a loud, localized stop. They are how you know
  you got a hit.
- If a hardware watchpoint is impractical, the fallback is a **layout canary /
  poisoning** approach: `VIBEOS_LAYOUT_CANARY=1`, or widen the corruption search with
  `./vibeos-cli farm --arches amd64 --tripwire "SPECIALPOOL=*"` + `farm-report` to
  cluster faces (note VIBEOS_QEMU_SMP=2 for SMP farms).

**Alternate leads worth a look (cheap, before the watchpoint):**
- **`0x19 BAD_POOL_HEADER` is the most tractable face** — it names a pool block. Triage
  *that* (`triage-bugcheck`, `pool-tag`, `identify-object`) and it may finger the
  subsystem that owns the wild pointer. Arguably do this FIRST: it's a cheaper path to
  the same culprit than a watchpoint.
- The known-suspect list from earlier in the lane: device-I/O completion / lost wakeup
  under real concurrency (the §9.51 uniata family). Not TLB — the invlpg narrowing was
  bisected clean.
- The **`§9.36` residual** (stack-resident wait-block / abandoned-IRP scribble class) is
  an *open* bug of exactly this shape (late completion writes a reused frame). Worth
  checking whether it can reach amd64 SMP. `ke/waitdep.rs` is ALWAYS-ON and prints
  `[WAITDEP] VIOLATION` — **grep the serial logs for it.**

---

## 6. Traps that already bit us (don't repeat)

1. **`.rs` is not the whole kernel.** The arch context-switch primitives are hand-written
   **assembly** (`ke/{i386,amd64,arm64}/ctxswitch.S`), and asm offsets (`ThSwapBusy`) come
   from generated `ksamd64.inc` (genincdata), not Rust `offset_of`. A `.rs`-only grep
   produced a confidently-wrong root cause. **Grep the asm too.**
2. **Symbolicate against the RIGHT binary.** SMP boots run **`ntkrnlmp.exe`**, not
   `ntoskrnl.exe`. `qemu/symbols-amd64.tsv` is indexed from the UP kernel and will
   mis-name SMP frames (this produced a bogus `KxQueueReadyThread+0x76` early on).
3. **`#[track_caller]` gets flattened by extern-C shims.** Both `KxQueueReadyThread` and
   `KiInsertDeferredReadyList` were reached via bindgen `extern "C"` decls, so every
   caller reported the shim's line (`shim_exports.rs:2069` / `:1849`). Fixed by
   `pub use`-ing the **inline** under `#[cfg(feature="smp")]` (see `ke_defs.rs`, and the
   3 callers `thrdschd.rs`/`apc.rs`/`balmgr.rs`). If you add a tracked guard elsewhere,
   check the call path isn't going through a shim.
4. **`State` is not linkage.** The legit `RemoveEntryList`-then-defer paths leave
   `State==Ready` on an already-unlinked thread. A state-based guard false-positives on
   **every boot**. Test linkage: `flink != entry && (*flink).Blink == entry && (*blink).Flink == entry`.
5. **NEVER broad-kill QEMUs** (`pkill -9 -f qemu` etc.) — it matches your own process
   ancestors and SIGKILLs your agent. Kill by run-id/uid (`qemu/ps`, `./vibeos-cli ps`).
6. Don't switch the smoke backend to KVM "to repro faster" — same-arch KVM boots
   serialize; the default TCG backend runs them in parallel.

---

## 7. Ground rules (binding)

- **Never weaken a smoke test.** Fix the OS, not the test. Never lower boot markers.
- **Rust stays Rust.** Never revert a translation to C; never delete/regenerate a
  translated file without explicit user approval; never `#[cfg]`-out a Rust module to
  dodge a regression. A broken-but-visible translation is the correct state.
- **Defaults stay UP.** All SMP is opt-in (`--smp-kernel`, `--smp N`). Don't flip
  defaults.
- **Fail loud.** No silent fallbacks.
- Commit only your own files (shared tree). Don't `git reset --hard`. Push only on a
  ×3-clean-gate-green landing.

---

## 8. Code map (what's already in the tree, on `smp-bringup`)

- **Tripwires** — `sdk/lib/rust/ntkrnl-sys/src/inline/ke.rs`:
  - `ki_rq_integrity(prcb, priority, site)` — ready-list invariant; sites
    `0x11` (KxQueueReadyThread post-insert), `0x22` (KiDeferredReadyThread),
    `0x33` (KiSelectReadyThread post-remove), `0x44` (balmgr), `0x51` (entry).
  - `KiSelectReadyThread` — reader-side empty-but-flagged / torn-Blink check.
  - `KxQueueReadyThread` — `0x54` already-linked guard (state-independent).
  - `KiInsertDeferredReadyList` — `0x53` still-linked guard.
  - All bugcheck **`0x96`**; `param2 = (tag<<40) | (caller_line<<8) | priority`,
    `param4 = caller_line`.
- **Scheduler** — `ntoskrnl-rust/src/ke/{thrdschd,wait,dpc,balmgr,apc}.rs`,
  `ke/amd64/{thrdini,traphandler,ipi}.rs`.
- **cswitchdep** — `ntoskrnl-rust/src/ke/cswitchdep.rs` (amplifier; detector broken
  under SMP, see §2).

**Open, adjacent, NOT this bug:** amd64 AP-start caps at **1 AP** —
`HalStartNextProcessor` returns FALSE for CPU2 ⇒ `BUGCHECK 0x61` at `-smp 4`. This
**blocks the "add vCPUs" amplifier**, so the hunt stays at `-smp 2`. Likely a
single-AP assumption in the amd64 HAL AP-start (SIPI trampoline scratch reused?).
