//! Rust providers for the C macros / MSVC intrinsics / COM-vtbl dispatchers the
//! consrv translation references — the in-Rust replacement for the old
//! `consrv_rust_shim.c` (deleted). Each `#[no_mangle]` fn reproduces the exact
//! symbol the C shim emitted: on i386 the natural `extern "system"` (stdcall)
//! decoration gives `_Name@N` / `__Interlocked*@N`, and `extern "C"` gives the
//! cdecl `_Name`; on amd64/arm64 every name is undecorated. Verified symbol sets
//! (i386 / amd64) match the C object 1:1 — a mismatch fails the link loudly.
//!
//! Nothing here is novel logic; it is a faithful transliteration of the C shim:
//! interlocked ops (atomics), CONTAINING_RECORD (pointer-minus-offset),
//! InitializeObjectAttributes (struct init), IS_FULL_WIDTH (calls the shared wcwidth-rust
//! `mk_wcwidth_cjk`), IsListEmpty, and the IShellLinkW / IPersistFile COBJMACRO
//! dispatchers (call through the COM object's vtbl).

use core::ffi::c_void;
use core::sync::atomic::{AtomicI32, Ordering::SeqCst};

type LONG = i32;
type ULONG = u32;
type HRESULT = i32;
type WCHAR = u16;
type PVOID = *mut c_void;
type LPVOID = *mut c_void;

#[inline(always)]
unsafe fn atomic(p: *mut LONG) -> &'static AtomicI32 {
    // AtomicI32 has the same layout/ABI as i32; the consrv callers pass a
    // pointer to a live LONG counter.
    &*(p as *const AtomicI32)
}

// ---- MSVC interlocked intrinsics ------------------------------------------
// InterlockedIncrement/Decrement return the NEW value; Exchange the OLD value.
// i386 callers use both decorations: condrv (extern "system") -> `__Interlocked*@N`,
// terminal.rs (extern "C") -> `__InterlockedDecrement`.

/// i386 `__InterlockedIncrement@4` / amd64 `_InterlockedIncrement`.
#[no_mangle]
pub unsafe extern "system" fn _InterlockedIncrement(addend: *mut LONG) -> LONG {
    atomic(addend).fetch_add(1, SeqCst).wrapping_add(1)
}

/// i386 `__InterlockedDecrement@4` / amd64 `_InterlockedDecrement`.
#[no_mangle]
pub unsafe extern "system" fn _InterlockedDecrement(addend: *mut LONG) -> LONG {
    atomic(addend).fetch_sub(1, SeqCst).wrapping_sub(1)
}

/// i386 `__InterlockedExchange@8` / amd64 `_InterlockedExchange`.
#[no_mangle]
pub unsafe extern "system" fn _InterlockedExchange(target: *mut LONG, value: LONG) -> LONG {
    atomic(target).swap(value, SeqCst)
}

// ---- NT macros -------------------------------------------------------------

/// `CONTAINING_RECORD(addr, type, field_off)` — type is unused; on i386 the
/// offset is a 4-byte ULONG, on amd64/arm64 a pointer-sized value (usize covers
/// both). i386 `_CONTAINING_RECORD@12` / amd64 `CONTAINING_RECORD`.
#[no_mangle]
pub unsafe extern "system" fn CONTAINING_RECORD(
    addr: *const u8,
    _type: usize,
    field_off: usize,
) -> *mut u8 {
    (addr as usize - field_off) as *mut u8
}

#[repr(C)]
struct ObjectAttributes {
    length: ULONG,
    root_directory: PVOID,
    object_name: PVOID,
    attributes: ULONG,
    security_descriptor: PVOID,
    security_quality_of_service: PVOID,
}

/// `InitializeObjectAttributes(p, ObjectName, Attributes, RootDir, SecDesc)`.
/// i386 `_InitializeObjectAttributes@20` / amd64 `InitializeObjectAttributes`.
#[no_mangle]
pub unsafe extern "system" fn InitializeObjectAttributes(
    p: *mut c_void,
    object_name: PVOID,
    attributes: ULONG,
    root_directory: PVOID,
    security_descriptor: PVOID,
) {
    let oa = p as *mut ObjectAttributes;
    (*oa).length = core::mem::size_of::<ObjectAttributes>() as ULONG;
    (*oa).root_directory = root_directory;
    (*oa).object_name = object_name;
    (*oa).attributes = attributes;
    (*oa).security_descriptor = security_descriptor;
    (*oa).security_quality_of_service = core::ptr::null_mut();
}

use wcwidth_rust::mk_wcwidth_cjk;

/// `IS_FULL_WIDTH(wch)` — wide iff >= 0x0080 and CJK width 2.
/// i386 `_IS_FULL_WIDTH@4` / amd64 `IS_FULL_WIDTH`.
#[no_mangle]
pub unsafe extern "system" fn IS_FULL_WIDTH(wch: WCHAR) -> ULONG {
    (wch >= 0x0080 && mk_wcwidth_cjk(wch) == 2) as ULONG
}

/// `IsListEmpty(ListHead)` — empty iff Flink (first field) points back at head.
/// i386 cdecl `_IsListEmpty` / amd64 `IsListEmpty`.
#[no_mangle]
pub unsafe extern "C" fn IsListEmpty(list_head: *const c_void) -> u8 {
    (*(list_head as *const *const c_void) == list_head) as u8
}

// ---- IShellLinkW / IPersistFile COBJMACRO dispatchers ----------------------
// A COM object is `{ const Vtbl **lpVtbl; ... }`; method `idx` takes `this` as
// its first (__stdcall) arg. vtbl: 0 QI, 1 AddRef, 2 Release; IShellLinkW adds
// 14 GetShowCmd, 16 GetIconLocation; IPersistFile adds 5 Load.

#[inline(always)]
unsafe fn com_method(p: LPVOID, idx: usize) -> *const c_void {
    let vtbl = *(p as *const *const *const c_void);
    *vtbl.add(idx)
}

/// i386 `_IShellLinkW_GetShowCmd@8` / amd64 `IShellLinkW_GetShowCmd`.
#[no_mangle]
pub unsafe extern "system" fn IShellLinkW_GetShowCmd(pshl: LPVOID, show_cmd: *mut i32) -> HRESULT {
    let m: extern "system" fn(LPVOID, *mut i32) -> HRESULT =
        core::mem::transmute(com_method(pshl, 14));
    m(pshl, show_cmd)
}

/// i386 `_IShellLinkW_GetIconLocation@16` / amd64 `IShellLinkW_GetIconLocation`.
#[no_mangle]
pub unsafe extern "system" fn IShellLinkW_GetIconLocation(
    pshl: LPVOID,
    icon_path: *mut WCHAR,
    cch_icon_path: i32,
    icon_index: *mut i32,
) -> HRESULT {
    let m: extern "system" fn(LPVOID, *mut WCHAR, i32, *mut i32) -> HRESULT =
        core::mem::transmute(com_method(pshl, 16));
    m(pshl, icon_path, cch_icon_path, icon_index)
}

/// i386 `_IShellLinkW_Release@4` / amd64 `IShellLinkW_Release`.
#[no_mangle]
pub unsafe extern "system" fn IShellLinkW_Release(pshl: LPVOID) -> ULONG {
    let m: extern "system" fn(LPVOID) -> ULONG = core::mem::transmute(com_method(pshl, 2));
    m(pshl)
}

/// i386 `_IPersistFile_QueryInterface@12` / amd64 `IPersistFile_QueryInterface`.
#[no_mangle]
pub unsafe extern "system" fn IPersistFile_QueryInterface(
    pshl: LPVOID,
    riid: *const c_void,
    ppv_object: *mut LPVOID,
) -> HRESULT {
    let m: extern "system" fn(LPVOID, *const c_void, *mut LPVOID) -> HRESULT =
        core::mem::transmute(com_method(pshl, 0));
    m(pshl, riid, ppv_object)
}

/// i386 `_IPersistFile_Load@12` / amd64 `IPersistFile_Load`.
#[no_mangle]
pub unsafe extern "system" fn IPersistFile_Load(
    ppf: LPVOID,
    psz_file_name: *const WCHAR,
    dw_mode: ULONG,
) -> HRESULT {
    let m: extern "system" fn(LPVOID, *const WCHAR, ULONG) -> HRESULT =
        core::mem::transmute(com_method(ppf, 5));
    m(ppf, psz_file_name, dw_mode)
}

/// i386 `_IPersistFile_Release@4` / amd64 `IPersistFile_Release`.
#[no_mangle]
pub unsafe extern "system" fn IPersistFile_Release(ppf: LPVOID) -> ULONG {
    let m: extern "system" fn(LPVOID) -> ULONG = core::mem::transmute(com_method(ppf, 2));
    m(ppf)
}
