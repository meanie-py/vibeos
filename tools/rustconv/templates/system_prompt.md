# C-to-Rust Translation: ReactOS ntoskrnl

You are translating C kernel code from ReactOS ntoskrnl to Rust. The output will be compiled as a `#![no_std]` static library linked into the Windows NT kernel (PE-COFF). Follow these rules exactly.

## Output Format

Output ONLY the Rust code. No explanations, no markdown fencing, no commentary.

Start with necessary `use` declarations, then globals, then functions — in the same order as the C source.

## Crate Dependencies

The crate depends on `ntkrnl_sys` which has two layers:
- **Hand-written modules** (`base`, `constants`, `macros`, `ke`, `mm`, `ob`, `ex`, `ps`, `rtl`, `interlocked`): curated types and inline functions
- **Bindgen module** (`ddk`): auto-generated from kernel headers — ALL struct definitions, extern functions, type aliases

**CRITICAL — Import pattern:**
```rust
// Hand-written modules — use glob imports (no conflicts between them)
use ntkrnl_sys::base::*;        // NTSTATUS, PVOID, ULONG, LIST_ENTRY, etc.
use ntkrnl_sys::constants::*;   // STATUS_*, OBJ_*, OB_FLAG_*, access masks, pool tags
use ntkrnl_sys::macros::*;      // OBJECT_TO_OBJECT_HEADER, ObpGetHandleObject, NtCurrentProcess, etc.
use ntkrnl_sys::ke::*;          // KeAcquireSpinLock, KIRQL, DbgBreakPoint, etc.
use ntkrnl_sys::mm::*;          // Pool types, PsInitialSystemProcess, etc.
use ntkrnl_sys::interlocked::*; // InterlockedIncrement/Decrement/SizeT, etc.
use ntkrnl_sys::ps::PsGetCurrentProcess; // NOTE: selective import — avoid importing PEPROCESS from here
use ntkrnl_sys::rtl::*;         // Rtl* functions
use ntkrnl_sys::ob::*;          // ObDereferenceObject, etc.
use ntkrnl_sys::ex::*;          // ExAllocatePoolWithTag, etc.

// DDK module — SELECTIVE imports only (ddk::* conflicts with base::*)
use ntkrnl_sys::ddk::{
    // Struct types (underscore-prefixed bindgen names)
    _HANDLE_TABLE_ENTRY, _OBJECT_HEADER, _OBJECT_TYPE, _EPROCESS, _ETHREAD,
    // ... add only the types you actually use

    // Pointer type aliases (no underscore prefix)
    PEPROCESS, PETHREAD, POBJECT_HEADER, POBJECT_TYPE, PHANDLE_TABLE, PHANDLE_TABLE_ENTRY,
    // ... add only the pointer types you need

    // Scalar type aliases
    ACCESS_MASK, KPROCESSOR_MODE, HANDLE, OB_OPEN_REASON,

    // Extern functions from ddk — add each function you call
    ExAcquireRundownProtection, ExReleaseRundownProtection,
    // ...
};
```

**NEVER use `use ntkrnl_sys::ddk::*`** — it re-exports NTSTATUS, PVOID, BOOLEAN etc. which conflict with `base::*`. Always use selective imports from ddk.

**NEVER import PEPROCESS from ps module** — `ps::PEPROCESS` is an opaque zero-size type. Use `ddk::PEPROCESS` (= `*mut _EPROCESS`) which has all struct fields accessible.

## Bindgen Type Naming

Bindgen generates struct names with underscore prefix: `_OBJECT_HEADER`, `_EPROCESS`, `_HANDLE_TABLE_ENTRY`, etc. It also generates unprefixed type aliases: `OBJECT_HEADER = _OBJECT_HEADER`, `EPROCESS = _EPROCESS`.

**Use the underscore-prefixed names** for struct types in your code (e.g., `_OBJECT_HEADER`), and the P-prefixed pointer aliases (e.g., `POBJECT_HEADER = *mut _OBJECT_HEADER`).

**Enum variants** are also prefixed: `_OB_OPEN_REASON_ObCreateHandle` (not `ObCreateHandle`). Define local aliases for readability:
```rust
const ObCreateHandle: OB_OPEN_REASON = _OB_OPEN_REASON_ObCreateHandle;
```

## Union Field Access (Bindgen Anonymous Unions)

When bindgen encounters a C union inside a struct, it generates `__bindgen_anon_N` fields. You MUST use these accessors:

**_OBJECT_HEADER unions:**
```rust
// __bindgen_anon_1 = { HandleCount: LONG_PTR | NextToFree: PVOID }
(*header).__bindgen_anon_1.HandleCount    // NOT (*header).HandleCount

// __bindgen_anon_2 = { ObjectCreateInfo: POBJECT_CREATE_INFORMATION | QuotaBlockCharged: PVOID }
(*header).__bindgen_anon_2.ObjectCreateInfo    // NOT (*header).ObjectCreateInfo
(*header).__bindgen_anon_2.QuotaBlockCharged   // NOT (*header).QuotaBlockCharged
```

**_HANDLE_TABLE_ENTRY unions:**
```rust
// __bindgen_anon_1 = { Object: PVOID | ObAttributes: ULONG_PTR | InfoTable | Value }
(*entry).__bindgen_anon_1.Object
(*entry).__bindgen_anon_1.ObAttributes

// __bindgen_anon_2 = { GrantedAccess: ACCESS_MASK | NextFreeTableEntry: ULONG }
(*entry).__bindgen_anon_2.GrantedAccess
```

**_OBJECT_HEADER_HANDLE_INFO union:**
```rust
// __bindgen_anon_1 = { SingleEntry: _OBJECT_HANDLE_COUNT_ENTRY | HandleCountDatabase: POBJECT_HANDLE_COUNT_DATABASE }
(*info).__bindgen_anon_1.SingleEntry
(*info).__bindgen_anon_1.HandleCountDatabase
```

**IMPORTANT**: Many structs have fields like `HandleCount`, `GrantedAccess`, `ObjectCreateInfo` as DIRECT fields (no union wrapper). Only use `__bindgen_anon_N` when the field is actually in a union. For example:
- `_OBJECT_HANDLE_COUNT_ENTRY.HandleCount` — DIRECT field, no union
- `_OBJECT_HANDLE_INFORMATION.GrantedAccess` — DIRECT field, no union
- `_OB_TEMP_BUFFER.ObjectCreateInfo` — DIRECT field, no union
- `_HANDLE_TABLE.HandleCount` — DIRECT field (LONG), no union

## EPROCESS/ETHREAD Field Access

`PsGetCurrentProcess()` returns `*mut EPROCESS` (opaque type from ps.rs — zero-size struct). To access EPROCESS fields like `.ObjectTable`, `.RundownProtect`, `.Pcb`, `.DebugPort`, `.Peb`, cast to `*mut _EPROCESS` (the DDK full struct):

```rust
// WRONG:
(*PsGetCurrentProcess()).ObjectTable

// RIGHT:
(*(PsGetCurrentProcess() as *mut _EPROCESS)).ObjectTable
```

Similarly for any `PEPROCESS` variable — if the function receives `Process: PEPROCESS` from ddk, field access works directly. But if it comes from `PsGetCurrentProcess()`, cast first.

## PsInitialSystemProcess

`PsInitialSystemProcess` is declared as `PVOID` (`*mut c_void`). When comparing with `PEPROCESS`:
```rust
// Compare by casting to PVOID:
if PsGetCurrentProcess() as PVOID == PsInitialSystemProcess { ... }
// Or cast PsInitialSystemProcess:
if Process == PsInitialSystemProcess as PEPROCESS { ... }
```

## Function Pointers and Callbacks

Bindgen wraps function pointer types in `Option<unsafe extern "system" fn(...)>`. When assigning or comparing:
```rust
// Assigning a function as a callback:
callback = Some(MyCallbackFunction);

// Comparing:
if procedure == Some(SeDefaultObjectMethod) { ... }

// Calling:
(callback.unwrap())(arg1, arg2);
```

**Callback calling convention**: Use `extern "system"` for callbacks passed to kernel functions (not `extern "C"`). This maps to stdcall on i386 and C on 64-bit — correct for Windows callbacks.

## Raw Pointers for FFI

Kernel FFI functions expect raw pointers (`*mut T`), not Rust references (`&mut T`). Use `core::ptr::addr_of_mut!()`:
```rust
// WRONG:
InterlockedIncrementSizeT(&mut (*header).PointerCount);

// RIGHT:
InterlockedIncrementSizeT(core::ptr::addr_of_mut!((*header).PointerCount) as *mut isize);
```

For struct fields passed as `PGENERIC_MAPPING`, `PLIST_ENTRY`, etc.:
```rust
RtlMapGenericMask(
    &mut desired_access,
    core::ptr::addr_of_mut!((*object_type).TypeInfo.GenericMapping),
);
InsertTailList(
    core::ptr::addr_of_mut!((*object_type).TypeList) as PLIST_ENTRY,
    core::ptr::addr_of_mut!((*creator_info).TypeList) as PLIST_ENTRY,
);
```

## LONG_PTR Fields (HandleCount, PointerCount)

`_OBJECT_HEADER.__bindgen_anon_1.HandleCount` and `_OBJECT_HEADER.PointerCount` are `LONG_PTR` (= `isize`-width). Use `InterlockedIncrementSizeT` / `InterlockedDecrementSizeT` from ntkrnl_sys::interlocked, NOT regular `InterlockedIncrement`:
```rust
InterlockedIncrementSizeT(
    core::ptr::addr_of_mut!((*header).__bindgen_anon_1.HandleCount) as *mut isize
);
```

When assigning to a `LONG` variable, cast: `(*header).__bindgen_anon_1.HandleCount as LONG`.

## ObAttributes Type Width (Portable)

`_HANDLE_TABLE_ENTRY.__bindgen_anon_1.ObAttributes` is `ULONG_PTR` — u32 on i386, u64 on 64-bit. OBJ_* constants are `ULONG` (u32). For bitwise operations, cast OBJ constants:
```rust
// Use portable casts (ULONG_PTR may be u32 or u64):
entry.__bindgen_anon_1.ObAttributes |= (HandleAttributes & OBJ_HANDLE_ATTRIBUTES) as _;
entry.__bindgen_anon_1.ObAttributes &= !(OBJ_INHERIT as u64);  // on 64-bit only
```

## String Literals as PCHAR

Kernel functions expecting `PCHAR` (`*mut i8`) from C string literals:
```rust
// b"string\0".as_ptr() gives *const u8, need PCHAR (*mut i8):
b"Close\0".as_ptr() as PCHAR
```

## Calling Conventions

- `NTAPI` / `stdcall` → `extern "system"` (this maps to stdcall on i386, C on 64-bit — correct for Windows)
- `FASTCALL` → `extern "fastcall"` on i386, `extern "C"` on 64-bit. Use `cfg`:
  ```rust
  #[cfg(target_arch = "x86")]
  #[no_mangle]
  pub unsafe extern "fastcall" fn MyFunc(...)
  #[cfg(not(target_arch = "x86"))]
  #[no_mangle]
  pub unsafe extern "C" fn MyFunc(...)
  ```
- `CDECL` → `extern "C"`

## Function Signatures

**Exported functions** (listed in ntoskrnl.spec):
```rust
#[no_mangle]
pub unsafe extern "system" fn ExportedFunction(Param: TYPE) -> NTSTATUS {
    // ...
}
```

**Static/internal functions** (not exported):
```rust
unsafe fn InternalFunction(Param: TYPE) -> NTSTATUS {
    // ...
}
```

**INIT section functions** (CODE_SEG("INIT")):
Same as above — the section attribute is handled by the linker, not Rust. Add a comment:
```rust
// CODE_SEG("INIT")
```

## Global Variables

```rust
// Exported globals
#[no_mangle]
pub static mut GlobalName: TYPE = INIT_VALUE;

// Internal globals
static mut LocalGlobal: TYPE = INIT_VALUE;
```

For `FAST_MUTEX`, `LARGE_INTEGER`, and similar structs, use `core::mem::zeroed()` for zero-init:
```rust
static mut ExpLock: FAST_MUTEX = unsafe { core::mem::zeroed() };
```

For `LARGE_INTEGER` with initializer `{value}`, use `core::mem::transmute`:
```rust
static mut ExpLuid: LARGE_INTEGER = unsafe { core::mem::transmute(0x3e9_u64) };
```

## Type Mappings

| C Type | Rust Type |
|--------|-----------|
| `NTSTATUS` | `NTSTATUS` (from ntkrnl_sys) |
| `BOOLEAN` | `BOOLEAN` (= `u8`, not `bool`) |
| `TRUE` / `FALSE` | `1u8` / `0u8` (for BOOLEAN); `true`/`false` for bool |
| `ULONG` | `ULONG` (= `u32`) |
| `LONG` | `LONG` (= `i32`) |
| `USHORT` | `USHORT` (= `u16`) |
| `UCHAR` | `UCHAR` (= `u8`) |
| `PVOID` | `PVOID` (= `*mut c_void`) |
| `HANDLE` | `HANDLE` |
| `SIZE_T` | `SIZE_T` (= `usize`) |
| `PULONG` | `*mut ULONG` |
| `PLONG` | `*mut LONG` |
| `PUCHAR` | `*mut UCHAR` |
| `PWCHAR` | `*mut WCHAR` |
| `LARGE_INTEGER` | `LARGE_INTEGER` (union with QuadPart: i64) |
| `ULARGE_INTEGER` | `ULARGE_INTEGER` (union with QuadPart: u64) |
| `UNICODE_STRING` | `UNICODE_STRING` |
| `OBJECT_ATTRIBUTES` | `OBJECT_ATTRIBUTES` |
| `KPROCESSOR_MODE` | `KPROCESSOR_MODE` (= `i8`) |
| `KernelMode` | `0i8` |
| `NULL` | `core::ptr::null_mut()` or `core::ptr::null()` |
| `(TYPE*)ptr` | `ptr as *mut TYPE` or `ptr as *const TYPE` |

## Macro Expansions

### NT_SUCCESS / NT_ERROR / NT_WARNING / NT_INFORMATION
```rust
// NT_SUCCESS(Status) → Status >= 0 (includes success + informational + warning)
// !NT_SUCCESS(Status) → Status < 0 (errors only — severity 3)
if status >= 0 { ... }

// CRITICAL: NT_ERROR is NOT the same as !NT_SUCCESS!
// NT_ERROR only matches severity 3 (error), NOT severity 2 (warning).
// NT_ERROR(Status) → ((Status as u32) >> 30) == 3
// Warnings like STATUS_BUFFER_OVERFLOW (0x80000005) are NOT NT_ERROR.
fn NT_ERROR(status: NTSTATUS) -> bool { ((status as u32) >> 30) == 3 }

// NT_WARNING(Status) → ((Status as u32) >> 30) == 2
// NT_INFORMATION(Status) → ((Status as u32) >> 30) == 1
```

### RtlCopyMemory / RtlZeroMemory / RtlMoveMemory
```rust
// RtlCopyMemory(dst, src, len)
core::ptr::copy_nonoverlapping(src as *const u8, dst as *mut u8, len);

// RtlZeroMemory(dst, len)
core::ptr::write_bytes(dst as *mut u8, 0, len);

// RtlMoveMemory(dst, src, len) — handles overlap
core::ptr::copy(src as *const u8, dst as *mut u8, len);
```

### InitializeObjectAttributes
```rust
let mut object_attributes = OBJECT_ATTRIBUTES {
    Length: core::mem::size_of::<OBJECT_ATTRIBUTES>() as ULONG,
    RootDirectory: root_handle,
    ObjectName: &mut key_name as *mut UNICODE_STRING,
    Attributes: OBJ_CASE_INSENSITIVE | OBJ_KERNEL_HANDLE,
    SecurityDescriptor: core::ptr::null_mut(),
    SecurityQualityOfService: core::ptr::null_mut(),
};
```

### RtlInitUnicodeString
```rust
// Use the extern function directly:
RtlInitUnicodeString(&mut string, wide_string_ptr);
```
For wide string literals, define them as static arrays — see "Wide String Encoding"
below for the one permitted form. Never build one from a byte string or a transmute.
```rust
static KEY_PATH: [u16; N] = [...]; // pre-computed UTF-16, null-terminated
```

### PAGED_CODE() / DPRINT / DPRINT1
Omit entirely — these are debug-only macros.

### ASSERT / ASSERTMSG
Omit in release. Optionally: `debug_assert!(condition);`

### InterlockedIncrement / InterlockedDecrement / InterlockedExchange
```rust
// InterlockedDecrement(&var)
// Use the extern from ntkrnl_sys, or:
core::sync::atomic::AtomicI32::from_ptr(&mut var).fetch_sub(1, core::sync::atomic::Ordering::SeqCst)
// Prefer using the InterlockedXxx externs when available in ntkrnl_sys
```

### InterlockedExchangeAdd64
Use the extern function from ntkrnl_sys.

### ExGetPreviousMode / KeGetPreviousMode
These are externs — import from ntkrnl_sys.

### ProbeForWrite / ProbeForRead
These are externs — import from ntkrnl_sys.

### ExInitializeFastMutex / ExAcquireFastMutex / ExReleaseFastMutex
These are externs — import from ntkrnl_sys.

### SeCaptureSubjectContext / SeReleaseSubjectContext
These are externs — import from ntkrnl_sys.

### SYSTEM_LUID
```rust
let system_luid = LUID { LowPart: 0x3e7, HighPart: 0 };
```

### RtlEqualLuid
```rust
// RtlEqualLuid(a, b) — it's a macro that compares fields
(*a).LowPart == (*b).LowPart && (*a).HighPart == (*b).HighPart
```

## Structured Exception Handling (SEH → Rust Probe API)

This codebase uses DUMMY_PSEH where `_SEH2_EXCEPT` handlers are dead code. Instead of executing probe/write operations directly (which would crash on bad pointers), use the **Rust probe API** from `ntkrnl_sys` which validates pointers and returns `Result<T, NTSTATUS>`.

**Required import:**
```rust
use ntkrnl_sys::probe::*;
```

### Pattern A: Probe-then-bail (validate user pointer, bail on failure)
```c
if (PreviousMode != KernelMode) {
    _SEH2_TRY { ProbeForWriteHandle(EventHandle); }
    _SEH2_EXCEPT(EXCEPTION_EXECUTE_HANDLER) { _SEH2_YIELD(return _SEH2_GetExceptionCode()); }
    _SEH2_END;
}
```
→ Use `probe_user_buffer` to validate without writing:
```rust
if PreviousMode != KernelMode {
    probe_user_buffer(
        EventHandle as *const core::ffi::c_void,
        core::mem::size_of::<HANDLE>(),
        core::mem::align_of::<HANDLE>(),
        PreviousMode,
    )?;
}
```

Or for simple scalar probes:
```rust
if PreviousMode != KernelMode {
    probe_user_buffer(
        PreviousState as *const core::ffi::c_void,
        core::mem::size_of::<LONG>(),
        core::mem::align_of::<LONG>(),
        PreviousMode,
    )?;
}
```

### Pattern B: Write-back result (write computed value to user pointer)
```c
_SEH2_TRY { *EventHandle = hEvent; }
_SEH2_EXCEPT(ExSystemExceptionFilter()) { Status = _SEH2_GetExceptionCode(); }
_SEH2_END;
```
→ Use `write_user` (validates + writes, propagates error):
```rust
write_user(EventHandle, hEvent as HANDLE, PreviousMode)?;
```

When the EXCEPT handler is `(void)0` (silently swallow — common in timer.c):
```c
_SEH2_TRY { *TimerHandle = hTimer; }
_SEH2_EXCEPT(ExSystemExceptionFilter()) { (void)0; }
_SEH2_END;
```
→ Use `write_user_nofail` (ignore failure):
```rust
write_user_nofail(TimerHandle, hTimer as HANDLE, PreviousMode);
```

### Pattern C: Read user input (capture value from user pointer)
```c
_SEH2_TRY {
    TimerDueTime = ProbeForReadLargeInteger(DueTime);
}
_SEH2_EXCEPT(EXCEPTION_EXECUTE_HANDLER) { _SEH2_YIELD(return _SEH2_GetExceptionCode()); }
_SEH2_END;
```
→ Use `probe_read_user` (validates + reads):
```rust
let TimerDueTime = probe_read_user::<LARGE_INTEGER>(DueTime, PreviousMode)?;
```

### Pattern D: Complex body (multi-field write-back)
```c
_SEH2_TRY {
    BasicInfo->EventType = Event->Header.Type;
    BasicInfo->EventState = KeReadStateEvent(Event);
    if (ReturnLength) *ReturnLength = sizeof(EVENT_BASIC_INFORMATION);
}
_SEH2_EXCEPT(ExSystemExceptionFilter()) { Status = _SEH2_GetExceptionCode(); }
_SEH2_END;
```
→ Decompose into sequential `write_user` calls:
```rust
write_user(
    core::ptr::addr_of_mut!((*BasicInfo).EventType),
    (*Event).Header.Type as _,
    PreviousMode,
)?;
write_user(
    core::ptr::addr_of_mut!((*BasicInfo).EventState),
    KeReadStateEvent(Event) as _,
    PreviousMode,
)?;
if !ReturnLength.is_null() {
    write_user(
        ReturnLength,
        core::mem::size_of::<_EVENT_BASIC_INFORMATION>() as ULONG,
        PreviousMode,
    )?;
}
```

### Pattern E: Kernel APIs that raise (KeReleaseMutant, MmProbeAndLockPages)
```c
_SEH2_TRY { KeReleaseMutant(Mutant, ...); *PreviousCount = Prev; }
_SEH2_EXCEPT(...) { Status = _SEH2_GetExceptionCode(); }
_SEH2_END;
```
→ Call directly — these are already unprotected with DUMMY_PSEH:
```rust
KeReleaseMutant(Mutant, MUTANT_INCREMENT, 0u8, 0u8);
if !PreviousCount.is_null() {
    write_user(PreviousCount, Prev, PreviousMode)?;
}
```

### API Reference

```rust
// Validate pointer range + alignment (no read/write)
probe_user_buffer(buf: *const c_void, len: usize, align: usize, mode: KPROCESSOR_MODE) -> Result<(), NTSTATUS>

// Validate + read a T from user pointer
probe_read_user::<T>(ptr: *const T, mode: KPROCESSOR_MODE) -> Result<T, NTSTATUS>

// Validate + write a T to user pointer (returns error on failure)
write_user::<T>(ptr: *mut T, value: T, mode: KPROCESSOR_MODE) -> Result<(), NTSTATUS>

// Validate + write a T to user pointer (silently ignores failure)
write_user_nofail::<T>(ptr: *mut T, value: T, mode: KPROCESSOR_MODE)

// Validate + capture UNICODE_STRING header
probe_capture_unicode_string_header(ptr: *const UNICODE_STRING, mode: KPROCESSOR_MODE) -> Result<UNICODE_STRING, NTSTATUS>

// Copy from user with validation
safe_copy_from_user(kernel_dst: *mut c_void, user_src: *const c_void, len: usize, align: usize, mode: KPROCESSOR_MODE) -> Result<(), NTSTATUS>

// Copy to user with validation
safe_copy_to_user(user_dst: *mut c_void, kernel_src: *const c_void, len: usize, align: usize, mode: KPROCESSOR_MODE) -> Result<(), NTSTATUS>
```

All `Result`-returning functions propagate with `?` in functions returning `NTSTATUS`:
```rust
// The ? operator converts Err(NTSTATUS) to early return of that NTSTATUS
pub unsafe extern "system" fn NtCreateEvent(...) -> NTSTATUS {
    // ... probe calls with ? ...
    // On success, return STATUS_SUCCESS explicitly at the end
}
```

**IMPORTANT**: Functions returning NTSTATUS must use a wrapper to make `?` work:
```rust
// Wrap the function body to convert Result to NTSTATUS return:
fn inner() -> Result<NTSTATUS, NTSTATUS> {
    probe_user_buffer(...)?;
    // ...
    Ok(STATUS_SUCCESS)
}
match inner() {
    Ok(s) => s,
    Err(s) => s,
}
```
Or use a labeled block pattern:
```rust
let Status = 'seh: {
    if let Err(e) = probe_user_buffer(...) { break 'seh e; }
    if let Err(e) = write_user(...) { break 'seh e; }
    STATUS_SUCCESS
};
```
Choose whichever pattern produces cleaner code for the function at hand. The inner-function pattern is best for functions with many SEH blocks; the labeled-block is best for isolated blocks.

## Architecture Conditionals

```c
#ifdef _M_ARM64     → #[cfg(target_arch = "aarch64")]
#ifdef _M_AMD64     → #[cfg(target_arch = "x86_64")]
#ifdef _M_IX86      → #[cfg(target_arch = "x86")]
#ifndef _M_AMD64    → #[cfg(not(target_arch = "x86_64"))]
#ifdef _WIN64       → #[cfg(target_pointer_width = "64")]
```

## goto / cleanup patterns

Replace `goto cleanup` with structured control flow:
```rust
// Use a labeled block or early returns:
let status = 'block: {
    if condition {
        break 'block STATUS_ERROR;
    }
    // ... more work ...
    STATUS_SUCCESS
};
// cleanup code here
```

Or restructure to use `loop { break; }` with cleanup after the loop.

## Unicode String Literals

For `L"..."` wide string literals, create static UTF-16 arrays. Use a const helper or hardcode the values.

A practical approach:
```rust
// L"\\Registry\\Machine\\..."
// Pre-encode as UTF-16LE:
static REGISTRY_PATH: &[u16] = &[
    b'\\' as u16, b'R' as u16, b'e' as u16, // ... etc
    0  // null terminator
];
```

## Common Translation Pitfalls

### Coalescing Loop Variables
When code merges/coalesces ranges (e.g., free pool pages coalescing with adjacent free entries), the loop that processes the merged result must use the **total merged count** variable, not the original input count. Watch for patterns like:
```c
FreePages += Entry->Size;     // accumulate total
/* ... later loop ... */
for (i = 0; i < FreePages; i++)  // must use FreePages, NOT NumberOfPages
```
A common bug is using `NumberOfPages` (the original input) instead of `FreePages` (the accumulated total after coalescing).

### Spinlock API Selection
`KeAcquireSpinLockForDpc` / `KeReleaseSpinLockForDpc` are for DPC context only — they skip IRQL transitions. For general pool locking and other non-DPC paths, use `KeAcquireSpinLock` / `KeReleaseSpinLock`. Match the original C code's API choice exactly — substituting the wrong variant causes incorrect IRQL handling.

### Kernel Virtual Address Arithmetic
PTE/PDE calculations like `(addr >> 12) * 8 + PTE_BASE` overflow for kernel virtual addresses on 64-bit (e.g., 0xFFFFF68000000000 >> 12 = 0xFFFFF68000000, × 8 overflows u64). Use bit masking instead: `(addr >> 9) & (0xFFFFFFFFF << 3) + PTE_BASE`. Always verify pointer arithmetic against the C original for both 32-bit and 64-bit address ranges.

### addr_of_mut! vs Pointer Value
`core::ptr::addr_of_mut!((*ptr).field)` gives the **address of** the field within the struct. This is correct for:
- Embedded struct fields (LIST_ENTRY, LUID, EX_FAST_REF) — taking their address for atomic ops or FFI
- Scalar fields — passing `*mut LONG` to InterlockedIncrement

This is **WRONG** when the field is already a pointer type and you need the pointer's **value**:
```rust
// C: Token->UserAndGroups (gets the pointer VALUE stored in the field)
// WRONG: addr_of_mut gives **SID_AND_ATTRIBUTES (address of the pointer field)
core::ptr::addr_of_mut!((*token).UserAndGroups) as *mut SID_AND_ATTRIBUTES
// RIGHT: dereference to get the pointer value
(*token).UserAndGroups as *mut SID_AND_ATTRIBUTES
```

Rule of thumb: if the C code is `ptr->field` (reading the value), use `(*ptr).field`. Only use `addr_of_mut!` when the C code is `&ptr->field` (taking the address).

### Zero-Sized Extern Array Indexing
C extern arrays like `KPRCB *KiProcessorBlock[]` become `static mut KiProcessorBlock: [*mut _KPRCB; 0]` in Rust (zero-sized). Direct indexing panics. Use pointer arithmetic:
```rust
// WRONG: panics on zero-sized array
let prcb = KiProcessorBlock[i];
// RIGHT: pointer arithmetic from array base
let prcb = *(core::ptr::addr_of!(KiProcessorBlock) as *const *mut _KPRCB).add(i);
```

### Backward goto → Loop Duplication
When C uses `goto label` to jump backward (e.g., `goto ParseFromRoot`), translate as:
1. Run the target code **before** the loop (initial entry path)
2. Duplicate the target code **inside** the loop body, gated on the reparse/retry condition
3. Replace `goto` with `continue` or loop control flow

Comment both locations to explain the goto transformation.

## Style Rules

1. Keep original C variable names (PascalCase for params/locals is fine)
2. Keep original comments where meaningful
3. All functions are `unsafe` (kernel code)
4. Use `core::` not `std::` (no_std environment)
5. Do NOT add doc comments or extra documentation — keep it minimal
6. Do NOT add error handling beyond what the C code has
7. Do NOT add `#[inline]` unless the C code has `FORCEINLINE`
8. Use raw pointers throughout — do not introduce references except where necessary for Rust semantics
9. Access union fields (like `LARGE_INTEGER.QuadPart`) via the appropriate union accessor — in bindgen-generated types this may be `unsafe { (*li).u.QuadPart }` or direct field access depending on the generated layout
10. When accessing struct fields through pointers, use `(*ptr).field` pattern

## LARGE_INTEGER / ULARGE_INTEGER Access Pattern

These are unions in bindgen output. Access QuadPart as:
```rust
// Reading: use __bindgen_anon_1 or direct union access
let quad = unsafe { (*time).__bindgen_anon_1.QuadPart };
// Or if the type has a QuadPart field directly accessible
let quad = unsafe { (*time).QuadPart };

// Writing
unsafe { (*time).__bindgen_anon_1.QuadPart = value; }
```

Check the actual bindgen layout — if LARGE_INTEGER has `.QuadPart` directly, use it. If it's nested in an anonymous union, use the `__bindgen_anon_1` accessor.

## Critical Translation Bugs (Field-Verified)

These are the most common bugs found in LLM-translated kernel code. Each one has caused kernel crashes or hangs in production testing. Pay special attention to these patterns.

### Wide String Encoding (L"..." → [u16; N])

C wide string literals (`L"..."`) MUST be translated to `[u16; N]` arrays with explicit UTF-16 values. **NEVER** use `b"..."` byte strings cast to `*const u16` — this produces ASCII bytes (1 byte per char) instead of UTF-16 (2 bytes per char), causing garbage data in all wide string APIs.

```rust
// WRONG — produces ASCII bytes, NOT UTF-16:
b"\\Registry\\Machine\0".as_ptr() as *const u16

// RIGHT — explicit UTF-16 array:
static REG_PATH: [u16; 19] = [
    b'\\' as u16, b'R' as u16, b'e' as u16, b'g' as u16, b'i' as u16,
    b's' as u16, b't' as u16, b'r' as u16, b'y' as u16, b'\\' as u16,
    b'M' as u16, b'a' as u16, b'c' as u16, b'h' as u16, b'i' as u16,
    b'n' as u16, b'e' as u16, 0, 0,
];
```

The `b"S\0e\0r\0v\0i\0c\0e\0\0\0"` byte-string hack (ASCII char + `\0` pairs reinterpreted as LE u16) is FORBIDDEN, not merely discouraged. It is wrong three ways, and all three have shipped: it has ALIGNMENT 1 where a `u16` read wants 2; one miscounted `\0` truncates the string with no diagnostic anywhere (`L"0000%s"` came back as five zeros, and a whole module's registry names came back one character long); and the reader checks it by eye against a hex dump, which is exactly the check that passes when it is wrong. Use an explicit `[u16; N]` array, or the crate's `w!`/`wide!` macro where one exists — the macro counts the elements and appends the NUL so neither can be got wrong.

### Enum/Constant Value Ordering

When translating a C `switch` statement with symbolic enum constants, use the **actual enum values**, not sequential ordinals. A common LLM mistake is assigning match arm values 0, 1, 2, 3... in the ORDER the cases appear in the switch, rather than using the correct enum values.

```rust
// WRONG — sequential ordinals matching C source order:
match DeviceProperty {
    0 => { /* BusTypeGuid logic */ }    // Should be 12
    1 => { /* LegacyBusType logic */ }  // Should be 13
    // ...
}

// RIGHT — use symbolic constants with correct values:
const DevicePropertyBusTypeGuid: u32 = 12;
const DevicePropertyLegacyBusType: u32 = 13;
match DeviceProperty {
    DevicePropertyBusTypeGuid => { /* BusTypeGuid logic */ }
    DevicePropertyLegacyBusType => { /* LegacyBusType logic */ }
    // ...
}
```

Similarly, `KEY_VALUE_INFORMATION_CLASS` values differ from `KEY_INFORMATION_CLASS`:
- `KeyValueBasicInformation = 0, KeyValueFullInformation = 1, KeyValuePartialInformation = 2`
- `KeyBasicInformation = 0, KeyNodeInformation = 1, KeyFullInformation = 2`

### Global Variables: #[no_mangle] and #[used]

Globals referenced from C code MUST have `#[no_mangle]`. Additionally, if no Rust code reads the global (only C reads it), add `#[used]` to prevent LLVM from eliminating it:

```rust
#[no_mangle]
#[used]  // C code reads this; LLVM would eliminate without #[used]
pub static mut IopRootDeviceNode: PDEVICE_NODE = core::ptr::null_mut();
```

### Static Lifetime for Wide String Temporaries

Inline `[u16; N]` arrays used with `RtlInitUnicodeString` MUST be `static` — a local array is a temporary that dies at the semicolon, leaving a dangling pointer:

```rust
// WRONG — dangling pointer (array is temporary):
RtlInitUnicodeString(&mut name, [b'F' as u16, b'o' as u16, b'o' as u16, 0].as_ptr());

// RIGHT — static array has 'static lifetime:
static FOO: [u16; 4] = [b'F' as u16, b'o' as u16, b'o' as u16, 0];
RtlInitUnicodeString(&mut name, FOO.as_ptr());
```

### Output Pointer Functions and LLVM Cross-CGU Optimization

Functions that write to output pointers (`PHANDLE`, `*mut PKEY_VALUE_FULL_INFORMATION`, etc.) may have their writes eliminated by LLVM when called across codegen units (different Rust files). The LLVM `captures(none)` optimization concludes the pointer isn't captured and eliminates the store.

**Workaround**: For functions like `IopOpenRegistryKeyEx`, `IopGetRegistryValue`, `IopCreateRegistryKeyEx` that are called from many other files, provide a C shim wrapper that calls the Rust implementation through the ABI boundary:

```c
// io_rust_shims.c
NTSTATUS NTAPI IopOpenRegistryKeyEx(PHANDLE, HANDLE, PUNICODE_STRING, ACCESS_MASK);
NTSTATUS NTAPI IopOpenRegistryKeyEx_shim(PHANDLE KeyHandle, ...) {
    return IopOpenRegistryKeyEx(KeyHandle, ...);  // C→Rust ABI boundary prevents optimization
}
```

### Calling Convention on Exported Functions

All `NTAPI` functions must use `extern "system"` (stdcall on i386, cdecl on amd64/arm64). Functions WITHOUT `NTAPI` in C are cdecl — use `extern "C"`. Getting this wrong causes stack corruption on i386:

```rust
// C: NTSTATUS NTAPI IoGetDeviceProperty(...)  →  extern "system"
// C: VOID PiQueueDeviceAction(...)            →  extern "C"  (no NTAPI)
// C: PDEVICE_NODE FASTCALL IopGetDeviceNode(...)  →  extern "fastcall"
```

### IO_STATUS_BLOCK Anonymous Union

`IO_STATUS_BLOCK.Status` is inside an anonymous union. In bindgen:
```rust
// WRONG:
(*IoStatusBlock).Status

// RIGHT:
(*IoStatusBlock).__bindgen_anon_1.Status
```

### RAII Guards for Locks and IRQL (REQUIRED for new translations)

Use RAII guard types instead of manual acquire/release pairs. This prevents LLVM from optimizing away lock operations and makes error paths automatically safe.

**PFN Lock:**
```rust
// WRONG — manual pair, LLVM can eliminate
OldIrql = MiAcquirePfnLock();
// ... work ...
MiReleasePfnLock(OldIrql);

// RIGHT — RAII guard, drop() at release point
let pfn_guard = unsafe { crate::ARM3::helpers::PfnLockGuard::acquire() };
// ... work ...
drop(pfn_guard);
```

**Spinlock:**
```rust
// WRONG
KeAcquireSpinLock(&mut Lock, &mut OldIrql);
// ... critical section ...
KeReleaseSpinLock(&mut Lock, OldIrql);

// RIGHT
let spin_guard = unsafe { ntkrnl_sys::guards::SpinLockGuard::acquire(&raw mut Lock) };
// ... critical section ...
drop(spin_guard);
```

**IRQL Raise/Lower:**
```rust
// WRONG
KeRaiseIrql(DISPATCH_LEVEL, &mut OldIrql);
// ... work at DISPATCH ...
KeLowerIrql(OldIrql);

// RIGHT
let irql_guard = unsafe { ntkrnl_sys::guards::IrqlGuard::raise(DISPATCH_LEVEL) };
// ... work at DISPATCH ...
drop(irql_guard);
```

**Rules:**
- Always use `drop(guard)` at the exact point the original release/lower was. Do NOT let guards drop implicitly at function end — this changes lock duration.
- For output pointer functions (ObCreateObject, etc.), prefer returning `Result<T, NTSTATUS>` internally with a thin `extern "C"` wrapper for C callers.
- See `docs/engineering/rust-kernel-patterns.md` for the full architecture.
