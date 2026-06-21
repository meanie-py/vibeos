# ARM64 periodic clock interrupt — done; and the `arm64_lpcexitthread_race` it exposes

**Lane:** arm64-love · **Date:** 2026-06-20 · **Branch:** `arm64-love` (base c-r-p `a812475681a`)

## 1. The clock fix (DONE + proven correct)

The arm64 port never advanced the system clock — `KUSER_SHARED_DATA.InterruptTime`
frozen at 0 because nothing fired a periodic clock interrupt → every relative timer /
`KeDelayExecutionThread` / DPC-timer broken. **Fixed by wiring the ARM Generic (EL1
virtual) timer through the GICv2 as the NT periodic clock.**

Files:
- **`sdk/lib/rust/hal-rust/src/arm64/clock.rs`** (new) — `HalInitializeClockArm64(gicd, gicc)`:
  GICv2 init (GICD `0x08000000` / GICC `0x08010000` on QEMU virt; enable distributor +
  CPU interface, set-enable + priority for the **virtual-timer PPI 27**, PMR `0xF0`) and
  programs CNTV (`CNTV_TVAL_EL0 = CNTFRQ_EL0/100` = 10ms, `CNTV_CTL_EL0=1`).
  `HalClockInterruptArm64(tf)`: ISR — read `GICC_IAR`; if INTID 27, re-arm TVAL,
  `KeUpdateSystemTime(tf, 100000, old_irql)`, then `GICC_EOIR`.
- **`hal/hal.spec`** — exports `HalInitializeClockArm64` + `HalClockInterruptArm64` (`-arch=arm64`).
- **`ntoskrnl-rust/src/ke/arm64/trap.rs`** — `KiIrqHandler` → `HalClockInterruptArm64(tf)`
  (both EL1 `KiIrqEntry` and EL0 `KiEL0IrqEntry` vectors route here).
- **`ntoskrnl-rust/src/ex/init.rs`** — after the Phase-1 UART remap, `MmMapIoSpace` the
  GICD/GICC and call `HalInitializeClockArm64` (before the first process switch; the ISR
  runs in arbitrary process context and the early PXE[0] identity map is gone after SMSS).

**Note — the live arm64 trap/init are the RUST cohabs** (`ntoskrnl-rust/src/ke/arm64/trap.rs`,
`ntoskrnl-rust/src/ex/init.rs`), NOT `ntoskrnl/ke/arm64/trap.c` / `ntoskrnl/ex/init.c`
(those `.c` are cohab'd OUT — 0 build statements). Editing the `.c` is the dormant-translation
trap (commit `94d3c4511f7`).

Timer-expiration follow-through (waking a delayed thread) is handled by the existing arm64
`KiIdleLoop`, which already polls `Prcb->TimerRequest` → `KiRetireDpcList`; so once
InterruptTime advances and `KiCheckForTimerExpiration` sets `TimerRequest`, the idle loop
retires it. No new dispatch-level soft-int handler needed.

**Oracle verified:** direct-booted worktree media (QEMU on the worktree's `boot.img`;
`vibeos-cli` boots *main's* media due to `.venv`→main shadowing, so bypass it). Serial:
`[CLKVERIFY] irqs=0x64 InterruptTime=0x989680` → after 100 ticks InterruptTime =
10,000,000 (100ns units) = **exactly 1.000s**. KeUpdateSystemTime fires at 100Hz, advances
InterruptTime at the configured rate. Clean c-r-p (no-clock) on clean media = 2/2 green.

## 2. The blocker the working clock EXPOSES — `arm64_lpcexitthread_race`

Landing the clock turns a **pre-existing, intermittent** arm64 boot wedge (confirmed by
control-plane: mainline-no-clock wedges ~1/3 at the same spot; tracked as KNOWN_FLAKE
`arm64_lpcexitthread_race`) into a **deterministic** one — the timer path that triggers it
now fires every boot. The clock is NOT the cause; it's the exposer. **The clock must not
land until this is fixed** (else arm64 goes from mostly-green to always-red).

### Root cause (mechanism, fully traced)

1. With the clock, a **timer-driven cached read** (read-ahead / lazy path) runs on a System
   worker thread (seen at `PspExitThread → LpcExitThread`, but that's incidental — it's the
   read on that thread that faults).
2. `CcCopyRead`/page-in → **`cc/view.c:983 CcRosEnsureVacbResident` → `MmMakeDataSectionResident`**
   pages the file in → the FS paging read returns `STATUS_FILE_CORRUPT_ERROR`.
3. **fastfat raises it: `read.rs:757-758`** —
   `if FileSize > (*FcbOrDcb).Header.AllocationSize.u.LowPart { FatPopUpFileCorrupt; FatRaiseStatus(STATUS_FILE_CORRUPT_ERROR) }`
   (raiser confirmed in fastfat.sys: backtrace caller `0xFFFFF8004237C2CC` is *past
   ntoskrnl's end* RVA ~0x581f80 → next-loaded module).
4. `FatRaiseStatus → ExRaiseStatus → RtlRaiseStatus`, but under **USE_DUMMY_PSEH the FS's
   `_SEH2_EXCEPT` catch-filter is dead code**, so it propagates unhandled →
   `KiDispatchException` (PreviousMode=Kernel, `TrapFrame=NULL` software-raise) →
   `KeBugCheckEx(0x1E /*KMODE_EXCEPTION_NOT_HANDLED*/, 0xC0000102 /*STATUS_FILE_CORRUPT_ERROR*/, 0, 0)`.

### Why the raise is SPURIOUS (the actual bug to fix — fastfat-rust)

`FileSize` at read.rs:758 is `ULONG = Header.FileSize.u.LowPart` (read.rs:579/665) — already
the low dword, no high-dword garbage. So `FileSize.LowPart > AllocationSize.LowPart` true
means **`AllocationSize` is wrong (0 or too small)** for a non-empty file. Path:
- `strucsup.rs:1055` — `if (*fcb).FirstClusterOfFile == 0 { AllocationSize.QuadPart = 0 } else { = HINT(-1) }`.
  HINT is what makes read.rs:755 run `FatLookupFileAllocationSize`. If `AllocationSize == 0`
  (not HINT), the lookup is **skipped** and `FileSize(>0) > 0` raises.
- So either **(a)** `(*Dirent).FirstClusterOfFile` (strucsup.rs:1049, +FAT32 hi at :1052) is
  mis-read as 0 (arm64 dirent field) forcing the `AllocationSize=0` branch; or **(b)**
  `FatLookupFileAllocationSize` populates `AllocationSize` as 0 on arm64.

**Leading hypothesis (b) — refined by control-plane → likely a deep arm64 Cc/Mm page-in race,
NOT a fastfat constant bug:**
- `FatLookupFileAllocationSize` (**allocsup.rs:1282**) sets AllocationSize as a *side effect*
  of `FatLookupFileAllocation(IrpContext, Fcb, MAXULONG-1, …)` walking the FAT chain to the
  sentinel VBO; the write happens in the **`allocsup.rs:707 if Vbo == MAXULONG.wrapping_sub(1)`**
  branch. There is a **2nd identical `FatRaiseStatus(FILE_CORRUPT)` site at allocsup.rs:1292**
  — the fastfat raiser in the crash could be read.rs:758 **or** allocsup.rs:1292.
- **It's INTERMITTENT on mainline-no-clock** (deterministic only under the clock's
  timer-driven read) → a pure constant bug would be deterministic on all arches. So the
  suspect is the **FAT-chain walk inside `FatLookupFileAllocation` reading the FAT via
  `Cc`/`CcMapData`**, hitting the **known arm64 Cc/CcMapData page-in race** (`MmMakeDataSectionResident`/
  Mm section page-in intermittently handing back **stale/zero** pages — the "CcMapData busy
  livelock" / netstack-era arm64 Cc notes). Zero FAT bytes → ClusterCount/AllocationSize = 0
  → `FileSize(>0) > 0` → false `FILE_CORRUPT`.

**Confirming experiment (owed):** instrument `FatLookupFileAllocation`'s cluster walk
(StartingCluster / ClusterCount / the FAT entries it reads) when AllocationSize comes back 0.
If the FAT bytes read as zero but the on-disk FAT isn't → it's the arm64 Cc/Mm page-in race
(deeper Mm/Cc, the real root, likely shared). Requires a fastfat.sys (+ maybe ntoskrnl Cc/Mm)
instrument boot. If instead the bytes are correct but the count math is wrong → simpler
fastfat fix.

**NOT yet pinpointed** (a) vs (b)-race vs (b)-math. The clock's determinism makes this a
one-boot catch once instrumented.

## 3. Reusable: symbolicating the cargo-built arm64 kernel

`objdump`/`llvm-nm`/`llvm-objdump` all return empty on the cargo-linked `ntoskrnl.exe`
(opaque PE; RVAs exceed the 2MB file — large virtual layout). What works:
- `qemu/build-index.sh arm64` → `qemu/symbols-arm64.tsv` (RVA `<tab>` symbol; ~9k syms but
  **only ntoskrnl Rust+asm**, tops out ~RVA 0x581f80; C-object & other-module funcs absent).
- **Runtime base:** print a known symbol's runtime address from inside the kernel
  (e.g. `KiDispatchException as usize` — tsv RVA `0x410b40`) → `base = runtime − rva`.
  Observed base ≈ `0xFFFFF80041D91000` (shifts ~0x30 per rebuild). Then
  `rva = runtime_addr − base`; nearest tsv symbol ≤ rva names it. A raiser past the tsv max
  ⇒ it's in a module loaded *after* ntoskrnl (fastfat/hal/driver), not ntoskrnl.
- Frame-pointer (x29) unwind did NOT work from `KiDispatchException` (large fn, x29 not a
  clean FP there); a raw stack scan works but is polluted by stale clock-tick frames.
- Best raiser-ID: hook the **`ExRaiseStatus` shim** (`shim_exports.rs`, exists expressly for
  this) to print `x30` (caller LR) when `status == 0xC0000102`.

## 4. Status / next

- Clock fix: complete, correct, preserved on `arm64-love`. **Gated** on the fastfat fix.
- Next: fastfat instrument boot → confirm (a) vs (b) → fix in fastfat-rust → re-verify arm64
  deterministically desktop-green with the clock → hand SHA to control-plane for cold ×3.
