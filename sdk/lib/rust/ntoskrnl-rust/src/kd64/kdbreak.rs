use ntkrnl_sys::base::*;

use core::ffi::c_void;

extern "C" {
    fn KdpDprintf(format: *const u8, ...);
}

#[no_mangle]
pub unsafe extern "system" fn KdpAddBreakpoint(Address: *mut core::ffi::c_void) -> u32 {
    let mut Content: KD_BREAKPOINT_TYPE = core::mem::zeroed();
    let mut i: u32;
    let mut Status: i32;

    for i in 0..KD_BREAKPOINT_MAX {
        if (KdpBreakpointTable[i as usize].Flags & KD_BREAKPOINT_ACTIVE) != 0 &&
            KdpBreakpointTable[i as usize].Address == Address {
            if (KdpBreakpointTable[i as usize].Flags & KD_BREAKPOINT_EXPIRED) != 0 {
                KdpBreakpointTable[i as usize].Flags &= !KD_BREAKPOINT_EXPIRED;
                return i + 1;
            } else {
                return 0;
            }
        }
    }

    i = 0;
    while i < KD_BREAKPOINT_MAX {
        if KdpBreakpointTable[i as usize].Flags == 0 {
            break;
        }
        i += 1;
    }

    if i == KD_BREAKPOINT_MAX {
        return 0;
    }

    KdpBreakpointTable[i as usize].Address = Address;

    if (Address as usize) < KD_HIGHEST_USER_BREAKPOINT_ADDRESS {
        KdpBreakpointTable[i as usize].DirectoryTableBase = 
            (*(*(*KeGetCurrentThread()).ApcState.Process).DirectoryTableBase.as_ptr().offset(0));
    }

    Status = KdpCopyMemoryChunks(
        Address as u64,
        &mut Content as *mut _ as *mut core::ffi::c_void,
        KD_BREAKPOINT_SIZE,
        0,
        MMDBG_COPY_UNSAFE,
        core::ptr::null_mut(),
    );

    if NT_SUCCESS(Status) {
        KdpBreakpointTable[i as usize].Content = Content;
        KdpBreakpointTable[i as usize].Flags = KD_BREAKPOINT_ACTIVE;

        Status = KdpCopyMemoryChunks(
            Address as u64,
            &KdpBreakpointInstruction as *const _ as *mut core::ffi::c_void,
            KD_BREAKPOINT_SIZE,
            0,
            MMDBG_COPY_UNSAFE | MMDBG_COPY_WRITE,
            core::ptr::null_mut(),
        );

        if !NT_SUCCESS(Status) {
            KdpDprintf(b"Unable to write breakpoint to address 0x%p\n\0".as_ptr(), Address);
        }
    } else {
        KdpDprintf(b"Failed to set breakpoint at address 0x%p, adding deferred breakpoint.\n\0".as_ptr(), Address);
        KdpBreakpointTable[i as usize].Flags = KD_BREAKPOINT_ACTIVE | KD_BREAKPOINT_PENDING;
        KdpOweBreakpoint = 1u8;
    }

    i + 1
}

#[no_mangle]
pub unsafe extern "system" fn KdSetOwedBreakpoints() {
    let mut Enable: u8;
    let mut Content: KD_BREAKPOINT_TYPE;
    let mut i: u32;
    let mut Status: i32;

    if KdpOweBreakpoint == 0 {
        return;
    }

    Enable = KdEnterDebugger(core::ptr::null_mut(), core::ptr::null_mut());

    KdpOweBreakpoint = 0u8;

    i = 0;
    while i < KD_BREAKPOINT_MAX {
        if (KdpBreakpointTable[i as usize].Flags & (KD_BREAKPOINT_PENDING | KD_BREAKPOINT_EXPIRED)) != 0 {
            if (KdpBreakpointTable[i as usize].Address as usize) < KD_HIGHEST_USER_BREAKPOINT_ADDRESS &&
                KdpBreakpointTable[i as usize].DirectoryTableBase != (*(*KeGetCurrentThread()).ApcState.Process).DirectoryTableBase[0] {
                KdpOweBreakpoint = 1u8;
                i += 1;
                continue;
            }

            Status = KdpCopyMemoryChunks(
                KdpBreakpointTable[i as usize].Address as u64,
                core::ptr::addr_of_mut!(Content) as *mut core::ffi::c_void,
                KD_BREAKPOINT_SIZE,
                0,
                MMDBG_COPY_UNSAFE,
                core::ptr::null_mut(),
            );
            if !NT_SUCCESS(Status) {
                KdpOweBreakpoint = 1u8;
                i += 1;
                continue;
            }

            if (KdpBreakpointTable[i as usize].Flags & KD_BREAKPOINT_PENDING) != 0 {
                KdpBreakpointTable[i as usize].Content = Content;

                Status = KdpCopyMemoryChunks(
                    KdpBreakpointTable[i as usize].Address as u64,
                    core::ptr::addr_of_mut!(KdpBreakpointInstruction) as *mut core::ffi::c_void,
                    KD_BREAKPOINT_SIZE,
                    0,
                    MMDBG_COPY_UNSAFE | MMDBG_COPY_WRITE,
                    core::ptr::null_mut(),
                );
                if !NT_SUCCESS(Status) {
                    KdpDprintf(
                        b"Unable to write deferred breakpoint to address 0x%p\n\0".as_ptr() as *const i8,
                        KdpBreakpointTable[i as usize].Address,
                    );
                    KdpOweBreakpoint = 1u8;
                } else {
                    KdpBreakpointTable[i as usize].Flags = KD_BREAKPOINT_ACTIVE;
                }

                i += 1;
                continue;
            }

            if (KdpBreakpointTable[i as usize].Flags & KD_BREAKPOINT_EXPIRED) != 0 {
                Status = KdpCopyMemoryChunks(
                    KdpBreakpointTable[i as usize].Address as u64,
                    core::ptr::addr_of_mut!(KdpBreakpointTable[i as usize].Content) as *mut core::ffi::c_void,
                    KD_BREAKPOINT_SIZE,
                    0,
                    MMDBG_COPY_UNSAFE | MMDBG_COPY_WRITE,
                    core::ptr::null_mut(),
                );
                if !NT_SUCCESS(Status) {
                    KdpDprintf(
                        b"Unable to delete deferred breakpoint at address 0x%p\n\0".as_ptr() as *const i8,
                        KdpBreakpointTable[i as usize].Address,
                    );
                    KdpOweBreakpoint = 1u8;
                } else {
                    if (KdpBreakpointTable[i as usize].Flags & KD_BREAKPOINT_SUSPENDED) != 0 {
                        KdpBreakpointTable[i as usize].Flags = KD_BREAKPOINT_SUSPENDED | KD_BREAKPOINT_ACTIVE;
                    } else {
                        KdpBreakpointTable[i as usize].Flags = 0;
                    }
                }

                i += 1;
                continue;
            }
        }
        i += 1;
    }

    KdExitDebugger(Enable);
}

#[no_mangle]
pub unsafe extern "system" fn KdpLowWriteContent(BpIndex: u32) -> u8 {
    let Status: i32;

    if ((*KdpBreakpointTable.as_mut_ptr().add(BpIndex as usize)).Flags & KD_BREAKPOINT_PENDING) != 0 {
        (*KdpBreakpointTable.as_mut_ptr().add(BpIndex as usize)).Flags &= !KD_BREAKPOINT_PENDING;
        return 1u8;
    }

    if (*KdpBreakpointTable.as_mut_ptr().add(BpIndex as usize)).Content == KdpBreakpointInstruction {
        return 1u8;
    }

    Status = KdpCopyMemoryChunks(
        (*KdpBreakpointTable.as_mut_ptr().add(BpIndex as usize)).Address as u64,
        core::ptr::addr_of_mut!((*KdpBreakpointTable.as_mut_ptr().add(BpIndex as usize)).Content) as *mut core::ffi::c_void,
        KD_BREAKPOINT_SIZE,
        0,
        MMDBG_COPY_UNSAFE | MMDBG_COPY_WRITE,
        core::ptr::null_mut(),
    );
    if NT_SUCCESS(Status) == 0 {
        (*KdpBreakpointTable.as_mut_ptr().add(BpIndex as usize)).Flags |= KD_BREAKPOINT_EXPIRED;
        KdpOweBreakpoint = 1u8;
        return 0u8;
    }

    return 1u8;
}

#[no_mangle]
pub unsafe extern "system" fn KdpLowRestoreBreakpoint(BpIndex: u32) -> u8 {
    let Status: i32;

    if ((*KdpBreakpointTable.as_ptr().add(BpIndex as usize)).Flags & KD_BREAKPOINT_EXPIRED) != 0 {
        (*KdpBreakpointTable.as_mut_ptr().add(BpIndex as usize)).Flags &= !KD_BREAKPOINT_EXPIRED;
        return 1u8;
    }

    if (*KdpBreakpointTable.as_ptr().add(BpIndex as usize)).Content == KdpBreakpointInstruction {
        return 1u8;
    }

    Status = KdpCopyMemoryChunks(
        (*KdpBreakpointTable.as_ptr().add(BpIndex as usize)).Address as u64,
        core::ptr::addr_of_mut!(KdpBreakpointInstruction) as *mut core::ffi::c_void,
        KD_BREAKPOINT_SIZE,
        0,
        MMDBG_COPY_UNSAFE | MMDBG_COPY_WRITE,
        core::ptr::null_mut(),
    );
    if !NT_SUCCESS(Status) {
        (*KdpBreakpointTable.as_mut_ptr().add(BpIndex as usize)).Flags |= KD_BREAKPOINT_PENDING;
        KdpOweBreakpoint = 1u8;
        return 0u8;
    }

    (*KdpBreakpointTable.as_mut_ptr().add(BpIndex as usize)).Flags &= !KD_BREAKPOINT_PENDING;
    return 1u8;
}

#[no_mangle]
pub unsafe extern "system" fn KdpDeleteBreakpoint(BpEntry: u32) -> u8 {
    let BpIndex = BpEntry.wrapping_sub(1);

    if BpEntry == 0 || BpEntry > KD_BREAKPOINT_MAX {
        return 0u8;
    }

    if (*KdpBreakpointTable.as_mut_ptr().add(BpIndex as usize)).Flags == 0 {
        return 0u8;
    }

    if ((*KdpBreakpointTable.as_mut_ptr().add(BpIndex as usize)).Flags & KD_BREAKPOINT_SUSPENDED) != 0 {
        if ((*KdpBreakpointTable.as_mut_ptr().add(BpIndex as usize)).Flags & KD_BREAKPOINT_EXPIRED) == 0 {
            (*KdpBreakpointTable.as_mut_ptr().add(BpIndex as usize)).Flags = 0;
            return 1u8;
        }
    }

    if KdpLowWriteContent(BpIndex) != 0 {
        (*KdpBreakpointTable.as_mut_ptr().add(BpIndex as usize)).Flags = 0;
    }
    return 1u8;
}

#[no_mangle]
pub unsafe extern "system" fn KdpDeleteBreakpointRange(
    Base: PVOID,
    Limit: PVOID,
) -> BOOLEAN {
    let mut BpIndex: ULONG;
    let mut DeletedBreakpoints: BOOLEAN;

    DeletedBreakpoints = 0u8;

    BpIndex = 0;
    while BpIndex < KD_BREAKPOINT_MAX {
        if ((*KdpBreakpointTable.as_mut_ptr().add(BpIndex as usize)).Flags & KD_BREAKPOINT_ACTIVE) != 0
            && ((*KdpBreakpointTable.as_mut_ptr().add(BpIndex as usize)).Address >= Base as usize)
            && ((*KdpBreakpointTable.as_mut_ptr().add(BpIndex as usize)).Address <= Limit as usize)
        {
            if KdpDeleteBreakpoint(BpIndex.wrapping_add(1)) != 0 {
                DeletedBreakpoints = 1u8;
            }
        }
        BpIndex = BpIndex.wrapping_add(1);
    }

    DeletedBreakpoints
}

#[no_mangle]
pub unsafe extern "system" fn KdpRestoreAllBreakpoints() {
    let mut BpIndex: u32;

    BreakpointsSuspended = 0u8;

    BpIndex = 0;
    while BpIndex < KD_BREAKPOINT_MAX {
        if ((*KdpBreakpointTable.as_ptr().add(BpIndex as usize)).Flags & KD_BREAKPOINT_ACTIVE) != 0 &&
           ((*KdpBreakpointTable.as_ptr().add(BpIndex as usize)).Flags & KD_BREAKPOINT_SUSPENDED) != 0
        {
            (*KdpBreakpointTable.as_mut_ptr().add(BpIndex as usize)).Flags &= !KD_BREAKPOINT_SUSPENDED;
            KdpLowRestoreBreakpoint(BpIndex);
        }
        BpIndex += 1;
    }
}

#[no_mangle]
pub unsafe extern "system" fn KdpSuspendBreakPoint(BpEntry: u32) {
    let BpIndex: u32 = BpEntry - 1;

    if (((*KdpBreakpointTable.as_mut_ptr().add(BpIndex as usize)).Flags & KD_BREAKPOINT_ACTIVE) != 0) &&
       (((*KdpBreakpointTable.as_mut_ptr().add(BpIndex as usize)).Flags & KD_BREAKPOINT_SUSPENDED) == 0) {
        (*KdpBreakpointTable.as_mut_ptr().add(BpIndex as usize)).Flags |= KD_BREAKPOINT_SUSPENDED;
        KdpLowWriteContent(BpIndex);
    }
}

#[no_mangle]
pub unsafe extern "system" fn KdpSuspendAllBreakPoints() {
    let mut BpEntry: u32;

    BreakpointsSuspended = 1u8;

    BpEntry = 1;
    while BpEntry <= KD_BREAKPOINT_MAX {
        KdpSuspendBreakPoint(BpEntry);
        BpEntry += 1;
    }
}

// ============================================================================
// ORIGINAL C SOURCE
// ============================================================================
//
// /*
//  * PROJECT:         ReactOS Kernel
//  * LICENSE:         GPL - See COPYING in the top level directory
//  * FILE:            ntoskrnl/kd64/kdbreak.c
//  * PURPOSE:         KD64 Breakpoint Support
//  * PROGRAMMERS:     Alex Ionescu (alex.ionescu@reactos.org)
//  *                  Stefan Ginsberg (stefan.ginsberg@reactos.org)
//  */
//
// /* INCLUDES ******************************************************************/
//
// #include <ntoskrnl.h>
// #define NDEBUG
// #include <debug.h>
//
// /* FUNCTIONS *****************************************************************/
//
// ULONG
// NTAPI
// KdpAddBreakpoint(IN PVOID Address)
// {
//     KD_BREAKPOINT_TYPE Content;
//     ULONG i;
//     NTSTATUS Status;
//
//     /* Check whether we are not setting a breakpoint twice */
//     for (i = 0; i < KD_BREAKPOINT_MAX; i++)
//     {
//         /* Check if the breakpoint is valid */
//         if ((KdpBreakpointTable[i].Flags & KD_BREAKPOINT_ACTIVE) &&
//             (KdpBreakpointTable[i].Address == Address))
//         {
//             /* Were we not able to remove it earlier? */
//             if (KdpBreakpointTable[i].Flags & KD_BREAKPOINT_EXPIRED)
//             {
//                 /* Just re-use it! */
//                 KdpBreakpointTable[i].Flags &= ~KD_BREAKPOINT_EXPIRED;
//                 return i + 1;
//             }
//             else
//             {
//                 /* Fail */
//                 return 0;
//             }
//         }
//     }
//
//     /* Find a free entry */
//     for (i = 0; i < KD_BREAKPOINT_MAX; i++)
//     {
//         if (KdpBreakpointTable[i].Flags == 0)
//             break;
//     }
//
//     /* Fail if no free entry was found */
//     if (i == KD_BREAKPOINT_MAX) return 0;
//
//     /* Save the breakpoint */
//     KdpBreakpointTable[i].Address = Address;
//
//     /* If we are setting the breakpoint in user space, save the active process context */
//     if (Address < KD_HIGHEST_USER_BREAKPOINT_ADDRESS)
//         KdpBreakpointTable[i].DirectoryTableBase = KeGetCurrentThread()->ApcState.Process->DirectoryTableBase[0];
//
//     /* Try to save the old instruction */
//     Status = KdpCopyMemoryChunks((ULONG_PTR)Address,
//                                  &Content,
//                                  KD_BREAKPOINT_SIZE,
//                                  0,
//                                  MMDBG_COPY_UNSAFE,
//                                  NULL);
//     if (NT_SUCCESS(Status))
//     {
//         /* Memory accessible, set the breakpoint */
//         KdpBreakpointTable[i].Content = Content;
//         KdpBreakpointTable[i].Flags   = KD_BREAKPOINT_ACTIVE;
//
//         /* Write the breakpoint */
//         Status = KdpCopyMemoryChunks((ULONG_PTR)Address,
//                                      &KdpBreakpointInstruction,
//                                      KD_BREAKPOINT_SIZE,
//                                      0,
//                                      MMDBG_COPY_UNSAFE | MMDBG_COPY_WRITE,
//                                      NULL);
//         if (!NT_SUCCESS(Status))
//         {
//             /* This should never happen */
//             KdpDprintf("Unable to write breakpoint to address 0x%p\n", Address);
//         }
//     }
//     else
//     {
//         /* Memory is inaccessible now, setting breakpoint is deferred */
//         KdpDprintf("Failed to set breakpoint at address 0x%p, adding deferred breakpoint.\n", Address);
//         KdpBreakpointTable[i].Flags = KD_BREAKPOINT_ACTIVE | KD_BREAKPOINT_PENDING;
//         KdpOweBreakpoint = TRUE;
//     }
//
//     /* Return the breakpoint handle */
//     return i + 1;
// }
//
// VOID
// NTAPI
// KdSetOwedBreakpoints(VOID)
// {
//     BOOLEAN Enable;
//     KD_BREAKPOINT_TYPE Content;
//     ULONG i;
//     NTSTATUS Status;
//
//     /* If we don't owe any breakpoints, just return */
//     if (!KdpOweBreakpoint) return;
//
//     /* Enter the debugger */
//     Enable = KdEnterDebugger(NULL, NULL);
//
//     /*
//      * Suppose we succeed in setting all the breakpoints.
//      * If we fail to do so, the flag will be set again.
//      */
//     KdpOweBreakpoint = FALSE;
//
//     /* Loop through current breakpoints and try to set or delete the pending ones */
//     for (i = 0; i < KD_BREAKPOINT_MAX; i++)
//     {
//         if (KdpBreakpointTable[i].Flags & (KD_BREAKPOINT_PENDING | KD_BREAKPOINT_EXPIRED))
//         {
//             /*
//              * Set the breakpoint only if it is in kernel space, or if it is
//              * in user space and the active process context matches.
//              */
//             if (KdpBreakpointTable[i].Address < KD_HIGHEST_USER_BREAKPOINT_ADDRESS &&
//                 KdpBreakpointTable[i].DirectoryTableBase != KeGetCurrentThread()->ApcState.Process->DirectoryTableBase[0])
//             {
//                 KdpOweBreakpoint = TRUE;
//                 continue;
//             }
//
//             /* Try to save the old instruction */
//             Status = KdpCopyMemoryChunks((ULONG_PTR)KdpBreakpointTable[i].Address,
//                                          &Content,
//                                          KD_BREAKPOINT_SIZE,
//                                          0,
//                                          MMDBG_COPY_UNSAFE,
//                                          NULL);
//             if (!NT_SUCCESS(Status))
//             {
//                 /* Memory is still inaccessible, breakpoint setting will be deferred again */
//                 // KdpDprintf("Failed to set deferred breakpoint at address 0x%p\n",
//                 //            KdpBreakpointTable[i].Address);
//                 KdpOweBreakpoint = TRUE;
//                 continue;
//             }
//
//             /* Check if we need to write the breakpoint */
//             if (KdpBreakpointTable[i].Flags & KD_BREAKPOINT_PENDING)
//             {
//                 /* Memory accessible, set the breakpoint */
//                 KdpBreakpointTable[i].Content = Content;
//
//                 /* Write the breakpoint */
//                 Status = KdpCopyMemoryChunks((ULONG_PTR)KdpBreakpointTable[i].Address,
//                                              &KdpBreakpointInstruction,
//                                              KD_BREAKPOINT_SIZE,
//                                              0,
//                                              MMDBG_COPY_UNSAFE | MMDBG_COPY_WRITE,
//                                              NULL);
//                 if (!NT_SUCCESS(Status))
//                 {
//                     /* This should never happen */
//                     KdpDprintf("Unable to write deferred breakpoint to address 0x%p\n",
//                                KdpBreakpointTable[i].Address);
//                     KdpOweBreakpoint = TRUE;
//                 }
//                 else
//                 {
//                     KdpBreakpointTable[i].Flags = KD_BREAKPOINT_ACTIVE;
//                 }
//
//                 continue;
//             }
//
//             /* Check if we need to restore the original instruction */
//             if (KdpBreakpointTable[i].Flags & KD_BREAKPOINT_EXPIRED)
//             {
//                 /* Write it back */
//                 Status = KdpCopyMemoryChunks((ULONG_PTR)KdpBreakpointTable[i].Address,
//                                              &KdpBreakpointTable[i].Content,
//                                              KD_BREAKPOINT_SIZE,
//                                              0,
//                                              MMDBG_COPY_UNSAFE | MMDBG_COPY_WRITE,
//                                              NULL);
//                 if (!NT_SUCCESS(Status))
//                 {
//                     /* This should never happen */
//                     KdpDprintf("Unable to delete deferred breakpoint at address 0x%p\n",
//                                KdpBreakpointTable[i].Address);
//                     KdpOweBreakpoint = TRUE;
//                 }
//                 else
//                 {
//                     /* Check if the breakpoint is suspended */
//                     if (KdpBreakpointTable[i].Flags & KD_BREAKPOINT_SUSPENDED)
//                     {
//                         KdpBreakpointTable[i].Flags = KD_BREAKPOINT_SUSPENDED | KD_BREAKPOINT_ACTIVE;
//                     }
//                     else
//                     {
//                         /* Invalidate it */
//                         KdpBreakpointTable[i].Flags = 0;
//                     }
//                 }
//
//                 continue;
//             }
//         }
//     }
//
//     /* Exit the debugger */
//     KdExitDebugger(Enable);
// }
//
// BOOLEAN
// NTAPI
// KdpLowWriteContent(IN ULONG BpIndex)
// {
//     NTSTATUS Status;
//
//     /* Make sure that the breakpoint is actually active */
//     if (KdpBreakpointTable[BpIndex].Flags & KD_BREAKPOINT_PENDING)
//     {
//         /* So we have a valid breakpoint, but it hasn't been used yet... */
//         KdpBreakpointTable[BpIndex].Flags &= ~KD_BREAKPOINT_PENDING;
//         return TRUE;
//     }
//
//     /* Is the original instruction a breakpoint anyway? */
//     if (KdpBreakpointTable[BpIndex].Content == KdpBreakpointInstruction)
//     {
//         /* Then leave it that way... */
//         return TRUE;
//     }
//
//     /* We have an active breakpoint with an instruction to bring back. Do it. */
//     Status = KdpCopyMemoryChunks((ULONG_PTR)KdpBreakpointTable[BpIndex].Address,
//                                  &KdpBreakpointTable[BpIndex].Content,
//                                  KD_BREAKPOINT_SIZE,
//                                  0,
//                                  MMDBG_COPY_UNSAFE | MMDBG_COPY_WRITE,
//                                  NULL);
//     if (!NT_SUCCESS(Status))
//     {
//         /* Memory is inaccessible now, restoring original instruction is deferred */
//         // KdpDprintf("Failed to delete breakpoint at address 0x%p\n",
//         //            KdpBreakpointTable[BpIndex].Address);
//         KdpBreakpointTable[BpIndex].Flags |= KD_BREAKPOINT_EXPIRED;
//         KdpOweBreakpoint = TRUE;
//         return FALSE;
//     }
//
//     /* Everything went fine, return */
//     return TRUE;
// }
//
// BOOLEAN
// NTAPI
// KdpLowRestoreBreakpoint(IN ULONG BpIndex)
// {
//     NTSTATUS Status;
//
//     /* Were we not able to remove it earlier? */
//     if (KdpBreakpointTable[BpIndex].Flags & KD_BREAKPOINT_EXPIRED)
//     {
//         /* Just re-use it! */
//         KdpBreakpointTable[BpIndex].Flags &= ~KD_BREAKPOINT_EXPIRED;
//         return TRUE;
//     }
//
//     /* Are we merely writing a breakpoint on top of another breakpoint? */
//     if (KdpBreakpointTable[BpIndex].Content == KdpBreakpointInstruction)
//     {
//         /* Nothing to do */
//         return TRUE;
//     }
//
//     /* Ok, we actually have to overwrite the instruction now */
//     Status = KdpCopyMemoryChunks((ULONG_PTR)KdpBreakpointTable[BpIndex].Address,
//                                  &KdpBreakpointInstruction,
//                                  KD_BREAKPOINT_SIZE,
//                                  0,
//                                  MMDBG_COPY_UNSAFE | MMDBG_COPY_WRITE,
//                                  NULL);
//     if (!NT_SUCCESS(Status))
//     {
//         /* Memory is inaccessible now, restoring breakpoint is deferred */
//         // KdpDprintf("Failed to restore breakpoint at address 0x%p\n",
//         //            KdpBreakpointTable[BpIndex].Address);
//         KdpBreakpointTable[BpIndex].Flags |= KD_BREAKPOINT_PENDING;
//         KdpOweBreakpoint = TRUE;
//         return FALSE;
//     }
//
//     /* Clear any possible previous pending flag and return success */
//     KdpBreakpointTable[BpIndex].Flags &= ~KD_BREAKPOINT_PENDING;
//     return TRUE;
// }
//
// BOOLEAN
// NTAPI
// KdpDeleteBreakpoint(IN ULONG BpEntry)
// {
//     ULONG BpIndex = BpEntry - 1;
//
//     /* Check for invalid breakpoint entry */
//     if (!BpEntry || (BpEntry > KD_BREAKPOINT_MAX)) return FALSE;
//
//     /* If the specified breakpoint table entry is not valid, then return FALSE. */
//     if (!KdpBreakpointTable[BpIndex].Flags) return FALSE;
//
//     /* Check if the breakpoint is suspended */
//     if (KdpBreakpointTable[BpIndex].Flags & KD_BREAKPOINT_SUSPENDED)
//     {
//         /* Check if breakpoint is not being deleted */
//         if (!(KdpBreakpointTable[BpIndex].Flags & KD_BREAKPOINT_EXPIRED))
//         {
//             /* Invalidate it and return success */
//             KdpBreakpointTable[BpIndex].Flags = 0;
//             return TRUE;
//         }
//     }
//
//     /* Restore original data, then invalidate it and return success */
//     if (KdpLowWriteContent(BpIndex)) KdpBreakpointTable[BpIndex].Flags = 0;
//     return TRUE;
// }
//
// BOOLEAN
// NTAPI
// KdpDeleteBreakpointRange(IN PVOID Base,
//                          IN PVOID Limit)
// {
//     ULONG BpIndex;
//     BOOLEAN DeletedBreakpoints;
//
//     /* Assume no breakpoints will be deleted */
//     DeletedBreakpoints = FALSE;
//
//     /* Loop the breakpoint table */
//     for (BpIndex = 0; BpIndex < KD_BREAKPOINT_MAX; BpIndex++)
//     {
//         /* Make sure that the breakpoint is active and matches the range. */
//         if ((KdpBreakpointTable[BpIndex].Flags & KD_BREAKPOINT_ACTIVE) &&
//             ((KdpBreakpointTable[BpIndex].Address >= Base) &&
//              (KdpBreakpointTable[BpIndex].Address <= Limit)))
//         {
//             /* Delete it, and remember if we succeeded at least once */
//             if (KdpDeleteBreakpoint(BpIndex + 1)) DeletedBreakpoints = TRUE;
//         }
//     }
//
//     /* Return whether we deleted anything */
//     return DeletedBreakpoints;
// }
//
// VOID
// NTAPI
// KdpRestoreAllBreakpoints(VOID)
// {
//     ULONG BpIndex;
//
//     /* No more suspended Breakpoints */
//     BreakpointsSuspended = FALSE;
//
//     /* Loop the breakpoints */
//     for (BpIndex = 0; BpIndex < KD_BREAKPOINT_MAX; BpIndex++)
//     {
//         /* Check if they are valid, suspended breakpoints */
//         if ((KdpBreakpointTable[BpIndex].Flags & KD_BREAKPOINT_ACTIVE) &&
//             (KdpBreakpointTable[BpIndex].Flags & KD_BREAKPOINT_SUSPENDED))
//         {
//             /* Unsuspend them */
//             KdpBreakpointTable[BpIndex].Flags &= ~KD_BREAKPOINT_SUSPENDED;
//             KdpLowRestoreBreakpoint(BpIndex);
//         }
//     }
// }
//
// VOID
// NTAPI
// KdpSuspendBreakPoint(IN ULONG BpEntry)
// {
//     ULONG BpIndex = BpEntry - 1;
//
//     /* Check if this is a valid, unsuspended breakpoint */
//     if ((KdpBreakpointTable[BpIndex].Flags & KD_BREAKPOINT_ACTIVE) &&
//         !(KdpBreakpointTable[BpIndex].Flags & KD_BREAKPOINT_SUSPENDED))
//     {
//         /* Suspend it */
//         KdpBreakpointTable[BpIndex].Flags |= KD_BREAKPOINT_SUSPENDED;
//         KdpLowWriteContent(BpIndex);
//     }
// }
//
// VOID
// NTAPI
// KdpSuspendAllBreakPoints(VOID)
// {
//     ULONG BpEntry;
//
//     /* Breakpoints are suspended */
//     BreakpointsSuspended = TRUE;
//
//     /* Loop every breakpoint */
//     for (BpEntry = 1; BpEntry <= KD_BREAKPOINT_MAX; BpEntry++)
//     {
//         /* Suspend it */
//         KdpSuspendBreakPoint(BpEntry);
//     }
// }
//
// ============================================================================
// END ORIGINAL C SOURCE
// ============================================================================
