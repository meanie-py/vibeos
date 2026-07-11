# PSEH Elimination Plan

> **ARCHIVED 2026-07-10 — superseded in direction by
> [dummy-pseh-verdict.md](../dummy-pseh-verdict.md)** (2026-07-06), which rules: keep
> DUMMY_PSEH as the C default, give Rust the three PSEH semantics idiomatically
> (validate-first probes / try_* contract conversions / RAII Drop), native SEH as a
> per-module tool. Also stale here: "the unwinder is broken"-era claims — Rust frames now
> ship real `.pdata`/`.xdata` (`requires-uwtable`) and the unwinder works (field manual
> §9.35). Kept as the historical layer map the verdict built on.


## Background

PSEH is ReactOS's structured exception handling (SEH) implementation. Windows NT uses `__try`/`__except`/`__finally` blocks pervasively — in the kernel, drivers, win32k, and user-mode DLLs — to catch hardware exceptions (access violations, divide-by-zero) and software exceptions (`ExRaiseStatus`). The kernel uses SEH primarily for safely probing user-mode memory and recovering from faults in protected code paths.

Our llvm-mingw (Clang 19) toolchain can't use any of the existing PSEH implementations correctly, so we use DUMMY_PSEH — which makes all `_SEH2_EXCEPT` handlers dead code. This means any code that relies on catching exceptions is silently broken.

## Current PSEH Implementations

### PSEH3 (original, GCC x86 only)
Uses the x86 exception chain (`fs:[0]`). The filter function accesses the enclosing function's local variables via ESP-relative offsets. **Completely broken with Clang** because Clang uses EBP-relative locals, so the filter can't find the right variables. Not viable.

### PSEH_CLANG (i386 only)
Setjmp/longjmp-based replacement. Uses `setjmp` to save state before the `_SEH2_TRY` block, then `longjmp` back if an exception occurs. The filter evaluates in the original function context. **Only exists for i386** — amd64 and ARM64 don't have an equivalent. Partially viable but single-arch.

### DUMMY_PSEH (all arches, current default)
Makes `_SEH2_EXCEPT` handlers dead code. The `_SEH2_TRY` body executes normally, but if an exception occurs, there's no handler to catch it — the system crashes or the exception propagates unhandled.

**Consequences of DUMMY_PSEH visible in our bug history:**
- `RtlRaiseException` swallowing fixes on amd64/i386 (exception dispatcher found no handler → process killed)
- `KiUserExceptionDispatcher` defense-in-depth patches (swallow software exceptions below 0xC0000000)
- `DbgPrint` output silently eaten (debug exception handler was dead code)
- Every user-mode buffer probe needs pre-validation instead of fault-and-catch
- The Rust `probe.rs` API was built specifically to work around DUMMY_PSEH

## Key Architectural Insight: SEH Does Not Cross Module Boundaries

Each module (ntoskrnl.exe, win32k.sys, drivers, DLLs) registers its own exception handlers on its own stack frames. Exception handlers are **module-local** — a driver's `__try`/`__except` catches exceptions in the driver's code, not in the kernel's code.

The kernel's role is **exception dispatching**, not exception handling:

```
Hardware fault → trap handler → KiDispatchException
  → walks exception handler chain/tables on the stack
  → each handler belongs to the MODULE that installed it
  → finds handler → calls it → resumes or unwinds
  → no handler found → bugcheck (kernel) or kill process (user)
```

The dispatcher reads handler registrations from the faulting module's PE image:
- **amd64/ARM64:** Table-based (`.pdata` + `.xdata` sections in each PE). The dispatcher reads the faulting module's PE headers to find handler info. Pure data lookup — no asm tricks needed.
- **i386:** Chain-based (`fs:[0]` exception registration records on the stack). Each `__try` pushes a record; the dispatcher walks the chain.

**This means:** The kernel can stop USING SEH entirely while still correctly DISPATCHING exceptions for drivers, win32k, and user-mode code. KiDispatchException (already in Rust, `ke/except.rs`) is the infrastructure — it doesn't need SEH itself.

## The Four Layers

### Layer 1: ntoskrnl Rust Code — DONE
**Status:** Uses `Result<T, NTSTATUS>` via `probe.rs` API. No SEH needed.

The Rust probe API replaces bare `ProbeForRead`/`ProbeForWrite`:
```rust
probe_read_user::<T>(ptr, mode) -> Result<T, NTSTATUS>
probe_write_user::<T>(ptr, value, mode) -> Result<(), NTSTATUS>
probe_user_buffer(buf, len, align, mode) -> Result<(), NTSTATUS>
```

SE (12 files, 62 probe sites) and OB (6 files, 35 probe sites) are fully migrated. The `rustconv probe-migrate` pipeline automates this transformation.

### Layer 2: ntoskrnl Remaining C Code — 14 Files, ~120 Try Blocks
**Status:** DUMMY_PSEH makes all handlers dead code. Some of these are silently broken.

Active C files with `_SEH2_TRY`:
| File | Try blocks | Purpose |
|------|-----------|---------|
| io/iomgr/iofunc.c | 39 | Nt* syscall user-buffer probing |
| mm/ARM3/virtual.c | 28 | Virtual memory user-buffer probing |
| config/ntapi.c | 16 | Registry Nt* syscall probing |
| config/cmapi.c | 8 | Registry CM API probing |
| ex/sysinfo.c | 7 | NtQuerySystemInformation probing |
| lpc/connect.c | 4 | LPC port connection probing |
| kd64/kdprint.c | 3 | Debugger print probing |
| ps/i386/psldt.c | 2 | LDT descriptor probing (i386 only) |
| kd64/i386/kdx86.c | 2 | Debugger context probing |
| kd64/amd64/kdx64.c | 2 | Debugger context probing |
| kdbg/kdb_cli.c | 1 | Debugger CLI |
| kd64/kdtrap.c | 1 | Debugger trap handling |
| config/cmsysini.c | 1 | Registry init |

**Elimination path:** Two options, can be mixed per-file:

**Option A — Translate to Rust:** Use rustconv/Codex to translate the file to Rust with `Result<T>` for every SEH block. This is the north star and already proven for ~300 files.

**Option B — Explicit validation in C:** Replace the `_SEH2_TRY`/`_SEH2_EXCEPT` with explicit pointer validation before access. This is a mechanical transformation:
```c
// Before (DUMMY_PSEH — handler is dead code):
_SEH2_TRY {
    ProbeForRead(UserBuffer, Length, 1);
    RtlCopyMemory(KernelBuffer, UserBuffer, Length);
} _SEH2_EXCEPT(EXCEPTION_EXECUTE_HANDLER) {
    Status = _SEH2_GetExceptionCode();
} _SEH2_END;

// After (explicit validation):
Status = MmValidateUserBuffer(UserBuffer, Length, 1);
if (NT_SUCCESS(Status)) {
    RtlCopyMemory(KernelBuffer, UserBuffer, Length);
}
```

### Layer 3: KiDispatchException — The Dispatcher — DONE
**Status:** Already in Rust (`ke/except.rs`). Correctly dispatches exceptions to other modules.

The dispatcher doesn't USE SEH — it IS the SEH infrastructure. It reads exception handler registrations from the faulting module and calls them. No changes needed for the kernel to stop using SEH.

Architecture-specific dispatch:
- **amd64:** Table-based. Reads `.pdata`/`.xdata` from PE image. Pure data lookup.
- **ARM64:** Table-based. Same mechanism as amd64.
- **i386:** Chain-based. Walks `fs:[0]` exception registration records.

All three work correctly today for dispatching to drivers and user-mode code.

### Layer 4: win32k, Drivers, DLLs — 245 Files With SEH
**Status:** DUMMY_PSEH makes all handlers dead code. These modules are compiled with our Clang toolchain.

| Module | Files with SEH | Long-term plan |
|--------|---------------|----------------|
| win32k | 86 | Translate to Rust |
| drivers | 112 | Translate to Rust |
| DLLs | 47 | Translate to Rust |

**Short-to-medium term: Reverse shims.** Instead of fixing PSEH for C, shift the exception-sensitive work into Rust functions that the C code calls:

```c
// C code (win32k, driver, DLL):
// Instead of __try/__except around a probe:
Status = RustSafeCopyFromUser(KernelBuffer, UserBuffer, Length);
if (!NT_SUCCESS(Status)) {
    // handle error — no SEH needed
}

// Rust side:
#[no_mangle]
pub unsafe extern "system" fn RustSafeCopyFromUser(
    dst: PVOID, src: PVOID, len: ULONG
) -> NTSTATUS {
    match probe_user_buffer(src, len, 1, UserMode) {
        Ok(()) => {
            core::ptr::copy_nonoverlapping(src as *const u8, dst as *mut u8, len as usize);
            STATUS_SUCCESS
        }
        Err(status) => status,
    }
}
```

This pattern:
- Works today with the existing Rust probe API
- Doesn't require any PSEH changes
- Can be adopted incrementally per-file
- The C code becomes a simple status check — no exception handling needed
- Eventually the entire C file gets translated to Rust anyway

**Building the reverse shim library:** A `safe_user_io` Rust crate could provide:
- `RustSafeCopyFromUser(dst, src, len) -> NTSTATUS`
- `RustSafeCopyToUser(dst, src, len) -> NTSTATUS`
- `RustSafeProbeRead(ptr, len, alignment) -> NTSTATUS`
- `RustSafeProbeWrite(ptr, len, alignment) -> NTSTATUS`
- `RustSafeCaptureUnicodeString(ptr) -> NTSTATUS` (with output param)
- `RustSafeProbeAndLockPages(mdl, mode, operation) -> NTSTATUS`

These cover 90%+ of SEH usage patterns in win32k/drivers/DLLs.

## Priority Order

| Priority | Layer | Work | Impact |
|----------|-------|------|--------|
| 1 | ntoskrnl C (14 files) | Translate to Rust or explicit validation | Eliminates silent DUMMY_PSEH bugs in kernel |
| 2 | Reverse shim library | Build `safe_user_io` Rust crate | Enables win32k/driver SEH elimination without full rewrite |
| 3 | win32k (86 files) | Replace `_SEH2_TRY` with reverse shim calls | Critical for desktop rendering — most complex |
| 4 | Drivers (112 files) | Replace `_SEH2_TRY` with reverse shim calls | Needed for device functionality |
| 5 | DLLs (47 files) | Replace `_SEH2_TRY` with reverse shim calls | Needed for user-mode stability |
| 6 | Full rewrite | Translate all C modules to Rust | North star — eliminates the problem entirely |

## Clang as a Temporary Measure

The llvm-mingw toolchain is a pragmatic choice for cross-compilation. When we control the entire codebase (which we increasingly do), the PSEH problem goes away naturally:
- Rust code uses `Result<T>` — no SEH needed
- Remaining C code calls Rust reverse shims — no SEH needed
- The exception DISPATCHING mechanism works regardless — it's infrastructure, not a consumer of SEH
- Third-party drivers (if we ever support them) would use MSVC or their own toolchain, which has real SEH support

## Reference Files

| File | Description |
|------|-------------|
| `AGENTS.md` | Project instructions, DUMMY_PSEH documentation |
| `sdk/lib/rust/ntkrnl-sys/src/probe.rs` | Rust probe API (Result-returning) |
| `sdk/lib/rust/ntoskrnl-rust/src/ke/except.rs` | KiDispatchException (Rust) |
| `sdk/lib/pseh/` | PSEH implementations (pseh3, pseh_clang, dummy) |
| `sdk/include/reactos/probe.h` | C probe macros |
| `tools/rustconv/` | C-to-Rust translation pipeline |
| `ntoskrnl/ntos.cmake` | Active C vs Rust file selection |
