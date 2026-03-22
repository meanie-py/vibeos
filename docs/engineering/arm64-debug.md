# ARM64 Desktop Boot — Debug Brief

## Current State

ARM64 boots through kernel init and IoInitSystem under CGU=16, but SMSS never starts (black screen). Under CGU=1 the boot gets further but hits a different crash. There are two independent bugs with different root causes.

**i386 and amd64 boot to desktop.** ARM64 previously had a full desktop (Milestone 27+, Feb 2026).

## The Two Bugs

### Bug 1: PspCreateThread `captures(none)` (CGU=16) — WORKED AROUND
- **Symptom:** BUGCHECK 0x6B — PsCreateSystemThread returns STATUS_INSUFFICIENT_RESOURCES during Phase 1
- **Root cause:** LLVM cross-CGU `captures(none)` attribute propagation eliminates output pointer writes in PspCreateThread
- **Workaround:** `#[inline(never)]` on PspCreateThread (committed `08f89eb8`)
- **With workaround:** Boot reaches IoInitSystem, but SMSS never starts. Serial goes silent after `IoInitSystem returning TRUE`. No crash, just idle.
- **Note:** The CGU=16 idle-out (SMSS never starting) is likely ANOTHER `captures(none)` victim — some APC queueing or thread state transition is being silently optimized away, causing the kernel to idle forever waiting for a flag that was never set.
- **Proper fix:** CGU=1 eliminates this class entirely, but exposes Bug 2

### Bug 2: ARM64 Zw* Stubs at Elevated IRQL (CGU=1) — UNRESOLVED
- **Symptom:** BUGCHECK 0x6A — `ASSERT FAILED: KeGetCurrentIrql() <= APC_LEVEL` in MiLockProcessWorkingSetShared
- **Caller:** CmpFindSubKeyInLeaf → ZwFlushVirtualMemory (via `svc #0`) at IRQL=2
- **Root cause:** ARM64 kernel-mode `Zw*` stubs use `svc #0` (software exception) for ALL callers, including kernel-mode callers at elevated IRQL. The SVC handler (`KiSvcDispatch` in trap.rs) dispatches the Nt* function without managing IRQL.

**This is an architectural bug, NOT a compiler optimization bug.** Multiple independent reviews (Gemini, Claude, ChatGPT) agree:
- On real Windows ARM64, kernel-mode `Zw*` calls are **inline thunks** that set PreviousMode and call the Nt* function directly — they do NOT go through SVC
- SVC (`svc #0`) should only be used for **user-mode → kernel-mode transitions**
- On x86/amd64, `int 0x2E` / `syscall` trap handlers implicitly manage IRQL as part of the hardware exception entry, masking the issue. ARM64 SVC has no such implicit IRQL management.

## Revised Root Cause Analysis

### Why Bug 1 ≠ Bug 2

Bug 1 is a `captures(none)` attribute propagation issue — well-understood, has workaround, would be fixed by stripping the attribute.

Bug 2 is an **architecture-level** problem with how ARM64 `Zw*` stubs work. Attempts to fix it by adding IRQL management to the SVC handler all failed — not because LLVM "deletes arbitrary opaque C calls," but because the approach is architecturally wrong. Lowering IRQL in the generic SVC handler would mask violations and risk breaking invariants for callers that are legitimately at DISPATCH.

The earlier diagnosis ("LLVM eliminates extern C calls under CGU=1") was too broad. The more likely explanation is one of:
1. The IRQL helper has `memory(none)` or `readnone` attributes in LLVM IR, making the elimination legitimate from LLVM's perspective
2. LLVM proves the store to `HalpCurrentIrql` (in the HAL DLL) is not observable from the Rust module and eliminates it
3. The entire issue is moot — the fix shouldn't be "lower IRQL in SVC handler" but "don't use SVC for kernel-mode Zw* calls"

## Recommended Fix: Direct Zw* Dispatch for Kernel Mode

Instead of `svc #0`, ARM64 kernel-mode `Zw*` stubs should:

```
ZwFlushVirtualMemory:
    // Save PreviousMode, set to KernelMode
    mrs x9, tpidr_el1           // get KPCR/KTHREAD
    ldrb w10, [x9, #PreviousMode_offset]  // save old mode
    strb wzr, [x9, #PreviousMode_offset]  // set KernelMode (0)

    // Call Nt* function directly (no SVC, no IRQL change)
    bl NtFlushVirtualMemory

    // Restore PreviousMode
    strb w10, [x9, #PreviousMode_offset]
    ret
```

This matches the Windows NT architecture: `Zw*` from kernel mode = direct call with PreviousMode set to KernelMode. No trap, no IRQL transition, no SVC overhead.

**Implementation options:**
1. Generate ARM64 `Zw*` thunks in assembly (`.S` file) — cleanest, LLVM can't touch it
2. Generate them as `global_asm!` in Rust — same effect, stays in Rust source
3. Generate them as C functions with `__attribute__((naked))` — Clang handles ARM64 naked functions

This fix would simultaneously:
- Eliminate the IRQL assertion failure (no SVC = no IRQL context switch)
- Fix the performance overhead of going through SVC for kernel calls
- Match the NT architecture more closely
- Be immune to LLVM optimization (assembly is not optimized)

## Debugging Leads Still Open

### Pre-opt vs Post-opt IR Comparison
Use `-C no-prepopulate-passes --emit=llvm-ir` to see the IR before LLVM's optimization pipeline, then compare with the normal `--emit=llvm-ir` output. This would reveal:
- Whether IRQL helper calls have wrong attributes (`memory(none)`, `readnone`, `speculatable`)
- Whether the call elimination is a legitimate optimization from LLVM's perspective
- Whether `captures(none)` is even the right attribute to blame

### ASM Memory Clobber Barrier
`core::arch::asm!("", options(nostack))` WITHOUT `nomem` forces LLVM to treat all memory as potentially modified. This is stronger than `black_box` (which Rust docs explicitly say is best-effort only). Worth testing as a lighter alternative to the LLVM pass:
```rust
// Before IRQL-sensitive call
core::arch::asm!("", options(nostack)); // memory clobber
KeLowerIrql(PASSIVE_LEVEL);
core::arch::asm!("", options(nostack)); // memory clobber
```

### CGU=16 Idle-Out Investigation
The CGU=16 workaround gets past PspCreateThread but SMSS never starts. This is likely another `captures(none)` victim in the Phase1 init path — some thread state transition or APC delivery is being silently eliminated. Adding IRQL/state traces between IoInitSystem return and the SMSS creation point would identify which call is being lost.

## Resolution Priority

| Priority | Action | Fixes |
|----------|--------|-------|
| **1** | Implement direct Zw* dispatch for ARM64 kernel mode | Bug 2 (architectural fix) |
| **2** | Custom LLVM pass to strip `captures(none)` from declarations | Bug 1 + CGU=16 idle-out + future captures bugs |
| **3** | Enable CGU=1 with coordinated workaround revert | Eliminates all cross-CGU issues |
| **4** | Audit LLVM IR for wrong attributes on IRQL helpers | Confirms/denies "LLVM deletes C calls" theory |

With Priority 1 done, CGU=1 should boot ARM64 to SMSS. With Priority 2, CGU=1 becomes reliable. With Priority 3, CGU=1 becomes production.

## How to Reproduce

```bash
cd /home/curi/Projects/os-playground/vibeos

# Bug 1 workaround (CGU=16, current production):
# ARM64 boots but SMSS doesn't start — idle detection passes, black screen
.venv/bin/python -m pytest tests/qemu/test_smoke.py -v --arch arm64 --timeout 300

# Bug 2 (CGU=1):
sed -i 's/codegen-units = 16/codegen-units = 1/' sdk/lib/rust/Cargo.toml
cd build-arm64 && rm -rf ntoskrnl/rust-kernel-target/ ntoskrnl/libkernel_rust.a
ninja ntoskrnl
cd .. && mcopy -o -i qemu/ramdisk.img build-arm64/ntoskrnl/ntoskrnl.exe ::/reactos/system32/ntoskrnl.exe
mcopy -o -i qemu/boot.img qemu/ramdisk.img ::/ramdisk.img
.venv/bin/python -m pytest tests/qemu/test_smoke.py -v --arch arm64 --timeout 300
# IMPORTANT: revert after
sed -i 's/codegen-units = 1/codegen-units = 16/' sdk/lib/rust/Cargo.toml
```

## Instrumentation In Tree (uncommitted)

- `ntoskrnl/mm/mm_rust_shims.c`: UART-based IRQL checks in MiLockProcessWorkingSet, MiLockProcessWorkingSetShared, MiUnlockProcessWorkingSet, MiReleaseExpansionLock, plus `MmDbgCheckIrql` utility, plus `MmSvcLowerIrql` helper
- `sdk/lib/rust/ntoskrnl-rust/src/ARM3/procsup.rs`: `MmDbgCheckIrql` calls at MmInitializeProcessAddressSpace and MmCreateProcessAddressSpace boundaries
- `sdk/lib/rust/ntoskrnl-rust/src/ps/process.rs`: `MmDbgCheckIrql` calls at KeInitializeProcess, PspInitializeProcessSecurity, ObInitProcess boundaries
- Various Codex debug traces in ps/thread.rs, ke/arm64/stubs.rs, ex/init.rs

## Reference Files

| File | Description |
|------|-------------|
| `/home/curi/Projects/os-playground/vibeos/AGENTS.md` | Project instructions |
| `/home/curi/.claude/projects/-home-curi-Projects-os-playground-vibeos/memory/MEMORY.md` | Memory index |
| `/home/curi/Projects/os-playground/vibeos/sdk/lib/rust/ntoskrnl-rust/src/ke/arm64/trap.rs` | ARM64 SVC handler (KiSvcDispatch) — no IRQL management |
| `/home/curi/Projects/os-playground/vibeos/hal/halarm64/generic/halstubs.c` | HAL IRQL functions (KfRaiseIrql, KfLowerIrql, HalpCurrentIrql) |
| `/home/curi/Projects/os-playground/vibeos/ntoskrnl/mm/mm_rust_shims.c` | MM shims with UART IRQL instrumentation |
| `/home/curi/Projects/os-playground/vibeos/sdk/lib/rust/ntoskrnl-rust/src/ARM3/procsup.rs` | MmInitializeProcessAddressSpace / MmCreateProcessAddressSpace |
| `/home/curi/Projects/os-playground/vibeos/sdk/lib/rust/ntoskrnl-rust/src/ps/process.rs` | PspCreateProcess with IRQL check points |
| `/home/curi/Projects/os-playground/vibeos/sdk/lib/rust/ntoskrnl-rust/src/ps/thread.rs` | PspCreateThread with `#[inline(never)]` workaround |
| `/home/curi/Projects/os-playground/vibeos/sdk/lib/rust/Cargo.toml` | codegen-units setting (line 37) |
| `/home/curi/Projects/os-playground/vibeos/ntoskrnl/ex/zw.S` | ARM64 Zw* stubs (current: all use `svc #0`) |
| `/home/curi/Projects/os-playground/vibeos/docs/engineering/shim-elimination.md` | Shim elimination plan with CGU=1 analysis |
| `/home/curi/Projects/os-playground/vibeos/~temp-ai-chats.md` | External AI reviews of this bug |
