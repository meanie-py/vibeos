# IRQL/spinlock RAII-conversion queue (tabled 2026-07-06)

> **ARCHIVED 2026-07-10** — completed audit (2026-07-06, tabled) — 24 crates / 504 acquire sites, 0 confirmed + 0 plausible IRQL/spinlock leaks. The lifecycle is clean; the scanner (`scan_irql_balance.py`, field manual §9.26) is the standing guard.


A 24-crate adversarial audit (30 subagents, 504 raw acquire sites) for leaked
IRQL raises / spinlock acquires. Companion to `tools/scan_irql_balance.py`
(field manual §9.26) and the `IrqlGuard`/`SpinLockGuard` RAII types in
`ntkrnl_sys::guards`.

## Headline result: the lifecycle is clean

**0 confirmed and 0 plausible latent leaks across 504 sites.** Every crate was
audited for the leak shapes the intraprocedural scanner *cannot* see —
panic/`unwrap`-while-held, `?`-in-helper error paths, interprocedural (fn A
acquires, callee early-returns), conditional-branch releases, loop-break past a
release. Each reported leak got a second adversarial agent that read the code and
refuted-by-default. Nothing survived. Combined with the scanner having already
caught+fixed the one real leak (usbport `USBPORT_MiniportCompleteTransfer`,
field manual §9.26), the IRQL/lock balance is genuinely sound today.

## The conversions are HARDENING, not fixes — and NOT safe to auto-apply

134 raw pairs were flagged as RAII-conversion candidates (82 "trivial", 52
"moderate"). **The agent "trivial" labels are ~50% unreliable** — verified on the
4 ntoskrnl-rust picks: 2 (`sysinfo` lookaside pair) were genuinely safe 1:1
swaps, but 2 (`povolume` `PopGetDope` / `PoRemoveVolumeDevice`) were mislabeled —
the agent claimed "release is the last statement" when an `ExFreePoolWithTag`
runs *after* the release, at the lowered IRQL. A naive guard swap (Drop at
function end) would pull that pool-free **inside** the spinlock at DISPATCH_LEVEL
— a latent bug smoke would not catch. **Every conversion needs per-site human
verification** that the release is truly the last thing that must happen under
the lock; where it isn't, the guard must be block-scoped. This negates the
parallelism win for the mislabeled majority, so a blind campaign is the wrong
move given 0 leaks.

## Status

**Proof batch landed (2026-07-06):** the 4 ntoskrnl-rust candidates, done
correctly — `sysinfo` as 1:1 swaps, `povolume` block-scoped so the pool-free
stays outside the lock (doing right what the agent got wrong). Demonstrates the
migration path + the block-scope discipline.

**Tabled for later:** the remaining 130 candidates. Full machine-readable list in
[`irql-raii-conversion-queue.json`](irql-raii-conversion-queue.json) (file, line,
function, rationale, risk, crate). Adopt incrementally + hand-verified as files
are touched for other reasons — there is no urgency (0 leaks; the scanner gates
any NEW leak at the door).

| crate | trivial | moderate |
|---|---|---|
| classpnp-rust | 12 | 5 |
| iomgr-rust | 8 | 2 |
| serial-rust | 6 | 4 |
| usbhub-rust | 6 | 3 |
| msfs-rust | 6 | 1 |
| afd-rust | 5 | 2 |
| ntoskrnl-rust | 4 (done) | 5 |
| ks-rust | 4 | 2 |
| cdrom-rust | 4 | 2 |
| pnpmgr-rust | 4 | 1 |
| usbport-rust | 4 | 0 |
| acpi-rust | 3 | 0 |
| mouclass-rust | 3 | 2 |
| kbdclass-rust | 3 | 2 |
| usbstor-rust | 2 | 7 |
| scsiport-rust | 2 | 3 |
| npfs-rust | 2 | 2 |
| hidclass-rust | 1 | 4 |
| mountmgr-rust | 1 | 2 |
| arm64input-rust | 1 | 2 |
| usbaudio-rust | 1 | 1 |

**Verification discipline for each conversion** (learned from the povolume
mislabels): confirm the manual `KeReleaseSpinLock`/`KeLowerIrql` is the last
statement that must run under the lock. If any work follows it (esp.
`ExFreePoolWithTag`, a paged operation, or a call that may raise/wait), the guard
MUST be block-scoped to drop before that work — never a bare function-scope
`let _g`.
