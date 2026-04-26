use ntkrnl_sys::base::*;
use ntkrnl_sys::ex::*;
use ntkrnl_sys::interlocked::*;
use ntkrnl_sys::mm::*;
use ntkrnl_sys::rtl::*;

use ntkrnl_sys::ddk::{
    ExAcquireFastReference, ExAcquirePushLockExclusive, ExAcquirePushLockShared,
    ExFreePool, ExGetCountFastReference, ExGetObjectFastReference, ExInitializePushLock,
    ExInsertFastReference, ExReleasePushLock, ExReleasePushLockExclusive, KeEnterCriticalRegion,
    KeLeaveCriticalRegion, OB_SD_CACHE_LIST, ObpAcquireObjectLockShared, ObpReleaseObjectLock,
    PEX_FAST_REF, POBJECT_HEADER, POB_SD_CACHE_LIST, PSECURITY_DESCRIPTOR,
    PSECURITY_DESCRIPTOR_HEADER, QUAD, RtlLengthSecurityDescriptor, SECURITY_DESCRIPTOR_HEADER,
};
use ntkrnl_sys::{opaque_ptr, opaque_ptr_const};

const SD_CACHE_ENTRIES: usize = 0x100;
static mut ObsSecurityDescriptorCache: [OB_SD_CACHE_LIST; SD_CACHE_ENTRIES] =
    unsafe { core::mem::zeroed() };

const TAG_OB_SD_CACHE: ULONG = 0x6353624F;
// ReactOS uses 7 for ALL architectures (3 low bits), matching the C #define.
pub const MAX_FAST_REFS: ULONG = 7;

unsafe fn ObpSdAcquireLock(CacheEntry: POB_SD_CACHE_LIST) {
    KeEnterCriticalRegion();
    ExAcquirePushLockExclusive(core::ptr::addr_of_mut!((*CacheEntry).PushLock));
}

unsafe fn ObpSdReleaseLock(CacheEntry: POB_SD_CACHE_LIST) {
    ExReleasePushLockExclusive(core::ptr::addr_of_mut!((*CacheEntry).PushLock));
    KeLeaveCriticalRegion();
}

unsafe fn ObpSdAcquireLockShared(CacheEntry: POB_SD_CACHE_LIST) {
    KeEnterCriticalRegion();
    ExAcquirePushLockShared(core::ptr::addr_of_mut!((*CacheEntry).PushLock));
}

unsafe fn ObpSdReleaseLockShared(CacheEntry: POB_SD_CACHE_LIST) {
    ExReleasePushLock(core::ptr::addr_of_mut!((*CacheEntry).PushLock));
    KeLeaveCriticalRegion();
}

// CODE_SEG("INIT")
#[no_mangle]
pub unsafe extern "system" fn ObpInitSdCache() -> NTSTATUS {
    for i in 0..SD_CACHE_ENTRIES {
        InitializeListHead(core::ptr::addr_of_mut!(
            ObsSecurityDescriptorCache[i].Head
        ) as PLIST_ENTRY);
        ExInitializePushLock(core::ptr::addr_of_mut!(
            ObsSecurityDescriptorCache[i].PushLock
        ) as *mut _);
    }

    STATUS_SUCCESS
}

unsafe fn ObpHash(Buffer: PVOID, Length: ULONG) -> ULONG {
    let mut p = Buffer as *mut ULONG;
    let pb = Buffer as *mut UCHAR;
    let ppb = (Buffer as usize + Length as usize) as *mut UCHAR;
    let pp = ((pb as usize + Length as usize) & !(core::mem::size_of::<ULONG>() - 1)) as *mut ULONG;
    let mut Hash: ULONG = 0;

    while (p as usize) < (pp as usize) {
        Hash ^= *p;
        Hash = Hash.rotate_left(3);
        p = p.add(1);
    }

    let mut pb_iter = p as *mut UCHAR;
    while (pb_iter as usize) < (ppb as usize) {
        Hash ^= *pb_iter as ULONG;
        Hash = Hash.rotate_left(3);
        pb_iter = pb_iter.add(1);
    }

    Hash
}

unsafe fn ObpHashSecurityDescriptor(
    SecurityDescriptor: PSECURITY_DESCRIPTOR,
    Length: ULONG,
) -> ULONG {
    ObpHash(SecurityDescriptor as PVOID, Length)
}

unsafe fn ObpCreateCacheEntry(
    SecurityDescriptor: PSECURITY_DESCRIPTOR,
    Length: ULONG,
    FullHash: ULONG,
    RefCount: ULONG,
) -> PSECURITY_DESCRIPTOR_HEADER {
    let CacheSize = Length + (core::mem::size_of::<SECURITY_DESCRIPTOR_HEADER>() as ULONG
        - core::mem::size_of::<QUAD>() as ULONG);
    let SdHeader = ExAllocatePoolWithTag(PagedPool, CacheSize as usize, TAG_OB_SD_CACHE)
        as PSECURITY_DESCRIPTOR_HEADER;
    if SdHeader.is_null() {
        return core::ptr::null_mut();
    }

    (*SdHeader).RefCount = RefCount;
    (*SdHeader).FullHash = FullHash;

    RtlCopyMemory(
        core::ptr::addr_of_mut!((*SdHeader).SecurityDescriptor) as PVOID,
        SecurityDescriptor as PVOID,
        Length as usize,
    );

    SdHeader
}

unsafe fn ObpCompareSecurityDescriptors(
    Sd1: PSECURITY_DESCRIPTOR,
    Length1: ULONG,
    Sd2: PSECURITY_DESCRIPTOR,
) -> BOOLEAN {
    let Length2 = RtlLengthSecurityDescriptor(Sd2);

    if Length1 != Length2 {
        return 0u8;
    }

    let p1 = Sd1 as *const u8;
    let p2 = Sd2 as *const u8;
    let len = Length1 as usize;
    let mut i = 0;
    while i < len {
        if *p1.add(i) != *p2.add(i) {
            return 0u8;
        }
        i += 1;
    }
    1u8
}

unsafe fn ObpDestroySecurityDescriptorHeader(
    SdHeader: PSECURITY_DESCRIPTOR_HEADER,
) -> PVOID {
    RemoveEntryList(core::ptr::addr_of_mut!((*SdHeader).Link) as PLIST_ENTRY);
    SdHeader as PVOID
}

#[inline(always)]
unsafe fn ObpGetHeaderForSd(
    SecurityDescriptor: PSECURITY_DESCRIPTOR,
) -> PSECURITY_DESCRIPTOR_HEADER {
    ((SecurityDescriptor as usize)
        - core::mem::offset_of!(SECURITY_DESCRIPTOR_HEADER, SecurityDescriptor))
        as PSECURITY_DESCRIPTOR_HEADER
}

#[inline(always)]
unsafe fn ObpGetHeaderForEntry(Entry: PLIST_ENTRY) -> PSECURITY_DESCRIPTOR_HEADER {
    ((Entry as usize) - core::mem::offset_of!(SECURITY_DESCRIPTOR_HEADER, Link))
        as PSECURITY_DESCRIPTOR_HEADER
}

#[no_mangle]
pub unsafe extern "system" fn ObpReferenceSecurityDescriptor(
    ObjectHeader: POBJECT_HEADER,
) -> PSECURITY_DESCRIPTOR {
    let ObjectHeader = opaque_ptr(ObjectHeader);
    let FastRef = core::ptr::addr_of_mut!((*ObjectHeader).SecurityDescriptor) as PEX_FAST_REF;
    let OldValue = ExAcquireFastReference(FastRef);

    let SecurityDescriptor = ExGetObjectFastReference(OldValue) as PSECURITY_DESCRIPTOR;
    let Count = ExGetCountFastReference(OldValue);

    if (Count >= 1) || SecurityDescriptor.is_null() {
        if Count == 1 {
            let SdHeader = ObpGetHeaderForSd(SecurityDescriptor);
            InterlockedExchangeAdd(
                core::ptr::addr_of_mut!((*SdHeader).RefCount) as *mut LONG,
                MAX_FAST_REFS as LONG,
            );

            if ExInsertFastReference(FastRef, SecurityDescriptor as PVOID) == 0 {
                InterlockedExchangeAdd(
                    core::ptr::addr_of_mut!((*SdHeader).RefCount) as *mut LONG,
                    -(MAX_FAST_REFS as LONG),
                );
            }
        }

        return SecurityDescriptor;
    }

    ObpAcquireObjectLockShared(ObjectHeader);

    let SecurityDescriptor = ExGetObjectFastReference(*FastRef) as PSECURITY_DESCRIPTOR;
    let SdHeader = ObpGetHeaderForSd(SecurityDescriptor);

    InterlockedIncrement(core::ptr::addr_of_mut!((*SdHeader).RefCount) as *mut LONG);

    ObpReleaseObjectLock(ObjectHeader);
    SecurityDescriptor
}

#[no_mangle]
pub unsafe extern "system" fn ObReferenceSecurityDescriptor(
    SecurityDescriptor: PSECURITY_DESCRIPTOR,
    Count: LONG,
) {
    let SecurityDescriptor = opaque_ptr(SecurityDescriptor);
    let SdHeader = ObpGetHeaderForSd(SecurityDescriptor);
    InterlockedExchangeAdd(
        core::ptr::addr_of_mut!((*SdHeader).RefCount) as *mut LONG,
        Count,
    );
}

#[no_mangle]
pub unsafe extern "system" fn ObDereferenceSecurityDescriptor(
    SecurityDescriptor: PSECURITY_DESCRIPTOR,
    Count: LONG,
) {
    let SecurityDescriptor = opaque_ptr(SecurityDescriptor);
    let SdHeader = ObpGetHeaderForSd(SecurityDescriptor);

    let mut OldValue = (*SdHeader).RefCount as LONG;

    while OldValue != Count {
        let atomic = &*(core::ptr::addr_of_mut!((*SdHeader).RefCount)
            as *const core::sync::atomic::AtomicU32);
        let result = atomic.compare_exchange(
            OldValue as u32,
            (OldValue - Count) as u32,
            core::sync::atomic::Ordering::SeqCst,
            core::sync::atomic::Ordering::SeqCst,
        );
        match result {
            Ok(_) => return,
            Err(val) => OldValue = val as LONG,
        }
    }

    let Index = ((*SdHeader).FullHash as usize) % SD_CACHE_ENTRIES;
    let CacheEntry = &mut ObsSecurityDescriptorCache[Index] as POB_SD_CACHE_LIST;

    ObpSdAcquireLock(CacheEntry);

    if InterlockedExchangeAdd(
        core::ptr::addr_of_mut!((*SdHeader).RefCount) as *mut LONG,
        -Count,
    ) == Count
    {
        let SdHeader = ObpDestroySecurityDescriptorHeader(SdHeader);
        ObpSdReleaseLock(CacheEntry);
        ExFreePool(SdHeader);
    } else {
        ObpSdReleaseLock(CacheEntry);
    }
}

#[no_mangle]
pub unsafe extern "system" fn ObLogSecurityDescriptor(
    InputSecurityDescriptor: PSECURITY_DESCRIPTOR,
    OutputSecurityDescriptor: *mut PSECURITY_DESCRIPTOR,
    RefBias: LONG,
) -> NTSTATUS {
    let InputSecurityDescriptor = opaque_ptr(InputSecurityDescriptor);
    let OutputSecurityDescriptor = opaque_ptr(OutputSecurityDescriptor);
    let mut SdHeader: PSECURITY_DESCRIPTOR_HEADER = core::ptr::null_mut();
    let mut NewHeader: PSECURITY_DESCRIPTOR_HEADER = core::ptr::null_mut();

    let Length = RtlLengthSecurityDescriptor(InputSecurityDescriptor);
    let Hash = ObpHashSecurityDescriptor(InputSecurityDescriptor, Length);

    let Index = (Hash as usize) % SD_CACHE_ENTRIES;
    let CacheEntry = &mut ObsSecurityDescriptorCache[Index] as POB_SD_CACHE_LIST;

    ObpSdAcquireLockShared(CacheEntry);

    let mut NextEntry: PLIST_ENTRY = core::ptr::null_mut();

    loop {
        let mut Result: BOOLEAN = 0u8;
        let HeadPtr = core::ptr::addr_of_mut!((*CacheEntry).Head) as PLIST_ENTRY;

        NextEntry = (*HeadPtr).Flink;
        while NextEntry != HeadPtr {
            SdHeader = ObpGetHeaderForEntry(NextEntry);

            if (*SdHeader).FullHash > Hash {
                break;
            }

            if (*SdHeader).FullHash == Hash {
                Result = ObpCompareSecurityDescriptors(
                    InputSecurityDescriptor,
                    Length,
                    core::ptr::addr_of_mut!((*SdHeader).SecurityDescriptor) as PSECURITY_DESCRIPTOR,
                );
                if Result != 0 {
                    break;
                }
            }

            NextEntry = (*NextEntry).Flink;
        }

        if Result != 0 {
            InterlockedExchangeAdd(
                core::ptr::addr_of_mut!((*SdHeader).RefCount) as *mut LONG,
                RefBias,
            );

            ObpSdReleaseLockShared(CacheEntry);

            *OutputSecurityDescriptor =
                core::ptr::addr_of_mut!((*SdHeader).SecurityDescriptor) as PSECURITY_DESCRIPTOR;

            if !NewHeader.is_null() {
                ExFreePoolWithTag(NewHeader as PVOID, TAG_OB_SD_CACHE);
            }
            return STATUS_SUCCESS;
        }

        if NewHeader.is_null() {
            ObpSdReleaseLockShared(CacheEntry);

            NewHeader = ObpCreateCacheEntry(InputSecurityDescriptor, Length, Hash, RefBias as ULONG);
            if NewHeader.is_null() {
                return STATUS_INSUFFICIENT_RESOURCES;
            }

            ObpSdAcquireLock(CacheEntry);
        } else {
            break;
        }
    }

    InsertTailList(
        NextEntry,
        core::ptr::addr_of_mut!((*NewHeader).Link) as PLIST_ENTRY,
    );

    ObpSdReleaseLock(CacheEntry);

    *OutputSecurityDescriptor =
        core::ptr::addr_of_mut!((*NewHeader).SecurityDescriptor) as PSECURITY_DESCRIPTOR;
    STATUS_SUCCESS
}


// ============================================================================
// ORIGINAL C SOURCE
// ============================================================================
//
// /*
//  * PROJECT:         ReactOS Kernel
//  * LICENSE:         GPL - See COPYING in the top level directory
//  * FILE:            ntoskrnl/ob/obsdcach.c
//  * PURPOSE:         Security Descriptor Caching
//  * PROGRAMMERS:     Aleksey Bragin (aleksey@reactos.org)
//  */
//
// /* INCLUDES *******************************************************************/
//
// #include <ntoskrnl.h>
// #define NDEBUG
// #include <debug.h>
//
// /* GLOBALS ********************************************************************/
//
// #define SD_CACHE_ENTRIES 0x100
// OB_SD_CACHE_LIST ObsSecurityDescriptorCache[SD_CACHE_ENTRIES];
//
// /* PRIVATE FUNCTIONS **********************************************************/
//
// FORCEINLINE
// VOID
// ObpSdAcquireLock(IN POB_SD_CACHE_LIST CacheEntry)
// {
//     /* Acquire the lock */
//     KeEnterCriticalRegion();
//     ExAcquirePushLockExclusive(&CacheEntry->PushLock);
// }
//
// FORCEINLINE
// VOID
// ObpSdReleaseLock(IN POB_SD_CACHE_LIST CacheEntry)
// {
//     /* Release the lock */
//     ExReleasePushLockExclusive(&CacheEntry->PushLock);
//     KeLeaveCriticalRegion();
// }
//
// FORCEINLINE
// VOID
// ObpSdAcquireLockShared(IN POB_SD_CACHE_LIST CacheEntry)
// {
//     /* Acquire the lock */
//     KeEnterCriticalRegion();
//     ExAcquirePushLockShared(&CacheEntry->PushLock);
// }
//
// FORCEINLINE
// VOID
// ObpSdReleaseLockShared(IN POB_SD_CACHE_LIST CacheEntry)
// {
//     /* Release the lock */
//     ExReleasePushLock(&CacheEntry->PushLock);
//     KeLeaveCriticalRegion();
// }
//
// CODE_SEG("INIT")
// NTSTATUS
// NTAPI
// ObpInitSdCache(VOID)
// {
//     ULONG i;
//
//     /* Loop each cache entry */
//     for (i = 0; i < SD_CACHE_ENTRIES; i++)
//     {
//         /* Initialize the lock and the list */
//         InitializeListHead(&ObsSecurityDescriptorCache[i].Head);
//         ExInitializePushLock(&ObsSecurityDescriptorCache[i].PushLock);
//     }
//
//     /* Return success */
//     return STATUS_SUCCESS;
// }
//
// ULONG
// NTAPI
// ObpHash(IN PVOID Buffer,
//         IN ULONG Length)
// {
//     PULONG p, pp;
//     PUCHAR pb, ppb;
//     ULONG Hash = 0;
//
//     /* Setup aligned and byte buffers */
//     p = Buffer;
//     pb = (PUCHAR)p;
//     ppb = (PUCHAR)((ULONG_PTR)Buffer + Length);
//     pp = (PULONG)ALIGN_DOWN(pb + Length, ULONG);
//
//     /* Loop aligned data */
//     while (p < pp)
//     {
//         /* XOR-rotate */
//         Hash ^= *p++;
//         Hash = _rotl(Hash, 3);
//     }
//
//     /* Loop non-aligned data */
//     pb = (PUCHAR)p;
//     while (pb < ppb)
//     {
//         /* XOR-rotate */
//         Hash ^= *pb++;
//         Hash = _rotl(Hash, 3);
//     }
//
//     /* Return the hash */
//     return Hash;
// }
//
// ULONG
// NTAPI
// ObpHashSecurityDescriptor(IN PSECURITY_DESCRIPTOR SecurityDescriptor,
//                           IN ULONG Length)
// {
//     /* Just hash the entire SD */
//     return ObpHash(SecurityDescriptor, Length);
// }
//
// PSECURITY_DESCRIPTOR_HEADER
// NTAPI
// ObpCreateCacheEntry(IN PSECURITY_DESCRIPTOR SecurityDescriptor,
//                     IN ULONG Length,
//                     IN ULONG FullHash,
//                     IN ULONG RefCount)
// {
//     ULONG CacheSize;
//     PSECURITY_DESCRIPTOR_HEADER SdHeader;
//     ASSERT(Length == RtlLengthSecurityDescriptor(SecurityDescriptor));
//
//     /* Calculate the memory we'll need to allocate and allocate it */
//     CacheSize = Length + (sizeof(SECURITY_DESCRIPTOR_HEADER) - sizeof(QUAD));
//     SdHeader = ExAllocatePoolWithTag(PagedPool, CacheSize, TAG_OB_SD_CACHE);
//     if (!SdHeader) return NULL;
//
//     /* Setup the header */
//     SdHeader->RefCount = RefCount;
//     SdHeader->FullHash = FullHash;
//
//     /* Copy the descriptor */
//     RtlCopyMemory(&SdHeader->SecurityDescriptor, SecurityDescriptor, Length);
//
//     /* Return it */
//     return SdHeader;
// }
//
// BOOLEAN
// NTAPI
// ObpCompareSecurityDescriptors(IN PSECURITY_DESCRIPTOR Sd1,
//                               IN ULONG Length1,
//                               IN PSECURITY_DESCRIPTOR Sd2)
// {
//     ULONG Length2;
//     ASSERT(Length1 == RtlLengthSecurityDescriptor(Sd1));
//
//     /* Get the length of the second SD */
//     Length2 = RtlLengthSecurityDescriptor(Sd2);
//
//     /* Compare lengths */
//     if (Length1 != Length2) return FALSE;
//
//     /* Compare contents */
//     return RtlEqualMemory(Sd1, Sd2, Length1);
// }
//
// PVOID
// NTAPI
// ObpDestroySecurityDescriptorHeader(IN PSECURITY_DESCRIPTOR_HEADER SdHeader)
// {
//     ASSERT(SdHeader->RefCount == 0);
//
//     /* Just unlink the SD and return it back to the caller */
//     RemoveEntryList(&SdHeader->Link);
//     return SdHeader;
// }
//
// PSECURITY_DESCRIPTOR
// NTAPI
// ObpReferenceSecurityDescriptor(IN POBJECT_HEADER ObjectHeader)
// {
//     PSECURITY_DESCRIPTOR SecurityDescriptor;
//     PSECURITY_DESCRIPTOR_HEADER SdHeader;
//     PEX_FAST_REF FastRef;
//     EX_FAST_REF OldValue;
//     ULONG_PTR Count;
//
//     /* Acquire a reference to the security descriptor */
//     FastRef = (PEX_FAST_REF)&ObjectHeader->SecurityDescriptor;
//     OldValue = ExAcquireFastReference(FastRef);
//
//     /* Get the descriptor and reference count */
//     SecurityDescriptor = ExGetObjectFastReference(OldValue);
//     Count = ExGetCountFastReference(OldValue);
//
//     /* Check if there's no descriptor or if there's still cached references */
//     if ((Count >= 1) || !(SecurityDescriptor))
//     {
//         /* Check if this is the last reference */
//         if (Count == 1)
//         {
//             /* Add the extra references that we'll take */
//             SdHeader = ObpGetHeaderForSd(SecurityDescriptor);
//             InterlockedExchangeAdd((PLONG)&SdHeader->RefCount, MAX_FAST_REFS);
//
//             /* Now insert them */
//             if (!ExInsertFastReference(FastRef, SecurityDescriptor))
//             {
//                 /* Undo the references since we failed */
//                 InterlockedExchangeAdd((PLONG)&SdHeader->RefCount,
//                                        -MAX_FAST_REFS);
//             }
//         }
//
//         /* Return the SD */
//         return SecurityDescriptor;
//     }
//
//     /* Lock the object */
//     ObpAcquireObjectLockShared(ObjectHeader);
//
//     /* Get the object header */
//     SecurityDescriptor = ExGetObjectFastReference(*FastRef);
//     SdHeader = ObpGetHeaderForSd(SecurityDescriptor);
//
//     /* Do the reference */
//     InterlockedIncrement((PLONG)&SdHeader->RefCount);
//
//     /* Release the lock and return */
//     ObpReleaseObjectLock(ObjectHeader);
//     return SecurityDescriptor;
// }
//
// /* PUBLIC FUNCTIONS ***********************************************************/
//
// /*++
//  * @name ObReferenceSecurityDescriptor
//  * @implemented NT5.2
//  *
//  *     The ObReferenceSecurityDescriptor routine <FILLMEIN>
//  *
//  * @param SecurityDescriptor
//  *        <FILLMEIN>
//  *
//  * @param Count
//  *        <FILLMEIN>
//  *
//  * @return STATUS_SUCCESS or appropriate error value.
//  *
//  * @remarks None.
//  *
//  *--*/
// VOID
// NTAPI
// ObReferenceSecurityDescriptor(IN PSECURITY_DESCRIPTOR SecurityDescriptor,
//                               IN ULONG Count)
// {
//     PSECURITY_DESCRIPTOR_HEADER SdHeader;
//
//     /* Get the header */
//     SdHeader = ObpGetHeaderForSd(SecurityDescriptor);
//
//     /* Do the references */
//     InterlockedExchangeAdd((PLONG)&SdHeader->RefCount, Count);
// }
//
// /*++
//  * @name ObDereferenceSecurityDescriptor
//  * @implemented NT5.2
//  *
//  *     The ObDereferenceSecurityDescriptor routine <FILLMEIN>
//  *
//  * @param SecurityDescriptor
//  *        <FILLMEIN>
//  *
//  * @param Count
//  *        <FILLMEIN>
//  *
//  * @return STATUS_SUCCESS or appropriate error value.
//  *
//  * @remarks None.
//  *
//  *--*/
// VOID
// NTAPI
// ObDereferenceSecurityDescriptor(IN PSECURITY_DESCRIPTOR SecurityDescriptor,
//                                 IN ULONG Count)
// {
//     PSECURITY_DESCRIPTOR_HEADER SdHeader;
//     LONG OldValue, NewValue;
//     ULONG Index;
//     POB_SD_CACHE_LIST CacheEntry;
//
//     /* Get the header */
//     SdHeader = ObpGetHeaderForSd(SecurityDescriptor);
//
//     /* Get the current reference count */
//     OldValue = SdHeader->RefCount;
//
//     /* Check if the caller is destroying this SD -- we need the lock for that */
//     while (OldValue != Count)
//     {
//         /* He isn't, we can just try to derefeference atomically */
//         NewValue = InterlockedCompareExchange((PLONG)&SdHeader->RefCount,
//                                               OldValue - Count,
//                                               OldValue);
//         if (NewValue == OldValue) return;
//
//         /* Try again */
//         OldValue = NewValue;
//     }
//
//     /* At this point, we need the lock, so choose an entry */
//     Index = SdHeader->FullHash % SD_CACHE_ENTRIES;
//     CacheEntry = &ObsSecurityDescriptorCache[Index];
//
//     /* Acquire the lock for it */
//     ObpSdAcquireLock(CacheEntry);
//     ASSERT(SdHeader->RefCount != 0);
//
//     /* Now do the dereference */
//     if (InterlockedExchangeAdd((PLONG)&SdHeader->RefCount, -(LONG)Count) == Count)
//     {
//         /* We're down to zero -- destroy the header */
//         SdHeader = ObpDestroySecurityDescriptorHeader(SdHeader);
//
//         /* Release the lock */
//         ObpSdReleaseLock(CacheEntry);
//
//         /* Free the header */
//         ExFreePool(SdHeader);
//     }
//     else
//     {
//         /* Just release the lock */
//         ObpSdReleaseLock(CacheEntry);
//     }
//
// }
//
// /*++
// * @name ObLogSecurityDescriptor
// * @implemented NT5.2
// *
// *     The ObLogSecurityDescriptor routine <FILLMEIN>
// *
// * @param InputSecurityDescriptor
// *        <FILLMEIN>
// *
// * @param OutputSecurityDescriptor
// *        <FILLMEIN>
// *
// * @param RefBias
// *        <FILLMEIN>
// *
// * @return STATUS_SUCCESS or appropriate error value.
// *
// * @remarks None.
// *
// *--*/
// NTSTATUS
// NTAPI
// ObLogSecurityDescriptor(IN PSECURITY_DESCRIPTOR InputSecurityDescriptor,
//                         OUT PSECURITY_DESCRIPTOR *OutputSecurityDescriptor,
//                         IN ULONG RefBias)
// {
//     PSECURITY_DESCRIPTOR_HEADER SdHeader = NULL, NewHeader  = NULL;
//     ULONG Length, Hash, Index;
//     POB_SD_CACHE_LIST CacheEntry;
//     BOOLEAN Result;
//     PLIST_ENTRY NextEntry;
//
//     /* Get the length */
//     Length = RtlLengthSecurityDescriptor(InputSecurityDescriptor);
//
//     /* Get the hash */
//     Hash = ObpHashSecurityDescriptor(InputSecurityDescriptor, Length);
//
//     /* Now select the appropriate cache entry */
//     Index = Hash % SD_CACHE_ENTRIES;
//     CacheEntry = &ObsSecurityDescriptorCache[Index];
//
//     /* Lock it shared */
//     ObpSdAcquireLockShared(CacheEntry);
//
//     /* Start our search */
//     while (TRUE)
//     {
//         /* Reset result found */
//         Result = FALSE;
//
//         /* Loop the hash list */
//         NextEntry = CacheEntry->Head.Flink;
//         while (NextEntry != &CacheEntry->Head)
//         {
//             /* Get the header */
//             SdHeader = ObpGetHeaderForEntry(NextEntry);
//
//             /* Our hashes are ordered, so quickly check if we should stop now */
//             if (SdHeader->FullHash > Hash) break;
//
//             /* We survived the quick hash check, now check for equalness */
//             if (SdHeader->FullHash == Hash)
//             {
//                 /* Hashes match, now compare descriptors */
//                 Result = ObpCompareSecurityDescriptors(InputSecurityDescriptor,
//                                                        Length,
//                                                        &SdHeader->SecurityDescriptor);
//                 if (Result) break;
//             }
//
//             /* Go to the next entry */
//             NextEntry = NextEntry->Flink;
//         }
//
//         /* Check if we found anything */
//         if (Result)
//         {
//             /* Increment its reference count */
//             InterlockedExchangeAdd((PLONG)&SdHeader->RefCount, RefBias);
//
//             /* Release the lock */
//             ObpSdReleaseLockShared(CacheEntry);
//
//             /* Return the descriptor */
//             *OutputSecurityDescriptor = &SdHeader->SecurityDescriptor;
//
//             /* Free anything that we may have had to create */
//             if (NewHeader) ExFreePoolWithTag(NewHeader, TAG_OB_SD_CACHE);
//             return STATUS_SUCCESS;
//         }
//
//         /* Check if we got here, and didn't create a descriptor yet */
//         if (!NewHeader)
//         {
//             /* Release the lock */
//             ObpSdReleaseLockShared(CacheEntry);
//
//             /* This should be our first time in the loop, create it */
//             NewHeader = ObpCreateCacheEntry(InputSecurityDescriptor,
//                                             Length,
//                                             Hash,
//                                             RefBias);
//             if (!NewHeader) return STATUS_INSUFFICIENT_RESOURCES;
//
//             /* Now acquire the exclusive lock and we should hit the right path */
//             ObpSdAcquireLock(CacheEntry);
//         }
//         else
//         {
//             /* We have inserted the SD, we're fine now */
//             break;
//         }
//     }
//
//     /* Okay, now let's do the insert, we should have the exclusive lock */
//     InsertTailList(NextEntry, &NewHeader->Link);
//
//     /* Release the lock */
//     ObpSdReleaseLock(CacheEntry);
//
//     /* Return the SD*/
//     *OutputSecurityDescriptor = &NewHeader->SecurityDescriptor;
//     return STATUS_SUCCESS;
// }
//
// /* EOF */
//
// ============================================================================
// END ORIGINAL C SOURCE
// ============================================================================
