# Stable-everywhere toolchain — amd64 + arm64 off build-std (i386 retires via wow64)

**Lane:** `stable-toolchain` (worktree `vibeos-stable-toolchain`). **Opened:** 2026-07-12.
**Goal:** get amd64 + arm64 fully onto **stable rustc** by dropping `-Z build-std`, and
leave i386 as the *only* nightly cell — to be deleted (not fixed) when the wow64 lane makes
64-bit the sole supported kernel. **No `core` vendoring.**

> **Why this shape (the decision):** the sole genuine nightly requirement is **i386
> kernel-mode softfloat** (`rustc-abi: x86-softfloat`, target-spec-only, no `-C` equivalent,
> upstream-gated). That requirement is *exactly coextensive with shipping a 32-bit kernel* —
> it's a kernel-mode-only constraint (KTRAP_FRAME doesn't save xmm across i386 kernel entry;
> usermode saves full FP state, so i386 *usermode* has no softfloat need). WoW64 is a pure
> usermode layer on a 64-bit kernel — **zero 32-bit kernel-mode code**. So once wow64 lands
> and 64-bit is the only supported kernel, the i386 kernel is deleted, the softfloat
> requirement evaporates, and the remaining i386 *usermode* DLLs flip to the stable builtin
> `i686-pc-windows-gnu` triple (already wired in reactos-link as `rust_user_target`).
> Vendoring a softfloat `core` would be medium standup + **perpetual** maintenance (re-vendor
> + full ud2/captures/sacred-primitives/offset audit each toolchain bump, and you own every
> miscompile in the rebuilt core — the exact class that produced the arm64 outline-atomics
> BUGCHECK) — all for a target we're retiring. So: **don't vendor; let wow64 retire i386.**
> Full reasoning: `../archive/cargo-target-model-builtin-vs-buildstd.md`,
> `../archive/execution-context-ownership-model.md` §"Toolchain reality (v3)".

---

## The remaining nightly surface (inventory + disposition)

| Nightly dep | Disposition in this lane |
|---|---|
| `link_llvm_intrinsics` | ✅ **already killed** (`code_addr!()` stable inline-asm macro, landed w/ cswitchdep). Nothing to do. |
| `linkage` | **Vestigial** — one unused `#![feature(linkage)]` at `sdk/lib/rust/hal-cargo/src/main.rs:12`; **zero** `#[linkage = ...]` sites in the whole workspace. Delete-and-verify (step 0). |
| `c_variadic` | **Not a blocker** (user ruling): headerless `cc`-from-cargo C trampolines to native Rust structs are permitted. The ~12 x86-only variadic driver entry points can be fixed-arity thunks. Out of scope here; also only builds into x86 kernels. |
| `-Z build-std` **amd64 + arm64** | **The work of this lane.** Spike-proven droppable (below). Migrate to builtin triple + stable. |
| `-Z build-std` **i386 kernel** | **Left nightly, marked deprecated-pending-wow64.** Blocked on `rustc-abi: x86-softfloat` (upstream). NOT this lane's job to fix — the wow64 lane deletes the i386 kernel. |

---

## The spike result we're productionizing (2026-07-11, from the cswitchdep branch)

A complete **stable** link of `swenum-cargo` (a real kernel-mode PE driver) was produced on
**both amd64 and arm64** whose **PE import table is byte-identical** to the nightly-JSON
build — same DLLs, same symbols, same NT-native subsystem, same 0x10000 image base +
DriverEntry. Recipe (all mechanical, **no source changes**):

1. `RUSTUP_TOOLCHAIN=stable`; `--target x86_64-pc-windows-gnu` (amd64) /
   `aarch64-pc-windows-gnullvm` (arm64) — builtin, precompiled std.
2. `RUSTFLAGS`: `-C panic=abort -C force-frame-pointers=yes -C linker=<llvm-mingw per-arch
   clang>` **plus amd64-only** `-C link-self-contained=no` (the gnullvm/arm64 target
   *rejects* that flag — must be applied per-arch).
3. **Neutralize the builtin CRT late-link.** The builtin target appends `-lgcc_eh -lgcc
   -lmsvcrt -lmingwex …` (the kernel JSON has none). Point `-L<stubdir>` at a dir of **empty
   `llvm-ar` archives** named `libgcc.a libgcc_eh.a libmsvcrt.a libmingwex.a libmingw32.a
   libpthread.a libuser32.a libkernel32.a` — lld resolves the `-l`s to empty archives and
   pulls nothing. (Without this: `unable to find library -lgcc_eh`; and if found, a spurious
   `msvcrt.dll` import.)
4. **Supply mem intrinsics statically.** memcmp/memcpy/memset came from
   `-Zbuild-std-features=compiler-builtins-mem`; precompiled core lacks them and
   `ntoskrnl.spec` doesn't export them, so they otherwise fall through to `-lmsvcrt`. Prepend
   the cmake-built `libmemcmp.a` + `libcntpr_rust.a` (both define `memcpy/memset/memcmp` as
   `T`) — import table then matches nightly exactly.

**Residual to weigh before migrating the KERNEL itself:** `.text` grows **~2.4×** (swenum
59K→139K) — precompiled unwind-carrying core vs the build-std-slimmed one. Fine for a small
driver; **measure on ntoskrnl** (the big body) before committing the kernel. Drivers/user
DLLs can migrate first regardless.

---

## The one correctness gate — the ABI-return-struct audit (DO FIRST)

The builtin target sets **`abi-return-struct-as-int: true`** (which *matches* clang/mingw C);
every kernel `*.json` leaves it **false**. If any `extern "C"` boundary in the tree returns a
**small struct by value**, today's JSON builds are the divergent side, and this migration
would (correctly) change that ABI at the boundary. **Before flipping any cell:** grep the FFI
surface for by-value small-struct returns (`extern "C"`/`extern "system"` fns returning a
`#[repr(C)]` struct ≤ 16 bytes, both our decls and the C `.h` counterparts they mirror).
Zero hits ⇒ safe. Any hits ⇒ enumerate + confirm each is not on a live boundary, or fix.
This is read-only investigation and blocks nothing else — do it up front.

---

## Plan of record

**Step 0 — kill vestigial `linkage`.** Delete `#![feature(linkage)]` from
`hal-cargo/src/main.rs:12` (+ the now-unneeded `#![allow(internal_features)]` if it guarded
only that). Build hal-cargo ×3 arches; confirm it still links `_HalInitSystem@8`. Cheap win,
removes a nightly *language* feature from the surface.

**Step 1 — the ABI-return-struct audit** (above). Gate for everything below.

**Step 2 — a stable link mode in `reactos-link`.** Add a `ModuleKind`/`Linker` mode (or
extend the existing one) that: selects the **builtin triple** per-arch, injects the
**stub-`-L`** dir of empty archives, **prepends** `libmemcmp.a`+`libcntpr_rust.a`, and applies
the per-arch RUSTFLAGS (amd64-only `link-self-contained=no`). The engine already accepts
builtin triples (`rust_user_target` path, `Linker::new`) and gates build-std on
`kernel_build_std = kernel && target.is_json()` — so a builtin (non-JSON) target auto-drops
build-std. Wire the empty-archive stubdir generation into the build (a tiny `llvm-ar` step).

**Step 3 — flip the outer drive per opted-in cell.** `add_cargo_module` in
`sdk/cmake/gcc.cmake` pins `RUSTUP_TOOLCHAIN=nightly` + `--target *.json` + `-Z build-std`.
For amd64/arm64 opted-in cells, flip to `stable` + builtin triple. **Order:** a driver first
(swenum — the proven one), then user DLLs, then measure `.text` on **ntoskrnl** before doing
the kernel. Each flip: boot-validate the arch, then ×3-gate before landing.

**Step 4 — DON'T touch i386.** Leave i386 kernel on `i686-kernel.json` + nightly. Add a
one-line comment at its `add_cargo_module` site: *"i386 kernel stays nightly (x86-softfloat,
upstream-gated); retired by the wow64 lane, not fixed here."* When wow64 makes 64-bit the sole
kernel, a *separate* change deletes the i386 kernel and flips i386 usermode cells to
`rust_user_target` (`i686-pc-windows-gnu`, stable, SSE-ok).

## Definition of done (this lane)

- amd64 + arm64: **no `*-cargo` cell uses `-Z build-std`**; all build on `RUSTUP_TOOLCHAIN=stable`.
- `linkage` feature gate gone; workspace has zero nightly *language* features on amd64/arm64.
- ABI-return-struct audit written up (clean, or gaps enumerated+resolved).
- ×3 gate green; ntoskrnl `.text` delta measured + recorded (accept/defer the kernel itself).
- i386 kernel untouched + clearly labeled as wow64-retirement scope.

## Verify the current matrix
```bash
grep -hoE "--target [^ ]*\.json|--target (i686|x86_64)-pc-windows-gnu|--target aarch64-pc-windows-gnullvm" \
  build-<arch>/build.ninja | sort | uniq -c
```

## Pointers
- reactos-link engine: `sdk/lib/rust/reactos-link/src/lib.rs` (`rust_user_target` fields
  L267/281/292, `is_json`/`kernel_build_std` gate, `Linker::new`, `ModuleKind`).
- Outer drive: `sdk/cmake/gcc.cmake` (`add_cargo_module`).
- Target specs: `sdk/lib/rust/*-cargo/{i686,x86_64,aarch64}-kernel.json`.
- Full analysis: `../archive/cargo-target-model-builtin-vs-buildstd.md`,
  `../archive/execution-context-ownership-model.md` §"Toolchain reality (v3)".
- Roadmap ledger entry (retire on land): `docs/product/vibeos-roadmap.md` §Owed-work
  "Stable-everything toolchain program".
