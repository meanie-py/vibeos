# Vendored-code relocation plan

**Status (2026-09-25): destination DECIDED — `vendor/<libname>` (owner). Batches 1–3 moved; CRT trees held for the CRT lane.**

| Batch | Trees | State |
|---|---|---|
| 1 | dxtn, virtio, dosfstools, cmidriver, ac97, netkvm, ms-compiler-tests, stlport (sources), glu (src + include), libsamplerate, schilytools, treelist, tsclient | **moved** |
| 2a | standalone public headers: GL → `vendor/opengl-headers/include/GL`, `d3dkmthk.h` → `vendor/libdxg/include`, stlport's include tree → `vendor/stlport/stlport` (upstream's layout; stlport is now whole) | **moved** — the two public roots are the `VENDOR_INCLUDE_ROOTS` list in the top-level CMakeLists, appended LAST so a vendored header can never shadow an SDK one |
| 2b | zlib, libjpeg, libpng, libtiff, libxslt — each rejoined into one `vendor/<lib>` with its headers at `vendor/<lib>/include` (libxslt at `include/libxslt`, so `<libxslt/x.h>` still resolves; msxml3 gained that root) | **moved** |
| 2c | mbedtls + psa headers: 78 files spell `<mbedtls/…>`/`<psa/…>` through the GLOBAL `sdk/include/reactos/libs` root, so it goes with its sources in its own commit (it is also one of the expensive five) | owed |
| 3 | the rest of `sdk/lib/3rdparty` (adns, freetype, libmpg123, libwin-iconv, libxml2) + libtirpc, mesa, edk2, and the hhpcomp pair chmc + lzx_compress. edk2 was split first: `AppleGraphInfo.h` is ReactOS-authored and went to `boot/freeldr/freeldr/include/arch/uefi/`. chmc's `"../../port/port.h"` became `"port/port.h"` through a `sdk/tools` root | **moved** |
| 4 | mstsc — split first: rdesktop's engine is vendored, the Win32 front end (connectdialog.c, settings.c, .rc) is ReactOS's | owed |
| 5 | the expensive ones, one per commit: wdf, acpica, mbedtls, fex | owed |
| CRT | `sdk/lib/ucrt`, `sdk/lib/crt/math/ieee754`, `libm_sse2` | **held** — owner, 2026-09-26: the CRT lane is working in these trees; do not move them from here |

**How a tree moves** (batch 1 established it): `git mv` to `vendor/<libname>`, then
in the consumer `add_subdirectory(${REACTOS_SOURCE_DIR}/vendor/<libname> <old-name>)`.
The explicit second argument keeps the BINARY dir where it was, so build-output
consumers (`qemu/mkramdisk.sh`, `$BUILD_DIR/dll/3rdparty/...`) do not change — hazard 1
below is neutralised rather than chased. Source-list trees (dosfstools, glu,
schilytools, treelist) get `${REACTOS_SOURCE_DIR}/vendor/<libname>/...` in the list and
an `include_directories` for anything a neighbour includes. Then update vendor.yaml
(`vendor` is now a `containers:` entry, so every child of `vendor/` must be
registered), `--write-stamps`, `--notices`, and build all three arches.

## Goal

Gather every `vendor.yaml` `trees:` entry into one directory, so that "is this ours to
translate?" becomes a fact about the path rather than a registry lookup. Today the 38
vendored trees sit wherever upstream ReactOS put each drop: ACPICA under `drivers/`,
FreeType under `sdk/lib/3rdparty/`, FEX under `external/`, dosfstools inside
`vfatlib`, two loose files in `base/setup/reactos/`. `ours:` code (Wine, the MS
filesystem samples, cryptlib…) does NOT move — it is a translation target and lives
with the rest of the OS.

## The tool

```bash
python3 tools/vendor_move_plan.py                 # blast radius per tree, cheapest first
python3 tools/vendor_move_plan.py --tree drivers/bus/acpi/acpica   # every reference
python3 tools/vendor_move_plan.py --json
```

Columns: `build` (CMake, including relative `add_subdirectory(x)` and `.cmake`
fragments whose paths resolve against the including directory), `rust` (build.rs,
bindgen wrappers, Cargo.toml), `tooling`, `source` (C outside the tree spelling a path
into it), `outbound` (the tree's own files reaching OUT by relative path), `include`
(consumers reaching a header tree under `sdk/include` through an include root —
`#include <GL/gl.h>`), `review` (scripts that build the path from a variable), and
`registry`/`docs`. The counts include comments, so read `--tree` rather than trusting a
number: `ucrt`'s 17 `rust` hits are mostly doc comments in `ucrt-rust`.

## What it says today

- **18 of 38 trees have at most one must-change reference** — typically the single
  `add_subdirectory` in their parent. dxtn, virtio, dosfstools, CMIDriver, netkvm,
  ac97, mstsc, stlport, glu32, libsamplerate, the host tools. These are the first
  movers and could go in one batch.
- **The expensive five:** `sdk/lib/ucrt` (6 build, 17 rust, 9 tooling — the ucrt
  takeover tooling hard-codes it), `sdk/lib/drivers/wdf`, `drivers/bus/acpi/acpica`
  (the `hal/halx86/acpi.cmake` build plus the ntkrnl-sys and acpica-sys bindgen
  scripts), `dll/3rdparty/mbedtls` (two outbound includes and five include-root
  consumers), and `external/fex`. Move these last, one per commit, each with a full
  gate.

## Hazards the survey found

1. **Build-output paths mirror the source layout.** `qemu/mkramdisk.sh` names
   `$BUILD_DIR/dll/3rdparty/libjpeg/libjpeg.dll` and its siblings, and
   `windowscodecs`/`bcrypt`/`schannel` CMakeLists reach into the binary dir the same
   way. Moving a source tree moves its build output, so these break even though they
   never mention the source path. The `review` column is where they show up.
2. **Bindgen scripts assemble paths from variables.** `acpica-sys/generate_bindings.sh`
   uses `${ACPI_DIR}/acpica/include`; `cnt-difftest` and `uefildr-cargo` build.rs name
   `libm_sse2`/`ieee754` relative to a variable. A moved tree must have its islands
   REGENERATED and diffed to zero, never hand-edited, or a stale layout ships silently.
3. **Include roots.** GL, `d3dkmthk.h` and the `sdk/include/reactos/libs/*` header
   halves are reached via global include directories. Moving them needs the new
   location added as a root, and the tool cannot see consumers of a header dir that is
   *itself* the root (`#include <zlib.h>`) — grep those by header name before moving.
4. **Mixed and single-file entries must be split first.** `treelist.c`/`.h` and
   `d3dkmthk.h` are loose files in ReactOS-authored directories. `hdaudbus` is
   unregistered precisely because it mixes ReactOS files (compiled) with CoolStar's BSD
   sources (unbuilt); the CoolStar files belong in the vendored directory, the rest
   does not.
5. **Upstream sync.** Every moved tree diverges from upstream ReactOS's path. Git's
   rename detection follows a moved directory on merge, but a file upstream ADDS to the
   old directory lands at the old path — run the merge with
   `-X find-renames` and check the census. Our version probes read
   `media/doc/3rd Party Files.txt` by path (`Path: sdk/lib/fslib/vfatlib/check`), and
   that file keeps upstream's paths, so those probes stay correct only if the pattern
   keeps quoting the OLD path. That is intended; do not "fix" them to the new path.
6. **A renamed tree loses the prefix its includers spelled — and a bare name can
   COLLIDE.** vfatlib's `dosfsck.h` said `#include "check/common.h"`; after
   `check/` became `vendor/dosfstools`, stripping the prefix would have resolved to
   vfatlib's OWN `common.h` (the includer's directory is searched first). Keep an
   explicit prefix through a root instead: `#include "dosfstools/common.h"` with
   `${REACTOS_SOURCE_DIR}/vendor` on the include path. The move also exposed that
   `dosfsck.h` was ReactOS glue filed inside the vendored dir; it moved back beside
   vfatlib. `vendor_move_plan.py` did not flag this one, because the spelling was
   relative to an include ROOT outside `sdk/include` — only the build caught it.
7. **Generators that SCAN a header directory lose a moved header silently.**
   `tools/rustconv/gen_enum_constants.py` walks `sdk/include/ddk` for the CamelCase enum
   constants fact-sealing protects; moving `d3dkmthk.h` out dropped its D3DKMT_*
   constants from the sealed set, and only the `generated:ddk_enum_constants` drift
   check caught it. The fix is to scan the header where it now lives (it is in that
   script's `HEADER_ROOTS`). Before moving a header tree, grep tools/ for scripts
   that glob its current directory, not just ones that name the file.
8. **A global include root is baked into committed flag templates.** Adding
   `VENDOR_INCLUDE_ROOTS` put two `-I` lines into every
   `sdk/lib/rust/reactos-link/cflags/*.flags` (the cmake-off fallbacks), and the
   queue rejected batches 2a+2b as `LINT_FAIL` on `committed_drift`. After changing
   any global include directory, run `python3 tools/gen_cflags.py` (all arches) and
   confirm `python3 tools/check_committed_drift.py` shows 0 DRIFT before `land`.
9. **Registry follow-through.** After each move: update `path`/`headers`/`license_*`/
   `version.file` in vendor.yaml, `--write-stamps`, `--notices`, and make the new
   directory a `containers:` entry. Once everything has moved, the three containers
   collapse into one and the container ratchet covers every vendored tree structurally,
   which is the real payoff.

## The open decision

The destination directory's name (a top-level `vendor/` or `third_party/`, flat by
project name or mirroring today's subpaths). Mirroring subpaths keeps build-output
paths predictable but keeps the scatter; flat-by-name is what makes the path
self-describing. Needs the owner's word before the first `git mv`.
