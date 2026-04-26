use ntkrnl_sys::globals::*;
use ntkrnl_sys::base::*;
use ntkrnl_sys::constants::*;
use ntkrnl_sys::macros::*;
use ntkrnl_sys::ke::*;
use ntkrnl_sys::mm::*;
use ntkrnl_sys::interlocked::*;
use ntkrnl_sys::ex::*;
use ntkrnl_sys::rtl::*;

use ntkrnl_sys::ddk::{
    IO_REMOVE_LOCK_COMMON_BLOCK,
    IO_REMOVE_LOCK_DBG_BLOCK,
    PIO_REMOVE_LOCK,
    PEXTENDED_IO_REMOVE_LOCK,
    LPCSTR,
    LARGE_INTEGER,
    KeQueryTimeIncrement,
    KeInitializeEvent,
    KeInitializeSpinLock,
    KeWaitForSingleObject,
    _EVENT_TYPE_SynchronizationEvent,
    PRKEVENT,
    PKSPIN_LOCK as DdkPKSPIN_LOCK,
};
use ntkrnl_sys::inline::ke::KeInitializeSpinLock as KeInitializeSpinLockInline;
use ntkrnl_sys::{opaque_ptr, opaque_ptr_const};

// _IO_REMOVE_LOCK_TRACKING_BLOCK is opaque in bindgen, define it here
// with its real layout matching the C definition.
#[repr(C)]
struct IO_REMOVE_LOCK_TRACKING_BLOCK {
    Next: *mut IO_REMOVE_LOCK_TRACKING_BLOCK,
    Tag: PVOID,
    LockMoment: LARGE_INTEGER,
    File: LPCSTR,
    Line: ULONG,
}

// KeQueryTickCount is a FORCEINLINE macro — on i386 it reads the
// 32-bit tick count atomically; on 64-bit it's `*x = KeTickCount`.
// We use the exported function from the KE subsystem.
extern "system" {
    fn KeQueryTickCount(TickCount: *mut LARGE_INTEGER);
}

#[no_mangle]
pub unsafe extern "system" fn IoInitializeRemoveLockEx(
    RemoveLock: PIO_REMOVE_LOCK,
    AllocateTag: ULONG,
    MaxLockedMinutes: ULONG,
    HighWatermark: ULONG,
    RemlockSize: ULONG,
) {
    let Lock = RemoveLock as PEXTENDED_IO_REMOVE_LOCK;

    if Lock.is_null() {
        return;
    }

    match RemlockSize {
        size if size == (core::mem::size_of::<IO_REMOVE_LOCK_DBG_BLOCK>() + core::mem::size_of::<IO_REMOVE_LOCK_COMMON_BLOCK>()) as ULONG => {
            (*Lock).Dbg.Signature = 0x434F4C52i32;
            (*Lock).Dbg.HighWatermark = HighWatermark as LONG;
            (*Lock).Dbg.MaxLockedTicks = (KeQueryTimeIncrement() as i64) * (MaxLockedMinutes as i64) * 600000000i64;
            (*Lock).Dbg.AllocateTag = AllocateTag as LONG;
            KeInitializeSpinLockInline(core::ptr::addr_of_mut!((*Lock).Dbg.Spin) as *mut ULONG_PTR);
            (*Lock).Dbg.LowMemoryCount = 0;
            (*Lock).Dbg.Blocks = core::ptr::null_mut();

            (*Lock).Common.Removed = 0u8;
            (*Lock).Common.IoCount = 1;
            KeInitializeEvent(
                core::ptr::addr_of_mut!((*Lock).Common.RemoveEvent),
                _EVENT_TYPE_SynchronizationEvent,
                0u8,
            );
        }
        size if size == core::mem::size_of::<IO_REMOVE_LOCK_COMMON_BLOCK>() as ULONG => {
            (*Lock).Common.Removed = 0u8;
            (*Lock).Common.IoCount = 1;
            KeInitializeEvent(
                core::ptr::addr_of_mut!((*Lock).Common.RemoveEvent),
                _EVENT_TYPE_SynchronizationEvent,
                0u8,
            );
        }
        _ => {}
    }
}

#[no_mangle]
pub unsafe extern "system" fn IoAcquireRemoveLockEx(
    RemoveLock: PIO_REMOVE_LOCK,
    Tag: PVOID,
    File: LPCSTR,
    Line: ULONG,
    RemlockSize: ULONG,
) -> NTSTATUS {
    let Tag = opaque_ptr(Tag);
    let Lock = RemoveLock as PEXTENDED_IO_REMOVE_LOCK;

    let LockValue = InterlockedIncrement(core::ptr::addr_of_mut!((*Lock).Common.IoCount));

    if (*Lock).Common.Removed == 0 {
        if RemlockSize == (core::mem::size_of::<IO_REMOVE_LOCK_DBG_BLOCK>() + core::mem::size_of::<IO_REMOVE_LOCK_COMMON_BLOCK>()) as ULONG {
            let TrackingBlock = ExAllocatePoolWithTag(
                NonPagedPool,
                core::mem::size_of::<IO_REMOVE_LOCK_TRACKING_BLOCK>(),
                (*Lock).Dbg.AllocateTag as ULONG,
            ) as *mut IO_REMOVE_LOCK_TRACKING_BLOCK;

            if TrackingBlock.is_null() {
                InterlockedIncrement(core::ptr::addr_of_mut!((*Lock).Dbg.LowMemoryCount));
            } else {
                core::ptr::write_bytes(
                    TrackingBlock as *mut u8,
                    0,
                    core::mem::size_of::<IO_REMOVE_LOCK_TRACKING_BLOCK>(),
                );
                (*TrackingBlock).Tag = Tag;
                (*TrackingBlock).File = File;
                (*TrackingBlock).Line = Line;
                KeQueryTickCount(core::ptr::addr_of_mut!((*TrackingBlock).LockMoment));

                let mut OldIrql: KIRQL = 0;
                KeAcquireSpinLock(core::ptr::addr_of_mut!((*Lock).Dbg.Spin) as *mut ULONG_PTR, &mut OldIrql);
                (*TrackingBlock).Next = (*Lock).Dbg.Blocks as *mut IO_REMOVE_LOCK_TRACKING_BLOCK;
                (*Lock).Dbg.Blocks = TrackingBlock as *mut _;
                KeReleaseSpinLock(core::ptr::addr_of_mut!((*Lock).Dbg.Spin) as *mut ULONG_PTR, OldIrql);
            }
        }
    } else {
        if InterlockedDecrement(core::ptr::addr_of_mut!((*Lock).Common.IoCount)) == 0 {
            KeSetEvent(
                core::ptr::addr_of_mut!((*Lock).Common.RemoveEvent) as PVOID,
                0i32,
                0u8,
            );
        }

        return STATUS_DELETE_PENDING;
    }

    STATUS_SUCCESS
}

#[no_mangle]
pub unsafe extern "system" fn IoReleaseRemoveLockEx(
    RemoveLock: PIO_REMOVE_LOCK,
    Tag: PVOID,
    RemlockSize: ULONG,
) {
    let Tag = opaque_ptr(Tag);
    let Lock = RemoveLock as PEXTENDED_IO_REMOVE_LOCK;

    if RemlockSize == (core::mem::size_of::<IO_REMOVE_LOCK_DBG_BLOCK>() + core::mem::size_of::<IO_REMOVE_LOCK_COMMON_BLOCK>()) as ULONG {
        let mut OldIrql: KIRQL = 0;
        KeAcquireSpinLock(core::ptr::addr_of_mut!((*Lock).Dbg.Spin) as *mut ULONG_PTR, &mut OldIrql);

        let mut CurrentMoment: LARGE_INTEGER = core::mem::zeroed();
        KeQueryTickCount(core::ptr::addr_of_mut!(CurrentMoment));

        let mut TagFound = 0u8;
        let mut TrackingBlock = (*Lock).Dbg.Blocks as *mut IO_REMOVE_LOCK_TRACKING_BLOCK;
        let mut TrackingBlockLink = core::ptr::addr_of_mut!((*Lock).Dbg.Blocks) as *mut *mut IO_REMOVE_LOCK_TRACKING_BLOCK;

        while !TrackingBlock.is_null() {
            if (*Lock).Dbg.MaxLockedTicks != 0 &&
                CurrentMoment.QuadPart - (*TrackingBlock).LockMoment.QuadPart > (*Lock).Dbg.MaxLockedTicks {
            }

            if TagFound == 0 && (*TrackingBlock).Tag == Tag {
                TagFound = 1u8;
                *TrackingBlockLink = (*TrackingBlock).Next;
                ExFreePoolWithTag(TrackingBlock as PVOID, (*Lock).Dbg.AllocateTag as ULONG);
                TrackingBlock = *TrackingBlockLink;
            } else {
                TrackingBlockLink = core::ptr::addr_of_mut!((*TrackingBlock).Next);
                TrackingBlock = (*TrackingBlock).Next;
            }
        }

        KeReleaseSpinLock(core::ptr::addr_of_mut!((*Lock).Dbg.Spin) as *mut ULONG_PTR, OldIrql);

        if TagFound == 0 {
            if InterlockedDecrement(core::ptr::addr_of_mut!((*Lock).Dbg.LowMemoryCount)) < 0 {
                InterlockedIncrement(core::ptr::addr_of_mut!((*Lock).Dbg.LowMemoryCount));
            }
        }
    }

    let LockValue = InterlockedDecrement(core::ptr::addr_of_mut!((*Lock).Common.IoCount));

    if LockValue == 0 {
        KeSetEvent(
            core::ptr::addr_of_mut!((*Lock).Common.RemoveEvent) as PVOID,
            0i32,
            0u8,
        );
    }
}

#[no_mangle]
pub unsafe extern "system" fn IoReleaseRemoveLockAndWaitEx(
    RemoveLock: PIO_REMOVE_LOCK,
    Tag: PVOID,
    RemlockSize: ULONG,
) {
    let Tag = opaque_ptr(Tag);
    let Lock = RemoveLock as PEXTENDED_IO_REMOVE_LOCK;

    (*Lock).Common.Removed = 1u8;
    let LockValue = InterlockedDecrement(core::ptr::addr_of_mut!((*Lock).Common.IoCount));

    if InterlockedDecrement(core::ptr::addr_of_mut!((*Lock).Common.IoCount)) > 0 {
        KeWaitForSingleObject(
            core::ptr::addr_of_mut!((*Lock).Common.RemoveEvent) as PVOID,
            0u32,
            0i8,
            0u8,
            core::ptr::null_mut(),
        );
    }

    if RemlockSize == (core::mem::size_of::<IO_REMOVE_LOCK_DBG_BLOCK>() + core::mem::size_of::<IO_REMOVE_LOCK_COMMON_BLOCK>()) as ULONG {
        let TrackingBlock = (*Lock).Dbg.Blocks;

        ExFreePoolWithTag((*Lock).Dbg.Blocks as PVOID, (*Lock).Dbg.AllocateTag as ULONG);
    }
}


// ============================================================================
// ORIGINAL C SOURCE
// ============================================================================
//
// /*
//  * PROJECT:         ReactOS Kernel
//  * LICENSE:         GPL - See COPYING in the top level directory
//  * FILE:            ntoskrnl/io/iomgr/remlock.c
//  * PURPOSE:         Remove Lock Support
//  * PROGRAMMERS:     Alex Ionescu (alex.ionescu@reactos.org)
//  *                  Filip Navara (navaraf@reactos.org)
//  *                  Pierre Schweitzer (pierre.schweitzer@reactos.org)
//  */
//
// /* INCLUDES ******************************************************************/
//
// #include <ntoskrnl.h>
// #define NDEBUG
// #include <debug.h>
//
// typedef struct _IO_REMOVE_LOCK_TRACKING_BLOCK
// {
//     PIO_REMOVE_LOCK_TRACKING_BLOCK Next;
//     PVOID Tag;
//     LARGE_INTEGER LockMoment;
//     LPCSTR File;
//     ULONG Line;
// } IO_REMOVE_LOCK_TRACKING_BLOCK;
//
// /* FUNCTIONS *****************************************************************/
//
// /*
//  * @implemented
//  */
// VOID
// NTAPI
// IoInitializeRemoveLockEx(IN PIO_REMOVE_LOCK RemoveLock,
//                          IN ULONG AllocateTag,
//                          IN ULONG MaxLockedMinutes,
//                          IN ULONG HighWatermark,
//                          IN ULONG RemlockSize)
// {
//     PEXTENDED_IO_REMOVE_LOCK Lock = (PEXTENDED_IO_REMOVE_LOCK)RemoveLock;
//     PAGED_CODE();
//
//     DPRINT("%s(%p 0x%08x %u %u %u)\n", __FUNCTION__, RemoveLock, AllocateTag, MaxLockedMinutes, HighWatermark, RemlockSize);
//
//     ASSERT(HighWatermark < MAXLONG);
//
//     /* If no lock given, nothing to do */
//     if (!Lock)
//     {
//         return;
//     }
//
//     switch (RemlockSize)
//     {
//         /* Check if this is a debug lock */
//         case (sizeof(IO_REMOVE_LOCK_DBG_BLOCK) + sizeof(IO_REMOVE_LOCK_COMMON_BLOCK)):
//             /* Setup debug parameters */
//             Lock->Dbg.Signature = 'COLR';
//             Lock->Dbg.HighWatermark = HighWatermark;
//             Lock->Dbg.MaxLockedTicks = KeQueryTimeIncrement() * MaxLockedMinutes * 600000000;
//             Lock->Dbg.AllocateTag = AllocateTag;
//             KeInitializeSpinLock(&(Lock->Dbg.Spin));
//             Lock->Dbg.LowMemoryCount = 0;
//             Lock->Dbg.Blocks = NULL;
//
//         case sizeof(IO_REMOVE_LOCK_COMMON_BLOCK):
//             /* Setup a free block */
//             Lock->Common.Removed = FALSE;
//             Lock->Common.IoCount = 1;
//             KeInitializeEvent(&Lock->Common.RemoveEvent,
//                               SynchronizationEvent,
//                               FALSE);
//     }
// }
//
// /*
//  * @implemented
//  */
// NTSTATUS
// NTAPI
// IoAcquireRemoveLockEx(IN PIO_REMOVE_LOCK RemoveLock,
//                       IN OPTIONAL PVOID Tag,
//                       IN LPCSTR File,
//                       IN ULONG Line,
//                       IN ULONG RemlockSize)
// {
//     KIRQL OldIrql;
//     LONG LockValue;
//     PIO_REMOVE_LOCK_TRACKING_BLOCK TrackingBlock;
//     PEXTENDED_IO_REMOVE_LOCK Lock = (PEXTENDED_IO_REMOVE_LOCK)RemoveLock;
//
//     DPRINT("%s(%p %p %s %u %u)\n", __FUNCTION__, RemoveLock, Tag, File, Line, RemlockSize);
//
//     /* Increase the lock count */
//     LockValue = InterlockedIncrement(&(Lock->Common.IoCount));
//     ASSERT(LockValue > 0);
//     if (!Lock->Common.Removed)
//     {
//         /* Check what kind of lock this is */
//         if (RemlockSize == (sizeof(IO_REMOVE_LOCK_DBG_BLOCK) + sizeof(IO_REMOVE_LOCK_COMMON_BLOCK)))
//         {
//             ASSERT(Lock->Dbg.HighWatermark == 0 || LockValue <= Lock->Dbg.HighWatermark);
//
//             /* Allocate a tracking block */
//             TrackingBlock = ExAllocatePoolWithTag(NonPagedPool, sizeof(IO_REMOVE_LOCK_TRACKING_BLOCK), Lock->Dbg.AllocateTag);
//             if (!TrackingBlock)
//             {
//                 /* Keep count of failures for lock release and missing tags */
//                 InterlockedIncrement(&(Lock->Dbg.LowMemoryCount));
//             }
//             else
//             {
//                 /* Initialize block */
//                 RtlZeroMemory(TrackingBlock, sizeof(IO_REMOVE_LOCK_TRACKING_BLOCK));
//                 TrackingBlock->Tag = Tag;
//                 TrackingBlock->File = File;
//                 TrackingBlock->Line = Line;
//                 KeQueryTickCount(&(TrackingBlock->LockMoment));
//
//                 /* Queue the block */
//                 KeAcquireSpinLock(&(Lock->Dbg.Spin), &OldIrql);
//                 TrackingBlock->Next = Lock->Dbg.Blocks;
//                 Lock->Dbg.Blocks = TrackingBlock;
//                 KeReleaseSpinLock(&(Lock->Dbg.Spin), OldIrql);
//             }
//         }
//     }
//     else
//     {
//         /* Otherwise, decrement the count and check if it's gone */
//         if (!InterlockedDecrement(&(Lock->Common.IoCount)))
//         {
//             /* Signal the event */
//             KeSetEvent(&(Lock->Common.RemoveEvent), IO_NO_INCREMENT, FALSE);
//         }
//
//         /* Return pending delete */
//         return STATUS_DELETE_PENDING;
//     }
//
//     /* Otherwise, return success */
//     return STATUS_SUCCESS;
// }
//
// /*
//  * @implemented
//  */
// VOID
// NTAPI
// IoReleaseRemoveLockEx(IN PIO_REMOVE_LOCK RemoveLock,
//                       IN PVOID Tag,
//                       IN ULONG RemlockSize)
// {
//     KIRQL OldIrql;
//     LONG LockValue;
//     BOOLEAN TagFound;
//     LARGE_INTEGER CurrentMoment;
//     PIO_REMOVE_LOCK_TRACKING_BLOCK TrackingBlock;
//     PIO_REMOVE_LOCK_TRACKING_BLOCK *TrackingBlockLink;
//     PEXTENDED_IO_REMOVE_LOCK Lock = (PEXTENDED_IO_REMOVE_LOCK)RemoveLock;
//
//     DPRINT("%s(%p %p %u)\n", __FUNCTION__, RemoveLock, Tag, RemlockSize);
//
//     /* Check what kind of lock this is */
//     if (RemlockSize == (sizeof(IO_REMOVE_LOCK_DBG_BLOCK) + sizeof(IO_REMOVE_LOCK_COMMON_BLOCK)))
//     {
//         /* Acquire blocks queue */
//         KeAcquireSpinLock(&(Lock->Dbg.Spin), &OldIrql);
//
//         /* Get the release moment */
//         KeQueryTickCount(&CurrentMoment);
//
//         /* Start browsing tracking blocks to find a block that would match given tag */
//         TagFound = FALSE;
//         TrackingBlock = Lock->Dbg.Blocks;
//         TrackingBlockLink = &(Lock->Dbg.Blocks);
//         while (TrackingBlock != NULL)
//         {
//             /* First of all, check if the lock was locked for too long */
//             if (Lock->Dbg.MaxLockedTicks &&
//                 CurrentMoment.QuadPart - TrackingBlock->LockMoment.QuadPart > Lock->Dbg.MaxLockedTicks)
//             {
//                 DPRINT("Lock %#08lx (with tag %#08lx) was supposed to be held at max %I64d ticks but lasted longer\n",
//                        Lock, TrackingBlock->Tag, Lock->Dbg.MaxLockedTicks);
//                 DPRINT("Lock was acquired in file %s at line %lu\n", TrackingBlock->File, TrackingBlock->Line);
//                 ASSERT(FALSE);
//             }
//
//             /* Check if this is the first matching tracking block */
//             if ((TagFound == FALSE) && (TrackingBlock->Tag == Tag))
//             {
//                 /* Unlink this tracking block, and free it */
//                 TagFound = TRUE;
//                 *TrackingBlockLink = TrackingBlock->Next;
//                 ExFreePoolWithTag(TrackingBlock, Lock->Dbg.AllocateTag);
//                 TrackingBlock = *TrackingBlockLink;
//             }
//             else
//             {
//                 /* Go to the next tracking block */
//                 TrackingBlockLink = &(TrackingBlock->Next);
//                 TrackingBlock = TrackingBlock->Next;
//             }
//         }
//
//         /* We're done, release queue lock */
//         KeReleaseSpinLock(&(Lock->Dbg.Spin), OldIrql);
//
//         /* If we didn't find any matching block */
//         if (TagFound == FALSE)
//         {
//             /* Check if it was because we were low in memory
//              * If yes, then ignore, that's normal
//              */
//             if (InterlockedDecrement(&(Lock->Dbg.LowMemoryCount)) < 0)
//             {
//                 /* Otherwise signal the issue, it shouldn't happen */
//                 InterlockedIncrement(&(Lock->Dbg.LowMemoryCount));
//                 DPRINT("Failed finding block for tag: %#08lx\n", Tag);
//                 ASSERT(FALSE);
//             }
//         }
//     }
//
//     /* Decrement the lock count */
//     LockValue = InterlockedDecrement(&(Lock->Common.IoCount));
//     ASSERT(LockValue >= 0);
//
//     if (!LockValue)
//     {
//         /* Someone should be waiting... */
//         ASSERT(Lock->Common.Removed);
//
//         /* Signal the event */
//         KeSetEvent(&(Lock->Common.RemoveEvent), IO_NO_INCREMENT, FALSE);
//     }
// }
//
// /*
//  * @implemented
//  */
// VOID
// NTAPI
// IoReleaseRemoveLockAndWaitEx(IN PIO_REMOVE_LOCK RemoveLock,
//                              IN PVOID Tag,
//                              IN ULONG RemlockSize)
// {
//     LONG LockValue;
//     PIO_REMOVE_LOCK_TRACKING_BLOCK TrackingBlock;
//     PEXTENDED_IO_REMOVE_LOCK Lock = (PEXTENDED_IO_REMOVE_LOCK)RemoveLock;
//     PAGED_CODE();
//
//     DPRINT("%s(%p %p %u)\n", __FUNCTION__, RemoveLock, Tag, RemlockSize);
//
//     /* Remove the lock and decrement the count */
//     Lock->Common.Removed = TRUE;
//     LockValue = InterlockedDecrement(&Lock->Common.IoCount);
//     ASSERT(LockValue > 0);
//
//     /* If we are still > 0, then wait for the others to remove lock */
//     if (InterlockedDecrement(&Lock->Common.IoCount) > 0)
//     {
//         /* Wait for it */
//         KeWaitForSingleObject(&(Lock->Common.RemoveEvent),
//                               Executive,
//                               KernelMode,
//                               FALSE,
//                               NULL);
//     }
//
//     /* Check what kind of lock this is */
//     if (RemlockSize == (sizeof(IO_REMOVE_LOCK_DBG_BLOCK) + sizeof(IO_REMOVE_LOCK_COMMON_BLOCK)))
//     {
//         /* A block must be remaining */
//         ASSERT(Lock->Dbg.Blocks);
//
//         /* Get it */
//         TrackingBlock = Lock->Dbg.Blocks;
//         /* Tag should match */
//         if (TrackingBlock->Tag != Tag)
//         {
//             DPRINT("Last tracking block tag invalid! Expected: %p, having: %p\n", Tag, TrackingBlock->Tag);
//             ASSERT(TrackingBlock->Tag != Tag);
//         }
//
//         /* Release block */
//         ExFreePoolWithTag(Lock->Dbg.Blocks, Lock->Dbg.AllocateTag);
//     }
// }
//
// /* EOF */
//
// ============================================================================
// END ORIGINAL C SOURCE
// ============================================================================
