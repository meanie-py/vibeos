# Cargo-drive cmake-off — roadmap to a cmake-free kernel build

**Status:** active north star (2026-06-20). Owner: staticlibs lane.
**Companion doc:** `cargo-drive-unification-plan.md` (the *std-provider / `-nostdlib` self-containment* axis — orthogonal to this doc, which is the *cmake-off* axis).

## The goal

The cargo-driven kernel binary (`ntoskrnl.exe`/`ntkrnlmp.exe`) already links with
**zero cmake-built static libraries** (`static_libs` is an empty `vec![]`). But the
*build* still depends on cmake as the orchestrator and as the producer of several
inputs the cargo cells consume. "Cmake-off" = the kernel + core binaries build from
a top-level cargo (or a thin driver) with **no cmake configure / ninja step** —
every irreducible C file is self-contained and `cc`-compiled directly, every
generated input is either committed or produced by a cargo build step.

This is a quality/architecture goal, not a boot goal — same binaries, simpler and
more hermetic build.

## Current state — what cmake still provides to the cargo kernel build

| Coupling | Mechanism | Blocker? |
|---|---|---|
| **compile_commands.json** | `.c_from_compile_db()` replays cmake's clang for the irreducible C floor (~50 files across 10 cells) | **YES — biggest** |
| **Generated asm includes** | `ks386.inc`/`ksamd64.inc`/`ksarm64.h` under `build-<arch>/sdk/include/asm/` (cmake `native-geninc`) | **YES** |
| **Host tools** | `spec2def` (self-hosted) · `native-hpp` (self-hosted, xdk headers) · `windres` (committed `.res` fallback) · `widl` (committed `.rs`-stub fallback) | **DONE** |
| **Resource `.res`** | `gen_ntoskrnl_res.py` needs cmake-built `bugcodes.rc` + `.bin` (windmc) | **YES (narrow)** |
| **bindgen headers** | `ntkrnl-sys` FFI | **NO — already committed** |
| **ninja orchestration** | `add_cargo_module()` in `sdk/cmake/gcc.cmake` sequences 44 cells | **YES** |

## Workstreams

### WS1 — C floor: `c_from_compile_db` → self-contained `c_shims`

**STATUS (2026-06-21): LEAF + printf + scanf batches DONE — the *kernel* C floor is
cmake-configure-free.** After `46b48272a0f` (LEAF) + `f1e182c0f1d` (printf/scanf):
- **ntoskrnl-cargo + ntkrnlmp-cargo: `cntpr_residue` is now EMPTY** → the cargo kernel
  link calls `c_from_compile_db` ZERO times (i386 integer/ftol/chkstk asm stays as an
  `asm_archive` — asm, not C). This is the WS1 kernel milestone.
- **ntdll-cargo: residue EMPTY** (8 printf shells + `_vscwprintf` + `scanf_trampoline`
  + LEAF all → `c_shims`).
- **win32k-cargo, hal-cargo (i386), kdgdb-cargo, kdcom-serial-cargo, kdvm-cargo**: each
  now links free of `compile_commands.json`.
- Converted source: i386 `ci{cos,sin,sqrt,log,pow}.c` + `fltused.c`, amd64
  `except/amd64/ehandler.c`, the printf engine `_sxprintf.c` + `_vscwprintf.c` (guarded
  headerless, see below), `scanf_trampoline.c` (toolchain headers only). All verified
  symbol+disasm-identical (i386+amd64) and i386/amd64 smoke green.
- **winsrv-cargo**: the usersrv strsafe shim (already headerless) moved to `c_shims`
  (`55842b1d4d9`) → winsrv free of `compile_commands.json` too.
- **DEEP loaders — DONE (2026-06-21).** `rosload`/`uefildr`'s `blrtl_c` (the boot-RTL C
  floor: `bitmap/compress/crc32/image/largeint/random/unicode.c` + `nlsboot.c`, +amd64
  `bitmap64/byteswap/mem.c`) no longer needs `compile_commands.json`. Rather than
  headerless-ify ~5.5K LOC (the original Phase-4 fear), the cmake-off path keeps the
  real `#include <rtl.h>` and supplies the compile env directly: a committed per-arch
  **flag template** (`sdk/lib/rust/reactos-link/cflags/blrtl.<arch>.flags`, = cmake's
  exact blrtl flags minus the inert PCH + build-dir `-I`) + the one generated header the
  closure needs (`psdk/winnt.h`) regenerated in-build via **native-hpp** (`sdk/tools/hpp/
  hpp.c`, self-hosted like spec2def, byte-identical to cmake's). `reactos-link`'s
  `c_db_libs` loop now branches: compile_commands.json wins when configured (replay, can't
  go stale), else `compile_c_db_committed` (the committed flags + native-hpp xdk headers).
  Verified: blrtl objects are **code+data+reloc identical** to cmake's on all 3 arches
  (only `.debug_*` comp_dir differs); cmake-off `uefildr.exe` (i386) has a **byte-identical
  `.text`** to the replay/cmake loader; rosload i386 (NativeExe) + uefildr amd64
  (EfiApplication) also build cmake-off. Regen: `tools/gen_cflags.py`.
- **`c_from_compile_db` is now FULLY cmake-off (2026-06-21).** Precise enumeration of the
  live `.c_from_compile_db(` calls: only `blrtl` (loaders, done above) + three
  `("libcntpr", &cntpr_residue)` in ntoskrnl/ntkrnlmp/ntdll that are **empty-guarded**
  (`cntpr_residue = Vec::new()`, never fire). Every i8042prt/kbdclass/basesrv/csrss/hal/
  win32k/winsrv match is a *comment* (historical). So no other cell uses it.
- **`libcntpr` static_lib → in-cargo c_db (loaders, 2026-06-21).** Generalized the
  mechanism to a 2nd target: `rosload`/`uefildr` dropped the reused cmake
  `sdk/lib/crt/liblibcntpr.a` for `c_from_compile_db("libcntpr", &libcntpr_c)` (the cntpr
  C floor: math/printf/string/scanf + i386 x87 wrappers / amd64 ehandler). Its MSVC asm
  intrinsics (`_alldiv`/`_ftol2`/`_chkstk`/`seh`) are GC'd by the gnu-target loaders, so
  no asm_archive is needed (empirically confirmed: C-only links on all arches). The
  engine `compile_c_db_committed` is target-agnostic; the gen script generalized to
  `tools/gen_cflags.py <target>` (committed `cflags/libcntpr.<arch>.flags`). Verified:
  uefildr i386 cmake-off `.text` byte-identical to the liblibcntpr.a build; rosload
  i386/amd64 + uefildr i386/amd64/arm64 all build (replay + cmake-off). This leaves the
  loaders free of reused cmake C `.a`'s (rosload keeps only `libchkstk.a` asm).

**Shared-file guard pattern (printf):** `_sxprintf.c`/`_vscwprintf.c` are `#include`d by
BOTH the cargo cells (non-secapi shells) and cmake's `crt.dll` (secapi `_s` shells), so
they can't be unconditionally stripped. Solution: hand-define the headerless types
behind `#ifdef CRT_PRINTF_HEADERLESS`, and set `-DCRT_PRINTF_HEADERLESS` ONLY on the
cargo `.c_shims()` compile (via `.c_shim_flags()`). cmake-crt takes the unchanged
`#else` branch → byte-identical → zero blast radius. Same trick works for any shared
floor file whose other consumers must stay on headers.

**Verify harness (`/tmp/ws1_verify.py`, reusable):** for each file, compile the CURRENT
source with its exact `compile_commands.json` command (baseline) and again with
`-DCRT_PRINTF_HEADERLESS` (candidate), then diff `nm` symbols + `objdump -d` disasm. The
only variable is the source edit, so a clean diff PROVES the headerless rewrite is a
semantic drop-in before any build/boot. This is the WS1 acceptance gate.

**Finding (original research):** ~50 distinct C files go through `.c_from_compile_db()`
across 10 cells. They split into:
- **18 LEAF** — zero/trivial includes, or a handful of hand-definable types. Includes
  the kernel-relevant `dbgprint_variadic.c`, `scanf_trampoline.c`, the `ci*.c` i386
  x87 math wrappers, `fltused.c`, `ehandler.c` (amd64 SEH), `mem{cpy,move,set}.c`.
- **13 EXTRACTABLE** — the **8 printf shells** (`sprintf.c`/`_snprintf.c`/`vsprintf.c`/…)
  which all `#include "_sxprintf.c"` (the shared engine pulling `stdio.h`/`tchar.h`);
  convert by extracting `_sxprintf.c` **once** (hand-define the minimal `FILE`/flags it
  needs) and reusing across all 8. Plus `byteswap.c`/`crc32.c`/`random.c`/`nlsboot.c`.
- **4 DEEP** — `bitmap.c`/`compress.c`/`image.c`/`unicode.c` (~500–1000 LOC engines).
  **Boot-loader-only** (rosload/uefildr) → lowest ROI for the *kernel* cmake-off goal;
  can be deferred or kept on `c_from_compile_db` longest.

**Plan:** convert LEAF first (template below), then the printf engine + its 8 wrappers,
then the 4 EXTRACTABLE non-printf, and DEEP last (or never, if we accept the loaders
keep a compile_commands.json dep).

**The template (proven by the spike, commit `1b2e69fb0e1`):**
`dbgprint_variadic.c` was converted from 4 deep cmake includes to a headerless
`c_shim` by hand-defining the types/macros it needed
(`ULONG`/`BOOLEAN`/`PCCH`/`size_t`/`va_list` via `__builtin_va_list`, `NTAPI`=`__stdcall`,
`DPFLTR_ERROR_LEVEL`, `TRUE`/`FALSE`, and the layout-faithful `__wine_debug_channel`
struct + `__wine_debug_class` enum), then switching all 3 consumers
(`ntoskrnl`/`ntkrnlmp`/`ntdll-cargo`) from `.c_from_compile_db("rtl", …)` to
`.c_shims([…])`. **Result: byte-for-byte symbol-identical object, export parity,
i386 ✓boot + amd64 ✓boot+net.** Reference shims: `strsafe_shim.c`, `variadic_shims.c`,
`ntprintf_trampoline.c`, `pseh/dummy.c`.

**Gotcha (from the spike):** a `c_from_compile_db` file may rely on cmake's pch
force-include; once headerless it must move to `.c_shims()` (which does NOT
force-include) — and ALL its consumers must switch together (shared file). Validate
each conversion with an nm symbol-diff vs the cmake baseline + a boot.

**Effort:** LEAF ≈ 0.25–0.5 day each (~5–9 days for 18); printf engine + 8 wrappers
≈ 2–3 days; other EXTRACTABLE ≈ 4 days; DEEP ≈ multi-week (defer). **Kernel-relevant
subset (LEAF + printf) ≈ 8–12 days.**

### WS2 — Generated asm includes (`ks*.inc`) — **SELF-HOSTED ×3 (geninc, 2026-06-21)**

**Finding:** `ks386.inc`/`ksamd64.inc`/`ksarm64.h` are generated by compiling
`sdk/include/asm/genincdata.c` (which includes the per-arch `ks*.template.h`) to a PE
module, then `native-geninc` (`sdk/tools/geninc/geninc.c`) parses its `.asmdef`
section and emits the `.inc`. **Deterministic / byte-stable per (source, arch)** —
NOT config-dependent. All *other* asm includes the kernel uses (`asm.inc`,
`kxamd64.inc`, `syscalls.inc`, `macamd64.inc`, …) are **source-tree** (committed),
not generated.

**DONE (2026-06-21): self-hosted in-engine, all 3 arches (Option B).**
`reactos-link::generate_ks_inc` compiles `genincdata.c` for the target arch
(committed `cflags/genincdata.<arch>.flags` + native-hpp xdk headers + the
self-hosted `bugcodes.h`), reads its `.asmdef` COFF section directly (pure
compile-time constants — no relocations — so the `.obj` section bytes equal the
linked module's), and emits the per-arch ks*.inc to `OUT_DIR`.
`add_kernel_asm_includes` uses the cmake build-dir copy when configured (so it can't
go stale), else self-hosts. Validated **end-to-end**: a cmake-off `ntoskrnl.exe`
built with the self-hosted ks (cmake's moved aside) is **byte-identical** (bar the PE
timestamp) to cmake's on **all three arches**, and the emitted ks*.inc matches
cmake's `native-geninc` byte-for-byte ×3. The committed `sdk/include/asm/committed/`
dir is **removed** (no committed `ks*.inc`); arm64 self-generates. Drift-guarded by
`check_geninc` (re-runs the pipeline vs the build-dir copy) + the `genincdata.<arch>.flags`
under `check_cflags`. (`tools/gen_ks386_inc.py` remains as an i386 alt regenerator.)

**windmc → self-hosted for `bugcodes.h` (the only `.mc`→`.h` in any C-compile
closure).** `tools/gen_bugcodes_h.py` is a 130-LOC windmc-subset that emits
`bugcodes.h` byte-identical to windmc ×3 with no Homebrew dependency; the committed
header is regenerated by it and dual-drift-guarded (generator-vs-committed +
committed-vs-real-windmc). See [build-tools-self-hosting.md](build-tools-self-hosting.md).

### WS3 — Host tools (spec2def / dlltool / windres / widl) — **kernel-critical DONE; usermode .res fallback added (2026-06-21)**

> **Full host-tool inventory + self-hosting plan:** see
> [build-tools-self-hosting.md](build-tools-self-hosting.md) — every tool cmake
> produces with, its cmake-off status (self-hosted / committed-fallback /
> toolchain / external), and the prioritized plan to self-host the rest
> (gendib → geninc → windmc → widl). The committed fallbacks are kept honest by
> `tools/check_committed_drift.py` (`vibeos-cli check --strict`).

**Status:**
- **`spec2def` — DONE (already self-hosted).** `reactos-link::generate_def` runs
  `compile_host_tool("sdk/tools/spec2def/spec2def.c", "spec2def")`, which cc-compiles
  the single `.c` on the HOST toolchain and runs it — no cmake. Same for the import-lib
  path (`spec2def --implib` + `i686-w64-mingw32-dlltool`, both llvm-mingw/host).
- **`llvm-dlltool`/`llvm-ar`/`clang`/`llvm-nm`/`windres`** — all from
  `.build-tools/llvm-mingw`, standalone (no cmake). ✅
- **`windres` resource `.res` — kernel-critical DONE.** ntoskrnl/ntkrnlmp/hal/ntdll/
  win32k all use the committed `.resource("data/<m>-<arch>.rc.res")` path (the data/
  files are checked in for all built arches; `gen_ntoskrnl_res.py`). No live windmc on
  the kernel path. (Only `ntoskrnl.rc` ever pulled windmc-generated `bugcodes.rc`, and
  it's pre-baked into the committed `.res`.)
- **Usermode spine `.res` — committed-fallback ADDED (2026-06-21).** The 4 cells still
  on live windres (`basesrv`/`services`/`userinit`/`winlogon`, via `resource_from_rc`)
  now have a committed-`.res` fallback in `reactos-link::resource_from_rc`, mirroring the
  WS2 ks*.inc pattern: cmake's windres is used when a build dir is configured (so the
  committed copy can't make a live build stale), else the committed
  `<crate>/data/<rc-stem>-<arch>.rc.res` is used. Committed i386+amd64 copies (byte-
  identical to the windres output by construction — generated by replaying the exact
  cmake windres command); regen via **`tools/gen_committed_res.py`**. Committed for
  **all three arches** (i386/amd64 smoke green; arm64 .res generated from a
  COPY+repoint build-arm64 and committed).
- **`widl` (RPC IDL→C) — cmake-off DONE via committed-stub fallback (2026-06-21).** widl
  is a real 15-file compiler (flex/bison parser + NDR codegen + `wpphost`), so rather than
  self-host it (deferred — "option A"), `generate_rpc_stub_rust` gains a committed fallback:
  the RPC stubs are already transliterated to Rust (`widl_to_rust.py` + `midl-rt`, so no C
  links), and that `.rs` is the deterministic final artifact — so commit it per-arch
  (`<stub-crate>/data/<name>.<arch>.rs`) and return it when there is no configured cmake
  build (no `widl`/`build.ninja`/`compile_commands.json`). cmake regenerates it when
  configured (can't go stale); the committed copy is used only cmake-off. 2 stubs
  (`svcctl_s`→services, `winreg_s`→winlogon) × 3 arches. Verified: committed
  `svcctl_s.i386.rs` == fresh replay; **services i386 builds cmake-off** (committed branch);
  the change is inert on the cmake build (replay path untouched). Full widl self-host
  ("option A") remains a future option if the IDLs ever need to change off-cmake.

**Net:** every cargo cell's host-tool + resource dependency on cmake is removed on all
three arches except `widl` (usermode RPC). spec2def/dlltool were already self-hosted.

### WS4 — bindgen / generated headers — **DONE**

**Finding:** `ntkrnl-sys` bindgen output is **committed** (`bindgen_{i386,amd64,arm64}.rs`,
`hal_bindgen_*.rs`); there is no `build.rs` regen — cargo never invokes bindgen, so it
needs no cmake-generated headers at build time. `generate_bindings.sh` is a manual
regen tool. Message headers (`ntstatus.h`/`bugcodes.h`) are NOT consumed by cargo
(bindgen committed + constants hardcoded). **No work required.**

### WS5 — Orchestration: replace ninja for the cells — **MVP DONE (2026-06-21)**

**MVP shipped:** `tools/cargo_orchestrate.py` — the thin cmake/ninja-free driver. It
self-hosts the post-link host tool (`native-pefixup`, single `.c` like spec2def/
native-hpp: `cc -O2 -DTARGET_<host> [-D_TARGET_PE64] -I sdk/include/host
sdk/tools/pefixup.c`), runs the exact `cargo +nightly build-std` invocation
`add_cargo_module()` emits per cell in dependency (topo) order, copies the `.exe`
out, and applies the KIND post-link (`native-pefixup --kernel/--kerneldll/
--kernelmodedriver`, i386 `fix_lld_forwarding.py`, efi strip). **Proof:** with
`build-i386/compile_commands.json` moved aside (no cmake artifact consultable) and
**zero ninja**, it builds `ntoskrnl.exe` (KIND kernelexe) → the result is
**byte-identical to the cmake-built kernel except the 4-byte PE TimeDateStamp**
(2 bytes at file off 0x80, two link runs at different wall-clock — runtime-irrelevant).
ntoskrnl/ntkrnlmp need **no** cargo-cell deps: they synth their hal/kdcom/bootvid
import libs in-cargo from `.spec` via the self-hosted spec2def, and (cntpr_residue
empty) call no `c_from_compile_db`, so the kernel EXE has no cmake prerequisite at
all.

**Kernel + usermode spine proven (2026-06-21).** `--spine` builds **12 modules**
cmake-off — `ntoskrnl` `ntkrnlmp` `ntdll` `ntdll_vista` `csrsrv` `basesrv` `csrss`
`smss` `lsass` `winlogon` `services` `userinit` — in ~35s (warm) with
`compile_commands.json` moved aside + no ninja; **all 12 byte-identical to their
cmake-built twins except the 2-byte PE TimeDateStamp**. Two structural findings
made this fall out cleanly:

- **The inter-cell DAG is fully dissolved.** The roadmap's blocker (2) (import-lib
  ordering) is gone: *every* spine cell synthesizes the import libs it needs from
  `.spec` via `.import_lib()` (self-hosted spec2def+dlltool), so no cell consumes
  another cell's built `.dll`/`.a`. `deps=()` for all — they build in any order /
  in parallel.
- **`resource_from_rc` already has a committed-`.res` fallback** (WS3), so
  basesrv/winlogon/services/userinit need no windres at build time cmake-off.

**`winsrv` is now cmake-off too (2026-06-21)** — the whole spine is done. Its
three former cmake static libs were removed by scoping to what winsrv *actually*
references (an offline link drop-test: drop the libs, read lld's undefined list —
only **11 symbols** out of the libs' ~4200):
- `libuuid.a` (2698 GUIDs) → just **3** shell GUIDs (`CLSID_ShellLink`,
  `IID_IShellLinkW`, `IID_IPersistFile` — winsrv creates .lnk shortcuts). Provided
  by a self-contained `winsrv_guids.c` `c_shim` (verbatim midl values + `selectany`
  → byte-identical to libuuid.a's members; the linker GCs the rest just as before).
- `libwin32ksys.a` (1474 win32k syscall stubs) → **8** `NtUser*` stubs, provided by
  `asm_archive("win32ksys", &["win32ss/sys-stubs.S"])` (the ntdll `ntdllsys`
  pattern; sys-stubs.S + its checked-in `w32ksvc32/64.h` + `syscalls.inc`).
- `libchkstk.a` → **0** symbols — already inert (`__alloca`/`__chkstk` come from
  cnt-rust-lib / clang_rt.builtins); dropped (the userinit `libwine.a` pattern).

Verified i386: cmake-off `winsrv.dll` has a **byte-identical `.text`** (439296 B) to
the cmake build, all 8 stubs defined, all 3 GUID byte-patterns present; the only
delta is a **leaner** `.rdata` (−3376 B — it drops the ~211 dead unreferenced GUIDs
cmake's coarse archive-member link dragged in). Built clean on all 3 arches.

**Drivers — audited + proven cmake-off (2026-06-21).** Scanned all 25 driver cells
(`kerneldll`/`kernelmodedriver`) for cmake-off blockers (cmake `static_libs`,
`c_from_compile_db`, `resource_from_rc`): **24/25 are already clean** —
`rust_staticlib` + self-hosted `.import_lib()` from `.spec`, no cmake `.a` / no
`c_db` / no live windres (hal carries only a committed `.res`). Empirically built
all **19 i386-applicable drivers** (`cargo_orchestrate.py --arch i386 --drivers`,
`compile_commands.json` moved aside, zero ninja) — every one **byte-identical to
its cmake artifact except the 2-byte PE TimeDateStamp** (hal, kdcom, bootvid,
beep, null, nmidebug, pci, pciide, classpnp, condrv, hidparse, mouclass, kbdclass,
i8042prt, swenum, usbd, processr, framebuf, netstack). The arm64-only
`arm64vid`/`arm64input` self-filter to arm64 via `Cell.arches`. (Note: `kdcom.dll`
has 4 alternative provider crates — `kdcom-cargo` is the configured default; the
others are equally clean.)

**`win32k` is now cmake-off too (2026-06-21) — the last cell.** Its only cmake
`.a`, `win32ss/libwin32k_residue.a` (13 members), is compiled in-cargo:
- **5 `*_rust_shims.c` + (i386) `floatobj.c`** → `c_from_compile_db("win32k_residue")`
  (replay when cmake configured; committed-flags + native-hpp xdk + committed
  `bugcodes.h` when cmake-off — the blrtl pattern). The `win32k.h` closure turned
  out tiny: only `ntddk.h`/`ntifs.h`/`wdm.h`/`ntdef.h` (all already produced by
  `generate_xdk_headers`) + `bugcodes.h` (the one windmc header, committed
  arch-identical to `reactos-link/committed-headers/`).
- **3 gendib DIB tables** (`dib{8,16,32}gen.c`, generated) → the new `.gendib()`
  marker on the c_db lib: replay finds them under `build-<arch>/`; cmake-off runs
  the self-hosted **native-gendib** (`sdk/tools/gendib/gendib.c`, the spec2def
  precedent — byte-identical to cmake's).
- **(i386) 4 asm** (3 `dib*_hline/colorfill.s` + `floatobj.S`) → `asm_archive`.

Verified ×3: cmake-off `win32k.sys` is **byte-identical to the cmake build except
the PE TimeDateStamp** (i386/amd64 2 bytes, arm64 3 — all inside the 4-byte
stamp). So the **whole driver set (25/25) is cmake-off**.

The `CELLS` table now covers kernel + spine + **all** drivers; it generalizes to
the full 44-cell set — add a `Cell(...)` with its KIND (`deps` stays empty unless a
future cell breaks the self-hosted-import-lib pattern).

**Per-arch fan-out — DONE (2026-06-21).** `cargo_orchestrate.py` now issues ONE
`cargo build … -p <cell> …` per arch (all cells share the byte-identical
`<stem>.json` target) instead of N sequential invocations: cargo builds the shared
rlibs once and the per-cell links in parallel (concurrent cargo invocations can't
help — cargo locks the target dir). Validated: the 13-cell spine via one fan-out
invocation is byte-identical to the sequential build (heavy parallelism: ~4.5 min
user / 1.5 min wall). `--spine`/`--drivers`/`--loaders`/`--all` all use it.

**Loaders (uefildr/rosload) — cmake-off DONE ×3 (2026-06-21).** Added to `CELLS`
(uefildr efi ×3, rosload native i386+amd64) + a cross-strip
(`.build-tools/llvm-mingw/bin/llvm-strip`, == cmake's CMAKE_STRIP) for the efi
KIND. The committed-branch closure (broader than the kernel's because the loaders'
blrtl/libcntpr pull `sdk/include/wine/winnt.h`) needed two fixes in
`compile_c_db_committed`: (1) `-I <xdk-gen>` (base) for the `<psdk/winnt.h>`
prefixed form; (2) **generated-header dirs placed FIRST** (before the source `-I`),
matching cmake's interleave of the `build-<arch>/` generated `-I` ahead of
`sdk/include/wine` — so the generated `winnt.h`/`ddk/{wdm,ntddk}.h` win over wine's
competing copies (else `ndk/rtltypes.h` redefines `_EXCEPTION_REGISTRATION_RECORD`
vs wine's). The 6 native-hpp headers have no other real source `.h` (only
`.template.h`), so front-placing them matches cmake's resolution everywhere and
leaves blrtl/win32k/ntoskrnl byte-identical. `generate_xdk_headers` now writes each
header via temp+atomic-rename (a partial concurrent write showed up as a spurious
"unterminated conditional directive"). Verified **replay-vs-committed byte-identical
(1-byte PE timestamp)** ×3 for win32k + uefildr; i386 uefildr/rosload + win32k +
ntoskrnl all 2-byte vs a fresh build-dir. The whole OS — kernel + spine + 25
drivers + loaders — is now cmake-off byte-equivalent.

**Remaining:** the optional media tools (full-ISO, Tier 4) + the `--trace`
north-star. No kernel/spine/driver/loader blockers left.

**Original finding:** 44 cargo cells are sequenced by `add_cargo_module()` (`sdk/cmake/gcc.cmake`)
+ ninja. The inter-cell DAG is derived from `.import_lib()`/`.spec2def()`:
`ntoskrnl → hal → {kdcom, bootvid} → drivers`; `ntdll → {csrsrv, winsrv, basesrv} →
{csrss, smss, lsass, winlogon, services, userinit}`; loaders (uefildr/rosload)
independent. **Hardest blockers (ranked):** (1) the single 192-member cargo workspace
serializes per-cell builds (ninja currently parallelizes the custom_commands); (2)
import-lib ordering (a cell's `dlltool` `.a` must exist before a downstream cell links);
(3) compile_commands.json must pre-exist (until WS1 finishes); (4) host-tool discovery
paths; (5) per-arch asm `-I` paths; (6) nightly toolchain; (7) cmake-residue `.a`
staleness (until WS1/static-lib floor fully done).

**Plan:** a thin orchestrator (script or a "driver" cargo xtask) that: validates
prereqs → builds host tools → runs spec2def+dlltool in topo order → dispatches
`cargo build -p <cell>` per cell in dependency order, per arch. Post-WS1/WS2/WS3 this
needs neither cmake nor ninja. **Effort:** ≈ 5–8 days (design + per-arch fan-out +
the import-lib DAG); the long pole, do last.

## Phasing

1. **Phase 0 (done):** spike — prove the C-floor conversion (`dbgprint_variadic.c`).
2. **Phase 1 — C floor LEAF + printf (WS1): DONE for the kernel + spine (2026-06-21).**
   LEAF (`46b48272a0f`) + printf/scanf (`f1e182c0f1d`): ntoskrnl/ntkrnlmp/ntdll
   `cntpr_residue` EMPTY, win32k/hal/kdgdb/kdcom/kdvm freed too. i386+amd64 smoke green;
   arm64 object-identical but gates on a full arm64 build+smoke (no `build-arm64` in the
   staticlibs worktree; lands with the arm64-deterministic-green train). Only the DEEP
   loader engines (`blrtl_c`, boot-only) + `winsrv` strsafe shim remain on
   compile_commands.json.
3. **Phase 2 — asm includes (WS2-A) + host tools (WS3): DONE, all 3 arches
   (2026-06-21).** Committed `ks386.inc`/`ksamd64.inc`/`ksarm64.h` + reactos-link
   fallback (`ccac0ab9733`); spec2def already self-hosted; kernel/hal `.res` already
   committed; usermode-spine `.res` committed-fallback added (i386/amd64/arm64). The
   *kernel* cargo build now needs no cmake-generated artifact on any arch. Remaining:
   `widl` (usermode RPC) + the WS1 DEEP loaders.
4. **Phase 3 — orchestration (WS5): MVP DONE (2026-06-21).** `tools/cargo_orchestrate.py`
   replaces ninja for the cells; **proven** on `ntoskrnl.exe` — builds with
   `compile_commands.json` moved aside + no ninja, byte-identical to cmake's kernel
   bar the PE timestamp. Remaining: usermode-spine import-lib DAG + per-arch fan-out
   (extend the `CELLS` table).
5. **Phase 4 — DEEP loaders (WS1): DONE (2026-06-21).** `rosload`/`uefildr`'s `blrtl_c`
   off `compile_commands.json` via committed flag templates + native-hpp-generated xdk
   headers (`compile_c_db_committed` in `reactos-link`). Loaders now build cmake-off
   (uefildr i386 `.text` byte-identical to the shipping loader). Remaining for 100%
   cmake-off: `widl` (usermode RPC), the same committed-flags generalization for the
   non-loader `c_from_compile_db` cells (driver/spine), WS2-B (in-cargo ks*.inc), and
   WS5 (the ninja-replacement orchestrator).

## Constraints / notes
- Every conversion is **symbol-identical** by construction (same clang, same C); always
  nm-diff the object vs the cmake baseline + boot all 3 arches before landing.
- arm64 changes gate on a **FULL `ninja` + `mkramdisk --refresh`** (field-manual §9.6c),
  and currently also on the `arm64_lpcexitthread_race` fix (arm64 must be deterministic
  desktop-green to land kernel-link changes).
- This is orthogonal to `cargo-drive-unification-plan.md`'s `-nostdlib`/build-std axis;
  the two can proceed independently.

## Provenance
Five parallel read-only research agents (2026-06-20) produced the per-workstream
findings above (C-floor inventory, ks*.inc generation, host-tool inventory, bindgen
status, cell-orchestration DAG). The spike (`1b2e69fb0e1`) validated WS1's template.
