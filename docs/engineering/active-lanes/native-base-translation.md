# Native ReactOS base — translation lane (`msvcrt-homebrew` worktree)

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

## A′: the plain-C audio stack (done 2026-10-16)

Ten modules on Haiku for **$0.66**, all flipped and booting on three arches: mmixer,
mmebuddy, mment4, wdmaud.sys, wdmaud.drv (stem `wdmaud_drv`), kmixer, sysaudio, mmdrv,
beepmidi and sndblst. The row in rust-port-status.md has the details; port state is in
`~/vibeos-rust-staging/port/<stem>/` with the hand fixes banked. Not in the batch:
`uartmidi`, `shared` (`audio`), `legacy` (`audioleg`) and `soundblaster` are built or
present but nothing links them (prune candidates); everything C++ belongs to cpp-cluster.

Lessons a resume or the next batch should know:
- **A crate CMake already builds is never "not flipped."** sysaudio's CMake built
  `sysaudio-rust` for the pilot's `pin.rs`, so swapping in the scaffold's `lib.rs` put four
  red modules into every arch's build. Grep CMake for the crate before scaffolding over it.
- **Don't edit sources while a build runs.** The artifact store snapshots each body's inputs
  and rejects a result whose lane inputs changed mid-build ("lane inputs changed; result
  rejected"). Editing ntkrnl-sys during an i386 build failed win32k's link.
- **A dual-mode library crate** (mmixer: kernel in wdmaud.sys, user in wdmaud.drv) takes
  `kernel`/`user` features mapped onto panic-common's, like hidparser-rust. The scanners
  judge its kernel island against the user consumer's import libraries, so an uncalled
  kernel declaration can still be reported (the two SList entries in scan_import_cc's
  baseline).
- **A header FORCEINLINE that is an import on one arch** (`KeSaveFloatingPointState`)
  belongs in `ntkrnl_sys::inline`, per arch. The island's extern links on x86 and
  fails on arm64.

What the batch added to the shared crates and tools:
- **mmebuddy-macros-rust** (new): `mmebuddy_debug.h` / `mmebuddy.h` / `sndtypes.h` statement
  macros, in a quiet `_NDEBUG` and a printing `_DBG` flavour (mmebuddy and wdmaud.drv define
  NDEBUG; mment4 and sndblst do not). `port macros` cannot lower them: `__FILE__` is the
  probe's per-arch temp path.
- **win32macros-rust**: `_byteswap_ushort/_ulong/_uint64` and math.h's inline `lrintf`.
- **ntkrnl-sys**: `inline::ke::{KeSave,KeRestore}FloatingPointState`.
- **crate-repair**: a union reached through the island's `Deref` inside a packed struct
  (E0793), or through a raw pointer to the union itself (the autoref lint), names the
  anonymous member instead.
- **scan_cc_mismatch**: a qualified call into a `-sys` module that defines a Rust fn of
  that name (`ntkrnl_sys::inline::ke::KeInitializeSpinLock`) is not the crate's extern.

## Next (after A′)

1. **C, RDBSS and copysup.** First check what still links or loads RDBSS. If nothing does,
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

- **Run `./vibeos-cli check` before `land`.** Since 2026-10-16 it fails on a NEW finding
  in every source partition; before that only the land preflight's strict run did. On E′+G it caught a real bug: sptilib's
  `_SEH2_TRY` compiled as a no-op borrowed from cdfs, and resolve no longer donates PSEH
  names.
- **A kernel-library crate needs a `TARGETS` entry** in
  `tools/rustconv/rustconv/projects/reactos/targets.py`, plus a path-routing branch.
- **The first Rust caller of a shared `ntkrnl-sys` helper on arm64 can expose an import
  that arm64 ntoskrnl never exported.** That was `KeGetCurrentPrcb`, now read via
  `TPIDR_EL1`. Build all three arches; a two-arch green proves nothing about arm64's link.
