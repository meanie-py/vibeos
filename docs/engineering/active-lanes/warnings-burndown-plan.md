# Rust warnings burndown (lane: `warnings-burndown`)

**Status:** OPEN (opened 2026-07-16). Worktree `vibeos-warnings-burndown`, branch
`warnings-burndown`. Follow-on from the closed `rust-warnings` lane (which fixed the 3
dangling-pointer UAFs + landed `scan_dangling_transmute`); this lane burns down the *rest*.

**Deliverable:** the Rust workspace compiles with a dramatically smaller warning surface,
the genuine-signal warnings are audited and fixed, and — critically — the fixed classes
**stay** fixed (no blanket re-suppression). Incremental, per-crate, each cluster gated ×3
and landed on its own so conflicts stay localized and every step is revertible.

## The census finding that reframes everything (2026-07-16)

**201 of 219 workspace crates (92%) carry a crate-root `#![allow(...)]`** — most of them
blanket `dead_code`/`non_snake_case`/`unused` suppressions parked at `lib.rs` top. So:

- The "~9,555 warnings" figure from the rust-warnings survey was measured **with those
  blanket allows in place** — it is an *undercount*. The real surface is hidden.
- A burndown that only fixes what currently *leaks past* the allows is cosmetic. The
  durable version **peels the blanket allow off a crate**, fixes what that exposes, and
  lands the crate clean **without** re-adding the blanket — so future warnings can't hide.
- These allows exist because the c2rust/LLM translation output is noisy by construction
  (non-snake-case FFI names, dead translated-but-unused helpers). Much of it is legitimately
  suppressible — but **at the item/module level with a reason**, not a crate-root sledgehammer
  that also hides real bugs (an unused `Result`, an unreachable arm, a shadowed write).

## Three tiers — sort every warning into one

### Tier 1 — NOISE (mechanical, subagent-fannable, the bulk)
Auto-fixable style: `unused_imports`, `unused_mut`, `unused_variables` (→ `_`-prefix),
redundant clones, `unused_parens`, needless `return`. **`cargo fix` handles these.**
- This is the **subagent work** the user green-lit ("sonnet or even haiku to burn down
  simple things"). Fan-out model per AGENTS.md: one crate per agent, agent runs `cargo fix`
  + hand-cleans the residual in its crate ONLY, writes the patched `.rs` to
  `~/vibeos-rust-staging/warnings-<crate>/`, does **not** edit shared files or build. **You**
  integrate + build + gate ×3 centrally, per cluster.
- Keep legitimate suppressions, but **narrow them**: crate-root `#![allow(non_snake_case)]`
  on an FFI crate is fine to keep (the whole crate is FFI); crate-root `#![allow(dead_code)]`
  is NOT — dead code should be deleted or `#[allow(dead_code)]`'d per-item with a reason.

### Tier 2 — SIGNAL (manual audit — NOT subagent work)
Warnings that can indicate real bugs. Do these yourself, carefully:
- **`clashing_extern_declarations`** (~60 in the last census) — the same extern symbol
  declared with two different signatures across crates. This is an ABI-mismatch class; it's
  what `verify_ffi.py` also hunts. Each one is a potential silent-miscall on amd64/arm64
  (links fine, wrong ABI) that only i386 would catch. Reconcile to one declaration.
- **`unpredictable_function_pointer_comparisons`** — `ob-rust/src/obname.rs`,
  `ob-rust/src/obsecure.rs`, `ps-rust/src/psnotify.rs`. Comparing fn pointers is
  optimization-fragile (LLVM may merge/split identical bodies). Verify each compares what the
  C intended; replace with a stable discriminant if the identity matters.

### Tier 3 — TOOLCHAIN TIMEBOMB (do FIRST — it's the only time-sensitive one)
**`deref_nullptr` / FIELD_OFFSET-via-null: 55 sites and GROWING** (~32 at the last census;
ole32/oleaut32/msvcrt landing added ~23). Shape: `addr_of!((*null::<T>()).field) as usize` —
the C `FIELD_OFFSET(T,f)` idiom translated literally. **On pinned rustc 1.92 it's a warning;
on a newer toolchain it's a HARD ERROR.** The stable-toolchain lane just landed; the next
rustc bump breaks the build on all 55.
- **Fix = `core::mem::offset_of!(T, field)`** — this is `scan_magic_offsets.py`'s domain
  (the offset-drift lint). Coordinate with it: the fix is exactly what that gate wants.
- Do this tier first and land it — it's the one with an external deadline (the toolchain pin).

## ⚠️ Coordination fence — do NOT churn crates active lanes are editing

A workspace-wide `cargo fix` touches ~200 crates. Landing that under an active lane's feet
creates brutal merge conflicts. **Before touching any crate, check `git worktree list` +
recent commits.** Known-HOT crates as of 2026-07-16 — **skip these until their lane lands**:

| Active lane | Hot crates — off-limits |
|---|---|
| `shell32-com` | `shell32-rust`, `tools/rustconv/**` |
| `smp-bringup` | `ntoskrnl-rust` (ke/mm), `ntkrnlmp-cargo`, `hal-rust`, `ke-*` |
| `msvcrt-homebrew` | `msvcrt-*`, `cnt-rust`, `libm`-side crates |
| `research` | `ole32-rust`, `oleaut32-rust` |

**Start with cold crates** — the driver fleet, the smaller algorithm crates, the settled
user-DLLs. There are ~200; there is plenty to do without touching a hot one. The timebomb
(Tier 3) is the exception where you may need to touch a hot crate's file — if so, hand that
lane a tested one-line `offset_of!` diff rather than landing it yourself.

## Ordering

1. **Census first** — real per-crate warning counts on the current tree (the 9,555 is stale;
   run `cargo build` capturing warnings, or per-crate `cargo check`, across all 3 arches).
   Record the baseline so progress is measurable. Sort into the three tiers.
2. **Tier 3 (timebomb)** — the 55 `deref_nullptr` → `offset_of!`. External deadline. Land it.
3. **Tier 2 (signal)** — clashing externs + fn-ptr comparisons. Manual. Land per-cluster.
4. **Tier 1 (noise)** — subagent fan-out over COLD crates, cluster-integrated + gated by you.
5. As hot lanes land, fold their crates into the Tier-1 sweep.

## Lane rules

- **Per-cluster commits, each gated ×3** — never one giant workspace-wide diff. Localized
  conflicts, revertible steps, and the smoke harness stays a usable signal.
- **Never suppress to green.** Removing a real warning by fixing it ✅; re-adding a blanket
  `#![allow]` to silence it ❌. If a warning is a genuine false-positive, `#[allow(...)]` it
  **per-item with a `// reason`**, never crate-root.
- **A fix that changes behavior is a bug, not a cleanup.** `cargo fix` is *supposed* to be
  behavior-preserving, but review every hunk — an "unused" `Result` it drops may be a
  swallowed error (`scan_silent_failures`' domain). When in doubt, keep the value and bind it.
- **Consider a gate at the end** — once a crate is clean, a lint that keeps it clean (deny the
  fixed classes for that crate, or a workspace `-D` for the ones fully eliminated) is what
  makes the burndown *stay* burned down. Design it so it can't be defeated by a new blanket allow.
