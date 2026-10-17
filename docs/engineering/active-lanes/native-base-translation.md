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

**Graded 2026-10-16 (`audio-convert-difftest`, $0).** kmixer's `pin.c` and wdmaud.drv's
`mixer.c` carry the same PCM converters, and their C is right in only two cases: 8-bit
channel up-mixing and 16→32-bit. Everywhere else it is wrong:
- The channel converter sizes its buffer in samples, not bytes. It reads the input at
  byte offsets, fills one channel too many on the last frame, and reports a sample count
  as the length. So every 16-bit mono↔stereo stream overran the pool (kmixer) or the
  heap (wdmaud.drv).
- The quality converter never re-centres unsigned 8-bit samples. 8→32 maps silence to full
  scale.
- Under `_X86_` it byte-swaps every sample, so i386 produced different audio from amd64 for
  the same stream.

The Rust in both crates now does the right thing, with divergence comments. The harness
grades it against the C where the C is defined, and against an independent PCM model
everywhere else. It also pins each C defect, so a later fix to the C shows up as a test
change.

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

## Next

1. **Area C is closed by pruning, not translating (2026-10-16).** Nothing in the tree links
   rdbsslib, rxce or copysup (19.5k lines of C; upstream they serve the NFS redirector,
   which this tree does not carry), and nothing links sound's `shared` (`audio`) or
   `uartmidi`. Their `add_subdirectory` lines are commented out with the reason; the C stays
   on disk, so re-enabling one is a one-line change if a consumer ever arrives.
2. **The static `crt` library is pruned too** (`#include(crt.cmake)`, 2026-10-16). Nothing
   linked it and no target pulled it in, so its ~720 TUs only appeared in
   compile_commands.json.
3. **Audio difftests, round two (2026-10-17, `mmixer-difftest`, $0).** Both mmixer findings
   from the survey were real and are fixed:
   - `MMixerSetGetVolumeControlDetails` reported full volume as 0x10000, one past the
     control's 0xFFFF maximum, and wrapped a driver value outside its range. The Rust
     clamps; the C formula is kept.
   - The topology walkers: `MMixerAddPinIndexToArray` had no equality case, so a pin or node
     reached along two paths was listed twice. With the array full that pushed the count
     past MaxPins. Neither walker skipped a visited node; they only ASSERTed, and ASSERT!
     compiles out in the Rust, so a cycle would recurse without end. `GetUpOrDownstreamNodes`
     also marked the parent where it meant the child. The harness grades trees exactly
     against the C and diamonds and cycles against a reachability model.
   - mmebuddy's looped playback (`mmebuddy-difftest`, 2026-10-17) was wrong four ways,
     all now fixed in `streaming.rs`:
     - `DoWaveStreaming` reloaded `LoopsRemaining = dwLoops` on every call, so a finite
       loop never ended.
     - The repeat was rewound from `CompleteIO`, after the commit side had already moved
       on to the next header, so the repeat played after it.
     - `CompleteIO` dropped the completion that triggered a rewind while counting partials
       across passes, so a header could be returned with chunks still in flight.
     - A completion spilling into the next header freed one buffer per header.

     The loop counts now live per header in `WAVEHDR_EXTENSION` (`LoopsToCommit`/
     `LoopsToComplete`, set once per write). `dwLoops` is honoured as "times to play",
     so 0 and 1 both mean once. The harness grades plain streaming against the C and a
     client-side model, grades loops against the model, and pins the C's defects. Five
     mutants are killed.
4. **Batch H flipped (2026-10-17):** reactos.exe, blue, ndisuio, netio, rtl8139, tdi and
   tzlib. The row in rust-port-status.md has the details. After it, the ReactOS-native
   plain-C tail is small: tdihelpers (owed below), sdk/lib/crt's `strtol` lib, normaliz's
   native part, the boot loader's `blcmlib`/`blrtl`, and cdrom's data TUs. The rest is Wine
   (needs the owner's call) or other lanes'. Measured from build-amd64/compile_commands.json,
   filtering any TU with a Wine debug channel or `wine/` include.
5. Pick the next native area from `rust-port-status.md` (what is still C and not on the
   skip list below).

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

Closed 2026-10-16: the substrate-in-declared test (`_prelude_declared_fns`), the merge
driver (`tools/merge_drivers.py`, which also merges the cohab census), and psapi's
`main.c` (psapi-rust's `dllmain` feature, built only for psapi.dll). `port resolve`'s
"10+ minutes" did not reproduce: resolve plus crate-repair took 2-7 minutes per crate on
2026-10-16. The first run per stem compiles ntkrnl-sys cold in its private
`target-<arch>`, which is the likely source of the earlier figure.

- **tdihelpers needs one shared crate** (owed since 2026-10-17). `drivers/network/tdihelpers/
  tdi.c`/`tdiconn.c` compile into BOTH afd.sys and netio.sys, so translate refuses them
  ("no proven CMake source owner"), and buying them per driver would pay twice. netio was
  ported with `port derive --keep-c tdi.c tdiconn.c`; they stay C in both drivers until a
  `tdihelpers-rust` static crate exists that both link.
- **`port macros` loses two things a hand fix then has to restore** (reactos.exe): it does
  not widen integer operands (`GetDiskSizeInBytes` needed `as ULONGLONG`) and it drops the
  `!` when one lowered macro calls another (`SetDlgItemFont` -> `SetWindowFont!`). The three
  hand edits live inside reactos-rust's generated prelude block, so a `port macros` re-run
  reverts them.
- **usetup's cabinet/file-queue code exists twice** (usetup-rust, and reactos-rust's
  `fileqsup.rs`, because the C `#include`s the .c files). One shared crate, like tdihelpers.
- **`port flip` of a static library misses cargo cells.** timedate.cpl's cell linked
  `engine:c:tzlib` (now only a stub) and failed every arch's link until it was pointed at
  `engine:tzlib_rust`. flip should find `engine:c:<target>` users and rewrite or refuse.
- **The call seal leaves a live `#ifdef` arm's calls optional** (from control-plane's owed
  list, 2026-10-17; the original report, "it demands calls in dead arms", no longer
  reproduces). `dead_regions.preprocessor_conditional_spans` marks EVERY arm of any `#if`
  in a body dead. So on rundll32's `#ifdef UNICODE WideCharToMultiByte #else strncpy`, both
  calls are optional, and a model that drops the live call passes the seal. The fix:
  - Decide arms with `dead_regions.cond_truth`, given the TU's own defines. That means
    `pinned_int_macros(module.definitions)`, plus the bare `-DNAME`s as `local` (UNICODE is
    one); `pinned_int_macros` drops them. Plumb these through `seal_calls` into
    `dead_region_spans`.
  - Live arm required, dead arm optional, an undecidable chain optional as today.
  - The prompt and cache are untouched: `call_contract` never reads `.dead`.

  It TIGHTENS an audit, though, so measure it first. Re-check every banked body against
  its C function's live-arm callees under its TU's defines. Then fix (re-buy) or allowlist
  each body that dropped one, before the change can turn a `port replay` red.
- **The live cost counter skips failed attempts.** Every attempt's `result.json` has
  `cost_usd: null`, and a body that ends in `max_tokens` is never added to the running total
  (DriveDlgProc's two 32K-token failures printed `$0.0000`). Batch H's real spend was summed
  from the attempts' token counts. The counter should bill every attempt it sent.
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
