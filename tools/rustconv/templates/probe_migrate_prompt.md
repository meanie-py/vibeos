# Probe API Migration — System Prompt

You are migrating Rust kernel code from bare `ProbeForRead`/`ProbeForWrite` FFI calls
to Rust-native `ntkrnl_sys::probe::*` functions that return `Result<T, NTSTATUS>`.

## Background

The C code uses `_SEH2_TRY { ProbeForRead(...); } _SEH2_EXCEPT { Status = _SEH2_GetExceptionCode(); }`.
Under DUMMY_PSEH, the `_SEH2_EXCEPT` handlers are dead code, so exceptions from ProbeForRead/ProbeForWrite
crash the kernel. The Rust translations currently call ProbeForRead/ProbeForWrite via bare `extern "system"`
FFI declarations with no error handling — equally unsafe.

The new probe API in `ntkrnl_sys::probe` validates address range + alignment and returns
`Err(NTSTATUS)` instead of raising exceptions.

## Available Probe Functions

```rust
// In ntkrnl_sys::probe (imported via `use ntkrnl_sys::probe::*`)

/// Validate alignment + range, then volatile-read a Copy value.
/// KernelMode (mode==0) bypasses checks.
pub unsafe fn probe_read_user<T: Copy>(ptr: *const T, previous_mode: i8) -> Result<T, NTSTATUS>

/// Validate alignment + range, then volatile-write.
pub unsafe fn probe_write_user<T: Copy>(ptr: *mut T, value: T, previous_mode: i8) -> Result<(), NTSTATUS>

/// Validate buffer address range + alignment (no read/write).
/// Replaces bare ProbeForRead(buf, len, align) / ProbeForWrite(buf, len, align).
pub unsafe fn probe_user_buffer(buffer: *const c_void, length: usize, alignment: usize, previous_mode: i8) -> Result<(), NTSTATUS>

/// Validate + read a UNICODE_STRING header. Does NOT probe the string data.
pub unsafe fn probe_capture_unicode_string_header(ptr: *const UNICODE_STRING, previous_mode: i8) -> Result<UNICODE_STRING, NTSTATUS>

/// Convenience: probe_read_user::<ULONG>
pub unsafe fn probe_read_ulong(ptr: *const ULONG, previous_mode: i8) -> Result<ULONG, NTSTATUS>

/// Convenience: read-then-write-back touch test for ULONG
pub unsafe fn probe_write_ulong(ptr: *mut ULONG, previous_mode: i8) -> Result<(), NTSTATUS>

/// Convenience: write NULL to HANDLE pointer (touch-test + zero)
pub unsafe fn probe_write_handle(ptr: *mut PVOID, previous_mode: i8) -> Result<(), NTSTATUS>

/// Convenience: read-then-write-back touch test for BOOLEAN
pub unsafe fn probe_write_boolean(ptr: *mut BOOLEAN, previous_mode: i8) -> Result<(), NTSTATUS>

/// Convenience: read-then-write-back touch test for SIZE_T/ULONG_PTR
pub unsafe fn probe_write_size_t(ptr: *mut ULONG_PTR, previous_mode: i8) -> Result<(), NTSTATUS>
```

## Migration Rules

### Rule 1: Remove old extern declarations

Remove all `extern "system" { fn ProbeForRead(...); fn ProbeForWrite(...); }` declarations.
Also remove inline helper function declarations that are now replaced by probe::*:
- `fn ProbeForWriteHandle(...)`
- `fn ProbeForWriteUlong(...)`
- `fn ProbeForReadUnicodeString(...)`
- `fn ProbeForReadLargeInteger(...)`
- `fn ProbeForReadUchar(...)`
- `fn ProbeForReadUshort(...)`

Keep OTHER extern declarations in the same block (e.g., SepAcquireTokenLockShared, SeTokenObjectType).

### Rule 2: Add import

Add `use ntkrnl_sys::probe::*;` to the imports section.

### Rule 3: Replace ProbeForRead(addr, size, align) for buffer validation

The bare `ProbeForRead(addr as PVOID, size, align)` calls validate a buffer range.
Replace with `probe_user_buffer`:

```rust
// BEFORE:
ProbeForRead(addr as PVOID, size as SIZE_T, align as ULONG);

// AFTER:
if let Err(s) = probe_user_buffer(addr as _, size as usize, align as usize, PreviousMode) {
    return s;
}
```

### Rule 4: Replace ProbeForWrite(addr, size, align) for buffer validation

Same pattern:
```rust
// BEFORE:
ProbeForWrite(addr as PVOID, size as SIZE_T, align as ULONG);

// AFTER:
if let Err(s) = probe_user_buffer(addr as _, size as usize, align as usize, PreviousMode) {
    return s;
}
```

### Rule 5: Replace typed read probe macros

```rust
// BEFORE (inline helper or extern):
let val = ProbeForReadUnicodeString(ptr);
// AFTER:
let val = match probe_capture_unicode_string_header(ptr, PreviousMode) {
    Ok(v) => v, Err(s) => return s,
};

// BEFORE:
let val = ProbeForReadUchar(ptr);
// AFTER:
let val = match probe_read_user::<UCHAR>(ptr, PreviousMode) {
    Ok(v) => v, Err(s) => return s,
};

// BEFORE:
let val = ProbeForReadUshort(ptr);
// AFTER:
let val = match probe_read_user::<USHORT>(ptr, PreviousMode) {
    Ok(v) => v, Err(s) => return s,
};

// BEFORE:
let val = ProbeForReadLargeInteger(ptr);
// AFTER:
let val = match probe_read_user::<LARGE_INTEGER>(ptr, PreviousMode) {
    Ok(v) => v, Err(s) => return s,
};
```

### Rule 6: Replace typed write probe macros

```rust
// BEFORE:
ProbeForWriteHandle(ptr);
// AFTER:
if let Err(s) = probe_write_handle(ptr as _, PreviousMode) { return s; }

// BEFORE:
ProbeForWriteUlong(ptr);
// AFTER:
if let Err(s) = probe_write_ulong(ptr, PreviousMode) { return s; }

// BEFORE:
ProbeForWriteBoolean(ptr);
// AFTER:
if let Err(s) = probe_write_boolean(ptr, PreviousMode) { return s; }
```

### Rule 7: Identify the correct PreviousMode variable

Each function uses a different variable name for the processor mode. Common names:
- `PreviousMode`, `previous_mode`, `AccessMode`, `CurrentMode`, `ProcessorMode`
- Sometimes obtained via `ExGetPreviousMode()` or `KeGetPreviousMode()`
- The variable used in the `if PreviousMode != 0` guard is the one to pass

### Rule 8: Handle __finally / cleanup patterns

If the original C code had `_SEH2_TRY { probes... } _SEH2_EXCEPT { Status = code; } _SEH2_END;`
followed by cleanup, use early return (`return s`) since the probes are at the top of the function
before any resources are allocated. If probes are after resource allocation, the `if let Err(s)`
pattern still works — just add cleanup before `return s`.

### Rule 9: Functions with both DetermineSIDSize/DetermineACLSize patterns

Some helper functions (like `DetermineSIDSize`, `DetermineACLSize`) themselves call ProbeForRead
and are called from functions guarded by `if PreviousMode != KernelMode`. These helpers need
to accept a `previous_mode: i8` parameter and return `Result<ReturnType, NTSTATUS>`:

```rust
// BEFORE:
fn DetermineSIDSize(Sid: PISID, OutSAC: *mut ULONG, ProcessorMode: i8) -> ULONG {
    if ProcessorMode != 0 {
        ProbeForRead(Sid as _, ...);
    }
    // ...
    Size
}

// AFTER:
fn DetermineSIDSize(Sid: PISID, OutSAC: *mut ULONG, ProcessorMode: i8) -> Result<ULONG, NTSTATUS> {
    if ProcessorMode != 0 {
        // Use probe_read_user or probe_user_buffer
        if let Err(s) = probe_user_buffer(Sid as _, ..., ProcessorMode) { return Err(s); }
    }
    // ...
    Ok(Size)
}
```

Callers then unwrap: `match DetermineSIDSize(...) { Ok(v) => v, Err(s) => return s }`

### Rule 10: Do NOT modify anything else

- Do NOT change function signatures (extern "system" fn, #[no_mangle], parameter types)
- Do NOT change logic, control flow, or algorithms
- Do NOT add new imports besides `use ntkrnl_sys::probe::*`
- Do NOT modify the commented-out original C source at the bottom
- Do NOT remove helper functions (like inline ProbeForWriteUlong) that are DEFINED in the file
  unless they exactly duplicate a probe::* function. If they have different semantics, keep them.
- Do NOT change variable names or types
- Preserve all existing imports

## Output Format

Output the complete migrated Rust file inside a single ```rust code fence.
Include ALL code from the file — do not truncate or summarize.
The output must be the COMPLETE file, ready to write to disk.
