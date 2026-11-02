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
     kernel32/locale.c", and nameprep.c is their table. The owner approved it: item 7.
6. **The boot loaders' RTL is Rust except `unicode.c` (2026-10-18, $0).** freeldr (BIOS),
   rosload and uefildr link `blrtl-rust`: the kernel's bitmap/compress/crc32/largeint/
   random/byteswap/mem rlibs and rtl-diag's new `image` feature, with blrtl.cmake's per-arch
   membership, plus a hand-ported nlsboot (graded vs the C: `difftest/nlsboot.json`, 14,534
   cases). Smoke reached on i386, amd64, arm64 and amd64-bios. Along the way: mbstrlen.c's
   private always-NULL `NlsLeadByteInfo` is `extern` now (it had won or lost by link
   order), and vibeos-lint's link model honours `[target.'cfg(target_arch)'.dependencies]`.
   **`unicode.c` is Rust too (2026-10-18, owner's call: make it faithful, $0)**, and the
   change reaches the kernel and ntdll. rtl-strings' unicode layer called a fixed CP1252
   model in every build, so the kernel ignored the tables `RtlResetRtlTranslations` loads
   (OEM is 437: bytes >= 0x80 differed from the C). Now:
   - `cabi/nlsapi.rs` routes unicode.c's NLS calls per build: nls_tables.rs (kernel/ntdll),
     nlsboot (`bldr`, the loaders), the model (the host harness only).
   - nls_tables.rs gained table-driven ports of nls.c's 12 converters/case helpers (they
     were the model); `rtl-difftest/tests/nls_tables.rs` grades them against genuine nls.c
     on txt2nls-generated 1252/437 and 932 (DBCS) tables, every WCHAR and byte.
   - The wrappers free and NULL the destination on a failed allocating conversion, as the C
     does; `RtlAnsiCharToUnicodeChar` honours lead bytes; `RtlIsValidOemCharacter` walks the
     tables; `RtlEqualComputerName` was `return FALSE` (the harness's NULL-alloc outcome
     shipped) and is the C now.
   - `bldr` exports exactly unicode.c's 61 functions; the rest of librtl sits in
     `librtl_rest`, compiled out for the loaders. boot/rtl's C target is pruned.
   - librtl's crates take ntkrnl-sys without `hal_bindgen`: it raw-dylib-imported the three
     Nls* globals librtl itself defines, and amd64 ntdll's link crashed lld over it.
   Smoke reached on i386, amd64, arm64 and amd64-bios.
7. **normaliz is Rust (2026-11-01, owner's call on this Wine-derived DLL, $0.03).** All four
   TUs went through `rustconv port`: `nameprep.c`'s two tables as `port data` ($0), the
   other three as 9 sealed Haiku functions. `cargo_cell_gen` moved the link to
   `normaliz-cargo`, which is PE-EQUIVALENT to the cmake build on all three arches, and smoke
   reached the marker on i386, amd64 and arm64. `strlenW`/`tolowerW` (ReactOS
   `wine/unicode.h`'s `wcslen`/`towlower`) went into `win32macros-rust` for every Wine crate.
   - **Two Wine bugs, fixed in the Rust with divergence comments.** `IdnToUnicode` bounds
     its punycode insert by `cchASCIIChar`, the INPUT length, so a shorter output buffer is
     overrun; and under `IDN_USE_STD3_ASCII_RULES` an input of just `L"\0"` reads
     `lpASCIICharStr[-1]` (an empty label's "last character"). The host harness found the
     second: the genuine C faulted on its guard page.
   - **Graded vs the genuine C** (`difftest/normaliz_ascii.json`, 13,344 cases;
     `normaliz_unicode.json`, 1,060 curated cases): 0 mismatches, every mutant detected.
     Two harness features made it possible: `c_env_probe` (the shared env reports state
     after each call, here the last error and live heap blocks, so an error code or a leak
     is a mismatch) and `{"arg": n, "mul": 2}` extents for WCHAR-counted buffers.
   - **`port data` flips CMake, and ran under a live build.** It comments the data TU out
     of the source list, so the running arm64 build linked normaliz.dll without the tables
     and said LINK_FAIL, nothing hinting that the tree had moved. Fixed both ways
     (2026-11-01): `port flip`/`port data` refuse while a build dir of THIS tree holds its
     broker lock, and `build` reclassifies any FAILURE over a moved tree as
     BUILD_INPUTS_MOVED naming the original verdict (a `Run.finalize_hooks` entry, so every
     early return gets it; the success path already refused to stamp a moved tree).
8. **Survey: the native C still compiled (2026-11-01, amd64 `compile_commands.json`).**
   Excluding vendored trees, `sdk/tools`, rostests, and every module DIRECTORY whose sources
   carry a Wine marker (a debug channel, a `wine/` include, a Wine/CodeWeavers notice; a
   per-file test let mshtml, `directx/wine` and setupapi through via their precomp.h):
   **~20,900 non-blank lines in 237 TUs across 122 directories.** Most of the bulk is not
   ours: `dll/win32/wow64` (8,422, fex-arm) and `wow64cpu`, `bootvid` (~1,000,
   control-plane B), `shimgvw` (C++ lane), and the audio `backpln` bits. Ours, by size:
   - **GUID data libraries:** `uuid`, `strmiids`, `dxguid` (~1,960 lines, 1,841
     DEFINE_GUIDs). **They stay C (decided 2026-11-01).** Under `INITGUID`, `guiddef.h`
     defines each GUID `DECLSPEC_SELECTANY`, a COMDAT that every TU including the header
     with INITGUID may also emit; the linker keeps one. Stable Rust has no weak or
     selectany definition (the `linkage` attribute is nightly), so Rust copies would be
     strong symbols and collide with every C or widl TU that still emits the same GUID.
     The content is pure data with no logic to grade, and most of it is not ours anyway:
     `dxguid` is Wine's (Julliard/Gouget), `uuid` is mostly widl output from the IDL list
     in its CMakeLists, and `strmiids`' `evr.c` is Wine's. Teaching `port data` to emit
     header GUIDs would buy nothing while selectany can't be expressed.
   - **The CRT's own residue:** `crt/math` (43 TUs incl. `arm64/`), `crt/printf`,
     `crt/misc`, `crt/except/amd64`, `crtheap`, `vcstartup`, `gcc-compat`. This is the
     msvcrt lane's, i.e. ours; read the CRT rows in `rust-port-status.md` before
     starting, since some are deliberately C (startup, compiler support).
   - **A tail of small native libs and DLLs, 60-420 lines each:** `epsapi`, `lsalib`,
     `scrnsave`, `streamci`, `batt`, `shimlib`, `aux_klib`, `dxapi`, `acledit`,
     `udmihelp`, `sdk/lib/nt`, `tdilib`, `conutils`, `ksuser`, `devcpux`, `sfc`, `sfc_os`,
     `dmilib`, `wdfdriverentry`, `esent`, `wing32`, `skiplist`, and a few win32ss
     stragglers. One `rustconv port` batch, a few dollars on Haiku.
   Re-measure before starting; the script is in this commit's message.
9. **Batch J: the small native tail is Rust (2026-11-01, Haiku, $0.13 for 129 calls).**
   19 modules through `rustconv port`: epsapi, scrnsave, streamci, batt, shimlib,
   aux_klib, dxapi, acledit, udmihelp, `sdk/lib/nt`, tdilib, ksuser, devcpux, sfc, sfc_os,
   dmilib, wdfdriverentry, esent, wing32. The 8 PE modules among them (streamci, dxapi,
   acledit, ksuser, devcpux, sfc, sfc_os, wing32) are cargo-linked, PE-EQUIVALENT to the
   cmake build on all three arches. Hand fixes worth knowing, all banked with `port hand`:
   - **An ntoskrnl DATA export goes through its `__imp_` slot.** aux_klib's
     `KiBugCheckData` came through the island as a plain `extern static`, asking the link
     for a symbol no driver defines (wdf01000/wdfldr failed). The import is spelled like
     bootvid's `KeLoaderBlock`: `link_name = "_imp__X"` on i386, `"__imp_X"` elsewhere.
   - **A spec that pre-decorates its names** (`dxapi.spec`'s `_DxApi@20`) made
     `cohab_stub` anchor `__DxApi@20@20` on i386 and the i386 spelling on the 64-bit
     arches. `cohab_stub._c_name` now strips it the way spec2def does.
   - **`port flip` drops a module's C target even when C stays** (shimlib's varargs shim,
     batt's and esent's `rust_cohab_stub.c`). `port flip` now repoints the cargo cells
     that named the old C target to BOTH `engine:c:T` and the body crate when C sources
     remain (`_repoint_cargo_cells(.., keeps_c)`). Still owed: it does not reshape a
     module that is itself a C-body cargo cell; batt and esent were done by hand
     (`<m>-cargo/build.rs`: `.rust_staticlib("<m>-rust")` plus
     `c_from_compile_db("<m>_shims", [".../rust_cohab_stub.c"])`).
   - **`scan_bitwise_not_bool` caught a shipped-to-be bug:** devcpux's `AddFeature` came
     back as `if !*bFirst != 0`, Rust's bitwise NOT of a `BOOL`, true for every value but
     -1, so every CPU feature got a leading separator. Now `*bFirst == 0`.
   - The 9 static libraries are baselined in `scan_cmake_linked_rust`, like tdihelpers:
     `cargo_cell_gen` refuses a static library (no PE to link).
   - esent's 12 `Jet*` exports lacked `#[no_mangle]`; scrnsave's entry is `wWinMain`;
     dxapi's `DxApi` calls cdecl handlers through a stdcall PFN, as the C does (comment).
   **Deferred:** user-mode `lsalib`. Its Rust twin is ntkrnl-sys-bound and needs a crate
   feature design for a user-mode build. conutils (variadic shims only) and skiplist (a test
   exe) are not translation targets.
10. **CRT residue triage (2026-11-01, measured, not guessed).** Item 8 counted ~120
   compiled TUs under `sdk/lib/crt`. What WINS a link is far less. Method: per arch,
   `llvm-nm --defined-only -g` each C archive against the Rust archives linked ahead of
   it, then intersect the C-only names with the PE's export table (`llvm-readobj
   --coff-exports`); the archives each PE links come from `build.ninja` and the
   `*-cargo/build.rs` link orders.
   - **crtmath is linked by NO PE on any arch.** ucrtbase.dll's `build.ninja` edge lists
     `libcrtmath.a` only as an IMPLICIT dependency (after the `|`); its real link line is
     `ucrtbase_stubs.rs.obj` + `crt_shared_cargo` + kernel32/ntdll. Reading the implicit
     deps as inputs (as this item first did) makes `_cabs` look like crtmath's C, when
     ucrtbase.spec's `-stub _cabs` is served by the spec2def stub; a Rust `_cabs` then
     fails the link as a duplicate. Measure link inputs from the explicit inputs and
     `LINK_LIBRARIES` only.
   - **libcntpr** (the kernel CRT; no PE links it through cmake, but 22 cargo cells name
     `engine:c:libcntpr` AHEAD of `cntpr_rust`; 17-20 PEs per arch link it) defines 30 (arm64) / 42 (amd64) / 98
     (i386) names. Overlap with Rust: only `atol` and `exp2f`, and there the C wins by
     order. The rest has no Rust twin:
     - **amd64 `except/amd64/ehandler.c`** (`__C_specific_handler`, `_local_unwind`):
       live, ours (ReactOS MIT), compiled into ntoskrnl's and ntdll's cells, freeldr,
       rosload and libcntpr. The one substantive piece: Rust now (item 11).
     - **never linked:** `_mbstrlen`, `exp2f`, `_invoke_matherr`/`__setusermatherr`, and
       winesup's `_internal_handle_float` have no callers (libcntpr.cmake's own notes),
       so their archive members are never pulled. Not worth buying.
     - **stays C:** every printf/scanf shell, the `va_list` ones included. Defining a C
       variadic needs `c_variadic` and consuming a `va_list` needs `core::ffi::VaList`,
       both nightly; they are `va_start; streamout(...)` shells over the Rust engine.
       Also the asm (`chkstk`, `seh.s`, i386 `all*`/`ftol*`), `wine_wctype_table`
       (Wine's data), and `atol.c`, which also defines `_atoldbl` (no Rust twin yet).
     - **i386 `_CI*` / `__libm_sse2_*`:** done, item 12.
   - **Also stays C:** `memcmp` (~340 PEs per arch; AGENTS.md: LLVM folds a Rust byte
     loop back into `memcmp`); `vcstartup` (C++ startup objects, linked by avifil32 and
     d2d1 only); `getopt` (libtiff only). `dbgrpt` and `chkesp` reach no PE.
11. **amd64 `__C_specific_handler` is Rust (2026-11-02, $0, hand port).** `ehandler.c` →
   `rtl-except/src/amd64/ehandler.rs` (`__C_specific_handler`, `_local_unwind`), which
   ntoskrnl and ntdll link through rtl-rust; both still export it. The C left every list:
   ntoskrnl's two cells, ntdll's, libcntpr (amd64 REMOVE_ITEM, and its clib recipe), and
   freeldr/rosload/uefildr, which never referenced it (no SEH in the boot loaders; they
   link without it). Drivers that took the C statically from libcntpr now import it from
   ntoskrnl, as Windows drivers do. Gate ×3 green
   (`/tmp/vibeos-cli/20261102-001430-k2z_6ff8-gate-i386+amd64+arm64`); amd64 boots to the
   desktop through it. It is NOT the arm64 handler (`arm64/except.rs`), which is a
   different algorithm: this one bumps `ScopeIndex` before each scope, stops a target
   unwind at the scope containing the target or whose `__except` is the target, and
   filters on any `HandlerAddress` other than the literal 1.
   - **`cargo_cell_gen --convert` dropped new cells' recipes silently.** Its
     `refresh_manifests` skips a library with no compile command (fine for a library
     absent on some arch), but a converted cell's own `<m>_shims` has none until cmake
     reconfigures, so normaliz's and batch J's eight `*_shims.recipe` sets were never
     written. `convert` now fails loud naming each missing (arch, library) and the fix
     (`missing_cell_recipes`); the missing recipes are committed.

12. **The i386 `_CI*` thunks are Rust everywhere (2026-11-02, $0).** Wine's
   CREATE_FPU_FUNC1/2 `global_asm!` thunks moved out of `crt-shared-rust` into a
   dependency-free `ci-thunks` crate: feature `all` (sixteen) for the CRT DLLs, `kernel`
   (cos, sin, sqrt, log, pow) for ntoskrnl and ntdll, which take it through
   `cnt-rust-lib`'s new `ci` feature on i386 only. Their cells drop crtmath's
   `ci{cos,sin,sqrt,log,pow}.c`. These thunks also spill and restore the rest of the
   caller's x87 stack around the call, where the C only popped the operand. Export
   counts are unchanged (ntoskrnl 3, ntdll 5, msvcrt/ucrtbase 16), and ntoskrnl's
   `_CIsqrt` disassembles to the `fxam` spill loop calling the kernel's `_sqrt`.
   Nothing else reached them: no compiler we use emits `_CI*` or `__libm_sse2_*`, so
   libcntpr (i386 REMOVE_ITEM) and freeldr/rosload/uefildr drop `ci*.c` and
   `libm_sse2.c` and still link. Both consumers name it with `extern crate
   ci_thunks;`: since edition 2018 rustc only loads a dependency something references,
   and nothing calls a `global_asm!`-only crate.


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
  types for a macro parameter), so `GetDiskSizeInBytes` stays a hand port. Typing a
  member by field NAME across the island cannot settle it (measured 2026-11-01:
  reactos-rust's `SectorCount` is `c_uint`, `LARGE_INTEGER` or `ULARGE_INTEGER`, and the
  two with `.QuadPart` disagree on sign); it needs the parameter's type at the C call
  sites, a clang `__typeof__` probe per site. Low value while the two hand ports are
  stamp-protected, so parked. **resolve's
  tree-consensus block is stamped (2026-11-01):** `render` stamps each item, and
  `preserve_hand_edits` moves an edited one above the block (an `extern` declaration in
  its own `extern "ABI" {}`) before the run re-derives the rest from the tree, which used to
  overwrite it. All 269 such blocks in the tree round-trip unchanged. resolve's other
  generated blocks (cross-TU, substrate, facts, local types, externs, INITGUID, TCHAR,
  wine channels) are still unstamped.
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
  decided on tree-wide defines only. **File-scope `#define`s count too (2026-11-01):** the
  seal finds the function's definition in the TU by name (exactly one) and seeds the walker
  with what the lines above it, compiled by every build, `#define` and do not `#undef`
  (`dead_regions.file_scope_defines`), so beepmidi's `CONTINUOUS_NOTES` arms are required.
  A body it cannot place sees none, as before. Measured first: `live_arm_audit` (which
  already walks whole files, a superset) reports 475 live-arm calls and 7 the Rust does
  not name, all macros or a cast the seal never tokenizes, so no banked body turns red.
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
