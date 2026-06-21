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
| **Host tools** | `spec2def` (exports/import-libs), `widl` (RPC), `windres` (resources) | **PARTIAL** |
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
- **Only remaining `c_from_compile_db` users:** `rosload`/`uefildr` (`blrtl_c` = the DEEP
  loader engines — `bitmap.c`/`compress.c`/`image.c`/`unicode.c`, boot-only, deferred per
  Phase 4). The entire **non-DEEP** WS1 C-floor surface is converted.

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

### WS2 — Generated asm includes (`ks*.inc`)

**Finding:** `ks386.inc`/`ksamd64.inc`/`ksarm64.h` are generated by compiling
`sdk/include/asm/genincdata.c` (which includes the per-arch `ks*.template.h`) to a PE
module, then `native-geninc` (`sdk/tools/geninc/geninc.c`) parses its `.asmdef`
section and emits the `.inc`. **Deterministic / byte-stable per (source, arch)** —
NOT config-dependent. The i386 file is **already committed** at
`sdk/lib/rust/ntoskrnl-cargo/data/ks386-i386.inc` (byte-identical to the cmake
output). All *other* asm includes the kernel uses (`asm.inc`, `kxamd64.inc`,
`syscalls.inc`, `macamd64.inc`, …) are **source-tree** (committed), not generated.

**Plan (two options):**
- **A (fast):** commit the amd64 + arm64 `ks*.inc` alongside the existing i386 one
  (`data/ksamd64-amd64.inc`, `data/ksarm64-arm64.h` for both ntoskrnl + ntkrnlmp),
  and add a reactos-link fallback: use `data/ks*-<arch>` when `build-<arch>/…/ks*.inc`
  is absent. Mirrors the committed-`.rc.res` pattern. Re-commit when templates change
  (rare; CI can diff-check).
- **B (hermetic):** replicate generation in-cargo (~200 LOC Rust: cc-rs compile
  `genincdata.c` → parse `.asmdef` → emit `.inc` to `OUT_DIR`). Removes the
  manual-regen burden but adds a COFF-parse build step.

**Recommendation:** A now (unblocks cmake-off cheaply), B later if regen-drift bites.
**Effort:** A ≈ 1 day; B ≈ 3–4 days.

### WS3 — Host tools (spec2def / dlltool / windres / widl)

**Finding:**
- `spec2def` — single `.c` (`sdk/tools/spec2def/spec2def.c`), standalone; can be
  **self-compiled in a build.rs** (cc-rs → run) trivially.
- `llvm-dlltool` / `llvm-ar` / `clang` / `llvm-nm` / `windres` — all from
  `.build-tools/llvm-mingw`, already standalone (no cmake).
- **`windres` resource pipeline is the narrow blocker:** `ntoskrnl.rc` `#include`s
  `bugcodes.rc`, which (plus `MSG*.bin` sidecars) is generated by cmake's
  `add_message_headers`→`windmc`. ntoskrnl/hal already ship a **committed `.res`**
  (`gen_ntoskrnl_res.py`); finish that pattern so no cell needs live windmc.
- `widl` (RPC IDL→C, used by a few user-mode cells) — cmake host-tool; pre-build once
  and store, or port the handful of needed stubs.

**Plan:** (1) self-compile spec2def in the engine; (2) commit `.res` for every cell
that needs one (extend the gen_*_res.py pattern) OR commit the `bugcodes.rc`+`.bin`
inputs; (3) provision widl as a prebuilt binary (kernel path doesn't need it — defer).
**Effort:** ≈ 2–3 days for the kernel-critical subset (spec2def + ntoskrnl/hal `.res`).

### WS4 — bindgen / generated headers — **DONE**

**Finding:** `ntkrnl-sys` bindgen output is **committed** (`bindgen_{i386,amd64,arm64}.rs`,
`hal_bindgen_*.rs`); there is no `build.rs` regen — cargo never invokes bindgen, so it
needs no cmake-generated headers at build time. `generate_bindings.sh` is a manual
regen tool. Message headers (`ntstatus.h`/`bugcodes.h`) are NOT consumed by cargo
(bindgen committed + constants hardcoded). **No work required.**

### WS5 — Orchestration: replace ninja for the cells

**Finding:** 44 cargo cells are sequenced by `add_cargo_module()` (`sdk/cmake/gcc.cmake`)
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
3. **Phase 2 — asm includes (WS2-A) + host tools (WS3):** commit `ks*.inc` amd64/arm64
   + the `.res` set + self-compile spec2def. After this the *kernel* cargo build needs
   no cmake-generated artifact.
4. **Phase 3 — orchestration (WS5):** the thin driver replacing ninja for the kernel +
   core cells; prove `ntoskrnl.exe` builds with no cmake/ninja.
5. **Phase 4 (optional) — DEEP C (WS1) + WS2-B:** loader C engines + in-cargo ks*.inc,
   for 100% cmake-off including loaders.

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
