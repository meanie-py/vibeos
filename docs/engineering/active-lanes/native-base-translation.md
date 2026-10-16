# Native ReactOS base — translation lane (`msvcrt-homebrew` worktree)

> **Resumed 2026-10-16.** control-plane's immutable-build change (the shared artifact store,
> [`immutable-build-artifacts.md`](../immutable-build-artifacts.md)) is merged into the lane.

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

## A′ in progress: the plain-C audio stack (resume here)

Translated 2026-10-15 for **$0.66** (Haiku, 0 rejects), committed, **not flipped**: the
CMake files still build the C, so the tree boots exactly as before. Ten crates:
mmixer, mmebuddy, mment4, wdmaud (the .sys), wdmaud_drv (`dll/win32/wdmaud.drv`, stem
`wdmaud_drv`), kmixer, sysaudio (the four TUs beside the pilot's `pin.rs`), mmdrv, beepmidi,
sndblst. Port state is in `~/vibeos-rust-staging/port/<stem>/`, logs in
`~/vibeos-rust-staging/batch-a/`. Not in the batch: `uartmidi`, `shared` (`audio`),
`legacy` (`audioleg`) and `soundblaster` are built or present but nothing links them (prune
candidates); everything C++ belongs to cpp-cluster.

**sysaudio is flipped (2026-10-16)**: all five TUs are Rust, and `rust_cohab_stub.c` is the
MODULE's only C source. It had to go first. The wip commit swapped the pilot's `lib.rs`
(`pin` only) for the scaffold's, so CMake's existing `sysaudio-rust` build compiled the
four red modules and broke the build on every arch. A crate CMake already builds is never
"not flipped". Fixes: a stray `use crate::types::*;` in each of the four files (ambiguous
with the prelude's TRUE/FALSE), explicit `ntkrnl_sys::ddk` imports in `pin.rs`, and
`KSDATAFORMAT` fields reached as `.c2rust_unnamed.X` places, not through `Deref`, since a
`Deref` borrow inside the packed `KSDATAFORMAT_WAVEFORMATEX` is E0793.

`cargo check` state (amd64): beepmidi, mmdrv, sndblst, mmebuddy, mment4 and sysaudio at 0;
kmixer, wdmaud, wdmaud_drv and mmixer still red. To resume, run, per crate,
`port resolve` → `crate-repair <crate>-rust --arches i386 amd64 arm64` → `port check`, then
hand-fix what is left:
- **Copy_0** (mmixer, wdmaud, wdmaud_drv): crate-repair's keyword-field rule now renames it
  to the island's `Copy` when rustc offers that name; one repair pass should clear it.
- **`_byteswap_*` / `lrintf`** (kmixer, wdmaud_drv): now in win32macros-rust; resolve
  re-lists them in the prelude (the `pub const fn` pattern fix).
- **wdmaud_drv**: `STATIC_KSCATEGORY_WDMAUD`, six `#[cfg]`-on-expression sites in mixer.rs,
  five `break` outside a labelled block in legacy.rs.
- **wdmaud**: `_SEH2_TRY`/`_SEH2_END` in control.rs (rewrite with
  `ntdll_sys::seh::try_except` or a non-raising call), `PDEVICE_EXTENSION`.
- **mmixer**: `PINNAME_CAPTURE`, an index through `*mut __TOPOLOGY_NODE__`, seven E0793 in
  wave.rs. It links into both wdmaud.sys and the user-mode wdmaud.drv: give it
  `kernel`/`user` features like hidparser-rust, and have wdmaud.drv link
  `rust_staticlib_variant(... mmixer-rust "user" FALSE)`.

Then flip the other nine (sysaudio-difftest reads `sysaudio-rust/src/pin.rs`; keep that path), build ×3, smoke ×3, `check --strict --scope
source-deferred`, and land. Bank hand fixes with `port hand <stem> <file.rs>:<fn>`.

What the batch added to the shared crates:
- **mmebuddy-macros-rust** (new): `mmebuddy_debug.h` / `mmebuddy.h` / `sndtypes.h` statement
  macros, in a quiet `_NDEBUG` and a printing `_DBG` flavour (mmebuddy and wdmaud.drv define
  NDEBUG; mment4 and sndblst do not). `port macros` cannot lower them: `__FILE__` is the
  probe's per-arch temp path.
- **win32macros-rust**: `_byteswap_ushort/_ulong/_uint64` and math.h's inline `lrintf`.

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

- **Run `./vibeos-cli check --strict --scope source-deferred` before `land`.** The land
  preflight runs it and plain `check` does not. On E′+G it caught a real bug: sptilib's
  `_SEH2_TRY` compiled as a no-op borrowed from cdfs, and resolve no longer donates PSEH
  names.
- **A kernel-library crate needs a `TARGETS` entry** in
  `tools/rustconv/rustconv/projects/reactos/targets.py`, plus a path-routing branch.
- **The first Rust caller of a shared `ntkrnl-sys` helper on arm64 can expose an import
  that arm64 ntoskrnl never exported.** That was `KeGetCurrentPrcb`, now read via
  `TPIDR_EL1`. Build all three arches; a two-arch green proves nothing about arm64's link.
