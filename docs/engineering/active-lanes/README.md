# Active-lane docs

Full design/plan/quest docs for **currently-open worktree lanes** — work in flight, not
yet landed-and-closed. When a lane closes, its doc moves **out** of here: to
`docs/engineering/archive/` (with a closure banner) if it's spent, or back up to
`docs/engineering/` if it graduates into a living reference. Outstanding items from a
closing lane go to `docs/product/vibeos-roadmap.md` (Check-back ledger / Known-bugs
registry), never left to rot in an archived doc.

**Open lanes as of 2026-07-11** (`git worktree list` is the live source of truth):

| Lane (worktree/branch) | Doc |
|---|---|
| `fex-arm` | [fex-bringup-plan.md](fex-bringup-plan.md), [wow64-plan.md](wow64-plan.md) |
| `screen-painting` | [ui-render-completeness-investigation.md](ui-render-completeness-investigation.md) |
| `conformance-ratchet` | conformance-ratchet-quest.md *(lives on the branch until it lands)* |
| `smp-bringup` | [smp-bringup-quest.md](smp-bringup-quest.md) |
