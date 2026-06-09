//! Rust translation of `sdk/lib/rtl/propvar.c` — native properties and
//! variants API (Pierre Schweitzer).
//!
//! Every public entry point is a thin thunk: load `ole32.dll` via
//! `LdrLoadDll`, resolve the matching `Stg*` export with
//! `LdrGetProcedureAddress`, forward the call, then `LdrUnloadDll`. rtl
//! treats `PROPVARIANT` and the serialized property blob opaquely (only
//! ever passes them by pointer), so we mirror the C `typedef PVOID`
//! aliases and keep `PROPVARIANT` as an opaque tag type.
//!
//! Under DUMMY_PSEH the C `_SEH2_FINALLY` (the `LdrUnloadDll` cleanup)
//! runs on the normal path, which is exactly the straight-line form here.
//! `RtlRaiseStatus` is `-> !`, so a failed `LdrLoadDll`/resolve never
//! reaches the unload (and `BaseAddress` is NULL on the load-failure path
//! anyway, so nothing leaks).
#![cfg_attr(not(test), no_std)]
#![allow(non_snake_case, non_upper_case_globals)]

use ntkrnl_sys::base::*; // PVOID, ULONG, USHORT, NTSTATUS, NT_SUCCESS, BOOLEAN, PCHAR, PULONG, PWSTR, UNICODE_STRING
use ntkrnl_sys::ddk::{ANSI_STRING, BYTE, LdrGetProcedureAddress, PCSZ, RtlInitAnsiString};
use ntkrnl_sys::rtl::RtlRaiseStatus;

// FIXME mirror of propvar.c: these Ole32 API were private till Vista, so
// rtl defines the types as PVOID to "allow build" — never dereferenced.
pub type PPMemoryAllocator = PVOID;
pub type PSERIALIZEDPROPERTYVALUE = PVOID;
pub type PROPID = ULONG;

/// `PROPVARIANT` is an OLE structure; rtl only forwards a pointer to it.
#[repr(C)]
pub struct PROPVARIANT {
    _opaque: [u8; 0],
}

// `UNICODE_STRING Old32Dll = RTL_CONSTANT_STRING(L"ole32.dll");`
// External linkage in C (no `static`); keep it #[no_mangle].
static OLE32_DLL_NAME: [u16; 10] = [
    0x006f, 0x006c, 0x0065, 0x0033, 0x0032, 0x002e, 0x0064, 0x006c, 0x006c, 0x0000,
]; // "ole32.dll\0"

#[no_mangle]
pub static mut Old32Dll: UNICODE_STRING = UNICODE_STRING {
    Length: 18,        // 9 wchars * 2, excluding NUL
    MaximumLength: 20, // 10 wchars * 2, including NUL
    Buffer: core::ptr::addr_of!(OLE32_DLL_NAME) as PWSTR,
};

// NTAPI loader helpers not surfaced by ntkrnl-sys bindgen. Declared
// extern "system" (stdcall @N on i386, cdecl elsewhere).
extern "system" {
    fn LdrLoadDll(
        SearchPath: PWSTR,
        DllCharacteristics: PULONG,
        DllName: *mut UNICODE_STRING,
        BaseAddress: *mut PVOID,
    ) -> NTSTATUS;
    fn LdrUnloadDll(BaseAddress: PVOID) -> NTSTATUS;
}

/// Internal (cdecl) helper: load ole32.dll and resolve `ProcedureName`.
#[no_mangle]
pub unsafe extern "C" fn LoadOle32Export(BaseAddress: *mut PVOID, ProcedureName: PCHAR) -> PVOID {
    let mut ExportName: ANSI_STRING = core::mem::zeroed();
    let mut ProcedureAddress: PVOID = core::ptr::null_mut();

    // First load ole32.dll
    let mut Status = LdrLoadDll(
        core::ptr::null_mut(),
        core::ptr::null_mut(),
        core::ptr::addr_of_mut!(Old32Dll),
        BaseAddress,
    );
    if !NT_SUCCESS(Status) {
        RtlRaiseStatus(Status);
    }

    RtlInitAnsiString(core::ptr::addr_of_mut!(ExportName), ProcedureName as PCSZ);

    // Look for the procedure
    Status = LdrGetProcedureAddress(
        *BaseAddress,
        core::ptr::addr_of_mut!(ExportName),
        0,
        core::ptr::addr_of_mut!(ProcedureAddress),
    );
    if !NT_SUCCESS(Status) {
        RtlRaiseStatus(Status);
    }

    ProcedureAddress
}

/// @implemented
#[no_mangle]
pub unsafe extern "system" fn PropertyLengthAsVariant(
    pProp: PSERIALIZEDPROPERTYVALUE,
    cbProp: ULONG,
    CodePage: USHORT,
    bReserved: BYTE,
) -> ULONG {
    let mut BaseAddress: PVOID = core::ptr::null_mut();

    let ProcedureAddress: unsafe extern "system" fn(
        PSERIALIZEDPROPERTYVALUE,
        ULONG,
        USHORT,
        BYTE,
    ) -> ULONG = core::mem::transmute(LoadOle32Export(
        core::ptr::addr_of_mut!(BaseAddress),
        b"StgPropertyLengthAsVariant\0".as_ptr() as PCHAR,
    ));

    let Length = ProcedureAddress(pProp, cbProp, CodePage, bReserved);

    if !BaseAddress.is_null() {
        LdrUnloadDll(BaseAddress);
    }

    Length
}

/// @implemented
#[no_mangle]
pub unsafe extern "system" fn RtlConvertPropertyToVariant(
    prop: PSERIALIZEDPROPERTYVALUE,
    CodePage: USHORT,
    pvar: *mut PROPVARIANT,
    pma: PPMemoryAllocator,
) -> BOOLEAN {
    let mut BaseAddress: PVOID = core::ptr::null_mut();

    let ProcedureAddress: unsafe extern "system" fn(
        PSERIALIZEDPROPERTYVALUE,
        USHORT,
        *mut PROPVARIANT,
        PPMemoryAllocator,
    ) -> BOOLEAN = core::mem::transmute(LoadOle32Export(
        core::ptr::addr_of_mut!(BaseAddress),
        b"StgConvertPropertyToVariant\0".as_ptr() as PCHAR,
    ));

    let Success = ProcedureAddress(prop, CodePage, pvar, pma);

    if !BaseAddress.is_null() {
        LdrUnloadDll(BaseAddress);
    }

    Success
}

/// @implemented
#[no_mangle]
pub unsafe extern "system" fn RtlConvertVariantToProperty(
    pvar: *const PROPVARIANT,
    CodePage: USHORT,
    pprop: PSERIALIZEDPROPERTYVALUE,
    pcb: PULONG,
    pid: PROPID,
    fReserved: BOOLEAN,
    pcIndirect: PULONG,
) -> PSERIALIZEDPROPERTYVALUE {
    let mut BaseAddress: PVOID = core::ptr::null_mut();

    let ProcedureAddress: unsafe extern "system" fn(
        *const PROPVARIANT,
        USHORT,
        PSERIALIZEDPROPERTYVALUE,
        PULONG,
        PROPID,
        BOOLEAN,
        PULONG,
    ) -> PSERIALIZEDPROPERTYVALUE = core::mem::transmute(LoadOle32Export(
        core::ptr::addr_of_mut!(BaseAddress),
        b"StgConvertVariantToProperty\0".as_ptr() as PCHAR,
    ));

    let Serialized = ProcedureAddress(pvar, CodePage, pprop, pcb, pid, fReserved, pcIndirect);

    if !BaseAddress.is_null() {
        LdrUnloadDll(BaseAddress);
    }

    Serialized
}
