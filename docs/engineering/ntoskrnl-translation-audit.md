# ntoskrnl C→Rust Translation Audit

Last updated 2026-05-06.

## Where we are

**~99% of the original ntoskrnl C body of work is now Rust.** The only
active C in the default i386/amd64 build is the shim layer — ABI anchors,
FORCEINLINE/macro bridges, and vararg trampolines that augment the Rust
impls that replaced the original bodies. Every other ntoskrnl C file is
either inactive (commented out in `ntos.cmake`) or a Rust replacement.

Counts (from `find … | xargs wc -l`):

| Bucket | LOC |
|---|---:|
| Original ntoskrnl C body of work (excl. shims) | ~272,000 |
| Rust under `sdk/lib/rust/ntoskrnl-rust/src/` | ~439,400 |
| Active C in default i386/amd64 build (excl. shims) | 0 |
| C shim layer (`*_rust_shims.c` + `kernel_c_helpers.c` + `mm_sysldr_shims.c` + `ntprintf_trampoline.c`) | ~5,500 |

The Rust LOC is higher than the original C because (a) bindgen unions need
explicit accessor paths, (b) every output-pointer write goes through
`write_volatile` or an `opaque_ptr` launder, (c) the augment+isolated-rlib
pattern means C trampolines + Rust impls coexist for ~165 fns. Net work-done
metric is closer to 1:1.

Smoke status: i386 7/7, amd64 7/7, arm64 0/7 (pre-existing baseline hang at
`[HIVE] CmCheckRegistry done`, identical 13,449 bytes serial across recent
kernel changes — not in any specific subsystem).

## What's still C

| Bucket | LOC | Notes |
|---|---:|---|
| `*_rust_shims.c` + `kernel_c_helpers.c` + `mm_sysldr_shims.c` + `ntprintf_trampoline.c` | ~5,500 | Bridge layer. See `shim-elimination.md` for status — most are tractable, the "must-stay" set is small and shrinking. Down from ~7,200 after the 2026-05-06 dead-trampoline sweep (Rounds 1–9). |
| `cmdata.c` Phase B holdouts | ~150 | `MmConsumedPoolPercentage` and a handful of inherent registry-structure tables. |

**Translation candidates with no Rust translation in tree: zero.** Every C
file the default i386/amd64 build pulls in either has its Rust replacement
active, or is a shim that augments the Rust impl living in a sibling rlib.

The `cache/` directory (the historical NEWCC alternate cache subsystem)
is gone as of 2026-05-06. The existing Rust CC subsystem
(`ntoskrnl-rust/src/cc/`) is the only cache implementation; if it needs
work, the fix lands directly in Rust, not via porting a parallel C
version.

## What's still C — and what we used to think had to be

The previous version of this doc had a long list of "intentional-stay"
classifications (captures(none) workarounds, complex-delegation Iop fns,
calling-convention bridges, AVL trees with `#include <avlsupp.c>`, vararg
trampolines, etc.). **Most of them turned out to be tractable.** As of
2026-05-06:

- All 10 io captures(none) workarounds: eliminated. The "LLVM cross-CGU
  attribute propagation" excuses on every one of them turned out to be
  defeated by the workspace `lto = false` + `codegen-units = 256` firewall
  (which has been the default for months — the shim comments were just
  stale). Re-probing them was cheap, and not one of the 10 needed
  structural changes.
- 8 `_c`-suffixed PnP shims: all eliminated, mix of direct translations and
  dead-code deletions. Each had a "Rust hangs on amd64" comment which
  turned out to be a captures(none) ghost.
- `IopInitializeDriverModule` (240 LOC, the last "complex delegation"
  on the io ledger): translated. The bug that surfaced during the port
  wasn't a translation bug — it was an `extern "C"` vs `extern "system"`
  ABI mismatch on `IopReadyDeviceObjects` that had been latent in
  driver.rs for months, working on amd64+arm64 (where NTAPI = cdecl) and
  blowing up on i386. The shim had been hiding it.
- ~28 calling-convention bridges (`extern "fastcall"` / NTAPI / cm
  dual-decoration `_shim_*`): all eliminated. The "Rust can't generate
  the right decoration on i686-pc-windows-gnu" excuse was stale; mingw
  lld emits the right `_Name@N` / `@Name@N` for `extern "system"` /
  `extern "fastcall"` directly.

**Pattern.** Many shim categories that were classified "must stay in C"
turned out to be hiding real bugs in the Rust caller (ABI mismatch,
captures(none) workaround left over from before the firewall, latent
flexible-array indexing bug, etc.). The shim wraps the symptom; the
underlying Rust bug only surfaces when the shim is removed and the call
flow goes direct.

**2026-05-06 sweep result (Rounds 1–9).** A second-pass audit over the
shim layer dropped ~925 LOC (-14.4%) from `*_rust_shims.c` +
`kernel_c_helpers.c` by collapsing trampolines that fell into one of
two patterns:

1. **3-layer trampoline collapse** — `Rust caller → C trampoline X →
   Rust impl _rs_X` where the C trampoline did nothing but ABI bridge.
   Rename the rlib export from `_rs_X` to `X` (or `X_shim`) directly;
   delete the C trampoline. Done for io-inlines, cm-inlines, mm-avl
   (7 of 8 — `RtlpInsertAvlTreeNode` keeps a guard).
2. **Linker-confirmed duplicate delete** — both `*_rust_shims.c` and
   `ntoskrnl-rust::shim_exports` were exporting the same `Mi*` /
   `Ki*` / `Kx*` / `Mm*` name with the same i386 decoration; lld's
   `--allow-multiple-definition` let the link succeed but the C
   trampoline was unreachable. Done for ~25 Mi* (mm), ~50 Ki/Kx (ke),
   ~16 cross-cutting (mm + io). Dropped `kernel_c_helpers.c` AVL
   block + amd64 KfRaiseIrql + i386 cdecl InterlockedExchange wrappers
   + Mm globals along the way.

Failure modes encountered:
- `KeGetTrapFramePc` and `KiReleaseProcessLock`: shim_exports.rs only
  provides one ABI variant (stdcall via `extern "system"`) but bindgen
  emits the call as cdecl `extern "C"`; both decorations need a
  definition on i386. Caught at link time; restored just those two
  trampolines.
- `MiAcquireExpansionLock` / `MiReleaseExpansionLock` debug tracking
  (`MiExpansionLockOwner = …`) was dropped during the sweep; it was
  already dead in the linked binary because shim_exports.rs's
  no-tracking version won the link under
  `--allow-multiple-definition`. Re-adding tracking on the Rust side
  is a planned follow-up.

So **assume any "intentional-stay" classification is a candidate for
re-probing**, not a settled answer. The current must-stay set, conservatively:

- Variadic trampolines (the `va_arg` pull is the function — Rust can't
  cross that boundary safely)
- A handful of LLVM/KVM workarounds with documented evidence
  (`KiSaveXState`/`KiRestoreXState` on amd64, `SepInitTokensShim` i386
  fat-LTO ud2)
- ~~C `#include <avlsupp.c>` AVL bridges~~ — closed 2026-05-06; the
  Mm AVL flavor lives in the `mm-avl` rlib now.
- `__FILE__`/`__LINE__` macros that need the C preprocessor
- KDBG enum-name-to-string lookup tables (debugger-only, no Rust value-add)

Everything else is in principle eliminable. The audit's job is to track
which are actually worth eliminating, not to bless the current state.

## Active in Rust (default build)

Counts are *active modules*. Files on disk that exist but aren't wired into
`mod.rs` (or are gated off per-arch) live in the parked-translation list,
which is currently empty.

| Subsystem | Active mods | LOC active |
|---|---:|---:|
| KE (kernel) | 68 | ~54,900 |
| ARM3 (mm internals) | 25 | ~44,100 |
| iomgr | 26 | ~42,100 |
| EX (executive) | 31 | ~38,800 |
| SE (security) | 18 | ~35,200 |
| config (registry CM) | 22 | ~33,950 |
| pnpmgr | 14 (+arb) | ~26,000 |
| ps (process) | 13 + per-arch | ~23,400 |
| ob (object) | 11 | ~22,500 |
| fsrtl | 17 | ~21,000 |
| kdbg (KDB) | 9 | ~13,500 |
| fstub | 4 | ~12,300 |
| mm | 12 | ~11,900 |
| kd64 | 7 + per-arch | ~12,000 |
| cc (cache) | 7 | ~6,200 |
| dbgk / po / inbv / lpc / kd / wmi / vdm / arch / vf | 38 | ~25,200 |
| **TOTAL active (i386/amd64)** | **~322** | **~439K LOC** |

Parked translations: **none.** Every Rust file in tree is wired in.

## Stale-claim watch (worth keeping for the next session)

The previous audit's notes on parked subsystems were several months out of
date. **Confirm before working from them.** The general lesson, repeated
across many sessions:

> When you see ud2 + folded success path on i386, run `nm` on the binary
> and check `verify_abi.py` BEFORE going down the captures(none) rabbit
> hole. Both produce the same disasm pattern; ABI is a 1-second check.

Specific examples worth remembering:

- **kd64** ("recursive exception flood on amd64"): silently fixed months
  before re-test. Just needed wiring.
- **lpc/connect** ("captures(none) cascade — confirmed"): file already had
  the right `opaque_ptr` defenses. Built clean on all arches with zero
  source changes.
- **pnpres** ("translated, never wired"): 2 small fixes, builds + 7/7 on
  i386 and amd64.
- **iomgr/ramdisk on arm64** ("PE layout issue → 0x6B crash"): half-stale.
  Cfg gate had to come off, but the actual fix was wiring `_force_iomgr_symbols`
  to pull statics out of the staticlib archive.
- **ke/amd64/usercall** ("return-address corruption BSOD"): actual bug was
  importing `KeGdiFlushUserBatch` as `extern "system" fn ...` when it's a
  global `PGDI_BATCHFLUSH_ROUTINE` data symbol (function pointer filled in
  at win32k bringup). Fix: import as `static mut` and call via
  `(KeGdiFlushUserBatch.unwrap())()`.
- **`IopInitializeDriverModule` "complex delegation"** (the 2026-05-06
  case): real bug was an `extern "C"` vs `extern "system"` ABI mismatch on
  `IopReadyDeviceObjects` in driver.rs predating this commit. amd64+arm64
  linked silently because NTAPI=cdecl on those arches; i386 stdcall callee
  + cdecl call-site folded the success-path tail to ud2.
- **All 10 io captures(none) workarounds**: every comment claiming "LLVM
  miscompiles cross-CGU captures(none)" turned out to be solvable by the
  workspace firewall. Not one needed structural changes.

When picking a parked file or shim to revive, the cheapest first step is
often: re-enable it, build, smoke. The bug may already be gone.

## What this audit doesn't measure

- **Boot correctness.** A line-count metric says nothing about whether the
  Rust version actually works under load. Smoke is the source of truth.
- **Per-arch differentials.** i386 and amd64 are at parity; arm64 has its
  own baseline issue.
- **Implicit shim debt.** The ~7.2K LOC of `*_rust_shims.c` is technically
  "C in the build", but most of it is one-line `_rs_*` delegations to a
  Rust impl in a sibling rlib. The shim-elimination plan tracks this in
  detail.

## How these numbers are computed

```bash
# Total ntoskrnl C body of work (excludes shims; ntoskrnl/tests/ + ntoskrnl/cache/
# both deleted — tests migrated to Rust kernel-test harness, NEWCC retired):
find ntoskrnl -name "*.c" \
  | grep -v -E '_rust_shims\.c|kernel_c_helpers\.c|mm_sysldr_shims\.c|ntprintf_trampoline\.c' \
  | xargs wc -l   # → ~272,000

# Active C in default build (excludes shims):
grep -E '^\s+\$\{REACTOS_SOURCE_DIR\}/ntoskrnl/.*\.c' ntoskrnl/ntos.cmake \
  | grep -v '^\s*#' \
  | grep -v -E '_rust_shims\.c|kernel_c_helpers\.c|mm_sysldr_shims\.c|ntprintf_trampoline\.c' \
  | sed 's|^\s*\${REACTOS_SOURCE_DIR}/||; s|).*||; s| .*||' \
  | sort -u | xargs wc -l   # → 0

# Rust LOC across ntoskrnl-rust/:
find sdk/lib/rust/ntoskrnl-rust/src -name "*.rs" | xargs wc -l   # → ~439,400
```
