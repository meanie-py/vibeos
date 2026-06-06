# Cargo-drive unification — ideal end-state & the cohesive task

**Status:** design / future work. Prereq reading: `cargo-target-model-builtin-vs-buildstd.md`
(the precompiled-std vs build-std mechanics). This doc is the *target architecture*
for the cargo-driven PE links once all the in-flight cargo-moving settles, written
so **one agent can execute it cohesively** rather than piecemeal.

## Why this exists

The cargo-drive cells (`*-cargo` crates: ntdll, ntdll_vista, kdcom, bootvid,
uefildr, hal, csrsrv, basesrv, lsass, services, smss, winlogon, winsrv, userinit,
…) each carry a custom `<crate>-cargo/<arch>-kernel.json` + `-Z build-std`. An
attempt to move the arm64 cells to the builtin precompiled-std triple (to dodge the
arm64 `compiler_builtins` build-std miscompile that bit ntoskrnl — see
`arm64-cargo-ntoskrnl-logon-bugcheck.md`) revealed *why* a naive swap fails, and in
doing so revealed the clean target architecture.

## The core realization: two concerns are conflated

The custom `*-kernel.json` bundles two **independent** things:

1. **Std provider** — precompiled std (builtin triple, stable) vs `-Z build-std`
   (custom json, nightly + `rust-src`).
2. **Link self-containment** — `-nostdlib`: no UCRT / `kernel32` imports, fully
   self-contained statics.

Bundling them forces a false either/or on amd64/arm64:

| target | self-contained? | codegen |
|---|---|---|
| custom json | ✅ `-nostdlib` | ❌ build-std (arm64 miscompiles `compiler_builtins` outline-atomics) |
| builtin triple | ❌ re-adds UCRT/`kernel32` imports | ✅ precompiled, correct |

### Empirical proof of the failure mechanism (2026-06-06)

Diffing the two uefildr.efi builds (same source, different target):

- **builtin** uefildr.efi imports `api-ms-win-crt-string/stdio/private/convert-l1-1-0.dll`
  (the user-mode UCRT). 348 KB. → boots into an EDK2 `ResetSystemPei` HOB assert +
  watchdog reset, because a UEFI binary has no UCRT to resolve the IAT; the first
  call through it faults.
- **json** uefildr.efi imports **nothing** (`-nostdlib`). 454 KB — *bigger* precisely
  because the string/stdio routines are statically included instead of imported.

Per-cell on the builtin triple:

| cell | builtin imports | verdict |
|---|---|---|
| kdcom | `ntoskrnl.exe` only | ✅ clean (too small to touch fmt/string → nothing pulled) |
| bootvid | ntoskrnl, hal, **+ api-ms-win-crt-string/private, KERNEL32.dll** | ❌ UCRT/kernel32 in a kernel driver |
| uefildr | **api-ms-win-crt-string/stdio/private/convert** | ❌ UCRT in an EFI app |

So: the builtin std lowers Rust `core::fmt`/string/stdio to **dynamic UCRT imports**
(via the target's default `late-link-args`, `-lmsvcrt` → `api-ms-win-crt-*`), which
are **fatal for any freestanding binary** (kernel driver, EFI app, native exe). The
`-nostdlib` json forces self-contained statics — that's its real job, *separate*
from build-std.

### The two concerns ARE separable

`ntoskrnl-cargo` already proves it: `KernelExe` on the **builtin** triple
(precompiled std, correct codegen) **with a `-nostdlib` link** (no UCRT) — boots
green. bootvid/uefildr only broke because the engine's link for *their* kinds
doesn't strip the default CRT. This is also how cmake's *traditional* arm64 link
wins: libraries built on builtin/precompiled (correct codegen, `.a` = no imports),
final link `-nostdlib`.

## Ideal end-state

**1. Decouple in the engine.** `reactos-link`'s `link()` applies `-nostdlib`
(strip the builtin target's default CRT/UCRT linkage) for **every freestanding
ModuleKind** — KernelExe, KernelDll, EfiApplication, NativeExe — as KernelExe does
today. Self-containment becomes a property of the **link kind**, not the target.

**2. Collapsed target/toolchain rule:**

| crate role | i386 | amd64 | arm64 |
|---|---|---|---|
| libraries (rlib/`.a`) | builtin + stable (+`-sse` flag) | builtin + stable | builtin + stable |
| drives (`*-cargo` final link) | softfloat json + build-std + nightly | **builtin + precompiled + stable, `-nostdlib` link** | **builtin + precompiled + stable, `-nostdlib` link** |

Only **i386 keeps build-std**, and only because no-SSE softfloat has no precompiled
std (irreducible). amd64/arm64 get correct codegen *and* no UCRT imports.

**3. Kill the duplication.** Today target choice is encoded in THREE places that
must agree — cmake `add_cargo_module` `_stem`, each `build.rs` `target_json`,
`.cargo/config.toml` `[target.*] linker` — and there are ~16 cells × 3 arches of
near-identical `*-kernel.json` files. Ideal:
- amd64/arm64 use the **builtin triple** → those per-cell jsons vanish.
- i386 uses **one shared softfloat json** (not per-cell) → ~48 jsons → 1.
- Target/toolchain/link-args choice is **single-source in the engine** (keyed by
  arch × ModuleKind). `build.rs` keeps only binary-identity facts (entry,
  image-base, `.res`); cmake passes arch + kind.

**4. Uniform self-contained CRT/mem.** C `mem*` shims (or cnt-rust) everywhere —
precompiled `compiler_builtins` doesn't carry `mem*`, and nothing may rely on UCRT.

## Open questions to settle before executing

- **Can i386 drop build-std too?** Its *libraries* already use `i686-pc-windows-gnu`
  builtin + `-sse` and boot. If the i386 *drive* link can do the same (builtin +
  `-sse` + `-nostdlib`) instead of softfloat build-std, build-std disappears
  entirely and the matrix is fully uniform. Needs verifying the precompiled i686
  core's SSE usage isn't hit at kernel runtime. Entangled with the cnt-rust
  MATH-TIERC i386 softfloat work — coordinate.
- **hal's nightly need.** hal-rust pulls a dep using `#![feature]` → needs nightly,
  but nightly lacks the precompiled gnullvm std. Fix at the toolchain layer:
  `rustup target add aarch64-pc-windows-gnullvm` for nightly (CI prereq) → hal
  becomes builtin + nightly + precompiled + `-nostdlib`, no special-case. Otherwise
  hal is the lone build-std holdout.
- **Exact `-nostdlib` link args per kind.** Confirm a single strip recipe (e.g.
  `-nostdlib`/`-nodefaultlibs` + explicit needed libs) cleanly removes UCRT for
  *all* kinds, not just KernelExe. KernelExe's current recipe is the starting point.

## Why one agent, and when

Inherently cross-cutting: the shared engine (`reactos-link`), the shared cmake
helper (`gcc.cmake`), every cell's `build.rs`, `.cargo/config.toml`, and **per-arch
boot validation for each cell**. Piecemeal fails — you can't validate cell N when
cell N-1 (e.g. uefildr) breaks boot first. Run it as the **final consolidation
pass** *after* the in-flight cargo-moving settles (more cells still being
cargo-ified; the i386 softfloat cnt-rust flip is pending).

**The cohesive task, in one sentence:** decouple `-nostdlib` from build-std in the
engine; move amd64/arm64 cargo-drives to builtin + precompiled with a `-nostdlib`
link; collapse the per-cell jsons to one shared i386-softfloat json; make target
choice single-source; validate boot per-cell per-arch.

## Confidence

- **Proven this session:** the `-nostdlib`-on-builtin pattern works (ntoskrnl-cargo);
  the UCRT-import mechanism is the real failure (binary import-table diff).
- **Validate before committing the agent:** the i386 build-std-elimination stretch,
  and that one `-nostdlib` recipe strips UCRT for every ModuleKind.
