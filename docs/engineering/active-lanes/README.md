# Active-lane docs

Full design/plan/quest docs for **currently-open worktree lanes** — work in flight, not
yet landed-and-closed. When a lane closes, its doc moves **out** of here: to
`docs/engineering/archive/` (with a closure banner) if it's spent, or back up to
`docs/engineering/` if it graduates into a living reference. Outstanding items from a
closing lane go to `docs/product/vibeos-roadmap.md` (Check-back ledger / Known-bugs
registry), never left to rot in an archived doc.

**Open lanes as of 2026-09-20** (`git worktree list` is the live source of truth):

| Lane (worktree/branch) | Doc |
|---|---|
| `fex-arm` | [fex-bringup-plan.md](fex-bringup-plan.md), [wow64-plan.md](wow64-plan.md) |
| `conformance-ratchet` | [../conformance-ratchet-quest.md](../conformance-ratchet-quest.md) *(paused 2026-10-12; resume from its last progress-log entry)* |
| `smp-bringup` | [smp-bringup-quest.md](smp-bringup-quest.md) |
| `msvcrt-homebrew` | [msvcrt-from-pieces-plan.md](msvcrt-from-pieces-plan.md) |
| `shell32-com` | [shell32-com-plan.md](shell32-com-plan.md) *(translation driven by Codex, not rustconv)* |
| `warnings-burndown` | [warnings-burndown-plan.md](warnings-burndown-plan.md) *(follow-on from the closed `rust-warnings` lane)* |
| `qemu-perf` | [qemu-perf-plan.md](qemu-perf-plan.md) *(QEMU execution layer: windows-host backend, TCG-vs-WHPX, boot concurrency)* |
| `research` | [../winref.md](../winref.md) — the Windows 11 reference VM · [../modern-windows-compat-roadmap.md](../modern-windows-compat-roadmap.md) Rung 0 — the export-surface stubs and gap census *(paused 2026-10-12 until the weekend; both "Owed" lists are the queue)* · older: [x64-primary-plan.md](x64-primary-plan.md) |
| `upstream-sync` | [upstream-sync-plan.md](upstream-sync-plan.md) — strategy · [upstream-sync-merge-decisions.md](upstream-sync-merge-decisions.md) — **what the merge decided, and what it owes** · [upstream-sync-phase4-review-queue.md](upstream-sync-phase4-review-queue.md) — the 194-file review queue *(ONGOING: upstream is a bug oracle, not a patch queue; the big merge landed 2026-09-20, phase 4 is the lane now)* |

*Recently landed & closed:*
- `i386-apic-hal` (Phase A landed 2026-07-16 — APIC is now the i386 default HAL, legacy
  PIC behind `--hal-pic`; doc graduated to [../i386-apic-hal-plan.md](../i386-apic-hal-plan.md);
  Phase B lives on in [smp-bringup-quest.md](smp-bringup-quest.md)).
- `stable-toolchain` (program landed 2026-07-12 — tree is on pinned stable; the one
  sanctioned nightly cell is the i386 kernel's `-Z build-std` softfloat, which retires with
  the wow64 lane. Doc graduated to [../stable-toolchain-plan.md](../stable-toolchain-plan.md)).
