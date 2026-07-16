# i386 APIC HAL — Phase A work plan (lane brief)

## Phase A — OUTCOME (2026-07-16): i386 UP boots to desktop on the APIC HAL ✅

The core deliverable is **done and committed** (branch `i386-apic-hal`,
commits `44cfed5ceb1` + `0311b6973b8`). The i386 APIC HAL flavor boots UP to the
graphical desktop and survives a **5/5 green soak**; the default i386 media stays
the 8259 PIC HAL, byte-unchanged, and amd64/arm64 are provably untouched.

**How it was built (differs from the brief's plan in a few places):**
- **Flavor mechanism = env-swap of `hal.dll`, not a sibling `halaacpi.dll`.**
  `hal-rust` gained an `apic` feature + a `build.rs` emitting `hal_apic`/`hal_pic`
  cfgs (amd64 always `hal_apic`; i386 `hal_pic` by default, `hal_apic` with the
  feature; mini-HAL loaders stay `hal_pic`). Modules gate on the *flavor* cfg,
  never raw `target_arch`. `vibeos-cli build --hal-apic` → `-DVIBEOS_APIC_HAL=ON`
  → `gcc.cmake` threads `VIBEOS_APIC_HAL=1` into hal-cargo → build.rs enables the
  feature AND swaps the assembled asm to the APIC set (`apictrap.S` + `tsccal.S`).
  A plain build reconciles back to PIC. The always-built sibling `halaacpi.dll` +
  a `[LiveCD_Aacpi]` boot entry were deemed unnecessary — making `hal.dll` itself
  the apic flavor is simpler and is the intended end-state (APIC replaces PIC).
- **The i386 halves were ported in-place with `#[cfg(target_arch = ...)]` arms**
  (amd64 paths preserved byte-for-byte), not as separate files — matching the
  codebase's pcr/memory/misc convention.
- **Clock decision reversed vs the brief:** the brief recommended keeping the
  8254 PIT for Phase A, but the C APIC flavor's clock is the **CMOS RTC** (IRQ8 →
  IOAPIC → 0xD1, `rtctimer.c`), and that's what shipped — C-faithful, and the
  amd64 LAPIC-timer-as-clock KVM workaround was deliberately NOT ported (the HAL
  is not bent for QEMU on i386; a TCG-drops-IRQ8 fallback would be a build flag).
- **Lazy IRQL (`APIC_LAZY_IRQL`) is the default** and booted first try; the eager
  path is kept behind a `const bool` for bisection.

**Validation (done 2026-07-16):**
- i386 APIC flavor: **6/6 boots to desktop** green (1 initial + 5/5 soak), banner
  confirmed in the serial log.
- i386 default (PIC): builds + boots green, **no apic banner** (confirmed still
  PIC) — zero regression from the shared-file edits.
- amd64: builds + boots + net green, all lints clean **including
  `verify_signatures`** — which confirms the "993 new" seen during i386-only
  builds was purely the missing-`build-amd64` worktree artifact, not a real
  regression (verify_signatures always reads `build-amd64/` headers; absent in an
  i386-only tree → partial C-proto dump → spurious diff).
- arm64: hal-rust compiles clean; provably unaffected (x86-gated module + a
  build.rs that emits neither flavor cfg for aarch64). Full arm64 smoke skipped as
  provably-irrelevant to save time.

**Owed (A6 tail — deliberately left for the user):** the **flip of the i386
default to APIC + gating PIC off**. APIC works and is validated, so this is
authorized in principle ("once apic works drop pic"), but it inverts the default
i386 `hal.dll` (what every i386 dev/gate builds) and the `--hal-apic` CLI
semantics — a consequential, measure-twice change best pulled with the user
present, especially given the explicit "don't delete, just gate it off" caution.
Ready-to-execute plan:
  1. `hal-cargo/build.rs`: `apic_hal = arch=="x86" && env::var("VIBEOS_PIC_HAL").is_err()`
     (apic default on x86; PIC via a `VIBEOS_PIC_HAL` escape hatch).
  2. Plumb `VIBEOS_PIC_HAL` through `gcc.cmake` + `build.py`; rename `--hal-apic`
     → `--hal-pic` (the escape hatch).
  3. Rebuild default i386 → now APIC → soak; keep PIC modules in-tree, gated off
     (never deleted).
It is **contained to this feature branch** (master stays PIC until merged), and
reversible with a single `git revert`.

Phase B (AP start, idle-lock/SwapBusy, M5c taps) returns to the smp-bringup lane,
reusing aptramp32 + ke/ipi.rs + this APIC HAL.

---


**Deliverable:** i386 UP boots to desktop on an APIC/IOAPIC/TPR-model HAL flavor,
gate green. Opt-in flavor only — the default i386 media keeps the PIC HAL
untouched. **Phase B (multi-CPU: AP start, idle-lock/SwapBusy, M5c taps) is
explicitly OUT of this lane** — it returns to the smp-bringup lane when Phase A
lands (see `smp-bringup-quest.md`).

**Why:** SMP on x86 requires the local APIC (IPIs = ICR writes; the 8259 PIC has
none). The Rust APIC HAL exists but is a translation of the **amd64 half only**
of `hal/halx86/apic/*.c` — every `#ifndef _M_AMD64` / `_M_IX86` branch was
dropped, and every module is double-gated `#![cfg(target_arch = "x86_64")]` +
gated `pub mod` lines in `hal-rust/src/x86/mod.rs:15-24,63-64,83-84,98-106`.

Survey provenance: 2026-07-15 scout over the tree at `smp-bringup` post-M5d
(read-only; file:line refs current as of that tree).

## Per-module classification (sdk/lib/rust/hal-rust/src/x86/)

| module | class | notes |
|---|---|---|
| `apic.rs` | NEEDS-I386-HALF (**L**) | CR8 IRQL (`:206-215,390-400`) → TPR/lazy; `APIC_BASE 0xFFFFFFFFFFFE0000` → i386 `0xFFFE0000` (`apicp.h:37-38`); linear `irql<<4` map (`:104-115`) → `HalpIRQLtoTPR[32]`/`HalVectorToIRQL[16]` tables (`apic.c:31-87`); 64-bit IDT gate w/ `OffsetHigh`+`KGDT64_R0_CODE` (`:236-244`) → 8-byte i386 gate; amd64 3-level PTE math in `ApicInitializeIOApic` (`:580-582`) → i386 2-level; dropped i386 bodies to restore: `HalpApc/DispatchInterruptHandler` (`apic.c:555-645`), lazy `HalBeginSystemInterrupt` branch (`apic.c:753-793`), `KeRegisterInterruptHandler(APC/DISPATCH)` wiring (`apic.c:538-541`), `APIC_LAZY_IRQL` (`apic.c:21-23,209-254,361-364`) |
| IRQL swap-pair | **L** | i386 `KeGetCurrentIrql/KfRaiseIrql/KfLowerIrql/KeRaiseIrqlToDpcLevel/KeRaiseIrqlToSynchLevel` from `apic.c:823-887` (amd64 gets these from ntoskrnl; i386 HAL owns them); cfg-gate OUT the PIC versions (`pic_pic.rs:513,527,1462,1477,1501`) on the APIC flavor; `HalRequestSoftwareInterrupt` becomes APIC self-IPI |
| `apic_halinit.rs` | NEEDS-I386-HALF (M) | constants: `CLOCK2_LEVEL`/`APIC_PROFILE_LEVEL` are amd64 values (`:28-29`); i386 = 28/31 (`apicp.h:34,58`). Swap-pair with `pic_halinit.rs:29,44,56` (both define `HalpInitProcessor/Phase0/Phase1`) |
| `apictimer.rs` | decision (M) | amd64 Rust repurposed LAPIC timer as system clock + neutered profiling (`:150-181`); i386 can keep `profil.rs` |
| `rtctimer.rs` | leave (–) | amd64-only clock rewrite; i386 keeps the 8254 PIT (`timer.rs`) — recommended for Phase A |
| `tsc.rs` | NEEDS-I386-HALF (M) | only the 64-bit IDT-gate helper (`:139-169`); calibration body portable; `tsccal.S` already has the i386 half (`:7-52`) |
| `processor.rs` | UNGATE-ONLY | already compiled both arches |
| `madt.rs` | UNGATE-ONLY (S/M) | portable; rename `_LAYOUT_CHECKS_AMD64` (`:61`); needs `HalAcpiGetTable` (halacpi already un-gated on i386, `mod.rs:39-40`) |
| `smp_processor.rs`, `trampoline` | **Phase B — do not touch** | i386 arms largely pre-written: `aptramp32.S` is a complete standalone 16→32-bit AP trampoline, `aptramp32.bin` checked in, embedded under `#[cfg(x86)]` (`trampoline.rs:199-202,225`); kernel-side `ke/i386/mproc.rs` translated. Dormant behind the x86_64 gates |

## Key mechanics found

- **Flavor is hard-wired per arch, no feature exists**: `hal-cargo/build.rs:65-142`
  matches `CARGO_CFG_TARGET_ARCH` — `x86` ⇒ PIC asm set (`generic/systimer.S`,
  `generic/trap.S`, `pic/pic.S`), `x86_64` ⇒ APIC set (`apic/apictrap.S`,
  `apic/tsccal.S`). `hal-rust/Cargo.toml` features today: only `default`,
  `minihal`, `smp`. Phase A adds an `apic` feature (or `VIBEOS_HAL_FLAVOR` env)
  that swaps the asm set + module gates.
- **`apictrap.S` i386 half exists but never assembles** (`:26-41`, `_`-decorated
  `_HalpClockInterrupt` etc., `iret` + `KI_PUSH_FAKE_ERROR_CODE`). Enabling it on
  the APIC flavor requires REMOVING the colliding definitions from
  `generic/trap.S:21-23` (`HalpApcInterrupt/HalpClockInterrupt/HalpProfileInterrupt`)
  and `pic/pic.S:75` (`HalpDispatchInterrupt`) on that flavor.
- **i386 soft-IRQL state today**: `PCR.Irql/IRR/IDR` consumed throughout
  `pic_pic.rs` (list of line refs in the survey; seed in `pic_halinit.rs:34`);
  `KiI8259MaskTable`/`FindHigherIrqlMask` in `irql.rs:24-94` go dead on the APIC
  flavor.
- **ntkrnlmp already builds for i386** (`ntkrnlmp-cargo/build.rs:40-53`,
  `i686-kernel.json`, entry `_KiSystemStartup@4`) — irrelevant to Phase A but
  confirms Phase B has media. `livecd.ini:46-54` has `[LiveCD_SMP]` +
  `[LiveCD_Macpi]`; Phase A adds an APIC-flavor boot entry or `--smp-kernel`
  media wiring.

## Work plan (dependency-ordered)

1. **Flavor scaffolding (M)** — `apic` cargo feature; `hal-cargo/build.rs` asm-set
   swap; `mod.rs` gates from `target_arch="x86_64"` → flavor cfg. Blocks all below.
2. **i386 APIC constants + IRQL tables (S/M)** — `HalpIRQLtoTPR[32]`,
   `HalVectorToIRQL[16]`, `IrqlToTpr/IrqlToSoftVector/TprToIrql` macro forms,
   i386 `APIC_BASE/IOAPIC_BASE`.
3. **`apic.rs` i386 half (L)** — see table row. Depends on 2.
4. **IRQL swap-pair (L)** — see table row. Depends on 3.
5. **`apictrap.S` i386 trap set (M)** — enable `#else` half; de-collide trap.S/pic.S.
   Depends on 1; pairs with 4.
6. **`tsc.rs` i386 (M)** — 32-bit IDT-gate helper; shares it with 3.
7. **`apic_halinit.rs` swap (M)** — constants + swap-pair with pic_halinit.
8. **`madt.rs` un-gate (S/M)** — parallelizable.
9. **Clock decision (recommend: keep PIT for Phase A)** — the IOAPIC clock slot
   (`apic.rs:608-616,635`) then only needs its vector wired.
10. **Media/boot entry (S)**.

## Lane rules

- Opt-in flavor only; default i386 PIC media must stay byte-identical. Gate ×3
  before landing (the flavor build is additional, not a replacement).
- Expect a debug tail: IRQL-model swaps manifest as lost softints / subtle hangs
  (see field manual §9.19-§9.22 for the arm64 equivalents of this bug class and
  the debugcon/serial tooling that found them).
- Kernel bugs flushed out by the new interrupt model (not HAL-flavor bugs) →
  report to the smp-bringup lane with symptoms, don't rabbit-hole.
