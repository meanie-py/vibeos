use ntkrnl_sys::globals::*;
use ntkrnl_sys::constants::*;
use ntkrnl_sys::base::*;
use ntkrnl_sys::ex::*;
use ntkrnl_sys::ke::*;
use ntkrnl_sys::rtl::*;
use ntkrnl_sys::ddk::{CcFlushCache, CcRosEnsureVacbResident, CcRosGetVacb, CcRosMarkDirtyVacb, CcRosReleaseVacb, ERESOURCE_THREAD, ExAcquireResourceExclusiveLite, ExAcquireSharedStarveExclusive, ExDeleteResourceLite, ExInitializeResourceLite, ExReleaseResourceForThreadLite, ExReleaseResourceLite, INTERNAL_BCB, MmMakeSegmentDirty, NPAGED_LOOKASIDE_LIST, PFILE_OBJECT, PINTERNAL_BCB, PIO_STATUS_BLOCK, PLARGE_INTEGER, PROS_SHARED_CACHE_MAP, PROS_VACB, PsGetCurrentThread};
use core::ffi::c_void;

extern "C" {
    fn CCTRACE(level: u32, format: *const u8, ...);
    fn ExFreeToNPagedLookasideList(Lookaside: *mut core::ffi::c_void, Entry: *mut core::ffi::c_void);
    static mut iBcbLookasideList: core::ffi::c_void;
    fn BooleanFlagOn(Flags: u32, Mask: u32) -> u8;
    fn ROUND_DOWN(value: i64, alignment: i64) -> i64;
    static mut CcPinReadWait: u32;
    static mut CcPinReadNoWait: u32;
    static mut CcPinMappedDataCount: u32;
}

unsafe fn CcpFindBcb(
    SharedCacheMap: PROS_SHARED_CACHE_MAP,
    FileOffset: PLARGE_INTEGER,
    Length: u32,
    Pinned: u8,
) -> PINTERNAL_BCB {
    let mut Bcb: PINTERNAL_BCB;
    let mut Found: u8 = 0u8;
    let mut NextEntry: PLIST_ENTRY;

    NextEntry = (*SharedCacheMap).BcbList.Flink;
    while NextEntry != core::ptr::addr_of_mut!((*SharedCacheMap).BcbList) {
        Bcb = (NextEntry as *mut u8).offset(-(core::mem::offset_of!(INTERNAL_BCB, BcbEntry) as isize)) as PINTERNAL_BCB;

        if (*Bcb).PFCB.MappedFileOffset.QuadPart <= (*FileOffset).QuadPart &&
            ((*Bcb).PFCB.MappedFileOffset.QuadPart + (*Bcb).PFCB.MappedLength as i64) >=
            ((*FileOffset).QuadPart + Length as i64)
        {
            if (Pinned != 0 && (*Bcb).PinCount > 0) || (Pinned == 0 && (*Bcb).PinCount == 0)
            {
                Found = 1u8;
                break;
            }
        }

        NextEntry = (*NextEntry).Flink;
    }

    if Found != 0 { Bcb } else { core::ptr::null_mut() }
}

unsafe fn CcpDereferenceBcb(
    SharedCacheMap: PROS_SHARED_CACHE_MAP,
    Bcb: PINTERNAL_BCB,
) {
    let mut RefCount: u32;
    let mut OldIrql: KIRQL = 0;

    KeAcquireSpinLock(core::ptr::addr_of_mut!((*SharedCacheMap).BcbSpinLock), &mut OldIrql);
    (*Bcb).RefCount -= 1;
    RefCount = (*Bcb).RefCount;
    if RefCount == 0 {
        RemoveEntryList(core::ptr::addr_of_mut!((*Bcb).BcbEntry));
        KeReleaseSpinLock(core::ptr::addr_of_mut!((*SharedCacheMap).BcbSpinLock), OldIrql);

        CcRosReleaseVacb(SharedCacheMap,
                         (*Bcb).Vacb,
                         0u8,
                         0u8);

        ExDeleteResourceLite(core::ptr::addr_of_mut!((*Bcb).Lock));
        ExFreeToNPagedLookasideList(core::ptr::addr_of_mut!(iBcbLookasideList), Bcb as *mut core::ffi::c_void);
    } else {
        KeReleaseSpinLock(core::ptr::addr_of_mut!((*SharedCacheMap).BcbSpinLock), OldIrql);
    }
}

unsafe fn CcpGetAppropriateBcb(
    SharedCacheMap: PROS_SHARED_CACHE_MAP,
    Vacb: PROS_VACB,
    FileOffset: PLARGE_INTEGER,
    Length: u32,
    PinFlags: u32,
    ToPin: u8,
) -> *mut core::ffi::c_void {
    let mut OldIrql: KIRQL = 0;
    let mut Result: u8;
    let mut iBcb: PINTERNAL_BCB;
    let mut DupBcb: PINTERNAL_BCB;

    iBcb = ExAllocateFromNPagedLookasideList(core::ptr::addr_of_mut!(iBcbLookasideList));
    if iBcb.is_null() {
        CcRosReleaseVacb(SharedCacheMap, Vacb, 0u8, 0u8);
        return core::ptr::null_mut();
    }

    RtlZeroMemory(iBcb as *mut core::ffi::c_void, core::mem::size_of::<INTERNAL_BCB>() as u32);
    (*iBcb).PFCB.NodeTypeCode = 0x2FD;
    (*iBcb).PFCB.NodeByteSize = 0;
    (*iBcb).PFCB.MappedLength = Length;
    (*iBcb).PFCB.MappedFileOffset = *FileOffset;
    (*iBcb).Vacb = Vacb;
    (*iBcb).PinCount = 0;
    (*iBcb).RefCount = 1;
    ExInitializeResourceLite(core::ptr::addr_of_mut!((*iBcb).Lock));

    KeAcquireSpinLock(core::ptr::addr_of_mut!((*SharedCacheMap).BcbSpinLock), &mut OldIrql);

    DupBcb = CcpFindBcb(SharedCacheMap, FileOffset, Length, ToPin);
    if !DupBcb.is_null() {
        (*DupBcb).RefCount += 1;
        Result = 1u8;
        KeReleaseSpinLock(core::ptr::addr_of_mut!((*SharedCacheMap).BcbSpinLock), OldIrql);

        if ToPin != 0 {
            if BooleanFlagOn(PinFlags, PIN_EXCLUSIVE) != 0 {
                Result = ExAcquireResourceExclusiveLite(core::ptr::addr_of_mut!((*iBcb).Lock), BooleanFlagOn(PinFlags, PIN_WAIT));
            } else {
                Result = ExAcquireSharedStarveExclusive(core::ptr::addr_of_mut!((*iBcb).Lock), BooleanFlagOn(PinFlags, PIN_WAIT));
            }

            if Result != 0 {
                (*DupBcb).PinCount += 1;
            } else {
                CcpDereferenceBcb(SharedCacheMap, DupBcb);
                DupBcb = core::ptr::null_mut();
            }
        }

        if !DupBcb.is_null() {
            CcRosReleaseVacb(SharedCacheMap, Vacb, 0u8, 0u8);
            ExDeleteResourceLite(core::ptr::addr_of_mut!((*iBcb).Lock));
            ExFreeToNPagedLookasideList(core::ptr::addr_of_mut!(iBcbLookasideList), iBcb as *mut core::ffi::c_void);
        }

        iBcb = DupBcb;
    } else {
        if ToPin != 0 {
            (*iBcb).PinCount += 1;

            if BooleanFlagOn(PinFlags, PIN_EXCLUSIVE) != 0 {
                Result = ExAcquireResourceExclusiveLite(core::ptr::addr_of_mut!((*iBcb).Lock), BooleanFlagOn(PinFlags, PIN_WAIT));
            } else {
                Result = ExAcquireSharedStarveExclusive(core::ptr::addr_of_mut!((*iBcb).Lock), BooleanFlagOn(PinFlags, PIN_WAIT));
            }
        }

        InsertTailList(core::ptr::addr_of_mut!((*SharedCacheMap).BcbList), core::ptr::addr_of_mut!((*iBcb).BcbEntry));
        KeReleaseSpinLock(core::ptr::addr_of_mut!((*SharedCacheMap).BcbSpinLock), OldIrql);
    }

    iBcb as *mut core::ffi::c_void
}

unsafe fn CcpPinData(
    SharedCacheMap: PROS_SHARED_CACHE_MAP,
    FileOffset: PLARGE_INTEGER,
    Length: u32,
    Flags: u32,
    Bcb: *mut *mut core::ffi::c_void,
    Buffer: *mut *mut core::ffi::c_void,
) -> u8 {

    const CC_API_DEBUG: u32 = 0;
    const VACB_MAPPING_GRANULARITY: i64 = 262144;
    const PIN_EXCLUSIVE: u32 = 0x01;
    const PIN_WAIT: u32 = 0x02;
    const PIN_IF_BCB: u32 = 0x04;
    const PIN_NO_READ: u32 = 0x10;

    let mut NewBcb: PINTERNAL_BCB;
    let mut OldIrql: KIRQL = 0;
    let VacbOffset: u32;
    let mut Status: i32;
    let mut Result: u8;
    let mut Length = Length;

    VacbOffset = ((*FileOffset).QuadPart % VACB_MAPPING_GRANULARITY) as u32;

    if (VacbOffset + Length) > VACB_MAPPING_GRANULARITY as u32 {
        Length = VACB_MAPPING_GRANULARITY as u32 - VacbOffset;
    }

    KeAcquireSpinLock(core::ptr::addr_of_mut!((*SharedCacheMap).BcbSpinLock), &mut OldIrql);
    NewBcb = CcpFindBcb(SharedCacheMap, FileOffset, Length, 1u8);

    if !NewBcb.is_null() {
        let Result: u8;

        (*NewBcb).RefCount += 1;
        KeReleaseSpinLock(core::ptr::addr_of_mut!((*SharedCacheMap).BcbSpinLock), OldIrql);

        if BooleanFlagOn(Flags, PIN_EXCLUSIVE) != 0 {
            Result = ExAcquireResourceExclusiveLite(core::ptr::addr_of_mut!((*NewBcb).Lock), BooleanFlagOn(Flags, PIN_WAIT));
        } else {
            Result = ExAcquireSharedStarveExclusive(core::ptr::addr_of_mut!((*NewBcb).Lock), BooleanFlagOn(Flags, PIN_WAIT));
        }

        if Result == 0 {
            CcpDereferenceBcb(SharedCacheMap, NewBcb);
            return 0u8;
        }

        (*NewBcb).PinCount += 1;
    } else {
        let ROffset: i64;
        let mut Vacb: PROS_VACB = core::ptr::null_mut();

        KeReleaseSpinLock(core::ptr::addr_of_mut!((*SharedCacheMap).BcbSpinLock), OldIrql);

        if BooleanFlagOn(Flags, PIN_IF_BCB) != 0 {
            return 0u8;
        }

        ROffset = ROUND_DOWN((*FileOffset).QuadPart, VACB_MAPPING_GRANULARITY);
        Status = CcRosGetVacb(SharedCacheMap, ROffset, &mut Vacb);
        if !NT_SUCCESS(Status) {
            CCTRACE(CC_API_DEBUG, b"FileObject=%p FileOffset=%p Length=%lu Flags=0x%lx -> FALSE\n\0".as_ptr(),
                (*SharedCacheMap).FileObject, FileOffset, Length, Flags);
            ExRaiseStatus(Status as u32);
            return 0u8;
        }

        NewBcb = CcpGetAppropriateBcb(SharedCacheMap, Vacb, FileOffset, Length, Flags, 1u8);
        if NewBcb.is_null() {
            CcRosReleaseVacb(SharedCacheMap, Vacb, 0u8, 0u8);
            return 0u8;
        }
    }

    Result = 0u8;
    Result = CcRosEnsureVacbResident((*NewBcb).Vacb,
            BooleanFlagOn(Flags, PIN_WAIT),
            BooleanFlagOn(Flags, PIN_NO_READ),
            VacbOffset, Length);

    Result
}

#[no_mangle]
pub unsafe extern "system" fn CcMapData(
    FileObject: PFILE_OBJECT,
    FileOffset: PLARGE_INTEGER,
    Length: u32,
    Flags: u32,
    pBcb: *mut *mut core::ffi::c_void,
    pBuffer: *mut *mut core::ffi::c_void,
) -> u8 {
    let mut OldIrql: KIRQL = 0;
    let mut iBcb: PINTERNAL_BCB;
    let mut Vacb: PROS_VACB;
    let SharedCacheMap: PROS_SHARED_CACHE_MAP;
    let VacbOffset: u32;
    let mut Status: i32;
    let mut Result: u8;

    let SharedCacheMap = (*(*FileObject).SectionObjectPointer).SharedCacheMap;

    if (Flags & 0x00000001) != 0 {
        CcMapDataWait = CcMapDataWait.wrapping_add(1);
    } else {
        CcMapDataNoWait = CcMapDataNoWait.wrapping_add(1);
    }

    let mut VacbOffset = ((*FileOffset).QuadPart % 262144) as u32;
    let mut Length = Length;
    if (VacbOffset + Length) > 262144 {
        Length = 262144 - VacbOffset;
    }

    KeAcquireSpinLock(core::ptr::addr_of_mut!((*SharedCacheMap).BcbSpinLock), core::ptr::addr_of_mut!(OldIrql));
    iBcb = CcpFindBcb(SharedCacheMap, FileOffset, Length, 0u8);

    if iBcb.is_null() {
        KeReleaseSpinLock(core::ptr::addr_of_mut!((*SharedCacheMap).BcbSpinLock), OldIrql);

        Status = CcRosGetVacb(SharedCacheMap, (*FileOffset).QuadPart, core::ptr::addr_of_mut!(Vacb));
        if !NT_SUCCESS(Status) {
            ExRaiseStatus(Status);
        }

        iBcb = CcpGetAppropriateBcb(SharedCacheMap, Vacb, FileOffset, Length, 0, 0u8);
        if iBcb.is_null() {
            CcRosReleaseVacb(SharedCacheMap, Vacb, 0u8, 0u8);
            *pBcb = core::ptr::null_mut();
            ExRaiseStatus(STATUS_INSUFFICIENT_RESOURCES);
        }
    } else {
        (*iBcb).RefCount = (*iBcb).RefCount.wrapping_add(1);
        KeReleaseSpinLock(core::ptr::addr_of_mut!((*SharedCacheMap).BcbSpinLock), OldIrql);
    }

    Result = 0u8;
    Result = CcRosEnsureVacbResident(
        (*iBcb).Vacb,
        if (Flags & 0x00000001) != 0 { 1u8 } else { 0u8 },
        if (Flags & 0x00000002) != 0 { 1u8 } else { 0u8 },
        VacbOffset,
        Length,
    );

    if Result != 0 {
        *pBcb = iBcb as *mut core::ffi::c_void;
        *pBuffer = ((*(*iBcb).Vacb).BaseAddress as usize + VacbOffset as usize) as *mut core::ffi::c_void;
        (*iBcb).Dirty = if (Flags & 0x00000004) != 0 { 1u8 } else { (*iBcb).Dirty };
    } else {
        CcRosReleaseVacb(SharedCacheMap, (*iBcb).Vacb, 0u8, 0u8);
    }

    Result
}

#[no_mangle]
pub unsafe extern "system" fn CcPinMappedData(
    FileObject: PFILE_OBJECT,
    FileOffset: PLARGE_INTEGER,
    Length: u32,
    Flags: u32,
    Bcb: *mut *mut core::ffi::c_void,
) -> u8 {

    const CC_API_DEBUG: u32 = 0;

    let mut Buffer: *mut core::ffi::c_void = core::ptr::null_mut();
    let iBcb: PINTERNAL_BCB;
    let SharedCacheMap: PROS_SHARED_CACHE_MAP;

    CCTRACE(
        CC_API_DEBUG,
        b"FileObject=%p FileOffset=%p Length=%lu Flags=0x%lx\n\0".as_ptr() as *const u8,
        FileObject,
        FileOffset,
        Length,
        Flags,
    );

    SharedCacheMap = (*(*FileObject).SectionObjectPointer).SharedCacheMap;
    if (*SharedCacheMap).PinAccess == 0 {
        return 0u8;
    }

    iBcb = if !(*Bcb).is_null() {
        ((*Bcb) as *mut u8).offset(-(core::mem::offset_of!(INTERNAL_BCB, PFCB) as isize)) as PINTERNAL_BCB
    } else {
        core::ptr::null_mut()
    };

    CcPinMappedDataCount = CcPinMappedDataCount.wrapping_add(1);

    let Result = CcpPinData(SharedCacheMap, FileOffset, Length, Flags, Bcb, &mut Buffer);
    if Result != 0 {
        CcUnpinData(core::ptr::addr_of_mut!((*iBcb).PFCB) as *mut core::ffi::c_void);
    }

    Result
}

#[no_mangle]
pub unsafe extern "system" fn CcPinRead(
    FileObject: PFILE_OBJECT,
    FileOffset: PLARGE_INTEGER,
    Length: u32,
    Flags: u32,
    Bcb: *mut *mut core::ffi::c_void,
    Buffer: *mut *mut core::ffi::c_void,
) -> u8 {

    const CC_API_DEBUG: u32 = 0;
    const PIN_WAIT: u32 = 1;

    CCTRACE(
        CC_API_DEBUG,
        b"FileOffset=%p FileOffset=%p Length=%lu Flags=0x%lx\n\0".as_ptr(),
        FileObject,
        FileOffset,
        Length,
        Flags,
    );

    let SharedCacheMap = (*(*FileObject).SectionObjectPointer).SharedCacheMap;

    if (*SharedCacheMap).PinAccess == 0 {
        return 0u8;
    }

    if (Flags & PIN_WAIT) != 0 {
        CcPinReadWait += 1;
    } else {
        CcPinReadNoWait += 1;
    }

    CcpPinData(SharedCacheMap, FileOffset, Length, Flags, Bcb, Buffer)
}

#[no_mangle]
pub unsafe extern "system" fn CcPreparePinWrite(
    FileObject: PFILE_OBJECT,
    FileOffset: PLARGE_INTEGER,
    Length: u32,
    Zero: u8,
    Flags: u32,
    Bcb: *mut *mut core::ffi::c_void,
    Buffer: *mut *mut core::ffi::c_void,
) -> u8 {

    CCTRACE(
        0x80,
        b"FileOffset=%p FileOffset=%p Length=%lu Zero=%d Flags=0x%lx\n\0".as_ptr(),
        FileObject,
        FileOffset,
        Length,
        Zero,
        Flags,
    );

    CcPinRead(FileObject, FileOffset, Length, Flags, Bcb, Buffer)
}

#[no_mangle]
pub unsafe extern "system" fn CcSetDirtyPinnedData(
    Bcb: *mut core::ffi::c_void,
    Lsn: PLARGE_INTEGER,
) {
    let iBcb = (Bcb as *mut u8).offset(-(core::mem::offset_of!(INTERNAL_BCB, PFCB) as isize)) as PINTERNAL_BCB;
    let Vacb = (*iBcb).Vacb;

    CCTRACE(CC_API_DEBUG, b"Bcb=%p Lsn=%p\n\0".as_ptr() as *const i8, Bcb, Lsn);

    MmMakeSegmentDirty(
        (*(*Vacb).SharedCacheMap).FileObject.SectionObjectPointer,
        (*iBcb).PFCB.MappedFileOffset.QuadPart,
        (*iBcb).PFCB.MappedLength,
    );

    if (*Vacb).Dirty == 0 {
        CcRosMarkDirtyVacb(Vacb);
    }
}

#[no_mangle]
pub unsafe extern "system" fn CcUnpinData(Bcb: PVOID) {
    CCTRACE(CC_API_DEBUG, b"Bcb=%p\n\0".as_ptr() as PCHAR, Bcb);

    CcUnpinDataForThread(Bcb, PsGetCurrentThread() as ERESOURCE_THREAD);
}

#[no_mangle]
pub unsafe extern "system" fn CcUnpinDataForThread(
    Bcb: PVOID,
    ResourceThreadId: ERESOURCE_THREAD,
) {
    let iBcb: PINTERNAL_BCB = (Bcb as *mut u8).offset(-(core::mem::offset_of!(_INTERNAL_BCB, PFCB) as isize)) as PINTERNAL_BCB;

    CCTRACE(CC_API_DEBUG, b"Bcb=%p ResourceThreadId=%lu\n\0".as_ptr() as PCHAR, Bcb, ResourceThreadId);

    if (*iBcb).PinCount != 0 {
        ExReleaseResourceForThreadLite(core::ptr::addr_of_mut!((*iBcb).Lock), ResourceThreadId);
        (*iBcb).PinCount -= 1;
    }

    CcpDereferenceBcb((*(*iBcb).Vacb).SharedCacheMap, iBcb);
}

#[no_mangle]
pub unsafe extern "system" fn CcRepinBcb(
    Bcb: PVOID
) {
    let iBcb: PINTERNAL_BCB = (Bcb as *mut u8).offset(-(core::mem::offset_of!(INTERNAL_BCB, PFCB) as isize)) as PINTERNAL_BCB;

    CCTRACE(0x00000001u32, b"Bcb=%p\n\0".as_ptr() as *const i8, Bcb);

    (*iBcb).RefCount += 1;
}

#[no_mangle]
pub unsafe extern "system" fn CcUnpinRepinnedBcb(
    Bcb: *mut core::ffi::c_void,
    WriteThrough: u8,
    IoStatus: PIO_STATUS_BLOCK,
) {
    let iBcb = (Bcb as *mut u8).offset(-(core::mem::offset_of!(INTERNAL_BCB, PFCB) as isize)) as PINTERNAL_BCB;
    let mut OldIrql: KIRQL = 0;
    let SharedCacheMap: PROS_SHARED_CACHE_MAP;

    CCTRACE(0x80, b"Bcb=%p WriteThrough=%d\n\0".as_ptr(), Bcb, WriteThrough as i32);

    SharedCacheMap = (*(*iBcb).Vacb).SharedCacheMap;
    (*IoStatus).Anonymous.__bindgen_anon_1.Status = STATUS_SUCCESS;

    if WriteThrough != 0 {
        CcFlushCache(
            (*(*(*(*iBcb).Vacb).SharedCacheMap).FileObject).SectionObjectPointer,
            core::ptr::addr_of_mut!((*iBcb).PFCB.MappedFileOffset),
            (*iBcb).PFCB.MappedLength,
            IoStatus,
        );
    } else {
        (*IoStatus).Anonymous.__bindgen_anon_1.Status = STATUS_SUCCESS;
        (*IoStatus).Information = 0;
    }

    KeAcquireSpinLock(core::ptr::addr_of_mut!((*SharedCacheMap).BcbSpinLock), core::ptr::addr_of_mut!(OldIrql));
    (*iBcb).RefCount -= 1;
    if (*iBcb).RefCount == 0 {
        RemoveEntryList(core::ptr::addr_of_mut!((*iBcb).BcbEntry));
        KeReleaseSpinLock(core::ptr::addr_of_mut!((*SharedCacheMap).BcbSpinLock), OldIrql);

        if (*iBcb).PinCount != 0 {
            ExReleaseResourceLite(core::ptr::addr_of_mut!((*iBcb).Lock));
            (*iBcb).PinCount -= 1;
        }

        CcRosReleaseVacb(
            (*(*iBcb).Vacb).SharedCacheMap,
            (*iBcb).Vacb,
            0u8,
            0u8,
        );

        ExDeleteResourceLite(core::ptr::addr_of_mut!((*iBcb).Lock));
        ExFreeToNPagedLookasideList(core::ptr::addr_of_mut!(iBcbLookasideList), iBcb as *mut core::ffi::c_void);
    } else {
        KeReleaseSpinLock(core::ptr::addr_of_mut!((*SharedCacheMap).BcbSpinLock), OldIrql);
    }
}

// ============================================================================
// ORIGINAL C SOURCE
// ============================================================================
//
// /*
//  * COPYRIGHT:       See COPYING in the top level directory
//  * PROJECT:         ReactOS kernel
//  * FILE:            ntoskrnl/cc/pin.c
//  * PURPOSE:         Implements cache managers pinning interface
//  *
//  * PROGRAMMERS:     ?
//                     Pierre Schweitzer (pierre@reactos.org)
//  */
//
// /* INCLUDES ******************************************************************/
//
// #include <ntoskrnl.h>
// #define NDEBUG
// #include <debug.h>
//
// /* GLOBALS *******************************************************************/
//
// extern NPAGED_LOOKASIDE_LIST iBcbLookasideList;
//
// /* Counters:
//  * - Number of calls to CcMapData that could wait
//  * - Number of calls to CcMapData that couldn't wait
//  * - Number of calls to CcPinRead that could wait
//  * - Number of calls to CcPinRead that couldn't wait
//  * - Number of calls to CcPinMappedDataCount
//  */
// ULONG CcMapDataWait = 0;
// ULONG CcMapDataNoWait = 0;
// ULONG CcPinReadWait = 0;
// ULONG CcPinReadNoWait = 0;
// ULONG CcPinMappedDataCount = 0;
//
// /* FUNCTIONS *****************************************************************/
//
// static
// PINTERNAL_BCB
// NTAPI
// CcpFindBcb(
//     IN PROS_SHARED_CACHE_MAP SharedCacheMap,
//     IN PLARGE_INTEGER FileOffset,
//     IN ULONG Length,
//     IN BOOLEAN Pinned)
// {
//     PINTERNAL_BCB Bcb;
//     BOOLEAN Found = FALSE;
//     PLIST_ENTRY NextEntry;
//
//     for (NextEntry = SharedCacheMap->BcbList.Flink;
//          NextEntry != &SharedCacheMap->BcbList;
//          NextEntry = NextEntry->Flink)
//     {
//         Bcb = CONTAINING_RECORD(NextEntry, INTERNAL_BCB, BcbEntry);
//
//         if (Bcb->PFCB.MappedFileOffset.QuadPart <= FileOffset->QuadPart &&
//             (Bcb->PFCB.MappedFileOffset.QuadPart + Bcb->PFCB.MappedLength) >=
//             (FileOffset->QuadPart + Length))
//         {
//             if ((Pinned && Bcb->PinCount > 0) || (!Pinned && Bcb->PinCount == 0))
//             {
//                 Found = TRUE;
//                 break;
//             }
//         }
//     }
//
//     return (Found ? Bcb : NULL);
// }
//
// static
// VOID
// CcpDereferenceBcb(
//     IN PROS_SHARED_CACHE_MAP SharedCacheMap,
//     IN PINTERNAL_BCB Bcb)
// {
//     ULONG RefCount;
//     KIRQL OldIrql;
//
//     KeAcquireSpinLock(&SharedCacheMap->BcbSpinLock, &OldIrql);
//     RefCount = --Bcb->RefCount;
//     if (RefCount == 0)
//     {
//         RemoveEntryList(&Bcb->BcbEntry);
//         KeReleaseSpinLock(&SharedCacheMap->BcbSpinLock, OldIrql);
//
//         ASSERT(Bcb->PinCount == 0);
//         /*
//          * Don't mark dirty, if it was dirty,
//          * the VACB was already marked as such
//          * following the call to CcSetDirtyPinnedData
//          */
//         CcRosReleaseVacb(SharedCacheMap,
//                          Bcb->Vacb,
//                          FALSE,
//                          FALSE);
//
//         ExDeleteResourceLite(&Bcb->Lock);
//         ExFreeToNPagedLookasideList(&iBcbLookasideList, Bcb);
//     }
//     else
//     {
//         KeReleaseSpinLock(&SharedCacheMap->BcbSpinLock, OldIrql);
//     }
// }
//
// static
// PVOID
// CcpGetAppropriateBcb(
//     IN PROS_SHARED_CACHE_MAP SharedCacheMap,
//     IN PROS_VACB Vacb,
//     IN PLARGE_INTEGER FileOffset,
//     IN ULONG Length,
//     IN ULONG PinFlags,
//     IN BOOLEAN ToPin)
// {
//     KIRQL OldIrql;
//     BOOLEAN Result;
//     PINTERNAL_BCB iBcb, DupBcb;
//
//     iBcb = ExAllocateFromNPagedLookasideList(&iBcbLookasideList);
//     if (iBcb == NULL)
//     {
//         CcRosReleaseVacb(SharedCacheMap, Vacb, FALSE, FALSE);
//         return NULL;
//     }
//
//     RtlZeroMemory(iBcb, sizeof(*iBcb));
//     iBcb->PFCB.NodeTypeCode = 0x2FD; /* As per KMTests */
//     iBcb->PFCB.NodeByteSize = 0;
//     iBcb->PFCB.MappedLength = Length;
//     iBcb->PFCB.MappedFileOffset = *FileOffset;
//     iBcb->Vacb = Vacb;
//     iBcb->PinCount = 0;
//     iBcb->RefCount = 1;
//     ExInitializeResourceLite(&iBcb->Lock);
//
//     KeAcquireSpinLock(&SharedCacheMap->BcbSpinLock, &OldIrql);
//
//     /* Check if we raced with another BCB creation */
//     DupBcb = CcpFindBcb(SharedCacheMap, FileOffset, Length, ToPin);
//     /* Yes, and we've lost */
//     if (DupBcb != NULL)
//     {
//         /* We will return that BCB */
//         ++DupBcb->RefCount;
//         Result = TRUE;
//         KeReleaseSpinLock(&SharedCacheMap->BcbSpinLock, OldIrql);
//
//         if (ToPin)
//         {
//             if (BooleanFlagOn(PinFlags, PIN_EXCLUSIVE))
//             {
//                 Result = ExAcquireResourceExclusiveLite(&iBcb->Lock, BooleanFlagOn(PinFlags, PIN_WAIT));
//             }
//             else
//             {
//                 Result = ExAcquireSharedStarveExclusive(&iBcb->Lock, BooleanFlagOn(PinFlags, PIN_WAIT));
//             }
//
//             if (Result)
//             {
//                 DupBcb->PinCount++;
//             }
//             else
//             {
//                 CcpDereferenceBcb(SharedCacheMap, DupBcb);
//                 DupBcb = NULL;
//             }
//         }
//
//         if (DupBcb != NULL)
//         {
//             /* Delete the loser */
//             CcRosReleaseVacb(SharedCacheMap, Vacb, FALSE, FALSE);
//             ExDeleteResourceLite(&iBcb->Lock);
//             ExFreeToNPagedLookasideList(&iBcbLookasideList, iBcb);
//         }
//
//         /* Return the winner - no need to update buffer address, it's
//          * relative to the VACB, which is unchanged.
//          */
//         iBcb = DupBcb;
//     }
//     /* Nope, insert ourselves */
//     else
//     {
//         if (ToPin)
//         {
//             iBcb->PinCount++;
//
//             if (BooleanFlagOn(PinFlags, PIN_EXCLUSIVE))
//             {
//                 Result = ExAcquireResourceExclusiveLite(&iBcb->Lock, BooleanFlagOn(PinFlags, PIN_WAIT));
//             }
//             else
//             {
//                 Result = ExAcquireSharedStarveExclusive(&iBcb->Lock, BooleanFlagOn(PinFlags, PIN_WAIT));
//             }
//
//             ASSERT(Result);
//         }
//
//         InsertTailList(&SharedCacheMap->BcbList, &iBcb->BcbEntry);
//         KeReleaseSpinLock(&SharedCacheMap->BcbSpinLock, OldIrql);
//     }
//
//     return iBcb;
// }
//
// static
// BOOLEAN
// CcpPinData(
//     IN PROS_SHARED_CACHE_MAP SharedCacheMap,
//     IN PLARGE_INTEGER FileOffset,
//     IN ULONG Length,
//     IN ULONG Flags,
//     OUT	PVOID * Bcb,
//     OUT	PVOID * Buffer)
// {
//     PINTERNAL_BCB NewBcb;
//     KIRQL OldIrql;
//     ULONG VacbOffset;
//     NTSTATUS Status;
//     _SEH2_VOLATILE BOOLEAN Result;
//
//     VacbOffset = (ULONG)(FileOffset->QuadPart % VACB_MAPPING_GRANULARITY);
//
//     if ((VacbOffset + Length) > VACB_MAPPING_GRANULARITY)
//     {
//         /* Complain loudly, we shoud pin the whole range */
//         DPRINT1("TRUNCATING DATA PIN FROM %lu to %lu!\n", Length, VACB_MAPPING_GRANULARITY - VacbOffset);
//         Length = VACB_MAPPING_GRANULARITY - VacbOffset;
//     }
//
//     KeAcquireSpinLock(&SharedCacheMap->BcbSpinLock, &OldIrql);
//     NewBcb = CcpFindBcb(SharedCacheMap, FileOffset, Length, TRUE);
//
//     if (NewBcb != NULL)
//     {
//         BOOLEAN Result;
//
//         ++NewBcb->RefCount;
//         KeReleaseSpinLock(&SharedCacheMap->BcbSpinLock, OldIrql);
//
//         if (BooleanFlagOn(Flags, PIN_EXCLUSIVE))
//             Result = ExAcquireResourceExclusiveLite(&NewBcb->Lock, BooleanFlagOn(Flags, PIN_WAIT));
//         else
//             Result = ExAcquireSharedStarveExclusive(&NewBcb->Lock, BooleanFlagOn(Flags, PIN_WAIT));
//
//         if (!Result)
//         {
//             CcpDereferenceBcb(SharedCacheMap, NewBcb);
//             return FALSE;
//         }
//
//         NewBcb->PinCount++;
//     }
//     else
//     {
//         LONGLONG ROffset;
//         PROS_VACB Vacb;
//
//         KeReleaseSpinLock(&SharedCacheMap->BcbSpinLock, OldIrql);
//
//         if (BooleanFlagOn(Flags, PIN_IF_BCB))
//         {
//             return FALSE;
//         }
//
//         /* Properly round offset and call internal helper for getting a VACB */
//         ROffset = ROUND_DOWN(FileOffset->QuadPart, VACB_MAPPING_GRANULARITY);
//         Status = CcRosGetVacb(SharedCacheMap, ROffset, &Vacb);
//         if (!NT_SUCCESS(Status))
//         {
//             CCTRACE(CC_API_DEBUG, "FileObject=%p FileOffset=%p Length=%lu Flags=0x%lx -> FALSE\n",
//                 SharedCacheMap->FileObject, FileOffset, Length, Flags);
//             ExRaiseStatus(Status);
//             return FALSE;
//         }
//
//         NewBcb = CcpGetAppropriateBcb(SharedCacheMap, Vacb, FileOffset, Length, Flags, TRUE);
//         if (NewBcb == NULL)
//         {
//             CcRosReleaseVacb(SharedCacheMap, Vacb, FALSE, FALSE);
//             return FALSE;
//         }
//     }
//
//     Result = FALSE;
//     _SEH2_TRY
//     {
//         /* Ensure the pages are resident */
//         Result = CcRosEnsureVacbResident(NewBcb->Vacb,
//                 BooleanFlagOn(Flags, PIN_WAIT),
//                 BooleanFlagOn(Flags, PIN_NO_READ),
//                 VacbOffset, Length);
//     }
//     _SEH2_FINALLY
//     {
//         if (!Result)
//         {
//             CCTRACE(CC_API_DEBUG, "FileObject=%p FileOffset=%p Length=%lu Flags=0x%lx -> FALSE\n",
//                             SharedCacheMap->FileObject, FileOffset, Length, Flags);
//             CcUnpinData(&NewBcb->PFCB);
//             *Bcb = NULL;
//             *Buffer = NULL;
//         }
//     }
//     _SEH2_END;
//
//     if (Result)
//     {
//         *Bcb = &NewBcb->PFCB;
//         *Buffer = (PVOID)((ULONG_PTR)NewBcb->Vacb->BaseAddress + VacbOffset);
//     }
//
//     return Result;
// }
//
// /*
//  * @implemented
//  */
// BOOLEAN
// NTAPI
// CcMapData (
//     IN PFILE_OBJECT FileObject,
//     IN PLARGE_INTEGER FileOffset,
//     IN ULONG Length,
//     IN ULONG Flags,
//     OUT PVOID *pBcb,
//     OUT PVOID *pBuffer)
// {
//     KIRQL OldIrql;
//     PINTERNAL_BCB iBcb;
//     PROS_VACB Vacb;
//     PROS_SHARED_CACHE_MAP SharedCacheMap;
//     ULONG VacbOffset;
//     NTSTATUS Status;
//     _SEH2_VOLATILE BOOLEAN Result;
//
//     CCTRACE(CC_API_DEBUG, "CcMapData(FileObject 0x%p, FileOffset 0x%I64x, Length %lu, Flags 0x%lx,"
//            " pBcb 0x%p, pBuffer 0x%p)\n", FileObject, FileOffset->QuadPart,
//            Length, Flags, pBcb, pBuffer);
//
//     ASSERT(FileObject);
//     ASSERT(FileObject->SectionObjectPointer);
//     ASSERT(FileObject->SectionObjectPointer->SharedCacheMap);
//
//     SharedCacheMap = FileObject->SectionObjectPointer->SharedCacheMap;
//     ASSERT(SharedCacheMap);
//
//     if (Flags & MAP_WAIT)
//     {
//         ++CcMapDataWait;
//     }
//     else
//     {
//         ++CcMapDataNoWait;
//     }
//
//     VacbOffset = (ULONG)(FileOffset->QuadPart % VACB_MAPPING_GRANULARITY);
//     /* KMTests seem to show that it is allowed to call accross mapping granularity */
//     if ((VacbOffset + Length) > VACB_MAPPING_GRANULARITY)
//     {
//         DPRINT1("TRUNCATING DATA MAP FROM %lu to %lu!\n", Length, VACB_MAPPING_GRANULARITY - VacbOffset);
//         Length = VACB_MAPPING_GRANULARITY - VacbOffset;
//     }
//
//     ARM64_UART_TRACE("[CC] CcMapData: AcquireSpinLock\r\n");
//     KeAcquireSpinLock(&SharedCacheMap->BcbSpinLock, &OldIrql);
//     iBcb = CcpFindBcb(SharedCacheMap, FileOffset, Length, FALSE);
//     ARM64_UART_TRACE("[CC] CcMapData: FindBcb done\r\n");
//
//     if (iBcb == NULL)
//     {
//         KeReleaseSpinLock(&SharedCacheMap->BcbSpinLock, OldIrql);
// #ifdef _M_ARM64
//         ARM64_BARRIER();
// #endif
//         /* Call internal helper for getting a VACB */
//         ARM64_UART_TRACE("[CC] CcMapData: CcRosGetVacb\r\n");
//         Status = CcRosGetVacb(SharedCacheMap, FileOffset->QuadPart, &Vacb);
//         if (!NT_SUCCESS(Status))
//         {
//             CCTRACE(CC_API_DEBUG, "FileObject=%p FileOffset=%p Length=%lu Flags=0x%lx -> FALSE\n",
//                 SharedCacheMap->FileObject, FileOffset, Length, Flags);
//             ExRaiseStatus(Status);
//         }
//         ARM64_UART_TRACE("[CC] CcMapData: CcRosGetVacb done\r\n");
//
//         ARM64_UART_TRACE("[CC] CcMapData: GetBcb\r\n");
//         iBcb = CcpGetAppropriateBcb(SharedCacheMap, Vacb, FileOffset, Length, 0, FALSE);
//         if (iBcb == NULL)
//         {
//             CcRosReleaseVacb(SharedCacheMap, Vacb, FALSE, FALSE);
//             CCTRACE(CC_API_DEBUG, "FileObject=%p FileOffset=%p Length=%lu Flags=0x%lx -> FALSE\n",
//                 SharedCacheMap->FileObject, FileOffset, Length, Flags);
//             *pBcb = NULL; // If you ever remove this for compat, make sure to review all callers for using an unititialized value
//             ExRaiseStatus(STATUS_INSUFFICIENT_RESOURCES);
//         }
//         ARM64_UART_TRACE("[CC] CcMapData: GetBcb done\r\n");
//     }
//     else
//     {
//         ARM64_UART_TRACE("[CC] CcMapData: BCB found\r\n");
//         ++iBcb->RefCount;
//         KeReleaseSpinLock(&SharedCacheMap->BcbSpinLock, OldIrql);
//     }
//
//     _SEH2_TRY
//     {
//         Result = FALSE;
//         /* Ensure the pages are resident */
//         ARM64_UART_TRACE("[CC] CcMapData: EnsureVacbResident\r\n");
//         Result = CcRosEnsureVacbResident(iBcb->Vacb, BooleanFlagOn(Flags, MAP_WAIT),
//                 BooleanFlagOn(Flags, MAP_NO_READ), VacbOffset, Length);
//         ARM64_UART_TRACE("[CC] CcMapData: EnsureVacbResident done\r\n");
//     }
//     _SEH2_FINALLY
//     {
//         if (!Result)
//         {
//             CcpDereferenceBcb(SharedCacheMap, iBcb);
//             *pBcb = NULL;
//             *pBuffer = NULL;
//         }
//     }
//     _SEH2_END;
//
//     if (Result)
//     {
//         *pBcb = &iBcb->PFCB;
//         *pBuffer = (PVOID)((ULONG_PTR)iBcb->Vacb->BaseAddress + VacbOffset);
//         ARM64_UART_TRACE("[CC] CcMapData: returning TRUE\r\n");
//     }
//
//     CCTRACE(CC_API_DEBUG, "FileObject=%p FileOffset=%p Length=%lu Flags=0x%lx -> TRUE Bcb=%p, Buffer %p\n",
//         FileObject, FileOffset, Length, Flags, *pBcb, *pBuffer);
//     return Result;
// }
//
// /*
//  * @unimplemented
//  */
// BOOLEAN
// NTAPI
// CcPinMappedData (
//     IN	PFILE_OBJECT FileObject,
//     IN	PLARGE_INTEGER FileOffset,
//     IN	ULONG Length,
//     IN	ULONG Flags,
//     OUT	PVOID * Bcb)
// {
//     BOOLEAN Result;
//     PVOID Buffer;
//     PINTERNAL_BCB iBcb;
//     PROS_SHARED_CACHE_MAP SharedCacheMap;
//
//     CCTRACE(CC_API_DEBUG, "FileObject=%p FileOffset=%p Length=%lu Flags=0x%lx\n",
//         FileObject, FileOffset, Length, Flags);
//
//     ASSERT(FileObject);
//     ASSERT(FileObject->SectionObjectPointer);
//     ASSERT(FileObject->SectionObjectPointer->SharedCacheMap);
//
//     SharedCacheMap = FileObject->SectionObjectPointer->SharedCacheMap;
//     ASSERT(SharedCacheMap);
//     if (!SharedCacheMap->PinAccess)
//     {
//         DPRINT1("FIXME: Pinning a file with no pin access!\n");
//         return FALSE;
//     }
//
//     iBcb = *Bcb ? CONTAINING_RECORD(*Bcb, INTERNAL_BCB, PFCB) : NULL;
//
//     ++CcPinMappedDataCount;
//
//     Result = CcpPinData(SharedCacheMap, FileOffset, Length, Flags, Bcb, &Buffer);
//     if (Result)
//     {
//         CcUnpinData(&iBcb->PFCB);
//     }
//
//     return Result;
// }
//
// /*
//  * @unimplemented
//  */
// BOOLEAN
// NTAPI
// CcPinRead (
//     IN	PFILE_OBJECT FileObject,
//     IN	PLARGE_INTEGER FileOffset,
//     IN	ULONG Length,
//     IN	ULONG Flags,
//     OUT	PVOID * Bcb,
//     OUT	PVOID * Buffer)
// {
//     PROS_SHARED_CACHE_MAP SharedCacheMap;
//
//     CCTRACE(CC_API_DEBUG, "FileOffset=%p FileOffset=%p Length=%lu Flags=0x%lx\n",
//         FileObject, FileOffset, Length, Flags);
//
//     ASSERT(FileObject);
//     ASSERT(FileObject->SectionObjectPointer);
//     ASSERT(FileObject->SectionObjectPointer->SharedCacheMap);
//
//     SharedCacheMap = FileObject->SectionObjectPointer->SharedCacheMap;
//     ASSERT(SharedCacheMap);
//     if (!SharedCacheMap->PinAccess)
//     {
//         DPRINT1("FIXME: Pinning a file with no pin access!\n");
//         return FALSE;
//     }
//
//     if (Flags & PIN_WAIT)
//     {
//         ++CcPinReadWait;
//     }
//     else
//     {
//         ++CcPinReadNoWait;
//     }
//
//     return CcpPinData(SharedCacheMap, FileOffset, Length, Flags, Bcb, Buffer);
// }
//
// /*
//  * @unimplemented
//  */
// BOOLEAN
// NTAPI
// CcPreparePinWrite (
//     IN	PFILE_OBJECT FileObject,
//     IN	PLARGE_INTEGER FileOffset,
//     IN	ULONG Length,
//     IN	BOOLEAN Zero,
//     IN	ULONG Flags,
//     OUT	PVOID * Bcb,
//     OUT	PVOID * Buffer)
// {
//     CCTRACE(CC_API_DEBUG, "FileOffset=%p FileOffset=%p Length=%lu Zero=%d Flags=0x%lx\n",
//         FileObject, FileOffset, Length, Zero, Flags);
//
//     /*
//      * FIXME: This is function is similar to CcPinRead, but doesn't
//      * read the data if they're not present. Instead it should just
//      * prepare the VACBs and zero them out if Zero != FALSE.
//      *
//      * For now calling CcPinRead is better than returning error or
//      * just having UNIMPLEMENTED here.
//      */
//     return CcPinRead(FileObject, FileOffset, Length, Flags, Bcb, Buffer);
// }
//
// /*
//  * @implemented
//  */
// VOID NTAPI
// CcSetDirtyPinnedData (
//     IN PVOID Bcb,
//     IN PLARGE_INTEGER Lsn)
// {
//     PINTERNAL_BCB iBcb = CONTAINING_RECORD(Bcb, INTERNAL_BCB, PFCB);
//     PROS_VACB Vacb = iBcb->Vacb;
//
//     CCTRACE(CC_API_DEBUG, "Bcb=%p Lsn=%p\n", Bcb, Lsn);
//
//     /* Tell Mm */
//     MmMakeSegmentDirty(Vacb->SharedCacheMap->FileObject->SectionObjectPointer,
//                        iBcb->PFCB.MappedFileOffset.QuadPart,
//                        iBcb->PFCB.MappedLength);
//
//     if (!Vacb->Dirty)
//     {
//         CcRosMarkDirtyVacb(Vacb);
//     }
// }
//
//
// /*
//  * @implemented
//  */
// VOID NTAPI
// CcUnpinData (
//     IN PVOID Bcb)
// {
//     CCTRACE(CC_API_DEBUG, "Bcb=%p\n", Bcb);
//
//     CcUnpinDataForThread(Bcb, (ERESOURCE_THREAD)PsGetCurrentThread());
// }
//
// /*
//  * @unimplemented
//  */
// VOID
// NTAPI
// CcUnpinDataForThread (
//     IN	PVOID Bcb,
//     IN	ERESOURCE_THREAD ResourceThreadId)
// {
//     PINTERNAL_BCB iBcb = CONTAINING_RECORD(Bcb, INTERNAL_BCB, PFCB);
//
//     CCTRACE(CC_API_DEBUG, "Bcb=%p ResourceThreadId=%lu\n", Bcb, ResourceThreadId);
//
//     if (iBcb->PinCount != 0)
//     {
//         ExReleaseResourceForThreadLite(&iBcb->Lock, ResourceThreadId);
//         iBcb->PinCount--;
//     }
//
//     CcpDereferenceBcb(iBcb->Vacb->SharedCacheMap, iBcb);
// }
//
// /*
//  * @implemented
//  */
// VOID
// NTAPI
// CcRepinBcb (
//     IN	PVOID Bcb)
// {
//     PINTERNAL_BCB iBcb = CONTAINING_RECORD(Bcb, INTERNAL_BCB, PFCB);
//
//     CCTRACE(CC_API_DEBUG, "Bcb=%p\n", Bcb);
//
//     iBcb->RefCount++;
// }
//
// /*
//  * @unimplemented
//  */
// VOID
// NTAPI
// CcUnpinRepinnedBcb (
//     IN	PVOID Bcb,
//     IN	BOOLEAN WriteThrough,
//     IN	PIO_STATUS_BLOCK IoStatus)
// {
//     PINTERNAL_BCB iBcb = CONTAINING_RECORD(Bcb, INTERNAL_BCB, PFCB);
//     KIRQL OldIrql;
//     PROS_SHARED_CACHE_MAP SharedCacheMap;
//
//     CCTRACE(CC_API_DEBUG, "Bcb=%p WriteThrough=%d\n", Bcb, WriteThrough);
//
//     SharedCacheMap = iBcb->Vacb->SharedCacheMap;
//     IoStatus->Status = STATUS_SUCCESS;
//
//     if (WriteThrough)
//     {
//         CcFlushCache(iBcb->Vacb->SharedCacheMap->FileObject->SectionObjectPointer,
//                      &iBcb->PFCB.MappedFileOffset,
//                      iBcb->PFCB.MappedLength,
//                      IoStatus);
//     }
//     else
//     {
//         IoStatus->Status = STATUS_SUCCESS;
//         IoStatus->Information = 0;
//     }
//
//     KeAcquireSpinLock(&SharedCacheMap->BcbSpinLock, &OldIrql);
//     if (--iBcb->RefCount == 0)
//     {
//         RemoveEntryList(&iBcb->BcbEntry);
//         KeReleaseSpinLock(&SharedCacheMap->BcbSpinLock, OldIrql);
//
//         if (iBcb->PinCount != 0)
//         {
//             ExReleaseResourceLite(&iBcb->Lock);
//             iBcb->PinCount--;
//             ASSERT(iBcb->PinCount == 0);
//         }
//
//         /*
//          * Don't mark dirty, if it was dirty,
//          * the VACB was already marked as such
//          * following the call to CcSetDirtyPinnedData
//          */
//         CcRosReleaseVacb(iBcb->Vacb->SharedCacheMap,
//                          iBcb->Vacb,
//                          FALSE,
//                          FALSE);
//
//         ExDeleteResourceLite(&iBcb->Lock);
//         ExFreeToNPagedLookasideList(&iBcbLookasideList, iBcb);
//     }
//     else
//     {
//         KeReleaseSpinLock(&SharedCacheMap->BcbSpinLock, OldIrql);
//     }
// }
//
// ============================================================================
// END ORIGINAL C SOURCE
// ============================================================================
