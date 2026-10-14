//! Vista's cross-thread cancel and batched completion calls, over the Nt* system calls
//! of the same names (iomgr-rust's cancel_ex.rs / iocomp.rs, ke/ipi.rs).

use ntdll_sys::nt::*;
use crate::prelude::*;
use core::ffi::c_void;

/// ntstatus.h; ntdll_sys does not carry it.
const STATUS_USER_APC: NTSTATUS = 0x0000_00C0;

extern "system" {
    fn BaseFormatTimeOut(Timeout: PLARGE_INTEGER, dwMilliseconds: DWORD) -> PLARGE_INTEGER;
    fn NtCancelIoFileEx(
        FileHandle: HANDLE,
        IoRequestToCancel: PIO_STATUS_BLOCK,
        IoStatusBlock: PIO_STATUS_BLOCK,
    ) -> NTSTATUS;
    fn NtCancelSynchronousIoFile(
        ThreadHandle: HANDLE,
        IoRequestToCancel: PIO_STATUS_BLOCK,
        IoStatusBlock: PIO_STATUS_BLOCK,
    ) -> NTSTATUS;
    fn NtRemoveIoCompletionEx(
        IoCompletionHandle: HANDLE,
        IoCompletionInformation: *mut c_void,
        Count: ULONG,
        NumEntriesRemoved: PULONG,
        Timeout: PLARGE_INTEGER,
        Alertable: BOOLEAN,
    ) -> NTSTATUS;
    fn NtFlushProcessWriteBuffers() -> NTSTATUS;
}

/// `OVERLAPPED_ENTRY` (minwinbase.h). Same layout as the kernel's
/// FILE_IO_COMPLETION_INFORMATION {KeyContext, ApcContext, IO_STATUS_BLOCK}: the
/// overlapped pointer is the ApcContext, Internal the status, and the byte count the
/// low half of Information. Windows hands the array straight to the kernel; so do we.
#[repr(C)]
pub struct OVERLAPPED_ENTRY {
    pub lpCompletionKey: ULONG_PTR,
    pub lpOverlapped: LPOVERLAPPED,
    pub Internal: ULONG_PTR,
    pub dwNumberOfBytesTransferred: DWORD,
}

/// Cancel `hFile`'s pending I/O from every thread of the process, or only the request
/// `lpOverlapped` started. ERROR_NOT_FOUND when nothing was pending.
#[no_mangle]
pub unsafe extern "system" fn CancelIoEx(hFile: HANDLE, lpOverlapped: LPOVERLAPPED) -> BOOL {
    let mut iosb: IO_STATUS_BLOCK = core::mem::zeroed();
    let status = NtCancelIoFileEx(hFile, lpOverlapped as PIO_STATUS_BLOCK, &mut iosb);
    if !NT_SUCCESS(status) {
        BaseSetLastNTError(status);
        return FALSE as BOOL;
    }
    TRUE as BOOL
}

/// Cancel the synchronous I/O `hThread` is blocked in. ERROR_NOT_FOUND when it has none.
#[no_mangle]
pub unsafe extern "system" fn CancelSynchronousIo(hThread: HANDLE) -> BOOL {
    let mut iosb: IO_STATUS_BLOCK = core::mem::zeroed();
    let status = NtCancelSynchronousIoFile(hThread, core::ptr::null_mut(), &mut iosb);
    if !NT_SUCCESS(status) {
        BaseSetLastNTError(status);
        return FALSE as BOOL;
    }
    TRUE as BOOL
}

/// Up to `ulCount` completions at once. Waits (alertably if asked) for the first only.
/// Fails with WAIT_TIMEOUT, or WAIT_IO_COMPLETION when an APC ran instead.
#[no_mangle]
pub unsafe extern "system" fn GetQueuedCompletionStatusEx(
    CompletionPort: HANDLE,
    lpCompletionPortEntries: *mut OVERLAPPED_ENTRY,
    ulCount: ULONG,
    ulNumEntriesRemoved: PULONG,
    dwMilliseconds: DWORD,
    fAlertable: BOOL,
) -> BOOL {
    let mut time: LARGE_INTEGER = core::mem::zeroed();
    let timeout = BaseFormatTimeOut(&mut time, dwMilliseconds);
    let status = NtRemoveIoCompletionEx(
        CompletionPort,
        lpCompletionPortEntries as *mut c_void,
        ulCount,
        ulNumEntriesRemoved,
        timeout,
        (fAlertable != 0) as BOOLEAN,
    );
    if status == STATUS_SUCCESS {
        return TRUE as BOOL;
    }
    if !ulNumEntriesRemoved.is_null() {
        *ulNumEntriesRemoved = 0;
    }
    if status == STATUS_TIMEOUT {
        SetLastError(WAIT_TIMEOUT as DWORD);
    } else if status == STATUS_USER_APC {
        SetLastError(WAIT_IO_COMPLETION as DWORD);
    } else {
        BaseSetLastNTError(status);
    }
    FALSE as BOOL
}

/// Every processor running this process drains its store buffer before this returns.
#[no_mangle]
pub unsafe extern "system" fn FlushProcessWriteBuffers() {
    NtFlushProcessWriteBuffers();
}
