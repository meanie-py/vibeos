//! kernel32_vista/threadpool.c: the CreateThreadpool* / TrySubmitThreadpoolCallback wrappers
//! over ntdll_vista's NT threadpool (Tp*, the rtl-threadpool crate). Everything else in the
//! family (Submit/WaitFor/Close/Set*) is a straight spec forward to ntdll_vista, as upstream.
//!
//! One divergence: the C's CreateThreadpoolIo stores the win32 callback into the TP_IO
//! before checking TpAllocIoCompletion's status, so a failed allocation writes through an
//! uninitialised pointer. Here the store happens only on success.

use crate::common::SetLastError;
use ntdll_sys::nt::{BOOL, NTSTATUS, PVOID, ULONG};

/// PTP_WIN32_IO_CALLBACK: (Instance, Context, Overlapped, IoResult, NumberOfBytesTransferred, Io).
type Win32IoCallback = Option<unsafe extern "system" fn(PVOID, PVOID, PVOID, ULONG, usize, PVOID)>;
/// PTP_IO_CALLBACK (ntdll's): (Instance, Context, CompletionValue, IoStatusBlock, Io).
type NtIoCallback = Option<unsafe extern "system" fn(PVOID, PVOID, PVOID, *mut IoStatusBlock, PVOID)>;

#[repr(C)]
struct IoStatusBlock {
    Status: NTSTATUS,
    Information: usize,
}

extern "system" {
    fn RtlNtStatusToDosError(Status: NTSTATUS) -> ULONG;
    fn TpSimpleTryPost(callback: PVOID, userdata: PVOID, environment: PVOID) -> NTSTATUS;
    fn TpAllocWork(out: *mut PVOID, callback: PVOID, userdata: PVOID, environment: PVOID) -> NTSTATUS;
    fn TpAllocCleanupGroup(out: *mut PVOID) -> NTSTATUS;
    fn TpAllocWait(out: *mut PVOID, callback: PVOID, userdata: PVOID, environment: PVOID) -> NTSTATUS;
    fn TpAllocTimer(out: *mut PVOID, callback: PVOID, userdata: PVOID, environment: PVOID) -> NTSTATUS;
    fn TpAllocPool(out: *mut PVOID, reserved: PVOID) -> NTSTATUS;
    fn TpAllocIoCompletion(
        out: *mut PVOID,
        file: PVOID,
        callback: NtIoCallback,
        userdata: PVOID,
        environment: PVOID,
    ) -> NTSTATUS;
}

#[inline(always)]
fn nt_success(status: NTSTATUS) -> bool {
    status >= 0
}

/// The shared tail of every wrapper: a failed NTSTATUS becomes the thread's last error and a
/// NULL object; a success hands the object back.
unsafe fn finish(status: NTSTATUS, object: PVOID) -> PVOID {
    if !nt_success(status) {
        SetLastError(RtlNtStatusToDosError(status));
        return core::ptr::null_mut();
    }
    object
}

#[no_mangle]
pub unsafe extern "system" fn CreateThreadpoolTimer(callback: PVOID, userdata: PVOID, environment: PVOID) -> PVOID {
    let mut timer: PVOID = core::ptr::null_mut();
    let status = TpAllocTimer(&mut timer, callback, userdata, environment);
    finish(status, timer)
}

/// ntdll calls this with its NT-shaped arguments; it recovers the caller's win32 callback from
/// the slot ntdll reserves at the front of TP_IO and converts the status to a win32 error.
unsafe extern "system" fn tp_io_callback(
    instance: PVOID,
    userdata: PVOID,
    cvalue: PVOID,
    iosb: *mut IoStatusBlock,
    io: PVOID,
) {
    let callback: Win32IoCallback = core::mem::transmute(*(io as *mut PVOID));
    if let Some(callback) = callback {
        callback(instance, userdata, cvalue, RtlNtStatusToDosError((*iosb).Status), (*iosb).Information, io);
    }
}

#[no_mangle]
pub unsafe extern "system" fn CreateThreadpoolIo(
    handle: PVOID,
    callback: Win32IoCallback,
    userdata: PVOID,
    environment: PVOID,
) -> PVOID {
    let mut io: PVOID = core::ptr::null_mut();
    let status = TpAllocIoCompletion(&mut io, handle, Some(tp_io_callback), userdata, environment);
    if nt_success(status) {
        // ntdll leaves us space to store our callback at the beginning of the TP_IO.
        *(io as *mut PVOID) = core::mem::transmute::<Win32IoCallback, PVOID>(callback);
    }
    finish(status, io)
}

#[no_mangle]
pub unsafe extern "system" fn CreateThreadpoolWait(callback: PVOID, userdata: PVOID, environment: PVOID) -> PVOID {
    let mut wait: PVOID = core::ptr::null_mut();
    let status = TpAllocWait(&mut wait, callback, userdata, environment);
    finish(status, wait)
}

#[no_mangle]
pub unsafe extern "system" fn CreateThreadpoolWork(callback: PVOID, userdata: PVOID, environment: PVOID) -> PVOID {
    let mut work: PVOID = core::ptr::null_mut();
    let status = TpAllocWork(&mut work, callback, userdata, environment);
    finish(status, work)
}

#[no_mangle]
pub unsafe extern "system" fn TrySubmitThreadpoolCallback(callback: PVOID, userdata: PVOID, environment: PVOID) -> BOOL {
    let status = TpSimpleTryPost(callback, userdata, environment);
    if !nt_success(status) {
        SetLastError(RtlNtStatusToDosError(status));
        return 0;
    }
    1
}

#[no_mangle]
pub unsafe extern "system" fn CreateThreadpoolCleanupGroup() -> PVOID {
    let mut group: PVOID = core::ptr::null_mut();
    let status = TpAllocCleanupGroup(&mut group);
    finish(status, group)
}

#[no_mangle]
pub unsafe extern "system" fn CreateThreadpool(reserved: PVOID) -> PVOID {
    let mut pool: PVOID = core::ptr::null_mut();
    let status = TpAllocPool(&mut pool, reserved);
    finish(status, pool)
}
