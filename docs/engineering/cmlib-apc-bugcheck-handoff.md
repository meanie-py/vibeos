# Handoff: i386 BUGCHECK 0x0A in the C-free cmlib kernel (boot-thread APC corruption)

**Status: RESOLVED (2026-06-11).** Root cause was `HvInitialize` calling
`core::ptr::write_bytes(Hive, 0, size_of::<HHIVE>())` with `Hive: *mut HHIVE` — Rust's
`write_bytes(dst, val, count)` writes `count * size_of::<T>()` bytes, so this zeroed
`size_of::<HHIVE>()` whole HHIVEs (~115 KB overrun) and smashed the adjacent boot-thread
APC state, producing exactly the `KernelApcPending`-set / NULL-`ApcListHead` corruption
diagnosed below → recursive trap-exit APC delivery → P0 boot-stack overflow → bugcheck.
Fix: `write_bytes(Hive as *mut u8, 0, size_of::<HHIVE>())`. Found + fixed by a Codex
`codex exec` handoff run; i386 + amd64 now boot C-free (clean-build smoke `verdict: OK`).
Secondary fixes in the same pass: CM_CHECK_REGISTRY_* / STATUS_REGISTRY_* constant values,
CmpAddToLeaf flex-array indexing, CmpValidateKey `assert!`→kernel `ASSERT`, and an
`ntkrnl-sys` `inline_exports` feature gate so leaf crates don't export KE inline helpers.
The historical hunt notes below are kept for the *method* (QEMU TCG `-d int` is the reliable
way to catch a too-early-for-gdb boot fault) and as a worked example of the
`write_bytes` element-vs-byte footgun in C→Rust translation.

---

**(historical — the open-hunt brief as handed to Codex):** The cmlib C→Rust translation is
COMPLETE and the kernel links genuinely C-free on i386 + amd64 (commit `85479bd0878`).
But i386 BUGCHECKs 0x0A very early in boot. Everything below is verified fact.

## The goal

Find why the i386 C-free build hits BUGCHECK 0x0A and fix it **without** reverting any
Rust translation (see AGENTS.md "Translation Philosophy"). The cmlib translation itself
is done; this is a link-time / KE-state regression exposed by the FIRST genuine clean
C-free build (the prior "100% Rust cmlib boots" commit `6a26940` was a false-green — its
on-disk `libcmlib.a` still bundled the 13 C objects, which `--allow-multiple-definition`
silently linked; a clean build of that state does not even link).

## Reproduce

```bash
cd /home/curi/Projects/os-playground/vibeos-wt-cohab
./vibeos-cli build --arch i386 --clean      # links C-free; verify: llvm-ar t build-i386/sdk/lib/cmlib/libcmlib.a == placeholder only
./vibeos-cli smoke --arch i386              # hangs at "ACPI v1.0-1.0b detected" (silent — it is actually a bugcheck)
```

The bugcheck is SILENT on serial (the BSOD/Kd path never flushes). The CPU parks in
`HaliHaltSystem` (`cli; hlt`) via KeBugCheckWithTf's `for(;;){DbgBreakPointWithStatus(4);
HalHaltSystem();}` loop. `KiBugCheckData` = `{0x0A, 0x18, 0xFFFFFFFF, 0x00000000, 0x804b01d2}`.

## The reliable diagnostic tool: QEMU TCG `-d int`

GDB breakpoints are USELESS here: the fault is microseconds after the kernel starts, and
KiInitializeKernel clears DR7 so any hardware breakpoint set during freeldr vanishes;
software breakpoints can't be inserted before the page is mapped. **Use QEMU TCG interrupt
logging instead** — it records every exception deterministically:

```bash
qemu-system-i386 -M pc -accel tcg -cpu max -m 512M \
  -drive if=pflash,format=raw,file=qemu/OVMF32_CODE.fd,readonly=on \
  -drive if=pflash,format=raw,file=qemu/runs/<any>/ovmf32_vars.fd \
  -drive id=cd0,if=none,format=raw,file=build-i386/livecd.iso,readonly=on \
  -device ide-cd,drive=cd0,bootindex=0 -vga std -no-reboot -no-shutdown \
  -serial file:/tmp/qint_serial.log -d int -D /tmp/qint.log -display none
# wait for "detected. Tables" in /tmp/qint_serial.log, then read the TAIL of /tmp/qint.log
```

Symbolicate: i386 reloc = **0x80091000** (PE_VA = runtime − 0x80091000; ntoskrnl load
base 0x80491000). `llvm-nm --numeric-sort --defined-only build-i386/ntoskrnl/ntoskrnl.exe`.
`llvm-*` tools live in `.build-tools/llvm-mingw/bin/`.

## Root cause (captured from the TCG int log — exact)

```
... normal demand-zero pool loop: v=0e e=0002 IP=804d1388 CR2 stepping e1010000,e1011000,... (healthy)
2448: v=0e e=0000 IP=8053335a CR2=00000010      <-- TRIGGER: KiDeliverApc+0xba (PE 0x4a235a)
2449..2486: v=0e IP=804a653d CR2=00000228 SP decreasing each line   <-- recursive faulting, stack draining
2487: v=0e e=0002 IP=804946a1 CR2=f7bb8ffc       <-- fault ON the stack pointer = stack overflow / guard page
2488: v=08 (DOUBLE FAULT) IP=804946a1 SP=f7bb8ffc EAX=804a653d
2489: v=0e IP=804b01d2 CR2=00000018              <-- #DF handler KiTrap08Handler+0x22 faults reading null+0x18 (=P1)
2490: v=03 (the bugcheck DbgBreakPointWithStatus loop)
```

`KiDeliverApc+0xba` disassembly (PE 0x4a235a):
```
4a234b: mov 0x28(%esi),%eax    ; eax = thread->ApcState.ApcListHead[Kernel].Flink
4a234e: cmp %edi,%eax          ; edi = &ApcListHead (empty-list sentinel)
4a2350: je  ...                ; if empty, skip — NOT taken (eax != edi)
4a2356: movb $0x0,0x3d(%esi)
4a235a: mov 0x10(%eax),%ecx    ; FAULT: eax==0 (CR2=0x10) -> reads [null+0x10]
```

**Diagnosis:** the boot thread's APC state is inconsistent — `KernelApcPending`
(thread+0x3e) is set, but `ApcState.ApcListHead[Kernel].Flink` (thread+0x28) is **NULL**
(should be self-linked for an empty list). `KiCheckForApcDelivery` sees the pending flag
and calls `KiDeliverApc`, which walks the null list and faults. The flag is never cleared
(the fault aborts before that), so `KiCheckForApcDelivery` re-fires on EVERY trap exit →
infinite recursive page-faulting → kernel stack overflow → #DF → bugcheck 0x0A.

NOTE: an earlier KVM gdb session pointed at `MmArmInitSystem -> MiInitializeLoadedModuleList
-> ExAllocatePoolWithTag`. That was a RED HERRING (a benign earlier fault). Trust the TCG
int log: the fatal trigger is `KiDeliverApc` at PE 0x4a235a.

## What is RULED OUT (do not re-investigate)

- **Stale artifacts** — a `--clean` build reproduces it.
- **The 43 raw-dylib `__imp_` thunks** cmlib_rust.a had for ntoskrnl statics — they all
  resolve to the real `D` data in the final binary (MmSystemRangeStart=0x80000000 etc.),
  not to thunks. (Fixed for hygiene anyway via Cargo.toml `default-features=false`.)
- **IRQL primitive leaks** — `_KfRaiseIrql@4`/`_KfLowerIrql@4` in cmlib_rust.a are correct
  stdcall→fastcall adapters (`mov ecx,arg; jmp @KfRaiseIrql@4`).
- **The loader** — uefildr/rosload build.rs diffs are comment-only; the cmake/ninja smoke
  build doesn't use the cargo loader crates anyway. freeldr markers all print fine.
- **MM/pool globals** — healthy at the crash: MmSystemRangeStart=0x80000000,
  NonPagedPoolStart=0xb047f000, free-list heads self-linked, 0x1ff57 physical pages.
- **KiDeliverApc / KiCheckForApcDelivery symbol collisions** — single real definition each
  in the final binary; cmlib's bindgen copies are real impls, not stubs, and were GC'd/deduped.

## The open question

cmlib RUNTIME code cannot execute this early (Phase 0, long before registry init), so the
cmlib→KE corruption MUST be link-time. Leading hypotheses to test:

1. **KTHREAD / KAPC_STATE layout skew** between cmlib-rust's ntkrnl-sys bindgen and the C
   kernel (an offset mismatch that mis-sets/mis-reads thread+0x28 vs +0x3e). Check the
   bindgen struct offsets for KTHREAD.ApcState / KAPC_STATE against the C headers
   (`ntkrnl-sys` has size/offset asserts — see `verify_sizes.py`).
2. ~~A pre-existing i386-branch bug exposed by clean build~~ **— PARTLY RESOLVED.**
   amd64 now links C-free and **ALSO bugchecks at the same boot stage** (right after
   "ACPI v1.0-1.0b detected"): `BUGCHECK 0x50 PAGE_FAULT_IN_NONPAGED_AREA
   (P1=0x1fc7c5, P2=0x10, P3=0xfffff8000063d2f0, P4=0)`. The faulting address P3 maps to
   **`KiP0BootStackData`** (the P0 boot stack, defined in `ntoskrnl_rust::ke::amd64::kiinit`)
   — i.e. amd64 is **also overflowing the P0 boot stack**, same mechanism as i386, and the
   `0x10` offset recurs. **So the bug is SYSTEMIC to the cmlib flip, NOT i386-specific
   layout/ABI.** Whatever cmlib-rust's *presence* changes, it breaks early KE boot-thread/
   APC setup on BOTH arches. (amd64 reloc: imagebase 0x400000; load base 0xfffff80000000000.)
   This makes the strongest lead: **the cmlib flip changed the kernel binary/link enough to
   expose (or cause) an uninitialized/corrupt boot-thread APC state.** Since the false-green
   never clean-linked, this code path was never exercised before. Look for: what the
   C-free link changes vs the C-masked one (symbol selection under --allow-multiple-definition,
   init/.CRT order, a global that KiInitialThread/KeInitThread depends on), AND whether
   KiInitialThread's KAPC_STATE is actually self-linked at the point KiCheckForApcDelivery
   first runs.
3. **Who sets KernelApcPending with a null ApcListHead?** Instrument with TCG: watch writes
   to the boot thread's +0x3e (pending flag) and +0x28 (list Flink). Find the code that sets
   the flag without linking the list, or that zeroes the list after the flag is set. The boot
   thread is KiInitialThread (a static in ntoskrnl .data); KeInitThread should self-link the
   APC lists. Check whether KeInitThread ran / wrote the right offsets for this thread.

## Dangling compiler-rt builtins (separate, lower priority)

cmlib_rust.a has undefined `___ashldi3`/`___lshrdi3`/`___muldi3` (64-bit int) and
`___addtf3`/`___multf3`/`___subtf3`/... (**128-bit float, TFmode**) + `__compilerrt_abort_impl`.
These are GC'd on i386 (in dead code) so they don't affect THIS bug, but the 128-bit-float
ones smell like a translation bug (cmlib has no business doing f128 math) and would crash
cmlib at RUNTIME if a live fn used them. Worth a grep once the boot bug is fixed.

## Key files

- Translation crate: `sdk/lib/rust/cmlib-rust/src/` (cohab worktree).
- Cohab CMake flip (committed): `sdk/lib/cmlib/CMakeLists.txt` (CMLIB_RUST_FLIPPED + placeholder TU).
- Full prior context + every dead-end: agent memory `project_cmlib_rust_track.md`
  ("ROOT BUG #2"), and AGENTS.md (translation philosophy, smoke = north star, vibeos-cli).
