# Control-plane native translation — `native-batch` (worktree `vibeos-gicv3`)

> **PAUSED 2026-10-12 for the work week (WSL maintenance).** Everything below is landed on
> master (last landing `8dddded607b6`); nothing is in flight, uncommitted, or parked in
> `/tmp`. Port state for every module lives in `~/vibeos-rust-staging/port/<stem>/` (its
> `target-*` cargo caches were pruned; `port check` rebuilds them). Resume from "Next".

Control-plane's share of ReactOS base: plain-C system tools, small apps and their DLL
chains, ported with `rustconv port` on Haiku (`-m haiku`). Per-module status is in
[`rust-port-status.md`](../rust-port-status.md) (the "Native wave" rows), never here. The
other base lane is CRT's [native-base-translation.md](native-base-translation.md); its
"Not ours" list records the split.

## Landed

- **Waves A–H** (2026-10-09 → 10-11): see the rust-port-status rows.
- **Wave I, 2026-10-12, `8dddded607b6`.** 23 modules for $0.72: conime, kbswitch, indicdll,
  welcome, winmine, utilman, umandlg, autochk, format, chkdsk, subst, rundll32, regsvr32,
  expand, runonce, bootok, dllhost, text3d (3dtext), logon, wuauserv, fmifs, vfatlib,
  vfatxlib. Clean ×3, built ×3; about 20 `port` tool fixes with it (the row lists them).

Haiku spend on this lane's current $10 allowance: about $6.13.

## Next

1. **Wave J: `base/setup/reactos`** (the text-mode/GUI setup front end, ~6.5K lines).
   `treelist.c` is vendored and stays C. Same shape as Wave I: `port derive` → island →
   scaffold → `translate -m haiku --escalate --cost-cap 2` → resolve → macros → integrate →
   resolve → macros (again: integrate surfaces new callees) → headers → crate-repair ×3
   arches → check → hand-fix and `port hand` → flip → build ×3 → land. Proposed, not yet
   approved by the user: ask before paying.

## Owed (small)

- **The call seal counts calls in dead `#ifdef` arms.** rundll32 had to keep the dead
  non-UNICODE `strncpy` arm as `if false { .. }` to satisfy it. The seal should only count
  the arm the compile takes.
- **`_countof!` returns `u32` where callers compare with `usize`.** It recurred across Wave I
  (kbswitch, others) and was cast by hand each time; settle the macro's type once.
- **A merge driver for `gen_driver_types.sh`** (shared with CRT's owed list). Wave I
  conflicted on it three times in one night against lanes landing just ahead; each time the
  fix was the union of `case` arms, rebuilt from master's copy plus ours. A git
  `merge=union`-style driver that understands arms would remove the whole class.
  `sdk/lib/rust/Cargo.toml` members and `tools/cohab-census.txt` conflict the same way (the
  census is regenerated with `python3 tools/cohab_census.py --write`; `Cargo.lock` is
  master's plus `cargo metadata --offline`).

## Ops state left behind

- **Disk, 2026-10-12:** every lane's build/target dirs were pruned for the pause (fleet
  closed); sccache (150 G cap) and ccache were kept so the first build back is warm.
  Rebuild a lane with `./vibeos-cli build --arch X` before smoke or land.
- **`prune --apply` needs a pinned maintenance owner** (`/tmp/vibeos-qemu-locks/host/
  maintenance-owner`). It was pinned to this worktree for the pause prune and removed
  afterwards; `/tmp` does not survive the WSL restart anyway. Pin it again only for a
  deliberate fleet prune.
