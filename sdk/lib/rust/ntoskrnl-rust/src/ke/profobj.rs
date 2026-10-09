use super::ke_defs::*;
use ntkrnl_sys::globals::*;
use ntkrnl_sys::base::*;
use ntkrnl_sys::constants::*;
use ntkrnl_sys::ke::*;
// mm::* import removed to avoid KeGetCurrentThread ambiguity
use ntkrnl_sys::mm::NonPagedPool;
use ntkrnl_sys::ex::*;
use ntkrnl_sys::rtl::*;

use ntkrnl_sys::ddk::{
    KPROFILE, PKPROFILE, PKPROCESS, KPROFILE_SOURCE_OBJECT, PKPROFILE_SOURCE_OBJECT,
    PKTRAP_FRAME, KAFFINITY, LIST_ENTRY, KPROFILE_SOURCE, HAL_PROFILE_SOURCE_INFORMATION,
    HAL_PROFILE_SOURCE_INTERVAL,
    HalStartProfileInterrupt, HalStopProfileInterrupt, HalSetProfileInterval,
};
use ntkrnl_sys::{opaque_ptr, opaque_ptr_const};

extern "system" {
    fn HalQuerySystemInformation(
        InformationClass: ULONG,
        BufferSize: ULONG,
        Buffer: PVOID,
        ReturnedLength: *mut ULONG,
    ) -> NTSTATUS;
    fn HalSetSystemInformation(
        InformationClass: ULONG,
        BufferSize: ULONG,
        Buffer: PVOID,
    ) -> NTSTATUS;
}

#[cfg(any(target_arch = "x86", target_arch = "x86_64"))]
extern "system" {
    fn KeGetTrapFramePc(TrapFrame: PKTRAP_FRAME) -> ULONG_PTR;
}

// On arm64, KeGetTrapFramePc is a macro: TrapFrame->Pc
#[cfg(target_arch = "aarch64")]
#[inline(always)]
unsafe fn KeGetTrapFramePc(TrapFrame: PKTRAP_FRAME) -> ULONG_PTR {
    (*TrapFrame).Pc as ULONG_PTR
}

#[no_mangle]
pub static mut KiProfileIrql: KIRQL = 27;
#[no_mangle]
pub static mut KiProfileListHead: LIST_ENTRY = LIST_ENTRY {
    Flink: core::ptr::null_mut(),
    Blink: core::ptr::null_mut(),
};
#[no_mangle]
pub static mut KiProfileSourceListHead: LIST_ENTRY = LIST_ENTRY {
    Flink: core::ptr::null_mut(),
    Blink: core::ptr::null_mut(),
};
#[no_mangle]
pub static mut KiProfileLock: KSPIN_LOCK = 0;
#[no_mangle]
pub static mut KiProfileTimeInterval: ULONG = 78125;
#[no_mangle]
pub static mut KiProfileAlignmentFixupInterval: ULONG = 0;

#[no_mangle]
pub unsafe extern "system" fn KeInitializeProfile(
    Profile: PKPROFILE,
    Process: PKPROCESS,
    ImageBase: PVOID,
    ImageSize: SIZE_T,
    BucketSize: ULONG,
    ProfileSource: KPROFILE_SOURCE,
    Affinity: KAFFINITY,
) {
    let Process = opaque_ptr(Process);
    let ImageBase = opaque_ptr(ImageBase);
    (*Profile).Type = 24i16;
    (*Profile).Size = core::mem::size_of::<KPROFILE>() as i16;

    (*Profile).Process = Process;
    (*Profile).RangeBase = ImageBase;
    (*Profile).BucketShift = BucketSize - 2;
    (*Profile).RangeLimit = (ImageBase as ULONG_PTR + ImageSize) as PVOID;
    (*Profile).Started = 0u8;
    (*Profile).Source = ProfileSource;
    (*Profile).Affinity = Affinity;
}

#[no_mangle]
pub unsafe extern "system" fn KeStartProfile(
    Profile: PKPROFILE,
    Buffer: PVOID,
) -> BOOLEAN {
    let Buffer = opaque_ptr(Buffer);
    let mut OldIrql: KIRQL = 0;
    let mut SourceBuffer: PKPROFILE_SOURCE_OBJECT;
    let mut CurrentSource: PKPROFILE_SOURCE_OBJECT = core::ptr::null_mut();
    let mut FreeBuffer = 1u8;
    let mut SourceFound = 0u8;
    let StartedProfile: BOOLEAN;
    let ProfileProcess: PKPROCESS;
    let mut NextEntry: PLIST_ENTRY;

    SourceBuffer = ExAllocatePoolWithTag(
        NonPagedPool,
        core::mem::size_of::<KPROFILE_SOURCE_OBJECT>(),
        u32::from_ne_bytes(*b"forP"),
    ) as PKPROFILE_SOURCE_OBJECT;
    if SourceBuffer.is_null() {
        return 0u8;
    }
    core::ptr::write_bytes(SourceBuffer as *mut u8, 0, core::mem::size_of::<KPROFILE_SOURCE_OBJECT>());

    // [guard] KeRaiseIrql(KiProfileIrql, core::ptr::addr_of_mut!(OldIrql));
    let irql_guard = unsafe { ntkrnl_sys::guards::IrqlGuard::raise(KiProfileIrql) };
    KeAcquireSpinLockAtDpcLevel(core::ptr::addr_of_mut!(KiProfileLock));

    if (*Profile).Started == 0 {
        (*Profile).Buffer = Buffer;
        (*Profile).Started = 1u8;
        StartedProfile = 1u8;

        ProfileProcess = (*Profile).Process;

        if !ProfileProcess.is_null() {
            InsertTailList(
                core::ptr::addr_of_mut!((*ProfileProcess).ProfileListHead) as PLIST_ENTRY,
                core::ptr::addr_of_mut!((*Profile).ProfileListEntry) as PLIST_ENTRY,
            );
        } else {
            InsertTailList(
                core::ptr::addr_of_mut!(KiProfileListHead) as PLIST_ENTRY,
                core::ptr::addr_of_mut!((*Profile).ProfileListEntry) as PLIST_ENTRY,
            );
        }

        NextEntry = KiProfileSourceListHead.Flink;
        while NextEntry != core::ptr::addr_of_mut!(KiProfileSourceListHead) as PLIST_ENTRY {
            CurrentSource = crate::CONTAINING_RECORD!(NextEntry, KPROFILE_SOURCE_OBJECT, ListEntry);

            if (*CurrentSource).Source == (*Profile).Source {
                SourceFound = 1u8;
                break;
            }

            NextEntry = (*NextEntry).Flink;
        }

        if SourceFound == 0 {
            CurrentSource = SourceBuffer;

            (*CurrentSource).Source = (*Profile).Source;
            InsertHeadList(
                core::ptr::addr_of_mut!(KiProfileSourceListHead) as PLIST_ENTRY,
                core::ptr::addr_of_mut!((*CurrentSource).ListEntry) as PLIST_ENTRY,
            );

            FreeBuffer = 0u8;
        }
    } else {
        StartedProfile = 0u8;
    }

    KeReleaseSpinLockFromDpcLevel(core::ptr::addr_of_mut!(KiProfileLock));

    HalStartProfileInterrupt((*Profile).Source);

    drop(irql_guard);

    if FreeBuffer != 0 {
        ExFreePoolWithTag(SourceBuffer as PVOID, u32::from_ne_bytes(*b"forP"));
    }

    StartedProfile
}

#[no_mangle]
pub unsafe extern "system" fn KeStopProfile(Profile: PKPROFILE) -> BOOLEAN {
    let mut OldIrql: KIRQL = 0;
    let mut CurrentSource: PKPROFILE_SOURCE_OBJECT = core::ptr::null_mut();
    let mut NextEntry: PLIST_ENTRY;
    let mut SourceFound = 0u8;
    let StoppedProfile: BOOLEAN;

    // [guard] KeRaiseIrql(KiProfileIrql, core::ptr::addr_of_mut!(OldIrql));
    let irql_guard = unsafe { ntkrnl_sys::guards::IrqlGuard::raise(KiProfileIrql) };
    KeAcquireSpinLockAtDpcLevel(core::ptr::addr_of_mut!(KiProfileLock));

    if (*Profile).Started != 0 {
        RemoveEntryList(core::ptr::addr_of_mut!((*Profile).ProfileListEntry) as PLIST_ENTRY);
        (*Profile).Started = 0u8;
        StoppedProfile = 1u8;

        NextEntry = KiProfileSourceListHead.Flink;
        while NextEntry != core::ptr::addr_of_mut!(KiProfileSourceListHead) as PLIST_ENTRY {
            CurrentSource = crate::CONTAINING_RECORD!(NextEntry, KPROFILE_SOURCE_OBJECT, ListEntry);

            if (*CurrentSource).Source == (*Profile).Source {
                SourceFound = 1u8;
                RemoveEntryList(core::ptr::addr_of_mut!((*CurrentSource).ListEntry) as PLIST_ENTRY);
                break;
            }

            NextEntry = (*NextEntry).Flink;
        }
    } else {
        StoppedProfile = 0u8;
    }

    KeReleaseSpinLockFromDpcLevel(core::ptr::addr_of_mut!(KiProfileLock));

    HalStopProfileInterrupt((*Profile).Source);

    drop(irql_guard);

    if SourceFound != 0 {
        ExFreePool(CurrentSource as PVOID);
    }

    StoppedProfile
}

#[no_mangle]
pub unsafe extern "system" fn KeQueryIntervalProfile(
    ProfileSource: KPROFILE_SOURCE,
) -> ULONG {
    let mut ProfileSourceInformation: HAL_PROFILE_SOURCE_INFORMATION = core::mem::zeroed();
    let mut ReturnLength: ULONG = 0;
    let Interval: ULONG;
    let Status: NTSTATUS;

    if ProfileSource == 0 {
        Interval = KiProfileTimeInterval;
    } else if ProfileSource == 1 {
        Interval = KiProfileAlignmentFixupInterval;
    } else {
        ProfileSourceInformation.Source = ProfileSource;
        Status = HalQuerySystemInformation(
            6,
            core::mem::size_of::<HAL_PROFILE_SOURCE_INFORMATION>() as ULONG,
            core::ptr::addr_of_mut!(ProfileSourceInformation) as PVOID,
            core::ptr::addr_of_mut!(ReturnLength),
        );

        if Status >= 0 && ProfileSourceInformation.Supported != 0 {
            Interval = ProfileSourceInformation.Interval;
        } else {
            Interval = 0;
        }
    }

    Interval
}

#[no_mangle]
pub unsafe extern "system" fn KeSetIntervalProfile(
    Interval: ULONG,
    ProfileSource: KPROFILE_SOURCE,
) {
    let mut ProfileSourceInterval: HAL_PROFILE_SOURCE_INTERVAL = core::mem::zeroed();

    if ProfileSource == 0 {
        KiProfileTimeInterval = HalSetProfileInterval(Interval as _) as ULONG;
    } else if ProfileSource == 1 {
        KiProfileAlignmentFixupInterval = Interval;
    } else {
        ProfileSourceInterval.Source = ProfileSource;
        ProfileSourceInterval.Interval = Interval as _;
        HalSetSystemInformation(
            ntkrnl_sys::ddk::_HAL_SET_INFORMATION_CLASS_HalProfileSourceInterval as _,
            core::mem::size_of::<HAL_PROFILE_SOURCE_INTERVAL>() as ULONG,
            core::ptr::addr_of_mut!(ProfileSourceInterval) as PVOID,
        );
    }
}

#[cfg(any(target_arch = "x86", target_arch = "arm"))]
#[no_mangle]
pub unsafe extern "system" fn KeProfileInterrupt(TrapFrame: PKTRAP_FRAME) {
    KeProfileInterruptWithSource(TrapFrame, 0);
}

unsafe fn KiParseProfileList(
    TrapFrame: PKTRAP_FRAME,
    Source: KPROFILE_SOURCE,
    ListHead: PLIST_ENTRY,
) {
    let mut BucketValue: *mut ULONG;
    let mut Profile: PKPROFILE;
    let mut NextEntry: PLIST_ENTRY;
    let ProgramCounter: ULONG_PTR;

    ProgramCounter = KeGetTrapFramePc(TrapFrame);

    NextEntry = (*ListHead).Flink;
    while NextEntry != ListHead {
        Profile = crate::CONTAINING_RECORD!(NextEntry, KPROFILE, ProfileListEntry);

        if (*Profile).Source != Source
            || ProgramCounter < (*Profile).RangeBase as ULONG_PTR
            || ProgramCounter > (*Profile).RangeLimit as ULONG_PTR
        {
            NextEntry = (*NextEntry).Flink;
            continue;
        }

        BucketValue = ((*Profile).Buffer as ULONG_PTR
            + (((ProgramCounter - (*Profile).RangeBase as ULONG_PTR)
                >> (*Profile).BucketShift)
                & !0x3)) as *mut ULONG;

        *BucketValue = (*BucketValue).wrapping_add(1);

        NextEntry = (*NextEntry).Flink;
    }
}

#[no_mangle]
pub unsafe extern "system" fn KeProfileInterruptWithSource(
    TrapFrame: PKTRAP_FRAME,
    Source: KPROFILE_SOURCE,
) {
    let Process = (*PsGetCurrentThread()).__bindgen_anon_1.ApcState.Process;

    KiParseProfileList(
        TrapFrame,
        Source,
        core::ptr::addr_of_mut!((*Process).ProfileListHead) as PLIST_ENTRY,
    );
    KiParseProfileList(TrapFrame, Source, core::ptr::addr_of_mut!(KiProfileListHead) as PLIST_ENTRY);
}

#[no_mangle]
pub unsafe extern "system" fn KeSetProfileIrql(ProfileIrql: KIRQL) {
    KiProfileIrql = ProfileIrql;
}

// ============================================================================
// ORIGINAL C SOURCE
// ============================================================================
//
// /*
//  * COPYRIGHT:       GPL - See COPYING in the top level directory
//  * PROJECT:         ReactOS Kernel
//  * FILE:            ntoskrnl/ke/profobj.c
//  * PURPOSE:         Kernel Profiling
//  * PROGRAMMERS:     Alex Ionescu (alex@relsoft.net)
//  */
//
// /* INCLUDES *****************************************************************/
//
// #include <ntoskrnl.h>
// #define NDEBUG
// #include <debug.h>
//
// /* GLOBALS *******************************************************************/
//
// KIRQL KiProfileIrql = PROFILE_LEVEL;
// LIST_ENTRY KiProfileListHead;
// LIST_ENTRY KiProfileSourceListHead;
// KSPIN_LOCK KiProfileLock;
// ULONG KiProfileTimeInterval = 78125; /* Default resolution 7.8ms (sysinternals) */
// ULONG KiProfileAlignmentFixupInterval;
//
// /* FUNCTIONS *****************************************************************/
//
// VOID
// NTAPI
// KeInitializeProfile(PKPROFILE Profile,
//                     PKPROCESS Process,
//                     PVOID ImageBase,
//                     SIZE_T ImageSize,
//                     ULONG BucketSize,
//                     KPROFILE_SOURCE ProfileSource,
//                     KAFFINITY Affinity)
// {
//     /* Initialize the Header */
//     Profile->Type = ProfileObject;
//     Profile->Size = sizeof(KPROFILE);
//
//     /* Copy all the settings we were given */
//     Profile->Process = Process;
//     Profile->RangeBase = ImageBase;
//     Profile->BucketShift = BucketSize - 2; /* See ntinternals.net -- Alex */
//     Profile->RangeLimit = (PVOID)((ULONG_PTR)ImageBase + ImageSize);
//     Profile->Started = FALSE;
//     Profile->Source = ProfileSource;
//     Profile->Affinity = Affinity;
// }
//
// BOOLEAN
// NTAPI
// KeStartProfile(IN PKPROFILE Profile,
//                IN PVOID Buffer)
// {
//     KIRQL OldIrql;
//     PKPROFILE_SOURCE_OBJECT SourceBuffer;
//     PKPROFILE_SOURCE_OBJECT CurrentSource;
//     BOOLEAN FreeBuffer = TRUE, SourceFound = FALSE, StartedProfile;
//     PKPROCESS ProfileProcess;
//     PLIST_ENTRY NextEntry;
//
//     /* Allocate a buffer first, before we raise IRQL */
//     SourceBuffer = ExAllocatePoolWithTag(NonPagedPool,
//                                          sizeof(KPROFILE_SOURCE_OBJECT),
//                                          'forP');
//     if (!SourceBuffer) return FALSE;
//     RtlZeroMemory(SourceBuffer, sizeof(KPROFILE_SOURCE_OBJECT));
//
//     /* Raise to profile IRQL and acquire the profile lock */
//     KeRaiseIrql(KiProfileIrql, &OldIrql);
//     KeAcquireSpinLockAtDpcLevel(&KiProfileLock);
//
//     /* Make sure it's not running */
//     if (!Profile->Started)
//     {
//         /* Set it as Started */
//         Profile->Buffer = Buffer;
//         Profile->Started = TRUE;
//         StartedProfile = TRUE;
//
//         /* Get the process, if any */
//         ProfileProcess = Profile->Process;
//
//         /* Check where we should insert it */
//         if (ProfileProcess)
//         {
//             /* Insert it into the Process List */
//             InsertTailList(&ProfileProcess->ProfileListHead, &Profile->ProfileListEntry);
//         }
//         else
//         {
//             /* Insert it into the Global List */
//             InsertTailList(&KiProfileListHead, &Profile->ProfileListEntry);
//         }
//
//         /* Start looping */
//         for (NextEntry = KiProfileSourceListHead.Flink;
//              NextEntry != &KiProfileSourceListHead;
//              NextEntry = NextEntry->Flink)
//         {
//             /* Get the entry */
//             CurrentSource = CONTAINING_RECORD(NextEntry,
//                                               KPROFILE_SOURCE_OBJECT,
//                                               ListEntry);
//
//             /* Check if it's the same as the one being requested now */
//             if (CurrentSource->Source == Profile->Source)
//             {
//                 /* It is, break out */
//                 SourceFound = TRUE;
//                 break;
//             }
//         }
//
//         /* See if the loop found something */
//         if (!SourceFound)
//         {
//             /* Nothing found, use our allocated buffer */
//             CurrentSource = SourceBuffer;
//
//             /* Set up the Source Object */
//             CurrentSource->Source = Profile->Source;
//             InsertHeadList(&KiProfileSourceListHead, &CurrentSource->ListEntry);
//
//             /* Don't free the pool later on */
//             FreeBuffer = FALSE;
//         }
//     }
//     else
//     {
//         /* Already running so nothing to start */
//         StartedProfile = FALSE;
//     }
//
//     /* Release the profile lock */
//     KeReleaseSpinLockFromDpcLevel(&KiProfileLock);
//
//     /* Tell HAL to start the profile interrupt */
//     HalStartProfileInterrupt(Profile->Source);
//
//     /* Lower back to original IRQL */
//     KeLowerIrql(OldIrql);
//
//     /* Free the pool */
//     if (FreeBuffer) ExFreePoolWithTag(SourceBuffer, 'forP');
//
//     /* Return whether we could start the profile */
//     return StartedProfile;
// }
//
// BOOLEAN
// NTAPI
// KeStopProfile(IN PKPROFILE Profile)
// {
//     KIRQL OldIrql;
//     PKPROFILE_SOURCE_OBJECT CurrentSource = NULL;
//     PLIST_ENTRY NextEntry;
//     BOOLEAN SourceFound = FALSE, StoppedProfile;
//
//     /* Raise to profile IRQL and acquire the profile lock */
//     KeRaiseIrql(KiProfileIrql, &OldIrql);
//     KeAcquireSpinLockAtDpcLevel(&KiProfileLock);
//
//     /* Make sure it's running */
//     if (Profile->Started)
//     {
//         /* Remove it from the list and disable */
//         RemoveEntryList(&Profile->ProfileListEntry);
//         Profile->Started = FALSE;
//         StoppedProfile = TRUE;
//
//         /* Start looping */
//         for (NextEntry = KiProfileSourceListHead.Flink;
//              NextEntry != &KiProfileSourceListHead;
//              NextEntry = NextEntry->Flink)
//         {
//             /* Get the entry */
//             CurrentSource = CONTAINING_RECORD(NextEntry,
//                                               KPROFILE_SOURCE_OBJECT,
//                                               ListEntry);
//
//             /* Check if this is the Source Object */
//             if (CurrentSource->Source == Profile->Source)
//             {
//                 /* Remember we found one */
//                 SourceFound = TRUE;
//
//                 /* Remove it and break out */
//                 RemoveEntryList(&CurrentSource->ListEntry);
//                 break;
//             }
//         }
//
//     }
//     else
//     {
//         /* It wasn't! */
//         StoppedProfile = FALSE;
//     }
//
//     /* Release the profile lock */
//     KeReleaseSpinLockFromDpcLevel(&KiProfileLock);
//
//     /* Stop the profile interrupt */
//     HalStopProfileInterrupt(Profile->Source);
//
//     /* Lower back to original IRQL */
//     KeLowerIrql(OldIrql);
//
//     /* Free the Source Object */
//     if (SourceFound) ExFreePool(CurrentSource);
//
//     /* Return whether we could stop the profile */
//     return StoppedProfile;
// }
//
// ULONG
// NTAPI
// KeQueryIntervalProfile(IN KPROFILE_SOURCE ProfileSource)
// {
//     HAL_PROFILE_SOURCE_INFORMATION ProfileSourceInformation;
//     ULONG ReturnLength, Interval;
//     NTSTATUS Status;
//
//     /* Check what profile this is */
//     if (ProfileSource == ProfileTime)
//     {
//         /* Return the time interval */
//         Interval = KiProfileTimeInterval;
//     }
//     else if (ProfileSource == ProfileAlignmentFixup)
//     {
//         /* Return the alignment interval */
//         Interval = KiProfileAlignmentFixupInterval;
//     }
//     else
//     {
//         /* Request it from HAL */
//         ProfileSourceInformation.Source = ProfileSource;
//         Status = HalQuerySystemInformation(HalProfileSourceInformation,
//                                            sizeof(HAL_PROFILE_SOURCE_INFORMATION),
//                                            &ProfileSourceInformation,
//                                            &ReturnLength);
//
//         /* Check if HAL handled it and supports this profile */
//         if (NT_SUCCESS(Status) && (ProfileSourceInformation.Supported))
//         {
//             /* Get the interval */
//             Interval = ProfileSourceInformation.Interval;
//         }
//         else
//         {
//             /* Unsupported or invalid source, fail */
//             Interval = 0;
//         }
//     }
//
//     /* Return the interval we got */
//     return Interval;
// }
//
// VOID
// NTAPI
// KeSetIntervalProfile(IN ULONG Interval,
//                      IN KPROFILE_SOURCE ProfileSource)
// {
//     HAL_PROFILE_SOURCE_INTERVAL ProfileSourceInterval;
//
//     /* Check what profile this is */
//     if (ProfileSource == ProfileTime)
//     {
//         /* Set the interval through HAL */
//         KiProfileTimeInterval = (ULONG)HalSetProfileInterval(Interval);
//     }
//     else if (ProfileSource == ProfileAlignmentFixup)
//     {
//         /* Set the alignment interval */
//         KiProfileAlignmentFixupInterval = Interval;
//     }
//     else
//     {
//         /* HAL handles any other interval */
//         ProfileSourceInterval.Source = ProfileSource;
//         ProfileSourceInterval.Interval = Interval;
//         HalSetSystemInformation(HalProfileSourceInterval,
//                                 sizeof(HAL_PROFILE_SOURCE_INTERVAL),
//                                 &ProfileSourceInterval);
//     }
// }
//
// /*
//  * @implemented
//  */
// VOID
// NTAPI
// KeProfileInterrupt(IN PKTRAP_FRAME TrapFrame)
// {
//     /* Called from HAL for Timer Profiling */
//     KeProfileInterruptWithSource(TrapFrame, ProfileTime);
// }
//
// VOID
// NTAPI
// KiParseProfileList(IN PKTRAP_FRAME TrapFrame,
//                    IN KPROFILE_SOURCE Source,
//                    IN PLIST_ENTRY ListHead)
// {
//     PULONG BucketValue;
//     PKPROFILE Profile;
//     PLIST_ENTRY NextEntry;
//     ULONG_PTR ProgramCounter;
//
//     /* Get the Program Counter */
//     ProgramCounter = KeGetTrapFramePc(TrapFrame);
//
//     /* Loop the List */
//     for (NextEntry = ListHead->Flink;
//          NextEntry != ListHead;
//          NextEntry = NextEntry->Flink)
//     {
//         /* Get the entry */
//         Profile = CONTAINING_RECORD(NextEntry, KPROFILE, ProfileListEntry);
//
//         /* Check if the source is good, and if it's within the range */
//         if ((Profile->Source != Source) ||
//             (ProgramCounter < (ULONG_PTR)Profile->RangeBase) ||
//             (ProgramCounter > (ULONG_PTR)Profile->RangeLimit))
//         {
//             continue;
//         }
//
//         /* Get the Pointer to the Bucket Value representing this Program Counter */
//         BucketValue = (PULONG)((ULONG_PTR)Profile->Buffer +
//                                (((ProgramCounter - (ULONG_PTR)Profile->RangeBase)
//                                 >> Profile->BucketShift) &~ 0x3));
//
//         /* Increment the value */
//         (*BucketValue)++;
//     }
// }
//
// /*
//  * @implemented
//  *
//  * Remarks:
//  *         Called from HAL, this function looks up the process
//  *         entries, finds the proper source object, verifies the
//  *         ranges with the trapframe data, and inserts the information
//  *         from the trap frame into the buffer, while using buckets and
//  *         shifting like we specified. -- Alex
//  */
// VOID
// NTAPI
// KeProfileInterruptWithSource(IN PKTRAP_FRAME TrapFrame,
//                              IN KPROFILE_SOURCE Source)
// {
//     PKPROCESS Process = PsGetCurrentThread()->ApcState.Process;
//
//     /* We have to parse 2 lists. Per-Process and System-Wide */
//     KiParseProfileList(TrapFrame, Source, &Process->ProfileListHead);
//     KiParseProfileList(TrapFrame, Source, &KiProfileListHead);
// }
//
// /*
//  * @implemented
//  */
// VOID
// NTAPI
// KeSetProfileIrql(IN KIRQL ProfileIrql)
// {
//     /* Set the IRQL at which Profiling will run */
//     KiProfileIrql = ProfileIrql;
// }
//
// ============================================================================
// END ORIGINAL C SOURCE
// ============================================================================
