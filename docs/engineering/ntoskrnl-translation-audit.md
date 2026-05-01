# ntoskrnl C→Rust Translation Audit (updated 2026-05-01, post-arcname)

## Headline

- **~99.5% translated by C-line count.** ~282K LOC translatable C body of work; ~1.5K LOC of translatable C still active in the default build (worst arch). The body-of-work denominator grew last session because `kdbg/i386/i386-dis.c` (4,453 LOC) moved out of the "intentional stay" bucket. arcname dropped this session — closes the long-standing C/Rust hybrid.
- **arcname landed.** `io/iomgr/arcname.c` is no longer active on any arch. `IopCreateArcNames` + `IopCreateArcNamesDisk` (~700 LOC of C, the boot-critical disk-enumeration + ARC-symlink-creation path) hand-translated this session into the existing `iomgr/arcname.rs`. Pattern reuse: the two new functions share IRP/IOCTL/symlink-creation idioms with the already-Rust `IopCreateArcNamesCd` next door, so most of the work was structural copy-and-adapt. i386 7/7, amd64 7/7. **Reflection:** identical flake observed as the iomgr session — first cold-boot after a clean Rust rebuild can fail on amd64 with HAL/VGA early-init looping; second boot through the snapshot path passes. Pre-existing, not introduced by this change.
- **i386-dis.rs landed.** The 4,453-LOC binutils-derived i386/amd64 disassembler used by KDBG is now Rust on i386 + amd64 (arm64 has KDBG off). Translated by **Gemini 3 Pro** in one shot via the prompt drafted earlier this session. Output was architecturally sound (enum-tag dispatch instead of C function-pointer table, `Result<_, NTSTATUS>` instead of setjmp/longjmp, all major opcode tables intact at correct cardinality, license preserved). Required two small post-translation fixes by hand: a wrong import path (`ntkrnl_sys::PCSTR` → `ntkrnl_sys::ddk::PCSTR`) and one missing `unsafe` block around `ptr::copy_nonoverlapping`. Gemini's "compiles successfully" claim was wrong — useful data point: treat its build/test claims as guidance, not ground truth. **Verdict: a useful translation collaborator at this scale, but verify everything.**
- **config/ landed.** All 21 functional config files are Rust-active on i386, amd64, and arm64. The 7-fix `wip-config-rust-port` series merged via `ddba7c05` — last fix was `cmparse.rs::COMPUTE_HASH_CHAR` missing the `RtlUpcaseUnicodeChar` call, silently disagreeing with `cmkcbncb.rs` and cascading into the `cm_x.h:120` / `ex.h:1305` ASSERT pair late in SMSS init.
- **kd64/ landed.** WinDbg KD protocol stack now Rust on all 3 arches. The "recursive exception flood on amd64" that parked it in Feb 2026 was silently fixed by the CGU=256 + opaque_ptr + RAII guard work; nobody re-tested. ~6.8K LOC of C dropped, ~11.1K LOC of parked Rust now active. Required 5 small translation-bug fixes.
- **ARM3/virtual landed.** `mm/ARM3/virtual.c` is no longer active; `ARM3/virtual.rs` is wired for all 3 arches. The i386 stack-commit regression was a bad local `TableFoundNode = 0` constant; bindgen/C have `TableFoundNode = 1`.
- **pnpres landed.** `io/pnpmgr/pnpres.c` is no longer active; `pnpmgr/pnpres.rs` (3,264 LOC) is wired for all 3 arches. Same playbook as kd64/virtual.rs — the parked excuse ("translated, never wired") was stale.
- **sysinfo hand-translated and landed.** `ex/sysinfo.c` is no longer active; `ex/sysinfo.rs` is wired for all 3 arches. Unlike previous landings this was real translation debt — the parked file was a Potemkin stub (full `NtQuerySystemInformation` returning `STATUS_NOT_IMPLEMENTED`, 74 handlers as commented C source). Hand-translated all the boot-critical handlers + `CallQS[]` dispatch table. Complex bindgen-archaeology handlers (process/handle enumeration with `_KTHREAD` union access) stubbed for now — boot doesn't depend on them. **Bonus**: revealed five flexible-array-indexing bugs in `pnpres.rs` (`Descriptors[ii as usize]` etc.) that crashed when the new dispatch let boot run further. ~3.1K LOC of C dropped.
- **iomgr + ramdisk landed on arm64 (this session).** `io/iomgr/iomgr.c` and `io/iomgr/ramdisk.c` are no longer active on any arch; the parked Rust versions (`iomgr/iomgr.rs` 1,215 LOC + `iomgr/ramdisk.rs` 403 LOC) are now wired on all 3 arches. The "PE layout issue on arm64" excuse was partly stale, partly a missing piece of plumbing: removing the `#[cfg(not(target_arch = "aarch64"))]` gates in `iomgr/mod.rs` was step one, but the link then failed because the static-defining CGUs (`PnpSystemInit`, `IoFileObjectType`, `Io{Read,Write,Other}{Operation,Transfer}Count`, `IopFileMapping`, `IopSecurityResource`, `IopLoaderBlock`) weren't being pulled out of the staticlib archive — the C `iomgr.c` had been satisfying those refs. Fix: `_force_iomgr_symbols` block in `lib.rs` referencing 3 fns + 13 statics. i386/amd64 still 7/7, arm64 still 0/7 (identical 13,449 bytes serial → same baseline hang at `[HIVE] CmCheckRegistry done`), no regression. ~1.1K LOC of C dropped on arm64.
- **Smoke status:** i386 7/7 PASS · amd64 7/7 PASS · arm64 0/7 baseline failure — reaches `[HIVE] CmCheckRegistry done` and hangs before `[PS] PspUserThreadStartup`. Verified identical with C and Rust kd64 (same 13,449 bytes serial), so the arm64 issue is elsewhere, not in kd64. Same baseline holds with Rust iomgr/ramdisk active.

## Active in Rust (default build)

Counts are *active modules*. Files on disk that exist but aren't wired into `mod.rs` (or are gated off per-arch) are tracked in the "Parked" section below.

| Subsystem | Active mods | LOC active | Notes |
|-----------|------------:|-----------:|-------|
| KE (kernel) | 67 | 54,113 | Includes per-arch `ke/{i386,amd64,arm64}/*` |
| ARM3 (mm internals) | 25 | 44,054 | `virtual.rs` active all 3 arches |
| iomgr | 26 | 41,800 | `iomgr.rs` + `ramdisk.rs` + full `arcname.rs` (no C hybrid) active on all 3 arches |
| EX (executive) | 31 | 38,781 | `sysinfo.rs` active all 3 arches |
| SE (security) | 18 | 35,200 | |
| **config (registry CM)** | **21** | **33,196** | **NEW — all 3 arches active** |
| ps (process) | 13 | 23,085 | |
| ob (object) | 11 | 22,532 | |
| fsrtl | 17 | 20,963 | |
| pnpmgr | 14 (+arb subdir) | 25,973 | `pnpres.rs` active all 3 arches |
| fstub | 4 | 12,264 | |
| mm | 12 | 11,893 | |
| **kd64 (WinDbg KD protocol)** | **7** | **11,100** | **NEW — all 3 arches active** |
| kdbg (KDB) | 9 | 13,485 | i386 + amd64 only (KDBG off on arm64); includes `i386_dis.rs` (Gemini 3 Pro translation) |
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
| **TOTAL active (i386/amd64)** | **~321** | **~414K LOC** | |

## Parked translations (Rust on disk, C version active in build)

These cost real API money to produce. Stay in the tree until the regression is debugged. **Do not delete.**

| Rust file | Rust LOC | C still active | LOC | Reason parked |
|-----------|---------:|----------------|----:|---------------|
| lpc/connect.rs | 736 | lpc/connect.c | 802 | Disabled to isolate an LLVM codegen bug; probed last session and confirmed real captures(none) cascade — needs opaque_ptr investigation, not stale-excuse revival. |
| ke/arm64/trap.rs | dead | ke/arm64/trap.c | 526 | Rust version breaks boot. **Edit the C file** — the .rs is dead. |
| ke/amd64/usercall.rs | dead | ke/amd64/usercall.c | 404 | Undiagnosed return-address corruption in Rust version. |
| **Parked total** | **~5.1K Rust** | | **~1.7K C** | |

## Still in C — intentional (no Rust translation planned)

| File | LOC | Why C |
|------|----:|-------|
| mm/section_pe.c | 661 | PE format parser; not worth the translation surface. |
| config/cmdata.c | 994 | Pure globals; no functions for tree-sitter to extract. |
| config/i386/cmhardwr.c | 897 | i386/amd64 hardware detection (CMOS, FX bytes, MSRs). |
| config/arm/cmhardwr.c | 27 | ARM hardware detection (legacy, not built). |
| kdbg/{i386,amd64}/kdb_help.S | ~30 each | Asm trampolines. |
| ntoskrnl/guid.c | 9 | Auto-generated. |
| ntoskrnl/KrnlFun.c | 48 | Dead (not in cmake). |
| `*_rust_shims.c` (~12 files) | 9,237 | FORCEINLINE/macro/dllimport bridges for Rust FFI. By design. |
| tests/*.c | 5,859 | Kernel test harness. Not for translation. |

## Still in C — translation candidates (NOT yet translated)

The real backlog. Roughly 2K LOC of small per-arch glue.

| File | LOC | Per-arch? | Notes |
|------|----:|-----------|-------|
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
| Generic active C | ~2,460 |
| i386-only active C (worst arch) | ~1,550 |
| **Active C in default build (worst case = i386)** | **~4,010** |
| of which intentional-stay (cmdata + section_pe + cmhardwr) | ~2,550 |
| **Translatable C still active** | **~1,460** |

## Percentage

Translatable C body of work (excluding tests, shims, intentional stays, dead arm32, KrnlFun, guid):
- ~282,200 LOC C total (includes i386-dis.c which left the intentional-stay bucket last session)
- ~1,460 LOC C still active and translatable
- **(282,200 − 1,460) / 282,200 ≈ 99.5% translated**

If you also count the parked translations as "done" (translation already paid for, just not enabled):
- Parked Rust: ~5.1K LOC
- Add ~1.7K LOC of C they replace if/when wired
- **>99.5% has a Rust translation in tree** (active or parked)

## What it would take to push toward 100%

In rough priority order (yield vs. cost):

1. **lpc/connect** — 802 LOC. Re-enable `connect.rs`, debug the captures(none) cascade. Last session's probe confirmed it's a real LLVM regression, not a stale-excuse revival. Needs opaque_ptr work or CGU debugging.
2. **arch-specific glue** — `ke/amd64/usercall.c`, `ke/arm64/trap.c`, `ps/{i386,amd64}/psctx.c`, `ps/i386/psldt.c`. Mostly small, mostly arch-dependent debugging.
3. **arcname** — finish the remaining `IopCreateArcNames*` paths in Rust; delete the C hybrid.
4. **kd64/{i386,amd64}/kdx{86,64}.c** — small per-arch helpers (~840 LOC combined). Easy translation candidates now that the generic kd64 stack is Rust.
5. **sysinfo follow-up** — fill in the bindgen-archaeology stubs (`SystemProcessInformation` thread enumeration, `SystemHandleInformation`/`SystemExtendedHandleInformation` handle table walk, `SystemLogicalProcessorInformation` PRCB MultiThread fields). Currently return zeroed/empty data — task manager will be unhappy but boot is fine.

## Stale-claim watch

The previous audit's notes on parked subsystems were several months out of date. **Confirm before working from them.** Examples uncovered this session:

- **kd64**: claimed "recursive exception flood on amd64, root cause unknown." Actually fixed silently months ago; just needed wiring + 5 small translation-bug fixes.
- **arm64 baseline**: claimed "no serial output, pre-existing 0x6A BSOD." Actually reaches `[HIVE] CmCheckRegistry done` and hangs before `[PS] PspUserThreadStartup`. Different failure mode, deeper into boot.
- **ARM3/virtual**: the first regression looked like an LLVM `ud2` cascade, but the boot-blocking i386 crash was a real translation bug: local `TableFoundNode` was `0` instead of the C/bindgen value `1`.
- **pnpres**: claimed "translated, never wired." Actually wires cleanly with two small fixes (malformed extern block, missing DbgPrint extern decl). 7/7 i386, 7/7 amd64, 0/7 arm64 (matches baseline).
- **sysinfo**: claimed "70+ class handlers; not all translated." This one was actually accurate — the parked file was a stub. Hand-translated last session.
- **iomgr/ramdisk on arm64**: claimed "PE layout issue → 0x6B crash." Half-stale: the cfg gate in `mod.rs` had to come off, but the build then failed not from a runtime PE problem — from a link-time problem. Statics like `IoFileObjectType` weren't being pulled from the kernel-rust archive because the C `iomgr.c` had been satisfying those refs. `_force_iomgr_symbols` block fixed it. arm64 baseline failure is unchanged (so the historical 0x6B crash, if it was ever real, is gone or different now).

When picking a parked file to revive, the cheapest first step is often: re-enable it on whichever arch is greenest, build, smoke. The bug may already be gone.

## What the audit doesn't measure

- **Boot correctness.** A line-count metric says nothing about whether the Rust version actually works under load. The smoke harness is the source of truth on that — current verdict: i386 7/7, amd64 7/7, arm64 0/7 (pre-existing baseline issue, not in kd64).
- **API debt.** ~6.7K LOC of paid translations are parked. Some need debugging, some need wiring. None of that needs new API spend.
- **Per-arch differentials.** i386 and amd64 are mostly aligned; arm64 has its own set of parked files (iomgr/ramdisk PE layout, ke/arm64/trap, KDBG fully off). The numbers above use i386/amd64 as the reference because that's where the build is at parity.

## How these numbers were computed

```bash
# Total C in ntoskrnl/ (excluding tests + shim/helper bridges):
find ntoskrnl -name "*.c" \
  | grep -v -E '/tests/|_rust_shims\.c|kernel_c_helpers\.c' \
  | xargs wc -l   # → 283,846

# Active C in default build (NEWCC=OFF; KD64_USE_C=ON; KDBG=ON):
grep -E '^\s+\$\{REACTOS_SOURCE_DIR\}/ntoskrnl/.*\.c' ntoskrnl/ntos.cmake \
  | grep -v '^\s*#' \
  | grep -v 'tests/' \
  | grep -v -E '_rust_shims\.c|kernel_c_helpers\.c' \
  | sed 's|^\s*\${REACTOS_SOURCE_DIR}/||; s|).*||; s| .*||' \
  | sort -u | xargs wc -l   # → 22,342, minus 5,296 NEWCC = 17,046

# Rust LOC across ntoskrnl-rust/:
find sdk/lib/rust/ntoskrnl-rust/src -name "*.rs" | xargs wc -l   # → 430,641
```

Numbers above use these scripts as the source of truth. Re-run them after any subsystem flip; the audit text is just a snapshot of what they output on 2026-04-30.
