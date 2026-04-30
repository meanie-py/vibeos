# ntoskrnl C→Rust Translation Audit (updated 2026-04-30)

## Headline

- **~93% translated by C-line count.** ~278K LOC translatable C body of work; ~18K LOC of translatable C still active in the default build.
- **config/ landed.** All 21 functional config files are Rust-active on i386, amd64, and arm64. The 7-fix `wip-config-rust-port` series merged this branch (`continue-rust-port`); last fix `ddba7c05` — `cmparse.rs::COMPUTE_HASH_CHAR` was missing the `RtlUpcaseUnicodeChar` call, silently disagreeing with `cmkcbncb.rs` and cascading into the `cm_x.h:120` / `ex.h:1305` ASSERT pair late in SMSS init.
- **Smoke status:** i386 7/7 PASS · amd64 6/7 (only `[EXPLORER] _tWinMain entered` missing — pre-existing taskbar regression, unrelated to translation) · arm64 baseline failure stands (no serial output, pre-existing 0x6A BSOD).

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
| **TOTAL active (i386/amd64)** | **~313** | **~400K LOC** | |

## Parked translations (Rust on disk, C version active in build)

These cost real API money to produce. Stay in the tree until the regression is debugged. **Do not delete.**

| Rust file | Rust LOC | C still active | LOC | Reason parked |
|-----------|---------:|----------------|----:|---------------|
| kd64/* (8 files) | 11,100 | kd64/*.c (7 files) | 5,535 | Recursive exception flood on amd64 (INT3 → KdpTrap → KdpReport → ...). Compiled out via `-DKD64_USE_C`. Root cause unknown. |
| ARM3/virtual.rs | 3,286 | mm/ARM3/virtual.c | 5,815 | NtAllocateVirtualMemory/NtFreeVirtualMemory stubs, heavy SEH. Disabled in `ARM3/mod.rs:29-30`. |
| ex/sysinfo.rs | 3,744 | ex/sysinfo.c | 3,080 | NtQuerySystemInformation has 70+ class handlers; not all translated. Disabled in `ex/mod.rs:26`. |
| pnpmgr/pnpres.rs | 3,264 | io/pnpmgr/pnpres.c | 1,498 | Translated, never wired into `pnpmgr/mod.rs`. |
| iomgr/iomgr.rs (arm64) | 1,215 | io/iomgr/iomgr.c (arm64) | 765 | PE layout issue on arm64. `#[cfg(not(target_arch = "aarch64"))]`. |
| iomgr/ramdisk.rs (arm64) | 403 | io/iomgr/ramdisk.c (arm64) | 307 | Same arm64 PE layout issue → 0x6B crash. |
| lpc/connect.rs | 736 | lpc/connect.c | 802 | Disabled to isolate an LLVM codegen bug; never re-enabled. |
| ke/arm64/trap.rs | dead | ke/arm64/trap.c | 526 | Rust version breaks boot. **Edit the C file** — the .rs is dead. |
| ke/amd64/usercall.rs | dead | ke/amd64/usercall.c | 404 | Undiagnosed return-address corruption in Rust version. |
| **Parked total** | **~28K Rust** | | **~18.7K C** | |

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
| kd64/i386/kdx86.c | 446 | i386 | Coupled to the parked kd64 work. |
| kd64/amd64/kdx64.c | 391 | amd64 | Coupled to the parked kd64 work. |
| kd64/arm/kdarm.c | 156 | arm (not built) | |
| ps/i386/psctx.c | 91 | i386 | Context save/restore. |
| ps/i386/psldt.c | 113 | i386 | LDT setup. |
| ps/amd64/psctx.c | 69 | amd64 | |
| ps/arm/psctx.c | 47 | arm (not built) | |
| arm32 misc | ~2,500 | arm (not built) | `ke/arm/*.c` (7 files), `mm/arm/page.c`, `mm/ARM3/arm/init.c`, `rtl/arm/rtlexcpt.c`. ARM32 is dead in this fork. |

## Active C in default build, summed

| Bucket | LOC |
|--------|----:|
| Generic active C | ~19,400 |
| i386-only active C (worst arch) | ~1,550 |
| **Active C in default build (worst case = i386)** | **~20,950** |
| of which intentional-stay (cmdata + section_pe + cmhardwr) | ~2,550 |
| **Translatable C still active** | **~18,400** |

## Percentage

Translatable C body of work (excluding tests, shims, intentional stays, dead arm32, KrnlFun, guid):
- ~277,750 LOC C total
- ~18,400 LOC C still active and translatable
- **(277,750 − 18,400) / 277,750 ≈ 93.4% translated**

If you also count the parked translations as "done" (translation already paid for, just not enabled):
- Parked Rust: ~28K LOC
- Add ~12K LOC of C they replace if/when wired
- **~98% has a Rust translation in tree** (active or parked)

## What it would take to push toward 100%

In rough priority order (yield vs. cost):

1. **kd64 recursive-exception debug** — 11,100 LOC parked Rust + ~6,500 LOC of active C unblocks at once. No API spend; pure debugging.
2. **mm/ARM3/virtual.c** — 5,815 LOC of active C → re-enable `virtual.rs` (3,286 LOC parked). Needs SEH cleanup + missing-function fill-in.
3. **ex/sysinfo.c** — 3,080 LOC. `sysinfo.rs` parked at 3,744 LOC; finish the missing NtQuerySystemInformation classes.
4. **pnpres** — 1,498 LOC. Wire `pnpres.rs` into `pnpmgr/mod.rs` and shake out compile/link.
5. **lpc/connect** — 802 LOC. Re-enable `connect.rs`, debug the original LLVM codegen issue (or confirm CGU=256 fix already covers it).
6. **arch-specific glue** — `ke/amd64/usercall.c`, `ke/arm64/trap.c`, `io/iomgr/iomgr.c` (arm64), `ps/{i386,amd64}/psctx.c`, `ps/i386/psldt.c`. Mostly small, mostly arch-dependent debugging.
7. **arcname** — finish the remaining `IopCreateArcNames*` paths in Rust; delete the C hybrid.

After 1–4 land, translatable-C-still-active drops below ~5K LOC and we're ≥98% by the strict metric.

## What the audit doesn't measure

- **Boot correctness.** A line-count metric says nothing about whether the Rust version actually works under load. The smoke harness is the source of truth on that — current verdict: i386 7/7, amd64 6/7 (taskbar regression unrelated to translation), arm64 0/7 (pre-existing baseline issue).
- **API debt.** ~28K LOC of paid translations are parked. Some need debugging, some need wiring. None of that needs new API spend.
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
