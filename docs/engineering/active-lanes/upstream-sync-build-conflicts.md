# upstream-sync: build-file conflict recipes (Rust cohab-flip risk)

Fork point `FP=5213cf717cff6946945922e6a7e03d87b0a58e4f`, merging `upstream/master`
(`2def708122ac27e642f77f656cc3eb2a200aa640` at analysis time). A dry-run
(`git merge-tree --write-tree --name-only HEAD upstream/master`) reports 281 conflicts, 81 in
CMake files. Of those 81, these 8 contain a commented-out source reference — i.e. these are the
ones where a wrong resolution can silently re-enable dead C next to its live Rust replacement
(the exact failure class that froze the guest on 2026-09-19: a partial cohab flip, smoke green,
guest dead).

This document is a set of **recipes**, not a patch. Apply each by hand during the real merge,
re-deriving the three-way inputs at that time (the base/ours/theirs blobs may have shifted if
either branch moves before the merge lands).

## Summary table

| File | Upstream commits touching it | At-risk comment-outs | Difficulty |
|---|---|---|---|
| `boot/freeldr/freeldr/CMakeLists.txt` | 3 | ~50 (whole file restructured to Rust) | NEEDS-CARE |
| `boot/freeldr/freeldr/rosload.cmake` | 7 | ~25 (whole file restructured to Rust) | ASK-OWNER (ke/config.c, advopts.c gaps) |
| `dll/3rdparty/libtiff/CMakeLists.txt` | 3 | 0 (vendored, no cohab here) | MECHANICAL |
| `dll/cpl/sysdm/CMakeLists.txt` | 1 | 12 (all commented) | MECHANICAL |
| `dll/win32/kernel32/CMakeLists.txt` | 2 | ~50 (whole SOURCE list commented) | NEEDS-CARE |
| `dll/win32/shell32/CMakeLists.txt` | 6 | ~55 (whole SOURCE list commented) | NEEDS-CARE (CSimpleShellItemArray gap, flag only) |
| `dll/win32/syssetup/CMakeLists.txt` | 2 | 8 (all commented) | NEEDS-CARE (addons.c/wizard.c wiring gap, flag only) |
| `ntoskrnl/ntos.cmake` | 2 | ~150+ (whole file restructured to Rust) | MECHANICAL for the actual conflict (cpuinfo.c is a clean commented-add) |

None of the eight risk a **build-breaking dual-definition** if the recipes below are followed —
every already-flipped file that upstream still lists gets re-commented, not re-enabled. Two
files (`rosload.cmake`, `syssetup/CMakeLists.txt`) surface a **functional gap**: upstream wired
new logic into a `.c` file we've already flipped to Rust, and that logic doesn't exist in our
Rust yet. That gap is real regardless of how this CMake conflict is resolved — it lives in the
`.c`-file merge (out of this document's scope, since it's not a CMake conflict), but it explains
*why* a certain line should stay commented rather than be treated as "just enable it."

---

## `boot/freeldr/freeldr/CMakeLists.txt`

**Upstream** (3 commits): `a9544043208` "[FREELDR] Improve build generation (#9521)",
`925d227faaf` "Move NtLdrUpdateLoadOptions() to ntldropts.c and rename it", `4235244ac9e` "Move
the NTLDR-specific files from the generic CMakeLists file to rosload/uefi.cmake". Net effect:
removed the `ntoskrnl/include` etc. `include_directories()` block (redundant with target-level
includes elsewhere), deleted the entire `FREELDR_NTLDR_SOURCE` list from this file (moved to
`rosload.cmake`/`uefi.cmake` — the NTLDR-specific loader files no longer belong to the generic
freeldr/uefildr target), dropped the amd64 `arch/i386/ntoskrnl.c` and `ntldr/arch/amd64/winldr.c`
entries, changed `arm` → the file still doesn't support arm64 upstream, and reworded a comment
on `ntldr/ntldropts.c`.

**We** did a full Rust cohabitation flip of essentially every `.c`/`.S` file freeldr/uefildr
used to compile, replacing `add_executable`+cmake-driven asm with a cargo-driven link
(`sdk/lib/rust/freeldr-rust`), and switched `elseif(ARCH STREQUAL "arm")` to
`elseif(ARCH STREQUAL "arm64")`.

**At-risk comment-outs in ours** (the ones a wrong resolution could re-enable): every entry
under `FREELDR_ARC_SOURCE`, `FREELDR_BOOTLIB_SOURCE`, `FREELDR_BOOTMGR_SOURCE`,
`FREELDR_NTLDR_SOURCE`, and `FREELDR_BASE_SOURCE` — e.g. `# arch/i386/i386bug.c`,
`# ntldr/winldr.c`, `# freeldr.c`, `# lib/rtl/libsupp.c`. None of these lines are touched by
upstream's diff *content* (upstream never re-adds a `.c` upstream had deleted); the conflicts
are purely structural (both sides rewrote the same region).

**Conflicting hunks** (5, via `git merge-file --diff3`):

1. `FREELDR_BOOTMGR_SOURCE`/`FREELDR_NTLDR_SOURCE`: base and theirs are IDENTICAL for the
   `BOOTMGR_SOURCE` half (`settings.c`, `ui/*.c` unchanged); theirs deletes `FREELDR_NTLDR_SOURCE`
   entirely (moved to rosload.cmake — see below); ours commented out both halves.
   **Resolution: take ours** (all commented, Rust-owned) — theirs' deletion of
   `FREELDR_NTLDR_SOURCE` is compatible with "already commented," and upstream's move target
   (rosload.cmake) is handled in that file's own recipe.
2. i386 branch (`i386bug.c`, `ntldr/headless.c`, `ntldr/arch/i386/winldr.c`): theirs keeps
   `i386bug.c` active, drops `headless.c`+`winldr.c` (moved to rosload.cmake). Ours comments
   all three out. **Resolution: take ours.**
3. amd64 branch (`i386bug.c`, `ntoskrnl.c`, `ntldr/arch/amd64/winldr.c`): same shape.
   **Resolution: take ours.**
4. `elseif(ARCH STREQUAL "arm")` vs `elseif(ARCH STREQUAL "arm64")`: this is **not really a
   content conflict** — upstream never added arm64 support here (still says `arm`, `#TBD`); we
   renamed the branch when the project moved to arm64-only. **Resolution: take ours**
   (`arm64`, with our `winldr_arm64.rs` comment) — reverting to `arm` would delete arm64 support
   for freeldr.
5. `FREELDR_BASE_SOURCE` (`freeldr.c`, `ntldr/ntldropts.c`, `lib/rtl/libsupp.c`): theirs only
   changed the trailing comment on `ntldropts.c` ("Should be" → "Is... needed by machpc.c and
   ramdisk.c"); the files themselves are untouched by upstream. Ours commented all three out
   (Rust-owned). **Resolution: take ours** — the comment-text update is superseded by our own
   "→ Rust... Cohabitation" note on the same line; no functional content is lost.

**Recommended resolution: take OURS for every one of the 5 hunks.** This file needs no manual
blending — every upstream change here is either (a) a pure move-away that our comment-out
already anticipates, or (b) a comment-text tweak on a line we've already retired. The one thing
to verify post-merge: confirm `include(rosload.cmake)` is still guarded by
`if(NOT ARCH STREQUAL "arm64")` as in our version (upstream doesn't have arm64 at all, so this
guard is ours to keep) and that `include(uefi.cmake)` still runs unconditionally after it — this
part of the file (near the bottom, past the conflict zone) merges cleanly since only we touched
it, but double-check after resolving the 5 hunks above.

---

## `boot/freeldr/freeldr/rosload.cmake`

**Upstream** (7 commits, most relevantly `4235244ac9e`, `ea93b886df7`, `62deebac2d8`,
`9ff4e4b9ef4`, `b2e33f26eb3`, `925d227faaf`, `a9544043208`): moved the NTLDR-specific files INTO
`ROSLOAD_SOURCE` from the main `CMakeLists.txt` (see above), and added FOUR new entries to the
common `ROSLOAD_SOURCE` list:
- `${REACTOS_SOURCE_DIR}/ntoskrnl/ke/config.c` — **new to this list**, brand new caller.
- `ntldr/advopts.c` — **brand new file** (commit `9ff4e4b9ef4`, "Move the ReactOS/NT-specific
  advanced boot menu to its separate file").
- `ntldr/headless.c` — moved here from the i386-only block (now common to all arches).
- `ntldr/ntldropts.c` — moved here from the main CMakeLists.txt's `FREELDR_BASE_SOURCE`.

Also cosmetic reflow (trailing-paren style) and rosload-target changes unrelated to sources
(`target_include_directories`, link-library reordering, `add_cd_file` FOR-list trim).

**We** rewrote this file as a second, near-total Rust cohab flip: the whole `ROSLOAD_SOURCE`
list is commented out (`bootmgr.c`, `custom.c`, `linuxboot.c`, `miscboot.c`, `options.c`,
`oslist.c`, `lib/rtl/libsupp.c`, `cmboot.c`, `ntldr/conversion.c`, `ntldr/inffile.c`,
`ntldr/registry.c`, `ntldr/setupldr.c`, `ntldr/winldr.c`, `ntldr/wlmemory.c`,
`ntldr/wlregistry.c`, plus the i386/amd64 arch blocks), and the final link is driven by
`add_cargo_module(rosload rosload-cargo ...)` instead of `add_executable`.

**At-risk comment-outs**: all ~25 entries above — every one already covered by
`freeldr-rust`'s `rosload` feature (verified: `headless.rs` and `ntldropts.rs` both exist in
`sdk/lib/rust/freeldr-rust/src/`).

**Recommended resolution, mechanical part:** take ours wholesale for everything upstream just
moved or reflowed — `ntldr/headless.c` and `ntldr/ntldropts.c` are already Rust-owned
(`freeldr-rust/src/headless.rs`, `freeldr-rust/src/ntldropts.rs`); add them to our comment block
in the common `ROSLOAD_SOURCE` section (matching local style, e.g.
`# ntldr/headless.c  # replaced by Rust (freeldr-rust/src/headless.rs)`) rather than leaving them
only under the old i386-only comment, so the file's structure tracks upstream's (headless.c is
now arch-common upstream, not i386-only).

**ASK-OWNER (two items — do not resolve mechanically):**

1. **`ntoskrnl/ke/config.c`.** This file is *already* cohab-flipped for the **kernel** target
   (`ntoskrnl/ntos.cmake:187` has `#${REACTOS_SOURCE_DIR}/ntoskrnl/ke/config.c`, and
   `sdk/lib/rust/ntoskrnl-rust/src/ke/config.rs` provides `#[no_mangle] KeFindConfigurationEntry`
   / `KeFindConfigurationNextEntry`). But `ntoskrnl-rust` is the **kernel's own crate** — it is
   NOT linked into `rosload-cargo`. Upstream adds `ke/config.c` to `ROSLOAD_SOURCE` because
   `ntldr/winldr.c` gained new calls to `KeFindConfigurationEntry` (confirmed:
   `git show upstream/master:boot/freeldr/freeldr/ntldr/winldr.c` — 3 call sites building the
   ARC boot-device configuration tree; this logic did **not** exist in `winldr.c` at the fork
   point — `grep KeFindConfiguration` on the fork-point `winldr.c` finds nothing). Our
   `freeldr-rust/src/winldr.rs` was translated from the old, pre-this-feature `winldr.c`, so it
   also does not call `KeFindConfigurationEntry` (`grep -rn KeFindConfiguration
   sdk/lib/rust/freeldr-rust/` — zero hits). **Simply commenting this line out (matching local
   convention) is SAFE today** (nothing in rosload references the symbol yet) but is coupled to
   the separate `winldr.c` merge conflict: if that conflict is resolved by pulling in upstream's
   new ARC-boot-device logic (even partially, even by hand-porting later), `rosload-cargo` will
   need a real provider for `KeFindConfigurationEntry`/`KeFindConfigurationNextEntry` — either a
   new Rust translation of `ke/config.c` reachable from rosload (candidate home:
   `sdk/lib/rust/config-rust`, which already exists but implements different CM-config logic —
   confirmed by reading `config-rust/src/cmconfig.rs`, it is NOT the same functions), or
   re-enabling the tiny `ke/config.c` as plain C linked into rosload only. **Owner should decide:
   (a) leave commented + track "ARC boot-device config tree" as an owed translation task
   alongside the `winldr.c` conflict, or (b) add `ke/config.c` active-but-rosload-only now as a
   stopgap.** I did not resolve this because it crosses into a `.c`-file conflict outside this
   document's scope and a wrong guess here either silently drops working upstream functionality
   or risks a rosload-only symbol duplication if `ntoskrnl-rust` is ever also pulled in.
2. **`ntldr/advopts.c`.** Brand new file (extracted the "NT advanced boot options" menu out of
   `options.c`/`oslist.c`, both of which we've already flipped to Rust — `options.rs`,
   `oslist.rs` — per commit `bac97c58e25` "Rework the contents of the FreeLoader setup,
   custom-boot, and NT advanced boot options menus" + `9ff4e4b9ef4`). Since this is logic
   extracted from files we already translated, the **subtle-case policy applies**: add it
   commented out, matching convention, IF our existing `options.rs`/`oslist.rs` translation
   already covers the advanced-boot-menu behavior it was split from. **I could not verify this
   from CMake alone** — it requires diffing `advopts.c`'s logic against what's actually in
   `options.rs`/`oslist.rs` today, which is a `.rs`-vs-`.c` content question, not a CMake one.
   Flag as ASK-OWNER: confirm coverage before commenting `advopts.c` out, or treat it as a
   genuine new-functionality gap (like `ke/config.c` above) if coverage turns out partial.

---

## `dll/3rdparty/libtiff/CMakeLists.txt`

**Vendored** (`vendor.yaml` line 1268-1272: `dll/3rdparty/libtiff`, no Rust translation
target). **Upstream** (3 commits, most relevant `47a91101ace` "Use target_include_directories()"
and `dcd45a59d15` "[LIBJPEG][LIBPNG][LIBTIFF] Update"): removed the `include_directories()`
block (superseded by `target_include_directories`), enabled `tif_jpeg.c` (previously commented
`#tif_jpeg.c` — this is a **pre-existing, non-cohab comment-out** unrelated to any of our
translations, just libtiff's own "not wired yet" marker that upstream is now wiring), added
`tif_hash_set.c`, added a `libjpeg` importlib and `${PSEH_LIB}`, added `-Wno-array-parameter`.

**We** independently added the exact same `tif_hash_set.c` line (with an explanatory trailing
comment: `# new in 4.5; tif_dirread.c requires it`) — nothing else.

**At-risk comment-outs:** none. `#tif_jbig.c`/`#tif_jpeg.c` predate the fork and are not
cohab-flip markers (this whole directory is vendored, not translated).

**Conflicting hunk:** `git merge-file --diff3` auto-resolves the entire file except a single
one-line collision where both sides added `tif_hash_set.c` at the same spot (ours with a
comment, theirs without).

**Recommended resolution: MECHANICAL.** Accept the full auto-merge; for the one real conflict,
keep our annotated form: `tif_hash_set.c   # new in 4.5; tif_dirread.c requires it`. Everything
else (jpeg enablement, importlibs, compiler flags) is a pure upstream vendor-code change with no
cohab interaction — take theirs as auto-merged.

---

## `dll/cpl/sysdm/CMakeLists.txt`

**Upstream** (1 commit, `b5db92a5064` "[CMAKE] Improve build generation"): removed the
`include_directories(${REACTOS_SOURCE_DIR}/sdk/lib/udmihelp)` block and moved
`add_pch(sysdm precomp.h SOURCE)` earlier in the file. Checked `sdk/lib/udmihelp/CMakeLists.txt`:
upstream changed `udmihelp` to `add_library(udmihelp STATIC ...)` +
`target_include_directories(udmihelp INTERFACE ${CMAKE_CURRENT_SOURCE_DIR})` — the include path
is now propagated transitively through `target_link_libraries(sysdm udmihelp ...)`, making the
explicit `include_directories()` block in `sysdm`'s own file redundant.

**We** did a full Rust cohab flip: all 12 `.c` files commented out, replaced by
`sysdm-rust` linked as `${SYSDM_RUST_LIB}`; only `rust_varargs_shim.c` remains a real compiled
source (a C-variadic shim, not a translation reversion — confirmed by reading the file, its
`va_start` forwards straight to `*_v` Rust bodies).

**At-risk comment-outs:** all 12 (`advanced.c`, `environment.c`, `general.c`, `hardprof.c`,
`hardware.c`, `licence.c`, `performance.c`, `smbios.c`, `startrec.c`, `sysdm.c`,
`userprofile.c`, `virtmem.c`). None touched by upstream's diff content.

**Conflicting hunk:** one large region (lines 3–74 of the 3-way merge) because both sides
rewrote overlapping context, but the actual disagreement is narrow: upstream drops 2 lines
(`include_directories(...udmihelp)`), we added ~70 lines of Rust-build wiring in the same spot.

**Recommended resolution: MECHANICAL — take ours in full, MINUS the 2-line
`include_directories(${REACTOS_SOURCE_DIR}/sdk/lib/udmihelp)` block at the top**, since (a)
`rust_varargs_shim.c` doesn't include any udmihelp header (verified: its only includes are
`stdarg.h`/`windef.h`/`winbase.h`/`windows.h`), and (b) `sysdm-rust`'s cargo build doesn't consult
CMake include paths at all. Dropping it matches upstream's own reasoning and is provably safe.

---

## `dll/win32/kernel32/CMakeLists.txt`

**Upstream** (2 commits): `c81bc9f21ea` "Split off some ANSI functions from the bulk of
Kernel32 (#8555)" and `19d57369ca5` "Port GetProcessDEPPolicy and related functions from Wine
10.1". The first adds **14 new `*ansi.c` files** by splitting ANSI-suffixed wrapper functions out
of existing files: `client/atomansi.c` (from `atom.c`), `client/dep.c` (see below — actually
new, not a split), `client/pathansi.c` (from `path.c`), `client/procansi.c` (from `proc.c`),
`client/synchansi.c` (from `synch.c`), `client/sysinfoansi.c` (from `sysinfo.c`),
`client/versionansi.c` (from `version.c`), `client/file/copyansi.c` (from `file/copy.c`),
`client/file/diransi.c` (from `file/dir.c`), `client/file/fileinfoansi.c` (from
`file/fileinfo.c`), `client/file/moveansi.c` (from `file/move.c`), `client/file/mntpointansi.c`
(from `file/mntpoint.c`), `client/file/volumeansi.c` (from `file/volume.c`),
`winnls/string/localeansi.c` (from `winnls/string/locale.c`), `winnls/string/lcformatansi.c`
(from `winnls/string/lcformat.c`). The second adds **`client/dep.c`** — genuinely new
functionality (`GetSystemDEPPolicy`/`GetProcessDEPPolicy`/`SetProcessDEPPolicy`, ported from
Wine), not a split of anything pre-existing.

**We** did a full cohab flip: the entire `SOURCE` list (all the pre-split originals:
`atom.c`, `path.c`, `proc.c`, `synch.c`, `sysinfo.c`, `version.c`, `file/copy.c`, `file/dir.c`,
`file/fileinfo.c`, `file/move.c`, `file/mntpoint.c`, `file/volume.c`, `winnls/string/locale.c`,
`winnls/string/lcformat.c`, plus everything else) is commented out, replaced by
`kernel32-rust`/`kernel32-vista-rust`/`kernel32-shared-rust`.

**Verified (this is the load-bearing check for this file): every one of the 14 ANSI-split
symbols already exists as `#[no_mangle]` in `kernel32-rust`**, e.g.
`GlobalAddAtomA`/`GlobalFindAtomA` in `src/client/atom.rs`, `GetShortPathNameA` in
`src/client/path.rs`, `CreateProcessInternalW`/`CreateProcessInternalA` in
`src/proc_create.rs`, `CreateWaitableTimerA`/`OpenSemaphoreA` in `src/client/synch.rs`,
`GetFirmwareEnvironmentVariableA` in `src/client/sysinfo.rs`, `VerifyVersionInfoA` in
`src/client/version.rs`, `CopyFileExA`/`CopyFileA` in `src/file/copy.rs`, `CreateDirectoryExA`
in `src/file/dir.rs`, `GetCompressedFileSizeA` in `src/file/fileinfo.rs`,
`MoveFileA`/`MoveFileExA` in `src/file/move.rs`, `GetVolumeNameForVolumeMountPointA` in
`src/file/mntpoint.rs`, `GetVolumePathNameA` in `src/file/volume.rs`. (This is expected: our
translation was done from the pre-split originals, so it already contains the ANSI wrappers that
upstream later split into their own TUs — the exact "subtle case" the task brief describes.)

By contrast, **`GetSystemDEPPolicy`/`GetProcessDEPPolicy`/`SetProcessDEPPolicy` (from the
genuinely-new `client/dep.c`) do NOT exist anywhere in `kernel32-rust`** — confirmed by grep,
zero hits.

**Recommended resolution:**
- Add all 14 `*ansi.c` files to the SOURCE list **commented out**, in local style, e.g.
  `# client/atomansi.c — already covered by Rust (kernel32-rust/src/client/atom.rs; split
  post-fork)`. Interleave them at the position upstream's diff shows (each right after its
  parent file's would-be entry — see the upstream diff hunk for exact ordering:
  `atom.c`→`atomansi.c`, `path.c`→`pathansi.c`, etc.)
- Add **`client/dep.c` ACTIVE (take theirs, uncommented)** — it's real new C functionality with
  no Rust equivalent and no naming collision risk.
- For the `ASM_SOURCE`/`ARCH` block (arm→arm64 rename, `client/arm64/fiber.S`): this region is
  untouched by upstream's diff (upstream still has `elseif(ARCH STREQUAL "arm")` unchanged from
  base elsewhere in the file, not touched by either commit) — it will auto-merge cleanly via
  `git merge-file`; verify post-merge that our `arm64` branch survived (take ours if it somehow
  conflicts).

**Difficulty: NEEDS-CARE**, not ASK-OWNER — every ANSI split was verified against the actual Rust
source, and `client/dep.c` has an unambiguous take-theirs resolution. The only manual work is
correctly interleaving 14 comment lines at the right positions and not missing `client/dep.c`.

---

## `dll/win32/shell32/CMakeLists.txt`

**Upstream** (6 commits, most relevant `095860df576` "A big bundle of NT6+ shell32
implementations and stubs (#8834)" and `c891aefdcff` "Implement CFolder[Item] getters/setters
(#9173)"): adds `add_subdirectory(evalcmd)` (new subdirectory/library, conditionally linked when
`DLL_EXPORT_VERSION >= 0x600`), adds four brand-new files to `SOURCE` — `CShellItemArray.cpp`
(split off `CShellItem.cpp`, PLUS a brand-new `CSimpleShellItemArray` class not present before),
`prop.cpp`, `CUserEventTimer.cpp`/`.h`, `CShellLibrary.cpp`/`.h` — reworks `PCH_SKIP_SOURCE` into
a new `WINE_SOURCE` variable with a per-file `_CRT_NON_CONFORMING_SWPRINTFS` compile definition
(commit `b13e568d991`, removing the global define), adds a GCC13 `-Wno-overloaded-virtual`
workaround, and adds `shcore` to `add_importlibs`.

**We** did a full cohab flip of the `SOURCE` and `PCH_SKIP_SOURCE` lists (`shell32-rust`), and
dropped the `shellrecyclebin` subdirectory entirely (`add_subdirectory(shellrecyclebin)` →
commented out — "the WHOLE object library is Rust now", linked as `${RECYCLEBIN_RUST_LIB}`).

**At-risk comment-outs:** ~55 entries across `SOURCE` (every `.cpp` in the folders/droptargets/
etc. trees) and 13 in `PCH_SKIP_SOURCE` (`appbar.c`, all the `wine/*.c` files, `vista.c`) — none
touched by upstream's diff content except via the `WINE_SOURCE` restructuring (see below).

**Conflicting hunks (5):**

1. `SOURCE` list, `CShellItem.cpp`/`CShellLink.cpp` region: theirs inserts `CShellItemArray.cpp`
   between them. **Verified: base's `CShellItem.cpp` already contained a `CShellItemArray`
   class** (grep confirms it at line 339 of the fork-point file); upstream's commit `c891aefdcff`
   moved that class into its own file — matching our existing comment
   (`# CShellItem.cpp — CShellItem, CShellItemArray, SHCreateShellItem, and
   SHCreateShellItemArrayFromDataObject are Rust-owned.`). **But** the same commit also added a
   brand-new `CSimpleShellItemArray` class inside the new `CShellItemArray.cpp` that did **not**
   exist anywhere before the fork (confirmed: it's declared in a new
   `CSimpleShellItemArray.h`, included from `precomp.h`, but **not constructed anywhere** in the
   shell32 tree per `git grep -n CSimpleShellItemArray upstream/master -- dll/win32/shell32/` —
   only the header/class definition, zero call sites). **Resolution: add
   `# CShellItemArray.cpp` commented out** at the position theirs shows (between `CShellItem.cpp`
   and `CShellLink.cpp`), noting in the comment that `CShellItemArray` itself is already covered
   by `shell32-rust/src/shell_item_array.rs` but `CSimpleShellItemArray` is a new, currently
   unreferenced class with no Rust translation — **safe to leave uncompiled today** (nothing
   calls it, so no undefined-symbol risk), but track it as an owed translation if/when a future
   upstream sync wires a caller to it.
2. Same list region, `CRecyclerDropTarget.cpp`/`shldataobject.cpp`: theirs inserts `prop.cpp`
   here. Confirmed genuinely new (no equivalent existed pre-fork; implements
   `SHELL_GetPropertyKeyFromString` and friends). **Resolution: take theirs, add `prop.cpp`
   ACTIVE** (uncommented) at this exact position — no Rust translation exists or is needed to
   exist yet.
3. Same list region, `systray.cpp`/`CUserNotification.cpp`: theirs inserts `CUserEventTimer.cpp`
   + `.h` here. Confirmed genuinely new. **Resolution: take theirs, add both ACTIVE.**
4. Same list region, `CShellDispatch.cpp`/`CFolder.cpp`: theirs inserts `CShellLibrary.cpp` +
   `.h` here. Confirmed genuinely new (`IShellLibrary` implementation, no pre-fork equivalent).
   **Resolution: take theirs, add both ACTIVE.**
5. `PCH_SKIP_SOURCE`→`WINE_SOURCE` restructuring: **verified every single file upstream puts
   into the new `WINE_SOURCE` variable (`appbar.c` + all 12 `wine/*.c` files) is ALREADY
   commented out / Rust-owned in our tree** (all 13 lines read
   `# wine/xxx.c — replaced by Rust (shell32-rust/src/wine/xxx.rs); cohab via
   libshell32_rust.a`). Since none of those files compile in our tree, upstream's
   `set_source_files_properties(${WINE_SOURCE} PROPERTIES COMPILE_DEFINITIONS
   _CRT_NON_CONFORMING_SWPRINTFS)` would be a no-op applied to zero real sources.
   **Resolution: take OURS** — keep `PCH_SKIP_SOURCE` as our fully-commented list plus
   `rust_varargs_shim.c`, `rust_strsafe_shim.c`, `${CMAKE_CURRENT_BINARY_DIR}/shell32_stubs.c`;
   drop the `WINE_SOURCE` variable and its `set_source_files_properties` call entirely (dead
   weight for us, not a functional loss).
6. `target_link_libraries`/`add_importlibs` tail: base said
   `target_link_libraries(shell32 shellmenu shelldesktop wine uuid recyclebin cpprt atl_classes
   oldnames)`; ours drops `recyclebin` and adds `${SHELL32_RUST_LIB} ${MENUBAND_RUST_LIB}
   ${RECYCLEBIN_RUST_LIB}` + `add_dependencies(shell32 rust_menuband rust_recyclebin)`; theirs
   keeps `recyclebin` unchanged but wraps it with a new GNU `-Wno-overloaded-virtual` block and
   an `if(DLL_EXPORT_VERSION GREATER_EQUAL 0x600) target_link_libraries(shell32 evalcmd) endif()`
   addition. **Resolution: blend** — keep our link line (RUST libs, no `recyclebin`), but add
   upstream's GNU compile-options block and the `evalcmd` conditional. Also blend
   `add_importlibs`: take upstream's addition of `shcore`, keep our `rust_shell32` in
   `add_dependencies`:
   ```
   if(CMAKE_C_COMPILER_ID STREQUAL "GNU")
       # FIXME: This is some ATL related nonsense that triggers GCC 13+ warnings
       target_compile_options(shell32 PRIVATE "$<$<COMPILE_LANGUAGE:CXX>:-Wno-overloaded-virtual>")
   endif()
   target_link_libraries(shell32 shellmenu shelldesktop wine uuid cpprt atl_classes oldnames ${SHELL32_RUST_LIB} ${MENUBAND_RUST_LIB} ${RECYCLEBIN_RUST_LIB})
   if(DLL_EXPORT_VERSION GREATER_EQUAL 0x600)
       target_link_libraries(shell32 evalcmd)
   endif()
   add_dependencies(shell32 rust_menuband rust_recyclebin)
   add_delay_importlibs(shell32 powrprof shdocvw devmgr winspool.drv winmm mpr uxtheme ole32 oleaut32 userenv browseui version fmifs netapi32 secur32)
   add_importlibs(shell32 advapi32 gdi32 user32 comctl32 comdlg32 shlwapi shcore msvcrt kernel32 ntdll)
   add_dependencies(shell32 stdole2 rust_shell32) # shell32_shldisp.tlb needs stdole2.tlb; rust_shell32 = cohab staticlib
   ```

**The top-of-file `add_subdirectory(evalcmd)` line is NOT in a conflict region** — `git
merge-file` auto-merges it cleanly alongside our `shellrecyclebin` comment-out. Verify it
survives (it should, since it's a pure addition upstream made in an area we didn't touch).

**Difficulty: NEEDS-CARE.** The mechanical parts (3 new active files, 1 commented-out new file,
the WINE_SOURCE drop) are fully verified. The one open item — `CSimpleShellItemArray` — is
flagged but NOT blocking: it has zero call sites anywhere in the upstream tree, so leaving
`CShellItemArray.cpp` commented cannot break the build; it's a translation-completeness note,
not a merge risk.

---

## `dll/win32/syssetup/CMakeLists.txt`

**Upstream** (2 commits): `b5db92a5064` "[CMAKE] Improve build generation" (removes the
top-level `include_directories(${REACTOS_SOURCE_DIR}/sdk/lib/tzlib)` block — verified
`sdk/lib/tzlib/CMakeLists.txt` now does `target_include_directories(tzlib INTERFACE
${CMAKE_CURRENT_SOURCE_DIR})`, same pattern as the `sysdm`/`udmihelp` case above, making the
explicit include redundant) and `6117b1bdefe` "[SYSSETUP][BOOTDATA] Add winevdm as an optional
module (#9219)", which adds **`addons.c`** — a brand-new file
(`RunCommandAndWait`/`DoesFileExist`/`InstallAddon`/`InstallOptionalComponents`).

**We** did a full cohab flip: all 8 `.c` files (`classinst.c`, `dllmain.c`, `install.c`,
`logfile.c`, `netinstall.c`, `proppage.c`, `security.c`, `wizard.c`) commented out, replaced by
`syssetup-rust`; kept the top `include_directories(...tzlib)` line untouched.

**At-risk comment-outs:** all 8, none touched by upstream's diff content.

**Conflicting hunks (2):** (1) the `SOURCE` list insertion point for `addons.c` (right before
`classinst.c`); (2) the tail (`add_pch` position, `target_link_libraries` — ours adds
`${SYSSETUP_RUST_LIB}` + `--allow-multiple-definition` + `add_dependencies(syssetup
rust_syssetup)`).

**Recommended resolution:**
- Add `addons.c` to the SOURCE list **ACTIVE (take theirs)** — genuinely new, no Rust
  equivalent (`grep -rn "addons\|Addon" sdk/lib/rust/syssetup-rust/` — zero hits).
- Drop the top `include_directories(${REACTOS_SOURCE_DIR}/sdk/lib/tzlib)` line (take theirs'
  removal) — safe for the same reason as the `sysdm`/`udmihelp` case: `rust_varargs_shim.c`
  doesn't include tzlib headers and `syssetup-rust`'s cargo build doesn't use CMake include
  paths.
- Keep our tail wiring in full (`${SYSSETUP_RUST_LIB}` link, `--allow-multiple-definition`,
  `add_dependencies(syssetup rust_syssetup)`) — upstream's tail changes here are purely
  cosmetic (add_pch reordering) and don't survive as meaningful content once ours' larger
  restructuring is applied.

**Flag, not ASK-OWNER (functional gap, not a CMake ambiguity):** `addons.c`'s
`InstallOptionalComponents` is called from `wizard.c` line 2511
(`git grep -n InstallOptionalComponents upstream/master -- dll/win32/syssetup/`), and `wizard.c`
is one of our 8 cohab-flipped files (`syssetup-rust/src/wizard.rs` — wherever that conflict is
resolved separately, outside this document's CMake scope). Since this call site is new
post-fork functionality, our existing `wizard.rs` does not call `InstallOptionalComponents`.
Compiling `addons.c` active (as recommended above) is safe by itself — it just means the
function is defined but not yet called from the Rust wizard path. Whoever resolves the
`wizard.c` conflict should know `addons.c`/`InstallOptionalComponents` exists and is available to
wire in, matching the winevdm optional-module feature; until then this is a silent feature gap,
not a build break.

**Difficulty: NEEDS-CARE.**

---

## `ntoskrnl/ntos.cmake`

**Upstream** (2 commits): `d46172032cd` "[NTOS] Improve build generation (#9523)" (trims the
top `include_directories()` block — drops `${REACTOS_SOURCE_DIR}` itself, the arbiter/cmlib
paths, adds `sdk/lib/rtl`) and `0e95a343b33` "[NTOS:KE] Implement shared CPU vendor
identification for x86/x64" — adds **`ntoskrnl/ke/x86x64/cpuinfo.c`** (brand new file,
`KiGetCpuVendorString`/`KiIdentifyCpuVendor`/`KiGetCpuSignature`) to BOTH the i386 and amd64
per-arch `SOURCE` lists.

**Verified this is a pure refactor, not new functionality:** diffing `ntoskrnl/ke/i386/cpu.c`
between fork-point and upstream shows the CPU-vendor-detection logic (the `CmpIntelID`/
`CmpAmdID`/etc. signature table, the `CPU_SIGNATURE` union, `KiGetCpuVendor`'s body) was
**deleted from `cpu.c` and moved into the new shared `cpuinfo.c`**, with `cpu.c` presumably now
calling the shared functions (both `ke/amd64/cpu.c` and `ke/i386/cpu.c` are listed as callers per
`git grep KiGetCpuVendorString\|KiIdentifyCpuVendor upstream/master -- ntoskrnl/`). Both
`ke/i386/cpu.c` and `ke/amd64/cpu.c` are already commented out in our tree (cohab-flipped), and
**`sdk/lib/rust/ntoskrnl-rust/src/ke/amd64/cpu.rs` already contains this exact vendor-detection
logic** (`CmpIntelID`, `KiGetCpuVendor()`, the `strcmp` against `"GenuineIntel"` etc. — verified
by reading the file directly). This is the textbook "subtle case": upstream extracted logic we
already translated into a new file our build now references.

**We** did a full cohab flip of `ntoskrnl` (this is the largest and oldest flip in the tree —
essentially the entire `ntos.cmake` SOURCE list is commented out or restructured, including
dropping the NEWCC/non-NEWCC cache branch entirely in favor of a single Rust CC implementation).

**Conflicting hunks (2, both narrow despite the file's size):**

1. i386 arch block: theirs adds `${REACTOS_SOURCE_DIR}/ntoskrnl/ke/x86x64/cpuinfo.c` right
   after `ke/i386/v86vdm.c` and before `mm/i386/page.c`. **Resolution: add it to our comment
   block at that exact position**: `# ke/x86x64/cpuinfo.c — vendor-ID logic already ported as
   part of ke/i386/cpu.rs & ke/amd64/cpu.rs (KiGetCpuVendor); upstream extracted it to a shared
   file post-fork, no new functionality.` `#${REACTOS_SOURCE_DIR}/ntoskrnl/ke/x86x64/cpuinfo.c`
2. amd64 arch block: theirs adds the same file after `ke/amd64/xstate.c`. **Resolution: same —
   add commented, at that position.**

The top `include_directories()` trim is **not in a conflict region** (`git merge-file`
auto-merges it, since neither side touched the same lines as the other in a colliding way) —
verify post-merge that the merged block matches upstream's trimmed version (it should, since we
never modified that block).

**Difficulty: MECHANICAL.** This is the largest file by far but the actual disagreement is two
one-line additions, both fully verified as already-covered-by-Rust. No ambiguity, no functional
gap (unlike the `rosload.cmake`/`ke/config.c` case, nothing new calls `cpuinfo.c`'s functions
that our Rust doesn't already implement under the old, unconsolidated names).
