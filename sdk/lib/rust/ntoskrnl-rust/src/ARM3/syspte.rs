use ntkrnl_sys::globals::*;
use ntkrnl_sys::constants::*;
use ntkrnl_sys::base::*;
use ntkrnl_sys::ke::*;
use ntkrnl_sys::mm::*;
use ntkrnl_sys::rtl::*;
use ntkrnl_sys::ddk::KeFlushProcessTb;

#[no_mangle]
pub static mut MmSystemPteBase: PMMPTE = core::ptr::null_mut();

#[no_mangle]
pub static mut MmSystemPtesStart: [PMMPTE; 2] = [core::ptr::null_mut(); 2];

#[no_mangle]
pub static mut MmSystemPtesEnd: [PMMPTE; 2] = [core::ptr::null_mut(); 2];

#[no_mangle]
pub static mut MmFirstFreeSystemPte: [MMPTE; 2] = unsafe { [core::mem::zeroed(); 2] };

#[no_mangle]
pub static mut MmTotalFreeSystemPtes: [ULONG; 2] = [0; 2];

#[no_mangle]
pub static mut MmTotalSystemPtes: ULONG = 0;

#[no_mangle]
pub static mut MiNumberOfExtraSystemPdes: ULONG = 0;

static MM_SYS_PTE_INDEX: [ULONG; 5] = [1, 2, 4, 8, 16];

static MM_SYS_PTE_TABLES: [UCHAR; 17] = [
    0, 0, 1, 2, 2, 3, 3, 3, 3, 4, 4, 4, 4, 4, 4, 4, 4,
];

#[no_mangle]
pub static mut MmSysPteListBySizeCount: [LONG; 5] = [0; 5];

#[inline(always)]
unsafe fn mi_get_cluster_size(pte: PMMPTE) -> ULONG {
    if (*pte).list_one_entry() != 0 {
        return 1;
    }

    let next_pte = pte.add(1);
    (*next_pte).list_next_entry() as ULONG
}

pub const SystemPteSpace: MMSYSTEM_PTE_POOL_TYPE = 0;

#[no_mangle]
pub unsafe extern "system" fn MiReserveAlignedSystemPtes(
    number_of_ptes: ULONG,
    system_pte_pool_type: MMSYSTEM_PTE_POOL_TYPE,
    alignment: ULONG,
) -> PMMPTE {
    let mut previous_pte: PMMPTE;
    let mut next_pte: PMMPTE = core::ptr::null_mut();
    let return_pte: PMMPTE;
    let mut cluster_size: ULONG = 0;

    let old_irql = KeAcquireQueuedSpinLock(LockQueueSystemSpaceLock);

    previous_pte = core::ptr::addr_of_mut!(MmFirstFreeSystemPte[system_pte_pool_type as usize]);

    while (*previous_pte).list_next_entry() != MM_EMPTY_PTE_LIST {
        next_pte = MmSystemPteBase.add((*previous_pte).list_next_entry() as usize);
        cluster_size = mi_get_cluster_size(next_pte);

        if number_of_ptes <= cluster_size {
            break;
        }

        previous_pte = next_pte;
    }

    if (*previous_pte).list_next_entry() == MM_EMPTY_PTE_LIST {
        KeReleaseQueuedSpinLock(LockQueueSystemSpaceLock, old_irql);
        return core::ptr::null_mut();
    }

    (*previous_pte).set_list_next_entry((*next_pte).list_next_entry() as ULONG_PTR);

    if cluster_size == number_of_ptes {
        return_pte = next_pte;

        if (*next_pte).list_one_entry() == 0 {
            (*next_pte).Long = 0;
            next_pte = next_pte.add(1);
        }
        (*next_pte).Long = 0;
    } else {
        cluster_size -= number_of_ptes;
        return_pte = next_pte.add(cluster_size as usize);

        if cluster_size == 1 {
            (*next_pte).set_list_one_entry(1);
            (*return_pte).Long = 0;
        } else {
            next_pte = next_pte.add(1);
            (*next_pte).set_list_next_entry(cluster_size as u64 as ULONG_PTR);
        }

        previous_pte = core::ptr::addr_of_mut!(MmFirstFreeSystemPte[system_pte_pool_type as usize]);

        while (*previous_pte).list_next_entry() != MM_EMPTY_PTE_LIST {
            next_pte = MmSystemPteBase.add((*previous_pte).list_next_entry() as usize);

            if cluster_size <= mi_get_cluster_size(next_pte) {
                break;
            }

            previous_pte = next_pte;
        }

        next_pte = return_pte.sub(cluster_size as usize);

        (*next_pte).set_list_next_entry((*previous_pte).list_next_entry() as ULONG_PTR);
        (*previous_pte).set_list_next_entry(next_pte.offset_from(MmSystemPteBase) as ULONG_PTR as ULONG_PTR);
    }

    MmTotalFreeSystemPtes[system_pte_pool_type as usize] -= number_of_ptes;

    KeReleaseQueuedSpinLock(LockQueueSystemSpaceLock, old_irql);

    KeFlushProcessTb();

    return_pte
}

#[no_mangle]
pub unsafe extern "system" fn MiReserveSystemPtes(
    number_of_ptes: ULONG,
    system_pte_pool_type: MMSYSTEM_PTE_POOL_TYPE,
) -> PMMPTE {
    MiReserveAlignedSystemPtes(number_of_ptes, system_pte_pool_type, 0)
}

#[no_mangle]
pub unsafe extern "system" fn MiReleaseSystemPtes(
    mut starting_pte: PMMPTE,
    mut number_of_ptes: ULONG,
    system_pte_pool_type: MMSYSTEM_PTE_POOL_TYPE,
) {
    let mut cluster_size: ULONG;
    let mut previous_pte: PMMPTE;
    let mut next_pte: PMMPTE;
    let mut insert_pte: PMMPTE;

    // Bounds check: verify starting_pte is within the system PTE range
    if starting_pte < MmSystemPtesStart[system_pte_pool_type as usize]
        || starting_pte.add(number_of_ptes as usize - 1) > MmSystemPtesEnd[system_pte_pool_type as usize]
    {
        KeBugCheckEx(0x50, starting_pte as ULONG_PTR, number_of_ptes as ULONG_PTR,
            MmSystemPtesStart[system_pte_pool_type as usize] as ULONG_PTR,
            MmSystemPtesEnd[system_pte_pool_type as usize] as ULONG_PTR);
    }

    RtlZeroMemory(
        starting_pte as PVOID,
        (number_of_ptes as usize * core::mem::size_of::<MMPTE>()) as SIZE_T,
    );

    let old_irql = KeAcquireQueuedSpinLock(LockQueueSystemSpaceLock);

    MmTotalFreeSystemPtes[system_pte_pool_type as usize] += number_of_ptes;

    previous_pte = core::ptr::addr_of_mut!(MmFirstFreeSystemPte[system_pte_pool_type as usize]);
    insert_pte = core::ptr::null_mut();

    while (*previous_pte).list_next_entry() != MM_EMPTY_PTE_LIST {
        next_pte = MmSystemPteBase.add((*previous_pte).list_next_entry() as usize);
        cluster_size = mi_get_cluster_size(next_pte);

        if next_pte.add(cluster_size as usize) == starting_pte
            || starting_pte.add(number_of_ptes as usize) == next_pte
        {
            number_of_ptes += cluster_size;

            if next_pte < starting_pte {
                starting_pte = next_pte;
            }

            (*previous_pte).set_list_next_entry((*next_pte).list_next_entry() as ULONG_PTR);

            if (*next_pte).list_one_entry() == 0 {
                (*next_pte).Long = 0;
                next_pte = next_pte.add(1);
            }
            (*next_pte).Long = 0;

            insert_pte = core::ptr::null_mut();
        } else {
            if insert_pte.is_null() && number_of_ptes <= cluster_size {
                insert_pte = previous_pte;
            }

            previous_pte = next_pte;
        }
    }

    if insert_pte.is_null() {
        insert_pte = previous_pte;
    }

    if number_of_ptes != 1 {
        (*starting_pte).set_list_one_entry(0);

        next_pte = starting_pte.add(1);
        (*next_pte).set_list_next_entry(number_of_ptes as u64 as ULONG_PTR);
    } else {
        (*starting_pte).set_list_one_entry(1);
    }

    (*starting_pte).set_list_next_entry((*insert_pte).list_next_entry() as ULONG_PTR);
    (*insert_pte).set_list_next_entry(starting_pte.offset_from(MmSystemPteBase) as ULONG_PTR as ULONG_PTR);

    KeReleaseQueuedSpinLock(LockQueueSystemSpaceLock, old_irql);
}

#[no_mangle]
pub unsafe extern "system" fn MiInitializeSystemPtes(
    starting_pte: PMMPTE,
    number_of_ptes: ULONG,
    pool_type: MMSYSTEM_PTE_POOL_TYPE,
) {
    MmSystemPteBase = MI_SYSTEM_PTE_BASE as PMMPTE;
    MmSystemPtesStart[pool_type as usize] = starting_pte;
    MmSystemPtesEnd[pool_type as usize] = starting_pte.add(number_of_ptes as usize - 1);

    RtlZeroMemory(
        starting_pte as PVOID,
        (number_of_ptes as usize * core::mem::size_of::<MMPTE>()) as SIZE_T,
    );

    (*starting_pte).set_list_next_entry(MM_EMPTY_PTE_LIST as ULONG_PTR);
    MmFirstFreeSystemPte[pool_type as usize].Long = 0;
    MmFirstFreeSystemPte[pool_type as usize].set_list_next_entry(starting_pte.offset_from(MmSystemPteBase) as ULONG_PTR as ULONG_PTR);

    let next_pte = starting_pte.add(1);
    (*next_pte).Long = 0;
    (*next_pte).set_list_next_entry(number_of_ptes as u64 as ULONG_PTR);

    MmTotalFreeSystemPtes[pool_type as usize] = number_of_ptes;

    if pool_type == SystemPteSpace {
        MmTotalSystemPtes = number_of_ptes;
    }
}


// ============================================================================
// ORIGINAL C SOURCE
// ============================================================================
//
// /*
//  * PROJECT:         ReactOS Kernel
//  * LICENSE:         BSD - See COPYING.ARM in the top level directory
//  * FILE:            ntoskrnl/mm/ARM3/syspte.c
//  * PURPOSE:         ARM Memory Manager System PTE Allocator
//  * PROGRAMMERS:     ReactOS Portable Systems Group
//  *                  Roel Messiant (roel.messiant@reactos.org)
//  */
//
// /* INCLUDES *******************************************************************/
//
// #include <ntoskrnl.h>
// #define NDEBUG
// #include <debug.h>
//
// #define MODULE_INVOLVED_IN_ARM3
// #include <mm/ARM3/miarm.h>
//
// /* GLOBALS ********************************************************************/
//
// PMMPTE MmSystemPteBase;
// PMMPTE MmSystemPtesStart[MaximumPtePoolTypes];
// PMMPTE MmSystemPtesEnd[MaximumPtePoolTypes];
// MMPTE MmFirstFreeSystemPte[MaximumPtePoolTypes];
// ULONG MmTotalFreeSystemPtes[MaximumPtePoolTypes];
// ULONG MmTotalSystemPtes;
// ULONG MiNumberOfExtraSystemPdes;
// const ULONG MmSysPteIndex[5] = { 1, 2, 4, 8, 16 };
// const UCHAR MmSysPteTables[] = { 0, // 1
//                                  0, // 1
//                                  1, // 2
//                                  2, 2, // 4
//                                  3, 3, 3, 3, // 8
//                                  4, 4, 4, 4, 4, 4, 4, 4 // 16
//                                };
// LONG MmSysPteListBySizeCount[5];
//
// /* PRIVATE FUNCTIONS **********************************************************/
//
// //
// // The free System Page Table Entries are stored in a bunch of clusters,
// // each consisting of one or more PTEs.  These PTE clusters are connected
// // in a singly linked list, ordered by increasing cluster size.
// //
// // A cluster consisting of a single PTE is marked by having the OneEntry flag
// // of its PTE set.  The forward link is contained in the NextEntry field.
// //
// // Clusters containing multiple PTEs have the OneEntry flag of their first PTE
// // reset.  The NextEntry field of the first PTE contains the forward link, and
// // the size of the cluster is stored in the NextEntry field of its second PTE.
// //
// // Reserving PTEs currently happens by walking the linked list until a cluster
// // is found that contains the requested amount of PTEs or more.  This cluster
// // is removed from the list, and the requested amount of PTEs is taken from the
// // tail of this cluster.  If any PTEs remain in the cluster, the linked list is
// // walked again until a second cluster is found that contains the same amount
// // of PTEs or more.  The first cluster is then inserted in front of the second
// // one.
// //
// // Releasing PTEs currently happens by walking the whole linked list, recording
// // the first cluster that contains the amount of PTEs to release or more. When
// // a cluster is found that is adjacent to the PTEs being released, this cluster
// // is removed from the list and subsequently added to the PTEs being released.
// // This ensures no two clusters are adjacent, which maximizes their size.
// // After the walk is complete, a new cluster is created that contains the PTEs
// // being released, which is then inserted in front of the recorded cluster.
// //
//
// FORCEINLINE
// ULONG
// MI_GET_CLUSTER_SIZE(IN PMMPTE Pte)
// {
//     //
//     // First check for a single PTE
//     //
//     if (Pte->u.List.OneEntry)
//         return 1;
//
//     //
//     // Then read the size from the trailing PTE
//     //
//     Pte++;
//     return (ULONG)Pte->u.List.NextEntry;
// }
//
// PMMPTE
// NTAPI
// MiReserveAlignedSystemPtes(IN ULONG NumberOfPtes,
//                            IN MMSYSTEM_PTE_POOL_TYPE SystemPtePoolType,
//                            IN ULONG Alignment)
// {
//     KIRQL OldIrql;
//     PMMPTE PreviousPte, NextPte, ReturnPte;
//     ULONG ClusterSize;
//
//     //
//     // Sanity check
//     //
//     ASSERT(Alignment <= PAGE_SIZE);
//
//     //
//     // Acquire the System PTE lock
//     //
//     OldIrql = KeAcquireQueuedSpinLock(LockQueueSystemSpaceLock);
//
//     //
//     // Find the last cluster in the list that doesn't contain enough PTEs
//     //
//     PreviousPte = &MmFirstFreeSystemPte[SystemPtePoolType];
//
//     while (PreviousPte->u.List.NextEntry != MM_EMPTY_PTE_LIST)
//     {
//         //
//         // Get the next cluster and its size
//         //
//         NextPte = MmSystemPteBase + PreviousPte->u.List.NextEntry;
//         ClusterSize = MI_GET_CLUSTER_SIZE(NextPte);
//
//         //
//         // Check if this cluster contains enough PTEs
//         //
//         if (NumberOfPtes <= ClusterSize)
//             break;
//
//         //
//         // On to the next cluster
//         //
//         PreviousPte = NextPte;
//     }
//
//     //
//     // Make sure we didn't reach the end of the cluster list
//     //
//     if (PreviousPte->u.List.NextEntry == MM_EMPTY_PTE_LIST)
//     {
//         //
//         // Release the System PTE lock and return failure
//         //
//         KeReleaseQueuedSpinLock(LockQueueSystemSpaceLock, OldIrql);
//         return NULL;
//     }
//
//     //
//     // Unlink the cluster
//     //
//     PreviousPte->u.List.NextEntry = NextPte->u.List.NextEntry;
//
//     //
//     // Check if the reservation spans the whole cluster
//     //
//     if (ClusterSize == NumberOfPtes)
//     {
//         //
//         // Return the first PTE of this cluster
//         //
//         ReturnPte = NextPte;
//
//         //
//         // Zero the cluster
//         //
//         if (NextPte->u.List.OneEntry == 0)
//         {
//             NextPte->u.Long = 0;
//             NextPte++;
//         }
//         NextPte->u.Long = 0;
//     }
//     else
//     {
//         //
//         // Divide the cluster into two parts
//         //
//         ClusterSize -= NumberOfPtes;
//         ReturnPte = NextPte + ClusterSize;
//
//         //
//         // Set the size of the first cluster, zero the second if needed
//         //
//         if (ClusterSize == 1)
//         {
//             NextPte->u.List.OneEntry = 1;
//             ReturnPte->u.Long = 0;
//         }
//         else
//         {
//             NextPte++;
//             NextPte->u.List.NextEntry = ClusterSize;
//         }
//
//         //
//         // Step through the cluster list to find out where to insert the first
//         //
//         PreviousPte = &MmFirstFreeSystemPte[SystemPtePoolType];
//
//         while (PreviousPte->u.List.NextEntry != MM_EMPTY_PTE_LIST)
//         {
//             //
//             // Get the next cluster
//             //
//             NextPte = MmSystemPteBase + PreviousPte->u.List.NextEntry;
//
//             //
//             // Check if the cluster to insert is smaller or of equal size
//             //
//             if (ClusterSize <= MI_GET_CLUSTER_SIZE(NextPte))
//                 break;
//
//             //
//             // On to the next cluster
//             //
//             PreviousPte = NextPte;
//         }
//
//         //
//         // Retrieve the first cluster and link it back into the cluster list
//         //
//         NextPte = ReturnPte - ClusterSize;
//
//         NextPte->u.List.NextEntry = PreviousPte->u.List.NextEntry;
//         PreviousPte->u.List.NextEntry = NextPte - MmSystemPteBase;
//     }
//
//     //
//     // Decrease availability
//     //
//     MmTotalFreeSystemPtes[SystemPtePoolType] -= NumberOfPtes;
//
//     //
//     // Release the System PTE lock
//     //
//     KeReleaseQueuedSpinLock(LockQueueSystemSpaceLock, OldIrql);
//
//     //
//     // Flush the TLB
//     //
//     KeFlushProcessTb();
//
//     //
//     // Return the reserved PTEs
//     //
//     return ReturnPte;
// }
//
// PMMPTE
// NTAPI
// MiReserveSystemPtes(IN ULONG NumberOfPtes,
//                     IN MMSYSTEM_PTE_POOL_TYPE SystemPtePoolType)
// {
//     PMMPTE PointerPte;
//
//     //
//     // Use the extended function
//     //
//     PointerPte = MiReserveAlignedSystemPtes(NumberOfPtes, SystemPtePoolType, 0);
//
//     //
//     // Return the PTE Pointer
//     //
//     return PointerPte;
// }
//
// VOID
// NTAPI
// MiReleaseSystemPtes(IN PMMPTE StartingPte,
//                     IN ULONG NumberOfPtes,
//                     IN MMSYSTEM_PTE_POOL_TYPE SystemPtePoolType)
// {
//     KIRQL OldIrql;
//     ULONG ClusterSize;
//     PMMPTE PreviousPte, NextPte, InsertPte;
//
//     //
//     // Check to make sure the PTE address is within bounds
//     //
//     ASSERT(NumberOfPtes != 0);
//     ASSERT(StartingPte >= MmSystemPtesStart[SystemPtePoolType]);
//     ASSERT(StartingPte + NumberOfPtes - 1 <= MmSystemPtesEnd[SystemPtePoolType]);
//
//     //
//     // Zero PTEs
//     //
//     RtlZeroMemory(StartingPte, NumberOfPtes * sizeof(MMPTE));
//
//     //
//     // Acquire the System PTE lock
//     //
//     OldIrql = KeAcquireQueuedSpinLock(LockQueueSystemSpaceLock);
//
//     //
//     // Increase availability
//     //
//     MmTotalFreeSystemPtes[SystemPtePoolType] += NumberOfPtes;
//
//     //
//     // Step through the cluster list to find where to insert the PTEs
//     //
//     PreviousPte = &MmFirstFreeSystemPte[SystemPtePoolType];
//     InsertPte = NULL;
//
//     while (PreviousPte->u.List.NextEntry != MM_EMPTY_PTE_LIST)
//     {
//         //
//         // Get the next cluster and its size
//         //
//         NextPte = MmSystemPteBase + PreviousPte->u.List.NextEntry;
//         ClusterSize = MI_GET_CLUSTER_SIZE(NextPte);
//
//         //
//         // Check if this cluster is adjacent to the PTEs being released
//         //
//         if ((NextPte + ClusterSize == StartingPte) ||
//             (StartingPte + NumberOfPtes == NextPte))
//         {
//             //
//             // Add the PTEs in the cluster to the PTEs being released
//             //
//             NumberOfPtes += ClusterSize;
//
//             if (NextPte < StartingPte)
//                 StartingPte = NextPte;
//
//             //
//             // Unlink this cluster and zero it
//             //
//             PreviousPte->u.List.NextEntry = NextPte->u.List.NextEntry;
//
//             if (NextPte->u.List.OneEntry == 0)
//             {
//                 NextPte->u.Long = 0;
//                 NextPte++;
//             }
//             NextPte->u.Long = 0;
//
//             //
//             // Invalidate the previously found insertion location, if any
//             //
//             InsertPte = NULL;
//         }
//         else
//         {
//             //
//             // Check if the insertion location is right before this cluster
//             //
//             if ((InsertPte == NULL) && (NumberOfPtes <= ClusterSize))
//                 InsertPte = PreviousPte;
//
//             //
//             // On to the next cluster
//             //
//             PreviousPte = NextPte;
//         }
//     }
//
//     //
//     // If no insertion location was found, use the tail of the list
//     //
//     if (InsertPte == NULL)
//         InsertPte = PreviousPte;
//
//     //
//     // Create a new cluster using the PTEs being released
//     //
//     if (NumberOfPtes != 1)
//     {
//         StartingPte->u.List.OneEntry = 0;
//
//         NextPte = StartingPte + 1;
//         NextPte->u.List.NextEntry = NumberOfPtes;
//     }
//     else
//         StartingPte->u.List.OneEntry = 1;
//
//     //
//     // Link the new cluster into the cluster list at the insertion location
//     //
//     StartingPte->u.List.NextEntry = InsertPte->u.List.NextEntry;
//     InsertPte->u.List.NextEntry = StartingPte - MmSystemPteBase;
//
//     //
//     // Release the System PTE lock
//     //
//     KeReleaseQueuedSpinLock(LockQueueSystemSpaceLock, OldIrql);
// }
//
// CODE_SEG("INIT")
// VOID
// NTAPI
// MiInitializeSystemPtes(IN PMMPTE StartingPte,
//                        IN ULONG NumberOfPtes,
//                        IN MMSYSTEM_PTE_POOL_TYPE PoolType)
// {
//     //
//     // Sanity checks
//     //
//     ASSERT(NumberOfPtes >= 1);
//
//     //
//     // Set the starting and ending PTE addresses for this space
//     //
//     MmSystemPteBase = MI_SYSTEM_PTE_BASE as PMMPTE;
//     MmSystemPtesStart[PoolType] = StartingPte;
//     MmSystemPtesEnd[PoolType] = StartingPte + NumberOfPtes - 1;
//     DPRINT("System PTE space for %d starting at: %p and ending at: %p\n",
//            PoolType, MmSystemPtesStart[PoolType], MmSystemPtesEnd[PoolType]);
//
//     //
//     // Clear all the PTEs to start with
//     //
//     RtlZeroMemory(StartingPte, NumberOfPtes * sizeof(MMPTE));
//
//     //
//     // Make the first entry free and link it
//     //
//     StartingPte->u.List.NextEntry = MM_EMPTY_PTE_LIST;
//     MmFirstFreeSystemPte[PoolType].Long = 0;
//     MmFirstFreeSystemPte[PoolType].set_list_next_entry(StartingPte -
//                                                       MmSystemPteBase as ULONG_PTR);
//
//     //
//     // The second entry stores the size of this PTE space
//     //
//     StartingPte++;
//     StartingPte->u.Long = 0;
//     StartingPte->u.List.NextEntry = NumberOfPtes;
//
//     //
//     // We also keep a global for it
//     //
//     MmTotalFreeSystemPtes[PoolType] = NumberOfPtes;
//
//     //
//     // Check if this is the system PTE space
//     //
//     if (PoolType == SystemPteSpace)
//     {
//         //
//         // Remember how many PTEs we have
//         //
//         MmTotalSystemPtes = NumberOfPtes;
//     }
// }
//
// /* EOF */
//
// ============================================================================
// END ORIGINAL C SOURCE
// ============================================================================
