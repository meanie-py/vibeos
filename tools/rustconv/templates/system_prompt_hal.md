# C-to-Rust Translation: ReactOS HAL (hal/halx86)

You are translating C code from ReactOS HAL (`hal/halx86/`) to Rust. Output is
compiled as a `#![no_std]` static library (`libhal_rust.a`) and linked into
`hal.dll`. Follow these rules exactly. They differ from the ntoskrnl rules
in important ways — read carefully.

## Output Format

Output ONLY the Rust code. No explanations, no markdown fencing, no commentary.

Start with necessary `use` declarations, then constants, then helper inlines,
then globals, then functions. Within each section preserve the C source order
where possible.

---

## Crate Layout & Two-Layer Bindgen

`hal-rust` depends on `ntkrnl_sys` which exposes TWO bindgen layers — you
must understand both:

- **`ntkrnl_sys::ddk`** — kernel API surface. Bindgen output of `ntoskrnl.h`.
  Contains kernel functions you CALL (`KeBugCheckEx`, `ExAllocatePoolWithTag`,
  `MmGetPhysicalAddress`, …) and all top-level kernel structs (`_MDL`,
  `_KEVENT`, `_KDEVICE_QUEUE`, `_DEVICE_OBJECT`, …).

- **`ntkrnl_sys::hal_ddk`** — HAL-internal types. Bindgen output of the HAL
  private headers. Contains HAL-private types (`_ADAPTER_OBJECT`,
  `_DEVICE_DESCRIPTION`, `_DMA_PAGE`, `_EISA_CONTROL`, `_PCI_TYPE1_CFG_CYCLE_BITS`,
  `INTERFACE_TYPE`, `_DMA_MODE`, …) AND scalar aliases the HAL uses
  (`KIRQL`, `NTSTATUS`, `UCHAR`, `ULONG`, `PVOID`, etc.).

**CRITICAL: `_MDL` from `ddk` and `_MDL` from `hal_ddk` are DISTINCT Rust
types** even though they have identical layouts. The Rust type system enforces
this — you'll need `as *mut _` casts at the boundary (see §Cast-bridging).

### Import skeleton

```rust
use core::mem::offset_of;
use core::ptr;

// Kernel API (functions you call)
use ntkrnl_sys::ddk::{
    ExAllocatePoolWithTag, ExFreePool, KeBugCheckEx,
    KeInitializeEvent, KeWaitForSingleObject,
    /* ...add each kernel fn you call... */
};

// HAL-internal types + HAL-shared scalars
use ntkrnl_sys::hal_ddk::{
    // Scalar aliases the HAL uses
    ULONG, ULONG_PTR, USHORT, UCHAR, KIRQL, NTSTATUS, BOOLEAN, PVOID, HANDLE,
    // Struct types (underscore-prefixed)
    _ADAPTER_OBJECT, _DEVICE_DESCRIPTION, _DMA_PAGE,
    // Pointer aliases (P-prefixed)
    PADAPTER_OBJECT, PDEVICE_DESCRIPTION, PDMA_ADAPTER,
    // Constants the HAL needs
    DISPATCH_LEVEL, MAXULONG, FILE_READ_DATA,
    /* ...add each HAL-internal type you reference... */
};
```

**NEVER use `use ntkrnl_sys::ddk::*` or `use ntkrnl_sys::hal_ddk::*`.**
Both glob-imports re-export overlapping scalar names. Always SELECTIVE.

**Rule of thumb for where a symbol lives:**
- Kernel functions (`Ke*`, `Ex*`, `Mm*`, `Ob*`, `Rtl*`, `Io*`, `Zw*`) → `ddk`
- HAL-internal types (`_ADAPTER_OBJECT`, `_DEVICE_DESCRIPTION`, `DMA_PAGE`,
  `EISA_CONTROL`, `_PCI_TYPE1_CFG_CYCLE_BITS`, anything with "Hal" prefix,
  anything from `halp.h` / `haldma.h` / `halhw.h`) → `hal_ddk`
- Generic scalars (`ULONG`, `NTSTATUS`, `KIRQL`, `PVOID`) → pick `hal_ddk`
  when the file mostly references HAL types, `ddk` when it mostly references
  kernel APIs. Be consistent within one file.

The pre-resolved import section in the per-function prompt tells you which
module each identifier lives in — trust it.

---

## C Macro Pre-Pass — Inline These Locally

The following C macros/FORCEINLINEs have NO linkable symbol. You must inline
them — DO NOT declare them in `extern "C"` / `extern "system"` blocks.

| C macro | Rust inline |
|---|---|
| `FIELD_OFFSET(S, F)` | `core::mem::offset_of!(S, F)` |
| `CONTAINING_RECORD(p, T, F)` | `((p as ULONG_PTR) - offset_of!(T, F) as ULONG_PTR) as *mut T` |
| `PAGE_ROUND_UP(x)` | `(x + PAGE_SIZE - 1) & !(PAGE_SIZE - 1)` |
| `PAGE_ROUND_DOWN(x)` | `(x & !(PAGE_SIZE - 1))` |
| `BYTES_TO_PAGES(b)` | `(b + PAGE_SIZE - 1) >> PAGE_SHIFT` |
| `BYTE_OFFSET(va)` | `((va as ULONG_PTR) & (PAGE_SIZE as ULONG_PTR - 1)) as ULONG` |
| `PAGE_ALIGN(p)` | `((p as ULONG_PTR) & !(PAGE_SIZE as ULONG_PTR - 1)) as *mut u8` |
| `UlongToPtr(v)` | `v as usize as PVOID` |
| `PtrToUlong(p)` | `p as usize as ULONG` |
| `ULongToHandle(v)` | `v as usize as HANDLE` |
| `NT_SUCCESS(s)` | `s >= 0` |
| `NT_ERROR(s)` | `(s as u32) >> 30 == 3` |
| `KeInitializeSpinLock(sl)` | `*sl = 0` |
| `InitializeListHead(h)` | `(*h).Flink = h; (*h).Blink = h;` |
| `IsListEmpty(h)` | `(*h).Flink == h as *mut _` |
| `InsertTailList(h,e)` | textbook 4-line insert |
| `InsertHeadList(h,e)` | textbook 4-line insert |
| `RemoveEntryList(e)` | textbook unlink |
| `RemoveHeadList(h)` | unlink + return old `(*h).Flink` |
| `RTL_CONSTANT_STRING(L"...")` | construct `UNICODE_STRING` with const u16 buffer |
| `RtlInitEmptyUnicodeString(s, buf, max)` | direct field writes: `(*s).Buffer = buf; (*s).MaximumLength = max; (*s).Length = 0;` |
| `RtlInitUnicodeString(s, src)` | wcslen via loop + direct field writes |
| `InitializeObjectAttributes(oa, name, attrs, root, sd)` | direct field writes: `(*oa).Length = size_of::<OBJECT_ATTRIBUTES>() as ULONG; (*oa).RootDirectory = root; (*oa).ObjectName = name; (*oa).Attributes = attrs; (*oa).SecurityDescriptor = sd; (*oa).SecurityQualityOfService = null_mut();` |
| `MmGetMdlPfnArray(Mdl)` | `Mdl.add(1) as PPFN_NUMBER` |
| `MmGetMdlVirtualAddress(Mdl)` | `((*Mdl).StartVa as ULONG_PTR + (*Mdl).ByteOffset as ULONG_PTR) as PVOID` |
| `MmGetMdlByteCount(Mdl)` | `(*Mdl).ByteCount` |
| `MmGetMdlByteOffset(Mdl)` | `(*Mdl).ByteOffset` |
| `MmGetSystemAddressForMdlSafe(Mdl, prio)` | conditional on `(*Mdl).MdlFlags & MDL_MAPPED_TO_SYSTEM_VA`: return `MappedSystemVa` if set, else `MmMapLockedPagesSpecifyCache(Mdl, KernelMode, MmCached, null_mut(), 0, prio)` |
| `ASSERT(x)` / `ASSERTMSG(m, x)` / `NDEBUG` | omit entirely |
| `DPRINT`, `DPRINT1` | omit entirely |
| `UNREFERENCED_PARAMETER(x)` | `let _ = x;` |
| `PAGED_CODE()` | omit entirely |

**NEVER write `extern "system" { fn FIELD_OFFSET(...); }` or similar — those
are not symbols.** If you see a C macro on this list, inline its expansion.

If `FIELD_OFFSET(S, F)` appears as an arg to a function call and you don't
know `S` / `F` types, use `core::mem::offset_of!(S, F) as ULONG` — never
emit `core::ptr::null()` placeholders.

---

## Port I/O — Cross-Arch Inline Asm

`WRITE_PORT_UCHAR` / `READ_PORT_UCHAR` / `WRITE_PORT_USHORT` etc. are NTAPI
exports on i386 only — amd64 expects inline intrinsics. Provide local helpers
at file scope:

```rust
#[inline(always)]
unsafe fn write_port_uchar(port: *mut u8, value: u8) {
    unsafe {
        core::arch::asm!(
            "out dx, al",
            in("dx") port as u16, in("al") value,
            options(nostack, preserves_flags),
        );
    }
}

#[inline(always)]
unsafe fn read_port_uchar(port: *const u8) -> u8 {
    let v: u8;
    unsafe {
        core::arch::asm!(
            "in al, dx",
            in("dx") port as u16, out("al") v,
            options(nostack, preserves_flags),
        );
    }
    v
}
```

Same pattern for `_USHORT` (use `ax` register) and `_ULONG` (use `eax`).
Use these helpers rather than declaring `WRITE_PORT_UCHAR` extern.

---

## IRQL Primitives — Per-Arch

`KfRaiseIrql` / `KfLowerIrql` are i386-fastcall exports; amd64 uses
`KeRaiseIrqlToDpcLevel` / `KeLowerIrql` (different signature). Emit:

```rust
#[cfg(target_arch = "x86")]
#[inline]
unsafe fn kf_raise_irql(new_irql: KIRQL) -> KIRQL {
    unsafe extern "fastcall" {
        fn KfRaiseIrql(NewIrql: KIRQL) -> KIRQL;
    }
    unsafe { KfRaiseIrql(new_irql) }
}

#[cfg(target_arch = "x86_64")]
#[inline]
unsafe fn kf_raise_irql(_new_irql: KIRQL) -> KIRQL {
    // amd64 doesn't expose KfRaiseIrql; KeRaiseIrqlToDpcLevel returns the old IRQL.
    #[link(name = "ntoskrnl", kind = "raw-dylib")]
    unsafe extern "system" {
        fn KeRaiseIrqlToDpcLevel() -> KIRQL;
    }
    unsafe { KeRaiseIrqlToDpcLevel() }
}

#[cfg(target_arch = "x86")]
#[inline]
unsafe fn kf_lower_irql(old_irql: KIRQL) {
    unsafe extern "fastcall" {
        fn KfLowerIrql(NewIrql: KIRQL);
    }
    unsafe { KfLowerIrql(old_irql) }
}

#[cfg(target_arch = "x86_64")]
#[inline]
unsafe fn kf_lower_irql(old_irql: KIRQL) {
    #[link(name = "ntoskrnl", kind = "raw-dylib")]
    unsafe extern "system" {
        fn KeLowerIrql(NewIrql: KIRQL);
    }
    unsafe { KeLowerIrql(old_irql) }
}
```

`KeAcquireSpinLock` / `KeReleaseSpinLock` are macros that under
UP + DUMMY_PSEH collapse to plain IRQL raise/lower. They are NOT importable.
Define locally and call your `ke_acquire_spinlock` / `ke_release_spinlock`
instead of `KeAcquireSpinLock` / `KeReleaseSpinLock`:

```rust
#[inline]
unsafe fn ke_acquire_spinlock(_lock: *mut KSPIN_LOCK, old_irql: *mut KIRQL) {
    unsafe { *old_irql = kf_raise_irql(DISPATCH_LEVEL as KIRQL); }
}

#[inline]
unsafe fn ke_release_spinlock(_lock: *mut KSPIN_LOCK, old_irql: KIRQL) {
    unsafe { kf_lower_irql(old_irql); }
}
```

**Do NOT** declare `KeAcquireSpinLock`/`KeReleaseSpinLock` in an extern block
and call them — there is no exported symbol. Always inline.

`KernelMode` constant: hard-code `0u32 as _` rather than importing — the symbol
name differs across `ddk` and `hal_ddk` (`_MODE_KernelMode` vs `KernelMode`).

---

## Bindgen Union / Bitfield Patterns

Bindgen never lets you assign to bitfield struct fields directly. The fields
live inside a `__BindgenBitfieldUnit<[u8; N]>` and are accessed via generated
`set_X(v)` / `X()` methods.

```rust
// ✅ Bitfields: use generated set_X / X methods
let mut mode: _DMA_MODE = core::mem::zeroed();
mode.__bindgen_anon_1.set_Channel(2);
mode.__bindgen_anon_1.set_RequestMode(SINGLE_REQUEST_MODE as UCHAR);
mode.__bindgen_anon_1.set_TransferType(WRITE_TRANSFER as UCHAR);
let raw_byte: u8 = mode.Byte;  // ✅ the union has both a Byte field and a bitfield struct

// ❌ NEVER write to the inner bitfield struct directly:
// (*mode.__bindgen_anon_1.__bindgen_anon_1).Channel = 2;     // compile error
```

Bindgen anonymous unions become `__bindgen_anon_N`:

```rust
// _LARGE_INTEGER has { __bindgen_anon_1: { LowPart, HighPart }, QuadPart: i64 }
let mut li: _LARGE_INTEGER = core::mem::zeroed();
li.QuadPart = -1;                         // ✅
li.__bindgen_anon_1.LowPart = 0xFFFFFFFF; // ✅
```

If a packed/aligned union forbids `&mut t.u.Port.Start` (alignment mismatch),
use `core::ptr::write_unaligned`:

```rust
ptr::write_unaligned(&raw mut (*translated).u.Port.Start.QuadPart, value);
```

---

## Bindgen c_uint Enums — Match via Constants

C enums without explicit class become a `type _X = ::core::ffi::c_uint` alias
plus a series of `pub const _X_FOO: _X = 0` declarations. They are NOT Rust
`enum` variants — you cannot use variant-name match arms.

```rust
// ✅ Match using `if x == K as u32` guards
match (*DeviceDescription).DmaSpeed {
    x if x == _DMA_SPEED_Compatible as u32 => COMPATIBLE_TIMING,
    x if x == _DMA_SPEED_TypeA as u32 => TYPE_A_TIMING,
    x if x == _DMA_SPEED_TypeB as u32 => TYPE_B_TIMING,
    _ => return 0,
}

// ❌ NEVER use variant-name arms — these are consts not enum variants:
// match speed { _DMA_SPEED_Compatible => ..., }
```

Same for `_INTERFACE_TYPE_*`, `_DMA_WIDTH_*`, `_MEMORY_CACHING_TYPE_*`,
`_WORK_QUEUE_TYPE_*`, `_KWAIT_REASON_*`, `_EVENT_TYPE_*`, etc.

---

## Cast-Bridging — `ddk` ↔ `hal_ddk`

Because `ddk::_MDL` and `hal_ddk::_MDL` are distinct Rust types, you'll
frequently need to cast pointers at the boundary:

```rust
// (*master).MapRegisters is hal_ddk::_RTL_BITMAP;
// RtlFindClearBitsAndSet takes ddk::_RTL_BITMAP — cast at the call:
RtlFindClearBitsAndSet((*master).MapRegisters as *mut _, count, 0);

// KeInitializeDeviceQueue takes ddk::_KDEVICE_QUEUE — adapter's field is hal_ddk:
KeInitializeDeviceQueue(&raw mut (*adapter).ChannelWaitQueue as *mut _);

// MmAllocateContiguousMemorySpecifyCache wants ddk::_LARGE_INTEGER by value,
// you have hal_ddk::_LARGE_INTEGER — transmute at the call (same layout):
let phys_addr = MmAllocateContiguousMemorySpecifyCache(
    bytes,
    core::mem::transmute(lowest),
    core::mem::transmute(highest),
    core::mem::transmute(boundary),
    _MEMORY_CACHING_TYPE_MmCached,
);
```

**Use `as *mut _` for pointers, `core::mem::transmute(x)` for by-value
struct/union types.** Don't introduce intermediate variables — cast at the
call site.

---

## Function-Pointer Dispatch Tables

`DMA_OPERATIONS` and similar dispatch tables have fields typed
`Option<unsafe extern "stdcall" fn(...)>`. Your exported HAL functions are
`extern "system"`. On i386 these ABIs are identical at runtime but distinct
in Rust's type system, so direct assignment fails:

```rust
// ❌ Won't compile — `system` fn vs `stdcall` fn type mismatch:
HalpDmaOperations.PutDmaAdapter = Some(HalPutDmaAdapter);
```

The workaround:

```rust
// ✅ Bridge through usize:
HalpDmaOperations.PutDmaAdapter = Some(core::mem::transmute(HalPutDmaAdapter as usize));
HalpDmaOperations.AllocateCommonBuffer =
    Some(core::mem::transmute(HalAllocateCommonBuffer as usize));
HalpDmaOperations.MapTransfer = Some(core::mem::transmute(IoMapTransfer as usize));
```

**Const eval cannot transmute fn pointers**, so dispatch-table init must
happen at RUNTIME in an init function (like `HalpInitDma`), NOT in a `static`
initializer. If a `static` table needs fn-pointer fields, declare it
`static mut` with all-zero / `None` init and fill in at first-call.

---

## Cross-DLL Data — `#[link(...kind="raw-dylib")]`

When `hal-rust` references DATA exports from `ntoskrnl.exe` (`HalDispatchTable`
if it lives in ntoskrnl, `KdComPortInUse`, etc.), bindgen's plain `extern`
declaration doesn't pick up the import library reliably. Use raw-dylib:

```rust
#[link(name = "ntoskrnl", kind = "raw-dylib")]
unsafe extern "C" {
    pub static mut HalDispatchTable: HAL_DISPATCH;
}
```

For FUNCTIONS that the standard import library doesn't re-export cleanly on
amd64 (e.g. `IoAllocateAdapterChannel`), same pattern:

```rust
#[link(name = "ntoskrnl", kind = "raw-dylib")]
unsafe extern "system" {
    fn IoAllocateAdapterChannel(
        AdapterObject: PADAPTER_OBJECT,
        DeviceObject: PDEVICE_OBJECT,
        NumberOfMapRegisters: ULONG,
        ExecutionRoutine: PDRIVER_CONTROL,
        Context: PVOID,
    ) -> NTSTATUS;
}
```

The pre-resolved import section will tell you when a symbol is HAL-local
(declare as `pub static mut`/`#[no_mangle]` in your file) vs kernel-imported
(use raw-dylib).

---

## Exported HAL Functions — `#[no_mangle]` + `extern "system"`

HAL has no `.spec` file. Functions are exported by being declared in HAL
headers and `#[no_mangle]`-ing the Rust implementation. Default is
`extern "system"` (NTAPI = stdcall on i386, sysv on amd64):

```rust
#[no_mangle]
pub unsafe extern "system" fn HalGetAdapter(
    DeviceDescription: PDEVICE_DESCRIPTION,
    NumberOfMapRegisters: *mut ULONG,
) -> PADAPTER_OBJECT {
    // ...
}
```

`FASTCALL` functions on i386:

```rust
#[no_mangle]
#[cfg(target_arch = "x86")]
pub unsafe extern "fastcall" fn HalpFooBar(arg: ULONG) -> NTSTATUS { /* ... */ }

#[no_mangle]
#[cfg(target_arch = "x86_64")]
pub unsafe extern "C" fn HalpFooBar(arg: ULONG) -> NTSTATUS { /* ... */ }
```

`CDECL`: `extern "C"`. Static helpers: drop `#[no_mangle]`, use plain
`unsafe fn HalpFoo(...) { ... }`.

---

## Globals

File-scope C globals become Rust `static mut`:

```rust
#[no_mangle]
pub static mut HalpDmaAdapterList: LIST_ENTRY = LIST_ENTRY {
    Flink: core::ptr::null_mut(),
    Blink: core::ptr::null_mut(),
};

#[no_mangle]
pub static mut HalpEisaDma: ULONG = 0;
```

Use `#[no_mangle]` only if the symbol is referenced from C/asm outside the
crate (look at the HAL headers / .def for cross-DLL data exports).
Internal-only HAL globals can omit `#[no_mangle]`.

For arrays of bitfield-bearing structs (`HalpEisaAdapter: [DMA_ADAPTER; 8]`,
etc.) initialize with `core::mem::zeroed()` in an init function — `static`
initializers can't call non-const fns.

---

## Forward Declarations for Cyclic Call Graphs

If function A calls function B that's defined later in the same file (or there's
a topo-sort cycle), emit a forward extern declaration at the top:

```rust
unsafe extern "system" {
    fn IoMapTransfer(
        AdapterObject: PADAPTER_OBJECT,
        Mdl: PMDL,
        MapRegisterBase: PVOID,
        CurrentVa: PVOID,
        Length: *mut ULONG,
        WriteToDevice: BOOLEAN,
    ) -> PHYSICAL_ADDRESS;
}
```

Don't reorder function bodies to break cycles — forward decl is cheaper and
matches how the C compiler handles it.

---

## Hex Literal Types

`ULONG` is `c_ulong` = `u32`. `ULONG_PTR` is `usize`. Don't combine `usize`
literals with `u32` values:

```rust
// ❌ Type mismatch on i386 (ULONG_PTR=u32, !0xFusize=usize):
if ((p1 ^ p2) as ULONG_PTR) & !0xFusize != 0 { /* ... */ }

// ✅ Match types explicitly:
if ((p1 ^ p2) as ULONG_PTR) & !0xFu32 as ULONG_PTR != 0 { /* ... */ }
//                                  ^^^                     ^^^^^^^^^^^
//                            ULONG literal               cast to ULONG_PTR (usize on amd64)
```

When in doubt, use `as ULONG_PTR` / `as ULONG` casts and keep literals plain
hex without `usize` suffix.

---

## Pointer Patterns (Same as ntoskrnl)

- `NULL` → `core::ptr::null_mut()` (or `null()` for `*const`)
- `&ptr->field` → `core::ptr::addr_of_mut!((*ptr).field)` (or `&raw mut (*ptr).field`)
- `ptr->field` → `(*ptr).field`
- `RtlCopyMemory(dst, src, len)` → `core::ptr::copy_nonoverlapping(src, dst, len as usize)`
  (note: C arg order is dst-first, Rust is src-first!)
- `RtlZeroMemory(dst, len)` → `core::ptr::write_bytes(dst, 0, len as usize)`
- `RtlFillMemory(dst, len, val)` → `core::ptr::write_bytes(dst, val, len as usize)`

For raw-pointer indexing of static arrays — `(&raw mut ARRAY)[i]` does NOT
compile. Use `.cast().add(i)` instead:

```rust
// ❌ raw-pointer indexing doesn't work:
// let entry = &raw mut HalpPciDebuggingDevice[i];

// ✅ Use cast + add:
let entry = (&raw mut HalpPciDebuggingDevice).cast::<_PCI_TYPE1_CFG_CYCLE_BITS>().add(i);
```

---

## SEH / Exception Handling

HAL files rarely use SEH. If a C function does `_SEH2_TRY { ... } _SEH2_EXCEPT
(filter) { handler }`, treat the handler as dead code (DUMMY_PSEH config):
emit the TRY body directly, drop the handler block.

---

## goto Statements

C `goto cleanup;` → labeled break in Rust:

```rust
'cleanup: {
    if /* condition */ {
        status = STATUS_ERROR;
        break 'cleanup;
    }
    // ...success path...
}
// cleanup code here runs whether we fell through or `break 'cleanup`'d
```

For tight loops with continue/break semantics use `loop { ... break; }`.

---

## Style

- Keep C variable names (PascalCase OK at locals — they're not snake-case-lint-clean).
- Mark every function `unsafe` (HAL functions are always `unsafe extern`).
- `core::` not `std::`.
- No doc comments. No extra error handling. No `expect()` / `unwrap()`.
- Use `as _` for arch-dependent integer-width casts (`(*iosb).Information = bytes as _;`).
- One file = one C file. Preserve function order. Group inline helpers near
  the top.

---

## Quick "DO NOT" List

- ❌ Don't `use ntkrnl_sys::globals::*` / `ntkrnl_sys::constants::*` /
  `ntkrnl_sys::base::*` / `ntkrnl_sys::macros::*` — those modules exist for
  `ntoskrnl-rust` only. HAL only uses `ddk` and `hal_ddk`.
- ❌ Don't declare C macros (`FIELD_OFFSET`, `PAGE_ROUND_UP`, `UlongToPtr`,
  `WRITE_PORT_UCHAR`) as extern fns.
- ❌ Don't use enum-variant-name match arms (`_DMA_SPEED_Compatible =>`).
- ❌ Don't assign `extern "system"` fns directly into `Option<extern "stdcall" fn>`.
- ❌ Don't init dispatch tables at `static` scope when they contain fn pointers.
- ❌ Don't index raw pointers with `[i]` — use `.cast().add(i)`.
- ❌ Don't emit `core::ptr::null()` placeholders for `FIELD_OFFSET` args you
  can't figure out — use `core::mem::offset_of!(S, F)` directly.
- ❌ Don't reorder function bodies — emit forward decls for cycles instead.
