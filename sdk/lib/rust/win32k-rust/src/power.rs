//! Rust translation of `win32ss/user/ntuser/power.c` — power callouts.
//! [ntuser wave 6: power]

use win32k_sys::win32k::*;
use ntkrnl_sys::ddk::{KeIsAttachedProcess,
    ObReferenceObjectByHandle, PsGetCurrentThread, PsIsSystemThread, ExFreePoolWithTag,
    ObfDereferenceObject, KeEnterCriticalRegion, KeLeaveCriticalRegion,
    ExAcquireFastMutexUnsafe, ExReleaseFastMutexUnsafe, KeSetEvent, KeInitializeEvent};
use ntkrnl_sys::base::{NT_SUCCESS, STATUS_SUCCESS, IsListEmpty, InsertTailList,
    RemoveEntryList, InitializeListHead, STATUS_INSUFFICIENT_RESOURCES, STATUS_NOT_IMPLEMENTED};
use ntkrnl_sys::constants::{STATUS_INVALID_PARAMETER, STATUS_UNSUCCESSFUL};

// USERTAG_POWER 'opsU' (multichar, big-endian).
const USERTAG_POWER: u32 = u32::from_be_bytes(*b"opsU");
// EVENT_ALL_ACCESS / HWND_BROADCAST aren't in the win32k-sys glob.
const EVENT_ALL_ACCESS: ACCESS_MASK = 0x1F0003;
const HWND_BROADCAST: u32 = 0xFFFF;

// These three are DDK/win32k FORCEINLINEs with no exported symbol — inlined.
extern "system" {
    fn ExAllocatePoolWithTag(PoolType: POOL_TYPE, NumberOfBytes: SIZE_T, Tag: ULONG) -> PVOID;
}
// ExAllocatePoolZero (xdk/exfuncs.h): alloc + zero.
#[inline(always)]
unsafe fn ExAllocatePoolZero(PoolType: POOL_TYPE, NumberOfBytes: SIZE_T, Tag: ULONG) -> PVOID {
    let p = ExAllocatePoolWithTag(PoolType, NumberOfBytes, Tag);
    if !p.is_null() {
        core::ptr::write_bytes(p as *mut u8, 0, NumberOfBytes as usize);
    }
    p
}
// ExInitializeFastMutex (xdk/exfuncs.h): FM_LOCK_BIT=1, SynchronizationEvent=1.
#[inline(always)]
unsafe fn ex_initialize_fast_mutex(fm: PFAST_MUTEX) {
    (*fm).Count = 1; // FM_LOCK_BIT
    (*fm).Owner = core::ptr::null_mut();
    (*fm).Contention = 0;
    KeInitializeEvent(core::ptr::addr_of_mut!((*fm).Event) as _, 1, 0u8);
}
// IntIsThreadWin32Thread (power.h): Thread->Tcb.Win32Thread != NULL.
#[inline(always)]
unsafe fn int_is_thread_win32_thread(thread: PETHREAD) -> BOOL {
    if (*thread).Tcb.Win32Thread.is_null() { 0 } else { 1 }
}

// ExEventObjectType is an ntoskrnl DATA export. On i386/arm64 the cross-DLL data
// ref needs the raw-dylib `__imp_` thunk (--disable-auto-import); declared there
// as POBJECT_TYPE (`*mut _OBJECT_TYPE`, a single pointer) → pass directly. On
// amd64 the `__imp_`-linked win32k-sys glob types it as `*mut POBJECT_TYPE`
// (pointer-to-pointer, matching win32k.h's `_WIN32K_` decl) → must deref once.
// A plain amd64 extern under --disable-auto-import builds a broken `.refptr`
// (passed a garbage object type → ObReferenceObjectByHandle STATUS 0xc0000024).
#[cfg(any(target_arch = "x86", target_arch = "aarch64"))]
#[link(name = "ntoskrnl.exe", kind = "raw-dylib", modifiers = "+verbatim")]
extern "C" {
    static mut ExEventObjectType: *mut _OBJECT_TYPE;
}
#[cfg(any(target_arch = "x86", target_arch = "aarch64"))]
#[inline(always)]
unsafe fn ex_event_object_type() -> PVOID {
    ExEventObjectType as PVOID
}
#[cfg(target_arch = "x86_64")]
#[inline(always)]
unsafe fn ex_event_object_type() -> PVOID {
    *ExEventObjectType as PVOID // glob is *mut POBJECT_TYPE — deref to POBJECT_TYPE
}

// IntAcquire/ReleasePowerCalloutLock are FORCEINLINE (power.h).
#[inline]
unsafe fn int_acquire_power_callout_lock() {
    KeEnterCriticalRegion();
    ExAcquireFastMutexUnsafe(gpPowerCalloutMutexLock as _);
    // C uses KeGetCurrentThread() (PKTHREAD); KTHREAD is at offset 0 of ETHREAD,
    // so PsGetCurrentThread() as PKTHREAD is identical and avoids the arm64
    // KeGetCurrentPrcb FORCEINLINE (no exported symbol). Only feeds a retail-noop ASSERT.
    gpPowerCalloutMutexOwnerThread = PsGetCurrentThread() as PKTHREAD;
}
#[inline]
unsafe fn int_release_power_callout_lock() {
    ExReleaseFastMutexUnsafe(gpPowerCalloutMutexLock as _);
    gpPowerCalloutMutexOwnerThread = core::ptr::null_mut();
    KeLeaveCriticalRegion();
}

// ---- file-scope preamble (translated) ----
pub static mut gPowerCalloutsQueueList: LIST_ENTRY = LIST_ENTRY { Flink: core::ptr::null_mut(), Blink: core::ptr::null_mut() };
pub static mut gpPowerCalloutMutexLock: PFAST_MUTEX = core::ptr::null_mut();
pub static mut gpPowerRequestCalloutEvent: PKEVENT = core::ptr::null_mut();
pub static mut gpPowerCalloutMutexOwnerThread: PKTHREAD = core::ptr::null_mut();

unsafe fn IntHandlePowerEventWorker(pParameters: PWIN32_POWEREVENT_PARAMETERS) {
    let PwrEventType = (*pParameters).EventNumber;
    let _Code = (*pParameters).Code;
    if PwrEventType == PsW32SystemTime {
        UserSendNotifyMessage(
            HWND_BROADCAST as _,
            WM_TIMECHANGE,
            0,
            0,
        );
    }
}

unsafe fn IntValidateWin32PowerParams(pParameters: PWIN32_POWEREVENT_PARAMETERS) -> NTSTATUS {
    let PwrEventType = (*pParameters).EventNumber;
    if PwrEventType < PsW32FullWake || PwrEventType > PsW32MonitorOff {
        return STATUS_INVALID_PARAMETER;
    }

    STATUS_SUCCESS
}

unsafe fn IntGetNextPowerCallout(
    pPowerCallout: PWIN32POWERCALLOUT,
) -> PWIN32POWERCALLOUT {
    let Entry: PLIST_ENTRY;
    let pPowerCalloutEntry: PWIN32POWERCALLOUT;

    if IsListEmpty(core::ptr::addr_of_mut!(gPowerCalloutsQueueList) as _) {
        return core::ptr::null_mut();
    }

    if pPowerCallout.is_null() {
        Entry = gPowerCalloutsQueueList.Flink;
    } else {
        Entry = (*pPowerCallout).Link.Flink;
    }

    pPowerCalloutEntry = (Entry as *mut u8)
        .offset(-(core::mem::offset_of!(WIN32POWERCALLOUT, Link) as isize))
        as PWIN32POWERCALLOUT;
    RemoveEntryList(core::ptr::addr_of_mut!((*pPowerCalloutEntry).Link) as _);
    pPowerCalloutEntry
}

unsafe fn IntDeployPowerCallout() {
    let mut pWin32PwrCallout: PWIN32POWERCALLOUT;

    UserEnterExclusive();

    int_acquire_power_callout_lock();
    pWin32PwrCallout = IntGetNextPowerCallout(core::ptr::null_mut());
    while !pWin32PwrCallout.is_null() {
        if (*pWin32PwrCallout).Type == POWER_CALLOUT_EVENT {
            IntHandlePowerEventWorker(core::ptr::addr_of_mut!((*pWin32PwrCallout).Params));
        } else {
        }

        ExFreePoolWithTag(pWin32PwrCallout as _, USERTAG_POWER);

        pWin32PwrCallout = IntGetNextPowerCallout(pWin32PwrCallout);
    }

    int_release_power_callout_lock();
    UserLeave();
}

unsafe fn IntEnlistPowerCallout(pPowerCallout: PWIN32POWERCALLOUT) {
    let CurrentThread: PETHREAD;

    int_acquire_power_callout_lock();
    InsertTailList(core::ptr::addr_of_mut!(gPowerCalloutsQueueList) as _, core::ptr::addr_of_mut!((*pPowerCallout).Link) as _);
    int_release_power_callout_lock();

    CurrentThread = PsGetCurrentThread() as _;
    if PsIsSystemThread(CurrentThread as _) != 0
        || KeIsAttachedProcess() != 0
        || int_is_thread_win32_thread(CurrentThread) == 0
    {
        KeSetEvent(gpPowerRequestCalloutEvent as _, EVENT_INCREMENT as _, 0u8);
        return;
    }

    IntDeployPowerCallout();
}
#[no_mangle]
pub unsafe extern "system" fn IntInitWin32PowerManagement(
    hPowerRequestEvent: HANDLE,
) -> NTSTATUS {
    let Status: NTSTATUS;


    gpPowerCalloutMutexLock = ExAllocatePoolZero(
        NonPagedPool,
        core::mem::size_of::<FAST_MUTEX>() as SIZE_T,
        USERTAG_POWER,
    ) as PFAST_MUTEX;
    if gpPowerCalloutMutexLock.is_null() {
        return STATUS_INSUFFICIENT_RESOURCES;
    }

    ex_initialize_fast_mutex(gpPowerCalloutMutexLock);
    gpPowerCalloutMutexOwnerThread = core::ptr::null_mut();

    InitializeListHead(core::ptr::addr_of_mut!(gPowerCalloutsQueueList) as _);
    Status = ObReferenceObjectByHandle(
        hPowerRequestEvent,
        EVENT_ALL_ACCESS,
        ex_event_object_type() as _,
        0i32 as i8,
        core::ptr::addr_of_mut!(gpPowerRequestCalloutEvent) as _,
        core::ptr::null_mut(),
    );
    if !NT_SUCCESS(Status) {
        ExFreePoolWithTag(gpPowerCalloutMutexLock as PVOID, USERTAG_POWER);
        return Status;
    }

    STATUS_SUCCESS
}
#[no_mangle]
pub unsafe extern "system" fn IntWin32PowerManagementCleanup() -> NTSTATUS {
    let mut pWin32PwrCallout: PWIN32POWERCALLOUT;

    ObfDereferenceObject(gpPowerRequestCalloutEvent as _);
    gpPowerRequestCalloutEvent = core::ptr::null_mut();

    pWin32PwrCallout = IntGetNextPowerCallout(core::ptr::null_mut());
    while !pWin32PwrCallout.is_null() {
        ExFreePoolWithTag(pWin32PwrCallout as _, USERTAG_POWER);
        pWin32PwrCallout = IntGetNextPowerCallout(pWin32PwrCallout);
    }

    ExFreePoolWithTag(gpPowerCalloutMutexLock as _, USERTAG_POWER);
    gpPowerCalloutMutexLock = core::ptr::null_mut();
    STATUS_SUCCESS
}
#[no_mangle]
pub unsafe extern "system" fn IntHandlePowerEvent(
    pWin32PwrEventParams: PWIN32_POWEREVENT_PARAMETERS,
) -> NTSTATUS {
    let pWin32PwrCallout: PWIN32POWERCALLOUT;
    let Status: NTSTATUS;

    if gpepCSRSS.is_null() {
        return STATUS_UNSUCCESSFUL;
    }

    Status = IntValidateWin32PowerParams(pWin32PwrEventParams);
    if !NT_SUCCESS(Status) {
        return Status;
    }

    pWin32PwrCallout = ExAllocatePoolZero(
        NonPagedPool,
        core::mem::size_of::<WIN32POWERCALLOUT>() as SIZE_T,
        USERTAG_POWER,
    ) as PWIN32POWERCALLOUT;
    if pWin32PwrCallout.is_null() {
        return STATUS_INSUFFICIENT_RESOURCES;
    }

    (*pWin32PwrCallout).Type = POWER_CALLOUT_EVENT;
    (*pWin32PwrCallout).Params.EventNumber = (*pWin32PwrEventParams).EventNumber;
    (*pWin32PwrCallout).Params.Code = (*pWin32PwrEventParams).Code;

    IntEnlistPowerCallout(pWin32PwrCallout);
    STATUS_SUCCESS
}
#[no_mangle]
pub unsafe extern "system" fn IntHandlePowerState(
    pWin32PwrStateParams: PWIN32_POWERSTATE_PARAMETERS,
) -> NTSTATUS {
    STATUS_NOT_IMPLEMENTED
}
