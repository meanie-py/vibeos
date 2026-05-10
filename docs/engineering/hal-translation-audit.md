# HAL C→Rust Translation Audit

Last updated 2026-05-08. **Status: pre-translation.** No Rust HAL code
exists yet; this audit defines the keep-set, the prune-set, and the
forward path so we can start with a known landscape.

Companion to [ntoskrnl-translation-audit.md](ntoskrnl-translation-audit.md).
The ntoskrnl translation is ~99.7% done; the HAL is the next chunky
target between us and a fully-Rust kernel-mode lower edge.

## TL;DR

- **Keep one HAL per arch** for our smoke-test boot path: `hal.dll`
  (i386 PIC), `hal.dll` (amd64 ACPI APIC UP), `hal.dll` (arm64 stubs).
  Drop the other 8 DLLs that ship today.
- **After pruning, total source under translation: ~28K LOC + 670 LOC
  asm**, of which ~13K is shared between i386 and amd64. Single
  `hal-rust` crate with cfg-gated modules is the natural shape.
- **ACPICA is header-only** — the HAL borrows ~30 ACPI table struct
  definitions from `drivers/bus/acpi/acpica/include/`. No link-time
  dep on the 120K-LOC vendor blob. **bindgen those headers, $0 API
  cost**, same pattern as `ntkrnl-sys`.
- **Build artifact stays a separate `hal.dll` for now.** Long-term,
  folding it into `ntoskrnl.exe` is plausible (Windows keeps them
  separate for OEM-pluggability we don't care about), but that's a
  cleanup, not a precondition.
- **i386 is the ordering choice.** Most stable arch in our tree, and
  the i386-only paths (legacy PCI, 8259 PIC, fastcall ABI, v86) are
  the hardest tests of the structure — after i386 lands, amd64 mostly
  reuses the `generic/` subset with cfg gates.

## Today's HAL inventory

The top-level dispatcher is `hal/CMakeLists.txt`:

```
i386, amd64 → halx86/   (shared C tree, multiple DLL flavours per arch)
arm64       → halarm64/ (single-DLL stub tree)
arm         → halarm/   (DEAD — never built by any arch in our matrix)
```

DLLs produced by the current build:

| Arch | DLLs built | Used by smoke? |
|---|---|---|
| i386 | `hal`, `halacpi`, `halaacpi`, `halapic`, `halmacpi`, `halxbox`, `halpc98` (7) | `hal.dll` only — livecd default boot has no `/HAL=` override |
| amd64 | `hal`, `halmp` (2) | `hal.dll` only (UP) |
| arm64 | `hal` (1) | `hal.dll` (the only option) |

Each i386 DLL is a different combination of shared object libraries:

| DLL | Components | Boards |
|---|---|---|
| `hal.dll` (i386) | generic + legacy + up + pic | Standard PC, 8259 PIC, no ACPI |
| `halacpi.dll` | generic + acpi + up + pic | ACPI on legacy PIC |
| `halaacpi.dll` | generic + acpi + up + apic | ACPI + APIC, UP |
| `halapic.dll` | generic + legacy + up + apic | Pre-ACPI APIC, UP |
| `halmacpi.dll` | generic + acpi + smp + apic | ACPI + APIC, multiproc |
| `halxbox.dll` | xbox + up | Original Microsoft Xbox |
| `halpc98.dll` | pc98 + up | NEC PC-98 (Japanese) |
| `hal.dll` (amd64) | generic + acpi + up + apic | x86_64 standard |
| `halmp.dll` (amd64) | generic + acpi + smp + apic | x86_64 multiproc |
| `hal.dll` (arm64) | halarm64/generic only | Stubs |

## Prune-set (no Rust work needed — just delete)

These are dead weight that complicates every code-walk and inflates
the i386 build by 6 useless DLLs. None are referenced by our smoke
boot.

| Path / target | LOC | Why drop |
|---|---:|---|
| `hal/halarm/` | several thousand | 32-bit ARM tree (omap3, versa boards). Top-level `hal/CMakeLists.txt` only builds it for `ARCH STREQUAL "arm"`, which we never set. Pure dead source. |
| `hal/halx86/xbox/` | 601 | Original Xbox console support. `add_hal(halxbox …)` only target. |
| `hal/halx86/pc98/` | 858 | NEC PC-98 architecture. `add_hal(halpc98 …)` only target. |
| `hal/halx86/smp/` | 476 | Multiprocessor primitives. Only consumed by `halmacpi.dll` (i386) and `halmp.dll` (amd64), which we don't smoke-test. |
| `hal/halx86/mp/` | 3,166 | Legacy MPS (Multi-Processor Specification, pre-ACPI multiproc). Only referenced via SMP DLL paths; commented-out target in current cmake (`#add_hal(halmp …)` for i386). |
| `add_hal(halacpi …)` | (no unique source) | i386 ACPI+PIC variant DLL — we'd never boot with `/HAL=halacpi.dll`. |
| `add_hal(halaacpi …)` | (no unique source) | i386 ACPI+APIC variant DLL. |
| `add_hal(halapic …)` | (no unique source) | i386 legacy+APIC variant DLL. |
| `add_hal(halmacpi …)` | (no unique source) | i386 SMP variant DLL. |
| `add_hal(halmp …)` (amd64) | (no unique source) | amd64 SMP variant DLL. |
| `hal/halx86/minihal/` | 65 | text-mode setup HAL stub. Not in livecd boot; if usetup needs something we'll re-add. |

Estimated direct deletion: **~5,200 LOC + 6 cmake DLL declarations + 1
dead arch tree.** Reversible (git). No runtime risk if smoke still
passes after.

## Keep-set landscape

After pruning, the HAL surface is three groups:

```
halarm64/generic/          (arm64 only — single stub file)
halx86/generic/             (shared by i386 + amd64)
halx86/{legacy,pic}/        (i386 only)
halx86/{acpi,apic}/         (amd64 only — but acpi is header-only on i386 today)
halx86/up.cmake             (uniprocessor spinlocks, both x86 arches)
halx86/include/             (shared headers, mostly typedefs)
```

### `halarm64/generic/` — 1,007 LOC (arm64 only)

Single file: `halstubs.c`. 32-line `hal.h` companion. **No assembly.**

Roughly **~80 `Hal*` exports**, all stubs returning `STATUS_NOT_IMPLEMENTED`,
`TRUE`, `0`, or empty. Includes:

- `HalInitSystem`, `HalInitializeProcessor`, `HalAllProcessorsStarted`
- `HalDisplayString`, `HalAcquireDisplayOwnership` (display)
- `HalGetBusData{ByOffset}`, `HalSetBusData{ByOffset}`,
  `HalGetInterruptVector`, `HalAssignSlotResources` (bus / IRQ stubs)
- `HalAllocateAdapterChannel`, `HalGetAdapter`, `HalAllocateCommonBuffer`,
  `HalFreeCommonBuffer`, `HalReadDmaCounter` (DMA stubs)
- `HalRequestSoftwareInterrupt`, `HalClearSoftwareInterrupt`,
  `HalEnable/DisableSystemInterrupt` (interrupt stubs)
- `HalQuery/SetRealTimeClock`, `HalSetTimeIncrement`,
  `HalCalibratePerformanceCounter` (timer stubs)
- `HalProcessorIdle`, `HalReturnToFirmware`, `HalMakeBeep` (misc)
- IRQL software-interrupt state machine (`HalpCurrentIrql`,
  `HalpDispatchSoftwareInterrupts`) — **the only non-trivial logic**

**Translation difficulty:** trivial. One Rust file, no FFI surprises,
no asm. Doable in a day. The IRQL state machine is the only piece
that warrants more than a literal port.

### `halx86/generic/` — 7,936 LOC (shared i386 + amd64)

The bulk of the keep-set. 26 files. Split:

| File | LOC | Role | i386 only? |
|---|---:|---|---|
| `dma.c` | 2,377 | ISA/EISA DMA (8237 controllers, map registers, scatter-gather) | shared |
| `usage.c` | 643 | Hardware-resource arbiter accounting | shared |
| `kdpci.c` | 578 | KD over PCI bridges | shared |
| `x86bios.c` | 518 | int10 graphics calls via fast486 emulator | shared |
| `bios.c` | 715 | Real-mode BIOS call infrastructure | **i386 only** |
| `cmos.c` | 319 | CMOS/RTC access (port 0x70/71) | shared |
| `timer.c` | 317 | Timer dispatch | shared |
| `misc.c` | 299 | grab-bag: HalReturnToFirmware, etc. | shared |
| `spinlock.c` | 256 | KSPIN_LOCK acquire/release (compiled twice for UP+MP) | shared |
| `memory.c` | 255 | HalpAllocPhysicalMemory, MapPhysicalMemory64 | shared |
| `halinit.c` | 175 | HalpGetParameters, init dispatch | shared |
| `sysinfo.c` | 124 | HalQuerySystemInformation handlers | shared |
| `portio.c` | 121 | inb/outb wrappers | **i386 only** |
| `profil.c` | 117 | Profile-interrupt handlers | shared |
| `beep.c` | 116 | PC speaker (port 0x61) | shared |
| `nmi.c` | 109 | NMI handler | shared |
| `reboot.c` | 110 | Triple-fault reboot | shared |
| `pic.c` | 84 | (generic component, separate from `pic/pic.c`) | shared |
| `drive.c` | 77 | HalpGetPartialGeometry stubs | shared |
| `up.c` | 70 | UP processor stubs | shared |
| `display.c` | 66 | HAL display fallback | shared |
| `clock.c` | 32 | (clock generic stub) | shared |
| `buildtype.c` | 20 | Build-string symbols | shared |
| `systimer.S` | 374 | System timer ISR (i386 asm, hand-coded interrupt entry) | **i386 only** |
| `v86.S` | 41 | v86-mode trampoline | **i386 only** |
| `trap.S` | 23 | Trap entry stub | **i386 only** |

**Translation difficulty notes:**
- `dma.c` is the largest single file in the keep-set. Map-register
  bookkeeping plus 8237 register pokes; algorithmic Rust port is
  straightforward but tedious.
- `x86bios.c` is the BIOS-call thunk to fast486 (a x86 emulator
  library used for VESA/int10). fast486 itself stays C — translating
  *it* is way out of scope. HAL's role is glue: set up registers,
  call `Fast486Initialize`/`Fast486Step`, copy results back.
- `bios.c` (i386) is the legacy real-mode BIOS path — only built on
  i386, only called from int10 callers. Pairs with `v86.S`.
- All asm files are i386-specific and small. The trap/v86 entry is
  load-bearing for boot; `systimer.S` is the timer ISR.

### `halx86/legacy/` — 4,790 LOC (i386 only)

| File | LOC | Role |
|---|---:|---|
| `bussupp.c` | 1,701 | PnP bus enumeration / arbitration glue |
| `bus/pcibus.c` | 1,307 | PCI configuration-space access (port 0xCF8/0xCFC + MMConfig) |
| `halpnpdd.c` | 931 | Root PnP device |
| `bus/bushndlr.c` | 473 | Bus handler infrastructure |
| `halpcat.c` | 134 | Standard PC-AT initialization |
| `bus/sysbus.c` | 135 | System bus handler |
| `bus/cmosbus.c` | 58 | CMOS bus handler |
| `bus/isabus.c` | 51 | ISA bus handler |
| `bus/pci_classes.ids` | 256 (data) | PCI class code → name strings |
| `bus/pci_vendors.ids` | 41,247 (data) | PCI vendor + device → name strings |

The two `.ids` files are processed at build time by `native-bin2c`
into `pci_classes.c`/`pci_vendors.c` and end up in the `INITDATA`
section as a string blob. In Rust they become a `static` byte array
(literally `include_bytes!`) — no translation needed for the data
itself.

`pcibus.c` is the meaty piece — PCI config-space I/O via legacy 0xCF8
mechanism. amd64 doesn't include this directly (it goes through
`acpi/`'s `pcibus.c` companion), so this is genuinely i386-only.

`bussupp.c` is the largest single file — bus enumeration logic that
talks to ntoskrnl's `arbiter` library (already linked into `hal.dll`
today via `target_link_libraries(hal arbiter …)`).

### `halx86/pic/` — 1,650 LOC (i386 only)

| File | LOC | Role |
|---|---:|---|
| `pic.c` | 1,300 | 8259 PIC programming, IRQL ↔ vector mapping, software-interrupt dispatch |
| `irql.c` | 163 | KIRQL acquire/lower/raise (i386 — fastcall ABI) |
| `halinit.c` | 56 | PIC-flavour init |
| `processor.c` | 53 | Processor wakeup stubs |
| `pic.S` | 78 | PIC ISR entry (asm) |

This is the core 8259 driver. Tightly coupled to i386 trap-frame
layout and fastcall ABI. The IRQL work is `KfRaiseIrql` /
`KfLowerIrql` and friends — same names that live in
`tools/verify_sacred_primitives.sh` already, so the calling
convention is well-understood from the ntoskrnl translation.

### `halx86/acpi/` — 2,686 LOC (built into amd64 hal.dll today)

| File | LOC | Role |
|---|---:|---|
| `halacpi.c` | 1,167 | RSDP/RSDT/XSDT walking, FADT cache, table lookup (`HalAcpiGetTable`) |
| `halpnpdd.c` | 920 | ACPI PnP root device |
| `busemul.c` | 337 | Bus-handler emulation for ACPI-only systems |
| `madt.c` | 262 | MADT (APIC table) parsing for `HalpParseApicTables` |

**This is where the ACPICA header dependency lives.** `acpi.cmake` adds
`drivers/bus/acpi/acpica/include` to the include path; the source
files `#include <acpi.h>` for `FADT`, `MADT`, `RSDT`, `XSDT` struct
layouts and signature constants. **No link-time dep on ACPICA library
code.** ACPICA's runtime (AML interpreter, namespace, ~120K LOC) is
used only by `drivers/bus/acpi/acpi.sys`, which is a separate kernel
driver — the HAL never calls `AcpiInitialize*`, `AcpiGetTable`, etc.
The HAL parses tables itself with `HalpAcpiGetTable` /
`HalpAcpiGetTableFromBios`.

**Bindgen plan (no API cost):** generate a Rust `acpi-tables-sys`
sub-crate (or fold into `ntkrnl-sys`) from
`drivers/bus/acpi/acpica/include/{actbl.h,actbl1.h,actbl2.h,actbl3.h}`.
These define ~30 spec-portable struct layouts (FADT, MADT subtables,
DEBUG_PORT_TABLE, SRAT, etc.) and signature constants. Same pattern as
the ntkrnl-sys bindgen layer. ACPICA upstream targets x86/x64/ARM/ARM64
from one source, so the headers are platform-portable — useful when
arm64 grows real ACPI support later (SBBR uses identical FADT/MADT
formats with GIC subtables instead of LAPIC).

### `halx86/apic/` — 2,580 LOC (amd64 only in our build)

| File | LOC | Role |
|---|---:|---|
| `apic.c` | 888 | LAPIC/IOAPIC programming, IRQL via TPR, software-interrupt model |
| `rtctimer.c` | 251 | RTC-based system timer |
| `tsc.c` | 178 | TSC calibration and read |
| `apictimer.c` | 159 | LAPIC timer |
| `halinit.c` | 78 | APIC-flavour init |
| `processor.c` | 53 | AP processor startup |
| `apicsmp.c` | 301 | (SMP path — would compile out for our UP build) |
| `apictrap.S` | 48 | APIC ISR entry (asm) |
| `tsccal.S` | 106 | TSC calibration loop (asm) |
| `apicp.h` / `tsc.h` | 364 | local headers |

This is the modern x86 interrupt controller — replaces `pic/`. amd64
always uses it; on i386 it would only matter if we ever wanted
APIC-flavour boot (we don't, smoke uses 8259 PIC).

`apicsmp.c` only matters for SMP, which we're dropping. The remainder
is UP LAPIC/IOAPIC plus timer.

### `halx86/include/` — 2,569 LOC (shared)

Headers, mostly typedefs and inline helpers. Ten files: `bus.h` (638),
`halp.h` (635), `haldma.h` (380), `halhw.h` (374), `mps.h` (200),
`ioapic.h` (97), `smp.h` (92), `hal.h` (82), `halacpi.h` (41),
`halirq.h` (30).

**Translation handling:** mostly bindgen targets. `mps.h` (MPS-table
layouts) is dead with the SMP prune. `ioapic.h` defines IOAPIC MMIO
register layouts — bindgen-able.

### `halx86/up.cmake` (uniprocessor object lib, both x86 arches)

Three files, all already counted under generic:
- `generic/buildtype.c` (20)
- `generic/spinlock.c` (256, compiled with `CONFIG_UP`)
- `generic/up.c` (70)

These will end up as one Rust module with the SMP-only branches
deleted (the original C uses `#ifdef CONFIG_SMP` to compile twice).

## Total keep-set sizes

| Arch | Components | LOC C | LOC asm | Total |
|---|---|---:|---:|---:|
| arm64 | halarm64/generic | 1,007 | 0 | 1,007 |
| i386 | halx86: generic + legacy + pic + up | 14,464 | 670 | 15,134 |
| amd64 | halx86: generic + acpi + apic + up | 12,964 | 528 | 13,492 |

Shared between i386 and amd64 (the entire `generic/` minus
i386-specific files): **~6,800 LOC C + 0 asm**. The i386-unique
work is `legacy + pic + i386-asm` = ~7,650 LOC. The amd64-unique
work is `acpi + apic + i386-asm-equivalents (none — no asm needed)`
= ~5,160 LOC.

## Spec / ABI surface

`hal/hal.spec` is 113 lines, **110 exports**:

- 84 `stdcall`
- 25 `fastcall`
- 1 `extern`

Per-arch breakdown of `-arch=` annotations:

| Cohort | Count |
|---|---:|
| Untagged (all arches) | 50 |
| `-arch=i386,arm,arm64` | 34 |
| `-arch=i386,x86_64` | 15 |
| `-arch=x86_64` (amd64-only) | 5 |
| `-arch=i386` | 3 |
| `-arch=arm,arm64` | 3 |
| `-arch=arm` | 1 (HalGetInterruptSource — dead with halarm dropped) |

Internal `Halp*` functions in `include/halp.h`: ~144 declarations.
These are not exports but must keep stable layouts/calling conventions
since C and Rust files coexist during the translation window.

## Build artifact decision

**Now: keep `hal.dll` separate**, same as today. FreeLDR loads it
before ntoskrnl in a sequence that's well-trodden across all three
arches; no reason to disturb that during the source-port.

**Long-term option: fold HAL into ntoskrnl.** Windows ships them
separate so OEMs can ship custom HALs (Xbox, custom firmware, etc.)
— a constraint we don't share. Folding would let us:
- Drop the `hal.def` / `hal.spec` import-lib machinery for ~110 calls.
- Drop the `add_importlibs(${_halname} ntoskrnl)` cycle (the HAL
  imports from ntoskrnl AND ntoskrnl imports from HAL — currently
  resolved by FreeLDR loading both before either has executed).
- Possibly merge `hal-rust` into `kernel-rust` and let LTO across
  the whole kernel-mode boundary.

But folding requires changes to FreeLDR's loader path (currently
`winldr.c:719` hardcodes `"hal.dll"`), and it's a refactor we can do
*after* the HAL is in Rust. **Note this as a future cleanup, don't
gate translation on it.**

For Rust output: a `hal-rust` cdylib (PE DLL) crate is the natural
shape, mirroring the kernel-mode `cdylib` setup that the cargo PoC
established for `ntoskrnl.exe`. Custom target spec, link the Rust
output against the existing `hal.def`. The thin `halstubs.c` / arch
shims become `cc-rs`-compiled C in `build.rs`, same pattern.

## Translation strategy

### What we don't pay for

- **ACPICA headers → bindgen.** ~30 struct layouts, regenerable. $0.
- **PCI ID tables → `include_bytes!`.** Already binary-baked at build
  time; just shift the include site to Rust. $0.
- **arm64 stubs (1,007 LOC) → manual port.** Trivial scope, no LLM
  needed. $0.

### What we do pay for (rustconv)

The remaining ~28K LOC of substantive C across `halx86/{generic,
legacy, pic, acpi, apic}/`. At Sonnet-batch rates (~$0.03 per file
from the ntoskrnl audit), this is ~30 files × $0.03 ≈ **~$1 in
translation API spend**, plus iteration overhead. Plus the sub-$0.30
incidental cost for review/touch-up passes.

For reference: the entire ntoskrnl translation was ~$8.70 per
iteration. The HAL is ~10% the size, ~10% the cost.

### Pipeline staging (mirrors ntoskrnl)

1. **Stage 0 — bindgen substrate.** Generate `hal-sys` (or extend
   `ntkrnl-sys`) with HAL-internal struct layouts (`halhw.h`,
   `halp.h`, `haldma.h`, `bus.h`) plus the ACPI table headers. Same
   `bindgen` invocation pattern as the existing kernel layer, three
   targets.
2. **Stage 1 — bulk transliteration via rustconv.** File-by-file
   translation, each output goes into `sdk/lib/rust/hal-rust/src/`.
   Order: arm64 first (trivially) → generic shared → pic (i386) →
   apic (amd64) → legacy (i386) → acpi (amd64).
3. **Stage 2 — probe migration.** Mostly N/A — the HAL doesn't
   `ProbeForRead`/`ProbeForWrite` user pointers (it's the lowest layer,
   under usermode entirely). Skip stage.
4. **Stage 3 — SEH idiomation.** Also mostly N/A. The HAL has near-zero
   SEH-protected code paths.
5. **Stage 4 — DLL boundary.** Once enough is in Rust to attempt a
   full DLL build, set up `hal-rust` cdylib, link it against the
   existing `hal.def`, swap the cmake `add_hal()` target to consume it.

## Forward path

In rough priority order (mirrors the structure of the ntoskrnl audit's
`### 1. Cargo PoC follow-ups`):

1. **Prune dead trees** — delete `halarm/`, `halx86/{xbox,pc98,smp,
   mp,minihal}/`, the 6 unused `add_hal()` declarations. Run smoke.
   This is reversible-cheap and isolates the keep-set so nothing
   else has to grep around the dead paths. Estimate: 1 session.
2. **Bindgen substrate** — `hal-sys` (or extend `ntkrnl-sys`).
   Generate ACPI table types from `acpica/include/`. 1 session.
3. **arm64 first port** — `halstubs.c` → Rust. Single file, no asm,
   exercises the `hal-rust` cdylib build pipeline end-to-end on the
   easiest target. Validates the DLL link / def-file integration.
   1–2 sessions.
4. **i386 generic + pic + legacy** — bulk rustconv. Biggest chunk by
   LOC, but most of it is mechanical transliteration. Order:
   `generic/` (shared subset) → `pic/` (interrupt path) →
   `legacy/` (PCI). 4–6 sessions.
5. **amd64 acpi + apic** — once i386 lands, amd64 mostly reuses
   `generic/` with cfg gates. The ACPI + APIC modules are net-new.
   2–3 sessions.
6. **Asm decisions per file.** `systimer.S`, `pic.S`, `apictrap.S`,
   `tsccal.S`, `v86.S`, `trap.S` — most are interrupt entry stubs
   that talk to the trap-frame layout. Likely worth keeping as
   `global_asm!` (or `cc-rs`-compiled) rather than translating to
   Rust intrinsics, same as the ntoskrnl asm strategy.
7. **Re-probe HAL static lib link** — current `hal.dll` links
   `libcntpr setjmp arbiter fast486 ${PSEH_LIB}`. Each is a sibling
   target; verify all five still link cleanly against the Rust
   output. (`fast486` and `arbiter` are C and stay C; `libcntpr`
   is the kernel-mode CRT subset; `setjmp` is the C stdlib subset.)
8. **Long-term: HAL-into-ntoskrnl merge.** Drop the DLL boundary,
   one `kernel-rust` blob, one LTO unit, drop `hal.spec`. Not
   blocking; revisit after Rust HAL is stable.

## Definition of done

`cargo build -p hal-rust --target <arch>` produces a working `hal.dll`
for all three arches. All three smoke tests pass on the cargo-driven
binary. `tools/scan_ud2.py --check` is clean. The cmake-driven `halx86`
/ `halarm64` directories can be deleted (or kept as a parallel CI
check).

## What this audit doesn't measure

- **Boot correctness.** Same caveat as the ntoskrnl audit. Smoke is
  the source of truth.
- **Driver dependency on HAL exports.** Anyone who imports from
  `hal.dll` (PnP managers, ACPI driver, video drivers) needs the
  ABI to stay stable through translation. The `hal.spec` is the
  contract; translation must preserve it byte-for-byte.
- **Real ACPI on arm64.** Today `halarm64` is stubs; some day we'll
  want SBBR-class ACPI parsing. The bindgen ACPI tables work for
  that future, but the rest of the arm64 HAL surface (GIC
  programming, generic timer) is net-new and not in scope of *this*
  translation pass.

## Open questions

1. **Should we land the prune-set as a single PR before any Rust
   work?** Pro: clean baseline, easier diff review. Con: large
   destructive change in flight while the build is otherwise stable.
   Recommendation: **yes, prune first.** Smoke gates the prune; if
   any of the dead targets turn out to be live, smoke catches it.
2. **bindgen ACPI tables: standalone `acpi-tables-sys` crate, or
   extend `ntkrnl-sys`?** ntkrnl-sys is already 80K-line bindgen output
   per arch; adding ~30 more types is noise. Probably extend.
3. **Do we want to keep amd64 SMP capability?** Today's smoke uses
   UP. Dropping `halmp.dll` removes SMP entirely from amd64. If
   we ever want to test multi-CPU boot, we'd have to undo. Cheap
   to keep `halmp.dll` building if the cost is just one extra
   target; expensive if it pulls in the whole `mp/` legacy tree.
   Recommendation: **drop now, re-add later if needed** — `mp/`
   is 3,166 LOC of pre-ACPI MPS legacy that we don't want to port.
4. **`fast486` dependency.** The HAL links it for x86bios.c real-mode
   emulation. We're not translating fast486 (it's a self-contained x86
   emulator library). Keep as-is; Rust HAL still links the C `fast486`
   library.
