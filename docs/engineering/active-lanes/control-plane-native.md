# Control-plane native translation — `native-batch` (worktree `vibeos-gicv3`)

> **State 2026-10-16.** Since 10-12 this lane ran the cargo-drive program
> ([cargo-drive-plan.md](../cargo-drive-plan.md), waves 1–12) and the shared Rust artifact
> store ([immutable-build-artifacts.md](../immutable-build-artifacts.md)), then two lint
> fixes (`22add78b63f5`). Last landing `22add78b63f5`; nothing uncommitted. The full owed
> list across all three tracks is "Open threads" below.

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

## Open threads (2026-10-16)

**Store and lint follow-ups (next, in this order).**
- `immutable-build-artifacts.md` rollout section: 375 of 391 bodies eligible; the
  hal/ntdll/fastfat changes and the `data_inputs` mechanism.
- Store GC on a schedule (the dry run found 1,342 collectable actions; never applied).
- Store refs leak when a lane is pruned or its worktree removed.
- About 0.8 s of store latency per request.
- gicv3's ~8 G of orphaned body-shard content.
- The premerge preflight's runs land in `.git/vibeos-premerge/vibeos-ledger.jsonl`, not
  the fleet ledger.

**Store-ineligible bodies (16).** Registry crates: atan2 (libm), cabinet (libz-rs-sys),
netstack (smoltcp), vioblk (virtio-drivers). cnt-rust's target-specific dependency tables,
which also make rtl-rust ineligible. icuuc, lsa, sam, svcctl and winreg stubs
(`build-dependencies`: the reactos-link IDL generator). user32-wsprintf (build script, plus
a `#[path]` into cnt-rust). menuband (`#[path]` into shell32-rust). wowx64 (build script
generating `OUT_DIR` services.rs). i386 kernel cells on the json target are never offered.

**Cargo-drive leftovers** (details in the plan):
- Rung 1: run `cargo_orchestrate --all --verify` somewhere automatic (it rotted once).
- Wave-2 refusals: kernel-mode `native:` DLLs (bmfd, ftfd, ndis, watchdog); per-arch link
  inputs (acgenral, opengl32); `rust_staticlib_variant` extras (calc, taskmgr, wordpad);
  per-arch CD destination (winprint); a shared `SOURCE` list (gdi32_vista_static).
- Still refused: user32_vista (exports what it imports; cell parked in stash "wave11
  user32_vista cell"), mstsc (vendored rdesktop C), psapi_static and the other
  C_STATIC_LIBRARY targets.
- bootvid's version resource (`.rc` is a configured `#cmakedefine` template).
- Kernel and loader cells still link cmake's `libcntpr` / `mini_hal` / `freeldr_bios_asm`.
- Rungs 4–6 not started: resources without cmake, the ISO from a manifest, cargo compiling
  the remaining C.
- i386 PIC HAL (no `apic`, no minihal) fails E0308, before and after the build.rs removal.

**Translation (paused since 10-12).** Wave J, `base/setup/reactos`, is next and needs the
user's go-ahead before paying. Its small owed items are under "Owed (small)" above.

**Registry items this lane touches:** install-smoke amd64-uefi takes 44–50 min (older
than wave 11).



- **Disk, 2026-10-12:** every lane's build/target dirs were pruned for the pause (fleet
  closed); sccache (150 G cap) and ccache were kept so the first build back is warm.
  Rebuild a lane with `./vibeos-cli build --arch X` before smoke or land.
- **`prune --apply` needs a pinned maintenance owner** (`/tmp/vibeos-qemu-locks/host/
  maintenance-owner`). It was pinned to this worktree for the pause prune and removed
  afterwards; `/tmp` does not survive the WSL restart anyway. Pin it again only for a
  deliberate fleet prune.
