# Splitting `ntoskrnl-rust` into per-subsystem crates

**Goal:** cut the kernel edit→rebuild loop by breaking the ~453K-LOC `ntoskrnl-rust`
monolith (one rlib) into per-subsystem crates, so touching one subsystem no longer
re-runs rustc's whole-crate frontend over all 453K LOC (and so sccache, which is
crate-granular, can partition it).

Status: **lpc extracted as the PoC (2026-07-06) — validated end-to-end.** fsrtl next.

---

## The prize (measured)

`ntoskrnl-rust` is a single `crate-type = ["rlib"]` (405 files, ~453K LOC). Measured on
amd64, warm + sccache-hot:

| Action | Rebuild | Why |
|---|---|---|
| Touch any monolith file (before) | **~20.5s** | whole-crate frontend (parse/expand/typecheck/borrowck all 453K LOC) + link |
| Touch lpc file, lpc still a *dependency* of monolith | ~21.5s | **no win** — monolith depends on lpc → still rebuilds |
| Touch lpc file, dependency **severed** | **~11.2s** | only lpc-rust (4K LOC) recompiles; monolith untouched |

The pain is worse than the 20.5s floor suggests: a change to a shared *type/macro/common
fn* invalidates the whole crate's frontend (→ minutes); a cold/CI/`--clean` build compiles
all 453K LOC as one unit; **any `ntkrnl-sys` change rebuilds the entire monolith**; and the
monolith is one all-or-nothing sccache entry.

## Feasibility (why it's tractable)

Lighter coupling than a C mental model predicts, because the shared-type layer is already
externalized into `ntkrnl-sys` (its own crate) and most cross-subsystem *calls* go through
`#[no_mangle]`/`extern` FFI (link-time, zero Rust dep). Directed `crate::`-path coupling:

- **`ke` is the hub** (mm, ARM3, ex, ps, ob, config, po, kd64, vdm, inbv → ke).
- **Small cycles**: `ke↔ps`, `ke↔ARM3`, `ex↔config` (single-digit ref counts).
- **~10 clean leaves** — `se, iomgr, pnpmgr, fsrtl, cc, lpc, dbgk, kdbg, wmi, fstub` — have
  **zero** cross-subsystem `crate::` deps (~150K LOC, a third of the crate).

Subsystem LOC: ke 57K · ARM3 46K · iomgr 43K · ex 41K · se 35K · config 32K · pnpmgr 26K ·
ps 24K · ob 23K · fsrtl 21K · (kdbg mm fstub kd64 rtl cc po dbgk kd lpc wmi vdm … smaller).

---

## The three rules the lpc PoC established

**1. SEVER the monolith→crate dependency — this is the whole game.** Extracting lpc but
leaving `ntoskrnl-rust` depending on it (via `pub use lpc_rust as lpc`) gave **zero**
speedup (21.5s): touching lpc still invalidated the monolith. The win (→11.2s) only appears
once nothing in the monolith `use`s the extracted crate. **Every reference from the monolith
*into* an extracted subsystem must become FFI** (`extern` decl / link-time `#[no_mangle]`),
never a Rust `use`. For lpc the sole offender was `global_barrier.rs` (see rule 3).

**2. Force symbol inclusion via `extern crate` in the `kernel-rust` umbrella.** A subsystem's
`#[no_mangle]` symbols (Nt*/Lpcp*) must land in `libkernel_rust.a`. A plain Cargo dep
compiles the rlib but the linker drops its archive members (nothing references them at the
Rust level anymore, by rule 1). `kernel-rust`'s `extern crate lpc_rust;` forces them in —
the exact `cmboot-rust` pattern. Verified: `NtCreatePort`, `LpcpInitializePortQueue`,
`LpcPortObjectType` etc. all present in the linked `ntoskrnl.exe`.

**3. Each crate owns its own const-fold barrier.** `global_barrier.rs` is an auto-generated
(`tools/gen_global_barrier.py`) `#[no_mangle]` never-called fn that volatile-touches every
`pub static mut` so LLVM can't prove a 0-init global "never written" (writes come from C).
It referenced lpc's statics via `crate::lpc::…` — a monolith→lpc dep (violates rule 1). Fix:
each extracted crate gets its **own** barrier (`_lpc_global_write_barrier`) over its own
statics; the monolith's barrier drops those lines. → `gen_global_barrier.py` must become
crate-aware (regenerate one barrier per kernel crate) — **owed follow-up**.

### And one integration trap: lint/verify tools with hardcoded scan roots
Several tools hardcode `sdk/lib/rust/ntoskrnl-rust/src` as their sole root; extracting a
subsystem silently DROPS it from coverage (lint_kernel lost lpc's 34 findings — a
silent-degradation regression). Fixed via a single source of truth,
**`tools/kernel_crate_roots.py`** (`KERNEL_CRATE_SRC_ROOTS`), now consumed by `lint_kernel.py`
and `verify_signatures.py` (`verify_ffi.py` already globs every `*/src`). **Owed:** migrate
the one-shot tools that still hardcode the root (gen_global_barrier, apply_opaque_ptr,
fix_abi_mismatch, fix_extern_to_use, migrate_dbgprint, migrate_guards, verify_abi) if/when
re-run against a split kernel.

---

## The lpc extraction recipe (repeatable)

1. `git mv sdk/lib/rust/ntoskrnl-rust/src/<sub>/*.rs sdk/lib/rust/<sub>-rust/src/`; the old
   `mod.rs` → `lib.rs` + crate attrs (`#![no_std]` + the `allow`s from ntoskrnl-rust lib.rs).
2. New `<sub>-rust/Cargo.toml`: `crate-type = ["rlib"]`, dep `ntkrnl-sys` (default features).
   Add to workspace `members`.
3. **Sever** every monolith→sub reference (rule 1): drop `pub mod <sub>;`, repoint or FFI-ify
   any `crate::<sub>::…` consumer, move the sub's barrier lines into a `_<sub>_global_write_barrier`
   in the new crate.
4. `kernel-rust`: add the dep + `extern crate <sub>_rust;` (rule 2).
5. Add the sub's `src` to `tools/kernel_crate_roots.py` (rule integration-trap).
6. Build + verify symbols in `ntoskrnl.exe` (`llvm-nm`) + refresh baselines (pure relocation)
   + gate ×3 (boots).

Pre-flight per subsystem: `grep -rhoE "crate::[a-z]" src/<sub>` (what it needs FROM the
monolith — must be ~empty for a clean leaf) and `grep -rn "crate::<sub>" src/` outside the
sub (what the monolith needs FROM it — each becomes an FFI/sever site).

## Phased plan

- **Phase 0 — PoC**: ✅ lpc (done, this doc). ⏳ fsrtl (21K, next — bigger, still a clean leaf).
- **Phase 1 — clean leaves**: se, iomgr, pnpmgr, cc, dbgk, kdbg, wmi, fstub (~150K LOC, ~zero
  dep surgery). Biggest bang for the least risk.
- **Phase 2 — hub + cycles**: ke (foundational — many depend on it), mm/ARM3, ex, ps, ob,
  config. Break the small back-edges (`ke↔ps`, `ke↔ARM3`, `ex↔config`) via FFI or a shared
  micro-crate; move shared inline fns/macros to `ntkrnl-sys` or a `ntos-core` crate.

## The residual floor

After severing, touching lpc is ~11.2s — the remainder is a **fixed per-iteration floor**:
the `kernel-rust` staticlib archive rebuild + the `lld` link of `ntoskrnl.exe`, paid on *any*
change regardless of split. Attacking that (incremental/faster linking) is a separate axis
from the frontend win this split delivers; note it, don't conflate it.

## Invariants preserved

- **ABI unchanged** — `#[no_mangle]` symbols don't care about crate boundaries; the linked
  `ntoskrnl.exe` is functionally identical (symbol check + boot ×3).
- **The `lto=false` + `cgu=256` captures(none) firewall is strengthened**, not weakened —
  more crate boundaries = more codegen separation.
