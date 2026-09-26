# Vendored-code relocation plan

**Status (2026-09-25): PREP ONLY — nothing has moved.** The destination directory is
the owner's call and has not been made; everything below holds for any destination.

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
6. **Registry follow-through.** After each move: update `path`/`headers`/`license_*`/
   `version.file` in vendor.yaml, `--write-stamps`, `--notices`, and make the new
   directory a `containers:` entry. Once everything has moved, the three containers
   collapse into one and the container ratchet covers every vendored tree structurally,
   which is the real payoff.

## The open decision

The destination directory's name (a top-level `vendor/` or `third_party/`, flat by
project name or mirroring today's subpaths). Mirroring subpaths keeps build-output
paths predictable but keeps the scatter; flat-by-name is what makes the path
self-describing. Needs the owner's word before the first `git mv`.
