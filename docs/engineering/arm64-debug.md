# ARM64 Boot Regression — Debug Brief

## Problem

ARM64 crashes with BUGCHECK 0x6B (PROCESS1_INITIALIZATION_FAILED) during early kernel init. `PsCreateSystemThread` returns `STATUS_INSUFFICIENT_RESOURCES` (0xC000009A) when trying to create the Phase 1 initialization thread. The system process itself is created successfully.

**i386 and amd64 both boot to desktop (4/4 smoke tests).** Only ARM64 is affected.

The ARM64 regression was confirmed as pre-existing — it reproduces identically with and without the recent constant-fix changes (tested by stashing all changes and retesting baseline).

## Boot Trace (Serial Output)

The kernel gets through:
1. FreeLDR UEFI boot, page table setup, KiSystemStartup
2. Full MM init (MmArmInitSystem, pool, PFN database, paged pool)
3. PS Phase 0 init (PspInitPhase0)
4. System process creation (PspCreateProcess succeeds, Status=0)

Then fails at:
```
[PS] Creating Phase1 thread
[PS] FAIL: PsCreateSystemThread status=0xFFFFFFFFC000009A
*** BUGCHECK: 0x0000006B
```

## Call Chain

```
ExpInitializeExecutive (Phase 0)
  → PspInitPhase0
    → PspCreateProcess (system process) → OK
    → PsCreateSystemThread (Phase 1 thread)
      → PspCreateThread(ThreadHandle, ..., PsInitialSystemProcess, ...)
        → ObCreateObject(PsThreadType, ...) → returns ETHREAD
        → MmCreateKernelStack → allocates thread stack
        → KeInitThread → sets up KTHREAD context
        → ObInsertObject → inserts into handle table
        → ... → returns STATUS_INSUFFICIENT_RESOURCES somewhere
```

STATUS_INSUFFICIENT_RESOURCES (0xC000009A) can come from:
- Pool allocation failure (unlikely — pool just initialized, ~64MB available)
- ObCreateObject failure (object creation)
- MmCreateKernelStack failure (kernel stack allocation)
- ObInsertObject failure (handle table)

## Key Differences: ARM64 vs amd64/i386

1. **Boot media**: ARM64 uses UEFI ramdisk (boot.img + ramdisk.img), not LiveCD ISO
2. **No KVM**: ARM64 runs in TCG (software emulation), not KVM
3. **QEMU machine**: ARM64 uses `-M virt -cpu cortex-a72`, not PC/q35
4. **No debugcon**: ARM64 has no 0xE9 port for boot tracing
5. **Module gating**: Some Rust modules are `#[cfg(not(target_arch = "aarch64"))]`:
   - `iomgr.rs` — excluded on ARM64
   - `ramdisk.rs` — excluded on ARM64
   - `devaction.rs` — excluded on ARM64 (just enabled for amd64)
6. **KiSystemStartup entry**: ARM64 has a different entry path (`FFFFF800422418D8`)
7. **PspCreateThread**: Same Rust code on all arches (thread.rs)

## Last Known Working State

Per memory (MEMORY.md, "PREVIOUS STATUS as of 2026-03-13"):
> ALL 3 ARCHITECTURES PASS SMOKE TESTS 4/4

So ARM64 was working on 2026-03-13 and broke sometime between then and 2026-03-19. The commits in that range include constant fixes, iofunc.rs completion, devaction enablement, and various IO module activations.

## Theories

### A. Rust code change broke ARM64-specific path
A commit between 2026-03-13 and 2026-03-19 changed Rust kernel code that is active on ARM64 and affects PspCreateThread or its callees. Most likely candidates:
- ps/thread.rs changes
- ob/obhandle.rs or ob/oblife.rs changes (ObCreateObject, ObInsertObject)
- ke/arm64/ changes (KeInitThread)
- mm/ changes (MmCreateKernelStack)

### B. Stale ramdisk
The ARM64 ramdisk might have stale system files from a much older build. However, `mkramdisk.sh` was run recently and the kernel was updated via mcopy. Other system DLLs on the ramdisk might be out of date with the current kernel.

### C. Constant fix side effect
The 154-constant bulk fix changed values in files active on ARM64 (se/token*.rs, ps/*.rs, ob/*.rs). A wrong constant could cause ObCreateObject or handle operations to fail. However, the baseline test (without constant fixes) also fails.

### D. Pool allocator issue on ARM64
The Rust pool allocator might have a latent ARM64-specific bug that wasn't triggered before. Since the system process is created successfully (which also uses pool), this seems unlikely unless thread creation uses a different pool path.

### E. `#[used]` / LLVM optimization eliminating ARM64-specific globals
Similar to the i386 `captures(none)` issue but specific to ARM64. Some critical global might be optimized away on aarch64 target.

## How to Test

```bash
# Rebuild ARM64 kernel:
cd build-arm64
rm -rf ntoskrnl/rust-kernel-target/ ntoskrnl/libkernel_rust.a
ninja ntoskrnl

# Update ramdisk:
mcopy -o -i ../qemu/ramdisk.img ntoskrnl/ntoskrnl.exe ::/reactos/system32/ntoskrnl.exe
mcopy -o -i ../qemu/boot.img ../qemu/ramdisk.img ::/ramdisk.img

# Full ramdisk rebuild (if needed):
cd .. && bash qemu/mkramdisk.sh

# Smoke test:
cd reactos
.venv/bin/python -m pytest tests/qemu/test_smoke.py -v --arch arm64 --timeout 300
```

## Suggested Debug Approach

1. **Git bisect** between the last known-good commit and current HEAD to find which commit broke ARM64. The last known-good was around 2026-03-13 (`da76333d` era). Use `git stash` to save current work, checkout each commit, rebuild ARM64, test.

2. **Add serial tracing to PspCreateThread** on ARM64 to see exactly which sub-call returns STATUS_INSUFFICIENT_RESOURCES:
   - Before/after ObCreateObject (does ETHREAD creation succeed?)
   - Before/after MmCreateKernelStack (does stack allocation succeed?)
   - Before/after KeInitThread
   - Before/after ObInsertObject

3. **Full ramdisk rebuild** with `bash qemu/mkramdisk.sh` to rule out stale system files. Also rebuild the SYSTEM hive: `ninja -C build-arm64 boot/bootdata/system`.

4. **Check if reverting specific subsystem constant fixes helps**: The se/token*.rs and ps/*.rs constant changes are most suspicious since they affect security token creation during thread setup.

5. **Check the PspCreateThread → KeInitThread path** on ARM64 — KeInitThread sets up the ARM64 thread context (CONTEXT structure, stack pointer, etc.) and might have different layout requirements.

## Reference Files

| File | Description |
|------|-------------|
| `/home/curi/Projects/os-playground/reactos/CLAUDE.md` | Project instructions, build commands, architecture overview |
| `/home/curi/.claude/projects/-home-curi-Projects-os-playground/memory/MEMORY.md` | Memory index with project state and known bugs |
| `/home/curi/.claude/projects/-home-curi-Projects-os-playground/memory/io_translation_status.md` | IO subsystem translation progress |
| `/home/curi/Projects/os-playground/reactos/sdk/lib/rust/ntoskrnl-rust/src/ps/thread.rs` | PsCreateSystemThread + PspCreateThread (the failing function) |
| `/home/curi/Projects/os-playground/reactos/sdk/lib/rust/ntoskrnl-rust/src/ps/process.rs` | PspCreateProcess (succeeds — useful for comparison) |
| `/home/curi/Projects/os-playground/reactos/sdk/lib/rust/ntoskrnl-rust/src/ob/obhandle.rs` | ObInsertObject (handle table operations) |
| `/home/curi/Projects/os-playground/reactos/sdk/lib/rust/ntoskrnl-rust/src/ob/oblife.rs` | ObCreateObject (object creation) |
| `/home/curi/Projects/os-playground/reactos/sdk/lib/rust/ntoskrnl-rust/src/ke/arm64/thrdini.rs` | ARM64 KeInitThread |
| `/home/curi/Projects/os-playground/reactos/sdk/lib/rust/ntoskrnl-rust/src/mm/ARM3/procsup.rs` | MmCreateKernelStack |
| `/home/curi/Projects/os-playground/reactos/ntoskrnl/ntos.cmake` | Build config — C vs Rust file selection |
| `/home/curi/Projects/os-playground/qemu/mkramdisk.sh` | ARM64 ramdisk creation script |
| `/home/curi/Projects/os-playground/reactos/tests/qemu/test_smoke.py` | Smoke test framework |
