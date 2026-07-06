# `--allow-multiple-definition` — feasibility verdict + the ratchet

**TL;DR.** We cannot remove `-Wl,--allow-multiple-definition` from the kernel
link: the duplicates it masks are dominated by a *structural, partly-impossible*
class. So instead we **ratchet** it — freeze the set of masked duplicates and
fail the gate on any NEW one — and **shrink** the reducible part. This is the
`tools/scan_duplicate_symbols.py` lint (wired into `vibeos-cli check`).

## Where the flag lives

`sdk/lib/rust/reactos-link/src/lib.rs` emits `-Wl,--allow-multiple-definition`
as a PE link flag "common to every module kind" — so *every* Rust-linked module
(ntoskrnl, every driver, every DLL, win32k) gets it. lld then resolves any
symbol defined in >1 input archive **first-match** instead of erroring.

## Why it can't be removed (the 3 classes)

Census of the 40 distinct duplicates lld masks on the i386 kernel link:

| Class | Count | Verdict |
|---|---|---|
| **Rust runtime** — `rust_eh_personality`, `rust_begin_unwind`, `memcpy`, compiler-builtins | ~3 (×N archives) | **Structural, partly IMPOSSIBLE.** The kernel PE links ~11 *independent* `crate-type=["staticlib"]` graphs (kernel-rust umbrella + rtl / cmlib / guids / cport / csq / arbiter / lsalib / rossym / cnt-rust-lib / pseh). Each bundles its own copy of the panic/eh/builtins runtime — irreducible while N>1 staticlibs feed one link. And they *can't* be merged into one graph: `rtl-rust` / `cnt-rust-lib` build a **different rustc target** (user-mode hardfloat + precompiled std) than the kernel-json softfloat `kernel-rust` (reactos-link `build_rust_crate` doc) — physically incompatible in one crate graph. |
| **GUID data** — `BATTERY_*_WMI_GUID`, `GUID_DEVICE_*`, `GUID_CLASS_INPUT` | ~17 | **Reducible — DONE.** `ntoskrnl-rust/src/guid.rs` carried byte-identical copies of GUIDs that `guids-rust/src/defs.rs` (the auto-generated wdmguid/poguid/ioevent replacement, `#[used]`, in the kernel link) already owns. Removed from ntoskrnl-rust; guids-rust is the single owner. Consumers still reference the symbol via `extern { static … }` and resolve to it. |
| **Deliberate cohab primitives** — e.g. `KeGetCurrentIrql` | few | **Reducible but low-value / intentional.** `arbiter-rust` defines its own amd64 `KeGetCurrentIrql` (a standalone `cr8` read, `#[cfg(x86_64)]`) so the arbiter lib is self-contained; `ntoskrnl-rust/shim_exports.rs` has the canonical one. Deduping would add a cross-crate link dependency for one amd64 symbol — not worth it. Baselined as accepted. |

Note: nm-over-archives *over-reports* this massively (it counts unpulled
`ntkrnl-sys` members bundled into each staticlib — the linker never pulls them,
so they're not real link collisions). Only lld's own diagnostics are ground
truth, and rustc suppresses those on a *successful* link. That's why the ratchet
is a **source** scan (see below), not an nm census.

## The ratchet — `tools/scan_duplicate_symbols.py`

Since we can't drop the flag, we prevent it from silently absorbing *new* zombie
masking (the `IsEqualGUID` near-miss: a phantom/duplicate symbol that only linked
by accident). The tool scans the kernel-link crate group for `#[no_mangle]`
fn/static **definitions** (the attribute only ever precedes a real definition —
`extern {}` import decls are correctly excluded) and reports any symbol defined
in ≥2 of those crates. Wired as a `vibeos-cli check` source lint; the harness
baseline-diffs `--lint` output against
`tools/lint-baseline/scan_duplicate_symbols.txt` and gates **NEW lines only**.

It is a deliberately **differential** check, not an exact model of the link:
- doesn't know which archive members lld pulls → benign entries freeze into the
  baseline (which doubles as the accepted-cohab registry);
- doesn't see compiler-emitted runtime or C-side cohab (out of scope);
- what it *reliably* catches is the thing that matters: a translation adding a
  `#[no_mangle]` that collides with a symbol another kernel crate already owns.

Structural runtime symbols (`rust_eh_personality`, `__rust_*`, …) are excluded
from the census so they don't read as debt.

Current baseline: **1** entry (`KeGetCurrentIrql`, the deliberate arbiter cohab).

## Owed / future shrink

- **Reducible primitive cohab** (`KeGetCurrentIrql`, and any that surface): route
  to a single owner if the cross-crate link dependency is acceptable. Low priority.
- **C-side cohab** (a `.c` still compiled beside its Rust replacement): a separate
  concern; not covered by this ratchet.
- The **structural runtime** class is only removable by collapsing the
  multi-staticlib design — blocked for the mixed-target siblings. Not planned.
