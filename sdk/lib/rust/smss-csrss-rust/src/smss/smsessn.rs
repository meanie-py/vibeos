use ntdll_sys::nt::*;

// CONTAINING_RECORD is a C macro. Inline as `*mut u8` (call sites cast).
#[inline(always)]
unsafe fn CONTAINING_RECORD(
    address: PLIST_ENTRY,
    _type_: *const u8,
    field_offset: usize,
) -> *mut u8 {
    (address as *mut u8).wrapping_sub(field_offset)
}

#[repr(C)]
struct SMP_SESSION {
    Entry: LIST_ENTRY,
    SessionId: ULONG,
    Subsystem: PSMP_SUBSYSTEM,
    OtherSubsystem: PSMP_SUBSYSTEM,
}

type PSMP_SESSION = *mut SMP_SESSION;

// SMP_SUBSYSTEM / PSMP_SUBSYSTEM come from `ntdll_sys::nt::*` (bindgen of
// smss.h). The earlier local re-definition here had MuSessionId at offset 8
// (right after Entry) — WRONG: the real layout is Entry, Event, ProcessHandle,
// ImageType, PortHandle, SbApiPort, ClientId, *then* MuSessionId (offset 36 on
// i386). So CheckDup read the Event HANDLE (e.g. 0x3D0=976) as MuSessionId,
// never matched session 0, and SmpSbCreateSession returned OBJECT_NAME_NOT_FOUND
// → smss terminated → BUGCHECK 0xc000021a (task #18). smsubsys.rs/smloop.rs
// already use the correct bindgen layout; this file must too.

extern "C" {
    static mut SmpKnownSubSysLock: RTL_CRITICAL_SECTION;
    static mut SmpKnownSubSysHead: LIST_ENTRY;
    static mut SmpHeap: PVOID;
}

// Exported so cohabited C sminit.c can read/write them via extern decls.
#[no_mangle]
pub static mut SmpSessionListLock: RTL_CRITICAL_SECTION = unsafe { core::mem::zeroed() };
#[no_mangle]
pub static mut SmpSessionListHead: LIST_ENTRY = LIST_ENTRY {
    Flink: core::ptr::null_mut(),
    Blink: core::ptr::null_mut(),
};
#[no_mangle]
pub static mut SmpNextSessionId: ULONG = 0;
#[no_mangle]
pub static mut SmpNextSessionIdScanMode: BOOLEAN = 0u8;
#[no_mangle]
pub static mut SmpDbgSsLoaded: BOOLEAN = 0u8;
#[no_mangle]
pub static mut SmpSessionsObjectDirectory: HANDLE = core::ptr::null_mut();

#[no_mangle]
pub unsafe extern "system" fn SmpCheckDuplicateMuSessionId(MuSessionId: ULONG) -> BOOLEAN {
    let mut FoundDuplicate: BOOLEAN = 0u8;
    let mut NextEntry: PLIST_ENTRY;

    RtlEnterCriticalSection(core::ptr::addr_of_mut!(SmpKnownSubSysLock));

    NextEntry = SmpKnownSubSysHead.Flink;
    while NextEntry != core::ptr::addr_of_mut!(SmpKnownSubSysHead) {
        let Subsystem = CONTAINING_RECORD(
            NextEntry,
            core::ptr::null(),
            core::mem::offset_of!(SMP_SUBSYSTEM, Entry),
        ) as PSMP_SUBSYSTEM;
        if (*Subsystem).MuSessionId == MuSessionId {
            FoundDuplicate = 1u8;
            break;
        }

        NextEntry = (*NextEntry).Flink;
    }

    RtlLeaveCriticalSection(core::ptr::addr_of_mut!(SmpKnownSubSysLock));
    FoundDuplicate
}

unsafe fn SmpSessionIdToSession(SessionIdParam: ULONG) -> PSMP_SESSION {
    let mut FoundSession: PSMP_SESSION = core::ptr::null_mut();
    let mut NextEntry: PLIST_ENTRY;

    NextEntry = SmpSessionListHead.Flink;
    while NextEntry != core::ptr::addr_of_mut!(SmpSessionListHead) {
        let Session = CONTAINING_RECORD(
            NextEntry,
            core::ptr::null(),
            core::mem::offset_of!(SMP_SESSION, Entry),
        ) as PSMP_SESSION;

        if (*Session).SessionId == SessionIdParam {
            FoundSession = Session;
            break;
        }

        NextEntry = (*NextEntry).Flink;
    }

    FoundSession
}

#[no_mangle]
pub unsafe extern "system" fn SmpDeleteSession(SessionIdParam: ULONG) {
    RtlEnterCriticalSection(core::ptr::addr_of_mut!(SmpSessionListLock));
    let Session = SmpSessionIdToSession(SessionIdParam);

    if !Session.is_null() {
        RemoveEntryList(core::ptr::addr_of_mut!((*Session).Entry));
        RtlLeaveCriticalSection(core::ptr::addr_of_mut!(SmpSessionListLock));
        RtlFreeHeap(SmpHeap, 0, Session as PVOID);
    } else {
        RtlLeaveCriticalSection(core::ptr::addr_of_mut!(SmpSessionListLock));
    }
}

#[no_mangle]
pub unsafe extern "system" fn SmpAllocateSessionId(
    Subsystem: PSMP_SUBSYSTEM,
    OtherSubsystem: PSMP_SUBSYSTEM,
) -> ULONG {
    let SessionIdLocal: ULONG;

    RtlEnterCriticalSection(core::ptr::addr_of_mut!(SmpSessionListLock));
    SessionIdLocal = SmpNextSessionId;
    SmpNextSessionId = SmpNextSessionId.wrapping_add(1);

    if SmpNextSessionIdScanMode != 0 {
        // UNIMPLEMENTED_DBGBREAK
    } else {
        if SmpNextSessionId == 0 {
            SmpNextSessionIdScanMode = 1u8;
        }
    }

    let Session = RtlAllocateHeap(
        SmpHeap,
        0,
        core::mem::size_of::<SMP_SESSION>().try_into().unwrap(),
    ) as PSMP_SESSION;

    if !Session.is_null() {
        (*Session).Subsystem = Subsystem;
        (*Session).SessionId = SessionIdLocal;
        (*Session).OtherSubsystem = OtherSubsystem;
        InsertTailList(
            core::ptr::addr_of_mut!(SmpSessionListHead),
            core::ptr::addr_of_mut!((*Session).Entry),
        );
    }

    RtlLeaveCriticalSection(core::ptr::addr_of_mut!(SmpSessionListLock));
    SessionIdLocal
}

#[no_mangle]
pub unsafe extern "system" fn SmpGetProcessMuSessionId(
    ProcessHandle: HANDLE,
    SessionIdOut: *mut ULONG,
) -> NTSTATUS {
    let mut ProcessSession: ULONG = 0;

    let Status = NtQueryInformationProcess(
        ProcessHandle,
        ProcessSessionInformation,
        core::ptr::addr_of_mut!(ProcessSession) as PVOID,
        core::mem::size_of::<ULONG>() as ULONG,
        core::ptr::null_mut(),
    );

    if Status >= 0 {
        *SessionIdOut = ProcessSession;
    } else {
        *SessionIdOut = 0;
    }

    Status
}

#[no_mangle]
pub unsafe extern "system" fn SmpSetProcessMuSessionId(
    ProcessHandle: HANDLE,
    SessionIdIn: ULONG,
) -> NTSTATUS {
    let Status = NtSetInformationProcess(
        ProcessHandle,
        ProcessSessionInformation,
        core::ptr::addr_of!(SessionIdIn) as PVOID,
        core::mem::size_of::<ULONG>() as ULONG,
    );

    Status
}

// ============================================================================
// ORIGINAL C SOURCE
// ============================================================================
//
// /*
//  * PROJECT:         ReactOS Windows-Compatible Session Manager
//  * LICENSE:         BSD 2-Clause License
//  * FILE:            base/system/smss/smsessn.c
//  * PURPOSE:         Main SMSS Code
//  * PROGRAMMERS:     Alex Ionescu
//  */
//
// /* INCLUDES *******************************************************************/
//
// #include "smss.h"
//
// #define NDEBUG
// #include <debug.h>
//
// /* GLOBALS ********************************************************************/
//
// typedef struct _SMP_SESSION
// {
//     LIST_ENTRY Entry;
//     ULONG SessionId;
//     PSMP_SUBSYSTEM Subsystem;
//     PSMP_SUBSYSTEM OtherSubsystem;
// } SMP_SESSION, *PSMP_SESSION;
//
// RTL_CRITICAL_SECTION SmpSessionListLock;
// LIST_ENTRY SmpSessionListHead;
// ULONG SmpNextSessionId;
// BOOLEAN SmpNextSessionIdScanMode;
// BOOLEAN SmpDbgSsLoaded;
// HANDLE SmpSessionsObjectDirectory;
//
// /* FUNCTIONS ******************************************************************/
//
// BOOLEAN
// NTAPI
// SmpCheckDuplicateMuSessionId(IN ULONG MuSessionId)
// {
//     PSMP_SUBSYSTEM Subsystem;
//     BOOLEAN FoundDuplicate = FALSE;
//     PLIST_ENTRY NextEntry;
//
//     /* Lock the subsystem database */
//     RtlEnterCriticalSection(&SmpKnownSubSysLock);
//
//     /* Scan each entry */
//     NextEntry = SmpKnownSubSysHead.Flink;
//     while (NextEntry != &SmpKnownSubSysHead)
//     {
//         /* Check if this entry has the same session ID */
//         Subsystem = CONTAINING_RECORD(NextEntry, SMP_SUBSYSTEM, Entry);
//         if (Subsystem->MuSessionId == MuSessionId)
//         {
//             /* Break out of here! */
//             FoundDuplicate = TRUE;
//             break;
//         }
//
//         /* Keep going */
//         NextEntry = NextEntry->Flink;
//     }
//
//     /* Release the database and return the result */
//     RtlLeaveCriticalSection(&SmpKnownSubSysLock);
//     return FoundDuplicate;
// }
//
// PSMP_SESSION
// NTAPI
// SmpSessionIdToSession(IN ULONG SessionId)
// {
//     PSMP_SESSION Session, FoundSession = NULL;
//     PLIST_ENTRY NextEntry;
//
//     /* Loop the session list -- lock must already be held! */
//     NextEntry = SmpSessionListHead.Flink;
//     while (NextEntry != &SmpSessionListHead)
//     {
//         /* Check if this session's ID matches */
//         Session = CONTAINING_RECORD(NextEntry, SMP_SESSION, Entry);
//         if (Session->SessionId == SessionId)
//         {
//             /* Set this as the found session and break out */
//             FoundSession = Session;
//             break;
//         }
//
//         /* Keep going */
//         NextEntry = NextEntry->Flink;
//     }
//
//     /* Return the session that was found and exit */
//     return FoundSession;
// }
//
// VOID
// NTAPI
// SmpDeleteSession(IN ULONG SessionId)
// {
//     PSMP_SESSION Session;
//
//     /* Enter the lock and get the session structure */
//     RtlEnterCriticalSection(&SmpSessionListLock);
//     Session = SmpSessionIdToSession(SessionId);
//     if (Session)
//     {
//         /* Remove it from the list */
//         RemoveEntryList(&Session->Entry);
//         RtlLeaveCriticalSection(&SmpSessionListLock);
//
//         /* Now free the structure outside of the lock */
//         RtlFreeHeap(SmpHeap, 0, Session);
//     }
//     else
//     {
//         /* ID doesn't map to one of our structures, nothing to do... */
//         RtlLeaveCriticalSection(&SmpSessionListLock);
//     }
// }
//
// ULONG
// NTAPI
// SmpAllocateSessionId(IN PSMP_SUBSYSTEM Subsystem,
//                      IN PSMP_SUBSYSTEM OtherSubsystem)
// {
//     ULONG SessionId;
//     PSMP_SESSION Session;
//
//     /* Allocate a new ID while under the lock */
//     RtlEnterCriticalSection(&SmpSessionListLock);
//     SessionId = SmpNextSessionId++;
//
//     /* Check for overflow */
//     if (SmpNextSessionIdScanMode)
//     {
//         /* Break if it happened */
//         UNIMPLEMENTED_DBGBREAK("SMSS: SessionId's Wrapped\n");
//     }
//     else
//     {
//         /* Detect it for next time */
//         if (!SmpNextSessionId)
//             SmpNextSessionIdScanMode = TRUE;
//     }
//
//     /* Allocate a session structure */
//     Session = RtlAllocateHeap(SmpHeap, 0, sizeof(SMP_SESSION));
//     if (Session)
//     {
//         /* Write the session data and insert it into the session list */
//         Session->Subsystem = Subsystem;
//         Session->SessionId = SessionId;
//         Session->OtherSubsystem = OtherSubsystem;
//         InsertTailList(&SmpSessionListHead, &Session->Entry);
//     }
//     else
//     {
//         DPRINT1("SMSS: Unable to keep track of session ID -- no memory available\n");
//     }
//
//     /* Release the session lock */
//     RtlLeaveCriticalSection(&SmpSessionListLock);
//     return SessionId;
// }
//
// NTSTATUS
// NTAPI
// SmpGetProcessMuSessionId(IN HANDLE ProcessHandle,
//                          OUT PULONG SessionId)
// {
//     NTSTATUS Status;
//     ULONG ProcessSession;
//
//     /* Query the kernel for the session ID */
//     Status = NtQueryInformationProcess(ProcessHandle,
//                                        ProcessSessionInformation,
//                                        &ProcessSession,
//                                        sizeof(ProcessSession),
//                                        NULL);
//     if (NT_SUCCESS(Status))
//     {
//         /* Copy it back into the buffer */
//         *SessionId = ProcessSession;
//     }
//     else
//     {
//         /* Failure -- assume session zero */
//         DPRINT1("SMSS: GetProcessMuSessionId, Process=%p, Status=%x\n",
//                 ProcessHandle, Status);
//         *SessionId = 0;
//     }
//
//     /* Return result */
//     return Status;
// }
//
// NTSTATUS
// NTAPI
// SmpSetProcessMuSessionId(IN HANDLE ProcessHandle,
//                          IN ULONG SessionId)
// {
//     NTSTATUS Status;
//
//     /* Tell the kernel about the session ID */
//     Status = NtSetInformationProcess(ProcessHandle,
//                                      ProcessSessionInformation,
//                                      &SessionId,
//                                      sizeof(SessionId));
//     if (!NT_SUCCESS(Status))
//     {
//         DPRINT1("SMSS: SetProcessMuSessionId, Process=%p, Status=%x\n",
//                 ProcessHandle, Status);
//     }
//
//     /* Return */
//     return Status;
// }
//
// ============================================================================
// END ORIGINAL C SOURCE
// ============================================================================
