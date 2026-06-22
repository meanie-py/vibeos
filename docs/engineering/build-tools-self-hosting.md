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
| **geninc** | `ks*.inc` / `ksarm64.h` asm offset constants | `sdk/tools/geninc/geninc.c` | 228 | yes | ✅ self-hosted (`reactos-link::generate_ks_inc`) | every kernel `.S` (asm offsets) |
| **windres `.res`** | compiled resource blobs | (llvm-mingw windres) | — | — | 📦 committed (`<crate>/data/*.rc.res`) | ntoskrnl/hal/ntdll/win32k + basesrv/services/userinit/winlogon |
| **widl** | IDL → RPC stub C (`_s.c`/`_c.c`) + GUID `_i.c` | `sdk/tools/widl/` (17 files) | ~22 360 (+ wpp) | **no** | 📦 committed `.rs` (`{svcctl,winreg}-stub/data/*.rs`) | services (svcctl), winlogon (winreg), libuuid GUIDs |
| **windmc** | `bugcodes.h` (the only `.mc`→`.h` in any C-compile closure) | `tools/gen_bugcodes_h.py` (windmc-subset) | 130 | yes | ✅ self-hosted for `bugcodes.h`; message-table `.rc`/`.bin` still pre-baked into committed kernel `.res` | genincdata.c + win32k residue (`<bugcodes.h>`) |
| **gendib** | win32k DIB dispatch tables (`dib{8,16,32}gen.c`) | `sdk/tools/gendib/gendib.c` | 1037 | yes | ✅ self-hosted (`reactos-link::generate_gendib`, via `.gendib()`) | win32k (`libwin32k_residue`) |
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

`spec2def`, `native-hpp`, `pefixup`, `gendib` are each a single `.c` compiled on
the host toolchain via the spec2def precedent. `reactos-link` builds + runs
spec2def (exports, import libs), native-hpp (xdk headers), and gendib (win32k DIB
tables, via `.gendib()`) inside `build.rs`; `cargo_orchestrate.py` builds + runs
pefixup as the post-link. No cmake at any point. **These are the template for
self-hosting the rest.**

### Committed-fallback, drift-guarded (📦)

These have a deterministic output that's checked in and used only when no cmake
build is configured. Each is now protected by `tools/check_committed_drift.py`
(regenerate from a configured `build-<arch>/` + byte-compare; **DRIFT = stale
committed copy**), wired into `vibeos-cli check --strict`.

- **resource `.res` (windres)** — committed per consumer as
  `<crate>/data/<rc-stem>-<arch>.rc.res`. The kernel set (ntoskrnl/hal/ntdll/
  win32k) is generated by `tools/gen_ntoskrnl_res.py`; the four usermode cells
  (basesrv/services/userinit/winlogon) via `tools/gen_committed_res.py` (replays
  cmake's exact windres). **Drift-check:** replay windres to a temp + byte-diff.
- **widl `.rs` RPC stubs** — see widl below; the *transliterated Rust* is the
  committed artifact, not C. **Drift-check:** re-run `widl_to_rust.py` against the
  build-dir `_s.c`/`.obj` + diff.

### windmc — self-hosted for `bugcodes.h` (✅, 2026-06-21)

`windmc` (message compiler: `.mc` → `.h` + `.rc` + binary `.bin`) is **not** in
llvm-mingw — it's a multi-file Wine tool (needs flex/bison), so unlike the
single-`.c` geninc/gendib/spec2def it can't be compiled-and-run by the engine. The
build originally found the host's `x86_64-w64-mingw32-windmc` (Homebrew mingw-w64).

The key scoping insight: across the **entire cmake-off C-compile closure** (kernel
+ spine + drivers + win32k) the *only* windmc output any source `#include`s is
**`bugcodes.h`** — pulled by `genincdata.c` and win32k's residue. (The message
*strings* — `bugcodes.rc`/`ntstatus.rc` MESSAGETABLE `.bin` — live only in the
committed kernel `.res`, never in a C compile; the four usermode `.rc` pull no
`.mc` at all.) So self-hosting just `bugcodes.h` removes windmc from the build
*and* regen path.

`tools/gen_bugcodes_h.py` is a minimal windmc-subset (~130 LOC, single file, no
deps) that parses `sdk/include/reactos/mc/bugcodes.mc` — `MessageIdTypedef`,
`SeverityNames`, `FacilityNames`, the `MessageId/Severity/Facility/SymbolicName`
records, plus windmc's built-in default `Application` facility — and emits
`bugcodes.h` **byte-identical** to windmc's (verified ×3 vs the build-dir copies).
The committed `bugcodes.h` is now regenerated by it (no Homebrew windmc), and the
[drift-detector](../../tools/check_committed_drift.py) double-guards it: (1) the
generator still reproduces the committed copy (runs always), and (2) the committed
copy still matches a configured build's *real* windmc (when a build dir exists).
The message-table `.rc`/`.bin` for the `.res` are still pre-baked (committed `.res`,
windres-drift-guarded); a full `.mc`→`.bin` self-host is only needed for cmake-off
*`.res` regeneration*, not the cell links.

### geninc — self-hosted (2026-06-21)

`geninc` parses the `.asmdef` section of a PE compiled from
`sdk/include/asm/genincdata.c` (which pulls the per-arch `ks*.template.h`) into the
kernel asm offset constants (`ks386.inc`/`ksamd64.inc`/`ksarm64.h`). cmake builds +
runs the `geninc` host tool; **now self-hosted in-engine**:
`reactos-link::generate_ks_inc` compiles `genincdata.c` (committed
`cflags/genincdata.<arch>.flags` + native-hpp xdk headers + the committed
`bugcodes.h`), reads its `.asmdef` directly (the table is pure compile-time
constants — no relocations — so the `.obj` section bytes equal the linked
module's), and emits the ks*.inc to `OUT_DIR`, byte-identical to cmake's
`native-geninc` ×3. `add_kernel_asm_includes` uses the build-dir copy when a cmake
build is configured, else self-hosts — **no committed `ks*.inc`** (the old
`sdk/include/asm/committed/` dir is gone). **Drift-check:** `check_geninc`
reproduces the pipeline and byte-compares vs the build-dir copy; the genincdata
*flags* are covered by `check_cflags`.

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

### gendib — self-hosted (2026-06-21, with the win32k residue)

`gendib` (single `.c`, 1037 LOC) generates win32k's DIB blit dispatch tables
(`dib8gen.c`/`dib16gen.c`/`dib32gen.c`) via `native-gendib <outdir>`. It's the
direct analogue of spec2def/geninc — fully self-contained (only
`stdio`/`stdlib`/`string`). **Now self-hosted:** `reactos-link::generate_gendib`
(`compile_host_tool` + run → the 3 `.c` in `OUT_DIR`), driven by the `.gendib()`
marker on a `c_from_compile_db` lib. Output is **byte-identical** to cmake's.

It landed *with* its only consumer — the **win32k `libwin32k_residue.a`**
conversion (the last cmake `.a` in the tree). That residue (13 members: 5
`*_rust_shims.c` + `floatobj.c` + the 3 gendib `.c` + 4 asm) is now compiled
in-cargo: the C via `c_from_compile_db("win32k_residue")` (committed-flags +
native-hpp xdk + committed `bugcodes.h`), the gendib via `.gendib()`, the i386 asm
via `asm_archive`. The `win32k.h` closure turned out small — only `ntddk/ntifs/
wdm/ntdef` (already from `generate_xdk_headers`) + `bugcodes.h`. Result: cmake-off
`win32k.sys` byte-identical to cmake ×3 (bar the PE timestamp).

## Plan / priority

**Tier 0 — done.** spec2def, native-hpp, pefixup self-hosted; dlltool/ar/nm/clang/
windres from the toolchain.

**Tier 1 — done + now protected.** `.res`, widl `.rs` committed-fallback, the
committed `bugcodes.h`, all drift-guarded by `check_committed_drift.py` in
`vibeos-cli check --strict`. This is the safety net that lets the committed copies
be trusted.

**Tier 2 — self-host the generators (remove committed fallbacks / unlock cells). ALL DONE (2026-06-21).**
1. ✅ **gendib** → `reactos-link::generate_gendib` + the win32k `libwin32k_residue`
   conversion; `win32k.sys` cmake-off byte-identical ×3.
2. ✅ **geninc** → `reactos-link::generate_ks_inc`: compiles `genincdata.c` in-engine,
   reads `.asmdef`, emits `ks*.inc` to `OUT_DIR` byte-identical ×3. The committed
   `sdk/include/asm/committed/` dir is **removed**; arm64 self-generates. Drift:
   `check_geninc` + the `genincdata.<arch>.flags` under `check_cflags`.
3. ✅ **windmc** → `tools/gen_bugcodes_h.py` (windmc-subset) regenerates the committed
   `bugcodes.h` byte-identical with no Homebrew dependency; dual drift-guard
   (generator-vs-committed + committed-vs-real-windmc). The only `.mc`→`.h` in any
   C-compile closure. The message-table `.rc`/`.bin` (committed `.res`) remain for a
   future full-ISO `.res`-regen self-host (Tier 4).

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
  `tools/gen_ntoskrnl_res.py`, `tools/gen_bugcodes_h.py`, `tools/widl_to_rust.py`
- Self-hosted in-engine (no committed output): `reactos-link::generate_ks_inc`
  (geninc), `::generate_gendib`, `::generate_xdk_headers` (native-hpp)
- Committed fallbacks: `sdk/lib/rust/reactos-link/cflags/`,
  `reactos-link/committed-headers/bugcodes.h`, `<crate>/data/*.rc.res`,
  `{svcctl,winreg}-stub/data/*.rs`
