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

---

## 5. RESOLVED root cause of the FILE_CORRUPT (supersedes §2 speculation) — 2026-06-20

The "Cc/CcMapData page-in race" hypothesis in §2 was **WRONG**. With the clock making the
fault deterministic, a `FatRaiseStatus` chokepoint instrument (log caller LR + a runtime
anchor in the `rust_cohab_stub.c` shim) pinned the raise to **`FatSetFatEntry`'s
volume-dirty switch** (`allocsup.rs`), reached from **`FatMarkVolume`** (`verfysup.rs`).

Real bug = a **translation constant error**: `verfysup.rs` defined LOCAL
`const FAT_DIRTY_VOLUME = 0x0FFFFFF7` / `FAT_CLEAN_VOLUME = 0x0FFFFFF8` (those are FAT32
*cluster* end/bad markers) instead of the volume dirty-state values
`0xF0000001` / `0xF0000000` (`~FAT32_ENTRY_MASK | {1,0}`, per `fastfat/fat.h`). So
`FatMarkVolume(dirty)` passed `0x0FFFFFF7`; `FatSetFatEntry`'s `switch (FatEntry)` matched
neither `FAT_CLEAN_VOLUME` nor `FAT_DIRTY_VOLUME` and fell to `default: FatRaiseStatus(
STATUS_FILE_CORRUPT_ERROR)` (this `default` exists in pristine C — the translation was
faithful; only the *constants* were wrong). DUMMY_PSEH → unhandled → BUGCHECK 0x1E. The
clock made the timer-driven volume-dirty mark fire every boot → deterministic.

**Fixed** (both the file-scope pair and the fn-local dup in `verfysup.rs`). Committed with
the clock in `65c82a286a0`. New field-manual bug class: *wrong-value local constant
shadowing a correct module/header constant* — grep fastfat/*.rs for re-declared FAT_*
constants vs `fat.h`. Same family as the cdfs grind-constant fixes.

## 6. NEW downstream blocker: 0xC2 / 0x19 pool corruption ("Irp " double-free) — OPEN

With FILE_CORRUPT cleared, boot reaches a **BUGCHECK 0xC2 BAD_POOL_CALLER subcode 0x0D
(POOL_BILLED_PROCESS_INVALID)** at `expool.rs:1404`, args
`(0x0d, p=0xfffffa000638e330, tag=0x20707249 "Irp ", process=0xfffffa000638e508)`, during a
**System worker thread exit** (`PspExitThread → LpcExitThread`). Deterministic.

Diagnostic instrument (log block+IRP fields, guard-skip the bugcheck) revealed:
`PoolType=0` (zeroed/free header — the quota branch fired *spuriously* because
`0u8.wrapping_sub(1) & QUOTA_POOL_MASK != 0`), `IrpType=0xc7c0` (garbage; valid IRP Type=6),
`AllocFlags=4` (**no `IRP_QUOTA_CHARGED` bit**). Guard-skipping then hit **BUGCHECK 0x19
BAD_POOL_HEADER subcode 3** on a *different* block ⇒ **broad pool corruption / double-free**,
NOT a quota-logic bug.

**GDB backtrace** (run-until KeBugCheckEx; load_base from KeBugCheckEx-entry break =
`0xfffff80042192000`):
```
#0 KeBugCheckEx
#1 ExFreePoolWithTag   @ +0x8cc into the fn   (faithful: bad header → bugcheck)
#2 0xfffff800423aafcc  → VA 0x218f8c  (BEYOND ntoskrnl image 0x1FA000 → a module loaded
                                       AFTER ntoskrnl; NOT hal.dll — hal.dll is 0x7000 and
                                       never calls ExFree/IoFreeIrp)
#3 garbage (no-DWARF unwind broke here)
```
Ruled OUT: IRP stack-location layout/stride (208 base + 0x44 stride consistent end-to-end),
quota alloc/free slot offsets (identical `pool_next_block-1`), `FatMarkVolume`'s own IRP
lifecycle (completion routine correctly returns STATUS_MORE_PROCESSING_REQUIRED, no
double-free), the quota constant path (faithful to C).

**Hypothesis:** a real IRP double-free / use-after-free in the FAT-volume-WRITE path that my
verfysup fix newly exercises (FatSetFatEntry→FatPrepareWriteVolumeFile→Cc/Mm paging write),
OR a clock-IRQ-enablement interaction (KiIrqHandler was a no-op print before the clock; the
clock is the FIRST code to enable the GICv2 distributor/CPU-interface on arm64 — devices
were polled). frame#2-in-a-later-module is suspicious but the no-DWARF unwind is unreliable
past frame#1.

**Next (decisive):** DWARF rebuild (`cmake -DCMAKE_C_FLAGS_RELEASE="-O2 -g3" .` +
`RUSTFLAGS=-C debuginfo=2`, full arm64 ninja) so `run-until --break KeBugCheckEx --then
"bt 60"` gives a CFI-unwound, fully-symbolicated stack across all modules → exact double-free
site. Then fix the lifetime bug. Clock + FILE_CORRUPT fix stay parked on `arm64-love`
(`65c82a286a0`) — correct but NOT desktop-green until the 0xC2 is fixed (the clock correctly
*exposes* this latent bug; per the translation philosophy the red state is the right state).

## 7. §6 refined: it's NON-DETERMINISTIC HEAP CORRUPTION (not a fixed logic bug)

Adding a tiny timing perturbation to `ExFreePoolWithTag` (an `x30`-capture + DbgPrint in the
bad-free branch) changed the crash from the deterministic **0xC2** to **0xF4
CRITICAL_OBJECT_TERMINATION** (`0x03, proc=0xfffffa0006384760, …, 0xfffff800422ca710`) — a
critical process died instead. Across builds the symptom rotates 0xC2 / 0x19 / 0xF4. That is
the signature of **timing-sensitive heap/pool corruption** with random downstream victims,
NOT a deterministic logic error. (The *clean* committed build is reproducibly 0xC2; any
instrumentation that shifts timing redirects the victim — so print-based caller capture and
no-DWARF unwind are both unreliable for this.)

Tooling walls hit this session: (a) DWARF build is blocked — the cmake/cargo link clobbers
`RUSTFLAGS=-C debuginfo=2` and the PE link strips `.debug_*` (preflight DWARF-richness fails
by default); (b) the sparse `objdump -t` table can't name internal Rust fns (frames land at
`+0xffXXX` past the nearest symbol); (c) `find-kbase` gives the true base
`kbase=0xfffff80042191000`, and `objdump_VA = runtime − 0xfffff80042192040` (kbase + 0x1040
header gap) — but that only resolves exported symbols.

**Recommended next approach (catch the corruptor, not the victim):**
1. Enable **special pool / page-heap for tag `Irp `** (and/or pool-corruption checks) so the
   guard page faults at the moment of the bad write — turns a random downstream bugcheck into
   a deterministic catch at the offending instruction.
2. OR fix the DWARF plumbing (force `[profile.release] debug=true` in the workspace
   `Cargo.toml` so rlibs carry DWARF regardless of RUSTFLAGS, and stop the cargo-link from
   stripping `.debug_*`), then `run-until --break KeBugCheckEx --then "bt 60"` for a clean
   CFI stack.
3. Bisect the trigger: the corruption is reached only via the FAT-volume-WRITE path that the
   verfysup fix newly exercises (FatSetFatEntry→FatPrepareWriteVolumeFile→Cc/Mm paging
   write). Worth auditing that specific write/flush path on arm64 (MDL handling, buffer
   sizing with the 0x44 IO_STACK_LOCATION ABI) as the prime suspect region.

Clean repro (committed `arm64-love`, no instrument): boot arm64 → deterministic 0xC2 at the
System-thread-exit `ExFreePoolWithTag` of a corrupt `Irp ` block.

---

## RESOLVED (2026-06-20) — it was NOT an LPC/Cc race and NOT the 0xC2; it was a fastfat stack overflow

Everything above (the `0xC2` "Irp " corruption hunt, the special-pool plan, the
"LpcExitThread race" framing) was chasing the wrong layer. After the `0xC2` double-free was
fixed (`fddff701ac9`, SL_INVOKE constants) the remaining deterministic late-boot bugcheck was
a **`0x50` kernel STACK OVERFLOW from unbounded recursion in fastfat** — surfaced (made
every-boot) by the periodic clock, which drives the writable-ramdisk dirty path.

**The actual bug (commit `3b3f9744c0a`):** `cachesup.rs` `FatSetDirtyBcb` had a local
`IRP_CONTEXT_FLAG_DISABLE_DIRTY = 0x100` placeholder; canonical value is `0x00000001`
(fatstruc.h). `FatSetFatEntry` sets that flag (0x1) around its `FatPrepareWriteVolumeFile`
writes precisely to stop the nested `FatSetDirtyBcb` from re-marking the volume dirty — but
with the checker reading 0x100, the guard never fired:

```
FatSetDirtyBcb → FatMarkVolume → FatSetFatEntry → FatPrepareWriteVolumeFile → FatSetDirtyBcb → …∞
```

→ the 12 KB arm64 kernel stack (= i386's value; amd64 uses 48 KB) blew its guard page → a
first data-abort in the Cc dirty-flush path, then `KiSynchronousExceptionEntry` faulted again
saving its own trapframe on the exhausted stack → the nested `0x50` (fault addr one page above
SP) that triage kept mislabeling.

**How it was finally cracked (the technique that worked where everything above failed):** read
the `KTRAP_FRAME` raw for `Pc`/`Lr`/`Sp`/`Fp`/`Esr`; **walk the x29 frame chain** → saw the
same 4 return addresses repeating (the recursion); the addresses were **past ntoskrnl's last
symbol** so **walked `PsLoadedModuleList`** to map them into **fastfat.sys**; resolved with
`aarch64-w64-mingw32-nm` (NOT llvm-nm/objdump, which return empty). Distilled into field-manual
**§9.11**.

**Verified:** arm64 boots clean to the desktop (`shell ready`), 0 bugchecks, deterministically
(was a deterministic `0x50` before). i386/amd64 unaffected (they never run this writable-FS
path). This is the fix that unblocks the parked floor-completion stack.
