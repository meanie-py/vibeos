use ntkrnl_sys::globals::*;
use ntkrnl_sys::base::*;
use ntkrnl_sys::constants::*;
use ntkrnl_sys::ke::*;
use ntkrnl_sys::ddk::{IRP, IoMarkIrpPending, KeCancelTimer, KeInitializeDpc, KeInitializeSpinLock, KeInitializeTimer, KeSetTimerEx, LARGE_INTEGER, LOGICAL, PDEVICE_OBJECT, PIRP};
use core::ffi::c_void;
use core::sync::atomic::{AtomicI32, Ordering};
// resolve_imports.py: explicit kernel/private-type imports
use crate::glue::{TRACE_LEVEL_INFORMATION};
use crate::types::{CLASS_IDLE_INTERVAL, CLASS_IDLE_INTERVAL_MIN, CLASS_STARVATION_INTERVAL, PCLASS_PRIVATE_FDO_DATA, PFUNCTIONAL_DEVICE_EXTENSION};
use ntkrnl_sys::ddk::{PCCHAR, PCSTR};
// resolve_imports.py: explicit kernel/private-type imports
use crate::glue::{IO_TYPE_IRP, USHORT_MAX};
// resolve_imports.py: explicit kernel/private-type imports
use crate::glue::{TRACE_FLAG_TIMER};
use crate::shims::*;
// resolve_imports.py: explicit kernel/private-type imports
use crate::types::{ClassGetDeviceParameter, ServiceTransferRequest};
use ntkrnl_sys::driver_ddk::{DbgPrint};

extern "C" {
    pub fn ClasspGetCurrentTime() -> LARGE_INTEGER;
    pub fn ClasspTimeDiffToMs(time: u64) -> u64;
    pub fn RtlULongLongSub(Minuend: u64, Subtrahend: u64, Result: *mut u64) -> NTSTATUS;
    pub fn StorDebugPrint(DebugPrintLevel: i32, Flag: u32, Format: *const i8, ...);
}

// ---- file-scope preamble (translated) ----
// @c2rust file-scope data (compiler-exact tables/statics)
pub const CLASSP_REG_SUBKEY_NAME: [::core::ffi::c_ushort; 9] = unsafe {
    ::core::mem::transmute::<[u8; 18], [::core::ffi::c_ushort; 9]>(*b"C\0l\0a\0s\0s\0p\0n\0p\0\0\0")
};

pub const CLASSP_REG_IDLE_INTERVAL_NAME: [::core::ffi::c_ushort; 13] = unsafe {
    ::core::mem::transmute::<[u8; 26], [::core::ffi::c_ushort; 13]>(
        *b"I\0d\0l\0e\0I\0n\0t\0e\0r\0v\0a\0l\0\0\0",
    )
};

pub const CLASSP_REG_IDLE_ACTIVE_MAX: [::core::ffi::c_ushort; 21] = unsafe {
    ::core::mem::transmute::<[u8; 42], [::core::ffi::c_ushort; 21]>(
        *b"I\0d\0l\0e\0O\0u\0t\0s\0t\0a\0n\0d\0i\0n\0g\0I\0o\0M\0a\0x\0\0\0",
    )
};

pub const CLASSP_REG_IDLE_PRIORITY_SUPPORTED: [::core::ffi::c_ushort; 22] = unsafe {
    ::core::mem::transmute::<[u8; 44], [::core::ffi::c_ushort; 22]>(
        *b"I\0d\0l\0e\0P\0r\0i\0o\0r\0i\0t\0y\0S\0u\0p\0p\0o\0r\0t\0e\0d\0\0\0",
    )
};
#[no_mangle]
pub unsafe extern "C" fn EnqueueDeferredClientIrp(Fdo: PDEVICE_OBJECT, Irp: PIRP) {
    let fdoExtension = (*Fdo).DeviceExtension as PFUNCTIONAL_DEVICE_EXTENSION;
    let fdoData = (*fdoExtension).PrivateFdoData;
    let mut oldIrql: KIRQL = 0;
    KeAcquireSpinLock((core::ptr::addr_of_mut!((*fdoData).SpinLock)).cast(), core::ptr::addr_of_mut!(oldIrql));
    InsertTailList(
        core::ptr::addr_of_mut!((*fdoData).DeferredClientIrpList),
        core::ptr::addr_of_mut!((*Irp).Tail.Overlay.__bindgen_anon_2.ListEntry),
    );
    KeReleaseSpinLock((core::ptr::addr_of_mut!((*fdoData).SpinLock)).cast(), oldIrql);
}
#[no_mangle]
pub unsafe extern "C" fn DequeueDeferredClientIrp(Fdo: PDEVICE_OBJECT) -> PIRP {
    let fdoExtension = (*Fdo).DeviceExtension as PFUNCTIONAL_DEVICE_EXTENSION;
    let fdoData = (*fdoExtension).PrivateFdoData;
    let irp: PIRP;
    if IsListEmpty(&raw mut (*fdoData).DeferredClientIrpList) {
        irp = core::ptr::null_mut();
    } else {
        let listEntry: PLIST_ENTRY;
        let mut oldIrql: KIRQL = 0;
        KeAcquireSpinLock((&raw mut (*fdoData).SpinLock).cast(), &mut oldIrql);
        if IsListEmpty(&raw mut (*fdoData).DeferredClientIrpList) {
            listEntry = core::ptr::null_mut();
        } else {
            listEntry = RemoveHeadList(&raw mut (*fdoData).DeferredClientIrpList);
        }
        KeReleaseSpinLock((&raw mut (*fdoData).SpinLock).cast(), oldIrql);
        if listEntry.is_null() {
            irp = core::ptr::null_mut();
        } else {
            let offset = core::mem::offset_of!(IRP, Tail.Overlay.__bindgen_anon_2.ListEntry);
            irp = (listEntry as *const u8).offset(-(offset as isize)) as PIRP;
            ntkrnl_sys::ASSERT!((*irp).Type == 6);
            InitializeListHead(&raw mut (*irp).Tail.Overlay.__bindgen_anon_2.ListEntry);
        }
    }
    irp
}
#[no_mangle]
pub unsafe extern "C" fn ClasspInitializeIdleTimer(FdoExtension: PFUNCTIONAL_DEVICE_EXTENSION) {
    let fdoData: PCLASS_PRIVATE_FDO_DATA = (*FdoExtension).PrivateFdoData;
    let mut idleInterval: ULONG = CLASS_IDLE_INTERVAL;
    let mut idlePrioritySupported: ULONG = 1;
    let mut activeIdleIoMax: ULONG = 1;

    ClassGetDeviceParameter(
        FdoExtension,
        CLASSP_REG_SUBKEY_NAME.as_ptr() as PWSTR,
        CLASSP_REG_IDLE_PRIORITY_SUPPORTED.as_ptr() as PWSTR,
        &raw mut idlePrioritySupported,
    );

    if idlePrioritySupported == 0 {
        TracePrint(
            TRACE_LEVEL_INFORMATION,
            TRACE_FLAG_TIMER,
            b"ClasspInitializeIdleTimer: Idle priority not supported for disk %p\n\0".as_ptr() as PCCHAR,
            FdoExtension,
        );
        (*fdoData).IdlePrioritySupported = 0u8;
        (*fdoData).IdleIoCount = 0;
        (*fdoData).ActiveIoCount = 0;
        return;
    }

    ClassGetDeviceParameter(
        FdoExtension,
        CLASSP_REG_SUBKEY_NAME.as_ptr() as PWSTR,
        CLASSP_REG_IDLE_INTERVAL_NAME.as_ptr() as PWSTR,
        &raw mut idleInterval,
    );

    if idleInterval < CLASS_IDLE_INTERVAL_MIN || idleInterval > USHORT_MAX {
        idleInterval = CLASS_IDLE_INTERVAL;
    }

    (*fdoData).IdlePrioritySupported = 1u8;
    KeInitializeSpinLock(&raw mut (*fdoData).IdleListLock);
    KeInitializeTimer(&raw mut (*fdoData).IdleTimer);
    KeInitializeDpc(
        &raw mut (*fdoData).IdleDpc,
        {
            let fp: unsafe extern "system" fn(PKDPC, PVOID, PVOID, PVOID) = ClasspIdleTimerDpc;
            Some(core::mem::transmute(fp))
        },
        FdoExtension as PVOID,
    );
    InitializeListHead(&raw mut (*fdoData).IdleIrpList);
    (*fdoData).IdleTimerStarted = (0u8) as i32;
    (*fdoData).StarvationDuration = CLASS_STARVATION_INTERVAL as USHORT;
    (*fdoData).IdleInterval = idleInterval as USHORT;
    (*fdoData).IdleIoCount = 0;
    (*fdoData).ActiveIoCount = 0;
    (*fdoData).ActiveIdleIoCount = 0;

    ClassGetDeviceParameter(
        FdoExtension,
        CLASSP_REG_SUBKEY_NAME.as_ptr() as PWSTR,
        CLASSP_REG_IDLE_ACTIVE_MAX.as_ptr() as PWSTR,
        &raw mut activeIdleIoMax,
    );

    activeIdleIoMax = if activeIdleIoMax > 1 { activeIdleIoMax } else { 1 };
    activeIdleIoMax = if activeIdleIoMax < USHORT_MAX { activeIdleIoMax } else { USHORT_MAX };
    (*fdoData).IdleActiveIoMax = activeIdleIoMax as USHORT;
}
#[no_mangle]
pub unsafe extern "C" fn ClasspStartIdleTimer(FdoData: PCLASS_PRIVATE_FDO_DATA) {
    let mut dueTime: LARGE_INTEGER = LARGE_INTEGER { QuadPart: 0 };
    let mstotimer: LONG = (-10i32 * 1000) as LONG;

    TracePrint(
        TRACE_LEVEL_INFORMATION,
        TRACE_FLAG_TIMER,
        b"ClasspStartIdleTimer: Start idle timer\n\0".as_ptr() as PCCHAR,
    );

    // Perform the interlocked compare-exchange inline
    let atomic_ref = &*(core::ptr::addr_of!((*FdoData).IdleTimerStarted) as *const AtomicI32);
    let timerStarted = atomic_ref
        .compare_exchange(0, 1, Ordering::AcqRel, Ordering::Relaxed)
        .unwrap_or(0);

    if timerStarted == 0 {
        (*FdoData).AntiStarvationStartTime = ClasspGetCurrentTime();
        dueTime.QuadPart = ((*FdoData).IdleInterval as i64 * mstotimer as i64) as LONGLONG;
        KeSetTimerEx(
            &raw mut (*FdoData).IdleTimer,
            dueTime,
            (*FdoData).IdleInterval as LONG,
            &raw mut (*FdoData).IdleDpc,
        );
    }
}
#[no_mangle]
pub unsafe extern "C" fn ClasspStopIdleTimer(FdoData: PCLASS_PRIVATE_FDO_DATA) {
    StorDebugPrint(
        4 as i32,
        TRACE_FLAG_TIMER,
        b"ClasspStopIdleTimer: Stop idle timer\n\0".as_ptr() as PCCHAR,
    );

    // InterlockedCompareExchange(&IdleTimerStarted, 0, 1) — swap 1→0, return old value
    let was_started = {
        let atomic_ref = &*(core::ptr::addr_of!((*FdoData).IdleTimerStarted) as *const AtomicI32);
        atomic_ref.compare_exchange(1, 0, Ordering::AcqRel, Ordering::Relaxed).is_ok()
    };

    if was_started {
        KeCancelTimer(core::ptr::addr_of_mut!((*FdoData).IdleTimer));
    }
}
#[no_mangle]
pub unsafe extern "C" fn ClasspGetIdleTime(
    FdoData: PCLASS_PRIVATE_FDO_DATA,
    CurrentTime: LARGE_INTEGER,
) -> ULONGLONG {
    let mut idleTime: ULONGLONG = 0;
    let status: NTSTATUS = RtlULongLongSub(
        CurrentTime.QuadPart as ULONGLONG,
        (*FdoData).LastNonIdleIoTime.QuadPart as ULONGLONG,
        &raw mut idleTime,
    );
    if NT_SUCCESS(status) {
        idleTime = ClasspTimeDiffToMs(idleTime);
    } else {
        idleTime = (*FdoData).IdleInterval as ULONGLONG;
    }
    idleTime
}
#[no_mangle]
pub unsafe extern "C" fn ClasspIdleDurationSufficient(
    FdoData: PCLASS_PRIVATE_FDO_DATA,
    CurrentTimeIn: *mut *mut LARGE_INTEGER,
) -> u8 {
    let idleInterval: u64;

    if (*FdoData).ActiveIoCount > 0 {
        if !CurrentTimeIn.is_null() {
            *CurrentTimeIn = core::ptr::null_mut();
        }
        return 0u8;
    }

    let CurrentTime: LARGE_INTEGER = ClasspGetCurrentTime();
    idleInterval = ClasspGetIdleTime(FdoData, CurrentTime);

    if !CurrentTimeIn.is_null() {
        **CurrentTimeIn = CurrentTime;
    }

    if idleInterval >= (*FdoData).IdleInterval as u64 {
        return 1u8;
    }

    0u8
}
#[no_mangle]
pub unsafe extern "system" fn ClasspIdleTimerDpc(
    Dpc: PKDPC,
    Context: PVOID,
    SystemArgument1: PVOID,
    SystemArgument2: PVOID,
) {
    let fdoExtension = Context as PFUNCTIONAL_DEVICE_EXTENSION;
    let mut fdoData: PCLASS_PRIVATE_FDO_DATA;
    let mut idleTime: ULONGLONG = 0;
    let mut status: NTSTATUS;
    let mut currentTime: LARGE_INTEGER = core::mem::zeroed();
    let mut pCurrentTime: *mut LARGE_INTEGER = core::ptr::null_mut();

    let _ = (Dpc, SystemArgument1, SystemArgument2);

    if fdoExtension.is_null() {
        NT_ASSERT!((!fdoExtension.is_null()) as u8);
        return;
    }

    fdoData = (*fdoExtension).PrivateFdoData;

    if (*fdoData).ActiveIoCount <= 0 {
        if (*fdoData).ActiveIdleIoCount >= (*fdoData).IdleActiveIoMax as i32 {
            return;
        }

        pCurrentTime = core::ptr::addr_of_mut!(currentTime);
        if ClasspIdleDurationSufficient(fdoData, &raw mut pCurrentTime) != 0 {
            NT_ASSERT!((!pCurrentTime.is_null()) as u8);
            (*fdoData).AntiStarvationStartTime = *pCurrentTime;
            ClasspServiceIdleRequest(fdoExtension, 0u8);
        }
        return;
    }

    currentTime = ClasspGetCurrentTime();

    status = RtlULongLongSub(
        currentTime.QuadPart as ULONGLONG,
        (*fdoData).AntiStarvationStartTime.QuadPart as ULONGLONG,
        &raw mut idleTime,
    );

    if NT_SUCCESS(status) {
        idleTime = ClasspTimeDiffToMs(idleTime);
    } else {
        idleTime = (*fdoData).StarvationDuration as ULONGLONG;
    }

    if idleTime >= (*fdoData).StarvationDuration as ULONGLONG {
        (*fdoData).AntiStarvationStartTime = currentTime;
        TracePrint(
            TRACE_LEVEL_INFORMATION,
            TRACE_FLAG_TIMER,
            b"ClasspIdleTimerDpc: Starvation timer. Send one idle request\n\0".as_ptr() as PCCHAR,
        );
        ClasspServiceIdleRequest(fdoExtension, 0u8);
    }
}
#[no_mangle]
pub unsafe extern "C" fn ClasspEnqueueIdleRequest(
    DeviceObject: PDEVICE_OBJECT,
    Irp: PIRP,
) -> NTSTATUS {
    let fdoExtension: PFUNCTIONAL_DEVICE_EXTENSION = (*DeviceObject).DeviceExtension as PFUNCTIONAL_DEVICE_EXTENSION;
    let fdoData: PCLASS_PRIVATE_FDO_DATA = (*fdoExtension).PrivateFdoData;
    let mut oldIrql: KIRQL = 0;
    let mut issueRequest: BOOLEAN = 1u8; // TRUE
    let mut currentTime: LARGE_INTEGER = core::mem::zeroed();
    let mut pCurrentTime: *mut LARGE_INTEGER = core::ptr::null_mut();

    TracePrint(
        4,
        TRACE_FLAG_TIMER,
        b"ClasspEnqueueIdleRequest: Queue idle request %p\n\0".as_ptr() as PCHAR,
        Irp,
    );

    IoMarkIrpPending(Irp);

    pCurrentTime = core::ptr::addr_of_mut!(currentTime);
    if ClasspIdleDurationSufficient(
        fdoData,
        core::ptr::addr_of_mut!(pCurrentTime),
    ) == 0u8
    {
        issueRequest = 0u8;
    }

    if (*fdoData).ActiveIdleIoCount >= (*fdoData).IdleActiveIoMax as i32 {
        issueRequest = 0u8;
    }

    KeAcquireSpinLock(
        (core::ptr::addr_of_mut!((*fdoData).IdleListLock)).cast(),
        &mut oldIrql,
    );

    if IsListEmpty(core::ptr::addr_of_mut!((*fdoData).IdleIrpList)) {
        if (*fdoData).IdleIoCount != 0 {
            DbgPrint(
                b"Assertion failed at %s(%d): %s\n\0".as_ptr() as PCSTR,
                b"drivers/storage/class/classpnp/clntirp.c\0"
                    .as_ptr() as PCSTR,
                611i32,
                b"fdoData->IdleIoCount == 0\0".as_ptr() as PCSTR,
            );
            __int2c();
        }
    }

    InsertTailList(
        core::ptr::addr_of_mut!((*fdoData).IdleIrpList),
        core::ptr::addr_of_mut!((*Irp).Tail.Overlay.__bindgen_anon_2.ListEntry),
    );

    (*fdoData).IdleIoCount += 1;

    if (*fdoData).IdleTimerStarted == 0 {
        ClasspStartIdleTimer(fdoData);
    }

    if (*fdoData).IdleIoCount != 1 {
        issueRequest = 0u8;
    }

    KeReleaseSpinLock(
        (core::ptr::addr_of_mut!((*fdoData).IdleListLock)).cast(),
        oldIrql,
    );

    if issueRequest != 0 {
        ClasspServiceIdleRequest(fdoExtension, 0u8);
    }

    STATUS_PENDING
}
#[no_mangle]
pub unsafe extern "C" fn ClasspDequeueIdleRequest(
    FdoExtension: PFUNCTIONAL_DEVICE_EXTENSION,
) -> PIRP {
    let fdoData = (*FdoExtension).PrivateFdoData;
    let mut listEntry: PLIST_ENTRY = core::ptr::null_mut::<LIST_ENTRY>();
    let mut irp: PIRP = core::ptr::null_mut::<IRP>();
    let mut oldIrql: KIRQL = 0;

    KeAcquireSpinLock(
        (core::ptr::addr_of_mut!((*fdoData).IdleListLock)).cast(),
        core::ptr::addr_of_mut!(oldIrql),
    );

    if (*fdoData).IdleIoCount > 0 {
        listEntry = RemoveHeadList(core::ptr::addr_of_mut!((*fdoData).IdleIrpList));
        NT_ASSERT!((listEntry != core::ptr::addr_of_mut!((*fdoData).IdleIrpList)) as u8);
        (*fdoData).IdleIoCount -= 1;
        if (*fdoData).IdleIoCount == 0 {
            ClasspStopIdleTimer(fdoData);
        }
        irp = ((listEntry as *const u8 as usize)
            .wrapping_sub(core::mem::offset_of!(IRP, Tail.Overlay.__bindgen_anon_2.ListEntry)))
            as *mut IRP;
        NT_ASSERT!(((*irp).Type == (IO_TYPE_IRP) as i16) as u8);
        InitializeListHead(core::ptr::addr_of_mut!((*irp).Tail.Overlay.__bindgen_anon_2.ListEntry));
    }

    KeReleaseSpinLock(
        (core::ptr::addr_of_mut!((*fdoData).IdleListLock)).cast(),
        oldIrql,
    );

    irp
}
#[no_mangle]
pub unsafe extern "C" fn ClasspCompleteIdleRequest(
    FdoExtension: PFUNCTIONAL_DEVICE_EXTENSION,
) {
    let fdoData: PCLASS_PRIVATE_FDO_DATA = (*FdoExtension).PrivateFdoData;
    if (*fdoData).IdleIoCount > 0
        && (*fdoData).ActiveIdleIoCount < ((*fdoData).IdleActiveIoMax) as i32
        && (*fdoData).ActiveIoCount <= 0
        && ClasspIdleDurationSufficient(fdoData, core::ptr::null_mut()) != 0
    {
        TracePrint(
            TRACE_LEVEL_INFORMATION,
            TRACE_FLAG_TIMER,
            b"ClasspCompleteIdleRequest: Service next idle reqeusts\n\0".as_ptr()
                as *const core::ffi::c_char,
        );
        ClasspServiceIdleRequest(FdoExtension, 1u8);
    }
}
#[no_mangle]
pub unsafe extern "C" fn ClasspServiceIdleRequest(
    FdoExtension: PFUNCTIONAL_DEVICE_EXTENSION,
    PostToDpc: BOOLEAN,
) {
    let irp: PIRP = ClasspDequeueIdleRequest(FdoExtension);
    if !irp.is_null() {
        ServiceTransferRequest(
            (*FdoExtension).__bindgen_anon_1.__bindgen_anon_1.as_ref().DeviceObject,
            irp,
            PostToDpc,
        );
    }
}
