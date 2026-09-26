use win32k_sys::win32k::{
    PEVENT, PENG_EVENT, ENG_EVENT, BOOL, BOOLEAN, NTSTATUS, PVOID, HDEV, HANDLE,
    PLARGE_INTEGER, SIZE_T,
};
use ntkrnl_sys::ddk::{
    ExAllocatePoolWithTag, ExFreePoolWithTag, KeInitializeEvent, KeClearEvent,
    KeSetEvent, KeReadStateEvent, KePulseEvent, KeWaitForSingleObject,
    ObReferenceObjectByHandle, _POOL_TYPE_NonPagedPool, _EVENT_TYPE_SynchronizationEvent,
    _KWAIT_REASON_Executive, _MODE_UserMode, _OBJECT_TYPE, PRKEVENT, KPROCESSOR_MODE,
    KWAIT_REASON, ACCESS_MASK, KPRIORITY,
};
use ntkrnl_sys::ob::ObDereferenceObject;
use ntkrnl_sys::base::NT_SUCCESS;
use core::ffi::c_void;

// ENG_EVENT.fFlags bit
const ENG_EVENT_USERMAPPED: u32 = 1;

// GDITAG_ENG_EVENT = 'msfD' multi-char literal (big-endian per C; 'D','f','s','m')
// In Rust: u32::from_le_bytes(b"Dfsm") to match the byte layout.
const GDITAG_ENG_EVENT: u32 = u32::from_le_bytes(*b"Dfsm");

// EVENT_ALL_ACCESS = STANDARD_RIGHTS_REQUIRED | SYNCHRONIZE | 0x3 (NT5.x value)
const EVENT_ALL_ACCESS: ACCESS_MASK = 0x1F0003;

// IO_NO_INCREMENT / EVENT_INCREMENT
const IO_NO_INCREMENT: KPRIORITY = 0;
const EVENT_INCREMENT: KPRIORITY = 1;

// ExEventObjectType is in libntoskrnl_implib.def as DATA. Per-arch ABI:
// - i386 PE needs the `__imp_` thunk that `raw-dylib +verbatim` generates
//   (cross-DLL data refs with --disable-auto-import). Plain extern fails link.
// - amd64 auto-resolves through the regular import library AND bugchecks
//   0x6B at PsLocateSystemDll when raw-dylib is added (dup-descriptor +
//   cgu=256 partitioning interaction).
// - arm64 USED to auto-resolve but the wave-2-after-device.rs cgu shift
//   moved the reference into a partition that doesn't (link error on
//   `ExEventObjectType`). Gate raw-dylib in for arm64 too — same way as i386.
//   (Boot test still needed; arm64 doesn't have the amd64 BUGCHECK exposure.)
#[cfg(any(target_arch = "x86", target_arch = "aarch64"))]
#[link(name = "ntoskrnl.exe", kind = "raw-dylib", modifiers = "+verbatim")]
extern "C" {
    static mut ExEventObjectType: *mut _OBJECT_TYPE;
}

#[cfg(target_arch = "x86_64")]
extern "C" {
    static mut ExEventObjectType: *mut _OBJECT_TYPE;
}

// ---- file-scope preamble (translated) ----
// No file-scope declarations to translate from the preamble
#[no_mangle]
pub unsafe extern "system" fn EngCreateEvent(
    ppEvent: *mut PEVENT
) -> BOOL {
    let mut Result: BOOLEAN = 1u8;
    let mut EngEvent: PENG_EVENT;

    EngEvent = ExAllocatePoolWithTag(
        _POOL_TYPE_NonPagedPool,
        (core::mem::size_of::<ENG_EVENT>() + core::mem::size_of::<ntkrnl_sys::ddk::_KEVENT>()) as SIZE_T,
        GDITAG_ENG_EVENT,
    ) as PENG_EVENT;

    if !EngEvent.is_null() {
        (*EngEvent).fFlags = 0;
        (*EngEvent).pKEvent = EngEvent.offset(1) as PVOID;

        KeInitializeEvent(
            (*EngEvent).pKEvent as PRKEVENT,
            _EVENT_TYPE_SynchronizationEvent,
            0u8,
        );

        *ppEvent = EngEvent as PEVENT;
    } else {
        Result = 0u8;
    }

    Result as BOOL
}
#[no_mangle]
pub unsafe extern "system" fn EngDeleteEvent(
    Event: PEVENT,
) -> BOOL {
    if ((*Event).fFlags & ENG_EVENT_USERMAPPED) != 0 {
        return 0;
    }

    ExFreePoolWithTag(Event as *mut c_void, GDITAG_ENG_EVENT);

    return 1;
}
#[no_mangle]
pub unsafe extern "system" fn EngClearEvent(Event: PEVENT) {
    KeClearEvent((*Event).pKEvent as PRKEVENT);
}
#[no_mangle]
pub unsafe extern "system" fn EngSetEvent(
    Event: PEVENT,
) -> i32 {
    KeSetEvent((*Event).pKEvent as PRKEVENT, IO_NO_INCREMENT, 0u8)
}
#[no_mangle]
pub unsafe extern "system" fn EngReadStateEvent(
    Event: PEVENT,
) -> i32 {
    KeReadStateEvent((*Event).pKEvent as PRKEVENT)
}
#[no_mangle]
pub unsafe extern "system" fn EngMapEvent(
    _hDev: HDEV,
    hUserObject: HANDLE,
    Reserved1: PVOID,
    _Reserved2: PVOID,
    _Reserved3: PVOID,
) -> PEVENT {
    let mut EngEvent: PENG_EVENT;
    let mut pvEvent: PVOID = core::ptr::null_mut();
    let Status: NTSTATUS;

    EngEvent = ExAllocatePoolWithTag(
        _POOL_TYPE_NonPagedPool,
        core::mem::size_of::<ENG_EVENT>() as SIZE_T,
        GDITAG_ENG_EVENT,
    ) as PENG_EVENT;
    if EngEvent.is_null() {
        return core::ptr::null_mut();
    }

    (*EngEvent).fFlags = 0;
    (*EngEvent).pKEvent = core::ptr::null_mut();

    Status = ObReferenceObjectByHandle(
        hUserObject,
        EVENT_ALL_ACCESS,
        ExEventObjectType,
        _MODE_UserMode as KPROCESSOR_MODE,
        &mut pvEvent,
        core::ptr::null_mut(),
    );
    if NT_SUCCESS(Status) {
        KePulseEvent(pvEvent as PRKEVENT, EVENT_INCREMENT, 0u8);
        (*EngEvent).pKEvent = pvEvent;
        (*EngEvent).fFlags |= ENG_EVENT_USERMAPPED;
    } else {
        ExFreePoolWithTag(EngEvent as PVOID, GDITAG_ENG_EVENT);
        EngEvent = core::ptr::null_mut();
    }

    if !Reserved1.is_null() {
        *(Reserved1 as *mut PVOID) = EngEvent as PVOID;
    }
    EngEvent as PEVENT
}
#[no_mangle]
pub unsafe extern "system" fn EngUnmapEvent(
    Event: PEVENT,
) -> BOOL {
    if ((*Event).fFlags & ENG_EVENT_USERMAPPED) == 0 {
        return 0;
    }

    ObDereferenceObject((*Event).pKEvent as *mut c_void);

    ExFreePoolWithTag(Event as *mut c_void, GDITAG_ENG_EVENT);
    return 1;
}
#[no_mangle]
pub unsafe extern "system" fn EngWaitForSingleObject(
    Event: PEVENT,
    TimeOut: PLARGE_INTEGER,
) -> BOOL {
    let Status: NTSTATUS;

    if Event.is_null() {
        return 0;
    }

    if ((*Event).fFlags & ENG_EVENT_USERMAPPED) != 0 {
        return 0;
    }

    Status = KeWaitForSingleObject(
        (*Event).pKEvent,
        _KWAIT_REASON_Executive as KWAIT_REASON,
        0i32 as KPROCESSOR_MODE, // KernelMode = 0
        0u8,
        TimeOut as *mut ntkrnl_sys::ddk::_LARGE_INTEGER,
    );

    if NT_SUCCESS(Status) {
        1
    } else {
        0
    }
}


// ============================================================================
// ORIGINAL C SOURCE
// ============================================================================
//
// /*
//  * PROJECT:         ReactOS Win32K
//  * LICENSE:         BSD - See COPYING.ARM in the top level directory
//  * FILE:            win32ss/gdi/eng/engevent.c
//  * PURPOSE:         Event Support Routines
//  * PROGRAMMERS:     Aleksey Bragin <aleksey@reactos.org>
//  *                  ReactOS Portable Systems Group
//  */
//
// (full C source preserved in git history for engevent.c)
//
// ============================================================================
// END ORIGINAL C SOURCE
// ============================================================================
