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
   plain-C tail is small: tdihelpers (done, item 5), sdk/lib/crt's `strtol` lib (done,
   item 5), normaliz (Wine-derived after all, item 5), the boot loader's `blrtl` `unicode.c` (item 6; `blcmlib` was already `cmlib-rust --features bldr`), and cdrom's data TUs (data.c done, item 5). The rest is Wine
   (needs the owner's call) or other lanes'. Measured from build-amd64/compile_commands.json,
   filtering any TU with a Wine debug channel or `wine/` include.
5. **tdihelpers is one shared Rust library (2026-10-17, $0.06).** `tdi.c`/`tdiconn.c` are
   the `tdihelpers` STATIC target now. afd.sys links it, and netio's cell takes
   `engine:crate:tdihelpers-rust`. Their objects were byte-identical under afd's and netio's
   `_WIN32_WINNT` on all three arches, so one build serves both. The six `_SEH2_TRY` sites are
   hand-ported to `ntkrnl_sys::seh::try_except`. The C's except arms are dead under
   DUMMY_PSEH, so the Rust now really catches a failed `MmProbeAndLockPages`. tdikrnl.h's
   `TdiBuild*` request builders went into `ddk-macros-rust`. Tool fixes:
   - The `c` fact-seal no longer seals PSEH's statement macros.
   - A preamble of bare conditionals and `#undef`s is $0.
   - crate-repair's flex-array rule sees `addr_of_mut!`/`&raw mut` borrows.
   - `port flip` rewrites cargo cells' `engine:c:<target>` to the crate archive.
   - vibeos-lint's `scan_raising_probes` no longer reads a probe's name inside a string.
   **The `strtol` library is pruned ($0).** acpi.sys used only isprint/isspace/isxdigit/
   toupper from it, which cnt-difftest grades against the C. It now links them from
   libcntpr_rust.a, as hal.dll does. Linked on all three arches; smoke boots on all three,
   and amd64 smoke:net passed 16/16. `acpi_new`, which is not built, still names the library.
   **cdrom's `data.c` is Rust ($0, `rustconv.c_data_port`).** Its two hack tables went
   into cdrom-rust's `src/data`. The tool runs directly because cdrom-rust predates `port`.
   `guid.c` stays C, since its GUIDs are header DEFINE_GUIDs the data port does not emit.
   - **Header tables are left to C.** `cdrom.h`'s WDF context-type infos are selectany in
     every C includer, and the shim still includes them. Rust copies would be strong
     definitions, and the link guard refused them; the port takes only data.c's own
     tables (`headers=("…/data.c",), scope_tables=False`).
   - **Tool fixes.** An unevaluated enum constant fails only where a table uses it. An
     island's `[T; 0]` is an incomplete declaration, not a length mismatch.
   - **Smoke** passes on all three arches.
   - **normaliz is not native:** IdnToAscii/IdnToUnicode say "Taken from Wine
     kernel32/locale.c", and nameprep.c is their table. It waits for the owner's call on
     Wine.
6. **The boot loaders' RTL is Rust except `unicode.c` (2026-10-18, $0).** freeldr (BIOS),
   rosload and uefildr link `blrtl-rust`: the kernel's bitmap/compress/crc32/largeint/
   random/byteswap/mem rlibs and rtl-diag's new `image` feature, with blrtl.cmake's per-arch
   membership, plus a hand-ported nlsboot (graded vs the C: `difftest/nlsboot.json`, 14,534
   cases). Smoke reached on i386, amd64, arm64 and amd64-bios. Along the way: mbstrlen.c's
   private always-NULL `NlsLeadByteInfo` is `extern` now (it had won or lost by link
   order), and vibeos-lint's link model honours `[target.'cfg(target_arch)'.dependencies]`.
   **`unicode.c` waits for the owner's call**, because the Rust that would replace it
   (`rtl-strings`' cabi) does not do what the C does: unicode.c converts and case-maps
   through the NLS exports (`RtlUnicodeToOemN`, `RtlUpcaseUnicodeChar`, ...), and the Rust
   inlines a fixed CP1252 model instead (`rtl-strings/src/cabi/mod.rs`, e.g.
   `RtlUnicodeStringToOemString` -> `nls::uni_to_oem`). In the loader that would replace
   nlsboot's ASCII-only behaviour; in the kernel and ntdll it already ignores the tables
   `RtlResetRtlTranslations` loads (OEM is CP437 there, so bytes >= 0x80 differ from the
   C). The fix routes rtl-strings' unicode layer through the NLS exports, as the C does,
   which also changes kernel/ntdll behaviour for non-ASCII text.
7. Pick the next native area from `rust-port-status.md` (what is still C and not on the
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

- **`port macros` no longer reverts hand edits** (2026-10-17): items are `gen:`-stamped and
  an edited one leaves the block (`generated_items.py`). reactos-rust's three fixes now sit
  above their blocks; the `!` one was also fixed at the root (render applies the bang).
  Still owed: the SDK lowering does not widen a `ULONGLONG * ULONG` (it has no member
  types for a macro parameter), so `GetDiskSizeInBytes` stays a hand port; and resolve's
  generated blocks have no stamps yet.
- **usetup's cabinet/file-queue code is one crate now** (2026-10-17): `spapisup-rust`, an
  rlib usetup-rust and reactos-rust both link, $0 (the two copies were the same banked
  translation; usetup's was moved, its island generated by the hand-written `spapisup`
  entry in gen_driver_types.sh). usetup.exe and reactos.exe link on all three arches.
  reactos-rust's superseded `fileqsup.rs` and the prelude section only it used were deleted
  with the user's approval (2026-10-18).
- **The call seal requires a live `#ifdef` arm's calls** (2026-10-17). `seal_calls` takes
  the TU's defines (`_module_pinned_macros(.., bare_as_one=True)`: a bare `-DNAME` and a
  UNICODE module's UNICODE count as 1) and `dead_regions.decided_live_lines` decides arms:
  live required, dead and undecidable and `#elif` optional. Prompt and cache untouched.
  Measured first with `python -m rustconv.live_arm_audit` over the 986 flipped TUs the
  census resolves: 445 live-arm calls in 208 functions, and every shipped body names its
  callee except freeldr's `WinLdrSetupMemoryLayout`, which dropped a `#if DBG` `MempDump()`
  (restored, under the crate's `debug_assertions` spelling of DBG; the rest of its 8 leads
  are macros the seal never tokenizes). Caveats: that audits the
  INTEGRATED Rust, not the bank -- a replay could not check the bank, because a flipped TU
  has no compile command and translate refuses it (`MacroAuthorityError`); 119 census rows
  do not resolve to a file pair, and 162 TUs whose CMakeLists profile_derive refuses were
  decided on tree-wide defines only. Still owed: the seal sees only the body, so a
  file-scope `#define` (beepmidi's `CONTINUOUS_NOTES`) still leaves its arms optional.
- **The cost counter now bills failed attempts** (2026-10-17). Three causes: the Claude
  backend never set `usage_known`, so all 6,637 attempt `result.json`s said
  `cost_usd: null`; FAIL lines did not print the running figure `port translate` bills a
  run by; and `[running $X.XX]` rounded a Haiku run to $0.00. The old records stay null;
  their token counts are intact if a past run needs re-pricing.
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
