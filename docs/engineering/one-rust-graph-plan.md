# One Rust crate graph per binary

**Goal:** every PE links exactly one Rust crate graph, so the Rust runtime symbols exist
once and `tools/link_guard.py` can stop allowing the Rust panic runtime as a duplicate.
For those binaries, that is also the last thing `--allow-multiple-definition` is still
needed for.

**Status (2026-10-04): scoped, not started.** Sequenced after two smaller items on the
roadmap: the ~160 kernel probe sites ("Kernel user-pointer probes" in the bug registry)
and deleting the 37 per-module `--allow-multiple-definition` copies that `gcc.cmake` made
redundant.

---

## Why Rust libraries clash at all

A Rust `staticlib` is a final artifact. It bundles its whole crate graph (`core`,
`alloc`, `compiler_builtins`, every dependency), and a few things in that graph have
fixed, unmangled names because `core`/`alloc` call them by name:

| Symbol | What it is |
|---|---|
| `__rustc::rust_begin_unwind` | the `#[panic_handler]`. A `no_std` graph must define exactly one. Ours all forward to `panic_common`: format the message, DbgPrint, backtrace, then `KeBugCheckEx` / `NtTerminateProcess`. |
| `rust_eh_personality` | the unwinder hook. Empty stubs here: everything is `panic=abort`. |
| `__rust_alloc`, `__rust_dealloc`, `__rust_alloc_error_handler`, … | shims rustc generates so `alloc` reaches the `#[global_allocator]`. |
| `vibeos_unwind_resume_fallback` | panic-common's `_Unwind_Resume` fallback (an lld `-alternatename`, see panic-common). |

A normal cargo build has one graph per binary and rustc drives the link, so each of these
exists once. They only collide when two or more independently built Rust staticlibs go into
one non-Rust link, and that is exactly what the cohab model does. Ordinary Rust functions do
not collide: in the 2026-10-04 census only the names above were duplicated between Rust
libraries, and ntoskrnl carries a single `core::fmt::write` and a single `panic_fmt`.
So this is about correctness and simplicity, not binary size.

Today the duplicates are harmless only by construction: the panic handlers all call the
same `panic_common` body (only the crate name in the message differs), and exactly one crate
defines a global allocator (`netstack-rust`). Both are conventions, not checks. A second
`#[global_allocator]` would make every graph in its binary silently use whichever shim lld
kept. There are **216** `#[panic_handler]`s in the tree.

## Inventory (2026-10-04)

Measured from the link-guard census (`VIBEOS_LINK_GUARD=report
VIBEOS_LINK_GUARD_LOG=<f> ./vibeos-cli build --arch X`; JSON lines, verdict
`ok:Rust panic runtime`) and from the link lines themselves.

**CMake links:** about 154 PEs per arch link Rust; **15 link more than one staticlib**:

| Binary | Rust staticlibs |
|---|---|
| kernel32.dll | kernel32_rust, kernel32_shared_rust, kernel32_vista_rust |
| kernel32_vista.dll | kernel32_shared_rust, kernel32_vista_rust |
| shell32.dll | shell32_rust, menuband_rust, recyclebin_rust |
| advapi32.dll, ksecdd.sys | (own), cryptlib_rust |
| lsasrv.dll / samsrv.dll | lsa_stub / sam_stub, lsasrv_rust / samsrv_rust |
| pcix.sys | pcix_rust, arbiter_rust |
| cdrom, disk, atapi, buslogic, usbaudio, portcls, sysaudio (.sys) | (own), **cntpr_rust** |

**Cargo links (reactos-link):** every one of the **42** `*-cargo` bin crates defines its own
`#[panic_handler]`, and the module body is a *separate* staticlib (`rust_staticlib(...)`). So
every cargo-linked module is at least two graphs. Siblings added with `rust_lib(...)` are
more graphs each: ntoskrnl is about seven (bin, `kernel-rust`, `rtl-rust`, `cnt-rust-lib`,
`cport-rust`, `pseh-rust`, …; `build.rs` has 18 `rust_lib` lines with arch variants),
ntkrnlmp about 6, uefildr/rosload 4–5, and services/winlogon/userinit add `crt0-rust` plus
an RPC stub.

## Why the siblings are separate on purpose

Not all of this is accident. reactos-link builds some siblings separately because one cargo
graph cannot give individual crates their own target or arbitrary `-C` flags:

- **Target.** The CRT-class siblings (`cnt-rust-lib`, `rtl-rust`, `utf8`) must build on the
  user-mode hardfloat target. Building them against the i386 kernel JSON (`x86-softfloat`)
  corrupted cnt-rust-lib's by-value `f64` ABI and crashed i386 boot (2026-06-02).
- **Flags.** `rtl-rust` and `cnt-rust-lib` get `-C no-prepopulate-passes` (plus `-sse,-sse2`
  on i386). Without it LLVM's default pre-pass pipeline miscompiled them: memset DCE'd to a
  spin, output writes elided under `captures(none)`, a kernel hang after ACPI init.
  `[profile.*.package.X]` can set opt-level and codegen-units per crate, not these flags.
- **Feature isolation.** Per-crate target dirs exist so cargo does not unify features
  across consumers (`cnt-rust-lib` `freeldr` / `float`, `rtl-rust` `kernel`).
- **CGU layout.** cnt-ctype is its own crate because adding it as a module reshuffled
  cnt-rust's codegen units and moved `_wcsicmp` into a CGU winsrv pulls for `memcpy`
  ("X was replaced").

## Plan

**Tier 1 — cargo bin + body (42 modules, mostly central).** Make the body crate an rlib
dependency of the `*-cargo` bin crate instead of a nested `rust_staticlib` build, and drop the
bin crate's own panic handler. Both already build for the same target. Check per module:
the body's `rust_build_rustflags`, its features (lockdep/cswitchdep are plumbed through
build.rs today *because* the body is not a cargo dep), and the depfile/rebuild-edge story.
The change lives mostly in reactos-link and `add_cargo_module`, so do it centrally, then
prove it per module with the census.

**Tier 2 — same-flag CMake sets (8 binaries).** kernel32 (3) and kernel32_vista,
shell32 + menuband + recyclebin, advapi32/ksecdd + cryptlib, lsasrv + lsa_stub,
samsrv + sam_stub, pcix + arbiter, plus the services/winlogon RPC stubs and `crt0-rust`.
Give each binary one umbrella staticlib that depends on the rest as rlibs; each library
crate loses its `#[panic_handler]` behind a feature only the umbrella turns on.

**Tier 3 — blocked by target/flags: anything with `cnt-rust-lib` / `rtl-rust` / `utf8`.**
The seven drivers above, plus ntoskrnl, ntkrnlmp, ntdll, win32k and the freeldr loaders.
These can only join one graph once the reasons above go away:
1. Find and fix whatever `-C no-prepopulate-passes` is hiding. If it is the `captures(none)`
   class, it may already be toolchain-fixed on LLVM ≥ 21 (see the roadmap's `lint_kernel`
   note); that decision covers both.
2. The softfloat/hardfloat split is i386-only and goes away with the i386 kernel (roadmap §9).

Until then, **narrow the guard instead of widening it.** Allow a panic-runtime twin only when
one side is a declared CRT-class archive, and hard-fail any twin in a binary that should be
one graph. That turns Tier 1–2 progress into a ratchet.

**Not in scope: compiler_builtins vs `libclang_rt.builtins`.** These are the C toolchain's
and Rust's copies of the same intrinsics (`__udivti3`, `___chkstk_ms`, i386 `__*di3`, arm64
CPU-feature symbols) in modules linking both. One-graph-per-binary does not remove them.
The guard allows them as "toolchain runtime twice", and the CRT's own `mem*` over
compiler_builtins' copy.

## Done means

- The census shows zero `ok:Rust panic runtime` for every Tier 1–2 binary on all three arches.
- `link_guard.py` takes the Tier 3 set as an explicit list and fails a panic-runtime twin
  anywhere else.
- Where a binary's census has no twins of any kind left, it links without
  `--allow-multiple-definition` (a per-target opt-out first, global only when the list is
  empty).

## Risks

- **Feature unification:** one graph unifies features across everything in it. Diff the
  resolved feature set (`cargo tree -e features`) before and after for each binary.
- **CGU repartitioning** moves symbols between archive members and can change which member
  a C link pulls (the cnt-ctype `_wcsicmp` incident). The guard now makes that loud instead
  of silent.
- **Panic message identity:** the crate named in `*** RUST PANIC in <crate>` becomes the
  umbrella's unless the handler passes the real one through. Keep that working;
  `flake-report` and the boot triage key on it.
