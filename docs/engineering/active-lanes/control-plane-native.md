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

1. **Wave J: `base/setup/reactos`** — **taken by the CRT lane, 2026-10-17** (the text-mode/GUI setup front end, ~6.5K lines).
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

**Store follow-ups.** Done 2026-10-16: the rollout doc (375/391), daily GC with dead-ref
retirement, GC no longer hashing kept blobs, request revalidation by stat stamps, idle
body-shard retirement, premerge runs in the fleet ledger, and BUILD_INPUTS_MOVED (a build refuses to stamp a tree edited
under it; reported by the CRT lane). Still owed:
- The supervisor preflight re-hashes every blob the lane references on every build
  (13-53 s), the store's largest fixed cost.
- Discovery itself, ~0.4 s per request.

**Store-ineligible bodies (16).** Registry crates: atan2 (libm), cabinet (libz-rs-sys),
netstack (smoltcp), vioblk (virtio-drivers). cnt-rust's target-specific dependency tables,
which also make rtl-rust ineligible. icuuc, lsa, sam, svcctl and winreg stubs
(`build-dependencies`: the reactos-link IDL generator). user32-wsprintf (build script, plus
a `#[path]` into cnt-rust). menuband (`#[path]` into shell32-rust). wowx64 (build script
generating `OUT_DIR` services.rs). i386 kernel cells on the json target are never offered.

**Cargo-drive.** Wave 13 and the first `--convert` batch on 2026-10-17; new modules
convert by default (`cargo_cell_gen --convert`, enforced by `scan_cmake_linked_rust`).
Still owed, in the plan's "Still refused" list: the C_STATIC_LIBRARY placeholder libraries,
static variants with no `rust_staticlib_variant`, `add_idl_reg_scripts` (evr, mf,
mfreadwrite), psapi's i386 link, user32_vista's never-linked Rust; the kernel and loader
cells' `libcntpr` / `mini_hal` / `freeldr_bios_asm`; bootvid's resource; rung 1's
automatic `--verify`; rungs 4-6. (The wave-2 refusals bmfd through winprint were moved in
later waves.)
- i386 PIC HAL (no `apic`, no minihal) fails E0308, before and after the build.rs removal.

**Translation.** Wave J (`base/setup/reactos`) moved to the CRT lane on 2026-10-17 (its
batch H, already translating). The small owed items under "Owed (small)" stay open.

**Registry items this lane touches:** install-smoke amd64-uefi takes 44–50 min (older
than wave 11).



- **Disk, 2026-10-12:** every lane's build/target dirs were pruned for the pause (fleet
  closed); sccache (150 G cap) and ccache were kept so the first build back is warm.
  Rebuild a lane with `./vibeos-cli build --arch X` before smoke or land.
- **`prune --apply` needs a pinned maintenance owner** (`/tmp/vibeos-qemu-locks/host/
  maintenance-owner`). It was pinned to this worktree for the pause prune and removed
  afterwards; `/tmp` does not survive the WSL restart anyway. Pin it again only for a
  deliberate fleet prune.
