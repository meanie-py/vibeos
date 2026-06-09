use crate::types::*;
use ntkrnl_sys::globals::*;
use ntkrnl_sys::constants::*;
use ntkrnl_sys::base::*;
use ntkrnl_sys::ke::*;
use ntkrnl_sys::rtl::*;
use ntkrnl_sys::ddk::{FILE_OBJECT_EXTENSION, KeInitializeSpinLock};
use core::ffi::c_void;

extern "system" {
    fn FREE_POOL(ptr: *mut core::ffi::c_void);
    fn NT_ASSERT(expr: i32);
}

// ---- file-scope preamble (translated) ----
pub const DICTIONARY_SIGNATURE: u32 = 0x44696374; // 'tciD'
pub const MdlMappingNoExecute: u32 = 0;
pub const NonPagedPoolNx: u32 = 0;            // NonPagedPool
pub const NonPagedPoolNxCacheAligned: u32 = 0; // NonPagedPoolCacheAligned
pub const POOL_NX_ALLOCATION: u32 = 0;

#[repr(C)]
pub struct DICTIONARY_HEADER {
    pub Next: *mut DICTIONARY_HEADER,
    pub Key: u64,
    pub Data: [u8; 0],
}
pub type PDICTIONARY_HEADER = *mut DICTIONARY_HEADER;

const DICTIONARY_SIGNATURE: u32 = 0x44636974;
#[no_mangle]
pub unsafe extern "C" fn InitializeDictionary(Dictionary: *mut DICTIONARY) {
    core::ptr::write_bytes(Dictionary as *mut u8, 0, core::mem::size_of::<DICTIONARY>());
    (*Dictionary).Signature = DICTIONARY_SIGNATURE;
    KeInitializeSpinLock(core::ptr::addr_of_mut!((*Dictionary).SpinLock));
}
#[no_mangle]
pub unsafe extern "C" fn TestDictionarySignature(Dictionary: PDICTIONARY) -> BOOLEAN {
    unsafe {
        ((*Dictionary).Signature == DICTIONARY_SIGNATURE.into()) as BOOLEAN
    }
}
#[no_mangle]
pub unsafe extern "C" fn GetDictionaryEntry(
    Dictionary: *mut DICTIONARY,
    Key: u64,
) -> *mut core::ffi::c_void {
    let mut data: *mut core::ffi::c_void;
    let mut oldIrql: KIRQL = core::mem::zeroed();
    let mut entry: *mut DICTIONARY_HEADER;

    data = core::ptr::null_mut();

    KeAcquireSpinLock(
        core::ptr::addr_of_mut!((*Dictionary).SpinLock),
        core::ptr::addr_of_mut!(oldIrql),
    );

    entry = (*Dictionary).List;
    while !entry.is_null() {
        if (*entry).Key == Key {
            data = core::ptr::addr_of!((*entry).Data) as *mut core::ffi::c_void;
            break;
        } else {
            entry = (*entry).Next;
        }
    }

    KeReleaseSpinLock(
        core::ptr::addr_of_mut!((*Dictionary).SpinLock),
        oldIrql,
    );

    return data;
}
#[no_mangle]
pub unsafe extern "C" fn FreeDictionaryEntry(
    Dictionary: *mut DICTIONARY,
    Entry: *mut core::ffi::c_void,
) {
    let mut found: u8 = 0;
    let offset = core::mem::offset_of!(DICTIONARY_HEADER, Data);
    let header = (Entry as *mut u8).byte_offset(-(offset as isize)) as *mut DICTIONARY_HEADER;

    let mut oldIrql: KIRQL = core::mem::zeroed();
    KeAcquireSpinLock(
        core::ptr::addr_of_mut!((*Dictionary).SpinLock),
        core::ptr::addr_of_mut!(oldIrql),
    );

    let mut entry = core::ptr::addr_of_mut!((*Dictionary).List);
    while !(*entry).is_null() {
        if *entry == header {
            *entry = (*header).Next;
            found = 1;
            break;
        } else {
            entry = core::ptr::addr_of_mut!((*(*entry)).Next);
        }
    }

    KeReleaseSpinLock(core::ptr::addr_of_mut!((*Dictionary).SpinLock), oldIrql);

    NT_ASSERT(found as i32);
    if found != 0 {
        FREE_POOL(header as *mut core::ffi::c_void);
    }

    return;
}


// ============================================================================
// ORIGINAL C SOURCE
// ============================================================================
//
// /*++
//
// Copyright (C) Microsoft Corporation, 1990 - 1999
//
// Module Name:
//
//     dictlib.c
//
// Abstract:
//
//     Support library for maintaining a dictionary list (list of objects
//     referenced by a key value).
//
// Environment:
//
//     kernel mode only
//
// Notes:
//
//     This module generates a static library
//
// Revision History:
//
// --*/
//
// #include <ntddk.h>
// #include <classpnp.h>
//
// #ifdef __REACTOS__
// #undef MdlMappingNoExecute
// #define MdlMappingNoExecute 0
// #define NonPagedPoolNx NonPagedPool
// #define NonPagedPoolNxCacheAligned NonPagedPoolCacheAligned
// #undef POOL_NX_ALLOCATION
// #define POOL_NX_ALLOCATION 0
// #endif
//
// #define DICTIONARY_SIGNATURE 'tciD'
//
// #ifdef _MSC_VER
// #pragma warning(push)
// #pragma warning(disable:4200) // nonstandard extension used : zero-sized array in struct/union
// #endif
// struct _DICTIONARY_HEADER {
//     PDICTIONARY_HEADER Next;
//     ULONGLONG Key;
//     UCHAR Data[0];
// };
// #ifdef _MSC_VER
// #pragma warning(pop)
// #endif
//
// struct _DICTIONARY_HEADER;
// typedef struct _DICTIONARY_HEADER DICTIONARY_HEADER, *PDICTIONARY_HEADER;
//
//
// VOID
// InitializeDictionary(
//     IN PDICTIONARY Dictionary
//     )
// {
//     RtlZeroMemory(Dictionary, sizeof(DICTIONARY));
//     Dictionary->Signature = DICTIONARY_SIGNATURE;
//     KeInitializeSpinLock(&Dictionary->SpinLock);
//     return;
// }
//
//
// BOOLEAN
// TestDictionarySignature(
//     IN PDICTIONARY Dictionary
//     )
// {
//     return Dictionary->Signature == DICTIONARY_SIGNATURE;
// }
//
// NTSTATUS
// AllocateDictionaryEntry(
//     IN PDICTIONARY Dictionary,
//     IN ULONGLONG Key,
//     (0, sizeof(FILE_OBJECT_EXTENSION)) IN ULONG Size,
//     IN ULONG Tag,
//     OUT PVOID *Entry
//     )
// {
//     PDICTIONARY_HEADER header;
//     KIRQL oldIrql;
//     PDICTIONARY_HEADER *entry;
//
//     NTSTATUS status = STATUS_SUCCESS;
//
//     *Entry = NULL;
//
//     header = ExAllocatePoolWithTag(NonPagedPoolNx,
//                                    Size + sizeof(DICTIONARY_HEADER),
//                                    Tag);
//
//     if(header == NULL) {
//         return STATUS_INSUFFICIENT_RESOURCES;
//     }
//
//     RtlZeroMemory(header, sizeof(DICTIONARY_HEADER) + Size);
//     header->Key = Key;
//
//     //
//     // Find the correct location for this entry in the dictionary.
//     //
//
//     KeAcquireSpinLock(&(Dictionary->SpinLock), &oldIrql);
//
//     TRY {
//
//         entry = &(Dictionary->List);
//
//         while(*entry != NULL) {
//             if((*entry)->Key == Key) {
//
//                 //
//                 // Dictionary must have unique keys.
//                 //
//
//                 status = STATUS_OBJECT_NAME_COLLISION;
//                 LEAVE;
//
//             } else if ((*entry)->Key < Key) {
//
//                 //
//                 // We will go ahead and insert the key in here.
//                 //
//                 break;
//             } else {
//                 entry = &((*entry)->Next);
//             }
//         }
//
//         //
//         // If we make it here then we will go ahead and do the insertion.
//         //
//
//         header->Next = *entry;
//         *entry = header;
//
//     } FINALLY {
//         KeReleaseSpinLock(&(Dictionary->SpinLock), oldIrql);
//
//         if(!NT_SUCCESS(status)) {
//             FREE_POOL(header);
//         } else {
//             *Entry = (PVOID) header->Data;
//         }
//     }
//     return status;
// }
//
//
// PVOID
// GetDictionaryEntry(
//     IN PDICTIONARY Dictionary,
//     IN ULONGLONG Key
//     )
// {
//     PDICTIONARY_HEADER entry;
//     PVOID data;
//     KIRQL oldIrql;
//
//
//     data = NULL;
//
//     KeAcquireSpinLock(&(Dictionary->SpinLock), &oldIrql);
//
//     entry = Dictionary->List;
//     while (entry != NULL) {
//
//         if (entry->Key == Key) {
//             data = entry->Data;
//             break;
//         } else {
//             entry = entry->Next;
//         }
//     }
//
//     KeReleaseSpinLock(&(Dictionary->SpinLock), oldIrql);
//
//     return data;
// }
//
//
// VOID
// FreeDictionaryEntry(
//     IN PDICTIONARY Dictionary,
//     IN PVOID Entry
//     )
// {
//     PDICTIONARY_HEADER header;
//     PDICTIONARY_HEADER *entry;
//     KIRQL oldIrql;
//     BOOLEAN found;
//
//     found = FALSE;
//     header = CONTAINING_RECORD(Entry, DICTIONARY_HEADER, Data);
//
//     KeAcquireSpinLock(&(Dictionary->SpinLock), &oldIrql);
//
//     entry = &(Dictionary->List);
//     while(*entry != NULL) {
//
//         if(*entry == header) {
//             *entry = header->Next;
//             found = TRUE;
//             break;
//         } else {
//             entry = &(*entry)->Next;
//         }
//     }
//
//     KeReleaseSpinLock(&(Dictionary->SpinLock), oldIrql);
//
//     //
//     // calling this w/an invalid pointer invalidates the dictionary system,
//     // so NT_ASSERT() that we never try to Free something not in the list
//     //
//
//     NT_ASSERT(found);
//     if (found) {
//         FREE_POOL(header);
//     }
//
//     return;
//
// }
//
//
// ============================================================================
// END ORIGINAL C SOURCE
// ============================================================================
