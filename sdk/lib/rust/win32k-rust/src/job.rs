//! win32ss/user/ntuser/job.c — USER side of job objects (upstream b272427baa3).
//!
//! The kernel keeps a job's JOB_OBJECT_UILIMIT_* mask and hands every restricted job to us
//! through the job callout (ps-rust `PspInvokeW32JobCallout` -> `Win32kJobCallout`). We keep a
//! JOBINFO per such job: its processes, their restriction flags, and the USER handles granted
//! to it. Like upstream, nothing enforces the restrictions yet; this is the bookkeeping that
//! enforcement will read.
//!
//! Locking: the kernel holds the job lock when it calls out, and the callout takes the USER
//! lock. To keep that the only order, win32k never takes the job lock; gJobInfoList and
//! everything on it is protected by the USER lock alone.
//!
//! Translated 2026-10-03 (rustconv, sealed, DeepSeek; banked under tools/rustconv/.responses/
//! 20261003-123507-job.c-run-*) and integrated by hand. Two fixes to the translation:
//! - Its callout `match` named `PsW32JobCalloutSetInformation` & co. bare. No such constants
//!   are in scope (bindgen spells them `_PSW32JOBCALLOUTTYPE_*`), so the first arm was a
//!   catch-all binding that sent every callout to IntJobSetRestrictions.
//! - The `pClientInfo->dwTIFlags` write sits in a `_SEH2_TRY` in the C: the CLIENTINFO lives in
//!   the process's TEB. Here it is a fault-tolerant copy.

use crate::error::{EngSetLastError, SetLastNtError};
use crate::main::gppiList;
use crate::ntuser::{UserEnterExclusive, UserIsEntered, UserIsEnteredExclusive, UserLeave};
use crate::object::UserMarkHandleGranted;
use ntkrnl_sys::ddk::{
    ExAllocatePoolWithTag, ExFreePoolWithTag, KeStackAttachProcess, KeUnstackDetachProcess,
    ObReferenceObjectByHandle, PsGetCurrentProcessWin32Process, PsGetJobUIRestrictionsClass,
    PsGetProcessJob, RtlCreateAtomTable, RtlDestroyAtomTable,
};
use ntkrnl_sys::base::{STATUS_NO_MEMORY, STATUS_SUCCESS};
use ntkrnl_sys::constants::{STATUS_INVALID_PARAMETER, STATUS_UNSUCCESSFUL};
use ntkrnl_sys::ob::ObDereferenceObject;
use ntkrnl_sys::ASSERT;
use win32k_sys::win32k::*;

// PsJobType is an ntoskrnl.spec `@ extern` (data) export: a bare extern is unsatisfiable, so
// use the per-arch `__imp_` pattern desktop.rs uses for ExDesktopObjectType.
#[cfg(any(target_arch = "x86", target_arch = "aarch64"))]
#[link(name = "ntoskrnl.exe", kind = "raw-dylib", modifiers = "+verbatim")]
extern "C" {
    #[link_name = "PsJobType"]
    static mut PsJobType_data: POBJECT_TYPE;
}
#[cfg(target_arch = "x86_64")]
extern "C" {
    #[link_name = "__imp_PsJobType"]
    static mut PsJobType_imp: *mut POBJECT_TYPE;
}
#[cfg(any(target_arch = "x86", target_arch = "aarch64"))]
#[inline(always)]
unsafe fn ps_job_type() -> POBJECT_TYPE {
    PsJobType_data
}
#[cfg(target_arch = "x86_64")]
#[inline(always)]
unsafe fn ps_job_type() -> POBJECT_TYPE {
    *PsJobType_imp
}

/// win32ss/user/ntuser/tags.h
const USERTAG_W32JOB: u32 = u32::from_be_bytes(*b"bjsU");
const USERTAG_W32JOBEXTRA: u32 = u32::from_be_bytes(*b"xjsU");

const JOB_PROCESS_GROWTH: ULONG = 8;
const JOB_HANDLE_GROWTH: ULONG = 16;

static mut gJobInfoList: PJOBINFO = core::ptr::null_mut();

/// `ExAllocatePoolZero`: a header inline (not an ntoskrnl export), so spelled out as it is.
unsafe fn ExAllocatePoolZero(size: usize, tag: u32) -> PVOID {
    let p = ExAllocatePoolWithTag(PagedPool as _, size as _, tag) as PVOID;
    if !p.is_null() {
        core::ptr::write_bytes(p as *mut u8, 0, size);
    }
    p
}

/// PROCESSINFO's W32PROCESS base (offset 0; bindgen flattens it into padding).
#[inline(always)]
unsafe fn w32(ppi: PPROCESSINFO) -> *mut _W32PROCESS {
    ppi as *mut _W32PROCESS
}

// ---- private --------------------------------------------------------------------------------

unsafe fn IntFindJobInfo(pEJob: PEJOB) -> PJOBINFO {
    ASSERT!(UserIsEntered() != 0);

    let mut pJobInfo = gJobInfoList;
    while !pJobInfo.is_null() {
        if (*pJobInfo).pEJob == pEJob {
            return pJobInfo;
        }
        pJobInfo = (*pJobInfo).Next;
    }
    core::ptr::null_mut()
}

unsafe fn IntCreateJobInfo(pEJob: PEJOB) -> PJOBINFO {
    ASSERT!(UserIsEnteredExclusive() != 0);

    let pJobInfo = ExAllocatePoolZero(core::mem::size_of::<JOBINFO>(), USERTAG_W32JOB) as PJOBINFO;
    if pJobInfo.is_null() {
        ERR!("Failed to allocate a JOBINFO for job %p\n", pEJob);
        return core::ptr::null_mut();
    }

    let status = RtlCreateAtomTable(37, core::ptr::addr_of_mut!((*pJobInfo).pAtomTable) as _);
    if status < 0 {
        ERR!("Failed to create the atom table of job %p: 0x%lx\n", pEJob, status);
        ExFreePoolWithTag(pJobInfo as PVOID, USERTAG_W32JOB);
        return core::ptr::null_mut();
    }

    (*pJobInfo).pEJob = pEJob;
    (*pJobInfo).Next = gJobInfoList;
    gJobInfoList = pJobInfo;

    TRACE!("Created JOBINFO %p for job %p\n", pJobInfo, pEJob);
    pJobInfo
}

/// Mark a process and its threads as belonging to a restricted job, so enforcement is a flag
/// test rather than a walk back to the job. user32 sees the thread flag through CLIENTINFO.
unsafe fn IntSetProcessRestricted(ppi: PPROCESSINFO, bRestricted: bool) {
    let mut ApcState: KAPC_STATE = core::mem::zeroed();
    let mut bAttached = false;

    ASSERT!(UserIsEnteredExclusive() != 0);

    if bRestricted {
        (*w32(ppi)).W32PF_flags |= W32PF_JOBRESTRICTED as ULONG;
    } else {
        (*w32(ppi)).W32PF_flags &= !(W32PF_JOBRESTRICTED as ULONG);
    }

    // The threads and the address space are going away anyway.
    if (*w32(ppi)).W32PF_flags & W32PF_TERMINATED as ULONG != 0 {
        return;
    }

    // The CLIENTINFO of a thread lives in its own process.
    if !(*w32(ppi)).peProcess.is_null() && ppi != PsGetCurrentProcessWin32Process() as PPROCESSINFO {
        KeStackAttachProcess(core::ptr::addr_of_mut!((*(*w32(ppi)).peProcess).Pcb) as _, &mut ApcState as *mut _ as _);
        bAttached = true;
    }

    let mut pti = (*ppi).ptiList;
    while !pti.is_null() {
        if bRestricted {
            (*pti).TIF_flags |= TIF_JOBRESTRICTED as FLONG;
        } else {
            (*pti).TIF_flags &= !(TIF_JOBRESTRICTED as FLONG);
        }

        // _SEH2_TRY { pClientInfo->dwTIFlags = TIF_flags; } _SEH2_EXCEPT { /* kernel copy stands */ }
        if !(*pti).pClientInfo.is_null() {
            let flags = (*pti).TIF_flags as u32;
            let _ = ntkrnl_sys::probe::safe_copy_raw(
                core::ptr::addr_of_mut!((*(*pti).pClientInfo).dwTIFlags) as *mut core::ffi::c_void,
                &flags as *const u32 as *const core::ffi::c_void,
                core::mem::size_of::<u32>(),
            );
        }

        pti = (*pti).ptiSibling;
    }

    if bAttached {
        KeUnstackDetachProcess(&mut ApcState as *mut _ as _);
    }
}

unsafe fn IntDestroyJobInfo(pJobInfo: PJOBINFO) {
    ASSERT!(UserIsEnteredExclusive() != 0);

    // Unlink it first, so nothing can find it.
    let mut ppJobInfo: *mut PJOBINFO = core::ptr::addr_of_mut!(gJobInfoList);
    while !(*ppJobInfo).is_null() {
        if *ppJobInfo == pJobInfo {
            *ppJobInfo = (*pJobInfo).Next;
            break;
        }
        ppJobInfo = core::ptr::addr_of_mut!((**ppJobInfo).Next);
    }

    for i in 0..(*pJobInfo).ProcessCount as usize {
        let ppi = *(*pJobInfo).pProcesses.add(i);
        if !ppi.is_null() {
            IntSetProcessRestricted(ppi, false);
            (*ppi).pW32Job = core::ptr::null_mut();
        }
    }

    if !(*pJobInfo).pProcesses.is_null() {
        ExFreePoolWithTag((*pJobInfo).pProcesses as PVOID, USERTAG_W32JOBEXTRA);
    }
    if !(*pJobInfo).pGrantedHandles.is_null() {
        ExFreePoolWithTag((*pJobInfo).pGrantedHandles as PVOID, USERTAG_W32JOBEXTRA);
    }
    if !(*pJobInfo).pAtomTable.is_null() {
        RtlDestroyAtomTable((*pJobInfo).pAtomTable as _);
    }

    TRACE!("Destroyed JOBINFO %p\n", pJobInfo);
    ExFreePoolWithTag(pJobInfo as PVOID, USERTAG_W32JOB);
}

/// Grow one of a job's arrays by a fixed step (these lists are short and rarely added to).
/// NULL when out of memory; the old array is kept then.
unsafe fn IntGrowJobArray(pOldArray: PVOID, Used: ULONG, Max: *mut ULONG, Increment: ULONG, EntrySize: ULONG) -> PVOID {
    ASSERT!(UserIsEnteredExclusive() != 0);

    let NewMax = *Max + Increment;
    let pNewArray = ExAllocatePoolZero((NewMax * EntrySize) as usize, USERTAG_W32JOBEXTRA) as PVOID;
    if pNewArray.is_null() {
        return core::ptr::null_mut();
    }

    if !pOldArray.is_null() {
        core::ptr::copy_nonoverlapping(pOldArray as *const u8, pNewArray as *mut u8, (Used * EntrySize) as usize);
        ExFreePoolWithTag(pOldArray, USERTAG_W32JOBEXTRA);
    }

    *Max = NewMax;
    pNewArray
}

unsafe fn IntAddProcessToJobInfo(pJobInfo: PJOBINFO, ppi: PPROCESSINFO) -> NTSTATUS {
    ASSERT!(UserIsEnteredExclusive() != 0);

    // Already a member?
    for i in 0..(*pJobInfo).ProcessCount as usize {
        if *(*pJobInfo).pProcesses.add(i) == ppi {
            return STATUS_SUCCESS;
        }
    }

    if (*pJobInfo).ProcessCount >= (*pJobInfo).ProcessCountMax {
        let pProcesses = IntGrowJobArray(
            (*pJobInfo).pProcesses as PVOID,
            (*pJobInfo).ProcessCount,
            core::ptr::addr_of_mut!((*pJobInfo).ProcessCountMax),
            JOB_PROCESS_GROWTH,
            core::mem::size_of::<PPROCESSINFO>() as ULONG,
        ) as *mut PPROCESSINFO;
        if pProcesses.is_null() {
            return STATUS_NO_MEMORY;
        }
        (*pJobInfo).pProcesses = pProcesses;
    }

    *(*pJobInfo).pProcesses.add((*pJobInfo).ProcessCount as usize) = ppi;
    (*pJobInfo).ProcessCount += 1;
    (*ppi).pW32Job = pJobInfo;
    IntSetProcessRestricted(ppi, true);

    TRACE!("Process %p joined JOBINFO %p (restrictions 0x%lx)\n", ppi, pJobInfo, (*pJobInfo).UIRestrictions);
    STATUS_SUCCESS
}

/// PsW32JobCalloutSetInformation.
unsafe fn IntJobSetRestrictions(pEJob: PEJOB, UIRestrictions: ULONG) -> NTSTATUS {
    let mut Status = STATUS_SUCCESS;
    let mut Created = false;

    UserEnterExclusive();

    let mut pJobInfo = IntFindJobInfo(pEJob);

    'quit: {
        // No restrictions left, so no state to keep.
        if UIRestrictions == 0 {
            if !pJobInfo.is_null() {
                IntDestroyJobInfo(pJobInfo);
            }
            break 'quit;
        }

        if pJobInfo.is_null() {
            pJobInfo = IntCreateJobInfo(pEJob);
            if pJobInfo.is_null() {
                Status = STATUS_NO_MEMORY;
                break 'quit;
            }
            Created = true;

            // The job may already hold processes that connected to USER before the
            // restrictions were applied.
            let mut ppi = gppiList;
            while !ppi.is_null() {
                if !(*w32(ppi)).peProcess.is_null() && PsGetProcessJob((*w32(ppi)).peProcess as _) as PEJOB == pEJob {
                    Status = IntAddProcessToJobInfo(pJobInfo, ppi);
                    if Status < 0 {
                        break 'quit;
                    }
                }
                ppi = (*ppi).ppiNext;
            }
        }

        (*pJobInfo).UIRestrictions = UIRestrictions;
    }

    // Undo a job we did not manage to finish building.
    if Status < 0 && Created {
        IntDestroyJobInfo(pJobInfo);
    }

    UserLeave();
    Status
}

/// PsW32JobCalloutAddProcess.
unsafe fn IntJobAddProcess(pEJob: PEJOB, ppi: PPROCESSINFO) -> NTSTATUS {
    UserEnterExclusive();

    let pJobInfo = IntFindJobInfo(pEJob);
    let Status = if pJobInfo.is_null() {
        // The kernel only calls us for restricted jobs, so we should know this one already.
        ERR!("No JOBINFO for job %p\n", pEJob);
        STATUS_UNSUCCESSFUL
    } else {
        IntAddProcessToJobInfo(pJobInfo, ppi)
    };

    UserLeave();
    Status
}

/// PsW32JobCalloutTerminate.
unsafe fn IntJobTerminate(pEJob: PEJOB) -> NTSTATUS {
    UserEnterExclusive();

    let pJobInfo = IntFindJobInfo(pEJob);
    if !pJobInfo.is_null() {
        IntDestroyJobInfo(pJobInfo);
    }

    UserLeave();
    STATUS_SUCCESS
}

/// Whether the handle was on the list.
unsafe fn IntRemoveGrantedHandle(pJobInfo: PJOBINFO, hUserHandle: HANDLE) -> bool {
    ASSERT!(UserIsEnteredExclusive() != 0);

    let count = (*pJobInfo).GrantedHandleCount as usize;
    for i in 0..count {
        if *(*pJobInfo).pGrantedHandles.add(i) == hUserHandle {
            // Keep the array dense.
            *(*pJobInfo).pGrantedHandles.add(i) = *(*pJobInfo).pGrantedHandles.add(count - 1);
            *(*pJobInfo).pGrantedHandles.add(count - 1) = core::ptr::null_mut();
            (*pJobInfo).GrantedHandleCount -= 1;
            return true;
        }
    }
    false
}

unsafe fn IntIsHandleGrantedToJob(pJobInfo: PJOBINFO, hUserHandle: HANDLE) -> bool {
    ASSERT!(UserIsEntered() != 0);

    for i in 0..(*pJobInfo).GrantedHandleCount as usize {
        if *(*pJobInfo).pGrantedHandles.add(i) == hUserHandle {
            return true;
        }
    }
    false
}

unsafe fn IntGrantHandleToJob(pJobInfo: PJOBINFO, hUserHandle: HANDLE, bGrant: BOOL) -> BOOL {
    ASSERT!(UserIsEnteredExclusive() != 0);

    // Granting and revoking both need a handle that exists. A handle destroyed while granted
    // comes off the list through IntCleanupGrantedHandle, not by the caller revoking it.
    if UserMarkHandleGranted(hUserHandle) == 0 {
        EngSetLastError(ERROR_INVALID_PARAMETER);
        return 0;
    }

    if bGrant == 0 {
        if !IntRemoveGrantedHandle(pJobInfo, hUserHandle) {
            EngSetLastError(ERROR_INVALID_HANDLE);
        }
        return 1;
    }

    if IntIsHandleGrantedToJob(pJobInfo, hUserHandle) {
        return 1;
    }

    if (*pJobInfo).GrantedHandleCount >= (*pJobInfo).GrantedHandleCountMax {
        let pGrantedHandles = IntGrowJobArray(
            (*pJobInfo).pGrantedHandles as PVOID,
            (*pJobInfo).GrantedHandleCount,
            core::ptr::addr_of_mut!((*pJobInfo).GrantedHandleCountMax),
            JOB_HANDLE_GROWTH,
            core::mem::size_of::<HANDLE>() as ULONG,
        ) as *mut HANDLE;
        if pGrantedHandles.is_null() {
            EngSetLastError(ERROR_NOT_ENOUGH_MEMORY);
            return 0;
        }
        (*pJobInfo).pGrantedHandles = pGrantedHandles;
    }

    *(*pJobInfo).pGrantedHandles.add((*pJobInfo).GrantedHandleCount as usize) = hUserHandle;
    (*pJobInfo).GrantedHandleCount += 1;
    1
}

// ---- public ---------------------------------------------------------------------------------

/// The job callout registered with the kernel (PsEstablishWin32Callouts). Called with the job
/// lock held and without the USER lock.
pub unsafe extern "system" fn Win32kJobCallout(Parameters: PWIN32_JOBCALLOUT_PARAMETERS) -> NTSTATUS {
    let job = (*Parameters).Job as PEJOB;
    match (*Parameters).CalloutType {
        _PSW32JOBCALLOUTTYPE_PsW32JobCalloutSetInformation => {
            IntJobSetRestrictions(job, (*Parameters).Data as usize as ULONG)
        }
        _PSW32JOBCALLOUTTYPE_PsW32JobCalloutAddProcess => IntJobAddProcess(job, (*Parameters).Data as PPROCESSINFO),
        _PSW32JOBCALLOUTTYPE_PsW32JobCalloutTerminate => IntJobTerminate(job),
        other => {
            ERR!("Unknown job callout %d\n", other as i32);
            STATUS_INVALID_PARAMETER
        }
    }
}

/// A process connects to USER. A process is normally assigned to its job while still
/// suspended, so this is where restricted processes join. The job lock is deliberately not
/// taken (it would invert the callout's order); ppi is already on gppiList, so a concurrent
/// restriction picks it up in IntJobSetRestrictions.
pub(crate) unsafe fn IntJobConnectProcess(ppi: PPROCESSINFO) -> NTSTATUS {
    ASSERT!(UserIsEnteredExclusive() != 0);

    let pEJob = PsGetProcessJob((*w32(ppi)).peProcess as _) as PEJOB;
    if pEJob.is_null() {
        return STATUS_SUCCESS;
    }

    // Unrestricted jobs have no state in win32k.
    if PsGetJobUIRestrictionsClass(pEJob as _) == 0 {
        return STATUS_SUCCESS;
    }

    let pJobInfo = IntFindJobInfo(pEJob);
    if pJobInfo.is_null() {
        // The restrictions have not reached us yet, or are being torn down.
        return STATUS_SUCCESS;
    }

    IntAddProcessToJobInfo(pJobInfo, ppi)
}

/// A process disconnects from USER.
pub(crate) unsafe fn IntJobDisconnectProcess(ppi: PPROCESSINFO) {
    ASSERT!(UserIsEnteredExclusive() != 0);

    let pJobInfo = (*ppi).pW32Job;
    if pJobInfo.is_null() {
        return;
    }

    let count = (*pJobInfo).ProcessCount as usize;
    for i in 0..count {
        if *(*pJobInfo).pProcesses.add(i) == ppi {
            // Keep the array dense.
            *(*pJobInfo).pProcesses.add(i) = *(*pJobInfo).pProcesses.add(count - 1);
            *(*pJobInfo).pProcesses.add(count - 1) = core::ptr::null_mut();
            (*pJobInfo).ProcessCount -= 1;
            break;
        }
    }

    IntSetProcessRestricted(ppi, false);
    (*ppi).pW32Job = core::ptr::null_mut();
}

/// Withdraw a freed USER handle from every job it was granted to. Handle values are reused,
/// so a grant left on a dead handle would be inherited by the next object in the slot.
pub(crate) unsafe fn IntCleanupGrantedHandle(hUserHandle: HANDLE) {
    ASSERT!(UserIsEnteredExclusive() != 0);

    let mut pJobInfo = gJobInfoList;
    while !pJobInfo.is_null() {
        if IntRemoveGrantedHandle(pJobInfo, hUserHandle) {
            TRACE!("Withdrew handle %p from JOBINFO %p\n", hUserHandle, pJobInfo);
        }
        pJobInfo = (*pJobInfo).Next;
    }
}

#[no_mangle]
pub unsafe extern "system" fn NtUserUserHandleGrantAccess(hUserHandle: HANDLE, hJob: HANDLE, bGrant: BOOL) -> BOOL {
    let mut pEJob: PEJOB = core::ptr::null_mut();
    let mut Ret: BOOL = 0;

    if hUserHandle.is_null() {
        EngSetLastError(ERROR_INVALID_PARAMETER);
        return 0;
    }

    let Status = ObReferenceObjectByHandle(
        hJob as _,
        JOB_OBJECT_SET_ATTRIBUTES as _,
        ps_job_type() as _,
        1, // UserMode
        core::ptr::addr_of_mut!(pEJob) as *mut _ as _,
        core::ptr::null_mut(),
    );
    if Status < 0 {
        SetLastNtError(Status);
        return 0;
    }

    UserEnterExclusive();

    let pJobInfo = IntFindJobInfo(pEJob);
    if pJobInfo.is_null() {
        // A job with no UI restrictions at all keeps no granted list.
        EngSetLastError(ERROR_INVALID_PARAMETER);
    } else {
        Ret = IntGrantHandleToJob(pJobInfo, hUserHandle, bGrant);
    }

    UserLeave();

    ObDereferenceObject(pEJob as PVOID);
    Ret
}
