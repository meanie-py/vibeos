# cmdata.c Chopping Plan (Phase B)

**Status:** Phase A landed (verbatim translation `cmdata.c` → `cmdata.rs`). Phase B is the actual chop — moving `CmControlVector[]` entries into the subsystem files that own the variables they point at.

**Why this exists:** This file captures the chopping plan that previously lived only in chat history. The original cmalloc/cmdelay/cmlazy/cmkcbncb chops applied the same pattern when those files were translated; cmdata is the last and biggest holdout.

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

### Migration order — risk-ranked

Smallest, lowest-risk subsystems first so the mechanism is proven before bigger chunks.

| Order | Subsystem | Entries | Owning files | Notes |
|------:|-----------|--------:|--------------|-------|
| 1 | **ob** | 5 | `ob/obinit.rs` (3), `ob/devicemap.rs` (1), `ob/obname.rs` (3 — buffer, length, case-insensitive) | Smallest non-trivial chop. Validates the const concat scheme. |
| 2 | **kd64 debug filters** | 156 | `kd64/kddata.rs` (already owns `Kd_*_Mask` statics) | Big mechanical chunk. Builds a `KD_FILTER_CONTROL_VECTOR` slice next to the masks. After this, `cmdata.rs` shrinks dramatically. |
| 3 | **mm/ARM3** | ~28 | `ARM3/{mminit,helpers,pfnlist,sysldr,mdlsup,drvmgmt}.rs` + `mm/{mminit,pagefile}.rs` | Biggest by file count. Probably one slice per file or one per directory. |
| 4 | **ex** | 1 | `ex/resource.rs` (`ExpResourceTimeoutCount`) | Trivial single-entry chop. |
| 5 | **ps** | 1 | `ps/process.rs` (`PsRawPrioritySeparation`) | Trivial. |
| 6 | **iomgr** | 1 | `iomgr/iomgr.rs` (`IopAutoReboot`) | Trivial. |
| 7 | **cmlazy/cmdelay** | 3 | `config/{cmlazy,cmdelay}.rs` (already own the variables) | Trivial. |
| 8 | **kd64 KdpContext** | 1 | `kd64/kddata.rs` (`KdpContext.KdpDefaultRetries`) | Field-of-struct chop; should fold in with #2. |

What's *left* in `cmdata.rs` after all 8 are done:
- The simple cmdata-owned globals (`DummyData`, `CmNtGlobalFlag`, language IDs, suite buffer, `CmControlHive`, push locks, `CmpVolatileBoot`, `CmSelfHeal`, etc.).
- `CmpMachineHiveList`, `CmClassName`, `CmTypeName`, `CmpMultifunctionTypes` (these are inherently cmdata's because they describe registry structure, not subsystem-specific data).
- `CMDATA_LOCAL_CONTROL_VECTOR` — entries that reference cmdata-owned globals (`DummyData` placeholders, language IDs, suite buffer entries, `CmpVolatileBoot`, `CmSelfHeal` settings, `CmpLazyFlush*`, `CmpDelayedCloseSize`, the WMI/CrashControl trailing entries — most of which use `&DummyData`).
- The const concat assembler.

Estimated final cmdata.rs size: ~250 LOC (down from 994 LOC of C / ~750 LOC of Phase-A Rust).

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
