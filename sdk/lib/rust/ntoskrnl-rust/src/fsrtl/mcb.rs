use ntkrnl_sys::globals::*;
use ntkrnl_sys::constants::*;
use ntkrnl_sys::base::*;
use ntkrnl_sys::mm::*;
use ntkrnl_sys::ddk::{LBN, PLBN, PMCB, PVBN, VBN};
use core::ffi::c_void;

#[no_mangle]
pub unsafe extern "system" fn FsRtlAddMcbEntry(
    Mcb: PMCB,
    Vbn: VBN,
    Lbn: LBN,
    SectorCount: ULONG,
) -> BOOLEAN {
    FsRtlAddLargeMcbEntry(
        core::ptr::addr_of_mut!((*Mcb).DummyFieldThatSizesThisStructureCorrectly),
        Vbn as LONGLONG,
        Lbn as LONGLONG,
        SectorCount as LONGLONG,
    )
}

#[no_mangle]
pub unsafe extern "system" fn FsRtlGetNextMcbEntry(
    Mcb: PMCB,
    RunIndex: ULONG,
    Vbn: PVBN,
    Lbn: PLBN,
    SectorCount: PULONG,
) -> BOOLEAN {
    let mut Return: BOOLEAN = 0u8;
    let mut llVbn: LONGLONG = 0;
    let mut llLbn: LONGLONG = 0;
    let mut llSectorCount: LONGLONG = 0;

    Return = FsRtlGetNextLargeMcbEntry(
        core::ptr::addr_of_mut!((*Mcb).DummyFieldThatSizesThisStructureCorrectly),
        RunIndex,
        &mut llVbn,
        &mut llLbn,
        &mut llSectorCount,
    );

    *Vbn = llVbn as ULONG;
    *Lbn = llLbn as ULONG;
    *SectorCount = llSectorCount as ULONG;

    return Return;
}

#[no_mangle]
pub unsafe extern "system" fn FsRtlInitializeMcb(
    Mcb: PMCB,
    PoolType: POOL_TYPE,
) {
    FsRtlInitializeLargeMcb(
        core::ptr::addr_of_mut!((*Mcb).DummyFieldThatSizesThisStructureCorrectly),
        PoolType,
    );
}

#[no_mangle]
pub unsafe extern "system" fn FsRtlLookupLastMcbEntry(
    Mcb: PMCB,
    Vbn: PVBN,
    Lbn: PLBN,
) -> BOOLEAN {
    let mut Return: BOOLEAN = 0u8;
    let mut llVbn: i64 = 0;
    let mut llLbn: i64 = 0;

    Return = FsRtlLookupLastLargeMcbEntry(
        core::ptr::addr_of_mut!((*Mcb).DummyFieldThatSizesThisStructureCorrectly),
        &mut llVbn,
        &mut llLbn,
    );

    *Vbn = llVbn as u32;
    *Lbn = llLbn as u32;

    return Return;
}

#[no_mangle]
pub unsafe extern "system" fn FsRtlLookupMcbEntry(
    Mcb: PMCB,
    Vbn: VBN,
    Lbn: PLBN,
    SectorCount: PULONG,
    Index: PULONG,
) -> BOOLEAN {
    let mut Return: BOOLEAN = 0u8;
    let mut llLbn: LONGLONG = 0;
    let mut llSectorCount: LONGLONG = 0;

    Return = FsRtlLookupLargeMcbEntry(
        core::ptr::addr_of_mut!((*Mcb).DummyFieldThatSizesThisStructureCorrectly),
        Vbn as LONGLONG,
        &mut llLbn,
        &mut llSectorCount,
        core::ptr::null_mut(),
        core::ptr::null_mut(),
        Index,
    );

    *Lbn = llLbn as ULONG;
    if !SectorCount.is_null() {
        *SectorCount = llSectorCount as ULONG;
    }

    return Return;
}

#[no_mangle]
pub unsafe extern "system" fn FsRtlNumberOfRunsInMcb(Mcb: PMCB) -> ULONG {
    FsRtlNumberOfRunsInLargeMcb(
        core::ptr::addr_of_mut!((*Mcb).DummyFieldThatSizesThisStructureCorrectly)
    )
}

#[no_mangle]
pub unsafe extern "system" fn FsRtlRemoveMcbEntry(
    Mcb: PMCB,
    Vbn: VBN,
    SectorCount: u32,
)
{
    FsRtlRemoveLargeMcbEntry(
        core::ptr::addr_of_mut!((*Mcb).DummyFieldThatSizesThisStructureCorrectly),
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
        core::ptr::addr_of_mut!((*Mcb).DummyFieldThatSizesThisStructureCorrectly),
        Vbn as i64,
    );
}

#[no_mangle]
pub unsafe extern "system" fn FsRtlUninitializeMcb(Mcb: PMCB) {
    FsRtlUninitializeLargeMcb(core::ptr::addr_of_mut!((*Mcb).DummyFieldThatSizesThisStructureCorrectly));
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
