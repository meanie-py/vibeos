# Native ReactOS base — translation lane (`msvcrt-homebrew` worktree)

> **PAUSED 2026-10-12 (Sunday) until next weekend.** Everything below is landed on master;
> nothing is in flight, uncommitted, or parked in `/tmp`. Resume from "Next" below.

The CRT is done on the porting axis (see [msvcrt-from-pieces-plan.md](msvcrt-from-pieces-plan.md)),
so this worktree took on the rest of ReactOS base that no other translation lane touches. The
other lanes are control-plane, rustconv-speedrun/cpp-cluster and shell32. Per-module status
is in [`rust-port-status.md`](../rust-port-status.md), never here.

## Landed

- **2026-10-11, `5c8ff1d7a529`.** This landing did four things:
  - It renamed arm64vid to **sysfb.sys**, which is the display path on all arches.
  - It removed videoprt from the build. Its crate is parked at `~/vibeos-rust-staging/parked/videoprt-rust`.
  - It flipped watchdog.
  - It added the PnP write-up, [`sysfb-pnp-design.md`](../sysfb-pnp-design.md). That doc's work is owed and is not started.
- **2026-10-12, `09fa034f9338`.** The E′+G batch: 17 modules on Haiku 5.5 for $0.68.
  - Win32 DLLs: msports, newdev, psapi, winsta, ifmon, fltlib, wlnotify, hid, serialui,
    storprop, winfax, avicap32 and cards.
  - Kernel libraries: wdfldr, ntoskrnl_vista, sptilib and apisets.
  - hid.dll links `hidparser-rust`, so the C hidparser targets are gone.
  - The batch row in rust-port-status.md has the details.

## Next (in this order)

1. **A′, the plain-C audio stack:** mmixer, mmebuddy, wdmaud, kmixer, sysaudio, mment4,
   mmdrv, beepmidi, sndblst. Use the same shape as E′+G: Haiku (`-m haiku`) with a $5 cap,
   then `port` derive → … → flip, then gate ×3.
2. **C, RDBSS and copysup.** First check what still links or loads RDBSS. If nothing does,
   it is a prune candidate rather than a translation.

## Not ours: skip these

- **B and D** belong to control-plane.
- **All C++** belongs to speedrun/cpp-cluster: portcls, hdaudbus, framedyn, iernonce,
  shimgvw, shell extensions, devmgr, rapps, ….
- **shell32** and its base/applications batches D1/D2 belong to the shell32 lane.
- **control-plane asked us to skip:** conime, kbswitch, indicdll, winmine, utilman, umandlg,
  fc, rundll32, regsvr32, expand, runonce, bootok, dllhost, the 3dtext and logon
  screensavers, wuauserv, sehtest, nettest, stortest, testgui, setup16, filebrowser,
  devmgmt and rapps_com.

## Owed (small)

- **A test for the substrate-in-declared fix.** In `port.py`, `_resolve_externs`'s
  `declared` now includes `prelude_resolve.existing_substrate(prelude)`. Without that, a
  substrate alias was declared twice. It is the one tool change from E′+G that has no test.
- **A merge driver for `gen_driver_types.sh`.** Every lane appends arms to it, so merges
  conflict. E′+G resolved one by taking the union of the arms (labels contain `-`).
- **psapi's `main.c` (DllMain) is still C.** It stayed outside the batch's hand-flip.
- The three tech-debt items are in the roadmap's "Known bugs & flakes" registry, so they
  are not repeated here:
  - the i386-uefi install's fastfat flush fault;
  - the amd64 BUGCHECK 0x1E;
  - the arm64 install-smoke, which needs a non-IDE disk layout.

## Lessons from E′+G that a resume should know

- **Run `./vibeos-cli check --strict --scope source-deferred` before `land`.** The land
  preflight runs it and plain `check` does not. On E′+G it caught a real bug: sptilib's
  `_SEH2_TRY` compiled as a no-op borrowed from cdfs, and resolve no longer donates PSEH
  names.
- **A kernel-library crate needs a `TARGETS` entry** in
  `tools/rustconv/rustconv/projects/reactos/targets.py`, plus a path-routing branch.
- **The first Rust caller of a shared `ntkrnl-sys` helper on arm64 can expose an import
  that arm64 ntoskrnl never exported.** That was `KeGetCurrentPrcb`, now read via
  `TPIDR_EL1`. Build all three arches; a two-arch green proves nothing about arm64's link.
