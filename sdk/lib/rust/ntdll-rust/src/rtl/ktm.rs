//! Kernel Transaction Manager surface (Vista) that programs probe: the msys2/Cygwin
//! runtime (Git for Windows) tries a transaction around unlink/rename and carries on
//! without one when NtCreateTransaction fails.
//!
//! Our kernel has no transaction manager, so creating, committing or rolling back a
//! transaction fails with STATUS_NOT_IMPLEMENTED. Wine instead hands out a fake handle
//! and reports success, which tells the caller its file operations are transacted when
//! they are not. The current-transaction accessors are real: they are the TEB field
//! Windows keeps, and with nothing able to create a transaction it stays NULL unless a
//! caller stores something there itself.

#![allow(non_snake_case)]

use ntdll_sys::nt::*;

const STATUS_NOT_IMPLEMENTED_: NTSTATUS = 0xC000_0002_u32 as NTSTATUS;

#[no_mangle]
pub unsafe extern "system" fn NtCreateTransaction(
    TransactionHandle: PHANDLE,
    _DesiredAccess: ACCESS_MASK,
    _ObjectAttributes: POBJECT_ATTRIBUTES,
    _Uow: *mut GUID,
    _TmHandle: HANDLE,
    _CreateOptions: ULONG,
    _IsolationLevel: ULONG,
    _IsolationFlags: ULONG,
    _Timeout: PLARGE_INTEGER,
    _Description: PUNICODE_STRING,
) -> NTSTATUS {
    if !TransactionHandle.is_null() {
        core::ptr::write_volatile(TransactionHandle, core::ptr::null_mut());
    }
    STATUS_NOT_IMPLEMENTED_
}

#[no_mangle]
pub unsafe extern "system" fn NtCommitTransaction(_TransactionHandle: HANDLE, _Wait: BOOLEAN) -> NTSTATUS {
    STATUS_NOT_IMPLEMENTED_
}

#[no_mangle]
pub unsafe extern "system" fn NtRollbackTransaction(_TransactionHandle: HANDLE, _Wait: BOOLEAN) -> NTSTATUS {
    STATUS_NOT_IMPLEMENTED_
}

#[no_mangle]
pub unsafe extern "system" fn RtlGetCurrentTransaction() -> HANDLE {
    (*NtCurrentTeb()).CurrentTransactionHandle
}

#[no_mangle]
pub unsafe extern "system" fn RtlSetCurrentTransaction(TransactionHandle: HANDLE) -> BOOLEAN {
    (*NtCurrentTeb()).CurrentTransactionHandle = TransactionHandle;
    1
}
