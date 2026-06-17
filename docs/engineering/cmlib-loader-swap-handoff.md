# Handoff: swap the bootloader's blcmlib (C) → cmlib-rust (`bldr` feature)

**Status: RESOLVED (2026-06-11, commit `da9c72b240d` on continue-rust-port).**
The loader swap is DONE and committed: `uefildr-cargo/build.rs:167` and
`rosload-cargo/build.rs:133` both link `.rust_lib("cmlib-rust", &["bldr"], true)`;
the C `blcmlib` is no longer linked by any loader (its `.c_from_compile_db` is
gone, only `blrtl` remains; the cmake `add_library(blcmlib)` target is vestigial).
The `bldr` cargo feature mirrors `-D_BLDR_` via `#[cfg(not(feature="bldr"))]` gates
across cmcheck/cmheal/hivewrt/hiveinit/hivecell; `boot/freeldr/freeldr/lib/cmlib_loader_stubs.c`
supplies the loader-side externs (CmpSelfHeal/CmpBootType/KeQuerySystemTime/
IoSetThreadHardErrorMode/CmiVolatileHive). `cmlib-rust --features bldr` compiles
0-err on all 3 targets (verified 2026-06-15). Validated implicitly: the swap is on
continue-rust-port and the whole lane fleet boots ×3 through the loader's SYSTEM-hive
read — so **cmlib is now 100% Rust end-to-end, kernel AND loader, all 3 arches.**
The historical brief below is kept for the method/spec.

---

**(historical brief as handed off):** The KERNEL cmlib is fully Rust + boots C-free ×3.
The LOADER (freeldr/uefildr/rosload) still links the reduced **`_BLDR_`** C variant
(`blcmlib`). This is the last piece to make cmlib 100% Rust. Branch
`integ/mlang-cabinet-cohab`, worktree `/home/curi/Projects/os-playground/vibeos-wt-cohab`.

## Goal

Make the bootloader read the SYSTEM hive via **cmlib-rust built with `--features bldr`**
instead of the C `blcmlib`, and keep i386/amd64/arm64 booting to desktop.

Follow AGENTS.md: never revert/delete Rust translations; `./vibeos-cli` is the front door;
smoke is the source of truth; don't weaken markers.

## Why a `bldr` feature is required (not just "link cmlib-rust")

The bootloader's hive is **read-only**. The C `blcmlib` is compiled `-D_BLDR_`, which
strips the write/dirty-flush/self-heal/validation/recovery surface. The FULL cmlib-rust
(what the kernel links) keeps all of that, and when the loader exercised it earlier the
**CmCheckRegistry validation path (HINIT_FLAT + VALIDATE_HIVE) hung the loader at boot**.
So cmlib-rust needs a `bldr` cargo feature that mirrors `-D_BLDR_` exactly.

## The spec: mirror every `#if !defined(_BLDR_)` in C

The C source IS the spec. Enumerate the gates:

```bash
git grep -nE "_BLDR_" sdk/lib/cmlib/*.c sdk/lib/cmlib/*.h
```

They are SURGICAL (strip specific calls/blocks, NOT whole functions). Examples:
- `hivecell.c` HvMarkCellDirty: `_BLDR_` strips only the `CmpLazyFlush()` call — the rest
  of HvMarkCellDirty still runs. (So do NOT no-op the whole function; gate the inner call.)
- `cmcheck.c`: strips `HvMarkCellDirty` calls in the volatile-purge path + (L1651) a block;
  L870 strips one HvMarkCellDirty. The deep-validation that hangs the loader lives here.
- `cmheal.c`: self-heal (CmpSelfHeal/CmpBootType externs + heal logic).
- `hiveinit.c`: log/recovery — note the `#else` branches (≈L1040–1142, L1093) provide
  _BLDR_-SPECIFIC alternate code, not just deletions. Mirror both arms.
- `hivewrt.c`: the write path — many gates (HvWriteHive/HvSyncHive/log writes/CmpLazyFlush).

For each C `#if !defined(_BLDR_) … #endif`, gate the corresponding cmlib-rust code with
`#[cfg(not(feature = "bldr"))]`; for `#if !defined(_BLDR_) … #else … #endif`, use
`#[cfg(not(feature="bldr"))]` / `#[cfg(feature="bldr")]`. The feature already exists in
`sdk/lib/rust/cmlib-rust/Cargo.toml` (`bldr = []`, default off).

Sanity after gating: `cargo build -p cmlib-rust` (default) AND with `--features bldr` must
both compile 0-err for all three targets. The kernel cohab keeps default (no bldr) so the
kernel build is unaffected — verify i386/amd64/arm64 kernel smoke still boots.

## Build wiring (the loader side)

The loaders are built via cargo: `sdk/lib/rust/uefildr-cargo/build.rs` and
`sdk/lib/rust/rosload-cargo/build.rs`. Today each does
`.c_from_compile_db("blcmlib", BLCMLIB_C)` (compiles the 13 C files `-D_BLDR_`). Replace
that with building+linking `cmlib-rust --features bldr` (mirror how the kernel cohab links
`libcmlib_rust.a`, and how these build.rs already build other Rust leaf crates). The
loader-only stubs are pre-written: `boot/freeldr/freeldr/lib/cmlib_loader_stubs.c`
(CmiVolatileHive/CmpSelfHeal/CmpBootType/KeQuerySystemTime/IoSetThreadHardErrorMode) —
wire them in as the loader's substrate for the externs cmlib-rust expects.

ABI note (i386): cmlib CMAPI = **stdcall** on i386 (`_Name@N`); the loader is the same
decoration as the kernel cohab. The loader build does NOT use `-C no-prepopulate-passes`;
i386 gets `-sse,-sse2` via reactos-link's generic i386 rule. blcmlib position is "C-archive
slot, before rust libs."

## Verify

`./vibeos-cli build --arch <a> --clean && ./vibeos-cli smoke --arch <a>` for i386, amd64,
arm64 — all must reach the boot marker (the freeldr chain: MachInit → … → "Switching
stack" → ACPI → kernel → desktop). The loader change is pre-kernel, so a regression shows
as a hang/reset in the **freeldr** stage. Loader DbgPrint goes to **debugcon** and is
UNRELIABLE for instrumentation; prefer QEMU TCG `-d int` (see
docs/engineering/cmlib-apc-bugcheck-handoff.md for the recipe) if you need to catch a
loader fault, and the freeldr serial markers for progress.

## Definition of done

- cmlib-rust compiles 0-err ×3 in BOTH default and `--features bldr`.
- Loaders link cmlib-rust(`bldr`) instead of blcmlib C; `blcmlib` C no longer compiled by
  the loader build (the C library may remain in-tree, just unused by the loaders).
- i386 + amd64 + arm64 smoke all boot (clean-build) — the loader reads the SYSTEM hive via
  Rust cmlib and hands off to the kernel.
- Report the diff + before/after smoke verdicts. If a loader stage hangs and you can't crack
  it, write findings back to this file (don't lose them) and leave it true-red.
