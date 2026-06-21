# Build-tool self-hosting — inventory & plan (cmake-off)

The [cargo-drive-cmake-off effort](cargo-drive-cmake-off-roadmap.md) removes cmake
+ ninja as the *orchestrator* of the build. But cmake is also a *producer*: it
builds a set of **host tools** (small C/C++ programs that run on the build
machine) and runs them to generate sources, headers, import libs, resources, and
post-link fixups. To build cargo-off, every one of those tool dependencies must
be either (a) **self-hosted** (compiled + run by `reactos-link`/the orchestrator
with no cmake), (b) covered by a **committed artifact** (the tool's deterministic
output checked in, used only cmake-off, and drift-guarded), or (c) **provided by
the standalone toolchain** (`llvm-mingw`).

This document is the inventory + the plan. It is the companion to the roadmap's
workstreams: WS1–WS3 are largely *about* these tools, and the
[drift-detector](../../tools/check_committed_drift.py) (`vibeos-cli check
--strict`) is what keeps the committed-fallback option honest.

## Status legend

| Mark | Meaning |
|---|---|
| ✅ **self-hosted** | compiled from its single `.c` and run by `reactos-link`/`cargo_orchestrate.py`; no cmake |
| 📦 **committed-fallback** | the tool's deterministic output is checked in + used cmake-off; regen needs the tool; **drift-guarded** |
| 🔧 **toolchain** | shipped standalone in `.build-tools/llvm-mingw`; never needed cmake |
| 🌐 **external** | not in llvm-mingw (host package, e.g. Homebrew mingw-w64) |
| ⛔ **cmake-only** | still requires a configured cmake build; not yet addressed |

## Inventory

| Tool | What it produces | Source | LOC | Single-file? | Status | Consumers |
|---|---|---|---|---|---|---|
| **spec2def** | `.spec` → `.def` (+ import-lib def, + `-stub` bodies) | `sdk/tools/spec2def/spec2def.c` | 1784 | yes | ✅ self-hosted (`reactos-link::generate_def`) | every cell's exports + import libs |
| **native-hpp** | xdk headers (winnt.h, ntddk.h, …) from `.template.h` | `sdk/tools/hpp/hpp.c` | 631 | yes | ✅ self-hosted (`reactos-link::generate_xdk_headers`) | the DEEP-loader C floor (blrtl) |
| **pefixup** | PE post-link fixup (`--kernel`/`--kerneldll`/…) | `sdk/tools/pefixup.c` | 538 | yes | ✅ self-hosted (`cargo_orchestrate.py::build_pefixup`) | every kernel/driver module post-link |
| **dlltool / ar / nm / clang / windres** | import libs, archives, symbols, compile, `.rc`→`.res` | `.build-tools/llvm-mingw` | — | — | 🔧 toolchain | everything |
| **geninc** | `ks*.inc` / `ksarm64.h` asm offset constants | `sdk/tools/geninc/geninc.c` | 228 | yes | 📦 committed (`sdk/include/asm/committed/`) | every kernel `.S` (asm offsets) |
| **windres `.res`** | compiled resource blobs | (llvm-mingw windres) | — | — | 📦 committed (`<crate>/data/*.rc.res`) | ntoskrnl/hal/ntdll/win32k + basesrv/services/userinit/winlogon |
| **widl** | IDL → RPC stub C (`_s.c`/`_c.c`) + GUID `_i.c` | `sdk/tools/widl/` (17 files) | ~22 360 (+ wpp) | **no** | 📦 committed `.rs` (`{svcctl,winreg}-stub/data/*.rs`) | services (svcctl), winlogon (winreg), libuuid GUIDs |
| **windmc** | `.mc` message file → `.rc` + `.bin` | host mingw-w64 (`x86_64-w64-mingw32-windmc`) | — | — | 🌐 external; pre-baked into committed kernel `.res` | ntoskrnl.rc (bugcodes), ntstatus |
| **gendib** | win32k DIB dispatch tables (`dib{8,16,32}gen.c`) | `sdk/tools/gendib/gendib.c` | 1037 | yes | 🔬 tool self-host proven; wiring = win32k chapter | win32k (`libwin32k_residue`) |
| **bin2c / obj2bin / mkshelllink / utf16le** | misc build helpers (single-file each) | `sdk/tools/…` | 180 / 178 / 428 / 315 | yes | 🔧/⛔ as needed | media/build glue |
| **mkisofs / mkhive / cabman / hhpcomp** | ISO image / registry hive / cab / help | `sdk/tools/…` (multi-file) | 2.6K–27.7K | no | ⛔ cmake-only | **final image** packaging, not the cell links |

The recurring pattern: **single-`.c` tools are cheap to self-host** (the spec2def /
native-hpp / pefixup precedent — `cc -O2 -I <its includes> tool.c -o native-tool`,
then run it). The only genuinely hard tool is **widl** (a real compiler — see
below). The media tools (mkisofs/mkhive/cabman/hhpcomp) are multi-file but live on
the *image-packaging* path, not the kernel/spine/driver link path, so they're out
of scope until full-ISO cmake-off.

## The tools, in detail

### Already self-hosted (✅)

`spec2def`, `native-hpp`, `pefixup` are each a single `.c` compiled on the host
toolchain via the spec2def precedent. `reactos-link` builds + runs spec2def
(exports, import libs) and native-hpp (xdk headers) inside `build.rs`;
`cargo_orchestrate.py` builds + runs pefixup as the post-link. No cmake at any
point. **These are the template for self-hosting the rest.**

### Committed-fallback, drift-guarded (📦)

These have a deterministic output that's checked in and used only when no cmake
build is configured. Each is now protected by `tools/check_committed_drift.py`
(regenerate from a configured `build-<arch>/` + byte-compare; **DRIFT = stale
committed copy**), wired into `vibeos-cli check --strict`.

- **`ks*.inc` (geninc)** — generated by compiling `sdk/include/asm/genincdata.c`
  (which `#include`s the per-arch `ks*.template.h`) to a PE, then running
  `native-geninc` to parse its `.asmdef` section into assembler offset constants.
  Committed to `sdk/include/asm/committed/{ks386.inc,ksamd64.inc,ksarm64.h}`;
  `reactos-link::add_kernel_asm_includes` uses the build-dir copy when present,
  else the committed one. **Drift-check:** direct byte-diff vs the build-dir copy.
- **resource `.res` (windres)** — committed per consumer as
  `<crate>/data/<rc-stem>-<arch>.rc.res`. The kernel set (ntoskrnl/hal/ntdll/
  win32k) is generated by `tools/gen_ntoskrnl_res.py`; the four usermode cells
  (basesrv/services/userinit/winlogon) via `tools/gen_committed_res.py` (replays
  cmake's exact windres). **Drift-check:** replay windres to a temp + byte-diff.
- **widl `.rs` RPC stubs** — see widl below; the *transliterated Rust* is the
  committed artifact, not C. **Drift-check:** re-run `widl_to_rust.py` against the
  build-dir `_s.c`/`.obj` + diff.

### windmc — the one external dependency (🌐)

`windmc` (message compiler: `.mc` → `.rc` + binary `.bin`) is **not** in
llvm-mingw; the build finds the host's `x86_64-w64-mingw32-windmc` (Homebrew
mingw-w64). On the kernel path it compiles `bugcodes.mc` → `bugcodes.rc`, which
`ntoskrnl.rc` `#include`s. **Today this is invisible to cmake-off builds** because
the kernel `.res` (with bugcodes baked in) is already committed — only
*regenerating* that `.res` touches windmc. The four usermode `.rc` pull no `.mc`
content at all. So windmc is not a build-time blocker; it's a regen-time
convenience. To remove the dependency entirely (lowest→highest effort): commit
`bugcodes.rc`/`.bin` too (it's stable); or replace windmc with a small generator
(its I/O format is simple); or pre-bake all `.mc` across the SDK.

### widl — why it's special (the one hard tool)

`widl` is **Wine's IDL compiler** — a real cross-boundary contract compiler, not a
flat list processor. It lives in `sdk/tools/widl/` (17 `.c`/`.l`/`.y` files,
~22 360 LOC) plus the `wpp` C preprocessor, and at build time it needs **flex +
bison** to generate its lexer/parser. It parses MIDL `.idl` (direction `[in]`/
`[out]`, extent `[size_is]`, pointer kind `[ref]`/`[unique]`, ownership
`[context_handle]`, wire identity `[uuid]` — things a C signature can't express)
and emits NDR marshalling C: client stubs (`_c.c`), **server stubs (`_s.c`)**, and
GUID definition files (`_i.c`).

Its outputs feed three places here:
- **`svcctl_s.c`** → the Service Control Manager RPC server (consumed by
  `services.exe`),
- **`winreg_s.c`** → the remote-registry RPC server (consumed by `winlogon.exe`),
- **the COM GUID `_i.c`** files → `libuuid` (the GUID constants; e.g. winsrv's
  3 shell GUIDs — see WS5 winsrv notes).

**Current cmake-off approach (committed `.rs`, "option B").** Rather than
self-host widl, we checkpoint its *deterministic final artifact*. The RPC stubs
are already transliterated from widl's generated `_s.c` to Rust by
`tools/widl_to_rust.py` (it regex-parses the NDR tables out of the `_s.c` and uses
`nm` on the compiled `.obj` only to recover the exact i386 stdcall symbol
decorations, emitting against the `midl-rt` crate — no C is linked). That Rust is
the committed artifact: `{svcctl,winreg}-stub/data/<name>.<arch>.rs`, returned by
`reactos-link::generate_rpc_stub_rust` when no cmake build is present, regenerated
when one is. It's drift-guarded.

**Full self-host (option A) — tradeoffs.** Building widl as a host tool would mean
either carrying flex/bison as build prerequisites or committing the
flex/bison-generated lexer/parser, then compiling its 17 files + wpp. Worth it
*only* if the IDLs change off-cmake (rare — they're frozen interfaces) or to
regenerate `libuuid` wholesale cmake-off. Until then the committed `.rs` (tiny,
deterministic, drift-guarded) dominates. **Recommendation: keep option B; revisit
option A only if an IDL changes or full-`libuuid` cmake-off is needed.**

### gendib — tool self-host PROVEN; wiring is the win32k chapter

`gendib` (single `.c`, 1037 LOC) generates win32k's DIB blit dispatch tables
(`dib8gen.c`/`dib16gen.c`/`dib32gen.c`) via `native-gendib <outdir>`. It's the
direct analogue of spec2def/geninc — fully self-contained (only
`stdio`/`stdlib`/`string`). **The tool self-host is proven (2026-06-21):**
`cc -O2 gendib.c -o native-gendib` then `native-gendib <out>` produces the three
`.c` **byte-identical** (0-line diff ×3) to cmake's `build-<arch>/win32ss/gdi/dib/`.

But gendib does **not** stand alone: its 3 outputs are bundled into
`libwin32k_residue.a` (13 members: 5 `*_rust_shims.c` + `floatobj.c` + the 3
gendib `.c` + 4 asm), which is an **atomic** cmake static lib — you can't replace
3 of its 13 members. Freeing it means compiling the *whole* residue in-cargo, and
every residue source `#include`s `win32k.h`, whose closure pulls **7 generated
include dirs** (`build-<arch>/sdk/include/{psdk,ddk,dxsdk,reactos,reactos/mc}` +
`build-<arch>/win32ss` + `build-<arch>/sdk/include`) — notably `bugcodes.h`
(windmc-generated) and a broader psdk/ddk set than blrtl's lone `winnt.h`. So the
residue conversion is **win32k driver-audit work** (the win32k.h header closure
cmake-off, entangled with windmc), not a quick standalone win.

**Plan:** the gendib engine primitive (`compile_host_tool` + run → 3 `.c` in
`OUT_DIR`, mirroring `generate_xdk_headers`) lands *with* the win32k residue
conversion in the driver audit — where it has a caller. Adding it earlier would
be an uncalled fn (dead-code) with a guessed API. The tool itself is ready and
proven; only its consumer is gated.

## Plan / priority

**Tier 0 — done.** spec2def, native-hpp, pefixup self-hosted; dlltool/ar/nm/clang/
windres from the toolchain.

**Tier 1 — done + now protected.** ks*.inc, `.res`, widl `.rs` committed-fallback,
all drift-guarded by `check_committed_drift.py` in `vibeos-cli check --strict`.
This is the safety net that lets the committed copies be trusted.

**Tier 2 — self-host the single-file generators (remove committed fallbacks / unlock cells).** In rough ROI order:
1. **gendib** → tool self-host **proven** (byte-identical ×3); the engine
   primitive + the `libwin32k_residue` conversion land together in the win32k
   driver-audit chapter (gated on the `win32k.h` header closure cmake-off:
   psdk/ddk/dxsdk generated headers + `bugcodes.h`/windmc — see above).
2. **geninc** → WS2 "Option B": compile `genincdata.c` in-cargo, parse `.asmdef`,
   emit `ks*.inc` to `OUT_DIR` (~200 LOC; `tools/gen_ks386_inc.py` is the i386
   reference). Removes the committed `ks*.inc` and makes arm64 self-generating.
3. **windmc** → commit `bugcodes.rc`/`.bin` (or a tiny replacement) to drop the
   one external (Homebrew) dependency.

**Tier 3 — widl (option A), only if needed.** Self-host widl (flex/bison or
committed generated lexer/parser) — required only to change IDLs off-cmake or
regenerate full `libuuid`. Otherwise the committed `.rs` stands.

**Tier 4 — image-packaging tools (full-ISO cmake-off).** mkisofs / mkhive /
cabman / hhpcomp are multi-file and live on the *final image* path, not the
cell-link path. Out of scope until the orchestrator (WS5) graduates from "builds
the modules" to "builds the bootable ISO."

## Cross-references
- Roadmap + workstreams: [cargo-drive-cmake-off-roadmap.md](cargo-drive-cmake-off-roadmap.md)
- The orchestrator: `tools/cargo_orchestrate.py` (WS5)
- The drift-detector: `tools/check_committed_drift.py` (`vibeos-cli check --strict`)
- The generators: `tools/gen_cflags.py`, `tools/gen_committed_res.py`,
  `tools/gen_ntoskrnl_res.py`, `tools/gen_ks386_inc.py`, `tools/widl_to_rust.py`
- Committed fallbacks: `sdk/include/asm/committed/`,
  `sdk/lib/rust/reactos-link/cflags/`, `<crate>/data/*.rc.res`,
  `{svcctl,winreg}-stub/data/*.rs`
