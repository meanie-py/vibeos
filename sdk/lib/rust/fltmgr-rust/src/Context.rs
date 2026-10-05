use ntkrnl_sys::globals::*;
use ntkrnl_sys::constants::*;
use ntkrnl_sys::base::*;
use ntkrnl_sys::ex::*;
use ntkrnl_sys::mm::*;
use ntkrnl_sys::rtl::*;
use core::ffi::c_void;
use crate::types::*;
use crate::glue::*;

// RtlAssert is provided as a no-op in crate::glue (imported via `use crate::glue::*`).

// ---- file-scope preamble (translated) ----
#[allow(non_upper_case_globals)]
pub const NDEBUG: () = ();

#[no_mangle]
pub unsafe extern "C" fn FltpRegisterContexts(
    mut Filter: PFLT_FILTER,
    mut Context: *const FLT_CONTEXT_REGISTRATION,
) -> NTSTATUS {
    // FLT_CONTEXT_END comes from the fltkernel.h island (0xffff). A local `-1` shadowed
    // it here: ContextType is a USHORT, so the widened compare could never match, the
    // terminator scan ran off the array, and LLVM reduced the side-effect-free endless
    // walk to a bare `b .` in FltRegisterFilter (scan_ud2).
    const MAX_CONTEXT_TYPES: ::core::ffi::c_int = 6;
    const FM_TAG_CONTEXT_REGISTA: ULONG = 0x67655243; // 'CReg'
    const STATUS_FLT_INVALID_CONTEXT_REGISTRATION: NTSTATUS = 0xC01C0017_u32 as i32;

    let mut ContextPtr: PCFLT_CONTEXT_REGISTRATION;
    let mut ContextHeader: PALLOCATE_CONTEXT_HEADER;
    let mut Prev: PALLOCATE_CONTEXT_HEADER;
    let mut Buffer: PVOID;
    let mut BufferSize: ULONG = 0 as ULONG;
    let mut i: USHORT;
    let mut Status: NTSTATUS;

    /* Loop through all entries in the context registration array */
    ContextPtr = Context as PCFLT_CONTEXT_REGISTRATION;
    while !ContextPtr.is_null() {
        /* Bail if we found the terminator */
        if (*ContextPtr).ContextType as u32 == FLT_CONTEXT_END {
            break;
        }

        /* Make sure we have a valid context  */
        if IsContextTypeValid((*ContextPtr).ContextType) != 0 {
            /* Each context is backed by a crtl struct. Reserve space for it */
            BufferSize =
                BufferSize.wrapping_add(::core::mem::size_of::<STREAM_LIST_CTRL>() as ULONG);
        }

        /* Move to the next entry */
        ContextPtr = ContextPtr.offset(1);
    }

    /* Bail if we found no valid registration requests */
    if BufferSize == 0 {
        return STATUS_SUCCESS;
    }

    /* Allocate the pool that'll hold the context crtl structs */
    Buffer = ExAllocatePoolWithTag(NonPagedPool, BufferSize as SIZE_T, FM_TAG_CONTEXT_REGISTA);
    if Buffer.is_null() {
        return STATUS_INSUFFICIENT_RESOURCES;
    }

    RtlZeroMemory(Buffer, BufferSize as SIZE_T);

    /* Setup our loop data */
    ContextHeader = Buffer as PALLOCATE_CONTEXT_HEADER;
    Prev = core::ptr::null_mut();
    Status = STATUS_SUCCESS;

    i = 0 as USHORT;
    's_78: while (i as ::core::ffi::c_int) < MAX_CONTEXT_TYPES {
        ContextPtr = Context as PFLT_CONTEXT_REGISTRATION as PCFLT_CONTEXT_REGISTRATION;
        while !ContextPtr.is_null() {
            /* We don't support variable sized contents yet */
            if !((*ContextPtr).Size != -(1 as ::core::ffi::c_int) as _) {
                RtlAssert(
                    b"ContextPtr->Size != ((SIZE_T)-1)\0" as *const u8 as *const i8 as PVOID,
                    b"drivers/filters/fltmgr/Context.c\0"
                        as *const u8 as *const i8 as PVOID,
                    95 as ULONG,
                    core::ptr::null_mut::<CHAR>(),
                );
            } else {
            };

            /* Bail if we found the terminator */
            if (*ContextPtr).ContextType as u32 == FLT_CONTEXT_END {
                break;
            }

            /* Size and pooltag are only checked when ContextAllocateCallback is null */
            if (*ContextPtr).ContextAllocateCallback.is_none()
                && (*ContextPtr).PoolTag == 0 as ::core::ffi::c_ulong
            {
                Status = STATUS_FLT_INVALID_CONTEXT_REGISTRATION;
                break 's_78;
            } else {
                /* Make sure we have a valid context  */
                if IsContextTypeValid((*ContextPtr).ContextType) != 0 {
                    Status = SetupContextHeader(Filter, ContextPtr, ContextHeader);
                    if NT_SUCCESS(Status) {
                        if !Prev.is_null() {
                            (*Prev).Next = ContextHeader;
                        }

                        (*Filter).SupportedContexts[i as usize] = ContextHeader;
                    }
                }

                Prev = ContextHeader;

                /* Move to the next entry */
                ContextPtr = ContextPtr.offset(1);
            }
        }
        i = i.wrapping_add(1);
    }

    // Quit:
    if NT_SUCCESS(Status) {
        (*Filter).SupportedContextsListHead = Buffer as PALLOCATE_CONTEXT_HEADER;
    } else {
        ExFreePoolWithTag(Buffer, FM_TAG_CONTEXT_REGISTA);
        //FIXME: Cleanup anything that SetupContextHeader may have allocated
    }

    return Status;
}
unsafe fn IsContextTypeValid(mut ContextType: FLT_CONTEXT_TYPE) -> BOOLEAN {
    match ContextType as u32 {
        FLT_VOLUME_CONTEXT
        | FLT_INSTANCE_CONTEXT
        | FLT_FILE_CONTEXT
        | FLT_STREAM_CONTEXT
        | FLT_STREAMHANDLE_CONTEXT
        | FLT_TRANSACTION_CONTEXT => return TRUE as BOOLEAN,
        _ => {}
    }
    return FALSE as BOOLEAN;
}
unsafe fn SetupContextHeader(
    mut Filter: PFLT_FILTER,
    mut ContextPtr: PCFLT_CONTEXT_REGISTRATION,
    mut ContextHeader: PALLOCATE_CONTEXT_HEADER,
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
// * FILE:            drivers/filters/fltmgr/Context.c
// * PURPOSE:         Contains context routines
// * PROGRAMMERS:     Ged Murphy (gedmurphy@reactos.org)
// */
//
// /* INCLUDES ******************************************************************/
//
// #include "fltmgr.h"
// #include "fltmgrint.h"
//
// #define NDEBUG
// #include <debug.h>
//
//
// /* DATA *********************************************************************/
//
// static
// BOOLEAN
// IsContextTypeValid(
//      FLT_CONTEXT_TYPE ContextType
// );
//
// static
// NTSTATUS
// SetupContextHeader(
//      PFLT_FILTER Filter,
//      PCFLT_CONTEXT_REGISTRATION ContextPtr,
//      PALLOCATE_CONTEXT_HEADER ContextHeader
// );
//
// /* EXPORTED FUNCTIONS ******************************************************/
//
//
//
//
// /* INTERNAL FUNCTIONS ******************************************************/
//
//
// NTSTATUS
// FltpRegisterContexts( PFLT_FILTER Filter,
//                       const FLT_CONTEXT_REGISTRATION *Context)
// {
//     PCFLT_CONTEXT_REGISTRATION ContextPtr;
//     PALLOCATE_CONTEXT_HEADER ContextHeader, Prev;
//     PVOID Buffer;
//     ULONG BufferSize = 0;
//     USHORT i;
//     NTSTATUS Status;
//
//     /* Loop through all entries in the context registration array */
//     ContextPtr = Context;
//     while (ContextPtr)
//     {
//         /* Bail if we found the terminator */
//         if (ContextPtr->ContextType == FLT_CONTEXT_END)
//             break;
//
//         /* Make sure we have a valid context  */
//         if (IsContextTypeValid(ContextPtr->ContextType))
//         {
//             /* Each context is backed by a crtl struct. Reserve space for it */
//             BufferSize += sizeof(STREAM_LIST_CTRL);
//         }
//
//         /* Move to the next entry */
//         ContextPtr++;
//     }
//
//     /* Bail if we found no valid registration requests */
//     if (BufferSize == 0)
//     {
//         return STATUS_SUCCESS;
//     }
//
//     /* Allocate the pool that'll hold the context crtl structs */
//     Buffer = ExAllocatePoolWithTag(NonPagedPool, BufferSize, FM_TAG_CONTEXT_REGISTA);
//     if (!Buffer) return STATUS_INSUFFICIENT_RESOURCES;
//
//     RtlZeroMemory(Buffer, BufferSize);
//
//     /* Setup our loop data */
//     ContextHeader = Buffer;
//     Prev = NULL;
//     Status = STATUS_SUCCESS;
//
//     for (i = 0; i < MAX_CONTEXT_TYPES; i++)
//     {
//         ContextPtr = (PFLT_CONTEXT_REGISTRATION)Context;
//         while (ContextPtr)
//         {
//             /* We don't support variable sized contents yet */
//             FLT_ASSERT(ContextPtr->Size != FLT_VARIABLE_SIZED_CONTEXTS);
//
//             /* Bail if we found the terminator */
//             if (ContextPtr->ContextType == FLT_CONTEXT_END)
//                 break;
//
//             /* Size and pooltag are only checked when ContextAllocateCallback is null */
//             if (ContextPtr->ContextAllocateCallback == FALSE && ContextPtr->PoolTag == FALSE)
//             {
//                 Status = STATUS_FLT_INVALID_CONTEXT_REGISTRATION;
//                 goto Quit;
//             }
//
//             /* Make sure we have a valid context  */
//             if (IsContextTypeValid(ContextPtr->ContextType))
//             {
//                 Status = SetupContextHeader(Filter, ContextPtr, ContextHeader);
//                 if (NT_SUCCESS(Status))
//                 {
//                     if (Prev)
//                     {
//                         Prev->Next = ContextHeader;
//                     }
//
//                     Filter->SupportedContexts[i] = ContextHeader;
//                 }
//             }
//
//             Prev = ContextHeader;
//
//             /* Move to the next entry */
//             ContextPtr++;
//         }
//     }
//
// Quit:
//     if (NT_SUCCESS(Status))
//     {
//         Filter->SupportedContextsListHead = Buffer;
//     }
//     else
//     {
//         ExFreePoolWithTag(Buffer, FM_TAG_CONTEXT_REGISTA);
//         //FIXME: Cleanup anything that SetupContextHeader may have allocated
//     }
//
//     return Status;
// }
//
//
// /* PRIVATE FUNCTIONS ******************************************************/
//
// static
// BOOLEAN
// IsContextTypeValid( FLT_CONTEXT_TYPE ContextType)
// {
//     switch (ContextType)
//     {
//     case FLT_VOLUME_CONTEXT:
//     case FLT_INSTANCE_CONTEXT:
//     case FLT_FILE_CONTEXT:
//     case FLT_STREAM_CONTEXT:
//     case FLT_STREAMHANDLE_CONTEXT:
//     case FLT_TRANSACTION_CONTEXT:
//         return TRUE;
//     }
//
//     return FALSE;
// }
//
// static
// NTSTATUS
// SetupContextHeader( PFLT_FILTER Filter,
//                     PCFLT_CONTEXT_REGISTRATION ContextPtr,
//                     PALLOCATE_CONTEXT_HEADER ContextHeader)
// {
//     return 0;
// }
//
// ============================================================================
// END ORIGINAL C SOURCE
// ============================================================================
