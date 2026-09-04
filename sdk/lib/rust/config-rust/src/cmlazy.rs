use ntkrnl_sys::constants::*;
use ntkrnl_sys::base::*;
use ntkrnl_sys::ex::*;
use ntkrnl_sys::interlocked::*;
use ntkrnl_sys::ke::*;
use ntkrnl_sys::mm::*;
use ntkrnl_sys::ddk::{CMHIVE, CmpInitHiveFromFile, CmpInitializeHiveList, CmpLockRegistry, CmpLockRegistryExclusive, CmpUnlockRegistry, ExAcquirePushLockShared, ExQueueWorkItem, ExReleasePushLock, KTIMER, KeCancelTimer, KeInitializeDpc, KeInitializeTimer, KeSetTimer, LARGE_INTEGER, OBJECT_NAME_INFORMATION, PCMHIVE, POBJECT_ATTRIBUTES, POBJECT_NAME_INFORMATION, PSECURITY_CLIENT_CONTEXT, PsRevertToSelf, RtlAppendUnicodeStringToString, RtlCopyUnicodeString, RtlInitEmptyUnicodeString, SeImpersonateClientEx, WORK_QUEUE_ITEM, ZwQueryObject, _EX_PUSH_LOCK, _KDPC, _OBJECT_INFORMATION_CLASS_ObjectNameInformation, _UNICODE_STRING, _WORK_QUEUE_TYPE_DelayedWorkQueue};

use ntkrnl_sys::opaque_ptr;
// Constants not in ntkrnl_sys::constants
const TAG_CM: u32 = u32::from_le_bytes(*b"CM  ");
use crate::hiveflags::{HIVE_NOLAZYFLUSH, HIVE_VOLATILE};
const STATUS_NO_SUCH_USER: NTSTATUS = 0xC0000064_u32 as i32;
const STATUS_WRONG_PASSWORD: NTSTATUS = 0xC000006A_u32 as i32;
const STATUS_ACCOUNT_RESTRICTION: NTSTATUS = 0xC000006E_u32 as i32;
const STATUS_ACCOUNT_DISABLED: NTSTATUS = 0xC0000072_u32 as i32;
const STATUS_ACCOUNT_EXPIRED: NTSTATUS = 0xC0000193_u32 as i32;

// Int32x32To64 is a C macro: ((__int64)(long)(a))*((__int64)(long)(b))
#[inline(always)]
fn Int32x32To64(a: i32, b: i32) -> i64 {
    (a as i64).wrapping_mul(b as i64)
}

// HvSyncHive returns BOOLEAN in C; treat as NTSTATUS-compatible i32 for the
// existing NT_SUCCESS check (mirrors original C: implicit BOOLEAN→NTSTATUS).
extern "system" {
    fn HvSyncHive(Hive: ntkrnl_sys::ddk::PHHIVE) -> u8;
}
extern "C" {
    fn ExInitializeWorkItem(
        Item: *mut WORK_QUEUE_ITEM,
        Routine: Option<unsafe extern "system" fn(PVOID)>,
        Context: PVOID,
    );
}

// Globals defined in cmlazy.c (cmlazy.c is excluded from build; this Rust file
// is the sole definer).
#[no_mangle] pub static mut CmpLazyFlushTimer: KTIMER = unsafe { core::mem::zeroed() };
#[no_mangle] pub static mut CmpLazyFlushDpc: _KDPC = unsafe { core::mem::zeroed() };
#[no_mangle] pub static mut CmpLazyWorkItem: WORK_QUEUE_ITEM = unsafe { core::mem::zeroed() };
#[no_mangle] pub static mut CmpEnableLazyFlushTimer: KTIMER = unsafe { core::mem::zeroed() };
#[no_mangle] pub static mut CmpEnableLazyFlushDpc: _KDPC = unsafe { core::mem::zeroed() };
#[no_mangle] pub static mut CmpLazyFlushPending: BOOLEAN = 0;
#[no_mangle] pub static mut CmpForceForceFlush: BOOLEAN = 0;
#[no_mangle] pub static mut CmpHoldLazyFlush: BOOLEAN = 1;
#[no_mangle] pub static mut CmpLazyFlushIntervalInSeconds: ULONG = 5;
#[no_mangle] pub static mut CmpLazyFlushHiveCount: ULONG = 7;

// Registry-controlled values owned by config/cmlazy. Concatenated into
// CmControlVector by config::cmdata. See docs/engineering/cmdata-chopping-plan.md.
pub static CMLAZY_CONTROL_VECTOR: ntkrnl_sys::SubVector<2> =
    ntkrnl_sys::SubVector({ [
        ntkrnl_sys::cv!("Session Manager\\Configuration Manager", "RegistryLazyFlushInterval",
            core::ptr::addr_of_mut!(CmpLazyFlushIntervalInSeconds)),
        ntkrnl_sys::cv!("Session Manager\\Configuration Manager", "RegistryLazyFlushHiveCount",
            core::ptr::addr_of_mut!(CmpLazyFlushHiveCount)),
    ] });
#[no_mangle] pub static mut CmpLazyFlushCount: ULONG = 1;
#[no_mangle] pub static mut CmpFlushStarveWriters: LONG = 0;

// Globals defined elsewhere (cminit.rs / cmsysini).
extern "C" {
    static mut CmpHiveListHead: ntkrnl_sys::base::LIST_ENTRY;
    static mut CmpHiveListHeadLock: _EX_PUSH_LOCK;
    static mut CmpNoWrite: BOOLEAN;
    static mut CmpWasSetupBoot: BOOLEAN;
    static mut CmpMiniNTBoot: BOOLEAN;
    static mut CmpShareSystemHives: BOOLEAN;
}

#[no_mangle]
pub unsafe extern "system" fn CmpDoFlushNextHive(
    ForceFlush: BOOLEAN,
    Error: PBOOLEAN,
    DirtyCount: *mut ULONG,
) -> BOOLEAN {
    let Error = opaque_ptr(Error);
    let DirtyCount = opaque_ptr(DirtyCount);
    let mut Status: NTSTATUS;
    let mut NextEntry: PLIST_ENTRY;
    let mut CmHive: PCMHIVE;
    let mut Result: BOOLEAN;
    let mut HiveCount: ULONG = CmpLazyFlushHiveCount;

    *Error = 0u8;
    *DirtyCount = 0;

    if CmpNoWrite != 0 {
        return 1u8;
    }

    if HiveCount == 0 {
        HiveCount = 1;
    }

    ExAcquirePushLockShared(core::ptr::addr_of_mut!(CmpHiveListHeadLock));
    NextEntry = CmpHiveListHead.Flink;
    while NextEntry != core::ptr::addr_of_mut!(CmpHiveListHead) && HiveCount != 0 {
        CmHive = (NextEntry as *mut u8).offset(-(core::mem::offset_of!(CMHIVE, HiveList) as isize)) as PCMHIVE;
        if ((*CmHive).Hive.HiveFlags & HIVE_NOLAZYFLUSH) == 0 &&
            (*CmHive).FlushCount != CmpLazyFlushCount
        {
            Result = 1u8;
            HiveCount -= 1;

            if ((*CmHive).Hive.DirtyCount == 0 && ForceFlush == 0) ||
                ((*CmHive).Hive.HiveFlags & HIVE_VOLATILE) != 0
            {
                (*CmHive).FlushCount = CmpLazyFlushCount;
            } else {
                // HvSyncHive returns BOOLEAN (TRUE on success), not NTSTATUS
                if HvSyncHive(core::ptr::addr_of_mut!((*CmHive).Hive)) == 0 {
                    // Upstream prints the NTSTATUS here; our HvSyncHive answers
                    // BOOLEAN (see the line above), so there is none to report.
                    crate::dbg_print!(
                        b"Failed to flush %wZ on handle %p\n\0",
                        core::ptr::addr_of!((*CmHive).FileFullPath),
                        (*CmHive).FileHandles[0]
                    );
                    *Error = 1u8;
                    Result = 0u8;
                    break;
                }
                (*CmHive).FlushCount = CmpLazyFlushCount;
            }
        } else if (*CmHive).Hive.DirtyCount != 0 &&
                  ((*CmHive).Hive.HiveFlags & HIVE_VOLATILE) == 0 &&
                  ((*CmHive).Hive.HiveFlags & HIVE_NOLAZYFLUSH) == 0
        {
            *DirtyCount += (*CmHive).Hive.DirtyCount;
        }

        NextEntry = (*NextEntry).Flink;
    }

    if NextEntry == core::ptr::addr_of_mut!(CmpHiveListHead) {
        Result = 0u8;
    } else {
        Result = 1u8;
    }

    ExReleasePushLock(core::ptr::addr_of_mut!(CmpHiveListHeadLock));
    Result
}

#[no_mangle]
pub unsafe extern "system" fn CmpEnableLazyFlushDpcRoutine(
    Dpc: PKDPC,
    DeferredContext: PVOID,
    SystemArgument1: PVOID,
    SystemArgument2: PVOID,
) {
    let Dpc = opaque_ptr(Dpc);
    let DeferredContext = opaque_ptr(DeferredContext);
    let SystemArgument1 = opaque_ptr(SystemArgument1);
    let SystemArgument2 = opaque_ptr(SystemArgument2);
    CmpHoldLazyFlush = 0u8;
}

#[no_mangle]
pub unsafe extern "system" fn CmpLazyFlushDpcRoutine(
    Dpc: PKDPC,
    DeferredContext: PVOID,
    SystemArgument1: PVOID,
    SystemArgument2: PVOID,
) {
    let Dpc = opaque_ptr(Dpc);
    let DeferredContext = opaque_ptr(DeferredContext);
    let SystemArgument1 = opaque_ptr(SystemArgument1);
    let SystemArgument2 = opaque_ptr(SystemArgument2);
    if CmpLazyFlushPending == 0 && CmpHoldLazyFlush == 0 {
        CmpLazyFlushPending = 1u8;
        ExQueueWorkItem(
            core::ptr::addr_of_mut!(CmpLazyWorkItem),
            _WORK_QUEUE_TYPE_DelayedWorkQueue,
        );
    }
}

#[no_mangle]
pub unsafe extern "system" fn CmpLazyFlush() {
    let mut DueTime: LARGE_INTEGER = core::mem::zeroed();

    if CmpNoWrite == 0 && CmpHoldLazyFlush == 0 {
        DueTime.QuadPart = Int32x32To64(CmpLazyFlushIntervalInSeconds as i32, -10 * 1000 * 1000);
        KeSetTimer(
            core::ptr::addr_of_mut!(CmpLazyFlushTimer),
            DueTime,
            core::ptr::addr_of_mut!(CmpLazyFlushDpc),
        );
    }
}

#[no_mangle]
pub unsafe extern "system" fn CmpLazyFlushWorker(Parameter: PVOID) {
    let Parameter = opaque_ptr(Parameter);
    let mut ForceFlush: BOOLEAN = 0u8;
    let mut Result: BOOLEAN = 0u8;
    let mut MoreWork: BOOLEAN = 0u8;
    let mut DirtyCount: ULONG = 0;

    if CmpHoldLazyFlush != 0 {
        crate::dbg_print!(b"Lazy flush held. Bye bye.\n\0");
        CmpLazyFlushPending = 0u8;
        return;
    }

    ForceFlush = CmpForceForceFlush;
    if ForceFlush != 0 {
        CmpLockRegistryExclusive();
    } else {
        InterlockedIncrement(core::ptr::addr_of_mut!(CmpFlushStarveWriters) as *mut LONG);
        CmpLockRegistry();
    }

    MoreWork = CmpDoFlushNextHive(ForceFlush, core::ptr::addr_of_mut!(Result), core::ptr::addr_of_mut!(DirtyCount));
    if MoreWork == 0 {
        InterlockedIncrement(core::ptr::addr_of_mut!(CmpLazyFlushCount) as *mut LONG);
    }

    if ForceFlush == 0 {
        InterlockedDecrement(core::ptr::addr_of_mut!(CmpFlushStarveWriters) as *mut LONG);
    }

    CmpLazyFlushPending = 0u8;
    CmpUnlockRegistry();

    if MoreWork != 0 {
        CmpLazyFlush();
    }
}

#[no_mangle]
pub unsafe extern "system" fn CmpCmdInit(SetupBoot: BOOLEAN) {
    let mut DueTime: LARGE_INTEGER = core::mem::zeroed();

    KeInitializeDpc(
        &raw mut CmpLazyFlushDpc,
        core::mem::transmute(Some(CmpLazyFlushDpcRoutine as unsafe extern "system" fn(_, _, _, _))),
        core::ptr::null_mut(),
    );

    KeInitializeTimer(&raw mut CmpLazyFlushTimer);

    ExInitializeWorkItem(&raw mut CmpLazyWorkItem, Some(CmpLazyFlushWorker), core::ptr::null_mut());

    KeInitializeDpc(
        &raw mut CmpEnableLazyFlushDpc,
        core::mem::transmute(Some(CmpEnableLazyFlushDpcRoutine as unsafe extern "system" fn(_, _, _, _))),
        core::ptr::null_mut(),
    );
    KeInitializeTimer(&raw mut CmpEnableLazyFlushTimer);

    DueTime.QuadPart = Int32x32To64(600, -10 * 1000 * 1000);
    KeSetTimer(&raw mut CmpEnableLazyFlushTimer, DueTime, &raw mut CmpEnableLazyFlushDpc);

    CmpNoWrite = CmpMiniNTBoot;
    CmpWasSetupBoot = SetupBoot;

    CmpHoldLazyFlush = 0u8;

    if SetupBoot == 0 {
        CmpInitializeHiveList();
    }

    if CmpMiniNTBoot != 0 {
        CmpShareSystemHives = 0u8;
    }
}

#[no_mangle]
pub unsafe extern "system" fn CmpCmdHiveOpen(
    FileAttributes: POBJECT_ATTRIBUTES,
    ImpersonationContext: PSECURITY_CLIENT_CONTEXT,
    Allocate: PBOOLEAN,
    NewHive: *mut PCMHIVE,
    CheckFlags: u32,
) -> i32 {
    let FileAttributes = opaque_ptr(FileAttributes);
    let ImpersonationContext = opaque_ptr(ImpersonationContext);
    let Allocate = opaque_ptr(Allocate);
    let NewHive = opaque_ptr(NewHive);
    let mut Status: i32;
    let mut FileName: _UNICODE_STRING = core::mem::zeroed();
    let mut FilePath: *mut u16;
    let mut Length: u32 = 0;
    let mut DummyNameInfo: OBJECT_NAME_INFORMATION = core::mem::zeroed();
    let mut FileNameInfo: POBJECT_NAME_INFORMATION;

    if (*FileAttributes).RootDirectory != core::ptr::null_mut() {
        if !(*FileAttributes).ObjectName.is_null() &&
           !(*(*FileAttributes).ObjectName).Buffer.is_null() &&
           (*(*FileAttributes).ObjectName).Length >= core::mem::size_of::<u16>() as u16 &&
           *(*(*FileAttributes).ObjectName).Buffer == 0x5C {
            return STATUS_OBJECT_PATH_SYNTAX_BAD;
        }

        Status = ZwQueryObject(
            (*FileAttributes).RootDirectory,
            _OBJECT_INFORMATION_CLASS_ObjectNameInformation,
            &mut DummyNameInfo as *mut _ as *mut core::ffi::c_void,
            core::mem::size_of::<OBJECT_NAME_INFORMATION>() as u32,
            &mut Length,
        );
        if Status != STATUS_BUFFER_OVERFLOW {
            crate::dbg_print!(
                b"CmpCmdHiveOpen(): Root directory handle object name size query failed, Status = 0x%08lx\n\0",
                Status
            );
            return Status;
        }

        FileNameInfo = ExAllocatePoolWithTag(
            PagedPool,
            (Length + core::mem::size_of::<u16>() as u32) as usize,
            TAG_CM,
        ) as POBJECT_NAME_INFORMATION;
        if FileNameInfo.is_null() {
            crate::dbg_print!(b"CmpCmdHiveOpen(): Unable to allocate memory\n\0");
            return STATUS_INSUFFICIENT_RESOURCES;
        }

        Status = ZwQueryObject(
            (*FileAttributes).RootDirectory,
            _OBJECT_INFORMATION_CLASS_ObjectNameInformation,
            FileNameInfo as *mut core::ffi::c_void,
            Length,
            &mut Length,
        );
        if !NT_SUCCESS(Status) {
            crate::dbg_print!(
                b"CmpCmdHiveOpen(): Root directory handle object name query failed, Status = 0x%08lx\n\0",
                Status
            );
            ExFreePoolWithTag(FileNameInfo as *mut core::ffi::c_void, TAG_CM);
            return Status;
        }

        Length -= core::mem::size_of::<OBJECT_NAME_INFORMATION>() as u32;
        FilePath = (*FileNameInfo).Name.Buffer;
        *FilePath.add((Length / core::mem::size_of::<u16>() as u32) as usize) = 0;
        Length += core::mem::size_of::<u16>() as u32;

        Length = Length + core::mem::size_of::<u16>() as u32 + (*(*FileAttributes).ObjectName).Length as u32;
        if Length > 0xFFFF {
            ExFreePoolWithTag(FileNameInfo as *mut core::ffi::c_void, TAG_CM);
            return STATUS_OBJECT_PATH_INVALID;
        }

        RtlInitEmptyUnicodeString(&mut FileName, core::ptr::null_mut(), 0);
        FileName.Buffer = ExAllocatePoolWithTag(
            PagedPool,
            Length as usize,
            TAG_CM,
        ) as *mut u16;
        if FileName.Buffer.is_null() {
            crate::dbg_print!(b"CmpCmdHiveOpen(): Unable to allocate memory\n\0");
            ExFreePoolWithTag(FileNameInfo as *mut core::ffi::c_void, TAG_CM);
            return STATUS_INSUFFICIENT_RESOURCES;
        }
        FileName.MaximumLength = Length as u16;
        RtlCopyUnicodeString(&mut FileName, &(*FileNameInfo).Name);
        ExFreePoolWithTag(FileNameInfo as *mut core::ffi::c_void, TAG_CM);

        if FileName.Length > 0 && *FileName.Buffer.add((FileName.Length / core::mem::size_of::<u16>() as u16 - 1) as usize) != 0x5C {
            *FileName.Buffer.add((FileName.Length / core::mem::size_of::<u16>() as u16) as usize) = 0x5C;
            FileName.Length += core::mem::size_of::<u16>() as u16;
            *FileName.Buffer.add((FileName.Length / core::mem::size_of::<u16>() as u16) as usize) = 0;
        }

        Status = RtlAppendUnicodeStringToString(&mut FileName, (*FileAttributes).ObjectName);
        if !NT_SUCCESS(Status) {
            crate::dbg_print!(
                b"CmpCmdHiveOpen(): RtlAppendUnicodeStringToString() failed, Status = 0x%08lx\n\0",
                Status
            );
            ExFreePoolWithTag(FileName.Buffer as *mut core::ffi::c_void, TAG_CM);
            return Status;
        }
    } else {
        FileName = *(*FileAttributes).ObjectName;
    }

    Status = CmpInitHiveFromFile(
        &FileName,
        0,
        NewHive,
        Allocate,
        CheckFlags,
    );
    if ((Status == STATUS_ACCESS_DENIED) ||
        (Status == STATUS_NO_SUCH_USER) ||
        (Status == STATUS_WRONG_PASSWORD) ||
        (Status == STATUS_ACCOUNT_EXPIRED) ||
        (Status == STATUS_ACCOUNT_DISABLED) ||
        (Status == STATUS_ACCOUNT_RESTRICTION)) &&
       !ImpersonationContext.is_null() {
        Status = SeImpersonateClientEx(ImpersonationContext, core::ptr::null_mut());
        if NT_SUCCESS(Status) {
            Status = CmpInitHiveFromFile(
                &FileName,
                0,
                NewHive,
                Allocate,
                CheckFlags,
            );

            PsRevertToSelf();
        }
    }

    if (*FileAttributes).RootDirectory != core::ptr::null_mut() {
        ExFreePoolWithTag(FileName.Buffer as *mut core::ffi::c_void, TAG_CM);
    }

    Status
}

#[no_mangle]
pub unsafe extern "system" fn CmpShutdownWorkers() {
    KeCancelTimer(core::ptr::addr_of_mut!(CmpLazyFlushTimer));
}

#[no_mangle]
pub unsafe extern "system" fn CmSetLazyFlushState(Enable: BOOLEAN) {
    CmpHoldLazyFlush = if Enable != 0 { 0 } else { 1 };
}


// ============================================================================
// ORIGINAL C SOURCE
// ============================================================================
//
// /*
//  * PROJECT:     ReactOS Kernel
//  * LICENSE:     GPL - See COPYING in the top level directory
//  * PURPOSE:     Configuration Manager - Internal Registry APIs
//  * PROGRAMMERS: Alex Ionescu <alex.ionescu@reactos.org>
//  */
//
// /* INCLUDES *******************************************************************/
//
// #include "ntoskrnl.h"
// #define NDEBUG
// #include "debug.h"
//
// /* GLOBALS ********************************************************************/
//
// KTIMER CmpLazyFlushTimer;
// KDPC CmpLazyFlushDpc;
// WORK_QUEUE_ITEM CmpLazyWorkItem;
// KTIMER CmpEnableLazyFlushTimer;
// KDPC CmpEnableLazyFlushDpc;
// BOOLEAN CmpLazyFlushPending;
// BOOLEAN CmpForceForceFlush;
// BOOLEAN CmpHoldLazyFlush = TRUE;
// ULONG CmpLazyFlushIntervalInSeconds = 5;
// ULONG CmpLazyFlushHiveCount = 7;
// ULONG CmpLazyFlushCount = 1;
// LONG CmpFlushStarveWriters;
//
// /* FUNCTIONS ******************************************************************/
//
// BOOLEAN
// NTAPI
// CmpDoFlushNextHive(_In_  BOOLEAN ForceFlush,
//                    _Out_ PBOOLEAN Error,
//                    _Out_ PULONG DirtyCount)
// {
//     NTSTATUS Status;
//     PLIST_ENTRY NextEntry;
//     PCMHIVE CmHive;
//     BOOLEAN Result;
//     ULONG HiveCount = CmpLazyFlushHiveCount;
//
//     /* Set Defaults */
//     *Error = FALSE;
//     *DirtyCount = 0;
//
//     /* Don't do anything if we're not supposed to */
//     if (CmpNoWrite) return TRUE;
//
//     /* Make sure we have to flush at least one hive */
//     if (!HiveCount) HiveCount = 1;
//
//     /* Acquire the list lock and loop */
//     ExAcquirePushLockShared(&CmpHiveListHeadLock);
//     NextEntry = CmpHiveListHead.Flink;
//     while ((NextEntry != &CmpHiveListHead) && HiveCount)
//     {
//         /* Get the hive and check if we should flush it */
//         CmHive = CONTAINING_RECORD(NextEntry, CMHIVE, HiveList);
//         if (!(CmHive->Hive.HiveFlags & HIVE_NOLAZYFLUSH) &&
//             (CmHive->FlushCount != CmpLazyFlushCount))
//         {
//             /* Great success! */
//             Result = TRUE;
//
//             /* One less to flush */
//             HiveCount--;
//
//             /* Ignore clean or volatile hives */
//             if ((!CmHive->Hive.DirtyCount && !ForceFlush) ||
//                 (CmHive->Hive.HiveFlags & HIVE_VOLATILE))
//             {
//                 /* Don't do anything but do update the count */
//                 CmHive->FlushCount = CmpLazyFlushCount;
//                 DPRINT("Hive %wZ is clean.\n", &CmHive->FileFullPath);
//             }
//             else
//             {
//                 /* Do the sync */
//                 DPRINT("Flushing: %wZ\n", &CmHive->FileFullPath);
//                 DPRINT("Handle: %p\n", CmHive->FileHandles[HFILE_TYPE_PRIMARY]);
//                 Status = HvSyncHive(&CmHive->Hive);
//                 if (!NT_SUCCESS(Status))
//                 {
//                     /* Let them know we failed */
//                     DPRINT1("Failed to flush %wZ on handle %p (status 0x%08lx)\n",
//                         &CmHive->FileFullPath,  CmHive->FileHandles[HFILE_TYPE_PRIMARY], Status);
//                     *Error = TRUE;
//                     Result = FALSE;
//                     break;
//                 }
//                 CmHive->FlushCount = CmpLazyFlushCount;
//             }
//         }
//         else if (CmHive->Hive.DirtyCount &&
//                  !(CmHive->Hive.HiveFlags & HIVE_VOLATILE) &&
//                  !(CmHive->Hive.HiveFlags & HIVE_NOLAZYFLUSH))
//         {
//             /* Use another lazy flusher for this hive */
//             ASSERT(CmHive->FlushCount == CmpLazyFlushCount);
//             *DirtyCount += CmHive->Hive.DirtyCount;
//             DPRINT("CmHive %wZ already uptodate.\n", &CmHive->FileFullPath);
//         }
//
//         /* Try the next one */
//         NextEntry = NextEntry->Flink;
//     }
//
//     /* Check if we've flushed everything */
//     if (NextEntry == &CmpHiveListHead)
//     {
//         /* We have, tell the caller we're done */
//         Result = FALSE;
//     }
//     else
//     {
//         /* We need to be called again */
//         Result = TRUE;
//     }
//
//     /* Unlock the list and return the result */
//     ExReleasePushLock(&CmpHiveListHeadLock);
//     return Result;
// }
//
// _Function_class_(KDEFERRED_ROUTINE)
// VOID
// NTAPI
// CmpEnableLazyFlushDpcRoutine(IN PKDPC Dpc,
//                              IN PVOID DeferredContext,
//                              IN PVOID SystemArgument1,
//                              IN PVOID SystemArgument2)
// {
//     /* Don't stop lazy flushing from happening anymore */
//     CmpHoldLazyFlush = FALSE;
// }
//
// _Function_class_(KDEFERRED_ROUTINE)
// VOID
// NTAPI
// CmpLazyFlushDpcRoutine(IN PKDPC Dpc,
//                        IN PVOID DeferredContext,
//                        IN PVOID SystemArgument1,
//                        IN PVOID SystemArgument2)
// {
//     /* Check if we should queue the lazy flush worker */
//     DPRINT("Flush pending: %s, Holding lazy flush: %s.\n", CmpLazyFlushPending ? "yes" : "no", CmpHoldLazyFlush ? "yes" : "no");
//     if (!CmpLazyFlushPending && !CmpHoldLazyFlush)
//     {
//         CmpLazyFlushPending = TRUE;
//         ExQueueWorkItem(&CmpLazyWorkItem, DelayedWorkQueue);
//     }
// }
//
// VOID
// NTAPI
// CmpLazyFlush(VOID)
// {
//     LARGE_INTEGER DueTime;
//     PAGED_CODE();
//
//     /* Check if we should set the lazy flush timer */
//     if (!CmpNoWrite && !CmpHoldLazyFlush)
//     {
//         /* Do it */
//         DueTime.QuadPart = Int32x32To64(CmpLazyFlushIntervalInSeconds,
//                                         -10 * 1000 * 1000);
//         KeSetTimer(&CmpLazyFlushTimer, DueTime, &CmpLazyFlushDpc);
//     }
// }
//
// _Function_class_(WORKER_THREAD_ROUTINE)
// VOID
// NTAPI
// CmpLazyFlushWorker(IN PVOID Parameter)
// {
//     BOOLEAN ForceFlush, Result, MoreWork = FALSE;
//     ULONG DirtyCount = 0;
//     PAGED_CODE();
//
//     /* Don't do anything if lazy flushing isn't enabled yet */
//     if (CmpHoldLazyFlush)
//     {
//         DPRINT1("Lazy flush held. Bye bye.\n");
//         CmpLazyFlushPending = FALSE;
//         return;
//     }
//
//     /* Check if we are forcing a flush */
//     ForceFlush = CmpForceForceFlush;
//     if (ForceFlush)
//     {
//         DPRINT("Forcing flush.\n");
//         /* Lock the registry exclusively */
//         CmpLockRegistryExclusive();
//     }
//     else
//     {
//         DPRINT("Not forcing flush.\n");
//         /* Starve writers before locking */
//         InterlockedIncrement(&CmpFlushStarveWriters);
//         CmpLockRegistry();
//     }
//
//     /* Flush the next hive */
//     MoreWork = CmpDoFlushNextHive(ForceFlush, &Result, &DirtyCount);
//     if (!MoreWork)
//     {
//         /* We're done */
//         InterlockedIncrement((PLONG)&CmpLazyFlushCount);
//     }
//
//     /* Check if we have starved writers */
//     if (!ForceFlush)
//         InterlockedDecrement(&CmpFlushStarveWriters);
//
//     /* Not pending anymore, release the registry lock */
//     CmpLazyFlushPending = FALSE;
//     CmpUnlockRegistry();
//
//     DPRINT("Lazy flush done. More work to be done: %s. Entries still dirty: %u.\n",
//         MoreWork ? "Yes" : "No", DirtyCount);
//
//     if (MoreWork)
//     {
//         /* Relaunch the flush timer, so the remaining hives get flushed */
//         CmpLazyFlush();
//     }
// }
//
// VOID
// NTAPI
// CmpCmdInit(IN BOOLEAN SetupBoot)
// {
//     LARGE_INTEGER DueTime;
//     PAGED_CODE();
//
//     /* Setup the lazy DPC */
//     KeInitializeDpc(&CmpLazyFlushDpc, CmpLazyFlushDpcRoutine, NULL);
//
//     /* Setup the lazy timer */
//     KeInitializeTimer(&CmpLazyFlushTimer);
//
//     /* Setup the lazy worker */
//     ExInitializeWorkItem(&CmpLazyWorkItem, CmpLazyFlushWorker, NULL);
//
//     /* Setup the forced-lazy DPC and timer */
//     KeInitializeDpc(&CmpEnableLazyFlushDpc,
//                     CmpEnableLazyFlushDpcRoutine,
//                     NULL);
//     KeInitializeTimer(&CmpEnableLazyFlushTimer);
//
//     /* Enable lazy flushing after 10 minutes */
//     DueTime.QuadPart = Int32x32To64(600, -10 * 1000 * 1000);
//     KeSetTimer(&CmpEnableLazyFlushTimer, DueTime, &CmpEnableLazyFlushDpc);
//
//     /* Setup flush variables */
//     CmpNoWrite = CmpMiniNTBoot;
//     CmpWasSetupBoot = SetupBoot;
//
//     /* Testing: Force Lazy Flushing */
//     CmpHoldLazyFlush = FALSE;
//
//     /* Setup the system hives list if this is not a Setup boot */
//     if (!SetupBoot)
//         CmpInitializeHiveList();
//
//     /* Now that the system hives are loaded, if we are in PE mode,
//      * all other hives will be loaded with full access */
//     if (CmpMiniNTBoot)
//         CmpShareSystemHives = FALSE;
// }
//
// NTSTATUS
// NTAPI
// CmpCmdHiveOpen(IN POBJECT_ATTRIBUTES FileAttributes,
//                IN PSECURITY_CLIENT_CONTEXT ImpersonationContext,
//                IN OUT PBOOLEAN Allocate,
//                OUT PCMHIVE *NewHive,
//                IN ULONG CheckFlags)
// {
//     NTSTATUS Status;
//     UNICODE_STRING FileName;
//     PWCHAR FilePath;
//     ULONG Length;
//     OBJECT_NAME_INFORMATION DummyNameInfo;
//     POBJECT_NAME_INFORMATION FileNameInfo;
//
//     PAGED_CODE();
//
//     if (FileAttributes->RootDirectory)
//     {
//         /*
//          * Validity check: The ObjectName is relative to RootDirectory,
//          * therefore it must not start with a path separator.
//          */
//         if (FileAttributes->ObjectName && FileAttributes->ObjectName->Buffer &&
//             FileAttributes->ObjectName->Length >= sizeof(WCHAR) &&
//             *FileAttributes->ObjectName->Buffer == OBJ_NAME_PATH_SEPARATOR)
//         {
//             return STATUS_OBJECT_PATH_SYNTAX_BAD;
//         }
//
//         /* Determine the right buffer size and allocate */
//         Status = ZwQueryObject(FileAttributes->RootDirectory,
//                                ObjectNameInformation,
//                                &DummyNameInfo,
//                                sizeof(DummyNameInfo),
//                                &Length);
//         if (Status != STATUS_BUFFER_OVERFLOW)
//         {
//             DPRINT1("CmpCmdHiveOpen(): Root directory handle object name size query failed, Status = 0x%08lx\n", Status);
//             return Status;
//         }
//
//         FileNameInfo = ExAllocatePoolWithTag(PagedPool,
//                                              Length + sizeof(UNICODE_NULL),
//                                              TAG_CM);
//         if (FileNameInfo == NULL)
//         {
//             DPRINT1("CmpCmdHiveOpen(): Unable to allocate memory\n");
//             return STATUS_INSUFFICIENT_RESOURCES;
//         }
//
//         /* Try to get the value */
//         Status = ZwQueryObject(FileAttributes->RootDirectory,
//                                ObjectNameInformation,
//                                FileNameInfo,
//                                Length,
//                                &Length);
//         if (!NT_SUCCESS(Status))
//         {
//             /* Fail */
//             DPRINT1("CmpCmdHiveOpen(): Root directory handle object name query failed, Status = 0x%08lx\n", Status);
//             ExFreePoolWithTag(FileNameInfo, TAG_CM);
//             return Status;
//         }
//
//         /* Null-terminate and add the length of the terminator */
//         Length -= sizeof(OBJECT_NAME_INFORMATION);
//         FilePath = FileNameInfo->Name.Buffer;
//         FilePath[Length / sizeof(WCHAR)] = UNICODE_NULL;
//         Length += sizeof(UNICODE_NULL);
//
//         /* Compute the size of the full path; Length already counts the terminating NULL */
//         Length = Length + sizeof(WCHAR) + FileAttributes->ObjectName->Length;
//         if (Length > MAXUSHORT)
//         {
//             /* Name size too long, bail out */
//             ExFreePoolWithTag(FileNameInfo, TAG_CM);
//             return STATUS_OBJECT_PATH_INVALID;
//         }
//
//         /* Build the full path */
//         RtlInitEmptyUnicodeString(&FileName, NULL, 0);
//         FileName.Buffer = ExAllocatePoolWithTag(PagedPool, Length, TAG_CM);
//         if (!FileName.Buffer)
//         {
//             /* Fail */
//             DPRINT1("CmpCmdHiveOpen(): Unable to allocate memory\n");
//             ExFreePoolWithTag(FileNameInfo, TAG_CM);
//             return STATUS_INSUFFICIENT_RESOURCES;
//         }
//         FileName.MaximumLength = Length;
//         RtlCopyUnicodeString(&FileName, &FileNameInfo->Name);
//         ExFreePoolWithTag(FileNameInfo, TAG_CM);
//
//         /*
//          * Append a path terminator if needed (we have already accounted
//          * for a possible extra one when allocating the buffer).
//          */
//         if (/* FileAttributes->ObjectName->Buffer[0] != OBJ_NAME_PATH_SEPARATOR && */ // We excluded ObjectName starting with a path separator above.
//             FileName.Length > 0 && FileName.Buffer[FileName.Length / sizeof(WCHAR) - 1] != OBJ_NAME_PATH_SEPARATOR)
//         {
//             /* ObjectName does not start with '\' and PathBuffer does not end with '\' */
//             FileName.Buffer[FileName.Length / sizeof(WCHAR)] = OBJ_NAME_PATH_SEPARATOR;
//             FileName.Length += sizeof(WCHAR);
//             FileName.Buffer[FileName.Length / sizeof(WCHAR)] = UNICODE_NULL;
//         }
//
//         /* Append the object name */
//         Status = RtlAppendUnicodeStringToString(&FileName, FileAttributes->ObjectName);
//         if (!NT_SUCCESS(Status))
//         {
//             /* Fail */
//             DPRINT1("CmpCmdHiveOpen(): RtlAppendUnicodeStringToString() failed, Status = 0x%08lx\n", Status);
//             ExFreePoolWithTag(FileName.Buffer, TAG_CM);
//             return Status;
//         }
//     }
//     else
//     {
//         FileName = *FileAttributes->ObjectName;
//     }
//
//     /* Open the file in the current security context */
//     Status = CmpInitHiveFromFile(&FileName,
//                                  0,
//                                  NewHive,
//                                  Allocate,
//                                  CheckFlags);
//     if (((Status == STATUS_ACCESS_DENIED) ||
//          (Status == STATUS_NO_SUCH_USER) ||
//          (Status == STATUS_WRONG_PASSWORD) ||
//          (Status == STATUS_ACCOUNT_EXPIRED) ||
//          (Status == STATUS_ACCOUNT_DISABLED) ||
//          (Status == STATUS_ACCOUNT_RESTRICTION)) &&
//         ImpersonationContext)
//     {
//         /* We failed due to an account/security error, impersonate SYSTEM */
//         Status = SeImpersonateClientEx(ImpersonationContext, NULL);
//         if (NT_SUCCESS(Status))
//         {
//             /* Now try again */
//             Status = CmpInitHiveFromFile(&FileName,
//                                          0,
//                                          NewHive,
//                                          Allocate,
//                                          CheckFlags);
//
//             /* Restore impersonation token */
//             PsRevertToSelf();
//         }
//     }
//
//     if (FileAttributes->RootDirectory)
//     {
//         ExFreePoolWithTag(FileName.Buffer, TAG_CM);
//     }
//
//     /* Return status of open attempt */
//     return Status;
// }
//
// VOID
// NTAPI
// CmpShutdownWorkers(VOID)
// {
//     /* Stop lazy flushing */
//     PAGED_CODE();
//     KeCancelTimer(&CmpLazyFlushTimer);
// }
//
// VOID
// NTAPI
// CmSetLazyFlushState(IN BOOLEAN Enable)
// {
//     /* Set state for lazy flusher */
//     CmpHoldLazyFlush = !Enable;
// }
//
// /* EOF */
//
// ============================================================================
// END ORIGINAL C SOURCE
// ============================================================================
