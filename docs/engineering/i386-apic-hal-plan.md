# i386 APIC HAL — Phase A work plan (lane brief)

## Phase A — LANDED (2026-07-16): APIC is now the i386 DEFAULT HAL ✅

The core deliverable **and** the A6 default-flip are **done** (branch
`i386-apic-hal`). The i386 APIC HAL (LAPIC/IOAPIC/TPR, lazy-IRQL, RTC clock) is
now the **default** `hal.dll` on i386 — a plain `vibeos-cli build --arch i386`
produces APIC media that boots UP to the graphical desktop. The legacy 8259 PIC
HAL is preserved in-tree, gated off, and still reachable via the `--hal-pic`
(`VIBEOS_PIC_HAL`) escape hatch — **never deleted**, so the whole change is one
`git revert` away. amd64/arm64 are provably untouched.

**How it was built (differs from the brief's plan in a few places):**
- **Flavor mechanism = env-swap of `hal.dll`, not a sibling `halaacpi.dll`.**
  `hal-rust` gained an `apic` feature + a `build.rs` emitting `hal_apic`/`hal_pic`
  cfgs (amd64 always `hal_apic`; i386 `hal_apic` by default now, `hal_pic` only
  via the `--hal-pic` escape hatch; mini-HAL loaders stay `hal_pic`). Modules
  gate on the *flavor* cfg, never raw `target_arch`. Post-flip a plain
  `vibeos-cli build --arch i386` is the APIC HAL: hal-cargo/build.rs treats the
  ABSENCE of `VIBEOS_PIC_HAL` as "build APIC" → enables the `apic` feature AND
  assembles the APIC asm set (`apictrap.S` + `tsccal.S`). The `--hal-pic` escape
  hatch → `-DVIBEOS_PIC_HAL=ON` → `gcc.cmake` threads `VIBEOS_PIC_HAL=1` into
  hal-cargo → the legacy 8259 PIC set (`generic/systimer.S` + `generic/trap.S` +
  `pic/pic.S`). A plain build reconciles back to APIC. The always-built sibling `halaacpi.dll` +
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

**Validation (2026-07-16):**
- **Audit** — the i386 halves are a *faithful* port of the C: `HalpIRQLtoTPR[32]`
  / `HalVectorToIRQL[16]` byte-identical to `apic.c`; the lazy-IRQL
  get/set/lower model, `HalBeginSystemInterrupt`'s lazy re-queue, and every
  handler (APC / DISPATCH / clock / profile) match `apic.c` / `rtctimer.c` /
  `apictimer.c`; all ABIs verified against `ntoskrnl.spec` (`KeUpdateSystemTime`
  fastcall, `KiEoiHelper` fastcall-noreturn, etc.). No correctness bugs found.
- **i386 APIC opt-in (pre-flip)** — cold-built, **6/6 boots to desktop** (1 +
  5/5 soak), banner confirmed at runtime.
- **i386 DEFAULT (post-flip)** — a plain `--clean` build produces an APIC
  `hal.dll` (banner string present) and boots **3/3 to desktop**, banner
  confirmed at runtime. All 40+ scanners green (`scan_ud2`, `sacred_primitives`,
  `verify_signatures`, `scan_irql_balance`, `scan_cc_mismatch`, …).
- **Legacy PIC escape hatch (`--hal-pic`)** — a `--clean` build produces a PIC
  `hal.dll` (no banner string) and boots green with no runtime banner: zero
  regression, the escape hatch is intact.
- **amd64 / arm64** — provably unaffected by the flip (amd64's `hal_apic` comes
  from the unchanged `x86_64` match arm; `VIBEOS_PIC_HAL` is never set for them).
  Confirmed green by the landing gate ×3.

**A6 default-flip — DONE** (was "owed, left for the user"; authorized by "once
apic works drop pic", pulled after the hardening pass above). APIC is now the
i386 default; the legacy PIC is the `--hal-pic` / `VIBEOS_PIC_HAL` escape hatch.
Six files:
  1. `hal-cargo/build.rs` — `apic_hal = arch=="x86" && env::var("VIBEOS_PIC_HAL").is_err()`
     (absence of the env ⇒ build APIC).
  2. `hal-rust/build.rs` — doc/comment only (the cfg logic keys off the `apic`
     feature, which hal-cargo now enables by default; unchanged).
  3. `gcc.cmake` + `hal/halx86/CMakeLists.txt` — thread `VIBEOS_PIC_HAL=1` into
     hal-cargo only when requested; invert the i386 flavor-asm dep-track default
     to the APIC set.
  4. `tests/qemu/cli/build.py` + `__main__.py` — reconcile key `VIBEOS_APIC_HAL`
     → `VIBEOS_PIC_HAL`; rename `--hal-apic` → `--hal-pic`.
The PIC modules stay in-tree, **gated off, never deleted** (translation-preserve
rule) — the flip is one `git revert` away. Landed on `master` via the usual
merge after a green FULL gate ×3.

Phase B (AP start, idle-lock/SwapBusy, M5c taps) returns to the smp-bringup lane,
reusing aptramp32 + ke/ipi.rs + this APIC HAL.

---


**Deliverable:** i386 UP boots to desktop on an APIC/IOAPIC/TPR-model HAL flavor,
gate green. ✅ **DONE + made the default** (A6 flip landed; legacy PIC preserved
behind `--hal-pic`). **Phase B (multi-CPU: AP start, idle-lock/SwapBusy, M5c taps) is
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

- ~~Opt-in flavor only; default i386 PIC media must stay byte-identical.~~
  **Superseded by the A6 flip (2026-07-16): APIC is now the default; the legacy
  PIC is the `--hal-pic` escape hatch.** Gate ×3 before landing.
- Expect a debug tail: IRQL-model swaps manifest as lost softints / subtle hangs
  (see field manual §9.19-§9.22 for the arm64 equivalents of this bug class and
  the debugcon/serial tooling that found them).
- Kernel bugs flushed out by the new interrupt model (not HAL-flavor bugs) →
  report to the smp-bringup lane with symptoms, don't rabbit-hole.
