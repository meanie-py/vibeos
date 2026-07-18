# Active-lane docs

Full design/plan/quest docs for **currently-open worktree lanes** — work in flight, not
yet landed-and-closed. When a lane closes, its doc moves **out** of here: to
`docs/engineering/archive/` (with a closure banner) if it's spent, or back up to
`docs/engineering/` if it graduates into a living reference. Outstanding items from a
closing lane go to `docs/product/vibeos-roadmap.md` (Check-back ledger / Known-bugs
registry), never left to rot in an archived doc.

**Open lanes as of 2026-07-16** (`git worktree list` is the live source of truth):

| Lane (worktree/branch) | Doc |
|---|---|
| `fex-arm` | [fex-bringup-plan.md](fex-bringup-plan.md), [wow64-plan.md](wow64-plan.md) |
| `conformance-ratchet` | conformance-ratchet-quest.md *(lives on the branch until it lands)* |
| `smp-bringup` | [smp-bringup-quest.md](smp-bringup-quest.md) |
| `msvcrt-homebrew` | [msvcrt-from-pieces-plan.md](msvcrt-from-pieces-plan.md) |
| `shell32-com` | [shell32-com-plan.md](shell32-com-plan.md) *(translation driven by Codex, not rustconv)* |
| `warnings-burndown` | [warnings-burndown-plan.md](warnings-burndown-plan.md) *(follow-on from the closed `rust-warnings` lane)* |

*Recently landed & closed:*
- `i386-apic-hal` (Phase A landed 2026-07-16 — APIC is now the i386 default HAL, legacy
  PIC behind `--hal-pic`; doc graduated to [../i386-apic-hal-plan.md](../i386-apic-hal-plan.md);
  Phase B lives on in [smp-bringup-quest.md](smp-bringup-quest.md)).
- `stable-toolchain` (program landed 2026-07-12 — tree is on pinned stable; the one
  sanctioned nightly cell is the i386 kernel's `-Z build-std` softfloat, which retires with
  the wow64 lane. Doc graduated to [../stable-toolchain-plan.md](../stable-toolchain-plan.md)).
