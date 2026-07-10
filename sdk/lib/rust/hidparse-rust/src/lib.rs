//! Rust translation of `drivers/hid/hidparse/hidparse.c` — the HID parser
//! `.sys` wrapper. It is a thin shim: a near-empty `DriverEntry` plus the five
//! allocator/utility callbacks that the `hidparser_km` C library
//! (`sdk/lib/drivers/hidparser`, kept C in Batch 1) calls **by name** as
//! external symbols (see `hidp.h`).
//!
//! Faithful port of the C:
//!   * `AllocFunction`  — `ExAllocatePoolWithTag(NonPagedPool, size, 'PdiH')`,
//!                        zero-fill on success, return the block.
//!   * `FreeFunction`   — `ExFreePoolWithTag(item, 'PdiH')`.
//!   * `ZeroFunction`   — `RtlZeroMemory` (byte fill 0).
//!   * `CopyFunction`   — `RtlCopyMemory` (non-overlapping copy).
//!   * `DebugFunction`  — guarded by `#if HID_DBG`; HID_DBG is undefined in the
//!                        tree, so the C body is empty → an empty cdecl stub.
//!   * `DriverEntry`    — DPRINT (dropped) + return STATUS_SUCCESS.
//!
//! Substrate: only `ExAllocatePoolWithTag` / `ExFreePoolWithTag` (ntoskrnl
//! exports), declared in a local `extern "system"` block → self-contained (no
//! ntkrnl-sys dependency). The zero/copy lower to `memset`/`memcpy`, which
//! ntoskrnl also exports.
//!
//! ABI: the 4 util callbacks are `NTAPI` (`extern "system"`: stdcall `_Name@N`
//! on i386, cdecl on amd64/arm64); `DebugFunction` is `__cdecl` variadic — a
//! non-variadic `extern "C"` stub is ABI-safe (cdecl is caller-cleaned, so the
//! callee may ignore the varargs). `DriverEntry` is `NTAPI`.
//!
//! crate-type = staticlib → `libhidparse_rust.a` (cohabitation; hidparse.c kept
//! on disk per the never-delete rule).

#![no_std]
#![allow(non_snake_case, non_camel_case_types, non_upper_case_globals)]
#![allow(clippy::missing_safety_doc)]

use core::ffi::c_void;

type NTSTATUS = i32;
type ULONG = u32;
type PVOID = *mut c_void;

const STATUS_SUCCESS: NTSTATUS = 0;

// hidparse.h: #define HIDPARSE_TAG 'PdiH'  (pool tag, reads "HidP" in memory)
const HIDPARSE_TAG: u32 = 0x5064_6948; // 'P'<<24 | 'd'<<16 | 'i'<<8 | 'H'

// POOL_TYPE::NonPagedPool
const NonPagedPool: i32 = 0;

extern "system" {
    // PVOID ExAllocatePoolWithTag(POOL_TYPE, SIZE_T, ULONG) — ntoskrnl export.
    fn ExAllocatePoolWithTag(PoolType: i32, NumberOfBytes: usize, Tag: u32) -> PVOID;
    // VOID ExFreePoolWithTag(PVOID, ULONG) — ntoskrnl export.
    fn ExFreePoolWithTag(P: PVOID, Tag: u32);
}

#[no_mangle]
pub unsafe extern "system" fn AllocFunction(ItemSize: ULONG) -> PVOID {
    let item = ExAllocatePoolWithTag(NonPagedPool, ItemSize as usize, HIDPARSE_TAG);
    if !item.is_null() {
        // zero item
        core::ptr::write_bytes(item as *mut u8, 0, ItemSize as usize);
    }
    item
}

#[no_mangle]
pub unsafe extern "system" fn FreeFunction(Item: PVOID) {
    // free item
    ExFreePoolWithTag(Item, HIDPARSE_TAG);
}

#[no_mangle]
pub unsafe extern "system" fn ZeroFunction(Item: PVOID, ItemSize: ULONG) {
    // zero item
    core::ptr::write_bytes(Item as *mut u8, 0, ItemSize as usize);
}

#[no_mangle]
pub unsafe extern "system" fn CopyFunction(Target: PVOID, Source: PVOID, Length: ULONG) {
    // copy item (RtlCopyMemory — non-overlapping)
    core::ptr::copy_nonoverlapping(Source as *const u8, Target as *mut u8, Length as usize);
}

#[no_mangle]
pub unsafe extern "C" fn DebugFunction(_FormatStr: *const i8) {
    // #if HID_DBG ... #endif — HID_DBG is undefined in the tree → empty body.
}

#[no_mangle]
pub unsafe extern "system" fn DriverEntry(
    _DriverObject: PVOID,  // PDRIVER_OBJECT
    _RegPath: PVOID,       // PUNICODE_STRING
) -> NTSTATUS {
    // DPRINT("********* HID PARSE *********\n") — dropped (debug-only).
    STATUS_SUCCESS
}

#[panic_handler]
fn panic(info: &core::panic::PanicInfo) -> ! {
    panic_common::panic_common(env!("CARGO_PKG_NAME"), info)
}

#[no_mangle]
pub extern "C" fn rust_eh_personality() {}
