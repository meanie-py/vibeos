# cmdata.c Chopping Plan (Phase B) — DONE

**Status (2026-05-04):** Phase A and all four Phase B steps landed. Every cross-subsystem CmControlVector entry now lives in the file that owns its variable. cmdata.rs is down to ~593 LOC (from 994 LOC of original C / ~750 LOC of Phase A Rust); only entries pointing at cmdata-owned globals (`DummyData`, language IDs, suite buffer, the `Cm*` config vars, `MmConsumedPoolPercentage` extern) plus the 4 inherent registry-structure tables (`CmpMachineHiveList`, `CmClassName`, `CmTypeName`, `CmpMultifunctionTypes`) remain.

This file is preserved as the historical record of how the chop was designed and executed; future readers wanting to understand the const-concat scheme should still find it useful.

**Original framing:** This file captured the chopping plan that previously lived only in chat history. The original cmalloc/cmdelay/cmlazy/cmkcbncb chops applied the same pattern when those files were translated; cmdata was the last and biggest holdout.

## Phase A — done

- `sdk/lib/rust/ntoskrnl-rust/src/config/cmdata.rs` — verbatim port of `ntoskrnl/config/cmdata.c`. All globals defined as `#[no_mangle] pub static mut` with the same names; arrays (`CmpMachineHiveList`, `CmClassName`, `CmTypeName`, `CmpMultifunctionTypes`, `CmControlVector`) initialized as const arrays.
- `CmControlVector: [_CM_SYSTEM_CONTROL_VECTOR; 249]` — single flat table. 88 hand-coded entries + 156 `kd_filter!` debug-print-mask entries + 4 trailing + 1 NULL terminator.
- C-side `cmdata.c` commented out of `ntos.cmake`.
- `ex/init.rs:CmGetSystemControlValues` call site updated to use path `crate::config::cmdata::CmControlVector` instead of the old `extern "C"` block.
- `tools/gen_global_barrier.py` will need to pick up the new `pub static mut`s on next regen, but `_global_write_barrier()` is regenerated lazily; not a build blocker.

## Phase B — the chop

### Goal

Each subsystem owns the `CmControlVector` entries that reference its variables. Reading the audit table for "what's registry-controllable in subsystem X?" becomes a local question.

### Mechanism (the question we resolved before Phase A)

**Compile-time const concat.** Each subsystem exports:

```rust
// ob/obinit.rs
pub static OB_CONTROL_VECTOR: [_CM_SYSTEM_CONTROL_VECTOR; N] = unsafe { [
    cv!("Session Manager", "ProtectionMode", core::ptr::addr_of_mut!(ObpProtectionMode)),
    cv!("Session Manager", "ObjectSecurityMode", core::ptr::addr_of_mut!(ObpObjectSecurityMode)),
    // …
] };
```

`config/cmdata.rs` keeps `CmControlVector` as the public symbol but builds it from the sub-vectors with a `const fn` concat loop:

```rust
const fn concat_n(slices: &[&[_CM_SYSTEM_CONTROL_VECTOR]]) -> [_CM_SYSTEM_CONTROL_VECTOR; TOTAL] {
    let mut out: [_CM_SYSTEM_CONTROL_VECTOR; TOTAL] = [NULL_ENTRY; TOTAL];
    let mut dst = 0;
    let mut s = 0;
    while s < slices.len() {
        let mut i = 0;
        while i < slices[s].len() { out[dst] = slices[s][i]; i += 1; dst += 1; }
        s += 1;
    }
    out
}

pub static mut CmControlVector: [_CM_SYSTEM_CONTROL_VECTOR; 249] = concat_n(&[
    &crate::ob::obinit::OB_CONTROL_VECTOR,
    &crate::ARM3::pfnlist::PFNLIST_CONTROL_VECTOR,
    // …
    &CMDATA_LOCAL_CONTROL_VECTOR,  // entries that reference cmdata-owned globals
    &CMDATA_TERMINATOR,             // [NULL_ENTRY; 1]
]);
```

This keeps:
- `CmControlVector` as the single C-compatible symbol (no consumer changes).
- Zero runtime moving parts (no boot-time alloc, no init step).
- Each subsystem's slice colocated with the variables it points at.

The `cv!` macro from cmdata.rs needs to be exported (or re-defined per-subsystem). Easiest: lift it into `config/cmdata.rs` as `pub macro_rules!` and `use` it at the call site.

### Migration order — what actually shipped

Done in 4 commits, smallest to biggest, smallest-risk first. The const-concat scheme handled every step the same way: each owning file declares `pub static FOO_CONTROL_VECTOR: SubVector<N>`, cmdata.rs's `SLICES` list grows by one entry, `CMDATA_LOCAL_CV` shrinks by N. CV_TOTAL recomputes from `SLICES`.

| Step | Commit | Entries | Subsystems chopped | New SubVectors |
|-----:|--------|--------:|--------------------|----------------|
| 1 | `06156c2b` | 5 | **ob** | `ob/{obinit,devicemap,obname}::{OBINIT,DEVICEMAP,OBNAME}_CONTROL_VECTOR` |
| 2 | `2609d390` | 156 | **kd64 debug filters** | `kd64/kddata::KD_FILTER_CONTROL_VECTOR` (also moved the local `kd_filter!` macro) |
| 3 | `c372ecd8` | 27 | **mm/ARM3** (8 files) | `ARM3/{pfnlist,mminit,helpers,mdlsup,drvmgmt,sysldr}::*_CONTROL_VECTOR` + `mm/{mminit,pagefile}::*_CONTROL_VECTOR` |
| 4 | (this commit) | 9 | **ex (3), ps (1), iomgr (1), cmlazy (2), cmdelay (1), kd64 tunables (1)** | `ex/{resource,init}::*_CONTROL_VECTOR`, `ps/process::PROCESS_CONTROL_VECTOR`, `iomgr/iomgr::IOMGR_CONTROL_VECTOR`, `config/{cmlazy,cmdelay}::*_CONTROL_VECTOR`, `kd64/kddata::KDDATA_TUNABLES_CV` |
| **Total** | | **197** | 19 sub-vectors across 18 files | |

**One entry held back:** `MmConsumedPoolPercentage` (Session Manager\Memory Management / PoolUsageMaximum) stays in `cmdata.rs` because the variable itself is still C-side (no Rust home). When pool.c is fully translated to Rust and that global gets a Rust definition, this entry can move.

**One step bisect lesson** (step 3): the first attempt with `--skip-clean-build` showed an i386 boot failure (FreeLdr loop, 0 milestones). Bisect-by-revert proved the chop was sound — cargo's incremental cache produced a corrupt staticlib for i386 only; full clean rebuild was 7/7 PASS. AGENTS.md already calls out the rule: **always run smoke without `--skip-clean-build` for final verification.**

What's left in `cmdata.rs` after all four steps:
- The simple cmdata-owned globals (`DummyData`, `CmNtGlobalFlag`, language IDs, suite buffer, `CmControlHive`, push locks, `CmpVolatileBoot`, `CmSelfHeal`, etc.).
- `CmpMachineHiveList`, `CmClassName`, `CmTypeName`, `CmpMultifunctionTypes` (these are inherently cmdata's because they describe registry structure, not subsystem-specific data).
- `CMDATA_LOCAL_CONTROL_VECTOR` — entries that reference cmdata-owned globals (`DummyData` placeholders, language IDs, suite buffer entries, `CmpVolatileBoot`, `CmSelfHeal` settings, `CmpLazyFlush*`, `CmpDelayedCloseSize`, the WMI/CrashControl trailing entries — most of which use `&DummyData`).
- The const concat assembler.

Final cmdata.rs size: ~593 LOC (down from 994 LOC of C / ~750 LOC of Phase-A Rust). The estimate was lower because the cmdata-local table still has ~50 entries pointing at DummyData (registry slots reserved but not actually wired to anything functional), plus the four inherent registry-structure tables (CmpMachineHiveList, CmClassName, CmTypeName, CmpMultifunctionTypes) which weigh ~150 LOC.

### Mechanism details

**Where to put the cv! macro.** Currently in `cmdata.rs`. Either:
- Move it to a small `config/control_vector.rs` module that everyone `use`s.
- Make it `#[macro_export]` from `cmdata.rs` and add `extern crate self as ntoskrnl_rust;` if needed for path resolution.
- (Easiest) Duplicate the macro per subsystem — it's 13 lines and the cv! pattern is stable.

**Total entry count.** The `[T; TOTAL]` constant needs to know `TOTAL` at compile time. Since each subsystem's slice has `.len()` (const since 1.39), we can compute:

```rust
const TOTAL: usize =
    crate::ob::obinit::OB_CONTROL_VECTOR.len() +
    crate::kd64::kddata::KD_FILTER_CONTROL_VECTOR.len() +
    // …
    1; // NULL terminator
```

**addr_of_mut! on extern statics in const context.** Already exercised in Phase A — works. The relocation is resolved at link time.

**addr_of_mut! on a struct field of an extern static.** `addr_of_mut!(KdpContext.KdpDefaultRetries)` — this is a place-expression macro; it accepts field access. Works in const context for `static mut`s as well (the address is `&KdpContext + offsetof(KdpDefaultRetries)`, both compile-time-known).

### Per-step verification

For each migration step:
1. Add `pub static SUB_CONTROL_VECTOR: [...; N] = unsafe { [...] };` in the owning file.
2. Update `cmdata.rs` concat list to include the new slice; remove the same entries from cmdata's own table; recompute the array size (or use the `const TOTAL` formula).
3. `cargo check ×3` — should pass with no errors.
4. `ninja ntoskrnl ×3` — should link cleanly.
5. `pytest tests/qemu/test_smoke.py --arch i386 --arch amd64 --skip-clean-build` — both 7/7.
6. Commit with the specific subsystem name.

If a step fails: inspect the build/smoke output, fix in place. The chop is reversible per step (just swap the entries back into cmdata.rs's local list).

### Why not "split CmGetSystemControlValues to walk slices directly"

Considered and rejected: it changes the function's API (`PCM_SYSTEM_CONTROL_VECTOR` → `&[&[CM_SYSTEM_CONTROL_VECTOR]]`), and the function is the only consumer. Net code change is similar but the API churn buys nothing — the consumer doesn't *care* about the slice structure.

### Why not "linker section concatenation"

Considered and rejected: PE custom-section ordering on llvm-mingw is fiddly and arch-dependent (tested in early ARM3 work — see `feedback_volatile_static.md` for the nearby gotcha about `Volatile<T>` on `#[no_mangle]` statics breaking section placement). Const concat is portable, deterministic, and zero-runtime-cost.

### Open questions for Phase B

- **Does `ex/init.rs:940` need any further change** once the concat scheme is in place? It currently does `addr_of_mut!(crate::config::cmdata::CmControlVector)` — that should keep working since the symbol still exists at the same place.
- **Should `OB_CONTROL_VECTOR` etc. be `pub static` (not `mut`)?** The slices themselves don't need to be mutated — they're just initializers. But the entries contain raw mutable pointers. Try `pub static` first; if compiler complains about pointer mutability, `pub static mut`. (It probably wants `static` — pointers in const context are fine even if they're `*mut`.)
