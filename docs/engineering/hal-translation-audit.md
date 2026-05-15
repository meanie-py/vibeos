# HAL C→Rust Translation Audit

Last updated 2026-05-14. Companion to [`ntoskrnl-translation-audit.md`](ntoskrnl-translation-audit.md).

## Status — 2026-05-14

**~39 of ~45 substantive C files are now Rust on i386 + amd64.** mini_hal Rust-ification complete (Phase 2 closed `80ba39c6`). All 3 arches smoke green to a painted taskbar. Remaining substantive work: `acpi/halacpi.c` (1167 LOC) and `acpi/halpnpdd.c` (920 LOC). Everything in `generic/`, `pic/`, `legacy/`, and the entire `apic/` subtree is now Rust.

### Strategy in force (set 2026-05-10)

1. **arm64 HAL stays as C stubs** — `hal/halarm64/generic/halstubs.c` is 1,007 LOC of stubs we wrote ourselves; not a real HAL. Translating stubs to Rust stubs delivers no functional improvement. Real arm64 HAL (GIC + Generic Timer + PSCI + SBBR ACPI) is a separate scoped project for some future date.
2. **All Rust HAL work targets x86 (i386 + amd64).** That's where the ~28K LOC of real hardware drivers live.
3. **Crate is single `hal-rust` cdylib-flavored staticlib** producing `libhal_rust.a` (and `libhal_rust_mini.a` via `--features minihal`), linked into the cmake-built `hal.dll` modules. Cohabitation, not full DLL takeover.
4. **Portable submodules + arch backends** — design intent preserved. In practice, current Rust is mostly under `src/x86/` with cfg-gated arch splits inside files; haven't needed to factor out portable trait surface yet.

## Build paths

```
sdk/lib/rust/hal-rust/
├── Cargo.toml                  # staticlib, [features] minihal = []
└── src/x86/                    # 40 .rs files, ~15K LOC
```

CMake links `libhal_rust.a` into each `add_hal()` target and `libhal_rust_mini.a` into the freeldr-side mini_hal consumers. C files retired from the build are commented out in the per-flavor `.cmake` files (`generic.cmake`, `legacy.cmake`, `pic.cmake`, `acpi.cmake`, `apic.cmake`) with a `# replaced by Rust (path)` line so the mapping is greppable.

## Per-file translation status

`✓` = swapped to Rust in the active build. `C` = still compiled as C. `S` = stays C by design (see footnotes).

### `halx86/generic/` — shared i386 + amd64

| File | Status | Rust file |
|---|---|---|
| beep.c | ✓ | beep.rs |
| bios.c | S¹ | — |
| buildtype.c | ✓ | buildtype.rs |
| clock.c | ✓ | clock.rs |
| cmos.c | ✓ | cmos.rs |
| display.c | ✓ | display.rs |
| dma.c | ✓ | dma.rs |
| drive.c | ✓ | drive.rs |
| halinit.c | ✓ | halinit.rs |
| kdpci.c | ✓ | kdpci.rs |
| memory.c | ✓ | memory.rs |
| misc.c | ✓ | misc.rs |
| nmi.c | ✓ | nmi.rs |
| pic.c | ✓ | pic.rs |
| portio.c (i386) | ✓ | portio.rs |
| profil.c | ✓ | profil.rs |
| reboot.c | ✓ | reboot.rs |
| spinlock.c | ✓ | spinlock.rs |
| sysinfo.c | ✓ | sysinfo.rs |
| timer.c | ✓ | timer.rs |
| up.c | ✓ | up.rs |
| usage.c | ✓ | usage.rs |
| x86bios.c | S¹ | — |
| systimer.S, v86.S, trap.S | S² | — |

¹ Stays C by design — `bios.c` + `x86bios.c` use `setjmp`/`longjmp` for v86 BIOS trampolines (i386 int10 graphics path). Rust has no story for catching a stack-rewinding `longjmp` from another asm-entered stack frame. Path is i386-only, never hot, isolated; a thin extern surface from Rust is the correct cut.

² Asm files stay as `.S` in cmake until/unless converted to `global_asm!`. No translation buying anything today.

### `halx86/legacy/` — i386 only

| File | Status | Rust file |
|---|---|---|
| bus/bushndlr.c | ✓ | bushndlr.rs |
| bus/cmosbus.c | ✓ | cmosbus.rs |
| bus/isabus.c | ✓ | isabus.rs |
| bus/pcibus.c | ✓ | pcibus.rs |
| bus/sysbus.c | ✓ | sysbus.rs |
| bussupp.c | ✓ | bussupp.rs |
| halpcat.c | ✓ | halpcat.rs |
| halpnpdd.c | ✓ | legacy_halpnpdd.rs |

Whole subtree done.

### `halx86/pic/` — i386 only

| File | Status | Rust file |
|---|---|---|
| halinit.c | ✓ | pic_halinit.rs |
| irql.c | ✓ | irql.rs |
| pic.c | ✓ | pic_pic.rs |
| processor.c | ✓ | processor.rs (shared with apic/) |
| pic.S | S² | — |

Whole subtree done.

### `halx86/acpi/` — amd64 (header-only on i386)

| File | Status | Rust file |
|---|---|---|
| busemul.c | ✓ | busemul.rs |
| halacpi.c | C | (1,167 LOC — the chunky ACPI table walker, biggest remaining piece) |
| halpnpdd.c | C | (920 LOC — ACPI PnP root device, planned) |
| madt.c | ✓ (amd64) | madt.rs |

ACPI table layouts are bindgen-generated (`hal_ddk` ACPI types); no link-time dep on ACPICA library code (the AML interpreter lives in `drivers/bus/acpi/acpi.sys`).

### `halx86/apic/` — amd64

| File | Status | Rust file |
|---|---|---|
| apic.c | ✓ | apic.rs |
| apictimer.c | ✓ | apictimer.rs |
| halinit.c | ✓ | apic_halinit.rs |
| processor.c | ✓ | processor.rs |
| rtctimer.c | ✓ | rtctimer.rs |
| tsc.c | ✓ | tsc.rs |
| apictrap.S, tsccal.S | S² | — |

### `halx86/minihal/`

| File | Status | Notes |
|---|---|---|
| halinit.c | C | The only mini_hal C file kept. Phase 2 retired the other 17 (`80ba39c6`). |

## Stage 0 — bindgen substrate (DONE)

`ntkrnl-sys::hal_ddk` provides the HAL-private bindgen layer for all 3 arches (only x86 is consumed today). Includes:

- HAL internal headers (`halhw.h`, `halp.h`, `haldma.h`, `bus.h`, `ioapic.h`)
- ACPI table layouts (`drivers/bus/acpi/acpica/include/{actbl,actbl1,actbl2,actbl3}.h`) — ~30 spec-portable struct definitions
- 41 kernel-data exports wrapped via raw-dylib (`postprocess_bindgen.py --hal`, commit `b811836f`) so HAL Rust can reference `KdDebuggerNotPresent`, `KeServiceDescriptorTable`, etc., across the DLL boundary

## Stages 1-2 — translation pipeline (mostly DONE)

- Stage 1 pipeline validated end-to-end via `HalMakeBeep` (`2d6c375a`).
- Stage 2 bulk translation: 36 of ~45 substantive C files swapped. Cohabitation pattern keeps each translation independently revertible until proven via smoke.
- Stage 3 (probe migration) — N/A. HAL doesn't probe user pointers (it's the lowest layer).
- Stage 4 (SEH idiomation) — N/A. Verified zero `_SEH2_*` / `__try` / `__except` usage anywhere under `hal/`.

## Stage 3 — DLL boundary (current state: incremental cohabitation)

The original plan was a hard cutover: `hal-rust` becomes a cdylib and `add_hal()` consumes it directly. In practice we've taken the **incremental cohabitation** path:

- `libhal_rust.a` is a staticlib that ntoskrnl HAL flavors link in.
- C files retired one at a time in the per-flavor `.cmake`; Rust takes over without changing the DLL build target.
- `--allow-multiple-definition` linker flag allows side-by-side C+Rust during the swap-in phase; only one definition wins (lld first-match), and the comment-out commits cleanly when the smoke confirms.

This trades the "clean cargo-only HAL link" milestone for ability to swap files individually with smoke as the gate. Cleaner in practice; each swap is a small reversible PR.

## Pre-translation prune (DONE 2026-05-10, commit `457efb25`)

Deleted these dead trees and DLL flavors:

- `hal/halarm/` (32-bit ARM tree — ARCH=arm not in our matrix)
- `hal/halx86/{xbox,pc98,smp,mp}/` and the corresponding cmake files
- `hal/halx86/apic/apicsmp.c`
- 6 i386 `add_hal()` declarations + 1 amd64 (`halacpi`, `halaacpi`, `halapic`, `halmacpi`, `halmp`, `halxbox`, `halpc98`)
- 7 baseaddress entries

`minihal/` was kept (FreeLDR links it on every arch). Smoke green after.

Subsequent dead-platform purge (`fe69a065`, `f6c19a05`, `c0407909` — 2026-05-13/14): xbox/pc98/arm32/SAC/arm64ec source trees removed top-to-bottom; xdk template-arch dispatch trimmed to IX86/AMD64/ARM64 only.

## Spec / ABI surface

`hal/hal.spec` is 113 lines, **110 exports**:

- 84 `stdcall`, 25 `fastcall`, 1 `extern`

Per-arch breakdown of `-arch=` annotations:

| Cohort | Count |
|---|---:|
| Untagged (all arches) | 50 |
| `-arch=i386,arm,arm64` | 34 |
| `-arch=i386,x86_64` | 15 |
| `-arch=x86_64` | 5 |
| `-arch=i386` | 3 |
| `-arch=arm,arm64` | 3 |

Internal `Halp*` functions in `include/halp.h`: ~144 declarations. The cohabitation pattern requires these signatures to stay stable through the translation window — both C and Rust have to agree exactly on calling convention, return type, and parameter layout. Per-arch ABI mismatches caused real bugs (memory `feedback_hal_extern_system_vs_c.md`); current best practice is to declare HAL Rust functions with `extern "system"` for NTAPI symbols, `extern "fastcall"` for FASTCALL, and bare `extern "C"` for pure cdecl, matching the C header verbatim.

## Remaining substantive work

In rough order of effort:

1. **`acpi/halacpi.c` (1,167 LOC)** — RSDP/RSDT/XSDT walking + FADT cache + `HalAcpiGetTable`. Largest remaining single file; the bindgen ACPI types are ready.
2. **`acpi/halpnpdd.c` (920 LOC)** — ACPI PnP root device. Pairs with #1.
3. **`minihal/halinit.c`** — keep as C until/unless the mini_hal entry-points need restructuring (existing Rust `pic_halinit.rs` etc. provide real impls — minihal stub semantics need a feature flag pass first).

Total remaining: ~2,100 LOC of substantive C.

`bios.c`, `x86bios.c`, `systimer.S`, `v86.S`, `trap.S`, `pic.S`, `apictrap.S`, `tsccal.S` — explicitly stay-as-C/asm per design.

## What this audit doesn't measure

- **Boot correctness.** Smoke is the source of truth.
- **Driver dependency on HAL exports.** Anyone importing from `hal.dll` (PnP managers, ACPI driver, video drivers) needs the ABI to stay stable. `hal.spec` is the contract.
- **Real ACPI on arm64.** The bindgen ACPI tables work for that future, but the rest of the arm64 HAL surface (GIC programming, generic timer) is net-new and not in scope.

## Definition of done

- All `halx86/` substantive C files Rust except the explicit S-class (bios, x86bios, asm).
- i386 + amd64 smoke pass on the cohabitation build.
- `tools/scan_ud2.py --check` clean on x86.
- Long-term: replace cohabitation with cargo-built cdylib, drop `--allow-multiple-definition`, drop `hal.def`/`hal.spec` import-lib machinery if HAL folds into ntoskrnl.

## Open questions

- **Cohabitation vs. full cargo cdylib.** Cohabitation works today and avoids a flag-day cutover. The cdylib path is still possible later but no longer urgent.
- **HAL-into-ntoskrnl merge.** Drops the DLL boundary entirely (one `kernel-rust` blob, one LTO unit). Not blocking; revisit after Rust HAL is stable. FreeLDR's `winldr.c:719` hardcodes `"hal.dll"`; merge requires loader path changes.
- **`bios.c` setjmp/longjmp story.** Recommendation stands: keep as C, call from Rust through extern surface. Forcing this through Rust buys nothing.
- **Portable-submodule trait extraction.** Original plan had `irql/`, `acpi/`, `bus/`, `arbiter/` as portable submodules taking arch backends as traits. Hasn't been needed yet — the x86 arch split inside files is sufficient. Revisit when (if) arm64 grows a real Rust HAL.
