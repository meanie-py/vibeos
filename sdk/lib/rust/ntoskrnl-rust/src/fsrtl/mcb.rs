use ntkrnl_sys::globals::*;
use ntkrnl_sys::constants::*;
use ntkrnl_sys::base::*;
use ntkrnl_sys::mm::*;

use ntkrnl_sys::ddk::{
    LBN,
    PLBN,
    PMCB,
    PVBN,
    VBN,
};
use ntkrnl_sys::{opaque_ptr, opaque_ptr_const};

extern "system" {
    fn FsRtlAddLargeMcbEntry( Mcb: PVOID, Vbn: i64, Lbn: i64, SectorCount: i64, ) -> BOOLEAN;
    fn FsRtlGetNextLargeMcbEntry( Mcb: PVOID, RunIndex: ULONG, Vbn: *mut i64, Lbn: *mut i64, SectorCount: *mut i64, ) -> BOOLEAN;
    fn FsRtlInitializeLargeMcb( Mcb: PVOID, PoolType: POOL_TYPE, );
    fn FsRtlLookupLastLargeMcbEntry( Mcb: PVOID, Vbn: *mut i64, Lbn: *mut i64, ) -> BOOLEAN;
    fn FsRtlLookupLargeMcbEntry( Mcb: PVOID, Vbn: i64, Lbn: *mut i64, SectorCount: *mut i64, StartingLbn: *mut i64, CountFromStartingLbn: *mut i64, Index: *mut ULONG, ) -> BOOLEAN;
    fn FsRtlNumberOfRunsInLargeMcb( Mcb: PVOID, ) -> ULONG;
    fn FsRtlRemoveLargeMcbEntry( Mcb: PVOID, Vbn: i64, SectorCount: i64, );
    fn FsRtlTruncateLargeMcb( Mcb: PVOID, Vbn: i64, );
    fn FsRtlUninitializeLargeMcb( Mcb: PVOID, );
}

#[no_mangle]
pub unsafe extern "system" fn FsRtlAddMcbEntry(
    Mcb: PMCB,
    Vbn: VBN,
    Lbn: LBN,
    SectorCount: ULONG,
) -> BOOLEAN {
    FsRtlAddLargeMcbEntry(
        Mcb as PVOID,
        Vbn as i64,
        Lbn as i64,
        SectorCount as i64,
    )
}

#[no_mangle]
pub unsafe extern "system" fn FsRtlGetNextMcbEntry(
    Mcb: PMCB,
    RunIndex: ULONG,
    Vbn: PVBN,
    Lbn: PLBN,
    SectorCount: *mut ULONG,
) -> BOOLEAN {
    let SectorCount = opaque_ptr(SectorCount);
    let mut ll_vbn: i64 = 0;
    let mut ll_lbn: i64 = 0;
    let mut ll_sector_count: i64 = 0;

    let return_value = FsRtlGetNextLargeMcbEntry(
        Mcb as PVOID,
        RunIndex,
        &mut ll_vbn,
        &mut ll_lbn,
        &mut ll_sector_count,
    );

    *Vbn = ll_vbn as ULONG;
    *Lbn = ll_lbn as ULONG;
    *SectorCount = ll_sector_count as ULONG;

    return_value
}

#[no_mangle]
pub unsafe extern "system" fn FsRtlInitializeMcb(
    Mcb: PMCB,
    PoolType: POOL_TYPE,
) {
    FsRtlInitializeLargeMcb(
        Mcb as PVOID,
        PoolType,
    );
}

#[no_mangle]
pub unsafe extern "system" fn FsRtlLookupLastMcbEntry(
    Mcb: PMCB,
    Vbn: PVBN,
    Lbn: PLBN,
) -> BOOLEAN {
    let mut ll_vbn: i64 = 0;
    let mut ll_lbn: i64 = 0;

    let return_value = FsRtlLookupLastLargeMcbEntry(
        Mcb as PVOID,
        &mut ll_vbn,
        &mut ll_lbn,
    );

    *Vbn = ll_vbn as ULONG;
    *Lbn = ll_lbn as ULONG;

    return_value
}

#[no_mangle]
pub unsafe extern "system" fn FsRtlLookupMcbEntry(
    Mcb: PMCB,
    Vbn: VBN,
    Lbn: PLBN,
    SectorCount: *mut ULONG,
    Index: *mut ULONG,
) -> BOOLEAN {
    let SectorCount = opaque_ptr(SectorCount);
    let Index = opaque_ptr(Index);
    let mut ll_lbn: i64 = 0;
    let mut ll_sector_count: i64 = 0;

    let return_value = FsRtlLookupLargeMcbEntry(
        Mcb as PVOID,
        Vbn as i64,
        &mut ll_lbn,
        &mut ll_sector_count,
        core::ptr::null_mut(),
        core::ptr::null_mut(),
        Index,
    );

    *Lbn = ll_lbn as ULONG;
    if !SectorCount.is_null() {
        *SectorCount = ll_sector_count as ULONG;
    }

    return_value
}

#[no_mangle]
pub unsafe extern "system" fn FsRtlNumberOfRunsInMcb(
    Mcb: PMCB,
) -> ULONG {
    FsRtlNumberOfRunsInLargeMcb(Mcb as PVOID)
}

#[no_mangle]
pub unsafe extern "system" fn FsRtlRemoveMcbEntry(
    Mcb: PMCB,
    Vbn: VBN,
    SectorCount: ULONG,
) {
    FsRtlRemoveLargeMcbEntry(
        Mcb as PVOID,
        Vbn as i64,
        SectorCount as i64,
    );
}

#[no_mangle]
pub unsafe extern "system" fn FsRtlTruncateMcb(
    Mcb: PMCB,
    Vbn: VBN,
) {
    FsRtlTruncateLargeMcb(
        Mcb as PVOID,
        Vbn as i64,
    );
}

#[no_mangle]
pub unsafe extern "system" fn FsRtlUninitializeMcb(
    Mcb: PMCB,
) {
    FsRtlUninitializeLargeMcb(Mcb as PVOID);
}

// ============================================================================
// ORIGINAL C SOURCE
// ============================================================================
//
// /*
//  * PROJECT:         ReactOS Kernel
//  * LICENSE:         GPL - See COPYING in the top level directory
//  * FILE:            ntoskrnl/fsrtl/mcb.c
//  * PURPOSE:         Mapped Control Block (MCB) support for File System Drivers
//  * PROGRAMMERS:     Alex Ionescu (alex.ionescu@reactos.org)
//  */
//
// /* INCLUDES ******************************************************************/
//
// #include <ntoskrnl.h>
// #define NDEBUG
// #include <debug.h>
//
// /* PUBLIC FUNCTIONS **********************************************************/
//
// /*
//  * @implemented
//  */
// BOOLEAN
// NTAPI
// FsRtlAddMcbEntry(IN PMCB Mcb,
//                  IN VBN Vbn,
//                  IN LBN Lbn,
//                  IN ULONG SectorCount)
// {
//     /* Call the newer function */
//     return FsRtlAddLargeMcbEntry(&Mcb->
//                                  DummyFieldThatSizesThisStructureCorrectly,
//                                  (LONGLONG)Vbn,
//                                  (LONGLONG)Lbn,
//                                  (LONGLONG)SectorCount);
// }
//
// /*
//  * @implemented
//  */
// BOOLEAN
// NTAPI
// FsRtlGetNextMcbEntry(IN PMCB Mcb,
//                      IN ULONG RunIndex,
//                      OUT PVBN Vbn,
//                      OUT PLBN Lbn,
//                      OUT PULONG SectorCount)
// {
//     BOOLEAN Return = FALSE;
//     LONGLONG llVbn;
//     LONGLONG llLbn;
//     LONGLONG llSectorCount;
//
//     /* Call the Large version */
//     Return = FsRtlGetNextLargeMcbEntry(
//         &Mcb->DummyFieldThatSizesThisStructureCorrectly,
//         RunIndex,
//         &llVbn,
//         &llLbn,
//         &llSectorCount);
//
//     /* Return the lower 32 bits */
//     *Vbn = (ULONG)llVbn;
//     *Lbn = (ULONG)llLbn;
//     *SectorCount = (ULONG)llSectorCount;
//
//     /* And return the original value */
//     return Return;
// }
//
// /*
//  * @implemented
//  */
// VOID
// NTAPI
// FsRtlInitializeMcb(IN PMCB Mcb,
//                    IN POOL_TYPE PoolType)
// {
//     /* Call the newer function */
//     FsRtlInitializeLargeMcb(&Mcb->DummyFieldThatSizesThisStructureCorrectly,
//                             PoolType);
// }
//
// /*
//  * @implemented
//  */
// BOOLEAN
// NTAPI
// FsRtlLookupLastMcbEntry(IN PMCB Mcb,
//                         OUT PVBN Vbn,
//                         OUT PLBN Lbn)
// {
//     BOOLEAN Return = FALSE;
//     LONGLONG llVbn = 0;
//     LONGLONG llLbn = 0;
//
//     /* Call the Large version */
//     Return = FsRtlLookupLastLargeMcbEntry(
//         &Mcb->DummyFieldThatSizesThisStructureCorrectly,
//         &llVbn,
//         &llLbn);
//
//     /* Return the lower 32-bits */
//     *Vbn = (ULONG)llVbn;
//     *Lbn = (ULONG)llLbn;
//
//     /* And return the original value */
//     return Return;
// }
//
// /*
//  * @implemented
//  */
// BOOLEAN
// NTAPI
// FsRtlLookupMcbEntry(IN PMCB Mcb,
//                     IN VBN Vbn,
//                     OUT PLBN Lbn,
//                     OUT PULONG SectorCount OPTIONAL,
//                     OUT PULONG Index)
// {
//     BOOLEAN Return = FALSE;
//     LONGLONG llLbn;
//     LONGLONG llSectorCount;
//
//     /* Call the Large version */
//     Return = FsRtlLookupLargeMcbEntry(&Mcb->
//                                       DummyFieldThatSizesThisStructureCorrectly,
//                                       (LONGLONG)Vbn,
//                                       &llLbn,
//                                       &llSectorCount,
//                                       NULL,
//                                       NULL,
//                                       Index);
//
//     /* Return the lower 32-bits */
//     *Lbn = (ULONG)llLbn;
//     if (SectorCount) *SectorCount = (ULONG)llSectorCount;
//
//     /* And return the original value */
//     return Return;
// }
//
// /*
//  * @implemented
//  */
// ULONG
// NTAPI
// FsRtlNumberOfRunsInMcb(IN PMCB Mcb)
// {
//     /* Call the newer function */
//     return FsRtlNumberOfRunsInLargeMcb(
//         &Mcb->DummyFieldThatSizesThisStructureCorrectly);
// }
//
// /*
//  * @implemented
//  */
// VOID
// NTAPI
// FsRtlRemoveMcbEntry(IN PMCB Mcb,
//                     IN VBN Vbn,
//                     IN ULONG SectorCount)
// {
//     /* Call the large function */
//     FsRtlRemoveLargeMcbEntry(&Mcb->DummyFieldThatSizesThisStructureCorrectly,
//                              (LONGLONG)Vbn,
//                              (LONGLONG)SectorCount);
// }
//
// /*
//  * @implemented
//  */
// VOID
// NTAPI
// FsRtlTruncateMcb(IN PMCB Mcb,
//                  IN VBN  Vbn)
// {
//     /* Call the newer function */
//     FsRtlTruncateLargeMcb(&Mcb->DummyFieldThatSizesThisStructureCorrectly,
//                           (LONGLONG)Vbn);
// }
//
// /*
//  * @implemented
//  */
// VOID
// NTAPI
// FsRtlUninitializeMcb(IN PMCB Mcb)
// {
//     /* Call the newer function */
//     FsRtlUninitializeLargeMcb(&Mcb->DummyFieldThatSizesThisStructureCorrectly);
// }
//
// ============================================================================
// END ORIGINAL C SOURCE
// ============================================================================
