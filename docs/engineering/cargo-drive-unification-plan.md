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
  → **RESOLVED for KernelDll** (see PoC below). The engine already passes
  `-nostdlib` as a *common* link flag for every ModuleKind (reactos-link
  `link()`), so no per-kind work was needed; the real requirement is that the
  *crate be self-contained* (no referenced UCRT symbol) — then lld emits no UCRT
  import even though the triple's late-link-args still list `-lmsvcrt`.

## PoC results — bootvid KernelDll on the builtin triple (2026-06-06)

Validated the recipe on `bootvid.dll` (KernelDll), the first per-cell cut.

**1. Clean imports — open question #3 settled.** bootvid built on the builtin
precompiled-std triple (stable, NO build-std) imports **only `ntoskrnl.exe` +
`hal.dll`** — zero `api-ms-win-crt-*`, zero `kernel32` — byte-for-byte the same
import set as the json/build-std build. Verified by `llvm-readobj --coff-imports`
on both amd64 and arm64, via a manual `cargo build` *and* end-to-end through cmake.

**2. Mechanism.** lld only creates an IAT entry for a symbol that is actually
*referenced*. A self-contained crate references nothing from `-lmsvcrt`/
`-lkernel32`, so those late-link-args produce no imports despite being on the
link line. The earlier (this-session) failed migration pulled UCRT only because
the *old* bootvid-rust referenced `Rtl*`/string symbols; once bootvid-rust was
made self-contained (local `memcpy/memset/memcmp/_stricmp` + `core::ptr`
intrinsics), the builtin link is clean. **`-nostdlib` + self-containment = clean;
no `-nostdlib` json target needed.**

**3. CRITICAL refinement — use the GNULLVM flavor, not `-gnu`.** The builtin
triple MUST be the `*-pc-windows-gnullvm` flavor (compiler-rt), not
`*-pc-windows-gnu`. The `-gnu` flavor's `late-link-args` reference `libgcc`/
`libgcc_eh` (real mingw), which llvm-mingw does not ship → hard link error
`lld: unable to find library -lgcc`. `-nostdlib`/`-nodefaultlibs` do NOT strip
those (they precede the strip in the arg order). So the matrix's "builtin"
column means: arm64 `aarch64-pc-windows-gnullvm`, amd64 `x86_64-pc-windows-gnullvm`.
amd64's gnullvm std is **not installed by default** → `rustup target add
x86_64-pc-windows-gnullvm` is a CI prereq (arm64's was already present). Note the
kernel-rust *library* build uses `x86_64-pc-windows-gnu` and is fine — because a
staticlib build invokes no linker, so libgcc is never sought. The flavor only
matters at the *drive's* final link.

**4. Mechanism implemented: per-cell `BUILTIN` opt-in.** `add_cargo_module`
(gcc.cmake) gained a `BUILTIN` option: amd64/arm64 link against the gnullvm
triple (stable, no build-std), i386 stays on the softfloat json. This is the
incremental tool for the full unification — flip cells one at a time, boot-
validating each, instead of a global switch. bootvid is opted in
(`drivers/base/bootvid/CMakeLists.txt`); linkers pinned in `.cargo/config.toml`.

**5. Boot validation: ❌ FAILED — clean imports are NOT sufficient.** On a
*clean* baseline (amd64-null cold-green), the builtin-gnullvm bootvid **hangs
deterministically at boot** (amd64, ~6 min, right after "ACPI … detected" — i.e.
when early ntoskrnl first drives inbv/bootvid). bootvid.dll has zero `ud2` and
clean imports, yet it hangs. `-C no-prepopulate-passes` (the guard cnt-rust/
rtl-rust use for the captures(none)/whole-pipeline miscompile) **does NOT fix
it** → the miscompile is in the **precompiled-std interaction**, not bootvid-rust's
own codegen. The json/build-std path boots because the **workspace profile
firewall** (`lto=false`, `codegen-units=256`, panic=abort, build-std rebuilding
core/compiler_builtins under that firewall) suppresses the exact miscompile class
that **precompiled std re-exposes**.

(arm64 could not be used as a second data point: the fresh-crp arm64 baseline
itself hangs at Phase 1 — a separate, in-progress regression another agent is
debugging — so arm64 boot was unattributable.)

**Clean A/B attribution (same HEAD, both cold, amd64):** null/json bootvid →
OK (2m28s); PoC/gnullvm bootvid → HANG (6m). So the builtin bootvid is
*definitively* the cause, not baseline flakiness.

**Root-cause so far (gdb):** the hang is a *tight infinite spin* at a stable PC,
symbolicated to `CmpInitDelayDerefKCBEngine` (`ntoskrnl_rust::config::cmdelay`) —
i.e. **config-manager init, NOT bootvid code**. bootvid has ENTRYPOINT 0 (no
DllMain), so nothing of bootvid's runs at load; the corruption/perturbation is
indirect. Leading hypotheses (unconfirmed): (a) a miscompiled Vid* blit writes
out-of-bounds and corrupts kernel heap/structures that Cmp init later spins on;
(b) the different-sized gnullvm bootvid perturbs the early memory layout and
surfaces a latent layout-sensitive bug in Cmp init. Pinning it further needs
reliable kbase symbolication (the find-kbase heuristic was shaky) + watchpoints,
or a codegen diff of the Vid* functions — expensive, deferred pending a decision
on whether to keep drilling vs accept the builtin-for-drives BLOCKED conclusion.

### ⚠️ Revised conclusion — the builtin-drive migration is BLOCKED for now

The plan's central move ("amd64/arm64 cargo-drives → builtin + precompiled +
`-nostdlib`") is **not a free swap**. Builtin precompiled std gives clean imports
(✅) but **re-introduces a boot-time miscompile** that the build-std workspace
firewall is precisely what suppresses. So the two std providers trade one
miscompile for another:

| | build-std (json) | precompiled (builtin) |
|---|---|---|
| arm64 `compiler_builtins` outline-atomics | ❌ miscompiles (the ntoskrnl bug) | ✅ correct |
| captures(none)/whole-pipeline on a drive | ✅ suppressed by firewall | ❌ **re-exposed → boot hang (bootvid amd64)** |

**Neither is universally safe.** Before the unification can move drives to
builtin, the precompiled-std boot miscompile must be **root-caused** (why does a
self-contained KernelDll hang under precompiled core when it boots under build-std
core + the firewall? candidates: precompiled core/compiler_builtins built with
LTO/few-CGU defeating the cgu=256 firewall across the std boundary; panic=unwind
objects from precompiled std vs the kernel's panic=abort). Until then, **keep
KernelDll drives on the json/build-std path.** The only proven-booting builtin
user today is ntoskrnl-cargo (KernelExe, via boot-swap) — and KernelExe differs
(own panic/eh/mem runtime; not `-shared`), so it does NOT generalize to KernelDll.

**Net for the unification:** open Q#3 (UCRT-strip) is genuinely resolved, but the
bigger prerequisite is now the precompiled-vs-build-std miscompile question, which
this PoC newly surfaced. The `BUILTIN` opt-in mechanism + gnullvm-flavor finding
remain useful groundwork for if/when that's solved.

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
