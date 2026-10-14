//! NtCancelIoFileEx / NtCancelSynchronousIoFile (Vista): cancel I/O that OTHER threads
//! issued. kernel32's CancelIoEx and CancelSynchronousIo sit on these.
//!
//! A thread's `IrpList` belongs to that thread: IopQueueIrpToThread links an IRP under a
//! guarded region of the issuing thread, and completion unlinks it in that thread's own
//! APC, so no lock covers the list and no other thread may walk it. NtCancelIoFile (the
//! XP call) gets away with this because it only cancels the CALLER's IRPs. These calls
//! instead run the walk AS each owner: a special kernel APC queued to every target thread
//! cancels its matching IRPs at APC_LEVEL in its own context, and the caller waits until
//! every APC has reported. Like Windows, neither waits for the cancelled IRPs to complete.

use core::sync::atomic::{AtomicI32, AtomicU32, Ordering};

use ntkrnl_sys::globals::*;
use ntkrnl_sys::base::*;
use ntkrnl_sys::constants::*;
use ntkrnl_sys::macros::*;
use ntkrnl_sys::ke::*;
use ntkrnl_sys::mm::*;
use ntkrnl_sys::ob::*;
use ntkrnl_sys::ex::*;
use ntkrnl_sys::probe::*;
use ntkrnl_sys::ps::PsGetCurrentProcess;
use ntkrnl_sys::{opaque_ptr, opaque_ptr_const};

use ntkrnl_sys::ddk::{
    ObReferenceObjectByHandle, ACCESS_MASK, _ETHREAD, _IO_STATUS_BLOCK, _IRP, _KAPC, _KEVENT,
    PEPROCESS, PETHREAD, PFILE_OBJECT, PIRP, PIO_STATUS_BLOCK, PKAPC, PKNORMAL_ROUTINE, PKTHREAD,
    PLIST_ENTRY,
};

extern "C" {
    // POBJECT_TYPE; declared here because ntkrnl-sys has it only in the x86 HAL island.
    static mut PsThreadType: PVOID;
}

extern "system" {
    fn PsGetNextProcessThread(Process: PEPROCESS, Thread: PETHREAD) -> PETHREAD;
    fn PsGetCurrentThread() -> PETHREAD;
    fn KeInitializeApc(
        Apc: PKAPC,
        Thread: PKTHREAD,
        Environment: u32,
        KernelRoutine: Option<
            unsafe extern "system" fn(PKAPC, *mut PKNORMAL_ROUTINE, *mut PVOID, *mut PVOID, *mut PVOID),
        >,
        RundownRoutine: Option<unsafe extern "system" fn(PKAPC)>,
        NormalRoutine: PKNORMAL_ROUTINE,
        Mode: i8,
        Context: PVOID,
    );
    fn KeInsertQueueApc(Apc: PKAPC, Arg1: PVOID, Arg2: PVOID, Boost: i32) -> u8;
    fn KeInitializeEvent(Event: *mut _KEVENT, Type: u32, State: u8);
    fn KeSetEvent(Event: *mut _KEVENT, Increment: i32, Wait: u8) -> i32;
    fn KeWaitForSingleObject(
        Object: PVOID,
        WaitReason: u32,
        WaitMode: i8,
        Alertable: u8,
        Timeout: *mut LARGE_INTEGER,
    ) -> NTSTATUS;
    fn ExAllocatePoolWithTag(PoolType: ULONG, NumberOfBytes: usize, Tag: ULONG) -> PVOID;
    fn ExFreePoolWithTag(P: PVOID, Tag: ULONG);
    fn IoCancelIrp(Irp: PIRP) -> u8;
}

const TAG_CANCEL: ULONG = u32::from_le_bytes(*b"IoCx");
const NON_PAGED_POOL: ULONG = 0;
const NOTIFICATION_EVENT: u32 = 0;
const ORIGINAL_APC_ENVIRONMENT: u32 = 0;
const EXECUTIVE: u32 = 0;
const THREAD_TERMINATE: ACCESS_MASK = 0x0001;
/// `IRP_SYNCHRONOUS_API` (iotypes.h): the I/O manager waits on its own event for this IRP.
const IRP_SYNCHRONOUS_API: ULONG = 0x0004;
/// `FO_SYNCHRONOUS_IO` (iotypes.h): the file object was opened for synchronous I/O.
const FO_SYNCHRONOUS_IO: ULONG = 0x0002;

/// A request the issuing thread is blocked in: either the I/O manager waits on its own
/// event (`IRP_SYNCHRONOUS_API`), or the handle was opened for synchronous I/O, where
/// NtReadFile and friends leave the IRP flag clear and wait on the file object instead.
/// The guest test's blocked ReadFile on an anonymous pipe is the second kind.
unsafe fn is_synchronous(irp: PIRP) -> bool {
    let file = (*irp).Tail.Overlay.OriginalFileObject;
    (*irp).Flags & IRP_SYNCHRONOUS_API != 0
        || (!file.is_null() && (*file).Flags & FO_SYNCHRONOUS_IO != 0)
}

/// What to cancel, shared by the caller (on its stack, alive until every APC reports)
/// and the APCs.
struct CancelRequest {
    /// Only IRPs on this file object; null for any file (CancelSynchronousIo).
    file: PFILE_OBJECT,
    /// Only the IRP whose `UserIosb` is this; null for all of them.
    iosb: PVOID,
    sync_only: bool,
    found: AtomicU32,
    /// APCs not yet reported, plus one held by the caller while it queues them.
    outstanding: AtomicI32,
    done: _KEVENT,
}

#[repr(C)]
struct CancelApc {
    apc: _KAPC,
    req: *const CancelRequest,
}

/// Cancel the current thread's IRPs that `req` matches; the caller is at APC_LEVEL or
/// in the thread's own APC, so the list cannot change underneath.
unsafe fn cancel_own_irps(req: &CancelRequest) -> u32 {
    let thread = PsGetCurrentThread();
    let head: PLIST_ENTRY = core::ptr::addr_of_mut!((*(thread as *mut _ETHREAD)).IrpList);
    let mut entry = (*head).Flink;
    let mut n = 0;
    while entry != head {
        let irp = (entry as usize - core::mem::offset_of!(_IRP, ThreadListEntry)) as PIRP;
        // IoCancelIrp can complete the IRP and unlink it: step first.
        entry = (*entry).Flink;
        let matches = (req.file.is_null() || (*irp).Tail.Overlay.OriginalFileObject == req.file)
            && (req.iosb.is_null() || (*irp).UserIosb as PVOID == req.iosb)
            && (!req.sync_only || is_synchronous(irp));
        if matches {
            IoCancelIrp(irp);
            n += 1;
        }
    }
    n
}

unsafe fn report(req: &CancelRequest) {
    if req.outstanding.fetch_sub(1, Ordering::AcqRel) == 1 {
        KeSetEvent(&req.done as *const _ as *mut _KEVENT, 0, 0);
    }
}

unsafe extern "system" fn cancel_apc_kernel(
    apc: PKAPC,
    _normal: *mut PKNORMAL_ROUTINE,
    _ctx: *mut PVOID,
    _a1: *mut PVOID,
    _a2: *mut PVOID,
) {
    let c = apc as *mut CancelApc;
    let req = &*(*c).req;
    req.found.fetch_add(cancel_own_irps(req), Ordering::AcqRel);
    ExFreePoolWithTag(c as PVOID, TAG_CANCEL);
    report(req);
}

/// The thread exited with the APC still queued: it has no IRPs left to cancel.
unsafe extern "system" fn cancel_apc_rundown(apc: PKAPC) {
    let c = apc as *mut CancelApc;
    let req = &*(*c).req;
    ExFreePoolWithTag(c as PVOID, TAG_CANCEL);
    report(req);
}

/// Run `req` in `thread`: inline for the current thread, else by special kernel APC.
unsafe fn cancel_in_thread(req: &CancelRequest, thread: PETHREAD) {
    if thread == PsGetCurrentThread() {
        let guard = ntkrnl_sys::guards::IrqlGuard::raise(APC_LEVEL as u8);
        req.found.fetch_add(cancel_own_irps(req), Ordering::AcqRel);
        drop(guard);
        return;
    }
    let c = ExAllocatePoolWithTag(NON_PAGED_POOL, core::mem::size_of::<CancelApc>(), TAG_CANCEL)
        as *mut CancelApc;
    if c.is_null() {
        // Out of pool: that thread's I/O is not cancelled; the count says what was.
        return;
    }
    (*c).req = req;
    req.outstanding.fetch_add(1, Ordering::AcqRel);
    KeInitializeApc(
        core::ptr::addr_of_mut!((*c).apc),
        core::ptr::addr_of_mut!((*(thread as *mut _ETHREAD)).Tcb) as PKTHREAD,
        ORIGINAL_APC_ENVIRONMENT,
        Some(cancel_apc_kernel),
        Some(cancel_apc_rundown),
        None, // special kernel APC: runs even while the thread waits in the kernel
        0,
        core::ptr::null_mut(),
    );
    if KeInsertQueueApc(core::ptr::addr_of_mut!((*c).apc), core::ptr::null_mut(), core::ptr::null_mut(), 0) == 0 {
        // The thread no longer takes APCs (it is exiting): nothing of its to cancel.
        ExFreePoolWithTag(c as PVOID, TAG_CANCEL);
        report(req);
    }
}

unsafe fn finish(req: &CancelRequest, IoStatusBlock: PIO_STATUS_BLOCK, mode: i8) -> NTSTATUS {
    // Drop the caller's hold; wait for the APCs that are still out.
    if req.outstanding.fetch_sub(1, Ordering::AcqRel) != 1 {
        KeWaitForSingleObject(
            &req.done as *const _ as PVOID,
            EXECUTIVE,
            0,
            0,
            core::ptr::null_mut(),
        );
    }
    let status = if req.found.load(Ordering::Acquire) != 0 { STATUS_SUCCESS } else { STATUS_NOT_FOUND };
    let iosb = _IO_STATUS_BLOCK {
        __bindgen_anon_1: ntkrnl_sys::ddk::_IO_STATUS_BLOCK__bindgen_ty_1 { Status: status },
        Information: 0,
    };
    match write_user(IoStatusBlock, iosb, mode) {
        Ok(()) => status,
        Err(e) => e,
    }
}

/// The event is NOT initialized here: a KEVENT's wait-list head points at its own
/// address, so one initialized in a local and then returned by value points at the
/// dead copy, and the first waiter links its wait block into garbage (that was a
/// BUGCHECK 0x109 / 0xA in the first guest run). `arm` initializes it in place.
unsafe fn new_request(file: PFILE_OBJECT, iosb: PVOID, sync_only: bool) -> CancelRequest {
    CancelRequest {
        file,
        iosb,
        sync_only,
        found: AtomicU32::new(0),
        outstanding: AtomicI32::new(1),
        done: core::mem::zeroed(),
    }
}

unsafe fn arm(req: &mut CancelRequest) {
    KeInitializeEvent(&mut req.done, NOTIFICATION_EVENT, 0);
}

/// Cancel every pending request on `FileHandle` issued by any thread of this process,
/// or only the one whose IO_STATUS_BLOCK is `IoRequestToCancel`. STATUS_NOT_FOUND when
/// nothing matched.
#[no_mangle]
pub unsafe extern "system" fn NtCancelIoFileEx(
    FileHandle: HANDLE,
    IoRequestToCancel: PIO_STATUS_BLOCK,
    IoStatusBlock: PIO_STATUS_BLOCK,
) -> NTSTATUS {
    let FileHandle = opaque_ptr(FileHandle);
    let IoStatusBlock = opaque_ptr(IoStatusBlock);
    let mode = KeGetPreviousMode();
    let mut FileObject: PFILE_OBJECT = core::ptr::null_mut();

    if mode != 0 {
        if let Err(e) = probe_user_buffer_write(
            IoStatusBlock as *const core::ffi::c_void,
            core::mem::size_of::<_IO_STATUS_BLOCK>(),
            core::mem::align_of::<_IO_STATUS_BLOCK>(),
            mode,
        ) {
            return e;
        }
    }

    let status = ObReferenceObjectByHandle(
        FileHandle,
        0,
        IoFileObjectType as _,
        mode,
        &mut FileObject as *mut _ as *mut PVOID,
        core::ptr::null_mut(),
    );
    if status < 0 {
        return status;
    }

    // Compared, never dereferenced: a user pointer needs no probe.
    let mut req = new_request(FileObject, opaque_ptr_const(IoRequestToCancel) as PVOID, false);
    arm(&mut req);
    let process = PsGetCurrentProcess() as PEPROCESS;
    let mut thread = PsGetNextProcessThread(process, core::ptr::null_mut());
    while !thread.is_null() {
        cancel_in_thread(&req, thread);
        thread = PsGetNextProcessThread(process, thread);
    }
    let status = finish(&req, IoStatusBlock, mode);

    ObDereferenceObject(FileObject as PVOID);
    status
}

/// Cancel the synchronous request `ThreadHandle`'s thread is blocked in (any file), or
/// only the one whose IO_STATUS_BLOCK is `IoRequestToCancel`. Needs THREAD_TERMINATE,
/// as on Windows. STATUS_NOT_FOUND when the thread had none.
#[no_mangle]
pub unsafe extern "system" fn NtCancelSynchronousIoFile(
    ThreadHandle: HANDLE,
    IoRequestToCancel: PIO_STATUS_BLOCK,
    IoStatusBlock: PIO_STATUS_BLOCK,
) -> NTSTATUS {
    let ThreadHandle = opaque_ptr(ThreadHandle);
    let IoStatusBlock = opaque_ptr(IoStatusBlock);
    let mode = KeGetPreviousMode();
    let mut Thread: PETHREAD = core::ptr::null_mut();

    if mode != 0 {
        if let Err(e) = probe_user_buffer_write(
            IoStatusBlock as *const core::ffi::c_void,
            core::mem::size_of::<_IO_STATUS_BLOCK>(),
            core::mem::align_of::<_IO_STATUS_BLOCK>(),
            mode,
        ) {
            return e;
        }
    }

    let status = ObReferenceObjectByHandle(
        ThreadHandle,
        THREAD_TERMINATE,
        PsThreadType as _,
        mode,
        &mut Thread as *mut _ as *mut PVOID,
        core::ptr::null_mut(),
    );
    if status < 0 {
        return status;
    }

    let mut req = new_request(core::ptr::null_mut(), opaque_ptr_const(IoRequestToCancel) as PVOID, true);
    arm(&mut req);
    cancel_in_thread(&req, Thread);
    let status = finish(&req, IoStatusBlock, mode);

    ObDereferenceObject(Thread as PVOID);
    status
}
