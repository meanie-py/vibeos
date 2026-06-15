# ntdll cargo-driven build — C/asm header-dependency audit

> **Ground-truth re-verified 2026-06-14** against `dll/ntdll/CMakeLists.txt` + code — still accurate: ntdll's own C/asm surface is exactly `dispatch/{i386,amd64,arm64}/dispatch.S` (asm, pre-expand/assemble-direct), generated `ntdll_stubs.c` (self-host spec2def), and the empty cohab stub TUs; everything heavy (librtl/libcntpr) is still reused as cmake-built `.a`.

**Goal:** make cargo drive the `ntdll.dll` / `ntdll_vista.dll` build (mirroring
the `ntoskrnl-cargo` PoC), compiling the remaining C + asm from cargo and
removing cmake from the *ntdll link flow*. This doc audits every non-Rust input
for **transitive header dependencies** — the make-or-break factor, because
cc-rs/`build.rs` must compile each file *without* the cmake-orchestrated ReactOS
header tree.

Audited 2026-06-02 on `smlib-csrlib-client-flip` (worktree vibeos-wt-ntdll),
i386 build dir. Header counts are transitive closures pulled from
`build-i386/.ninja_deps` (`ninja -t deps <obj>`).

## Status

**Phase 1, ntdll.dll + ntdll_vista.dll — ALL 3 ARCHES DONE 2026-06-02**
(branch `smlib-csrlib-client-flip`; ntdll.dll/vista i386 committed
`4551c4625fa`/`1bb536bc1a0`, amd64+arm64 + the engine arch-generalization
uncommitted). Both links are cargo-driven via the `reactos-link` engine
(now fully arch-aware) for i386/amd64/arm64; cargo builds ntdll-rust
(build-std) + pre-expanded asm + spec stubs and reuses the cmake-built
sibling `.a`.

| | i386 | amd64 | arm64 |
|---|---|---|---|
| ntdll.dll exports vs cmake | 1402 **exact** | 1384 **exact** | 1370 **exact** |
| ntdll.dll PE | console, base 0x7c920000 | console, 0x7ffb70000000 | console, 0x7ffb70000000 |
| ntdll_vista.dll | spec ✓, GUI, 0x6fc70000 | spec ✓, GUI, 0x7feb20000000 | spec ✓, GUI, 0x7feb20000000 |

Validate (per arch): `bash sdk/lib/rust/ntdll{,_vista}-cargo/build-and-test.sh <arch>`.

Engine arch-generalization (`reactos-link`): `ArchCfg` from
`CARGO_CFG_TARGET_ARCH` drives clang triple, `build-<arch>`, spec2def `-a`
(**`x86_64`** for amd64, not `amd64` — unknown token silently falls back to
i386 @N decoration), dlltool `-m`, clang_rt suffix, `_M_*` defines, and the
target-json output stem; `apply_base` gates the x86-only `-march`/`-m*` flags.
New minimal `x86_64-kernel.json`/`aarch64-kernel.json` (modeled on
i686-kernel.json — strips the standard windows-gnu late-link-args; user-mode
so NO softfloat) + `.cargo/config.toml` linker pins. ntdll_vista's amd64/arm64
entry is undecorated `DllMain` (i386 `_DllMain@12`); ntdll's is `__ImageBase`
(i386 `___ImageBase`).

**Phase 1 is complete.** The three remaining tails landed 2026-06-02:
- **`.res` generator** — `tools/gen_ntdll_res.py` regenerates all six blobs
  (ntdll/ntdll_vista × i386/amd64/arm64) via the canonical REPO llvm-windres
  (`--check` gate). The i386 blobs were re-cut from GNU binutils windres
  (MESSAGETABLE-as-integer, 114798 B) to llvm-windres (MESSAGETABLE-as-string,
  114978 B) so all three arches are consistent + self-contained; functionally
  identical (same version info + message table), and the export-parity test is
  unaffected.
- **`check --strict` bit-rot guard** — `run_cargo_link_guard` in
  `tests/qemu/cli/lints.py` folds the ntoskrnl PoC with ntdll/ntdll_vista: it
  relinks each cargo-driven PoC and asserts a valid PE (no boot). Defaults to
  i386 (the only arch all three share — ntoskrnl-cargo is i386-only); pass
  `--arch amd64/arm64` to guard ntdll/ntdll_vista there. Self-skips on missing
  target json / `build-<arch>` / nightly.
- **boot-swap smoke** — `sdk/lib/rust/ntdll-cargo/boot-swap-smoke.sh` swaps the
  cargo-built ntdll.dll into the real livecd (overwrites the cmake output so
  `ninja livecd` repacks without relinking; `cmp`-asserts the iso carries the
  cargo bytes), boots to `shell ready` (`--allow-stale`, since the swap is a
  deliberate out-of-band media mutation), and always restores the cmake
  ntdll.dll on exit. Proves RUNTIME equivalence, not just PE shape.

## TL;DR

**A cargo-driven ntdll link is *more* tractable than ntoskrnl-cargo was** — the
C→Rust body flip already eliminated essentially all of ntdll's own C. What ntdll
still compiles from C/asm is tiny, and the only real header-dependency risk is
`dispatch.S`'s macro-asm includes (`asm.inc` + `ks386.inc`/`ksamd64.inc`) — an
**already-solved pattern** (ntoskrnl-cargo pre-expands the exact same headers).
Everything heavy (librtl, libcntpr) is **reused as a cmake-built `.a`**, exactly
as the kernel PoC does, so its deep headers don't touch cargo in phase 1.

## The established pattern (ntoskrnl-cargo) — what cargo compiles vs reuses

`sdk/lib/rust/ntoskrnl-cargo/build.rs` drives the link via the shared
`reactos-link` engine (`sdk/lib/rust/reactos-link/src/lib.rs`). It:
- compiles **header-free C shims** with cc-rs (no kernel `-I`; clang's own
  `stdarg/stddef/stdint` suffices);
- assembles **pre-expanded `.s`** (the macro-asm already run through the C
  preprocessor offline — `tools/gen_preprocessed_asm.py` + a checked-in
  `data/ks386-i386.inc`);
- self-hosts **spec2def** (compiles `sdk/tools/spec2def/spec2def.c` as a host
  tool) → `.def`, incl. import libs;
- embeds a **checked-in `.res`** (regenerated by a `gen_*_res.py`);
- **reuses ~20 cmake-built `.a`** for the heavy C bodies (librtl, libcntpr, …).

cmake still builds the sibling `.a`; cargo owns only the final link + the small
header-free pieces. "Remove cmake from the flow" = phase 1 removes it from the
*link*; phase 2 (later) would also cargo-compile the reused libs.

## Input inventory — what ntdll.dll / ntdll_vista.dll actually link

Post-flip, the link line is:
`ntdll_vista_static etwtrace csrlib rtl rtl_um rtl_vista ntdllsys libcntpr
setjmp uuid ${PSEH_LIB} ${NTDLL_RUST_MAIN_LIB} ${NTDLL_CSRLIB_RUST_LIB}`
(+ vista: `smlib ${NTDLL_RUST_LIB} ${NTDLL_SMLIB_RUST_LIB}`).

### Bucket 1 — ntdll's OWN C/asm that cargo must compile (the audit target)

| Input | Kind | Transitive headers | Cargo strategy |
|---|---|--:|---|
| `dispatch/i386/dispatch.S` | asm | `asm.inc` + `ks386.inc` (macro-heavy) | **pre-expand** — reuse `gen_preprocessed_asm.py` + `ks386-i386.inc` (machinery EXISTS) |
| `dispatch/amd64/dispatch.S` | asm | `asm.inc` + `ksamd64.inc` | **pre-expand** (same, ksamd64) |
| `dispatch/arm64/dispatch.S` | asm | **none** (no `#include`) | **assemble directly** — header-free |
| `ntdll_stubs.c` (generated) | C | light (PCH-skipped; spec stub bodies) | **self-host via spec2def** (engine already emits this) |
| `*_rust_stub.c` ×4 (our cohab stubs) | C | header-free (comment-only TUs) | drop or compile directly — trivial |

That is the **entire** own-C/asm surface. The flip did the heavy lifting:
all of ldr/rtl-um/dbg/etw/nt_0600 is Rust (`libntdll_rust.a`), the CSR/SM
clients are Rust (`libcsrlib_rust.a` / `libsmlib_rust.a`).

### Bucket 2 — reuse-as-`.a` (cmake builds; phase 1 leaves these alone)

| Lib | Nature | Transitive headers (sample) | Notes |
|---|---|--:|---|
| `librtl` (rtl/rtl_um/rtl_vista) | C+Rust hybrid | heap.c **121**, actctx.c **128** | DEEP — reuse. Owned by rtl-priv track. |
| `libcntpr` | C (CRT) | deep | reuse. Owned by libcntpr track (~17% Rust). |
| `ntdllsys` | generated **asm** syscall stubs (`add_library(ntdllsys ${ntdllsys_asm})`) | n/a (spec2asm output) | reuse `.a`, or regenerate pre-expanded `.s` |
| `setjmp` | asm | small | reuse or pre-expand |
| `uuid` | pure data (`DEFINE_GUID`) | trivial | reuse or compile directly |
| `${PSEH_LIB}` | DUMMY_PSEH | dead code | reuse (tiny) |

### Bucket 3 — Rust `.a` (cargo already builds these)

`ntdll-rust` (default → ntdll.dll; `--features vista` → ntdll_vista.dll),
`csrlib-rust` (lean `csrlib_only`), `smlib-rust` (lean `smlib_only`).

## Header-dependency verdict

- **Own C/asm header risk is fully contained.** The only files with non-trivial
  header deps are `dispatch.S` (i386/amd64) via `asm.inc` + `ks386/ksamd64.inc`
  — and that is the *same* macro-asm the kernel PoC already pre-expands. arm64
  `dispatch.S` has zero includes and assembles as-is. The C is either generated
  (spec2def, self-hosted) or our own header-free stubs.
- **The deep headers (121–128) live entirely in the reuse-as-`.a` libs** —
  irrelevant to phase 1 (cmake builds them). They define the *phase-2* scope and
  overlap the rtl / libcntpr tracks, so phase 2 should follow those flips, not
  lead them.

## Link facts the engine must reproduce (from cmake)

| | ntdll.dll | ntdll_vista.dll |
|---|---|---|
| module type | `win32dll` | `win32dll` |
| entry | `0` (none — loader calls `LdrpInitialize`) | `DllMain@12` |
| image base i386 / amd64 | `0x7c920000` / `0x7ffb7000000` | `0x72c40000` / `0x7feb2000000` |
| importlibs | (exports importlib for others) | `add_importlibs(ntdll_vista ntdll)` |
| resource | `def/ntdll.rc` | `ntdll_vista.rc` |
| subsystem | win32 (user) | win32 (user) |

## `reactos-link` engine gaps (NOT header-related)

The engine today supports `ModuleKind::KernelExe` + `KernelDll` only — both
freeldr-loaded, `--subsystem,native:5.01`, no `-shared`. ntdll is a **user-mode
win32 DLL**. Needed:
- a `ModuleKind::Win32Dll { entry: Option<String>, image_base }` →
  `-shared`, correct user subsystem, `entry: 0` allowed (DLL w/o DllMain),
  DllMain entry for vista;
- importlib chaining (`add_importlibs(ntdll_vista ntdll)`) — the engine already
  has an importlib path (`spec2def --implib` + `dlltool`), extend to consume a
  sibling module's importlib;
- per-arch image base wiring (table above), already an engine field.

spec2def (incl. forwarders like `RtlConnectToSm=SmConnectToSm`), resource
embedding, and `.a` reuse all work as-is.

## Proposed phasing

**Phase 1 — cargo drives the ntdll *link*** (mirrors ntoskrnl-cargo):
1. Add `Win32Dll` ModuleKind + importlib chaining to `reactos-link`.
2. Pre-expand `dispatch-i386.s` / `dispatch-amd64.s` via `gen_preprocessed_asm.py`
   (+ reuse `ks386`/add `ksamd64` `.inc`); arm64 `dispatch.S` direct.
3. New `ntdll-cargo` manifest crate: `rust_staticlib`s (ntdll-rust default +
   the lean csrlib/smlib), `.asm(dispatch)`, `.spec2def(ntdll.spec)`,
   `.resource(ntdll.rc.res)`, `.static_libs([librtl, libcntpr, ntdllsys,
   setjmp, uuid, pseh])`, `.link(Win32Dll{entry:0, base})`. Second manifest for
   ntdll_vista (`entry: DllMain`, importlib ntdll).
4. Guard it in `vibeos-cli check --strict` like the existing `cargo_link` PoC
   (bit-rot gate) — produces a valid ntdll.dll PE without booting.

**Phase 2 — full cmake removal** (later, follows rtl/cntpr tracks): cargo-compile
the reuse-as-`.a` C bodies. This is where the 121–128-header closures bite;
needs either an accepted SDK include set passed to cc-rs or header-shimming per
file. Out of scope until librtl/libcntpr are further flipped.

## Open questions

1. **Pre-expanded asm staleness.** ntoskrnl-cargo keeps `data/asm/*.s` checked
   in, regenerated out-of-band. Same tradeoff here — acceptable while
   `dispatch.S` is stable (it is; trap trampolines rarely change), but needs a
   regen note. (Mirror `feedback_freeldr_cargo_cache_warm` discipline.)
2. **`ntdllsys` (syscall stubs).** Reuse `.a` (simplest) vs regenerate as
   pre-expanded `.s`. Reuse first.
3. **Cross-arch.** reactos-link asserts i386 today
   (`reactos-link/src/lib.rs:140`). ntdll-cargo phase 1 is i386-only too; amd64
   /arm64 wait on the engine's arch generalization (shared with ntoskrnl-cargo).
4. **Do we keep cmake's ntdll target in parallel** during swap-in (dual build,
   like cohab) so smoke stays green off cmake while the cargo path is validated
   by the `check --strict` gate? Recommended yes.

## Spine binary cmake→cargo LINK playbook (SMSS/CSRSS/winlogon/services/lsass)

The same `reactos-link` engine drives the user-mode spine binaries
(smss/csrss/csrsrv/winsrv/basesrv + lsass/winlogon/userinit/services/lsass). They
are already 100% Rust-bodied but historically cmake-LINKED
(`add_executable`/`add_library` + `target_link_libraries` the Rust `.a` with
`--allow-multiple-definition`). Moving each LINK to cargo follows the recipe below;
these are the reusable, non-obvious mechanics.

### Pick the right ModuleKind

- **`Win32Exe { entry, subsystem }`** — a user-mode win32 executable (lsass,
  userinit, winlogon, services). `--subsystem,windows:5.01` (GUI) or
  `console:5.01` (CUI), `--gc-sections`, NO `-shared`, NO
  `--exclude-all-symbols`, auto image base. User-mode ⇒ `kernel_rust=false`
  (build the *standard* user rust target, not the softfloat kernel json).
- **`NativeExe`** — a native-subsystem exe (smss, csrss): `native:5.01`,
  `_NtProcessStartup@4` / `NtProcessStartup` entry, per-arch explicit image base
  (smss i386 `0x400000`, amd64/arm64 `0x140000000`). Needs
  `.rust_user_target()` — without it the bin shell builds the kernel json; the
  body crate is user-hardfloat (no `-sse`). On arm64 add `.stack_reserve(bytes)`
  (2MB; llvm-dlltool collapses delay-imports so all deps load at once and deep
  `LdrpLoadDll` overflows the 1MB default).
- **`Win32Dll { entry, image_base }` with the subsystem field set to
  `native:5.01` / `console:5.01`** — the spine DLLs (csrsrv native, winsrv/basesrv
  console). `_DllMain@12` (i386) / `DllMain`, no `--gc-sections`.

The float ABI of the empty cargo bin *shell* is irrelevant — the shell may build
the softfloat kernel json while the body `.a` builds the user target; only the
body's float ABI matters.

### EXTRA_DEPENDS must be `.a` FILE PATHS, not cmake target names

In `add_cargo_module(... EXTRA_DEPENDS <...>)`, pass **archive file paths**, not
cmake target names, whenever the producer lives in a directory configured *after*
the consumer. `base/` configures before `dll/` (top `CMakeLists.txt`), so a
`base/`-module can't see `dll/`-importlib *target* names — but a file path edges
to the producer order-independently. (rosload gets away with target names only
because its deps live in `boot/`, configured first.) OBJECT-lib residue/resource
targets that *are* in scope (e.g. `<m>_c`, `<m>_res`) can be passed by target
name.

### `resource_from_rc` = windres-replay; reuse the `<module>_res` OBJECT lib

Retiring `add_executable` drops the `.rc` compile. For binaries with a
load-bearing `.res` (MANIFEST / dialogs / bitmaps / STRINGTABLEs — **check the
`.rc`**; pure version info is omittable), compile it IN-cargo with
`Linker::resource_from_rc(rc_rel, res_target)`: it replays cmake's flag-exact
`windres` command from `compile_commands.json` (positional in/out, NO `-o` —
replace the last token with the OUT_DIR path; `current_dir = build-<arch>`;
carries the full `RC_INVOKED`/`LANGUAGE_*`/per-arch `-D`+`-I` set), then links the
COFF `.res`. Output is byte-identical to cmake's `.res` on every arch.

Keep a resource-only OBJECT lib alive so the windres entry stays in
`compile_commands.json` (and to generate any PCH): `add_library(<m>_res OBJECT
<m>.rc)` + `target_compile_definitions(<m>_res PRIVATE UNICODE _UNICODE)` +
`target_include_directories(<m>_res PRIVATE <srcdir>)`. Its objects are no longer
linked; `lookup_cc` matches the windres entry by the `.rc` abs-path +
`CMakeFiles/<res_target>.dir` marker. (The same OBJECT-lib trick — `<m>_c` via
`c_from_compile_db` — keeps any still-C residue compiling: the NAME arg MUST equal
the OBJECT-lib target since it's the `CMakeFiles/<name>.dir` marker; do NOT
`set_module_type` on an OBJECT lib.)

### Entry-point decoration varies by the UNICODE keyword

The CRT-startup entry symbol you pin depends on whether the binary's
`set_module_type` declares `UNICODE`:

- **win32gui + UNICODE** → wide `wWinMain` → entry `_wWinMainCRTStartup` (i386) /
  undecorated `wWinMainCRTStartup` (amd64/arm64). (lsass, userinit.)
- **win32gui WITHOUT UNICODE** → ANSI `WinMain` (`_WinMain@16`) → entry
  `_WinMainCRTStartup` (i386) / `WinMainCRTStartup`. (winlogon.)

Always read the binary's `set_module_type` for the UNICODE keyword — the wrong
entry surfaces as an undefined `_wWinMain@16` / `_WinMain@16` at link. (The
"hidden transitive dep" the CRT startup pulls in is just `libmsvcrtex.a`, which
provides the `*CRTStartup` that calls the Rust `wWinMain`/`WinMain`.)

### The silent-stale-link trap: arch-gated shim + reused C `.a`

A cohab-flip shim gated `#if defined(__i386__)` combined with a **reused C
archive** makes amd64/arm64 silently link *stale C* — the Rust flip is never
actually exercised on those arches. Concretely (consrv): `consrv_rust_shim.c`
provided `CONTAINING_RECORD` / `InitializeObjectAttributes` /
`_Interlocked{Inc,Dec,Exchange}` only inside `#if defined(__i386__)`, while
`coninput.rs`/`conoutput.rs` referenced them via a non-cfg-gated `extern "system"`
block (undecorated on amd64/arm64). The cmake `.a` (built once, all arches) still
carried 31/32 stale ar members from the pre-flip C, and `ar` never prunes — and
`vibeos-cli build --clean` is **RUST-only**, so it can never re-archive the C. The
amd64/arm64 flip was thus never run.

Fix: force-rebuild the C archive to expose it — `ninja -t clean <lib> && ninja
<lib>` prunes the archive down to the live shim TU(s). Never un-prune (it
re-hides the staleness). Add the missing `#else` (amd64/arm64) branch to the shim
with the same providers (undecorated asm labels, pointer-sized
`CONTAINING_RECORD` offset for Rust `usize`). Once a binary's residue moves to
`c_from_compile_db`, the staleness is moot — cargo recompiles the shim fresh every
link. (Related: `feedback_stale_c_archive_after_merge`.)
