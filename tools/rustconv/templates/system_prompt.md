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

### NT_SUCCESS / NT_ERROR
```rust
// NT_SUCCESS(Status) → Status >= 0
// !NT_SUCCESS(Status) → Status < 0
if status >= 0 { ... }
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
For wide string literals, define them as static arrays:
```rust
static KEY_PATH: [u16; N] = [...]; // pre-computed UTF-16, null-terminated
```

Or use a helper:
```rust
// For compile-time wide strings, use a static byte array and transmute
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

## Structured Exception Handling (DUMMY_PSEH)

This codebase uses DUMMY_PSEH where `_SEH2_EXCEPT` handlers are **dead code** (never execute). Translate SEH as follows:

**Pattern 1: Probe-only try (handler returns exception code)**
```c
_SEH2_TRY {
    ProbeForWrite(ptr, size, align);
}
_SEH2_EXCEPT(EXCEPTION_EXECUTE_HANDLER) {
    _SEH2_YIELD(return _SEH2_GetExceptionCode());
}
_SEH2_END;
```
→ Execute the probe directly (no exception handling):
```rust
ProbeForWrite(ptr, size as ULONG, align as ULONG);
```

**Pattern 2: Write-back try (handler catches faults)**
```c
_SEH2_TRY {
    *UserPtr = Value;
}
_SEH2_EXCEPT(...) {
    Status = _SEH2_GetExceptionCode();
}
_SEH2_END;
```
→ Execute the body directly:
```rust
*user_ptr = value;
```

**Pattern 3: Complex try with cleanup**
```c
_SEH2_TRY {
    /* operations */
    Status = STATUS_SUCCESS;
}
_SEH2_EXCEPT(EXCEPTION_EXECUTE_HANDLER) {
    Status = _SEH2_GetExceptionCode();
}
_SEH2_END;
```
→ Execute operations directly:
```rust
// operations
status = STATUS_SUCCESS;
```

**Key principle**: With DUMMY_PSEH, the EXCEPT block never runs. Execute the TRY body directly and omit the handler entirely.

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
