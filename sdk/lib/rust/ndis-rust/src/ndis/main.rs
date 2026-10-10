use crate::prelude::*;
use ntkrnl_sys::ddk::{PDRIVER_OBJECT, PUNICODE_STRING};
use core::ffi::c_void;

// ---- file-scope preamble (translated) ----
// @c2rust file-scope data (compiler-exact tables/statics)
#[no_mangle]
pub static mut DebugTraceLevel: ULONG = MIN_TRACE as ULONG;

#[no_mangle]
pub static mut CancelId: LONG = 0;

#[no_mangle]
pub unsafe extern "system" fn MainUnload(DriverObject: PDRIVER_OBJECT) {
    NDIS_DbgPrint!(MAX_TRACE, b"Leaving.\n\0".as_ptr() as *const i8);
}

#[no_mangle]
pub unsafe extern "system" fn DriverEntry(
    DriverObject: PDRIVER_OBJECT,
    RegistryPath: PUNICODE_STRING,
) -> NTSTATUS {
    NDIS_DbgPrint!(MAX_TRACE, b"Called.\n\0".as_ptr() as PCSTR);

    InitializeListHead(core::ptr::addr_of_mut!(ProtocolListHead));
    KeInitializeSpinLock(core::ptr::addr_of_mut!(ProtocolListLock));

    InitializeListHead(core::ptr::addr_of_mut!(MiniportListHead));
    KeInitializeSpinLock(core::ptr::addr_of_mut!(MiniportListLock));

    InitializeListHead(core::ptr::addr_of_mut!(AdapterListHead));
    KeInitializeSpinLock(core::ptr::addr_of_mut!(AdapterListLock));

    (*DriverObject).DriverUnload =
        Some(MainUnload);

    CancelId = 0 as LONG;

    return STATUS_SUCCESS;
}

/// C-variadic in the C (`_cdecl`, `...`), but the body reads only the three fixed arguments,
/// and on all three ABIs a cdecl caller's fixed integer arguments land where a 3-argument
/// callee reads them (i386: the caller pops). So the fixed-arity definition is the whole
/// function; stable Rust could not define the variadic one anyway.
#[no_mangle]
pub unsafe extern "C" fn NdisWriteErrorLogEntry(
    NdisAdapterHandle: NDIS_HANDLE,
    ErrorCode: NDIS_ERROR_CODE,
    NumberOfErrorValues: ULONG,
) {
    NDIS_DbgPrint!(MIN_TRACE, b"ERROR: ErrorCode 0x%x\n\0".as_ptr(), ErrorCode);
    /* ASSERT(0); */
}

#[no_mangle]
pub unsafe extern "system" fn NdisWriteEventLogEntry(
    LogHandle: PVOID,
    EventCode: NDIS_STATUS,
    UniqueEventValue: ULONG,
    NumStrings: USHORT,
    StringsList: PVOID,
    DataSize: ULONG,
    Data: PVOID,
) -> NDIS_STATUS {
    NDIS_DbgPrint!(MAX_TRACE, b"Called.\n\0".as_ptr() as PCSTR);
    return NDIS_STATUS_SUCCESS;
}
