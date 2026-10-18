use ntkrnl_sys::base::*;
use ntkrnl_sys::constants::KD_BREAKPOINT_SIZE;
use ntkrnl_sys::ddk::PKTHREAD;
use ntkrnl_sys::globals::KD_BREAKPOINT_TYPE;
use ntkrnl_sys::{opaque_ptr, opaque_ptr_const};

// External declarations from other KD modules
extern "C" {
    static mut KdpBreakpointTable: [BREAKPOINT_ENTRY; KD_BREAKPOINT_MAX];
    static mut KdpBreakpointInstruction: KD_BREAKPOINT_TYPE;
    static mut KdpOweBreakpoint: BOOLEAN;
    static mut BreakpointsSuspended: BOOLEAN;
}

extern "system" {
    // Address is ULONG64 (the kd debugger protocol is 64-bit even on i386) to
    // match the definition in kdapi.rs — ULONG_PTR here mis-sized the i386
    // stdcall arg (@24 vs the real @28) → undefined symbol at cold link.
    fn KdpCopyMemoryChunks( Address: u64, Buffer: PVOID, Size: ULONG, Unknown: ULONG, Flags: ULONG, ActualSize: *mut ULONG, ) -> NTSTATUS;
    fn KdEnterDebugger(TrapFrame: PVOID, ExceptionFrame: PVOID) -> BOOLEAN;
    fn KdExitDebugger(Enable: BOOLEAN);
    fn KeGetCurrentThread() -> PKTHREAD;
}

// Constants
const KD_BREAKPOINT_MAX: usize = 32;
const KD_BREAKPOINT_ACTIVE: ULONG = 0x01;
const KD_BREAKPOINT_PENDING: ULONG = 0x02;
const KD_BREAKPOINT_SUSPENDED: ULONG = 0x04;
const KD_BREAKPOINT_EXPIRED: ULONG = 0x08;
// mm.h's encoding, the one MmDbgCopyMemory reads (see kdapi.rs).
use ntkrnl_sys::mm::{MMDBG_COPY_UNSAFE, MMDBG_COPY_WRITE};

// Breakpoint table entry structure
#[repr(C)]
struct BREAKPOINT_ENTRY {
    Flags: ULONG,
    DirectoryTableBase: ULONG_PTR,
    Address: PVOID,
    Content: KD_BREAKPOINT_TYPE,
}

// Architecture-specific highest user breakpoint address
#[cfg(target_pointer_width = "64")]
const KD_HIGHEST_USER_BREAKPOINT_ADDRESS: PVOID = 0x000007FFFFFFFFFF as PVOID;

#[cfg(target_pointer_width = "32")]
const KD_HIGHEST_USER_BREAKPOINT_ADDRESS: PVOID = 0x7FFFFFFF as PVOID;

#[no_mangle]
pub unsafe extern "system" fn KdpAddBreakpoint(Address: PVOID) -> ULONG {
    let Address = opaque_ptr(Address);
    let mut content: KD_BREAKPOINT_TYPE = 0;
    let mut i: ULONG;
    let mut status: NTSTATUS;

    // Check whether we are not setting a breakpoint twice
    i = 0;
    while i < KD_BREAKPOINT_MAX as ULONG {
        // Check if the breakpoint is valid
        if (KdpBreakpointTable[i as usize].Flags & KD_BREAKPOINT_ACTIVE) != 0 &&
           KdpBreakpointTable[i as usize].Address == Address
        {
            // Were we not able to remove it earlier?
            if (KdpBreakpointTable[i as usize].Flags & KD_BREAKPOINT_EXPIRED) != 0 {
                // Just re-use it!
                KdpBreakpointTable[i as usize].Flags &= !KD_BREAKPOINT_EXPIRED;
                return i + 1;
            } else {
                // Fail
                return 0;
            }
        }
        i += 1;
    }

    // Find a free entry
    i = 0;
    while i < KD_BREAKPOINT_MAX as ULONG {
        if KdpBreakpointTable[i as usize].Flags == 0 {
            break;
        }
        i += 1;
    }

    // Fail if no free entry was found
    if i == KD_BREAKPOINT_MAX as ULONG {
        return 0;
    }

    // Save the breakpoint
    KdpBreakpointTable[i as usize].Address = Address;

    // If we are setting the breakpoint in user space, save the active process context
    if Address < KD_HIGHEST_USER_BREAKPOINT_ADDRESS {
        KdpBreakpointTable[i as usize].DirectoryTableBase =
            (*(*KeGetCurrentThread()).__bindgen_anon_1.ApcState.Process).DirectoryTableBase[0] as ULONG_PTR;
    }

    // Try to save the old instruction
    status = KdpCopyMemoryChunks(
        Address as u64,
        core::ptr::addr_of_mut!(content) as PVOID,
        KD_BREAKPOINT_SIZE,
        0,
        MMDBG_COPY_UNSAFE,
        core::ptr::null_mut(),
    );
    
    if status >= 0 {
        // Memory accessible, set the breakpoint
        KdpBreakpointTable[i as usize].Content = content;
        KdpBreakpointTable[i as usize].Flags = KD_BREAKPOINT_ACTIVE;

        // Write the breakpoint
        status = KdpCopyMemoryChunks(
            Address as u64,
            core::ptr::addr_of_mut!(KdpBreakpointInstruction) as PVOID,
            KD_BREAKPOINT_SIZE,
            0,
            MMDBG_COPY_UNSAFE | MMDBG_COPY_WRITE,
            core::ptr::null_mut(),
        );
        
        if status < 0 {
            // This should never happen
            // crate::kdp_dprintf!(b"Unable to write breakpoint to address 0x%p\n\0", Address);
        }
    } else {
        // Memory is inaccessible now, setting breakpoint is deferred
        // KdpDprintf(
            // b"Failed to set breakpoint at address 0x%p, adding deferred breakpoint.\n\0".as_ptr() as PCHAR,
            // Address,
        // );
        KdpBreakpointTable[i as usize].Flags = KD_BREAKPOINT_ACTIVE | KD_BREAKPOINT_PENDING;
        KdpOweBreakpoint = 1;
    }

    // Return the breakpoint handle
    i + 1
}

#[no_mangle]
pub unsafe extern "system" fn KdSetOwedBreakpoints() {
    let enable: BOOLEAN;
    let mut content: KD_BREAKPOINT_TYPE = 0;
    let mut i: ULONG;
    let mut status: NTSTATUS;

    // If we don't owe any breakpoints, just return
    if KdpOweBreakpoint == 0 {
        return;
    }

    // Enter the debugger
    enable = KdEnterDebugger(core::ptr::null_mut(), core::ptr::null_mut());

    // Suppose we succeed in setting all the breakpoints.
    // If we fail to do so, the flag will be set again.
    KdpOweBreakpoint = 0;

    // Loop through current breakpoints and try to set or delete the pending ones
    i = 0;
    while i < KD_BREAKPOINT_MAX as ULONG {
        if (KdpBreakpointTable[i as usize].Flags & (KD_BREAKPOINT_PENDING | KD_BREAKPOINT_EXPIRED)) != 0 {
            // Set the breakpoint only if it is in kernel space, or if it is
            // in user space and the active process context matches.
            if KdpBreakpointTable[i as usize].Address < KD_HIGHEST_USER_BREAKPOINT_ADDRESS &&
               KdpBreakpointTable[i as usize].DirectoryTableBase != 
               (*(*KeGetCurrentThread()).__bindgen_anon_1.ApcState.Process).DirectoryTableBase[0] as ULONG_PTR
            {
                KdpOweBreakpoint = 1;
                i += 1;
                continue;
            }

            // Try to save the old instruction
            status = KdpCopyMemoryChunks(
                KdpBreakpointTable[i as usize].Address as u64,
                core::ptr::addr_of_mut!(content) as PVOID,
                KD_BREAKPOINT_SIZE,
                0,
                MMDBG_COPY_UNSAFE,
                core::ptr::null_mut(),
            );
            
            if status < 0 {
                // Memory is still inaccessible, breakpoint setting will be deferred again
                KdpOweBreakpoint = 1;
                i += 1;
                continue;
            }

            // Check if we need to write the breakpoint
            if (KdpBreakpointTable[i as usize].Flags & KD_BREAKPOINT_PENDING) != 0 {
                // Memory accessible, set the breakpoint
                KdpBreakpointTable[i as usize].Content = content;

                // Write the breakpoint
                status = KdpCopyMemoryChunks(
                    KdpBreakpointTable[i as usize].Address as u64,
                    core::ptr::addr_of_mut!(KdpBreakpointInstruction) as PVOID,
                    KD_BREAKPOINT_SIZE,
                    0,
                    MMDBG_COPY_UNSAFE | MMDBG_COPY_WRITE,
                    core::ptr::null_mut(),
                );
                
                if status < 0 {
                    // This should never happen
                    // KdpDprintf(
                        // b"Unable to write deferred breakpoint to address 0x%p\n\0".as_ptr() as PCHAR,
                        // KdpBreakpointTable[i as usize].Address,
                    // );
                    KdpOweBreakpoint = 1;
                } else {
                    KdpBreakpointTable[i as usize].Flags = KD_BREAKPOINT_ACTIVE;
                }

                i += 1;
                continue;
            }

            // Check if we need to restore the original instruction
            if (KdpBreakpointTable[i as usize].Flags & KD_BREAKPOINT_EXPIRED) != 0 {
                // Write it back
                status = KdpCopyMemoryChunks(
                    KdpBreakpointTable[i as usize].Address as u64,
                    core::ptr::addr_of_mut!(KdpBreakpointTable[i as usize].Content) as PVOID,
                    KD_BREAKPOINT_SIZE,
                    0,
                    MMDBG_COPY_UNSAFE | MMDBG_COPY_WRITE,
                    core::ptr::null_mut(),
                );
                
                if status < 0 {
                    // This should never happen
                    // KdpDprintf(
                        // b"Unable to delete deferred breakpoint at address 0x%p\n\0".as_ptr() as PCHAR,
                        // KdpBreakpointTable[i as usize].Address,
                    // );
                    KdpOweBreakpoint = 1;
                } else {
                    // Check if the breakpoint is suspended
                    if (KdpBreakpointTable[i as usize].Flags & KD_BREAKPOINT_SUSPENDED) != 0 {
                        KdpBreakpointTable[i as usize].Flags = KD_BREAKPOINT_SUSPENDED | KD_BREAKPOINT_ACTIVE;
                    } else {
                        // Invalidate it
                        KdpBreakpointTable[i as usize].Flags = 0;
                    }
                }

                i += 1;
                continue;
            }
        }
        i += 1;
    }

    // Exit the debugger
    KdExitDebugger(enable);
}

#[no_mangle]
pub unsafe extern "system" fn KdpLowWriteContent(BpIndex: ULONG) -> BOOLEAN {
    let status: NTSTATUS;

    // Make sure that the breakpoint is actually active
    if (KdpBreakpointTable[BpIndex as usize].Flags & KD_BREAKPOINT_PENDING) != 0 {
        // So we have a valid breakpoint, but it hasn't been used yet...
        KdpBreakpointTable[BpIndex as usize].Flags &= !KD_BREAKPOINT_PENDING;
        return 1;
    }

    // Is the original instruction a breakpoint anyway?
    if KdpBreakpointTable[BpIndex as usize].Content == KdpBreakpointInstruction {
        // Then leave it that way...
        return 1;
    }

    // We have an active breakpoint with an instruction to bring back. Do it.
    status = KdpCopyMemoryChunks(
        KdpBreakpointTable[BpIndex as usize].Address as u64,
        core::ptr::addr_of_mut!(KdpBreakpointTable[BpIndex as usize].Content) as PVOID,
        KD_BREAKPOINT_SIZE,
        0,
        MMDBG_COPY_UNSAFE | MMDBG_COPY_WRITE,
        core::ptr::null_mut(),
    );
    
    if status < 0 {
        // Memory is inaccessible now, restoring original instruction is deferred
        KdpBreakpointTable[BpIndex as usize].Flags |= KD_BREAKPOINT_EXPIRED;
        KdpOweBreakpoint = 1;
        return 0;
    }

    // Everything went fine, return
    1
}

#[no_mangle]
pub unsafe extern "system" fn KdpLowRestoreBreakpoint(BpIndex: ULONG) -> BOOLEAN {
    let status: NTSTATUS;

    // Were we not able to remove it earlier?
    if (KdpBreakpointTable[BpIndex as usize].Flags & KD_BREAKPOINT_EXPIRED) != 0 {
        // Just re-use it!
        KdpBreakpointTable[BpIndex as usize].Flags &= !KD_BREAKPOINT_EXPIRED;
        return 1;
    }

    // Are we merely writing a breakpoint on top of another breakpoint?
    if KdpBreakpointTable[BpIndex as usize].Content == KdpBreakpointInstruction {
        // Nothing to do
        return 1;
    }

    // Ok, we actually have to overwrite the instruction now
    status = KdpCopyMemoryChunks(
        KdpBreakpointTable[BpIndex as usize].Address as u64,
        core::ptr::addr_of_mut!(KdpBreakpointInstruction) as PVOID,
        KD_BREAKPOINT_SIZE,
        0,
        MMDBG_COPY_UNSAFE | MMDBG_COPY_WRITE,
        core::ptr::null_mut(),
    );
    
    if status < 0 {
        // Memory is inaccessible now, restoring breakpoint is deferred
        KdpBreakpointTable[BpIndex as usize].Flags |= KD_BREAKPOINT_PENDING;
        KdpOweBreakpoint = 1;
        return 0;
    }

    // Clear any possible previous pending flag and return success
    KdpBreakpointTable[BpIndex as usize].Flags &= !KD_BREAKPOINT_PENDING;
    1
}

#[no_mangle]
pub unsafe extern "system" fn KdpDeleteBreakpoint(BpEntry: ULONG) -> BOOLEAN {
    let bp_index = BpEntry - 1;

    // Check for invalid breakpoint entry
    if BpEntry == 0 || BpEntry > KD_BREAKPOINT_MAX as ULONG {
        return 0;
    }

    // If the specified breakpoint table entry is not valid, then return FALSE.
    if KdpBreakpointTable[bp_index as usize].Flags == 0 {
        return 0;
    }

    // Check if the breakpoint is suspended
    if (KdpBreakpointTable[bp_index as usize].Flags & KD_BREAKPOINT_SUSPENDED) != 0 {
        // Check if breakpoint is not being deleted
        if (KdpBreakpointTable[bp_index as usize].Flags & KD_BREAKPOINT_EXPIRED) == 0 {
            // Invalidate it and return success
            KdpBreakpointTable[bp_index as usize].Flags = 0;
            return 1;
        }
    }

    // Restore original data, then invalidate it and return success
    if KdpLowWriteContent(bp_index) != 0 {
        KdpBreakpointTable[bp_index as usize].Flags = 0;
    }
    1
}

#[no_mangle]
pub unsafe extern "system" fn KdpDeleteBreakpointRange(Base: PVOID, Limit: PVOID) -> BOOLEAN {
    let Base = opaque_ptr(Base);
    let Limit = opaque_ptr(Limit);
    let mut bp_index: ULONG;
    let mut deleted_breakpoints: BOOLEAN;

    // Assume no breakpoints will be deleted
    deleted_breakpoints = 0;

    // Loop the breakpoint table
    bp_index = 0;
    while bp_index < KD_BREAKPOINT_MAX as ULONG {
        // Make sure that the breakpoint is active and matches the range.
        if (KdpBreakpointTable[bp_index as usize].Flags & KD_BREAKPOINT_ACTIVE) != 0 &&
           KdpBreakpointTable[bp_index as usize].Address >= Base &&
           KdpBreakpointTable[bp_index as usize].Address <= Limit
        {
            // Delete it, and remember if we succeeded at least once
            if KdpDeleteBreakpoint(bp_index + 1) != 0 {
                deleted_breakpoints = 1;
            }
        }
        bp_index += 1;
    }

    // Return whether we deleted anything
    deleted_breakpoints
}

#[no_mangle]
pub unsafe extern "system" fn KdpRestoreAllBreakpoints() {
    let mut bp_index: ULONG;

    // No more suspended Breakpoints
    BreakpointsSuspended = 0;

    // Loop the breakpoints
    bp_index = 0;
    while bp_index < KD_BREAKPOINT_MAX as ULONG {
        // Check if they are valid, suspended breakpoints
        if (KdpBreakpointTable[bp_index as usize].Flags & KD_BREAKPOINT_ACTIVE) != 0 &&
           (KdpBreakpointTable[bp_index as usize].Flags & KD_BREAKPOINT_SUSPENDED) != 0
        {
            // Unsuspend them
            KdpBreakpointTable[bp_index as usize].Flags &= !KD_BREAKPOINT_SUSPENDED;
            KdpLowRestoreBreakpoint(bp_index);
        }
        bp_index += 1;
    }
}

#[no_mangle]
pub unsafe extern "system" fn KdpSuspendBreakPoint(BpEntry: ULONG) {
    let bp_index = BpEntry - 1;

    // Check if this is a valid, unsuspended breakpoint
    if (KdpBreakpointTable[bp_index as usize].Flags & KD_BREAKPOINT_ACTIVE) != 0 &&
       (KdpBreakpointTable[bp_index as usize].Flags & KD_BREAKPOINT_SUSPENDED) == 0
    {
        // Suspend it
        KdpBreakpointTable[bp_index as usize].Flags |= KD_BREAKPOINT_SUSPENDED;
        KdpLowWriteContent(bp_index);
    }
}

#[no_mangle]
pub unsafe extern "system" fn KdpSuspendAllBreakPoints() {
    let mut bp_entry: ULONG;

    // Breakpoints are suspended
    BreakpointsSuspended = 1;

    // Loop every breakpoint
    bp_entry = 1;
    while bp_entry <= KD_BREAKPOINT_MAX as ULONG {
        // Suspend it
        KdpSuspendBreakPoint(bp_entry);
        bp_entry += 1;
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

// BEGIN OFFSET CHECKS — auto-generated by verify_offsets.py, do not edit
#[cfg(target_pointer_width = "64")]
const _LAYOUT_CHECKS_64: () = {
    assert!(core::mem::size_of::<BREAKPOINT_ENTRY>() == 32);
    assert!(core::mem::offset_of!(BREAKPOINT_ENTRY, Flags) == 0);
    assert!(core::mem::offset_of!(BREAKPOINT_ENTRY, DirectoryTableBase) == 8);
    assert!(core::mem::offset_of!(BREAKPOINT_ENTRY, Address) == 16);
    assert!(core::mem::offset_of!(BREAKPOINT_ENTRY, Content) == 24);
};
#[cfg(target_pointer_width = "32")]
const _LAYOUT_CHECKS_32: () = {
    assert!(core::mem::size_of::<BREAKPOINT_ENTRY>() == 16);
    assert!(core::mem::offset_of!(BREAKPOINT_ENTRY, Flags) == 0);
    assert!(core::mem::offset_of!(BREAKPOINT_ENTRY, DirectoryTableBase) == 4);
    assert!(core::mem::offset_of!(BREAKPOINT_ENTRY, Address) == 8);
    assert!(core::mem::offset_of!(BREAKPOINT_ENTRY, Content) == 12);
};
// END OFFSET CHECKS
