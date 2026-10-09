use ntkrnl_sys::globals::*;
use ntkrnl_sys::base::*;
use ntkrnl_sys::constants::*;
use ntkrnl_sys::interlocked::*;
use ntkrnl_sys::rtl::*;
use ntkrnl_sys::ddk::{EXCEPTION_RECORD, LARGE_INTEGER, LIST_ENTRY, LOGICAL, NtClose, NtCreateEvent, NtReleaseKeyedEvent, NtSetEvent, NtWaitForKeyedEvent, NtWaitForSingleObject, PRTL_CRITICAL_SECTION, PRTL_CRITICAL_SECTION_DEBUG, RTL_CRITICAL_SECTION, RTL_CRITICAL_SECTION_DEBUG, RtlAllocateHeap, RtlFreeHeap, RtlRaiseException, _EVENT_TYPE_SynchronizationEvent};
// NtCurrentTeb comes from crate::shared (inline asm), NOT ddk (which has no defined symbol).
use core::ffi::c_void;
use ntkrnl_sys::mm::HANDLE;
use ntkrnl_sys::inline::common::InterlockedCompareExchange;

// ---- substrate: constants + externs not surfaced by ntkrnl-sys ----
// RTL_CRITICAL_SECTION flag bits (winnt.h) — packed into the high byte of SpinCount.
const RTL_CRITICAL_SECTION_FLAG_NO_DEBUG_INFO: u32 = 0x01000000;
const RTL_CRITICAL_SECTION_FLAG_DYNAMIC_SPIN: u32 = 0x02000000;
const RTL_CRITICAL_SECTION_FLAG_STATIC_INIT: u32 = 0x04000000;
const RTL_CRITICAL_SECTION_FLAG_RESOURCE_TYPE: u32 = 0x08000000;
const RTL_CRITICAL_SECTION_FLAG_FORCE_DEBUG_INFO: u32 = 0x10000000;
const RTL_CRITICAL_SECTION_ALL_FLAG_BITS: u32 = 0xFF000000;
const EVENT_ALL_ACCESS: u32 = 0x001F_0003;
const STATUS_POSSIBLE_DEADLOCK: NTSTATUS = 0xC000_0194u32 as NTSTATUS;
const _WIN32_WINNT_WIN7: u32 = 0x0601;
const RTL_CRITSECT_TYPE: u32 = 0;
const STATUS_RESOURCE_NOT_OWNED: NTSTATUS = 0xC000_0264u32 as NTSTATUS;

use crate::shared::{LongToPtr, NtCurrentTeb, RtlGetProcessHeap};

// ntdll-internal shutdown state — defined in ntdll ldr/ldrinit.c, referenced here exactly
// as the C `extern BOOLEAN LdrpShutdownInProgress; extern HANDLE LdrpShutdownThreadId;`
// (resolved per-consumer at link, same as the C critical.c).
extern "C" {
    static mut LdrpShutdownInProgress: BOOLEAN;
    static mut LdrpShutdownThreadId: HANDLE;
}

// ---- file-scope preamble (translated) ----
pub const MAX_STATIC_CS_DEBUG_OBJECTS: usize = 64;

pub static mut RtlCriticalSectionLock: RTL_CRITICAL_SECTION = unsafe { core::mem::zeroed() };

pub static mut RtlCriticalSectionList: LIST_ENTRY = LIST_ENTRY {
    Flink: unsafe { core::ptr::addr_of!(RtlCriticalSectionList) as *mut LIST_ENTRY },
    Blink: unsafe { core::ptr::addr_of!(RtlCriticalSectionList) as *mut LIST_ENTRY },
};

pub static mut RtlpCritSectInitialized: BOOLEAN = 0u8;

pub static mut RtlpStaticDebugInfo: [RTL_CRITICAL_SECTION_DEBUG; MAX_STATIC_CS_DEBUG_OBJECTS] =
    unsafe { core::mem::zeroed() };

pub static mut RtlpDebugInfoFreeList: [BOOLEAN; MAX_STATIC_CS_DEBUG_OBJECTS] =
    [0u8; MAX_STATIC_CS_DEBUG_OBJECTS];

// Cross-DLL globals: ntdll ldrinit writes these from the registry CriticalSectionTimeout,
// so they must export the C symbol name.
#[no_mangle]
pub static mut RtlpTimeout: LARGE_INTEGER = unsafe { core::mem::zeroed() };

#[no_mangle]
pub static mut RtlpTimeoutDisable: BOOLEAN = 0u8;

#[no_mangle]
pub unsafe extern "system" fn RtlpCreateCriticalSectionSem(CriticalSection: PRTL_CRITICAL_SECTION) {
    let hEvent = (*CriticalSection).LockSemaphore;
    let mut hNewEvent: HANDLE;
    let Status: NTSTATUS;

    if hEvent.is_null() {
        hNewEvent = core::ptr::null_mut();
        Status = NtCreateEvent(
            &mut hNewEvent,
            EVENT_ALL_ACCESS,
            core::ptr::null_mut(),
            _EVENT_TYPE_SynchronizationEvent,
            0u8,
        );
        if !NT_SUCCESS(Status) {
            hNewEvent = INVALID_HANDLE_VALUE as HANDLE;
        }

        if !InterlockedCompareExchangePointer(
            core::ptr::addr_of_mut!((*CriticalSection).LockSemaphore) as *mut PVOID,
            hNewEvent as PVOID,
            core::ptr::null_mut(),
        )
        .is_null()
        {
            if hNewEvent != INVALID_HANDLE_VALUE as HANDLE {
                NtClose(hNewEvent);
            }
        }
    }
}

#[no_mangle]
pub unsafe extern "system" fn RtlpWaitForCriticalSection(CriticalSection: PRTL_CRITICAL_SECTION) -> NTSTATUS {
    let mut Status: NTSTATUS;
    let mut ExceptionRecord: EXCEPTION_RECORD = core::mem::zeroed();
    let mut LastChance: BOOLEAN = 0u8;

    if ((*CriticalSection).DebugInfo != core::ptr::null_mut())
        && ((*CriticalSection).DebugInfo != (-1isize as PRTL_CRITICAL_SECTION_DEBUG))
    {
        (*(*CriticalSection).DebugInfo).EntryCount += 1;
    }

    if LdrpShutdownInProgress != 0
        && LdrpShutdownThreadId == (*NtCurrentTeb()).RealClientId.UniqueThread
    {
        return STATUS_SUCCESS;
    }

    if (*CriticalSection).LockSemaphore.is_null() {
        RtlpCreateCriticalSectionSem(CriticalSection);
    }

    loop {
        if ((*CriticalSection).DebugInfo != core::ptr::null_mut())
            && ((*CriticalSection).DebugInfo != (-1isize as PRTL_CRITICAL_SECTION_DEBUG))
        {
            (*(*CriticalSection).DebugInfo).ContentionCount += 1;
        }

        let CurrentStatus: NTSTATUS;
        if (*CriticalSection).LockSemaphore == INVALID_HANDLE_VALUE as *mut _ {
            CurrentStatus = NtWaitForKeyedEvent(
                core::ptr::null_mut(),
                CriticalSection as *mut _,
                0u8,
                if RtlpTimeoutDisable != 0 {
                    core::ptr::null_mut()
                } else {
                    core::ptr::addr_of_mut!(RtlpTimeout)
                },
            );
        } else {
            CurrentStatus = NtWaitForSingleObject(
                (*CriticalSection).LockSemaphore,
                0u8,
                if RtlpTimeoutDisable != 0 {
                    core::ptr::null_mut()
                } else {
                    core::ptr::addr_of_mut!(RtlpTimeout)
                },
            );
        }

        Status = CurrentStatus;

        if Status == STATUS_TIMEOUT {
            if LastChance != 0 {
                ExceptionRecord.ExceptionCode = STATUS_POSSIBLE_DEADLOCK;
                ExceptionRecord.ExceptionFlags = 0;
                ExceptionRecord.ExceptionRecord = core::ptr::null_mut();
                ExceptionRecord.ExceptionAddress = core::mem::transmute(RtlRaiseException as *const ());
                ExceptionRecord.NumberParameters = 1;
                ExceptionRecord.ExceptionInformation[0] = CriticalSection as _;
                RtlRaiseException(core::ptr::addr_of_mut!(ExceptionRecord));
            }

            LastChance = 1u8;
        } else {
            return STATUS_SUCCESS;
        }
    }
}

#[no_mangle]
pub unsafe extern "system" fn RtlpUnWaitCriticalSection(CriticalSection: PRTL_CRITICAL_SECTION) {
    let Status: NTSTATUS;

    if (*CriticalSection).LockSemaphore.is_null() {
        RtlpCreateCriticalSectionSem(CriticalSection);
    }

    if (*CriticalSection).LockSemaphore == INVALID_HANDLE_VALUE as _ {
        Status = NtReleaseKeyedEvent(
            core::ptr::null_mut(),
            CriticalSection as _,
            0u8,
            core::ptr::addr_of_mut!(RtlpTimeout),
        );
    } else {
        Status = NtSetEvent((*CriticalSection).LockSemaphore, core::ptr::null_mut());
    }

    if !NT_SUCCESS(Status) {
        RtlRaiseStatus(Status);
    }
}
#[no_mangle]
pub unsafe extern "system" fn RtlpInitDeferredCriticalSection() {
    RtlInitializeCriticalSection(core::ptr::addr_of_mut!(RtlCriticalSectionLock));

    RtlpCritSectInitialized = 1u8;
}

#[no_mangle]
pub unsafe extern "system" fn RtlpAllocateDebugInfo() -> PRTL_CRITICAL_SECTION_DEBUG {
    let mut i: ULONG = 0;

    while i < MAX_STATIC_CS_DEBUG_OBJECTS as ULONG {
        if RtlpDebugInfoFreeList[i as usize] == 0 {
            RtlpDebugInfoFreeList[i as usize] = 1u8;
            return core::ptr::addr_of_mut!(RtlpStaticDebugInfo[i as usize]);
        }
        i += 1;
    }

    RtlAllocateHeap(
        RtlGetProcessHeap(),
        0,
        core::mem::size_of::<RTL_CRITICAL_SECTION_DEBUG>() as _,
    ) as PRTL_CRITICAL_SECTION_DEBUG
}
#[no_mangle]
pub unsafe extern "system" fn RtlpFreeDebugInfo(DebugInfo: PRTL_CRITICAL_SECTION_DEBUG) {
    let EntryId: usize;

    if (DebugInfo as *const RTL_CRITICAL_SECTION_DEBUG >= (&raw const RtlpStaticDebugInfo).cast::<RTL_CRITICAL_SECTION_DEBUG>()) &&
       (DebugInfo as *const RTL_CRITICAL_SECTION_DEBUG <= &RtlpStaticDebugInfo[(MAX_STATIC_CS_DEBUG_OBJECTS - 1) as usize] as *const RTL_CRITICAL_SECTION_DEBUG)
    {
        RtlZeroMemory(DebugInfo as *mut _, core::mem::size_of::<RTL_CRITICAL_SECTION_DEBUG>() as _);

        EntryId = (DebugInfo as usize - (&raw const RtlpStaticDebugInfo).cast::<RTL_CRITICAL_SECTION_DEBUG>() as usize) / core::mem::size_of::<RTL_CRITICAL_SECTION_DEBUG>();
        RtlpDebugInfoFreeList[EntryId] = 0u8;
    }
    else if (*DebugInfo).__bindgen_anon_1.__bindgen_anon_1.Flags == 0
    {
        let Peb = (*NtCurrentTeb()).ProcessEnvironmentBlock;
        RtlFreeHeap((*Peb).ProcessHeap, 0, DebugInfo as *mut _);
    }
    else
    {
    }
}

#[no_mangle]
pub unsafe extern "system" fn RtlDeleteCriticalSection(CriticalSection: PRTL_CRITICAL_SECTION) -> NTSTATUS {
    let mut Status: NTSTATUS = STATUS_SUCCESS as NTSTATUS;

    if !(*CriticalSection).LockSemaphore.is_null() {
        Status = NtClose((*CriticalSection).LockSemaphore);
    }

    RtlEnterCriticalSection(core::ptr::addr_of_mut!(RtlCriticalSectionLock));

    if !(*CriticalSection).DebugInfo.is_null()
        && (*CriticalSection).DebugInfo != (-1isize) as PRTL_CRITICAL_SECTION_DEBUG
    {
        RemoveEntryList(core::ptr::addr_of_mut!((*(*CriticalSection).DebugInfo).ProcessLocksList));
    }

    RtlLeaveCriticalSection(core::ptr::addr_of_mut!(RtlCriticalSectionLock));

    if !(*CriticalSection).DebugInfo.is_null()
        && (*CriticalSection).DebugInfo != (-1isize) as PRTL_CRITICAL_SECTION_DEBUG
    {
        RtlpFreeDebugInfo((*CriticalSection).DebugInfo);
    }

    RtlZeroMemory(
        CriticalSection as *mut core::ffi::c_void,
        core::mem::size_of::<RTL_CRITICAL_SECTION>(),
    );

    Status
}

#[no_mangle]
pub unsafe extern "system" fn RtlSetCriticalSectionSpinCount(
    CriticalSection: PRTL_CRITICAL_SECTION,
    SpinCount: ULONG,
) -> ULONG {
    let OldCount = (*CriticalSection).SpinCount as ULONG;

    let peb = (*NtCurrentTeb()).ProcessEnvironmentBlock;
    (*CriticalSection).SpinCount = if (*peb).NumberOfProcessors > 1 {
        SpinCount as usize as _
    } else {
        0
    };
    OldCount
}

#[no_mangle]
pub unsafe extern "system" fn RtlEnterCriticalSection(CriticalSection: PRTL_CRITICAL_SECTION) -> NTSTATUS {
    let Thread: HANDLE = (*NtCurrentTeb()).ClientId.UniqueThread as HANDLE;

    if InterlockedIncrement(core::ptr::addr_of_mut!((*CriticalSection).LockCount)) != 0 {
        if Thread == (*CriticalSection).OwningThread {
            (*CriticalSection).RecursionCount += 1;
            return STATUS_SUCCESS;
        }

        RtlpWaitForCriticalSection(CriticalSection);
    }

    (*CriticalSection).OwningThread = Thread;
    (*CriticalSection).RecursionCount = 1;
    (*NtCurrentTeb()).CountOfOwnedCriticalSections += 1;
    STATUS_SUCCESS
}

#[no_mangle]
pub unsafe extern "system" fn RtlInitializeCriticalSection(
    CriticalSection: PRTL_CRITICAL_SECTION,
) -> NTSTATUS {
    RtlInitializeCriticalSectionAndSpinCount(CriticalSection, 0)
}

#[no_mangle]
pub unsafe extern "system" fn RtlInitializeCriticalSectionEx(
    CriticalSection: PRTL_CRITICAL_SECTION,
    SpinCount: ULONG,
    mut Flags: ULONG,
) -> NTSTATUS {
    let CritcalSectionDebugData: PRTL_CRITICAL_SECTION_DEBUG;
    let mut AllowedFlags: ULONG;
    let OsVersion: ULONG;

    /* Remove lower bits from flags */
    Flags &= RTL_CRITICAL_SECTION_ALL_FLAG_BITS;

    /* These flags generally allowed */
    AllowedFlags = RTL_CRITICAL_SECTION_FLAG_NO_DEBUG_INFO
        | RTL_CRITICAL_SECTION_FLAG_DYNAMIC_SPIN
        | RTL_CRITICAL_SECTION_FLAG_STATIC_INIT;

    /* Flags for Windows 7+ (CHECKME) */
    let Peb = NtCurrentTeb().as_ref().map(|teb| teb.ProcessEnvironmentBlock).unwrap_or(core::ptr::null_mut());
    OsVersion = (((*Peb).OSMajorVersion << 8) | (*Peb).OSMinorVersion) as ULONG;
    if OsVersion >= _WIN32_WINNT_WIN7 {
        AllowedFlags |= RTL_CRITICAL_SECTION_FLAG_RESOURCE_TYPE
            | RTL_CRITICAL_SECTION_FLAG_FORCE_DEBUG_INFO;
    }

    if (Flags & !AllowedFlags) != 0 {
        return STATUS_INVALID_PARAMETER_3;
    }

    if (SpinCount & RTL_CRITICAL_SECTION_ALL_FLAG_BITS) != 0 {
        return STATUS_INVALID_PARAMETER_2;
    }

    /* First things first, set up the Object */
    (*CriticalSection).LockCount = -1;
    (*CriticalSection).RecursionCount = 0;
    (*CriticalSection).OwningThread = 0 as _;
    (*CriticalSection).SpinCount = if (*Peb).NumberOfProcessors > 1 {
        SpinCount as _
    } else {
        0
    };
    (*CriticalSection).LockSemaphore = 0 as _;

    if (Flags & RTL_CRITICAL_SECTION_FLAG_NO_DEBUG_INFO) != 0 {
        (*CriticalSection).DebugInfo = LongToPtr(-1) as PRTL_CRITICAL_SECTION_DEBUG;
    } else {
        /* Allocate the Debug Data */
        CritcalSectionDebugData = RtlpAllocateDebugInfo();

        if CritcalSectionDebugData.is_null() {
            /* This is bad! */
            return STATUS_NO_MEMORY;
        }

        /* Set it up */
        (*CritcalSectionDebugData).Type = RTL_CRITSECT_TYPE as _;
        (*CritcalSectionDebugData).ContentionCount = 0;
        (*CritcalSectionDebugData).EntryCount = 0;
        (*CritcalSectionDebugData).CriticalSection = CriticalSection;
        (*CritcalSectionDebugData).__bindgen_anon_1.__bindgen_anon_1.Flags = 0;
        (*CriticalSection).DebugInfo = CritcalSectionDebugData;

        /*
         * Add it to the List of Critical Sections owned by the process.
         */
        if (CriticalSection != core::ptr::addr_of_mut!(RtlCriticalSectionLock))
            && (RtlpCritSectInitialized != 0)
        {
            /* Protect List */
            RtlEnterCriticalSection(core::ptr::addr_of_mut!(RtlCriticalSectionLock));

            /* Add this one */
            InsertTailList(
                core::ptr::addr_of_mut!(RtlCriticalSectionList),
                core::ptr::addr_of_mut!((*CritcalSectionDebugData).ProcessLocksList),
            );

            /* Unprotect */
            RtlLeaveCriticalSection(core::ptr::addr_of_mut!(RtlCriticalSectionLock));
        } else {
            /* Add it directly */
            InsertTailList(
                core::ptr::addr_of_mut!(RtlCriticalSectionList),
                core::ptr::addr_of_mut!((*CritcalSectionDebugData).ProcessLocksList),
            );
        }
    }

    STATUS_SUCCESS
}

#[no_mangle]
pub unsafe extern "system" fn RtlInitializeCriticalSectionAndSpinCount(
    CriticalSection: PRTL_CRITICAL_SECTION,
    mut SpinCount: ULONG,
) -> NTSTATUS {
    SpinCount &= !RTL_CRITICAL_SECTION_ALL_FLAG_BITS;
    RtlInitializeCriticalSectionEx(CriticalSection, SpinCount, 0)
}

#[no_mangle]
pub unsafe extern "system" fn RtlGetCriticalSectionRecursionCount(CriticalSection: PRTL_CRITICAL_SECTION) -> LONG {
    if (*CriticalSection).OwningThread == (*NtCurrentTeb()).ClientId.UniqueThread {
        (*CriticalSection).RecursionCount
    } else {
        0
    }
}

#[no_mangle]
pub unsafe extern "system" fn RtlLeaveCriticalSection(CriticalSection: PRTL_CRITICAL_SECTION) -> NTSTATUS {
    (*CriticalSection).RecursionCount -= 1;
    if (*CriticalSection).RecursionCount != 0 {
        if (*CriticalSection).RecursionCount < 0 {
            return STATUS_UNSUCCESSFUL;
        }
        InterlockedDecrement(core::ptr::addr_of_mut!((*CriticalSection).LockCount));
    } else {
        (*CriticalSection).OwningThread = 0 as _;
        (*NtCurrentTeb()).CountOfOwnedCriticalSections -= 1;

        if InterlockedDecrement(core::ptr::addr_of_mut!((*CriticalSection).LockCount)) != -1 {
            RtlpUnWaitCriticalSection(CriticalSection);
        }
    }

    STATUS_SUCCESS
}

#[no_mangle]
pub unsafe extern "system" fn RtlTryEnterCriticalSection(CriticalSection: PRTL_CRITICAL_SECTION) -> LOGICAL {
    if InterlockedCompareExchange(core::ptr::addr_of_mut!((*CriticalSection).LockCount), 0, -1) == -1 {
        (*CriticalSection).OwningThread = (*NtCurrentTeb()).ClientId.UniqueThread;
        (*CriticalSection).RecursionCount = 1;
        (*NtCurrentTeb()).CountOfOwnedCriticalSections += 1;
        return 1;
    } else if (*CriticalSection).OwningThread == (*NtCurrentTeb()).ClientId.UniqueThread {
        InterlockedIncrement(core::ptr::addr_of_mut!((*CriticalSection).LockCount));
        (*CriticalSection).RecursionCount += 1;
        return 1;
    }

    0
}

#[no_mangle]
pub unsafe extern "system" fn RtlCheckForOrphanedCriticalSections(_ThreadHandle: HANDLE) {
}

#[no_mangle]
pub unsafe extern "system" fn RtlIsCriticalSectionLocked(CriticalSection: PRTL_CRITICAL_SECTION) -> LOGICAL {
    ((*CriticalSection).RecursionCount != 0) as LOGICAL
}

#[no_mangle]
pub unsafe extern "system" fn RtlIsCriticalSectionLockedByThread(CriticalSection: PRTL_CRITICAL_SECTION) -> LOGICAL {
    (((*CriticalSection).OwningThread == (*NtCurrentTeb()).ClientId.UniqueThread)
        && ((*CriticalSection).RecursionCount != 0)) as LOGICAL
}

#[no_mangle]
pub unsafe extern "system" fn RtlpNotOwnerCriticalSection(CriticalSection: PRTL_CRITICAL_SECTION) {
    RtlRaiseStatus(STATUS_RESOURCE_NOT_OWNED);
}
