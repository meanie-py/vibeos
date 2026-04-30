# ntoskrnl C→Rust Translation Audit (updated 2026-04-30, post-kd64)

## Headline

- **~95% translated by C-line count.** ~278K LOC translatable C body of work; ~12.8K LOC of translatable C still active in the default build.
- **config/ landed.** All 21 functional config files are Rust-active on i386, amd64, and arm64. The 7-fix `wip-config-rust-port` series merged via `ddba7c05` — last fix was `cmparse.rs::COMPUTE_HASH_CHAR` missing the `RtlUpcaseUnicodeChar` call, silently disagreeing with `cmkcbncb.rs` and cascading into the `cm_x.h:120` / `ex.h:1305` ASSERT pair late in SMSS init.
- **kd64/ landed (this session, `18bd525d`).** WinDbg KD protocol stack now Rust on all 3 arches. The "recursive exception flood on amd64" that parked it in Feb 2026 was silently fixed by the CGU=256 + opaque_ptr + RAII guard work; nobody re-tested. ~6.8K LOC of C dropped, ~11.1K LOC of parked Rust now active. Required 5 small translation-bug fixes (Processor u16→u32, 5× `Some(KdpStub)` → `Some(abi_cast!(KdpStub))`, `KiDebugRoutine` static = None, `KDDEBUGGER_DATA64_FULL` pub).
- **Smoke status:** i386 7/7 PASS · amd64 6/7 (only `[EXPLORER] _tWinMain entered` missing — pre-existing taskbar regression, unrelated to translation) · arm64 0/7 baseline failure — reaches `[HIVE] CmCheckRegistry done` and hangs before `[PS] PspUserThreadStartup`. Verified identical with C and Rust kd64 (same 13,449 bytes serial), so the arm64 issue is elsewhere, not in kd64. The previous audit's "no serial output, 0x6A BSOD" claim was stale.

## Active in Rust (default build)

Counts are *active modules*. Files on disk that exist but aren't wired into `mod.rs` (or are gated off per-arch) are tracked in the "Parked" section below.

| Subsystem | Active mods | LOC active | Notes |
|-----------|------------:|-----------:|-------|
| KE (kernel) | 67 | 54,113 | Includes per-arch `ke/{i386,amd64,arm64}/*` |
| ARM3 (mm internals) | 24 | 40,768 | `virtual.rs` parked |
| iomgr | 26 (arm64: 24) | 41,047 / 39,429 | `iomgr.rs` + `ramdisk.rs` arm64-gated |
| EX (executive) | 30 | 35,201 | `sysinfo.rs` parked |
| SE (security) | 18 | 35,200 | |
| **config (registry CM)** | **21** | **33,196** | **NEW — all 3 arches active** |
| ps (process) | 13 | 23,085 | |
| ob (object) | 11 | 22,532 | |
| fsrtl | 17 | 20,963 | |
| pnpmgr | 13 (+arb subdir) | 22,709 | `pnpres.rs` parked |
| fstub | 4 | 12,264 | |
| mm | 12 | 11,893 | |
| **kd64 (WinDbg KD protocol)** | **7** | **11,100** | **NEW — all 3 arches active** |
| kdbg (KDB) | 8 | 9,979 | i386 + amd64 only (KDBG off on arm64) |
| cc (cache) | 7 | 6,221 | |
| dbgk | 2 | 5,104 | |
| po (power) | 4 | 5,050 | |
| inbv (boot video) | 3 | 3,633 | |
| lpc | 7 | 3,206 | `connect.rs` parked |
| kd (low-level) | 6 | 3,808 | |
| wmi | 4 | 3,697 | |
| i386 (arch shared) | 7 | 2,797 | Used by i386 + amd64 |
| rtl | 2 | 2,183 | |
| vdm | 4 | 1,228 | |
| arm64 (arch) | 2 | 1,045 | |
| amd64 (arch) | 2 | 981 | |
| vf | 1 | 152 | |
| **TOTAL active (i386/amd64)** | **~320** | **~411K LOC** | |

## Parked translations (Rust on disk, C version active in build)

These cost real API money to produce. Stay in the tree until the regression is debugged. **Do not delete.**

| Rust file | Rust LOC | C still active | LOC | Reason parked |
|-----------|---------:|----------------|----:|---------------|
| ARM3/virtual.rs | 3,286 | mm/ARM3/virtual.c | 5,815 | NtAllocateVirtualMemory/NtFreeVirtualMemory stubs, heavy SEH. Disabled in `ARM3/mod.rs:29-30`. |
| ex/sysinfo.rs | 3,744 | ex/sysinfo.c | 3,080 | NtQuerySystemInformation has 70+ class handlers; not all translated. Disabled in `ex/mod.rs:26`. |
| pnpmgr/pnpres.rs | 3,264 | io/pnpmgr/pnpres.c | 1,498 | Translated, never wired into `pnpmgr/mod.rs`. |
| iomgr/iomgr.rs (arm64) | 1,215 | io/iomgr/iomgr.c (arm64) | 765 | PE layout issue on arm64. `#[cfg(not(target_arch = "aarch64"))]`. |
| iomgr/ramdisk.rs (arm64) | 403 | io/iomgr/ramdisk.c (arm64) | 307 | Same arm64 PE layout issue → 0x6B crash. |
| lpc/connect.rs | 736 | lpc/connect.c | 802 | Disabled to isolate an LLVM codegen bug; never re-enabled. |
| ke/arm64/trap.rs | dead | ke/arm64/trap.c | 526 | Rust version breaks boot. **Edit the C file** — the .rs is dead. |
| ke/amd64/usercall.rs | dead | ke/amd64/usercall.c | 404 | Undiagnosed return-address corruption in Rust version. |
| **Parked total** | **~17K Rust** | | **~13.2K C** | |

## Still in C — intentional (no Rust translation planned)

| File | LOC | Why C |
|------|----:|-------|
| mm/section_pe.c | 661 | PE format parser; not worth the translation surface. |
| config/cmdata.c | 994 | Pure globals; no functions for tree-sitter to extract. |
| config/i386/cmhardwr.c | 897 | i386/amd64 hardware detection (CMOS, FX bytes, MSRs). |
| config/arm/cmhardwr.c | 27 | ARM hardware detection (legacy, not built). |
| kdbg/i386/i386-dis.c | 4,453 | Binutils-derived i386 disassembler; not our code. |
| kdbg/{i386,amd64}/kdb_help.S | ~30 each | Asm trampolines. |
| ntoskrnl/guid.c | 9 | Auto-generated. |
| ntoskrnl/KrnlFun.c | 48 | Dead (not in cmake). |
| `*_rust_shims.c` (~12 files) | 9,237 | FORCEINLINE/macro/dllimport bridges for Rust FFI. By design. |
| tests/*.c | 5,859 | Kernel test harness. Not for translation. |

## Still in C — translation candidates (NOT yet translated)

The real backlog. Roughly 2K LOC of small per-arch glue.

| File | LOC | Per-arch? | Notes |
|------|----:|-----------|-------|
| io/iomgr/arcname.c | 998 | generic | Hybrid — `iomgr/arcname.rs` (1,633 LOC) covers some paths; C still owns `IopCreateArcNames` + `IopCreateArcNamesDisk`. Finish split. |
| kd64/i386/kdx86.c | 446 | i386 | Per-arch helpers called by Rust kdapi.rs (KdpSysReadMsr/WriteMsr/IoSpace, KdpGetStateChange, KdpSetContextState). Small, callable by Rust as-is — translate only when convenient. |
| kd64/amd64/kdx64.c | 391 | amd64 | Same role for amd64. |
| kd64/arm/kdarm.c | 156 | arm (not built) | |
| ps/i386/psctx.c | 91 | i386 | Context save/restore. |
| ps/i386/psldt.c | 113 | i386 | LDT setup. |
| ps/amd64/psctx.c | 69 | amd64 | |
| ps/arm/psctx.c | 47 | arm (not built) | |
| arm32 misc | ~2,500 | arm (not built) | `ke/arm/*.c` (7 files), `mm/arm/page.c`, `mm/ARM3/arm/init.c`, `rtl/arm/rtlexcpt.c`. ARM32 is dead in this fork. |

## Active C in default build, summed

| Bucket | LOC |
|--------|----:|
| Generic active C | ~13,850 |
| i386-only active C (worst arch) | ~1,550 |
| **Active C in default build (worst case = i386)** | **~15,400** |
| of which intentional-stay (cmdata + section_pe + cmhardwr) | ~2,550 |
| **Translatable C still active** | **~12,850** |

## Percentage

Translatable C body of work (excluding tests, shims, intentional stays, dead arm32, KrnlFun, guid):
- ~277,750 LOC C total
- ~12,850 LOC C still active and translatable
- **(277,750 − 12,850) / 277,750 ≈ 95.4% translated**

If you also count the parked translations as "done" (translation already paid for, just not enabled):
- Parked Rust: ~17K LOC
- Add ~13.2K LOC of C they replace if/when wired
- **~99% has a Rust translation in tree** (active or parked)

## What it would take to push toward 100%

In rough priority order (yield vs. cost):

1. **mm/ARM3/virtual.c** — 5,815 LOC of active C → re-enable `virtual.rs` (3,286 LOC parked). Needs SEH cleanup + missing-function fill-in. Highest single yield now that kd64 has landed.
2. **ex/sysinfo.c** — 3,080 LOC. `sysinfo.rs` parked at 3,744 LOC; finish the missing NtQuerySystemInformation classes.
3. **pnpres** — 1,498 LOC. Wire `pnpres.rs` into `pnpmgr/mod.rs` and shake out compile/link.
4. **lpc/connect** — 802 LOC. Re-enable `connect.rs`, debug the original LLVM codegen issue (or confirm CGU=256 fix already covers it — same story as kd64 may apply).
5. **arch-specific glue** — `ke/amd64/usercall.c`, `ke/arm64/trap.c`, `io/iomgr/iomgr.c` (arm64), `ps/{i386,amd64}/psctx.c`, `ps/i386/psldt.c`. Mostly small, mostly arch-dependent debugging.
6. **arcname** — finish the remaining `IopCreateArcNames*` paths in Rust; delete the C hybrid.
7. **kd64/{i386,amd64}/kdx{86,64}.c** — small per-arch helpers (~840 LOC combined). Easy translation candidates now that the generic kd64 stack is Rust.

After 1–3 land, translatable-C-still-active drops below ~3K LOC and we're ≥98% by the strict metric.

## Stale-claim watch

The previous audit's notes on parked subsystems were several months out of date. **Confirm before working from them.** Examples uncovered this session:

- **kd64**: claimed "recursive exception flood on amd64, root cause unknown." Actually fixed silently months ago; just needed wiring + 5 small translation-bug fixes.
- **arm64 baseline**: claimed "no serial output, pre-existing 0x6A BSOD." Actually reaches `[HIVE] CmCheckRegistry done` and hangs before `[PS] PspUserThreadStartup`. Different failure mode, deeper into boot.

When picking a parked file to revive, the cheapest first step is often: re-enable it on whichever arch is greenest, build, smoke. The bug may already be gone.

## What the audit doesn't measure

- **Boot correctness.** A line-count metric says nothing about whether the Rust version actually works under load. The smoke harness is the source of truth on that — current verdict: i386 7/7, amd64 6/7 (taskbar regression unrelated to translation), arm64 0/7 (pre-existing baseline issue, not in kd64).
- **API debt.** ~17K LOC of paid translations are parked. Some need debugging, some need wiring. None of that needs new API spend.
- **Per-arch differentials.** i386 and amd64 are mostly aligned; arm64 has its own set of parked files (iomgr/ramdisk PE layout, ke/arm64/trap, KDBG fully off). The numbers above use i386/amd64 as the reference because that's where the build is at parity.

## How these numbers were computed

```bash
# Total C in ntoskrnl/ (excluding tests + shim/helper bridges):
find ntoskrnl -name "*.c" \
  | grep -v -E '/tests/|_rust_shims\.c|kernel_c_helpers\.c' \
  | xargs wc -l   # → 282,469

# Active C in default build (NEWCC=OFF; KD64_USE_C=ON; KDBG=ON):
grep -E '^\s+\$\{REACTOS_SOURCE_DIR\}/ntoskrnl/.*\.c' ntoskrnl/ntos.cmake \
  | grep -v '^\s*#' \
  | grep -v 'tests/' \
  | grep -v -E '_rust_shims\.c|kernel_c_helpers\.c' \
  | sed 's|^\s*\${REACTOS_SOURCE_DIR}/||; s|).*||; s| .*||' \
  | sort -u | xargs wc -l   # → 33,692, minus 5,296 NEWCC = 28,396

# Rust LOC across ntoskrnl-rust/:
find sdk/lib/rust/ntoskrnl-rust/src -name "*.rs" | xargs wc -l   # → 429,437
```

Numbers above use these scripts as the source of truth. Re-run them after any subsystem flip; the audit text is just a snapshot of what they output on 2026-04-30.
