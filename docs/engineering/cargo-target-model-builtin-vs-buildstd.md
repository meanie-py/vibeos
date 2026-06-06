# Cargo target model: precompiled `std` (builtin triple) vs `-Z build-std`

> **See also:** `cargo-drive-unification-plan.md` — the target architecture this
> model points to (decouple `-nostdlib` from build-std; the cohesive one-agent task).


This explains the two ways the Rust-in-the-kernel build provides `core`/`alloc`/
`compiler_builtins`, when to use each, and the current per-(crate-role × arch)
matrix — including the arm64 wart and how to fix it.

## Background: even `#![no_std]` needs `core`

Three crates underpin all Rust code, kernel included:

- **`core`** — the language: `Option`, slices, iterators, `core::fmt`, intrinsics.
- **`alloc`** — `Box`/`Vec`/`String` (the kernel supplies the allocator).
- **`compiler_builtins`** — symbols **LLVM emits calls to but doesn't define**:
  `memcpy`/`memmove`/`memset`, wide integer divide (`__udivdi3`, `__divti3`),
  softfloat routines (`__addsf3`, `__floatdisf`), and on arm64 the **outline
  atomics** (`__aarch64_ldadd4_acq_rel`, …).

rustc-generated machine code *calls into* these. They must be linked from
somewhere. Where they come from is the entire decision below.

## The two providers

### 1. Precompiled std — "builtin triple"

Target a triple rustc ships (`i686-pc-windows-gnu`, `x86_64-pc-windows-gnu`,
`aarch64-pc-windows-gnullvm`). `rustup target add <triple>` installs a
**prebuilt** `core`/`alloc`/`compiler_builtins` for that exact triple; rustc just
links those `.rlib`s.

- Compiled **by the Rust release team, on stable, with official flags, tested in CI**.
- Toolchain: **stable**.
- Fast: no std recompile.
- Codegen of std is locked to Rust's release configuration.

### 2. `-Z build-std` — custom `.json` target

A custom target spec (`*-kernel.json`) is unknown to rustup, so no prebuilt std
exists. You **recompile `core`/`alloc`/`compiler_builtins` from source on every
build**:

```
cargo build --target foo-kernel.json \
  -Z build-std=core,alloc,compiler_builtins \
  -Z build-std-features=compiler-builtins-mem
```

- Requires **nightly** (the flag is unstable) + the **`rust-src`** component.
- Slow: recompiles std each clean build.
- **Full codegen control** — softfloat, `panic=abort`, custom data-layout,
  `frame-pointer=always`, no eh-frame, etc. — because std is rebuilt with *your*
  target spec and flags.

## The tradeoff

| | Precompiled (builtin) | build-std (custom `.json`) |
|---|---|---|
| Who built std | Rust team, CI-tested | **you, every build** |
| Toolchain | **stable** | **nightly** + `rust-src` |
| Speed | fast | slow (std recompile) |
| Codegen control | Rust's release flags | **full** |
| Risk | low (tested binaries) | **higher** — you eat any miscompile your nightly + flags produce in core/compiler_builtins |
| Determinism | high | depends on the nightly pin |

## The rule

**Default to the precompiled builtin std. Use build-std only when a codegen
requirement makes the precompiled std wrong for the environment.**

For this project that requirement is essentially **one thing: i386 softfloat /
no-SSE.** Kernel-mode i386 cannot use SSE (FP state isn't saved at the
kernel-entry boundary), but the precompiled `i686-pc-windows-gnu` std is built
*with* SSE and a hardfloat ABI. The only way to get a no-SSE `core` is to
recompile it with `-C target-feature=-sse,-sse2` against a softfloat `.json` —
i.e. build-std. **There is no precompiled softfloat-i686 std.** This is the
legitimate, unavoidable use.

(The cnt-rust MATH-TIERC track's pending i386 float flip — `rustc-abi:
x86-softfloat` + `-Z build-std=core` so `core::f64::from_bits` returns an x87
`ST(0)` value under the no-SSE ABI — is the *same* i386 softfloat requirement.
It reinforces "i386 = the designated build-std cell" rather than contradicting it.
Keep the nightly / `rust-src` / toolchain-version pin consistent with it.)

## Current matrix (cmake ground truth, all 3 arches)

The split is **by crate role, not by arch**:

| Crate role | i386 | amd64 | arm64 |
|---|---|---|---|
| **Library crates** (rlibs/staticlibs linked into binaries: `kernel_rust`, `rtl_rust`, `cnt_rust_lib`, `cntpr_rust`, `hal_rust`, `winsrv_rust`, `smss_csrss_rust`, `utf8`, …) | builtin `i686-pc-windows-gnu` | builtin `x86_64-pc-windows-gnu` | builtin `aarch64-pc-windows-gnullvm` |
| **Cargo-drive crates** (`*-cargo` final PE links: ntdll, ntdll_vista, kdcom, bootvid, uefildr, hal, lsass, services, smss, csrsrv, winlogon, winsrv, userinit, rosload) | `…-cargo/i686-kernel.json` + build-std | `…-cargo/x86_64-kernel.json` + build-std | `…-cargo/aarch64-kernel.json` + build-std |

So cmake builds *every* arch's libraries on the precompiled builtin std, and
drives *every* arch's final links through a custom kernel.json + build-std.

**Verify the matrix yourself:**
```
grep -hoE "--target [^ ]*\.json|--target (i686|x86_64)-pc-windows-gnu|--target aarch64-pc-windows-gnullvm" build-<arch>/build.ninja | sort | uniq -c
```

### The lone exception: `ntoskrnl-cargo` arm64

`ntoskrnl-cargo` (the standalone cargo-drive PoC, *not* wired into cmake's main
build) now targets `aarch64-pc-windows-gnullvm` (builtin) on arm64 — see the
arm64 logon BUGCHECK fix (commit `cdcfc9fed55`). Reason: the kernel.json +
build-std path is **broken on arm64**, and ntoskrnl's deep kernel code is what
trips it.

## Why arm64 + build-std is wrong (and the actual bug)

arm64 has **no softfloat requirement** (NEON; kernel float via `cnt_float=true`)
and the precompiled `aarch64-pc-windows-gnullvm` std works fine (cmake's 19 arm64
library crates all use it). So on arm64 the cargo-drive cells pay **all of
build-std's cost (nightly, slow, recompiled std) for zero benefit — plus its
risk.**

The risk was real: the ntoskrnl arm64 logon BUGCHECK 0x1E (corrupted
user-transfer PC → `STATUS_ILLEGAL_INSTRUCTION`) was a miscompile in the
**build-std-recompiled `compiler_builtins`**, specifically the arm64 outline-atomic
path (`+outline-atomics` on a QEMU cortex-a72 = ARMv8.0, no LSE → atomics route
through `__aarch64_*` helpers). The precompiled gnullvm std doesn't have the bug
because the Rust team compiled and tested it. Fix = drop build-std, use the
precompiled gnullvm std (+ link C `memcpy`/`memmove`/`memset` shims so the C mem
routines win regardless).

cmake "gets away with" build-std on its *other* arm64 drive cells (ntdll,
ntdll_vista, kdcom, bootvid, uefildr) only because their smaller code surface
doesn't trip the miscompile at boot. The risk there is **latent, not absent.**

## Fixing arm64 (the hardening)

**Goal:** every arm64 cargo-drive cell should use `aarch64-pc-windows-gnullvm` +
precompiled std (no build-std), matching how cmake builds the arm64 *libraries*.

Per cell, mechanically:
1. In the `*-cargo` `build.rs`, switch the arm64 `Linker::new(...)` target from
   `aarch64-kernel.json` to `aarch64-pc-windows-gnullvm`. (The engine drops
   nightly/build-std automatically: `kernel_build_std = kernel && target.is_json()`.)
2. Add C mem shims (`sdk/lib/crt/mem/{memcpy,memmove,memset}.c`) to `c_shims` if
   the cell doesn't already link them — precompiled `clang_rt.builtins` does
   **not** carry `mem*`, and you lose `-Z build-std-features=compiler-builtins-mem`.
3. **Update the cmake invocation too.** These `*-cargo` crates are wired into
   `build-<arch>/build.ninja` with an explicit `--target …aarch64-kernel.json`
   (cmake controls the outer crate's target). Changing only `build.rs` leaves a
   split: outer crate built with kernel.json, inner `.rust_lib` libs with gnullvm.
   The cmake side must pass `--target aarch64-pc-windows-gnullvm` to match.
4. Boot-validate (arm64 ramdisk smoke).

**Coordination:** the arm64 drive cells are owned across several tracks
(ntdll, loaders/freeldr for kdcom/bootvid/uefildr, smss-csrss for winsrv). This is
a cross-cutting change — do the cells you own, hand the rest this recipe; don't
fait-accompli another track's cmake wiring.

## TL;DR

- Precompiled builtin std = stable, fast, tested → **the default**.
- build-std = nightly, slow, you-own-the-codegen → **only when forced** (i386
  softfloat is the one real case).
- arm64 drive cells use build-std for no reason and carry a latent miscompile;
  align them to `aarch64-pc-windows-gnullvm` like the libraries already are.
