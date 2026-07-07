# Splitting the kernel monolith into per-subsystem crates

**Goal:** cut the kernel edit→rebuild loop by breaking the (originally) ~456K-LOC
`ntoskrnl-rust` monolith (one rlib) into per-subsystem crates, so touching one subsystem no
longer re-runs rustc's whole-crate frontend over all ~456K LOC (and so sccache, which is
crate-granular, can partition it).

**Status (2026-07-07): ~63% extracted.** `ntoskrnl-rust` is down from ~456K → **170K LOC**;
**~286K LOC across 21 crates** now live outside the monolith and talk to it over the FFI
symbol boundary. Phase 0 (PoC) + Phase 1 (clean leaves) + the leaf half of Phase 2
(config/ps/ob) are **landed on master and boot ×3**. What remains inside the monolith is the
dense center — **ke (58K) + mm/ARM3 (59K)** — plus **ex (41K), which is extracted and
compiles/links ×3 but is PARKED** on a boot regression (see "ex" below). After that: a
follow-on **Phase 3 for win32k**, the *other* hot 257K monolith.

---

## As-built — what's extracted (landed, ×3-green)

Each crate depends ONLY on `ntkrnl-sys` (+ `dbgprint-rust`); every reference to/from the
monolith crosses a **symbol** boundary (`#[no_mangle]` / `extern`), never a Rust
`crate::other::X` path. Ordered roughly by extraction.

| Crate | LOC | Crate | LOC |
|---|--:|---|--:|
| iomgr-rust | 43K | kd64-rust | 12K |
| se-rust | 35K | rtlk-rust (rtl-kernel glue) | 7.8K |
| config-rust (registry) | 33K | cc-rust | 6.3K |
| pnpmgr-rust | 27K | po-rust | 5.1K |
| ps-rust | 24K | dbgk-rust | 5.1K |
| ob-rust | 23K | lpc-rust | 4.0K |
| fsrtl-rust | 21K | kd-rust | 3.9K |
| kdbg-rust | 14K | wmi-rust | 3.8K |
| fstub-rust | 12K | inbv-rust | 3.7K |
| | | vdm-rust (i386-only) | 1.3K |
| | | vf-rust | 0.2K |
| | | dbgprint-rust (print-infra leaf) | 0.5K |

`kernel-rust` (the umbrella staticlib) now carries **~26 `extern crate` lines** forcing these
symbols into the link. `tools/kernel_crate_roots.py` and
`tools/scan_duplicate_symbols.py::KERNEL_LINK_CRATES` are the two source-of-truth lists that
must gain an entry per extraction (see integration-trap below).

## The prize (measured, from the lpc PoC)

`ntoskrnl-rust` was a single `crate-type = ["rlib"]`. Measured on amd64, warm + sccache-hot:

| Action | Rebuild | Why |
|---|---|---|
| Touch any monolith file (before) | **~20.5s** | whole-crate frontend (parse/expand/typecheck/borrowck the whole crate) + link |
| Touch lpc file, lpc still a *dependency* of monolith | ~21.5s | **no win** — monolith depends on lpc → still rebuilds |
| Touch lpc file, dependency **severed** | **~11.2s** | only lpc-rust recompiles; monolith untouched |

Same ~link-floor result regardless of subsystem size (lpc 4K and fsrtl 21K both landed at
~11–12s). The pain is worse than the 20.5s floor suggests: a change to a shared
*type/macro/common fn* invalidates the whole crate's frontend (→ minutes); a
cold/CI/`--clean` build compiles the whole monolith as one unit; **any `ntkrnl-sys` change
rebuilds the entire monolith**; and the monolith is one all-or-nothing sccache entry. Every
subsystem lifted out shrinks all of those.

## Feasibility (why it's tractable)

Lighter coupling than a C mental model predicts: the shared-type layer is already
externalized into `ntkrnl-sys` (its own crate) and most cross-subsystem *calls* already go
through `#[no_mangle]`/`extern` FFI (link-time, zero Rust dep). The whole perimeter and the
"leaf" core are C-derived and talk via symbol boundaries, so severance is mostly mechanical.
Directed `crate::`-path coupling:

- **`ke` is the hub** (mm, ARM3, ex, ps, ob, config, po, kd64, vdm, inbv → ke).
- **Small cycles**: `ke↔ps`, `ke↔ARM3`, `ex↔config` (single-digit ref counts).

---

## Rules the extraction wave established (corrected)

**1. SEVER the monolith→crate dependency — this is the whole game.** Extracting a subsystem
but leaving `ntoskrnl-rust` depending on it (via `pub use x_rust as x`) gives **zero**
speedup: touching it still invalidates the monolith. The win only appears once nothing in
the monolith `use`s the extracted crate. **Every reference from the monolith *into* an
extracted subsystem must become FFI** (`extern` decl / link-time `#[no_mangle]`), never a
Rust `use`.

**2. Force symbol inclusion via `extern crate` in the `kernel-rust` umbrella.** A subsystem's
`#[no_mangle]` symbols must land in `libkernel_rust.a`. A plain Cargo dep compiles the rlib
but the linker drops its archive members (nothing references them at the Rust level anymore,
by rule 1). `kernel-rust`'s `extern crate x_rust;` forces them in — the `cmboot-rust`
pattern. Verify post-link with `llvm-nm` on `ntoskrnl.exe`.

**3. `global_barrier` is DEAD — do NOT carry it into new crates.** *(Corrected from the
original plan, which had each crate own a barrier.)* The monolith's master `global_barrier.rs`
is **unwired** — there is no `mod global_barrier;` in `ntoskrnl-rust/lib.rs`, so it never
compiles — and an extracted crate's barrier is **dead-stripped from the exe** (proven via
`nm`: the `_force_*`/barrier symbols aren't present). Barriers are non-functional cosmetic
today (the captures(none) root they guarded is toolchain-fixed on LLVM ≥ 21 — see
rust-kernel-patterns "RESOLVED AT THE TOOLCHAIN"). **Skip them.** Likewise the
`_force_<sub>_symbols` CGU-forcing hacks are **redundant** when the subsystem's fns have
genuine callers (C or Rust) pulling them via `extern crate` — **delete them** (verify the
symbols are still in the exe after).

**4. Crate-root macros are hidden coupling — invisible to a `crate::`/`use` audit.** Bare
`name!(…)` invocations of a `macro_rules!` defined at `ntoskrnl-rust/lib.rs` (ASSERT,
dbg_print, wstr, `abi_cast`, …) don't show in a `crate::`/`use` grep. Pre-flight MUST also
`grep -rl '<macro>!'` each crate-root macro against the subsystem. Copy the macro into the
new crate (or re-export from `dbgprint-rust`) as needed. **Owed:** hoist the shared kernel
macros into `ntkrnl-sys` (or an `ntos-macros` crate) so every extracted crate shares one
definition instead of copying.

**5. The crate boundary EXPOSES latent ABI-decl bugs — expect them, they are the payoff.**
A `#[no_mangle] extern "system"` def (i386 stdcall `_Name@N`) referenced elsewhere via
`extern "C"` (cdecl, no `@N`) **links fine while caller and callee share a crate** — rustc
resolves the call to the local def and ignores the wrong-ABI decl. The crate boundary turns
that into an honest **undefined-symbol link error** (i386 only — amd64/arm64 NTAPI is cdecl,
so the mismatch is invisible there). Fix by matching the decl to the true C ABI: NTAPI →
`extern "system"`, plain-cdecl → `extern "C"`. This surfaced repeatedly (pnpmgr
`_IopGetSystemPowerDeviceObject`, kdbg `KdbpAcquireLock`, ex `_KiStartSyncCensus`, …) — each
was a **real pre-existing latent bug** the split flushed out, not a regression the split
introduced.

**6. const-aggregation across a to-be-split boundary must become runtime assembly.** A
compile-time `const` table that aggregates slices owned by *both* sides of a split (the
registry `CmControlVector`, which pulled slices from config AND ob/mm/ex/ps/kd64) **cannot**
survive the split: const-eval cannot reference an extern (other-crate) static's address. Fix:
each subsystem exports its slice as a `#[no_mangle] static`, and the owner **assembles the
table at boot** by copying each slice via extern reads (`cm_build_control_vector()` in
`config/cmdata.rs`, called before `CmGetSystemControlValues` in `ex/init.rs`). This
runtime-assembly pattern is what made config (and the core generally) severable at all.

### Integration trap: lint/verify tools with hardcoded scan roots
Several tools hardcode `sdk/lib/rust/ntoskrnl-rust/src` as their sole root; extracting a
subsystem silently DROPS it from coverage (a silent-degradation regression). Fixed via a
single source of truth, **`tools/kernel_crate_roots.py`** (`KERNEL_CRATE_SRC_ROOTS`),
consumed by `lint_kernel.py` / `verify_signatures.py` (`verify_ffi.py` already globs every
`*/src`). When you extract, add the new `src` there AND to
`scan_duplicate_symbols.py::KERNEL_LINK_CRATES`. Also re-check the **full** lint suite after
each extraction — findings just *relocate* (e.g. `scan_ob_refcount` OBREF_LEAK_RETURN moved
ps/process.rs→ps-rust), so the baseline diff shows moves as remove+add; regenerate baselines
(pure relocation).

---

## The extraction recipe (repeatable)

1. `git mv sdk/lib/rust/ntoskrnl-rust/src/<sub>/*.rs sdk/lib/rust/<sub>-rust/src/`; the old
   `mod.rs` → `lib.rs` + crate attrs (`#![no_std]` + the `allow`s from ntoskrnl-rust lib.rs).
   **Gate arch-specific modules, never the crate** — a crate-level `#![cfg(target_arch=…)]`
   strips `#![no_std]` on the excluded arch and demands std (E0463); keep `#![no_std]`
   unconditional and `#[cfg]` the `pub mod`s (vdm-rust is the i386-only exemplar).
2. New `<sub>-rust/Cargo.toml`: `crate-type = ["rlib"]`, dep `ntkrnl-sys` + `dbgprint-rust`.
   Add to workspace `members`.
3. **Sever** (rule 1): drop `pub mod <sub>;` from `ntoskrnl-rust/lib.rs`; rewrite intra-crate
   `crate::<sub>::…` → `crate::…`; convert every outbound `use crate::ke::X` (etc.) into an
   `extern` decl with the **correct ABI** (rule 5). Do NOT add a barrier (rule 3).
4. `kernel-rust`: add the dep + `extern crate <sub>_rust;` (rule 2).
5. Add the sub's `src` to `tools/kernel_crate_roots.py` AND `KERNEL_LINK_CRATES` (integration
   trap).
6. Build ×3 arches (`vibeos-cli build`; arm64 wants `--clean` to reprovision a stale config)
   + verify symbols in `ntoskrnl.exe` (`llvm-nm`) + refresh lint baselines (pure relocation)
   + gate ×3 (boots). Use `git add -u <path>` for the moved-file deletions (a bare
   `git add <deleted-dir>` aborts the whole command on the gone dir).

Pre-flight per subsystem: `grep -rhoE "crate::[a-z]" src/<sub>` (what it needs FROM the
monolith — becomes outbound externs); `grep -rn "crate::<sub>" src/` outside the sub (what
the monolith needs FROM it — each becomes a sever/FFI site); AND per rule 4, for each
crate-root `macro_rules!`, `grep -rl '<macro>!' src/<sub>`.

## What's left (the hard center)

- **ex (41K) — extracted, compiles/links ×3, arm64 BOOTS, but PARKED on an x86 boot hang.**
  i386 AND amd64 hang deterministically at `IopInitializeBuiltinDriver: ksecdd.sys`
  (preceded by `MmGetSystemRoutineAddress: 'RtlQueryModuleInformation' NOT FOUND`). NOT the
  known win32k `GateWait` flake — a real x86-only regression the extraction surfaced.
  **Preserved on branch `ex-crate-split-x86-hang` (commit `20043e96bed`); reverted from
  master so mainline stays clean.** Prime suspects (x86-specific, since arm64 works):
  (a) the pushlock debug recorders `synctrace_pl_{park,satisfied,wake}` + `lockdep_check_park`
  changed from Rust-`pub fn` to `#[no_mangle] extern "C"` (hot path — an ABI subtlety hangs
  late); (b) a wrong-typed x86 extern reading garbage (the array-typed ones —
  `KeServiceDescriptorTable[2]`, `KiProcessorBlock`, `KeNodeBlock`); (c) a symbol the
  `_force_sysinfo_symbols` deletion dropped that ksecdd/module-info needs. This is the one
  split that needs a debugging pass before it lands.
- **ke (58K) — the hub.** Everything depends on it; it has no clean FFI perimeter *because*
  it IS the perimeter for everyone else. Break the back-edges (`ke↔ps`, `ke↔ARM3`) via FFI or
  a shared micro-crate; move shared inline fns/macros to `ntkrnl-sys`/`ntos-core` first. A
  bad severance here hangs every arch, not just x86 — do it deliberately.
- **mm + ARM3 (59K) — the memory manager.** Same story: dense, foundational, `ke↔ARM3` cycle.

The residual per-iteration floor after severing (~11s) is the `kernel-rust` staticlib archive
rebuild + the `lld` link of `ntoskrnl.exe`, paid on *any* change regardless of split.
Attacking that (incremental/faster linking) is a separate axis from the frontend win this
split delivers.

---

## Phase 3 — win32k (the OTHER hot monolith)

**Rationale.** LOC ranking is the wrong axis for prioritizing the split; **edit-frequency ×
blast-radius** is. By that measure the giant driver/userland blobs (cdrom 211K, disk 170K,
classpnp 166K, kernel32 161K, the storage/USB fleet) are effectively *cold* — big but rarely
touched, and self-contained when they are. The heat is concentrated in exactly two places:
**ntoskrnl and win32k**, and they co-change (a GUI bug is usually win32k + a kernel primitive;
a boot bug is ntoskrnl + a win32k callback). So after the kernel, **win32k is the
highest-leverage remaining target** — and it is the same monolith pathology: **257K LOC in a
single `crate-type=["rlib"]`, all 166 files flat in `src/` (no subsystem dirs at all)**.

**It splits along a textbook seam.** win32k has always been two engines (literally separable
in NT):

- **USER half (~90K)** — windowing/messaging/input: `winpos`, `message`, `nonclient`,
  `callback`, `ime`, `clipboard`, `scrollbar`, `monitor`, `keyboard`, window/desktop/winsta
  management.
- **GDI / graphics-engine half (~63K+)** — `bitblt`/`stretchblt`/`ntbitblt`,
  `pdevobj`/`ldevobj`/`device`, `dc`/`windc`, `surface`, `gradient`, `coord`, `drawing`,
  `painting`.
- **`freetype.rs` (9K) is a free win** — a self-contained font rasterizer with zero kernel
  coupling; it can pop out into its own leaf crate first, on its own, as a low-risk warm-up.

USER↔GDI is a *cleaner* cut than ke↔mm.

**The catch — timing/coordination, not difficulty.** The reason the kernel *leaf* subsystems
(ps/ob/config/…) were safe to grind out is they're not what agents have open — they could be
churned without colliding. win32k is the opposite: it's a hot file under constant edit, and a
257K reshuffle (`git mv` of 100+ files + every intra-crate `crate::` rewrite) would conflict
with anyone mid-edit. So win32k is **high-value but must be done as one deliberate pass when
the win32k lane is quiet** — not ground out overnight alongside other work. Sequence:
`freetype` leaf first (anytime), then the GDI/USER cut in a single coordinated pass.

## Recommended order

1. **Finish the kernel** — debug + land **ex**, then split **ke** and **mm/ARM3**. This is
   the hot half actively under edit; completing it retires the monolith.
2. **win32k** — `freetype` leaf, then GDI/USER, done as one coordinated pass in a quiet
   window.

## Invariants preserved

- **ABI unchanged** — `#[no_mangle]` symbols don't care about crate boundaries; the linked
  `ntoskrnl.exe`/`win32k.sys` is functionally identical (symbol check + boot ×3).
- **The `lto=false` + `cgu=256` captures(none) firewall is strengthened**, not weakened —
  more crate boundaries = more codegen separation.
