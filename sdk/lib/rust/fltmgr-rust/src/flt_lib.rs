use ntkrnl_sys::globals::*;
use ntkrnl_sys::constants::*;
use ntkrnl_sys::base::*;
use ntkrnl_sys::ex::*;
use ntkrnl_sys::mm::*;
use ntkrnl_sys::rtl::*;
use ntkrnl_sys::ddk::{PUNICODE_STRING, PWCH};
use core::ffi::c_void;
use crate::types::*;
use crate::glue::*;

// ---- file-scope preamble (translated) ----
// #define NDEBUG (no Rust declaration needed)
#[no_mangle]
pub unsafe extern "C" fn FltpFreeUnicodeString(String: PUNICODE_STRING) {
    /* Free up any existing buffer */
    if !(*String).Buffer.is_null() {
        ExFreePoolWithTag((*String).Buffer as PVOID, FM_TAG_UNICODE_STRING as ULONG);
    }

    /* Empty the string */
    (*String).Buffer = core::ptr::null_mut::<WCHAR>();
    (*String).Length = 0 as USHORT;
    (*String).MaximumLength = 0 as USHORT;
}

const FM_TAG_UNICODE_STRING: u32 = 0x73754D46; // 'suMF' drivers/filters/fltmgr/fltmgr.h
#[no_mangle]
pub unsafe extern "C" fn FltpReallocateUnicodeString(
    mut String: PUNICODE_STRING,
    mut NewLength: SIZE_T,
    mut CopyExisting: BOOLEAN,
) -> NTSTATUS {
    let mut NewBuffer: PWCH = core::ptr::null_mut();

    /* Don't bother reallocating if the buffer is smaller */
    if NewLength <= (*String).MaximumLength as _ {
        (*String).Length = 0;
        return STATUS_SUCCESS;
    }

    /* Allocate a new buffer at the size requested */
    NewBuffer = ExAllocatePoolWithTag(PagedPool, NewLength, FM_TAG_UNICODE_STRING) as PWCH;
    if NewBuffer.is_null() {
        return STATUS_INSUFFICIENT_RESOURCES;
    }

    if CopyExisting != 0 {
        /* Copy the old data across */
        RtlCopyMemory(
            NewBuffer as *mut core::ffi::c_void,
            (*String).Buffer as *const core::ffi::c_void,
            (*String).Length as usize,
        );
    } else {
        /* Reset the length */
        (*String).Length = 0;
    }

    /* Free any old buffer */
    if !(*String).Buffer.is_null() {
        ExFreePoolWithTag((*String).Buffer as *mut core::ffi::c_void, FM_TAG_UNICODE_STRING);
    }

    /* Update the lengths */
    (*String).Buffer = NewBuffer;
    (*String).MaximumLength = NewLength as u16;

    return STATUS_SUCCESS;
}
#[no_mangle]
pub unsafe extern "C" fn FltpCopyUnicodeString(
    mut StringOne: PUNICODE_STRING,
    mut StringTwo: PUNICODE_STRING,
) -> NTSTATUS {
    return 0 as NTSTATUS;
}


// ============================================================================
// ORIGINAL C SOURCE
// ============================================================================
//
// /*
// * PROJECT:         Filesystem Filter Manager
// * LICENSE:         GPL - See COPYING in the top level directory
// * FILE:            drivers/filters/fltmgr/Lib.c
// * PURPOSE:         Miscellaneous library functions
// * PROGRAMMERS:     Ged Murphy (gedmurphy@reactos.org)
// */
//
// /* INCLUDES ******************************************************************/
//
// #include "fltmgr.h"
//
// #define NDEBUG
// #include <debug.h>
//
//
// /* DATA *********************************************************************/
//
//
//
// /* FUNCTIONS **********************************************/
//
// VOID
// FltpFreeUnicodeString( PUNICODE_STRING String)
// {
//     /* Free up any existing buffer */
//     if (String->Buffer)
//     {
//         ExFreePoolWithTag(String->Buffer, FM_TAG_UNICODE_STRING);
//     }
//
//     /* Empty the string */
//     String->Buffer = NULL;
//     String->Length = 0;
//     String->MaximumLength = 0;
// }
//
// NTSTATUS
// FltpReallocateUnicodeString( PUNICODE_STRING String,
//                              SIZE_T NewLength,
//                              BOOLEAN CopyExisting)
// {
//     PWCH NewBuffer;
//
//     /* Don't bother reallocating if the buffer is smaller */
//     if (NewLength <= String->MaximumLength)
//     {
//         String->Length = 0;
//         return STATUS_SUCCESS;
//     }
//
//     /* Allocate a new buffer at the size requested */
//     NewBuffer = ExAllocatePoolWithTag(PagedPool, NewLength, FM_TAG_UNICODE_STRING);
//     if (NewBuffer == NULL) return STATUS_INSUFFICIENT_RESOURCES;
//
//     if (CopyExisting)
//     {
//         /* Copy the old data across */
//         RtlCopyMemory(NewBuffer, String->Buffer, String->Length);
//     }
//     else
//     {
//         /* Reset the length */
//         String->Length = 0;
//     }
//
//     /* Free any old buffer */
//     if (String->Buffer)
//         ExFreePoolWithTag(String->Buffer, FM_TAG_UNICODE_STRING);
//
//     /* Update the lengths */
//     String->Buffer = NewBuffer;
//     String->MaximumLength = NewLength;
//
//     return STATUS_SUCCESS;
// }
//
// NTSTATUS
// FltpCopyUnicodeString( PUNICODE_STRING StringOne,
//                        PUNICODE_STRING StringTwo)
// {
//     return 0;
// }
//
// ============================================================================
// END ORIGINAL C SOURCE
// ============================================================================
