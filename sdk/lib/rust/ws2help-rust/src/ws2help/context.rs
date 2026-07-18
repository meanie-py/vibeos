use crate::prelude::*;
use core::ffi::c_void;

extern "system" {
    fn PulseEvent(hEvent: HANDLE) -> i32;
    fn SwitchToThread() -> i32;
    fn CreateEventA( lpEventAttributes: *mut core::ffi::c_void, bManualReset: i32, bInitialState: i32, lpName: *const i8, ) -> HANDLE;
    fn CloseHandle(hObject: HANDLE) -> i32;
    fn Sleep(dwMilliseconds: u32);
    fn WaitForSingleObject(hHandle: HANDLE, dwMilliseconds: u32) -> u32;
    fn HeapAlloc(hHeap: HANDLE, dwFlags: DWORD, dwBytes: SIZE_T) -> LPVOID;
    fn InitializeCriticalSectionAndSpinCount( lpCriticalSection: *mut CRITICAL_SECTION, dwSpinCount: DWORD, ) -> BOOL;
    fn EnterCriticalSection(lpCriticalSection: *mut CRITICAL_SECTION);
    fn LeaveCriticalSection(lpCriticalSection: *mut CRITICAL_SECTION);
    fn DeleteCriticalSection(lpCriticalSection: *mut CRITICAL_SECTION);
    fn HeapFree(hHeap: HANDLE, dwFlags: DWORD, lpMem: PVOID) -> BOOL;
}

// GlobalHeap is defined in dllmain (the module that owns the process-heap handle).
use crate::ws2help::dllmain::GlobalHeap;

// ---- file-scope preamble (translated) ----
// @c2rust file-scope data (compiler-exact tables/statics)
pub static mut WshHandleTableLock: CRITICAL_SECTION = CRITICAL_SECTION {
    DebugInfo: ::core::ptr::null::<_RTL_CRITICAL_SECTION_DEBUG>()
        as *mut _RTL_CRITICAL_SECTION_DEBUG,
    LockCount: 0,
    RecursionCount: 0,
    OwningThread: ::core::ptr::null::<::core::ffi::c_void>() as *mut ::core::ffi::c_void,
    LockSemaphore: ::core::ptr::null::<::core::ffi::c_void>() as *mut ::core::ffi::c_void,
    SpinCount: 0,
};

pub static mut ghWriterEvent: HANDLE =
    ::core::ptr::null::<::core::ffi::c_void>() as *mut ::core::ffi::c_void;

pub static mut gdwSpinCount: DWORD = 0 as DWORD;

pub static mut gHandleToIndexMask: DWORD = 0;

pub static mut SockPrimes: [DWORD; 15] = [
    31 as ::core::ffi::c_int as DWORD,
    61 as ::core::ffi::c_int as DWORD,
    127 as ::core::ffi::c_int as DWORD,
    257 as ::core::ffi::c_int as DWORD,
    521 as ::core::ffi::c_int as DWORD,
    1031 as ::core::ffi::c_int as DWORD,
    2053 as ::core::ffi::c_int as DWORD,
    4099 as ::core::ffi::c_int as DWORD,
    8191 as ::core::ffi::c_int as DWORD,
    16381 as ::core::ffi::c_int as DWORD,
    32749 as ::core::ffi::c_int as DWORD,
    65537 as ::core::ffi::c_int as DWORD,
    131071 as ::core::ffi::c_int as DWORD,
    261983 as ::core::ffi::c_int as DWORD,
    -(1 as ::core::ffi::c_int) as DWORD,
];

pub type VLONG = LONG;
pub type PVLONG = *mut VLONG;
unsafe extern "C" fn AcquireReadLock(Table: PWAH_SEARCH_TABLE, Count: *mut PVLONG) {
    let mut OldCount: LONG;

    loop {
        *Count = (*Table).CurrentCount as PVLONG;
        OldCount = **Count;

        if (OldCount > 0) && (InterlockedCompareExchange(*Count, OldCount + 1, OldCount) == OldCount) {
            break;
        }
    }
}
unsafe extern "C" fn ReleaseReadLock(Table: PWAH_SEARCH_TABLE, Count: PVLONG) {
    if InterlockedDecrement(Count) < 0 {
        PulseEvent(ghWriterEvent);
    }
}
#[no_mangle]
pub unsafe extern "system" fn DoWaitForReaders(Table: PWAH_SEARCH_TABLE, Counter: PVLONG) {

    let mut EventHandle: HANDLE;

    SwitchToThread();

    if *Counter > 0 {
        if ghWriterEvent.is_null() {
            EventHandle = CreateEventA(
                core::ptr::null_mut(),
                1,
                0,
                core::ptr::null(),
            );
            if !EventHandle.is_null() {
                if !InterlockedCompareExchangePointer(
                    core::ptr::addr_of_mut!(ghWriterEvent) as *mut *mut core::ffi::c_void,
                    EventHandle as *mut core::ffi::c_void,
                    core::ptr::null_mut(),
                )
                .is_null()
                {
                    CloseHandle(EventHandle);
                }
            } else {
                while *Counter > 0 {
                    Sleep(10);
                }
            }
        }

        if InterlockedDecrement(Counter) >= 0 {
            loop {
                WaitForSingleObject(ghWriterEvent, 10);
                if !(*Counter >= 0) {
                    break;
                }
            }
        }
    }
}
unsafe extern "C" fn TryWaitForReaders(Table: PWAH_SEARCH_TABLE) {
    let OldCount: PVLONG = (*Table).CurrentCount;
    let mut SpinCount: LONG;

    if OldCount == core::ptr::addr_of_mut!((*Table).Count1) as PVLONG {
        (*Table).Count2 = 1;
        (*Table).CurrentCount = core::ptr::addr_of_mut!((*Table).Count2);
    } else {
        (*Table).Count1 = 1;
        (*Table).CurrentCount = core::ptr::addr_of_mut!((*Table).Count1);
    }

    if InterlockedDecrement(OldCount) > 0 {
        if (*Table).SpinCount != 0 {
            SpinCount = (*Table).SpinCount;
            while *OldCount > 0 {
                SpinCount -= 1;
                if SpinCount <= 0 {
                    break;
                }
            }
        }

        if *OldCount > 0 {
            DoWaitForReaders(Table, OldCount);
        }
    }
}

#[no_mangle]
pub unsafe extern "system" fn WahCreateHandleContextTable(
    Table: *mut PWAH_HANDLE_TABLE,
) -> DWORD {
    let mut ErrorCode: DWORD;
    let mut LocalTable: PWAH_HANDLE_TABLE;
    let mut i: DWORD;

    ErrorCode = WS2HELP_PROLOG();
    if ErrorCode != ERROR_SUCCESS {
        return ErrorCode;
    }

    *Table = core::ptr::null_mut();

    // C: FIELD_OFFSET(WSH_HANDLE_TABLE, SearchTables[gHandleToIndexMask + 1]) — the byte offset
    // of the (mask+1)-th SearchTables element, i.e. the size of the variable-length table.
    LocalTable = HeapAlloc(
        GlobalHeap,
        0,
        (core::mem::offset_of!(WSH_HANDLE_TABLE, SearchTables)
            + (gHandleToIndexMask.wrapping_add(1) as usize)
                * core::mem::size_of::<WSH_SEARCH_TABLE>()) as SIZE_T,
    ) as PWAH_HANDLE_TABLE;

    if LocalTable.is_null() {
        return WSA_NOT_ENOUGH_MEMORY;
    }

    (*LocalTable).Mask = gHandleToIndexMask;

    i = 0;
    while i <= gHandleToIndexMask {
        // SearchTables is a C flexible-array member (declared `[_; 1]` stub, allocated
        // for gHandleToIndexMask+1 elements above). Index by pointer arithmetic to match
        // the C `LocalTable->SearchTables[i]` — the `[i]` operator would bounds-check
        // against the length-1 stub and panic for i>=1 (mask is >=7, see dllmain).
        let search_table = &mut *(*LocalTable).SearchTables.as_mut_ptr().add(i as usize);
        
        search_table.HashTable = core::ptr::null_mut();
        search_table.CurrentCount = core::ptr::addr_of_mut!(search_table.Count1);
        search_table.Count1 = 1;
        search_table.Count2 = 0;
        search_table.Expanding = 0;
        search_table.SpinCount = gdwSpinCount as LONG;
        
        InitializeCriticalSectionAndSpinCount(
            core::ptr::addr_of_mut!(search_table.Lock),
            gdwSpinCount,
        );
        
        i = i.wrapping_add(1);
    }

    *Table = LocalTable;

    return ERROR_SUCCESS;
}

#[no_mangle]
pub unsafe extern "system" fn WahDestroyHandleContextTable(Table: PWAH_HANDLE_TABLE) -> DWORD {
    let mut i: DWORD;

    if Table.is_null() {
        return ERROR_INVALID_PARAMETER;
    }

    i = 0;
    while i <= (*Table).Mask {
        if !(*Table).SearchTables.as_mut_ptr().offset(i as isize).is_null() {
            let search_table = &mut *(*Table).SearchTables.as_mut_ptr().offset(i as isize);
            if !search_table.HashTable.is_null() {
                HeapFree(GlobalHeap, 0, search_table.HashTable as PVOID);
            }
            DeleteCriticalSection(&mut search_table.Lock);
        }
        i = i.wrapping_add(1);
    }

    HeapFree(GlobalHeap, 0, Table as PVOID);

    return ERROR_SUCCESS;
}

#[no_mangle]
pub unsafe extern "system" fn WahEnumerateHandleContexts(
    Table: PWAH_HANDLE_TABLE,
    Callback: PWAH_HANDLE_ENUMERATE_PROC,
    Context: PVOID,
) -> BOOL {
    let mut i: DWORD;
    let mut j: DWORD;
    let mut SearchTable: PWAH_SEARCH_TABLE;
    let mut HashTable: PWAH_HASH_TABLE;
    let mut Handle: PWAH_HANDLE;
    let mut GoOn: BOOL = 1;

    i = 0;
    while i <= (*Table).Mask {
        SearchTable = (core::ptr::addr_of_mut!((*Table).SearchTables) as *mut WSH_SEARCH_TABLE)
            .offset(i as isize);

        EnterCriticalSection(core::ptr::addr_of_mut!((*SearchTable).Lock));

        (*SearchTable).Expanding = 1;
        TryWaitForReaders(SearchTable);

        HashTable = (*SearchTable).HashTable;

        if !HashTable.is_null() {
            j = 0;
            while j < (*HashTable).Size {
                Handle = *(core::ptr::addr_of_mut!((*HashTable).Handles) as *mut PWAH_HANDLE)
                    .offset(j as isize);
                if !Handle.is_null() {
                    if let Some(cb) = Callback {
                        GoOn = cb(Context, Handle);
                    }
                    if GoOn == 0 {
                        break;
                    }
                }
                j = j.wrapping_add(1);
            }
        }

        (*SearchTable).Expanding = 0;
        LeaveCriticalSection(core::ptr::addr_of_mut!((*SearchTable).Lock));

        if GoOn == 0 {
            break;
        }
        i = i.wrapping_add(1);
    }

    GoOn
}

#[no_mangle]
pub unsafe extern "system" fn WahInsertHandleContext(
    Table: PWAH_HANDLE_TABLE,
    Handle: PWAH_HANDLE,
) -> PWAH_HANDLE {
    let mut HashHandle: *mut PWAH_HANDLE;
    let mut OldHandle: PWAH_HANDLE;
    let mut Count: PVLONG = core::ptr::null_mut();
    let mut HashTable: PWAH_HASH_TABLE;
    let mut NewHashTable: PWAH_HASH_TABLE;
    let mut HandleCount: DWORD;
    let mut i: DWORD;
    let SearchTable: PWAH_SEARCH_TABLE;

    // Flexible-array member: index by pointer arithmetic (the `[..]` operator would
    // bounds-check against the length-1 stub and panic — the hash index is `& Mask`,
    // range 0..=Mask where Mask is 7 or 31, see WahCreateHandleContextTable/dllmain).
    SearchTable = (*Table).SearchTables.as_mut_ptr().add(((((*Handle).Handle as usize) >> 2) & ((*Table).Mask as usize)));

    'outer: loop {
        AcquireReadLock(SearchTable, &mut Count);

        HashTable = (*SearchTable).HashTable;

        if (*SearchTable).Expanding == 0 && !HashTable.is_null() {
            HashHandle = &mut *(*HashTable).Handles.as_mut_ptr().add(((*Handle).Handle as usize) % ((*HashTable).Size as usize));

            if InterlockedCompareExchangePointer(
                HashHandle as *mut PVOID as *mut *mut core::ffi::c_void,
                Handle as *mut core::ffi::c_void,
                core::ptr::null_mut(),
            )
            .is_null()
            {
                ReleaseReadLock(SearchTable, Count);
                OldHandle = Handle;
                break 'outer;
            }
        }

        ReleaseReadLock(SearchTable, Count);

        EnterCriticalSection(&mut (*SearchTable).Lock);

        (*SearchTable).Expanding = 1;

        TryWaitForReaders(SearchTable);

        'inner: loop {
            HashTable = (*SearchTable).HashTable;

            if !HashTable.is_null() {
                HashHandle = &mut *(*HashTable).Handles.as_mut_ptr().add(((*Handle).Handle as usize) % ((*HashTable).Size as usize));

                if (*HashHandle).is_null() {
                    OldHandle = Handle;
                    *HashHandle = Handle;
                    break 'inner;
                } else if (**HashHandle).Handle == (*Handle).Handle {
                    OldHandle = *HashHandle;
                    *HashHandle = Handle;
                    break 'inner;
                }

                HandleCount = (*HashTable).Size;
            } else {
                HandleCount = 0;
            }

            'expand: loop {
                i = 0;
                while HandleCount >= SockPrimes[i as usize] {
                    i += 1;
                }

                if SockPrimes[i as usize] != 0xFFFFFFFF {
                    HandleCount = SockPrimes[i as usize];
                } else {
                    HandleCount *= 2;
                }

                NewHashTable = HeapAlloc(
                    GlobalHeap,
                    0,
                    (core::mem::size_of::<_WSH_HASH_TABLE>() + (HandleCount as usize) * core::mem::size_of::<PVOID>()) as SIZE_T,
                ) as PWAH_HASH_TABLE;

                if !NewHashTable.is_null() {
                    (*NewHashTable).Size = HandleCount;

                    core::ptr::write_bytes(
                        (*NewHashTable).Handles.as_mut_ptr(),
                        0,
                        HandleCount as usize,
                    );

                    *(*NewHashTable).Handles.as_mut_ptr().add(((*Handle).Handle as usize) % ((*NewHashTable).Size as usize)) = Handle;

                    if !HashTable.is_null() {
                        i = 0;
                        while i < (*HashTable).Size {
                            if !(*(*HashTable).Handles.as_ptr().add(i as usize)).is_null() {
                                HashHandle = &mut *(*NewHashTable).Handles.as_mut_ptr().add(((*(*(*HashTable).Handles.as_ptr().add(i as usize))).Handle as usize) % ((*NewHashTable).Size as usize));

                                if (*HashHandle).is_null() {
                                    *HashHandle = *(*HashTable).Handles.as_ptr().add(i as usize);
                                } else {
                                    HeapFree(GlobalHeap, 0, NewHashTable as PVOID);
                                    continue 'expand;
                                }
                            }
                            i += 1;
                        }

                        (*SearchTable).HashTable = NewHashTable;

                        TryWaitForReaders(SearchTable);
                        HeapFree(GlobalHeap, 0, HashTable as PVOID);
                    } else {
                        (*SearchTable).HashTable = NewHashTable;
                    }

                    OldHandle = Handle;
                } else {
                    OldHandle = Handle;
                }

                break 'inner;
            }
        }

        (*SearchTable).Expanding = 0;
        LeaveCriticalSection(&mut (*SearchTable).Lock);
        break 'outer;
    }

    OldHandle
}

#[no_mangle]
pub unsafe extern "system" fn WahReferenceContextByHandle(
    Table: PWAH_HANDLE_TABLE,
    Handle: HANDLE,
) -> PWAH_HANDLE {
    let mut HashHandle: PWAH_HANDLE;
    let SearchTable: PWAH_SEARCH_TABLE;
    let HashTable: PWAH_HASH_TABLE;
    let mut Count: PVLONG = core::ptr::null_mut();

    SearchTable = (core::ptr::addr_of_mut!((*Table).SearchTables) as *mut WSH_SEARCH_TABLE)
        .offset((((Handle as usize) >> 2) & (*Table).Mask as usize) as isize)
        as PWAH_SEARCH_TABLE;

    AcquireReadLock(SearchTable, core::ptr::addr_of_mut!(Count));

    HashTable = (*SearchTable).HashTable;

    if !HashTable.is_null()
        && {
            HashHandle = *(core::ptr::addr_of_mut!((*HashTable).Handles) as *mut PWAH_HANDLE)
                .offset(((Handle as usize) % ((*HashTable).Size as usize)) as isize);
            !HashHandle.is_null()
        }
        && (*HashHandle).Handle == Handle
    {
        InterlockedIncrement(core::ptr::addr_of_mut!((*HashHandle).RefCount));
    } else {
        HashHandle = core::ptr::null_mut();
    }

    ReleaseReadLock(SearchTable, Count);

    return HashHandle;
}

#[no_mangle]
pub unsafe extern "system" fn WahRemoveHandleContext(
    Table: PWAH_HANDLE_TABLE,
    Handle: PWAH_HANDLE,
) -> DWORD {
    let mut HashHandle: *mut PWAH_HANDLE;
    let SearchTable: PWAH_SEARCH_TABLE;
    let HashTable: PWAH_HASH_TABLE;
    let mut ErrorCode: DWORD = ERROR_SUCCESS;

    SearchTable = core::ptr::addr_of_mut!((*Table).SearchTables)
        .offset((((*Handle).Handle as usize >> 2) & (*Table).Mask as usize) as isize)
        as PWAH_SEARCH_TABLE;

    EnterCriticalSection(core::ptr::addr_of_mut!((*SearchTable).Lock));

    HashTable = (*SearchTable).HashTable;
    HashHandle = core::ptr::addr_of_mut!((*HashTable).Handles)
        .offset(((*Handle).Handle as usize % (*HashTable).Size as usize) as isize)
        as *mut PWAH_HANDLE;

    if !HashHandle.is_null()
        && InterlockedCompareExchangePointer(
            HashHandle as *mut PVOID,
            core::ptr::null_mut(),
            Handle as PVOID,
        ) == Handle as PVOID
    {
        TryWaitForReaders(SearchTable);
    } else {
        ErrorCode = ERROR_INVALID_PARAMETER;
    }

    LeaveCriticalSection(core::ptr::addr_of_mut!((*SearchTable).Lock));

    ErrorCode
}


// ============================================================================
// ORIGINAL C SOURCE
// ============================================================================
//
// /*
//  * COPYRIGHT:   See COPYING in the top level directory
//  * PROJECT:     ReactOS WinSock 2 DLL
//  * FILE:        dll/win32/ws2help/context.c
//  * PURPOSE:     WinSock 2 DLL header
//  */
//
// /* INCLUDES ******************************************************************/
//
// #include "precomp.h"
//
// #include <ws2help.h>
//
// /* DATA **********************************************************************/
//
// CRITICAL_SECTION WshHandleTableLock;
// HANDLE ghWriterEvent;
// DWORD gdwSpinCount = 0;
// DWORD gHandleToIndexMask;
//
// const DWORD SockPrimes[] =
// {
//     31, 61, 127, 257, 521, 1031, 2053,
//     4099, 8191, 16381, 32749, 65537, 131071, 261983,
//     -1
// };
//
// typedef volatile LONG VLONG;
// typedef VLONG *PVLONG;
//
// /* DEFINES *******************************************************************/
//
// /* Yes, we "abuse" the lower bits */
// #define WSH_SEARCH_TABLE_FROM_HANDLE(h, t) \
//     (&t->SearchTables[(((ULONG_PTR)h >> 2) & t->Mask)])
//
// #define WSH_HASH_FROM_HANDLE(h, hs) \
//     (hs->Handles[((ULONG_PTR)h % hs->Size)])
//
// #define AcquireWriteLock(t) \
//     EnterCriticalSection(&t->Lock);
//
// #define ReleaseWriteLock(t) \
//     LeaveCriticalSection(&t->Lock);
//
// /* FUNCTIONS *****************************************************************/
//
// static __inline
// VOID
// AcquireReadLock(IN PWAH_SEARCH_TABLE Table,
//                 IN PVLONG *Count)
// {
//     LONG OldCount;
//
//     /* Start acquire loop */
//     do
//     {
//         /* Write and save count value */
//         *Count = Table->CurrentCount;
//         OldCount = **Count;
//
//         /* Check if it's valid and try to increment it */
//         if ((OldCount > 0) && (InterlockedCompareExchange(*Count,
//                                                           OldCount + 1,
//                                                           OldCount) == OldCount))
//         {
//             /* Everything went OK */
//             break;
//         }
//     } while (TRUE);
// }
//
// static __inline
// VOID
// ReleaseReadLock(IN PWAH_SEARCH_TABLE Table,
//                 IN PVLONG Count)
// {
//     /* Decrement the count. If we went below 0, someone is waiting... */
//     if (InterlockedDecrement(Count) < 0)
//     {
//         /* We use pulse since this is a shared event */
//         PulseEvent(ghWriterEvent);
//     }
// }
//
// VOID
// WINAPI
// DoWaitForReaders(IN PWAH_SEARCH_TABLE Table,
//                  IN PVLONG Counter)
// {
//     HANDLE EventHandle;
//
//     /* Do a context switch */
//     SwitchToThread();
//
//     /* Check if the counter is above one */
//     if (*Counter > 0)
//     {
//         /*
//          * This shouldn't happen unless priorities are messed up. Do a wait so
//          * that the threads with lower priority will get their chance now.
//          */
//         if (!ghWriterEvent)
//         {
//             /* We don't even have an event! Allocate one manually... */
//             EventHandle = CreateEvent(NULL, TRUE, FALSE, NULL);
//             if (EventHandle)
//             {
//                 /* Save the event handle atomically */
//                 if ((InterlockedCompareExchangePointer((PVOID*)&ghWriterEvent,
//                                                        EventHandle,
//                                                        NULL)))
//                 {
//                     /* Someone beat us to it, close ours */
//                     CloseHandle(EventHandle);
//                 }
//             }
//             else
//             {
//                 /* Things couldn't get worse for us. Do a last-resort hack */
//                 while (*Counter > 0) Sleep(10);
//             }
//         }
//
//         /*
//          * Our event is ready. Tell the others to signal us by making sure
//          * that the last counter will be -1, notifying the last thread of our
//          * request.
//          */
//         if (InterlockedDecrement(Counter) >= 0)
//         {
//             /* Keep looping */
//             do
//             {
//                 /* Wait in tiny bursts, so we can catch the PulseEvent */
//                 WaitForSingleObject(ghWriterEvent, 10);
//             } while (*Counter >= 0);
//         }
//     }
// }
//
// static __inline
// VOID
// TryWaitForReaders(IN PWAH_SEARCH_TABLE Table)
// {
//     PVLONG OldCount = Table->CurrentCount;
//     LONG SpinCount;
//
//     /* See which counter is being used */
//     if (OldCount == &Table->Count1)
//     {
//         /* Use counter 2 now */
//         Table->Count2 = 1;
//         Table->CurrentCount = &Table->Count2;
//     }
//     else
//     {
//         /* Use counter 1 now */
//         Table->Count1 = 1;
//         Table->CurrentCount = &Table->Count1;
//     }
//
//     /* Decrease the old count to block new readers */
//     if (InterlockedDecrement(OldCount) > 0)
//     {
//         /* On an MP machine, spin a bit first */
//         if (Table->SpinCount)
//         {
//             /* Get the spincount and loop it */
//             SpinCount = Table->SpinCount;
//             while (*OldCount > 0)
//             {
//                 /* Check if the spin failed */
//                 if (--SpinCount <= 0) break;
//             }
//         }
//
//         /* Check one last time if someone is still active */
//         if (*OldCount > 0)
//         {
//             /* Yep, we'll have to do a blocking (slow) wait */
//             DoWaitForReaders(Table, OldCount);
//         }
//     }
// }
//
// DWORD
// WINAPI
// WahCreateHandleContextTable(OUT PWAH_HANDLE_TABLE *Table)
// {
//     DWORD ErrorCode;
//     PWAH_HANDLE_TABLE LocalTable;
//     DWORD i;
//
//     /* Enter the prolog, make sure we're initialized */
//     ErrorCode = WS2HELP_PROLOG();
//     if (ErrorCode != ERROR_SUCCESS) return ErrorCode;
//
//     /* Assume NULL */
//     *Table = NULL;
//
//     /* Allocate enough tables */
//     LocalTable = HeapAlloc(GlobalHeap,
//                            0,
//                            FIELD_OFFSET(WSH_HANDLE_TABLE,
//                                         SearchTables[gHandleToIndexMask + 1]));
//
//     /* Make sure it was allocated */
//     if (!LocalTable) return WSA_NOT_ENOUGH_MEMORY;
//
//     /* Set the mask for the table */
//     LocalTable->Mask = gHandleToIndexMask;
//
//     /* Now initialize every table */
//     for (i = 0; i <= gHandleToIndexMask; i++)
//     {
//         /* No hash table yet */
//         LocalTable->SearchTables[i].HashTable = NULL;
//
//         /* Set the current count */
//         LocalTable->SearchTables[i].CurrentCount = &LocalTable->SearchTables[i].Count1;
//
//         /* Initialize the counts */
//         LocalTable->SearchTables[i].Count1 = 1;
//         LocalTable->SearchTables[i].Count2 = 0;
//
//         /* Set expanding state and spin count */
//         LocalTable->SearchTables[i].Expanding = FALSE;
//         LocalTable->SearchTables[i].SpinCount = gdwSpinCount;
//
//         /* Initialize the lock */
//         (VOID)InitializeCriticalSectionAndSpinCount(&LocalTable->
//                                                     SearchTables[i].Lock,
//                                                     gdwSpinCount);
//     }
//
//     /* Return pointer */
//     *Table = LocalTable;
//
//     /* Return success */
//     return ERROR_SUCCESS;
// }
//
// DWORD
// WINAPI
// WahDestroyHandleContextTable(IN PWAH_HANDLE_TABLE Table)
// {
//     DWORD i;
//
//     /* Make sure the table is valid */
//     if (!Table)
//     {
//         /* No valid table */
//         return ERROR_INVALID_PARAMETER;
//     }
//
//     /* Loop each search table */
//     for (i = 0; i <= Table->Mask; i++)
//     {
//         /* Check if there's a table here */
//         if (Table->SearchTables[i].HashTable)
//         {
//             /* Free it */
//             HeapFree(GlobalHeap, 0, Table->SearchTables[i].HashTable);
//         }
//
//         /* Delete the lock */
//         DeleteCriticalSection(&Table->SearchTables[i].Lock);
//     }
//
//     /* Delete the table */
//     HeapFree(GlobalHeap, 0, Table);
//
//     /* Return success */
//     return ERROR_SUCCESS;
// }
//
// BOOL
// WINAPI
// WahEnumerateHandleContexts(IN PWAH_HANDLE_TABLE Table,
//                            IN PWAH_HANDLE_ENUMERATE_PROC Callback,
//                            IN PVOID Context)
// {
//     DWORD i, j;
//     PWAH_SEARCH_TABLE SearchTable;
//     PWAH_HASH_TABLE HashTable;
//     PWAH_HANDLE Handle;
//     BOOL GoOn = TRUE;
//
//     /* Loop the table */
//     for (i = 0; i <= Table->Mask; i++)
//     {
//         /* Get the Search table */
//         SearchTable = &Table->SearchTables[i];
//
//         /* Lock it */
//         AcquireWriteLock(SearchTable);
//
//         /* Mark us as expanding and wait for everyone */
//         SearchTable->Expanding = TRUE;
//         TryWaitForReaders(SearchTable);
//
//         /* Get the hash table */
//         HashTable = SearchTable->HashTable;
//
//         /* Make sure it exists */
//         if (HashTable)
//         {
//             /* Loop every handle in it */
//             for (j = 0; j < HashTable->Size; j++)
//             {
//                 /* Get this handle */
//                 Handle = HashTable->Handles[j];
//                 if (!Handle) continue;
//
//                 /* Call the callback proc */
//                 GoOn = Callback(Context, Handle);
//                 if (!GoOn) break;
//             }
//         }
//
//         /* Disable the expansion bit and release the lock */
//         SearchTable->Expanding = FALSE;
//         ReleaseWriteLock(SearchTable);
//
//         /* Check again if we should leave */
//         if (!GoOn) break;
//     }
//
//     /* return */
//     return GoOn;
// }
//
// PWAH_HANDLE
// WINAPI
// WahInsertHandleContext(IN PWAH_HANDLE_TABLE Table,
//                        IN PWAH_HANDLE Handle)
// {
//     PWAH_HANDLE *HashHandle, OldHandle;
//     PVLONG Count;
//     PWAH_HASH_TABLE HashTable, NewHashTable;
//     DWORD HandleCount, i;
//     PWAH_SEARCH_TABLE SearchTable;
//
//     /* Get the current Search Table */
//     SearchTable = WSH_SEARCH_TABLE_FROM_HANDLE(Handle->Handle, Table);
//
//     /* Start loop */
//     do
//     {
//         /* Get reader lock */
//         AcquireReadLock(SearchTable, &Count);
//
//         /* Get the hash table */
//         HashTable = SearchTable->HashTable;
//
//         /* Make sure we are not expanding, and that the table is there */
//         if (!(SearchTable->Expanding) && (HashTable))
//         {
//             /* Get the hash handle */
//             HashHandle = &WSH_HASH_FROM_HANDLE(Handle->Handle, HashTable);
//
//             /* Do the insert */
//             if (InterlockedCompareExchangePointer((PVOID*)HashHandle,
//                                                   Handle,
//                                                   NULL) == NULL)
//             {
//                 /* Success, release the reader lock */
//                 ReleaseReadLock(SearchTable, Count);
//
//                 /* Save old handle */
//                 OldHandle = Handle;
//                 break;
//             }
//         }
//
//         /* Release the read lock since we're done with it now */
//         ReleaseReadLock(SearchTable, Count);
//
//         /* We need the writer lock to expand/create the table */
//         AcquireWriteLock(SearchTable);
//
//         /* Mark the table in use */
//         SearchTable->Expanding = TRUE;
//
//         /* Wait for all the readers to finish */
//         TryWaitForReaders(SearchTable);
//
//         /* Start loop */
//         do
//         {
//             /* Get the hash table again */
//             HashTable = SearchTable->HashTable;
//
//             /* Check if exists now */
//             if (HashTable)
//             {
//                 /* It does! Do what we wanted to do earlier. Get the hash... */
//                 HashHandle = &WSH_HASH_FROM_HANDLE(Handle->Handle, HashTable);
//
//                 /* Check if it doesn't exist */
//                 if (!(*HashHandle))
//                 {
//                     /* Write it (no need for interlock, we have the RW lock) */
//                     OldHandle = Handle;
//                     *HashHandle = Handle;
//                     break;
//                 }
//                 else if ((*HashHandle)->Handle == Handle->Handle)
//                 {
//                     /* Handle matches, write it (see comment above) */
//                     OldHandle = *HashHandle;
//                     *HashHandle = Handle;
//                     break;
//                 }
//
//                 /* No go, we need to expand the table. Remember the size now */
//                 HandleCount = HashTable->Size;
//             }
//             else
//             {
//                 /* Table is empty, we have to create it */
//                 HandleCount = 0;
//             }
//
// ExpandTable:
//             /* Find a free prime */
//             for (i = 0; HandleCount >= SockPrimes[i]; i++);
//
//             /* Check if we found one */
//             if (SockPrimes[i] != 0xFFFFFFFF)
//             {
//                 /* Use the prime */
//                 HandleCount = SockPrimes[i];
//             }
//             else
//             {
//                 /* No primes left. Table is quite large, so simply double it */
//                 HandleCount *= 2;
//             }
//
//             /* Allocate the table */
//             NewHashTable = HeapAlloc(GlobalHeap,
//                                      0,
//                                      FIELD_OFFSET(WSH_HASH_TABLE,
//                                                   Handles[HandleCount]));
//
//             /* Hopefully we have one now */
//             if (NewHashTable)
//             {
//                 /* Set its size */
//                 NewHashTable->Size = HandleCount;
//
//                 /* Initialize it */
//                 RtlZeroMemory(NewHashTable->Handles, HandleCount * sizeof(PVOID));
//
//                 /* Insert us first */
//                 WSH_HASH_FROM_HANDLE(Handle->Handle, NewHashTable) = Handle;
//
//                 /* Now check if our old table had entries in it */
//                 if (HashTable)
//                 {
//                     /* We need to move them */
//                     for (i = 0; i < HashTable->Size; i++)
//                     {
//                         /* Make sure the hash handle exists */
//                         if (HashTable->Handles[i])
//                         {
//                             /* Get it */
//                             HashHandle = &WSH_HASH_FROM_HANDLE(HashTable->
//                                                                Handles[i]->Handle,
//                                                                NewHashTable);
//
//                             /* Check if it has a value */
//                             if (!(*HashHandle))
//                             {
//                                 /* It's empty, so just write the handle */
//                                 *HashHandle = HashTable->Handles[i];
//                             }
//                             else
//                             {
//                                 /* Not empty :/... that implies a collision */
//                                 HeapFree(GlobalHeap, 0, NewHashTable);
//                                 goto ExpandTable;
//                             }
//                         }
//                     }
//
//                     /* Write the new hash table */
//                     SearchTable->HashTable = NewHashTable;
//
//                     /* Wait for everyone to be done with it, then free it */
//                     TryWaitForReaders(SearchTable);
//                     HeapFree(GlobalHeap, 0, HashTable);
//                 }
//                 else
//                 {
//                     /* It was empty, nothing to worry about */
//                     SearchTable->HashTable = NewHashTable;
//                 }
//
//                 /* Save the old handle */
//                 OldHandle = Handle;
//             }
//             else
//             {
//                 /* There was no old handle */
//                 OldHandle = Handle;
//             }
//         } while (0);
//
//         /* Mark us as free, and release the write lock */
//         SearchTable->Expanding = FALSE;
//         ReleaseWriteLock(SearchTable);
//         break;
//     } while (1);
//
//     /* Return the old handle */
//     return OldHandle;
// }
//
// PWAH_HANDLE
// WINAPI
// WahReferenceContextByHandle(IN PWAH_HANDLE_TABLE Table,
//                             IN HANDLE Handle)
// {
//     PWAH_HANDLE HashHandle;
//     PWAH_SEARCH_TABLE SearchTable;
//     PWAH_HASH_TABLE HashTable;
//     PVLONG Count;
//
//     /* Get the current Search Table */
//     SearchTable = WSH_SEARCH_TABLE_FROM_HANDLE(Handle, Table);
//
//     /* Lock it */
//     AcquireReadLock(SearchTable, &Count);
//
//     /* Get the hash table and handle */
//     HashTable = SearchTable->HashTable;
//
//     /* Check if it's valid, and if it's the one we want */
//     if ((HashTable) &&
//         (HashHandle = WSH_HASH_FROM_HANDLE(Handle, HashTable)) &&
//         (HashHandle->Handle == Handle))
//     {
//         /* Reference the handle */
//         InterlockedIncrement(&HashHandle->RefCount);
//     }
//     else
//     {
//         /* Invalid handle */
//         HashHandle = NULL;
//     }
//
//     /* Release the lock */
//     ReleaseReadLock(SearchTable, Count);
//
//     /* Return */
//     return HashHandle;
// }
//
// DWORD
// WINAPI
// WahRemoveHandleContext(IN PWAH_HANDLE_TABLE Table,
//                        IN PWAH_HANDLE Handle)
// {
//     PWAH_HANDLE *HashHandle;
//     PWAH_SEARCH_TABLE SearchTable;
//     PWAH_HASH_TABLE HashTable;
//     DWORD ErrorCode = ERROR_SUCCESS;
//
//     /* Get the current Search Table */
//     SearchTable = WSH_SEARCH_TABLE_FROM_HANDLE(Handle->Handle, Table);
//
//     /* Lock it */
//     AcquireWriteLock(SearchTable);
//
//     /* Get the hash table and handle */
//     HashTable = SearchTable->HashTable;
//     HashHandle = &WSH_HASH_FROM_HANDLE(Handle->Handle, HashTable);
//
//     /* Make sure we have a handle, and write the new pointer */
//     if (HashHandle && (InterlockedCompareExchangePointer((PVOID*)HashHandle,
//                                                          NULL,
//                                                          Handle) == Handle))
//     {
//         /* Wait for everyone to be done with it */
//         TryWaitForReaders(SearchTable);
//     }
//     else
//     {
//         /* Invalid handle */
//         ErrorCode = ERROR_INVALID_PARAMETER;
//     }
//
//     /* Release the lock */
//     ReleaseWriteLock(SearchTable);
//
//     /* Return */
//     return ErrorCode;
// }
//
// /* EOF */
//
// ============================================================================
// END ORIGINAL C SOURCE
// ============================================================================
